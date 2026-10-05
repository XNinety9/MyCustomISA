use crate::common::ParsedLine;
use crate::common::{Instruction, Operand, Interrupt};

pub fn parse(lines: &[String]) -> Vec<ParsedLine> {
    lines.iter()
        .filter_map(|line| parse_line(line))
        .collect()
}

fn parse_line(line: &str) -> Option<ParsedLine> {
    let l = line.split(';').next().unwrap_or("").trim();
    if l.is_empty() {
        return None;
    }

    match l.split_once(':') {
        None => Some(ParsedLine::Instruction(parse_instruction(l))),
        Some((label, rest)) => {
            let label = label.trim();
            let rest = rest.trim();

            if rest.is_empty() {
                Some(ParsedLine::Label(label.to_string()))
            } else {
                Some(ParsedLine::LabeledInstruction(label.to_string(), parse_instruction(rest)))
            }
        }
    }
}

fn parse_instruction(instruction: &str) -> Instruction {
    let tokens: Vec<&str> = instruction
        .split(|c| c == ' ' || c == ',')
        .filter(|s| !s.is_empty())
        .collect();

    match tokens[0] {
        // Format 0
        "HALT" => Instruction::Halt,
        "RET"  => Instruction::Ret,
        "RETI" => Instruction::Reti,
        "CLZ"  => Instruction::Clz,
        "SEZ"  => Instruction::Sez,
        "CLC"  => Instruction::Clc,
        "SEC"  => Instruction::Sec,
        "CLV"  => Instruction::Clv,
        "SEV"  => Instruction::Sev,
        "CLN"  => Instruction::Cln,
        "SEN"  => Instruction::Sen,
        // Format 1
        "JMP" => Instruction::Jmp(
            parse_operand(tokens.get(1).expect("JMP requires 1 operand"))
        ),
        "JE" => Instruction::Je(
            parse_operand(tokens.get(1).expect("JE requires 1 operand"))
        ),
        "JNE" => Instruction::Jne(
            parse_operand(tokens.get(1).expect("JNE requires 1 operand"))
        ),
        "JGT" => Instruction::Jgt(
            parse_operand(tokens.get(1).expect("JGT requires 1 operand"))
        ),
        "JGE" => Instruction::Jge(
            parse_operand(tokens.get(1).expect("JGE requires 1 operand"))
        ),
        "JLT" => Instruction::Jlt(
            parse_operand(tokens.get(1).expect("JLT requires 1 operand"))
        ),
        "JLE" => Instruction::Jle(
            parse_operand(tokens.get(1).expect("JLE requires 1 operand"))
        ),
        "JA" => Instruction::Ja(
            parse_operand(tokens.get(1).expect("JA requires 1 operand"))
        ),
        "JAE" => Instruction::Jae(
            parse_operand(tokens.get(1).expect("JAE requires 1 operand"))
        ),
        "JB" => Instruction::Jb(
            parse_operand(tokens.get(1).expect("JB requires 1 operand"))
        ),
        "JBE" => Instruction::Jbe(
            parse_operand(tokens.get(1).expect("JBE requires 1 operand"))
        ),
        "PUSH" => Instruction::Push(
            parse_operand(tokens.get(1).expect("PUSH requires 1 operand"))
        ),
        "POP" => Instruction::Pop(
            parse_operand(tokens.get(1).expect("POP requires 1 operand"))
        ),
        "NOT" => Instruction::Not(
            parse_operand(tokens.get(1).expect("NOT requires 1 operand"))
        ),
        "CALL" => Instruction::Call(
            parse_operand(tokens.get(1).expect("CALL requires 1 operand"))
        ),
        "TRAP" => Instruction::Trap(
            parse_operand(tokens.get(1).expect("TRAP requires 1 operand"))
        ),
        "GETSP" => Instruction::Getsp(
            parse_operand(tokens.get(1).expect("GETSP requires 1 operand"))
        ),
        "SETSP" => Instruction::Setsp(
            parse_operand(tokens.get(1).expect("SETSP requires 1 operand"))
        ),

        // Format 2
        "ADD" => Instruction::Add(
            parse_operand(tokens.get(1).expect("ADD requires 2 operands")),
            parse_operand(tokens.get(2).expect("ADD requires 2 operands")),
        ),
        "MOVE" => Instruction::Move(
            parse_operand(tokens.get(1).expect("MOVE requires 2 operands")),
            parse_operand(tokens.get(2).expect("MOVE requires 2 operands")),
        ),
        "SUB" => Instruction::Sub(
            parse_operand(tokens.get(1).expect("SUB requires 2 operands")),
            parse_operand(tokens.get(2).expect("SUB requires 2 operands")),
        ),
        "MUL" => Instruction::Mul(
            parse_operand(tokens.get(1).expect("MUL requires 2 operands")),
            parse_operand(tokens.get(2).expect("MUL requires 2 operands")),
        ),
        "DIV" => Instruction::Div(
            parse_operand(tokens.get(1).expect("DIV requires 2 operands")),
            parse_operand(tokens.get(2).expect("DIV requires 2 operands")),
        ),
        "SHL" => Instruction::Shl(
            parse_operand(tokens.get(1).expect("SHL requires 2 operands")),
            parse_operand(tokens.get(2).expect("SHL requires 2 operands")),
        ),
        "SHR" => Instruction::Shr(
            parse_operand(tokens.get(1).expect("SHR requires 2 operands")),
            parse_operand(tokens.get(2).expect("SHR requires 2 operands")),
        ),
        "AND" => Instruction::And(
            parse_operand(tokens.get(1).expect("AND requires 2 operands")),
            parse_operand(tokens.get(2).expect("AND requires 2 operands")),
        ),
        "OR" => Instruction::Or(
            parse_operand(tokens.get(1).expect("OR requires 2 operands")),
            parse_operand(tokens.get(2).expect("OR requires 2 operands")),
        ),
        "CMP" => Instruction::Cmp(
            parse_operand(tokens.get(1).expect("CMP requires 2 operands")),
            parse_operand(tokens.get(2).expect("CMP requires 2 operands")),
        ),
        "LOAD" => Instruction::Load(
            parse_operand(tokens.get(1).expect("LOAD requires 2 operands")),
            parse_operand(tokens.get(2).expect("LOAD requires 2 operands")),
        ),
        "STORE" => Instruction::Store(
            parse_operand(tokens.get(1).expect("STORE requires 2 operands")),
            parse_operand(tokens.get(2).expect("STORE requires 2 operands")),
        ),
        "MVINT" => Instruction::Mvint(
            parse_interrupt(tokens.get(1).expect("MVINT requires 2 operands")),
            parse_operand(tokens.get(2).expect("MVINT requires 2 operands")),
        ),
        _ => panic!("Unknown mnemonic: {}", tokens[0]),
    }
}

fn parse_operand(token: &str) -> Operand {
    if let Some(inner) = token.strip_prefix("[R").and_then(|s| s.strip_suffix(']')) {
        Operand::IndirectRegister(inner.parse().unwrap())
    } else if let Some(inner) = token.strip_prefix("[0x").and_then(|s| s.strip_suffix(']')) {
        Operand::IndirectAddress(u16::from_str_radix(inner, 16).unwrap())
    } else if let Some(rest) = token.strip_prefix('R').filter(|s| s.chars().all(|c| c.is_ascii_digit())) {
        Operand::Register(rest.parse().unwrap())
    } else if let Some(rest) = token.strip_prefix("0x") {
        Operand::Immediate(u16::from_str_radix(rest, 16).unwrap())
    } else if token.starts_with(|c: char| c.is_ascii_digit()) {
        Operand::Immediate(token.parse().unwrap())
    } else {
        Operand::Label(token.to_string())
    }
}

fn parse_interrupt(token: &str) -> Interrupt {
    match token {
        "KEYBOARD" | "0" => Interrupt::Keyboard,
        "TIMER" | "1" => Interrupt::Timer,
        "ILLEGAL_INSTRUCTION" | "2" => Interrupt::IllegalInstruction,
        "DIVIDE_BY_ZERO" | "3" => Interrupt::DivideByZero,
        "STACK_OVERFLOW" | "4" => Interrupt::StackOverflow,
        "USER0" | "5" => Interrupt::User0,
        "USER1" | "6" => Interrupt::User1,
        _ => panic!("Unknown interrupt: {}", token),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_operand ────────────────────────────────────────────────

    #[test]
    fn test_register() {
        assert_eq!(parse_operand("R0"), Operand::Register(0));
        assert_eq!(parse_operand("R3"), Operand::Register(3));
        assert_eq!(parse_operand("R15"), Operand::Register(15));
    }

    #[test]
    fn test_indirect_register() {
        assert_eq!(parse_operand("[R0]"), Operand::IndirectRegister(0));
        assert_eq!(parse_operand("[R7]"), Operand::IndirectRegister(7));
        assert_eq!(parse_operand("[R15]"), Operand::IndirectRegister(15));
    }

    #[test]
    fn test_indirect_address() {
        assert_eq!(parse_operand("[0xABCD]"), Operand::IndirectAddress(0xABCD));
        assert_eq!(parse_operand("[0x0000]"), Operand::IndirectAddress(0x0000));
        assert_eq!(parse_operand("[0xFFFF]"), Operand::IndirectAddress(0xFFFF));
    }

    #[test]
    fn test_immediate_hex() {
        assert_eq!(parse_operand("0xFF"), Operand::Immediate(255));
        assert_eq!(parse_operand("0xABCD"), Operand::Immediate(0xABCD));
        assert_eq!(parse_operand("0x0000"), Operand::Immediate(0));
        assert_eq!(parse_operand("0xFFFF"), Operand::Immediate(0xFFFF));
    }

    #[test]
    fn test_immediate_decimal() {
        assert_eq!(parse_operand("0"), Operand::Immediate(0));
        assert_eq!(parse_operand("42"), Operand::Immediate(42));
        assert_eq!(parse_operand("65535"), Operand::Immediate(65535));
    }

    // ── parse_interrupt ──────────────────────────────────────────────

    #[test]
    fn test_parse_interrupt_all_variants() {
        assert!(matches!(parse_interrupt("KEYBOARD"), Interrupt::Keyboard));
        assert!(matches!(parse_interrupt("TIMER"), Interrupt::Timer));
        assert!(matches!(parse_interrupt("ILLEGAL_INSTRUCTION"), Interrupt::IllegalInstruction));
        assert!(matches!(parse_interrupt("DIVIDE_BY_ZERO"), Interrupt::DivideByZero));
        assert!(matches!(parse_interrupt("STACK_OVERFLOW"), Interrupt::StackOverflow));
        assert!(matches!(parse_interrupt("USER0"), Interrupt::User0));
        assert!(matches!(parse_interrupt("USER1"), Interrupt::User1));
    }

    #[test]
    #[should_panic(expected = "Unknown interrupt")]
    fn test_parse_interrupt_unknown() {
        parse_interrupt("UNKNOWN");
    }

    // ── parse_line: empty / comment lines ───────────────────────────

    #[test]
    fn test_empty_line() {
        assert_eq!(parse_line(""), None);
    }

    #[test]
    fn test_whitespace_only() {
        assert_eq!(parse_line("   "), None);
    }

    #[test]
    fn test_comment_only() {
        assert_eq!(parse_line("; this is a comment"), None);
    }

    #[test]
    fn test_comment_with_spaces() {
        assert_eq!(parse_line("   ; indented comment"), None);
    }

    // ── parse_line: labels ───────────────────────────────────────────

    #[test]
    fn test_label_only() {
        assert!(matches!(parse_line("main:"), Some(ParsedLine::Label(l)) if l == "main"));
    }

    #[test]
    fn test_label_with_spaces() {
        assert!(matches!(parse_line("  main:  "), Some(ParsedLine::Label(l)) if l == "main"));
    }

    #[test]
    fn test_label_with_comment() {
        assert!(matches!(parse_line("main: ; start here"), Some(ParsedLine::Label(l)) if l == "main"));
    }

    // ── parse_line: instructions ─────────────────────────────────────

    #[test]
    fn test_instruction_halt() {
        assert!(matches!(parse_line("HALT"), Some(ParsedLine::Instruction(Instruction::Halt))));
    }

    #[test]
    fn test_instruction_with_comment() {
        assert!(matches!(
            parse_line("HALT ; stop here"),
            Some(ParsedLine::Instruction(Instruction::Halt))
        ));
    }

    #[test]
    fn test_instruction_indented() {
        assert!(matches!(
            parse_line("    HALT"),
            Some(ParsedLine::Instruction(Instruction::Halt))
        ));
    }

    #[test]
    fn test_instruction_one_operand() {
        assert!(matches!(
            parse_line("JMP 0x1234"),
            Some(ParsedLine::Instruction(Instruction::Jmp(Operand::Immediate(0x1234))))
        ));
    }

    #[test]
    fn test_instruction_two_operands() {
        assert!(matches!(
            parse_line("ADD R1, R3"),
            Some(ParsedLine::Instruction(Instruction::Add(
                Operand::Register(1),
                Operand::Register(3)
            )))
        ));
    }

    #[test]
    fn test_instruction_load_immediate() {
        assert!(matches!(
            parse_line("LOAD 42, R0"),
            Some(ParsedLine::Instruction(Instruction::Load(
                Operand::Immediate(42),
                Operand::Register(0)
            )))
        ));
    }

    #[test]
    fn test_instruction_store_indirect() {
        assert!(matches!(
            parse_line("STORE R1, [0xABCD]"),
            Some(ParsedLine::Instruction(Instruction::Store(
                Operand::Register(1),
                Operand::IndirectAddress(0xABCD)
            )))
        ));
    }

    // ── parse_line: labeled instructions ────────────────────────────

    #[test]
    fn test_labeled_instruction() {
        assert!(matches!(
            parse_line("main: HALT"),
            Some(ParsedLine::LabeledInstruction(l, Instruction::Halt)) if l == "main"
        ));
    }

    #[test]
    fn test_labeled_instruction_with_operands() {
        assert!(matches!(
            parse_line("loop: JMP 0x0010"),
            Some(ParsedLine::LabeledInstruction(l, Instruction::Jmp(Operand::Immediate(0x0010)))) if l == "loop"
        ));
    }

    #[test]
    fn test_labeled_instruction_with_comment() {
        assert!(matches!(
            parse_line("main: HALT ; entry point"),
            Some(ParsedLine::LabeledInstruction(l, Instruction::Halt)) if l == "main"
        ));
    }

    // ── parse_line: unknown mnemonic ─────────────────────────────────

    #[test]
    #[should_panic(expected = "Unknown mnemonic")]
    fn test_unknown_mnemonic() {
        parse_line("FOOBAR R0");
    }

    // ── parse: full pipeline ─────────────────────────────────────────

    #[test]
    fn test_parse_filters_empty_lines() {
        let lines = vec![
            "".to_string(),
            "; comment".to_string(),
            "HALT".to_string(),
        ];
        let result = parse(&lines);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_parse_full_program() {
        let lines = vec![
            "; simple program".to_string(),
            "main: LOAD 42, R0".to_string(),
            "loop:".to_string(),
            "    ADD R0, R1  ; increment".to_string(),
            "    JMP loop".to_string(),
            "    HALT".to_string(),
        ];
        let result = parse(&lines);
        assert_eq!(result.len(), 5);
        assert!(matches!(&result[0], ParsedLine::LabeledInstruction(l, _) if l == "main"));
        assert!(matches!(&result[1], ParsedLine::Label(l) if l == "loop"));
        assert!(matches!(&result[2], ParsedLine::Instruction(Instruction::Add(_, _))));
        assert!(matches!(&result[3], ParsedLine::Instruction(Instruction::Jmp(_))));
        assert!(matches!(&result[4], ParsedLine::Instruction(Instruction::Halt)));
    }

    #[test]
    fn test_label_operand() {
        assert_eq!(parse_operand("main"), Operand::Label("main".to_string()));
        assert_eq!(parse_operand("loop"), Operand::Label("loop".to_string()));
    }
}