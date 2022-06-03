use crate::mir::statement::Statement;
use crate::mir::terminator::Terminator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasicBlockId(pub(super) usize);

pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}
