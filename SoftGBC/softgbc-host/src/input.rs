//! Pluggable joypad **input sources** (host-side).
//!
//! Mirrors [`crate::FrameSink`] / [`crate::RemoteEgress`]: SoftGBC / Joypad never
//! know about keyboards, Bluetooth, or the browser — only [`Buttons`].
//!
//! ```text
//! Keyboard / Web / Bluetooth / Manual
//!        │
//!        ▼
//!   InputSource::poll() → Buttons
//!        │
//!        ▼
//!   Joypad::set_pressed  (peripherals)
//! ```
//!
//! Later: WebSocket / WebRTC data channel / BLE HID implement [`InputSource`]
//! (or push into [`ChannelInput`]) without touching the PPU or MMIO.

use peripherals::joypad::Buttons;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

/// Host input — **not** part of the Joypad MMIO device.
///
/// Implementations: keyboard window, web page, Bluetooth, manual (tests), null.
pub trait InputSource {
    /// Latest pressed set (released buttons are `false`).
    fn poll(&mut self) -> Buttons;

    /// Human-readable backend (`"manual"`, `"web-stub"`, `"bluetooth-stub"`, …).
    fn name(&self) -> &str;
}

/// Headless / default: nothing pressed.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullInput;

impl InputSource for NullInput {
    fn poll(&mut self) -> Buttons {
        Buttons::NONE
    }

    fn name(&self) -> &str {
        "null"
    }
}

/// Tests / docs lab / scripted presses — set [`Self::buttons`] directly.
#[derive(Debug, Default, Clone, Copy)]
pub struct ManualInput {
    pub buttons: Buttons,
}

impl ManualInput {
    pub fn new(buttons: Buttons) -> Self {
        Self { buttons }
    }

    pub fn press(&mut self, mask: Buttons) {
        self.buttons = self.buttons.union(mask);
    }

    pub fn release(&mut self, mask: Buttons) {
        self.buttons = self.buttons.difference(mask);
    }

    pub fn clear(&mut self) {
        self.buttons = Buttons::NONE;
    }
}

impl InputSource for ManualInput {
    fn poll(&mut self) -> Buttons {
        self.buttons
    }

    fn name(&self) -> &str {
        "manual"
    }
}

/// Placeholder for a future browser / WebRTC data-channel source.
///
/// Holds the last packet so a web handler can `push` without blocking the emu.
#[derive(Debug, Default)]
pub struct WebInputStub {
    pub last: Buttons,
    pub packets: u64,
}

impl WebInputStub {
    pub fn push(&mut self, buttons: Buttons) {
        self.last = buttons;
        self.packets += 1;
    }
}

impl InputSource for WebInputStub {
    fn poll(&mut self) -> Buttons {
        self.last
    }

    fn name(&self) -> &str {
        "web-stub"
    }
}

/// Placeholder for a future Bluetooth / BLE gamepad source.
#[derive(Debug, Default)]
pub struct BluetoothInputStub {
    pub last: Buttons,
    pub packets: u64,
}

impl BluetoothInputStub {
    pub fn push(&mut self, buttons: Buttons) {
        self.last = buttons;
        self.packets += 1;
    }
}

impl InputSource for BluetoothInputStub {
    fn poll(&mut self) -> Buttons {
        self.last
    }

    fn name(&self) -> &str {
        "bluetooth-stub"
    }
}

/// Merge several sources with **OR** (any source pressing a button → pressed).
///
/// Useful when local keyboard **and** a web viewer both feed SoftGBC.
pub struct FanInInput {
    sources: Vec<Box<dyn InputSource>>,
}

impl FanInInput {
    pub fn new(sources: Vec<Box<dyn InputSource>>) -> Self {
        Self { sources }
    }

    pub fn push(&mut self, source: Box<dyn InputSource>) {
        self.sources.push(source);
    }
}

impl InputSource for FanInInput {
    fn poll(&mut self) -> Buttons {
        let mut out = Buttons::NONE;
        for s in &mut self.sources {
            out = out.union(s.poll());
        }
        out
    }

    fn name(&self) -> &str {
        "fan-in"
    }
}

/// Non-blocking channel: web/Bluetooth worker threads `send`, emu `poll`s.
pub struct ChannelInput {
    rx: Receiver<Buttons>,
    last: Buttons,
}

impl ChannelInput {
    pub fn pair() -> (Sender<Buttons>, Self) {
        let (tx, rx) = mpsc::channel();
        (
            tx,
            Self {
                rx,
                last: Buttons::NONE,
            },
        )
    }
}

impl InputSource for ChannelInput {
    fn poll(&mut self) -> Buttons {
        loop {
            match self.rx.try_recv() {
                Ok(b) => self.last = b,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        self.last
    }

    fn name(&self) -> &str {
        "channel"
    }
}

/// Apply one poll into a SoftGBC joypad (bus must own [`peripherals::Joypad`]).
pub fn apply_input(joypad: &mut peripherals::Joypad, source: &mut dyn InputSource) {
    joypad.set_pressed(source.poll());
}

#[cfg(test)]
mod tests {
    use super::*;
    use peripherals::{IoDevice, Joypad};

    #[test]
    fn manual_input_reaches_joypad_a_bit() {
        let mut src = ManualInput::new(Buttons::A);
        let mut pad = Joypad::new();
        apply_input(&mut pad, &mut src);
        pad.write(0xFF00, 0x10); // select buttons (bit5=0), d-pad off (bit4=1)
        let v = pad.read(0xFF00);
        assert_eq!(v & 0x01, 0, "A pressed → bit0 low, got {v:#04X}");
        assert_eq!(v & 0x0E, 0x0E, "other button bits released");
    }

    #[test]
    fn fan_in_ors_sources() {
        let mut fan = FanInInput::new(vec![
            Box::new(ManualInput::new(Buttons::LEFT)),
            Box::new(ManualInput::new(Buttons::A)),
        ]);
        let b = fan.poll();
        assert!(b.contains(Buttons::LEFT));
        assert!(b.contains(Buttons::A));
        assert!(!b.contains(Buttons::RIGHT));
    }

    #[test]
    fn channel_input_keeps_last() {
        let (tx, mut src) = ChannelInput::pair();
        tx.send(Buttons::START).unwrap();
        assert!(src.poll().contains(Buttons::START));
        assert_eq!(src.name(), "channel");
    }

    #[test]
    fn stubs_have_stable_names() {
        assert_eq!(NullInput.name(), "null");
        assert_eq!(WebInputStub::default().name(), "web-stub");
        assert_eq!(BluetoothInputStub::default().name(), "bluetooth-stub");
    }
}
