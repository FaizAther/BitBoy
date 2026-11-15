//! Remote / multi-environment egress (WebRTC, webapp, etc.).
//!
//! The emulator loop only knows [`crate::FrameSink`]. Remote transports subscribe
//! to owned frames (see [`crate::ChannelSink`]) and push them out-of-process.
//!
//! ```text
//! PpuController ──► run_loop ──► FanOutSink
//!                                  ├─ HostWindow (X11 on this server / forwarded)
//!                                  └─ ChannelSink ──► RemoteEgress (WebRTC / WS / …)
//! ```

use softgbc_utils::OwnedShadeFrame;

use crate::PresentError;

/// Environment-agnostic remote video egress.
///
/// Implement this for WebRTC, WebSocket binary frames, gRPC, etc. The SoftGBC
/// core and PPU never see these types.
pub trait RemoteEgress: Send {
    /// Push one owned shade frame (encode / packetize as you like).
    fn push_frame(&mut self, frame: &OwnedShadeFrame) -> Result<(), PresentError>;

    /// Human-readable backend name (`"webrtc"`, `"mjpeg-http"`, …).
    fn name(&self) -> &str;
}

/// Placeholder WebRTC egress — wires the trait so a real `webrtc` crate (or
/// bridge process) can plug in later without touching the PPU.
///
/// For now it only counts frames (useful in tests / CI).
#[derive(Debug, Default)]
pub struct WebRtcEgressStub {
    pub frames: u64,
}

impl RemoteEgress for WebRtcEgressStub {
    fn push_frame(&mut self, _frame: &OwnedShadeFrame) -> Result<(), PresentError> {
        self.frames += 1;
        Ok(())
    }

    fn name(&self) -> &str {
        "webrtc-stub"
    }
}

/// Drain a [`crate::ChannelSink`] receiver into any [`RemoteEgress`] on a worker
/// thread (or call from your async runtime).
pub fn forward_to_remote(
    rx: std::sync::mpsc::Receiver<OwnedShadeFrame>,
    egress: &mut dyn RemoteEgress,
) -> Result<(), PresentError> {
    while let Ok(frame) = rx.recv() {
        egress.push_frame(&frame)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChannelSink, FrameSink};
    use softgbc_utils::ShadeFrame;

    #[test]
    fn stub_webrtc_counts_forwarded_frames() {
        let (mut sink, rx) = ChannelSink::new(4);
        let shades = [1u8; 8];
        sink.present(ShadeFrame::new(4, 2, &shades)).unwrap();
        sink.present(ShadeFrame::new(4, 2, &shades)).unwrap();
        drop(sink);

        let mut rtc = WebRtcEgressStub::default();
        forward_to_remote(rx, &mut rtc).unwrap();
        assert_eq!(rtc.frames, 2);
        assert_eq!(rtc.name(), "webrtc-stub");
    }
}
