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

### Documentation map

| Kind | Where |
|---|---|
| **Item pattern** | What → Structure → Min test → Done when (pseudocode; softgbc-hands-off skill) |
| **Drivers** | Checklists in this file + crate READMEs (`dmg-ppu`, `Timer`, `Serial`, `interrupt`, …) |
| HTML / PDFs | Repo [`../docs/`](../docs/) — `cd ../docs && ./serve.sh` |

**Architecture in one line:** `SoftGbc::step` → `cpu.step` → `bus.tick(m*4)` → devices (`Timer` / later `Ppu`) request IRQ bits → bus ORs `IF`. No threads.

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
| PPU / MBC / APU | **PPU next** — drive [`dmg-ppu` checklist](dmg-ppu/README.md); MBC/APU later |

**Immediate next step:** PPU checklist — LY unit tests → wire SoftGBC (drop `stub_ly`) → BG.

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
| `01` … `11` (incl. `02-interrupts`) | **Passed** |
| `cpu_instrs.gb` (combined 64 KiB) | **Not runnable** on 32 KiB ROM window (needs banking / larger ROM) |

---

### 6. Interrupts + timer *(medium-easy once CPU works)* — **DONE for Blargg 02**

- [x] `IF`/`IE` on the bus (`DmgBus`)
- [x] CPU accepts IRQ vectors (`sm83-iss`)
- [x] Master-clock SoC loop ticks `peripherals::Timer` after each `cpu.step()`
- [x] Timer overflow sets `IF` bit 2 — see [`peripherals/src/Timer.README.md`](peripherals/src/Timer.README.md) checklist for accuracy follow-ups
- [x] `02-interrupts.gb` **Passed**

**Structure:** `Timer` owns regs + `tick(t)` → `IrqLines`; `DmgBus` ORs into `IF`; CPU services between instructions. Not a thread.

---

### 7. OAM DMA *(easy → medium)*

Detail: [`peripherals/src/Dma.README.md`](peripherals/src/Dma.README.md).

- [ ] **Instant DMA**

  **What:** write `FF46` copies 160 bytes → OAM.  
  **Structure:**
```text
src = FF46_value << 8
for i in 0..159: oam[i] = read(src + i)
```
  **Min test:**
```text
mem[C000]=0xAB → write FF46=0xC0 → read(FE00)==0xAB
```
  **Done when:** smoke test green.

- [ ] **Timed DMA (later)**  
  **What:** ~160 M-cycles + access rules.  
  **Done when:** see Dma README.

---

### 8. PPU “lights on” *(medium)* ← **you are here**

**Driver (same checklist):** [`dmg-ppu/README.md`](dmg-ppu/README.md).  
Scanline = **456 T**; frame = **154** lines (`0..143` draw, `144..153` VBlank). Today SoftGBC fakes LY with `stub_ly = 144`.

---

- [ ] **`tick(t)` advances LY**

  **What:** LY counts scanlines from time alone (no ROM).  
  **Structure:**
```text
Ppu: ly, line_cycles          # line_cycles 0..455

tick(n):
  line_cycles += n
  while line_cycles >= 456:
    line_cycles -= 456
    ly += 1
    if ly == 154: ly = 0
```
  **Min test:**
```text
tick(455) → ly == 0
tick(1)   → ly == 1
fixture: 0→0, 456→1, 912→2, 65664→144, 70224→0
```
  **Done when:** [`dmg-ppu/fixtures/01_ly_only.expected.txt`](dmg-ppu/fixtures/01_ly_only.expected.txt) green.

---

- [ ] **VBlank IRQ edge at LY=144**

  **What:** one request when LY *becomes* 144 (not every tick on 144).  
  **Structure:**
```text
when ly becomes 144: vblank_irq = true
take_vblank_irq(): return and clear flag
```
  **Min test:**
```text
tick(143*456) → take == false
tick(456) → ly==144, take==true once
```
  **Done when:** edge-only behaviour locked.

---

- [ ] **IO `FF40`–`FF4B` (LY = `FF44`)**

  **What:** LCD regs; `FF44` reads as LY.  
  **Structure:**
```text
read_io / write_io
FF44 read → ly
```
  **Min test:**
```text
write FF40 LCD-on → read back
read FF44 == ly
```
  **Done when:** LCDC round-trip; LY readable.

---

- [ ] **Modes 0–3 / STAT (optional early, needed before acid2)**

  **What:** per-line OAM search / draw / HBlank; VBlank is mode 1. STAT can request `IF` bit 1.  
  **Structure:**
```text
within a line (456 T): advance mode by cycle budget
take_stat_irq on enabled edges (mode / LYC)
```
  **Min test:**
```text
after N T into a visible line, mode matches expected
LYC==LY with STAT enable → IF bit 1
```
  **Done when:** mode + one STAT path tested; see [`interrupt/README.md`](interrupt/README.md).

---

- [ ] **Background → framebuffer + CRC**

  **What:** BG tiles → 160×144; golden hash.  
  **Structure:**
```text
SCX/SCY + LCDC → tile in VRAM → BGP → framebuffer
frame_crc = hash(framebuffer)
```
  **Min test:**
```text
one solid tile, BG on → one frame → frame_crc == GOLDEN
```
  **Done when:** golden checked in under `dmg-ppu/fixtures/`.

---

- [ ] **Window**

  **What:** WY/WX window layer over BG.  
  **Structure:** when active, fetch window map instead of BG.  
  **Min test:** known rect → pixel or CRC.  
  **Done when:** window golden matches.

---

- [ ] **Sprites**

  **What:** OAM objects composited with BG.  
  **Structure:** OAM scan → merge (priority / transparency).  
  **Min test:** one OBJ → expected non-BG pixel.  
  **Done when:** sprite golden or acid2 progress.

---

- [ ] **Wire SoftGBC / remove `stub_ly`**

  **What:** real LY on the bus; VBlank into `IF`.  
  **Structure:**
```text
DmgBus owns Ppu
tick(t): timer.tick; ppu.tick; if take_vblank: IF |= 0x01
FF44 → ppu.ly()
stub_ly = none
```
  **Min test:**
```text
SoftGbc steps → FF44 changes
IME+IE VBlank → PC can reach $0040
```
  **Done when:** LCD waits work without stub.

---

- [ ] **`dmg-acid2` / visual golden**

  **What:** last v1 graphics gate.  
  **Min test:** ROM/CRC matches agreed reference.  
  **Done when:** noted green in status table.

---

### 9. MBC1 when a ROM needs it

Detail: [`cartridge/README.md`](cartridge/README.md).

- [ ] **Parse header / cart type**  
  **What:** read `0x0147` etc.  
  **Min test:** known `.gb` → expected type/title.  
  **Done when:** parse API exists.

- [ ] **MBC1 banking (or larger ROM window)**  
  **Min test:** two banks differ after switch.  
  **Done when:** chosen banked ROM runs.

- [ ] **SRAM + `.sav` if needed**  
  **Done when:** save round-trips.

---

### 10. Joypad + window

Detail: [`Joypad.README.md`](peripherals/src/Joypad.README.md).

- [ ] **`FF00` from host keys**  
  **What:** active-low nibble for selected group.  
  **Min test:** press Down → correct bit low.  
  **Done when:** bus read matches host.

- [ ] **Window: VBlank → blit → poll**  
  **What:** host UI; SoftGbc stays headless-capable.  
  **Done when:** you see a frame and can press keys.

- [ ] **Keep headless CI**  
  **Done when:** Blargg still runs without a window.

---

### 11. APU (mute first)

Detail: [`apu/README.md`](apu/README.md).

- [ ] **Mute-safe NR stub**  
  **What:** `FF10`–`FF3F` writable; audio out = silence.  
  **Min test:** write `FF26`, read back; samples stay 0.  
  **Done when:** games don’t hang on audio init.

- [ ] **Square / wave / noise** — see apu checklist.

---

### 12. Stubs & polish last *(easy individually)*

- [x] Serial capture for Blargg (`SB`/`SC`) — see [`Serial.README.md`](peripherals/src/Serial.README.md) checklist for IRQ
- [ ] Serial transfer-complete → `IF` bit 3 (checklist on Serial README)
- [ ] `boot` — optional; skip-boot is usually enough
- [ ] Better DMA timing, STAT edge cases, PPU quirks
- [ ] MBC3/5, RTC, rumble — per-game

---

## Suggested weekend path

| Session | Ship | Status |
|---|---|---|
| 1 | Workspace + ROM load | **Done** |
| 2 | Bus/WRAM + CPU steps | **Done** |
| 3 | Expand opcodes toward Blargg | **Done** |
| 4 | Timer + IRQs; Blargg `02` + full individuals green | **Done** |
| 5 | PPU LY (checklist in `dmg-ppu`) + wire SoftGBC; then BG/PPM | **Next** |
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
