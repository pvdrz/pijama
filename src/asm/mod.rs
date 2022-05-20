use self::{
    mod_rm::ModRmBuilder,
    sib::{Scale, SibBuilder},
};

mod mod_rm;
mod sib;

pub type Imm32 = i32;
pub type Imm64 = i64;

#[repr(u8)]
#[derive(Clone, Copy)]
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

impl Register {
    const fn as_rd(self, opcode: u8) -> u8 {
        opcode + self as u8
    }
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
    Jump(Address<()>),
    JumpEq {
        reg1: Register,
        reg2: Register,
        target: Imm32,
    },
    JumpLt {
        reg1: Register,
        reg2: Register,
        target: Imm32,
    },
    JumpGt {
        reg1: Register,
        reg2: Register,
        target: Imm32,
    },
    Return,
    Call(Register),
}

struct RexPrefix;

impl RexPrefix {
    const fn new(w_bit: bool, r_bit: bool, b_bit: bool) -> u8 {
        let mut prefix = 0b01000000;

        if w_bit {
            prefix |= 0b1000;
        }
        if r_bit {
            prefix |= 0b100;
        }
        if b_bit {
            prefix |= 0b1;
        }

        prefix
    }
}

#[derive(Default)]
pub struct Assembler {
    buf: Vec<u8>,
}

impl Assembler {
    pub fn assemble_instruction(&mut self, kind: InstructionKind) {
        match kind {
            InstructionKind::LoadImm { src, dst } => self.assemble_load_imm(src, dst),
            InstructionKind::LoadAddr { src, dst } => self.assemble_load_addr(src, dst),
            InstructionKind::Store { src, dst } => self.assemble_store(src, dst),
            InstructionKind::Push(reg) => self.assemble_push(reg),
            InstructionKind::Pop(reg) => self.assemble_pop(reg),
            InstructionKind::Add { src, dst } => self.assemble_add(src, dst),
            InstructionKind::AddImm { src, dst } => self.assemble_add_imm(src, dst),
            InstructionKind::Jump(addr) => self.assemmble_jump(addr),
            InstructionKind::JumpEq { reg1, reg2, target } => {
                self.assemble_conditional_jump::<0x84>(reg1, reg2, target)
            }
            InstructionKind::JumpLt { reg1, reg2, target } => {
                self.assemble_conditional_jump::<0x8C>(reg1, reg2, target)
            }
            InstructionKind::JumpGt { reg1, reg2, target } => {
                self.assemble_conditional_jump::<0x8F>(reg1, reg2, target)
            }
            InstructionKind::Return => self.assemble_return(),
            InstructionKind::Call(target) => self.assemble_call(target),
        }
    }

    fn assemble_load_imm(&mut self, src: i64, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = dst.as_rd(0xb8);
        let io = src.to_le_bytes();

        self.buf.extend_from_slice(&[rex_prefix, opcode]);
        self.buf.extend_from_slice(&io);
    }

    fn assemble_load_addr(&mut self, src: Address<Imm32>, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x8b;
        let mod_rm = ModRmBuilder::new()
            .displacement()
            .reg(src.base as u8)
            .rm(dst as u8)
            .build();

        if let Register::Sp = dst {
            let sib = SibBuilder::new()
                .scale(Scale::One)
                .index(dst)
                .base(dst)
                .build();

            self.buf
                .extend_from_slice(&[rex_prefix, opcode, mod_rm, sib]);
        } else {
            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        }

        self.buf.extend_from_slice(&src.offset.to_le_bytes());
    }

    fn assemble_store(&mut self, src: Register, dst: Address<Imm32>) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x89;
        let mod_rm = ModRmBuilder::new()
            .displacement()
            .reg(dst.base as u8)
            .rm(src as u8)
            .build();

        if let Register::Sp = src {
            let sib = SibBuilder::new()
                .scale(Scale::One)
                .index(src)
                .base(src)
                .build();

            self.buf
                .extend_from_slice(&[rex_prefix, opcode, mod_rm, sib]);
        } else {
            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        }

        self.buf.extend_from_slice(&dst.offset.to_le_bytes());
    }

    fn assemble_push(&mut self, reg: Register) {
        let opcode = 0x50 + reg as u8;

        self.buf.extend_from_slice(&[opcode]);
    }

    fn assemble_pop(&mut self, reg: Register) {
        let opcode = 0x58 + reg as u8;

        self.buf.extend_from_slice(&[opcode]);
    }

    fn assemble_add(&mut self, src: Register, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x01;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(src as u8)
            .rm(dst as u8)
            .build();

        self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
    }

    fn assemble_add_imm(&mut self, src: i32, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);

        if let Register::Ax = dst {
            let opcode = 0x05;

            self.buf.extend_from_slice(&[rex_prefix, opcode]);
        } else {
            let opcode = 0x81;
            let mod_rm = ModRmBuilder::new().direct().reg(0x0).rm(dst as u8).build();

            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        }

        self.buf.extend_from_slice(&src.to_le_bytes());
    }

    fn assemmble_jump(&mut self, addr: Address<()>) {
        let opcode = 0xFF;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(0x4)
            .rm(addr.base as u8)
            .build();

        self.buf.extend_from_slice(&[opcode, mod_rm]);
    }

    fn assemble_jump_if_zero(&mut self, mut target: Imm32, scr: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x83;
        let mod_rm = ModRmBuilder::new().direct().reg(0x7).rm(scr as u8).build();

        // apparently we need to make the target relative to the location of the instruction
        // pointer after reading the instruction. This instruction always takes 10 bytes.
        target -= self.buf.len() as i32 + 0xa;

        // cmp scr,0x0
        self.buf
            .extend_from_slice(&[rex_prefix, opcode, mod_rm, 0x0]);
        // je target
        self.buf.extend_from_slice(&[0x0F, 0x84]);
        self.buf.extend_from_slice(&target.to_le_bytes());
    }

    fn assemble_conditional_jump<const OPCODE: u8>(
        &mut self,
        reg1: Register,
        reg2: Register,
        mut target: i32,
    ) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x39;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(reg1 as u8)
            .rm(reg2 as u8)
            .build();

        // apparently we need to make the target relative to the location of the instruction
        // pointer after reading the instruction. This instruction always takes 9 bytes.
        target -= self.buf.len() as i32 + 0x9;

        // cmp reg2,reg1
        self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        // je target
        self.buf.extend_from_slice(&[0x0F, OPCODE]);
        self.buf.extend_from_slice(&target.to_le_bytes());
    }

    fn assemble_return(&mut self) {
        let opcode = 0xC3;

        self.buf.extend_from_slice(&[opcode]);
    }

    fn assemble_call(&mut self, target: Register) {
        let opcode = 0xFF;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(0x2)
            .rm(target as u8)
            .build();

        self.buf.extend_from_slice(&[opcode, mod_rm]);
    }

    pub fn emit_code(self) -> Vec<u8> {
        self.buf
    }
}

#[macro_export]
macro_rules! reg {
    (rax) => {
        $crate::asm::Register::Ax
    };
    (rcx) => {
        $crate::asm::Register::Cx
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
macro_rules! code {
    (loadi {$imm64:expr},{$($reg:tt)+}) => {
        $crate::asm::InstructionKind::LoadImm {
            src: $imm64,
            dst: $crate::reg!($($reg)+),
        };
    };
    (loada {$($addr:tt)+}+{$imm32:expr},{$($reg:tt)+}) => {
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
        };
    };   (jmp {$($addr:tt)*}) => {
        $crate::asm::InstructionKind::Jump($crate::asm::Address {
            base: $crate::reg!($($addr)*),
            offset: (),
        })
    };
    (je {$($reg1:tt)*},{$($reg2:tt)*},{$imm32:expr}) => {
        $crate::asm::InstructionKind::JumpEq {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: $imm32,
        }
    };
    (jl {$($reg1:tt)*},{$($reg2:tt)*},{$imm32:expr}) => {
        $crate::asm::InstructionKind::JumpLt {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: $imm32,
        }
    };
    (jg {$($reg1:tt)*},{$($reg2:tt)*},{$imm32:expr}) => {
        $crate::asm::InstructionKind::JumpGt {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: $imm32,
        }
    };
    (ret) => {
        $crate::asm::InstructionKind::Return
    };
    (call {$($reg:tt)*}) => {
        $crate::asm::InstructionKind::Call($crate::reg!($($reg)*))
    };
}
