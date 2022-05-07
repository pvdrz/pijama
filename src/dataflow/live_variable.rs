use crate::{
    index::{Index, IndexMap},
    mir::{Block, BlockData, FnDef, Local, Operand, Rvalue, Statement, Terminator},
};

type LiveSet = super::bit_set::BitSet<Local>;

pub struct LiveVariable<'flow> {
    fn_def: &'flow FnDef,
}

impl<'flow> LiveVariable<'flow> {
    pub fn new(fn_def: &'flow FnDef) -> Self {
        Self { fn_def }
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
                    Rvalue::Phi(values) => {
                        for (_, local) in values {
                            live.insert(local.index());
                        }
                    }
                    _ => (),
                }
            }
            Statement::Nop => {}
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
        LiveSet::new(self.fn_def.locals.len())
    }

    pub fn run(&self) {
        let mut values_in = IndexMap::<Block, LiveSet>::with_capacity(self.fn_def.succs.len());

        for _ in self.fn_def.succs.keys() {
            values_in.push(self.empty_liveset());
        }

        while {
            let mut changed = false;

            for (block, block_data) in self.fn_def.blocks.iter() {
                let mut new_in = self.empty_liveset();

                for &succ in self.fn_def.succs[block].iter() {
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
                .iter()
                .enumerate()
                .filter_map(
                    |(index, alive)| {
                        if alive {
                            Some(Local::new(index))
                        } else {
                            None
                        }
                    },
                )
                .collect::<Vec<_>>();

            println!("{:?}: {:?}", block, defs);
        }
    }
}
