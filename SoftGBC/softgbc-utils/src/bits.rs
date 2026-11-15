//! Named bitfields over an 8-bit register.
//!
//! Define a layout once with [`BitField`] constants (shift + width), then
//! [`get`](BitField::get) / [`set`](BitField::set) (or [`Reg8`] helpers).

/// One field inside a `u8` register: contiguous bits `[shift .. shift+width)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitField {
    /// Lowest bit index (0 = LSB).
    pub shift: u8,
    /// Number of bits (1..=8).
    pub width: u8,
}

impl BitField {
    pub const fn new(shift: u8, width: u8) -> Self {
        assert!(width >= 1 && width <= 8);
        assert!(shift < 8);
        assert!(shift + width <= 8);
        Self { shift, width }
    }

    /// Single-bit field at `bit` (0 = LSB).
    pub const fn bit(bit: u8) -> Self {
        Self::new(bit, 1)
    }

    /// Mask in place (already shifted), e.g. mode bits → `0b11`.
    pub const fn mask(self) -> u8 {
        ((1u16 << self.width) - 1) as u8
    }

    /// Mask in the register word, e.g. mode at bits 0–1 → `0b0000_0011`.
    pub const fn mask_in_place(self) -> u8 {
        self.mask() << self.shift
    }

    /// Extract the field as an integer (`0 .. 2^width`).
    pub const fn get(self, value: u8) -> u8 {
        (value >> self.shift) & self.mask()
    }

    /// Replace the field; other bits unchanged. `field` is truncated to `width`.
    pub const fn set(self, value: u8, field: u8) -> u8 {
        let cleared = value & !self.mask_in_place();
        cleared | ((field & self.mask()) << self.shift)
    }

    /// For 1-bit fields: is the bit set?
    pub const fn is_set(self, value: u8) -> bool {
        self.get(value) != 0
    }

    /// For 1-bit fields: set or clear.
    pub const fn set_bool(self, value: u8, on: bool) -> u8 {
        self.set(value, on as u8)
    }
}

/// Thin `u8` wrapper with named-field accessors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Reg8(pub u8);

impl Reg8 {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn get(self, field: BitField) -> u8 {
        field.get(self.0)
    }

    pub const fn set(mut self, field: BitField, value: u8) -> Self {
        self.0 = field.set(self.0, value);
        self
    }

    pub const fn bit(self, field: BitField) -> bool {
        field.is_set(self.0)
    }

    pub const fn set_bit(mut self, field: BitField, on: bool) -> Self {
        self.0 = field.set_bool(self.0, on);
        self
    }

    /// Keep only bits in `keep_mask` from `self`, merge with `live` under `!keep_mask`.
    /// Useful for STAT: writable selects OR live mode/LYC flags.
    pub const fn merge_live(self, keep_mask: u8, live: u8) -> Self {
        Self((self.0 & keep_mask) | (live & !keep_mask))
    }
}

impl From<u8> for Reg8 {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Reg8> for u8 {
    fn from(value: Reg8) -> u8 {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // STAT-shaped layout for examples / regression.
    const MODE: BitField = BitField::new(0, 2);
    const LYC_EQ: BitField = BitField::bit(2);
    const MODE0_INT: BitField = BitField::bit(3);
    const MODE1_INT: BitField = BitField::bit(4);
    const MODE2_INT: BitField = BitField::bit(5);
    const LYC_INT: BitField = BitField::bit(6);

    #[test]
    fn stat_get_set_mode_and_flags() {
        let mut r = Reg8(0);
        r = r.set(MODE, 2);
        assert_eq!(r.get(MODE), 2);
        assert_eq!(r.0, 0b0000_0010);

        r = r.set_bit(LYC_EQ, true).set_bit(LYC_INT, true);
        assert!(r.bit(LYC_EQ));
        assert!(r.bit(LYC_INT));
        assert_eq!(r.0, 0b0100_0110);
    }

    #[test]
    fn set_truncates_to_width() {
        assert_eq!(MODE.set(0, 0xFF), 0b0000_0011);
    }

    #[test]
    fn merge_live_keeps_selects() {
        // CPU wrote all selects; live mode=1, lyc_eq=1
        let stored = Reg8(0xFF);
        let live = Reg8(0).set(MODE, 1).set_bit(LYC_EQ, true).0;
        let out = stored.merge_live(0x78, live);
        assert_eq!(out.0 & 0x78, 0x78);
        assert_eq!(MODE.get(out.0), 1);
        assert!(LYC_EQ.is_set(out.0));
        let _ = (MODE0_INT, MODE1_INT, MODE2_INT); // named in layout
    }
}
