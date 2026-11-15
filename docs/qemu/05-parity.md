# 05 — Differential parity and acceptance gates

Prove equivalence in layers: focused state vectors first, short traces second,
whole ROM outcomes last. A Blargg “Passed” marker is valuable but cannot locate
the first bad instruction by itself.

**Refs:** [`00-harness.md`](00-harness.md) ·
[`semantics.md`](semantics.md) · SoftGBC
[`blargg_cpu_instrs.rs`](../../SoftGBC/soc/tests/blargg_cpu_instrs.rs) ·
[`tests/roms/README.md`](../../SoftGBC/tests/roms/README.md)

## Verification pyramid

```text
                    ┌─────────────────────┐
                    │ Blargg + LLVM ROMs  │  final behaviour
                  ┌─┴─────────────────────┴─┐
                  │ bounded differential     │  event sequences
                ┌─┴─────────────────────────┴─┐
                │ single-boundary vector corpus│  exact semantics
              ┌─┴─────────────────────────────┴─┐
              │ decoder / device / reset tests   │  plumbing
              └───────────────────────────────────┘
```

## Status *(2026-11)*

| Gate | SoftGBC | QEMU |
|---|---|---|
| Single-opcode corpus | Existing smoke/unit tests | Open |
| Early fixtures | Present | Open |
| Blargg individuals | 11/11 via SoftGBC SoC | Open |
| LLVM/result ROM | Planned/shared milestone | Open |

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **One reproducible parity entry point**

  **What:** Provide one command that builds/locates both runners, selects a test,
  normalizes reset/stop options, writes temporary traces, and invokes
  `trace-diff`. Avoid per-test shell scripts with subtly different defaults.

  **Structure:**
```text
parity-run TEST [--max-events N]
  resolve exact ROM bytes
  compute SHA-256
  run SoftGBC oracle
  run QEMU candidate
  compare metadata + events
  retain artifacts only on failure (or explicit flag)
```

  **Min test:**
```text
parity-run 01_nop succeeds from repository root
bad QEMU path reports setup error, not semantic mismatch
```

  **Done when:** documentation and CI call the same entry point with no hidden
  working-directory assumptions.

---

- [ ] **Shared single-boundary corpus**

  **What:** Convert SoftGBC’s opcode tests into transportable vectors containing
  pre-state, opcode/immediate bytes, memory patches, IF/IE, expected event, and
  post-state. Include more than one vector for flag/conditional instructions;
  “each opcode executes once” is not semantic coverage.

  **Structure:**
```text
vector:
  name
  reset/profile
  pre CPU state
  memory patches + IF/IE
  bytes at PC
  expected event kind/state/writes/M

runner adapters:
  SoftGBC adapter
  QEMU/QTest adapter
```

  **Min test:**
```text
representatives:
  NOP, immediate load, (HL) store, ALU carry, conditional both paths,
  CB register, CB (HL), stack, EI sequence, IRQ
```

  **Done when:** every legal primary/CB slot appears and semantic corner cases
  have named additional vectors.

---

- [ ] **Early fixture ladder**

  **What:** Run SoftGBC’s small fixtures before Blargg. They verify reset,
  loading, stop boundaries, loops, and basic state with tiny failure windows.
  Convert `.hex` once to identical binary bytes for both runners.

  **Structure:**
```text
fixture source
   └── canonical converter ──► binary + entry + stop policy
                                  ├── SoftGBC
                                  └── QEMU
```

  **Min test:**
```text
01_nop: one-event trace equality
02_jr_spin: bounded repeated-PC equality
03_ld_imm: register post-state equality
```

  **Done when:** every maintained SoftGBC fixture passes through the common
  parity entry point.

---

- [ ] **Determinism and repeatability gate**

  **What:** Differential traces are useful only if each producer is stable.
  Run each side twice with identical metadata/options and compare it to itself
  before cross-engine comparison.

  **Structure:**
```text
SoftGBC run A == SoftGBC run B
QEMU run A    == QEMU run B
then SoftGBC  == QEMU
```

  **Min test:**
```text
JR loop for 10,000 events produces byte-identical traces on repeated runs
```

  **Done when:** CI has no wall-clock-derived event fields and repeated traces
  are stable.

---

- [ ] **Blargg CPU individuals without interrupt dependencies**

  **What:** Run each suitable `cpu_instrs` individual with the same ROM bytes,
  serial pass/fail convention, maximum event count, and reset profile. Preserve
  individual test names so one failure is actionable.

  **Structure:**
```text
for each agreed individual except timer-dependent 02:
  SoftGBC → serial marker + bounded trace
  QEMU    → serial marker + bounded trace
  require same outcome
  on failure, diff bounded traces around first mismatch
```

  **Min test:**
```text
01-special.gb reaches "Passed" on both before max_events
```

  **Done when:** all non-interrupt individuals green in separate test cases;
  there is no blanket “Blargg failed” aggregate.

---

- [ ] **Blargg `02-interrupts`**

  **What:** Gate this only after timer and interrupt-device focused tests pass.
  Use the actual deterministic timer model—not a scripted IF injection—because
  the ROM is validating integration.

  **Structure:**
```text
timer unit tests
  ↓
CPU interrupt matrix
  ↓
02-interrupts.gb end-to-end
```

  **Min test:**
```text
same ROM SHA/reset profile on both runners
both reach "Passed"; neither reaches timeout/max_events
```

  **Done when:** `02-interrupts` is an ordinary required parity case, with no
  special host timing.

---

- [ ] **Final-state and debug-exit tests**

  **What:** Trace equality complements externally observable outcomes. Test
  `$FF80`, serial output, and guest debug-exit status explicitly so a trace
  producer bug cannot falsely declare success.

  **Structure:**
```text
test result:
  trace comparison
  final selected memory
  serial bytes
  guest status
  stop reason
```

  **Min test:**
```text
sum ROM → [$FF80]=$2E, guest status=0, same stop reason
intentional fail ROM → same nonzero guest code
```

  **Done when:** Chunk E smoke asserts all four result channels.

---

- [ ] **First-divergence reduction protocol**

  **What:** Treat the first differing event as evidence, not automatically as
  the root cause. Reproduce it with the smallest pre-state/memory vector, then
  determine whether QEMU, the trace producer, or the SoftGBC oracle is wrong.

  **Structure:**
```text
1. verify metadata and self-determinism
2. locate first differing event/field
3. capture preceding state + required memory
4. create named single-boundary regression vector
5. verify SoftGBC against existing unit/hardware reference
6. fix responsible component
7. rerun vector → fixture → ROM
```

  **Min test:**
```text
inject a wrong ADD half-carry in a test branch
comparator identifies post.af
reduced vector fails before fix and passes after fix
```

  **Done when:** every fixed semantic divergence leaves a focused regression
  vector, not only a large ROM test.

---

- [ ] **CI tiers and artifact policy**

  **What:** Keep fast semantic gates on every change and longer ROM traces in a
  broader tier. Store full traces only for failed jobs; report first divergence
  inline.

  **Structure:**
```text
per change:
  build + unit + decoder coverage + single-boundary corpus + early fixtures

broader:
  Blargg individuals + long differential traces + LLVM smoke

failure artifacts:
  metadata, first-divergence report, bounded trace window, commands
```

  **Min test:**
```text
forced mismatch uploads enough data to reproduce locally
successful run does not archive huge JSONL files
```

  **Done when:** parity failures are reproducible from CI output and normal runs
  remain fast.
