# GBC game SDK — agent-first approach

Provide a small, deterministic C SDK that lets coding agents and humans create
new Game Boy Color games without hand-writing cartridge headers, startup
assembly, PPU timing loops, bank switches, or asset conversion.

**Refs:** [`../llvm/README.md`](../llvm/README.md) ·
[`../qemu/README.md`](../qemu/README.md) · SoftGBC
[`README.md`](../../SoftGBC/README.md) · Pan Docs in `docs/vendor/pandocs/`

## Product goal

```text
 game.toml + C source + PNG/TMX/WAV assets
                    │
                    ▼
               gbc build
     ┌──────────────┼────────────────┐
     ▼              ▼                ▼
 compile/link    convert assets   validate budgets
     └──────────────┼────────────────┘
                    ▼
          game.elf + game.gbc + map/report
                    │
          ┌─────────┼───────────┐
          ▼         ▼           ▼
       SoftGBC    QEMU       RTL / FPGA
```

The first SDK is:

- **C-first** with a stable C ABI usable by future Rust/Zig bindings;
- **GBC-native** (CGB color hardware), with DMG compatibility deferred;
- **freestanding and deterministic**, with no implicit heap or host services;
- **agent-friendly**, using one manifest, one build command, bounded APIs,
  machine-readable reports, and runnable examples.

## Agent-facing design rules

1. One canonical project layout and CLI; no copied linker flags.
2. Safe high-level APIs first; raw MMIO remains available but visibly low-level.
3. Asset declarations generate typed symbols—no magic numeric tile offsets.
4. Every API states interrupt/timing/banking constraints.
5. Build errors name the exceeded hardware budget and suggested remedy.
6. Templates include tests, not only a playable binary.
7. Examples are small composable recipes agents can search and modify.

## Layering

```text
game code
  │
  ├── gbc/game.h       loop, scene, time
  ├── gbc/gfx.h        backgrounds, window, sprites, palettes
  ├── gbc/input.h      buttons and edge events
  ├── gbc/audio.h      music/SFX queues
  ├── gbc/save.h       cartridge RAM persistence
  └── gbc/bank.h       explicit far assets/code
           │
           ▼
libgbc (portable policy + hardware drivers)
           │
           ▼
crt0 / interrupt stubs / compiler runtime / SM83 ABI
           │
           ▼
CGB hardware (SoftGBC first, then QEMU/RTL/FPGA)
```

## Profiles

| Profile | Purpose |
|---|---|
| `gbc-rom32-debug` | First bring-up: 32 KiB ROM, assertions, test-control |
| `gbc-rom32-release` | Small unbanked games |
| `gbc-mbc5-debug` | Banked games with checked handles/thunks |
| `gbc-mbc5-release` | Larger production ROMs |

ROM32 comes first. MBC5 is added only after banked linker/runtime tests pass.

## Proposed project layout

```text
my-game/
├── game.toml
├── src/
│   ├── main.c
│   └── scenes/
├── assets/
│   ├── sprites/
│   ├── tiles/
│   ├── maps/
│   └── audio/
├── tests/
│   ├── smoke.toml
│   └── input/
└── build/                 # generated; never edited
```

Generated headers/sources live under `build/generated/`. Agents edit the
manifest, source, assets, and tests only.

## Runtime model

```text
reset → SDK init → game_init()
                    │
             wait for frame/VBlank
                    │
        poll input → game_update() → enqueue rendering/audio
                    │
          VBlank ISR commits bounded queues
                    └────────────── repeat
```

The public game callback never depends on host frame rate. Rendering APIs enqueue
work and report overflow instead of writing unsafe amounts during VBlank.

## Checklist pages

| Page | Deliverable |
|---|---|
| [`00-project-cli.md`](00-project-cli.md) | Manifest, template, CLI, reports |
| [`01-runtime.md`](01-runtime.md) | crt integration, interrupts, memory, timing |
| [`02-graphics-assets.md`](02-graphics-assets.md) | Typed assets, VRAM, tilemaps, sprites, palettes |
| [`03-input-loop.md`](03-input-loop.md) | Deterministic loop, input, scenes, timers |
| [`04-audio-save-banking.md`](04-audio-save-banking.md) | Audio, SRAM saves, MBC5 handles/thunks |
| [`05-agent-workflow.md`](05-agent-workflow.md) | Agent contracts, recipes, diagnostics, tests |
| [`06-validation-examples.md`](06-validation-examples.md) | Examples, visual/input regression, release gate |

## SDK v1 exit gate

- [ ] `gbc new` creates a building, tested project outside the repo.
- [ ] `gbc build` emits deterministic `.elf`, `.gbc`, map, and budget report.
- [ ] A sample uses color backgrounds, sprites, input, and one sound effect.
- [ ] Scripted input reaches a stable visual checkpoint in SoftGBC.
- [ ] ROM32 overflow, VRAM/OAM overflow, and invalid assets produce actionable errors.
- [ ] Agents can create a small new game by editing only documented project files.
- [ ] The same release ROM boots in SoftGBC and the QEMU/RTL targets that model
  its used hardware.
