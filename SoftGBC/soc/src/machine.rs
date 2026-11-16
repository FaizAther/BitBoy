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

    fn tetris_dx_rom_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../cartridge/roms/Tetris DX (World) (SGB Enhanced) (GB Compatible).gbc",
        )
    }

    fn sp_in_ram(sp: u16) -> bool {
        matches!(sp, 0xC000..=0xDFFF | 0xFF80..=0xFFFE)
    }

    fn pc_is_mapped(pc: u16) -> bool {
        // Instruction fetches belong in ROM, WRAM, or HRAM. Cart RAM, VRAM,
        // OAM, and I/O are not where this ROM's boot runs.
        matches!(pc, 0x0000..=0x7FFF | 0xC000..=0xDFFF | 0xFF80..=0xFFFE)
    }

    fn is_illegal_opcode(op: u8) -> bool {
        matches!(
            op,
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD
        )
    }

    /// Headless Tetris DX skip-boot: no illegal opcode, SP stays in RAM,
    /// LCD turns on, and LY moves. Skips when the ROM file is absent.
    #[test]
    fn tetris_dx_boots_lcd_on_and_ly_advances() {
        let path = tetris_dx_rom_path();
        if !path.is_file() {
            return;
        }
        let mut gb = SoftGbc::from_rom_file(&path).expect("load Tetris DX");
        assert_eq!(gb.cpu.a, 0x01, "skip-boot DMG leftover");
        assert_eq!(gb.cpu.pc, 0x0100);
        assert!(gb.bus.cart.is_some(), "MBC1 image");

        const MAX_STEPS: u32 = 500_000;
        let mut saw_lcd = false;
        let mut ly_when_lcd_on = 0u8;
        let mut ly_advanced = false;
        let mut last_pc = gb.cpu.pc;

        for step_i in 0..MAX_STEPS {
            let pc = gb.cpu.pc;
            let sp = gb.cpu.sp;
            last_pc = pc;
            assert!(
                sp_in_ram(sp),
                "SP {sp:04X} left RAM at step {step_i} PC {pc:04X}"
            );
            assert!(
                pc_is_mapped(pc),
                "PC {pc:04X} left mapped memory at step {step_i} SP {sp:04X}"
            );
            if !gb.cpu.halted {
                let op = gb.read8(pc).unwrap_or(0xFF);
                assert!(
                    !is_illegal_opcode(op),
                    "illegal opcode {op:02X} at PC {pc:04X} step {step_i}"
                );
            }

            let stepped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gb.step()));
            if let Err(payload) = stepped {
                let msg = payload
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "panic".to_string());
                panic!("step panicked at PC {pc:04X} step {step_i}: {msg}");
            }

            let lcdc = gb.read8(0xFF40).unwrap_or(0);
            let ly = gb.read8(0xFF44).unwrap_or(0);
            if lcdc & 0x80 != 0 {
                if !saw_lcd {
                    saw_lcd = true;
                    ly_when_lcd_on = ly;
                } else if ly != ly_when_lcd_on {
                    ly_advanced = true;
                    break;
                }
            }
        }

        assert!(
            saw_lcd,
            "LCD never turned on within {MAX_STEPS} steps (PC {last_pc:04X} halted {} LY {})",
            gb.cpu.halted,
            gb.read8(0xFF44).unwrap_or(0)
        );
        assert!(
            ly_advanced,
            "LCD on but LY stuck at {ly_when_lcd_on} (PC {last_pc:04X})"
        );
    }
}
