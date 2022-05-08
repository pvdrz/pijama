/// Builder for the ModR/M byte.
///
/// This byte is composed of three fields commonly known as `Mod`, `Reg` and `R/M`. Each generic
/// parameter specifies if the corresponding field has been set already.
#[repr(transparent)]
pub struct ModRmBuilder<const MOD: bool, const REG: bool, const RM: bool>(u8);

impl ModRmBuilder<false, false, false> {
    /// Create a new builder with blank fields.
    pub const fn new() -> Self {
        Self(0)
    }
}

/// Methods to set the `mod` field.
///
/// This field is used to specify the addressing mode of the operands.
impl<const REG: bool, const RM: bool> ModRmBuilder<false, REG, RM> {
    /// Set the register-direct addressing mode.
    pub const fn direct(self) -> ModRmBuilder<true, REG, RM> {
        ModRmBuilder(self.0 | (0b11 << 6))
    }

    /// Set the register-indirect addressing mode with displacement.
    ///
    /// The displacement is specified adding displacement bytes to the instruction.
    pub const fn displacement(self) -> ModRmBuilder<true, REG, RM> {
        ModRmBuilder(self.0 | (0b10 << 6))
    }

    /// Set the register-indirect addressing mode without displacement.
    ///
    /// This mode is also used for instruction-pointer-relative addressing when the `rm` field is
    /// set to `0b101`.
    pub const fn indirect(self) -> ModRmBuilder<true, REG, RM> {
        ModRmBuilder(self.0)
    }
}

/// Methods to set the `reg` field.
///
/// This field is used to specify an operand or an instruction extension code.
impl<const MOD: bool, const RM: bool> ModRmBuilder<MOD, false, RM> {
    /// Set the `reg` field using the three least significant bytes of the argument.
    pub const fn reg(self, reg: u8) -> ModRmBuilder<MOD, true, RM> {
        ModRmBuilder(self.0 | ((reg & 0b111) << 3))
    }
}

/// Methods to set the `rm` field.
///
/// This field is used to specify an operand or to alter the indirect addressing mode.
impl<const MOD: bool, const REG: bool> ModRmBuilder<MOD, REG, false> {
    pub const fn rm(self, rm: u8) -> ModRmBuilder<MOD, REG, true> {
        ModRmBuilder(self.0 | (rm & 0b111))
    }
}

impl<const REG: bool> ModRmBuilder<false, REG, false> {
    /// Set the register-indirect addressing mode relative to the instruction pointer.
    ///
    /// This requires setting both the `mod` and `rm` fields.
    pub const fn relative(self) -> ModRmBuilder<true, REG, true> {
        self.indirect().rm(0b101)
    }
}

/// Methods to be used when all the fields are set.
impl ModRmBuilder<true, true, true> {
    /// Return the ModR/M byte.
    pub const fn build(self) -> u8 {
        self.0
    }
}
