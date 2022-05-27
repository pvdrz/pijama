use pijama::{
    asm::{Assembler, Register},
    code,
};

use std::fmt::Write;

const REGISTERS: [Register; 8] = [
    Register::Ax,
    Register::Cx,
    Register::Dx,
    Register::Bx,
    Register::Sp,
    Register::Bp,
    Register::Si,
    Register::Di,
];

const DEADBEEF32: i32 = 0xdeadbeefu32 as i32;
const DEADBEEF64: i64 = 0xdeadbeefdeadbeefu64 as i64;

fn compare(expected: &[u8], found: &[u8]) {
    if expected != found {
        let len = expected.len().max(found.len());

        let mut lines1 = Vec::<String>::default();
        let mut lines2 = Vec::<String>::default();

        for i in 0..len {
            if i % 8 == 0 {
                if i % 16 == 0 {
                    lines1.last_mut().map(|s| {
                        *s = format!("{:08x} {s}", i - 16);
                        s.push('\n')
                    });
                    lines2.last_mut().map(|s| {
                        *s = format!("         {s}");
                        s.push('\n')
                    });

                    lines1.push(String::default());
                    lines2.push(String::default());
                } else {
                    lines1.last_mut().map(|s| s.push(' '));
                    lines2.last_mut().map(|s| s.push(' '));
                }
            }

            let buf1 = lines1.last_mut().unwrap();
            let buf2 = lines2.last_mut().unwrap();

            // Panic: writing to a string cannot fail.
            (|| match (expected.get(i), found.get(i)) {
                (None, None) => unreachable!(),
                (None, Some(found_byte)) => {
                    write!(buf1, "   ")?;
                    write!(buf2, " {:02x}", found_byte)
                }
                (Some(expected_byte), None) => {
                    write!(buf1, " {:02x}", expected_byte)?;
                    write!(buf2, "   ")
                }
                (Some(expected_byte), Some(found_byte)) => {
                    if expected_byte == found_byte {
                        write!(buf1, " {:02x}", expected_byte)?;
                        write!(buf2, "   ")
                    } else {
                        write!(buf1, " {:02x}", expected_byte)?;
                        write!(buf2, " {:02x}", found_byte)
                    }
                }
            })()
            .unwrap()
        }

        let res = len % 16;

        if res != 0 {
            lines1.last_mut().map(|s| {
                *s = format!("{:08x} {s}", len - res);
                s.push('\n')
            });
            lines2.last_mut().map(|s| {
                *s = format!("         {s}");
                s.push('\n')
            });
        }

        let output = lines1
            .into_iter()
            .zip(lines2.into_iter())
            .flat_map(|(x, y)| [x, y])
            .collect::<String>();

        panic!("output mismatch:\n{output}")
    }
}

#[test]
fn loadi() {
    let expected_bytes = include_bytes!("loadi.out");

    let mut asm = Assembler::default();

    for dst in REGISTERS {
        asm.assemble_instruction(code!(loadi { DEADBEEF64 }, { dst }));
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn load() {
    let expected_bytes = include_bytes!("load.out");

    let mut asm = Assembler::default();

    for base in REGISTERS {
        for dst in REGISTERS {
            asm.assemble_instruction(code!(load { base } + { DEADBEEF32 }, { dst }));
        }
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn store() {
    let expected_bytes = include_bytes!("store.out");

    let mut asm = Assembler::default();

    for src in REGISTERS {
        for dst in REGISTERS {
            asm.assemble_instruction(code!(store { src }, { dst } + { DEADBEEF32 }));
        }
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn push() {
    let expected_bytes = include_bytes!("push.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(code!(push { reg }));
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn pop() {
    let expected_bytes = include_bytes!("pop.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(code!(pop { reg }));
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn add() {
    let expected_bytes = include_bytes!("add.out");

    let mut asm = Assembler::default();

    for src in REGISTERS {
        for dst in REGISTERS {
            asm.assemble_instruction(code!(add { src }, { dst }));
        }
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn addi() {
    let expected_bytes = include_bytes!("addi.out");

    let mut asm = Assembler::default();

    for dst in REGISTERS {
        asm.assemble_instruction(code!(addi { DEADBEEF32 }, { dst }));
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn jmp() {
    let expected_bytes = include_bytes!("jmp.out");

    let mut asm = Assembler::default();

    asm.assemble_instruction(code!(jmp { DEADBEEF32 }));
    asm.assemble_instruction(code!(jmp { DEADBEEF32 }));

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn je() {
    let expected_bytes = include_bytes!("je.out");

    let mut asm = Assembler::default();

    for reg1 in REGISTERS {
        for reg2 in REGISTERS {
            asm.assemble_instruction(code!(je { reg1 }, { reg2 }, { DEADBEEF32 }));
        }
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn jl() {
    let expected_bytes = include_bytes!("jl.out");

    let mut asm = Assembler::default();

    for reg1 in REGISTERS {
        for reg2 in REGISTERS {
            asm.assemble_instruction(code!(jl { reg1 }, { reg2 }, { DEADBEEF32 }));
        }
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn jg() {
    let expected_bytes = include_bytes!("jg.out");

    let mut asm = Assembler::default();

    for reg1 in REGISTERS {
        for reg2 in REGISTERS {
            asm.assemble_instruction(code!(jg { reg1 }, { reg2 }, { DEADBEEF32 }));
        }
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn ret() {
    let expected_bytes = include_bytes!("ret.out");

    let mut asm = Assembler::default();

    asm.assemble_instruction(code!(ret));

    compare(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn call() {
    let expected_bytes = include_bytes!("call.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(code!(call { reg }));
    }

    compare(expected_bytes, asm.emit_code().as_slice());
}
