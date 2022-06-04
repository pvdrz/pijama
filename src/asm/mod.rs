mod assembler;
mod macros;

pub use assembler::Assembler;

pub type Imm32 = i32;
pub type Imm64 = i64;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Ax = 0,
    Cx = 1,
    Dx = 2,
    Bx = 3,
    Sp = 4,
    Bp = 5,
    Si = 6,
    Di = 7,
}

pub struct Address<I> {
    pub base: Register,
    pub offset: I,
}

pub enum InstructionKind {
    LoadImm {
        src: Imm64,
        dst: Register,
    },
    LoadAddr {
        src: Address<Imm32>,
        dst: Register,
    },
    Store {
        src: Register,
        dst: Address<Imm32>,
    },
    Mov {
        src: Register,
        dst: Register,
    },
    Push(Register),
    Pop(Register),
    Add {
        src: Register,
        dst: Register,
    },
    AddImm {
        src: Imm32,
        dst: Register,
    },
    SetIfLess {
        src1: Register,
        src2: Register,
        dst: Register,
    },
    Jump(Location),
    JumpIfZero {
        src: Register,
        target: Location,
    },
    Return,
    Call(Register),
}

pub enum Location {
    Imm32(Imm32),
    Label(Label),
}

impl From<Imm32> for Location {
    fn from(value: Imm32) -> Self {
        Self::Imm32(value)
    }
}

impl From<Label> for Location {
    fn from(label: Label) -> Self {
        Self::Label(label)
    }
}

#[derive(Clone, Copy)]
pub struct Label(usize);

pub struct Instruction {
    pub label: Option<Label>,
    pub kind: InstructionKind,
}
