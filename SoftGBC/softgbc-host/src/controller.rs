//! Controllers the host drives — wrap SoftGBC / demos, not the raw PPU GUI.

use dmg_ppu::{DmgPpu, OAM_SIZE, VRAM_SIZE};
use softgbc_utils::{ShadeFrame, ShadeFrameSource};
use soc::SoftGbc;

/// Host-facing PPU session: advance time + expose a shade frame.
///
/// Implemented by SoftGBC / demos. The PPU crate does **not** depend on this.
pub trait PpuController {
    /// Run enough work for one visible frame, then refresh the shade buffer.
    fn advance_frame(&mut self);

    /// Latest rendered shades (valid until the next `advance_frame`).
    fn frame(&self) -> ShadeFrame<'_>;
}

/// SoftGBC machine as a [`PpuController`].
pub struct SoftGbcController {
    pub gb: SoftGbc,
}

impl SoftGbcController {
    pub fn new(gb: SoftGbc) -> Self {
        Self { gb }
    }

    pub fn from_rom(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, softgbc_core::HexLoadError> {
        Ok(Self::new(SoftGbc::from_rom_file(path)?))
    }

    /// Load a raw cartridge image already in memory (skip boot).
    pub fn from_rom_bytes(data: &[u8]) -> Self {
        Self::new(SoftGbc::from_rom_bytes(data))
    }

    /// Poll an [`crate::InputSource`] into the machine joypad (each frame / tick).
    pub fn apply_input(&mut self, source: &mut dyn crate::InputSource) {
        crate::apply_input(&mut self.gb.bus.joypad, source);
    }
}

impl PpuController for SoftGbcController {
    fn advance_frame(&mut self) {
        // One full frame of machine time (154 lines × 456 T), via CPU steps when
        // possible; fall back to raw T ticks so headless still paints.
        let target_t: u32 = 154 * 456;
        let mut t_done = 0u32;
        while t_done < target_t {
            let m = self.gb.step();
            t_done += u32::from(m) * 4;
        }
        self.gb
            .bus
            .ppu
            .render_frame(&self.gb.bus.vram, &self.gb.bus.oam);
    }

    fn frame(&self) -> ShadeFrame<'_> {
        self.gb.bus.ppu.shade_frame()
    }
}

impl ShadeFrameSource for SoftGbcController {
    fn shade_frame(&self) -> ShadeFrame<'_> {
        self.frame()
    }
}

/// Standalone PPU demo (no CPU): solid / scrolling test pattern for the host window.
pub struct DemoPpuController {
    pub ppu: DmgPpu,
    pub vram: [u8; VRAM_SIZE],
    pub oam: [u8; OAM_SIZE],
    scroll: u8,
}

impl DemoPpuController {
    /// LCD+BG on, classic BGP, tile 0 solid shade 3 with a vertical stripe tile 1.
    pub fn solid_stripe() -> Self {
        let mut ppu = DmgPpu::new();
        ppu.LCDC = 0x91;
        ppu.BGP = 0xE4;
        let mut vram = [0u8; VRAM_SIZE];
        // Tile 0: colour 0. Tile 1: colour 3.
        for i in 16..32 {
            vram[i] = 0xFF;
        }
        let map = (0x9800 - 0x8000) as usize;
        for y in 0..32 {
            for x in 0..32 {
                vram[map + y * 32 + x] = if x % 2 == 0 { 0 } else { 1 };
            }
        }
        Self {
            ppu,
            vram,
            oam: [0; OAM_SIZE],
            scroll: 0,
        }
    }
}

impl PpuController for DemoPpuController {
    fn advance_frame(&mut self) {
        self.scroll = self.scroll.wrapping_add(1);
        self.ppu.SCX = self.scroll;
        self.ppu.tick(154 * 456);
        self.ppu.render_frame(&self.vram, &self.oam);
    }

    fn frame(&self) -> ShadeFrame<'_> {
        self.ppu.shade_frame()
    }
}

impl ShadeFrameSource for DemoPpuController {
    fn shade_frame(&self) -> ShadeFrame<'_> {
        self.frame()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{run_loop, NullSink};
    use dmg_ppu::{FRAME_PIXELS, SCREEN_H, SCREEN_W};

    #[test]
    fn demo_controller_fills_framebuffer() {
        let mut demo = DemoPpuController::solid_stripe();
        demo.advance_frame();
        let f = demo.frame();
        assert_eq!(f.width, SCREEN_W);
        assert_eq!(f.height, SCREEN_H);
        assert_eq!(f.shades.len(), FRAME_PIXELS);
        assert!(f.shades.iter().any(|&s| s == 3));
    }

    #[test]
    fn run_loop_with_null_sink() {
        let mut demo = DemoPpuController::solid_stripe();
        let mut sink = NullSink::default();
        run_loop(&mut demo, &mut sink, Some(3)).unwrap();
        assert_eq!(sink.presents, 3);
    }

    #[test]
    fn from_rom_bytes_advances_one_frame() {
        let mut raw = vec![0u8; 0x200];
        raw[0x100] = 0x00; // NOP
        let mut ctl = SoftGbcController::from_rom_bytes(&raw);
        ctl.advance_frame();
        let f = ctl.frame();
        assert_eq!(f.width, SCREEN_W);
        assert_eq!(f.height, SCREEN_H);
        assert_eq!(f.shades.len(), FRAME_PIXELS);
    }

    #[test]
    fn tetris_dx_rom_loads_when_present() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../cartridge/roms")
            .join("Tetris DX (World) (SGB Enhanced) (GB Compatible).gbc");
        if !path.is_file() {
            return;
        }
        let data = std::fs::read(&path).expect("read tetris");
        let ctl = SoftGbcController::from_rom_bytes(&data);
        assert_eq!(ctl.gb.cpu.pc, 0x0100);
        assert!(
            ctl.gb.bus.cart.is_some(),
            "Tetris DX is MBC1 and needs cart banking"
        );
    }
}
