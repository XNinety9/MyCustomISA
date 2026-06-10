use crate::common::{ParsedLine, Operand, Instruction, operand_is_wide, operand_mode};
use std::collections::HashMap;

const RST: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const YELLOW: &str = "\x1b[93m";
const CYAN: &str = "\x1b[96m";

pub fn encode(parsed_lines: &[ParsedLine], symbols: &HashMap<String, u16>) -> Vec<u16> {
    let mut words: Vec<u16> = Vec::new();
    let mut byte_addr: u16 = 0;

    for line in parsed_lines {
        match line {
            ParsedLine::Label(_) => {}
            ParsedLine::Instruction(instr) => {
                let encoded = encode_instruction(instr, symbols);
                log_line(byte_addr, None, &format!("{}", instr), &encoded);
                byte_addr = byte_addr.wrapping_add((encoded.len() as u16) * 2);
                words.extend(encoded);
            }
            ParsedLine::LabeledInstruction(label, instr) => {
                let encoded = encode_instruction(instr, symbols);
                log_line(byte_addr, Some(label), &format!("{}", instr), &encoded);
                byte_addr = byte_addr.wrapping_add((encoded.len() as u16) * 2);
                words.extend(encoded);
            }
            ParsedLine::Data(bytes) => {
                let encoded = encode_data(bytes);
                log_line(byte_addr, None, &data_text(bytes), &encoded);
                byte_addr = byte_addr.wrapping_add((encoded.len() as u16) * 2);
                words.extend(encoded);
            }
            ParsedLine::LabeledData(label, bytes) => {
                let encoded = encode_data(bytes);
                log_line(byte_addr, Some(label), &data_text(bytes), &encoded);
                byte_addr = byte_addr.wrapping_add((encoded.len() as u16) * 2);
                words.extend(encoded);
            }
        }
    }

    println!("  {}{}{}", DIM, "─".repeat(46), RST);
    println!("  {}{} words  ·  {} bytes{}", BOLD, words.len(), words.len() * 2, RST);

    words
}

fn log_line(addr: u16, label: Option<&str>, text: &str, encoded: &[u16]) {
    let visible_len = label.map(|l| l.len() + 2).unwrap_or(0) + text.len();
    let colored_prefix = label
        .map(|l| format!("{}{}: {}", CYAN, l, RST))
        .unwrap_or_default();
    let pad = " ".repeat(28usize.saturating_sub(visible_len));
    let words_str = encoded.iter()
        .map(|w| format!("{}0x{:04X}{}", YELLOW, w, RST))
        .collect::<Vec<_>>()
        .join("  ");
    println!("  {}0x{:04X}{}  {}{}{}  {}→{}  {}",
        DIM, addr, RST,
        colored_prefix, text, pad,
        DIM, RST,
        words_str);
}

fn data_text(bytes: &[u8]) -> String {
    let hex = bytes.iter().map(|b| format!("0x{:02X}", b)).collect::<Vec<_>>().join(", ");
    format!("data [{}]", hex)
}

fn encode_instruction(instruction: &Instruction, symbols: &HashMap<String, u16>) -> Vec<u16> {
    match instruction {
        // Format 0
        Instruction::Halt => encode_format0(0x00),
        Instruction::Ret  => encode_format0(0x01),
        Instruction::Reti => encode_format0(0x02),
        Instruction::Sez  => encode_format0(0x03),
        Instruction::Clz  => encode_format0(0x04),
        Instruction::Sec  => encode_format0(0x05),
        Instruction::Clc  => encode_format0(0x06),
        Instruction::Sev  => encode_format0(0x07),
        Instruction::Clv  => encode_format0(0x08),
        Instruction::Sen  => encode_format0(0x09),
        Instruction::Cln  => encode_format0(0x0A),

        // Format 1
        Instruction::Push(op)   => encode_format1(0x01, op, symbols),
        Instruction::Pop(op)    => encode_format1(0x02, op, symbols),
        Instruction::Not(op)    => encode_format1(0x03, op, symbols),
        Instruction::Jmp(op)    => encode_format1(0x04, op, symbols),
        Instruction::Je(op)     => encode_format1(0x05, op, symbols),
        Instruction::Jne(op)    => encode_format1(0x06, op, symbols),
        Instruction::Jgt(op)    => encode_format1(0x07, op, symbols),
        Instruction::Jge(op)    => encode_format1(0x08, op, symbols),
        Instruction::Jlt(op)    => encode_format1(0x09, op, symbols),
        Instruction::Jle(op)    => encode_format1(0x0A, op, symbols),
        Instruction::Ja(op)     => encode_format1(0x0B, op, symbols),
        Instruction::Jae(op)    => encode_format1(0x0C, op, symbols),
        Instruction::Jb(op)     => encode_format1(0x0D, op, symbols),
        Instruction::Jbe(op)    => encode_format1(0x0E, op, symbols),
        Instruction::Call(op)   => encode_format1(0x0F, op, symbols),
        Instruction::Trap(op)   => encode_format1(0x10, op, symbols),
        Instruction::Getsp(op)  => encode_format1(0x11, op, symbols),
        Instruction::Setsp(op)  => encode_format1(0x12, op, symbols),

        // Format 2
        Instruction::Load(src, dst)  => encode_format2(0x13, src, dst, symbols),
        Instruction::Store(src, dst) => encode_format2(0x14, src, dst, symbols),
        Instruction::Move(src, dst)  => encode_format2(0x15, src, dst, symbols),
        Instruction::Add(src, dst)   => encode_format2(0x16, src, dst, symbols),
        Instruction::Sub(src, dst)   => encode_format2(0x17, src, dst, symbols),
        Instruction::Mul(src, dst)   => encode_format2(0x18, src, dst, symbols),
        Instruction::Div(src, dst)   => encode_format2(0x19, src, dst, symbols),
        Instruction::Shl(src, dst)   => encode_format2(0x1A, src, dst, symbols),
        Instruction::Shr(src, dst)   => encode_format2(0x1B, src, dst, symbols),
        Instruction::And(src, dst)   => encode_format2(0x1C, src, dst, symbols),
        Instruction::Or(src, dst)    => encode_format2(0x1D, src, dst, symbols),
        Instruction::Cmp(src, dst)   => encode_format2(0x1E, src, dst, symbols),

        Instruction::Mvint(interrupt, handler) => {
            let int_num = *interrupt as u16;
            let handler_addr = match handler {
                Operand::Immediate(val)        => *val,
                Operand::IndirectAddress(addr) => *addr,
                Operand::Label(name)           => *symbols.get(name).expect(&format!("Undefined label: {}", name)),
                _ => unreachable!()
            };
            vec![(0x1F << 11) | (int_num << 6), handler_addr]
        }
    }
}

fn encode_data(bytes: &[u8]) -> Vec<u16> {
    bytes.chunks(2)
        .map(|chunk| {
            let hi = chunk[0] as u16;
            let lo = chunk.get(1).copied().unwrap_or(0) as u16;
            (hi << 8) | lo
        })
        .collect()
}

fn encode_format0(sub_opcode: u16) -> Vec<u16> {
    vec![sub_opcode << 7]
}

fn encode_format1(opcode: u16, op: &Operand, symbols: &HashMap<String, u16>) -> Vec<u16> {
    let mode = operand_mode(op);
    let first_word = (opcode << 11) | (mode << 9);
    if operand_is_wide(op) {
        let payload = match op {
            Operand::Immediate(val)        => *val,
            Operand::IndirectAddress(addr) => *addr,
            Operand::Label(name)           => *symbols.get(name).expect(&format!("Undefined label: {}", name)),
            _ => unreachable!()
        };
        vec![first_word, payload]
    } else {
        let reg = match op {
            Operand::Register(r)         => *r as u16,
            Operand::IndirectRegister(r) => *r as u16,
            _ => unreachable!()
        };
        vec![first_word | (reg << 5)]
    }
}

fn encode_format2(opcode: u16, src: &Operand, dst: &Operand, symbols: &HashMap<String, u16>) -> Vec<u16> {
    if operand_is_wide(src) {
        let mode = operand_mode(src);
        let payload = match src {
            Operand::Immediate(val)        => *val,
            Operand::IndirectAddress(addr) => *addr,
            Operand::Label(name)           => *symbols.get(name).expect(&format!("Undefined label: {}", name)),
            _ => unreachable!()
        };
        let reg = match dst {
            Operand::Register(r) => *r as u16,
            _ => unreachable!()
        };
        vec![(opcode << 11) | (mode << 9) | (reg << 5), payload]
    } else if operand_is_wide(dst) {
        let mode = operand_mode(dst);
        let payload = match dst {
            Operand::Immediate(val)        => *val,
            Operand::IndirectAddress(addr) => *addr,
            Operand::Label(name)           => *symbols.get(name).expect(&format!("Undefined label: {}", name)),
            _ => unreachable!()
        };
        let reg = match src {
            Operand::Register(r) => *r as u16,
            _ => unreachable!()
        };
        vec![(opcode << 11) | (mode << 9) | (reg << 5), payload]
    } else {
        let reg_s = match src {
            Operand::Register(r) => *r as u16,
            _ => unreachable!()
        };
        let reg_d = match dst {
            Operand::Register(r)         => *r as u16,
            Operand::IndirectRegister(r) => *r as u16,
            _ => unreachable!()
        };
        let mode = if matches!(dst, Operand::IndirectRegister(_)) {
            operand_mode(dst)
        } else {
            operand_mode(src)
        };
        vec![(opcode << 11) | (mode << 9) | (reg_s << 5) | (reg_d << 1)]
    }
}
