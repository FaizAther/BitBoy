//! Generic SoftGBC machine — CPU + `DmgBus`, agnostic to any particular test ROM.

use softgbc_core::{Bus, DmgBus, HexLoadError};
use sm83_iss::Cpu;
use std::path::Path;

/// How a long `run` stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    /// Hit `max_steps` without a serial pass/fail marker.
    MaxSteps,
    /// Serial output contains `Passed` (common test-ROM convention).
    SerialPassed,
    /// Serial output contains `Failed`.
    SerialFailed,
}

/// Skip-boot / headless defaults. LY comes from the real PPU (`FF44`); no stub.
#[derive(Debug, Clone, Copy)]
pub struct MachineConfig {
    /// `A` after skip-boot (DMG boot leftover is `0x01`; many tests expect this).
    pub a: u8,
    /// Initial `IF` (`FF0F`). `0xE0` matches common “unused bits high” reads.
    pub if_init: u8,
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            a: 0x01,
            if_init: 0xE0,
        }
    }
}

/// Soft Game Boy: one CPU + one DMG memory map / peripherals.
pub struct SoftGbc {
    pub cpu: Cpu,
    pub bus: DmgBus,
}

impl SoftGbc {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            bus: DmgBus::new(),
        }
    }

    pub fn from_rom_file(path: impl AsRef<Path>) -> Result<Self, HexLoadError> {
        let bus = DmgBus::from_rom_file(path)?;
        let mut gb = Self {
            cpu: Cpu::new(),
            bus,
        };
        gb.reset(&MachineConfig::default());
        Ok(gb)
    }

    /// Load a raw `.gb` / `.gbc` image already in memory (skip boot → `$0100`).
    pub fn from_rom_bytes(data: &[u8]) -> Self {
        let bus = DmgBus::from_rom_bytes(data);
        let mut gb = Self {
            cpu: Cpu::new(),
            bus,
        };
        gb.reset(&MachineConfig::default());
        gb
    }

    /// Apply config and set `PC` to cart entry (skip boot).
    pub fn reset(&mut self, cfg: &MachineConfig) {
        self.cpu = Cpu::new();
        self.cpu.pc = self.bus.entry;
        self.cpu.a = cfg.a;
        let _ = self.bus.write8(0xFF0F, cfg.if_init);
        // LY (`FF44`) is owned by `bus.ppu` — advanced by `step` → `tick_*`.
        self.bus.serial.clear_output();
    }

    /// One instruction + peripheral tick (master clock).
    pub fn step(&mut self) -> u8 {
        let m = self.cpu.step(&mut self.bus);
        self.bus.tick_m_cycles(m);
        m
    }

    /// Run up to `max_steps` instructions.
    ///
    /// Stops early if serial output contains `Passed` or `Failed` (generic
    /// headless test-ROM convention — not Blargg-specific).
    pub fn run(&mut self, max_steps: u64) -> RunStatus {
        for _ in 0..max_steps {
            let _ = self.step();
            let out = self.bus.serial_output();
            if out.contains("Passed") {
                return RunStatus::SerialPassed;
            }
            if out.contains("Failed") {
                return RunStatus::SerialFailed;
            }
        }
        RunStatus::MaxSteps
    }

    pub fn serial_output(&self) -> &str {
        self.bus.serial_output()
    }
}

impl Default for SoftGbc {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus for SoftGbc {
    fn read8(&self, addr: u16) -> Option<u8> {
        self.bus.read8(addr)
    }

    fn write8(&mut self, addr: u16, data: u8) -> bool {
        self.bus.write8(addr, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use softgbc_core::Bus;

    #[test]
    fn step_advances_ff44_ly() {
        let mut gb = SoftGbc::new();
        gb.reset(&MachineConfig::default());
        // Fill entry with NOPs so step keeps fetching.
        for a in 0x0100..0x0200 {
            assert!(gb.bus.poke(a, 0x00));
        }
        assert!(gb.write8(0xFF40, 0x80)); // LCD on — LY frozen while off
        assert_eq!(gb.read8(0xFF44), Some(0));

        // 1 NOP = 1 M = 4 T; one scanline = 456 T → 114 NOPs.
        for _ in 0..114 {
            let _ = gb.step();
        }
        assert_eq!(gb.read8(0xFF44), Some(1));
        assert_eq!(gb.bus.ppu.ly, 1);
    }

    #[test]
    fn vblank_irq_from_ppu_reaches_vector_0040() {
        let mut gb = SoftGbc::new();
        gb.reset(&MachineConfig::default());
        assert!(gb.bus.poke(0x0100, 0x00)); // NOP at entry
        assert!(gb.bus.poke(0x0040, 0x00)); // VBlank ISR stub
        gb.cpu.pc = 0x0100;
        gb.cpu.sp = 0xFFFE;
        gb.cpu.ime = true;
        assert!(gb.write8(0xFFFF, 0x01)); // IE: VBlank
        assert!(gb.write8(0xFF40, 0x80)); // LCD on

        // Real PPU edge: LY becomes 144 → IF bit 0 (already merged in bus.tick).
        gb.bus.tick_t_cycles(144 * 456);
        assert_eq!(gb.bus.ppu.ly, 144);
        assert_eq!(gb.read8(0xFF0F).unwrap() & 0x01, 0x01);

        let _ = gb.step(); // dispatch before/instead of NOP
        assert_eq!(gb.cpu.pc, 0x0040);
    }

    #[test]
    fn default_config_does_not_stub_ly() {
        let mut gb = SoftGbc::new();
        gb.reset(&MachineConfig::default());
        assert_eq!(gb.read8(0xFF44), Some(0), "LY starts at 0 from PPU");
    }

    #[test]
    fn from_rom_bytes_skips_boot_to_0100() {
        let mut raw = vec![0u8; 0x200];
        raw[0x100] = 0x00; // NOP at entry
        let gb = SoftGbc::from_rom_bytes(&raw);
        assert_eq!(gb.cpu.pc, 0x0100);
        assert_eq!(gb.bus.entry, 0x0100);
    }
}
