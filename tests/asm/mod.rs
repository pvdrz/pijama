use pijama::{
    asm::{
        x86_64::{assemble, Register},
        Instructions,
    },
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
                    if let Some(s) = lines1.last_mut() {
                        *s = format!("{:08x} {s}", i - 16);
                        s.push('\n')
                    }
                    if let Some(s) = lines2.last_mut() {
                        *s = format!("         {s}");
                        s.push('\n')
                    }

                    lines1.push(String::default());
                    lines2.push(String::default());
                } else {
                    if let Some(s) = lines1.last_mut() {
                        s.push(' ')
                    }
                    if let Some(s) = lines2.last_mut() {
                        s.push(' ')
                    }
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
            if let Some(s) = lines1.last_mut() {
                *s = format!("{:08x} {s}", len - res);
                s.push('\n')
            }
            if let Some(s) = lines2.last_mut() {
                *s = format!("         {s}");
                s.push('\n')
            }
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
    let expected_bytes = include_bytes!("out/loadi.out");

    let mut instructions = Instructions::new();

    for dst in REGISTERS {
        instructions.add_instruction(code!(loadi { DEADBEEF64 }, { dst }));
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn load() {
    let expected_bytes = include_bytes!("out/load.out");

    let mut instructions = Instructions::new();

    for base in REGISTERS {
        for dst in REGISTERS {
            instructions.add_instruction(code!(load { base } + { DEADBEEF32 }, { dst }));
        }
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn store() {
    let expected_bytes = include_bytes!("out/store.out");

    let mut instructions = Instructions::new();

    for src in REGISTERS {
        for dst in REGISTERS {
            instructions.add_instruction(code!(store { src }, { dst } + { DEADBEEF32 }));
        }
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}
#[test]
fn mov() {
    let expected_bytes = include_bytes!("out/mov.out");

    let mut instructions = Instructions::new();

    for src in REGISTERS {
        for dst in REGISTERS {
            instructions.add_instruction(code!(mov { src }, { dst }));
        }
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn push() {
    let expected_bytes = include_bytes!("out/push.out");

    let mut instructions = Instructions::new();

    for reg in REGISTERS {
        instructions.add_instruction(code!(push { reg }));
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn pop() {
    let expected_bytes = include_bytes!("out/pop.out");

    let mut instructions = Instructions::new();

    for reg in REGISTERS {
        instructions.add_instruction(code!(pop { reg }));
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn add() {
    let expected_bytes = include_bytes!("out/add.out");

    let mut instructions = Instructions::new();

    for src in REGISTERS {
        for dst in REGISTERS {
            instructions.add_instruction(code!(add { src }, { dst }));
        }
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn addi() {
    let expected_bytes = include_bytes!("out/addi.out");

    let mut instructions = Instructions::new();

    for dst in REGISTERS {
        instructions.add_instruction(code!(addi { DEADBEEF32 }, { dst }));
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn slt() {
    let expected_bytes = include_bytes!("out/slt.out");

    let mut instructions = Instructions::new();

    for src1 in REGISTERS {
        for src2 in REGISTERS {
            for dst in REGISTERS {
                instructions.add_instruction(code!(slt { src1 }, { src2 }, { dst }));
            }
        }
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn jmp() {
    let expected_bytes = include_bytes!("out/jmp.out");

    let mut instructions = Instructions::new();

    instructions.add_instruction(code!(jmp { DEADBEEF32 }));
    instructions.add_instruction(code!(jmp { DEADBEEF32 }));

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn jz() {
    let expected_bytes = include_bytes!("out/jz.out");

    let mut instructions = Instructions::new();

    for reg in REGISTERS {
        instructions.add_instruction(code!(jz { reg }, { DEADBEEF32 }));
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn ret() {
    let expected_bytes = include_bytes!("out/ret.out");

    let mut instructions = Instructions::new();

    instructions.add_instruction(code!(ret));

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}

#[test]
fn call() {
    let expected_bytes = include_bytes!("out/call.out");

    let mut instructions = Instructions::new();

    for reg in REGISTERS {
        instructions.add_instruction(code!(call { reg }));
    }

    let mut buf = Vec::new();
    assemble(instructions, &mut buf).unwrap();
    compare(expected_bytes, &buf);
}
