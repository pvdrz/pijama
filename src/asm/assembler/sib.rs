use super::Register;

/// Builder for the SIB byte.
///
/// This byte is composed of three fields commonly known as `scale`, `index` and `base`. Each generic
/// parameter specifies if the corresponding field has been set already. These fields are used to
/// compute an effective memory address: `scale * index  + base`.
#[repr(transparent)]
pub struct SibBuilder<const SCALE: bool, const INDEX: bool, const BASE: bool>(u8);

impl SibBuilder<false, false, false> {
    /// Create a new builder with blank fields.
    pub const fn new() -> Self {
        Self(0)
    }
}

/// Methods to set the `scale` field.
///
/// This field is used to specify the scale factor.
impl<const INDEX: bool, const BASE: bool> SibBuilder<false, INDEX, BASE> {
    /// Set the `scale` field to one.
    pub const fn scale(self, scale: Scale) -> SibBuilder<true, INDEX, BASE> {
        SibBuilder(self.0 | ((scale as u8) << 6))
    }
}

/// Methods to set the `index` field.
///
/// This field is used to specify the register containing the index portion.
impl<const SCALE: bool, const BASE: bool> SibBuilder<SCALE, false, BASE> {
    /// Set the `index` field using a register.
    ///
    /// If the `rsp` register is used as an argument the effective `index` is zero.
    pub const fn index(self, reg: Register) -> SibBuilder<SCALE, true, BASE> {
        SibBuilder(self.0 | ((reg as u8 & 0b111) << 3))
    }
}

/// Methods to set the `base` field.
///
/// This field is used to specify the register containing the base address.
impl<const SCALE: bool, const INDEX: bool> SibBuilder<SCALE, INDEX, false> {
    /// Set the `base` field using a register.
    ///
    /// If the `rbp` register is used as an argument and `ModRM.mod` is zero. Then the effective `base` is zero.
    pub const fn base(self, reg: Register) -> SibBuilder<SCALE, INDEX, true> {
        SibBuilder(self.0 | (reg as u8 & 0b111))
    }
}

/// Methods to be used when all the fields are set.
impl SibBuilder<true, true, true> {
    /// Return the SIB byte.
    pub const fn build(self) -> u8 {
        self.0
    }
}

/// Valid `scale` fields.
#[repr(u8)]
pub enum Scale {
    One = 0b00,
    Two = 0b01,
    Four = 0b10,
    Eight = 0b11,
}
