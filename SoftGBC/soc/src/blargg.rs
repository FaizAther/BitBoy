//! Headless Blargg ROM runner (serial SB/SC capture).

use softgbc_core::{Bus, DmgBus};
use sm83_iss::Cpu;
use std::path::Path;

/// DMG bus + VRAM/OAM/IO stubs enough for Blargg `cpu_instrs` shell.
pub struct BlarggBus {
    inner: DmgBus,
    vram: [u8; 0x2000],
    oam: [u8; 0xA0],
    io: [u8; 0x80],
    ie: u8,
    pub serial: String,
}

impl BlarggBus {
    pub fn from_rom_file(path: impl AsRef<Path>) -> Result<Self, softgbc_core::HexLoadError> {
        let inner = DmgBus::from_rom_file(path)?;
        Ok(Self::from_inner(inner))
    }

    fn from_inner(inner: DmgBus) -> Self {
        let mut bus = Self {
            inner,
            vram: [0xFF; 0x2000],
            oam: [0xFF; 0xA0],
            io: [0xFF; 0x80],
            ie: 0,
            serial: String::new(),
        };
        // LY always in vblank so console_wait_vbl exits quickly.
        bus.io[0x44] = 144;
        bus
    }

    pub fn entry(&self) -> u16 {
        self.inner.entry
    }
}

impl Bus for BlarggBus {
    fn read8(&self, addr: u16) -> Option<u8> {
        match addr {
            0x8000..=0x9FFF => Some(self.vram[(addr - 0x8000) as usize]),
            0xFE00..=0xFE9F => Some(self.oam[(addr - 0xFE00) as usize]),
            0xFF00..=0xFF7F => Some(self.io[(addr - 0xFF00) as usize]),
            0xFFFF => Some(self.ie),
            _ => self.inner.read8(addr),
        }
    }

    fn write8(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x8000..=0x9FFF => {
                self.vram[(addr - 0x8000) as usize] = data;
                true
            }
            0xFE00..=0xFE9F => {
                self.oam[(addr - 0xFE00) as usize] = data;
                true
            }
            0xFF00..=0xFF7F => {
                let i = (addr - 0xFF00) as usize;
                self.io[i] = data;
                // Serial transfer start (internal clock): capture SB instantly.
                if addr == 0xFF02 && data & 0x80 != 0 {
                    let ch = self.io[0x01] as char;
                    if ch == '\n' || (ch.is_ascii_graphic() || ch == ' ') {
                        self.serial.push(ch);
                    }
                    self.io[0x02] = data & 0x7F; // clear transfer-in-progress
                }
                true
            }
            0xFFFF => {
                self.ie = data;
                true
            }
            _ => self.inner.write8(addr, data),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlarggResult {
    Passed,
    Failed,
    Timeout,
}

/// Run until serial contains Passed/Failed or `max_steps` instructions.
pub fn run_rom(path: impl AsRef<Path>, max_steps: u64) -> Result<(BlarggResult, String), softgbc_core::HexLoadError> {
    let mut bus = BlarggBus::from_rom_file(path)?;
    let mut cpu = Cpu::new();
    cpu.pc = bus.entry();
    cpu.a = 0x01; // DMG boot leftover id Blargg stores as gb_id

    for _ in 0..max_steps {
        let _ = cpu.step(&mut bus);
        if bus.serial.contains("Passed") {
            return Ok((BlarggResult::Passed, bus.serial));
        }
        if bus.serial.contains("Failed") {
            return Ok((BlarggResult::Failed, bus.serial));
        }
        if cpu.halted {
            // Blargg ends in `jr forever`, not HALT — but stop if stuck halted.
            break;
        }
    }
    Ok((BlarggResult::Timeout, bus.serial))
}
