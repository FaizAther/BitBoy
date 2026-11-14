# peripherals

DMG IO devices for SoftGBC: **timer**, **serial**, joypad, DMA, …

```text
soc::SoftGbc
  ├── softgbc-core::DmgBus
  │     ├── timer   (peripherals)  FF04–FF07
  │     ├── serial  (peripherals)  FF01–FF02 + output log
  │     └── VRAM / OAM / WRAM / …
  └── sm83-iss::Cpu
```

## Docs (next to sources)

| File | Module |
|---|---|
| [`src/Timer.README.md`](src/Timer.README.md) | `timer` |
| [`src/Serial.README.md`](src/Serial.README.md) | `serial` |
| [`src/Joypad.README.md`](src/Joypad.README.md) | `joypad` |
| [`src/Dma.README.md`](src/Dma.README.md) | `dma` |

## Shared pattern

- Peripheral **owns** its registers and side effects
- `DmgBus` **routes** by address
- SoC **`SoftGbc::step`** ticks devices via `tick_m_cycles`
- IRQ requests return as [`IrqLines`](src/irq.rs); bus ORs into `IF`

Blargg is **not** a special bus — it is `soc/tests/blargg_cpu_instrs.rs` over the generic machine.
