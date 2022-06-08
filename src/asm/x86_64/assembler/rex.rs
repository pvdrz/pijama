#[repr(transparent)]
pub struct RexBuilder<const W: bool, const R: bool, const X: bool, const B: bool>(u8);

impl RexBuilder<false, false, false, false> {
    pub const fn new() -> Self {
        Self(0b01000000)
    }
}

impl<const R: bool, const X: bool, const B: bool> RexBuilder<false, R, X, B> {
    /// Sets the operand size:
    /// - `false`: Default operand size.
    /// - `true`: 64-bit operand size.
    pub const fn set_w(mut self, w: bool) -> RexBuilder<true, R, X, B> {
        if w {
            self.0 |= 0b1000;
        }

        RexBuilder(self.0)
    }
}

impl<const W: bool, const X: bool, const B: bool> RexBuilder<W, false, X, B> {
    /// Extends the `ModRm::reg` field.
    pub const fn set_r(mut self, r: bool) -> RexBuilder<W, true, X, B> {
        if r {
            self.0 |= 0b100;
        }

        RexBuilder(self.0)
    }
}

impl<const W: bool, const R: bool, const B: bool> RexBuilder<W, R, false, B> {
    /// Extends the `SIB::index` field.
    pub const fn set_x(mut self, x: bool) -> RexBuilder<W, R, true, B> {
        if x {
            self.0 |= 0b10;
        }

        RexBuilder(self.0)
    }
}

impl<const W: bool, const R: bool, const X: bool> RexBuilder<W, R, X, false> {
    /// Extends the `ModRm::r/m` or `SIB::base` field.
    pub const fn set_b(mut self, b: bool) -> RexBuilder<W, R, X, true> {
        if b {
            self.0 |= 0b1;
        }

        RexBuilder(self.0)
    }
}

impl RexBuilder<true, true, true, true> {
    pub fn finish(self) -> u8 {
        self.0
    }
}
