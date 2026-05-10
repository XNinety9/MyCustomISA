# MyCustomISA

MyCustomISA is a custom 16-bit CPU architecture designed to run on both an FPGA and a virtual machine. It is an educational project aimed at exploring the fundamentals of CPU design from the ground up — word size, memory mapping, registers, flags, interrupts, and instruction sets — with every decision documented and justified.

The CPU targets a 64KB address space, drives a 128×128 black-and-white display, and is simple enough to implement end-to-end while remaining powerful enough to run real assembly programs.

---

## Table of Contents

1. [Designing the Memory Map](./doc/mycustomisa_memory_map.md)
   - Word size and address space
   - Memory regions (code, stack, framebuffer, MMIO)
   - Boot state and register initialisation
   - Flags and the interrupt system
   - Complete 64KB memory map

2. Designing the Instruction Set `[TODO]`
   - Instruction encoding in 16-bit words
   - Operand types and addressing modes
   - Arithmetic, logic, and control flow instructions
   - PUSH / POP and the call convention
   - MVINT and the trap instruction

3. Writing the Assembler `[TODO]`
   - Tokenising and parsing assembly source
   - Resolving labels to concrete addresses
   - Emitting binary output

4. Building the Virtual Machine `[TODO]`
   - Fetch–decode–execute loop
   - Emulating MMIO (keyboard, display, timer, debug)
   - Interrupt dispatch and context save/restore
   - Debugger interface and state inspection

5. FPGA Implementation `[TODO]`
   - Top-level architecture and clock domain
   - Datapath and control unit in HDL
   - Mapping MMIO to physical peripherals
   - Synthesis and timing closure

---

*MyCustomISA — a custom 16-bit ISA for FPGA and virtual machine implementation.*
