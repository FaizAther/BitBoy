//! Headless SoftGBC play HTTP server for the docs Environment page.
//!
//! ```text
//! softgbc-play --port 8766 [--gdb-port 0]
//! POST /load          raw .gb/.gbc bytes
//! POST /input         {"bits":255}   # Buttons bitset
//! GET  /frame.rgb     160×144 packed RGB8 (fast path for canvas)
//! GET  /frame.png     PNG (slower; encodes on demand)
//! GET  /status        {"loaded":true,"frames":N,"title":"...","gdb_port":N}
//! GET  /debug         read-only CPU/PPU/IRQ/joypad + gdb_port snapshot
//! POST /stop          unload
//! ```
//!
//! GDB: `gdb -ex 'target remote 127.0.0.1:<gdb_port>'` then `info registers` / `stepi`.

use softgbc_core::Bus;
use softgbc_host::gdb::{run_gdb_listener, GdbMachine};
use softgbc_host::{
    apply_input, encode_frame_png, encode_frame_rgb, Buttons, ManualInput,
};
use softgbc_utils::OwnedShadeFrame;
use soc::SoftGbc;
use std::env;
use std::io::Read;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tiny_http::{Header, Method, Response, Server, StatusCode};

const SCREEN_W: usize = 160;
const SCREEN_H: usize = 144;
const FRAME_RGB_LEN: usize = SCREEN_W * SCREEN_H * 3;
/// DMG / CGB LCD ≈ 59.7 Hz.
const FRAME_PERIOD: Duration = Duration::from_nanos(16_743_646);

struct PlayState {
    machine: Arc<Mutex<GdbMachine>>,
    loaded: bool,
    buttons: Buttons,
    frame_rgb: Vec<u8>,
    frame_png: Option<Vec<u8>>,
    png_at_frame: u64,
    frames: u64,
    title: String,
    running: bool,
    gdb_port: u16,
}

fn empty_rgb() -> Vec<u8> {
    vec![0u8; FRAME_RGB_LEN]
}

fn empty_frame() -> OwnedShadeFrame {
    OwnedShadeFrame {
        width: SCREEN_W,
        height: SCREEN_H,
        shades: vec![0u8; SCREEN_W * SCREEN_H],
    }
}

fn title_from_rom(data: &[u8]) -> String {
    if data.len() < 0x144 {
        return "ROM".into();
    }
    let raw = &data[0x134..0x143];
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end])
        .trim()
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .collect::<String>()
}

fn json_escape(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn json_bits(body: &str) -> Option<u8> {
    let t = body.trim();
    let rest = t.strip_prefix("{\"bits\":")?;
    let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}

/// Build /debug JSON. Caller must **not** hold `PlayState` while locking `machine`
/// (avoids deadlock with the emu thread: state→machine vs machine→state).
fn debug_snapshot(
    joy: u8,
    frames: u64,
    title: &str,
    running: bool,
    gdb_port: u16,
    loaded: bool,
    machine: &Arc<Mutex<GdbMachine>>,
) -> String {
    let run = format!(
        "\"frames\":{},\"title\":{},\"running\":{},\"gdb_port\":{}",
        frames,
        json_escape(title),
        running,
        gdb_port
    );
    if !loaded {
        return format!(
            "{{\"loaded\":false,\"cpu\":null,\"ppu\":null,\"irq\":null,\"joypad\":{{\"bits\":{joy}}},\"run\":{{{run}}}}}"
        );
    }
    let Ok(m) = machine.lock() else {
        return format!(
            "{{\"loaded\":true,\"error\":\"lock\",\"joypad\":{{\"bits\":{joy}}},\"run\":{{{run}}}}}"
        );
    };
    let cpu = &m.gb.cpu;
    let ppu = &m.gb.bus.ppu;
    let if_reg = m.gb.bus.read8(0xFF0F).unwrap_or(0);
    let ie_reg = m.gb.bus.read8(0xFFFF).unwrap_or(0);
    let stat = m.gb.bus.read8(0xFF41).unwrap_or(ppu.STAT);
    format!(
        "{{\
          \"loaded\":true,\
          \"cpu\":{{\"pc\":{},\"sp\":{},\"af\":{},\"bc\":{},\"de\":{},\"hl\":{},\"ime\":{},\"halted\":{}}},\
          \"ppu\":{{\"ly\":{},\"lcdc\":{},\"stat\":{},\"mode\":{},\"bgp\":{}}},\
          \"irq\":{{\"if\":{},\"ie\":{}}},\
          \"joypad\":{{\"bits\":{joy}}},\
          \"gdb\":{{\"port\":{},\"halted\":{},\"breakpoints\":{}}},\
          \"run\":{{{run}}}\
        }}",
        cpu.pc,
        cpu.sp,
        cpu.af(),
        cpu.bc(),
        cpu.de(),
        cpu.hl(),
        cpu.ime,
        cpu.halted,
        ppu.ly,
        ppu.LCDC,
        stat,
        ppu.mode_now(),
        ppu.BGP,
        if_reg,
        ie_reg,
        gdb_port,
        m.gdb_halted,
        m.breakpoints.len(),
    )
}

fn emu_thread(state: Arc<Mutex<PlayState>>) {
    loop {
        let start = Instant::now();
        let mut did_frame = false;
        {
            let guard = state.lock().expect("play lock");
            if guard.running && guard.loaded {
                let buttons = guard.buttons;
                let machine = guard.machine.clone();
                drop(guard);
                if let Ok(mut m) = machine.lock() {
                    if !m.gdb_halted {
                        let mut input = ManualInput::new(buttons);
                        apply_input(&mut m.gb.bus.joypad, &mut input);
                        m.advance_frame();
                        let owned = m.gb.bus.ppu.shade_frame().to_owned();
                        let rgb = encode_frame_rgb(&owned);
                        // Drop machine lock before taking PlayState (lock order).
                        drop(m);
                        if let Ok(mut guard) = state.lock() {
                            guard.frame_rgb = rgb;
                            guard.frame_png = None;
                            guard.frames = guard.frames.wrapping_add(1);
                            did_frame = true;
                        }
                    }
                }
            }
        }
        if did_frame {
            let elapsed = start.elapsed();
            if elapsed < FRAME_PERIOD {
                thread::sleep(FRAME_PERIOD - elapsed);
            }
        } else {
            thread::sleep(Duration::from_millis(20));
        }
    }
}

fn main() {
    let mut port: u16 = 8766;
    let mut gdb_port_req: u16 = 0; // 0 = ephemeral
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--port" => {
                port = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(8766);
            }
            "--gdb-port" => {
                gdb_port_req = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
            "-h" | "--help" => {
                eprintln!("usage: softgbc-play [--port N] [--gdb-port N]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown arg: {other}");
                std::process::exit(2);
            }
        }
    }

    let machine = Arc::new(Mutex::new(GdbMachine::new(SoftGbc::new())));
    let gdb_port = run_gdb_listener(machine.clone(), gdb_port_req).unwrap_or_else(|e| {
        eprintln!("gdb listener failed: {e}");
        0
    });

    let state = Arc::new(Mutex::new(PlayState {
        machine: machine.clone(),
        loaded: false,
        buttons: Buttons::NONE,
        frame_rgb: empty_rgb(),
        frame_png: None,
        png_at_frame: u64::MAX,
        frames: 0,
        title: String::new(),
        running: false,
        gdb_port,
    }));

    let emu = state.clone();
    thread::spawn(move || emu_thread(emu));

    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("bind: {e}");
        std::process::exit(1);
    });
    let addr = listener.local_addr().unwrap();
    eprintln!("softgbc-play http://{addr}/  gdb://127.0.0.1:{gdb_port}/");
    let server = Server::from_listener(listener, None).unwrap();

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let path = url.split('?').next().unwrap_or("/");
        let method = request.method().clone();

        let response = match (method, path) {
            (Method::Get, "/status") => {
                let g = state.lock().unwrap();
                let body = format!(
                    "{{\"loaded\":{},\"running\":{},\"frames\":{},\"title\":{},\"gdb_port\":{}}}",
                    g.loaded,
                    g.running,
                    g.frames,
                    json_escape(&g.title),
                    g.gdb_port
                );
                Response::from_string(body).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                )
            }
            (Method::Get, "/debug") => {
                let (joy, frames, title, running, gdb_port, loaded, machine) = {
                    let g = state.lock().unwrap();
                    (
                        g.buttons.bits(),
                        g.frames,
                        g.title.clone(),
                        g.running,
                        g.gdb_port,
                        g.loaded,
                        g.machine.clone(),
                    )
                };
                let body = debug_snapshot(
                    joy, frames, &title, running, gdb_port, loaded, &machine,
                );
                Response::from_string(body).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                )
            }
            (Method::Get, "/frame.rgb") => {
                let rgb = state.lock().unwrap().frame_rgb.clone();
                Response::from_data(rgb).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..])
                        .unwrap(),
                )
            }
            (Method::Get, "/frame.png") => {
                let mut g = state.lock().unwrap();
                if g.frame_png.is_none() || g.png_at_frame != g.frames {
                    let png = rgb_to_png(&g.frame_rgb, SCREEN_W, SCREEN_H).unwrap_or_else(|_| {
                        encode_frame_png(&empty_frame()).unwrap_or_default()
                    });
                    g.frame_png = Some(png);
                    g.png_at_frame = g.frames;
                }
                let png = g.frame_png.clone().unwrap_or_default();
                drop(g);
                Response::from_data(png).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"image/png"[..]).unwrap(),
                )
            }
            (Method::Post, "/load") => {
                let rom = read_body(&mut request);
                if rom.len() < 0x150 {
                    Response::from_string("{\"error\":\"rom too small\"}")
                        .with_status_code(StatusCode(400))
                } else {
                    let title = title_from_rom(&rom);
                    let gb = SoftGbc::from_rom_bytes(&rom);
                    let (machine, gdb_port) = {
                        let g = state.lock().unwrap();
                        (g.machine.clone(), g.gdb_port)
                    };
                    if let Ok(mut m) = machine.lock() {
                        m.gb = gb;
                        m.breakpoints.clear();
                        m.gdb_halted = false;
                    }
                    {
                        let mut g = state.lock().unwrap();
                        g.loaded = true;
                        g.title = title.clone();
                        g.frames = 0;
                        g.buttons = Buttons::NONE;
                        g.running = true;
                        g.frame_rgb = empty_rgb();
                        g.frame_png = None;
                        g.png_at_frame = u64::MAX;
                    }
                    Response::from_string(format!(
                        "{{\"ok\":true,\"title\":{},\"bytes\":{},\"gdb_port\":{}}}",
                        json_escape(&title),
                        rom.len(),
                        gdb_port
                    ))
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                            .unwrap(),
                    )
                }
            }
            (Method::Post, "/input") => {
                let body = String::from_utf8_lossy(&read_body(&mut request)).into_owned();
                if let Some(bits) = json_bits(&body) {
                    let mut g = state.lock().unwrap();
                    g.buttons = Buttons::from_bits(bits);
                    Response::from_string("{\"ok\":true}").with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                            .unwrap(),
                    )
                } else {
                    Response::from_string("{\"error\":\"bad json\"}")
                        .with_status_code(StatusCode(400))
                }
            }
            (Method::Post, "/stop") => {
                let machine = {
                    let mut g = state.lock().unwrap();
                    g.running = false;
                    g.loaded = false;
                    g.buttons = Buttons::NONE;
                    g.frame_rgb = empty_rgb();
                    g.frame_png = None;
                    g.png_at_frame = u64::MAX;
                    g.title.clear();
                    g.machine.clone()
                };
                if let Ok(mut m) = machine.lock() {
                    m.gb = SoftGbc::new();
                    m.breakpoints.clear();
                    m.gdb_halted = false;
                }
                Response::from_string("{\"ok\":true}").with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                )
            }
            (Method::Get, "/") => Response::from_string(format!(
                "softgbc-play: POST /load, GET /frame.rgb|/frame.png|/status|/debug, POST /input|/stop\n\
                 gdb: target remote 127.0.0.1:{gdb_port}\n"
            )),
            _ => Response::from_string("not found").with_status_code(StatusCode(404)),
        };

        let _ = request.respond(response);
    }
}

fn read_body(request: &mut tiny_http::Request) -> Vec<u8> {
    let mut body = Vec::new();
    let len = request.body_length().unwrap_or(0);
    let reader = request.as_reader();
    if len > 0 {
        let mut take = reader.take(len as u64);
        let _ = take.read_to_end(&mut body);
    } else {
        let _ = reader.take(8 * 1024 * 1024).read_to_end(&mut body);
    }
    body
}

fn rgb_to_png(rgb: &[u8], w: usize, h: usize) -> Result<Vec<u8>, String> {
    use png::{ColorType, Encoder};
    use std::io::Cursor;
    if rgb.len() < w * h * 3 {
        return Err("short rgb".into());
    }
    let mut out = Cursor::new(Vec::new());
    {
        let mut enc = Encoder::new(&mut out, w as u32, h as u32);
        enc.set_color(ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(&rgb[..w * h * 3])
            .map_err(|e| e.to_string())?;
    }
    Ok(out.into_inner())
}
