use crate::{
    index::{Index, IndexMap},
    mir::{Block, BlockData, FnDef},
};

type DomSet = super::bit_set::BitSet<Block>;

pub struct Dominators<'flow> {
    fn_def: &'flow FnDef,
}

impl<'flow> Dominators<'flow> {
    pub fn new(fn_def: &'flow FnDef) -> Self {
        Self { fn_def }
    }

    fn transfer_block(&self, dominators: &mut DomSet, _block_data: &BlockData, block: Block) {
        dominators.insert(block.index());
    }

    fn new_domset(&self) -> DomSet {
        DomSet::full(self.fn_def.preds.len())
    }

    pub fn run(&self) {
        let mut values_out = IndexMap::<Block, DomSet>::with_capacity(self.fn_def.preds.len());

        for _ in 0..self.fn_def.blocks.len() + 1 {
            values_out.push(self.new_domset());
        }

        let mut entry_out = DomSet::new(self.fn_def.preds.len());
        entry_out.insert(self.fn_def.entry.index());
        values_out.push(entry_out);
        dbg!(&values_out);
        while {
            let mut changed = false;

            for (block, block_data) in self.fn_def.blocks.iter() {
                let mut new_out = self.new_domset();

                for &pred in self.fn_def.preds[block].iter() {
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
                    if is_dom && index != block.index() {
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
