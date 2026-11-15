//! CRC-32 (ISO-HDLC / zlib polynomial `0xEDB88320`).

const POLY: u32 = 0xEDB88320;

/// CRC-32 of `data`, init `0xFFFF_FFFF`, final XOR `0xFFFF_FFFF`.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ POLY;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_zero() {
        assert_eq!(crc32(&[]), 0);
    }

    #[test]
    fn known_string() {
        // "123456789" → 0xCBF43926 (common CRC-32 check vector)
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
