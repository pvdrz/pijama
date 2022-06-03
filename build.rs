use std::{
    ffi::OsStr,
    fs::{create_dir, read_dir},
    path::PathBuf,
    process::Command,
};

fn main() -> Result<(), std::io::Error> {
    println!("cargo:rerun-if-changed=tests/asm/src");

    let out = PathBuf::from("tests/asm/out");

    if !out.exists() {
        create_dir(&out)?;
    }

    for result in read_dir("tests/asm/src")? {
        let path = result?.path();

        if path.extension() == Some(OsStr::new("asm")) {
            let file_name = path.file_name().unwrap();
            let mut out = out.clone();
            out.push(file_name);

            Command::new("nasm")
                .arg(&path)
                .arg("-O0")
                .arg("-o")
                .arg(out.with_extension("out"))
                .output()?;
        }
    }

    Ok(())
}
