//! Pluggable present targets (local window, broadcast, null, …).

use softgbc_utils::{OwnedShadeFrame, ShadeFrame};

/// Error from a host sink (window closed mid-present, etc.).
#[derive(Debug)]
pub enum PresentError {
    /// User closed the window / sink is gone.
    Closed,
    /// Backend failure.
    Backend(String),
}

impl std::fmt::Display for PresentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresentError::Closed => write!(f, "display closed"),
            PresentError::Backend(s) => write!(f, "display backend: {s}"),
        }
    }
}

impl std::error::Error for PresentError {}

/// Host visual / remote sink — **not** part of the PPU.
///
/// Implementations: local X11 window, fan-out, channel→WebRTC/webapp, null.
pub trait FrameSink {
    /// Present one shade frame.
    ///
    /// `Ok(false)` = stop the run loop (e.g. local window closed).
    /// `Ok(true)` = continue.
    fn present(&mut self, frame: ShadeFrame<'_>) -> Result<bool, PresentError>;
}

/// Test / headless sink: records present count, never opens a window.
#[derive(Debug, Default)]
pub struct NullSink {
    pub presents: u64,
    pub last_crc_hint: u32,
}

impl FrameSink for NullSink {
    fn present(&mut self, frame: ShadeFrame<'_>) -> Result<bool, PresentError> {
        self.presents += 1;
        self.last_crc_hint = frame
            .shades
            .iter()
            .fold(0u32, |a, &s| a.wrapping_mul(16777619) ^ u32::from(s));
        Ok(true)
    }
}

/// Fan-out: same frame to several sinks (e.g. server window **and** WebRTC).
///
/// Continues while **any** sink wants to keep going; errors from one sink are
/// logged via `Ok(false)` only if *all* sinks stop (or one returns `Closed` for
/// a mandatory sink — see [`FanOutMode`]).
pub struct FanOutSink {
    sinks: Vec<Box<dyn FrameSink>>,
    mode: FanOutMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FanOutMode {
    /// Stop when every sink returns false / is removed.
    ContinueWhileAny,
    /// Stop when the first sink returns false (typical: local window is primary).
    #[default]
    StopWhenPrimaryStops,
}

impl FanOutSink {
    pub fn new(sinks: Vec<Box<dyn FrameSink>>, mode: FanOutMode) -> Self {
        Self { sinks, mode }
    }

    pub fn push(&mut self, sink: Box<dyn FrameSink>) {
        self.sinks.push(sink);
    }
}

impl FrameSink for FanOutSink {
    fn present(&mut self, frame: ShadeFrame<'_>) -> Result<bool, PresentError> {
        if self.sinks.is_empty() {
            return Ok(true);
        }
        match self.mode {
            FanOutMode::StopWhenPrimaryStops => {
                let (first, rest) = self.sinks.split_first_mut().unwrap();
                let keep = first.present(frame)?;
                for s in rest {
                    let _ = s.present(frame)?;
                }
                Ok(keep)
            }
            FanOutMode::ContinueWhileAny => {
                let mut any = false;
                let mut err: Option<PresentError> = None;
                for s in &mut self.sinks {
                    match s.present(frame) {
                        Ok(true) => any = true,
                        Ok(false) => {}
                        Err(e) => err = Some(e),
                    }
                }
                if any {
                    Ok(true)
                } else if let Some(e) = err {
                    Err(e)
                } else {
                    Ok(false)
                }
            }
        }
    }
}

/// Push owned frames onto a channel for WebRTC / web / another process.
///
/// Never blocks the emulator for long: if the consumer is slow, frames are
/// dropped (`try_send`) so the PPU loop stays realtime-ish.
pub struct ChannelSink {
    tx: std::sync::mpsc::SyncSender<OwnedShadeFrame>,
}

impl ChannelSink {
    /// Bounded queue (`cap` frames). Prefer `cap = 1` or `2` for live preview.
    pub fn new(cap: usize) -> (Self, std::sync::mpsc::Receiver<OwnedShadeFrame>) {
        let (tx, rx) = std::sync::mpsc::sync_channel(cap.max(1));
        (Self { tx }, rx)
    }
}

impl FrameSink for ChannelSink {
    fn present(&mut self, frame: ShadeFrame<'_>) -> Result<bool, PresentError> {
        let owned = frame.to_owned();
        match self.tx.try_send(owned) {
            Ok(()) => Ok(true),
            Err(std::sync::mpsc::TrySendError::Full(_)) => Ok(true), // drop frame
            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use softgbc_utils::ShadeFrame;

    #[test]
    fn null_sink_records_presents() {
        let shades = [0u8, 1, 2, 3];
        let mut sink = NullSink::default();
        assert!(sink.present(ShadeFrame::new(2, 2, &shades)).unwrap());
        assert_eq!(sink.presents, 1);
    }

    #[test]
    fn fan_out_hits_all_sinks() {
        let shades = [3u8; 4];
        let mut a = NullSink::default();
        let mut b = NullSink::default();
        // Use FanOut with boxed nulls
        let mut fan = FanOutSink::new(
            vec![
                Box::new(NullSink::default()),
                Box::new(NullSink::default()),
            ],
            FanOutMode::ContinueWhileAny,
        );
        assert!(fan.present(ShadeFrame::new(2, 2, &shades)).unwrap());
        let _ = (&mut a, &mut b);
    }

    #[test]
    fn channel_sink_delivers_owned_frame() {
        let (mut sink, rx) = ChannelSink::new(2);
        let shades = [0u8, 3, 1, 2];
        assert!(sink.present(ShadeFrame::new(2, 2, &shades)).unwrap());
        let got = rx.try_recv().unwrap();
        assert_eq!(got.shades, shades);
    }
}
