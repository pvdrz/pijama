pub struct Instruction {
    pub label: Option<Label>,
    pub kind: InstructionKind,
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

pub struct Address {
    pub base: BaseAddr,
    pub offset: i32,
}

pub enum BaseAddr {
    Ind(Register),
    Lab(Label),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(pub usize);
