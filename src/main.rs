mod asm;
mod dataflow;
mod dominator_tree;
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
    dominator_tree::DominatorTreeBuilder::new(&graph).build();

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
