//! Interrupt request bits peripherals can raise (SoC ORs into `FF0F`).

/// VBlank — `IF` bit 0
pub const IF_VBLANK: u8 = 0x01;
/// LCD STAT — `IF` bit 1
pub const IF_STAT: u8 = 0x02;
/// Timer — `IF` bit 2
pub const IF_TIMER: u8 = 0x04;
/// Serial — `IF` bit 3
pub const IF_SERIAL: u8 = 0x08;
/// Joypad — `IF` bit 4
pub const IF_JOYPAD: u8 = 0x10;

/// Bits to OR into `IF` after a peripheral `tick` (or side-effecting write).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IrqLines {
    pub bits: u8,
}

impl IrqLines {
    pub const NONE: Self = Self { bits: 0 };

    pub fn timer() -> Self {
        Self { bits: IF_TIMER }
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    pub fn is_empty(self) -> bool {
        self.bits == 0
    }
}
