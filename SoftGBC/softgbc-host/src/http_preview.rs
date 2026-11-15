//! HTTP preview for browsers / remote sessions.
//!
//! Serves a small HTML page and `/frame.png` from the latest shade frame.
//! PPU-agnostic: only consumes [`OwnedShadeFrame`] via [`FrameSink`].

use std::io::Cursor;
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use png::{ColorType, Encoder};
use softgbc_utils::{shade_to_argb, OwnedShadeFrame, ShadeFrame, GREY_ARGB};
use tiny_http::{Header, Response, Server, StatusCode};

use crate::{FrameSink, PresentError};

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <title>SoftGBC preview</title>
  <style>
    :root { color-scheme: dark; }
    body { margin: 0; min-height: 100vh; display: grid; place-items: center;
           background: #12141a; font-family: ui-sans-serif, system-ui, sans-serif; color: #e8eaef; }
    main { text-align: center; }
    h1 { font-weight: 600; font-size: 1.1rem; letter-spacing: 0.04em; margin: 0 0 1rem; opacity: 0.85; }
    img { image-rendering: pixelated; image-rendering: crisp-edges;
          width: min(90vw, 640px); background: #000; border: 1px solid #2a2f3a; border-radius: 4px; }
    p { margin-top: 0.75rem; font-size: 0.85rem; opacity: 0.55; }
  </style>
</head>
<body>
  <main>
    <h1>SoftGBC</h1>
    <img id="lcd" width="160" height="144" alt="Game Boy LCD"/>
    <p>Live PNG · poll /frame.png</p>
  </main>
  <script>
    const img = document.getElementById('lcd');
    async function tick() {
      try {
        const r = await fetch('/frame.png?t=' + Date.now(), { cache: 'no-store' });
        if (r.ok) {
          const blob = await r.blob();
          const url = URL.createObjectURL(blob);
          const prev = img.dataset.url;
          img.src = url;
          img.dataset.url = url;
          if (prev) URL.revokeObjectURL(prev);
        }
      } catch (_) {}
      requestAnimationFrame(() => setTimeout(tick, 33));
    }
    tick();
  </script>
</body>
</html>
"#;

/// Shared latest frame for the HTTP server.
#[derive(Clone, Default)]
pub struct FrameHub {
    inner: Arc<Mutex<Option<OwnedShadeFrame>>>,
}

impl FrameHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish(&self, frame: OwnedShadeFrame) {
        *self.inner.lock().expect("frame hub lock") = Some(frame);
    }

    pub fn latest(&self) -> Option<OwnedShadeFrame> {
        self.inner.lock().expect("frame hub lock").clone()
    }
}

/// [`FrameSink`] that only updates [`FrameHub`] (for HTTP / WebRTC workers).
pub struct HubSink {
    hub: FrameHub,
}

impl HubSink {
    pub fn new(hub: FrameHub) -> Self {
        Self { hub }
    }
}

impl FrameSink for HubSink {
    fn present(&mut self, frame: ShadeFrame<'_>) -> Result<bool, PresentError> {
        self.hub.publish(frame.to_owned());
        Ok(true)
    }
}

/// Expand shades to packed RGB8 (3 bytes/pixel) for canvas blit / raw HTTP.
pub fn encode_frame_rgb(frame: &OwnedShadeFrame) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(frame.shades.len() * 3);
    for &s in &frame.shades {
        let argb = shade_to_argb(s, &GREY_ARGB);
        rgb.push(((argb >> 16) & 0xFF) as u8);
        rgb.push(((argb >> 8) & 0xFF) as u8);
        rgb.push((argb & 0xFF) as u8);
    }
    rgb
}

/// Encode shades as RGB8 PNG bytes.
pub fn encode_frame_png(frame: &OwnedShadeFrame) -> Result<Vec<u8>, PresentError> {
    let rgb = encode_frame_rgb(frame);
    let mut out = Cursor::new(Vec::new());
    {
        let mut enc = Encoder::new(&mut out, frame.width as u32, frame.height as u32);
        enc.set_color(ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc
            .write_header()
            .map_err(|e| PresentError::Backend(e.to_string()))?;
        writer
            .write_image_data(&rgb)
            .map_err(|e| PresentError::Backend(e.to_string()))?;
    }
    Ok(out.into_inner())
}

/// Bind HTTP preview and serve until the process drops the handle / server errors.
pub struct HttpPreviewServer {
    pub addr: SocketAddr,
    _join: JoinHandle<()>,
    hub: FrameHub,
}

impl HttpPreviewServer {
    /// Bind `127.0.0.1:port` (`port = 0` → ephemeral). Returns bound address.
    pub fn spawn(port: u16) -> Result<Self, PresentError> {
        let listener = TcpListener::bind(("127.0.0.1", port))
            .map_err(|e| PresentError::Backend(e.to_string()))?;
        let addr = listener
            .local_addr()
            .map_err(|e| PresentError::Backend(e.to_string()))?;
        let server = Server::from_listener(listener, None)
            .map_err(|e| PresentError::Backend(e.to_string()))?;
        let hub = FrameHub::new();
        let hub_srv = hub.clone();
        let join = thread::spawn(move || serve_loop(server, hub_srv));
        Ok(Self {
            addr,
            _join: join,
            hub,
        })
    }

    pub fn hub(&self) -> FrameHub {
        self.hub.clone()
    }

    pub fn url(&self) -> String {
        format!("http://{}/", self.addr)
    }
}

fn serve_loop(server: Server, hub: FrameHub) {
    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let path = url.split('?').next().unwrap_or("/");
        let response = match path {
            "/" | "/index.html" => Response::from_string(INDEX_HTML)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap()),
            "/frame.png" => match hub.latest() {
                Some(frame) => match encode_frame_png(&frame) {
                    Ok(bytes) => Response::from_data(bytes).with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"image/png"[..]).unwrap(),
                    ),
                    Err(_) => Response::from_string("encode error")
                        .with_status_code(StatusCode(500)),
                },
                None => Response::from_string("no frame yet").with_status_code(StatusCode(503)),
            },
            "/health" => Response::from_string("ok"),
            _ => Response::from_string("not found").with_status_code(StatusCode(404)),
        };
        let _ = request.respond(response);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use softgbc_utils::ShadeFrame;

    #[test]
    fn encode_frame_rgb_is_3_bytes_per_pixel() {
        let frame = OwnedShadeFrame {
            width: 160,
            height: 144,
            shades: vec![0u8; 160 * 144],
        };
        let rgb = encode_frame_rgb(&frame);
        assert_eq!(rgb.len(), 160 * 144 * 3);
        // shade 0 → white in GREY_ARGB
        assert_eq!(&rgb[0..3], &[0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn encode_png_roundtrip_size() {
        let frame = OwnedShadeFrame {
            width: 2,
            height: 2,
            shades: vec![0, 1, 2, 3],
        };
        let png = encode_frame_png(&frame).unwrap();
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
        assert!(png.len() > 32);
    }

    #[test]
    fn hub_sink_publishes() {
        let hub = FrameHub::new();
        let mut sink = HubSink::new(hub.clone());
        let shades = [3u8; 4];
        sink.present(ShadeFrame::new(2, 2, &shades)).unwrap();
        let got = hub.latest().unwrap();
        assert_eq!(got.shades, shades);
    }

    #[test]
    fn http_server_serves_health_and_frame() {
        let srv = HttpPreviewServer::spawn(0).unwrap();
        let url = format!("http://{}/health", srv.addr);
        let body = std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = body;
        let resp = tiny_http_get(&url);
        assert!(resp.contains("ok"), "{resp}");

        srv.hub().publish(OwnedShadeFrame {
            width: 2,
            height: 2,
            shades: vec![0, 1, 2, 3],
        });
        let frame_url = format!("http://{}/frame.png", srv.addr);
        let bytes = tiny_http_get_bytes(&frame_url);
        assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    fn tiny_http_get(url: &str) -> String {
        let resp = ureq_get(url);
        String::from_utf8_lossy(&resp).into_owned()
    }

    fn tiny_http_get_bytes(url: &str) -> Vec<u8> {
        ureq_get(url)
    }

    fn ureq_get(url: &str) -> Vec<u8> {
        // Avoid extra deps: use std::net raw HTTP/1.0 GET.
        use std::io::{Read, Write};
        use std::net::TcpStream;
        let url = url.strip_prefix("http://").unwrap();
        let (host, path) = url.split_once('/').unwrap_or((url, ""));
        let path = format!("/{path}");
        let mut stream = TcpStream::connect(host).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf);
        let idx = text.find("\r\n\r\n").unwrap();
        buf[idx + 4..].to_vec()
    }
}
