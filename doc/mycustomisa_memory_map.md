# Designing the memory map of your custom CPU

So you want to build your own CPU. Great! Before you write a single line of assembly or a single hardware description, you need to answer a deceptively simple question: **what lives where in memory, and why?**

This document walks through every decision behind the memory map of **MyCustomISA** — a custom 16-bit CPU targeting both an FPGA and a virtual machine. Nothing here is arbitrary. Every choice has a reason, and understanding those reasons is more important than memorising the addresses.

## Step 1: pick a word size

The very first thing you need to decide is the **word size** — how many bits your CPU chews on at a time. This single number ripples through everything else: how big your registers are, how much memory you can address, how instructions are packed into bits.

The common choices for a first CPU are 8, 16, or 32 bits. To make a good decision, you need to think about what your CPU actually needs to do. For MyCustomISA, the goals are:

- Run small assembly programs
- Drive a 128×128 black and white display
- Be simple enough to actually implement

Let's do some concrete maths.

### How much memory does the display need?

A 128×128 display with black and white pixels needs exactly 1 bit per pixel. That's:

```text
128 × 128 = 16,384 pixels
16,384 bits ÷ 8 = 2,048 bytes = 2KB
```

So your framebuffer will eat 2KB of memory, no matter what word size you pick.

### How much memory does a 16-bit CPU give you?

A 16-bit address bus can point to **2^16 distinct memory locations**. Since each location holds 1 byte:

```text
2^16 = 65,536 bytes = 64KB
```

So a 16-bit CPU gives you 64KB of total addressable memory. Subtract 2KB for the framebuffer and you still have 62KB for your program and everything else. That's very comfortable for an educational CPU.

An 8-bit CPU would only give you 256 bytes of address space — not even enough for the framebuffer! A 32-bit CPU would give you 4GB, which is overkill and makes the hardware much more complex to implement.

**Decision: 16-bit word size, 64KB address space.**

## Step 2: figure out what needs to live in memory

Now that you know how much space you have, you need to figure out how to use it. A running program doesn't just need space for its own instructions — it needs several distinct regions, each serving a very different purpose.

### The code segment

This is the compiled program itself — the binary produced by your assembler. The CPU fetches instructions from here one by one. It lives at the bottom of memory starting at address `0x0000`, and grows upward as your program gets larger.

### The stack

When a function is called, the CPU needs somewhere to store the return address, local variables, and the state of registers. That place is the stack. It's managed automatically by the CPU via the Stack Pointer (SP) register: every PUSH decrements SP and writes data, every POP reads data and increments SP.

The important thing about the stack is that it grows in the **opposite direction** to the code segment — downward, from a high address toward a low one.

Here's why that matters. Imagine both regions growing in the same direction:

```text
0xFFFF  ┤
        │  (dead space, never used)
        │
        ├  top of stack ↑
        ├  bottom of stack
        ├  top of code
        ├  code...
0x0000  ┤  code start
```

The space above the stack is permanently wasted. Now imagine them growing toward each other:

```text
0xFFFF  ┤  stack start
        ├  stack grows ↓
        │
        │  ← shared free space →
        │
        ├  code grows ↑
0x0000  ┤  code start
```

Now they share the middle space. Both can grow as much as they want, and the only limit is when they actually meet — which is a real out-of-memory condition, not an artificial one. This is how virtually every real architecture works.

### The framebuffer

A fixed 2KB region where each bit represents one pixel on the 128×128 display. Writing a 1 to a bit turns that pixel white, writing a 0 turns it black. The display hardware (or VM) reads this region to know what to draw.

### Memory-mapped I/O (MMIO)

Here's a neat trick used by almost every simple CPU: instead of having special instructions to talk to hardware (keyboard, display controller, timer...), you just reserve a range of memory addresses for them. Writing to address `0xFD02` doesn't write to RAM — it sends a keycode to a register in the keyboard controller. Reading from `0xFD00` doesn't read RAM — it tells you how many keys are waiting in the queue.

This is called **memory-mapped I/O**, and it means your CPU only needs one set of memory instructions to talk to both RAM and hardware. Simple and elegant.

### Why no heap?

You might be wondering: where's the heap? Where does `malloc` live?

`malloc` is more complex than it looks. On a system with an OS, it works by asking the kernel for a chunk of memory, then managing that chunk internally — keeping a list of which blocks are allocated and which are free. On a bare-metal system like ours, **you** would have to implement that allocator yourself, in assembly.

For MyCustomISA v1, we skip the heap entirely. All memory layout is decided at compile time by the assembler. Dynamic allocation can be added in a future version once the basics are solid.

## Step 3: decide on boot state

When the CPU first powers on, what state is it in? If registers contain garbage, you can't safely execute a single instruction. So the hardware needs to guarantee certain things before the first instruction runs. The programmer should not have to do this manually — it's the CPU's job.

Here's what MyCustomISA guarantees at boot:

| Register | Initial value | Why |
|----------|--------------|-----|
| PC | `0x0000` | The first instruction is at the start of the code segment |
| SP | `0xF4FF` | First byte of the stack region, top-down (post-decrement convention) |
| R0–R15 | `0x0000` | Clean slate — no garbage values to cause bugs |
| Flags | All `0` | No operations have run yet, so no flags should be set |
| IVT registers | `0xFF00` | Points to the default fault handler (explained later) |

### The stack pointer convention: post-decrement

There are two common conventions for how the stack pointer behaves on a PUSH:

- **Pre-decrement**: SP decrements first, then data is written to the new SP
- **Post-decrement**: data is written to current SP, then SP decrements

MyCustomISA uses **post-decrement**, so SP always points to the last written value. This means the initial SP should point to the first valid stack address — the topmost byte of the stack region.

## Step 4: define the registers

### General purpose registers

MyCustomISA has **16 general purpose registers**, named R0 through R15. Each is 16 bits wide and initialised to zero at boot.

They are truly general purpose — any register can hold a plain integer value or a memory address. This is a deliberate RISC-style decision. Some older architectures like the Motorola 68000 split registers into "data registers" (for values) and "address registers" (for pointers). That distinction adds complexity without much benefit for a simple CPU. Here, any register can do either job.

### Special registers

Beyond the general purpose registers, the CPU maintains three registers that have specific hardware roles:

**PC (Program Counter):** always holds the address of the next instruction to fetch. The programmer doesn't usually write to PC directly — jump instructions do it implicitly.

**SP (Stack Pointer):** holds the address of the top of the stack. Automatically updated by PUSH and POP instructions. Can also be read or written directly for advanced use cases like saving and restoring a full execution context.

**FLAGS:** holds the state of the CPU's condition flags. Not readable or writable as a single 16-bit word — instead, individual flags are set and cleared by dedicated instructions. This prevents a programmer from accidentally corrupting CPU state by writing a raw value to the flags register.

## Step 5: define the flags

Flags are single-bit signals set as side effects of arithmetic and logic operations. They're what makes conditional branching possible — a `JZ` instruction (jump if zero) checks the Z flag, for example.

MyCustomISA has five flags:

| Flag | Name | Set when... |
|------|------|-------------|
| Z | Zero | The result of the last operation was exactly zero |
| C | Carry | An **unsigned** operation produced a bit that didn't fit in 16 bits |
| V | Overflow | A **signed** operation produced a mathematically wrong sign |
| N | Negative | The most significant bit of the result is 1 (i.e. the result is negative in signed arithmetic) |
| R | Reset | The CPU has just been reset |

### The difference between Carry and Overflow

This trips up a lot of people, so let's be concrete. Consider these two 16-bit additions:

```
Case 1:  0xFFFF + 0x0001 = 0x10000  →  truncated to 0x0000
Case 2:  0x7FFF + 0x0001 = 0x8000
```

In **Case 1**, the result doesn't fit in 16 bits at all — there's a carry out of the top bit. This is a problem for **unsigned** arithmetic (where 0xFFFF = 65535, and 65535 + 1 should be 65536). The **Carry flag** is set.

In **Case 2**, the result fits in 16 bits just fine. But look at it from a **signed** perspective: 0x7FFF is +32767, the largest positive 16-bit signed number. Adding 1 should give +32768 — but that number doesn't exist in 16-bit signed representation. Instead we get 0x8000, which is -32768. A positive plus a positive gave a negative — that's mathematically wrong. The **Overflow flag** is set.

In short: **Carry** means the result was wrong for unsigned arithmetic. **Overflow** means the result was wrong for signed arithmetic. The CPU always sets both flags appropriately — it's up to your program to decide which one to check, based on whether you're treating your numbers as signed or unsigned.

### Flag access rules

- **Z, C, V, N** are read/write, but only flag by flag — via dedicated instructions like `CLC` (clear carry) or `SEZ` (set zero). You cannot write the entire flags register as a word.
- **R** is read-only. The CPU sets it on reset and clears it automatically after the first instruction runs.

## Step 6: design the interrupt system

What happens when the user presses a key? What happens when your program accidentally divides by zero? Your CPU needs a way to handle unexpected or asynchronous events without the programmer having to constantly check for them in a loop.

The answer is **interrupts**.

When an interrupt fires, the CPU:

1. Finishes executing the current instruction
2. Saves its full context (PC, SP, all registers, flags) onto the stack
3. Looks up the address of the appropriate handler
4. Jumps to that handler
5. When the handler is done, executes a `RETI` instruction to restore the saved context and continue where it left off

This is a clean mechanism: the interrupted program has no idea an interrupt happened, and the interrupt handler has a full snapshot of the CPU state to work with.

### Three kinds of interrupts

Not all interrupts come from the same place:

**Hardware interrupts** are triggered by external events — a key being pressed, a timer reaching zero. The hardware signals the CPU asynchronously, at any point during execution.

**Exceptions** are triggered by the CPU itself when something goes wrong during execution — an illegal instruction, a division by zero, the stack colliding with the code segment. The program caused these, usually by accident.

**Traps** are triggered deliberately by the program itself, using a dedicated instruction. They're useful for implementing things like user-defined callbacks or simple "syscall" mechanisms in the absence of an OS.

### The Interrupt Vector Table (IVT)

For each interrupt, the CPU needs to know where to jump. MyCustomISA handles this with **dedicated 16-bit special registers** — one per interrupt — each holding the address of that interrupt's handler. This is the Interrupt Vector Table (IVT).

At boot, all IVT registers are initialised to `0xFF00` — the address of the default fault handler (more on that below). The programmer sets them to real handler addresses using the `MVINT` instruction, passing the handler's label, which the assembler resolves to a concrete address.

| IVT register | Address | Category | Fires when... |
|-------------|---------|----------|--------------|
| IVT_KEYBOARD | `0xFE00` | Hardware | A key is pressed |
| IVT_TIMER | `0xFE02` | Hardware | The timer countdown reaches zero |
| IVT_ILLEGAL_INSTRUCTION | `0xFE04` | Exception | The CPU encounters an unknown opcode |
| IVT_DIVIDE_BY_ZERO | `0xFE06` | Exception | A division by zero is attempted |
| IVT_STACK_OVERFLOW | `0xFE08` | Exception | The stack pointer collides with the code segment |
| IVT_USER_0 | `0xFE0A` | Trap | User-defined trap 0 |
| IVT_USER_1 | `0xFE0C` | Trap | User-defined trap 1 |

### The default fault handler

What if the programmer doesn't set a handler for, say, `IVT_DIVIDE_BY_ZERO`? The IVT register still points to `0xFF00` — the default fault handler. This is a small piece of code, always present at that fixed address in the reserved region, that does one thing: **halts the CPU and freezes its state**.

Why is this better than a silent reset or a jump to `0x0000`? Because when the CPU halts cleanly, you can inspect everything — which registers held what values, what address the crash happened at, what was on the stack. That's the difference between a debuggable crash and a mysterious silent reboot.

## Step 7: memory-mapped I/O in detail

All peripheral communication goes through the MMIO region at `0xFD00`–`0xFDFF`. Here's the full layout.

### Keyboard (`0xFD00`–`0xFD13`)

The keyboard is modelled as a 16-byte FIFO queue. Each time a key is pressed, its keycode is pushed into the queue. The program reads keycodes by reading from the DATA register, which pops the oldest entry.

| Address | Name | Description |
|---------|------|-------------|
| `0xFD00` | STATUS | Number of keycodes currently waiting in the queue |
| `0xFD01` | MODIFIERS | Current state of modifier keys (see below) |
| `0xFD02` | DATA | Read (and pop) the next keycode from the queue |
| `0xFD03` | KEYSTATE | `1` if the last popped key is still physically held down, `0` if released |
| `0xFD04`–`0xFD13` | QUEUE | The internal 16-byte FIFO buffer |

The MODIFIERS register is a bitmask — each bit represents one modifier key:

| Bit | Modifier |
|-----|----------|
| 0 | SHIFT |
| 1 | CTRL |
| 2 | ALT |
| 3 | CAPS LOCK |
| 4–7 | Reserved |

Key encoding uses a simple two-range scheme:

| Range | Meaning |
|-------|---------|
| `0x00`–`0x7F` | Standard ASCII (letters, digits, punctuation, ENTER, ESC, BACKSPACE...) |
| `0x80`–`0x8B` | Function keys F1–F12 |
| `0x90`–`0x99` | Arrow keys, PAGE UP/DOWN, HOME, END, INSERT, DELETE |

If the queue is full (16 keys pending) and another key is pressed, the **oldest entry is dropped**. If no interrupt handler is set for `IVT_KEYBOARD`, programs can poll the STATUS register instead:

```asm
wait_key:
    LOAD R0, [0xFD00]   ; how many keys are waiting?
    CMP  R0, 0
    JE   wait_key       ; none yet, keep checking
    LOAD R1, [0xFD02]   ; pop the keycode into R1
    LOAD R2, [0xFD03]   ; is it still being held? 1=yes, 0=no
```

### Display control (`0xFD14`)

The framebuffer is written to directly by your program. The DISPLAY_CTRL register lets you send control signals to the display:

| Address | Name | Description |
|---------|------|-------------|
| `0xFD14` | DISPLAY_CTRL | Display control register |

| Bit | Name | Effect when set to 1 |
|-----|------|----------------------|
| 0 | CLEAR | Erase the entire framebuffer (fill with black) |
| 1 | REFRESH | Push the current framebuffer contents to the display |

### Timer (`0xFD15`–`0xFD17`)

The timer counts down from a 16-bit value. When it hits zero, it fires the `IVT_TIMER` interrupt. With the LOOP bit set, it automatically reloads and starts again — useful for driving a consistent game loop at a fixed tick rate.

| Address | Name | Description |
|---------|------|-------------|
| `0xFD15` | TIMER_LO | Low byte of the 16-bit countdown value |
| `0xFD16` | TIMER_HI | High byte of the 16-bit countdown value |
| `0xFD17` | TIMER_CTRL | Control register |

| Bit | Name | Effect |
|-----|------|--------|
| 0 | ENABLE | `1` to start counting, `0` to pause |
| 1 | LOOP | `1` to automatically reload and restart on reaching zero |

### Debug output (`0xFD18`)

Writing a byte to this address prints it to the VM's debug console. No print routine needed — just write the value and it appears. Invaluable for debugging early programs before you have a proper output system.

| Address | Name | Description |
|---------|------|-------------|
| `0xFD18` | DEBUG_OUT | Write a byte here to print it to the VM console |

## Step 8: the complete memory map

Putting it all together, here is the full 64KB memory map for MyCustomISA. Fixed-size regions live at the top, leaving the maximum possible space at the bottom for code to grow into.

| Start | End | Region | Size | Notes |
|-------|-----|--------|------|-------|
| `0xFF00` | `0xFFFF` | Reserved / default handlers | 256 bytes | Default fault handler lives at `0xFF00` |
| `0xFE00` | `0xFEFF` | IVT registers | 256 bytes | 14 bytes used, rest reserved |
| `0xFD00` | `0xFDFF` | Memory-mapped I/O | 256 bytes | Keyboard, display, timer, debug |
| `0xF500` | `0xFCFF` | Framebuffer | 2KB | 128×128 pixels, 1 bit per pixel |
| `0xED00` | `0xF4FF` | Stack | 2KB | Grows downward ↓, SP init = `0xF4FF` |
| `0x0000` | `0xECFF` | Free space + code segment | ~59KB | Code grows upward ↑, PC init = `0x0000` |

### The design principles behind this layout

Everything at the top is fixed in size and position. The code segment at the bottom has no upper limit except the bottom of the stack. The stack has no lower limit except the top of the code segment. When they meet, the CPU raises `IVT_STACK_OVERFLOW` — a real, detectable error, not a silent corruption.

The reserved region at the very top (`0xFF00`–`0xFFFF`) always contains the default fault handler, no matter what program is loaded. Even a completely broken program lands safely there on any unhandled interrupt.

## What comes next

With the memory map locked down, the next step is designing the **instruction set** — deciding which operations the CPU can perform, how many operands each takes, and how instructions are packed into 16-bit words.

That's where your assembler starts to take shape.

*MyCustomISA — a custom 16-bit ISA for FPGA and virtual machine implementation.*
