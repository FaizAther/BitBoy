# 00 — Differential trace harness

Define one machine-readable boundary trace for SoftGBC, QEMU, Verilator, and
eventually FPGA capture. This is infrastructure, not another CPU model.

**Refs:** SoftGBC [`sm83-iss/README.md`](../../SoftGBC/sm83-iss/README.md) ·
[`semantics.md`](semantics.md) · repo Chunk A `tools/trace-diff`

## Design rules

- Emit one record for every CPU boundary event: instruction, IRQ, HALT idle, or
  HALT wake.
- Record both pre-state and post-state; never overload one `pc` field.
- JSON numbers are decimal. Hex belongs only in human-readable diagnostics.
- Record ordered writes caused by the CPU event, not reads or peripheral writes.
- Put run configuration in one metadata record so reset differences are obvious.
- Comparison stops at the first differing event.

```text
  SoftGBC ──► metadata + events ──┐
                                  ├──► trace-diff ──► first semantic mismatch
  QEMU    ──► metadata + events ──┘
```

## Status *(2026-11)*

| Piece | State |
|---|---|
| Schema | Specified below; implementation open |
| SoftGBC producer | Open |
| Comparator | Open |
| QEMU producer | Open |

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Versioned metadata record**

  **What:** The first line describes schema version, producer, image identity,
  reset profile, and options. A trace from the wrong ROM/reset must fail before
  event comparison.

  **Structure:**
```json
{"type":"meta","schema":"sm83-trace-v1","producer":"softgbc","image_sha256":"...","reset_profile":"dmg-skip-boot-v1","entry":256}
```
```text
required:
  type, schema, producer, image_sha256, reset_profile, entry

optional:
  producer_version, command, max_events, notes
```

  **Min test:**
```text
compare traces with different image_sha256 → fail as configuration mismatch
compare softgbc vs qemu with same schema/image/profile → enter event comparison
```

  **Done when:** both producers emit valid metadata and the comparator validates
  it before reading event 0.

---

- [ ] **Canonical boundary-event record**

  **What:** A record contains the event kind, opcode bytes when applicable,
  complete pre/post CPU state, M-cycles, and ordered CPU writes. Explicit
  pre/post state removes ambiguity around branches, interrupts, and HALT.

  **Structure:**
```json
{"type":"event","seq":0,"kind":"insn","bytes":[0],"pre":{"pc":256,"sp":65534,"af":256,"bc":0,"de":0,"hl":0,"ime":false,"halted":false},"post":{"pc":257,"sp":65534,"af":256,"bc":0,"de":0,"hl":0,"ime":false,"halted":false},"m_cycles":1,"writes":[]}
```
```text
kind:
  insn       bytes contains opcode and immediates consumed
  irq        bytes is empty
  halt-idle  bytes is empty
  halt-wake  bytes is empty

writes:
  [{"addr":65532,"value":52},{"addr":65533,"value":18}]
  ordered exactly as observed on the CPU bus
```

  **Min test:**
```text
NOP event parses as strict JSON and has pre.pc=256, post.pc=257, m_cycles=1
IRQ event has kind=irq, empty bytes, stack writes in SoftGBC order
```

  **Done when:** a schema validator accepts representative records for all four
  event kinds and rejects hexadecimal JSON literals or missing required keys.

---

- [ ] **SoftGBC trace producer**

  **What:** Instrument the SoC/CPU boundary without changing normal execution.
  Capture pre-state and opcode bytes before `step`, collect CPU-originated writes
  through a tracing bus wrapper, then capture post-state and returned M-cycles.
  IRQ/HALT event classification must follow actual `step` behaviour.

  **Structure:**
```text
if trace enabled:
  snapshot pre
  begin ordered CPU-write capture
  classify expected boundary inputs (halted, IF, IE, IME)
  m = cpu.step(bus)
  snapshot post
  end capture
  emit event
else:
  cpu.step(bus) with no allocation / serialization
```

  **Min test:**
```text
run 01_nop for one event → meta + one NOP event
run interrupt unit scenario → one irq event, not a fabricated opcode
run without trace → existing focused tests unchanged
```

  **Done when:** focused SoftGBC tests cover each event kind and tracing disabled
  has no semantic effect.

---

- [ ] **Deterministic stop conditions**

  **What:** Both runners must stop on the same boundary: event count, debug-exit,
  serial pass/fail, or explicit PC predicate. Wall-clock timeout is only a
  safety net and is not a comparison boundary.

  **Structure:**
```text
stop policy priority:
  debug-exit
  serial pass/fail
  max_events
  optional stop_pc

metadata records selected policy
```

  **Min test:**
```text
same ROM + max_events=3 → both traces contain exactly 3 event records
infinite JR loop terminates by max_events, not wall time
```

  **Done when:** fixture commands reproduce identical event counts on repeated
  runs.

---

- [ ] **First-divergence comparator**

  **What:** `trace-diff` compares metadata, event count, and semantic fields.
  It reports one compact failure with surrounding context and decimal + hex
  formatting. It contains no opcode semantics.

  **Structure:**
```text
trace-diff [--context N] oracle.jsonl candidate.jsonl

validate metadata
for corresponding events:
  compare kind, bytes, pre, post, m_cycles, writes
  on mismatch:
    print seq, field path, oracle, candidate
    print N events before/after
    exit 1
if one stream ends early: report length mismatch
exit 0
```

  **Min test:**
```text
identical files → 0
post.af changed → 1 and field path "post.af"
write order swapped → 1 and field path "writes"
truncated candidate → 1 and exact ending sequence
```

  **Done when:** comparator unit tests cover metadata, nested fields, ordered
  writes, and unequal lengths.

---

- [ ] **QEMU trace producer**

  **What:** Emit at the same runtime boundary as SoftGBC. During helper-first
  bring-up, one instruction per TB makes this direct. Do not scrape `-d in_asm`;
  that log lacks post-state, ordered writes, and stable formatting.

  **Structure:**
```text
before runtime event:
  snapshot pre; begin CPU-write capture
execute helper / IRQ / HALT boundary
after runtime event:
  snapshot post; emit canonical event

trace option:
  dedicated machine/CPU option or trace output argument
```

  **Min test:**
```text
QEMU NOP trace validates against the schema
trace-diff softgbc-nop.jsonl qemu-nop.jsonl → 0
```

  **Done when:** all event kinds have QEMU producer tests and NOP/IRQ parity is
  green.

---

- [ ] **Golden trace policy**

  **What:** Prefer generating oracle traces in tests over committing long
  JSONL files. Commit only tiny schema fixtures needed to test parsing and error
  output; avoid stale multi-megabyte goldens.

  **Structure:**
```text
committed:
  tiny valid/invalid trace fixtures

generated during test:
  SoftGBC oracle trace
  QEMU candidate trace
```

  **Min test:**
```text
delete generated traces; run parity command; both regenerate and compare
```

  **Done when:** CI needs no manually refreshed large golden trace.
