//! SoftGBC viewer — local window, HTTP preview page, and/or broadcast.
//!
//! ```text
//! cargo run -p softgbc-host --bin softgbc-view -- --demo --http 8765
//! # open http://127.0.0.1:8765/
//! ```

use dmg_ppu::{SCREEN_H, SCREEN_W};
use softgbc_host::{
    forward_to_remote, run_loop, DemoPpuController, FanOutMode, FanOutSink, FrameSink, HostWindow,
    HostWindowConfig, HttpPreviewServer, HubSink, RemoteEgress, SoftGbcController,
    WebRtcEgressStub, WindowPalette,
};
use std::env;
use std::thread;
use std::time::Duration;

fn usage() -> ! {
    eprintln!(
        "Usage:
  softgbc-view --demo [--http PORT] [--window] [--broadcast] [--frames N]
  softgbc-view --rom <path.gb> [--http PORT] [--window] [--broadcast] [--frames N]

  --http PORT   serve browser preview at http://127.0.0.1:PORT/ (0 = ephemeral)
  --window      also open a local minifb window (needs DISPLAY)
  --broadcast   also push to channel + WebRTC stub thread
  --frames N    stop after N frames (default: forever for window/http)"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }

    let mut demo = false;
    let mut rom: Option<String> = None;
    let mut http_port: Option<u16> = None;
    let mut window = false;
    let mut broadcast = false;
    let mut frames: Option<u64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--demo" => demo = true,
            "--window" => window = true,
            "--broadcast" => broadcast = true,
            "--http" => {
                i += 1;
                http_port = args.get(i).and_then(|s| s.parse().ok());
                if http_port.is_none() {
                    usage();
                }
            }
            "--rom" => {
                i += 1;
                rom = args.get(i).cloned();
                if rom.is_none() {
                    usage();
                }
            }
            "--frames" => {
                i += 1;
                frames = args.get(i).and_then(|s| s.parse().ok());
                if frames.is_none() {
                    usage();
                }
            }
            "-h" | "--help" => usage(),
            other => {
                eprintln!("unknown arg: {other}");
                usage();
            }
        }
        i += 1;
    }

    if demo == rom.is_some() {
        eprintln!("specify exactly one of --demo or --rom");
        usage();
    }
    if http_port.is_none() && !window && !broadcast {
        // Default: HTTP preview on 8765 so remote sessions can open a page.
        http_port = Some(8765);
    }

    let mut sinks: Vec<Box<dyn FrameSink>> = Vec::new();
    let mut remote_join = None;

    if let Some(port) = http_port {
        let srv = HttpPreviewServer::spawn(port).unwrap_or_else(|e| {
            eprintln!("http preview: {e}");
            std::process::exit(1);
        });
        eprintln!("SoftGBC HTTP preview: {}", srv.url());
        sinks.push(Box::new(HubSink::new(srv.hub())));
        // Keep server thread alive for the process lifetime.
        std::mem::forget(srv);
    }

    if window {
        let win = HostWindow::open(
            HostWindowConfig {
                title: if demo {
                    "SoftGBC demo".into()
                } else {
                    "SoftGBC".into()
                },
                scale: 4,
                palette: WindowPalette::DmgGreen,
            },
            SCREEN_W,
            SCREEN_H,
        )
        .unwrap_or_else(|e| {
            eprintln!("window: {e}");
            std::process::exit(1);
        });
        sinks.push(Box::new(win));
    }

    if broadcast {
        let (ch, rx) = softgbc_host::ChannelSink::new(2);
        sinks.push(Box::new(ch));
        remote_join = Some(thread::spawn(move || {
            let mut egress = WebRtcEgressStub::default();
            let _ = forward_to_remote(rx, &mut egress);
            eprintln!("[{}] received {} frames", egress.name(), egress.frames);
        }));
    }

    if sinks.is_empty() {
        eprintln!("no sinks configured");
        usage();
    }

    let mut sink: Box<dyn FrameSink> = if sinks.len() == 1 {
        sinks.pop().unwrap()
    } else {
        let mode = if window {
            FanOutMode::StopWhenPrimaryStops
        } else {
            FanOutMode::ContinueWhileAny
        };
        // Put window first if present so primary-stop works.
        Box::new(FanOutSink::new(sinks, mode))
    };

    // Give HTTP a moment to bind before we hammer frames (browser can connect).
    thread::sleep(Duration::from_millis(100));

    let max_frames = frames.or(if window || http_port.is_some() {
        None
    } else {
        Some(60)
    });

    let mut run = |c: &mut dyn softgbc_host::PpuController| run_loop(c, sink.as_mut(), max_frames);

    let result = if demo {
        let mut c = DemoPpuController::solid_stripe();
        run(&mut c)
    } else {
        let path = rom.unwrap();
        let mut c = SoftGbcController::from_rom(&path).unwrap_or_else(|e| {
            eprintln!("load ROM: {e}");
            std::process::exit(1);
        });
        run(&mut c)
    };

    if let Err(e) = result {
        eprintln!("viewer: {e}");
        std::process::exit(1);
    }

    drop(sink);
    if let Some(t) = remote_join {
        let _ = t.join();
    }
}
