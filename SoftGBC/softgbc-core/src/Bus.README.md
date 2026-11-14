# bus

16-bit address decode and routing for CPU (and later DMA).

**Implementation:** `softgbc-core/src/bus.rs` — `Bus` trait + `MemoryBus` / `DmgBus`.

## Current status *(2026-11)*

| Region | Routed? |
|---|---|
| `0000`–`7FFF` ROM | **Yes** |
| `8000`–`9FFF` VRAM | **Yes** (raw buffer) |
| `C000`–`DFFF` WRAM (+ echo) | **Yes** |
| `FE00`–`FE9F` OAM | **Yes** (raw buffer) |
| `FF01`–`FF02` | **`Serial`** (+ output log) |
| `FF04`–`FF07` | **`Timer`** |
| Other `FF00`–`FF7F` (incl. `IF`) | Flat IO RAM |
| `FF80`–`FFFE` HRAM | **Yes** |
| `FFFF` IE | **Yes** |

```rust
bus.tick_m_cycles(cpu.step(&mut bus));
let text = bus.serial_output();
```

Used by generic `soc::SoftGbc` — not a Blargg-specific bus.

## Checklist

- [x] `read8` / `write8`
- [x] ROM / WRAM / echo / HRAM / IE
- [x] VRAM / OAM buffers
- [x] Timer + serial peripheral routing
- [x] `tick_m_cycles` → timer → `IF`
- [ ] PPU-owned LCD regs
- [ ] Joypad / DMA routing
- [ ] Open-bus documentation
