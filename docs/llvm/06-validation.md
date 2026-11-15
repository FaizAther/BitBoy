# 06 — Toolchain validation and optimization

Gate each compiler layer independently, then execute generated ROMs against the
SoftGBC oracle and QEMU. Optimization begins only after unoptimized correctness.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Lit test tiers**

  **What:** Organize fast tests by component so failures identify registration,
  MC, relocation, lowering, ABI, or driver defects immediately.

  **Structure:**
```text
llvm/test/MC/SM83
llvm/test/CodeGen/SM83
clang/test/CodeGen + Driver
lld/test/ELF/sm83-*
runtime unit/integration ROMs
```

  **Min test:**
```text
one positive and negative smoke in every directory
target-only test invocation documented
```

  **Done when:** each checklist phase has a focused command under one minute.

---

- [ ] **MC exhaustive generation**

  **What:** Generate legal instruction cases from one machine-readable opcode
  description/test source and verify assembler bytes plus disassembler
  round-trip. Do not maintain a second hand-copied semantic table.

  **Structure:**
```text
legal opcode cases
  → canonical assembly
  → llvm-mc bytes
  → llvm-objdump text
  → reassemble identical bytes
```

  **Min test:**
```text
all base/CB legal encodings covered
undefined primary bytes covered by negative/disassembly policy
```

  **Done when:** coverage report has no unclassified encoding.

---

- [ ] **ABI differential ROMs**

  **What:** Run the conformance package from [`04-abi.md`](04-abi.md) in
  SoftGBC and QEMU. Use numbered failure codes and trace comparison around calls.

  **Structure:**
```text
assembly probes + separately compiled C
  → link/pack
  → SoftGBC trace/result
  → QEMU trace/result
```

  **Min test:**
```text
arguments, returns, preserved regs, frames, globals, interrupt stub
```

  **Done when:** ABI v1 is marked stable only after all probes pass both runners.

---

- [ ] **C semantic corpus**

  **What:** Compile small functions exercising integer promotion, signedness,
  overflow assumptions, structs, pointers, loops, globals, and runtime helpers.
  Compare outputs to host-computed expected vectors where C behavior is defined.

  **Structure:**
```text
test ROM:
  input vector table
  call compiled function
  write result/failure index to result buffer
  debug-exit
```

  **Min test:**
```text
i8/i16 arithmetic, signed/unsigned compare, shifts, memcpy, struct field access
```

  **Done when:** supported C subset has executable coverage beyond FileCheck.

---

- [ ] **Optimization-level equivalence**

  **What:** Compile every executable test at `-O0`, `-O1`, `-O2`, and `-Os`.
  All levels must preserve result and ABI; assembly shape may differ.

  **Structure:**
```text
same source/input
  ├── O0 ROM
  ├── O1 ROM
  ├── O2 ROM
  └── Os ROM
compare result; trace only where deterministic shape is expected
```

  **Min test:**
```text
sum/loop/call/global/runtime-helper corpus at all levels
```

  **Done when:** no optimization level is advertised until its corpus passes.

---

- [ ] **Code-size and cycle baselines**

  **What:** Track section bytes and SoftGBC M-cycles for representative kernels.
  Baselines detect regressions but do not override correctness.

  **Structure:**
```text
benchmark record:
  compiler revision/options
  .text/.rodata size
  dynamic M-cycles for fixed input
```

  **Min test:**
```text
byte copy, checksum, tile update, sprite loop, fixed-point update
```

  **Done when:** optimization work cites measured size/cycle deltas.

---

- [ ] **Reproducible ROM build**

  **What:** Eliminate timestamps, unstable archive order, host paths, and
  nondeterministic padding from deployment outputs.

  **Structure:**
```text
same inputs + toolchain
  → same ELF loadable contents
  → same .gbc bytes / SHA-256
```

  **Min test:**
```text
build twice in different absolute directories → identical .gbc
```

  **Done when:** SDK templates can use ROM hashes as regression identifiers.

---

- [ ] **Sanitizer-style development checks**

  **What:** Full LLVM sanitizers are unrealistic initially, but debug runtime
  can check stack canaries, result-buffer bounds, bank handles, and selected
  assertions. Keep checks profile-controlled and deterministic.

  **Structure:**
```text
debug profile:
  assertion failure → numbered test-control exit
  stack canary checks
  SDK handle/range checks

release profile:
  checks removable by documented policy
```

  **Min test:**
```text
deliberate stack overwrite/assertion gives stable diagnostic code
```

  **Done when:** agent-created games fail with actionable errors instead of
  unexplained emulator hangs.

---

- [ ] **Toolchain acceptance gate**

  **What:** Run one clean installed-toolchain scenario from C source through ROM
  execution, independent of LLVM build-tree paths.

  **Structure:**
```text
install prefix
  → compile/link/pack sum game
  → validate header/map
  → SoftGBC
  → QEMU
  → compare $FF80/status
```

  **Min test:**
```text
sum(20,22) → $2E and status 0 on both runners
```

  **Done when:** this gate is green before the SDK claims a usable compiler.
