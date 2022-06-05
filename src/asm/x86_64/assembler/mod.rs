mod mod_rm;
mod sib;

use std::error::Error;
use std::fmt;

use crate::asm::x86_64::register::Register;
use crate::asm::{Address, Imm32, Imm64, Instruction, InstructionKind, Instructions, Label};
use mod_rm::ModRmBuilder;
use sib::{Scale, SibBuilder};

pub fn assemble(
    instructions: Instructions<Register>,
    buf: &mut Vec<u8>,
) -> Result<(), AssemblerError> {
    let mut asm = Assembler {
        buf,
        label_locations: vec![None; instructions.labels_len],
        patches: Vec::with_capacity(instructions.labels_len),
    };

    for instruction in instructions.instructions {
        asm.assemble_instruction(instruction);
    }

    asm.finish()
}

#[derive(Debug)]
pub enum AssemblerError {
    MissingLabelLocation(Label),
}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLabelLocation(label) => {
                write!(f, "location of label {label:?} is missing")
            }
        }
    }
}

impl Error for AssemblerError {}

struct Assembler<'asm> {
    buf: &'asm mut Vec<u8>,
    label_locations: Vec<Option<usize>>,
    patches: Vec<Patch>,
}

impl<'asm> Assembler<'asm> {
    /// Adds a new patch in the current location of the instruction pointer for a [`Label`]. This
    /// means that an [`Imm32`] with the location of the label will be written at the current
    /// location of the instruction pointer when calling [`Assembler::emit_code`].
    ///
    /// This will overwrite the `4` bytes following the current instruction pointer location.
    fn add_patch(&mut self, label: Label) {
        self.patches.push(Patch {
            label,
            start: self.buf.len(),
        })
    }

    /// Assembles an instruction.:
    ///
    /// If the instruction has a label, the previous location of the label will be overwritten.
    pub fn assemble_instruction(&mut self, instruction: Instruction<Register>) {
        if let Some(label) = instruction.label {
            self.label_locations[label.0] = Some(self.buf.len());
        }

        match instruction.kind {
            InstructionKind::LoadImm { src, dst } => self.assemble_load_imm::<true>(src, dst),
            InstructionKind::LoadAddr { src, dst } => self.assemble_load_addr(src, dst),
            InstructionKind::Store { src, dst } => self.assemble_store(src, dst),
            InstructionKind::Mov { src, dst } => self.assemble_mov(src, dst),
            InstructionKind::Push(reg) => self.assemble_push(reg),
            InstructionKind::Pop(reg) => self.assemble_pop(reg),
            InstructionKind::Add { src, dst } => self.assemble_add(src, dst),
            InstructionKind::AddImm { src, dst } => self.assemble_add_imm(src, dst),
            InstructionKind::SetIfLess { src1, src2, dst } => {
                self.assemble_set_if::<0x9c>(src1, src2, dst)
            }
            InstructionKind::Jump(target) => self.assemble_jump(target),
            InstructionKind::JumpIfZero { src, target } => self.assemble_jump_if_zero(src, target),
            InstructionKind::Return => self.assemble_return(),
            InstructionKind::Call(target) => self.assemble_call(target),
            InstructionKind::Nop => {}
        }
    }

    fn assemble_load_imm<const OPTIMIZE: bool>(&mut self, src: Imm64, dst: Register) {
        if OPTIMIZE && src == 0 {
            let opcode = 0x31;
            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(dst as u8)
                .rm(dst as u8)
                .build();

            self.buf.extend_from_slice(&[opcode, mod_rm]);
        } else if let Ok(src) = Imm32::try_from(src) {
            let opcode = 0xb8 + dst as u8;
            let io = src.to_le_bytes();

            self.buf.extend_from_slice(&[opcode]);
            self.buf.extend_from_slice(&io);
        } else {
            let rex_prefix = rex(true, false, false);
            let opcode = 0xb8 + dst as u8;
            let io = src.to_le_bytes();

            self.buf.extend_from_slice(&[rex_prefix, opcode]);
            self.buf.extend_from_slice(&io);
        }
    }

    fn assemble_load_addr(&mut self, src: Address<Imm32, Register>, dst: Register) {
        let rex_prefix = rex(true, false, false);
        let opcode = 0x8b;
        let mod_rm = ModRmBuilder::new()
            .displacement()
            .reg(dst as u8)
            .rm(src.base as u8)
            .build();

        if let Register::Sp = src.base {
            let sib = SibBuilder::new()
                .scale(Scale::One)
                .index(src.base)
                .base(src.base)
                .build();

            self.buf
                .extend_from_slice(&[rex_prefix, opcode, mod_rm, sib]);
        } else {
            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        }

        self.buf.extend_from_slice(&src.offset.to_le_bytes());
    }

    fn assemble_store(&mut self, src: Register, dst: Address<Imm32, Register>) {
        let rex_prefix = rex(true, false, false);
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

    fn assemble_mov(&mut self, src: Register, dst: Register) {
        let rex_prefix = rex(true, false, false);
        let opcode = 0x89;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(src as u8)
            .rm(dst as u8)
            .build();

        self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
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
        let rex_prefix = rex(true, false, false);
        let opcode = 0x01;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(src as u8)
            .rm(dst as u8)
            .build();

        self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
    }

    fn assemble_add_imm(&mut self, src: i32, dst: Register) {
        let rex_prefix = rex(true, false, false);
        if let Ok(src) = i8::try_from(src) {
            let opcode = 0x83;
            let mod_rm = ModRmBuilder::new().direct().reg(0x0).rm(dst as u8).build();

            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
            self.buf.extend_from_slice(&src.to_le_bytes());
        } else if let Register::Ax = dst {
            let opcode = 0x05;

            self.buf.extend_from_slice(&[rex_prefix, opcode]);
            self.buf.extend_from_slice(&src.to_le_bytes());
        } else {
            let opcode = 0x81;
            let mod_rm = ModRmBuilder::new().direct().reg(0x0).rm(dst as u8).build();

            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
            self.buf.extend_from_slice(&src.to_le_bytes());
        }
    }

    fn assemble_set_if<const OPCODE: u8>(&mut self, src1: Register, src2: Register, dst: Register) {
        if dst != src1 && dst != src2 {
            self.assemble_load_imm::<true>(0x0, dst);

            let rex_prefix = rex(true, false, false);
            let opcode = 0x39;
            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(src2 as u8)
                .rm(src1 as u8)
                .build();

            // cmp reg2,reg1
            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        } else {
            let rex_prefix = rex(true, false, false);
            let opcode = 0x39;
            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(src2 as u8)
                .rm(src1 as u8)
                .build();

            // cmp reg2,reg1
            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);

            self.assemble_load_imm::<false>(0x0, dst);
        }

        // setl dst
        if let Register::Di | Register::Si | Register::Bp | Register::Sp = dst {
            let rex_prefix = rex(false, false, false);
            self.buf.extend_from_slice(&[rex_prefix]);
        }
        let opcode = 0x0f;
        let mod_rm = ModRmBuilder::new().direct().reg(0x0).rm(dst as u8).build();
        self.buf.extend_from_slice(&[opcode, OPCODE, mod_rm]);
    }

    fn assemble_jump(&mut self, target: Label) {
        let opcode = 0xe9;

        self.buf.extend_from_slice(&[opcode]);
        self.add_patch(target);
        self.buf.extend_from_slice(&0x0i32.to_le_bytes());
    }

    fn assemble_jump_if_zero(&mut self, src: Register, target: Label) {
        let rex_prefix = rex(true, false, false);

        // cmp src,0x0
        if let Register::Ax = src {
            let opcode = 0x3d;

            self.buf.extend_from_slice(&[rex_prefix, opcode]);
        } else {
            let opcode = 0x81;
            let mod_rm = ModRmBuilder::new().direct().reg(0x7).rm(src as u8).build();

            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        }

        self.buf.extend_from_slice(&0x0u32.to_le_bytes());

        // je target
        self.buf.extend_from_slice(&[0x0f, 0x84]);
        self.add_patch(target);
        self.buf.extend_from_slice(&0x0i32.to_le_bytes());
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

    pub fn finish(&mut self) -> Result<(), AssemblerError> {
        for patch in &self.patches {
            // The value to be patched is an `i32`.
            let patch_end = patch.start + std::mem::size_of::<i32>();

            let mut label_location = self.label_locations[patch.label.0 as usize]
                .ok_or_else(|| AssemblerError::MissingLabelLocation(patch.label))?
                as i32;
            // The label location must be written relative to the end of the instruction which
            // matches the end of the patch.
            label_location -= patch_end as i32;

            self.buf[patch.start..patch_end].copy_from_slice(&label_location.to_le_bytes());
        }

        Ok(())
    }
}

const fn rex(w_bit: bool, r_bit: bool, b_bit: bool) -> u8 {
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

struct Patch {
    label: Label,
    start: usize,
}
