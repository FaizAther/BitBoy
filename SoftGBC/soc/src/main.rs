use soc::{run_rom, BlarggResult};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(rom) = args.next() else {
        eprintln!("usage: softgbc <rom.gb> [max_steps]");
        return ExitCode::from(2);
    };
    let max_steps: u64 = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000_000);

    match run_rom(&rom, max_steps) {
        Ok((result, serial)) => {
            print!("{serial}");
            if !serial.ends_with('\n') && !serial.is_empty() {
                println!();
            }
            match result {
                BlarggResult::Passed => {
                    eprintln!("[softgbc] PASSED");
                    ExitCode::SUCCESS
                }
                BlarggResult::Failed => {
                    eprintln!("[softgbc] FAILED");
                    ExitCode::from(1)
                }
                BlarggResult::Timeout => {
                    eprintln!("[softgbc] TIMEOUT after {max_steps} steps");
                    ExitCode::from(3)
                }
            }
        }
        Err(e) => {
            eprintln!("[softgbc] load error: {e}");
            ExitCode::from(2)
        }
    }
}
