//! DMG PPU.

use peripherals::irq::{IrqLines, IF_STAT, IF_VBLANK};
use peripherals::IoDevice;
use softgbc_utils::{crc32, Reg8, ShadeFrame, ShadeFrameSource};

pub mod lcdc;
pub mod stat;

use lcdc::{
    BG_ENABLE, BG_MAP, LCD_ENABLE, OBJ_ENABLE, OBJ_SIZE, TILE_DATA, WIN_ENABLE, WIN_MAP,
};
use stat::{
    INT_SELECT_MASK, LYC_EQ, LYC_INT, MODE, MODE0_INT, MODE1_INT, MODE2_INT, UNUSED7,
};

// 456 T-cycles per scanline × (144 visible + 10 VBlank)

const MAX_COLUMNS: u16 = 456;
const V_BLANK: u8 = 10;
const MAX_ROWS: u8 = 144 + V_BLANK;

/// v1 fixed Mode 2 / Mode 3 lengths (real Mode 3 varies with sprites).
const MODE2_DOTS: u16 = 80;
const MODE3_DOTS: u16 = 172;

/// LCD width in pixels.
pub const SCREEN_W: usize = 160;
/// LCD height in pixels (visible lines).
pub const SCREEN_H: usize = 144;
/// Framebuffer length (shade bytes 0..=3).
pub const FRAME_PIXELS: usize = SCREEN_W * SCREEN_H;

/// VRAM size `$8000`–`$9FFF`.
pub const VRAM_SIZE: usize = 0x2000;
/// OAM size `$FE00`–`$FE9F` (40 objects × 4 bytes).
pub const OAM_SIZE: usize = 0xA0;
const MAX_OBJ_PER_LINE: usize = 10;

pub struct DmgPpu {
    cycles: u16,

    pub LCDC: u8,
    /// Stored interrupt-select bits only (`INT_SELECT_MASK`); mode/LYC are live on read.
    pub STAT: u8,
    pub SCY: u8,
    pub SCX: u8,
    pub ly: u8, // FF44 read-only
    pub LYC: u8,
    pub DMA: u8,
    pub BGP: u8,
    pub OBP0: u8,
    pub OBP1: u8,
    pub WY: u8,
    pub WX: u8,

    /// Last rendered BG frame: DMG shade 0..=3 per pixel, row-major.
    pub framebuffer: [u8; FRAME_PIXELS],

    prev_mode: u8,
    lyc_equal_prev: bool,
}

impl DmgPpu {
    pub fn new() -> Self {
        let mut ppu = Self {
            cycles: 0,
            LCDC: 0,
            STAT: 0,
            SCY: 0,
            SCX: 0,
            ly: 0,
            LYC: 0,
            DMA: 0,
            BGP: 0,
            OBP0: 0,
            OBP1: 0,
            WY: 0,
            WX: 0,
            framebuffer: [0; FRAME_PIXELS],
            prev_mode: 0,
            lyc_equal_prev: false,
        };
        ppu.prev_mode = ppu.mode_now();
        ppu.lyc_equal_prev = ppu.ly == ppu.LYC;
        ppu
    }

    /// Current PPU mode 0–3 from LY + line cycles.
    pub fn mode_now(&self) -> u8 {
        if self.ly >= 144 {
            return 1;
        }
        if self.cycles < MODE2_DOTS {
            return 2;
        }
        if self.cycles < MODE2_DOTS + MODE3_DOTS {
            return 3;
        }
        0
    }

    pub fn tick(&mut self, mut t_cycles: u32) -> IrqLines {
        let mut irq = IrqLines::NONE;
        let t_c_count: u8 = (t_cycles / MAX_COLUMNS as u32) as u8;
        let ly_modified = self.ly;
        self.ly = (self.ly + t_c_count) % MAX_ROWS as u8;
        t_cycles %= MAX_COLUMNS as u32;
        self.cycles += t_cycles as u16;
        if self.cycles >= MAX_COLUMNS as u16 {
            self.cycles -= MAX_COLUMNS as u16;
            self.ly = (self.ly + 1) % MAX_ROWS as u8;
        }
        if ly_modified != self.ly && self.ly == 144 {
            irq.bits |= IF_VBLANK;
        }
        irq = irq.merge(self.update_stat_edges());
        irq
    }

    fn stat_select_for_mode(mode: u8) -> softgbc_utils::BitField {
        match mode {
            0 => MODE0_INT,
            1 => MODE1_INT,
            2 => MODE2_INT,
            _ => MODE0_INT, // Mode 3 has no STAT mode interrupt
        }
    }

    /// Raise STAT (`IF` bit 1) on *edges*, not levels — same idea as VBlank.
    ///
    /// Call after `ly` / `cycles` (or `LYC`) may have changed. Compares current
    /// mode and LYC==LY against the previous sample stored on the PPU.
    ///
    /// Two independent sources (either can set `IF_STAT`):
    /// 1. **Mode enter** — mode number changed, and that mode's select bit in
    ///    STAT is set (Mode 0 / 1 / 2 only; Mode 3 has no STAT interrupt).
    /// 2. **LYC coincidence** — `ly == LYC` just became true, and STAT bit 6
    ///    (LYC interrupt select) is set. Staying equal does not re-fire.
    fn update_stat_edges(&mut self) -> IrqLines {
        let mut irq = IrqLines::NONE;

        // --- Mode transition (2→3→0 on a line, or →1 in VBlank, etc.) ---
        let mode = self.mode_now();
        if mode != self.prev_mode {
            // Hardware has no "Mode 3 interrupt select"; only 0 / 1 / 2.
            if mode != 3 && Self::stat_select_for_mode(mode).is_set(self.STAT) {
                irq.bits |= IF_STAT;
            }
            self.prev_mode = mode;
        }

        // --- LYC=LY rising edge (false → true) ---
        let lyc_eq = self.ly == self.LYC;
        if lyc_eq && !self.lyc_equal_prev && LYC_INT.is_set(self.STAT) {
            irq.bits |= IF_STAT;
        }
        self.lyc_equal_prev = lyc_eq;

        irq
    }

    fn read_stat(&self) -> u8 {
        let live = Reg8(0)
            .set(MODE, self.mode_now())
            .set_bit(LYC_EQ, self.ly == self.LYC)
            .set_bit(UNUSED7, true)
            .0;
        Reg8(self.STAT).merge_live(INT_SELECT_MASK, live).0
    }

    /// BGP maps colour ID 0..=3 → shade 0..=3.
    pub fn bgp_shade(&self, color_id: u8) -> u8 {
        (self.BGP >> (color_id.saturating_mul(2))) & 0b11
    }

    /// OBP0 / OBP1 shade for OBJ colour ID 0..=3.
    pub fn obp_shade(&self, color_id: u8, use_obp1: bool) -> u8 {
        let pal = if use_obp1 { self.OBP1 } else { self.OBP0 };
        (pal >> (color_id.saturating_mul(2))) & 0b11
    }

    /// CRC-32 of the current [`Self::framebuffer`] (stable golden hash).
    pub fn frame_crc(&self) -> u32 {
        crc32(&self.framebuffer)
    }

    /// Borrow the last rendered shade buffer as a host-agnostic [`ShadeFrame`].
    pub fn shade_frame(&self) -> ShadeFrame<'_> {
        ShadeFrame::new(SCREEN_W, SCREEN_H, &self.framebuffer)
    }

    /// Render BG + window only (`oam` treated as empty). Prefer [`Self::render_frame`].
    pub fn render_bg(&mut self, vram: &[u8]) {
        self.render_frame(vram, &[0; OAM_SIZE]);
    }

    /// Render BG + window + sprites into [`Self::framebuffer`].
    ///
    /// `vram` is `$8000`-based (`VRAM_SIZE` bytes). `oam` is `$FE00`-based (`OAM_SIZE`).
    pub fn render_frame(&mut self, vram: &[u8], oam: &[u8]) {
        debug_assert!(vram.len() >= VRAM_SIZE);
        debug_assert!(oam.len() >= OAM_SIZE);

        if !LCD_ENABLE.is_set(self.LCDC) {
            self.framebuffer.fill(0); // LCD off → blank
            return;
        }

        for sy in 0..SCREEN_H {
            let line_objs = self.objs_on_line(oam, sy as u8);
            for sx in 0..SCREEN_W {
                let shade = self.pixel_composite(vram, oam, sx as u8, sy as u8, &line_objs);
                self.framebuffer[sy * SCREEN_W + sx] = shade;
            }
        }
    }

    fn pixel_composite(
        &self,
        vram: &[u8],
        oam: &[u8],
        sx: u8,
        sy: u8,
        line_objs: &[usize],
    ) -> u8 {
        let bg_id = if BG_ENABLE.is_set(self.LCDC) {
            self.bg_win_color_id(vram, sx, sy)
        } else {
            0
        };
        let mut shade = self.bgp_shade(bg_id);

        if !OBJ_ENABLE.is_set(self.LCDC) {
            return shade;
        }

        if let Some((obj_id, obj_shade, behind)) =
            self.winning_obj_pixel(vram, oam, sx, sy, line_objs)
        {
            // Colour 0 is transparent.
            if obj_id != 0 {
                if behind && bg_id != 0 {
                    // BG/window colours 1–3 hide the OBJ.
                } else {
                    shade = obj_shade;
                }
            }
        }
        shade
    }

    fn bg_win_color_id(&self, vram: &[u8], sx: u8, sy: u8) -> u8 {
        if let Some((win_x, win_y)) = self.window_coords(sx, sy) {
            self.map_color_id(vram, win_map_base(self.LCDC), win_x, win_y)
        } else {
            let bg_x = self.SCX.wrapping_add(sx);
            let bg_y = self.SCY.wrapping_add(sy);
            self.map_color_id(vram, bg_map_base(self.LCDC), bg_x, bg_y)
        }
    }

    /// Screen position of window top-left is `(WX - 7, WY)`.
    /// Returns window-map pixel coords when this screen pixel is inside the window.
    fn window_coords(&self, sx: u8, sy: u8) -> Option<(u8, u8)> {
        if !WIN_ENABLE.is_set(self.LCDC) {
            return None;
        }
        // Pan Docs: visible when WX in 0..=166 and WY in 0..=143 (v1 gate).
        if self.WY > 143 || self.WX > 166 {
            return None;
        }
        if sy < self.WY {
            return None;
        }
        let win_x0 = i16::from(self.WX) - 7;
        let sx = i16::from(sx);
        if sx < win_x0 {
            return None;
        }
        let win_x = (sx - win_x0) as u8;
        let win_y = sy - self.WY;
        Some((win_x, win_y))
    }

    fn map_color_id(&self, vram: &[u8], map_base: u16, px: u8, py: u8) -> u8 {
        let map_x = u16::from(px / 8);
        let map_y = u16::from(py / 8);
        let fine_x = px % 8;
        let fine_y = py % 8;

        let tile_id = vram_at(vram, map_base + map_y * 32 + map_x);
        let tile_addr = tile_data_addr(tile_id, TILE_DATA.is_set(self.LCDC));
        tile_color_id(vram, tile_addr, fine_x, fine_y)
    }

    /// OAM scan for one line: first ≤10 objects whose Y range hits `ly` (Mode 2).
    fn objs_on_line(&self, oam: &[u8], ly: u8) -> [usize; MAX_OBJ_PER_LINE] {
        let mut out = [usize::MAX; MAX_OBJ_PER_LINE];
        let mut n = 0;
        let height = if OBJ_SIZE.is_set(self.LCDC) { 16i16 } else { 8i16 };
        let ly = i16::from(ly);
        for i in 0..40 {
            let y = oam[i * 4];
            let sy0 = i16::from(y) - 16;
            if ly >= sy0 && ly < sy0 + height {
                out[n] = i;
                n += 1;
                if n == MAX_OBJ_PER_LINE {
                    break;
                }
            }
        }
        out
    }

    /// Among line sprites covering `(sx,sy)`, pick DMG drawing winner:
    /// lowest OAM X, then earliest OAM index. Returns `(color_id, shade, bg_priority)`.
    fn winning_obj_pixel(
        &self,
        vram: &[u8],
        oam: &[u8],
        sx: u8,
        sy: u8,
        line_objs: &[usize],
    ) -> Option<(u8, u8, bool)> {
        let mut best: Option<(u8 /*oam_x*/, usize /*index*/, u8, u8, bool)> = None;
        for &i in line_objs {
            if i == usize::MAX {
                break;
            }
            let y = oam[i * 4];
            let x = oam[i * 4 + 1];
            let tile = oam[i * 4 + 2];
            let attr = oam[i * 4 + 3];

            let sx0 = i16::from(x) - 8;
            let sy0 = i16::from(y) - 16;
            let sx = i16::from(sx);
            let sy = i16::from(sy);
            if sx < sx0 || sx >= sx0 + 8 {
                continue;
            }

            let height = if OBJ_SIZE.is_set(self.LCDC) { 16i16 } else { 8i16 };
            if sy < sy0 || sy >= sy0 + height {
                continue;
            }

            let mut row = (sy - sy0) as u8;
            let mut col = (sx - sx0) as u8;
            let y_flip = attr & 0x40 != 0;
            let x_flip = attr & 0x20 != 0;
            if y_flip {
                row = (height as u8 - 1) - row;
            }
            if x_flip {
                col = 7 - col;
            }

            let mut tile_index = tile;
            if OBJ_SIZE.is_set(self.LCDC) {
                // Top tile NN&$FE, bottom NN|$01.
                tile_index = if row < 8 {
                    tile & 0xFE
                } else {
                    row -= 8;
                    tile | 0x01
                };
            }

            let tile_addr = 0x8000 + u16::from(tile_index) * 16;
            let color_id = tile_color_id(vram, tile_addr, col, row);
            let use_obp1 = attr & 0x10 != 0;
            let shade = self.obp_shade(color_id, use_obp1);
            let behind = attr & 0x80 != 0;

            let replace = match best {
                None => true,
                Some((bx, bi, ..)) => x < bx || (x == bx && i < bi),
            };
            if replace {
                best = Some((x, i, color_id, shade, behind));
            }
        }
        best.map(|(_, _, c, s, b)| (c, s, b))
    }
}

fn tile_color_id(vram: &[u8], tile_addr: u16, fine_x: u8, fine_y: u8) -> u8 {
    let row = tile_addr + u16::from(fine_y) * 2;
    let lo = vram_at(vram, row);
    let hi = vram_at(vram, row + 1);
    let bit = 7 - fine_x;
    ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1)
}

fn bg_map_base(lcdc: u8) -> u16 {
    if BG_MAP.is_set(lcdc) {
        0x9C00
    } else {
        0x9800
    }
}

fn win_map_base(lcdc: u8) -> u16 {
    if WIN_MAP.is_set(lcdc) {
        0x9C00
    } else {
        0x9800
    }
}

/// Absolute Game Boy address → byte in a `$8000`-based VRAM slice.
fn vram_at(vram: &[u8], addr: u16) -> u8 {
    vram[(addr - 0x8000) as usize]
}

/// Tile data base address for BG/Window (`LCDC.4`).
fn tile_data_addr(tile_id: u8, unsigned_8000: bool) -> u16 {
    if unsigned_8000 {
        0x8000 + u16::from(tile_id) * 16
    } else {
        // Signed index from $9000 (covers $8800–$97FF).
        (0x9000i32 + i32::from(tile_id as i8) * 16) as u16
    }
}

impl ShadeFrameSource for DmgPpu {
    fn shade_frame(&self) -> ShadeFrame<'_> {
        DmgPpu::shade_frame(self)
    }
}

impl IoDevice for DmgPpu {
    fn handles(addr: u16) -> bool {
        matches!(addr, 0xFF40..=0xFF4B)
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.LCDC,
            0xFF41 => self.read_stat(),
            0xFF42 => self.SCY,
            0xFF43 => self.SCX,
            0xFF44 => self.ly,
            0xFF45 => self.LYC,
            0xFF46 => self.DMA,
            0xFF47 => self.BGP,
            0xFF48 => self.OBP0,
            0xFF49 => self.OBP1,
            0xFF4A => self.WY,
            0xFF4B => self.WX,
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF40 => self.LCDC = data,
            0xFF41 => {
                self.STAT = data & INT_SELECT_MASK;
                // Enabling a select while already in that condition does not
                // re-fire here; edges are tracked from timing / LYC changes.
            }
            0xFF42 => self.SCY = data,
            0xFF43 => self.SCX = data,
            0xFF44 => {} // LY: ignore writes for now
            0xFF45 => {
                self.LYC = data;
                let _ = self.update_stat_edges();
            }
            0xFF46 => self.DMA = data,
            0xFF47 => self.BGP = data,
            0xFF48 => self.OBP0 = data,
            0xFF49 => self.OBP1 = data,
            0xFF4A => self.WY = data,
            0xFF4B => self.WX = data,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use peripherals::irq::IF_STAT;

    #[test]
    pub fn test_single() {
        let mut ppu = DmgPpu::new();
        assert_eq!(ppu.ly, 0);
        for ly in 0..MAX_ROWS as u8 {
            for column in 0..MAX_COLUMNS as u16 {
                assert_eq!(ppu.ly as u32, ly as u32);
                let irq = ppu.tick(1);
                if ly == 143 && column == MAX_COLUMNS - 1 {
                    assert_eq!(irq, IrqLines::vblank());
                } else {
                    assert_eq!(irq, IrqLines::NONE);
                }
            }
        }
        assert_eq!(ppu.ly, 0);
    }

    #[test]
    pub fn test_extra() {
        let mut ppu = DmgPpu::new();
        ppu.tick(MAX_COLUMNS as u32);
        assert_eq!(ppu.ly, 1);
        assert_eq!(ppu.cycles, 0);
        ppu.tick(256);
        assert_eq!(ppu.ly, 1);
        assert_eq!(ppu.cycles, 256);
        ppu.tick(256);
        assert_eq!(ppu.ly, 2);
        assert_eq!(ppu.cycles, 56);
    }

    #[test]
    fn test_fixture_cumulative_t() {
        // total T-cycles → expected LY (fresh PPU each row)
        let cases = [
            (0, 0),
            (456, 1),
            (912, 2),
            (65_664, 144), // 144 * MAX_COLUMNS
            (70_224, 0),   // 154 * MAX_COLUMNS
        ];
        for (t, want_ly) in cases {
            let mut ppu = DmgPpu::new();
            ppu.tick(t as u32);
            assert_eq!(ppu.ly, want_ly, "after {t} T-cycles");
        }
    }

    #[test]
    fn handles_lcd_io_window() {
        assert!(DmgPpu::handles(0xFF40));
        assert!(DmgPpu::handles(0xFF44));
        assert!(DmgPpu::handles(0xFF4B));
        assert!(!DmgPpu::handles(0xFF3F));
        assert!(!DmgPpu::handles(0xFF4C));
        assert!(!DmgPpu::handles(0xFF0F));
    }

    #[test]
    fn lcdc_round_trip() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, 0x80);
        assert_eq!(ppu.read(0xFF40), 0x80);
        ppu.write(0xFF40, 0x91);
        assert_eq!(ppu.read(0xFF40), 0x91);
    }

    #[test]
    fn ff44_tracks_ly_and_ignores_writes() {
        let mut ppu = DmgPpu::new();
        assert_eq!(ppu.read(0xFF44), 0);
        ppu.tick(MAX_COLUMNS as u32);
        assert_eq!(ppu.read(0xFF44), 1);
        assert_eq!(ppu.read(0xFF44), ppu.ly);
        ppu.write(0xFF44, 0x00);
        assert_eq!(ppu.read(0xFF44), 1);
    }

    #[test]
    fn scy_round_trip() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF42, 0x55);
        assert_eq!(ppu.read(0xFF42), 0x55);
    }

    #[test]
    fn mode_walk_visible_line() {
        let mut ppu = DmgPpu::new();
        assert_eq!(ppu.mode_now(), 2);
        assert_eq!(MODE.get(ppu.read(0xFF41)), 2);

        ppu.tick(79);
        assert_eq!(ppu.mode_now(), 2);
        assert_eq!(MODE.get(ppu.read(0xFF41)), 2);

        ppu.tick(1); // cycles == 80
        assert_eq!(ppu.mode_now(), 3);
        assert_eq!(MODE.get(ppu.read(0xFF41)), 3);

        ppu.tick(MODE3_DOTS as u32); // cycles == 252 → Mode 0
        assert_eq!(ppu.mode_now(), 0);
        assert_eq!(MODE.get(ppu.read(0xFF41)), 0);

        ppu.tick((MAX_COLUMNS - (MODE2_DOTS + MODE3_DOTS)) as u32);
        assert_eq!(ppu.ly, 1);
        assert_eq!(ppu.mode_now(), 2);
    }

    #[test]
    fn mode_1_during_vblank() {
        let mut ppu = DmgPpu::new();
        ppu.tick(144 * MAX_COLUMNS as u32);
        assert_eq!(ppu.ly, 144);
        assert_eq!(ppu.mode_now(), 1);
        assert_eq!(MODE.get(ppu.read(0xFF41)), 1);
    }

    #[test]
    fn lyc_eq_flag_tracks_coincidence() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF45, 0);
        assert!(LYC_EQ.is_set(ppu.read(0xFF41)));

        ppu.write(0xFF45, 5);
        assert!(!LYC_EQ.is_set(ppu.read(0xFF41)));

        ppu.tick(5 * MAX_COLUMNS as u32);
        assert_eq!(ppu.ly, 5);
        assert!(LYC_EQ.is_set(ppu.read(0xFF41)));
    }

    #[test]
    fn stat_write_keeps_selects_mode_stays_live() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF41, 0xFF);
        assert_eq!(ppu.read(0xFF41) & INT_SELECT_MASK, INT_SELECT_MASK);
        assert_eq!(MODE.get(ppu.read(0xFF41)), 2);
    }

    #[test]
    fn lyc_stat_irq_edge_once() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF41, LYC_INT.mask_in_place());
        ppu.write(0xFF45, 1);

        let irq = ppu.tick(MAX_COLUMNS as u32);
        assert_eq!(irq.bits & IF_STAT, IF_STAT);
        assert_eq!(ppu.ly, 1);

        let irq2 = ppu.tick(1);
        assert_eq!(irq2.bits & IF_STAT, 0);
    }

    #[test]
    fn mode2_enter_stat_irq_on_new_line() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF41, MODE2_INT.mask_in_place());

        // Leave Mode 2 → Mode 0 on line 0, then enter Mode 2 on line 1.
        ppu.tick((MODE2_DOTS + MODE3_DOTS) as u32);
        assert_eq!(ppu.mode_now(), 0);

        let irq = ppu.tick((MAX_COLUMNS - (MODE2_DOTS + MODE3_DOTS)) as u32);
        assert_eq!(ppu.ly, 1);
        assert_eq!(ppu.mode_now(), 2);
        assert_eq!(irq.bits & IF_STAT, IF_STAT);
    }

    /// LCD on + BG on + `$8000` tiles + `$9800` map (bits 7,4,0).
    const LCDC_BG_8000: u8 = 0x91;
    /// Classic DMG BGP: IDs 0,1,2,3 → shades 0,1,2,3.
    const BGP_E4: u8 = 0xE4;
    /// Golden CRC for solid colour-id-3 screen under BGP `$E4` (all shade 3).
    /// See `fixtures/02_solid_bg.expected.txt`.
    const GOLDEN_SOLID_BG_CRC: u32 = 0x511B_EFBF;

    fn solid_bg_vram() -> [u8; VRAM_SIZE] {
        let mut vram = [0u8; VRAM_SIZE];
        // Tile 0 at $8000: every pixel colour_id 3 (lo=hi=0xFF each row).
        for i in 0..16 {
            vram[i] = 0xFF;
        }
        // Map $9800: all tile index 0 (already zeroed).
        vram
    }

    #[test]
    fn solid_bg_frame_crc_matches_golden() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_BG_8000);
        ppu.write(0xFF47, BGP_E4);
        ppu.write(0xFF42, 0); // SCY
        ppu.write(0xFF43, 0); // SCX

        let vram = solid_bg_vram();
        ppu.render_bg(&vram);

        let expect_shade = ppu.bgp_shade(3);
        assert_eq!(expect_shade, 3);
        assert_eq!(ppu.framebuffer[0], expect_shade);
        assert_eq!(ppu.framebuffer[FRAME_PIXELS - 1], expect_shade);
        assert_eq!(
            ppu.frame_crc(),
            GOLDEN_SOLID_BG_CRC,
            "update GOLDEN_SOLID_BG_CRC / fixtures/02_solid_bg.expected.txt"
        );
    }

    #[test]
    fn bg_off_fills_blank_shade0() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, 0x90); // LCD on, BG off, $8000 tiles
        ppu.write(0xFF47, BGP_E4);
        ppu.render_bg(&solid_bg_vram());
        assert!(ppu.framebuffer.iter().all(|&p| p == 0));
    }

    #[test]
    fn scx_8_shifts_two_tone_left_by_one_tile() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_BG_8000);
        ppu.write(0xFF47, BGP_E4);

        let mut vram = [0u8; VRAM_SIZE];
        // Tile 0: solid id 0 (white). Tile 1: solid id 3 (black under BGP).
        for i in 16..32 {
            vram[i] = 0xFF;
        }
        // Map row 0: tile0, tile1, tile0, tile1, …
        let map0 = (0x9800 - 0x8000) as usize;
        for x in 0..32 {
            vram[map0 + x] = if x % 2 == 0 { 0 } else { 1 };
        }
        // Copy that pattern to all 32 map rows so every screen line matches.
        for y in 1..32 {
            for x in 0..32 {
                vram[map0 + y * 32 + x] = vram[map0 + x];
            }
        }

        ppu.write(0xFF43, 0);
        ppu.render_bg(&vram);
        let left = ppu.framebuffer[0];
        let at_8 = ppu.framebuffer[8];

        ppu.write(0xFF43, 8); // SCX = one tile
        ppu.render_bg(&vram);
        assert_eq!(ppu.framebuffer[0], at_8, "SCX=8 should show former x=8 at x=0");
        assert_ne!(left, at_8);
    }

    /// Prints CRC for maintaining the golden (ignored in normal `cargo test`).
    #[test]
    #[ignore]
    fn print_solid_bg_crc() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_BG_8000);
        ppu.write(0xFF47, BGP_E4);
        ppu.render_bg(&solid_bg_vram());
        eprintln!("solid_bg crc32 = 0x{:08X}", ppu.frame_crc());
    }

    /// LCD on, window on, BG on, tiles `$8000`, BG map `$9800`, win map `$9C00`.
    /// bits: 7,6,5,4,0 → `0xF1`.
    const LCDC_WIN_9C00: u8 = 0xF1;
    /// See `fixtures/03_window_corner.expected.txt`.
    const GOLDEN_WINDOW_CORNER_CRC: u32 = 0xAB9E_8A54;

    fn window_corner_vram() -> [u8; VRAM_SIZE] {
        let mut vram = [0u8; VRAM_SIZE];
        // Tile 0: solid colour_id 0 (BG). Tile 1: solid colour_id 3 (window).
        for i in 16..32 {
            vram[i] = 0xFF;
        }
        // BG map $9800 → tile 0 (zeros).
        // Window map $9C00 → tile 1.
        let win_map = (0x9C00 - 0x8000) as usize;
        for i in 0..(32 * 32) {
            vram[win_map + i] = 1;
        }
        vram
    }

    #[test]
    fn window_covers_bottom_right_corner() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_WIN_9C00);
        ppu.write(0xFF47, BGP_E4);
        // Window top-left at screen (80, 72): WX = 80+7 = 87, WY = 72.
        ppu.write(0xFF4A, 72); // WY
        ppu.write(0xFF4B, 87); // WX

        ppu.render_bg(&window_corner_vram());

        let bg = ppu.bgp_shade(0);
        let win = ppu.bgp_shade(3);
        assert_eq!(ppu.framebuffer[0], bg); // top-left BG
        assert_eq!(
            ppu.framebuffer[72 * SCREEN_W + 80],
            win,
            "window origin pixel"
        );
        assert_eq!(ppu.framebuffer[FRAME_PIXELS - 1], win); // bottom-right
        assert_eq!(
            ppu.framebuffer[72 * SCREEN_W + 79],
            bg,
            "just left of window stays BG"
        );
        assert_eq!(
            ppu.frame_crc(),
            GOLDEN_WINDOW_CORNER_CRC,
            "update GOLDEN_WINDOW_CORNER_CRC / fixtures/03_window_corner.expected.txt"
        );
    }

    #[test]
    fn window_disabled_leaves_bg() {
        let mut ppu = DmgPpu::new();
        // Same as LCDC_WIN_9C00 but clear window enable (bit 5) → 0xD1
        ppu.write(0xFF40, 0xD1);
        ppu.write(0xFF47, BGP_E4);
        ppu.write(0xFF4A, 72);
        ppu.write(0xFF4B, 87);
        ppu.render_bg(&window_corner_vram());
        assert!(ppu.framebuffer.iter().all(|&p| p == 0));
    }

    #[test]
    #[ignore]
    fn print_window_corner_crc() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_WIN_9C00);
        ppu.write(0xFF47, BGP_E4);
        ppu.write(0xFF4A, 72);
        ppu.write(0xFF4B, 87);
        ppu.render_bg(&window_corner_vram());
        eprintln!("window_corner crc32 = 0x{:08X}", ppu.frame_crc());
    }

    /// LCD + OBJ + `$8000` tiles + BG: bits 7,4,1,0 → `0x93`.
    const LCDC_OBJ_BG: u8 = 0x93;
    /// See `fixtures/04_sprite_one.expected.txt`.
    const GOLDEN_SPRITE_ONE_CRC: u32 = 0x6222_800E;

    fn sprite_one_scene() -> ([u8; VRAM_SIZE], [u8; OAM_SIZE]) {
        let mut vram = [0u8; VRAM_SIZE];
        // Tile 0: BG white (id 0). Tile 1: solid OBJ id 3.
        for i in 16..32 {
            vram[i] = 0xFF;
        }
        let mut oam = [0u8; OAM_SIZE];
        // One 8×8 OBJ at screen (40, 40): Y=56, X=48, tile 1, flags 0 (OBP0, above BG).
        oam[0] = 56;
        oam[1] = 48;
        oam[2] = 1;
        oam[3] = 0;
        (vram, oam)
    }

    #[test]
    fn one_sprite_over_bg_golden() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_OBJ_BG);
        ppu.write(0xFF47, BGP_E4);
        ppu.write(0xFF48, BGP_E4); // OBP0 same mapping
        let (vram, oam) = sprite_one_scene();
        ppu.render_frame(&vram, &oam);

        let bg = ppu.bgp_shade(0);
        let obj = ppu.obp_shade(3, false);
        assert_eq!(ppu.framebuffer[0], bg);
        assert_eq!(ppu.framebuffer[40 * SCREEN_W + 40], obj);
        assert_eq!(ppu.framebuffer[40 * SCREEN_W + 39], bg);
        assert_eq!(ppu.framebuffer[39 * SCREEN_W + 40], bg);
        assert_eq!(
            ppu.frame_crc(),
            GOLDEN_SPRITE_ONE_CRC,
            "update GOLDEN_SPRITE_ONE_CRC / fixtures/04_sprite_one.expected.txt"
        );
    }

    #[test]
    fn sprite_colour0_is_transparent() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_OBJ_BG);
        ppu.write(0xFF47, BGP_E4);
        ppu.write(0xFF48, BGP_E4);
        let mut vram = solid_bg_vram(); // BG solid shade 3
        // Tile 1 = all zeros → OBJ colour 0 (transparent)
        for i in 16..32 {
            vram[i] = 0;
        }
        let mut oam = [0u8; OAM_SIZE];
        oam[0] = 56;
        oam[1] = 48;
        oam[2] = 1;
        oam[3] = 0;
        ppu.render_frame(&vram, &oam);
        assert_eq!(ppu.framebuffer[40 * SCREEN_W + 40], 3, "BG shows through");
    }

    #[test]
    fn obj_behind_bg_hidden_on_colour_nonzero() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_OBJ_BG);
        ppu.write(0xFF47, BGP_E4);
        ppu.write(0xFF48, BGP_E4);
        let mut vram = solid_bg_vram(); // BG id 3
        for i in 16..32 {
            vram[i] = 0xFF; // OBJ tile solid id 3
        }
        let mut oam = [0u8; OAM_SIZE];
        oam[0] = 56;
        oam[1] = 48;
        oam[2] = 1;
        oam[3] = 0x80; // BG priority
        ppu.render_frame(&vram, &oam);
        assert_eq!(ppu.framebuffer[40 * SCREEN_W + 40], 3); // BG wins
    }

    #[test]
    #[ignore]
    fn print_sprite_one_crc() {
        let mut ppu = DmgPpu::new();
        ppu.write(0xFF40, LCDC_OBJ_BG);
        ppu.write(0xFF47, BGP_E4);
        ppu.write(0xFF48, BGP_E4);
        let (vram, oam) = sprite_one_scene();
        ppu.render_frame(&vram, &oam);
        eprintln!("sprite_one crc32 = 0x{:08X}", ppu.frame_crc());
    }
}
