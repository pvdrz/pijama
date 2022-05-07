use std::{error::Error as StdError, fs::File, io::BufWriter};

use object::{
    write::{Object, SymbolSection},
    Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};

fn main() -> Result<(), Box<dyn StdError>> {
    let file = BufWriter::new(File::create("./lib_2.o")?);

    // We know which kind of object we're going to emit thanks to `file lib.o`.
    let mut obj = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);

    // Create the `.text` section.
    let section = obj.add_section(vec![], b".text".to_vec(), SectionKind::Text);

    // The actual machine code we want to write
    let code = &[
        0x55, 0x48, 0x89, 0xe5, 0xb8, 0x0a, 0x00, 0x00, 0x00, 0x5d, 0xc3,
    ];

    // All this info is obtained from examining the `lib.o` file.
    // The `GLOBAL` binding flag.
    let bind = 1;
    // The `FUNC` type flag.
    let ty = 2;
    // The `DEFAULT` visibility flag.
    let vis = 0;

    let symbol = object::write::Symbol {
        name: b"start".to_vec(),
        value: 0,
        size: code.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::Elf {
            st_info: (bind << 4) + (ty & 0xf),
            st_other: vis & 0x3,
        },
    };

    let symbol_id = obj.add_symbol(symbol);

    obj.add_symbol_data(symbol_id, section, code, 16);

    // Write the object file.
    obj.write_stream(file)?;

    Ok(())
}
