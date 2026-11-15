//! GDB Remote Serial Protocol stub for SoftGBC SM83.
//!
//! ```text
//! gdb -ex 'target remote 127.0.0.1:<gdb-port>'
//! (gdb) info registers
//! (gdb) stepi
//! (gdb) x/16xb $pc
//! ```

mod arch;
mod target;

pub use arch::{Sm83Arch, Sm83Regs};
pub use target::{ExecMode, GdbMachine, SoftGbcGdbTarget};

use gdbstub::common::Signal;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::run_blocking;
use gdbstub::stub::{DisconnectReason, GdbStub, SingleThreadStopReason};
use gdbstub::target::Target;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub enum GdbRunEvent {
    IncomingData,
    DoneStep,
    Break,
}

enum EventLoop {}

impl run_blocking::BlockingEventLoop for EventLoop {
    type Target = SoftGbcGdbTarget;
    type Connection = TcpStream;
    type StopReason = SingleThreadStopReason<u32>;

    fn wait_for_stop_reason(
        target: &mut SoftGbcGdbTarget,
        conn: &mut Self::Connection,
    ) -> Result<
        run_blocking::Event<SingleThreadStopReason<u32>>,
        run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as Connection>::Error,
        >,
    > {
        let mut poll = || conn.peek().map(|b| b.is_some()).unwrap_or(true);

        // Read exec mode once, then step in short lock bursts so HTTP play can proceed.
        let mode = {
            let g = target
                .machine
                .lock()
                .map_err(|_| run_blocking::WaitForStopReasonError::Target("lock"))?;
            g.exec_mode
        };

        match mode {
            ExecMode::Step => {
                let hit = {
                    let mut g = target
                        .machine
                        .lock()
                        .map_err(|_| run_blocking::WaitForStopReasonError::Target("lock"))?;
                    let hit = g.step_insn();
                    g.gdb_halted = true;
                    hit
                };
                if hit {
                    Ok(run_blocking::Event::TargetStopped(
                        SingleThreadStopReason::SwBreak(()),
                    ))
                } else {
                    Ok(run_blocking::Event::TargetStopped(
                        SingleThreadStopReason::DoneStep,
                    ))
                }
            }
            ExecMode::Continue => loop {
                if poll() {
                    let byte = conn
                        .read()
                        .map_err(run_blocking::WaitForStopReasonError::Connection)?;
                    return Ok(run_blocking::Event::IncomingData(byte));
                }
                let hit = {
                    let mut g = target
                        .machine
                        .lock()
                        .map_err(|_| run_blocking::WaitForStopReasonError::Target("lock"))?;
                    let mut hit = false;
                    for _ in 0..256 {
                        if g.step_insn() {
                            hit = true;
                            break;
                        }
                    }
                    if hit {
                        g.gdb_halted = true;
                    }
                    hit
                };
                if hit {
                    return Ok(run_blocking::Event::TargetStopped(
                        SingleThreadStopReason::SwBreak(()),
                    ));
                }
                std::thread::yield_now();
            },
        }
    }

    fn on_interrupt(
        target: &mut SoftGbcGdbTarget,
    ) -> Result<Option<SingleThreadStopReason<u32>>, <SoftGbcGdbTarget as Target>::Error> {
        if let Ok(mut g) = target.machine.lock() {
            g.gdb_halted = true;
        }
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT)))
    }
}

/// Accept one GDB client and run the blocking stub until disconnect.
pub fn serve_gdb_client(
    machine: Arc<Mutex<GdbMachine>>,
    stream: TcpStream,
) -> Result<DisconnectReason, Box<dyn std::error::Error>> {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
    {
        let mut g = machine.lock().map_err(|_| "gdb machine lock")?;
        g.gdb_halted = true;
        g.exec_mode = ExecMode::Continue;
    }
    let mut target = SoftGbcGdbTarget::new(machine.clone());
    let stub = GdbStub::new(stream);
    let reason = stub.run_blocking::<EventLoop>(&mut target)?;
    if let Ok(mut g) = machine.lock() {
        g.gdb_halted = false;
    }
    Ok(reason)
}

/// Bind `127.0.0.1:port` (`port == 0` → ephemeral) and serve GDB clients in a background thread.
pub fn run_gdb_listener(
    machine: Arc<Mutex<GdbMachine>>,
    port: u16,
) -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let bound = listener.local_addr()?.port();
    eprintln!("softgbc-gdb stub on 127.0.0.1:{bound}");
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            match conn {
                Ok(stream) => {
                    let peer = stream
                        .peer_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| "?".into());
                    eprintln!("GDB connected from {peer}");
                    match serve_gdb_client(machine.clone(), stream) {
                        Ok(DisconnectReason::Disconnect) => eprintln!("GDB disconnected"),
                        Ok(other) => eprintln!("GDB session ended: {other:?}"),
                        Err(e) => eprintln!("GDB stub error: {e}"),
                    }
                }
                Err(e) => eprintln!("GDB accept error: {e}"),
            }
        }
    });
    Ok(bound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soc::SoftGbc;

    #[test]
    fn continue_batch_hits_breakpoint() {
        let mut raw = vec![0u8; 0x200];
        raw[0x0100] = 0x00;
        raw[0x0101] = 0x00;
        raw[0x0102] = 0x00;
        let mut m = GdbMachine::new(SoftGbc::from_rom_bytes(&raw));
        m.breakpoints.insert(0x0102);
        let mut hit = false;
        for _ in 0..8 {
            if m.step_insn() {
                hit = true;
                break;
            }
        }
        assert!(hit);
        assert_eq!(m.gb.cpu.pc, 0x0102);
    }
}
