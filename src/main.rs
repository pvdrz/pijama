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
    ancestor: HashMap<Block, Block>,
    idom: HashMap<Block, Block>,
    samedom: HashMap<Block, Block>,
    vertex: Vec<Block>,
    parent: HashMap<Block, Block>,
    bucket: IndexMap<Block, HashSet<Block>>,
    semi: HashMap<Block, Block>,
}
impl<'build> DominatorTreeBuilder<'build> {
    fn new(fn_def: &'build FnDef) -> Self {
        let len_blocks = fn_def.preds.len();

        let mut dfnum = IndexMap::<Block, usize>::with_capacity(len_blocks);
        let mut bucket = IndexMap::<Block, HashSet<Block>>::with_capacity(len_blocks);

        for _ in 0..len_blocks {
            dfnum.push(0);
            bucket.push(Default::default());
        }

        Self {
            fn_def,
            dfnum,
            ancestor: HashMap::with_capacity(len_blocks - 1),
            idom: HashMap::with_capacity(len_blocks - 1),
            samedom: HashMap::with_capacity(len_blocks - 1),
            vertex: Vec::with_capacity(len_blocks),
            parent: HashMap::with_capacity(len_blocks - 1),
            bucket,
            semi: HashMap::with_capacity(len_blocks - 1),
        }
    }

    fn dfs(&mut self, pred: Option<Block>, block: Block) {
        if self.dfnum[block] == 0 {
            self.dfnum[block] = self.vertex.len();
            self.vertex.push(block);
            if let Some(pred) = pred {
                self.parent.insert(block, pred);
            }

            for &succ in self.fn_def.succs[block].iter() {
                self.dfs(Some(block), succ)
            }
        }
    }

    fn ancestor_with_lowest_semi(&self, mut v: Block) -> Block {
        let mut u = v;
        while let Some(&a) = self.ancestor.get(&v) {
            if self.dfnum[self.semi[&v]] < self.dfnum[self.semi[&u]] {
                u = v;
            }
            v = a;
        }
        u
    }

    fn build(mut self) -> HashMap<Block, Block> {
        self.dfs(None, self.fn_def.entry);

        for &n in self.vertex.iter().skip(1).rev() {
            let p = self.parent[&n];
            let mut s = p;

            for &v in self.fn_def.preds[n].iter() {
                let s_prime = if self.dfnum[v] <= self.dfnum[n] {
                    v
                } else {
                    let a = self.ancestor_with_lowest_semi(v);
                    self.semi[&a]
                };

                if self.dfnum[s_prime] < self.dfnum[s] {
                    s = s_prime;
                }
            }

            self.semi.insert(n, s);
            self.bucket[s].insert(n);

            self.ancestor.insert(n, p);

            for &v in self.bucket[p].iter() {
                let y = self.ancestor_with_lowest_semi(v);
                if self.semi[&y] == self.semi[&v] {
                    self.idom.insert(v, p);
                } else {
                    self.samedom.insert(v, y);
                }
            }
            self.bucket[p].clear();
        }

        for n in self.vertex.iter().skip(1) {
            if let Some(m) = self.samedom.get(n) {
                self.idom.insert(*n, self.idom[m]);
            }
        }

        dbg!(self.idom)
    }
}
