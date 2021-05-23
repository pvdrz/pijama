use crate::{
    index::IndexMap,
    mir::{Block, BlockData, FnDef, Local, Statement, Terminator},
};

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Debug)]
enum Def {
    Dummy(Local),
    Real { block: Block, stmt_index: usize },
}

#[derive(PartialEq, Eq)]
struct DefSet(Box<[bool]>);

impl DefSet {
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
}

pub struct ReachingDefs<'flow> {
    definitions: Box<[Def]>,
    local_defsets: IndexMap<Local, DefSet>,
    preds: IndexMap<Block, Vec<Block>>,
    entry: Block,
    fn_def: &'flow FnDef,
}

impl<'flow> ReachingDefs<'flow> {
    pub fn new(fn_def: &'flow FnDef) -> Self {
        let mut definitions = Vec::new();
        let mut local_defs = Vec::new();
        let mut preds = IndexMap::<Block, Vec<Block>>::with_capacity(fn_def.blocks.len() + 1);

        let mut local_defsets = IndexMap::<Local, DefSet>::with_capacity(fn_def.locals.len());

        for _ in fn_def.blocks.keys() {
            preds.push(vec![]);
        }

        let entry = preds.push(vec![]);
        preds[Block(0)].push(entry);

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

                match block_data.terminator {
                    Terminator::Jump(blk) => {
                        let preds = &mut preds[blk];
                        if let Err(index) = preds.binary_search(&block) {
                            preds.insert(index, block);
                        }
                    }
                    Terminator::JumpIf {
                        then_blk, else_blk, ..
                    } => {
                        let preds_then = &mut preds[then_blk];
                        if let Err(index) = preds_then.binary_search(&block) {
                            preds_then.insert(index, block);
                        }

                        let preds_else = &mut preds[else_blk];
                        if let Err(index) = preds_else.binary_search(&block) {
                            preds_else.insert(index, block);
                        }
                    }
                    Terminator::Return(_) => {}
                }
            }
        }

        for local in fn_def.locals.keys() {
            let local_defset = DefSet(
                local_defs
                    .iter()
                    .map(|&local_def| local == local_def)
                    .collect::<Box<[_]>>(),
            );

            local_defsets.push(local_defset);
        }

        Self {
            definitions: definitions.into_boxed_slice(),
            local_defsets,
            preds,
            entry,
            fn_def,
        }
    }

    fn transfer_stmt(&self, def: &mut DefSet, stmt: &Statement, block: Block, stmt_index: usize) {
        match stmt {
            &Statement::Assign { lhs, .. } => {
                let kill = &self.local_defsets[lhs];
                def.diff(kill);

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
        DefSet(vec![false; self.definitions.len()].into_boxed_slice())
    }

    pub fn run(&self) {
        let mut values_out = IndexMap::<Block, DefSet>::with_capacity(self.preds.len());

        for _ in self.preds.keys() {
            values_out.push(self.empty_defset());
        }

        let entry_out = &mut values_out[self.entry];
        for i in 0..(self.local_defsets.len() - self.fn_def.arity) {
            entry_out.insert(i);
        }

        while {
            let mut changed = false;

            for (block, block_data) in self.fn_def.blocks.iter() {
                let mut new_out = self.empty_defset();

                for &pred in self.preds[block].iter() {
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
                .0
                .iter()
                .enumerate()
                .filter_map(|(index, reachable)| {
                    if *reachable {
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
