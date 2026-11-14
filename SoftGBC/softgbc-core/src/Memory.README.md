# memory

On-console RAM / ROM buffers.

**Implementation:** `softgbc-core/src/memory.rs` — `Memory` / `Ram` / `Rom`.  
Wired by `bus.rs` (`MemoryBus`). There is no separate SoftGBC `memory/` crate.

## Current status *(2026-11)*

| Buffer | State |
|---|---|
| ROM (32 KiB default in `DmgBus`) | **Done** |
| WRAM 8 KiB | **Done** |
| HRAM 127 B | **Done** |
| IO 128 B + IE 1 B | **Done** (as bus-owned) |
| VRAM / OAM | SoC Blargg arrays only — not here yet |
| Echo RAM | Not implemented |

## Checklist

- [x] WRAM 8KiB (`C000`–`DFFF`)
- [ ] Echo RAM behaviour (`E000`–`FDFF`)
- [x] HRAM (`FF80`–`FFFE`)
- [ ] VRAM 8KiB buffer (shared with PPU)
- [ ] OAM 160-byte buffer (shared with PPU/DMA)
- [x] Basic constructors / load helpers
- [ ] Optional dump helpers for debugging

## Testing

Prefer workspace tests:

```bash
cd SoftGBC
cargo test -p softgbc-core
```
