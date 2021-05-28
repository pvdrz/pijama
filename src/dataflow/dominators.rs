use crate::{
    index::{Index, IndexMap},
    mir::{Block, BlockData, FnDef, Terminator},
};

type DomSet = super::bit_set::BitSet<Block>;

pub struct Dominators<'flow> {
    preds: IndexMap<Block, Vec<Block>>,
    entry: Block,
    fn_def: &'flow FnDef,
}

impl<'flow> Dominators<'flow> {
    pub fn new(fn_def: &'flow FnDef) -> Self {
        let mut preds = IndexMap::<Block, Vec<Block>>::with_capacity(fn_def.blocks.len() + 1);

        for _ in fn_def.blocks.keys() {
            preds.push(vec![]);
        }

        let entry = preds.push(vec![]);
        preds[Block(0)].push(entry);

        for (block, block_data) in fn_def.blocks.iter() {
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

        Self {
            preds,
            entry,
            fn_def,
        }
    }

    fn transfer_block(&self, dominators: &mut DomSet, _block_data: &BlockData, block: Block) {
        dominators.insert(block.index());
    }

    fn new_domset(&self) -> DomSet {
        DomSet::full(self.preds.len())
    }

    pub fn run(&self) {
        let mut values_out = IndexMap::<Block, DomSet>::with_capacity(self.preds.len());

        for _ in self.fn_def.blocks.keys() {
            values_out.push(self.new_domset());
        }

        let mut entry_out = DomSet::new(self.preds.len());
        entry_out.insert(self.entry.index());
        values_out.push(entry_out);

        while {
            let mut changed = false;

            for (block, block_data) in self.fn_def.blocks.iter() {
                let mut new_out = self.new_domset();

                for &pred in self.preds[block].iter() {
                    new_out.intersection(&values_out[pred]);
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
                .filter_map(|(index, is_dom)| {
                    if is_dom {
                        Some(Block::new(index))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            println!("{:?}: {:?}", block, defs);
        }
    }
}
