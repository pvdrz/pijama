use std::{ffi::OsStr, fs::read_dir, path::PathBuf, process::Command};

fn main() -> Result<(), std::io::Error> {
    println!("cargo:rerun-if-changed=tests/asm/src");

    for result in read_dir("tests/asm/src")? {
        let path = result?.path();

        if path.extension() == Some(OsStr::new("asm")) {
            let mut out = PathBuf::from("tests/asm/");

            let file_name = path.file_name().unwrap();
            out.push(file_name);

            Command::new("nasm")
                .arg(&path)
                .arg("-o")
                .arg(out.with_extension("out"))
                .output()?;
        }
    }

    Ok(())
}
