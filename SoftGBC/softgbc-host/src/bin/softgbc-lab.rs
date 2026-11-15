//! CLI for the docs visual lab: list / run scenes as JSON.
//!
//! ```text
//! cargo run -p softgbc-host --bin softgbc-lab -- list
//! cargo run -p softgbc-host --bin softgbc-lab -- run solid_bg
//! ```

use softgbc_host::visual_tests::{list_tests, run_test};
use std::env;
use std::io::{self, Write};

fn b64(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let a = chunk[0] as u32;
        let b = chunk.get(1).copied().unwrap_or(0) as u32;
        let c = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (a << 16) | (b << 8) | c;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            T[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "list".into());
    match cmd.as_str() {
        "list" => {
            print!("[");
            for (i, t) in list_tests().into_iter().enumerate() {
                if i > 0 {
                    print!(",");
                }
                print!(
                    "{{\"id\":\"{}\",\"title\":\"{}\",\"description\":\"{}\",\"expected_crc\":\"0x{:08X}\",\"expected_notes\":\"{}\",\"fixture\":{}}}",
                    t.id,
                    json_escape(t.title),
                    json_escape(t.description),
                    t.expected_crc,
                    json_escape(t.expected_notes),
                    t.fixture
                        .map(|f| format!("\"{}\"", json_escape(f)))
                        .unwrap_or_else(|| "null".into())
                );
            }
            println!("]");
        }
        "run" => {
            let id = args.next().expect("usage: softgbc-lab run <id>");
            match run_test(&id) {
                Ok(r) => {
                    println!(
                        "{{\"id\":\"{}\",\"title\":\"{}\",\"passed\":{},\"expected_crc\":\"0x{:08X}\",\"actual_crc\":\"0x{:08X}\",\"expected_notes\":\"{}\",\"fixture\":{},\"actual_png_base64\":\"{}\",\"expected_png_base64\":\"{}\"}}",
                        json_escape(&r.id),
                        json_escape(&r.title),
                        r.passed,
                        r.expected_crc,
                        r.actual_crc,
                        json_escape(&r.expected_notes),
                        r.fixture
                            .as_deref()
                            .map(|f| format!("\"{}\"", json_escape(f)))
                            .unwrap_or_else(|| "null".into()),
                        b64(&r.actual_png),
                        b64(&r.expected_png),
                    );
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            let _ = writeln!(
                io::stderr(),
                "usage: softgbc-lab list | softgbc-lab run <id>"
            );
            std::process::exit(2);
        }
    }
}
