//! Local / server GUI window (X11 via minifb — works with SSH `-X` / remote desktop).

use minifb::{Key, Scale, Window, WindowOptions};
use softgbc_utils::{shades_to_argb, ShadeFrame, DMG_ARGB, GREY_ARGB};
use std::sync::{Arc, Mutex};

use crate::{Buttons, FrameSink, InputSource, PresentError};

/// Logical host keys for SoftGBC (minifb / web / tests map into these).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostKey {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

/// Map pressed host keys → [`Buttons`] (arrows = d-pad, Z/X = B/A, Enter/Backspace = Start/Select).
pub fn keys_to_buttons(down: impl Fn(HostKey) -> bool) -> Buttons {
    let mut b = Buttons::NONE;
    if down(HostKey::Right) {
        b |= Buttons::RIGHT;
    }
    if down(HostKey::Left) {
        b |= Buttons::LEFT;
    }
    if down(HostKey::Up) {
        b |= Buttons::UP;
    }
    if down(HostKey::Down) {
        b |= Buttons::DOWN;
    }
    if down(HostKey::A) {
        b |= Buttons::A;
    }
    if down(HostKey::B) {
        b |= Buttons::B;
    }
    if down(HostKey::Select) {
        b |= Buttons::SELECT;
    }
    if down(HostKey::Start) {
        b |= Buttons::START;
    }
    b
}

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
    /// Updated every [`FrameSink::present`] so play loop can poll without borrowing the window.
    keys_out: Option<Arc<Mutex<Buttons>>>,
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
            keys_out: None,
        })
    }

    /// Publish sampled keys into `keys` on every present (for [`SharedKeyboardInput`]).
    pub fn with_key_sink(mut self, keys: Arc<Mutex<Buttons>>) -> Self {
        self.keys_out = Some(keys);
        self
    }

    /// Sample keyboard into SoftGBC [`Buttons`].
    pub fn sample_buttons(&self) -> Buttons {
        keys_to_buttons(|k| {
            let key = match k {
                HostKey::Right => Key::Right,
                HostKey::Left => Key::Left,
                HostKey::Up => Key::Up,
                HostKey::Down => Key::Down,
                HostKey::A => Key::X,
                HostKey::B => Key::Z,
                HostKey::Select => Key::Backspace,
                HostKey::Start => Key::Enter,
            };
            self.window.is_key_down(key)
        })
    }
}

/// [`InputSource`] reading buttons published by [`HostWindow::with_key_sink`].
#[derive(Clone)]
pub struct SharedKeyboardInput {
    keys: Arc<Mutex<Buttons>>,
}

impl SharedKeyboardInput {
    pub fn new(keys: Arc<Mutex<Buttons>>) -> Self {
        Self { keys }
    }

    pub fn pair() -> (Arc<Mutex<Buttons>>, Self) {
        let keys = Arc::new(Mutex::new(Buttons::NONE));
        (keys.clone(), Self::new(keys))
    }
}

impl InputSource for SharedKeyboardInput {
    fn poll(&mut self) -> Buttons {
        *self.keys.lock().expect("keyboard lock")
    }

    fn name(&self) -> &str {
        "window-keyboard"
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
        if let Some(keys) = &self.keys_out {
            *keys.lock().expect("keyboard lock") = self.sample_buttons();
        }
        shades_to_argb(frame, self.palette, &mut self.argb);
        self.window
            .update_with_buffer(&self.argb, self.width, self.height)
            .map_err(|e| PresentError::Backend(e.to_string()))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_to_buttons_maps_arrows_and_face() {
        let b = keys_to_buttons(|k| matches!(k, HostKey::Left | HostKey::A));
        assert!(b.contains(Buttons::LEFT));
        assert!(b.contains(Buttons::A));
        assert!(!b.contains(Buttons::RIGHT));
        assert!(!b.contains(Buttons::B));
    }

    #[test]
    fn keys_to_buttons_none_when_up() {
        assert!(keys_to_buttons(|_| false).is_empty());
    }

    #[test]
    fn shared_keyboard_input_reads_slot() {
        let (slot, mut input) = SharedKeyboardInput::pair();
        *slot.lock().unwrap() = Buttons::START | Buttons::A;
        assert!(input.poll().contains(Buttons::START));
        assert!(input.poll().contains(Buttons::A));
        assert_eq!(input.name(), "window-keyboard");
    }
}
