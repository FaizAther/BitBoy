# 04 — `gbc-test` machine and test devices

Build a deterministic, headless board around the SM83 CPU. It is deliberately
small but uses a Game Boy-shaped address map so existing CPU ROMs need minimal
adaptation.

**Refs:** [`README.md`](README.md) · SoftGBC
[`DmgBus`](../../SoftGBC/softgbc-core/src/bus.rs) · QEMU `hw/avr/` · repo
Chunk D result convention (`$FF80 == $2E`)

## v1 memory map

```text
$0000 ┌───────────────────────────────────────┐
      │ ROM $0000-$7FFF                      │  32 KiB, readonly
$8000 ├───────────────────────────────────────┤
      │ test RAM backing $8000-$FFFF         │
      │   selected addresses overlaid by MMIO│
$FF01 │ SB serial data                       │
$FF02 │ SC serial control                    │
$FF04 │ DIV/TIMA/TMA/TAC ($FF04-$FF07)       │
$FF0F │ IF                                   │
$FF7F │ TEST_CTRL (gbc-test only)            │
$FF80 │ HRAM/result byte (ordinary RAM)      │
$FFFF └ IE                                   │
```

`$FF7F` is a **test-machine extension**, not SM83 or DMG hardware. It must never
appear in a future game-compatible machine. MMIO overlays need explicit priority
over the RAM backing; `$FF80` must remain ordinary writable memory.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Machine type and object ownership**

  **What:** Register one headless machine, `gbc-test`, which owns one SM83 CPU,
  ROM/RAM regions, IRQ controller, serial stub, timer (when enabled), and
  test-control device. Device references are explicit; no target-global board
  singleton.

  **Structure:**
```text
GbcTestMachineState
  cpu
  rom
  ram
  irq_controller (IF/IE)
  serial
  timer
  test_control

machine init:
  create/realize devices
  map MemoryRegions
  connect IRQ/work notification
  load image
```

  **Min test:**
```text
-machine help lists gbc-test
construct + reset + destroy under QTest
two independent test processes do not share state
```

  **Done when:** the machine lifecycle is deterministic and `-nographic` needs
  no display/audio backend.

---

- [ ] **ROM and RAM mapping**

  **What:** Map a readonly 32 KiB ROM and RAM backing for the upper address
  space. Use explicit MMIO overlays for IO registers. ROM writes are ignored or
  rejected according to normal MemoryRegion semantics and remain unchanged.

  **Structure:**
```text
system memory:
  ROM @ $0000 size $8000 readonly
  RAM @ $8000 size $8000

overlay with higher priority:
  serial, timer, IF, TEST_CTRL, IE
```

  **Min test:**
```text
read known ROM byte
attempt CPU write to ROM; read original value
write/read $C000 and $FF80
write TEST_CTRL; assert $FF80 is unaffected
```

  **Done when:** all boundaries `$7FFF/$8000/$FF7E/$FF7F/$FF80/$FFFF` have
  focused mapping tests.

---

- [ ] **Flat image loading and validation**

  **What:** Accept raw `.gb`/binary images up to 32 KiB, load at `$0000`, fill
  unused ROM deterministically, and start at `$0100` using the named skip-boot
  reset profile. SoftGBC `.hex` fixture syntax is converted by the test harness;
  the board loader does not grow a second text parser.

  **Structure:**
```text
input file:
  size 1..$8000 → copy to ROM; fill remainder with agreed byte
  size >$8000   → fail with "banking unsupported by gbc-test v1"

reset:
  apply dmg-skip-boot-v1
  entry PC=$0100
```

  **Min test:**
```text
valid 32 KiB image loads and first fetch sees byte at $0100
oversized image fails before CPU runs
missing image fails with actionable path
repeat reset does not reload/mutate ROM
```

  **Done when:** SoftGBC and QEMU test commands use identical binary bytes and
  image SHA-256 appears in trace metadata.

---

- [ ] **Stable command line**

  **What:** Pick one documented image option and use it in tests. If `-bios` is
  used for v1, treat it as the cartridge/test image consistently; do not support
  several aliases with different reset semantics.

  **Structure:**
```text
qemu-system-sm83 \
  -M gbc-test \
  -cpu sm83-cpu \
  -bios test.gb \
  -nographic \
  <trace/stop options>
```

  **Min test:**
```text
command runs from a clean build tree
bad image path exits nonzero before guest execution
```

  **Done when:** all docs/tests invoke the same option and machine.

---

- [ ] **Test-control / debug-exit register at `$FF7F`**

  **What:** A byte write to `TEST_CTRL` terminates the run deterministically.
  Guest value `0` means pass; nonzero is a guest failure code. The host process
  must distinguish guest failure from QEMU launch/configuration failure and
  preserve the guest value in diagnostics.

  **Structure:**
```text
read $FF7F:
  return documented identification/default value

write $FF7F = status:
  record status
  request orderly QEMU shutdown

host result:
  status 0   → success
  status >0  → guest-failure report + nonzero host exit
```

  **Min test:**
```text
ROM writes 0 → process succeeds and reports guest status 0
ROM writes 7 → process fails and reports guest status 7
write adjacent $FF80 → no shutdown
```

  **Done when:** CI can distinguish pass, guest fail, timeout, and QEMU startup
  error without scraping unrelated logs.

---

- [ ] **Result memory at `$FF80`**

  **What:** `$FF80` is ordinary HRAM-compatible test RAM. The Chunk D smoke
  writes `$2E` there before debug-exit. The board must offer a QTest/monitor
  path to inspect it even if the guest subsequently exits.

  **Structure:**
```text
guest:
  compute 20 + 22
  write $2E → [$FF80]
  write 0   → [$FF7F]

test:
  observe/store final $FF80
```

  **Min test:**
```text
run smoke ROM → final [$FF80]=$2E and guest status=0
```

  **Done when:** the same ROM and expectation run under SoftGBC and QEMU.

---

- [ ] **Blargg-compatible serial stub**

  **What:** Model `$FF01` (SB) and the minimal `$FF02` (SC) transfer behaviour
  used by SoftGBC’s Blargg runner. A completed transfer appends SB to a bounded
  host buffer and requests serial IF bit 3 according to the agreed coarse model.

  **Structure:**
```text
write SB → latch byte
write SC with start pattern:
  append SB to serial buffer
  complete transfer using deterministic cycle policy
  clear/complete SC state
  request IF bit 3 when model requires

runner watches bounded buffer for "Passed" / "Failed"
```

  **Min test:**
```text
tiny ROM emits "OK" through SB/SC → host captures exactly "OK"
buffer limit prevents unbounded guest output
```

  **Done when:** Blargg pass/fail markers use the same observable convention as
  SoftGBC.

---

- [ ] **IF/IE and timer subdevices**

  **What:** Integrate the device interfaces from
  [`03-interrupts.md`](03-interrupts.md). They are board devices driven by CPU
  cycle events, not opcode-specific special cases in the translator.

  **Structure:**
```text
CPU event M-cycles
       │ ×4
       ▼
timer.tick(T-cycles) ──request──► IF controller ──wake──► CPU
```

  **Min test:**
```text
QTest register access
focused timer overflow → IF bit 2
CPU HALT wake from device request
```

  **Done when:** device tests pass without running a full ROM, then interrupt
  ROM tests pass end-to-end.

---

- [ ] **Explicit v1 exclusions**

  **What:** Reject or clearly document unsupported facilities rather than
  creating misleading stubs: banked ROMs, boot ROM, PPU, APU, DMA, joypad, and
  game display. A later `dmg` machine can reuse the CPU after parity.

  **Structure:**
```text
gbc-test:
  CPU validation only
  no promise of commercial-ROM compatibility
```

  **Min test:**
```text
oversized/banked image receives an explicit unsupported error
```

  **Done when:** passing `gbc-test` is never described as “QEMU runs Game Boy
  games.”
