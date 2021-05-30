mod asm;
mod dataflow;
mod index;
mod mir;
mod x86;

use std::collections::{BTreeMap, HashMap, HashSet};

use asm::{Address, BaseAddr, Instruction, InstructionKind, Label, Register};
use index::IndexMap;
use mir::{Block, BlockData, FnDef};
use x86::Assembler;

use object::{
    write::{Object, StandardSection, Symbol, SymbolSection},
    Architecture, BinaryFormat, Endianness,
};

fn example() -> Vec<Instruction> {
    let start = Label(0);
    let done = Label(1);
    vec![
        // set `res` to `0`
        Instruction {
            label: None,
            kind: InstructionKind::LoadImm {
                src: 0x0,
                dst: Register::Ax,
            },
        },
        // Jump to `done` if `x <= 0`
        Instruction {
            label: Some(start),
            kind: InstructionKind::JumpLez {
                reg: Register::Di,
                addr: Address {
                    base: BaseAddr::Lab(done),
                    offset: 0,
                },
            },
        },
        // Set `res` to `x + res`
        Instruction {
            label: None,
            kind: InstructionKind::Add {
                src: Register::Di,
                dst: Register::Ax,
            },
        },
        // Set `x` to `x - 1`
        Instruction {
            label: None,
            kind: InstructionKind::AddImm {
                src: -0x1,
                dst: Register::Di,
            },
        },
        // Jump to `start`
        Instruction {
            label: None,
            kind: InstructionKind::Jump(Address {
                base: BaseAddr::Lab(start),
                offset: 0,
            }),
        },
        // Return
        Instruction {
            label: Some(done),
            kind: InstructionKind::Return,
        },
    ]
}

fn main() {
    let graph = mir::example();
    println!("Control-Flow Graph:");
    graph.dump();
    graph.graphviz().unwrap();

    println!("\nReaching definitions:");
    dataflow::ReachingDefs::new(&graph).run();
    println!("\nLive variables:");
    dataflow::LiveVariable::new(&graph).run();
    println!("\nDominators:");
    dataflow::Dominators::new(&graph).run();
    println!("\nDominator Tree:");
    DominatorTreeBuilder::new(&graph).build();

    let asm = example();

    println!("\nPseudo-Assembly:");
    for ins in &asm {
        println!("{}", ins);
    }

    let code = Assembler::assemble_code(asm);

    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);

    let text_id = object.section_id(StandardSection::Text);
    let abc_id = object.add_symbol(Symbol {
        name: b"sum".to_vec(),
        value: 0,
        size: code.len() as u64,
        kind: object::SymbolKind::Text,
        scope: object::SymbolScope::Dynamic,
        weak: false,
        section: SymbolSection::Section(text_id),
        flags: object::SymbolFlags::Elf {
            st_info: (2 & 0xff) | (1 << 4), // STT_FUNC + STB_GLOBAL
            st_other: 0,
        },
    });

    object.add_symbol_data(abc_id, text_id, &code, 16);

    let bytes = object.write().unwrap();
    std::fs::write("./test.o", &bytes).unwrap();
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

    fn build(mut self) -> HashMap<Block, Block> {
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
