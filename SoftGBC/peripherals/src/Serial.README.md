# Serial

Link cable (`SB` / `SC`). Low priority for games; **high value for headless test ROMs**.

**Implementation:** [`serial.rs`](serial.rs) — owned by `softgbc-core::DmgBus` (`FF01`/`FF02`).

## Current status *(2026-11)*

- Write `SC` with transfer-start bit → append `SB` to `Serial::output`, clear bit 7
- `DmgBus::serial_output()` exposes the log
- Generic `SoftGbc::run` / CLI treat serial `Passed` / `Failed` as stop conditions

## Checklist

- [x] Reads/writes to `FF01` / `FF02` (no hang)
- [x] Capture output on internal-clock transfer start
- [x] Wired through `DmgBus` (not a Blargg-only bus)
- [ ] Optional transfer complete IRQ
- [ ] Optional loopback or null peer for tests
- [ ] Real peer / file-backed link (later)
