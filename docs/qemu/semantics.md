# SoftGBC → QEMU behavioural contract

This is the semantic boundary for Chunk E. QEMU must produce the same
instruction-boundary observations as tested SoftGBC `sm83-iss`.

**Refs:** [`sm83-iss/src/lib.rs`](../../SoftGBC/sm83-iss/src/lib.rs) ·
[`interrupt/README.md`](../../SoftGBC/interrupt/README.md) ·
[`t-and-m-cycles.md`](../../SoftGBC/soc/docs/t-and-m-cycles.md) ·
[`00-harness.md`](00-harness.md)

## Contract surface

```text
 inputs                                  observable outputs
 ──────                                  ──────────────────
 CPU pre-state ─┐                   ┌──► CPU post-state
 opcode bytes ──┼──► one boundary ──┼──► ordered memory writes
 readable bus ──┤                   ├──► event kind (insn/irq/halt)
 IF / IE ───────┘                   └──► M-cycle cost
```

Host implementation details are not part of the contract. SoftGBC can use a
Rust `match`; QEMU can use helpers or TCG. Their boundary observations must
match.

## Status *(2026-11)*

| Area | SoftGBC | QEMU |
|---|---|---|
| Base + CB ISA | Tested | Open |
| Flags and conditional cycles | Tested | Open |
| EI/DI/HALT/interrupt order | Tested | Open |
| Trace representation | Planned | Open |

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Reset profiles are explicit**

  **What:** CPU construction and board reset are separate operations. Bare
  `Cpu::new` supplies the CPU defaults; `gbc-test` applies a named skip-boot
  profile (`PC=$0100`, `SP=$FFFE`, and any agreed `A`/`IF` values). Do not bury
  board policy inside QEMU’s generic CPU reset.

  **Structure:**
```text
CPU reset:
  deterministic architecture defaults

gbc-test reset profile:
  PC = $0100
  SP = $FFFE
  A  = agreed skip-boot value
  IF = agreed board value
```

  **Min test:**
```text
reset bare CPU → architecture defaults
reset gbc-test → PC=$0100, SP=$FFFE, exact documented A/IF
```

  **Done when:** SoftGBC and QEMU use the same named profile for every
  differential run; trace metadata records that profile.

---

- [ ] **Registers, pairs, and flag masking**

  **What:** The visible registers are `A F B C D E H L SP PC`. Pairs are
  `AF BC DE HL`; `F[3:0]` is always zero. All 16-bit arithmetic wraps, and
  writes through POP/GDB/reset obey the same `F` masking rule.

  **Structure:**
```text
AF = (A << 8) | (F & $F0)       F = Z N H C 0 0 0 0
BC = (B << 8) | C
DE = (D << 8) | E
HL = (H << 8) | L
SP, PC = unsigned 16-bit, wrapping
```

  **Min test:**
```text
write AF=$01BF through normal setter and debugger path
assert A=$01 and F=$B0
increment PC=$FFFF → PC=$0000
```

  **Done when:** all register mutation paths preserve widths and `F` masking,
  and one-step traces agree on register pairs.

---

- [ ] **Fetch, immediates, stack byte order, and wrapping**

  **What:** Opcode/immediate fetch increments 16-bit PC with wrapping.
  `fetch16` is little-endian. Stack operations decrement before writes on push
  and read before increment on pop, matching SoftGBC’s observable memory order.

  **Structure:**
```text
fetch8:   value = mem[PC]; PC = PC + 1
fetch16:  lo = fetch8(); hi = fetch8(); value = hi<<8 | lo

push16(v):
  SP--; mem[SP] = high(v)
  SP--; mem[SP] = low(v)

pop16:
  lo = mem[SP]; SP++
  hi = mem[SP]; SP++
```

  **Min test:**
```text
PC=$FFFF, bytes[$FFFF]=$34, bytes[$0000]=$12
fetch16 → $1234 and PC=$0001
push $BEEF from SP=$FFFE → [$FFFC]=$EF, [$FFFD]=$BE
```

  **Done when:** fetch and stack boundary vectors match SoftGBC, including
  ordered writes in the trace.

---

- [ ] **ALU flags and preserved flags**

  **What:** Z/N/H/C are instruction-specific; some operations preserve C or
  leave Z unchanged. QEMU must copy SoftGBC’s tested formulas, especially
  half-carry, signed SP additions, rotate-A instructions, BIT, and DAA.

  **Structure:**
```text
for each ALU family define:
  result width
  Z rule
  N rule
  H rule
  C rule
  flags preserved from pre-state

never infer all flags from the result alone
```

  **Min test:**
```text
shared vectors cover:
  $0F + $01 (half-carry)
  $FF + $01 (zero + carry)
  INC preserving C
  BIT preserving C
  DAA after add and subtract
  ADD SP,e8 positive and negative
```

  **Done when:** shared ALU vectors produce identical AF and cycles for helper
  and optimized TCG paths.

---

- [ ] **Conditional control flow and cycle selection**

  **What:** JR/JP/CALL/RET conditions change both PC and M-cycle cost.
  Translation must not bake in a path based on translation-time register
  values. The decision is made from runtime flags.

  **Structure:**
```text
condition = evaluate Z/C from runtime F
if taken:
  update PC / stack
  m = taken_cycles
else:
  keep sequential PC
  m = not_taken_cycles
```

  **Min test:**
```text
same JR NZ bytes, once with Z=0 and once with Z=1
assert different PC and the two SoftGBC cycle counts
```

  **Done when:** every conditional family has taken/not-taken vectors and
  matching trace events.

---

- [ ] **Instruction-boundary order**

  **What:** Interrupt recognition precedes fetch. If no interrupt event occurs,
  one opcode executes, then the delayed-EI state advances. This order is the
  central contract between SoftGBC `step` and QEMU’s execution loop.

  **Structure:**
```text
pending = IF & IE & $1F

if halted:
  if pending == 0:             halt-idle, 1 M
  else if IME == 0:            wake-only, 1 M
  else:                        wake + IRQ, 6 M
else if IME && pending:
                               IRQ, 5 M
else:
  fetch + execute one opcode
  advance EI delay
```

  **Min test:**
```text
PC=$0100 contains NOP; IME=1; IF=IE=1
one boundary → event=irq, PC=$0040, NOP not executed, M=5
```

  **Done when:** QEMU emits the same event kind, pre/post PC, and cycles as
  SoftGBC for all four branches above.

---

- [ ] **Delayed EI and DI cancellation**

  **What:** `EI` does not enable IME immediately. SoftGBC’s countdown is an
  implementation detail, but its visible rule is: the instruction after EI
  completes before interrupts can dispatch. `DI` clears IME and cancels a
  scheduled enable.

  **Structure:**
```text
EI boundary:      schedule enable
next instruction: execute with old IME; enable at its end
next boundary:    pending IRQ may dispatch

DI: IME=false; cancel schedule
```

  **Min test:**
```text
EI; NOP with IF&IE already pending
events = insn(EI), insn(NOP), irq

EI; DI; NOP with pending IRQ
assert no IRQ dispatch and no deferred enable
```

  **Done when:** both sequences match SoftGBC traces exactly.

---

- [ ] **Interrupt priority, acknowledge, stack, and cycles**

  **What:** Pending is `IF & IE & $1F`; the lowest set bit wins. Dispatch
  clears IME and only the selected IF bit, pushes the interrupted PC, and jumps
  to its vector.

  **Structure:**
```text
bit:      0       1       2       3       4
source: VBlank   STAT    Timer   Serial  Joypad
vector:  $0040   $0048   $0050   $0058   $0060

dispatch:
  choose lowest pending bit
  IME = false
  IF &= ~selected_bit
  push16(PC)
  PC = vector
```

  **Min test:**
```text
PC=$1234, SP=$FFFE, IF=IE=$05, IME=1
assert vector=$0040, SP=$FFFC
assert [$FFFC]=$34 and [$FFFD]=$12
assert IF timer bit remains set
assert M=5
```

  **Done when:** priority, ack, stack-write order, and 5/6 M-cycle variants
  match SoftGBC.

---

- [ ] **HALT and wake events**

  **What:** HALT suppresses opcode fetch while time advances. A masked-in
  pending interrupt wakes the CPU even with IME clear; in that case it does not
  acknowledge IF or dispatch. This contract follows current SoftGBC’s basic
  HALT model; any later HALT-bug refinement starts with a SoftGBC test.

  **Structure:**
```text
halted + no pending       → PC stable, halted=true,  event=halt-idle, 1 M
halted + pending + !IME   → PC stable, halted=false, event=halt-wake, 1 M
halted + pending + IME    → IRQ dispatch,            event=irq,       6 M
```

  **Min test:**
```text
IME=0, halted=true, IF=IE=$04
one boundary → halted=false, IF unchanged, PC unchanged, M=1
```

  **Done when:** all three boundary cases compare cleanly.

---

- [ ] **M-cycle accounting is data, not host timing**

  **What:** Every instruction/event produces the SoftGBC M-cycle count.
  `1 M = 4 T`. QEMU host time, translation-block count, and default icount are
  not substitutes for this value.

  **Structure:**
```text
event.m_cycles = opcode/event-specific value
cpu.total_m_cycles += event.m_cycles
board.tick_t_cycles(event.m_cycles * 4)
```

  **Min test:**
```text
NOP → 1 M
LD BC,d16 → 3 M
taken and untaken conditional → expected distinct M
IRQ → 5 M; HALT-to-IRQ → 6 M
```

  **Done when:** the opcode/event corpus has zero cycle mismatches and test
  devices advance only from this accounting path.

---

- [ ] **Undefined opcode and STOP policies are frozen**

  **What:** Do not guess undefined-opcode or STOP behaviour while porting.
  Record current SoftGBC behaviour as vectors. If hardware accuracy is later
  improved, update SoftGBC and this contract first.

  **Structure:**
```text
for every undefined primary opcode:
  expected event / PC / halted state / cycles

for STOP:
  expected bytes consumed / state / wake policy
```

  **Min test:**
```text
run one representative undefined opcode and STOP in both engines
assert same event, PC, state, and cycles
```

  **Done when:** no opcode falls through to an accidental host abort, and the
  agreed policies are represented in the shared corpus.
