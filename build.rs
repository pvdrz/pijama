use std::{
    ffi::OsStr,
    fs::{create_dir, read_dir},
    io,
    path::{Path, PathBuf},
    process::Command,
};

fn assemble_tests(
    platform: &'static str,
    tests_path: &Path,
    output_path: &Path,
) -> Result<(), io::Error> {
    let src_path = tests_path.join(platform);

    println!("cargo:rerun-if-changed={}", src_path.display());

    if !output_path.exists() {
        create_dir(&output_path)?;
    }

    for result in read_dir(src_path)? {
        let file_path = result?.path();

        if file_path.extension() == Some(OsStr::new("asm")) {
            let file_name = file_path.file_name().unwrap();
            let out = output_path.join(file_name).with_extension(platform);

            Command::new("nasm")
                .arg(&file_path)
                .arg("-O0")
                .arg("-o")
                .arg(out)
                .output()?;
        }
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let tests_path = PathBuf::from_iter(["tests", "asm"]);
    let output_path = tests_path.join("out");

    assemble_tests("x86_64", &tests_path, &output_path)?;

    Ok(())
}
