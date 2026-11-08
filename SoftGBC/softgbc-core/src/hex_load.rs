//! Commented hex program loader (SoftGBC fixture format).
//!
//! ```text
//! # entry: 0100
//! @0100
//! 00          ; NOP
//! 18 FE       ; JR -2
//! ```

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexImage {
    /// Sparse 64KiB image; unset bytes remain 0.
    pub mem: [u8; 65536],
    /// From `# entry: AAAA`, default 0x0100.
    pub entry: u16,
}

#[derive(Debug)]
pub enum HexLoadError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for HexLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
        }
    }
}

impl std::error::Error for HexLoadError {}

impl From<std::io::Error> for HexLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl HexImage {
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, HexLoadError> {
        let text = fs::read_to_string(path)?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, HexLoadError> {
        let mut mem = [0u8; 65536];
        let mut addr: u32 = 0;
        let mut entry = 0x0100u16;

        for (lineno, raw) in text.lines().enumerate() {
            let line_no = lineno + 1;
            let trimmed = raw.trim();

            if let Some(meta) = trimmed.strip_prefix('#') {
                let meta = meta.trim();
                if let Some(ent) = meta.strip_prefix("entry:") {
                    entry = u16::from_str_radix(ent.trim(), 16).map_err(|_| {
                        HexLoadError::Parse(format!("line {line_no}: bad entry addr"))
                    })?;
                }
                continue;
            }

            let line = strip_comment(raw).trim();
            if line.is_empty() {
                continue;
            }

            if let Some(a) = line.strip_prefix('@') {
                addr = u32::from_str_radix(a.trim(), 16).map_err(|_| {
                    HexLoadError::Parse(format!("line {line_no}: bad @addr"))
                })?;
                if addr > 0xFFFF {
                    return Err(HexLoadError::Parse(format!(
                        "line {line_no}: addr out of range"
                    )));
                }
                continue;
            }

            for tok in line.split_whitespace() {
                let byte = u8::from_str_radix(tok, 16).map_err(|_| {
                    HexLoadError::Parse(format!("line {line_no}: bad byte '{tok}'"))
                })?;
                if addr > 0xFFFF {
                    return Err(HexLoadError::Parse(format!(
                        "line {line_no}: past end of address space"
                    )));
                }
                mem[addr as usize] = byte;
                addr += 1;
            }
        }

        Ok(Self { mem, entry })
    }

    /// Bytes that fall in `0000..=0x7FFF` (ROM window), length `rom_size`.
    pub fn rom_slice(&self, rom_size: usize) -> &[u8] {
        let n = rom_size.min(0x8000).min(self.mem.len());
        &self.mem[..n]
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find(';') {
        Some(i) => &line[..i],
        None => line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_nop_fixture() {
        let img = HexImage::parse(
            r#"
# entry: 0100
@0100
00
00
00
"#,
        )
        .unwrap();
        assert_eq!(img.entry, 0x0100);
        assert_eq!(img.mem[0x0100], 0x00);
        assert_eq!(img.mem[0x0102], 0x00);
        assert_eq!(img.mem[0x0000], 0x00);
    }
}
