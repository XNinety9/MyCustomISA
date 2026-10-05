; ════════════════════════════════════════════════════════════════════════
;  snake.asm  —  Snake for MyCustomISA
; ════════════════════════════════════════════════════════════════════════
;
;  The playfield is a 32×32 grid of 4×4-pixel cells on the 128×128 display.
;  The outer ring of cells is a wall, drawn as a 1-pixel frame.
;
;  Two modes:
;    • DEMO   — a greedy autopilot steers the snake toward the food. Runs
;               on its own after boot and restarts itself on game over.
;    • PLAYER — any direction key starts a game. Arrows, WASD or ZQSD.
;               After game over, a key restarts; otherwise back to DEMO.
;
;  Concepts shown off:
;    • timer interrupt as the game clock (the ISR only raises a flag)
;    • keyboard interrupt feeding a 2-slot direction queue
;    • ring buffer of body cells + occupancy grid for O(1) collisions
;    • word-wide framebuffer read-modify-write (one cell row = one word)
;    • 16-bit LCG for food placement, seeded by the idle-loop spin count
;
;  Every game variable lives at a fixed address so a host (the VM UI, the
;  web page) can read the score and state straight out of RAM.
;
;  ── RAM layout ──────────────────────────────────────────────────────────
;    0x8000  TICK       set to 1 by the timer ISR, cleared by main loop
;    0x8002  DIR        current direction  0 up · 1 down · 2 left · 3 right
;    0x8004  KQ1        key queue slot 1   (0xFF = empty)
;    0x8006  KQ2        key queue slot 2   (0xFF = empty)
;    0x8008  MODE       0 demo · 1 player
;    0x800A  SCORE
;    0x800C  HISCORE    best player score since boot
;    0x800E  SEED       LCG state
;    0x8010  HEADPTR    byte offset of the head in BODY (0..0x7FE)
;    0x8012  TAILPTR    byte offset of the tail in BODY
;    0x8014  HEAD       head cell index (y*32 + x)
;    0x8016  FOOD       food cell index
;    0x8018  STATE      0 running · 1 dying (blinking) · 2 waiting for key
;    0x801A  COUNTER    ticks left in the dying / waiting state
;    0x801C  PERIOD     current timer reload value (smaller = faster)
;    0x801E  STARTREQ   set by the keyboard ISR to start a player game
;    0x8020  LENGTH     snake length in cells
;    0x8022  STARTDIR   direction key that started the game
;    0x8024  NEWHEAD    scratch: cell the head is moving into
;    0x8026  GROW       scratch: 1 if this move eats the food
;    0x8100  BODY       ring buffer, 1024 words of cell indices
;    0x9000  OCC        occupancy grid, 1024 words: 0 free · 1 snake · 2 wall
;
;  ── MMIO used ──────────────────────────────────────────────────────────
;    0xF500  framebuffer   0xFD00 KB_STATUS   0xFD02 KB_DATA
;    0xFD14  DISPLAY_CTRL  0xFD15 TIMER (16-bit period)   0xFD17 TIMER_CTRL
; ════════════════════════════════════════════════════════════════════════

start:
    MVINT 0, kb_isr
    MVINT 1, timer_isr

    LOAD 0x1234, R0
    STORE R0, [0x800E]      ; SEED
    LOAD 0, R0
    STORE R0, [0x8000]      ; TICK
    STORE R0, [0x8008]      ; MODE = demo
    STORE R0, [0x800C]      ; HISCORE
    STORE R0, [0x801E]      ; STARTREQ
    CALL new_game

    LOAD 3, R0              ; ENABLE | LOOP
    STORE R0, [0xFD17]

; ── Main loop: spin (stirring the seed) until the timer raises TICK ──────
main_loop:
    LOAD [0x800E], R0
    ADD 1, R0
    STORE R0, [0x800E]
    LOAD [0x8000], R0
    CMP 0, R0
    JE main_loop
    LOAD 0, R0
    STORE R0, [0x8000]
    CALL on_tick
    JMP main_loop

; ════════════════════════════════════════════════════════════════════════
;  Interrupt handlers — kept tiny, the CPU saves/restores all registers
; ════════════════════════════════════════════════════════════════════════
timer_isr:
    LOAD 1, R0
    STORE R0, [0x8000]
    RETI

kb_isr:
kb_next:
    LOAD [0xFD00], R0       ; keys waiting?
    CMP 0, R0
    JE kb_done
    LOAD [0xFD02], R1       ; pop keycode
    CALL key_to_dir         ; R1 -> R2
    CMP 0xFF, R2
    JE kb_next
    ; in demo, or waiting after a game over: request a new player game
    LOAD [0x8008], R3
    CMP 0, R3
    JE kb_start
    LOAD [0x8018], R3
    CMP 2, R3
    JE kb_start
    ; otherwise queue the direction (second slot if the first is taken)
    LOAD [0x8004], R3
    CMP 0xFF, R3
    JNE kb_slot2
    STORE R2, [0x8004]
    JMP kb_next
kb_slot2:
    STORE R2, [0x8006]
    JMP kb_next
kb_start:
    STORE R2, [0x8022]      ; STARTDIR
    LOAD 1, R3
    STORE R3, [0x801E]      ; STARTREQ
    JMP kb_next
kb_done:
    RETI

; key_to_dir — R1 = keycode  ->  R2 = direction, or 0xFF if not a direction
key_to_dir:
    LOAD 0, R2
    CMP 0x90, R1            ; UP
    JE ktd_ret
    CMP 0x77, R1            ; w
    JE ktd_ret
    CMP 0x7A, R1            ; z
    JE ktd_ret
    LOAD 1, R2
    CMP 0x91, R1            ; DOWN
    JE ktd_ret
    CMP 0x73, R1            ; s
    JE ktd_ret
    LOAD 2, R2
    CMP 0x92, R1            ; LEFT
    JE ktd_ret
    CMP 0x61, R1            ; a
    JE ktd_ret
    CMP 0x71, R1            ; q
    JE ktd_ret
    LOAD 3, R2
    CMP 0x93, R1            ; RIGHT
    JE ktd_ret
    CMP 0x64, R1            ; d
    JE ktd_ret
    LOAD 0xFF, R2
ktd_ret:
    RET

; ════════════════════════════════════════════════════════════════════════
;  on_tick — one game-clock tick, dispatched on STATE
; ════════════════════════════════════════════════════════════════════════
on_tick:
    LOAD [0x801E], R0
    CMP 0, R0
    JE ot_state
    LOAD 0, R0
    STORE R0, [0x801E]
    LOAD 1, R0
    STORE R0, [0x8008]      ; MODE = player
    CALL new_game
    LOAD [0x8022], R0
    STORE R0, [0x8004]      ; replay the key that started the game
    RET
ot_state:
    LOAD [0x8018], R0
    CMP 0, R0
    JE game_step            ; tail calls: their RET returns to main_loop
    CMP 1, R0
    JE dying_step
    JMP waiting_step

; dying_step — blink the screen: invert it every 4th tick, 4 times in all
;   (the clock is back to its slowest rate, so that's ~1 flash per second)
dying_step:
    LOAD [0x801A], R0
    SUB 1, R0
    STORE R0, [0x801A]
    AND 3, R0
    JNE ds_ret
    CALL invert_screen
    LOAD [0x801A], R0
    CMP 0, R0
    JNE ds_ret
    LOAD [0x8008], R0
    CMP 0, R0
    JE new_game             ; demo restarts right away
    LOAD 2, R0
    STORE R0, [0x8018]      ; STATE = waiting
    LOAD 60, R0
    STORE R0, [0x801A]      ; ~6 s to press a key
ds_ret:
    RET

; waiting_step — no key for a while: hand the screen back to the demo
waiting_step:
    LOAD [0x801A], R0
    SUB 1, R0
    STORE R0, [0x801A]
    JNE ws_ret
    LOAD 0, R0
    STORE R0, [0x8008]      ; MODE = demo
    JMP new_game
ws_ret:
    RET

; ════════════════════════════════════════════════════════════════════════
;  game_step — choose a direction, move, eat or die
; ════════════════════════════════════════════════════════════════════════
game_step:
    LOAD [0x8008], R0
    CMP 0, R0
    JNE gs_player
    CALL ai_choose
    JMP gs_move
gs_player:
    LOAD [0x8004], R1       ; pop KQ1, shift KQ2 into it
    CMP 0xFF, R1
    JE gs_move
    LOAD [0x8006], R2
    STORE R2, [0x8004]
    LOAD 0xFF, R2
    STORE R2, [0x8006]
    LOAD [0x8002], R2
    CALL opposite           ; R3 = reverse of current direction
    CMP R3, R1
    JE gs_move              ; can't turn back onto yourself
    STORE R1, [0x8002]
gs_move:
    LOAD [0x8002], R2
    CALL dir_delta          ; R3 = cell delta
    LOAD [0x8014], R4
    ADD R3, R4              ; R4 = new head cell
    STORE R4, [0x8024]
    LOAD 0, R13
    LOAD [0x8016], R5
    CMP R5, R4
    JNE gs_no_eat
    LOAD 1, R13
    STORE R13, [0x8026]     ; GROW = 1, keep the tail
    JMP gs_check
gs_no_eat:
    STORE R13, [0x8026]     ; GROW = 0
    ; drop the tail first, so the head may move into the cell it frees
    LOAD [0x8012], R0
    LOAD 0x8100, R1
    ADD R0, R1
    LOAD [R1], R4           ; R4 = tail cell
    MOVE R4, R2
    SHL 1, R2
    ADD 0x9000, R2
    LOAD 0, R3
    STORE R3, [R2]          ; OCC[tail] = free
    ADD 2, R0
    AND 0x07FF, R0
    STORE R0, [0x8012]
    LOAD 0x8100, R1
    ADD R0, R1
    LOAD [R1], R5           ; R5 = new tail cell
    MOVE R4, R6
    LOAD 0, R7
    CALL draw_link          ; erase the joint to the new tail
    MOVE R4, R6
    LOAD 0, R7
    CALL draw_cell          ; erase the old tail
    LOAD [0x8020], R0
    SUB 1, R0
    STORE R0, [0x8020]
gs_check:
    LOAD [0x8024], R4
    MOVE R4, R2
    SHL 1, R2
    ADD 0x9000, R2
    LOAD [R2], R3
    CMP 0, R3
    JNE gs_die              ; wall or body
    LOAD 1, R3
    STORE R3, [R2]          ; OCC[head] = snake
    LOAD [0x8010], R0
    ADD 2, R0
    AND 0x07FF, R0
    STORE R0, [0x8010]
    LOAD 0x8100, R1
    ADD R0, R1
    STORE R4, [R1]          ; BODY[head] = cell
    LOAD [0x8014], R5       ; R5 = previous head
    STORE R4, [0x8014]
    MOVE R4, R6
    LOAD 1, R7
    CALL draw_link
    LOAD [0x8014], R4
    LOAD [0x8020], R0
    ADD 1, R0
    STORE R0, [0x8020]
    MOVE R4, R6
    LOAD 1, R7
    CALL draw_cell
    LOAD [0x8026], R0
    CMP 0, R0
    JE gs_refresh
    ; ── ate the food ──
    LOAD [0x800A], R0
    ADD 1, R0
    STORE R0, [0x800A]
    LOAD [0x8008], R1
    CMP 0, R1
    JE gs_speed             ; demo scores don't count
    LOAD [0x800C], R1
    CMP R1, R0              ; SCORE - HISCORE
    JBE gs_speed
    STORE R0, [0x800C]
gs_speed:
    LOAD [0x801C], R0
    CMP 0x7800, R0          ; floor at 0x7800
    JBE gs_food
    SUB 0x0600, R0
    STORE R0, [0x801C]
    STORE R0, [0xFD15]
gs_food:
    CALL spawn_food
gs_refresh:
    LOAD 2, R0              ; DISPLAY_CTRL.REFRESH
    STORE R0, [0xFD14]
    RET
gs_die:
    LOAD 1, R0
    STORE R0, [0x8018]      ; STATE = dying
    LOAD 16, R0
    STORE R0, [0x801A]      ; 16 ticks, inverting on every 4th
    LOAD 0xFFFF, R0
    STORE R0, [0xFD15]      ; slow the clock right down for the blink
    RET

; ════════════════════════════════════════════════════════════════════════
;  ai_choose — demo autopilot. Greedy: head toward the food, never turn
;  back, avoid occupied cells. A first "strict" pass also refuses cells
;  with no free neighbour (dead ends); a second pass relaxes that.
;  Writes the chosen direction to DIR.
; ════════════════════════════════════════════════════════════════════════
ai_choose:
    LOAD [0x8014], R4
    LOAD [0x8016], R5
    MOVE R4, R6
    AND 31, R6              ; R6  = head x
    MOVE R5, R7
    AND 31, R7              ; R7  = food x
    MOVE R4, R10
    SHR 5, R10              ; R10 = head y
    MOVE R5, R11
    SHR 5, R11              ; R11 = food y
    LOAD 1, R12             ; strict pass
    CMP R6, R7
    JBE ai_left
    LOAD 3, R1              ; food is to the right
    CALL try_dir
    CMP 1, R0
    JE ai_ret
ai_left:
    CMP R6, R7
    JAE ai_down
    LOAD 2, R1              ; food is to the left
    CALL try_dir
    CMP 1, R0
    JE ai_ret
ai_down:
    CMP R10, R11
    JBE ai_up
    LOAD 1, R1              ; food is below
    CALL try_dir
    CMP 1, R0
    JE ai_ret
ai_up:
    CMP R10, R11
    JAE ai_any
    LOAD 0, R1              ; food is above
    CALL try_dir
    CMP 1, R0
    JE ai_ret
ai_any:
    LOAD [0x8002], R1       ; keep going straight if possible
    CALL try_dir
    CMP 1, R0
    JE ai_ret
    LOAD 0, R13
ai_loop:
    MOVE R13, R1
    CALL try_dir
    CMP 1, R0
    JE ai_ret
    ADD 1, R13
    CMP 4, R13
    JNE ai_loop
    CMP 0, R12
    JE ai_ret               ; trapped: keep DIR and crash
    LOAD 0, R12             ; relax and retry
    JMP ai_any
ai_ret:
    RET

; try_dir — R1 = candidate direction, R12 = strict flag
;   -> R0 = 1 and DIR = R1 if the move is safe, else R0 = 0
;   clobbers R0, R2, R3, R8, R9
try_dir:
    LOAD 0, R0
    LOAD [0x8002], R2
    CALL opposite
    CMP R3, R1
    JE td_ret
    MOVE R1, R2
    CALL dir_delta
    LOAD [0x8014], R8
    ADD R3, R8
    SHL 1, R8
    ADD 0x9000, R8          ; R8 = &OCC[next]
    LOAD [R8], R9
    CMP 0, R9
    JNE td_ret
    CMP 0, R12
    JE td_accept
    MOVE R8, R9             ; strict: need one free neighbour of next
    SUB 64, R9
    LOAD [R9], R9
    CMP 0, R9
    JE td_accept
    MOVE R8, R9
    ADD 64, R9
    LOAD [R9], R9
    CMP 0, R9
    JE td_accept
    MOVE R8, R9
    SUB 2, R9
    LOAD [R9], R9
    CMP 0, R9
    JE td_accept
    MOVE R8, R9
    ADD 2, R9
    LOAD [R9], R9
    CMP 0, R9
    JE td_accept
    JMP td_ret
td_accept:
    STORE R1, [0x8002]
    LOAD 1, R0
td_ret:
    RET

; opposite — R2 = direction -> R3 = reverse direction (d XOR 1)
opposite:
    MOVE R2, R3
    AND 1, R3
    JE opp_even
    MOVE R2, R3
    SUB 1, R3
    RET
opp_even:
    MOVE R2, R3
    ADD 1, R3
    RET

; dir_delta — R2 = direction -> R3 = cell index delta (-32, +32, -1, +1)
dir_delta:
    LOAD 0xFFE0, R3
    CMP 0, R2
    JE dd_ret
    LOAD 32, R3
    CMP 1, R2
    JE dd_ret
    LOAD 0xFFFF, R3
    CMP 2, R2
    JE dd_ret
    LOAD 1, R3
dd_ret:
    RET

; ════════════════════════════════════════════════════════════════════════
;  new_game — clear everything, lay out a 3-cell snake, drop some food
; ════════════════════════════════════════════════════════════════════════
new_game:
    CALL clear_screen
    CALL draw_frame
    CALL init_occ
    LOAD 0, R0
    STORE R0, [0x800A]      ; SCORE
    STORE R0, [0x8018]      ; STATE = running
    STORE R0, [0x8012]      ; TAILPTR
    LOAD 0xFF, R0
    STORE R0, [0x8004]      ; empty key queue
    STORE R0, [0x8006]
    LOAD 3, R0
    STORE R0, [0x8002]      ; DIR = right
    LOAD 0xFFFF, R0
    STORE R0, [0x801C]      ; slowest speed
    STORE R0, [0xFD15]
    LOAD 520, R4            ; cell (8, 16)
    LOAD 0, R5
ng_body:
    LOAD 0x8100, R1
    ADD R5, R1
    STORE R4, [R1]
    MOVE R4, R2
    SHL 1, R2
    ADD 0x9000, R2
    LOAD 1, R3
    STORE R3, [R2]
    MOVE R4, R6
    LOAD 1, R7
    CALL draw_cell          ; leaves R4, R5 alone
    STORE R4, [0x8014]
    STORE R5, [0x8010]
    ADD 1, R4
    ADD 2, R5
    CMP 6, R5
    JNE ng_body
    LOAD 520, R6
    LOAD 521, R5
    LOAD 1, R7
    CALL draw_link
    LOAD 521, R6
    LOAD 522, R5
    LOAD 1, R7
    CALL draw_link
    LOAD 3, R0
    STORE R0, [0x8020]
    CALL spawn_food
    LOAD 2, R0
    STORE R0, [0xFD14]
    RET

; init_occ — OCC = 0 everywhere, 2 on the border ring
init_occ:
    LOAD 0, R0
io_loop:
    LOAD 0, R3
    MOVE R0, R1
    AND 31, R1
    CMP 0, R1
    JE io_wall
    CMP 31, R1
    JE io_wall
    MOVE R0, R1
    SHR 5, R1
    CMP 0, R1
    JE io_wall
    CMP 31, R1
    JE io_wall
    JMP io_store
io_wall:
    LOAD 2, R3
io_store:
    MOVE R0, R2
    SHL 1, R2
    ADD 0x9000, R2
    STORE R3, [R2]
    ADD 1, R0
    CMP 1024, R0
    JNE io_loop
    RET

; spawn_food — random free cell (walls are occupied, so x,y in 0..31 is fine)
spawn_food:
    CALL rand
    MOVE R0, R1
    SHR 8, R1
    AND 31, R1              ; x
    CALL rand
    SHR 8, R0
    AND 31, R0
    SHL 5, R0
    ADD R1, R0              ; cell = y*32 + x
    MOVE R0, R2
    SHL 1, R2
    ADD 0x9000, R2
    LOAD [R2], R3
    CMP 0, R3
    JNE spawn_food
    STORE R0, [0x8016]
    MOVE R0, R6
    LOAD 2, R7
    CALL draw_cell
    RET

; rand — SEED = SEED * 25173 + 13849  ->  R0
rand:
    LOAD [0x800E], R0
    MUL 25173, R0
    ADD 13849, R0
    STORE R0, [0x800E]
    RET

; ════════════════════════════════════════════════════════════════════════
;  Drawing
;  Framebuffer: 1 bit per pixel, MSB first, 16 bytes per row. Memory is
;  accessed in 16-bit words, so the word at 0xF500 + y*16 + (x/16)*2 holds
;  pixels x&~15 .. x|15, with pixel x at bit 15 - (x & 15).
; ════════════════════════════════════════════════════════════════════════

; draw_cell — R6 = cell, R7 = 0 erase · 1 solid 3×3 block · 2 food diamond
;   A cell's 3 pixel columns never straddle a word, so each of its 3 rows
;   is a single word read-modify-write. Clobbers R6..R12.
draw_cell:
    CALL cell_addr
    LOAD 0xE000, R10        ; outer rows mask
    LOAD 0xE000, R11        ; middle row mask
    CMP 2, R7
    JNE dc_shift
    LOAD 0x4000, R10        ; .#.
    LOAD 0xA000, R11        ; #.#
dc_shift:
    SHR R8, R10
    SHR R8, R11
    CMP 0, R7
    JNE dc_set
    NOT R10
    NOT R11
    LOAD [R9], R12
    AND R10, R12
    STORE R12, [R9]
    ADD 16, R9
    LOAD [R9], R12
    AND R11, R12
    STORE R12, [R9]
    ADD 16, R9
    LOAD [R9], R12
    AND R10, R12
    STORE R12, [R9]
    RET
dc_set:
    LOAD [R9], R12
    OR R10, R12
    STORE R12, [R9]
    ADD 16, R9
    LOAD [R9], R12
    OR R11, R12
    STORE R12, [R9]
    ADD 16, R9
    LOAD [R9], R12
    OR R10, R12
    STORE R12, [R9]
    RET

; cell_addr — R6 = cell -> R9 = address of the word holding the cell's
;   top pixel row, R8 = bit offset of the cell's first pixel in that word.
;   Clobbers R10.
cell_addr:
    MOVE R6, R8
    AND 31, R8
    SHL 2, R8               ; R8 = pixel x
    MOVE R6, R9
    SHR 5, R9
    SHL 6, R9               ; R9 = pixel y * 16  (= cell y * 4 * 16)
    MOVE R8, R10
    SHR 4, R10
    SHL 1, R10
    ADD R10, R9
    ADD 0xF500, R9
    AND 15, R8
    RET

; draw_link — R6, R5 = two adjacent cells, R7 = 0 erase · 1 draw
;   Fills the 1-pixel gap between them so the body reads as one band:
;   a 3-pixel column right of the left cell, or a 3-pixel row under the
;   upper cell. Clobbers R5..R12.
draw_link:
    MOVE R6, R10
    SUB R5, R10             ; R10 = a - b
    CMP R5, R6
    JB dl_min
    MOVE R5, R6             ; R6 = min(a, b)
dl_min:
    MOVE R10, R11
    CALL cell_addr          ; keeps R11
    CMP 1, R11
    JE dl_h
    CMP 0xFFFF, R11
    JE dl_h
    LOAD 0xE000, R11        ; vertical neighbours: row 3 of the upper cell
    LOAD 1, R5
    ADD 48, R9
    JMP dl_go
dl_h:
    LOAD 0x1000, R11        ; horizontal neighbours: column 3, rows 0..2
    LOAD 3, R5
dl_go:
    SHR R8, R11
    CMP 0, R7
    JNE dl_loop
    NOT R11
dl_loop:
    LOAD [R9], R12
    CMP 0, R7
    JE dl_and
    OR R11, R12
    JMP dl_store
dl_and:
    AND R11, R12
dl_store:
    STORE R12, [R9]
    ADD 16, R9
    SUB 1, R5
    JNE dl_loop
    RET

; set_pixel — R6 = x, R7 = y. Clobbers R9..R12.
set_pixel:
    MOVE R7, R9
    SHL 4, R9
    MOVE R6, R10
    SHR 4, R10
    SHL 1, R10
    ADD R10, R9
    ADD 0xF500, R9
    MOVE R6, R10
    AND 15, R10
    LOAD 0x8000, R11
    SHR R10, R11
    LOAD [R9], R12
    OR R11, R12
    STORE R12, [R9]
    RET

; draw_frame — 1-pixel border at x/y = 2 and 124, one pixel off every cell
draw_frame:
    LOAD 2, R0
df_loop:
    MOVE R0, R6
    LOAD 2, R7
    CALL set_pixel
    MOVE R0, R6
    LOAD 124, R7
    CALL set_pixel
    LOAD 2, R6
    MOVE R0, R7
    CALL set_pixel
    LOAD 124, R6
    MOVE R0, R7
    CALL set_pixel
    ADD 1, R0
    CMP 125, R0
    JNE df_loop
    RET

clear_screen:
    LOAD 0xF500, R0
    LOAD 0, R1
cs_loop:
    STORE R1, [R0]
    ADD 2, R0
    CMP 0xFD00, R0
    JNE cs_loop
    RET

invert_screen:
    LOAD 0xF500, R0
is_loop:
    LOAD [R0], R1
    NOT R1
    STORE R1, [R0]
    ADD 2, R0
    CMP 0xFD00, R0
    JNE is_loop
    LOAD 2, R0
    STORE R0, [0xFD14]
    RET
