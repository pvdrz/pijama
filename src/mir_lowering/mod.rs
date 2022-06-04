use std::collections::BTreeMap;

use crate::{
    asm::{Assembler, Imm32, Imm64, Instruction, Label, Register},
    code,
    mir::{
        BasicBlock, BasicBlockId, BinOp, Function, Local, Operand, Rvalue, Statement, Terminator,
        Ty,
    },
};

pub fn lower_function(func: &Function) -> Vec<u8> {
    const AVAILABLE_REGISTERS: [Register; 5] = [
        Register::Ax,
        Register::Di,
        Register::Si,
        Register::Dx,
        Register::Cx,
    ];

    if func.args_len > 3 {
        todo!("cannot lower function with {} arguments", func.args_len);
    }

    if func.local_types.len() > 5 {
        todo!(
            "cannot lower function with {} locals",
            func.local_types.len()
        );
    }

    let mut asm = Assembler::default();

    let local_registers = func
        .local_types
        .keys()
        .copied()
        .zip(AVAILABLE_REGISTERS)
        .collect();

    let block_labels = func
        .basic_blocks
        .keys()
        .copied()
        .map(|label| (label, asm.add_label()))
        .collect();

    let mut ctx = LowerCtx {
        local_registers,
        block_labels,
        instructions: Vec::new(),
        asm,
    };

    for (bb, bb_data) in &func.basic_blocks {
        ctx.lower_block(*bb, bb_data);
    }

    ctx.asm.emit_code()
}

enum AsmOperand {
    Reg(Register),
    Imm32(Imm32),
}

struct LowerCtx {
    local_registers: BTreeMap<Local, Register>,
    block_labels: BTreeMap<BasicBlockId, Label>,
    instructions: Vec<Instruction>,
    asm: Assembler,
}

impl LowerCtx {
    fn lower_operand(&self, operand: &Operand) -> AsmOperand {
        match operand {
            Operand::Local(ref local) => AsmOperand::Reg(self.local_registers[local]),
            Operand::Constant(ref literal) => match literal.ty {
                Ty::Int => AsmOperand::Imm32(literal.data as Imm32),
                Ty::Bool => AsmOperand::Imm32(literal.data as Imm32),
            },
        }
    }

    fn lower_terminator(&mut self, terminator: &Terminator) {
        match terminator {
            Terminator::Jump(ref bb) => self
                .instructions
                .push(code!( jmp { self.block_labels[bb] } )),
            Terminator::Return => self.instructions.push(code! { ret }),
            Terminator::JumpIf {
                ref cond,
                ref then_bb,
                ref else_bb,
            } => match self.lower_operand(cond) {
                AsmOperand::Reg(cond) => {
                    self.instructions
                        .push(code!(jz { cond }, { self.block_labels[else_bb] }));
                    self.instructions
                        .push(code!(jmp { self.block_labels[then_bb] }));
                }
                AsmOperand::Imm32(cond) => {
                    let bb = match cond {
                        0 => else_bb,
                        _ => then_bb,
                    };
                    self.instructions.push(code!(jmp { self.block_labels[bb] }))
                }
            },
        }
    }

    fn lower_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Assign { ref lhs, ref rhs } => {
                let lhs = self.local_registers[lhs];

                match rhs {
                    Rvalue::Use(ref operand) => match self.lower_operand(operand) {
                        AsmOperand::Reg(rhs) => self.instructions.push(code!(mov { rhs }, { lhs })),
                        AsmOperand::Imm32(rhs) => {
                            self.instructions.push(code!(loadi { rhs.into() }, { lhs }))
                        }
                    },
                    Rvalue::BinaryOp {
                        ref op,
                        lhs: ref lhs_op,
                        rhs: ref rhs_op,
                    } => match (self.lower_operand(lhs_op), self.lower_operand(rhs_op)) {
                        (AsmOperand::Reg(lhs_op), AsmOperand::Reg(rhs_op)) => match op {
                            BinOp::Add => {
                                if lhs == lhs_op {
                                    // lhs = lhs + rhs_op -> lhs += rhs_op
                                    self.instructions.push(code!(add { rhs_op }, { lhs }));
                                } else if lhs == rhs_op {
                                    // lhs = lhs_op + lhs -> lhs += lhs_op
                                    self.instructions.push(code!(add { lhs_op }, { lhs }));
                                } else {
                                    // lhs = lhs_op + rhs_op -> lhs = lhs_op; lhs += rhs_op
                                    self.instructions.push(code!(mov { lhs_op }, { lhs }));
                                    self.instructions.push(code!(add { rhs_op }, { lhs }));
                                }
                            }
                            BinOp::Lt => {
                                self.instructions
                                    .push(code!(slt { lhs_op }, { rhs_op }, { lhs }))
                            }
                        },
                        (AsmOperand::Reg(lhs_op), AsmOperand::Imm32(rhs_op)) => match op {
                            BinOp::Add => {
                                if lhs == lhs_op {
                                    // lhs = lhs + rhs_op -> lhs += rhs_op
                                    self.instructions.push(code!(addi { rhs_op }, { lhs }));
                                } else {
                                    // lhs = lhs_op + rhs_op -> lhs = lhs_op; lhs += rhs_op
                                    self.instructions.push(code!(mov { lhs_op }, { lhs }));
                                    self.instructions.push(code!(addi { rhs_op }, { lhs }));
                                }
                            }
                            BinOp::Lt => todo!(),
                        },
                        (AsmOperand::Imm32(lhs_op), AsmOperand::Reg(rhs_op)) => match op {
                            BinOp::Add => {
                                if lhs == rhs_op {
                                    // lhs = lhs_op + lhs -> lhs += lhs_op
                                    self.instructions.push(code!(addi { lhs_op }, { lhs }));
                                } else {
                                    // lhs = lhs_op + rhs_op -> lhs = lhs_op; lhs += rhs_op
                                    self.instructions
                                        .push(code!(loadi { lhs_op.into() }, { lhs }));
                                    self.instructions.push(code!(add { rhs_op }, { lhs }));
                                }
                            }
                            BinOp::Lt => todo!(),
                        },
                        (AsmOperand::Imm32(lhs_op), AsmOperand::Imm32(rhs_op)) => match op {
                            BinOp::Add => {
                                // lhs = lhs_op + rhs_op -> lhs = lhs_op; lhs += rhs_op
                                self.instructions
                                    .push(code!(loadi { lhs_op.into() }, { lhs }));
                                self.instructions.push(code!(addi { rhs_op }, { lhs }));
                            }
                            BinOp::Lt => {
                                let imm = if lhs_op < rhs_op { 1 } else { 0 };
                                self.instructions.push(code!(loadi { imm }, { lhs }));
                            }
                        },
                    },
                }
            }
        }
    }

    fn lower_block(&mut self, bb: BasicBlockId, bb_data: &BasicBlock) {
        for statement in &bb_data.statements {
            self.lower_statement(statement);
        }

        self.lower_terminator(&bb_data.terminator);

        if let Some(instruction) = self.instructions.first_mut() {
            instruction.label = Some(self.block_labels[&bb]);
        }

        for instruction in self.instructions.drain(..) {
            self.asm.assemble_instruction(instruction);
        }
    }
}
