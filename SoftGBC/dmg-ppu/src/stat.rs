//! `STAT` (`FF41`) bit layout via [`softgbc_utils::BitField`].

use softgbc_utils::BitField;

/// PPU mode (bits 0–1), read-only / live from timing.
pub const MODE: BitField = BitField::new(0, 2);
/// LYC==LY coincidence flag (bit 2), read-only / live.
pub const LYC_EQ: BitField = BitField::bit(2);
/// Mode 0 (HBlank) interrupt select (bit 3).
pub const MODE0_INT: BitField = BitField::bit(3);
/// Mode 1 (VBlank) interrupt select (bit 4).
pub const MODE1_INT: BitField = BitField::bit(4);
/// Mode 2 (OAM) interrupt select (bit 5).
pub const MODE2_INT: BitField = BitField::bit(5);
/// LYC=LY interrupt select (bit 6).
pub const LYC_INT: BitField = BitField::bit(6);
/// Unused on DMG; often reads as 1 (bit 7).
pub const UNUSED7: BitField = BitField::bit(7);

/// Writable interrupt-select mask (bits 3–6).
pub const INT_SELECT_MASK: u8 = MODE0_INT.mask_in_place()
    | MODE1_INT.mask_in_place()
    | MODE2_INT.mask_in_place()
    | LYC_INT.mask_in_place();
