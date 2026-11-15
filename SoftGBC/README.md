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
| PPU / MBC / APU | **PPU wired** (LY/BG/window/sprites unit); joypad/window host + MBC/APU later |

**Immediate next step:** joypad / host window blit, or `dmg-acid2` / visual golden.

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
| [`softgbc-utils/`](softgbc-utils/) | Shared helpers (`BitField` / `Reg8`, …) | **In use** |
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
Scanline = **456 T**; frame = **154** lines (`0..143` draw, `144..153` VBlank). SoftGBC uses real PPU LY (`stub_ly` removed).

---

- [x] **`tick(t)` advances LY**

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

- [x] **VBlank IRQ edge at LY=144**

  **What:** one request when LY *becomes* 144 (not every tick on 144).  
  **Structure:**
```text
when ly becomes 144: raise VBlank (IF bit 0) once
# edge-only — not level while ly stays 144
```
  **Min test:**
```text
tick(143*456) → no VBlank
tick(456) → ly==144, VBlank once
further ticks on 144 → no extra VBlank
```
  **Done when:** edge-only raise when LY becomes 144 (not every tick on 144).

---

- [x] **IO `FF40`–`FF4B` (LY = `FF44`)**

  **What:** CPU-visible LCD register window. Games write `LCDC` (`FF40`) to enable the PPU and read `LY` (`FF44`) to wait for scanlines. Other bytes plain storage until later items.  
  **Refs:** [Hardware regs](https://gbdev.io/pandocs/Hardware_Reg_List.html) · [LCDC](https://gbdev.io/pandocs/LCDC.html) · [STAT / LY](https://gbdev.io/pandocs/STAT.html) · full detail in [`dmg-ppu/README.md`](dmg-ppu/README.md)  
  **Structure:**
```text
FF40 LCDC R/W  FF41 STAT R/W  FF42 SCY  FF43 SCX
FF44 LY   R    (return ly; ignore writes for now)
FF45 LYC  R/W  FF46 DMA (store / bus later)
FF47 BGP  FF48 OBP0  FF49 OBP1  FF4A WY  FF4B WX

IoDevice on Ppu; bus routes FF40–FF4B
FF44 read → ly; FF44 write → no-op
```
  **Min test:**
```text
write_io(FF40, 0x80) → read_io(FF40) == 0x80
tick(456) → read_io(FF44) == 1 == ly
write_io(FF44, 0) → read_io(FF44) still 1
write_io(FF42, 0x55) → read_io(FF42) == 0x55
```
  **Done when:** LCDC round-trip; `FF44` == `ly`; `FF44` writes ignored; bus routes via `IoDevice`.

---

- [x] **Modes 0–3 / STAT**

  **What:** Visible line: Mode 2 → 3 → 0; `LY >= 144` → Mode 1. `STAT` shows mode + LYC flag; enabled edges raise `IF` bit 1.  
  **Refs:** [STAT](https://gbdev.io/pandocs/STAT.html) · explainer + diagrams: [`dmg-ppu` — PPU modes 0–3](dmg-ppu/README.md#ppu-modes-0-3)  
  **Structure:**
```text
v1 budgets: Mode2=80 T, Mode3=172 T, rest Mode0; LY>=144 → Mode1
STAT read: bits0–1=mode, bit2=(LY==LYC), bits3–6=selects from write
on mode enter / LYC edge if select enabled → STAT irq (edge once)
```
  **Min test:**
```text
T=0 → mode 2; after 80 T → mode 3; after +172 T → mode 0
tick(144*456) → mode 1
LYC=ly → STAT bit2 set; write STAT 0xFF → mode bits still live
STAT bit6 + LYC=1, tick(456) → STAT irq once (no repeat)
Mode2 select + enter Mode 2 on next line → STAT irq once
```
  **Done when:** mode readable via `FF41`; LYC + Mode-2 enter STAT edges tested.

---

- [x] **Background → framebuffer + CRC**

  **What:** Draw BG-only into a **160×144** buffer (viewport into 256×256 tilemap); map each pixel through tile data + **BGP**; CRC for goldens. No window/sprites yet.  
  **Refs:** [Tile Data](https://gbdev.io/pandocs/Tile_Data.html) · [Tile Maps](https://gbdev.io/pandocs/Tile_Maps.html) · [BGP](https://gbdev.io/pandocs/Palettes.html) · full detail in [`dmg-ppu/README.md`](dmg-ppu/README.md)  
  **Structure:**
```text
LCDC: bit7 LCD, bit4 tile ($8000 vs $8800), bit3 map ($9800/$9C00), bit0 BG on
for each screen pixel (sx,sy):
  (wx,wy) = (SCX+sx, SCY+sy) mod 256
  tile_id = map[(wy/8)*32 + (wx/8)]
  color_id = 2bpp from tile row (fine_y), bit (7-fine_x)
  framebuffer[sy*160+sx] = BGP_shade(color_id)
frame_crc = crc32(framebuffer)   # softgbc_utils::crc32
# DmgPpu::render_bg(vram) / frame_crc()
```
  **Min test:**
```text
LCDC=0x91, BGP=0xE4, SCX=SCY=0
tile0 = solid color_id 3; map uses tile 0
render_bg → frame_crc == 0x511BEFBF (fixtures/02_solid_bg)
BG off → blank/white frame
SCX=8 shifts two-tone image by one tile
```
  **Done when:** `02_solid_bg` golden checked in under `dmg-ppu/fixtures/` and green.

---

- [x] **Window**

  **What:** Non-scrolling overlay; screen origin `(WX-7, WY)`. `LCDC.5` enable, `LCDC.6` map.  
  **Refs:** [Window](https://gbdev.io/pandocs/Window.html) · [`dmg-ppu` checklist](dmg-ppu/README.md) · [`03_window_corner`](dmg-ppu/fixtures/03_window_corner.expected.txt)  
  **Structure:**
```text
if win on and sy>=WY and sx>=(WX-7): fetch win map at (sx-(WX-7), sy-WY)
else: BG path
```
  **Min test:**
```text
LCDC=0xF1, WY=72, WX=87 → window at (80,72); crc 0xAB9E8A54
win off → BG only
```
  **Done when:** `03_window_corner` golden green.

---

- [x] **Sprites**

  **What:** OAM OBJs (≤10/line) over BG/window; colour 0 transparent; OBP0/1; priority bit.  
  **Refs:** [OAM](https://gbdev.io/pandocs/OAM.html) · [`04_sprite_one`](dmg-ppu/fixtures/04_sprite_one.expected.txt) · [`dmg-ppu` checklist](dmg-ppu/README.md)  
  **Structure:**
```text
render_frame(vram, oam)
scan ≤10 OBJs by Y; per pixel lowest-X wins; composite vs BGP colour id
```
  **Min test:**
```text
one OBJ at (40,40) → crc 0x6222800E; transparent + priority smokes
```
  **Done when:** `04_sprite_one` golden green.

---

- [x] **Wire SoftGBC / remove `stub_ly`**

  **What:** Real PPU on `DmgBus`; `step` ticks PPU into `IF`; no LY stub.  
  **Structure:**
```text
tick_t_cycles: IF |= timer.tick | ppu.tick
FF44 → ppu.ly; MachineConfig has no stub_ly
```
  **Min test:**
```text
114 NOP steps → FF44 == 1
VBlank tick + IME+IE → PC == $0040
Blargg cpu_instrs green
```
  **Done when:** soc SoftGbc unit tests + Blargg pass without stub.

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
