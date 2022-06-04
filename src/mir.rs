mod bb;
mod func;
mod statement;
mod terminator;

pub use bb::{BasicBlock, BasicBlockId};
pub use func::Function;
pub use statement::Statement;
pub use terminator::Terminator;

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
    pub data: u32,
    pub ty: Ty,
}

#[derive(Clone)]
pub enum Ty {
    Int,
    Bool,
}
