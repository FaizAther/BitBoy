//! Generic headless SoftGBC runner — any `.gb`; serial text is printed as-is.

use soc::{RunStatus, SoftGbc};
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

    let mut gb = match SoftGbc::from_rom_file(&rom) {
        Ok(gb) => gb,
        Err(e) => {
            eprintln!("[softgbc] load error: {e}");
            return ExitCode::from(2);
        }
    };

    let status = gb.run(max_steps);
    let serial = gb.serial_output();
    print!("{serial}");
    if !serial.ends_with('\n') && !serial.is_empty() {
        println!();
    }

    match status {
        RunStatus::SerialPassed => {
            eprintln!("[softgbc] PASSED");
            ExitCode::SUCCESS
        }
        RunStatus::SerialFailed => {
            eprintln!("[softgbc] FAILED");
            ExitCode::from(1)
        }
        RunStatus::MaxSteps => {
            eprintln!("[softgbc] TIMEOUT after {max_steps} steps");
            ExitCode::from(3)
        }
    }
}
