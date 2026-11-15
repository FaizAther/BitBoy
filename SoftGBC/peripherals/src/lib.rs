//! SoftGBC DMG peripherals — timer, serial, joypad, DMA, …
//!
//! Each device owns its MMIO behaviour via [`IoDevice`]. The SoC bus routes
//! addresses with `handles` / `read` / `write` and distributes master-clock
//! T-cycles via `tick`. IRQ requests are returned as [`IrqLines`] for the SoC
//! to OR into `FF0F`.
//!
//! PPU stays in `dmg-ppu` but implements the same [`IoDevice`] trait.

pub mod dma;
pub mod irq;
pub mod joypad;
pub mod mmio;
pub mod serial;
pub mod timer;

pub use dma::Dma;
pub use irq::{IrqLines, IF_JOYPAD, IF_SERIAL, IF_STAT, IF_TIMER, IF_VBLANK};
pub use joypad::{Buttons, Joypad};
pub use mmio::IoDevice;
pub use serial::Serial;
pub use timer::Timer;
