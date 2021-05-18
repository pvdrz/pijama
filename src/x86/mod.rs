mod modrm;
mod rex;

use modrm::ModRmBuilder;
use rex::RexBuilder;

use crate::asm::{Address, BaseAddr, Instruction, InstructionKind, Label, Register};

use std::collections::BTreeMap;
use std::convert::TryFrom;

pub struct Assembler<'buf> {
    buffer: &'buf mut Vec<u8>,
    patches: Vec<(Label, usize, i64)>,
}

impl<'buf> Assembler<'buf> {
    pub fn assemble_code(code: Vec<Instruction>) -> Vec<u8> {
        let mut buffer = Vec::new();

        let mut asm = Assembler {
            buffer: &mut buffer,
            patches: Vec::new(),
        };

        let mut labels = BTreeMap::new();

        for ins in code {
            if let Some(label) = ins.label {
                labels.insert(label, asm.buffer.len() as i64);
            }

            match ins.kind {
                InstructionKind::Load { src, dst } => asm.assemble_load(src, dst),
                InstructionKind::LoadImm { src, dst } => asm.assemble_load_imm(src, dst),
                InstructionKind::Store { src, dst } => asm.assemble_store(src, dst),
                InstructionKind::Push(src) => asm.assemble_push(src),
                InstructionKind::Pop(dst) => asm.assemble_pop(dst),
                InstructionKind::Add { src, dst } => asm.assemble_add(src, dst),
                InstructionKind::AddImm { src, dst } => asm.assemble_add_imm(src, dst),
                InstructionKind::JumpLez { reg, addr } => asm.assemble_jump_lez(reg, addr),
                InstructionKind::Jump(addr) => asm.assemble_jump(addr),
                InstructionKind::Return => asm.assemble_return(),
                InstructionKind::Call(reg) => asm.assemble_call(reg),
            }
        }

        for (label, ip, off) in asm.patches.drain(..) {
            let imm = i32::try_from(labels[&label] + off).unwrap().to_le_bytes();
            asm.buffer[ip..ip + imm.len()].copy_from_slice(&imm);
        }

        buffer
    }

    fn add_patch(&mut self, label: Label, offset: i64) {
        // This is the IP where the patch should go.
        let ip = self.buffer.len();
        // Write a dummy value.
        self.buffer.extend_from_slice(&i32::MAX.to_le_bytes());
        // Compute the offset to be added to the label from the IP after writing.
        let offset = offset - self.buffer.len() as i64;
        // Add the patch.
        self.patches.push((label, ip, offset));
    }

    fn assemble_load(&mut self, src: Address, dst: Register) {
        // We need the 64-bit operand mode regardless of the instruction used. We only use 8
        // registers.
        self.buffer.push(
            RexBuilder::new()
                .size_64()
                .set_b(false)
                .set_r(false)
                .set_x(false)
                .build(),
        );

        match src.base {
            BaseAddr::Ind(src_reg) => {
                // The instruction is `MOV reg64, reg/mem64`.
                self.buffer.push(0x8b);
                if src.offset == 0 {
                    // If the offset is zero we use indirect mode.
                    self.buffer.push(
                        ModRmBuilder::new()
                            .indirect()
                            .reg(src_reg as u8)
                            .rm(dst as u8)
                            .build(),
                    );
                } else {
                    // If the offset is non-zero we use indirect mode and displacement.
                    self.buffer.push(
                        ModRmBuilder::new()
                            .displacement()
                            .rm(src_reg as u8)
                            .reg(dst as u8)
                            .build(),
                    );
                    // Write the displacement
                    self.buffer.extend_from_slice(&src.offset.to_le_bytes());
                }
            }
            BaseAddr::Lab(src_lab) => {
                // The instruction is `MOV reg64, reg/mem64`.
                self.buffer.push(0x8b);
                // We use IP relative mode.
                self.buffer
                    .push(ModRmBuilder::new().relative().reg(dst as u8).build());
                // Add a patch to compute the relative offset.
                self.add_patch(src_lab, src.offset.into());
            }
        }
    }

    fn assemble_load_imm(&mut self, src: i64, dst: Register) {
        self.buffer.push(
            RexBuilder::new()
                .size_64()
                .set_b(false)
                .set_r(false)
                .set_x(false)
                .build(),
        );

        self.buffer.push(0xb8 + dst as u8);
        self.buffer.extend_from_slice(&src.to_le_bytes());
    }

    fn assemble_store(&mut self, src: Register, dst: Address) {
        self.buffer.push(
            RexBuilder::new()
                .size_64()
                .set_b(false)
                .set_r(false)
                .set_x(false)
                .build(),
        );

        match dst.base {
            BaseAddr::Ind(dst_reg) => {
                self.buffer.push(0x89);
                match dst.offset {
                    0 => self.buffer.push(
                        ModRmBuilder::new()
                            .indirect()
                            .rm(dst_reg as u8)
                            .reg(src as u8)
                            .build(),
                    ),
                    dst_off => {
                        self.buffer.push(
                            ModRmBuilder::new()
                                .displacement()
                                .rm(dst_reg as u8)
                                .reg(src as u8)
                                .build(),
                        );
                        self.buffer.extend_from_slice(&dst_off.to_le_bytes());
                    }
                }
            }
            BaseAddr::Lab(dst_lab) => {
                self.buffer.push(0x89);
                self.buffer
                    .push(ModRmBuilder::new().relative().reg(src as u8).build());
                // Add a patch to compute the relative offset.
                self.add_patch(dst_lab, dst.offset.into());
            }
        }
    }

    fn assemble_push(&mut self, src: Register) {
        self.buffer.push(0x50 + src as u8);
    }

    fn assemble_pop(&mut self, dst: Register) {
        self.buffer.push(0x58 + dst as u8);
    }

    fn assemble_add(&mut self, src: Register, dst: Register) {
        self.buffer.push(
            RexBuilder::new()
                .size_64()
                .set_b(false)
                .set_r(false)
                .set_x(false)
                .build(),
        );
        self.buffer.push(0x01);
        self.buffer.push(
            ModRmBuilder::new()
                .direct()
                .reg(src as u8)
                .rm(dst as u8)
                .build(),
        );
    }

    fn assemble_add_imm(&mut self, src: i64, dst: Register) {
        self.buffer.push(
            RexBuilder::new()
                .size_64()
                .set_b(false)
                .set_r(false)
                .set_x(false)
                .build(),
        );
        self.buffer.push(0x81);
        self.buffer
            .push(ModRmBuilder::new().direct().reg(0x0).rm(dst as u8).build());
        self.buffer
            .extend_from_slice(&i32::try_from(src).unwrap().to_le_bytes());
    }

    fn assemble_jump_lez(&mut self, reg: Register, addr: Address) {
        self.buffer.push(
            RexBuilder::new()
                .size_64()
                .set_b(false)
                .set_r(false)
                .set_x(false)
                .build(),
        );
        self.buffer.push(0x83);
        self.buffer
            .push(ModRmBuilder::new().direct().reg(0x7).rm(reg as u8).build());
        self.buffer.push(0x0);

        match addr.base {
            BaseAddr::Ind(_) => todo!(),
            BaseAddr::Lab(addr_lab) => {
                self.buffer.extend_from_slice(&[0x0f, 0x8c]);
                // Add a patch to compute the relative offset.
                self.add_patch(addr_lab, addr.offset.into());
            }
        }
    }

    fn assemble_jump(&mut self, addr: Address) {
        match addr.base {
            BaseAddr::Ind(addr_reg) => {
                self.buffer.push(0xff);
                match addr.offset {
                    0 => self.buffer.push(
                        ModRmBuilder::new()
                            .indirect()
                            .reg(0x4)
                            .rm(addr_reg as u8)
                            .build(),
                    ),
                    addr_off => {
                        self.buffer.push(
                            ModRmBuilder::new()
                                .displacement()
                                .reg(0x4)
                                .rm(addr_reg as u8)
                                .build(),
                        );
                        self.buffer.extend_from_slice(&addr_off.to_le_bytes());
                    }
                }
            }
            BaseAddr::Lab(addr_lab) => {
                self.buffer.push(0xe9);
                // Add a patch to compute the relative offset.
                self.add_patch(addr_lab, addr.offset.into());
            }
        }
    }

    fn assemble_return(&mut self) {
        self.buffer.push(0xc3);
    }

    fn assemble_call(&mut self, reg: Register) {
        self.buffer.push(0xff);
        self.buffer.push(
            ModRmBuilder::new()
                .indirect()
                .reg(0x2)
                .rm(reg as u8)
                .build(),
        );
    }
}
