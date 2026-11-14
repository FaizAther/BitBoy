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

**Accuracy follow-ups** — What → Structure → Min test → Done when  
**Refs:** [GBEDG — Timers](https://github.com/Ashiepaws/GBEDG/blob/master/timers/index.md)

---

- [ ] **All four TAC rates (falling-edge)**

  **What:** TIMA bumps on falling edge of selected DIV bit while TAC enable.  
  **Structure:**
```text
rates: 00→DIV bit9 (/1024), 01→bit3 (/16), 10→bit5 (/64), 11→bit7 (/256)
on each T: if enable and selected bit 1→0: TIMA += 1
```
  **Min test:**
```text
TAC = enable + /16, TIMA = 0
tick(15) → TIMA still 0
tick(1)  → TIMA == 1
```
  **Done when:** all four rates have a period test.

---

- [ ] **DIV-reset / TAC-disable edges**

  **What:** writing DIV (or disabling TAC) can cause a spurious TIMA edge (GBEDG).  
  **Structure:** model falling edge when reset clears a high tap bit.  
  **Min test:**
```text
arrange selected DIV bit high
write FF04 = 0
assert TIMA behaviour matches your documented rule
```
  **Done when:** behaviour matches GBEDG note you chose.

---

- [ ] **Overflow delay + TIMA-write cancel**

  **What:** after TIMA wraps, ~4 T before reload/IF; write TIMA in window cancels.  
  **Structure:**
```text
on TIMA 0xFF→0x00: start 4-T delay
after delay: TIMA = TMA, IF bit 2
if TIMA written during delay: cancel reload/IF
```
  **Min test:**
```text
force overflow → within 4 T write TIMA
assert IF bit 2 clear and TIMA == written value
```
  **Done when:** cancel window tested.

---

- [ ] **Mooneye / timing ROMs**

  **What:** external ROMs as acceptance.  
  **Structure:** SoftGbc run until serial Passed/Failed.  
  **Min test:**
```text
run timer / instr_timing / mem_timing ROM
expect serial "Passed"
```
  **Done when:** chosen suite green.

---

## Shape (pseudocode)

```text
Timer:
  tick(t) → maybe IRQ bit 2
  read/write FF04..FF07

bus ORs IRQ bits into IF after tick
```
