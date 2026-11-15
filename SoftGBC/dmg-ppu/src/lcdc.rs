//! `LCDC` (`FF40`) bit layout via [`softgbc_utils::BitField`].

use softgbc_utils::BitField;

/// BG & Window enable / priority (bit 0).
pub const BG_ENABLE: BitField = BitField::bit(0);
/// OBJ enable (bit 1) — unused until sprites.
pub const OBJ_ENABLE: BitField = BitField::bit(1);
/// OBJ size (bit 2) — unused until sprites.
pub const OBJ_SIZE: BitField = BitField::bit(2);
/// BG tile map: 0 = `$9800`, 1 = `$9C00` (bit 3).
pub const BG_MAP: BitField = BitField::bit(3);
/// BG & Window tile data: 0 = `$8800` signed, 1 = `$8000` unsigned (bit 4).
pub const TILE_DATA: BitField = BitField::bit(4);
/// Window enable (bit 5) — unused until window.
pub const WIN_ENABLE: BitField = BitField::bit(5);
/// Window tile map (bit 6) — unused until window.
pub const WIN_MAP: BitField = BitField::bit(6);
/// LCD & PPU enable (bit 7).
pub const LCD_ENABLE: BitField = BitField::bit(7);
