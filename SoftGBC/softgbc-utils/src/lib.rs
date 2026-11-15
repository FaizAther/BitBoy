//! SoftGBC shared helpers.

pub mod bits;
pub mod crc32;
pub mod video;

pub use bits::{BitField, Reg8};
pub use crc32::crc32;
pub use video::{
    shade_to_argb, shades_to_argb, OwnedShadeFrame, Shade, ShadeFrame, ShadeFrameSource, DMG_ARGB,
    GREY_ARGB,
};
