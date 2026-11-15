//! Environment-agnostic display driver.
//!
//! Wire any [`crate::PpuController`] to one or more [`crate::FrameSink`]s:
//! local window (this server / SSH `-X`), channel→WebRTC, null, …
//!
//! ```text
//!                    ┌─ HostWindow        (GUI on server or remote X)
//! PpuController ──► FanOutSink
//!                    └─ ChannelSink ──► RemoteEgress (WebRTC / webapp)
//! ```

use crate::{
    run_loop, ChannelSink, FanOutMode, FanOutSink, FrameSink, HostWindow, HostWindowConfig,
    PresentError, PpuController,
};
use softgbc_utils::OwnedShadeFrame;

/// Where frames should go for this process.
#[derive(Debug, Clone, Default)]
pub struct DriverConfig {
    /// Open a local minifb window (needs `DISPLAY` / Wayland).
    pub local_window: bool,
    /// Title / scale when `local_window` is set.
    pub window: HostWindowConfig,
    /// If set, also push owned frames on a bounded channel (for WebRTC/web).
    pub broadcast_capacity: Option<usize>,
    /// Max frames then exit (`None` = until sinks stop).
    pub max_frames: Option<u64>,
    pub fan_out_mode: FanOutMode,
}

impl DriverConfig {
    pub fn local_only() -> Self {
        Self {
            local_window: true,
            broadcast_capacity: None,
            fan_out_mode: FanOutMode::StopWhenPrimaryStops,
            ..Default::default()
        }
    }

    pub fn broadcast_only(cap: usize) -> Self {
        Self {
            local_window: false,
            broadcast_capacity: Some(cap),
            fan_out_mode: FanOutMode::ContinueWhileAny,
            ..Default::default()
        }
    }

    /// Server GUI **and** remote subscribers (WebRTC later).
    pub fn local_and_broadcast(cap: usize) -> Self {
        Self {
            local_window: true,
            broadcast_capacity: Some(cap),
            fan_out_mode: FanOutMode::StopWhenPrimaryStops,
            ..Default::default()
        }
    }
}

/// Built sinks + optional broadcast receiver for a remote egress thread.
pub struct DriverHandles {
    pub sink: Box<dyn FrameSink>,
    pub broadcast_rx: Option<std::sync::mpsc::Receiver<OwnedShadeFrame>>,
}

impl DriverHandles {
    pub fn build(cfg: &DriverConfig, frame_w: usize, frame_h: usize) -> Result<Self, PresentError> {
        let mut sinks: Vec<Box<dyn FrameSink>> = Vec::new();
        let mut broadcast_rx = None;

        if cfg.local_window {
            let win = HostWindow::open(cfg.window.clone(), frame_w, frame_h)?;
            sinks.push(Box::new(win));
        }

        if let Some(cap) = cfg.broadcast_capacity {
            let (ch, rx) = ChannelSink::new(cap);
            sinks.push(Box::new(ch));
            broadcast_rx = Some(rx);
        }

        if sinks.is_empty() {
            return Err(PresentError::Backend(
                "DriverConfig: enable local_window and/or broadcast_capacity".into(),
            ));
        }

        let sink: Box<dyn FrameSink> = if sinks.len() == 1 {
            sinks.pop().unwrap()
        } else {
            Box::new(FanOutSink::new(sinks, cfg.fan_out_mode))
        };

        Ok(Self {
            sink,
            broadcast_rx,
        })
    }
}

/// Run the controller into the configured environment(s).
pub fn run_driver(
    controller: &mut dyn PpuController,
    cfg: &DriverConfig,
    frame_w: usize,
    frame_h: usize,
) -> Result<Option<std::sync::mpsc::Receiver<OwnedShadeFrame>>, PresentError> {
    let DriverHandles {
        mut sink,
        broadcast_rx,
    } = DriverHandles::build(cfg, frame_w, frame_h)?;
    run_loop(controller, sink.as_mut(), cfg.max_frames)?;
    Ok(broadcast_rx)
}
