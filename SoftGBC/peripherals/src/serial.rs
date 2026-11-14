//! Link cable (`SB` / `SC`).
//!
//! Instant internal-clock transfer: on write to `SC` with bit 7 set, append `SB`
//! to [`Self::output`] and clear the transfer-in-progress bit. Enough for
//! headless test ROMs (Blargg, etc.); not a real serial peer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Serial {
    pub sb: u8,
    pub sc: u8,
    /// Bytes “sent” on the link (host-visible log).
    pub output: String,
}

impl Default for Serial {
    fn default() -> Self {
        Self::new()
    }
}

impl Serial {
    pub fn new() -> Self {
        Self {
            sb: 0x00,
            sc: 0x7E,
            output: String::new(),
        }
    }

    pub fn handles(addr: u16) -> bool {
        matches!(addr, 0xFF01 | 0xFF02)
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF01 => self.sb = data,
            0xFF02 => {
                self.sc = data;
                if data & 0x80 != 0 {
                    let ch = self.sb as char;
                    if ch == '\n' || ch.is_ascii_graphic() || ch == ' ' {
                        self.output.push(ch);
                    }
                    self.sc = data & 0x7F;
                }
            }
            _ => {}
        }
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }
}
