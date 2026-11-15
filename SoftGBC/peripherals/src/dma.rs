//! OAM DMA (`FF46`) — copy 160 bytes into OAM (`$FE00`–`$FE9F`).
//!
//! Hardware takes ~160 M-cycles (one byte per M). SoftGBC uses [`Dma::tick_m`]
//! for that; [`Dma::run_instant`] remains for unit helpers / finish-now.

use crate::mmio::IoDevice;

/// Bytes transferred per OAM DMA (`$FE00`–`$FE9F`).
pub const OAM_DMA_LEN: usize = 0xA0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dma {
    /// Last value written to `FF46` (source page = `page << 8`).
    pub page: u8,
    /// `true` after a write until all [`OAM_DMA_LEN`] bytes are copied.
    pub active: bool,
    /// Next OAM offset to write (`0..OAM_DMA_LEN`).
    pub index: u16,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            page: 0,
            active: false,
            index: 0,
        }
    }

    /// Source base address for the current page (`page << 8`).
    pub fn src_base(&self) -> u16 {
        u16::from(self.page) << 8
    }

    /// Advance DMA by up to `m_cycles` bytes (1 byte / M-cycle). Returns unused M.
    pub fn tick_m<F>(&mut self, mut m_cycles: u32, mut read8: F, oam: &mut [u8]) -> u32
    where
        F: FnMut(u16) -> u8,
    {
        if !self.active {
            return m_cycles;
        }
        debug_assert!(oam.len() >= OAM_DMA_LEN);
        let base = self.src_base();
        while self.active && m_cycles > 0 {
            let i = self.index as usize;
            if i >= OAM_DMA_LEN {
                self.active = false;
                break;
            }
            oam[i] = read8(base.wrapping_add(self.index));
            self.index = self.index.saturating_add(1);
            m_cycles -= 1;
            if self.index as usize >= OAM_DMA_LEN {
                self.active = false;
            }
        }
        m_cycles
    }

    /// Instant OAM DMA: copy all remaining (or a fresh full table) now.
    pub fn run_instant<F>(&mut self, read8: F, oam: &mut [u8])
    where
        F: FnMut(u16) -> u8,
    {
        if !self.active {
            self.active = true;
            self.index = 0;
        }
        let _ = self.tick_m(OAM_DMA_LEN as u32, read8, oam);
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new()
    }
}

impl IoDevice for Dma {
    fn handles(addr: u16) -> bool {
        addr == 0xFF46
    }

    fn read(&self, _addr: u16) -> u8 {
        self.page
    }

    fn write(&mut self, _addr: u16, data: u8) {
        self.page = data;
        self.active = true;
        self.index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wram_reader(wram: &[u8; 0x2000]) -> impl Fn(u16) -> u8 + '_ {
        move |addr| {
            if (0xC000..=0xDFFF).contains(&addr) {
                wram[(addr - 0xC000) as usize]
            } else {
                0xFF
            }
        }
    }

    #[test]
    fn write_ff46_marks_active_and_stores_page() {
        let mut dma = Dma::new();
        assert!(!dma.active);
        dma.write(0xFF46, 0xC0);
        assert!(dma.active);
        assert_eq!(dma.page, 0xC0);
        assert_eq!(dma.index, 0);
        assert_eq!(dma.read(0xFF46), 0xC0);
        assert_eq!(dma.src_base(), 0xC000);
    }

    /// Checklist: mem[C000]=0xAB → write FF46=0xC0 → OAM[0]==0xAB
    #[test]
    fn write_ff46_instant_copies_first_byte_from_wram() {
        let mut dma = Dma::new();
        let mut oam = [0u8; OAM_DMA_LEN];
        let mut wram = [0u8; 0x2000];
        wram[0] = 0xAB;

        dma.write(0xFF46, 0xC0);
        assert!(dma.active);
        dma.run_instant(wram_reader(&wram), &mut oam);

        assert_eq!(oam[0], 0xAB);
        assert!(!dma.active);
    }

    #[test]
    fn instant_dma_copies_full_160_bytes() {
        let mut dma = Dma::new();
        let mut oam = [0u8; OAM_DMA_LEN];
        let mut wram = [0u8; 0x2000];
        for i in 0..OAM_DMA_LEN {
            wram[i] = i as u8;
        }

        dma.write(0xFF46, 0xC0);
        dma.run_instant(wram_reader(&wram), &mut oam);

        for i in 0..OAM_DMA_LEN {
            assert_eq!(oam[i], i as u8, "oam[{i}]");
        }
        assert!(!dma.active);
    }

    /// Checklist: tick(1) → one byte; after 160 → inactive.
    #[test]
    fn timed_dma_one_byte_per_m_cycle() {
        let mut dma = Dma::new();
        let mut oam = [0u8; OAM_DMA_LEN];
        let mut wram = [0u8; 0x2000];
        for i in 0..OAM_DMA_LEN {
            wram[i] = (i as u8).wrapping_add(1);
        }

        dma.write(0xFF46, 0xC0);
        assert_eq!(dma.index, 0);

        let left = dma.tick_m(1, wram_reader(&wram), &mut oam);
        assert_eq!(left, 0);
        assert_eq!(oam[0], 1);
        assert_eq!(oam[1], 0, "second byte not yet");
        assert!(dma.active);
        assert_eq!(dma.index, 1);

        let left = dma.tick_m(159, wram_reader(&wram), &mut oam);
        assert_eq!(left, 0);
        assert!(!dma.active);
        assert_eq!(dma.index, 160);
        for i in 0..OAM_DMA_LEN {
            assert_eq!(oam[i], (i as u8).wrapping_add(1), "oam[{i}]");
        }

        // Idle ticks are no-ops.
        oam[0] = 0xEE;
        let left = dma.tick_m(10, wram_reader(&wram), &mut oam);
        assert_eq!(left, 10);
        assert_eq!(oam[0], 0xEE);
    }
}
