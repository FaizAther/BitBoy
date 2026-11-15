//! Shared MMIO contract for bus-routed peripherals (and PPU).

/// Device that owns a slice of the `$FF00`–`$FF7F` IO window.
///
/// The SoC bus asks [`Self::handles`] first, then [`Self::read`] / [`Self::write`].
/// Address decode stays on the concrete type (no `dyn` routing required).
pub trait IoDevice {
    /// `true` if this device owns `addr` (exclusive among wired devices).
    fn handles(addr: u16) -> bool;

    /// Read one register. Unknown/`handles` miss → typically `$FF`.
    fn read(&self, addr: u16) -> u8;

    /// Write one register. Unknown addresses are no-ops.
    fn write(&mut self, addr: u16, data: u8);
}
