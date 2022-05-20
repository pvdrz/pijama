use pijama::{
    asm::{Assembler, Register},
    code,
};

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

#[test]
fn loadi() {
    let expected_bytes = include_bytes!("loadi.out");

    let mut asm = Assembler::default();

    for dst in REGISTERS {
        asm.assemble_instruction(code!(loadi { DEADBEEF64 }, { dst }));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn load() {
    let expected_bytes = include_bytes!("load.out");

    let mut asm = Assembler::default();

    for base in REGISTERS {
        for dst in REGISTERS {
            asm.assemble_instruction(code!(loada { base } + { DEADBEEF32 }, { dst }));
        }
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
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

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn push() {
    let expected_bytes = include_bytes!("push.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(code!(push { reg }));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn pop() {
    let expected_bytes = include_bytes!("pop.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(code!(pop { reg }));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
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

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn addi() {
    let expected_bytes = include_bytes!("addi.out");

    let mut asm = Assembler::default();

    for dst in REGISTERS {
        asm.assemble_instruction(code!(addi { DEADBEEF32 }, { dst }));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn jmp() {
    let expected_bytes = include_bytes!("jmp.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(code!(jmp { reg }));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
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

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

// #[test]
// fn jl() {
//     let expected_bytes = include_bytes!("jl.out");
//
//     let mut asm = Assembler::default();
//
//     for reg1 in REGISTERS {
//         for reg2 in REGISTERS {
//             asm.assemble_instruction(code!(jl { reg1 }, { reg2 }, { DEADBEEF32 }));
//         }
//     }
//
//     assert_eq!(expected_bytes, asm.emit_code().as_slice());
// }
//
// #[test]
// fn jg() {
//     let expected_bytes = include_bytes!("jg.out");
//
//     let mut asm = Assembler::default();
//
//     for reg1 in REGISTERS {
//         for reg2 in REGISTERS {
//             asm.assemble_instruction(code!(jg { reg1 }, { reg2 }, { DEADBEEF32 }));
//         }
//     }
//
//     assert_eq!(expected_bytes, asm.emit_code().as_slice());
// }

#[test]
fn ret() {
    let expected_bytes = include_bytes!("ret.out");

    let mut asm = Assembler::default();

    asm.assemble_instruction(code!(ret));

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn call() {
    let expected_bytes = include_bytes!("call.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(code!(call { reg }));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}
