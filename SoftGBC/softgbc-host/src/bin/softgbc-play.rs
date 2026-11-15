//! Headless SoftGBC play HTTP server for the docs Environment page.
//!
//! ```text
//! softgbc-play --port 8766
//! POST /load          raw .gb/.gbc bytes
//! POST /input         {"bits":255}   # Buttons bitset
//! GET  /frame.rgb     160×144 packed RGB8 (fast path for canvas)
//! GET  /frame.png     PNG (slower; encodes on demand)
//! GET  /status        {"loaded":true,"frames":N,"title":"..."}
//! POST /stop          unload
//! ```

use softgbc_host::{
    apply_input, encode_frame_png, encode_frame_rgb, Buttons, ManualInput, PpuController,
    SoftGbcController,
};
use softgbc_utils::OwnedShadeFrame;
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
    gb: Option<SoftGbcController>,
    buttons: Buttons,
    /// Latest LCD as packed RGB8 (canvas blit).
    frame_rgb: Vec<u8>,
    /// Lazily encoded PNG cache (invalidated when `frames` advances).
    frame_png: Option<Vec<u8>>,
    png_at_frame: u64,
    frames: u64,
    title: String,
    running: bool,
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

fn emu_thread(state: Arc<Mutex<PlayState>>) {
    loop {
        let start = Instant::now();
        let mut did_frame = false;
        {
            let mut guard = state.lock().expect("play lock");
            if guard.running {
                let buttons = guard.buttons;
                if let Some(gb) = guard.gb.as_mut() {
                    let mut input = ManualInput::new(buttons);
                    apply_input(&mut gb.gb.bus.joypad, &mut input);
                    gb.advance_frame();
                    let owned = gb.frame().to_owned();
                    guard.frame_rgb = encode_frame_rgb(&owned);
                    guard.frame_png = None;
                    guard.frames = guard.frames.wrapping_add(1);
                    did_frame = true;
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
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--port" => {
                port = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(8766);
            }
            "-h" | "--help" => {
                eprintln!("usage: softgbc-play [--port N]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown arg: {other}");
                std::process::exit(2);
            }
        }
    }

    let state = Arc::new(Mutex::new(PlayState {
        gb: None,
        buttons: Buttons::NONE,
        frame_rgb: empty_rgb(),
        frame_png: None,
        png_at_frame: u64::MAX,
        frames: 0,
        title: String::new(),
        running: false,
    }));

    let emu = state.clone();
    thread::spawn(move || emu_thread(emu));

    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("bind: {e}");
        std::process::exit(1);
    });
    let addr = listener.local_addr().unwrap();
    eprintln!("softgbc-play http://{addr}/");
    let server = Server::from_listener(listener, None).unwrap();

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let path = url.split('?').next().unwrap_or("/");
        let method = request.method().clone();

        let response = match (method, path) {
            (Method::Get, "/status") => {
                let g = state.lock().unwrap();
                let body = format!(
                    "{{\"loaded\":{},\"running\":{},\"frames\":{},\"title\":{}}}",
                    g.gb.is_some(),
                    g.running,
                    g.frames,
                    json_escape(&g.title)
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
                    let gb = SoftGbcController::from_rom_bytes(&rom);
                    let mut g = state.lock().unwrap();
                    g.gb = Some(gb);
                    g.title = title.clone();
                    g.frames = 0;
                    g.buttons = Buttons::NONE;
                    g.running = true;
                    g.frame_rgb = empty_rgb();
                    g.frame_png = None;
                    g.png_at_frame = u64::MAX;
                    Response::from_string(format!(
                        "{{\"ok\":true,\"title\":{},\"bytes\":{}}}",
                        json_escape(&title),
                        rom.len()
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
                let mut g = state.lock().unwrap();
                g.running = false;
                g.gb = None;
                g.buttons = Buttons::NONE;
                g.frame_rgb = empty_rgb();
                g.frame_png = None;
                g.png_at_frame = u64::MAX;
                g.title.clear();
                Response::from_string("{\"ok\":true}").with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                )
            }
            (Method::Get, "/") => Response::from_string(
                "softgbc-play: POST /load, GET /frame.rgb|/frame.png, POST /input, POST /stop\n",
            ),
            _ => Response::from_string("not found").with_status_code(StatusCode(404)),
        };

        let _ = request.respond(response);
    }
}

fn rgb_to_png(rgb: &[u8], w: usize, h: usize) -> Result<Vec<u8>, String> {
    use png::{Encoder, ColorType};
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
