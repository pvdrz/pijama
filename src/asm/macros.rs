#[macro_export]
macro_rules! reg {
    (rax) => {
        $crate::asm::Register::Ax
    };
    (rcx) => {
        $crate::asm::Register::Cx
    };
    (rdx) => {
        $crate::asm::Register::Dx
    };
    (rbx) => {
        $crate::asm::Register::Bx
    };
    (rsp) => {
        $crate::asm::Register::Sp
    };
    (rbp) => {
        $crate::asm::Register::Bp
    };
    (rsi) => {
        $crate::asm::Register::Si
    };
    (rdi) => {
        $crate::asm::Register::Di
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
        };
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
        $crate::asm::InstructionKind::Jump({$loc}.into())
    };
    (je {$($reg1:tt)*},{$($reg2:tt)*},{$loc:expr}) => {
        $crate::asm::InstructionKind::JumpEq {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: {$loc}.into(),
        }
    };
    (jl {$($reg1:tt)*},{$($reg2:tt)*},{$loc:expr}) => {
        $crate::asm::InstructionKind::JumpLt {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: {$loc}.into(),
        }
    };
    (jg {$($reg1:tt)*},{$($reg2:tt)*},{$loc:expr}) => {
        $crate::asm::InstructionKind::JumpGt {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: {$loc}.into(),
        }
    };
    (ret) => {
        $crate::asm::InstructionKind::Return
    };
    (call {$($reg:tt)*}) => {
        $crate::asm::InstructionKind::Call($crate::reg!($($reg)*))
    };
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
