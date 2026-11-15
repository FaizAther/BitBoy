//! Local / server GUI window (X11 via minifb — works with SSH `-X` / remote desktop).

use minifb::{Key, Scale, Window, WindowOptions};
use softgbc_utils::{shades_to_argb, ShadeFrame, DMG_ARGB, GREY_ARGB};

use crate::{FrameSink, PresentError};

/// How to map DMG shades to ARGB in the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowPalette {
    #[default]
    DmgGreen,
    Grey,
}

/// Config for [`HostWindow`].
#[derive(Debug, Clone)]
pub struct HostWindowConfig {
    pub title: String,
    pub scale: u8,
    pub palette: WindowPalette,
}

impl Default for HostWindowConfig {
    fn default() -> Self {
        Self {
            title: "SoftGBC".into(),
            scale: 4,
            palette: WindowPalette::DmgGreen,
        }
    }
}

/// Popup framebuffer window on the machine (or forwarded display).
pub struct HostWindow {
    window: Window,
    argb: Vec<u32>,
    width: usize,
    height: usize,
    palette: &'static [u32; 4],
}

impl HostWindow {
    pub fn open(cfg: HostWindowConfig, width: usize, height: usize) -> Result<Self, PresentError> {
        let scale = match cfg.scale {
            0 | 1 => Scale::X1,
            2 => Scale::X2,
            3 => Scale::X4, // minifb has X1,X2,X4,X8,X16,X32 — map 3→4
            4 => Scale::X4,
            _ => Scale::X8,
        };
        let mut window = Window::new(
            &cfg.title,
            width,
            height,
            WindowOptions {
                resize: false,
                scale,
                ..WindowOptions::default()
            },
        )
        .map_err(|e| PresentError::Backend(e.to_string()))?;
        window.set_target_fps(60);
        let palette = match cfg.palette {
            WindowPalette::DmgGreen => &DMG_ARGB,
            WindowPalette::Grey => &GREY_ARGB,
        };
        Ok(Self {
            window,
            argb: vec![0; width * height],
            width,
            height,
            palette,
        })
    }
}

impl FrameSink for HostWindow {
    fn present(&mut self, frame: ShadeFrame<'_>) -> Result<bool, PresentError> {
        if !self.window.is_open() || self.window.is_key_down(Key::Escape) {
            return Ok(false);
        }
        if frame.width != self.width || frame.height != self.height {
            return Err(PresentError::Backend(format!(
                "frame {}x{} != window {}x{}",
                frame.width, frame.height, self.width, self.height
            )));
        }
        shades_to_argb(frame, self.palette, &mut self.argb);
        self.window
            .update_with_buffer(&self.argb, self.width, self.height)
            .map_err(|e| PresentError::Backend(e.to_string()))?;
        Ok(true)
    }
}
