pub struct Memory<const SIZE: usize, const IS_RAM: bool> {
    memory : [u8; SIZE],
}

impl<const SIZE: usize, const IS_RAM: bool> Memory<SIZE, IS_RAM> {
    pub fn new() -> Self {
        Self {
            memory: [0; SIZE]
        }
    }

    pub fn size(&self) -> usize {
        SIZE
    }

    pub fn read(&self, addr:u16) -> Option<u8> {
        let addr = addr as usize;

        if addr < SIZE {
            Some(self.memory[addr])
        } else {
            None
        }
    }

    pub fn load(&mut self, offset: usize, data: &[u8]) {
        let end = offset + data.len();

        assert!(end <= SIZE, "RAM write out of bounds");

        self.memory[offset..end].copy_from_slice(data);
    }
}

impl<const SIZE: usize> Memory<SIZE, true> {
    pub fn write(&mut self, addr:u16, data:u8) -> bool {
        let addr = addr as usize;

        if addr < SIZE {
            self.memory[addr] = data;
            return true;
        }
        return false;
    }
}

pub type Ram<const SIZE: usize> = Memory<SIZE, true>;
pub type Rom<const SIZE: usize> = Memory<SIZE, false>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ram_is_zeroed() {
        let ram = Memory::<0x10, false>::new();

        assert_eq!(ram.size(), 16);
        assert_eq!(ram.read(0x0), Some(0));
        assert_eq!(ram.read(0xF), Some(0));
    }

    #[test]
    fn write_then_read_round_trips() {
        let mut ram = Memory::<0x10, true>::new();

        assert!(ram.write(0x5, 0xAB));
        assert_eq!(ram.read(0x5), Some(0xAB));
    }

    #[test]
    fn out_of_bounds_read_returns_none() {
        let ram = Memory::<0x10, false>::new();

        assert_eq!(ram.read(0xF1), None);
    }

    #[test]
    fn out_of_bounds_write_returns_false() {
        let mut ram = Memory::<0x10, true>::new();

        assert!(!ram.write(0x10, 0xFF));
    }

    #[test]
    fn load_writes_consecutive_bytes() {
        let mut ram = Memory::<0x10, false>::new();

        ram.load(2, &[0x10, 0x20, 0x30]);

        assert_eq!(ram.read(0x2), Some(0x10));
        assert_eq!(ram.read(0x3), Some(0x20));
        assert_eq!(ram.read(0x4), Some(0x30));
    }

    #[test]
    #[should_panic(expected = "RAM write out of bounds")]
    fn load_writes_consecutive_bytes_end() {
        let mut ram = Memory::<0x10, false>::new();

        ram.load(0xE, &[0x10, 0x20, 0x30]);
    }
}