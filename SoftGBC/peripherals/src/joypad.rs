//! Joypad (`P1` / `FF00`) — stub for later windowed input.

use crate::mmio::IoDevice;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Joypad {
    /// Raw `P1` register (select bits + button bits).
    pub p1: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self { p1: 0xCF }
    }
}

impl IoDevice for Joypad {
    fn handles(addr: u16) -> bool {
        addr == 0xFF00
    }

    fn read(&self, _addr: u16) -> u8 {
        self.p1
    }

    fn write(&mut self, _addr: u16, data: u8) {
        // Only bits 4–5 are writable select lines on DMG; refine later.
        self.p1 = (self.p1 & 0xCF) | (data & 0x30);
    }
}
