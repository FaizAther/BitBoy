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
//! SoftGbc / Demo  (PpuController)
//!        │
//!        ▼
//!    run_driver / run_loop
//!        │
//!        ▼
//!    FrameSink(s)
//!     ├─ HostWindow      ← popup on this server (DISPLAY) or SSH -X
//!     └─ ChannelSink     ← RemoteEgress (WebRTC stub / future webapp)
//!
//! InputSource(s) ──► Joypad::set_pressed
//!   ├─ ManualInput / NullInput
//!   ├─ ChannelInput        ← web / Bluetooth worker later
//!   └─ FanInInput          ← OR several sources
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
pub use http_preview::{encode_frame_png, FrameHub, HttpPreviewServer, HubSink};
pub use input::{
    apply_input, BluetoothInputStub, ChannelInput, FanInInput, InputSource, ManualInput,
    NullInput, WebInputStub,
};
pub use peripherals::joypad::Buttons;
pub use remote::{forward_to_remote, RemoteEgress, WebRtcEgressStub};
pub use sink::{
    ChannelSink, FanOutMode, FanOutSink, FrameSink, NullSink, PresentError,
};
pub use window::{HostWindow, HostWindowConfig, WindowPalette};

use softgbc_utils::ShadeFrame;

/// Run `controller` into `sink` until the sink asks to quit or `max_frames`.
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
