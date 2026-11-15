# 03 — Interrupt boundaries, EI, and HALT

Make QEMU’s execution loop observe the same boundaries as SoftGBC `Cpu::step`.
Instruction semantics are already present from [`02-helper-isa.md`](02-helper-isa.md);
this page handles asynchronous state and wake-up.

**Refs:** [`semantics.md`](semantics.md) · SoftGBC
[`interrupt/README.md`](../../SoftGBC/interrupt/README.md) ·
[`Timer.README.md`](../../SoftGBC/peripherals/src/Timer.README.md)

## Boundary model

```text
                              boundary
                                 │
                 read pending = IF & IE & $1F
                                 │
              ┌──────────────────┼──────────────────┐
              ▼                  ▼                  ▼
          CPU halted       IME + pending      otherwise
              │                  │                  │
     idle / wake / IRQ        dispatch          execute one
                                                instruction
                                                    │
                                              advance EI delay
```

The QEMU main loop may use `CPUClass` interrupt hooks and `CPUState` wake
machinery, but the visible event order above is non-negotiable.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **IF/IE device interface**

  **What:** The CPU needs a stable interface to read `$FF0F` (IF), `$FFFF`
  (IE), clear one accepted IF bit, and be notified when pending changes. The
  registers are board state, not duplicated inside `CPUArchState`.

  **Structure:**
```text
gbc-test IRQ controller:
  if_reg, ie_reg
  pending() = if_reg & ie_reg & $1F
  request(bit) → if_reg |= bit; notify CPU
  acknowledge(bit) → if_reg &= ~bit

MMIO:
  $FF0F ↔ IF with documented upper-bit behaviour
  $FFFF ↔ IE
```

  **Min test:**
```text
write IF=$04, IE=$00 → pending=0
write IE=$04 → pending=$04 and CPU work notification asserted
ack $04 → pending=0 without changing unrelated IF bits
```

  **Done when:** QTest can seed/read/ack IF/IE and CPU code has one owner-neutral
  interface rather than direct board-struct access.

---

- [ ] **QEMU wake and interrupt hooks**

  **What:** Connect pending changes to QEMU’s CPU scheduling. A halted CPU must
  wake when `(IF & IE & $1F) != 0`, regardless of IME; IME controls dispatch,
  not wake. Keep `env.halted` and QEMU’s scheduler-visible halted state
  synchronized.

  **Structure:**
```text
has_work(cpu):
  return irq_controller.pending() != 0

on pending transition 0 → nonzero:
  signal CPU interrupt/work

on HALT instruction:
  env.halted = true
  put CPU into QEMU halted state after boundary

on wake:
  clear both representations exactly once
```

  **Min test:**
```text
HALT with no pending → QEMU does not busy-loop guest instructions
raise pending with IME=0 → CPU wakes and performs wake-only event
raise pending with IME=1 → CPU wakes and dispatches
```

  **Done when:** no lost wake-up, duplicate wake event, or host busy-spin occurs.

---

- [ ] **Interrupt recognition before fetch**

  **What:** At every non-halted boundary, dispatch a pending enabled interrupt
  when IME is set before translating/executing the next opcode. In helper-first
  mode each TB ends at a boundary, simplifying this check.

  **Structure:**
```text
boundary entry:
  pending = controller.pending()
  if env.ime && pending:
    execute IRQ event
    do not fetch guest opcode
    exit boundary
```

  **Min test:**
```text
PC points to a state-changing opcode; IME=1; pending VBlank
one event → IRQ vector, opcode side effect absent
```

  **Done when:** trace event is `irq` with empty opcode bytes and matches
  SoftGBC pre/post state.

---

- [ ] **Dispatch sequence and priority**

  **What:** Choose the lowest pending bit, clear IME, acknowledge only that IF
  bit, push the interrupted PC in SoftGBC byte order, and jump to its vector.
  Normal dispatch costs 5 M; HALT-to-dispatch costs 6 M.

  **Structure:**
```text
pending bit:  0      1      2      3       4
vector:      $40    $48    $50    $58     $60

dispatch:
  selected = lowest_set_bit(pending)
  IME = false
  controller.acknowledge(selected)
  push16(PC)
  PC = vector[selected]
  M = halted_before ? 6 : 5
```

  **Min test:**
```text
IF=IE=$1F → choose VBlank
verify remaining IF=$1E, stack bytes/order, vector, IME=false, M=5
repeat from HALT → M=6
```

  **Done when:** all five vectors plus multi-pending priority pass focused
  comparison.

---

- [ ] **Delayed EI boundary state**

  **What:** Represent the scheduled IME enable explicitly in architectural
  state so TB exits, tracing, reset, and GDB do not lose it. Advance it only
  after an executed instruction, not after IRQ or HALT-idle events.

  **Structure:**
```text
EI instruction schedules enable
end of following instruction enables IME
DI clears IME and schedule
IRQ event does not advance schedule
reset clears schedule
```

  **Min test:**
```text
EI; NOP; pending IRQ → instruction, instruction, IRQ
EI; DI; NOP; pending IRQ → no dispatch
reset after EI → no delayed enable survives
```

  **Done when:** exact event sequences and IME post-state match SoftGBC.

---

- [ ] **HALT idle and wake-only accounting**

  **What:** A halted CPU with no pending interrupt produces 1 M of elapsed CPU
  time per modeled boundary without fetching. Pending with IME clear produces
  one 1-M wake-only event, preserves IF, and leaves PC unchanged.

  **Structure:**
```text
no pending:
  event=halt-idle; halted remains true; PC stable; M=1

pending && !IME:
  event=halt-wake; halted=false; IF unchanged; PC stable; M=1
```

  **Min test:**
```text
capture one event for each branch
assert bytes=[], PC stable, exact halted/IF state, M=1
```

  **Done when:** SoftGBC and QEMU traces agree without fabricating NOP events.

---

- [ ] **Deterministic timer source for interrupt ROMs**

  **What:** Blargg `02-interrupts` needs `$FF04..$FF07` and timer overflow to IF
  bit 2. Port the already-tested SoftGBC coarse falling-edge model at the device
  level; do not substitute a host wall-clock timer. Advance it from CPU
  M-cycles × 4.

  **Structure:**
```text
after each CPU event:
  t_cycles = event.m_cycles * 4
  timer.tick(t_cycles)
  if TIMA overflow requests IRQ:
    controller.request($04)

timer MMIO:
  DIV $FF04, TIMA $FF05, TMA $FF06, TAC $FF07
```

  **Min test:**
```text
seed TIMA near overflow; configure TAC
advance exact event cycle sum
assert TIMA reload and IF bit 2 at same boundary as SoftGBC
```

  **Done when:** focused timer/IRQ trace matches and Blargg `02-interrupts`
  reaches the same pass marker as SoftGBC.

---

- [ ] **Interrupt regression matrix**

  **What:** Gate all combinations that are easy to break while optimizing TCG:
  IME on/off, pending/non-pending, halted/running, delayed EI, multiple sources,
  and RETI.

  **Structure:**
```text
matrix dimensions:
  halted × IME × pending
  EI schedule state
  one vs many pending bits
  RETI return
```

  **Min test:**
```text
at least one vector for every reachable matrix branch
```

  **Done when:** matrix runs against SoftGBC and QEMU helper engine in CI and
  remains required for direct-TCG changes.
