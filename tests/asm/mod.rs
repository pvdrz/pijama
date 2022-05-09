use pijama::asm::{Address, Assembler, InstructionKind, Register};

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

#[test]
fn loadi() {
    let expected_bytes = include_bytes!("loadi.out");

    let mut asm = Assembler::default();

    for dst in REGISTERS {
        asm.assemble_instruction(InstructionKind::LoadImm {
            src: 0xdeadbeefdeadbeefu64 as i64,
            dst,
        });
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn load() {
    let expected_bytes = include_bytes!("load.out");

    let mut asm = Assembler::default();

    for base in REGISTERS {
        for dst in REGISTERS {
            asm.assemble_instruction(InstructionKind::LoadAddr {
                src: Address {
                    base,
                    offset: 0xdeadbeefu32 as i32,
                },
                dst,
            });
        }
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn store() {
    let expected_bytes = include_bytes!("store.out");

    let mut asm = Assembler::default();

    for src in REGISTERS {
        for base in REGISTERS {
            asm.assemble_instruction(InstructionKind::Store {
                src,
                dst: Address {
                    base,
                    offset: 0xdeadbeefu32 as i32,
                },
            });
        }
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn push() {
    let expected_bytes = include_bytes!("push.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(InstructionKind::Push(reg));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn pop() {
    let expected_bytes = include_bytes!("pop.out");

    let mut asm = Assembler::default();

    for reg in REGISTERS {
        asm.assemble_instruction(InstructionKind::Pop(reg));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn add() {
    let expected_bytes = include_bytes!("add.out");

    let mut asm = Assembler::default();

    for src in REGISTERS {
        for dst in REGISTERS {
            asm.assemble_instruction(InstructionKind::Add { src, dst });
        }
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn jmp() {
    let expected_bytes = include_bytes!("jmp.out");

    let mut asm = Assembler::default();

    for base in REGISTERS {
        asm.assemble_instruction(InstructionKind::Jump(Address { base, offset: () }));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn jz() {
    let expected_bytes = include_bytes!("jz.out");

    let mut asm = Assembler::default();

    for scr in REGISTERS {
        asm.assemble_instruction(InstructionKind::JumpIfZero {
            trg: 0xdeadbeefu32 as i32,
            scr,
        });
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn ret() {
    let expected_bytes = include_bytes!("ret.out");

    let mut asm = Assembler::default();

    asm.assemble_instruction(InstructionKind::Return);

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}

#[test]
fn call() {
    let expected_bytes = include_bytes!("call.out");

    let mut asm = Assembler::default();

    for trg in REGISTERS {
        asm.assemble_instruction(InstructionKind::Call(trg));
    }

    assert_eq!(expected_bytes, asm.emit_code().as_slice());
}
