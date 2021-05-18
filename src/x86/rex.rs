/// Builder for the REX prefix byte.
///
/// This byte is composed of four fields commonly known as `W`, `R`, `X` and `B`. Each generic
/// parameter specifies if the corresponding field has been set already.
#[repr(transparent)]
pub struct RexBuilder<const W: bool, const R: bool, const X: bool, const B: bool>(u8);

impl RexBuilder<false, false, false, false> {
    /// Create a new builder with blank fields.
    pub const fn new() -> Self {
        Self(0b1 << 6)
    }
}

/// Methods to set the `W` field.
///
/// This field is used to specify the size of the operands for certain instructions.
impl<const R: bool, const X: bool, const B: bool> RexBuilder<false, R, X, B> {
    /// Set the 64-bit operand mode.
    pub const fn size_64(self) -> RexBuilder<true, R, X, B> {
        RexBuilder(self.0 | 0b1 << 3)
    }

    /// Set the default operand size mode.
    pub const fn size_default(self) -> RexBuilder<true, R, X, B> {
        RexBuilder(self.0)
    }
}

/// Methods to set the `R` field.
///
/// This field is used to extend the `reg` field of the ModR/M byte to access all 16 registers.
impl<const W: bool, const X: bool, const B: bool> RexBuilder<W, false, X, B> {
    /// Set the `R` field according to the argument.
    pub const fn set_r(self, turn_on: bool) -> RexBuilder<W, true, X, B> {
        RexBuilder(self.0 | ((turn_on as u8) << 2))
    }
}

/// Methods to set the `X` field.
///
/// This field is used to extend the `index` field of the SIB byte to access all 16 registers.
impl<const W: bool, const R: bool, const B: bool> RexBuilder<W, R, false, B> {
    /// Set the `X` field according to the argument.
    pub const fn set_x(self, turn_on: bool) -> RexBuilder<W, R, true, B> {
        RexBuilder(self.0 | ((turn_on as u8) << 1))
    }
}

/// Methods to set the `B` field.
///
/// This field is used to extend the `rm` field of the ModR/M byte to access all 16 registers.
impl<const W: bool, const R: bool, const X: bool> RexBuilder<W, R, X, false> {
    /// Set the `B` field according to the argument.
    pub const fn set_b(self, turn_on: bool) -> RexBuilder<W, R, X, true> {
        RexBuilder(self.0 | (turn_on as u8))
    }
}

/// Methods to be used when all the fields are set.
impl RexBuilder<true, true, true, true> {
    /// Return the REX prefix byte.
    pub const fn build(self) -> u8 {
        self.0
    }
}
