use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Operand {
    Immediate(u16),
    Register(u8),
    IndirectRegister(u8),
    IndirectAddress(u16),
    Label(String),
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Interrupt {
    Keyboard           = 0,
    Timer              = 1,
    IllegalInstruction = 2,
    DivideByZero       = 3,
    StackOverflow      = 4,
    User0              = 5,
    User1              = 6,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Halt,
    Ret,
    Reti,
    Clz,
    Sez,
    Clc,
    Sec,
    Clv,
    Sev,
    Cln,
    Sen,

    Jmp(Operand),
    Push(Operand),
    Pop(Operand),
    Not(Operand),
    Je(Operand),
    Jne(Operand),
    Jgt(Operand),
    Jge(Operand),
    Jlt(Operand),
    Jle(Operand),
    Ja(Operand),
    Jae(Operand),
    Jb(Operand),
    Jbe(Operand),
    Call(Operand),
    Trap(Operand),
    Getsp(Operand),
    Setsp(Operand),

    Add(Operand, Operand),
    Move(Operand, Operand),
    Sub(Operand, Operand),
    Mul(Operand, Operand),
    Div(Operand, Operand),
    Shl(Operand, Operand),
    Shr(Operand, Operand),
    And(Operand, Operand),
    Or(Operand, Operand),
    Cmp(Operand, Operand),
    Load(Operand, Operand),
    Store(Operand, Operand),
    Mvint(Interrupt, Operand)
}

#[derive(Debug, PartialEq)]
pub enum ParsedLine {
    Label(String),
    Instruction(Instruction),
    LabeledInstruction(String, Instruction),
    Data(Vec<u8>),
    LabeledData(String, Vec<u8>)
}

// Ok format 0: 16 bits everytime
// Format 1: 16 bits for register mode, 32 bits for address/immediate mode
// Format 2: 16 bits for reg/reg mode, 32 bits for reg/address or reg/immediate mode

pub fn operand_is_wide(operand: &Operand) -> bool {
    match operand {
        Operand::Immediate(_) => true,
        Operand::IndirectAddress(_) => true,
        Operand::IndirectRegister(_) => false,
        Operand::Register(_) => false,
        Operand::Label(_) => true,
    }
}

pub fn instruction_size(instruction: &Instruction) -> u16 {
    match instruction {
        // Format 0 — always 16-bit
        Instruction::Halt => 2,
        Instruction::Ret => 2,
        Instruction::Reti => 2,
        Instruction::Clz => 2,
        Instruction::Sez => 2,
        Instruction::Clc => 2,
        Instruction::Sec => 2,
        Instruction::Clv => 2,
        Instruction::Sev => 2,
        Instruction::Cln => 2,
        Instruction::Sen => 2,

        // Format 1 — depends on the single operand
        Instruction::Jmp(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Push(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Pop(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Not(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Je(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Jne(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Jgt(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Jge(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Jlt(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Jle(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Ja(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Jae(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Jb(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Jbe(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Call(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Trap(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Getsp(op) => if operand_is_wide(op) { 4 } else { 2 },
        Instruction::Setsp(op) => if operand_is_wide(op) { 4 } else { 2 },



        Instruction::Add(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Move(_, _) => 2,
        Instruction::Sub(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Mul(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Div(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Shl(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Shr(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::And(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Or(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Cmp(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Load(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Store(src, dest) => if operand_is_wide(src) || operand_is_wide(dest) { 4 } else { 2 },
        Instruction::Mvint(_, _) => 4
    }
}

pub fn operand_mode(operand: &Operand) -> u16 {
    match operand {
        Operand::Immediate(_) => 0b00,
        Operand::Register(_) => 0b01,
        Operand::IndirectRegister(_) => 0b10,
        Operand::IndirectAddress(_) => 0b11,
        Operand::Label(_) => 0b00,
    }
}

impl fmt::Display for Interrupt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Interrupt::Keyboard           => write!(f, "keyboard"),
            Interrupt::Timer              => write!(f, "timer"),
            Interrupt::IllegalInstruction => write!(f, "ill-instr"),
            Interrupt::DivideByZero       => write!(f, "div-zero"),
            Interrupt::StackOverflow      => write!(f, "stack-ovf"),
            Interrupt::User0              => write!(f, "user0"),
            Interrupt::User1              => write!(f, "user1"),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Immediate(v)        => write!(f, "#0x{:04X}", v),
            Operand::Register(r)         => write!(f, "r{}", r),
            Operand::IndirectRegister(r) => write!(f, "[r{}]", r),
            Operand::IndirectAddress(a)  => write!(f, "[0x{:04X}]", a),
            Operand::Label(name)         => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Halt  => write!(f, "halt"),
            Instruction::Ret   => write!(f, "ret"),
            Instruction::Reti  => write!(f, "reti"),
            Instruction::Sez   => write!(f, "sez"),
            Instruction::Clz   => write!(f, "clz"),
            Instruction::Sec   => write!(f, "sec"),
            Instruction::Clc   => write!(f, "clc"),
            Instruction::Sev   => write!(f, "sev"),
            Instruction::Clv   => write!(f, "clv"),
            Instruction::Sen   => write!(f, "sen"),
            Instruction::Cln   => write!(f, "cln"),
            Instruction::Push(op)    => write!(f, "push {}", op),
            Instruction::Pop(op)     => write!(f, "pop {}", op),
            Instruction::Not(op)     => write!(f, "not {}", op),
            Instruction::Jmp(op)     => write!(f, "jmp {}", op),
            Instruction::Je(op)      => write!(f, "je {}", op),
            Instruction::Jne(op)     => write!(f, "jne {}", op),
            Instruction::Jgt(op)     => write!(f, "jgt {}", op),
            Instruction::Jge(op)     => write!(f, "jge {}", op),
            Instruction::Jlt(op)     => write!(f, "jlt {}", op),
            Instruction::Jle(op)     => write!(f, "jle {}", op),
            Instruction::Ja(op)      => write!(f, "ja {}", op),
            Instruction::Jae(op)     => write!(f, "jae {}", op),
            Instruction::Jb(op)      => write!(f, "jb {}", op),
            Instruction::Jbe(op)     => write!(f, "jbe {}", op),
            Instruction::Call(op)    => write!(f, "call {}", op),
            Instruction::Trap(op)    => write!(f, "trap {}", op),
            Instruction::Getsp(op)   => write!(f, "getsp {}", op),
            Instruction::Setsp(op)   => write!(f, "setsp {}", op),
            Instruction::Load(s, d)  => write!(f, "load {}, {}", s, d),
            Instruction::Store(s, d) => write!(f, "store {}, {}", s, d),
            Instruction::Move(s, d)  => write!(f, "move {}, {}", s, d),
            Instruction::Add(s, d)   => write!(f, "add {}, {}", s, d),
            Instruction::Sub(s, d)   => write!(f, "sub {}, {}", s, d),
            Instruction::Mul(s, d)   => write!(f, "mul {}, {}", s, d),
            Instruction::Div(s, d)   => write!(f, "div {}, {}", s, d),
            Instruction::Shl(s, d)   => write!(f, "shl {}, {}", s, d),
            Instruction::Shr(s, d)   => write!(f, "shr {}, {}", s, d),
            Instruction::And(s, d)   => write!(f, "and {}, {}", s, d),
            Instruction::Or(s, d)    => write!(f, "or {}, {}", s, d),
            Instruction::Cmp(s, d)   => write!(f, "cmp {}, {}", s, d),
            Instruction::Mvint(i, h) => write!(f, "mvint {}, {}", i, h),
        }
    }
}
