//! Minimal MBC1: ROM banking (+ optional external RAM).

/// MBC1 cartridge image + banking state.
#[derive(Debug, Clone)]
pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    /// Low 5 bits of ROM bank (write `$2000`–`$3FFF`).
    rom_bank_lo: u8,
    /// RAM bank / upper ROM bits (write `$4000`–`$5FFF`).
    bank_hi: u8,
    /// 0 = simple/ROM mode, 1 = RAM banking mode.
    mode: u8,
    ram_enable: bool,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut rom = rom;
        if rom.len() < 0x8000 {
            rom.resize(0x8000, 0xFF);
        }
        // SoftGBC: give 8 KiB external RAM when header says any RAM (or always 8K).
        let ram_size = 0x2000;
        Self {
            rom,
            ram: vec![0; ram_size],
            rom_bank_lo: 1,
            bank_hi: 0,
            mode: 0,
            ram_enable: false,
        }
    }

    pub fn rom_len(&self) -> usize {
        self.rom.len()
    }

    fn num_rom_banks(&self) -> usize {
        (self.rom.len() / 0x4000).max(2)
    }

    fn rom_bank(&self) -> usize {
        let mut bank = (self.rom_bank_lo as usize) & 0x1F;
        if bank == 0 {
            bank = 1;
        }
        if self.mode == 0 {
            bank |= ((self.bank_hi as usize) & 0x03) << 5;
        }
        bank % self.num_rom_banks()
    }

    fn bank0_offset(&self) -> usize {
        if self.mode == 0 {
            0
        } else {
            (((self.bank_hi as usize) & 0x03) << 5) % self.num_rom_banks() * 0x4000
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                let i = self.bank0_offset() + addr as usize;
                self.rom.get(i).copied().unwrap_or(0xFF)
            }
            0x4000..=0x7FFF => {
                let i = self.rom_bank() * 0x4000 + (addr as usize - 0x4000);
                self.rom.get(i).copied().unwrap_or(0xFF)
            }
            0xA000..=0xBFFF => {
                if !self.ram_enable {
                    return 0xFF;
                }
                let bank = if self.mode != 0 {
                    (self.bank_hi as usize) & 0x03
                } else {
                    0
                };
                let i = bank * 0x2000 + (addr as usize - 0xA000);
                self.ram.get(i % self.ram.len()).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enable = data & 0x0F == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank_lo = data & 0x1F;
            }
            0x4000..=0x5FFF => {
                self.bank_hi = data & 0x03;
            }
            0x6000..=0x7FFF => {
                self.mode = data & 0x01;
            }
            0xA000..=0xBFFF => {
                if !self.ram_enable || self.ram.is_empty() {
                    return;
                }
                let bank = if self.mode != 0 {
                    (self.bank_hi as usize) & 0x03
                } else {
                    0
                };
                let len = self.ram.len();
                let i = (bank * 0x2000 + (addr as usize - 0xA000)) % len;
                self.ram[i] = data;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom_with_banks() -> Vec<u8> {
        // 4 × 16 KiB banks; mark each bank with its number at offset 0 of the bank.
        let mut rom = vec![0u8; 0x10000];
        for bank in 0..4u8 {
            let base = bank as usize * 0x4000;
            rom[base] = bank;
            rom[base + 0x100] = bank;
        }
        rom[0x0147] = 0x01; // MBC1
        rom
    }

    #[test]
    fn bank0_read_and_switch_bank1() {
        let mut m = Mbc1::new(rom_with_banks());
        assert_eq!(m.read(0x0000), 0);
        // Default rom_bank_lo=1 → bank 1 at $4000
        assert_eq!(m.read(0x4000), 1);
        m.write(0x2000, 2);
        assert_eq!(m.read(0x4000), 2);
        m.write(0x2000, 0); // bank 0 → forced to 1
        assert_eq!(m.read(0x4000), 1);
    }

    #[test]
    fn ram_enable_gate() {
        let mut m = Mbc1::new(rom_with_banks());
        m.write(0xA000, 0x55);
        assert_eq!(m.read(0xA000), 0xFF);
        m.write(0x0000, 0x0A);
        m.write(0xA000, 0x55);
        assert_eq!(m.read(0xA000), 0x55);
    }
}
