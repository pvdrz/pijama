mod mod_rm;
mod rex;
mod sib;

use std::error::Error;
use std::fmt;

use crate::asm::x86_64::register::Register;
use crate::asm::{Address, Imm32, Imm64, Instruction, InstructionKind, Instructions, Label};
use mod_rm::ModRmBuilder;
use rex::RexBuilder;
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

    fn push_byte(&mut self, byte: u8) {
        self.buf.push(byte)
    }

    fn push_bytes<const N: usize>(&mut self, bytes: [u8; N]) {
        self.buf.extend_from_slice(&bytes)
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
            // xor dst,dst
            if dst.needs_extension() {
                let rex_prefix = RexBuilder::new()
                    .set_w(false)
                    .set_r(true)
                    .set_x(false)
                    .set_b(true)
                    .finish();
                self.push_byte(rex_prefix);
            }

            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(dst.encode())
                .rm(dst.encode())
                .build();
            self.push_bytes([0x31, mod_rm]);
        } else if let Ok(src) = Imm32::try_from(src) {
            // mov dst,imm32
            if dst.needs_extension() {
                let rex_prefix = RexBuilder::new()
                    .set_w(false)
                    .set_r(false)
                    .set_x(false)
                    .set_b(true)
                    .finish();

                self.push_byte(rex_prefix);
            }

            self.push_byte(0xb8 + dst.encode());
            self.push_bytes(src.to_le_bytes());
        } else {
            // mov dst,imm64
            let rex_prefix = RexBuilder::new()
                .set_w(true)
                .set_r(false)
                .set_x(false)
                .set_b(dst.needs_extension())
                .finish();

            self.buf
                .extend_from_slice(&[rex_prefix, 0xb8 + dst.encode()]);
            self.push_bytes(src.to_le_bytes());
        }
    }

    fn assemble_load_addr(&mut self, src: Address<Imm32, Register>, dst: Register) {
        let rex_prefix = RexBuilder::new()
            .set_w(true)
            .set_r(dst.needs_extension())
            .set_x(false)
            .set_b(src.base.needs_extension())
            .finish();

        let mod_rm = ModRmBuilder::new()
            .displacement()
            .reg(dst.encode())
            .rm(src.base.encode())
            .build();

        self.push_bytes([rex_prefix, 0x8b, mod_rm]);

        if let Register::Sp | Register::R12 = src.base {
            let sib = SibBuilder::new()
                .scale(Scale::One)
                .index(src.base)
                .base(src.base)
                .build();

            self.push_byte(sib);
        }

        self.push_bytes(src.offset.to_le_bytes());
    }

    fn assemble_store(&mut self, src: Register, dst: Address<Imm32, Register>) {
        let rex_prefix = RexBuilder::new()
            .set_w(true)
            .set_r(dst.base.needs_extension())
            .set_x(false)
            .set_b(src.needs_extension())
            .finish();

        let mod_rm = ModRmBuilder::new()
            .displacement()
            .reg(dst.base.encode())
            .rm(src.encode())
            .build();

        self.push_bytes([rex_prefix, 0x89, mod_rm]);

        if let Register::Sp | Register::R12 = src {
            let sib = SibBuilder::new()
                .scale(Scale::One)
                .index(src)
                .base(src)
                .build();

            self.push_byte(sib);
        }

        self.push_bytes(dst.offset.to_le_bytes());
    }

    fn assemble_mov(&mut self, src: Register, dst: Register) {
        let rex_prefix = RexBuilder::new()
            .set_w(true)
            .set_r(src.needs_extension())
            .set_x(false)
            .set_b(dst.needs_extension())
            .finish();

        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(src.encode())
            .rm(dst.encode())
            .build();

        self.push_bytes([rex_prefix, 0x89, mod_rm]);
    }

    fn assemble_push(&mut self, reg: Register) {
        if reg.needs_extension() {
            let rex_prefix = RexBuilder::new()
                .set_w(false)
                .set_r(false)
                .set_x(false)
                .set_b(true)
                .finish();

            self.push_byte(rex_prefix);
        }

        self.push_byte(0x50 + reg.encode());
    }

    fn assemble_pop(&mut self, reg: Register) {
        if reg.needs_extension() {
            let rex_prefix = RexBuilder::new()
                .set_w(false)
                .set_r(false)
                .set_x(false)
                .set_b(true)
                .finish();

            self.push_byte(rex_prefix);
        }

        self.push_byte(0x58 + reg.encode());
    }

    fn assemble_add(&mut self, src: Register, dst: Register) {
        let rex_prefix = RexBuilder::new()
            .set_w(true)
            .set_r(src.needs_extension())
            .set_x(false)
            .set_b(dst.needs_extension())
            .finish();

        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(src.encode())
            .rm(dst.encode())
            .build();

        self.push_bytes([rex_prefix, 0x01, mod_rm]);
    }

    fn assemble_add_imm(&mut self, src: i32, dst: Register) {
        let rex_prefix = RexBuilder::new()
            .set_w(true)
            .set_r(false)
            .set_x(false)
            .set_b(dst.needs_extension())
            .finish();

        if let Ok(src) = i8::try_from(src) {
            // add dst,imm8
            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(0x0)
                .rm(dst.encode())
                .build();

            self.push_bytes([rex_prefix, 0x83, mod_rm]);
            self.push_bytes(src.to_le_bytes());
        } else if let Register::Ax = dst {
            // add rax,imm32
            self.push_bytes([rex_prefix, 0x05]);
            self.push_bytes(src.to_le_bytes());
        } else {
            // add rax,imm32
            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(0x0)
                .rm(dst.encode())
                .build();

            self.push_bytes([rex_prefix, 0x81, mod_rm]);
            self.push_bytes(src.to_le_bytes());
        }
    }

    fn assemble_set_if<const OPCODE: u8>(&mut self, src1: Register, src2: Register, dst: Register) {
        if dst != src1 && dst != src2 {
            // xor dst,dst
            self.assemble_load_imm::<true>(0x0, dst);

            let rex_prefix = RexBuilder::new()
                .set_w(true)
                .set_r(src2.needs_extension())
                .set_x(false)
                .set_b(src1.needs_extension())
                .finish();

            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(src2.encode())
                .rm(src1.encode())
                .build();

            self.push_bytes([rex_prefix, 0x39, mod_rm]);
        } else {
            // cmp src2,src1
            let rex_prefix = RexBuilder::new()
                .set_w(true)
                .set_r(src2.needs_extension())
                .set_x(false)
                .set_b(src1.needs_extension())
                .finish();
            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(src2.encode())
                .rm(src1.encode())
                .build();

            self.push_bytes([rex_prefix, 0x39, mod_rm]);

            // mov dst,0x0
            self.assemble_load_imm::<false>(0x0, dst);
        }

        // setl dst
        if !matches!(
            dst,
            Register::Ax | Register::Cx | Register::Bx | Register::Dx
        ) {
            let rex_prefix = RexBuilder::new()
                .set_w(false)
                .set_r(false)
                .set_x(false)
                .set_b(dst.needs_extension())
                .finish();

            self.push_byte(rex_prefix);
        }

        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(0x0)
            .rm(dst.encode())
            .build();

        self.push_bytes([0x0f, OPCODE, mod_rm]);
    }

    fn assemble_jump(&mut self, target: Label) {
        self.push_byte(0xe9);
        self.add_patch(target);
        self.push_bytes(0x0i32.to_le_bytes());
    }

    fn assemble_jump_if_zero(&mut self, src: Register, target: Label) {
        let rex_prefix = RexBuilder::new()
            .set_w(true)
            .set_r(false)
            .set_x(false)
            .set_b(src.needs_extension())
            .finish();

        // cmp src,0x0
        if let Register::Ax = src {
            self.push_bytes([rex_prefix, 0x3d]);
        } else {
            let mod_rm = ModRmBuilder::new()
                .direct()
                .reg(0x7)
                .rm(src.encode())
                .build();

            self.push_bytes([rex_prefix, 0x81, mod_rm]);
        }

        self.push_bytes(0x0u32.to_le_bytes());

        // je target
        self.push_bytes([0x0f, 0x84]);
        self.add_patch(target);
        self.push_bytes(0x0i32.to_le_bytes());
    }

    fn assemble_return(&mut self) {
        self.push_byte(0xc3);
    }

    fn assemble_call(&mut self, target: Register) {
        if target.needs_extension() {
            let rex_prefix = RexBuilder::new()
                .set_w(false)
                .set_r(false)
                .set_x(false)
                .set_b(true)
                .finish();

            self.push_byte(rex_prefix);
        }

        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(0x2)
            .rm(target.encode())
            .build();

        self.push_bytes([0xff, mod_rm]);
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

struct Patch {
    label: Label,
    start: usize,
}
