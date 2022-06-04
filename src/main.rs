use std::{error::Error as StdError, fs::File, io::BufWriter};

use object::{
    write::{Object, SectionId, StandardSection, SymbolSection},
    Architecture, BinaryFormat, Endianness, SymbolFlags, SymbolKind, SymbolScope,
};
use pijama::asm::x86_64::assemble;
use pijama::mir::{
    BasicBlock, BinOp, Function, Literal, Operand, Rvalue, Statement, Terminator, Ty,
};

const fn int(data: u32) -> Operand {
    Operand::Constant(Literal { data, ty: Ty::Int })
}

fn start_mir() -> Function {
    let mut builder = Function::builder(0);

    let output = builder.add_local(Ty::Int);

    let bb0 = builder.add_block();

    *builder.block_mut(bb0) = Some(BasicBlock {
        statements: vec![Statement::Assign {
            lhs: output,
            rhs: Rvalue::Use(int(10)),
        }],
        terminator: Terminator::Return,
    });

    builder.finish()
}

fn duplicate_mir() -> Function {
    let mut builder = Function::builder(1);

    let output = builder.add_local(Ty::Int);
    let arg = builder.add_local(Ty::Int);
    let i = builder.add_local(Ty::Int);
    let cmp = builder.add_local(Ty::Bool);

    let bb0 = builder.add_block();
    let bb1 = builder.add_block();
    let bb2 = builder.add_block();
    let bb3 = builder.add_block();

    *builder.block_mut(bb0) = Some(BasicBlock {
        statements: vec![
            Statement::Assign {
                lhs: output,
                rhs: Rvalue::Use(int(0)),
            },
            Statement::Assign {
                lhs: i,
                rhs: Rvalue::Use(int(0)),
            },
        ],
        terminator: Terminator::Jump(bb1),
    });

    *builder.block_mut(bb1) = Some(BasicBlock {
        statements: vec![Statement::Assign {
            lhs: cmp,
            rhs: Rvalue::BinaryOp {
                op: BinOp::Lt,
                lhs: Operand::Local(i),
                rhs: Operand::Local(arg),
            },
        }],
        terminator: Terminator::JumpIf {
            cond: Operand::Local(cmp),
            then_bb: bb2,
            else_bb: bb3,
        },
    });

    *builder.block_mut(bb2) = Some(BasicBlock {
        statements: vec![
            Statement::Assign {
                lhs: output,
                rhs: Rvalue::BinaryOp {
                    op: BinOp::Add,
                    lhs: Operand::Local(output),
                    rhs: int(2),
                },
            },
            Statement::Assign {
                lhs: i,
                rhs: Rvalue::BinaryOp {
                    op: BinOp::Add,
                    lhs: Operand::Local(i),
                    rhs: int(1),
                },
            },
        ],
        terminator: Terminator::Jump(bb1),
    });

    *builder.block_mut(bb3) = Some(BasicBlock {
        statements: vec![],
        terminator: Terminator::Return,
    });

    builder.finish()
}

fn main() -> Result<(), Box<dyn StdError>> {
    let file = BufWriter::new(File::create("./lib_2.o")?);

    // We know which kind of object we're going to emit thanks to `file lib.o`.
    let mut obj = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);

    // Create the `.text` section.
    let section = obj.section_id(StandardSection::Text);

    let mut start_code = Vec::new();
    let mut start_instructions = pijama::mir_lowering::lower_function(&start_mir());
    start_instructions.optimize();
    assemble(start_instructions, &mut start_code)?;
    add_function(&mut obj, section, b"start", &start_code);

    let mut duplicate_code = Vec::new();
    let mut duplicate_instructions = pijama::mir_lowering::lower_function(&duplicate_mir());
    duplicate_instructions.optimize();
    assemble(duplicate_instructions, &mut duplicate_code)?;
    add_function(&mut obj, section, b"duplicate", &duplicate_code);

    // Write the object file.
    obj.write_stream(file)?;

    Ok(())
}

fn add_function(object: &mut Object, section: SectionId, name: &[u8], code: &[u8]) {
    // The `GLOBAL` binding flag.
    const BIND: u8 = 1;
    // The `FUNC` type flag.
    const TYPE: u8 = 2;
    // The `DEFAULT` visibility flag.
    const VIS: u8 = 0;

    // All this info is obtained from examining the `lib.o` file.
    let symbol = object::write::Symbol {
        name: name.to_vec(),
        // It seems that `object` ignores this value so we can leave it be zero.
        value: 0,
        size: code.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::Elf {
            st_info: (BIND << 4) + (TYPE & 0xf),
            st_other: VIS & 0x3,
        },
    };

    let symbol_id = object.add_symbol(symbol);

    object.add_symbol_data(symbol_id, section, code, 16);
}
