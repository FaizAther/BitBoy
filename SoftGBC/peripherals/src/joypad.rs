//! Joypad (`P1` / `FF00`) — select lines + host button state.
//!
//! Host keyboards / web / Bluetooth feed [`Buttons`] via [`Joypad::set_pressed`].
//! SoftGBC host wraps those backends as `InputSource` (see `softgbc-host`).

use crate::mmio::IoDevice;

/// Eight DMG keys as a bitset (host-facing). OR-merge with [`Buttons::union`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Buttons(u8);

impl Buttons {
    pub const NONE: Self = Self(0);
    pub const RIGHT: Self = Self(1 << 0);
    pub const LEFT: Self = Self(1 << 1);
    pub const UP: Self = Self(1 << 2);
    pub const DOWN: Self = Self(1 << 3);
    pub const A: Self = Self(1 << 4);
    pub const B: Self = Self(1 << 5);
    pub const SELECT: Self = Self(1 << 6);
    pub const START: Self = Self(1 << 7);

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Joypad {
    /// Select bits 4–5 as last written (other bits rebuilt on read / set).
    select: u8,
    /// Keys currently held (from host).
    pressed: Buttons,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            // Neither group selected → low nibble reads as 0xF (0xCF classic power-on-ish).
            select: 0x30,
            pressed: Buttons::NONE,
        }
    }

    pub fn pressed(&self) -> Buttons {
        self.pressed
    }

    /// Host / `InputSource` updates which keys are down.
    pub fn set_pressed(&mut self, buttons: Buttons) {
        self.pressed = buttons;
    }

    fn low_nibble(&self) -> u8 {
        let mut nibble = 0x0Fu8;
        let dpad = self.select & 0x10 == 0;
        let btns = self.select & 0x20 == 0;
        if dpad {
            if self.pressed.contains(Buttons::RIGHT) {
                nibble &= !0x01;
            }
            if self.pressed.contains(Buttons::LEFT) {
                nibble &= !0x02;
            }
            if self.pressed.contains(Buttons::UP) {
                nibble &= !0x04;
            }
            if self.pressed.contains(Buttons::DOWN) {
                nibble &= !0x08;
            }
        }
        if btns {
            if self.pressed.contains(Buttons::A) {
                nibble &= !0x01;
            }
            if self.pressed.contains(Buttons::B) {
                nibble &= !0x02;
            }
            if self.pressed.contains(Buttons::SELECT) {
                nibble &= !0x04;
            }
            if self.pressed.contains(Buttons::START) {
                nibble &= !0x08;
            }
        }
        nibble
    }

    fn p1_value(&self) -> u8 {
        // Bits 6–7 unused/read as 1 on many docs; keep 0xC0 | select | nibble.
        0xC0 | (self.select & 0x30) | self.low_nibble()
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

impl IoDevice for Joypad {
    fn handles(addr: u16) -> bool {
        addr == 0xFF00
    }

    fn read(&self, _addr: u16) -> u8 {
        self.p1_value()
    }

    fn write(&mut self, _addr: u16, data: u8) {
        // Only bits 4–5 are writable select lines.
        self.select = data & 0x30;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_keys_selected_group_reads_f() {
        let mut pad = Joypad::new();
        pad.write(0xFF00, 0x20); // d-pad selected
        assert_eq!(pad.read(0xFF00) & 0x0F, 0x0F);
        pad.write(0xFF00, 0x10); // buttons selected
        assert_eq!(pad.read(0xFF00) & 0x0F, 0x0F);
    }

    #[test]
    fn a_pressed_select_buttons_clears_bit0() {
        let mut pad = Joypad::new();
        pad.set_pressed(Buttons::A);
        pad.write(0xFF00, 0x10); // select buttons
        let v = pad.read(0xFF00);
        assert_eq!(v & 0x01, 0);
        assert_eq!(v & 0x0E, 0x0E);
    }

    #[test]
    fn a_pressed_but_dpad_selected_hides_a() {
        let mut pad = Joypad::new();
        pad.set_pressed(Buttons::A);
        pad.write(0xFF00, 0x20); // select d-pad only
        assert_eq!(pad.read(0xFF00) & 0x0F, 0x0F);
    }

    #[test]
    fn right_pressed_select_dpad() {
        let mut pad = Joypad::new();
        pad.set_pressed(Buttons::RIGHT);
        pad.write(0xFF00, 0x20);
        assert_eq!(pad.read(0xFF00) & 0x0F, 0x0E);
    }

    #[test]
    fn neither_group_selected_reads_f() {
        let pad = Joypad::new();
        assert_eq!(pad.read(0xFF00) & 0x3F, 0x3F); // select 0x30 + nibble F
    }
}
