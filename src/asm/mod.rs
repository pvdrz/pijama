mod macros;
mod optimize;
pub mod portable;
pub mod x86_64;

pub type Imm32 = i32;
pub type Imm64 = i64;

pub struct Address<I, R> {
    pub base: R,
    pub offset: I,
}

pub enum InstructionKind<R> {
    LoadImm { src: Imm64, dst: R },
    LoadAddr { src: Address<Imm32, R>, dst: R },
    Store { src: R, dst: Address<Imm32, R> },
    Mov { src: R, dst: R },
    Push(R),
    Pop(R),
    Add { src: R, dst: R },
    AddImm { src: Imm32, dst: R },
    SetIfLess { src1: R, src2: R, dst: R },
    Jump(Label),
    JumpIfZero { src: R, target: Label },
    Return,
    Call(R),
    Nop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Label(usize);

#[derive(Default)]
struct LabelGenerator(usize);

pub struct Instruction<R> {
    pub label: Option<Label>,
    pub kind: InstructionKind<R>,
}

pub struct Instructions<R> {
    instructions: Vec<Instruction<R>>,
    labels_len: usize,
}

impl<R> Instructions<R> {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels_len: 0,
        }
    }
    pub fn add_label(&mut self) -> Label {
        let label = Label(self.labels_len);
        self.labels_len += 1;
        label
    }

    pub fn add_instruction(&mut self, instruction: Instruction<R>) {
        self.instructions.push(instruction)
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Instruction<R>> {
        self.instructions.get_mut(index)
    }
}
