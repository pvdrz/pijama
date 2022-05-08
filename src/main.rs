mod asm;

use std::{error::Error as StdError, fs::File, io::BufWriter};

use asm::{Assembler, InstructionKind, Register};
use object::{
    write::{Object, SectionId, StandardSection, SymbolSection},
    Architecture, BinaryFormat, Endianness, SymbolFlags, SymbolKind, SymbolScope,
};

fn main() -> Result<(), Box<dyn StdError>> {
    let file = BufWriter::new(File::create("./lib_2.o")?);

    // We know which kind of object we're going to emit thanks to `file lib.o`.
    let mut obj = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);

    // Create the `.text` section.
    let section = obj.section_id(StandardSection::Text);

    add_function(
        &mut obj,
        section,
        b"start",
        &[
            0x55, 0x48, 0x89, 0xe5, 0xb8, 0x0a, 0x00, 0x00, 0x00, 0x5d, 0xc3, 0x0f, 0x1f, 0x44,
            0x00, 0x00,
        ],
    );

    add_function(
        &mut obj,
        section,
        b"duplicate",
        &[
            0x55, 0x48, 0x89, 0xe5, 0x89, 0x7d, 0xfc, 0x8b, 0x45, 0xfc, 0xc1, 0xe0, 0x01, 0x5d,
            0xc3,
        ],
    );

    let mut assembler = Assembler::default();

    for &dst in Register::ALL {
        assembler.assemble_instruction(InstructionKind::LoadImm {
            src: 0xdeadbeef,
            dst,
        });
    }

    add_function(&mut obj, section, b"asm_test", &assembler.emit_code());

    let mut assembler = Assembler::default();

    for &base in Register::ALL {
        for &dst in Register::ALL {
            assembler.assemble_instruction(InstructionKind::LoadAddr {
                src: asm::Address {
                    base,
                    offset: 0xbeef,
                },
                dst,
            });
        }
    }

    add_function(&mut obj, section, b"loada_test", &assembler.emit_code());

    let mut assembler = Assembler::default();

    for &src in Register::ALL {
        for &base in Register::ALL {
            assembler.assemble_instruction(InstructionKind::Store {
                src,
                dst: asm::Address {
                    base,
                    offset: 0xbeef,
                },
            });
        }
    }

    add_function(&mut obj, section, b"store_test", &assembler.emit_code());

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
