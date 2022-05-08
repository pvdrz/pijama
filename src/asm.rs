pub type Immediate = ();
pub type Address = ();
pub type Register = ();

pub enum InstructionKind {
    LoadImm { src: Immediate, dst: Register },
    LoadAddr { src: Address, dst: Register },
    Store { src: Register, dst: Address },
    Push(Register),
    Pop(Register),
    Add { src: Register, dst: Register },
    AddImm { src: Immediate, dst: Register },
    Jump(Address),
    JumpLez { addr: Address, reg: Register },
    Return,
    Call(Register),
}
