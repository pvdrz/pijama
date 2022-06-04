use crate::mir::bb::{BasicBlock, BasicBlockId};
use crate::mir::{Local, Ty};
use std::collections::BTreeMap;

pub struct Function {
    pub args_len: usize,
    pub basic_blocks: BTreeMap<BasicBlockId, BasicBlock>,
    pub local_types: BTreeMap<Local, Ty>,
}

impl Function {
    pub fn builder(args_len: usize) -> FunctionBuilder {
        FunctionBuilder {
            args_len,
            basic_blocks: BTreeMap::default(),
            local_types: BTreeMap::default(),
        }
    }
}

pub struct FunctionBuilder {
    args_len: usize,
    basic_blocks: BTreeMap<BasicBlockId, Option<BasicBlock>>,
    local_types: BTreeMap<Local, Ty>,
}

impl FunctionBuilder {
    #[must_use]
    pub fn add_block(&mut self) -> BasicBlockId {
        let bb = BasicBlockId(self.basic_blocks.len());
        self.basic_blocks.insert(bb, None);
        bb
    }

    #[must_use]
    pub fn add_local(&mut self, ty: Ty) -> Local {
        let local = Local(self.local_types.len());
        self.local_types.insert(local, ty);
        local
    }

    #[must_use]
    pub fn block_mut(&mut self, bb: BasicBlockId) -> &mut Option<BasicBlock> {
        self.basic_blocks.get_mut(&bb).unwrap()
    }

    pub fn finish(self) -> Function {
        if self.local_types.len() + 1 < self.args_len || self.local_types.is_empty() {
            panic!("not enough locals")
        }

        Function {
            args_len: self.args_len,
            basic_blocks: self
                .basic_blocks
                .into_iter()
                .map(|(k, v)| (k, v.expect("missing block")))
                .collect(),
            local_types: self.local_types,
        }
    }
}
