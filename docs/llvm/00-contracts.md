# 00 — Target and compatibility contracts

Freeze the externally visible choices before writing TableGen or lowering code.
Changing these later invalidates objects, libraries, and generated games.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Target triple and CPU identity**

  **What:** Register architecture `sm83` and canonical triple
  `sm83-unknown-none`. The triple describes the CPU/freestanding environment;
  `gbc-rom32` and `gbc-mbc5` are SDK/linker profiles, not operating systems.

  **Structure:**
```text
architecture: sm83
vendor:       unknown
os:           none
environment:  empty

clang --target=sm83-unknown-none
llc   -mtriple=sm83-unknown-none
```

  **Min test:**
```text
LLVM Triple parses, normalizes, and prints sm83-unknown-none
llc/llvm-mc list the SM83 target
unknown -mcpu/-mattr receives a diagnostic
```

  **Done when:** LLVM, Clang, LLD, CMake checks, and SDK manifests use one
  spelling.

---

- [ ] **DataLayout and C type model**

  **What:** Define little-endian 16-bit pointers and byte-addressable,
  unaligned memory. Legal machine values are i8/i16; wider integers lower to
  pairs/helpers. C uses 16-bit `int` and 32-bit `long`.

  **Structure:**
```text
pointer / size_t / ptrdiff_t: 16 bits
char:       8 bits
short/int: 16 bits
long:      32 bits
long long: 64 bits (helper-lowered)
alignment: byte-safe; stack alignment frozen by ABI

DataLayout string:
  derive and test from these rules; do not copy another target blindly
```

  **Min test:**
```text
Clang sizeof/alignof compile-time assertions for every scalar/pointer type
LLVM IR global layouts have expected offsets and total sizes
pointer arithmetic wraps/diagnoses according to 16-bit address space
```

  **Done when:** one ABI test header and LLVM DataLayout test agree exactly.

---

- [ ] **Address spaces**

  **What:** Start with one 16-bit generic address space. Do not encode ROM,
  WRAM, VRAM, or bank numbers as LLVM address spaces until a demonstrated
  optimization requires it; banking is an SDK/runtime operation.

  **Structure:**
```text
addrspace(0): 16-bit CPU-visible address

banked ROM pointer:
  explicit SDK far-pointer struct / handle
  not a silently widened C pointer
```

  **Min test:**
```text
ordinary pointer lowers to i16
attempted unsupported address-space cast produces a clear diagnostic
```

  **Done when:** frontend, backend, and SDK far-pointer design do not disagree.

---

- [ ] **Object and ABI version note**

  **What:** Use ELF32 little-endian as the relocatable container even though
  addresses are 16-bit. Until an official ELF machine number exists, use one
  documented local provisional value and reject incompatible objects. Embed an
  ABI version note/flag.

  **Structure:**
```text
ELF class:       ELF32
data:            little-endian
e_machine:       documented provisional SM83 value
ABI version:     sm83-abi-v1
address fields:  validated to 16-bit where architecture requires
```

  **Min test:**
```text
llvm-readobj identifies SM83 object and ABI version
LLD rejects wrong e_machine / incompatible ABI version
```

  **Done when:** provisional allocation and migration policy are documented;
  objects cannot be mistaken for another architecture.

---

- [ ] **Feature and CPU policy**

  **What:** v1 has one CPU (`sm83`) and no invented ISA extensions. Hardware
  platform differences (DMG/CGB speed, MBC) are SDK profiles. Reserve feature
  names only when they change generated CPU instructions.

  **Structure:**
```text
-mcpu=sm83
-mattr: empty for v1

not CPU features:
  CGB color
  double-speed mode
  MBC1/MBC5
```

  **Min test:**
```text
-mcpu=sm83 succeeds
-mcpu=z80 and imaginary features fail
```

  **Done when:** generated code cannot accidentally depend on undocumented
  hardware through a CPU feature flag.

---

- [ ] **Supported-language envelope**

  **What:** Define the initial freestanding C subset. Support fixed-width
  integers, pointers, structs, functions, and static storage. Defer or helper-
  lower floating point, 64-bit arithmetic, division, varargs, atomics, TLS,
  exceptions, dynamic allocation, and hosted APIs.

  **Structure:**
```text
supported directly:
  i8/i16 arithmetic, control flow, pointers, structs, calls

supported through runtime:
  selected i32/i64 operations, shifts, multiply/divide as documented

unsupported v1:
  threads/TLS, exceptions/unwind, dynamic linking, hosted libc
```

  **Min test:**
```text
supported examples compile
each unsupported category has an intentional diagnostic or named helper path
```

  **Done when:** SDK docs can state capabilities without guessing backend
  behavior.

---

- [ ] **Oracle and test matrix**

  **What:** Assign a test authority to every layer. LLVM FileCheck proves shape,
  not CPU correctness; executable ROMs prove semantics.

  **Structure:**
```text
triple/types        → LLVM/Clang unit + lit
encoding            → MC byte tests
relocation/layout   → llvm-readobj + LLD map tests
codegen shape       → llc FileCheck
runtime semantics   → SoftGBC oracle + QEMU candidate
SDK behavior        → SoftGBC visual/input tests
```

  **Min test:**
```text
one smoke test exists in every row before later phases claim completion
```

  **Done when:** each checklist links to its focused gate and no layer relies
  only on end-to-end screenshots.
