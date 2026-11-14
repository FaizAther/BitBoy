# interrupt

`IF` (`FF0F`), `IE` (`FFFF`), priority, CPU IRQ entry.

## Current status *(2026-11)*

| Piece | Where | State |
|---|---|---|
| `IF` / `IE` storage | `softgbc-core` `DmgBus` (+ SoC Blargg overlay) | **Done** |
| Priority + vectoring | `sm83-iss` IRQ service | **Done** |
| Clear `IF` bit on entry | CPU | **Done** |
| Dedicated `interrupt/` crate API | this folder | **Not extracted** (checklist only) |
| Timer / VBlank / STAT sources | timer + PPU | **Not wired** (blocks Blargg `02`) |

Manual `IF` writes + CPU unit tests already force IRQ service. Hardware sources are the remaining work (start with **timer**).

The timer is a **separate peripheral** advanced by the SoC master clock (`cpu.step` → `timer.tick`); it only **sets** `IF` bit 2. The CPU decides later whether to vector. See [`../peripherals/src/Timer.README.md`](../peripherals/src/Timer.README.md) and [`../soc/README.md`](../soc/README.md).

## Checklist

- [x] Readable/writable `IF` and `IE` (on bus)
- [x] Priority order when multiple pending (in CPU)
- [x] CPU interrupt dispatch when `IME` and masked bit set
- [x] Clear corresponding `IF` bit on service
- [x] Interaction with `HALT` wake-up (basic)
- [ ] Sources: timer (next), VBlank, STAT, serial, joypad
- [ ] Optional dedicated module in this folder (vs bus + CPU)
- [ ] Integration tests with PPU VBlank + timer
