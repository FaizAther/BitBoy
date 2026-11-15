# QEMU SM83 — integration approach (Chunk E)

Port the Game Boy **SM83** into the local QEMU fork (`GBA/qemu/`) as a
softmmu guest target. SoftGBC `sm83-iss` is the executable specification:
QEMU re-expresses its tested behaviour; it does not become a second authority.

This page fixes scope, ownership, sequencing, and architectural boundaries.
Sibling pages contain implementation checklists.

**Refs:** repo [`README.md`](../../README.md) Chunk E · SoftGBC
[`sm83-iss`](../../SoftGBC/sm83-iss/README.md) ·
[`semantics.md`](semantics.md) · QEMU `target/avr/` and `hw/avr/`

---

## Outcome

Chunk E delivers a `qemu-system-sm83` binary that:

- boots a flat Game Boy-style ROM at `$0100`;
- executes the full base and CB-prefixed ISA;
- matches SoftGBC registers, memory writes, interrupts, and M-cycle counts;
- exposes a deterministic test machine, trace stream, and GDB register model;
- runs hand-written and later LLVM-produced ROMs without requiring a PPU.

It does **not** deliver another complete Game Boy emulator. SoftGBC remains the
place for cartridge controllers, PPU, APU, input, and game compatibility.

```text
             executable specification                    implementation
  ┌──────────────────────────────────┐        ┌────────────────────────────┐
  │ SoftGBC sm83-iss                 │        │ qemu-system-sm83           │
  │                                  │        │                            │
  │ Cpu state + execute + IRQ order  │◄──────►│ target/sm83 CPU + TCG      │
  │ DmgBus for reference scenarios   │ traces │ hw/sm83 gbc-test machine  │
  │ unit tests + Blargg              │        │ GDB + debug-exit           │
  └──────────────────────────────────┘        └────────────────────────────┘
                     │                                      │
                     └──────────── tools/trace-diff ─────────┘
```

Use **SM83** for the CPU target. “LR35902” names the original DMG SoC and would
blur the CPU/board boundary this port deliberately preserves.

---

## Authority and change control

The oracle is not “whatever SoftGBC currently prints.” It is the combination of
SoftGBC code, focused tests, and the frozen trace contract.

| Situation | Required action |
|---|---|
| QEMU differs from tested SoftGBC | Fix QEMU |
| A hardware reference proves SoftGBC wrong | Add/fix SoftGBC test and implementation first; then update QEMU |
| Documentation differs from tested code | Correct documentation; do not silently encode the prose in QEMU |
| An intentional model difference is needed | Record it in `semantics.md` and exclude only the affected trace field/scenario |

Every semantic change follows:

```text
SoftGBC focused test
        ↓
SoftGBC implementation + trace
        ↓
QEMU implementation
        ↓
trace-diff + QEMU test
```

---

## Architectural boundaries

```text
 qemu-system-sm83
 ┌───────────────────────────────────────────────────────────────────────┐
 │ target/sm83/                         hw/sm83/                         │
 │ ┌───────────────────────────────┐    ┌─────────────────────────────┐ │
 │ │ CPUArchState                  │    │ gbc-test machine            │ │
 │ │ decoder / translator          │◄──►│ ROM + RAM MemoryRegions     │ │
 │ │ runtime helpers               │    │ IF / IE / serial stubs      │ │
 │ │ interrupt boundary handling   │    │ debug-exit device           │ │
 │ │ disassembler + GDB mapping    │    │ reset / image loading       │ │
 │ └───────────────────────────────┘    └─────────────────────────────┘ │
 └───────────────────────────────────────────────────────────────────────┘
```

| Concern | Owner |
|---|---|
| Registers, flags, decode, PC changes, IME/HALT state | `target/sm83/` |
| Guest physical memory map and MMIO | `hw/sm83/` |
| Canonical CPU behaviour | `SoftGBC/sm83-iss/` |
| Shared comparison format | `docs/qemu/00-harness.md` + trace tool |
| Full Game Boy device behaviour | SoftGBC, not Chunk E |

Start from QEMU’s AVR target for source layout and softmmu plumbing—not for ISA
semantics. SM83 is byte-addressed with a 16-bit address space and its own
interrupt model.

---

## Bring-up engine: one instruction per translation block

Correctness comes before translation density. The first engine should decode one
guest instruction, emit a call to a runtime helper that performs that
instruction, account its M-cycles, and end the translation block.

```text
translation time                        execution time
────────────────                        ──────────────
fetch opcode bytes
emit gen_helper_exec(op, cb, …)  ─────► helper mutates env + guest memory
emit TB exit                            helper records cycles / trace event
                                        return to QEMU loop
                                        check interrupt boundary
```

This is intentionally slow but gives exact instruction boundaries for:

- delayed `EI`;
- HALT wake-up;
- interrupt dispatch;
- one-record-per-step tracing;
- first-divergence debugging.

After parity is green, replace hot helper paths with direct TCG operations and
allow multiple instructions per translation block. Helpers remain valid for
complex operations such as DAA and boundary events.

Do not call SoftGBC Rust from QEMU. The two engines must remain independently
executable so differential testing can detect mistakes.

---

## Cycle model

SoftGBC’s contract is:

```text
CPU instruction result = M-cycles
1 M-cycle              = 4 T-cycles
peripherals advance in T-cycles
```

The first QEMU model keeps an explicit SM83 M-cycle count and advances test
devices at instruction boundaries. QEMU `icount` is useful for deterministic
scheduling, but its default “one translated instruction” unit must not be
mistaken for SM83 M-cycles.

Chunk E is **instruction-boundary accurate** with correct aggregate M-cycles.
T-cycle bus contention and PPU/DMA arbitration are out of scope.

---

## Test-machine scope

The initial machine is `gbc-test`, a deterministic harness rather than a
consumer Game Boy:

```text
$0000 ┌──────────────────────────────┐
      │ cartridge/test ROM          │  readonly
$8000 ├──────────────────────────────┤
      │ test RAM / simple stubs      │
$FF00 ├──────────────────────────────┤
      │ selected IO registers       │  serial, IF, test control
$FF80 ├──────────────────────────────┤
      │ HRAM (includes result byte)  │
$FFFF └──────────────────────────────┘  IE
```

The exact map and the fixed test-control address are owned by
[`04-board.md`](04-board.md). MMIO must not ambiguously overlap the result byte
at `$FF80`.

Deferred until after CPU parity:

- boot ROM execution;
- MBC banking;
- PPU, APU, joypad, DMA, and accurate timer edges;
- window/audio host integration;
- a game-compatible `dmg` machine.

---

## Work graph

```text
00 trace contract ───────────────┐
                                 ▼
01 target skeleton ───────► 04 minimal board
          │                      │
          └──────────┬───────────┘
                     ▼
              02 helper ISA
                     ▼
              03 IRQ / HALT / EI
                     ▼
              05 parity gates
                     ▼
              06 TCG + GDB + LLVM
```

| Page | Deliverable |
|---|---|
| [`semantics.md`](semantics.md) | Frozen SoftGBC-visible CPU contract |
| [`00-harness.md`](00-harness.md) | Valid JSONL schema, producers, and first-divergence comparison |
| [`01-skeleton.md`](01-skeleton.md) | Registered target, CPU shell, NOP execution |
| [`02-helper-isa.md`](02-helper-isa.md) | Full base/CB ISA through runtime helpers |
| [`03-interrupts.md`](03-interrupts.md) | Instruction-boundary interrupt, EI, and HALT behaviour |
| [`04-board.md`](04-board.md) | ROM loader, memory map, serial, IF/IE, debug-exit |
| [`05-parity.md`](05-parity.md) | Fixture, opcode, and Blargg gates |
| [`06-tcg-debug.md`](06-tcg-debug.md) | Direct TCG optimization, GDB, LLVM handoff |

Each open item uses **What → Structure → Min test → Done when**. Implement one
item at a time: failing focused test, minimum implementation, focused test
green, then proceed.

---

## Daily workflow

```text
choose one checklist item
  ↓
add the smallest SoftGBC/QEMU-facing test vector
  ↓
capture oracle trace from SoftGBC
  ↓
implement or optimize QEMU behaviour
  ↓
compare trace + final observable state
  ↓
commit only with the focused gate green
```

When traces diverge, stop at the first differing event. Reduce it to one
instruction/boundary scenario before changing either engine.

---

## Chunk E exit gate

- [ ] `qemu-system-sm83` builds from a clean `sm83-softmmu` configuration.
- [ ] All legal base and CB opcodes pass the agreed single-step corpus.
- [ ] EI, DI, HALT, interrupt priority, stack push, and cycle tests match SoftGBC.
- [ ] Fixture and agreed Blargg runs pass with no unexplained trace divergence.
- [ ] `$FF80 == $2E` and debug-exit success for the shared LLVM/hand ROM.
- [ ] GDB can inspect registers and single-step at `$0100`.
- [ ] No PPU/APU/MBC implementation is required to claim Chunk E complete.
