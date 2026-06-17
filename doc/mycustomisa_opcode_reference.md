# MyCustomISA — Opcode Reference

## Addressing Modes

| Code | Syntax     | Meaning                                      | Payload |
|------|------------|----------------------------------------------|---------|
| `00` | `42`       | Immediate — 16-bit literal                   | 16 bits |
| `01` | `R3`       | Register — value in register                 | none    |
| `10` | `[R3]`     | Indirect register — memory at address in reg | none    |
| `11` | `[0xABCD]` | Indirect address — memory at literal address | 16 bits |

Modes `00` and `11` produce 32-bit instructions. Modes `01` and `10` produce 16-bit instructions.

---

## Format 0 — No operands (opcode `0x00` + sub-opcode)

| Sub-opcode | Mnemonic | Description                        |
|------------|----------|------------------------------------|
| `0x0`      | `HALT`   | Halt the CPU                       |
| `0x1`      | `RET`    | Return from subroutine             |
| `0x2`      | `RETI`   | Return from interrupt              |
| `0x3`      | `SEZ`    | Set Zero flag                      |
| `0x4`      | `CLZ`    | Clear Zero flag                    |
| `0x5`      | `SEC`    | Set Carry flag                     |
| `0x6`      | `CLC`    | Clear Carry flag                   |
| `0x7`      | `SEV`    | Set Overflow flag                  |
| `0x8`      | `CLV`    | Clear Overflow flag                |
| `0x9`      | `SEN`    | Set Negative flag                  |
| `0xA`      | `CLN`    | Clear Negative flag                |
| `0xB–0xF`  | —        | Reserved                           |

Encoding: `00000 [sub-opcode 4 bits] [padding 7 bits]`

---

## Format 1 — One operand (opcodes `0x01`–`0x12`)

| Opcode | Mnemonic | Valid Modes          | Flags    | Description                        |
|--------|----------|----------------------|----------|------------------------------------|
| `0x01` | `PUSH`   | imm, reg, [reg], [addr] | —     | Push operand onto stack            |
| `0x02` | `POP`    | reg, [reg], [addr]   | —        | Pop top of stack into operand      |
| `0x03` | `NOT`    | reg                  | Z, N     | Bitwise complement in place        |
| `0x04` | `JMP`    | imm, [reg], [addr]   | —        | Unconditional jump                 |
| `0x05` | `JE`     | imm, [reg], [addr]   | —        | Jump if Z=1                        |
| `0x06` | `JNE`    | imm, [reg], [addr]   | —        | Jump if Z=0                        |
| `0x07` | `JGT`    | imm, [reg], [addr]   | —        | Jump if signed > (Z=0 AND N=V)     |
| `0x08` | `JGE`    | imm, [reg], [addr]   | —        | Jump if signed ≥ (N=V)             |
| `0x09` | `JLT`    | imm, [reg], [addr]   | —        | Jump if signed < (N≠V)             |
| `0x0A` | `JLE`    | imm, [reg], [addr]   | —        | Jump if signed ≤ (Z=1 OR N≠V)      |
| `0x0B` | `JA`     | imm, [reg], [addr]   | —        | Jump if unsigned > (C=0 AND Z=0)   |
| `0x0C` | `JAE`    | imm, [reg], [addr]   | —        | Jump if unsigned ≥ (C=0)           |
| `0x0D` | `JB`     | imm, [reg], [addr]   | —        | Jump if unsigned < (C=1)           |
| `0x0E` | `JBE`    | imm, [reg], [addr]   | —        | Jump if unsigned ≤ (C=1 OR Z=1)    |
| `0x0F` | `CALL`   | imm, [reg], [addr]   | —        | Push PC, jump to operand           |
| `0x10` | `TRAP`   | imm                  | —        | Trigger user-defined trap          |
| `0x11` | `GETSP`  | reg                  | —        | Copy SP into register              |
| `0x12` | `SETSP`  | reg                  | —        | Copy register into SP              |

Encoding (16-bit): `[opcode 5b] [mode 2b] [reg 4b] [padding 5b]`
Encoding (32-bit): `[opcode 5b] [mode 2b] [padding 9b]` + `[16-bit immediate/address]`

---

## Format 2 — Two operands (opcodes `0x13`–`0x1F`)

| Opcode | Mnemonic | Source Modes         | Dest        | Flags     | Description                              |
|--------|----------|----------------------|-------------|-----------|------------------------------------------|
| `0x13` | `LOAD`   | imm, [reg], [addr]   | reg         | —         | Read from source into register           |
| `0x14` | `STORE`  | reg                  | [reg],[addr]| —         | Write register to memory                 |
| `0x15` | `MOVE`   | reg                  | reg         | —         | Copy register to register                |
| `0x16` | `ADD`    | reg, imm             | reg         | Z,C,V,N   | dest = dest + src                        |
| `0x17` | `SUB`    | reg, imm             | reg         | Z,C,V,N   | dest = dest − src                        |
| `0x18` | `MUL`    | reg, imm             | reg         | Z,C,V,N   | dest = dest × src                        |
| `0x19` | `DIV`    | reg, imm             | reg         | Z,N       | dest = dest ÷ src (traps on div/0)       |
| `0x1A` | `SHL`    | reg, imm             | reg         | Z,C,N     | Shift dest left by src bits              |
| `0x1B` | `SHR`    | reg, imm             | reg         | Z,C,N     | Shift dest right by src bits             |
| `0x1C` | `AND`    | reg, imm             | reg         | Z,N       | Bitwise AND                              |
| `0x1D` | `OR`     | reg, imm             | reg         | Z,N       | Bitwise OR                              |
| `0x1E` | `CMP`    | reg, imm             | reg         | Z,C,V,N   | Compute dest−src, set flags, discard     |
| `0x1F` | `MVINT`  | interrupt (3 bits)   | —           | —         | Set IVT register to handler address      |

Encoding (16-bit): `[opcode 5b] [mode 2b] [reg_s 4b] [reg_d 4b] [padding 1b]`
Encoding (32-bit): `[opcode 5b] [mode 2b] [reg 4b] [padding 5b]` + `[16-bit immediate/address]`

---

## Interrupt Numbers (for MVINT)

| Number | Name                  | Fires when...                        |
|--------|-----------------------|--------------------------------------|
| `0`    | `KEYBOARD`            | A key is pressed                     |
| `1`    | `TIMER`               | Timer countdown reaches zero         |
| `2`    | `ILLEGAL_INSTRUCTION` | Unknown opcode encountered           |
| `3`    | `DIVIDE_BY_ZERO`      | Division by zero attempted           |
| `4`    | `STACK_OVERFLOW`      | SP collides with code segment        |
| `5`    | `USER0`               | User-defined trap 0                  |
| `6`    | `USER1`               | User-defined trap 1                  |

---

*MyCustomISA — a custom 16-bit ISA for FPGA and virtual machine implementation.*
