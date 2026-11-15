//! DMG PPU.

use peripherals::irq::{IrqLines, IF_VBLANK};
use peripherals::IoDevice;


// 456 as u16 columns * ( 144 rows + 10 vblank )

const MAX_COLUMNS:  u16 = 456;
const V_BLANK:      u8  = 10;
const MAX_ROWS:     u8  = 144 + V_BLANK;

pub struct DmgPpu {
    cycles: u16,

    pub LCDC: u8,
    pub STAT: u8,
    pub SCY: u8,
    pub SCX: u8,
    pub ly: u8, // FF44 read-only
    pub LYC: u8,
    pub DMA: u8,
    pub BGP: u8,
    pub OBP0: u8,
    pub OBP1: u8,
    pub WY: u8,
    pub WX: u8,
}

impl DmgPpu {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            LCDC: 0,
            STAT: 0,
            SCY: 0,
            SCX: 0,
            ly: 0,
            LYC: 0,
            DMA: 0,
            BGP: 0,
            OBP0: 0,
            OBP1: 0,
            WY: 0,
            WX: 0,
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

impl IoDevice for DmgPpu {
    fn handles(addr: u16) -> bool {
        matches!(addr, 0xFF40..=0xFF4B)
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.LCDC,
            0xFF41 => self.STAT,
            0xFF42 => self.SCY,
            0xFF43 => self.SCX,
            0xFF44 => self.ly,
            0xFF45 => self.LYC,
            0xFF46 => self.DMA,
            0xFF47 => self.BGP,
            0xFF48 => self.OBP0,
            0xFF49 => self.OBP1,
            0xFF4A => self.WY,
            0xFF4B => self.WX,
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF40 => self.LCDC = data,
            0xFF41 => self.STAT = data,
            0xFF42 => self.SCY = data,
            0xFF43 => self.SCX = data,
            0xFF44 => {} // LY: ignore writes for now
            0xFF45 => self.LYC = data,
            0xFF46 => self.DMA = data,
            0xFF47 => self.BGP = data,
            0xFF48 => self.OBP0 = data,
            0xFF49 => self.OBP1 = data,
            0xFF4A => self.WY = data,
            0xFF4B => self.WX = data,
            _ => {}
        }
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

    #[test]
    fn handles_lcd_io_window() {
        assert!(DmgPpu::handles(0xFF40));
        assert!(DmgPpu::handles(0xFF44));
        assert!(DmgPpu::handles(0xFF4B));
        assert!(!DmgPpu::handles(0xFF3F));
        assert!(!DmgPpu::handles(0xFF4C));
        assert!(!DmgPpu::handles(0xFF0F));
    }

    #[test]
    fn lcdc_round_trip() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, 0x80);
        assert_eq!(ppu.read(0xFF40), 0x80);
        ppu.write(0xFF40, 0x91);
        assert_eq!(ppu.read(0xFF40), 0x91);
    }

    #[test]
    fn ff44_tracks_ly_and_ignores_writes() {
        let mut ppu = DmgPpu::new();
        assert_eq!(ppu.read(0xFF44), 0);
        ppu.tick(MAX_COLUMNS as u32);
        assert_eq!(ppu.read(0xFF44), 1);
        assert_eq!(ppu.read(0xFF44), ppu.ly);
        ppu.write(0xFF44, 0x00);
        assert_eq!(ppu.read(0xFF44), 1);
    }

    #[test]
    fn scy_round_trip() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF42, 0x55);
        assert_eq!(ppu.read(0xFF42), 0x55);
    }
}