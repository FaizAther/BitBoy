# 01 — `sm83-softmmu` target skeleton

Create the smallest correctly-shaped QEMU guest target: registered CPU type,
translator loop, 16-bit address space, and one executable NOP. The temporary
board comes from [`04-board.md`](04-board.md).

**Refs:** [`README.md`](README.md) · QEMU `target/avr/`, `target/rx/`,
`configs/targets/avr-softmmu.mak` · SoftGBC
[`01_nop.hex`](../../SoftGBC/sm83-iss/fixtures/01_nop.hex)

## Intended source shape

```text
qemu/
├── configs/targets/sm83-softmmu.mak
├── configs/devices/sm83-softmmu/default.mak
├── target/sm83/
│   ├── cpu.c          cpu.h          cpu-qom.h
│   ├── cpu-param.h    translate.c    helper.c / helper.h
│   ├── insn.decode    disas.c        gdbstub.c
│   ├── Kconfig        meson.build
│   └── gdb-xml/sm83-cpu.xml
└── hw/sm83/           # created with minimal board checklist
```

Do not copy AVR files wholesale and rename symbols. Copy their QEMU integration
shape, then define SM83 state and behaviour from SoftGBC.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Build and Kconfig registration**

  **What:** Add a softmmu-only `sm83` target and a board configuration. QEMU
  needs target configuration, target source sets, hardware source sets, and
  Kconfig wiring to agree on the architecture name.

  **Structure:**
```text
configs/targets/sm83-softmmu.mak:
  TARGET_ARCH=sm83
  TARGET_LONG_BITS=32
  TARGET_XML_FILES=sm83-cpu.xml

target/sm83/meson.build:
  sm83_ss            → target_arch['sm83']
  sm83_system_ss     → target_system_arch['sm83'] as needed

hw/sm83/meson.build + hw/sm83/Kconfig:
  register gbc-test machine
```

  **Min test:**
```text
fresh build directory
../configure --target-list=sm83-softmmu
ninja qemu-system-sm83
./qemu-system-sm83 --version
./qemu-system-sm83 -machine help contains gbc-test
```

  **Done when:** a clean target-only build succeeds without enabling AVR/RX as a
  hidden dependency.

---

- [ ] **QOM CPU type and architecture state**

  **What:** Define one CPU model (`sm83-cpu` initially), QOM type plumbing, and
  `CPUArchState`. Keep architectural state distinct from derived translator
  temporaries and board reset policy.

  **Structure:**
```text
SM83CPU
  CPUState parent
  CPUArchState env

CPUArchState
  a f b c d e h l : u8
  sp pc            : u16
  ime halted       : bool
  ei_delay         : u8
  mcycles_total    : u64       # accounting/debug, not guest register
```
```text
CPUClass hooks:
  reset_hold
  set_pc
  get_pc
  has_work
  dump_state
  tcg_ops / translator entry
```

  **Min test:**
```text
-cpu help lists sm83-cpu
QOM construction + reset does not crash
dump_state prints deterministic zero/default state
```

  **Done when:** CPU can be realized independently and all state widths/masking
  rules are covered by focused C unit/QTest assertions.

---

- [ ] **CPU versus machine reset**

  **What:** Generic CPU reset initializes architecture-owned state. The
  `gbc-test` machine then applies the named SoftGBC-compatible skip-boot profile.
  This prevents every future machine from inheriting `$0100` accidentally.

  **Structure:**
```text
cpu reset:
  deterministic architecture defaults

machine reset:
  apply dmg-skip-boot-v1
  set PC=$0100, SP=$FFFE, agreed A/IF
```

  **Min test:**
```text
unit reset CPU alone → bare defaults
QTest reset gbc-test → exact skip-boot profile
repeat reset → same state and cleared cycle counter
```

  **Done when:** reset ownership matches [`semantics.md`](semantics.md) and
  repeated machine reset is deterministic.

---

- [ ] **Address-space parameters and identity MMU**

  **What:** SM83 uses a 16-bit byte address space with no architectural MMU.
  Use the QEMU page-size pattern proven by the nearby small targets (prefer the
  AVR-compatible 1 KiB page unless this QEMU version requires otherwise) and
  mask architectural addresses to 16 bits.

  **Structure:**
```text
cpu-param.h:
  TARGET_PAGE_BITS = 10
  target address-space widths cover $0000..$FFFF

tlb_fill / address translation:
  virtual == physical
  permissions come from MemoryRegion mapping
```

  **Min test:**
```text
fetch at $0100 reads loader byte
architectural PC increment from $FFFF wraps to $0000
store to RAM at $C000 then load returns same byte
```

  **Done when:** code fetch and data access traverse normal QEMU address-space
  APIs and no host pointer is used as guest memory.

---

- [ ] **Translator framework shell**

  **What:** Implement QEMU’s translator callbacks and global TCG mappings for
  architectural fields. For bring-up, translate exactly one guest instruction
  per TB so every return is a SoftGBC-style boundary.

  **Structure:**
```text
DisasContext:
  base
  pc_first / pc_next
  decoded opcode bytes
  boundary reason

tb_start:
  initialize context from env PC

translate_insn:
  fetch/decode one opcode
  emit runtime helper or direct NOP operations
  force TB exit

tb_stop:
  synchronize PC/state
  return to main loop
```

  **Min test:**
```text
ROM has NOP at $0100
execute one TB
assert PC=$0101 and event cost=1 M
assert TB contains one guest instruction
```

  **Done when:** NOP runs through generated code, not a host-side ad hoc loop,
  and execution returns cleanly to QEMU after the instruction.

---

- [ ] **Defined failure for unsupported opcodes**

  **What:** During the ladder, unsupported opcodes must stop with a guest error
  containing PC/opcode. They must not become NOPs or trigger an unexplained host
  assertion. Replace this temporary path as each group lands.

  **Structure:**
```text
unsupported(op, pc):
  log guest error with bytes and PC
  request controlled stop / defined exception
```

  **Min test:**
```text
ROM begins with an opcode not yet implemented
process stops deterministically and diagnostic names PC=$0100 + opcode
```

  **Done when:** no decoder fallthrough silently mutates state.

---

- [ ] **Minimal disassembly and state dump**

  **What:** Debug output must identify NOP, raw unknown bytes, PC, register pairs,
  IME, and HALT. This is diagnostic only; parity uses canonical JSONL.

  **Structure:**
```text
disassembler:
  $00 → "nop"
  otherwise → ".byte $xx" until implemented

dump_state:
  AF BC DE HL SP PC IME HALT M-cycles
```

  **Min test:**
```text
-d in_asm shows "nop" at $0100
-d cpu (or monitor state dump) shows PC advancing to $0101
```

  **Done when:** a failed NOP smoke can be diagnosed without attaching GDB.
