use crate::{
    index::IndexMap,
    mir::{Block, BlockData, FnDef, Local, Statement},
};

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Debug)]
enum Def {
    Dummy(Local),
    Real { block: Block, stmt_index: usize },
}

type DefSet = super::bit_set::BitSet<Def>;

pub struct ReachingDefs<'flow> {
    definitions: Box<[Def]>,
    local_defsets: IndexMap<Local, DefSet>,
    fn_def: &'flow FnDef,
}

impl<'flow> ReachingDefs<'flow> {
    pub fn new(fn_def: &'flow FnDef) -> Self {
        let mut definitions = Vec::new();
        let mut local_defs = Vec::new();

        let mut local_defsets = IndexMap::<Local, DefSet>::with_capacity(fn_def.locals.len());

        for local in fn_def.locals.keys().skip(fn_def.arity) {
            definitions.push(Def::Dummy(local));
            local_defs.push(local);
        }

        for (block, block_data) in fn_def.blocks.iter() {
            for (stmt_index, stmt) in block_data.statements.iter().enumerate() {
                match stmt {
                    Statement::Assign { lhs, .. } => {
                        let def = Def::Real { block, stmt_index };
                        // Definitions are traversed in order so we can just push them.
                        definitions.push(def);
                        local_defs.push(*lhs);
                    }
                }
            }
        }

        for local in fn_def.locals.keys() {
            let mut local_defset = DefSet::new(local_defs.len());

            for (index, &local_def) in local_defs.iter().enumerate() {
                if local == local_def {
                    local_defset.insert(index);
                }
            }

            local_defsets.push(local_defset);
        }

        Self {
            definitions: definitions.into_boxed_slice(),
            local_defsets,
            fn_def,
        }
    }

    fn transfer_stmt(&self, def: &mut DefSet, stmt: &Statement, block: Block, stmt_index: usize) {
        match stmt {
            &Statement::Assign { lhs, .. } => {
                let kill = &self.local_defsets[lhs];
                def.difference(kill);

                let gen = self
                    .definitions
                    .binary_search(&Def::Real { block, stmt_index })
                    .unwrap();
                def.insert(gen);
            }
        }
    }

    fn transfer_block(&self, def: &mut DefSet, block_data: &BlockData, block: Block) {
        for (stmt_index, stmt) in block_data.statements.iter().enumerate() {
            self.transfer_stmt(def, stmt, block, stmt_index);
        }
    }

    fn empty_defset(&self) -> DefSet {
        DefSet::new(self.definitions.len())
    }

    pub fn run(&self) {
        let mut values_out = IndexMap::<Block, DefSet>::with_capacity(self.fn_def.preds.len());

        for _ in self.fn_def.preds.keys() {
            values_out.push(self.empty_defset());
        }

        let entry_out = &mut values_out[self.fn_def.entry];
        for i in 0..(self.local_defsets.len() - self.fn_def.arity) {
            entry_out.insert(i);
        }

        while {
            let mut changed = false;

            for (block, block_data) in self.fn_def.blocks.iter() {
                let mut new_out = self.empty_defset();

                for &pred in self.fn_def.preds[block].iter() {
                    new_out.union(&values_out[pred]);
                }

                self.transfer_block(&mut new_out, block_data, block);

                let value_out = &mut values_out[block];
                if &new_out != value_out {
                    changed = true;
                    *value_out = new_out;
                }
            }

            changed
        } {}

        for (block, defset) in values_out.iter() {
            let defs = defset
                .iter()
                .enumerate()
                .filter_map(|(index, reachable)| {
                    if reachable {
                        Some(&self.definitions[index])
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            println!("{:?}: {:?}", block, defs);
        }
    }
}
