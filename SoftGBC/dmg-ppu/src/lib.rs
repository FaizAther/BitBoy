//! DMG PPU.

use peripherals::irq::{IrqLines, IF_VBLANK};


// 456 as u16 columns * ( 144 rows + 10 vblank )

const MAX_COLUMNS:  u16 = 456;
const V_BLANK:      u8  = 10;
const MAX_ROWS:     u8  = 144 + V_BLANK;

pub struct DmgPpu {
    pub ly: u8,
    cycles: u16,
}

impl DmgPpu {
    pub fn new() -> Self {
        Self {
            ly: 0,
            cycles: 0,
        }
    }

    pub fn tick(&mut self, mut t_cycles:u32) -> IrqLines {
        let mut irq = IrqLines::NONE;
        let t_c_count: u8 = (t_cycles / MAX_COLUMNS as u32) as u8;
        let ly_modified = self.ly;
        self.ly =  (self.ly + t_c_count) % MAX_ROWS as u8;
        t_cycles %= MAX_COLUMNS as u32;
        self.cycles += t_cycles as u16;
        if self.cycles >= MAX_COLUMNS as u16 {
            self.cycles -= MAX_COLUMNS as u16;
            self.ly =  (self.ly + 1) % MAX_ROWS as u8;
        }
        if ly_modified != self.ly && self.ly == 144 {
            irq.bits |= IF_VBLANK;
        }
        irq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_single() {
        let mut ppu = DmgPpu::new();
        assert_eq!(ppu.ly, 0);
        for ly in 0..MAX_ROWS as u8 {
            for column in 0..MAX_COLUMNS as u16 {
                assert_eq!(ppu.ly as u32, ly as u32);
                let irq = ppu.tick(1);
                if ly == 143 && column == MAX_COLUMNS - 1 {
                    assert_eq!(irq, IrqLines::vblank());
                } else {
                    assert_eq!(irq, IrqLines::NONE);
                }
            }
        }
        assert_eq!(ppu.ly, 0);
    }

    #[test]
    pub fn test_extra() {
        let mut ppu = DmgPpu::new();
        ppu.tick(MAX_COLUMNS as u32);
        assert_eq!(ppu.ly, 1);
        assert_eq!(ppu.cycles, 0);
        ppu.tick(256);
        assert_eq!(ppu.ly, 1);
        assert_eq!(ppu.cycles, 256);
        ppu.tick(256);
        assert_eq!(ppu.ly, 2);
        assert_eq!(ppu.cycles, 56);
    }

    #[test]
    fn test_fixture_cumulative_t() {
        // total T-cycles → expected LY (fresh PPU each row)
        let cases = [
            (0, 0),
            (456, 1),
            (912, 2),
            (65_664, 144), // 144 * MAX_COLUMNS as u16
            (70_224, 0),   // 456 * MAX_COLUMNS as u16
        ];
        for (t, want_ly) in cases {
            let mut ppu = DmgPpu::new();
            ppu.tick(t as u32);
            assert_eq!(ppu.ly, want_ly, "after {t} T-cycles");
        }
    }
}