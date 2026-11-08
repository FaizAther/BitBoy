# SoftGba

Software Game Boy (DMG-class) in Rust — build the **easiest wins first**, then layer hardware until a real game runs.

```text
.gb ROM
   ↓
 SoftGba/soc
   ├── sm83-iss      (CPU crate)
   ├── dmg-ppu       (PPU crate)
   ├── bus, memory, cartridge, …
   ↓
 framebuffer / audio / input
```

Detailed SoC wiring order: [`soc/README.md`](soc/README.md).  
This file is the **whole SoftGba roadmap** — sorted by effort, not by “complete hardware purity.”

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

| Folder | Role |
|---|---|
| [`sm83-iss/`](sm83-iss/) | SM83 CPU crate |
| [`dmg-ppu/`](dmg-ppu/) | DMG PPU crate |
| [`cartridge/`](cartridge/) | Load `.gb`, header, later MBC |
| [`bus/`](bus/) | Address decode |
| [`memory/`](memory/) | WRAM / HRAM / VRAM / OAM |
| [`interrupt/`](interrupt/) | `IF` / `IE` |
| [`timer/`](timer/) | DIV / TIMA / TMA / TAC |
| [`dma/`](dma/) | OAM DMA |
| [`joypad/`](joypad/) | `P1` |
| [`apu/`](apu/) | Sound |
| [`serial/`](serial/) | Link (stub early) |
| [`boot/`](boot/) | Optional boot ROM |
| [`soc/`](soc/) | Glue + `softgba` binary |

---

## Step-by-step (easiest → harder)

### 1. Cargo workspace that builds *(~30 min)*

Lowest bar: nothing Game Boy–specific yet.

- [ ] `SoftGba/Cargo.toml` workspace
- [ ] Members: `soc`, `sm83-iss`, `dmg-ppu` (empty libs OK)
- [ ] Binary `softgba` prints help / version

**Done when:** `cargo run -p softgba -- --help` works.

---

### 2. ROM file → header dump *(easy, high dopamine)*

No CPU. Just parse bytes.

- [ ] Read file into `Vec<u8>`
- [ ] Parse title (`0x0134`), cart type (`0x0147`), ROM/RAM sizes
- [ ] `softgba game.gb` prints that and exits

**Done when:** you recognize a real ROM from the CLI.

**Why first:** proves carts, CI assets, and “we have a product shape” with almost no ISA work.

---

### 3. Flat memory + toy bus *(easy)*

Still no opcodes.

- [ ] 8KiB WRAM + HRAM arrays
- [ ] `read8`/`write8` match: ROM mirror `0000`–`7FFF`, WRAM, HRAM
- [ ] Unit test: write `0xC000`, read it back; read ROM `[0x0100]`

**Done when:** tests pass without a CPU.

---

### 4. Smallest CPU that moves *(medium-easy)*

Do **not** aim for Blargg yet.

In `sm83-iss`, implement only what you need to see progress:

- [ ] Registers + `PC`/`SP`
- [ ] `NOP` (`00`)
- [ ] `JR imm8` and/or `JP nn` (so PC isn’t stuck)
- [ ] `LD r,n` / a couple ALU ops if you want
- [ ] SoftGba `Bus` hooked into ISS
- [ ] `softgba rom --steps 20` prints PC/opcode

**Done when:** PC walks through NOPs or a tiny hand-assembled blob.

**Parallel tip:** while one person does cart/bus, another can table-drive opcodes in `sm83-iss`.

---

### 5. Grow the CPU to “test ROM useful” *(medium — biggest early grind)*

Still SoftGba-only map (ROM-only, skip boot).

- [ ] Loads / stores / stack (`PUSH`/`POP`/`CALL`/`RET`)
- [ ] Flags + ALU + `CB` prefix
- [ ] `EI`/`DI`/`HALT` (basic)
- [ ] Headless loop with a clear pass/fail (Blargg serial or known HRAM result)

**Done when:** Blargg `cpu_instrs` passes **in SoftGba** (or ISS alone first, then SoftGba).

**Effort note:** this is the first “real” chunk of work. Everything before it was scaffolding so this isn’t painful.

---

### 6. Interrupts + timer *(medium-easy once CPU works)*

- [ ] `IF`/`IE` on the bus
- [ ] CPU accepts IRQ vectors
- [ ] Timer ticks from CPU cycle budget; overflow sets timer IF bit

**Done when:** a timer unit test or Mooneye timer ROM behaves.

Skip serial IRQ and joypad IRQ until later.

---

### 7. OAM DMA stub-to-real *(easy → medium)*

- [ ] Easy: on write `FF46`, instantly `memcpy` 160 bytes to OAM (good enough for many early tests)
- [ ] Later: spread across ~160 M-cycles + CPU access restrictions

**Done when:** OAM contains the copied bytes after a DMA trigger.

Instant DMA is intentional low-hanging fruit — cycle-accurate DMA can wait until a game breaks.

---

### 8. PPU “lights on” before perfection *(medium)*

Don’t start with sprites.

1. **LY counter only** — tick 456 T-cycles → increment LY; VBlank IRQ at LY=144  
2. **BG framebuffer** — draw background tiles, ignore window/sprites  
3. **CRC / PPM dump** — `softgba rom --frames 1 --dump frame.ppm`  
4. **Window**, then **sprites**, then `dmg-acid2`

**Done when (early):** LY advances + VBlank fires + a non-black BG frame dumps.

**Done when (later):** `dmg-acid2` / agreed visual tests.

---

### 9. MBC1 only when a ROM needs it *(easy–medium)*

Stay ROM-only until your target game banks.

- [ ] Detect cart type from header
- [ ] MBC1 ROM banking
- [ ] SRAM + `.sav` only if that game needs saves

**Done when:** your chosen banked ROM reads the right banks.

---

### 10. Joypad + a window *(medium, very motivating)*

- [ ] `FF00` with keyboard → button bits
- [ ] Thin SDL/winit loop: run to VBlank → blit 160×144 → poll keys
- [ ] Keep `--headless` for CI

**Done when:** you can move in a simple homebrew or boot screen of a commercial game.

This is the “it feels real” milestone — put it before APU.

---

### 11. APU as silence, then noise *(easy stub → hard)*

- [ ] Easy: NR registers writable; audio callback outputs zeros (games don’t hang on init)
- [ ] Then channel 2 or 1 square only
- [ ] Then wave + noise + mixer

**Done when (early):** games boot without spinning on audio regs.  
**Done when (later):** music is recognizable.

---

### 12. Stubs & polish last *(easy individually)*

Only when something blocks a game:

- [ ] `serial` — accept `SB`/`SC` writes so link waits don’t deadlock
- [ ] `boot` — optional; skip-boot register init is usually enough
- [ ] Better DMA timing, STAT edge cases, PPU quirks
- [ ] MBC3/5, RTC, rumble — per-game

---

## Suggested weekend path

If you want the shortest path to dopamine:

| Session | Ship |
|---|---|
| 1 | Workspace + ROM header CLI |
| 2 | Bus/WRAM + NOP CPU + `--steps` |
| 3 | Expand opcodes toward Blargg |
| 4 | Timer + IRQs; Blargg green |
| 5 | Instant DMA + PPU LY/BG + PPM dump |
| 6 | Window + joypad; first playable ROM |
| 7+ | MBC as needed, sprites, APU, quirks |

---

## What *not* to do early

- Full cycle-accurate PPU FIFO before BG exists  
- QEMU / Verilog / FPGA before SoftGba runs CPU tests  
- Perfect SameBoy parity  
- CGB mode  
- Refactors of folder layout instead of a passing test  

---

## Component docs

Per-folder checklists (use after you’re on the relevant step):

[sm83-iss](sm83-iss/README.md) · [dmg-ppu](dmg-ppu/README.md) · [soc](soc/README.md) · [cartridge](cartridge/README.md) · [bus](bus/README.md) · [memory](memory/README.md) · [interrupt](interrupt/README.md) · [timer](timer/README.md) · [dma](dma/README.md) · [joypad](joypad/README.md) · [apu](apu/README.md) · [serial](serial/README.md) · [boot](boot/README.md)

SoC-only sequencing (finer glue steps): **[`soc/README.md`](soc/README.md)**.

---

## Name note

**SoftGba** = software machine under this folder. v1 target is **DMG / SM83 Game Boy**, not the ARM Game Boy Advance.
