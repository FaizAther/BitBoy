# 04 — SM83 ABI, calling convention, and frames

Define one conservative v1 ABI that is easy to validate and consume from C.
Optimization can improve internal calls later only when explicitly marked and
never across separately compiled ABI boundaries.

## Proposed v1 summary

```text
stack:         grows downward; SP restored by callee on return
arguments:     stack, right-to-left, 16-bit slots, caller cleanup
return i8:     A
return i16:    HL
return wider:  runtime/sret policy
caller-saved:  A, F, HL
callee-saved:  BC, DE
special:       SP, PC
varargs:       unsupported v1
stack align:   2 bytes at public call boundary
```

At callee entry:

```text
SP+0  return address low
SP+1  return address high
SP+2  first argument slot low
SP+3  first argument slot high
SP+4  second argument slot low
...
```

This proposal becomes final only after the conformance tests below pass.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Calling-convention TableGen and assignment**

  **What:** Encode argument/return locations in LLVM calling-convention
  definitions and call-lowering code. Narrow arguments occupy a defined
  zero/sign-extended 16-bit slot according to IR attributes.

  **Structure:**
```text
caller:
  extend and push argument slots right-to-left
  CALL
  read A/HL return
  reclaim argument bytes

callee:
  return address remains at entry SP
  access arguments at fixed positive offsets
```

  **Min test:**
```text
separately compile caller/callee with 0,1,3 mixed i8/i16 arguments
inspect stack bytes and returned values
```

  **Done when:** LLVM lowering and written stack diagram match byte-for-byte.

---

- [ ] **Preserved-register mask**

  **What:** Freeze caller/callee-saved sets and model F/flag clobbers. Calls may
  destroy A/F/HL; callees that use BC/DE save and restore them.

  **Structure:**
```text
call regmask:
  preserved = BC, DE, SP
  clobbered = A, F, HL

PC changes by CALL/RET and is not allocatable
```

  **Min test:**
```text
caller keeps live values across call in BC/DE
callee deliberately uses BC/DE and restores exact values
caller does not assume flags survive
```

  **Done when:** register allocator, asm documentation, and conformance ROM use
  the same mask.

---

- [ ] **Prologue, epilogue, and stack objects**

  **What:** Save used callee-saved pairs, allocate fixed local storage, restore
  it, and return. SM83 lacks general SP-relative loads, so materialize addresses
  with `LD HL,SP+e8` and `(HL)` access. Diagnose or lower frames beyond immediate
  reach deliberately.

  **Structure:**
```text
prologue:
  PUSH used callee-saved pairs
  adjust SP for locals

access object:
  HL = SP + signed offset
  load/store via HL

epilogue:
  deallocate locals
  POP saved pairs in reverse
  RET
```

  **Min test:**
```text
leaf/no-frame function emits no unnecessary prologue
locals + nested call retain values
large frame follows documented expansion/diagnostic
```

  **Done when:** MachineFrameInfo tests and executable stack-canary tests pass.

---

- [ ] **Return and aggregate policy**

  **What:** Return i8 in A and i16/pointers in HL. Define booleans as canonical
  0/1 in A at ABI boundaries. Larger scalars/aggregates use a hidden sret
  pointer or named runtime helper policy; do not invent DEHL conventions ad hoc.

  **Structure:**
```text
i8 / bool → A
i16 / ptr → HL
aggregate → caller storage + hidden pointer argument
i32/i64   → freeze one policy before exposing publicly
```

  **Min test:**
```text
return signed/unsigned i8, i16, pointer, small struct
call from a separately assembled ABI probe
```

  **Done when:** Clang IR attributes and backend lowering agree for every public
  return category.

---

- [ ] **Tail calls, indirect calls, and varargs policy**

  **What:** Disable unsupported optimizations explicitly. v1 has no varargs.
  Tail calls require proven compatible stack layout. Indirect calls require a
  documented trampoline because SM83 has no native `CALL (HL)`.

  **Structure:**
```text
v1:
  varargs         → diagnostic
  arbitrary tail → disabled
  indirect call   → runtime trampoline or diagnostic
```

  **Min test:**
```text
front/backend tests confirm each feature is either correctly lowered or rejected
no generic indirect-call pseudo reaches MC
```

  **Done when:** agents cannot compile a construct that silently miscalls.

---

- [ ] **Interrupt-function ABI**

  **What:** Normal C functions are not interrupt handlers. SDK vector stubs save
  complete required state, establish normal ABI conditions, call a handler, then
  restore and `RETI`. If an `interrupt` attribute is later added, it must produce
  the same contract.

  **Structure:**
```text
vector stub:
  save AF, BC, DE, HL
  call SDK dispatcher/handler
  restore HL, DE, BC, AF
  RETI

handler:
  ordinary non-blocking C ABI function
```

  **Min test:**
```text
seed every register, trigger interrupt, handler changes memory
after RETI all seeded registers/SP restored and IME behavior correct
```

  **Done when:** interrupt conformance ROM passes SoftGBC/QEMU trace comparison.

---

- [ ] **Global and static data ABI**

  **What:** Define symbol naming, common/BSS handling, alignment, initializer
  layout, function/data pointer size, and section attributes used by the SDK.

  **Structure:**
```text
external symbol: stable unmangled C name
.data: ROM load image → WRAM at startup
.bss: zeroed at startup
.rodata/.text: ROM
custom bank/HRAM sections: SDK-defined attributes/macros
```

  **Min test:**
```text
two translation units share initialized/uninitialized globals and function calls
struct layout matches Clang compile-time assertions
```

  **Done when:** separately compiled libraries link without private assumptions.

---

- [ ] **ABI conformance package**

  **What:** Build assembly probes and C callees/callers as separate objects so
  tests cannot pass through whole-program coincidence. Version expected stack
  diagrams and register masks.

  **Structure:**
```text
asm caller → C callee
C caller   → asm callee
C caller   → C callee in separate object
interrupt stub → C handler
```

  **Min test:**
```text
all argument, return, preservation, frame, global, and interrupt cases
write numbered failure code to test-control interface
```

  **Done when:** ABI v1 is not declared stable until the package passes under
  SoftGBC and QEMU.
