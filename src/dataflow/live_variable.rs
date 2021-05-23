use crate::{
    index::{Index, IndexMap},
    mir::{Block, BlockData, FnDef, Local, Operand, Rvalue, Statement, Terminator},
};

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Debug)]
enum Def {
    Dummy(Local),
    Real { block: Block, stmt_index: usize },
}

#[derive(PartialEq, Eq)]
struct LiveSet(Box<[bool]>);

impl LiveSet {
    fn union(&mut self, other: &Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(other.0.iter()) {
            *lhs = *lhs || *rhs;
        }
    }

    fn diff(&mut self, other: &Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(other.0.iter()) {
            *lhs = *lhs && !*rhs;
        }
    }

    fn insert(&mut self, index: usize) {
        self.0[index] = true;
    }

    fn remove(&mut self, index: usize) {
        self.0[index] = false;
    }
}

pub struct LiveVariable<'flow> {
    succs: IndexMap<Block, Vec<Block>>,
    exit: Block,
    fn_def: &'flow FnDef,
}

impl<'flow> LiveVariable<'flow> {
    pub fn new(fn_def: &'flow FnDef) -> Self {
        let mut succs = IndexMap::<Block, Vec<Block>>::with_capacity(fn_def.blocks.len() + 1);

        for _ in fn_def.blocks.keys() {
            succs.push(vec![]);
        }

        let exit = succs.push(vec![]);

        for (block, block_data) in fn_def.blocks.iter() {
            let succs = &mut succs[block];
            match block_data.terminator {
                Terminator::Jump(blk) => {
                    if let Err(index) = succs.binary_search(&blk) {
                        succs.insert(index, blk);
                    }
                }
                Terminator::JumpIf {
                    then_blk, else_blk, ..
                } => {
                    if let Err(index) = succs.binary_search(&then_blk) {
                        succs.insert(index, then_blk);
                    }

                    if let Err(index) = succs.binary_search(&else_blk) {
                        succs.insert(index, else_blk);
                    }
                }
                Terminator::Return(_) => succs.push(exit),
            }
        }

        Self {
            succs,
            exit,
            fn_def,
        }
    }

    fn transfer_stmt(&self, live: &mut LiveSet, stmt: &Statement) {
        match &stmt {
            Statement::Assign { lhs, rhs } => {
                live.remove(lhs.index());

                match rhs {
                    Rvalue::Use(Operand::Local(local)) => live.insert(local.index()),
                    Rvalue::BinaryOp {
                        lhs: Operand::Local(lhs_local),
                        rhs: Operand::Local(rhs_local),
                        ..
                    } => {
                        live.insert(lhs_local.index());
                        live.insert(rhs_local.index());
                    }
                    _ => (),
                }
            }
        }
    }
    fn transfer_term(&self, live: &mut LiveSet, term: &Terminator) {
        match term {
            Terminator::JumpIf {
                cond: Operand::Local(local),
                ..
            }
            | Terminator::Return(local) => {
                live.insert(local.index());
            }
            _ => (),
        }
    }

    fn transfer_block(&self, live: &mut LiveSet, block_data: &BlockData) {
        self.transfer_term(live, &block_data.terminator);

        for stmt in block_data.statements.iter().rev() {
            self.transfer_stmt(live, stmt);
        }
    }

    fn empty_liveset(&self) -> LiveSet {
        LiveSet(vec![false; self.fn_def.locals.len()].into_boxed_slice())
    }

    pub fn run(&self) {
        let mut values_in = IndexMap::<Block, LiveSet>::with_capacity(self.succs.len());

        for _ in self.succs.keys() {
            values_in.push(self.empty_liveset());
        }

        while {
            let mut changed = false;

            for (block, block_data) in self.fn_def.blocks.iter() {
                let mut new_in = self.empty_liveset();

                for &succ in self.succs[block].iter() {
                    new_in.union(&values_in[succ]);
                }

                self.transfer_block(&mut new_in, block_data);

                let value_in = &mut values_in[block];
                if &new_in != value_in {
                    changed = true;
                    *value_in = new_in;
                }
            }

            changed
        } {}

        for (block, defset) in values_in.iter() {
            let defs = defset
                .0
                .iter()
                .enumerate()
                .filter_map(|(index, alive)| {
                    if *alive {
                        Some(Local::new(index))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            println!("{:?}: {:?}", block, defs);
        }
    }
}
