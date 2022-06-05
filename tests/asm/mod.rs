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

macro_rules! asm_test {
    ($name:ident, $f:expr) => {
        #[test]
        fn $name() {
            let expected_bytes = include_bytes!(concat!("out/", stringify!($name), ".out"));

            let mut instructions = Instructions::<Register>::new();

            ($f)(&mut instructions);

            let mut bytes = Vec::new();
            assemble(instructions, &mut bytes).unwrap();
            compare(expected_bytes, &mut bytes);
        }
    };
}

asm_test!(loadi, |instructions: &mut Instructions<Register>| {
    for dst in REGISTERS {
        instructions.add_instruction(code!(loadi { DEADBEEF64 }, { dst }));
    }
    for dst in REGISTERS {
        instructions.add_instruction(code!(loadi { 0x0 }, { dst }));
    }
    for dst in REGISTERS {
        instructions.add_instruction(code!(loadi { DEADBEEF32.into() }, { dst }));
    }
});

asm_test!(load, |instructions: &mut Instructions<Register>| {
    for base in REGISTERS {
        for dst in REGISTERS {
            instructions.add_instruction(code!(load { base } + { DEADBEEF32 }, { dst }));
        }
    }
});

asm_test!(store, |instructions: &mut Instructions<Register>| {
    for src in REGISTERS {
        for dst in REGISTERS {
            instructions.add_instruction(code!(store { src }, { dst } + { DEADBEEF32 }));
        }
    }
});

asm_test!(mov, |instructions: &mut Instructions<Register>| {
    for src in REGISTERS {
        for dst in REGISTERS {
            instructions.add_instruction(code!(mov { src }, { dst }));
        }
    }
});

asm_test!(push, |instructions: &mut Instructions<Register>| {
    for reg in REGISTERS {
        instructions.add_instruction(code!(push { reg }));
    }
});

asm_test!(pop, |instructions: &mut Instructions<Register>| {
    for reg in REGISTERS {
        instructions.add_instruction(code!(pop { reg }));
    }
});

asm_test!(add, |instructions: &mut Instructions<Register>| {
    for src in REGISTERS {
        for dst in REGISTERS {
            instructions.add_instruction(code!(add { src }, { dst }));
        }
    }
});

asm_test!(addi, |instructions: &mut Instructions<Register>| {
    for dst in REGISTERS {
        instructions.add_instruction(code!(addi { DEADBEEF32 }, { dst }));
    }
});

asm_test!(slt, |instructions: &mut Instructions<Register>| {
    for src1 in REGISTERS {
        for src2 in REGISTERS {
            for dst in REGISTERS {
                instructions.add_instruction(code!(slt { src1 }, { src2 }, { dst }));
            }
        }
    }
});

asm_test!(jmp, |instructions: &mut Instructions<Register>| {
    let lbl = instructions.add_label();

    instructions.add_instruction(code!(lbl: nop));
    instructions.add_instruction(code!(jmp { lbl }));
    instructions.add_instruction(code!(jmp { lbl }));
});

asm_test!(jz, |instructions: &mut Instructions<Register>| {
    let lbl = instructions.add_label();

    instructions.add_instruction(code!(lbl: nop));
    for reg in REGISTERS {
        instructions.add_instruction(code!(jz { reg }, { lbl }));
    }
});

asm_test!(ret, |instructions: &mut Instructions<Register>| {
    instructions.add_instruction(code!(ret));
});

asm_test!(call, |instructions: &mut Instructions<Register>| {
    for reg in REGISTERS {
        instructions.add_instruction(code!(call { reg }));
    }
});
