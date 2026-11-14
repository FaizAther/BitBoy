# Timer

DMG timer peripheral: `DIV` / `TIMA` / `TMA` / `TAC`.

**Implementation:** [`timer.rs`](timer.rs) (`peripherals::timer`).

On real hardware this is a **separate device** on the same master clock as the CPU — not CPU code and **not** an OS thread. SoftGBC models it as its own struct; the **SoC** advances it from the cycle budget returned by `cpu.step()`. See [Master clock](#how-softgbc-should-tick-it) below and [`../../soc/README.md`](../../soc/README.md).

Primary reference (detailed behaviour): [GBEDG — The Timers](https://github.com/Ashiepaws/GBEDG/blob/master/timers/index.md) · also [Pan Docs — Timer](https://gbdev.io/pandocs/Timer_and_Divider_Registers.html).

---

## Current status *(2026-11)*

**Coarse implementation** in this crate; **owned by `softgbc-core::DmgBus`**. Generic `SoftGbc::step` calls `bus.tick_m_cycles(m)`.

| Item | State |
|---|---|
| `Timer` owns `div16` + `FF04`–`FF07` | Done |
| Routed from `DmgBus` read/write | Done |
| Falling-edge TIMA increment | Done (coarse) |
| Overflow → TMA reload + `IF` bit 2 via `tick_*` | Done (no 4 T-cycle delay yet) |
| Blargg `02-interrupts.gb` | **Passed** |

Accuracy follow-ups (DIV-reset glitches, overflow delay) still open — see checklist below.

---

## How SoftGBC should tick it

**T-cycle** = one system-clock tick (~4.19 MHz). **M-cycle** = one CPU machine step; **1 M-cycle = 4 T-cycles**.  
`cpu.step()` returns M-cycles; the timer’s internal DIV counts **T-cycles**. Convert in the SoC. See [`../../README.md`](../../README.md#t-cycles-vs-m-cycles).

```text
SoC (master clock)
  m = cpu.step(&mut bus)     // M-cycles for this instruction
  t = m * 4                  // 1 M-cycle = 4 T-cycles
  timer.tick(t)              // later: ppu / apu / dma too
  // timer may set IF bit 2; next cpu.step may service IRQ
```

- **Separate module** — yes (`Timer { … }` with `read` / `write` / `tick`)
- **Separate thread** — no (must stay cycle-deterministic with the CPU)

---

## Registers

### `$FF04` — Divider (`DIV`)

- Software sees an **8-bit** value that increments every **256 T-cycles**.
- Internally: a **16-bit** counter incremented **every T-cycle**; only the **upper 8 bits** are mapped at `$FF04`.
- **Any write** to `$FF04` resets the **entire** 16-bit internal counter to `0` (not just the visible byte).

### `$FF05` — Timer counter (`TIMA`)

- 8-bit counter software usually uses for periodic IRQs.
- Increment rate selected by `TAC`; increments are derived from the internal DIV counter (see [Operation](#timer-operation-falling-edge-model)).
- Fully readable and writable.

### `$FF06` — Timer modulo (`TMA`)

- When TIMA overflows, it is reloaded from TMA (after a short delay in accurate models).
- Fully readable and writable.

### `$FF07` — Timer control (`TAC`)

| Bits | Role |
|---|---|
| Bit 2 | Timer enable (see edge-case notes — disabling can still glitch TIMA) |
| Bits 1–0 | Clock select |

| TAC bits 1–0 | TIMA rate | DIV bit used for edge detect |
|---|---|---|
| `00` | CPU clock / 1024 | bit 9 |
| `01` | CPU clock / 16 | bit 3 |
| `10` | CPU clock / 64 | bit 5 |
| `11` | CPU clock / 256 | bit 7 |

Example: `TAC = 0b101` (`$05`) → enabled, clock / 16. **This is what Blargg `02-interrupts` uses.**

---

## Timer operation (falling-edge model)

Each T-cycle:

1. Increment the internal 16-bit DIV counter by 1.
2. Select one bit of that counter from TAC bits 1–0 (table above).
3. AND that bit with TAC enable (bit 2) → “AND result”.
4. If the AND result has a **falling edge** (was `1` last cycle, `0` this cycle) → **TIMA += 1**.

That is why TIMA rates match powers of two: the selected DIV bit falls every 16 / 64 / 256 / 1024 T-cycles.

### Edge case (TAC / DIV writes)

A falling edge can also appear when **enable goes 1→0** or when **DIV is reset**, even if the selected counter bit did not change. Accurate emulators still increment TIMA in those cases. Coarse SoftGBC v1 for Blargg `02` can ignore this until a test demands it.

### TIMA overflow → interrupt

When TIMA increments from `$FF`:

1. Accurate hardware: TIMA reads as `$00` for **4 T-cycles**, then loads **TMA**, and **then** sets **`IF` bit 2** (timer interrupt request at `$FF0F`).
2. Writing TIMA during that 4 T-cycle window can **cancel** the reload and the IRQ.
3. Same-cycle TIMA/TMA write races are documented in GBEDG; defer until Mooneye-level tests.

**Coarse model for `02-interrupts`:** on overflow, set TIMA = TMA (or 0 if you delay later) and `IF |= 0x04` in the same tick batch — good enough for that ROM’s “did IF get set after ~1000 cycles?” window.

---

## What Blargg `02` test 4 expects

```text
IE already has timer bit
TAC = $05, TIMA = 0, IF = 0
delay ~500 cycles  → IF timer bit still clear
delay ~500 more    → IF timer bit must be set
```

IME can stay off: the test only **reads IF**, it does not require the ISR to run. HALT wake (test 5) also depends on this same IF bit appearing.

---

## Checklist

**Minimum for SoftGBC now**

- [x] `Timer` type + `tick(t_cycles)` (or M-cycles, but one unit only)
- [x] Bus decode `FF04`–`FF07` (read/write semantics)
- [x] `DIV` increments; write resets internal counter
- [x] `TIMA` / `TMA` / `TAC` storage
- [x] TAC enable + clock select (at least `/16` for `TAC=$05`)
- [x] SoC: after each `cpu.step()`, tick timer with that cycle budget
- [x] TIMA overflow → reload TMA + set `IF` bit 2
- [x] `02-interrupts.gb` **Passed**

**Accuracy follow-ups**

- [ ] Full falling-edge AND model (all four rates) — basic model present; verify all rates
- [ ] DIV-reset / TAC-disable spurious increments
- [ ] 4 T-cycle overflow delay + TIMA-write cancel
- [ ] Mooneye timer / `instr_timing` / `mem_timing` as needed

---

## API

```rust
use peripherals::{Timer, IrqLines};

impl Timer {
    pub fn tick(&mut self, t_cycles: u32) -> IrqLines; // OR `.bits` into IF
    pub fn read(&self, addr: u16) -> u8;               // FF04..=FF07
    pub fn write(&mut self, addr: u16, data: u8);
}
```

SoC owns `Timer` inside the bus, routes IO, ORs `IrqLines` into `IF`.
