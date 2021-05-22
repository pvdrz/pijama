use crate::index::{Index, IndexMap};

pub fn example() -> FnDef {
    let mut fn_def = FnDef {
        arity: 1,
        locals: IndexMap::new(),
        blocks: IndexMap::new(),
    };

    let x = fn_def.locals.push(Ty::Int);
    let r = fn_def.locals.push(Ty::Int);
    let c = fn_def.locals.push(Ty::Bool);

    let bb0 = Block(0);
    let bb1 = Block(1);
    let bb2 = Block(2);
    let bb3 = Block(3);

    fn_def.blocks.push(BlockData {
        statements: vec![Statement::Assign {
            lhs: r,
            rhs: Rvalue::Use(Operand::Literal(0)),
        }],
        terminator: Terminator::Jump(bb1),
    });

    fn_def.blocks.push(BlockData {
        statements: vec![Statement::Assign {
            lhs: c,
            rhs: Rvalue::BinaryOp {
                op: BinOp::Le,
                lhs: Operand::Local(x),
                rhs: Operand::Literal(0),
            },
        }],
        terminator: Terminator::JumpIf {
            cond: Operand::Local(c),
            then_blk: bb2,
            else_blk: bb3,
        },
    });

    fn_def.blocks.push(BlockData {
        statements: vec![],
        terminator: Terminator::Return(r),
    });

    fn_def.blocks.push(BlockData {
        statements: vec![
            Statement::Assign {
                lhs: r,
                rhs: Rvalue::BinaryOp {
                    op: BinOp::Add,
                    lhs: Operand::Local(r),
                    rhs: Operand::Local(x),
                },
            },
            Statement::Assign {
                lhs: x,
                rhs: Rvalue::BinaryOp {
                    op: BinOp::Add,
                    lhs: Operand::Local(x),
                    rhs: Operand::Literal(-1),
                },
            },
        ],
        terminator: Terminator::Jump(bb1),
    });

    fn_def
}

pub struct FnDef {
    pub arity: usize,
    pub locals: IndexMap<Local, Ty>,
    pub blocks: IndexMap<Block, BlockData>,
}

pub enum Ty {
    Int,
    Bool,
}

pub struct BlockData {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub enum Statement {
    Assign { lhs: Local, rhs: Rvalue },
}

pub enum Terminator {
    Jump(Block),
    JumpIf {
        cond: Operand,
        then_blk: Block,
        else_blk: Block,
    },
    Return(Local),
}

pub enum Operand {
    Literal(i64),
    Local(Local),
}

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
    Le,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Local(pub usize);

impl Index for Local {
    fn new(index: usize) -> Self {
        Self(index)
    }

    fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Block(pub usize);

impl Index for Block {
    fn new(index: usize) -> Self {
        Self(index)
    }

    fn index(self) -> usize {
        self.0
    }
}
