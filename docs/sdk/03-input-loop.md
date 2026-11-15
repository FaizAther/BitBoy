# 03 — Deterministic game loop, input, scenes, and timers

Give games a simple update model that replays identically under scripted input.
The SDK owns hardware polling and frame pacing; game code owns state changes.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Canonical callbacks and loop**

  **What:** Define `game_init` and `game_update` as the minimal application
  contract. Runtime waits for a frame boundary, snapshots input, advances timers,
  calls update once, then leaves queued hardware work for VBlank.

  **Structure:**
```c
void game_init(void);
void game_update(const gbc_frame_t *frame);
```
```text
wait frame → snapshot input/time → update timers → game_update → repeat
```

  **Min test:**
```text
after N VBlanks, update called exactly N times
same input/frame stream produces identical game-state hash
```

  **Done when:** template games contain no custom busy-wait main loop.

---

- [ ] **Joypad snapshot and edge API**

  **What:** Poll hardware once per frame into held/pressed/released bitsets.
  Public code reads the immutable snapshot, preventing order-dependent repeated
  MMIO reads.

  **Structure:**
```text
current = poll()
pressed  = current & ~previous
released = previous & ~current
held     = current
previous = current

buttons: right left up down A B select start
```

  **Min test:**
```text
script press/hold/release A across frames
assert exact held/pressed/released sequence
simultaneous direction/button combinations preserved
```

  **Done when:** SoftGBC input scripts and physical input feed the same snapshot.

---

- [ ] **Input repeat and chords**

  **What:** Optional repeat is a pure frame-based helper with configurable delay
  and rate. Chords operate on the same snapshot. Defaults never synthesize
  events silently.

  **Structure:**
```text
repeat(button, delay_frames, rate_frames)
chord(mask) = (held & mask) == mask
```

  **Min test:**
```text
hold direction for exact delay/rate sequence including counter wrap
```

  **Done when:** menu movement is reproducible under tests.

---

- [ ] **Frame timers and scheduler**

  **What:** Provide fixed-capacity one-shot/periodic timers measured in frames.
  Callbacks run in the main update context, not ISR context. Cancellation and
  wrap behavior are defined.

  **Structure:**
```text
timer slots:
  deadline, period, callback/event id, active

per frame:
  collect due timers
  enqueue/call in stable slot order
```

  **Min test:**
```text
one-shot, periodic, cancel, capacity overflow, frame-counter wrap
```

  **Done when:** games need no ad hoc modulo checks for ordinary timers.

---

- [ ] **Scene/state-machine helper**

  **What:** Offer an optional allocation-free scene interface with enter/update/
  exit callbacks and explicit transition requests. A transition occurs at a
  defined point after update so callbacks cannot recurse unexpectedly.

  **Structure:**
```text
scene:
  enter(context)
  update(context, frame) → optional next scene
  exit(context)

transition:
  current.exit → next.enter → next frame updates next
```

  **Min test:**
```text
title → play → pause → play transition order
transition requested during enter is rejected/deferred by policy
```

  **Done when:** sample multi-screen game has no global callback spaghetti.

---

- [ ] **Fixed-point math helpers**

  **What:** Provide explicit, tested fixed-point types/functions for movement,
  interpolation, and camera values. Avoid hidden floating-point compiler helpers
  in game loops.

  **Structure:**
```text
named formats:
  gbc_fx8_8_t (or selected v1 format)

operations:
  from_int, to_int, add/sub, multiply with defined rounding/saturation policy
```

  **Min test:**
```text
positive/negative conversion, multiply rounding, overflow policy
compare against host reference vectors
```

  **Done when:** template movement compiles without float/double symbols.

---

- [ ] **Deterministic RNG**

  **What:** Supply a small explicit-state PRNG for gameplay. Seed is provided by
  the game/test; the SDK does not mix wall clock or unrecorded entropy.

  **Structure:**
```text
gbc_rng_t state
gbc_rng_seed(&rng, seed)
gbc_rng_next(&rng)
gbc_rng_range(&rng, bound) with unbiased/defined method
```

  **Min test:**
```text
known seed produces frozen sequence
range handles bound edge cases
```

  **Done when:** replay metadata can reproduce random gameplay exactly.

---

- [ ] **Replay input format**

  **What:** Define a compact, versioned frame→button stream used by SDK tests
  and play agents. Include ROM hash, seed, and initial profile.

  **Structure:**
```text
metadata: schema, ROM SHA-256, seed, reset profile
runs:     frame count + button mask (RLE allowed)
checkpoints: optional name/frame/state or frame CRC
```

  **Min test:**
```text
record/play 120-frame script
replay twice → identical checkpoint state/frame hashes
```

  **Done when:** bugs can be attached as small replay files rather than prose.
