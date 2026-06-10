; ─────────────────────────────────────────────────────────────────────
; draw_square.asm
;
; Draws a filled 32×32 square at pixel position (32, 32).
;
; Framebuffer: 0xF500, 16 bytes (8 words) per row.
; The square covers columns 32–63, which map to byte offsets 4–7
; within each row — exactly two 16-bit words, both 0xFFFF.
;
; Register allocation:
;   R0  — write pointer (current word address in framebuffer)
;   R1  — row counter (counts down from 32 to 0)
;   R2  — fill value (0xFFFF)
; ─────────────────────────────────────────────────────────────────────

    ; R0 = address of first word to write
    ;    = 0xF500 + (32 rows × 16 bytes/row) + 4 bytes
    ;    = 0xF500 + 0x200 + 0x4
    ;    = 0xF704
    LOAD 0xF704, R0

    ; R1 = number of rows to fill
    LOAD 32, R1

    ; R2 = all-ones fill value
    LOAD 0xFFFF, R2

row_loop:
    ; Write word covering columns 32–47 (byte offset +4 in this row)
    STORE R2, [R0]

    ; Advance two bytes to the next word
    ADD 2, R0

    ; Write word covering columns 48–63 (byte offset +6 in this row)
    STORE R2, [R0]

    ; Advance 14 bytes to reach byte offset +4 of the NEXT row.
    ; We're currently at offset +6, and the next row's offset +4
    ; is 16 bytes ahead of this row's offset +4, so: 16 - 2 = 14.
    ADD 14, R0

    ; Decrement row counter; sets Z flag when it hits 0
    SUB 1, R1

    ; Loop back if rows remain (Z = 0)
    JNE row_loop

    ; ── Trigger a display refresh ──────────────────────────────────
    ; DISPLAY_CTRL is at 0xFD14.
    ; Bit 1 = REFRESH: push framebuffer contents to the screen.
    ; Write 0b00000010 = 2.
    LOAD 2, R0
    STORE R0, [0xFD14]

    HALT