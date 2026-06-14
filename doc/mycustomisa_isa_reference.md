# MyCustomISA — Instruction Reference

A compact, per-instruction reference for MyCustomISA. For the reasoning behind these choices, see the [instruction set design document](mycustomisa_instruction_set.md).

## Conventions

- **Word size**: 16 bits. All instructions are 16 or 32 bits.
- **Operand order**: `MNEMONIC src, dest` (source first, AT&T style).
- **Byte order**: big-endian (high byte first in memory).
- **Registers**: `R0`–`R15` (16 general-purpose), plus `SP` and `PC` (special).
- **Flags**: `Z` (zero), `C` (carry), `V` (overflow), `N` (negative), `R` (reset, read-only).

## Addressing modes

| Mode | Binary | Syntax | Length impact |
|------|--------|--------|---------------|
| Immediate | `00` | `42`, `0xABCD` | +16-bit payload |
| Register | `01` | `R3` | none |
| Indirect register | `10` | `[R3]` | none |
| Indirect address | `11` | `[0xABCD]` | +16-bit payload |

If any operand uses mode `00` or `11`, the instruction is 32 bits. Otherwise 16.

## Quick reference

| Opcode | Mnemonic | Format | Length |
|--------|----------|--------|--------|
| `0x00` + sub `0x0` | `HALT` | 0 | 16-bit |
| `0x00` + sub `0x1` | `RET` | 0 | 16-bit |
| `0x00` + sub `0x2` | `RETI` | 0 | 16-bit |
| `0x00` + sub `0x3` | `SEZ` | 0 | 16-bit |
| `0x00` + sub `0x4` | `CLZ` | 0 | 16-bit |
| `0x00` + sub `0x5` | `SEC` | 0 | 16-bit |
| `0x00` + sub `0x6` | `CLC` | 0 | 16-bit |
| `0x00` + sub `0x7` | `SEV` | 0 | 16-bit |
| `0x00` + sub `0x8` | `CLV` | 0 | 16-bit |
| `0x00` + sub `0x9` | `SEN` | 0 | 16-bit |
| `0x00` + sub `0xA` | `CLN` | 0 | 16-bit |
| `0x01` | `PUSH` | 1 | 16 or 32 |
| `0x02` | `POP` | 1 | 16 or 32 |
| `0x03` | `NOT` | 1 | 16-bit |
| `0x04` | `JMP` | 1 | 16 or 32 |
| `0x05` | `JE` | 1 | 16 or 32 |
| `0x06` | `JNE` | 1 | 16 or 32 |
| `0x07` | `JGT` | 1 | 16 or 32 |
| `0x08` | `JGE` | 1 | 16 or 32 |
| `0x09` | `JLT` | 1 | 16 or 32 |
| `0x0A` | `JLE` | 1 | 16 or 32 |
| `0x0B` | `JA` | 1 | 16 or 32 |
| `0x0C` | `JAE` | 1 | 16 or 32 |
| `0x0D` | `JB` | 1 | 16 or 32 |
| `0x0E` | `JBE` | 1 | 16 or 32 |
| `0x0F` | `CALL` | 1 | 16 or 32 |
| `0x10` | `TRAP` | 1 | 32-bit |
| `0x11` | `GETSP` | 1 | 16-bit |
| `0x12` | `SETSP` | 1 | 16-bit |
| `0x13` | `LOAD` | 2 | 16 or 32 |
| `0x14` | `STORE` | 2 | 32-bit |
| `0x15` | `MOVE` | 2 | 16-bit |
| `0x16` | `ADD` | 2 | 16 or 32 |
| `0x17` | `SUB` | 2 | 16 or 32 |
| `0x18` | `MUL` | 2 | 16 or 32 |
| `0x19` | `DIV` | 2 | 16 or 32 |
| `0x1A` | `SHL` | 2 | 16 or 32 |
| `0x1B` | `SHR` | 2 | 16 or 32 |
| `0x1C` | `AND` | 2 | 16 or 32 |
| `0x1D` | `OR` | 2 | 16 or 32 |
| `0x1E` | `CMP` | 2 | 16 or 32 |
| `0x1F` | `MVINT` | 2 | 32-bit (special) |

## Encoding templates

Each instruction's encoding follows one of these templates. The bit ranges are referenced throughout the rest of the document.

### Format 0 — 16-bit, no operand

```
 bit:  15 14 13 12 11 | 10  9  8  7 |  6  5  4  3  2  1  0
       └── opcode ──┘  └── sub  ──┘  └────── padding ─────┘
            5 bits          4 bits         7 bits
```

### Format 1 — 16-bit (register / indirect register)

```
 bit:  15 14 13 12 11 | 10  9 |  8  7  6  5 |  4  3  2  1  0
       └── opcode ──┘  └ mode┘  └── reg ──┘  └── padding ──┘
            5 bits      2 b.       4 bits         5 bits
```

### Format 1 — 32-bit (immediate / indirect address)

```
 First word:
 bit:  15 14 13 12 11 | 10  9 |  8  7  6  5  4  3  2  1  0
       └── opcode ──┘  └ mode┘  └─────── padding ────────┘
            5 bits      2 b.            9 bits
 Second word:  16-bit immediate value or address
```

### Format 2 — 16-bit (source = register / indirect register, dest = register)

```
 bit:  15 14 13 12 11 | 10  9 |  8  7  6  5 |  4  3  2  1 |  0
       └── opcode ──┘  └ mode┘  └── reg_s ─┘  └── reg_d ─┘  └p┘
            5 bits      2 b.       4 bits        4 bits     1
```

### Format 2 — 32-bit (one operand needs 16-bit payload)

```
 First word:
 bit:  15 14 13 12 11 | 10  9 |  8  7  6  5 |  4  3  2  1  0
       └── opcode ──┘  └ mode┘  └─── reg ──┘  └── padding ──┘
            5 bits      2 b.       4 bits         5 bits
 Second word:  16-bit immediate value or address
```

---

# Format 0 — control & flag manipulation

All Format 0 instructions share opcode `0x00`. The sub-opcode field (bits 10–7) distinguishes them. All are 16 bits, padding is zero.

## HALT — `0x00` + sub `0x0`

**Syntax:** `HALT`

**Encoding:** `00000 0000 0000000` = `0x0000`

**Flags affected:** none

**Description:** Stops the CPU and freezes its full state. The CPU no longer fetches instructions. Used for clean program termination and by the default fault handler at `0xFF00`.

**Operation:**
```
running ← false
```

## RET — `0x00` + sub `0x1`

**Syntax:** `RET`

**Encoding:** `00000 0001 0000000` = `0x0080`

**Flags affected:** none

**Description:** Return from subroutine. Pops a 16-bit return address from the stack into PC.

**Operation:**
```
PC ← mem[SP+1 .. SP+2]      ; read 16-bit word from stack
SP ← SP + 2
```

## RETI — `0x00` + sub `0x2`

**Syntax:** `RETI`

**Encoding:** `00000 0010 0000000` = `0x0100`

**Flags affected:** all (restored from stack)

**Description:** Return from interrupt. Restores the full saved CPU context (PC, all 16 GP registers, FLAGS) from the stack, in the reverse order it was saved when the interrupt fired.

**Operation:**
```
FLAGS ← pop16()
for i in 15..0:
    R[i] ← pop16()
PC    ← pop16()
```

## Flag manipulation — sub `0x3`–`0xA`

All have the form `SEx` (set flag) or `CLx` (clear flag). Encoded as Format 0 with the corresponding sub-opcode.

| Sub-opcode | Mnemonic | Effect | Encoded value |
|------------|----------|--------|---------------|
| `0x3` | `SEZ` | Z ← 1 | `0x0180` |
| `0x4` | `CLZ` | Z ← 0 | `0x0200` |
| `0x5` | `SEC` | C ← 1 | `0x0280` |
| `0x6` | `CLC` | C ← 0 | `0x0300` |
| `0x7` | `SEV` | V ← 1 | `0x0380` |
| `0x8` | `CLV` | V ← 0 | `0x0400` |
| `0x9` | `SEN` | N ← 1 | `0x0480` |
| `0xA` | `CLN` | N ← 0 | `0x0500` |

**Flags affected:** the single flag named by the mnemonic. All others are unchanged.

**Description:** Set or clear a single user-controllable flag. The R flag is read-only and cannot be modified.

---

# Format 1 — one operand

## PUSH — `0x01`

**Syntax:**
- `PUSH Rn` (16-bit)
- `PUSH [Rn]` (16-bit)
- `PUSH 0xABCD` (32-bit, immediate value)
- `PUSH [0xABCD]` (32-bit, value at address)

**Valid modes:** immediate, register, indirect register, indirect address.

**Encoding (16-bit):**
```
00001 mm rrrr 00000
```

**Encoding (32-bit):**
```
00001 mm 000000000  |  16-bit payload
```

**Flags affected:** none

**Description:** Pushes the operand value onto the stack as a 16-bit word, then decrements SP by 2 (post-decrement convention).

**Operation:**
```
mem[SP-1 .. SP] ← value
SP ← SP - 2
```

## POP — `0x02`

**Syntax:**
- `POP Rn` (16-bit)
- `POP [Rn]` (16-bit)
- `POP [0xABCD]` (32-bit)

**Valid modes:** register, indirect register, indirect address. Immediate is **not valid** (you can't pop into a literal).

**Encoding:** same shape as PUSH, opcode `0x02`.

**Flags affected:** none

**Description:** Pops a 16-bit word from the stack into the destination. SP is incremented before the read.

**Operation:**
```
SP ← SP + 2
dest ← mem[SP-1 .. SP]
```

## NOT — `0x03`

**Syntax:** `NOT Rn`

**Valid modes:** register only.

**Encoding (16-bit):**
```
00011 01 rrrr 00000
```

**Flags affected:** Z, N

**Description:** Replaces the register's value with its bitwise complement.

**Operation:**
```
Rn ← ~Rn
Z  ← (Rn == 0)
N  ← (Rn & 0x8000) != 0
```

## Conditional & unconditional jumps — `0x04`–`0x0E`

All jumps share the same encoding shape and operand modes; they differ only by opcode and condition.

**Syntax:**
- `Jxx addr_label` (32-bit, immediate)
- `Jxx [Rn]` (16-bit)
- `Jxx [0xABCD]` (32-bit)

**Valid modes:** immediate (the value IS the target address), indirect register, indirect address.

**Encoding (16-bit, indirect register):**
```
ooooo 10 rrrr 00000
```

**Encoding (32-bit, immediate or indirect address):**
```
ooooo mm 000000000  |  16-bit target address
```

**Flags affected:** none

| Opcode | Mnemonic | Condition | Signed/Unsigned |
|--------|----------|-----------|-----------------|
| `0x04` | `JMP` | always | — |
| `0x05` | `JE` | Z = 1 | both |
| `0x06` | `JNE` | Z = 0 | both |
| `0x07` | `JGT` | Z = 0 AND N = V | signed > |
| `0x08` | `JGE` | N = V | signed ≥ |
| `0x09` | `JLT` | N ≠ V | signed < |
| `0x0A` | `JLE` | Z = 1 OR N ≠ V | signed ≤ |
| `0x0B` | `JA` | C = 0 AND Z = 0 | unsigned > |
| `0x0C` | `JAE` | C = 0 | unsigned ≥ |
| `0x0D` | `JB` | C = 1 | unsigned < |
| `0x0E` | `JBE` | C = 1 OR Z = 1 | unsigned ≤ |

**Operation:**
```
if condition_satisfied:
    PC ← target
```

## CALL — `0x0F`

**Syntax:**
- `CALL my_function` (32-bit, immediate)
- `CALL [Rn]` (16-bit)
- `CALL [0xABCD]` (32-bit)

**Valid modes:** immediate, indirect register, indirect address.

**Encoding:** same shape as JMP.

**Flags affected:** none

**Description:** Pushes the address of the next instruction onto the stack, then jumps to the target.

**Operation:**
```
push16(PC)            ; PC already points to next instruction
PC ← target
```

## TRAP — `0x10`

**Syntax:** `TRAP n` (where n is 0 or 1)

**Valid modes:** immediate only.

**Encoding (32-bit):**
```
10000 00 000000000  |  16-bit trap number
```

**Flags affected:** all flags are saved and restored automatically.

**Description:** Triggers a user-defined trap. Saves the full CPU context to the stack (as for any interrupt), then jumps to the handler in `IVT_USER_n`. Returns via `RETI`.

**Operation:**
```
push_context()
PC ← IVT_USER_n
```

> Only n = 0 and n = 1 are valid. The assembler should reject larger values.

## GETSP — `0x11`

**Syntax:** `GETSP Rn`

**Valid modes:** register only.

**Encoding (16-bit):**
```
10001 01 rrrr 00000
```

**Flags affected:** none

**Description:** Copies the current value of SP into a general-purpose register. Used to snapshot the stack pointer for context-save operations.

**Operation:**
```
Rn ← SP
```

## SETSP — `0x12`

**Syntax:** `SETSP Rn`

**Valid modes:** register only.

**Encoding (16-bit):**
```
10010 01 rrrr 00000
```

**Flags affected:** none

**Description:** Copies the value of a register into SP. Use with care — clobbering SP mid-program will corrupt the stack.

**Operation:**
```
SP ← Rn
```

---

# Format 2 — two operands

## LOAD — `0x13`

**Syntax:**
- `LOAD 42, Rn` (32-bit, immediate source)
- `LOAD [Rm], Rn` (16-bit, indirect register source)
- `LOAD [0xABCD], Rn` (32-bit, indirect address source)

**Valid source modes:** immediate, indirect register, indirect address.
**Destination:** always a register (implied by opcode).

**Encoding (16-bit, indirect register source):**
```
10011 10 ssss dddd 0
```

**Encoding (32-bit, immediate or indirect address):**
```
10011 mm dddd 00000  |  16-bit source value/address
```

**Flags affected:** none (LOAD does not set flags)

**Description:** Reads a value (literal or from memory) and writes it into a destination register.

**Operation:**
```
Rd ← source_value
```

## STORE — `0x14`

**Syntax:**
- `STORE Rn, [Rm]` (32-bit — see note)
- `STORE Rn, [0xABCD]` (32-bit)

**Valid source modes:** register only.
**Valid destination modes:** indirect register, indirect address.

> Note: STORE with `[Rm]` destination still uses the 32-bit encoding because the assembler emits a uniform STORE format. A future revision may add a 16-bit STORE variant.

**Encoding (32-bit):**
```
10100 mm ssss 00000  |  16-bit destination address (or unused for [Rm])
```

For `STORE Rs, [Rm]`, the mode field is `10` and `Rm` is encoded into the lower bits of the second word.

**Flags affected:** none

**Description:** Writes a register's value to a memory location.

**Operation:**
```
mem[dest_address] ← Rs
```

## MOVE — `0x15`

**Syntax:** `MOVE Rs, Rd`

**Valid source modes:** register only.
**Destination:** register.

**Encoding (16-bit):**
```
10101 01 ssss dddd 0
```

**Flags affected:** none

**Description:** Copies a register's value to another register.

**Operation:**
```
Rd ← Rs
```

## ADD — `0x16`

**Syntax:**
- `ADD Rs, Rd` (16-bit)
- `ADD 42, Rd` (32-bit)

**Valid source modes:** register, immediate.
**Destination:** register.

**Encoding (16-bit, register source):**
```
10110 01 ssss dddd 0
```

**Encoding (32-bit, immediate source):**
```
10110 00 dddd 00000  |  16-bit immediate value
```

**Flags affected:** Z, C, V, N

**Description:** Adds the source to the destination, storing the result in the destination.

**Operation:**
```
result ← Rd + source
Rd     ← result & 0xFFFF
Z ← (Rd == 0)
C ← (result > 0xFFFF)             ; unsigned overflow
V ← signed_overflow(Rd_prev, source, Rd)
N ← (Rd & 0x8000) != 0
```

## SUB — `0x17`

**Syntax:**
- `SUB Rs, Rd` (16-bit)
- `SUB 42, Rd` (32-bit)

**Valid source modes:** register, immediate.
**Destination:** register.

**Encoding:** same shape as ADD, opcode `0x17`.

**Flags affected:** Z, C, V, N

**Description:** Subtracts the source from the destination. Carry is set if the subtraction "borrowed" (i.e. unsigned result was negative).

**Operation:**
```
result ← Rd - source
Rd     ← result & 0xFFFF
Z ← (Rd == 0)
C ← (Rd_prev < source)            ; unsigned borrow
V ← signed_overflow(Rd_prev, -source, Rd)
N ← (Rd & 0x8000) != 0
```

## MUL — `0x18`

**Syntax:**
- `MUL Rs, Rd` (16-bit)
- `MUL 42, Rd` (32-bit)

**Valid source modes:** register, immediate.
**Destination:** register.

**Encoding:** same shape as ADD.

**Flags affected:** Z, C, V, N

**Description:** Multiplies destination by source. Only the low 16 bits of the result are kept in `Rd`. The high 16 bits are discarded (no extended-precision support in v1). C and V are set if the full result didn't fit in 16 bits.

**Operation:**
```
full ← Rd * source                ; 32-bit intermediate
Rd   ← full & 0xFFFF
Z ← (Rd == 0)
C ← (full > 0xFFFF)               ; unsigned overflow
V ← signed_overflow_mul(Rd_prev, source)
N ← (Rd & 0x8000) != 0
```

## DIV — `0x19`

**Syntax:**
- `DIV Rs, Rd` (16-bit)
- `DIV 42, Rd` (32-bit)

**Valid source modes:** register, immediate.
**Destination:** register.

**Encoding:** same shape as ADD.

**Flags affected:** Z, N

**Description:** Integer division: `Rd ← Rd / source`. If source is zero, triggers `IVT_DIVIDE_BY_ZERO` and `Rd` is left unchanged.

**Operation:**
```
if source == 0:
    raise IVT_DIVIDE_BY_ZERO
    return
Rd ← Rd / source
Z  ← (Rd == 0)
N  ← (Rd & 0x8000) != 0
```

## SHL — `0x1A`

**Syntax:**
- `SHL Rs, Rd` (16-bit)
- `SHL 4, Rd` (32-bit)

**Valid source modes:** register, immediate.
**Destination:** register.

**Encoding:** same shape as ADD.

**Flags affected:** Z, C, N

**Description:** Logical shift left by `source` bits. Bits shifted out the top are lost; the last bit shifted out is captured in C. Zeros are shifted in on the right.

**Operation:**
```
n      ← source & 0x1F           ; clamp; >16 shifts always zero out Rd
C      ← bit 16-n of Rd_prev (the last bit pushed out)
Rd     ← (Rd << n) & 0xFFFF
Z      ← (Rd == 0)
N      ← (Rd & 0x8000) != 0
```

## SHR — `0x1B`

**Syntax:**
- `SHR Rs, Rd` (16-bit)
- `SHR 4, Rd` (32-bit)

**Valid source modes:** register, immediate.
**Destination:** register.

**Encoding:** same shape as ADD.

**Flags affected:** Z, C, N

**Description:** Logical shift right by `source` bits. Bits shifted out the bottom are lost; the last bit shifted out is captured in C. Zeros are shifted in on the left (this is a **logical** shift, not arithmetic).

**Operation:**
```
n      ← source & 0x1F
C      ← bit n-1 of Rd_prev      ; the last bit pushed out
Rd     ← Rd >> n
Z      ← (Rd == 0)
N      ← (Rd & 0x8000) != 0     ; always 0 for logical SHR, but kept for consistency
```

## AND — `0x1C`

**Syntax:**
- `AND Rs, Rd` (16-bit)
- `AND 0x00FF, Rd` (32-bit, e.g. for masking low byte)

**Valid source modes:** register, immediate.
**Destination:** register.

**Encoding:** same shape as ADD.

**Flags affected:** Z, N

**Description:** Bitwise AND of source and destination.

**Operation:**
```
Rd ← Rd & source
Z  ← (Rd == 0)
N  ← (Rd & 0x8000) != 0
```

## OR — `0x1D`

**Syntax:**
- `OR Rs, Rd` (16-bit)
- `OR 0x0080, Rd` (32-bit)

**Valid source modes:** register, immediate.
**Destination:** register.

**Encoding:** same shape as ADD.

**Flags affected:** Z, N

**Description:** Bitwise OR of source and destination.

**Operation:**
```
Rd ← Rd | source
Z  ← (Rd == 0)
N  ← (Rd & 0x8000) != 0
```

## CMP — `0x1E`

**Syntax:**
- `CMP Rs, Rd` (16-bit)
- `CMP 42, Rd` (32-bit)

**Valid source modes:** register, immediate.
**Destination:** register (not modified).

**Encoding:** same shape as ADD.

**Flags affected:** Z, C, V, N

**Description:** Computes `Rd - source` and updates flags accordingly, but **discards the result**. Used in combination with conditional jumps for control flow.

**Operation:**
```
result ← Rd - source              ; result is discarded
Z ← (result == 0)
C ← (Rd < source)                 ; unsigned borrow
V ← signed_overflow(Rd, -source, result)
N ← (result & 0x8000) != 0
```

## MVINT — `0x1F`

**Syntax:** `MVINT interrupt_id, handler_address`

For example: `MVINT 0, my_keyboard_handler` sets the keyboard interrupt handler.

**Valid modes:** the first operand is a 3-bit interrupt identifier (an immediate small integer), and the second is a 16-bit immediate address.

**Encoding (32-bit, special):**
```
First word:  11111 00 0iii 00000           ; iii = interrupt ID (3 bits)
Second word: 16-bit handler address
```

The mode field is hardcoded to `00` (immediate). The "reg" field is repurposed to hold the 3-bit interrupt number (high bit reserved).

| Interrupt ID | Name |
|--------------|------|
| 0 | `IVT_KEYBOARD` |
| 1 | `IVT_TIMER` |
| 2 | `IVT_ILLEGAL_INSTRUCTION` |
| 3 | `IVT_DIVIDE_BY_ZERO` |
| 4 | `IVT_STACK_OVERFLOW` |
| 5 | `IVT_USER_0` |
| 6 | `IVT_USER_1` |

**Flags affected:** none

**Description:** Sets the IVT register for the given interrupt to point to the handler address. After this instruction, when the corresponding interrupt fires, the CPU will jump to `handler_address`.

**Operation:**
```
IVT[interrupt_id] ← handler_address
```

---

# Worked encoding examples

A few concrete instructions, fully encoded.

### `HALT`
```
0x0000     (16 bits)
```

### `CLC`
```
0x0300     (16 bits, sub-opcode 0x6 in bits 10-7)
```

### `PUSH R5`
```
opcode 0x01 = 00001, mode register = 01, reg R5 = 0101
00001 01 0101 00000  →  0x0AA0     (16 bits)
```

### `JMP [0xABCD]`
```
First word:  00100 11 000000000  →  0x26C0
Second word: 0xABCD
Encoded:     0x26C0 0xABCD                  (32 bits)
```

### `ADD R1, R3`
```
opcode 0x16 = 10110, mode_s register = 01, R1 = 0001, R3 = 0011
10110 01 0001 0011 0  →  0xB226            (16 bits)
```

### `STORE R1, [0xABCD]`
```
First word:  10100 11 0001 00000  →  0xA620
Second word: 0xABCD
Encoded:     0xA620 0xABCD                  (32 bits)
```

### `MVINT 0, 0x2800`  (keyboard handler at 0x2800)
```
First word:  11111 00 0000 00000  →  0xF800
Second word: 0x2800
Encoded:     0xF800 0x2800                  (32 bits)
```

---

*MyCustomISA — a custom 16-bit ISA for FPGA and virtual machine implementation.*
