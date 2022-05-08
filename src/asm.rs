pub type Immediate = i64;

pub enum Register {
    Ax,
    Bx,
    Cx,
    Dx,
    Bp,
    Si,
    Di,
    Sp,
}

pub struct Address {
    base: Register,
    /// We use `i32` because offsets are not that large anyway.
    offset: i32,
}

pub enum InstructionKind {
    LoadImm { src: Immediate, dst: Register },
    LoadAddr { src: Address, dst: Register },
    Store { src: Register, dst: Address },
    Push(Register),
    Pop(Register),
    Add { src: Register, dst: Register },
    AddImm { src: Immediate, dst: Register },
    Jump(Address),
    JumpLez { addr: Address, reg: Register },
    Return,
    Call(Register),
}
