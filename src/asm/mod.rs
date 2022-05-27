use self::{
    mod_rm::ModRmBuilder,
    sib::{Scale, SibBuilder},
};

mod mod_rm;
mod sib;

pub type Imm32 = i32;
pub type Imm64 = i64;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Register {
    Ax = 0,
    Cx = 1,
    Dx = 2,
    Bx = 3,
    Sp = 4,
    Bp = 5,
    Si = 6,
    Di = 7,
}

pub struct Address<I> {
    pub base: Register,
    pub offset: I,
}

pub enum InstructionKind {
    LoadImm {
        src: Imm64,
        dst: Register,
    },
    LoadAddr {
        src: Address<Imm32>,
        dst: Register,
    },
    Store {
        src: Register,
        dst: Address<Imm32>,
    },
    Push(Register),
    Pop(Register),
    Add {
        src: Register,
        dst: Register,
    },
    AddImm {
        src: Imm32,
        dst: Register,
    },
    Jump(Location),
    JumpEq {
        reg1: Register,
        reg2: Register,
        target: Location,
    },
    JumpLt {
        reg1: Register,
        reg2: Register,
        target: Location,
    },
    JumpGt {
        reg1: Register,
        reg2: Register,
        target: Location,
    },
    Return,
    Call(Register),
}

pub enum Location {
    Imm32(Imm32),
    Label(Label),
}

impl From<Imm32> for Location {
    fn from(value: Imm32) -> Self {
        Self::Imm32(value)
    }
}

impl From<Label> for Location {
    fn from(label: Label) -> Self {
        Self::Label(label)
    }
}

#[derive(Clone, Copy)]
pub struct Label(usize);

pub struct Instruction {
    pub label: Option<Label>,
    pub kind: InstructionKind,
}

struct RexPrefix;

impl RexPrefix {
    const fn new(w_bit: bool, r_bit: bool, b_bit: bool) -> u8 {
        let mut prefix = 0b01000000;

        if w_bit {
            prefix |= 0b1000;
        }
        if r_bit {
            prefix |= 0b100;
        }
        if b_bit {
            prefix |= 0b1;
        }

        prefix
    }
}

struct Patch {
    label: Label,
    start: usize,
}

#[derive(Default)]
pub struct Assembler {
    buf: Vec<u8>,
    label_locations: Vec<usize>,
    patches: Vec<Patch>,
}

impl Assembler {
    /// Creates a new label, different from all the other labels created by this [`Assembler`].
    pub fn add_label(&mut self) -> Label {
        let label = Label(self.label_locations.len());

        self.label_locations.push(usize::MAX);

        label
    }

    /// Adds a new patch in the current location of the instruction pointer for a [`Label`]. This
    /// means that an [`Imm32`] with the location of the label will be written at the current
    /// location of the instruction pointer when calling [`Assembler::emit_code`].
    ///
    /// This will overwrite the `4` bytes following the current instruction pointer location.
    fn add_patch(&mut self, label: Label) {
        self.patches.push(Patch {
            label,
            start: self.buf.len(),
        })
    }

    /// Assembles an instruction.
    ///
    /// If the instruction has a label, the previous location of the label will be overwritten.
    pub fn assemble_instruction(&mut self, instruction: Instruction) {
        if let Some(label) = instruction.label {
            self.label_locations[label.0] = self.buf.len();
        }

        match instruction.kind {
            InstructionKind::LoadImm { src, dst } => self.assemble_load_imm(src, dst),
            InstructionKind::LoadAddr { src, dst } => self.assemble_load_addr(src, dst),
            InstructionKind::Store { src, dst } => self.assemble_store(src, dst),
            InstructionKind::Push(reg) => self.assemble_push(reg),
            InstructionKind::Pop(reg) => self.assemble_pop(reg),
            InstructionKind::Add { src, dst } => self.assemble_add(src, dst),
            InstructionKind::AddImm { src, dst } => self.assemble_add_imm(src, dst),
            InstructionKind::Jump(target) => self.assemble_jump(target),
            InstructionKind::JumpEq { reg1, reg2, target } => {
                self.assemble_conditional_jump::<0x84>(reg1, reg2, target)
            }
            InstructionKind::JumpLt { reg1, reg2, target } => {
                self.assemble_conditional_jump::<0x8C>(reg1, reg2, target)
            }
            InstructionKind::JumpGt { reg1, reg2, target } => {
                self.assemble_conditional_jump::<0x8F>(reg1, reg2, target)
            }
            InstructionKind::Return => self.assemble_return(),
            InstructionKind::Call(target) => self.assemble_call(target),
        }
    }

    fn assemble_load_imm(&mut self, src: i64, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0xb8 + dst as u8;
        let io = src.to_le_bytes();

        self.buf.extend_from_slice(&[rex_prefix, opcode]);
        self.buf.extend_from_slice(&io);
    }

    fn assemble_load_addr(&mut self, src: Address<Imm32>, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x8b;
        let mod_rm = ModRmBuilder::new()
            .displacement()
            .reg(dst as u8)
            .rm(src.base as u8)
            .build();

        if let Register::Sp = src.base {
            let sib = SibBuilder::new()
                .scale(Scale::One)
                .index(src.base)
                .base(src.base)
                .build();

            self.buf
                .extend_from_slice(&[rex_prefix, opcode, mod_rm, sib]);
        } else {
            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        }

        self.buf.extend_from_slice(&src.offset.to_le_bytes());
    }

    fn assemble_store(&mut self, src: Register, dst: Address<Imm32>) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x89;
        let mod_rm = ModRmBuilder::new()
            .displacement()
            .reg(dst.base as u8)
            .rm(src as u8)
            .build();

        if let Register::Sp = src {
            let sib = SibBuilder::new()
                .scale(Scale::One)
                .index(src)
                .base(src)
                .build();

            self.buf
                .extend_from_slice(&[rex_prefix, opcode, mod_rm, sib]);
        } else {
            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        }

        self.buf.extend_from_slice(&dst.offset.to_le_bytes());
    }

    fn assemble_push(&mut self, reg: Register) {
        let opcode = 0x50 + reg as u8;

        self.buf.extend_from_slice(&[opcode]);
    }

    fn assemble_pop(&mut self, reg: Register) {
        let opcode = 0x58 + reg as u8;

        self.buf.extend_from_slice(&[opcode]);
    }

    fn assemble_add(&mut self, src: Register, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x01;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(src as u8)
            .rm(dst as u8)
            .build();

        self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
    }

    fn assemble_add_imm(&mut self, src: i32, dst: Register) {
        let rex_prefix = RexPrefix::new(true, false, false);

        if let Register::Ax = dst {
            let opcode = 0x05;

            self.buf.extend_from_slice(&[rex_prefix, opcode]);
        } else {
            let opcode = 0x81;
            let mod_rm = ModRmBuilder::new().direct().reg(0x0).rm(dst as u8).build();

            self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        }

        self.buf.extend_from_slice(&src.to_le_bytes());
    }

    fn assemble_jump(&mut self, target: Location) {
        let opcode = 0xE9;

        self.buf.extend_from_slice(&[opcode]);

        match target {
            Location::Imm32(mut target) => {
                // The jump target is relative to the instruction pointer after reading this
                // instruction which has 1 + 4 bytes.
                target -= self.buf.len() as i32 + 0x4;
                self.buf.extend_from_slice(&target.to_le_bytes());
            }
            Location::Label(label) => {
                self.add_patch(label);
                self.buf.extend_from_slice(&0x0i32.to_le_bytes());
            }
        }
    }

    fn assemble_conditional_jump<const OPCODE: u8>(
        &mut self,
        reg1: Register,
        reg2: Register,
        target: Location,
    ) {
        let rex_prefix = RexPrefix::new(true, false, false);
        let opcode = 0x39;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(reg2 as u8)
            .rm(reg1 as u8)
            .build();

        // cmp reg2,reg1
        self.buf.extend_from_slice(&[rex_prefix, opcode, mod_rm]);
        // je target
        self.buf.extend_from_slice(&[0x0F, OPCODE]);

        match target {
            Location::Imm32(mut target) => {
                // The jump target is relative to the instruction pointer after reading this
                // instruction which has 5 + 4 bytes.
                target -= self.buf.len() as i32 + 0x4;
                self.buf.extend_from_slice(&target.to_le_bytes());
            }
            Location::Label(label) => {
                self.add_patch(label);
                self.buf.extend_from_slice(&0x0i32.to_le_bytes());
            }
        }
    }

    fn assemble_return(&mut self) {
        let opcode = 0xC3;

        self.buf.extend_from_slice(&[opcode]);
    }

    fn assemble_call(&mut self, target: Register) {
        let opcode = 0xFF;
        let mod_rm = ModRmBuilder::new()
            .direct()
            .reg(0x2)
            .rm(target as u8)
            .build();

        self.buf.extend_from_slice(&[opcode, mod_rm]);
    }

    pub fn emit_code(mut self) -> Vec<u8> {
        for patch in self.patches {
            // The value to be patched is an `i32`.
            let patch_end = patch.start + std::mem::size_of::<i32>();

            let mut label_location = self.label_locations[patch.label.0 as usize] as i32;
            // The label location must be written relative to the end of the instruction which
            // matches the end of the patch.
            label_location -= patch_end as i32;

            self.buf[patch.start..patch_end].copy_from_slice(&label_location.to_le_bytes());
        }

        self.buf
    }
}

#[macro_export]
macro_rules! reg {
    (rax) => {
        $crate::asm::Register::Ax
    };
    (rcx) => {
        $crate::asm::Register::Cx
    };
    (rdx) => {
        $crate::asm::Register::Dx
    };
    (rbx) => {
        $crate::asm::Register::Bx
    };
    (rsp) => {
        $crate::asm::Register::Sp
    };
    (rbp) => {
        $crate::asm::Register::Bp
    };
    (rsi) => {
        $crate::asm::Register::Si
    };
    (rdi) => {
        $crate::asm::Register::Di
    };
    ($expr:expr) => {
        $expr
    };
}

#[macro_export]
macro_rules! instruction_kind {
    (loadi {$imm64:expr},{$($reg:tt)+}) => {
        $crate::asm::InstructionKind::LoadImm {
            src: $imm64,
            dst: $crate::reg!($($reg)+),
        };
    };
    (load {$($addr:tt)+}+{$imm32:expr},{$($reg:tt)+}) => {
        $crate::asm::InstructionKind::LoadAddr {
            src: $crate::asm::Address {
                base: $crate::reg!($($addr)+),
                offset: $imm32,
            },
            dst: $crate::reg!($($reg)+),
        }
    };
    (store {$($reg:tt)*},{$($addr:tt)*}+{$imm32:expr}) => {
        $crate::asm::InstructionKind::Store {
            src: $crate::reg!($($reg)*),
            dst: $crate::asm::Address {
                base: $crate::reg!($($addr)*),
                offset: $imm32,
            },
        }
    };
    (push {$($reg:tt)*}) => {
        $crate::asm::InstructionKind::Push($crate::reg!($($reg)*))
    };
    (pop {$($reg:tt)*}) => {
        $crate::asm::InstructionKind::Pop($crate::reg!($($reg)*))
    };
    (add {$($reg1:tt)*}, {$($reg2:tt)*}) => {
        $crate::asm::InstructionKind::Add {
            src: $crate::reg!($($reg1)*),
            dst: $crate::reg!($($reg2)*),
        }
    };
    (addi {$imm32:expr},{$($reg:tt)+}) => {
        $crate::asm::InstructionKind::AddImm {
            src: $imm32,
            dst: $crate::reg!($($reg)+),
        }
    };
    (jmp {$loc:expr}) => {
        $crate::asm::InstructionKind::Jump({$loc}.into())
    };
    (je {$($reg1:tt)*},{$($reg2:tt)*},{$loc:expr}) => {
        $crate::asm::InstructionKind::JumpEq {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: {$loc}.into(),
        }
    };
    (jl {$($reg1:tt)*},{$($reg2:tt)*},{$loc:expr}) => {
        $crate::asm::InstructionKind::JumpLt {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: {$loc}.into(),
        }
    };
    (jg {$($reg1:tt)*},{$($reg2:tt)*},{$loc:expr}) => {
        $crate::asm::InstructionKind::JumpGt {
            reg1: $crate::reg!($($reg1)*),
            reg2: $crate::reg!($($reg2)*),
            target: {$loc}.into(),
        }
    };
    (ret) => {
        $crate::asm::InstructionKind::Return
    };
    (call {$($reg:tt)*}) => {
        $crate::asm::InstructionKind::Call($crate::reg!($($reg)*))
    };
}

#[macro_export]
macro_rules! code {
    ($label:ident: $($tokens:tt)+) => {
        $crate::asm::Instruction {
            label: Some($label),
            kind: $crate::instruction_kind!($($tokens)*)
        }
    };

    ($($tokens:tt)*) => {
        $crate::asm::Instruction {
            label: None,
            kind: $crate::instruction_kind!($($tokens)*)
        }
    };
}
