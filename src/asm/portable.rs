#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Register(usize);

#[derive(Default)]
pub struct RegisterGenerator(usize);

impl RegisterGenerator {
    pub fn generate(&mut self) -> Register {
        let reg = Register(self.0);
        self.0 += 1;
        reg
    }
}
