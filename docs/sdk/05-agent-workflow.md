# 05 — Agent-oriented game creation workflow

Make the SDK legible and testable enough that an agent can create a game without
guessing hardware rules or editing generated/build-system internals.

## Agent contract

```text
agent may edit:
  game.toml, src/**, assets/**, tests/**, project README

agent must not edit:
  build/**, generated headers/data, installed SDK/toolchain, ROM bytes by hand

agent verifies:
  gbc build → gbc test → inspect report/artifacts
```

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Short machine-readable SDK index**

  **What:** Provide one index listing modules, headers, functions, constraints,
  examples, and maturity. Agents should not need to search the entire repo to
  discover the safe rendering or input API.

  **Structure:**
```text
SDK_INDEX.md / sdk-index.json:
  module
  header
  API names/signatures
  context: main / VBlank / ISR
  capacity/failure behavior
  example link
  status: stable/experimental
```

  **Min test:**
```text
index generator/check verifies every public header symbol is documented
links resolve
```

  **Done when:** public API discovery is complete and stale entries fail CI.

---

- [ ] **Task recipes**

  **What:** Add compact recipes for common game tasks: show a background, move a
  sprite, read buttons, animate, play SFX, change scene, save progress, and load
  a banked map. Each recipe names limits and a focused test.

  **Structure:**
```text
recipe:
  goal
  files to edit
  minimal API sequence
  hardware/budget constraints
  how to test
  link to complete example
```

  **Min test:**
```text
copy each recipe into template project and compile it in CI
```

  **Done when:** recipes are executable documentation, not stale snippets.

---

- [ ] **Explicit API context annotations**

  **What:** Every public function states whether it is safe in main context,
  VBlank callback, or ISR; whether it queues work; capacity; and return/error
  behavior. Encode useful annotations in generated docs/index.

  **Structure:**
```text
gbc_gfx_upload:
  context: main
  effect: queue
  capacity: N descriptors / B bytes per frame
  result: OK / QUEUE_FULL / BAD_HANDLE
```

  **Min test:**
```text
documentation checker rejects public declaration missing context/result metadata
```

  **Done when:** agents never infer interrupt safety from a function name.

---

- [ ] **Actionable diagnostics catalog**

  **What:** Assign stable diagnostic codes and remedies for manifest, asset,
  compile, link, hardware-budget, replay, and runtime failures. Text can improve;
  codes and structured fields remain stable.

  **Structure:**
```json
{"code":"GBC-VRAM-001","message":"tile allocation exceeds bank 1","requested":32,"available":12,"asset":"level_2","help":"split the tileset or unload another asset"}
```

  **Min test:**
```text
golden structured diagnostics for common failures
human text includes source asset/manifest location
```

  **Done when:** an agent can choose a repair from fields without regexing logs.

---

- [ ] **Plan-before-assets workflow**

  **What:** Encourage agents to declare game states, hardware budgets, controls,
  and asset inventory before generating art/code. `gbc inspect --plan` estimates
  limits from the manifest without requiring all source to compile.

  **Structure:**
```text
1. choose ROM profile
2. list scenes and transitions
3. declare controls
4. inventory tiles/maps/sprites/palettes/audio/save
5. inspect estimated budgets
6. implement one vertical slice
```

  **Min test:**
```text
manifest-only sample yields budget/unsupported-feature report
```

  **Done when:** project template and agent guide use this sequence.

---

- [ ] **Vertical-slice acceptance template**

  **What:** New games start with one playable loop: title/start, player input,
  one scene, collision/goal, and deterministic checkpoint. Expansion happens
  only after that test passes.

  **Structure:**
```text
input replay:
  start game
  move player
  trigger one interaction
  reach checkpoint

assert:
  state hash + frame CRC + no SDK warnings
```

  **Min test:**
```text
template vertical slice runs headlessly in bounded frames
```

  **Done when:** agent-generated examples demonstrate behavior before content
  volume.

---

- [ ] **No hidden generated-file edits**

  **What:** Generated files carry a header and are overwritten atomically.
  Build checks detect edits and direct agents to source manifest/assets. Stable
  generated symbol naming prevents unnecessary diffs.

  **Structure:**
```text
generated header:
  DO NOT EDIT
  source manifest path
  generator/schema version
  content hash
```

  **Min test:**
```text
modify generated file → build reports regeneration/source path
regenerate unchanged inputs → no diff
```

  **Done when:** repository review contains only intentional source/asset changes.

---

- [ ] **Failure artifact bundle**

  **What:** On test failure, emit a compact bundle: build report, ROM hash,
  replay, assertion/diagnostic, final state, trace window, frame image/CRC, and
  exact reproduction command.

  **Structure:**
```text
build/test-artifacts/FAILURE_NAME/
  result.json
  replay.*
  frame.png
  trace-window.jsonl
  reproduce.txt
```

  **Min test:**
```text
force visual/assertion failure
fresh process reproduces it using bundle command
```

  **Done when:** agents can debug without rerunning an unbounded play session.

---

- [ ] **Agent-created game evaluation**

  **What:** Maintain prompts/specifications for several small games and evaluate
  whether an agent can build them using public docs only. Score build success,
  test quality, budget compliance, and forbidden-file edits—not artistic taste.

  **Structure:**
```text
evaluation set:
  paddle/ball game
  tile puzzle
  top-down collect/exit game

metrics:
  clean build/test
  deterministic replay
  API correctness
  hardware-budget warnings
  source-only edits
```

  **Min test:**
```text
one clean-room agent run from installed SDK + docs
```

  **Done when:** SDK v1 is proven by creation of a new tested game, not only
  hand-maintained demos.
