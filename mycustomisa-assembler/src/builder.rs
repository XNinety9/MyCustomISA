use crate::common::{ParsedLine, instruction_size};
use std::collections::HashMap;

pub fn build(parsed_lines: &[ParsedLine]) -> HashMap<String, u16> {
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut address: u16 = 0;

    for line in parsed_lines {
        match line {
            ParsedLine::Label(name) => {
                labels.insert(name.clone(), address);
            }
            ParsedLine::Instruction(instr) => {
                address += instruction_size(instr);
            }
            ParsedLine::LabeledInstruction(name, instr) => {
                labels.insert(name.clone(), address);
                address += instruction_size(instr);
            }
            ParsedLine::Data(data) => {
                address += data.len() as u16;
            }
            ParsedLine::LabeledData(name, data) => {
                labels.insert(name.clone(), address);
                address += data.len() as u16;
            }
        }
    }

    labels
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Instruction, Operand};

    // ── helpers ──────────────────────────────────────────────────────

    fn instr(i: Instruction) -> ParsedLine { ParsedLine::Instruction(i) }
    fn label(s: &str) -> ParsedLine { ParsedLine::Label(s.to_string()) }
    fn labeled_instr(s: &str, i: Instruction) -> ParsedLine { ParsedLine::LabeledInstruction(s.to_string(), i) }
    fn data(bytes: Vec<u8>) -> ParsedLine { ParsedLine::Data(bytes) }
    fn labeled_data(s: &str, bytes: Vec<u8>) -> ParsedLine { ParsedLine::LabeledData(s.to_string(), bytes) }

    // ── basic cases ──────────────────────────────────────────────────

    #[test]
    fn test_empty_input() {
        assert!(build(&[]).is_empty());
    }

    #[test]
    fn test_instructions_only_produce_no_labels() {
        let lines = vec![instr(Instruction::Halt), instr(Instruction::Ret)];
        assert!(build(&lines).is_empty());
    }

    #[test]
    fn test_label_at_origin() {
        let lines = vec![label("main")];
        let result = build(&lines);
        assert_eq!(result["main"], 0);
    }

    // ── address advancement ──────────────────────────────────────────

    #[test]
    fn test_format0_advances_2_bytes() {
        // label after one HALT (2 bytes)
        let lines = vec![instr(Instruction::Halt), label("after")];
        assert_eq!(build(&lines)["after"], 2);
    }

    #[test]
    fn test_format1_narrow_advances_2_bytes() {
        // JMP R0 — register operand, narrow
        let lines = vec![instr(Instruction::Jmp(Operand::Register(0))), label("after")];
        assert_eq!(build(&lines)["after"], 2);
    }

    #[test]
    fn test_format1_wide_advances_4_bytes() {
        // JMP 0x1000 — immediate operand, wide
        let lines = vec![instr(Instruction::Jmp(Operand::Immediate(0x1000))), label("after")];
        assert_eq!(build(&lines)["after"], 4);
    }

    #[test]
    fn test_format2_narrow_advances_2_bytes() {
        // ADD R0, R1 — both registers, narrow
        let lines = vec![
            instr(Instruction::Add(Operand::Register(0), Operand::Register(1))),
            label("after"),
        ];
        assert_eq!(build(&lines)["after"], 2);
    }

    #[test]
    fn test_format2_wide_advances_4_bytes() {
        // LOAD 42, R0 — immediate src, wide
        let lines = vec![
            instr(Instruction::Load(Operand::Immediate(42), Operand::Register(0))),
            label("after"),
        ];
        assert_eq!(build(&lines)["after"], 4);
    }

    // ── labeled instructions ─────────────────────────────────────────

    #[test]
    fn test_labeled_instruction_records_label_before_advance() {
        let lines = vec![labeled_instr("entry", Instruction::Halt)];
        assert_eq!(build(&lines)["entry"], 0);
    }

    #[test]
    fn test_labeled_instruction_advances_address() {
        let lines = vec![labeled_instr("a", Instruction::Halt), label("b")];
        let result = build(&lines);
        assert_eq!(result["a"], 0);
        assert_eq!(result["b"], 2);
    }

    // ── data ─────────────────────────────────────────────────────────

    #[test]
    fn test_data_advances_by_byte_count() {
        let lines = vec![data(vec![0u8; 6]), label("after")];
        assert_eq!(build(&lines)["after"], 6);
    }

    #[test]
    fn test_labeled_data_records_label_before_advance() {
        let lines = vec![labeled_data("buf", vec![0u8; 4])];
        assert_eq!(build(&lines)["buf"], 0);
    }

    #[test]
    fn test_labeled_data_advances_address() {
        let lines = vec![labeled_data("buf", vec![0u8; 4]), label("after")];
        let result = build(&lines);
        assert_eq!(result["buf"], 0);
        assert_eq!(result["after"], 4);
    }

    // ── full program ─────────────────────────────────────────────────

    #[test]
    fn test_full_program() {
        let lines = vec![
            labeled_instr("main", Instruction::Load(Operand::Immediate(0), Operand::Register(0))), // 4 bytes (wide)
            labeled_instr("loop", Instruction::Add(Operand::Register(0), Operand::Register(1))),   // 2 bytes (narrow)
            instr(Instruction::Jmp(Operand::Label("loop".to_string()))),                           // 4 bytes (label → narrow)
            labeled_data("msg", vec![0x48, 0x69, 0x00]),                                           // 3 bytes
            label("end"),
        ];
        let result = build(&lines);
        assert_eq!(result["main"], 0);
        assert_eq!(result["loop"], 4);
        assert_eq!(result["msg"], 10);
        assert_eq!(result["end"], 13);
    }
}