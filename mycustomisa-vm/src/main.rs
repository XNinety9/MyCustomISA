pub mod ui;

use std::collections::{HashSet, VecDeque};

pub const FLAG_Z: u8 = 0;
pub const FLAG_C: u8 = 1;
pub const FLAG_V: u8 = 2;
pub const FLAG_N: u8 = 3;
pub const FLAG_R: u8 = 4;

pub struct CPU {
    pub registers: [u16; 16],
    pub flags: u8,
    pub sp: u16,
    pub pc: u16,
}

impl CPU {
    fn new() -> CPU {
        CPU { registers: [0; 16], flags: 0, sp: 0xF4FF, pc: 0 }
    }
}

pub struct Timer {
    pub period: u16,
    pub counter: u16,
    pub enabled: bool,
    pub loop_mode: bool,
    pub pending_interrupt: bool,
}

impl Timer {
    fn new() -> Timer {
        Timer { period: 0, counter: 0, enabled: false, loop_mode: false, pending_interrupt: false }
    }
}

pub struct Keyboard {
    pub queue: VecDeque<u8>,
    pub modifiers: u8,
    pub held: HashSet<u8>,
    pub last_keycode: u8,
    pub pending_interrupt: bool,
    pub caps_lock: bool,
}

impl Keyboard {
    fn new() -> Keyboard {
        Keyboard {
            queue: VecDeque::new(),
            modifiers: 0,
            held: HashSet::new(),
            last_keycode: 0,
            pending_interrupt: false,
            caps_lock: false,
        }
    }
}

pub struct VirtualMachine {
    pub cpu: CPU,
    pub ram: Vec<u8>,
    pub halted: bool,
    pub keyboard: Keyboard,
    pub timer: Timer,
}

impl VirtualMachine {
    pub fn read_u16(&mut self, addr: u16) -> u16 {
        match addr {
            // Keyboard MMIO
            0xFD00 => self.keyboard.queue.len() as u16,
            0xFD01 => self.keyboard.modifiers as u16,
            0xFD02 => {
                if let Some(kc) = self.keyboard.queue.pop_front() {
                    self.keyboard.last_keycode = kc;
                    kc as u16
                } else {
                    0
                }
            }
            0xFD03 => self.keyboard.held.contains(&self.keyboard.last_keycode) as u16,
            // TIMER_LO/HI — returns current countdown
            0xFD15 => self.timer.counter,
            // TIMER_CTRL — bit 0: ENABLE, bit 1: LOOP
            0xFD17 => (self.timer.enabled as u16) | ((self.timer.loop_mode as u16) << 1),
            _ => ((self.ram[addr as usize] as u16) << 8) | (self.ram[(addr + 1) as usize] as u16),
        }
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        match addr {
            // TIMER_LO/HI — set reload period and reset counter to that period
            0xFD15 => {
                self.timer.period = value;
                self.timer.counter = value;
            }
            // TIMER_CTRL — bit 0: ENABLE, bit 1: LOOP
            0xFD17 => {
                self.timer.enabled = (value & 0x01) != 0;
                self.timer.loop_mode = (value & 0x02) != 0;
            }
            _ => {
                self.ram[addr as usize] = ((value & 0xFF00) >> 8) as u8;
                self.ram[(addr + 1) as usize] = (value & 0xFF) as u8;
            }
        }
    }

    fn extract_register(&self, instruction: u16) -> u16 {
        (instruction & 0x01E0) >> 5
    }

    fn get_operand_value(&mut self, mode: u16, instruction_word: u16, next_word: Option<u16>) -> u16 {
        match mode {
            0b00 => next_word.expect("Immediate mode requires a following word"),
            0b01 => {
                let r = self.extract_register(instruction_word);
                self.cpu.registers[r as usize]
            }
            0b10 => {
                let r = self.extract_register(instruction_word);
                let addr = self.cpu.registers[r as usize];
                self.read_u16(addr)
            }
            0b11 => {
                let addr = next_word.expect("Indirect address mode requires a following word");
                self.read_u16(addr)
            }
            _ => unreachable!("Unknown mode"),
        }
    }

    pub fn new() -> VirtualMachine {
        VirtualMachine { cpu: CPU::new(), ram: vec![0; 65536], halted: false, keyboard: Keyboard::new(), timer: Timer::new() }
    }

    pub fn load(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        if data.len() > 0xECFF + 1 {
            return Err(format!("Binary too large: {} bytes (limit 0xED00)", data.len()));
        }
        self.ram[..data.len()].copy_from_slice(&data);
        Ok(())
    }

    fn set_flags_zcvn(&mut self, z: bool, c: bool, v: bool, n: bool) {
        let mask = (1 << FLAG_Z) | (1 << FLAG_C) | (1 << FLAG_V) | (1 << FLAG_N);
        let bits = ((z as u8) << FLAG_Z) | ((c as u8) << FLAG_C)
                 | ((v as u8) << FLAG_V) | ((n as u8) << FLAG_N);
        self.cpu.flags = (self.cpu.flags & !mask) | bits;
    }

    fn set_flags_zcn(&mut self, z: bool, c: bool, n: bool) {
        let mask = (1 << FLAG_Z) | (1 << FLAG_C) | (1 << FLAG_N);
        let bits = ((z as u8) << FLAG_Z) | ((c as u8) << FLAG_C) | ((n as u8) << FLAG_N);
        self.cpu.flags = (self.cpu.flags & !mask) | bits;
    }

    fn set_flags_zn(&mut self, z: bool, n: bool) {
        let mask = (1 << FLAG_Z) | (1 << FLAG_N);
        let bits = ((z as u8) << FLAG_Z) | ((n as u8) << FLAG_N);
        self.cpu.flags = (self.cpu.flags & !mask) | bits;
    }

    fn trigger_interrupt(&mut self, int_id: u8, return_pc: u16) {
        self.ram[(self.cpu.sp - 1) as usize] = self.cpu.flags;
        self.cpu.sp -= 1;
        for i in 0..=15_usize {
            self.ram[(self.cpu.sp - 2) as usize] = ((self.cpu.registers[i] & 0xFF00) >> 8) as u8;
            self.ram[(self.cpu.sp - 1) as usize] = (self.cpu.registers[i] & 0xFF) as u8;
            self.cpu.sp -= 2;
        }
        self.ram[(self.cpu.sp - 2) as usize] = ((return_pc & 0xFF00) >> 8) as u8;
        self.ram[(self.cpu.sp - 1) as usize] = (return_pc & 0xFF) as u8;
        self.cpu.sp -= 2;
        let ivt_addr = 0xFE00_u16 + int_id as u16 * 2;
        self.cpu.pc = self.read_u16(ivt_addr);
    }

    pub fn key_down(&mut self, keycode: u8, modifiers: u8) {
        self.keyboard.held.insert(keycode);
        if self.keyboard.queue.len() >= 16 {
            self.keyboard.queue.pop_front();
        }
        self.keyboard.queue.push_back(keycode);
        self.keyboard.modifiers = modifiers;
        self.keyboard.pending_interrupt = true;
    }

    pub fn key_up(&mut self, keycode: u8) {
        self.keyboard.held.remove(&keycode);
    }

    pub fn instruction_byte_range(&self) -> (u16, u16) {
        let pc = self.cpu.pc;
        if (pc as usize) + 1 >= self.ram.len() { return (pc, pc + 1); }
        let word = ((self.ram[pc as usize] as u16) << 8) | self.ram[(pc + 1) as usize] as u16;
        let opcode = (word & 0xF800) >> 11;
        let mode = (word & 0x0600) >> 9;
        let is_32bit = opcode != 0 && (mode == 0b00 || mode == 0b11);
        if is_32bit { (pc, pc + 3) } else { (pc, pc + 1) }
    }

    // Execute one instruction. Returns false when the CPU has halted.
    pub fn step(&mut self) -> bool {
        if self.halted { return false; }

        // Deliver pending keyboard interrupt before next instruction
        if self.keyboard.pending_interrupt {
            self.keyboard.pending_interrupt = false;
            let ret_pc = self.cpu.pc;
            self.trigger_interrupt(0, ret_pc);
            return !self.halted;
        }

        // Deliver pending timer interrupt (lower priority than keyboard)
        if self.timer.pending_interrupt {
            self.timer.pending_interrupt = false;
            let ret_pc = self.cpu.pc;
            self.trigger_interrupt(1, ret_pc);
            return !self.halted;
        }

        'execute: {
            let instruction: u16 = self.read_u16(self.cpu.pc);
            let opcode = (instruction & 0xF800) >> 11;

            match opcode {
                // ── Format 0 ─────────────────────────────────────────
                0x00 => {
                    let sub = (instruction & 0x0780) >> 7;
                    match sub {
                        // HALT — stop execution permanently
                        0x00 => { self.halted = true; break 'execute; }
                        // RET — pop PC from stack, return to caller
                        0x01 => {
                            self.cpu.pc = self.read_u16(self.cpu.sp);
                            self.cpu.sp += 2;
                            break 'execute;
                        }
                        // RETI — return from interrupt: pop PC, all 16 registers, then flags
                        0x02 => {
                            self.cpu.pc = self.read_u16(self.cpu.sp);
                            self.cpu.sp += 2;
                            for i in (0..=15).rev() {
                                self.cpu.registers[i] = self.read_u16(self.cpu.sp);
                                self.cpu.sp += 2;
                            }
                            self.cpu.flags = self.ram[self.cpu.sp as usize];
                            self.cpu.sp += 1;
                            break 'execute;
                        }
                        0x03 => self.cpu.flags |= 1 << FLAG_Z,                    // SEZ — set Zero flag
                        0x04 => self.cpu.flags &= !(1 << FLAG_Z),                 // CLZ — clear Zero flag
                        0x05 => self.cpu.flags |= 1 << FLAG_C,                    // SEC — set Carry flag
                        0x06 => self.cpu.flags &= !(1 << FLAG_C),                 // CLC — clear Carry flag
                        0x07 => self.cpu.flags |= 1 << FLAG_V,                    // SEV — set Overflow flag
                        0x08 => self.cpu.flags &= !(1 << FLAG_V),                 // CLV — clear Overflow flag
                        0x09 => self.cpu.flags |= 1 << FLAG_N,                    // SEN — set Negative flag
                        0x0A => self.cpu.flags &= !(1 << FLAG_N),                 // CLN — clear Negative flag
                        _ => unreachable!("Unknown sub-opcode 0x{:X}", sub),
                    }
                    self.cpu.pc += 2;
                }

                // ── Format 1 ─────────────────────────────────────────
                0x01..=0x12 => {
                    let mode     = (instruction & 0x0600) >> 9;
                    let register = (instruction & 0x01E0) >> 5;
                    let next_word = if mode == 0b00 || mode == 0b11 {
                        Some(self.read_u16(self.cpu.pc + 2))
                    } else { None };

                    match opcode {
                        // PUSH — SP -= 2; write operand word to [SP]
                        0x01 => {
                            let value = self.get_operand_value(mode, instruction, next_word);
                            self.ram[(self.cpu.sp - 2) as usize] = ((value & 0xFF00) >> 8) as u8;
                            self.ram[(self.cpu.sp - 1) as usize] = (value & 0xFF) as u8;
                            self.cpu.sp -= 2;
                        }
                        // POP — read word from [SP], SP += 2; write to register or memory destination
                        0x02 => {
                            self.cpu.sp += 2;
                            let value = self.read_u16(self.cpu.sp - 2);
                            if mode == 0b01 {
                                self.cpu.registers[register as usize] = value;
                            } else if mode == 0b10 {
                                let addr = self.cpu.registers[register as usize];
                                self.write_u16(addr, value);
                            } else if mode == 0b11 {
                                let dest = next_word.unwrap();
                                self.write_u16(dest, value);
                            }
                        }
                        // NOT — bitwise invert register in-place; sets Z, N
                        0x03 => {
                            let result = !self.cpu.registers[register as usize];
                            self.cpu.registers[register as usize] = result;
                            if result == 0 { self.cpu.flags |= 1 << FLAG_Z; } else { self.cpu.flags &= !(1 << FLAG_Z); }
                            if result & 0x8000 != 0 { self.cpu.flags |= 1 << FLAG_N; } else { self.cpu.flags &= !(1 << FLAG_N); }
                        }
                        // JMP — unconditional
                        0x04 => {
                            self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                            break 'execute;
                        }
                        // JE — Z = 1
                        0x05 => {
                            if self.cpu.flags & (1 << FLAG_Z) != 0 {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JNE — Z = 0
                        0x06 => {
                            if self.cpu.flags & (1 << FLAG_Z) == 0 {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JGT — signed >: Z=0 AND N=V
                        0x07 => {
                            let n = (self.cpu.flags >> FLAG_N) & 1;
                            let v = (self.cpu.flags >> FLAG_V) & 1;
                            if self.cpu.flags & (1 << FLAG_Z) == 0 && n == v {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JGE — signed ≥: N=V
                        0x08 => {
                            let n = (self.cpu.flags >> FLAG_N) & 1;
                            let v = (self.cpu.flags >> FLAG_V) & 1;
                            if n == v {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JLT — signed <: N≠V
                        0x09 => {
                            let n = (self.cpu.flags >> FLAG_N) & 1;
                            let v = (self.cpu.flags >> FLAG_V) & 1;
                            if n != v {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JLE — signed ≤: Z=1 OR N≠V
                        0x0A => {
                            let n = (self.cpu.flags >> FLAG_N) & 1;
                            let v = (self.cpu.flags >> FLAG_V) & 1;
                            if self.cpu.flags & (1 << FLAG_Z) != 0 || n != v {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JA — unsigned >: C=0 AND Z=0
                        0x0B => {
                            if self.cpu.flags & (1 << FLAG_C) == 0 && self.cpu.flags & (1 << FLAG_Z) == 0 {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JAE — unsigned ≥: C=0
                        0x0C => {
                            if self.cpu.flags & (1 << FLAG_C) == 0 {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JB — unsigned <: C=1
                        0x0D => {
                            if self.cpu.flags & (1 << FLAG_C) != 0 {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // JBE — unsigned ≤: C=1 OR Z=1
                        0x0E => {
                            if self.cpu.flags & (1 << FLAG_C) != 0 || self.cpu.flags & (1 << FLAG_Z) != 0 {
                                self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                                break 'execute;
                            }
                        }
                        // CALL — push return address onto stack, jump to target
                        0x0F => {
                            let pc_step: u16 = if mode == 0b00 || mode == 0b11 { 4 } else { 2 };
                            let ret = self.cpu.pc + pc_step;
                            self.ram[(self.cpu.sp - 2) as usize] = ((ret & 0xFF00) >> 8) as u8;
                            self.ram[(self.cpu.sp - 1) as usize] = (ret & 0xFF) as u8;
                            self.cpu.sp -= 2;
                            self.cpu.pc = self.get_operand_value(mode, instruction, next_word);
                            break 'execute;
                        }
                        // TRAP — software interrupt: push flags, all regs, and return PC; jump via IVT[0xFE0A + trap*2]
                        0x10 => {
                            let trap_num = next_word.unwrap();
                            let ret = self.cpu.pc + 4;
                            self.ram[(self.cpu.sp - 1) as usize] = self.cpu.flags;
                            self.cpu.sp -= 1;
                            for i in 0..=15_usize {
                                self.ram[(self.cpu.sp - 2) as usize] = ((self.cpu.registers[i] & 0xFF00) >> 8) as u8;
                                self.ram[(self.cpu.sp - 1) as usize] = (self.cpu.registers[i] & 0xFF) as u8;
                                self.cpu.sp -= 2;
                            }
                            self.ram[(self.cpu.sp - 2) as usize] = ((ret & 0xFF00) >> 8) as u8;
                            self.ram[(self.cpu.sp - 1) as usize] = (ret & 0xFF) as u8;
                            self.cpu.sp -= 2;
                            let ivt_addr = 0xFE0A_u16 + trap_num * 2;
                            self.cpu.pc = self.read_u16(ivt_addr);
                            break 'execute;
                        }
                        // GETSP — copy SP into destination register
                        0x11 => { self.cpu.registers[register as usize] = self.cpu.sp; }
                        // SETSP — copy source register into SP
                        0x12 => { self.cpu.sp = self.cpu.registers[register as usize]; }
                        _ => unreachable!("Unknown opcode in Format 1: 0x{:X}", opcode),
                    }

                    let pc_step: u16 = if mode == 0b00 || mode == 0b11 { 4 } else { 2 };
                    self.cpu.pc += pc_step;
                }

                // ── Format 2 ─────────────────────────────────────────
                0x13..=0x1F => {
                    let mode     = (instruction & 0x0600) >> 9;
                    let reg_a    = (instruction & 0x01E0) >> 5;
                    let reg_b    = (instruction & 0x001E) >> 1;
                    let next_word = if mode == 0b00 || mode == 0b11 {
                        Some(self.read_u16(self.cpu.pc + 2))
                    } else { None };
                    let pc_step: u16 = if mode == 0b00 || mode == 0b11 { 4 } else { 2 };

                    match opcode {
                        // LOAD — read value from immediate / [reg] / [addr], store into register
                        0x13 => {
                            let value = self.get_operand_value(mode, instruction, next_word);
                            let dest = if mode == 0b10 { reg_b } else { reg_a };
                            self.cpu.registers[dest as usize] = value;
                        }
                        // STORE — write R{reg_a} to memory; mode 10: dest=[R{reg_b}], mode 11: dest=next_word
                        0x14 => {
                            let src_val   = self.cpu.registers[reg_a as usize];
                            let dest_addr = if mode == 0b10 {
                                self.cpu.registers[reg_b as usize]
                            } else {
                                next_word.unwrap()
                            };
                            self.write_u16(dest_addr, src_val);
                        }
                        // MOVE — copy R{reg_a} into R{reg_b}; no flags
                        0x15 => {
                            self.cpu.registers[reg_b as usize] = self.cpu.registers[reg_a as usize];
                        }
                        // ADD — dest += src; sets Z, C (unsigned overflow), V (signed overflow), N
                        0x16 => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            let prev  = self.cpu.registers[di];
                            let full  = prev as u32 + src as u32;
                            let result = full as u16;
                            self.cpu.registers[di] = result;
                            self.set_flags_zcvn(
                                result == 0,
                                full > 0xFFFF,
                                (!(prev ^ src) & (prev ^ result) & 0x8000) != 0,
                                result & 0x8000 != 0,
                            );
                        }
                        // SUB — dest -= src; sets Z, C (borrow), V (signed overflow), N
                        0x17 => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            let prev   = self.cpu.registers[di];
                            let result = prev.wrapping_sub(src);
                            self.cpu.registers[di] = result;
                            self.set_flags_zcvn(
                                result == 0,
                                prev < src,
                                ((prev ^ src) & (prev ^ result) & 0x8000) != 0,
                                result & 0x8000 != 0,
                            );
                        }
                        // MUL — dest *= src; C and V both set if the full 32-bit product overflows 16 bits; sets Z, N
                        0x18 => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            let full   = self.cpu.registers[di] as u32 * src as u32;
                            let result = full as u16;
                            self.cpu.registers[di] = result;
                            let overflow = full > 0xFFFF;
                            self.set_flags_zcvn(result == 0, overflow, overflow, result & 0x8000 != 0);
                        }
                        // DIV — dest /= src; sets Z, N; triggers interrupt 3 on division by zero
                        0x19 => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            if src == 0 {
                                self.trigger_interrupt(3, self.cpu.pc + pc_step);
                                break 'execute;
                            }
                            let result = self.cpu.registers[di] / src;
                            self.cpu.registers[di] = result;
                            self.set_flags_zn(result == 0, result & 0x8000 != 0);
                        }
                        // SHL — dest <<= src (low 5 bits); C = last bit shifted out; dest = 0 if shift ≥ 16; sets Z, C, N
                        0x1A => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            let n    = (src & 0x1F) as u32;
                            let prev = self.cpu.registers[di] as u32;
                            let c    = n > 0 && n <= 16 && (prev >> (16 - n)) & 1 != 0;
                            let result = if n >= 16 { 0u16 } else { (prev << n) as u16 };
                            self.cpu.registers[di] = result;
                            self.set_flags_zcn(result == 0, c, result & 0x8000 != 0);
                        }
                        // SHR — dest >>= src (low 5 bits); C = last bit shifted out; dest = 0 if shift ≥ 16; sets Z, C, N
                        0x1B => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            let n    = (src & 0x1F) as u32;
                            let prev = self.cpu.registers[di] as u32;
                            let c    = n > 0 && n <= 16 && (prev >> (n - 1)) & 1 != 0;
                            let result = if n >= 16 { 0u16 } else { (prev >> n) as u16 };
                            self.cpu.registers[di] = result;
                            self.set_flags_zcn(result == 0, c, result & 0x8000 != 0);
                        }
                        // AND — dest &= src; sets Z, N
                        0x1C => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            let result = self.cpu.registers[di] & src;
                            self.cpu.registers[di] = result;
                            self.set_flags_zn(result == 0, result & 0x8000 != 0);
                        }
                        // OR — dest |= src; sets Z, N
                        0x1D => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            let result = self.cpu.registers[di] | src;
                            self.cpu.registers[di] = result;
                            self.set_flags_zn(result == 0, result & 0x8000 != 0);
                        }
                        // CMP — compute dest - src, set Z/C/V/N, discard result (flags only, dest unchanged)
                        0x1E => {
                            let (src, di) = if mode == 0b01 {
                                (self.cpu.registers[reg_a as usize], reg_b as usize)
                            } else {
                                (next_word.unwrap(), reg_a as usize)
                            };
                            let prev   = self.cpu.registers[di];
                            let result = prev.wrapping_sub(src);
                            self.set_flags_zcvn(
                                result == 0,
                                prev < src,
                                ((prev ^ src) & (prev ^ result) & 0x8000) != 0,
                                result & 0x8000 != 0,
                            );
                        }
                        // MVINT — write handler address into IVT slot at 0xFE00 + int_id*2
                        0x1F => {
                            let int_id   = (instruction & 0x00E0) >> 5;
                            let handler  = next_word.unwrap();
                            let ivt_addr = 0xFE00_u16 + int_id * 2;
                            self.write_u16(ivt_addr, handler);
                        }
                        _ => unreachable!("Unknown opcode in Format 2: 0x{:X}", opcode),
                    }

                    self.cpu.pc += pc_step;
                }

                _ => unreachable!("Unknown opcode: 0x{:X}", opcode),
            }
        } // end 'execute

        // Tick timer once per instruction; fire when counter reaches zero
        if self.timer.enabled && self.timer.period > 0 {
            self.timer.counter = self.timer.counter.saturating_sub(1);
            if self.timer.counter == 0 {
                self.timer.pending_interrupt = true;
                if self.timer.loop_mode {
                    self.timer.counter = self.timer.period;
                } else {
                    self.timer.enabled = false;
                }
            }
        }

        !self.halted
    }

    pub fn run(&mut self) {
        while self.step() {}
    }
}

// ── Instruction decoder ───────────────────────────────────────────────

pub fn decode_instruction(pc: u16, ram: &[u8]) -> String {
    let pc = pc as usize;
    if pc + 1 >= ram.len() { return "???".into(); }

    let word    = ((ram[pc] as u16) << 8) | ram[pc + 1] as u16;
    let opcode  = (word & 0xF800) >> 11;
    let mode    = (word & 0x0600) >> 9;
    let reg_a   = (word & 0x01E0) >> 5;
    let reg_b   = (word & 0x001E) >> 1;

    let next = || -> u16 {
        if pc + 3 < ram.len() { ((ram[pc + 2] as u16) << 8) | ram[pc + 3] as u16 } else { 0 }
    };

    let src = |mode: u16| -> String {
        match mode {
            0b00 => format!("0x{:04X}", next()),
            0b01 => format!("R{}", reg_a),
            0b10 => format!("[R{}]", reg_a),
            0b11 => format!("[0x{:04X}]", next()),
            _    => "???".into(),
        }
    };

    match opcode {
        0x00 => {
            let sub = (word & 0x0780) >> 7;
            match sub {
                0x0 => "HALT", 0x1 => "RET",  0x2 => "RETI",
                0x3 => "SEZ",  0x4 => "CLZ",
                0x5 => "SEC",  0x6 => "CLC",
                0x7 => "SEV",  0x8 => "CLV",
                0x9 => "SEN",  0xA => "CLN",
                _ => return format!("??? sub=0x{:X}", sub),
            }.into()
        }

        op @ 0x01..=0x12 => {
            let mne = match op {
                0x01 => "PUSH",  0x02 => "POP",   0x03 => "NOT",
                0x04 => "JMP",   0x05 => "JE",    0x06 => "JNE",
                0x07 => "JGT",   0x08 => "JGE",   0x09 => "JLT",
                0x0A => "JLE",   0x0B => "JA",    0x0C => "JAE",
                0x0D => "JB",    0x0E => "JBE",   0x0F => "CALL",
                0x10 => "TRAP",  0x11 => "GETSP", 0x12 => "SETSP",
                _ => "???",
            };
            format!("{} {}", mne, src(mode))
        }

        0x13 => match mode {
            0b10 => format!("LOAD [R{}], R{}", reg_a, reg_b),
            0b00 => format!("LOAD 0x{:04X}, R{}", next(), reg_a),
            0b11 => format!("LOAD [0x{:04X}], R{}", next(), reg_a),
            _    => format!("LOAD ???, R{}", reg_a),
        },

        0x14 => match mode {
            0b10 => format!("STORE R{}, [R{}]", reg_a, next() & 0xF),
            0b11 => format!("STORE R{}, [0x{:04X}]", reg_a, next()),
            _    => format!("STORE R{}, ???", reg_a),
        },

        0x15 => format!("MOVE R{}, R{}", reg_a, reg_b),

        op @ 0x16..=0x1E => {
            let mne = match op {
                0x16 => "ADD", 0x17 => "SUB", 0x18 => "MUL",
                0x19 => "DIV", 0x1A => "SHL", 0x1B => "SHR",
                0x1C => "AND", 0x1D => "OR",  0x1E => "CMP",
                _ => "???",
            };
            let (s, d) = match mode {
                0b01 => (format!("R{}",        reg_a), format!("R{}", reg_b)),
                0b00 => (format!("0x{:04X}",   next()), format!("R{}", reg_a)),
                _    => (format!("???"),                 format!("R{}", reg_a)),
            };
            format!("{} {}, {}", mne, s, d)
        }

        0x1F => {
            let int_id = (word & 0x00E0) >> 5;
            format!("MVINT {}, 0x{:04X}", int_id, next())
        }

        _ => format!("??? op=0x{:X}", opcode),
    }
}

// ── Entry point ───────────────────────────────────────────────────────

fn main() {
    ui::launch(VirtualMachine::new());
}
