use std::collections::HashSet;

use crate::{
    index::IndexMap,
    mir::{Block, BlockData, FnDef, Local, Statement, Terminator},
};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Def {
    Dummy(Local),
    Real {
        block: Block,
        stmt_index: usize,
        local: Local,
    },
}

impl Def {
    fn local(&self) -> Local {
        match self {
            Self::Dummy(local) | Self::Real { local, .. } => *local,
        }
    }
}

type Value = HashSet<Def>;

fn transfer_stmt(value: &mut Value, stmt: &Statement, block: Block, stmt_index: usize) {
    match stmt {
        Statement::Assign { lhs, .. } => {
            value.retain(|def| def.local() != *lhs);
            value.insert(Def::Real {
                local: *lhs,
                block,
                stmt_index,
            });
        }
    }
}

fn transfer_block(value: &Value, block_data: &BlockData, block: Block) -> Value {
    let mut value = value.clone();
    for (stmt_index, stmt) in block_data.statements.iter().enumerate() {
        transfer_stmt(&mut value, stmt, block, stmt_index);
    }
    value
}

fn preds_of(fn_def: &FnDef, target: Block) -> HashSet<Block> {
    let mut preds = HashSet::new();

    for (block, block_data) in fn_def.blocks.iter() {
        match &block_data.terminator {
            Terminator::Jump(blk) if &target == blk => {
                preds.insert(block);
            }
            Terminator::JumpIf {
                then_blk, else_blk, ..
            } if &target == then_blk || &target == else_blk => {
                preds.insert(block);
            }
            _ => (),
        }
    }

    preds
}

pub fn dataflow(fn_def: &FnDef) -> IndexMap<Block, HashSet<Def>> {
    let mut values_out = IndexMap::<Block, Value>::with_capacity(fn_def.blocks.len());
    let mut preds = IndexMap::<Block, HashSet<Block>>::with_capacity(fn_def.blocks.len());

    for block in fn_def.blocks.keys() {
        values_out.push(Value::default());
        preds.push(preds_of(fn_def, block));
    }

    let entry = values_out.push(
        fn_def
            .locals
            .keys()
            .skip(fn_def.arity)
            .map(Def::Dummy)
            .collect(),
    );

    preds[Block(0)].insert(entry);

    while {
        let mut changed = false;

        for (block, block_data) in fn_def.blocks.iter() {
            let value_in = preds[block].iter().map(|&pred| &values_out[pred]).fold(
                Value::new(),
                |mut acc, defs| {
                    for def in defs {
                        acc.insert(def.clone());
                    }
                    acc
                },
            );
            let new_out = transfer_block(&value_in, block_data, block);

            let value_out = &mut values_out[block];

            if &new_out != value_out {
                changed = true;
                *value_out = new_out;
            }
        }

        changed
    } {}

    values_out
}
