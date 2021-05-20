use std::fmt;

pub struct Instruction {
    pub label: Option<Label>,
    pub kind: InstructionKind,
}
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.label {
            Some(lbl) => write!(f, "{}: {}", lbl, self.kind),
            None => write!(f, "       {}", self.kind),
        }
    }
}
/// Instructions for the Pijama Abstract Machine (PAM).
pub enum InstructionKind {
    /// Load the contents of an address into a register.
    Load { src: Address, dst: Register },
    /// Load an immediate into a register.
    LoadImm { src: i64, dst: Register },
    /// Store the contents of a register into an address.
    Store { src: Register, dst: Address },
    /// Push the contents of a register into the stack.
    Push(Register),
    /// Pop the a value from the stack into a register.
    Pop(Register),
    /// Add the contents of one register to another.
    Add { src: Register, dst: Register },
    /// Add the an immediate to a register.
    AddImm { src: i64, dst: Register },
    /// Jump to an address if the contents of a register are less or equal than zero.
    JumpLez { reg: Register, addr: Address },
    /// Jump inconditionally to an address.
    Jump(Address),
    /// Return from the current call.
    Return,
    /// Call a function from the address stored in a register.
    Call(Register),
}

impl fmt::Display for InstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionKind::Load { src, dst } => write!(f, "load  {},{}", src, dst),
            InstructionKind::LoadImm { src, dst } => write!(f, "loadi {:#x},{}", src, dst),
            InstructionKind::Store { src, dst } => write!(f, "store {},{}", src, dst),
            InstructionKind::Push(src) => write!(f, "push  {}", src),
            InstructionKind::Pop(dst) => write!(f, "pop   {}", dst),
            InstructionKind::Add { src, dst } => write!(f, "add   {},{}", src, dst),
            InstructionKind::AddImm { src, dst } => write!(f, "addi  {:#x},{}", src, dst),
            InstructionKind::JumpLez { reg, addr } => write!(f, "jlez  {},{}", reg, addr),
            InstructionKind::Jump(addr) => write!(f, "jmp   {}", addr),
            InstructionKind::Return => write!(f, "ret"),
            InstructionKind::Call(reg) => write!(f, "call  {}", reg),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Register {
    Ax = 0x0,
    Cx = 0x1,
    Dx = 0x2,
    Bx = 0x3,
    Sp = 0x4,
    Bp = 0x5,
    Si = 0x6,
    Di = 0x7,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ax => "rax",
            Self::Cx => "rcx",
            Self::Dx => "rdx",
            Self::Bx => "rbx",
            Self::Sp => "rsp",
            Self::Bp => "rbp",
            Self::Si => "rsi",
            Self::Di => "rdi",
        }
        .fmt(f)
    }
}

pub struct Address {
    pub base: BaseAddr,
    pub offset: i32,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}({})", self.offset, self.base)
    }
}

pub enum BaseAddr {
    Ind(Register),
    Lab(Label),
}

impl fmt::Display for BaseAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseAddr::Ind(reg) => reg.fmt(f),
            BaseAddr::Lab(lbl) => lbl.fmt(f),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(pub usize);

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lb_{:02x}", self.0)
    }
}
