#[macro_export]
macro_rules! reg {
    (rax) => {
        $crate::asm::x86_64::Register::Ax
    };
    (rcx) => {
        $crate::asm::x86_64::Register::Cx
    };
    (rdx) => {
        $crate::asm::x86_64::Register::Dx
    };
    (rbx) => {
        $crate::asm::x86_64::Register::Bx
    };
    (rsp) => {
        $crate::asm::x86_64::Register::Sp
    };
    (rbp) => {
        $crate::asm::x86_64::Register::Bp
    };
    (rsi) => {
        $crate::asm::x86_64::Register::Si
    };
    (rdi) => {
        $crate::asm::x86_64::Register::Di
    };
    ($expr:expr) => {
        $expr
    };
}

#[macro_export]
macro_rules! instruction_kind {
    (loadi {$imm64:expr},{$($reg:tt)+}) => {
        $crate::asm::InstructionKind::LoadImm {
            src: $imm64,
            dst: $crate::reg!($($reg)+),
        }
    };
    (load {$($addr:tt)+}+{$imm32:expr},{$($reg:tt)+}) => {
        $crate::asm::InstructionKind::LoadAddr {
            src: $crate::asm::Address {
                base: $crate::reg!($($addr)+),
                offset: $imm32,
            },
            dst: $crate::reg!($($reg)+),
        }
    };
    (store {$($reg:tt)*},{$($addr:tt)*}+{$imm32:expr}) => {
        $crate::asm::InstructionKind::Store {
            src: $crate::reg!($($reg)*),
            dst: $crate::asm::Address {
                base: $crate::reg!($($addr)*),
                offset: $imm32,
            },
        }
    };
    (mov {$($reg1:tt)*}, {$($reg2:tt)*}) => {
        $crate::asm::InstructionKind::Mov {
            src: $crate::reg!($($reg1)*),
            dst: $crate::reg!($($reg2)*),
        }
    };
    (push {$($reg:tt)*}) => {
        $crate::asm::InstructionKind::Push($crate::reg!($($reg)*))
    };
    (pop {$($reg:tt)*}) => {
        $crate::asm::InstructionKind::Pop($crate::reg!($($reg)*))
    };
    (add {$($reg1:tt)*}, {$($reg2:tt)*}) => {
        $crate::asm::InstructionKind::Add {
            src: $crate::reg!($($reg1)*),
            dst: $crate::reg!($($reg2)*),
        }
    };
    (addi {$imm32:expr},{$($reg:tt)+}) => {
        $crate::asm::InstructionKind::AddImm {
            src: $imm32,
            dst: $crate::reg!($($reg)+),
        }
    };
    (jmp {$loc:expr}) => {
        $crate::asm::InstructionKind::Jump({$loc})
    };
    (jz {$($reg:tt)*},{$loc:expr}) => {
        $crate::asm::InstructionKind::JumpIfZero {
            src: $crate::reg!($($reg)*),
            target: {$loc},
        }
    };
    (slt {$($reg1:tt)*},{$($reg2:tt)*},{$($reg3:tt)*}) => {
        $crate::asm::InstructionKind::SetIfLess {
            src1: $crate::reg!($($reg1)*),
            src2: $crate::reg!($($reg2)*),
            dst: $crate::reg!($($reg3)*),
        }
    };
    (ret) => {
        $crate::asm::InstructionKind::Return
    };
    (call {$($reg:tt)*}) => {
        $crate::asm::InstructionKind::Call($crate::reg!($($reg)*))
    };
    (nop) => {
        $crate::asm::InstructionKind::Nop
    }
}

#[macro_export]
macro_rules! code {
    ($label:ident: $($tokens:tt)+) => {
        $crate::asm::Instruction {
            label: Some($label),
            kind: $crate::instruction_kind!($($tokens)*)
        }
    };

    ($($tokens:tt)*) => {
        $crate::asm::Instruction {
            label: None,
            kind: $crate::instruction_kind!($($tokens)*)
        }
    };
}
