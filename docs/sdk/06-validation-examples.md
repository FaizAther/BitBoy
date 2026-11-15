# 06 — SDK examples, tests, and release gate

Examples are executable specifications. Each one introduces one capability,
contains a deterministic replay/checkpoint, and stays small enough to diagnose.

## Example ladder

```text
00 boot/color
   ↓
01 background + palette
   ↓
02 sprite + input
   ↓
03 animation + collision
   ↓
04 scene transition + map
   ↓
05 audio
   ↓
06 save/reload
   ↓
07 banked vertical slice
```

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Boot and runtime smoke**

  **What:** Smallest SDK game initializes, increments a frame counter, writes a
  known result/checkpoint, and exits in test profile.

  **Structure:**
```text
game_init → set marker
game_update frame N → test checkpoint/exit
```

  **Min test:**
```text
SoftGBC reaches frame N, expected marker, status 0
QEMU runs CPU/runtime subset where hardware coverage permits
```

  **Done when:** installed SDK template has a sub-second smoke.

---

- [ ] **Graphics example ladder**

  **What:** Separate ROMs introduce palette/tile background, attributes/window,
  sprite/OAM, and multi-frame upload. Each has a frame CRC and failure PNG.

  **Structure:**
```text
source assets → build → scripted frames → checkpoint image/CRC
```

  **Min test:**
```text
one checkpoint per graphics feature; no combined mega-demo as sole test
```

  **Done when:** every public graphics module links to a passing example.

---

- [ ] **Input and gameplay example**

  **What:** A tiny playable vertical slice consumes replay input, moves a sprite,
  collides with a tile map, changes state at a goal, and exposes a stable state
  hash/checkpoint.

  **Structure:**
```text
replay → held/pressed input → fixed-point movement → collision → goal
```

  **Min test:**
```text
script reaches goal in exact bounded frame range
alternate script remains blocked by collision
```

  **Done when:** example demonstrates recommended loop rather than raw MMIO.

---

- [ ] **Audio trace example**

  **What:** Play one song and interrupt it with prioritized SFX. Validate
  timestamped APU-register writes and queue state; optionally capture audio once
  SoftGBC synthesis is authoritative.

  **Structure:**
```text
replay/checkpoint → audio commands → expected register event stream
```

  **Min test:**
```text
stable event-stream hash and no queue overflow
```

  **Done when:** audio API behavior is testable without subjective listening.

---

- [ ] **Save and reset example**

  **What:** Modify progress, commit, reset emulator, reload, and verify state.
  Inject interrupted/corrupt writes to demonstrate recovery policy.

  **Structure:**
```text
run A → save → persist SRAM artifact
reset/run B → load → checkpoint
fault variants → recover/default
```

  **Min test:**
```text
valid round-trip plus newest-slot corruption recovery
```

  **Done when:** runner can manage SRAM artifacts reproducibly.

---

- [ ] **MBC5/banked example**

  **What:** Load scenes/assets from multiple ROM banks while common runtime and
  ISR remain fixed. Include one far-call only after thunk support is stable.

  **Structure:**
```text
scene 1 bank → transition → scene 2 bank
music/interrupt continues across switch
map report identifies placement
```

  **Min test:**
```text
replay visits both banks and verifies distinct visual/state hashes
```

  **Done when:** banked profile proves assets, code (if enabled), and interrupts
  coexist.

---

- [ ] **Cross-runner result schema**

  **What:** Standardize SDK test output so SoftGBC, QEMU, Verilator, and FPGA
  runners report comparable stop reason, status, checkpoints, frames, traces,
  and artifacts. Runners may mark unsupported visual/audio checks explicitly.

  **Structure:**
```json
{"schema":"gbc-test-result-v1","runner":"softgbc","rom_sha256":"...","status":"pass","stop_reason":"checkpoint","frames":120,"checkpoints":[]}
```

  **Min test:**
```text
schema validates pass/fail/timeout/unsupported cases
ROM hash mismatch fails before comparison
```

  **Done when:** CI dashboard does not infer results from console prose.

---

- [ ] **Hardware-budget stress tests**

  **What:** Exercise exact and over-capacity ROM, WRAM, HRAM, VRAM tiles,
  palettes, sprites, queues, save slots, and banks. Exact capacity succeeds;
  one-over fails safely with diagnostic code.

  **Structure:**
```text
for each bounded resource:
  capacity case
  capacity + 1 case
  release/debug behavior if different
```

  **Min test:**
```text
all public capacities represented in generated matrix
```

  **Done when:** reported limits are executable facts.

---

- [ ] **SDK release acceptance**

  **What:** Test an installed SDK/toolchain from outside the source tree:
  generate project, build all profiles/examples, run deterministic tests, and
  package docs/templates/libraries.

  **Structure:**
```text
fresh install prefix
  → gbc new acceptance-game
  → gbc build/test
  → build example matrix
  → verify hashes/reports/no source-tree paths
```

  **Min test:**
```text
ROM32 debug/release vertical slice passes
MBC5 cases required only when profile is declared stable
```

  **Done when:** a release artifact—not a developer checkout—is sufficient for
  an agent to create and verify a new GBC game.
