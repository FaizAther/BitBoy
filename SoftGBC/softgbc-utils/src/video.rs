//! Host-agnostic DMG video types.
//!
//! The PPU (and SoftGBC) produce [`ShadeFrame`] buffers. Host windows, PPM
//! dumps, and tests consume them — nothing here opens a GUI.

/// DMG greyscale shade `0..=3` (BGP/OBP index after palette lookup).
pub type Shade = u8;

/// Borrowed DMG framebuffer (row-major shades).
#[derive(Debug, Clone, Copy)]
pub struct ShadeFrame<'a> {
    pub width: usize,
    pub height: usize,
    pub shades: &'a [Shade],
}

impl<'a> ShadeFrame<'a> {
    pub fn new(width: usize, height: usize, shades: &'a [Shade]) -> Self {
        debug_assert_eq!(shades.len(), width.saturating_mul(height));
        Self {
            width,
            height,
            shades,
        }
    }

    pub fn pixel(&self, x: usize, y: usize) -> Shade {
        self.shades[y * self.width + x]
    }

    /// Clone into an owned packet for channels / WebRTC / network.
    pub fn to_owned(&self) -> OwnedShadeFrame {
        OwnedShadeFrame {
            width: self.width,
            height: self.height,
            shades: self.shades.to_vec(),
        }
    }
}

/// Owned DMG shade frame — safe to send across threads and remotes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedShadeFrame {
    pub width: usize,
    pub height: usize,
    pub shades: Vec<Shade>,
}

impl OwnedShadeFrame {
    pub fn as_frame(&self) -> ShadeFrame<'_> {
        ShadeFrame::new(self.width, self.height, &self.shades)
    }
}

/// Something that can expose a DMG shade framebuffer (PPU, SoftGbc, mock).
pub trait ShadeFrameSource {
    fn shade_frame(&self) -> ShadeFrame<'_>;
}

/// Classic DMG greys as opaque `0xAARRGGBB` (for host blitting).
pub const DMG_ARGB: [u32; 4] = [
    0xFF_9B_BC_0F, // 0 lightest (olive Game Boy LCD-ish)
    0xFF_8B_AC_0F,
    0xFF_30_62_30,
    0xFF_0F_38_0F, // 3 darkest
];

/// Neutral greys (useful for tests / screenshots).
pub const GREY_ARGB: [u32; 4] = [
    0xFF_FF_FF_FF, // 0 white
    0xFF_AA_AA_AA,
    0xFF_55_55_55,
    0xFF_00_00_00, // 3 black
];

pub fn shade_to_argb(shade: Shade, palette: &[u32; 4]) -> u32 {
    palette[(shade & 3) as usize]
}

/// Expand a shade frame into a packed ARGB8888 buffer (length = w*h).
pub fn shades_to_argb(frame: ShadeFrame<'_>, palette: &[u32; 4], out: &mut [u32]) {
    assert_eq!(out.len(), frame.shades.len());
    for (dst, &s) in out.iter_mut().zip(frame.shades.iter()) {
        *dst = shade_to_argb(s, palette);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shade_to_argb_uses_palette() {
        assert_eq!(shade_to_argb(0, &GREY_ARGB), 0xFF_FF_FF_FF);
        assert_eq!(shade_to_argb(3, &GREY_ARGB), 0xFF_00_00_00);
        assert_eq!(shade_to_argb(5, &GREY_ARGB), GREY_ARGB[1]); // masked
    }

    #[test]
    fn shades_to_argb_fills_buffer() {
        let shades = [0u8, 1, 2, 3];
        let frame = ShadeFrame::new(2, 2, &shades);
        let mut out = [0u32; 4];
        shades_to_argb(frame, &GREY_ARGB, &mut out);
        assert_eq!(out[0], GREY_ARGB[0]);
        assert_eq!(out[3], GREY_ARGB[3]);
    }
}
