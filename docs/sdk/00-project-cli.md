# 00 — Project manifest, template, and CLI

Give agents one reproducible interface for creating, building, testing, and
inspecting a GBC game.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Versioned `game.toml` schema**

  **What:** Store human-authored game metadata, target profile, source roots,
  assets, and test defaults in one manifest. Unknown keys and invalid hardware
  combinations fail; defaults are explicit in `gbc inspect`.

  **Structure:**
```text
[game]
name, title, id, sdk_version

[target]
profile = "gbc-rom32-debug"

[build]
sources, include_dirs, defines

[[assets]]
name, type, source, options

[test]
runner, max_frames, input_script
```

  **Min test:**
```text
minimal valid manifest parses
unknown profile/key and overlong cartridge title report field location
normalized manifest output is stable
```

  **Done when:** schema has a version/migration policy and machine-readable
  diagnostics.

---

- [ ] **Canonical project generator**

  **What:** `gbc new NAME` creates the layout from `README.md`, a tiny color
  demo, one asset, and one smoke test. Generated source is intentionally small
  enough for an agent to understand in one pass.

  **Structure:**
```text
gbc new my-game
  → game.toml
  → src/main.c
  → assets/...
  → tests/smoke.toml
  → .gitignore
  → short project README
```

  **Min test:**
```text
generate in temporary directory
gbc build
gbc test
assert no path references the SDK source checkout
```

  **Done when:** a fresh project builds and tests without manual edits.

---

- [ ] **Single build command**

  **What:** `gbc build` resolves the installed LLVM toolchain/sysroot, converts
  assets, compiles, links, packs, validates, and writes all outputs under
  `build/`. It never edits source/assets.

  **Structure:**
```text
manifest
  ├── asset graph → generated headers/objects
  ├── source graph → SM83 objects
  └── profile → crt/libraries/linker script
                    ↓
           ELF + GBC + map + report
```

  **Min test:**
```text
clean build succeeds
no-op rebuild leaves outputs unchanged
asset/source change rebuilds only dependent steps
```

  **Done when:** verbose mode prints commands and normal mode gives concise
  actionable progress.

---

- [ ] **Machine-readable build report**

  **What:** Emit JSON plus a concise text summary for agents: ROM/WRAM/HRAM,
  bank, VRAM asset, palette, and estimated VBlank queue budgets. Include output
  hashes and tool versions.

  **Structure:**
```json
{"schema":"gbc-build-report-v1","rom":{"used":1234,"capacity":32768},"wram":{"used":512},"outputs":{"rom":"build/game.gbc","sha256":"..."},"warnings":[]}
```

  **Min test:**
```text
report validates against schema
deliberate ROM overflow names bytes used/capacity and largest sections
```

  **Done when:** an agent can decide what to shrink without parsing linker text.

---

- [ ] **Inspect and clean commands**

  **What:** `gbc inspect` explains resolved configuration, assets, memory map,
  symbols, and profile. `gbc clean` removes only generated output after proving
  the path is the project build directory.

  **Structure:**
```text
gbc inspect [--json]
gbc clean [--dry-run]
```

  **Min test:**
```text
inspect before/after build
clean dry-run lists only build/
refuse unsafe/symlinked output path outside project
```

  **Done when:** agents never need ad hoc deletion or linker-map scraping.

---

- [ ] **Run and test commands**

  **What:** `gbc run` launches the configured emulator with the built ROM;
  `gbc test` runs deterministic scripted tests headlessly and emits structured
  results/artifacts.

  **Structure:**
```text
gbc run --runner softgbc
gbc test [NAME] --runner softgbc --report json

later:
  --runner qemu | verilator | fpga
```

  **Min test:**
```text
template smoke test passes
missing runner gives installation guidance
timeout reports frame/event count and last checkpoint
```

  **Done when:** create→build→test→run is fully documented and scriptable.

---

- [ ] **Dependency/version locking**

  **What:** Record SDK/toolchain compatibility without fetching arbitrary code
  during a build. v1 uses installed SDK packages and exact/compatible versions;
  third-party libraries require explicit vendoring or a future lockfile design.

  **Structure:**
```text
game.toml sdk_version constraint
build report exact SDK/LLVM versions
offline build by default
```

  **Min test:**
```text
incompatible SDK version fails before conversion/compilation
offline template build succeeds
```

  **Done when:** agent-generated projects are reproducible and do not execute
  unreviewed network install scripts.
