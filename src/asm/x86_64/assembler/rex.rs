#[repr(transparent)]
pub struct RexBuilder<const W: bool, const R: bool, const X: bool, const B: bool>(u8);

impl RexBuilder<false, false, false, false> {
    pub const fn new() -> Self {
        Self(0b01000000)
    }
}

impl<const R: bool, const X: bool, const B: bool> RexBuilder<false, R, X, B> {
    pub const fn set_w<const W: bool>(mut self) -> RexBuilder<true, R, X, B> {
        if W {
            self.0 |= 0b1000;
        }

        RexBuilder(self.0)
    }
}

impl<const W: bool, const X: bool, const B: bool> RexBuilder<W, false, X, B> {
    pub const fn set_r<const R: bool>(mut self) -> RexBuilder<W, true, X, B> {
        if R {
            self.0 |= 0b100;
        }

        RexBuilder(self.0)
    }
}

impl<const W: bool, const R: bool, const B: bool> RexBuilder<W, R, false, B> {
    pub const fn set_x<const X: bool>(mut self) -> RexBuilder<W, R, true, B> {
        if X {
            self.0 |= 0b10;
        }

        RexBuilder(self.0)
    }
}

impl<const W: bool, const R: bool, const X: bool> RexBuilder<W, R, X, false> {
    pub const fn set_b<const B: bool>(mut self) -> RexBuilder<W, R, X, true> {
        if B {
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
