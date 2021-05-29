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
    dominators(&graph);

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

fn dominators(fn_def: &FnDef) {
    let mut n = 0;

    let mut dfnum = IndexMap::<Block, usize>::new();
    let mut ancestor = HashMap::<Block, Block>::new();
    let mut idom = HashMap::<Block, Block>::new();
    let mut samedom = HashMap::<Block, Block>::new();
    let mut vertex = BTreeMap::<usize, Block>::new();
    let mut parent = HashMap::<Block, Block>::new();
    let mut bucket = IndexMap::<Block, HashSet<Block>>::new();
    let mut semi = HashMap::<Block, Block>::new();

    for _ in fn_def.preds.keys() {
        dfnum.push(0);
        bucket.push(Default::default());
    }

    dfs(
        None,
        fn_def.entry,
        fn_def,
        &mut dfnum,
        &mut vertex,
        &mut parent,
        &mut n,
    );

    for i in (1..n).rev() {
        let n = vertex[&i];
        let p = parent[&n];
        let mut s = p;

        for &v in fn_def.preds[n].iter() {
            let s_prime = if dfnum[v] <= dfnum[n] {
                v
            } else {
                let a = ancestor_with_lowest_semi(v, &ancestor, &dfnum, &semi);
                semi[&a]
            };

            if dfnum[s_prime] < dfnum[s] {
                s = s_prime;
            }
        }

        semi.insert(n, s);
        bucket[s].insert(n);

        ancestor.insert(n, p);

        for &v in bucket[p].iter() {
            let y = ancestor_with_lowest_semi(v, &ancestor, &dfnum, &semi);
            if semi[&y] == semi[&v] {
                idom.insert(v, p);
            } else {
                samedom.insert(v, y);
            }
        }
        bucket[p] = Default::default();
    }

    for i in 1..n {
        let n = vertex[&i];
        if let Some(m) = samedom.get(&n) {
            idom.insert(n, idom[m]);
        }
    }

    dbg!(idom);
}

fn dfs(
    pred: Option<Block>,
    block: Block,
    fn_def: &FnDef,
    dfnum: &mut IndexMap<Block, usize>,
    vertex: &mut BTreeMap<usize, Block>,
    parent: &mut HashMap<Block, Block>,
    n: &mut usize,
) {
    if dfnum[block] == 0 {
        dfnum[block] = *n;
        vertex.insert(*n, block);
        if let Some(pred) = pred {
            parent.insert(block, pred);
        }
        *n += 1;

        for &succ in fn_def.succs[block].iter() {
            dfs(Some(block), succ, fn_def, dfnum, vertex, parent, n)
        }
    }
}

fn ancestor_with_lowest_semi(
    mut v: Block,
    ancestor: &HashMap<Block, Block>,
    dfnum: &IndexMap<Block, usize>,
    semi: &HashMap<Block, Block>,
) -> Block {
    let mut u = v;
    while let Some(&a) = ancestor.get(&v) {
        if dfnum[semi[&v]] < dfnum[semi[&u]] {
            u = v;
        }
        v = a;
    }
    u
}
