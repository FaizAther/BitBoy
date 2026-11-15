# 06 — Direct TCG, debugger, and toolchain handoff

Optimize only after the helper engine passes the full parity gate. Keep the
helper engine selectable as an internal QEMU oracle so every optimization can be
tested QEMU-helper ↔ QEMU-TCG ↔ SoftGBC.

**Refs:** [`02-helper-isa.md`](02-helper-isa.md) ·
[`05-parity.md`](05-parity.md) · SoftGBC
[`softgbc-host/src/gdb`](../../SoftGBC/softgbc-host/src/gdb/) · repo M3/M4 ·
QEMU TCG developer documentation

## Optimization contract

```text
                         same shared corpus
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
  SoftGBC oracle       QEMU helper engine     QEMU direct-TCG engine
          └─────────────────────┴─────────────────────┘
                         identical events
```

Performance work may change translation-block layout and host instructions. It
may not change guest state, ordered writes, cycle counts, or interrupt
boundaries.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Selectable helper and direct-TCG engines**

  **What:** Preserve the complete helper path behind a development/test CPU
  property or build option while direct TCG lands incrementally. Both modes use
  the same decoder and board.

  **Structure:**
```text
sm83-cpu,engine=helper   # one instruction/TB correctness baseline
sm83-cpu,engine=tcg      # optimized translation

decoder classification shared
execution emission differs
```

  **Min test:**
```text
same vector/ROM under both modes
helper trace == direct-TCG trace == SoftGBC trace
```

  **Done when:** CI runs the single-boundary corpus in both QEMU modes.

---

- [ ] **Direct TCG for register-only leaf operations**

  **What:** Start with NOP, register moves, and simple logical operations.
  Map architectural fields to TCG globals with widths/masks that preserve SM83
  semantics. Keep cycle accounting and trace hooks common.

  **Structure:**
```text
LD B,C:
  tcg move env.c → env.b
  PC += 1
  account 1 M

AND A,B:
  TCG AND
  set Z from low 8-bit result
  set N=0, H=1, C=0
```

  **Min test:**
```text
helper vs TCG for every register index and ALU edge vector
assert upper host bits never leak into 8-bit guest state
```

  **Done when:** first leaf group has no helper execution calls and all parity
  tiers remain green.

---

- [ ] **Direct TCG memory and conditional operations**

  **What:** Convert loads/stores and branches using QEMU TCG memory operations
  so MMIO/watchpoints remain functional. Evaluate conditions at runtime.
  PC-changing paths terminate or chain TBs correctly.

  **Structure:**
```text
memory:
  TCG guest load/store with 16-bit address masking

conditional branch:
  runtime test of F
  taken → assign target PC + taken cycles
  not taken → sequential PC + untaken cycles
  end TB when PC may change
```

  **Min test:**
```text
RAM + MMIO loads/stores visible to QTest/watchpoint
JR/CALL/RET taken and not-taken traces equal helper mode
```

  **Done when:** memory/control groups no longer require generic execute helpers
  and retain exact write/cycle events.

---

- [ ] **Translation-block policy**

  **What:** Normal mode may place several straight-line instructions in one TB.
  Branches, HALT/STOP, EI boundary-sensitive sequences, debug single-step, and
  trace mode must preserve observable boundaries. Trace mode may deliberately
  force one instruction per TB until per-instruction instrumentation is proven.

  **Structure:**
```text
end TB on:
  control-flow PC write
  HALT / STOP
  boundary requiring main-loop interrupt check
  single-step request
  configured trace boundary (initial policy)

normal straight-line path:
  continue until QEMU TB limit/page boundary
```

  **Min test:**
```text
long NOP sequence forms multi-insn TB in normal mode
trace/single-step still reports one instruction at a time
pending IRQ is observed at the SoftGBC-equivalent boundary
```

  **Done when:** TB optimization cannot skip an interrupt or merge trace events.

---

- [ ] **Cycle accounting under multi-instruction TBs**

  **What:** Accumulate exact per-instruction M-cycles at runtime, including
  conditional paths. Advance devices at boundaries required by the model.
  Do not replace this with TB instruction count.

  **Structure:**
```text
for each generated instruction:
  runtime_m = path-specific M
  total_m += runtime_m
  preserve device/interrupt observation boundary
```

  **Min test:**
```text
TB containing mixed 1/2/3-M instructions produces exact total
conditional inside TB selects correct M
timer IRQ appears at same event boundary as helper mode
```

  **Done when:** helper and TCG traces agree on cycles and timer-visible order.

---

- [ ] **GDB register description and mapping**

  **What:** Define a QEMU target description for the physical SM83 registers.
  Prefer `A F B C D E H L` (8-bit), `SP PC` (16-bit), and useful pseudo/state
  exposure for IME/HALT. Verify what the repository’s supported GDB accepts;
  target XML alone does not guarantee that stock GDB recognizes a new
  architecture name.

  **Structure:**
```text
gdb-xml/sm83-cpu.xml:
  stable register numbers and widths

gdbstub.c:
  read/write mappings
  F writes mask low nibble
  PC/SP writes mask to 16 bits

compatibility decision:
  recognized SM83 architecture if tool support exists
  otherwise documented generic/custom register feature
  do not silently reuse SoftGBC's i386 layout
```

  **Min test:**
```text
connect supported gdb/gdb-multiarch to -S -gdb tcp::PORT
read all registers
write AF-equivalent state and verify F masking
break at $0100; single-step NOP → PC=$0101
```

  **Done when:** debugger/version requirement is documented and an automated
  remote-protocol test verifies register serialization.

---

- [ ] **Breakpoints, watchpoints, and single-step**

  **What:** QEMU’s debugging value is more than register display. Software
  breakpoints must stop before execution; watchpoints must observe guest memory
  access; single-step must execute exactly one SM83 boundary event.

  **Structure:**
```text
breakpoint $0100 → stop with PC=$0100
watch $FF80 write → stop on result store
step:
  instruction → one instruction
  pending IRQ → one IRQ event
  halted CPU → documented behaviour
```

  **Min test:**
```text
scripted GDB remote session performs all three operations
```

  **Done when:** debugger tests pass in helper and direct-TCG modes.

---

- [ ] **LLVM/assembler handoff ROM**

  **What:** Consume a flat ROM produced by the project toolchain (or identical
  hand-assembled bytes until codegen lands). It computes `20 + 22`, writes
  `$2E` to `$FF80`, then writes success to `$FF7F`.

  **Structure:**
```text
source / assembly
  ↓ project assembler/linker
32 KiB flat image, entry $0100
  ├── SoftGBC → [$FF80]=$2E, status 0
  └── QEMU    → [$FF80]=$2E, status 0
```

  **Min test:**
```text
record image SHA-256
both runners agree on trace, result byte, stop reason, and status
```

  **Done when:** repo M4 exit criterion runs from one reproducible command.

---

- [ ] **Performance measurement without semantic relaxation**

  **What:** Measure before/after throughput on a bounded ROM loop after direct
  TCG is correct. Performance numbers must use tracing disabled and the same
  event count. No parity test is removed because an optimization is “faster.”

  **Structure:**
```text
benchmark:
  fixed image + reset
  fixed guest event count
  trace off
  helper vs TCG elapsed host time
```

  **Min test:**
```text
benchmark command reports reproducible event/s for both engines
```

  **Done when:** direct TCG has a measured benefit or remains explicitly
  optional; correctness gates stay unchanged.

---

- [ ] **Local-fork and upstream-readiness boundary**

  **What:** Develop in `GBA/qemu/` first. Keep QEMU coding style, licensing,
  generated-decoder conventions, and focused commits so upstreaming remains
  possible without making it a Chunk E dependency.

  **Structure:**
```text
SoftGBC/    oracle and tests
docs/qemu/  design/checklists
qemu/       independent implementation

commit layers:
  target registration
  test machine
  helper ISA groups
  IRQ/timer
  tracing/GDB
  direct TCG
```

  **Min test:**
```text
clean target-only build and test instructions work from a fresh checkout
```

  **Done when:** Chunk E can be reviewed as layered changes and does not depend
  on upstream acceptance.
