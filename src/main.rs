mod asm;
mod dataflow;
mod dominance_frontiers;
mod index;
mod mir;
mod x86;

use asm::{Address, BaseAddr, Instruction, InstructionKind, Label, Register};
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
    let mut graph = mir::example();
    println!("Control-Flow Graph:");
    graph.dump();
    graph.graphviz("./graph.dot").unwrap();

    println!("\nReaching definitions:");
    dataflow::ReachingDefs::new(&graph).run();
    println!("\nLive variables:");
    dataflow::LiveVariable::new(&graph).run();
    println!("\nDominators:");
    dataflow::Dominators::new(&graph).run();

    place_phi_fns(&mut graph);
    println!("\nSSA Control-Flow Graph:");
    graph.dump();
    graph.graphviz("./graph_ssa.dot").unwrap();

    println!("\nReaching definitions:");
    dataflow::ReachingDefs::new(&graph).run();
    println!("\nLive variables:");
    dataflow::LiveVariable::new(&graph).run();
    println!("\nDominators:");
    dataflow::Dominators::new(&graph).run();

    dead_code(&mut graph);
    println!("\nSSA Control-Flow Graph after DCE:");
    graph.dump();
    graph.graphviz("./graph_ssa_dce.dot").unwrap();

    println!("\nReaching definitions:");
    dataflow::ReachingDefs::new(&graph).run();
    println!("\nLive variables:");
    dataflow::LiveVariable::new(&graph).run();
    println!("\nDominators:");
    dataflow::Dominators::new(&graph).run();

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

use index::IndexMap;
use mir::Block;
use mir::FnDef;
use mir::Local;
use mir::Rvalue;
use mir::Statement;
use std::collections::{BTreeMap, BTreeSet, HashMap};

fn place_phi_fns(fn_def: &mut FnDef) {
    let (frontiers, domtree) = dominance_frontiers::dominance_frontiers(fn_def);
    let mut local_defs = IndexMap::<Local, Vec<Block>>::repeat(Vec::new, fn_def.locals.len());

    for (block, data) in fn_def.blocks.iter() {
        for stmt in &data.statements {
            match stmt {
                &Statement::Assign { lhs, .. } => {
                    let defs_lhs = &mut local_defs[lhs];

                    if let Err(index) = defs_lhs.binary_search(&block) {
                        defs_lhs.insert(index, block);
                    }
                }
                Statement::Nop => {}
            }
        }
    }

    for local in fn_def.locals.keys() {
        let mut worklist = local_defs[local].clone();
        let mut phi_uses = vec![];

        while let Some(block) = worklist.pop() {
            for &block_frontier in &frontiers[&block] {
                if let Err(index) = phi_uses.binary_search(&block_frontier) {
                    let values = fn_def.preds[block_frontier]
                        .iter()
                        .map(|&pred| (pred, local))
                        .collect();
                    fn_def.blocks[block_frontier].statements.insert(
                        0,
                        Statement::Assign {
                            lhs: local,
                            rhs: Rvalue::Phi(values),
                        },
                    );

                    phi_uses.insert(index, block_frontier);
                    if local_defs[local].binary_search(&block_frontier).is_err() {
                        if let Err(index) = worklist.binary_search(&block_frontier) {
                            worklist.insert(index, block_frontier);
                        }
                    }
                }
            }
        }
    }

    rename(fn_def, &domtree);
}

fn rename(fn_def: &mut FnDef, domtree: &IndexMap<Block, Vec<Block>>) {
    let locals_len = fn_def.locals.len();
    let mut stack = IndexMap::<Local, Vec<Local>>::with_capacity(locals_len);

    for local in fn_def.locals.keys() {
        stack.push(vec![local]);
    }

    aux(fn_def.entry, fn_def, &mut stack, domtree)
}

fn aux(
    n: Block,
    fn_def: &mut FnDef,
    stack: &mut IndexMap<Local, Vec<Local>>,
    domtree: &IndexMap<Block, Vec<Block>>,
) {
    let mut trim: HashMap<Local, usize> = HashMap::new();

    if let Some(data) = fn_def.blocks.get_mut(n) {
        for stmt in &mut data.statements {
            match stmt {
                Statement::Assign { lhs, rhs } => {
                    match rhs {
                        Rvalue::Use(op) => match op {
                            mir::Operand::Literal(_) => {}
                            mir::Operand::Local(local) => {
                                *local = *stack[*(local)].last().unwrap();
                            }
                        },
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            match lhs {
                                mir::Operand::Literal(_) => {}
                                mir::Operand::Local(local) => {
                                    *local = *stack[*local].last().unwrap();
                                }
                            }
                            match rhs {
                                mir::Operand::Literal(_) => {}
                                mir::Operand::Local(local) => {
                                    *local = *stack[*local].last().unwrap();
                                }
                            }
                        }
                        Rvalue::Phi(_) => {}
                    }

                    *trim.entry(*lhs).or_default() += 1;

                    let lhs_ty = fn_def.locals[*lhs].clone();
                    let new_lhs = fn_def.locals.push(lhs_ty);
                    stack[*lhs].push(new_lhs);
                    *lhs = new_lhs;
                }
                Statement::Nop => {}
            }
        }

        match &mut data.terminator {
            mir::Terminator::Jump(_) => {}
            mir::Terminator::JumpIf { cond, .. } => match cond {
                mir::Operand::Literal(_) => {}
                mir::Operand::Local(local) => {
                    *local = *stack[*local].last().unwrap();
                }
            },
            mir::Terminator::Return(local) => {
                *local = *stack[*local].last().unwrap();
            }
        }

        for y in fn_def.succs[n].iter() {
            if let Some(data) = fn_def.blocks.get_mut(*y) {
                for s in &mut data.statements {
                    match s {
                        Statement::Assign { rhs, .. } => match rhs {
                            Rvalue::Phi(values) => {
                                for (block, local) in values {
                                    if *block == n {
                                        *local = *stack[*local].last().unwrap();
                                    }
                                }
                            }
                            _ => {}
                        },
                        Statement::Nop => {}
                    }
                }
            }
        }
    }

    for &x in domtree[n].iter() {
        aux(x, fn_def, stack, domtree);
    }

    for (local, trim) in trim {
        let stack = &mut stack[local];
        let trim = stack.len() - trim;
        stack.truncate(trim);
    }
}

fn dead_code(fn_def: &mut FnDef) {
    let mut local_uses = IndexMap::repeat(BTreeSet::new, fn_def.locals.len());
    let mut used_locals: BTreeMap<(Block, usize), Vec<Local>> = BTreeMap::new();
    let mut defsites: BTreeMap<Local, (Block, usize)> = BTreeMap::new();
    let mut worklist = Vec::new();

    for (block, data) in fn_def.blocks.iter() {
        for (index, stmt) in data.statements.iter().enumerate() {
            match stmt {
                Statement::Assign { lhs, rhs } => {
                    worklist.push(*lhs);
                    defsites.insert(*lhs, (block, index));

                    match rhs {
                        Rvalue::Use(op) => match op {
                            mir::Operand::Literal(_) => {}
                            mir::Operand::Local(local) => {
                                local_uses[*local].insert((block, index));
                                used_locals.entry((block, index)).or_default().push(*local);
                            }
                        },
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            match lhs {
                                mir::Operand::Literal(_) => {}
                                mir::Operand::Local(local) => {
                                    local_uses[*local].insert((block, index));
                                    used_locals.entry((block, index)).or_default().push(*local);
                                }
                            }
                            match rhs {
                                mir::Operand::Literal(_) => {}
                                mir::Operand::Local(local) => {
                                    local_uses[*local].insert((block, index));
                                    used_locals.entry((block, index)).or_default().push(*local);
                                }
                            }
                        }
                        Rvalue::Phi(values) => {
                            for &(_, local) in values {
                                local_uses[local].insert((block, index));
                                used_locals.entry((block, index)).or_default().push(local);
                            }
                        }
                    }
                }
                Statement::Nop => {}
            }

            match &data.terminator {
                mir::Terminator::Jump(_) => {}
                mir::Terminator::JumpIf { cond, .. } => match cond {
                    mir::Operand::Literal(_) => {}
                    mir::Operand::Local(local) => {
                        local_uses[*local].insert((block, data.statements.len()));
                        used_locals.entry((block, index)).or_default().push(*local);
                    }
                },
                mir::Terminator::Return(local) => {
                    local_uses[*local].insert((block, data.statements.len()));
                    used_locals.entry((block, index)).or_default().push(*local);
                }
            }
        }
    }

    while let Some(v) = worklist.pop() {
        if let Some(site) = defsites.get(&v) {
            if local_uses[v].is_empty() {
                fn_def.blocks[site.0].statements[site.1] = Statement::Nop;
                for xi in used_locals.remove(site).unwrap() {
                    local_uses[xi].remove(site);
                    worklist.push(xi);
                }
            }
        }
    }
}
