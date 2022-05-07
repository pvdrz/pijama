use std::{error::Error as StdError, fs::File};

use object::{write::Object, Architecture, BinaryFormat, Endianness};

fn main() -> Result<(), Box<dyn StdError>> {
    let file = File::create("./lib_2.o")?;

    // We know which kind of object we're going to emit thanks to `file lib.o`.
    let obj = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);

    // Write the object file.
    obj.write_stream(file)?;

    Ok(())
}
