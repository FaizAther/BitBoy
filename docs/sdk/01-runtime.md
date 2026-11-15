# 01 — GBC runtime, memory, interrupts, and time

Turn compiler startup into a safe game runtime. Hardware drivers remain
single-threaded and deterministic; interrupt handlers perform bounded work.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Startup and hardware baseline**

  **What:** SDK platform init runs after generic `.data/.bss` startup. Establish
  known LCD, palette, VRAM, OAM, timer, serial, joypad, speed, and interrupt
  state before calling the game.

  **Structure:**
```text
crt0 memory init
  → __gbc_platform_init
      disable interrupts
      establish CGB mode/profile
      initialize drivers/queues
      install vector dispatch
      enable selected interrupts
  → game_init
```

  **Min test:**
```text
seed dirty emulated hardware state, reset, assert documented baseline
game_init sees initialized runtime and zeroed queues
```

  **Done when:** games never rely on emulator power-on accidents.

---

- [ ] **Interrupt vector stubs and dispatcher**

  **What:** Assembly stubs obey the LLVM interrupt ABI, acknowledge/dispatch
  VBlank, STAT, timer, serial, and joypad sources, then `RETI`. Public handlers
  are internal SDK hooks; game code subscribes through bounded callbacks/events.

  **Structure:**
```text
vector → save state → identify source → SDK handler
                                  ├── set event bits
                                  ├── commit bounded queue
                                  └── update counters
        ← restore state ← RETI
```

  **Min test:**
```text
each source triggers once, preserves registers, and sets correct SDK event
multiple pending sources follow hardware priority
```

  **Done when:** interrupt conformance ROM passes SoftGBC trace comparison.

---

- [ ] **Critical sections**

  **What:** Provide nest-safe interrupt masking for short shared-state updates.
  Restore the prior IME intent rather than always enabling interrupts. Never
  expose a blocking mutex abstraction on a single-core interrupt runtime.

  **Structure:**
```text
token = gbc_critical_enter()
  record prior state; DI
update shared queue/index
gbc_critical_leave(token)
  restore prior policy with correct EI delay
```

  **Min test:**
```text
nested enter/leave from enabled and disabled starting states
no early re-enable; pending IRQ dispatches at documented boundary
```

  **Done when:** queue producers/consumers use one tested primitive.

---

- [ ] **Memory-region API**

  **What:** Expose typed sections/macros for WRAM, HRAM, ROM data, and later
  banked assets. Validate linker budgets. Do not imply arbitrary heap allocation
  in v1.

  **Structure:**
```text
default globals → WRAM
GBC_HRAM        → tiny latency-sensitive storage
const           → ROM
far asset       → explicit banked handle

v1 allocation:
  static storage
  caller-provided arenas/fixed pools
```

  **Min test:**
```text
symbol map places annotated values in expected regions
HRAM overflow fails at link time
```

  **Done when:** memory placement is visible in the build report and headers.

---

- [ ] **Frame and monotonic time**

  **What:** Maintain deterministic frame and tick counters from hardware events.
  Public time uses wrap-safe unsigned arithmetic and never reads host time.

  **Structure:**
```text
VBlank ISR:
  frame_counter++

API:
  gbc_frame_now()
  gbc_frames_elapsed(since)
  gbc_wait_vblank()
```

  **Min test:**
```text
counter increments once per VBlank
elapsed calculation works across wrap
wait returns on next event and not arbitrary polling count
```

  **Done when:** game updates replay identically from the same input script.

---

- [ ] **Bounded deferred-work queues**

  **What:** Game code enqueues VRAM/OAM/palette/audio work; VBlank/interrupt code
  consumes a known maximum. Overflow is reported in debug and returned to the
  caller, never silently corrupting adjacent state.

  **Structure:**
```text
main loop producer → fixed-capacity queue → VBlank consumer
                          │
                          └── full: error/status + build/runtime diagnostic
```

  **Min test:**
```text
fill to capacity succeeds
one extra enqueue fails without changing queue/storage
consumer preserves FIFO/priority policy
```

  **Done when:** every asynchronous SDK API states capacity and failure behavior.

---

- [ ] **Assertions, panic, and test exit**

  **What:** Debug builds convert assertion/panic into a stable code, source
  identifier where affordable, and SoftGBC/QEMU test-control stop. Release
  policy is explicit and cannot return into corrupted state.

  **Structure:**
```text
GBC_ASSERT(condition, code)
  false → disable interrupts → publish diagnostic → test exit / safe halt
```

  **Min test:**
```text
deliberate assertion produces expected code and deterministic stop reason
release behavior matches profile documentation
```

  **Done when:** agent tests report a named failure rather than timing out.
