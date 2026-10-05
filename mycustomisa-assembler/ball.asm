; ════════════════════════════════════════════════════════════════════════
;  bounce.asm  —  a bouncing ball for MyCustomISA
; ════════════════════════════════════════════════════════════════════════
;
;  A single white pixel ricochets around the 128×128 display, bouncing off
;  the four walls. The motion is driven by the hardware timer firing an
;  interrupt at a fixed tick rate, so the ball moves at a steady speed
;  regardless of how fast the CPU runs. Press ESC (0x1B) to halt.
;
;  Concepts shown off:
;    • timer interrupt as a game-loop clock (IVT_TIMER + LOOP mode)
;    • keyboard interrupt for quit-on-ESC
;    • bit-addressed framebuffer plotting (the fiddly part)
;    • DISPLAY_CTRL CLEAR + REFRESH handshake
;    • signed velocity stored as +1 / -1 (0xFFFF) and reflected on bounce
;
;  Register conventions used throughout the main loop / ISR:
;    R0  ball X  (0..127)
;    R1  ball Y  (0..127)
;    R2  X velocity  (+1 or -1)
;    R3  Y velocity  (+1 or -1)
;    R4..R9  scratch
; ════════════════════════════════════════════════════════════════════════

; ── MMIO / IVT constants (from the memory map) ──────────────────────────
; FB_BASE        = 0xF500   framebuffer start
; DISPLAY_CTRL   = 0xFD14   bit0 CLEAR, bit1 REFRESH
; KB_STATUS      = 0xFD00   number of keys waiting
; KB_DATA        = 0xFD02   pop next keycode
; TIMER_LO/HI    = 0xFD15 / 0xFD16
; TIMER_CTRL     = 0xFD17   bit0 ENABLE, bit1 LOOP

; ════════════════════════════════════════════════════════════════════════
;  Entry point  (PC starts at 0x0000)
; ════════════════════════════════════════════════════════════════════════
start:
    ; --- initial ball state: middle-ish, moving down-right ---
    LOAD 30, R0            ; X = 30
    LOAD 20, R1            ; Y = 20
    LOAD 1,  R2            ; vx = +1
    LOAD 1,  R3            ; vy = +1

    ; --- install interrupt handlers ---
    MVINT 1, timer_isr     ; IVT_TIMER   (interrupt #1) -> timer_isr
    MVINT 0, kb_isr        ; IVT_KEYBOARD(interrupt #0) -> kb_isr

    ; --- clear the screen once before we start ---
    LOAD 1, R4             ; bit0 = CLEAR
    STORE R4, [0xFD14]

    ; --- program the timer: count down, then loop ---
    ; A small reload value = fast ticks. 0x0200 is a pleasant pace in the VM.
    LOAD 0x00, R4
    STORE R4, [0xFD15]     ; TIMER_LO = 0x00
    LOAD 0x02, R4
    STORE R4, [0xFD16]     ; TIMER_HI = 0x02  -> reload = 0x0200
    LOAD 3, R4             ; bit0 ENABLE | bit1 LOOP
    STORE R4, [0xFD17]     ; start the timer

    ; --- main loop does nothing but wait for interrupts ---
    ; All the action happens in timer_isr. We just spin here; the timer ISR
    ; redraws each frame, the keyboard ISR halts on ESC.
idle:
    JMP idle

; ════════════════════════════════════════════════════════════════════════
;  timer_isr  —  one animation frame
;    erase old pixel, advance position, bounce off walls, draw new pixel,
;    refresh the display.
; ════════════════════════════════════════════════════════════════════════
timer_isr:
    ; --- erase the ball at its current (X,Y) by clearing its bit ---
    MOVE R0, R6           ; plot wants X in R6
    MOVE R1, R7           ;            Y in R7
    LOAD 0, R8            ; R8 = 0  -> clear the pixel
    CALL plot

    ; --- advance X = X + vx ---
    ADD R2, R0            ; R0 = R0 + vx

    ; --- bounce on X walls: if X == 0 or X == 127, flip vx ---
    CMP 0, R0
    JE  flip_x
    CMP 127, R0
    JE  flip_x
    JMP x_done
flip_x:
    ; vx = 0 - vx   (negate by subtracting from zero)
    LOAD 0, R4
    SUB R2, R4           ; R4 = 0 - vx
    MOVE R4, R2          ; vx = -vx
x_done:

    ; --- advance Y = Y + vy ---
    ADD R3, R1

    ; --- bounce on Y walls: if Y == 0 or Y == 127, flip vy ---
    CMP 0, R1
    JE  flip_y
    CMP 127, R1
    JE  flip_y
    JMP y_done
flip_y:
    LOAD 0, R4
    SUB R3, R4
    MOVE R4, R3          ; vy = -vy
y_done:

    ; --- draw the ball at the new (X,Y) ---
    MOVE R0, R6
    MOVE R1, R7
    LOAD 1, R8           ; R8 = 1 -> set the pixel
    CALL plot

    ; --- push the framebuffer to the display ---
    LOAD 2, R4           ; bit1 = REFRESH
    STORE R4, [0xFD14]

    RETI

; ════════════════════════════════════════════════════════════════════════
;  kb_isr  —  pop the key; if it's ESC (0x1B), halt. Otherwise ignore.
; ════════════════════════════════════════════════════════════════════════
kb_isr:
    LOAD [0xFD02], R5     ; pop keycode
    CMP  0x1B, R5         ; ESC?
    JE   quit
    RETI
quit:
    HALT

; ════════════════════════════════════════════════════════════════════════
;  plot  —  set or clear one framebuffer pixel
;    inputs:  R6 = X (0..127), R7 = Y (0..127), R8 = 1 set / 0 clear
;    clobbers: R4, R5, R9  (callee-saved discipline isn't needed here —
;              the ISR reloads everything it cares about each frame)
;
;  Framebuffer layout: 1 bit per pixel, row-major, 128 px wide.
;    pixel index   = Y*128 + X
;    byte address  = FB_BASE + index/8
;    bit in byte   = 7 - (index % 8)        (MSB-first, matches the VM)
; ════════════════════════════════════════════════════════════════════════
plot:
    ; --- compute linear pixel index = Y*128 + X ---
    MOVE R7, R9          ; R9 = Y
    MUL  128, R9         ; R9 = Y*128
    ADD  R6, R9          ; R9 = Y*128 + X   = pixel index

    ; --- byte offset = index / 8, kept in R4 ---
    MOVE R9, R4
    SHR  3, R4           ; R4 = index >> 3  = byte offset within framebuffer

    ; --- bit position within the byte = 7 - (index & 7) ---
    MOVE R9, R5
    AND  7, R5           ; R5 = index & 7
    LOAD 7, R9
    SUB  R5, R9          ; R9 = 7 - (index & 7)   = shift amount

    ; --- build the mask = 1 << R9, in R5 ---
    LOAD 1, R5
    SHL  R9, R5          ; R5 = bit mask for this pixel

    ; --- absolute byte address = 0xF500 + byte offset, in R4 ---
    LOAD 0xF500, R9
    ADD  R9, R4          ; R4 = framebuffer byte address

    ; --- read-modify-write the byte ---
    ; We can't LOAD/STORE through a register-held address with arithmetic in
    ; one step, so use indirect-register mode: [R4].
    LOAD [R4], R9        ; R9 = current byte

    ; decide set vs clear based on R8
    CMP 0, R8
    JE  plot_clear

    ; set:  byte = byte OR mask
    OR  R5, R9
    JMP plot_write

plot_clear:
    ; clear: byte = byte AND (NOT mask)
    NOT R5               ; R5 = ~mask
    AND R5, R9           ; byte = byte & ~mask

plot_write:
    STORE R9, [R4]       ; write the byte back
    RET
