use self::sib::Scale;

mod mod_rm;
mod sib;

pub type Immediate = i64;

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
    pub const ALL: &'static [Self] = &[
        Self::Ax,
        Self::Cx,
        Self::Dx,
        Self::Bx,
        Self::Sp,
        Self::Bp,
        Self::Si,
        Self::Di,
    ];

    const fn as_rd(self, opcode: u8) -> u8 {
        opcode + self as u8
    }
}

pub struct Address {
    pub base: Register,
    pub offset: i32,
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
            InstructionKind::Push(_) => todo!(),
            InstructionKind::Pop(_) => todo!(),
            InstructionKind::Add { src, dst } => todo!(),
            InstructionKind::AddImm { src, dst } => todo!(),
            InstructionKind::Jump(_) => todo!(),
            InstructionKind::JumpLez { addr, reg } => todo!(),
            InstructionKind::Return => todo!(),
            InstructionKind::Call(_) => todo!(),
        }
    }

    fn assemble_load_imm(&mut self, src: i64, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = dst.as_rd(0xb8);
        let io = src.to_le_bytes();

        self.buf.extend_from_slice(&[rex_prefix, opcode]);
        self.buf.extend_from_slice(&io);
    }

    fn assemble_load_addr(&mut self, src: Address, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x8b;
        let mod_rm = mod_rm::ModRmBuilder::new()
            .displacement()
            .reg(src.base as u8)
            .rm(dst as u8)
            .build();

        if let Register::Sp = dst {
            let sib = sib::SibBuilder::new()
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

    fn assemble_store(&mut self, src: Register, dst: Address) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x89;
        let mod_rm = mod_rm::ModRmBuilder::new()
            .displacement()
            .reg(dst.base as u8)
            .rm(src as u8)
            .build();

        if let Register::Sp = src {
            let sib = sib::SibBuilder::new()
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

    pub fn emit_code(self) -> Vec<u8> {
        self.buf
    }
}
