//! OAM DMA (`FF46`) — stub; instant memcpy can live here later.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dma {
    pub page: u8,
    pub active: bool,
}

impl Dma {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handles(addr: u16) -> bool {
        addr == 0xFF46
    }

    pub fn read(&self, _addr: u16) -> u8 {
        self.page
    }

    pub fn write(&mut self, _addr: u16, data: u8) {
        self.page = data;
        self.active = true; // SoC will perform copy / clear when wired
    }
}
