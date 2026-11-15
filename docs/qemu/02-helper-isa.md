# 02 — Helper-first ISA implementation

Implement the full SM83 ISA as runtime helpers with one instruction per
translation block. This is the correctness engine and remains available after
direct TCG optimization.

**Refs:** [`semantics.md`](semantics.md) · SoftGBC
[`sm83-iss/src/lib.rs`](../../SoftGBC/sm83-iss/src/lib.rs) ·
[`every_opcode.rs`](../../SoftGBC/sm83-iss/tests/every_opcode.rs) · QEMU
`target/avr/insn.decode`, `translate.c`, `helper.c`

## Execution shape

```text
 guest bytes       generated TCG                  runtime
 ───────────       ─────────────                  ───────
 3E 2A        →    gen_helper_exec(op=$3E, …) →  fetch immediate $2A
                   force TB exit                 A=$2A; PC+=2; M=2

 CB 00        →    gen_helper_exec_cb(cb=$00) →  RLC B; flags; PC+=2; M=2
                   force TB exit
```

Translation-time code selects what runtime code to emit. It must never mutate
`CPUArchState` as if the guest instruction had already executed.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Primary and CB decode coverage**

  **What:** Define deterministic coverage for all 256 primary bytes and all 256
  CB bytes. Two generated decode functions are preferable: primary decode sees
  one byte and the `$CB` entry fetches/dispatches the second byte. Group fields
  (`x/y/z`, register index, condition) where that reduces duplication without
  hiding SoftGBC cycle differences.

  **Structure:**
```text
decode_primary(op):
  $CB → fetch second code byte; decode_cb(cb)
  legal opcode → emit grouped/specific runtime helper
  undefined opcode → emit agreed undefined-op event

decode_cb(cb):
  x = cb >> 6
  y = (cb >> 3) & 7
  z = cb & 7
  dispatch rotate/shift, BIT, RES, SET
```

  **Min test:**
```text
decode $00 → NOP path
decode $CB $00 → RLC B path
decode $CB $46 → BIT 0,(HL) path
decode each undefined primary → explicit policy path
```

  **Done when:** an automated decoder-coverage test classifies all 512 slots
  with no accidental default.

---

- [ ] **Runtime fetch and memory access layer**

  **What:** Helpers use QEMU’s CPU data-access APIs so MemoryRegion permissions,
  MMIO, watchpoints, and exceptions remain observable. Immediates are fetched
  at runtime from guest memory; translated host code must not retain mutable
  data bytes as constants.

  **Structure:**
```text
read8(addr) / write8(addr, value):
  route through CPU address space
  architectural address = addr & $FFFF

fetch8:
  value = read8(PC); PC = (PC + 1) & $FFFF

fetch16:
  low = fetch8(); high = fetch8()
```

  **Min test:**
```text
LD A,d8 reads immediate at runtime
LD (HL),A reaches RAM and MMIO through the same access layer
write to readonly ROM leaves ROM unchanged
```

  **Done when:** no helper dereferences guest memory through a host pointer and
  QEMU watchpoints can observe a CPU store.

---

- [ ] **Register-index and pair helpers**

  **What:** Centralize indexed register access used by regular and CB opcodes.
  Index 6 means memory `(HL)`, not a register. Pair setters must apply AF masking
  and 16-bit wrapping.

  **Structure:**
```text
r index: 0=B 1=C 2=D 3=E 4=H 5=L 6=(HL) 7=A

read_r(6)     → read8(HL)
write_r(6,v)  → write8(HL,v)
set_pair(AF)  → F &= $F0
```

  **Min test:**
```text
same grouped LD operation with z=0 updates B
with z=6 performs one memory access and leaves B unchanged
POP AF masks F low nibble
```

  **Done when:** base and CB grouped tests cover all eight register indices.

---

- [ ] **Load and 16-bit arithmetic group**

  **What:** Land loads first: register/register, immediate, indirect, high-page,
  auto-increment/decrement HL, pair loads, SP/HL arithmetic. Preserve exact
  SoftGBC PC changes, write ordering, signed-extension rules, and cycles.

  **Structure:**
```text
LD r,r / LD r,d8 / LD rr,d16
LD (BC|DE|HL±|a16),A and reverse
LDH ($FF00+a8|C),A and reverse
LD (a16),SP
INC/DEC rr; ADD HL,rr
ADD SP,e8; LD HL,SP+e8; LD SP,HL
```

  **Min test:**
```text
one shared vector per addressing mode
SP signed-offset vectors: +1, -1, boundary carry/half-carry
LD (a16),SP asserts low-byte then high-byte writes
```

  **Done when:** load/16-bit vectors match pre/post state, writes, and M-cycles.

---

- [ ] **8-bit ALU and flag group**

  **What:** Port INC/DEC, ADD/ADC, SUB/SBC, AND/XOR/OR/CP, accumulator rotates,
  DAA, CPL, SCF, and CCF from SoftGBC. Use small internal C functions for flag
  formulas so the later TCG path can be compared against one helper oracle.

  **Structure:**
```text
operation helper:
  capture old operands / carry
  compute widened intermediate
  assign result when operation is not CP
  set exactly the flags defined by SoftGBC
  preserve required flags
```

  **Min test:**
```text
table-driven vectors include zero, half-carry, carry, borrow, preserved C
DAA vectors cover add/subtract and incoming H/C combinations
```

  **Done when:** the SoftGBC ALU unit-vector set runs against QEMU helpers with
  identical AF.

---

- [ ] **Control flow and stack group**

  **What:** Implement JR/JP/CALL/RET/RETI/RST/PUSH/POP with runtime condition
  evaluation. Every PC-changing instruction ends its TB. Taken/not-taken paths
  choose different cycle counts where SoftGBC does.

  **Structure:**
```text
evaluate condition from runtime F
if taken:
  perform stack/PC operation
  cycles = taken value
else:
  PC remains sequential
  cycles = untaken value

always exit TB after possible PC write
```

  **Min test:**
```text
JR cc both outcomes
CALL then RET restores exact PC/SP and ordered stack bytes
RST pushes sequential PC and jumps to correct vector
RETI restores PC and sets IME per SoftGBC
```

  **Done when:** all control-flow families pass both outcome vectors and no
  stale translated fall-through executes.

---

- [ ] **CB rotate/shift/bit group**

  **What:** Implement all 256 CB encodings through `x/y/z` grouping. `(HL)`
  variants perform the correct number/type of memory accesses and use their
  distinct cycle counts. BIT reads but does not write.

  **Structure:**
```text
x=0: RLC RRC RL RR SLA SRA SWAP SRL
x=1: BIT y,z
x=2: RES y,z
x=3: SET y,z

z=6 → operand is memory at HL
```

  **Min test:**
```text
RLC B; RL B with incoming carry; SRA preserves sign
BIT 0,(HL) produces no write
RES/SET (HL) produces one ordered write
```

  **Done when:** all 256 CB codes are covered and match SoftGBC state/writes/
  cycles.

---

- [ ] **HALT, STOP, DI, and EI instruction effects**

  **What:** Decode the instructions and implement their immediate state changes.
  Boundary scheduling, wake-up, and interrupt dispatch are completed in
  [`03-interrupts.md`](03-interrupts.md).

  **Structure:**
```text
DI:    IME=false; cancel delayed enable
EI:    schedule delayed enable
HALT:  halted=true
STOP:  apply frozen SoftGBC policy
```

  **Min test:**
```text
each instruction alone matches SoftGBC post-state, PC bytes consumed, and M
```

  **Done when:** instruction effects are green before asynchronous boundary
  tests begin.

---

- [ ] **Full helper-engine opcode gate**

  **What:** Run the shared single-step corpus across every primary and CB slot.
  The helper engine is the long-lived QEMU correctness baseline for later TCG
  optimization.

  **Structure:**
```text
for vector in shared corpus:
  initialize CPU + memory
  execute exactly one boundary
  compare kind, bytes, state, writes, cycles
```

  **Min test:**
```text
all legal slots execute
all undefined slots follow explicit policy
no host crash, timeout, or unclassified decoder entry
```

  **Done when:** the full corpus is green and saved as a required test target.
