//! DMG timer peripheral (`FF04`–`FF07`).
//!
//! Owns the internal 16-bit DIV counter and TIMA/TMA/TAC state. The SoC bus
//! routes those addresses here; they are not stored as flat IO RAM.

use crate::irq::{IrqLines, IF_TIMER};

const ADDR_DIV: u16 = 0xFF04;
const ADDR_TIMA: u16 = 0xFF05;
const ADDR_TMA: u16 = 0xFF06;
const ADDR_TAC: u16 = 0xFF07;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timer {
    /// Internal 16-bit divider; `DIV` MMIO is the upper 8 bits.
    div16: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    /// Previous AND result for falling-edge TIMA increment detection.
    last_and: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div16: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            last_and: false,
        }
    }

    pub fn handles(addr: u16) -> bool {
        matches!(addr, ADDR_DIV..=ADDR_TAC)
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            ADDR_DIV => (self.div16 >> 8) as u8,
            ADDR_TIMA => self.tima,
            ADDR_TMA => self.tma,
            ADDR_TAC => self.tac | 0xF8, // upper bits read as 1 on DMG
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            ADDR_DIV => {
                // Any write resets the whole internal counter.
                self.div16 = 0;
                // Falling-edge glitch on reset is deferred (accuracy follow-up).
                self.last_and = self.and_result();
            }
            ADDR_TIMA => self.tima = data,
            ADDR_TMA => self.tma = data,
            ADDR_TAC => {
                self.tac = data & 0x07;
                self.last_and = self.and_result();
            }
            _ => {}
        }
    }

    /// Advance by `t_cycles` system clocks.
    /// Returns IRQ lines to OR into `FF0F` (timer bit on TIMA overflow).
    pub fn tick(&mut self, t_cycles: u32) -> IrqLines {
        let mut irq = IrqLines::NONE;
        for _ in 0..t_cycles {
            self.div16 = self.div16.wrapping_add(1);
            let and = self.and_result();
            if self.last_and && !and {
                // Falling edge → TIMA++
                let (next, overflow) = self.tima.overflowing_add(1);
                if overflow {
                    // Coarse model: reload TMA and raise IRQ in the same T-cycle.
                    // (Accurate HW delays reload/IRQ by 4 T-cycles.)
                    self.tima = self.tma;
                    irq.bits |= IF_TIMER;
                } else {
                    self.tima = next;
                }
            }
            self.last_and = and;
        }
        irq
    }

    fn enabled(&self) -> bool {
        self.tac & 0x04 != 0
    }

    fn selected_bit(&self) -> u8 {
        match self.tac & 0x03 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            _ => 7, // 0b11
        }
    }

    fn and_result(&self) -> bool {
        if !self.enabled() {
            return false;
        }
        let bit = self.selected_bit();
        (self.div16 >> bit) & 1 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_write_resets_internal_counter() {
        let mut t = Timer::new();
        let _ = t.tick(0x180);
        assert_ne!(t.read(ADDR_DIV), 0);
        t.write(ADDR_DIV, 0xFF);
        assert_eq!(t.div16, 0);
        assert_eq!(t.read(ADDR_DIV), 0);
    }

    #[test]
    fn tac_05_overflow_raises_irq() {
        // TAC=$05: enable + /16. From TIMA=0, 256 increments → overflow.
        // 256 * 16 = 4096 T-cycles.
        let mut t = Timer::new();
        t.write(ADDR_TAC, 0x05);
        t.write(ADDR_TIMA, 0);
        t.write(ADDR_TMA, 0x00);
        t.write(ADDR_DIV, 0);

        assert!(t.tick(2000).is_empty(), "no overflow before 256×16 T-cycles");
        t.write(ADDR_TIMA, 0);
        t.write(ADDR_DIV, 0);
        // Exactly 256 falling edges at /16 from a clean DIV.
        let irq = t.tick(4096);
        assert_eq!(irq.bits & IF_TIMER, IF_TIMER);
        assert_eq!(t.tima, 0, "reloaded from TMA on overflow");
    }

    #[test]
    fn handles_only_timer_regs() {
        assert!(Timer::handles(0xFF04));
        assert!(Timer::handles(0xFF07));
        assert!(!Timer::handles(0xFF0F));
        assert!(!Timer::handles(0xFF00));
    }
}
