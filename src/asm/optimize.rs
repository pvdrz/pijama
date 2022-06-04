use crate::asm::Instructions;

use super::InstructionKind;

impl<R> Instructions<R> {
    pub fn optimize(&mut self) {
        self.dead_jumps();
    }

    fn dead_jumps(&mut self) {
        for i in 0..self.len() - 1 {
            let next_label = self.instructions[i + 1].label;
            let curr = &mut self.instructions[i];

            if let InstructionKind::Jump(target) = curr.kind {
                if Some(target) == next_label {
                    curr.kind = InstructionKind::Nop;
                }
            }
        }
    }
}
