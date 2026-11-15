//! Host display / remote video / input for SoftGBC.
//!
//! **PPU stays GUI-agnostic.** It only produces [`softgbc_utils::ShadeFrame`]
//! buffers. This crate owns:
//!
//! - [`PpuController`] — drive SoftGBC / demos from the host side
//! - [`FrameSink`] — local window, channel broadcast, null, fan-out
//! - [`RemoteEgress`] — WebRTC / webapp plug-in point
//! - [`InputSource`] — keyboard / web / Bluetooth / manual → joypad
//! - [`driver`] — wire controller → one or many environments
//!
//! ```text
//! SoftGbc (play) / Demo
//!        │
//!        ▼
//!    run_play_loop / run_loop
//!     poll InputSource → Joypad
//!     advance_frame → FrameSink
//!        ├─ HostWindow
//!        └─ ChannelSink → RemoteEgress
//! ```

mod controller;
mod driver;
mod http_preview;
mod input;
mod remote;
mod sink;
pub mod visual_tests;
mod window;

pub use controller::{DemoPpuController, PpuController, SoftGbcController};
pub use driver::{run_driver, DriverConfig, DriverHandles};
pub use http_preview::{encode_frame_png, encode_frame_rgb, FrameHub, HttpPreviewServer, HubSink};
pub use input::{
    apply_input, BluetoothInputStub, ChannelInput, FanInInput, InputSource, ManualInput,
    NullInput, WebInputStub,
};
pub use peripherals::joypad::Buttons;
pub use remote::{forward_to_remote, RemoteEgress, WebRtcEgressStub};
pub use sink::{
    ChannelSink, FanOutMode, FanOutSink, FrameSink, NullSink, PresentError,
};
pub use window::{
    keys_to_buttons, HostKey, HostWindow, HostWindowConfig, SharedKeyboardInput, WindowPalette,
};

use softgbc_utils::ShadeFrame;

/// Run `controller` into `sink` until the sink asks to quit or `max_frames`.
///
/// No joypad polling — demos / headless blit. Prefer [`run_play_loop`] for ROMs.
pub fn run_loop(
    controller: &mut dyn PpuController,
    sink: &mut dyn FrameSink,
    max_frames: Option<u64>,
) -> Result<(), PresentError> {
    let mut n = 0u64;
    loop {
        controller.advance_frame();
        let frame: ShadeFrame<'_> = controller.frame();
        if !sink.present(frame)? {
            break;
        }
        n += 1;
        if max_frames.is_some_and(|m| n >= m) {
            break;
        }
    }
    Ok(())
}

/// SoftGBC play loop: **poll input → joypad → one frame → present**.
///
/// ```text
/// each frame:
///   buttons = input.poll()
///   SoftGbc.joypad.set_pressed(buttons)
///   SoftGbc.advance_frame()   # CPU + bus ticks + render
///   sink.present(shade_frame)
/// ```
pub fn run_play_loop(
    controller: &mut SoftGbcController,
    input: &mut dyn InputSource,
    sink: &mut dyn FrameSink,
    max_frames: Option<u64>,
) -> Result<(), PresentError> {
    let mut n = 0u64;
    loop {
        controller.apply_input(input);
        controller.advance_frame();
        let frame: ShadeFrame<'_> = controller.frame();
        if !sink.present(frame)? {
            break;
        }
        n += 1;
        if max_frames.is_some_and(|m| n >= m) {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod play_loop_tests {
    use super::*;
    use peripherals::{IoDevice, Joypad};
    use soc::SoftGbc;

    #[test]
    fn play_loop_applies_input_then_presents() {
        let mut gb = SoftGbcController::new(SoftGbc::new());
        let mut input = ManualInput::new(Buttons::A | Buttons::RIGHT);
        let mut sink = NullSink::default();
        run_play_loop(&mut gb, &mut input, &mut sink, Some(2)).unwrap();
        assert_eq!(sink.presents, 2);
        assert_eq!(gb.gb.bus.joypad.pressed(), Buttons::A | Buttons::RIGHT);
        // P1 still reflects pressed set when buttons group selected.
        gb.gb.bus.joypad.write(0xFF00, 0x10);
        assert_eq!(gb.gb.bus.joypad.read(0xFF00) & 0x01, 0);
    }

    #[test]
    fn play_loop_null_input_leaves_joypad_clear() {
        let mut gb = SoftGbcController::new(SoftGbc::new());
        gb.gb.bus.joypad.set_pressed(Buttons::START);
        let mut input = NullInput;
        let mut sink = NullSink::default();
        run_play_loop(&mut gb, &mut input, &mut sink, Some(1)).unwrap();
        assert!(gb.gb.bus.joypad.pressed().is_empty());
        let _ = Joypad::handles(0xFF00);
    }
}
