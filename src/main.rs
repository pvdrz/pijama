use std::{error::Error as StdError, fs::File, io::BufWriter};

use object::{
    write::{Object, SectionId, StandardSection, SymbolSection},
    Architecture, BinaryFormat, Endianness, SymbolFlags, SymbolKind, SymbolScope,
};
use pijama::asm::Assembler;
use pijama::code;
use pijama::mir::{
    BasicBlock, BinOp, Function, Literal, Operand, Rvalue, Statement, Terminator, Ty,
};

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

    const fn int(data: u64) -> Operand {
        Operand::Constant(Literal { data, ty: Ty::Int })
    }

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
        statements: vec![
        ],
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

    let mut asm = Assembler::default();
    asm.assemble_instruction(code!(loadi {0xa}, {rax}));
    asm.assemble_instruction(code!(ret));
    add_function(&mut obj, section, b"start", &asm.emit_code());

    let mut asm = Assembler::default();
    let end = asm.add_label();
    let cmp = asm.add_label();

    asm.assemble_instruction(code! {      loadi {0x0},{rax} });
    asm.assemble_instruction(code! {      loadi {0x0},{rdx} });

    asm.assemble_instruction(code! { cmp: slt {rdx},{rdi},{rcx} });
    asm.assemble_instruction(code! {      jz  {rcx}, {end} });

    asm.assemble_instruction(code! {      addi {0x2},{rax} });
    asm.assemble_instruction(code! {      addi {0x1},{rdx} });
    asm.assemble_instruction(code! {      jmp  {cmp} });

    asm.assemble_instruction(code! { end: ret });

    add_function(&mut obj, section, b"duplicate", &asm.emit_code());

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
