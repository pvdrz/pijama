use crate::mir::{bb::BasicBlockId, Operand};

pub enum Terminator {
    Jump(BasicBlockId),
    Return,
    JumpIf {
        cond: Operand,
        then_bb: BasicBlockId,
        else_bb: BasicBlockId,
    },
}
