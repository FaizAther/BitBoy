# 03 — TargetMachine, legalization, and instruction selection

Lower target-independent LLVM IR into legal SM83 machine instructions. Start
with correctness-oriented SelectionDAG/GlobalISel strategy appropriate to the
fork; do not maintain two selectors during bring-up.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **TargetMachine and Subtarget**

  **What:** Implement `SM83TargetMachine`, `SM83Subtarget`, data layout, pass
  configuration, object-file lowering, and target transform info. v1 has one
  CPU and no speculative hardware features.

  **Structure:**
```text
triple + -mcpu=sm83
  → TargetMachine
  → Subtarget
  → legalizer / selector / frame lowering / asm printer
```

  **Min test:**
```text
llc -mtriple=sm83-unknown-none empty.ll -o -
target data layout exactly matches 00-contracts
```

  **Done when:** empty and trivial modules compile through normal LLVM pipelines.

---

- [ ] **Legal type and operation policy**

  **What:** Legalize i8/i16 operations to SM83 registers/instructions. Promote
  i1 appropriately. Expand i32/i64, multiply/divide, variable shifts, and other
  unsupported operations into pairs or named runtime libcalls.

  **Structure:**
```text
legal:   core i8/i16 loads/stores/arithmetic/control flow
promote: i1 conditions to tested representation
expand:  wide operations
libcall: operations with stable runtime helper ABI
```

  **Min test:**
```text
one IR test per legalization action
no generic opcode survives after instruction selection
helper names/signatures match runtime docs
```

  **Done when:** supported IR cannot reach “cannot select” and unsupported IR
  has an intentional helper/diagnostic.

---

- [ ] **Loads, stores, constants, and addresses**

  **What:** Select register, immediate, absolute, HL-indirect, and high-page
  forms. Materialize 16-bit addresses correctly and let later combines choose
  compact forms only when semantics match.

  **Structure:**
```text
IR load/store
  → address analysis
  → legal SM83 addressing form
  → MC instruction

constants:
  i8 immediate
  i16 pair load
  symbol + relocation
```

  **Min test:**
```text
volatile store to $FF80 remains and selects valid sequence
global load emits relocation
adjacent byte accesses may use HL+/HL- only when safe
```

  **Done when:** memory tests execute correctly under SoftGBC, not only match
  assembly text.

---

- [ ] **Integer ALU and comparisons**

  **What:** Select i8 ALU directly and build i16/wider sequences with explicit
  carry/borrow. Comparisons must preserve/use flags correctly across intervening
  instructions.

  **Structure:**
```text
i8 add/sub/logical → native op
i16 add/sub        → low/high sequence + carry
icmp               → flag-producing compare + branch/materialization
select              → branch or safe sequence
```

  **Min test:**
```text
boundary vectors for signed/unsigned compare
i16 carry/borrow across bytes
flags are not reused after a clobbering instruction
```

  **Done when:** executable arithmetic corpus matches C/IR expected values.

---

- [ ] **Branches and control-flow lowering**

  **What:** Lower conditional/unconditional branches and switch. Use JR only
  when range is provably valid after layout or relaxation supports fallback;
  otherwise select JP. Preserve fallthrough and condition inversion correctly.

  **Structure:**
```text
MachineBasicBlocks
  → conditional branch + optional fallthrough jump
  → branch relaxation JR ↔ JP as required

switch:
  v1 branch chain; jump-table only after ABI/layout design
```

  **Min test:**
```text
near and far branch functions
condition inversion with both outcomes
large function does not produce out-of-range JR at assembly
```

  **Done when:** branch analysis/insert/remove hooks pass LLVM tests and linked
  large functions execute.

---

- [ ] **Calls, returns, and return values**

  **What:** Lower calls according to [`04-abi.md`](04-abi.md), including direct
  symbols, stack arguments, return registers, caller cleanup, and preserved
  registers. Indirect calls are deferred unless a tested sequence/thunk exists.

  **Structure:**
```text
call lowering:
  assign arguments
  emit stack/register copies
  CALL symbol
  model regmask and returned values
  restore caller stack
```

  **Min test:**
```text
zero/multiple args, i8/i16 returns, nested calls, recursive call
live callee-saved values survive
```

  **Done when:** ABI conformance functions compiled separately link and execute.

---

- [ ] **Pseudo-instructions and post-RA expansion**

  **What:** Use pseudos for operations that need context after register
  allocation (frame access, wide shifts, calls/thunks). Expand every pseudo
  before MC emission and preserve flags/liveness metadata.

  **Structure:**
```text
IR → target pseudo
   → register allocation
   → post-RA expansion
   → legal MC instructions only
```

  **Min test:**
```text
MIR test for each pseudo before/after expansion
final assembly contains no pseudo mnemonic
```

  **Done when:** pseudo expansion is deterministic and verifier-clean.

---

- [ ] **Instruction-selection regression ladder**

  **What:** Add one small IR test per operation before integrating large C
  examples. Pair FileCheck shape tests with executable ROM tests for semantics.

  **Structure:**
```text
llvm/test/CodeGen/SM83:
  legalize-*.ll
  load-store.ll
  alu.ll
  branch.ll
  call.ll

integration ROMs:
  same categories, observable result buffer
```

  **Min test:**
```text
sum(i8), sum(i16), if/else, loop, global load/store, function call
```

  **Done when:** each supported C construct has both lowering and execution
  coverage.
