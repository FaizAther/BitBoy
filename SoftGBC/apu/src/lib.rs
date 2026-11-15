//! DMG APU — **mute-safe stub**.
//!
//! Games write `FF10`–`FF3F` during boot. SoftGBC accepts those writes and
//! always mixes silence so titles do not spin waiting for audio hardware.

use peripherals::IoDevice;

/// NR register window `$FF10`–`$FF3F` (48 bytes), including wave RAM `$FF30`–`$FF3F`.
pub const APU_IO_SIZE: usize = 0x30;
const NR52: u16 = 0xFF26;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Apu {
    /// Raw regs indexed by `addr - 0xFF10`.
    regs: [u8; APU_IO_SIZE],
}

impl Apu {
    pub fn new() -> Self {
        let mut regs = [0u8; APU_IO_SIZE];
        // Power-on-ish: NR52 bit7 often set after boot; leave 0 until game writes.
        regs[(NR52 - 0xFF10) as usize] = 0x00;
        Self { regs }
    }

    fn idx(addr: u16) -> Option<usize> {
        if (0xFF10..=0xFF3F).contains(&addr) {
            Some((addr - 0xFF10) as usize)
        } else {
            None
        }
    }

    /// Stereo sample — always silence in the mute stub.
    pub fn mix(&self) -> (i16, i16) {
        (0, 0)
    }

    /// Advance APU by T-cycles — no-op while muted.
    pub fn tick(&mut self, _t_cycles: u32) {}
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

impl IoDevice for Apu {
    fn handles(addr: u16) -> bool {
        (0xFF10..=0xFF3F).contains(&addr)
    }

    fn read(&self, addr: u16) -> u8 {
        let Some(i) = Self::idx(addr) else {
            return 0xFF;
        };
        let v = self.regs[i];
        // NR52: bit7 writable; bits 0–3 are channel-on status (stub → 0).
        if addr == NR52 {
            return (v & 0x80) | 0x70;
        }
        v
    }

    fn write(&mut self, addr: u16, data: u8) {
        let Some(i) = Self::idx(addr) else {
            return;
        };
        if addr == NR52 {
            // Only bit7 is writable on hardware.
            self.regs[i] = (self.regs[i] & !0x80) | (data & 0x80);
            return;
        }
        self.regs[i] = data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_nr_window_only() {
        assert!(Apu::handles(0xFF10));
        assert!(Apu::handles(0xFF26));
        assert!(Apu::handles(0xFF3F));
        assert!(!Apu::handles(0xFF0F));
        assert!(!Apu::handles(0xFF40));
    }

    #[test]
    fn nr52_bit7_round_trips_channels_read_zero() {
        let mut apu = Apu::new();
        apu.write(0xFF26, 0x8F);
        assert_eq!(apu.read(0xFF26) & 0x80, 0x80);
        assert_eq!(apu.read(0xFF26) & 0x0F, 0x00);
    }

    #[test]
    fn wave_ram_round_trip() {
        let mut apu = Apu::new();
        for i in 0..16u16 {
            apu.write(0xFF30 + i, i as u8);
        }
        for i in 0..16u16 {
            assert_eq!(apu.read(0xFF30 + i), i as u8);
        }
    }

    #[test]
    fn mix_stays_silent_after_ticks_and_writes() {
        let mut apu = Apu::new();
        apu.write(0xFF26, 0x80);
        apu.write(0xFF12, 0xF0);
        apu.write(0xFF25, 0xFF);
        apu.tick(1_000_000);
        assert_eq!(apu.mix(), (0, 0));
    }
}
