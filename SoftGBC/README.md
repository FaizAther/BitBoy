# SoftGBC

Software Game Boy (DMG-class) in Rust — build the **easiest wins first**, then layer hardware until a real game runs.

```text
.gb ROM
   ↓
 SoftGBC/soc          (softgbc binary)
   ├── softgbc-core   (Bus / MemoryBus / hex+ROM load)
   ├── peripherals    (timer, serial/joypad/dma stubs)
   ├── sm83-iss       (CPU crate)
   ├── dmg-ppu        (PPU crate — stub)
   ├── cartridge, …   (other folders / docs)
   ↓
 framebuffer / audio / input
```

Detailed SoC wiring order: [`soc/README.md`](soc/README.md).  
This file is the **whole SoftGBC roadmap** — sorted by effort, not by “complete hardware purity.”

---

## Current status *(2026-11)*

| Area | State |
|---|---|
| Cargo workspace | **Done** — members: `soc`, `softgbc-core`, `peripherals`, `sm83-iss`, `dmg-ppu`, `cartridge` |
| `softgbc` CLI | **Done** — generic headless runner: `cargo run -p soc -- <rom.gb> [max_steps]` |
| Bus / memory | **Done** — `DmgBus`: ROM/WRAM/HRAM/VRAM/OAM/IO/`IE` + **Timer** + **Serial** |
| SoC machine | **Done** — `SoftGbc` (`cpu` + `bus` + `step`/`run`); Blargg is `soc/tests/` only |
| `sm83-iss` | **Done for ISA** — full base + `CB`, `EI`/`DI`/`HALT`, IRQ vectoring |
| `peripherals` | Timer + serial (capture); joypad/dma stubs |
| Blargg `cpu_instrs` (individuals) | **11/11 Passed** via `cargo test -p soc` |
| PPU / MBC / APU | **Not started** (`dmg-ppu` empty lib) |

**Immediate next step:** PPU LY → BG framebuffer (replace stub `LY=144` in `MachineConfig`).

**How to repro Blargg:**

```bash
cd SoftGBC
cargo run -p soc -- tests/roms/blargg/cpu_instrs/01-special.gb
cargo test -p soc --test blargg_cpu_instrs
```

Details per suite: [`tests/roms/README.md`](tests/roms/README.md).

---

## Master clock (SoC timebase)

On a real Game Boy, CPU, timer, PPU, APU, and DMA share **one** system clock. They are separate hardware blocks, but they advance in **lockstep** — not as OS threads.

SoftGBC mirrors that with an **instruction-driven master clock** inside `soc`:

```text
loop {
    m = cpu.step(&mut bus);   // M-cycles for this instruction
    t = m * 4;                // 1 M-cycle = 4 T-cycles
    timer.tick(t);            // next: ppu.tick(t); dma.tick(t); apu.tick(t);
}
```

| Do | Don’t |
|---|---|
| One SoC loop distributes cycles | A thread per peripheral |
| Separate `Timer` / `Ppu` types | Hide timer increments inside random opcodes |
| Pick M- or T-cycles and convert once | Mix units across devices |

Details: [`soc/README.md`](soc/README.md) (cycle discipline) · [`peripherals/src/Timer.README.md`](peripherals/src/Timer.README.md).

### T-cycles vs M-cycles

Game Boy docs use two related time units. SoftGBC must not mix them casually.

| Name | Also called | What it is |
|---|---|---|
| **T-cycle** | clock cycle, “dot” (PPU), Φ / clk | One tick of the **~4.19 MHz** system clock. Smallest useful time step for timer DIV, PPU dots, etc. |
| **M-cycle** | machine cycle | One **CPU memory / internal** step. Fixed relation: **1 M-cycle = 4 T-cycles**. |

Examples:

- `NOP` takes **1 M-cycle** = **4 T-cycles**
- A typical `LD r,n` takes **2 M-cycles** = **8 T-cycles**
- One PPU scanline is **456 T-cycles** (= 114 M-cycles)
- Timer `TAC=$05` increments TIMA every **16 T-cycles** (= 4 M-cycles)

**Why both exist:** opcode tables and Blargg CPU timing are usually quoted in **M-cycles** (how long an instruction takes). Timer rates, DIV’s internal counter, and PPU line timing are natural in **T-cycles** (the wire that actually toggles each clock).

**SoftGBC convention:**

- `sm83-iss::Cpu::step` returns **M-cycles** (instruction length).
- Convert **once** in `soc`: `t_cycles = m_cycles * 4`.
- Prefer ticking timer/PPU/APU in **T-cycles** (or document clearly if a device ticks in M).

```text
1 second ≈ 4_194_304 T-cycles ≈ 1_048_576 M-cycles
```

---

## Rule of thumb

| Prefer first | Defer |
|---|---|
| ROM-only carts | MBC5 / RTC |
| Skip boot (`PC = 0x0100`) | Real boot ROM |
| Headless CLI + prints/CRC | Window + audio |
| Stub unused IO as `0xFF` | Perfect open-bus |
| Mute APU | Channel envelopes |
| One test ROM at a time | Full commercial compatibility |

Every step should leave something you can **run or `cargo test`** before the next step.

---

## Layout

| Folder | Role | Status |
|---|---|---|
| [`softgbc-core/`](softgbc-core/) | Shared `Bus` trait, `MemoryBus`/`DmgBus`, hex + `.gb` load | **In use** |
| [`peripherals/`](peripherals/) | Timer + serial/joypad/dma (+ `*.README.md` next to sources) | **Timer wired**; others stub |
| [`sm83-iss/`](sm83-iss/) | SM83 CPU crate | **ISA done** |
| [`dmg-ppu/`](dmg-ppu/) | DMG PPU crate | Stub |
| [`soc/`](soc/) | `SoftGbc` machine + `softgbc` CLI; Blargg in `tests/` | **Generic headless** |
| [`cartridge/`](cartridge/) | Load `.gb`, header, later MBC | Stub (ROM load lives in `softgbc-core` for now) |
| [`interrupt/`](interrupt/) | `IF` / `IE` helpers | Checklist only; regs live on bus + CPU today |
| [`apu/`](apu/) | Sound | Stub |
| [`boot/`](boot/) | Optional boot ROM | Stub |
| [`tests/roms/`](tests/roms/) | Blargg (and later) test assets | Present |

Note: early docs mentioned separate `bus/` and `memory/` crates; those are in **`softgbc-core`**. Small IO devices + their docs live in **`peripherals`** (`Timer.README.md`, …).

---

## Step-by-step (easiest → harder)

### 1. Cargo workspace that builds *(~30 min)* — **DONE**

Lowest bar: nothing Game Boy–specific yet.

- [x] `SoftGBC/Cargo.toml` workspace
- [x] Members: `soc`, `sm83-iss`, `dmg-ppu`, `softgbc-core`, `peripherals`, `cartridge`
- [x] Binary `softgbc` (package `soc`) runs and reports usage / Blargg results

**Done when:** `cargo run -p soc -- --help` or a missing-ROM usage line works.  
*(Today the CLI expects `<rom.gb> [max_steps]`.)*

---

### 2. ROM file → header dump *(easy, high dopamine)* — **PARTIAL**

No CPU. Just parse bytes.

- [x] Read file into bytes / load into ROM window (`DmgBus::from_rom_file`)
- [ ] Dedicated header parse crate API (title `0x0134`, cart type `0x0147`, sizes) — `cartridge/` still stub
- [ ] `softgbc game.gb` prints header then exits (CLI currently runs the ROM as Blargg)

**Done when:** you recognize a real ROM from the CLI without executing it.

**Why first:** proves carts, CI assets, and “we have a product shape” with almost no ISA work.  
*(We skipped ahead via Blargg load; a header-only mode is still nice polish.)*

---

### 3. Flat memory + toy bus *(easy)* — **DONE (minimal)**

Still no opcodes.

- [x] 8KiB WRAM + HRAM arrays (`softgbc-core::memory`)
- [x] `read8`/`write8`: ROM `0000`–`7FFF`, WRAM, HRAM, IO/`IF`, `IE`
- [x] Unit tests in `softgbc-core` (WRAM round-trip, ROM load, `IF`/`IE`)

**Done when:** tests pass without a CPU.  
**Gaps:** VRAM/OAM/echo are SoC-stubbed for Blargg only; no open-bus doc; no DMA hooks.

---

### 4. Smallest CPU that moves *(medium-easy)* — **DONE**

Do **not** aim for Blargg yet — we already surpassed this milestone.

In `sm83-iss`:

- [x] Registers + `PC`/`SP`
- [x] `NOP` (`00`)
- [x] `JR` / `JP` (and the rest of control flow)
- [x] `LD` / ALU / stack / `CB`
- [x] SoftGBC `Bus` hooked into ISS (`softgbc_core::Bus`)
- [ ] `softgbc rom --steps 20` PC/opcode trace (CLI is Blargg-oriented; ISS unit tests cover stepping)

**Done when:** PC walks through NOPs or a tiny hand-assembled blob.  
Hex fixtures under `sm83-iss/fixtures/` remain for focused debugging.

---

### 5. Grow the CPU to “test ROM useful” *(medium)* — **NEARLY DONE**

- [x] Loads / stores / stack (`PUSH`/`POP`/`CALL`/`RET`)
- [x] Flags + ALU + `CB` prefix
- [x] `EI`/`DI`/`HALT` (basic) + IRQ service in CPU
- [x] Headless loop with Blargg serial pass/fail (`soc` Blargg runner)

**Done when:** Blargg `cpu_instrs` passes **in SoftGBC**.

| Individual ROM | Result |
|---|---|
| `01-special` … `11-op a,(hl)` except `02` | **Passed** |
| `02-interrupts` | **Failed** — needs timer IRQ |
| `cpu_instrs.gb` (combined 64 KiB) | **Not runnable** on 32 KiB ROM window (needs banking / larger ROM) |

---

### 6. Interrupts + timer *(medium-easy once CPU works)* — **IN PROGRESS** ← **you are here**

- [x] `IF`/`IE` on the bus (`DmgBus` / Blargg overlay)
- [x] CPU accepts IRQ vectors (`sm83-iss`)
- [x] Master-clock SoC loop ticks `peripherals::Timer` after each `cpu.step()`
- [x] Timer overflow sets `IF` bit 2 (coarse — see [`peripherals/src/Timer.README.md`](peripherals/src/Timer.README.md))
- [x] `02-interrupts.gb` **Passed**

**Done when:** `02-interrupts.gb` (or a Mooneye timer ROM) behaves. **Met for Blargg `02`.**

Skip serial IRQ and joypad IRQ until later. Model the timer as **separate hardware** advanced by the SoC, not a thread and not “inside” the CPU.

---

### 7. OAM DMA stub-to-real *(easy → medium)*

- [ ] Easy: on write `FF46`, instantly `memcpy` 160 bytes to OAM
- [ ] Later: spread across ~160 M-cycles + CPU access restrictions

**Done when:** OAM contains the copied bytes after a DMA trigger.

---

### 8. PPU “lights on” before perfection *(medium)*

Don’t start with sprites. `dmg-ppu` is still an empty crate.

1. **LY counter only** — tick 456 T-cycles → increment LY; VBlank IRQ at LY=144  
2. **BG framebuffer** — draw background tiles, ignore window/sprites  
3. **CRC / PPM dump** — `softgbc rom --frames 1 --dump frame.ppm`  
4. **Window**, then **sprites**, then `dmg-acid2`

*(Today `MachineConfig::stub_ly = Some(144)` so LCD waits exit — not a real PPU.)*

**Done when (early):** LY advances + VBlank fires + a non-black BG frame dumps.  
**Done when (later):** `dmg-acid2` / agreed visual tests.

---

### 9. MBC1 only when a ROM needs it *(easy–medium)*

Stay ROM-only until your target game (or combined Blargg) banks.

- [ ] Detect cart type from header
- [ ] MBC1 ROM banking (or enlarge ROM window for 64 KiB `cpu_instrs.gb`)
- [ ] SRAM + `.sav` only if that game needs saves

**Done when:** your chosen banked ROM reads the right banks.

---

### 10. Joypad + a window *(medium, very motivating)*

- [ ] `FF00` with keyboard → button bits
- [ ] Thin SDL/winit loop: run to VBlank → blit 160×144 → poll keys
- [ ] Keep `--headless` for CI

**Done when:** you can move in a simple homebrew or boot screen of a commercial game.

---

### 11. APU as silence, then noise *(easy stub → hard)*

- [ ] Easy: NR registers writable; audio callback outputs zeros
- [ ] Then channel 2 or 1 square only
- [ ] Then wave + noise + mixer

**Done when (early):** games boot without spinning on audio regs.  
**Done when (later):** music is recognizable.

---

### 12. Stubs & polish last *(easy individually)*

Only when something blocks a game:

- [x] `serial` — accept `SB`/`SC` writes for Blargg (in SoC Blargg bus)
- [ ] Move serial stub into `serial/` crate if desired
- [ ] `boot` — optional; skip-boot register init is usually enough
- [ ] Better DMA timing, STAT edge cases, PPU quirks
- [ ] MBC3/5, RTC, rumble — per-game

---

## Suggested weekend path

| Session | Ship | Status |
|---|---|---|
| 1 | Workspace + ROM load | **Done** |
| 2 | Bus/WRAM + CPU steps | **Done** |
| 3 | Expand opcodes toward Blargg | **Done** |
| 4 | Timer + IRQs; Blargg `02` + full individuals green | **Next** |
| 5 | Instant DMA + PPU LY/BG + PPM dump | Todo |
| 6 | Window + joypad; first playable ROM | Todo |
| 7+ | MBC as needed, sprites, APU, quirks | Todo |

---

## What *not* to do early

- Full cycle-accurate PPU FIFO before BG exists  
- QEMU / Verilog / FPGA before SoftGBC runs remaining CPU tests  
- Perfect SameBoy parity  
- CGB mode  
- Refactors of folder layout instead of a passing test  

---

## Component docs

[sm83-iss](sm83-iss/README.md) · [dmg-ppu](dmg-ppu/README.md) · [soc](soc/README.md) · [peripherals](peripherals/README.md) · [Timer](peripherals/src/Timer.README.md) · [Serial](peripherals/src/Serial.README.md) · [Joypad](peripherals/src/Joypad.README.md) · [Dma](peripherals/src/Dma.README.md) · [cartridge](cartridge/README.md) · [interrupt](interrupt/README.md) · [apu](apu/README.md) · [boot](boot/README.md) · [tests/roms](tests/roms/README.md)

Bus/memory notes live under [`softgbc-core/src/`](softgbc-core/src/) (`Bus.README.md`, `Memory.README.md`).  
SoC sequencing: **[`soc/README.md`](soc/README.md)**.

---

## Name note

**SoftGBC** = software machine under this folder (folder + binary name).  
Older docs said “SoftGba”; same project. v1 target is **DMG / SM83 Game Boy**, not the ARM Game Boy Advance.
