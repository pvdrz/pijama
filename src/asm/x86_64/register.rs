#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Ax,
    Cx,
    Dx,
    Bx,
    Sp,
    Bp,
    Si,
    Di,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Register {
    pub const fn encode(self) -> u8 {
        match self {
            Register::Ax | Register::R8 => 0,
            Register::Cx | Register::R9 => 1,
            Register::Dx | Register::R10 => 2,
            Register::Bx | Register::R11 => 3,
            Register::Sp | Register::R12 => 4,
            Register::Bp | Register::R13 => 5,
            Register::Si | Register::R14 => 6,
            Register::Di | Register::R15 => 7,
        }
    }

    pub const fn needs_extension(self) -> bool {
        match self {
            Register::Ax
            | Register::Cx
            | Register::Dx
            | Register::Bx
            | Register::Sp
            | Register::Bp
            | Register::Si
            | Register::Di => false,
            Register::R8
            | Register::R9
            | Register::R10
            | Register::R11
            | Register::R12
            | Register::R13
            | Register::R14
            | Register::R15 => true,
        }
    }
}
