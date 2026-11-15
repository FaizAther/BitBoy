//! OAM DMA (`FF46`) — copy 160 bytes into OAM (`$FE00`–`$FE9F`).
//!
//! Instant model first (enough for sprites). Timed DMA can replace
//! [`Dma::run_instant`] later without changing the FF46 write API.

use crate::mmio::IoDevice;

/// Bytes transferred per OAM DMA (`$FE00`–`$FE9F`).
pub const OAM_DMA_LEN: usize = 0xA0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dma {
    /// Last value written to `FF46` (source page = `page << 8`).
    pub page: u8,
    /// `true` after a write until the transfer finishes.
    pub active: bool,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            page: 0,
            active: false,
        }
    }

    /// Source base address for the current page (`page << 8`).
    pub fn src_base(&self) -> u16 {
        u16::from(self.page) << 8
    }

    /// Instant OAM DMA: copy [`OAM_DMA_LEN`] bytes via `read8`, then clear `active`.
    ///
    /// Supported sources in SoftGBC: any address `read8` can serve (typically
    /// WRAM `$C000`–`$DFFF`). Restricted regions follow whatever the bus returns.
    pub fn run_instant<F>(&mut self, mut read8: F, oam: &mut [u8])
    where
        F: FnMut(u16) -> u8,
    {
        debug_assert!(oam.len() >= OAM_DMA_LEN);
        let base = self.src_base();
        for i in 0..OAM_DMA_LEN {
            oam[i] = read8(base.wrapping_add(i as u16));
        }
        self.active = false;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_ff46_marks_active_and_stores_page() {
        let mut dma = Dma::new();
        assert!(!dma.active);
        dma.write(0xFF46, 0xC0);
        assert!(dma.active);
        assert_eq!(dma.page, 0xC0);
        assert_eq!(dma.read(0xFF46), 0xC0);
        assert_eq!(dma.src_base(), 0xC000);
    }

    /// Checklist: mem[C000]=0xAB → write FF46=0xC0 → OAM[0]==0xAB
    #[test]
    fn write_ff46_instant_copies_first_byte_from_wram() {
        let mut dma = Dma::new();
        let mut oam = [0u8; OAM_DMA_LEN];
        let mut wram = [0u8; 0x2000];
        wram[0] = 0xAB; // $C000

        dma.write(0xFF46, 0xC0);
        assert!(dma.active);
        dma.run_instant(
            |addr| {
                if (0xC000..=0xDFFF).contains(&addr) {
                    wram[(addr - 0xC000) as usize]
                } else {
                    0xFF
                }
            },
            &mut oam,
        );

        assert_eq!(oam[0], 0xAB);
        assert!(!dma.active);
    }

    /// Checklist: fill C000..C09F with i; DMA 0xC0; all 160 OAM bytes match.
    #[test]
    fn instant_dma_copies_full_160_bytes() {
        let mut dma = Dma::new();
        let mut oam = [0u8; OAM_DMA_LEN];
        let mut wram = [0u8; 0x2000];
        for i in 0..OAM_DMA_LEN {
            wram[i] = i as u8;
        }

        dma.write(0xFF46, 0xC0);
        dma.run_instant(
            |addr| {
                if (0xC000..=0xDFFF).contains(&addr) {
                    wram[(addr - 0xC000) as usize]
                } else {
                    0xFF
                }
            },
            &mut oam,
        );

        for i in 0..OAM_DMA_LEN {
            assert_eq!(oam[i], i as u8, "oam[{i}]");
        }
        assert!(!dma.active);
    }
}
