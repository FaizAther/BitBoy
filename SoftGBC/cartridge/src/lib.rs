//! Cartridge ROM loader + MBC1 (minimal).
//!
//! SoftGBC maps cart reads/writes through [`Mbc1`] when the image needs banking.

mod mbc1;

pub use mbc1::Mbc1;

/// Cartridge type byte at `$0147`.
pub fn cart_type(rom: &[u8]) -> u8 {
    rom.get(0x0147).copied().unwrap_or(0)
}

/// True if SoftGBC should use MBC1 banking for this image.
pub fn needs_mbc1(rom: &[u8]) -> bool {
    matches!(cart_type(rom), 0x01 | 0x02 | 0x03)
}
