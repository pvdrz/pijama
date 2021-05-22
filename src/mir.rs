use std::collections::BTreeMap;

pub fn example() -> FnDef {
    let mut fn_def = FnDef {
        arity: 1,
        locals: Default::default(),
        blocks: Default::default(),
    };

    let x = Local(0);
    let r = Local(1);
    let c = Local(2);

    fn_def.locals.insert(x, Ty::Int);
    fn_def.locals.insert(r, Ty::Int);
    fn_def.locals.insert(c, Ty::Bool);

    let bb0 = Block(0);
    let bb1 = Block(1);
    let bb2 = Block(2);
    let bb3 = Block(3);

    fn_def.blocks.insert(
        bb0,
        BlockData {
            statements: vec![Statement::Assign {
                lhs: r,
                rhs: Rvalue::Use(Operand::Literal(0)),
            }]
            .into_boxed_slice(),
            terminator: Terminator::Jump(bb1),
        },
    );

    fn_def.blocks.insert(
        bb1,
        BlockData {
            statements: vec![Statement::Assign {
                lhs: c,
                rhs: Rvalue::BinaryOp {
                    op: BinOp::Le,
                    lhs: Operand::Local(x),
                    rhs: Operand::Literal(0),
                },
            }]
            .into_boxed_slice(),
            terminator: Terminator::JumpIf {
                cond: Operand::Local(c),
                then_blk: bb2,
                else_blk: bb3,
            },
        },
    );

    fn_def.blocks.insert(
        bb2,
        BlockData {
            statements: vec![].into_boxed_slice(),
            terminator: Terminator::Return(r),
        },
    );

    fn_def.blocks.insert(
        bb3,
        BlockData {
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
            ]
            .into_boxed_slice(),
            terminator: Terminator::Jump(bb1),
        },
    );

    fn_def
}

pub struct FnDef {
    pub arity: usize,
    pub locals: BTreeMap<Local, Ty>,
    pub blocks: BTreeMap<Block, BlockData>,
}

pub enum Ty {
    Int,
    Bool,
}

pub struct BlockData {
    pub statements: Box<[Statement]>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Block(pub usize);
