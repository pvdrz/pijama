mod bb;
mod func;
mod statement;
mod terminator;

pub use func::Function;
pub use bb::{BasicBlock, BasicBlockId};
pub use terminator::Terminator;
pub use statement::Statement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Local(usize);

pub enum Rvalue {
    Use(Operand),
    BinaryOp {
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
    },
}

pub enum BinOp {
    Add,
    Lt,
}

#[derive(Clone)]
pub enum Operand {
    Local(Local),
    Constant(Literal),
}

#[derive(Clone)]
pub struct Literal {
    pub data: u64,
    pub ty: Ty,
}

#[derive(Clone)]
pub enum Ty {
    Int,
    Bool,
}
