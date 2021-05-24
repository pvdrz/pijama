use std::fmt;

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

impl FnDef {
    pub fn dump(&self) {
        println!("{{");

        for (local, ty) in self.locals.iter() {
            println!("  let {}: {};", local, ty);
        }

        for (block, block_data) in self.blocks.iter() {
            println!("  {}: {{", block);

            for stmt in block_data.statements.iter() {
                println!("    {};", stmt);
            }
            println!("    {};", block_data.terminator);

            println!("  }}");
        }

        println!("}}");
    }
}

pub enum Ty {
    Int,
    Bool,
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => "Int".fmt(f),
            Self::Bool => "Bool".fmt(f),
        }
    }
}

pub struct BlockData {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub enum Statement {
    Assign { lhs: Local, rhs: Rvalue },
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Assign { lhs, rhs } => write!(f, "{} = {}", lhs, rhs),
        }
    }
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

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Jump(target) => write!(f, "jump {}", target),
            Self::JumpIf {
                cond,
                then_blk,
                else_blk,
            } => write!(f, "if {} then {} else {}", cond, then_blk, else_blk),
            Self::Return(local) => write!(f, "return {}", local),
        }
    }
}

pub enum Rvalue {
    Use(Operand),
    BinaryOp {
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
    },
}

impl fmt::Display for Rvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Use(op) => op.fmt(f),
            Self::BinaryOp { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
        }
    }
}

pub enum Operand {
    Literal(i64),
    Local(Local),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(lit) => lit.fmt(f),
            Self::Local(loc) => loc.fmt(f),
        }
    }
}

pub enum BinOp {
    Add,
    Le,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => "+".fmt(f),
            Self::Le => "<=".fmt(f),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Local(pub usize);

impl Index for Local {
    fn new(index: usize) -> Self {
        Self(index)
    }

    fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_{}", self.0)
    }
}

impl fmt::Debug for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Block(pub usize);

impl Index for Block {
    fn new(index: usize) -> Self {
        Self(index)
    }

    fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
