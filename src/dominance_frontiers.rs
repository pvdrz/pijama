use crate::index::IndexMap;
use crate::mir::{Block, FnDef};

use std::collections::{HashMap, HashSet};

pub fn dominance_frontiers(fn_def: &FnDef) -> HashMap<Block, Vec<Block>> {
    let mut domtree_builder = DominatorTreeBuilder::new(fn_def);
    let domtree = domtree_builder.build();

    let frontiers = DominanceFrontierBuilder::new(fn_def, &domtree_builder.idoms, &domtree).build();
    for (block, frontier) in &frontiers {
        println!("{:?}; {:?}", block, frontier);
    }
    frontiers
}

struct DominatorTreeBuilder<'build> {
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
    fn new(fn_def: &'build FnDef) -> Self {
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

    pub fn build(&mut self) -> IndexMap<Block, Vec<Block>> {
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

        let mut domtree = IndexMap::repeat(Vec::new, self.vertices.len());

        for (&block, &idom) in self.idoms.iter() {
            domtree[idom].push(block);
        }

        domtree
    }
}

struct DominanceFrontierBuilder<'build> {
    fn_def: &'build FnDef,
    idoms: &'build HashMap<Block, Block>,
    domtree: &'build IndexMap<Block, Vec<Block>>,
    frontiers: HashMap<Block, Vec<Block>>,
}

impl<'build> DominanceFrontierBuilder<'build> {
    fn new(
        fn_def: &'build FnDef,
        idoms: &'build HashMap<Block, Block>,
        domtree: &'build IndexMap<Block, Vec<Block>>,
    ) -> Self {
        Self {
            fn_def,
            idoms,
            domtree,
            frontiers: HashMap::with_capacity(fn_def.preds.len()),
        }
    }

    fn dominates(&self, dom: Block, mut block: Block) -> bool {
        while let Some(&idom) = self.idoms.get(&block) {
            if dom == idom {
                return true;
            } else {
                block = idom;
            }
        }

        false
    }

    fn compute_frontier(&mut self, n: Block) {
        let mut s = Vec::new();

        for y in self.fn_def.succs[n].iter() {
            if self.idoms[y] != n {
                if let Err(index) = s.binary_search(y) {
                    s.insert(index, *y);
                }
            }
        }

        for &c in self.domtree[n].iter() {
            self.compute_frontier(c);
            for &w in self.frontiers[&c].iter() {
                if !self.dominates(n, w) {
                    if let Err(index) = s.binary_search(&w) {
                        s.insert(index, w);
                    }
                }
            }
        }

        self.frontiers.insert(n, s);
    }

    fn build(mut self) -> HashMap<Block, Vec<Block>> {
        self.compute_frontier(self.fn_def.entry);
        self.frontiers
    }
}
