use crate::index::IndexMap;
use crate::mir::{Block, FnDef};

use std::collections::{HashMap, HashSet};

pub struct DominatorTreeBuilder<'build> {
    fn_def: &'build FnDef,
    dfnum: IndexMap<Block, usize>,
    ancestors: HashMap<Block, Block>,
    idoms: HashMap<Block, Block>,
    samedoms: HashMap<Block, Block>,
    vertices: Vec<Block>,
    parents: HashMap<Block, Block>,
    bucket: IndexMap<Block, HashSet<Block>>,
    semidoms: HashMap<Block, Block>,
}

impl<'build> DominatorTreeBuilder<'build> {
    pub fn new(fn_def: &'build FnDef) -> Self {
        let mut len_blocks = fn_def.preds.len();

        let mut dfnum = IndexMap::<Block, usize>::with_capacity(len_blocks);
        let mut bucket = IndexMap::<Block, HashSet<Block>>::with_capacity(len_blocks);

        for _ in 0..len_blocks {
            dfnum.push(0);
            bucket.push(Default::default());
        }

        len_blocks -= 1;

        Self {
            fn_def,
            dfnum,
            ancestors: HashMap::with_capacity(len_blocks),
            idoms: HashMap::with_capacity(len_blocks),
            samedoms: HashMap::with_capacity(len_blocks),
            vertices: Vec::with_capacity(len_blocks),
            parents: HashMap::with_capacity(len_blocks),
            bucket,
            semidoms: HashMap::with_capacity(len_blocks),
        }
    }

    fn dfs(&mut self, parent: Option<Block>, block: Block) {
        if self.dfnum[block] == 0 {
            self.dfnum[block] = self.vertices.len();
            self.vertices.push(block);
            if let Some(pred) = parent {
                self.parents.insert(block, pred);
            }

            for &succ in self.fn_def.succs[block].iter() {
                self.dfs(Some(block), succ)
            }
        }
    }

    fn ancestor_with_lowest_semi(&self, mut block: Block) -> Block {
        let mut result = block;
        while let Some(&ancestor) = self.ancestors.get(&block) {
            if self.dfnum[self.semidoms[&block]] < self.dfnum[self.semidoms[&result]] {
                result = block;
            }
            block = ancestor;
        }
        result
    }

    pub fn build(mut self) -> HashMap<Block, Block> {
        self.dfs(None, self.fn_def.entry);

        for &block in self.vertices.iter().skip(1).rev() {
            let parent = self.parents[&block];
            let mut semidom = parent;

            for &pred in self.fn_def.preds[block].iter() {
                let semi_prime = if self.dfnum[pred] <= self.dfnum[block] {
                    pred
                } else {
                    let ancestor = self.ancestor_with_lowest_semi(pred);
                    self.semidoms[&ancestor]
                };

                if self.dfnum[semi_prime] < self.dfnum[semidom] {
                    semidom = semi_prime;
                }
            }

            self.semidoms.insert(block, semidom);
            self.bucket[semidom].insert(block);

            self.ancestors.insert(block, parent);

            for &v in self.bucket[parent].iter() {
                let y = self.ancestor_with_lowest_semi(v);
                if self.semidoms[&y] == self.semidoms[&v] {
                    self.idoms.insert(v, parent);
                } else {
                    self.samedoms.insert(v, y);
                }
            }
            self.bucket[parent].clear();
        }

        for block in self.vertices.iter().skip(1) {
            if let Some(samedom) = self.samedoms.get(block) {
                self.idoms.insert(*block, self.idoms[samedom]);
            }
        }

        dbg!(self.idoms)
    }
}
