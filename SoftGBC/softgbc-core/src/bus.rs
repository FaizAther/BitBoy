use std::path::Path;

use crate::hex_load::{HexImage, HexLoadError};
use crate::memory::{Ram, Rom};

pub trait Bus {
    fn read8(&self, addr: u16) -> Option<u8>;
    fn write8(&mut self, addr: u16, data: u8) -> bool;
}

/// Memory map with compile-time ROM / WRAM / HRAM sizes.
///
/// Default DMG-like sizes: `MemoryBus<0x8000, 0x2000, 0x7F>`.
pub struct MemoryBus<const ROM_SIZE: usize, const WRAM_SIZE: usize, const HRAM_SIZE: usize> {
    rom: Rom<ROM_SIZE>,
    wram: Ram<WRAM_SIZE>,
    hram: Ram<HRAM_SIZE>,
    /// Entry PC from last hex load (`# entry:`), if any.
    pub entry: u16,
}

/// 32 KiB ROM window, 8 KiB WRAM, 127-byte HRAM (`FF80`–`FFFE`).
pub type DmgBus = MemoryBus<0x8000, 0x2000, 0x7F>;

impl<const ROM_SIZE: usize, const WRAM_SIZE: usize, const HRAM_SIZE: usize>
    MemoryBus<ROM_SIZE, WRAM_SIZE, HRAM_SIZE>
{
    /// Empty bus: ROM/WRAM/HRAM zeroed. No file load.
    pub fn new() -> Self {
        Self {
            rom: Rom::new(),
            wram: Ram::new(),
            hram: Ram::new(),
            entry: 0x0100,
        }
    }

    /// Create a bus; if `path` is `Some`, load ROM from that file.
    /// If `None`, same as [`Self::new`] (all zeros).
    ///
    /// Supported files:
    /// - `.hex` — SoftGBC text hex fixtures
    /// - `.gb` / `.gbc` / `.bin` — raw cartridge ROM dumps (first `ROM_SIZE` bytes)
    pub fn new_with_file(path: Option<&Path>) -> Result<Self, HexLoadError> {
        match path {
            None => Ok(Self::new()),
            Some(path) => Self::from_file(path),
        }
    }

    /// Alias for [`Self::new_with_file`].
    pub fn new_with_hex(path: Option<&Path>) -> Result<Self, HexLoadError> {
        Self::new_with_file(path)
    }

    /// Load `.hex` fixture or raw `.gb`/`.gbc`/`.bin` into the ROM window.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, HexLoadError> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "hex" => Self::from_hex_file(path),
            "gb" | "gbc" | "bin" => Self::from_rom_file(path),
            other => Err(HexLoadError::Parse(format!(
                "unsupported ROM file extension '.{other}' (use .hex, .gb, .gbc, or .bin)"
            ))),
        }
    }

    /// Load a SoftGBC hex fixture into ROM (`0000`–`7FFF` portion of the image).
    pub fn from_hex_file(path: impl AsRef<Path>) -> Result<Self, HexLoadError> {
        let image = HexImage::load_file(path)?;
        Ok(Self::from_hex_image(&image))
    }

    /// Load a raw cartridge image (`.gb` / `.gbc`).
    ///
    /// Only the first `ROM_SIZE` bytes are mapped (no MBC banking yet).
    /// Entry defaults to `0x0100` (skip boot).
    pub fn from_rom_file(path: impl AsRef<Path>) -> Result<Self, HexLoadError> {
        let data = std::fs::read(path)?;
        Ok(Self::from_rom_bytes(&data))
    }

    pub fn from_rom_bytes(data: &[u8]) -> Self {
        let mut bus = Self::new();
        bus.entry = 0x0100;
        let n = data.len().min(ROM_SIZE);
        bus.rom.load(0, &data[..n]);
        bus
    }

    pub fn from_hex_image(image: &HexImage) -> Self {
        let mut bus = Self::new();
        bus.entry = image.entry;
        // Copy ROM window bytes that fit in ROM_SIZE.
        let rom_bytes = image.rom_slice(ROM_SIZE);
        bus.rom.load(0, rom_bytes);
        // Bytes placed at @C000 etc. in hex can seed WRAM for tests.
        for addr in 0xC000u16..=0xDFFF {
            let local = (addr - 0xC000) as usize;
            if local < WRAM_SIZE {
                let b = image.mem[addr as usize];
                if b != 0 {
                    let _ = bus.wram.write(local as u16, b);
                }
            }
        }
        for addr in 0xFF80u16..=0xFFFE {
            let local = (addr - 0xFF80) as usize;
            if local < HRAM_SIZE {
                let b = image.mem[addr as usize];
                if b != 0 {
                    let _ = bus.hram.write(local as u16, b);
                }
            }
        }
        bus
    }

    /// Load bytes into ROM (cart image / test program).
    pub fn load_rom(&mut self, offset: usize, data: &[u8]) {
        self.rom.load(offset, data);
    }

    /// Setup helper: write ROM or RAM regardless of CPU write rules.
    pub fn poke(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x0000..=0x7FFF => {
                let local = addr as usize;
                if local < ROM_SIZE {
                    self.rom.load(local, &[data]);
                    true
                } else {
                    false
                }
            }
            _ => self.write8(addr, data),
        }
    }

    fn wram_index(addr: u16) -> Option<u16> {
        let local = match addr {
            0xC000..=0xDFFF => addr - 0xC000,
            // Echo RAM mirrors C000–DDFF
            0xE000..=0xFDFF => addr - 0xE000,
            _ => return None,
        };
        if (local as usize) < WRAM_SIZE {
            Some(local)
        } else {
            None
        }
    }

    fn hram_index(addr: u16) -> Option<u16> {
        if (0xFF80..=0xFFFE).contains(&addr) {
            let local = addr - 0xFF80;
            if (local as usize) < HRAM_SIZE {
                Some(local)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<const ROM_SIZE: usize, const WRAM_SIZE: usize, const HRAM_SIZE: usize> Default
    for MemoryBus<ROM_SIZE, WRAM_SIZE, HRAM_SIZE>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const ROM_SIZE: usize, const WRAM_SIZE: usize, const HRAM_SIZE: usize> Bus
    for MemoryBus<ROM_SIZE, WRAM_SIZE, HRAM_SIZE>
{
    fn read8(&self, addr: u16) -> Option<u8> {
        match addr {
            0x0000..=0x7FFF => {
                let local = addr as usize;
                if local < ROM_SIZE {
                    self.rom.read(addr)
                } else {
                    Some(0xFF)
                }
            }
            0xC000..=0xFDFF => Self::wram_index(addr).and_then(|i| self.wram.read(i)),
            0xFF80..=0xFFFE => Self::hram_index(addr).and_then(|i| self.hram.read(i)),
            // Unmapped / not implemented yet (VRAM, OAM, IO, IE, …)
            _ => Some(0xFF),
        }
    }

    fn write8(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x0000..=0x7FFF => true, // ROM: ignore write, still "ok"
            0xC000..=0xFDFF => {
                if let Some(i) = Self::wram_index(addr) {
                    self.wram.write(i, data)
                } else {
                    true
                }
            }
            0xFF80..=0xFFFE => {
                if let Some(i) = Self::hram_index(addr) {
                    self.hram.write(i, data)
                } else {
                    true
                }
            }
            // VRAM / OAM / IO / IE stubs: ignore for now
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rom_read_only() {
        let mut bus: DmgBus = MemoryBus::new();
        bus.load_rom(0x0100, &[0x00, 0x01, 0x02]);
        assert_eq!(bus.read8(0x0100), Some(0x00));
        assert_eq!(bus.read8(0x0102), Some(0x02));
        assert!(bus.write8(0x0100, 0xFF)); // ignored
        assert_eq!(bus.read8(0x0100), Some(0x00));
    }

    #[test]
    fn wram_round_trip_and_echo() {
        let mut bus: DmgBus = MemoryBus::new();
        assert!(bus.write8(0xC000, 0xAB));
        assert_eq!(bus.read8(0xC000), Some(0xAB));
        assert_eq!(bus.read8(0xE000), Some(0xAB)); // echo
        assert!(bus.write8(0xE001, 0xCD));
        assert_eq!(bus.read8(0xC001), Some(0xCD));
    }

    #[test]
    fn hram_round_trip() {
        let mut bus: DmgBus = MemoryBus::new();
        assert!(bus.write8(0xFF80, 0x11));
        assert!(bus.write8(0xFFFE, 0x22));
        assert_eq!(bus.read8(0xFF80), Some(0x11));
        assert_eq!(bus.read8(0xFFFE), Some(0x22));
    }

    #[test]
    fn unmapped_reads_ff() {
        let bus: DmgBus = MemoryBus::new();
        assert_eq!(bus.read8(0x8000), Some(0xFF)); // VRAM stub
        assert_eq!(bus.read8(0xFF00), Some(0xFF)); // IO stub
        assert_eq!(bus.read8(0xFFFF), Some(0xFF)); // IE stub
    }

    #[test]
    fn poke_writes_rom_for_tests() {
        let mut bus: DmgBus = MemoryBus::new();
        assert!(bus.poke(0x0100, 0xC3));
        assert_eq!(bus.read8(0x0100), Some(0xC3));
    }

    #[test]
    fn new_with_file_none_is_zeroed() {
        let bus = DmgBus::new_with_file(None).unwrap();
        assert_eq!(bus.read8(0x0100), Some(0));
        assert_eq!(bus.entry, 0x0100);
    }

    #[test]
    fn from_hex_image_loads_rom() {
        let image = crate::hex_load::HexImage::parse(
            "# entry: 0100\n@0100\n00\n3E\n42\n",
        )
        .unwrap();
        let bus = DmgBus::from_hex_image(&image);
        assert_eq!(bus.entry, 0x0100);
        assert_eq!(bus.read8(0x0100), Some(0x00));
        assert_eq!(bus.read8(0x0101), Some(0x3E));
        assert_eq!(bus.read8(0x0102), Some(0x42));
    }

    #[test]
    fn from_rom_bytes_loads_gbc_like_blob() {
        let mut raw = vec![0u8; 0x200];
        raw[0x0100] = 0x00; // NOP at entry
        raw[0x0101] = 0x76; // HALT
        // fake header title "TEST"
        raw[0x0134] = b'T';
        raw[0x0135] = b'E';
        raw[0x0136] = b'S';
        raw[0x0137] = b'T';
        let bus = DmgBus::from_rom_bytes(&raw);
        assert_eq!(bus.entry, 0x0100);
        assert_eq!(bus.read8(0x0100), Some(0x00));
        assert_eq!(bus.read8(0x0101), Some(0x76));
        assert_eq!(bus.read8(0x0134), Some(b'T'));
    }
}
