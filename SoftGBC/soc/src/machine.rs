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

/// Skip-boot / headless defaults for bringing up a ROM without a PPU yet.
#[derive(Debug, Clone, Copy)]
pub struct MachineConfig {
    /// `A` after skip-boot (DMG boot leftover is `0x01`; many tests expect this).
    pub a: u8,
    /// If set, poke `LY` (`FF44`) so LCD waits don’t spin forever without a PPU.
    pub stub_ly: Option<u8>,
    /// Initial `IF` (`FF0F`). `0xE0` matches common “unused bits high” reads.
    pub if_init: u8,
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            a: 0x01,
            stub_ly: Some(144),
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

    /// Apply config and set `PC` to cart entry (skip boot).
    pub fn reset(&mut self, cfg: &MachineConfig) {
        self.cpu = Cpu::new();
        self.cpu.pc = self.bus.entry;
        self.cpu.a = cfg.a;
        let _ = self.bus.write8(0xFF0F, cfg.if_init);
        if let Some(ly) = cfg.stub_ly {
            let _ = self.bus.write8(0xFF44, ly);
        }
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
