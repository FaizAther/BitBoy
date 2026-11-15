# dmg-ppu

DMG Picture Processing Unit model — LY, later pixels, VBlank/STAT signals.

**Refs:** [GBEDG — PPU](https://github.com/Ashiepaws/GBEDG/blob/master/ppu/index.md) · fixture [`fixtures/01_ly_only.expected.txt`](fixtures/01_ly_only.expected.txt)  
**Roadmap:** SoftGBC [step 8](../README.md) (same checklist) · pattern in softgbc-hands-off skill

```text
unit-test Ppu alone first (no ROM)
     ▲
DmgBus owns Ppu later
SoftGbc.step → tick(t) → IF |= VBlank
```

## Status *(2026-11)*

| Item | State |
|---|---|
| Crate stub | Yes |
| LY / VBlank edge | **Done** |
| LCD IO `FF40`–`FF4B` | **Done** |
| Modes / STAT | **Done** (v1 fixed Mode 3 length) |
| Background → framebuffer | **Done** (`02_solid_bg`) |
| Window | **Done** (`03_window_corner`) |
| Sprites | **Done** (`04_sprite_one`) |
| SoftGBC wiring | **Done** (no `stub_ly`; LY/VBlank/STAT via bus) |

Scanline = **456 T**. Frame = **154** lines (`0..143` visible, `144..153` VBlank).

## PPU modes 0–3

The PPU paints **one scanline at a time**. Over the **456 T-cycles** of a line it is always in one of four **modes**. Games read the mode from **STAT bits 0–1** (`$FF41`).  
`LY` (`$FF44`) answers *which line*; **mode** answers *what stage of drawing* the PPU is in on that line right now.

This section also covers how the PPU notifies the CPU via **STAT** and the **STAT interrupt** (`IF` bit 1).

### Background tiles vs sprites (don’t mix these up)

| | **Background / Window** | **Sprites (OBJ)** |
|---|---|---|
| Layout | **32×32** tilemap → **256×256** pixels (`256 / 8 = 32`) | Up to **40** free-floating objects in OAM |
| Size | Each map cell is one **8×8** tile | **8×8**, or **8×16** if `LCDC` bit 2 is set (global) |
| Placement | Scroll viewport with `SCX`/`SCY` (BG) or `WX`/`WY` (window) | Any pixel position; **max 10 per scanline** |
| Tile data | Same 2bpp VRAM format (16 bytes / tile) | Same tile format; Mode 2 picks which OBJs apply this line |

Mode 2 / OAM search is about **sprites**, not the 32×32 BG grid.

### One frame (side view)

```text
        LY
         │
       0 ├─████████████████████████████████████████──┐
       1 ├─████████████████████████████████████████──┤
         │  … 144 visible lines (Mode 2→3→0 each) …  │  ← picture on LCD
     143 ├─████████████████████████████████████████──┘
     144 ├──────────────────────────────────────────┐
         │  … LY 144..153 = Mode 1 (VBlank) …       │  ← frame rest;
     153 ├──────────────────────────────────────────┘     upload tiles / move sprites
         ▼
      next frame (LY wraps to 0)
```

### One visible scanline (456 T) — SoftGBC v1 budgets

While `LY` is 0–143, every visible scanline takes **456 T-cycles** and steps through three modes in order:

1. **Mode 2 — OAM Search** (`0..79` / **80 T**): short-list sprites for this line (see below).
2. **Mode 3 — Drawing** (`80..251` / **172 T**): fetch tiles, mix BG + sprites, push pixels to the LCD.  
   *(Real HW Mode 3 stretches with sprites/scroll; SoftGBC v1 uses a fixed 172.)*
3. **Mode 0 — HBlank** (`252..455` / **204 T**): line finished; wait out the rest of the 456-dot budget. Games often swap palettes / scroll mid-frame here.

While `LY` is 144–153:

4. **Mode 1 — VBlank** (all 456 dots × 10 lines): visible frame done; safe window to update VRAM/OAM.

```text
 line_cycles →  0         80              252                    455
                │← Mode 2 →│←── Mode 3 ──→│←──── Mode 0 ────→│
                │  80 T    │    172 T     │   204 T (rest)   │
                │ OAM      │  draw pixels │  HBlank          │
                │ search   │  (BG+OBJ)    │                  │
```

ASCII “beam” on one line:

```text
  LCD pixels for this LY:  [ 0 ………………… 159 ]
                              ▲
                              │ Mode 3 is when the beam is writing these
  before that: Mode 2 (pick sprites)   after: Mode 0 (HBlank / rest)
```

```text
  Mode 2          Mode 3              Mode 0
  "who's on      "paint the          "line done —
   this line?"     line"               catch breath"
       │               │                    │
       ▼               ▼                    ▼
  ┌─────────┐    ┌───────────┐       ┌──────────┐
  │  OAM    │ →  │  pixels   │  →    │  HBlank  │ → next LY Mode 2
  │  scan   │    │  out      │       │          │
  └─────────┘    └───────────┘       └──────────┘

  Mode 1 (after LY 143):
  ┌──────────────────────────────────────────────┐
  │  VBlank — big safe window for the next frame │
  └──────────────────────────────────────────────┘
```

### Mode 2 (OAM Search) in depth

**Mode 2 is preparation only** — it does **not** draw pixels, and it does **not** show “left-over” graphics from the previous frame. It figures out *which* sprites belong on the **current scanline that Mode 3 is about to draw**.

#### Why Mode 2 exists

OAM can hold up to **40 sprites**, anywhere on screen. The PPU draws **one horizontal line at a time** (160 pixels) and does not have enough internal hardware to check all 40 sprites while also pushing pixels to the LCD.

So for the first **80 T-cycles** of every visible scanline (`LY` 0–143):

- The PPU walks all **40** OAM entries (**2 T** each → `40 × 2 = 80 T`).
- For each sprite, it checks whether its vertical range (`Y` … `Y + height`) intersects the **current `LY`**.
- It keeps the first **10** matches in an internal line buffer; further matches are ignored (this is why sprites **flicker / disappear** when too many share a line).

Mode 3 then uses that short-list to mix sprite pixels with background pixels.

#### Does it use last frame’s sprites?

**No.** It always reads **whatever is currently in OAM** for the frame being drawn.

- Games normally update OAM during **Mode 1 (VBlank)** before `LY` wraps to 0.
- When the new frame starts (`LY` 0), each line’s Mode 2 reads that updated OAM.
- While Mode 2 is reading OAM, the **CPU is locked out of OAM** on real hardware (reads → `$FF`, writes ignored) until Mode **0** or **1**.

### Mode cheat-sheet (use cases)

| Mode | Name | What the PPU is doing | Typical game / emu use |
|---|---|---|---|
| **2** | OAM search | Scan OAM; pick ≤10 OBJs for **this** line | OAM busy on real HW; STAT Mode-2 IRQ ≈ “new line sprite work” (uncommon) |
| **3** | Drawing | Fetch tiles, push pixels to the LCD | **VRAM/OAM busy** — avoid VRAM writes here; **no** Mode-3 STAT IRQ |
| **0** | HBlank | Line finished; idle until next Mode 2 | Mid-frame `SCX`/`BGP` tricks; safer VRAM writes; STAT Mode-0 IRQ = **once per visible line** |
| **1** | VBlank | Whole frame’s visible lines done (`LY` 144–153) | **Main** time to rewrite graphics; dedicated **VBlank IRQ** (`IF` bit 0) when `LY`→144; STAT Mode-1 is optional |

### `STAT` register (`$FF41`) layout

`STAT` mixes **live read-only status** (hardware) with **R/W interrupt enables** (game-configured):

| Bit | Name | Access | Meaning |
|---|---|---|---|
| **7** | Unused | R | Often reads as `1` |
| **6** | `LYC=LY` IRQ select | R/W | If `1`, STAT IRQ when `LY == LYC` (edge) |
| **5** | Mode 2 IRQ select | R/W | If `1`, STAT IRQ when **entering** Mode 2 |
| **4** | Mode 1 IRQ select | R/W | If `1`, STAT IRQ when **entering** Mode 1 |
| **3** | Mode 0 IRQ select | R/W | If `1`, STAT IRQ when **entering** Mode 0 |
| **2** | `LYC == LY` flag | **R** | `1` if `LY == LYC` right now |
| **1–0** | PPU mode | **R** | Current mode `0` / `1` / `2` / `3` |

- **Write `STAT`:** only bits **3–6** (mask `$78`) are stored. Writes to bits 0–2 are ignored (mode / coincidence are live).
- **Read `STAT`:** saved enables (`$78`) **OR** live `mode_now()` in bits 0–1, `(ly == lyc) << 2`, and usually bit 7 = 1.

SoftGBC field helpers: [`src/stat.rs`](src/stat.rs) (`BitField` / `Reg8`).

### Edge-triggered STAT interrupts (`IF` bit 1)

Unlike the dedicated **VBlank interrupt** (`IF` bit 0 — once when `LY` becomes 144), **STAT** (`IF` bit 1) is a **shared** line games configure for PPU events.

In `tick()` (see `update_stat_edges`), SoftGBC checks **rising edges** (false → true), not levels:

1. **Mode transitions** — when `mode_now()` changes:
   - Enter Mode **0** and bit 3 set → STAT IRQ  
   - Enter Mode **1** and bit 4 set → STAT IRQ  
   - Enter Mode **2** and bit 5 set → STAT IRQ  
   - *(No interrupt select for Mode 3.)*
2. **`LYC == LY` coincidence** — game writes a target line to `LYC` (`$FF45`) (e.g. status-bar split). When `ly == lyc` **becomes** true and bit 6 is set → STAT IRQ **once**. Staying equal does not re-fire.

**v1 note:** Real hardware has **STAT blocking** (internal IRQ line is an OR of enabled conditions, so a second condition becoming true while another is already true may not produce a new edge). SoftGBC v1 fires on individual mode-enter and LYC edges for clarity; blocking can come later.

### How games combine `LY` + mode

| Goal | Approach |
|---|---|
| Wait until safe to rewrite graphics | Poll until Mode **0** or **1**, or wait for **VBlank IRQ** |
| Do something every scanline | STAT **Mode 0** (HBlank) interrupt |
| Know which line | Read **`LY`** |
| Interrupt on a **specific** line | Set **`LYC`**, enable STAT **LYC=LY** interrupt |

**Minimum-game default:** ignore Modes 2/3 day-to-day; use **VBlank IRQ + `LY`**. Add Mode 0 / STAT when you need mid-frame effects.

**Refs:** [Pan Docs — STAT](https://gbdev.io/pandocs/STAT.html) · [OAM](https://gbdev.io/pandocs/OAM.html) · [GBEDG — PPU](https://github.com/Ashiepaws/GBEDG/blob/master/ppu/index.md)

## Checklist — **driver**

What → Structure → Min test → Done when (pseudocode). Top → bottom.  
Mirrored on SoftGBC README step 8.

---

- [x] `tick(t)` **advances LY**
  **What:** LY counts scanlines from time alone.  
  **Structure:**

```text
Ppu: ly, line_cycles   # line_cycles 0..455

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

fixture rows (fresh ppu each time):
  0 → 0,  456 → 1,  912 → 2,
  65664 → 144,  70224 → 0
```

  **Done when:** [`01_ly_only.expected.txt`](fixtures/01_ly_only.expected.txt) green.

---

- [x] **VBlank IRQ edge at LY=144**
  **What:** one request when LY *becomes* 144 (not every tick while on 144).  
  **Structure:**

```text
when ly becomes 144: raise VBlank (IF bit 0) once

# edge = previous ly != 144 and ly == 144 after tick
# (or: set flag / return IrqLines; do not level-trigger while ly stays 144)
```

  **Min test:**

```text
tick(143*456) → no VBlank
tick(456)     → ly==144, VBlank once
further ticks while ly==144 → no extra VBlank
```

  **Done when:** edge-only raise when LY becomes 144 (not every tick on 144).

---

- [x] **IO** `FF40`**–**`FF4B`
  **What:** CPU-visible LCD register window. Games write `LCDC` (`FF40`) to enable the PPU and read `LY` (`FF44`) to wait for scanlines. Other bytes in the range are plain storage until later items (modes/BG/…). Implemented as [`IoDevice`](../peripherals/src/mmio.rs) on `DmgPpu`; bus routes `FF40`–`FF4B` here.  
  **Refs:** [Pan Docs — Hardware regs](https://gbdev.io/pandocs/Hardware_Reg_List.html) · [LCDC](https://gbdev.io/pandocs/LCDC.html) · [STAT / LY](https://gbdev.io/pandocs/STAT.html)  
  **Structure:**

```text
# map (DMG)
FF40 LCDC  R/W
FF41 STAT  R/W   # selects stored; mode / LYC=LY live on read (see Modes item)
FF42 SCY   R/W
FF43 SCX   R/W
FF44 LY    R     # always returns current ly; ignore writes (start)
FF45 LYC   R/W
FF46 DMA   R/W   # stored on PPU until Dma device is wired
FF47 BGP   R/W
FF48 OBP0  R/W
FF49 OBP1  R/W
FF4A WY    R/W
FF4B WX    R/W

read_io / write_io (IoDevice)
FF44 read → ly; FF44 write → no-op
```

  **Min test:**

```text
write_io(FF40, 0x80) → read_io(FF40) == 0x80
write_io(FF40, 0x91) → read_io(FF40) == 0x91
tick(456) → read_io(FF44) == 1 == ly
write_io(FF44, 0) → read_io(FF44) still 1
write_io(FF42, 0x55) → read_io(FF42) == 0x55
# bus: write FF40 does not land in flat io[]
```

  **Done when:** LCDC round-trip; `FF44` == `ly`; `FF44` writes ignored; bus routes via `IoDevice`.

---

- [x] **Modes 0–3 / STAT**
  **What:** Each visible scanline walks Mode 2 (OAM search) → Mode 3 (draw) → Mode 0 (HBlank). Lines `LY >= 144` stay in Mode 1 (VBlank). `STAT` (`FF41`) exposes the current mode in bits 0–1, the LYC=LY flag in bit 2, and interrupt enables in bits 3–6. On an **enabled edge** (enter mode, or LYC becomes equal), raise STAT IRQ (`IF` bit 1) once — same edge style as VBlank.  
  **Refs:** [Pan Docs — STAT](https://gbdev.io/pandocs/STAT.html) · explainer + diagrams: [PPU modes 0–3](#ppu-modes-0-3)  
  **Structure:**

```text
# STAT (FF41) — softgbc-utils BitField layout in src/stat.rs
#  bit7     unused (often reads 1)
#  bit6     LYC=LY interrupt select   (R/W)
#  bit5     Mode 2 interrupt select   (R/W)
#  bit4     Mode 1 interrupt select   (R/W)
#  bit3     Mode 0 interrupt select   (R/W)
#  bit2     LYC==LY flag              (R)  live from (ly == lyc)
#  bit1-0   PPU mode                  (R)  live from timing

# v1 fixed budgets
MODE2_DOTS = 80          # line_cycles 0..79
MODE3_DOTS = 172         # line_cycles 80..251
# remainder → Mode 0     # line_cycles 252..455
# LY 144..153            → Mode 1 for the whole line

mode_now():
  if ly >= 144: return 1
  if line_cycles < 80: return 2
  if line_cycles < 80 + 172: return 3
  return 0

read STAT:
  keep interrupt-select bits from last write (mask 0x78)
  OR live mode bits + LYC flag (+ optional bit7=1)

write STAT:
  store only interrupt-select bits; ignore writes to bits 0–2

tick:
  on mode change, if that mode's select enabled → STAT irq (not Mode 3)
  on LYC becoming equal, if bit6 set → STAT irq once
```

  **Min test:**

```text
# mode walk (LY=0)
T=0 → mode 2; tick(79) still 2; tick(1) → mode 3
tick(172) → mode 0; finish line → ly==1, mode 2

tick(144*456) → ly==144, mode 1

# LYC flag
LYC=0 at ly=0 → STAT bit2 set; LYC=5 → clear; tick to ly=5 → set

# STAT write masking
write FF41=0xFF → selects 0x78 kept; mode bits still live (==2 at reset)

# LYC STAT IRQ edge once
STAT bit6 + LYC=1, tick(456) → IF_STAT once; next tick → no repeat

# Mode 2 enter IRQ
STAT Mode2 select; leave line via Mode 0 then enter next line Mode 2 → IF_STAT once
```

  **Done when:** mode readable via `FF41`; LYC + Mode-2 enter STAT edges tested (see [PPU modes 0–3](#ppu-modes-0-3) above).

---

- [x] **Background → framebuffer + CRC**
  **What:** Produce a DMG **160×144** greyscale frame from the **background** layer only (no window/sprites yet). The BG is a scrolling viewport into a **256×256** pixel tilemap (32×32 tiles). For each screen pixel, resolve tile index → 2bpp colour ID (0–3) → shade via **BGP**, write into a framebuffer, then **CRC/hash** that buffer for goldens. Implemented: `render_bg(vram)`, `frame_crc()` (CRC-32), [`fixtures/02_solid_bg.expected.txt`](fixtures/02_solid_bg.expected.txt).  
  **Refs:** [Tile Data](https://gbdev.io/pandocs/Tile_Data.html) · [Tile Maps](https://gbdev.io/pandocs/Tile_Maps.html) · [Scrolling](https://gbdev.io/pandocs/Scrolling.html) · [BGP](https://gbdev.io/pandocs/Palettes.html) · [LCDC](https://gbdev.io/pandocs/LCDC.html)  
  **Structure:**

```text
# ownership (unit-test first; bus later)
Ppu:
  framebuffer[144][160]   # or flat 160*144 bytes — shade 0..3 (or RGB later)
  # VRAM: either Ppu holds a copy, or render(vram: &[u8; 0x2000]) is passed in
  # SoftGBC bus already has vram[0x2000] at $8000 — wire later

# LCDC bits needed for BG (others ignored this item)
#  bit7 LCD on     — if clear, frame is blank (white); skip draw or fill white
#  bit4 tile data  — 0 = $8800 signed method; 1 = $8000 unsigned
#  bit3 BG map     — 0 = $9800–9BFF; 1 = $9C00–9FFF
#  bit0 BG enable  — 0 = blank BG (white on DMG); 1 = draw

# BGP (FF47): each colour ID → grey 0..3
#  bits1-0 = ID0, bits3-2 = ID1, bits5-4 = ID2, bits7-6 = ID3
shade(id) = (BGP >> (id * 2)) & 0b11

# one screen pixel (sx, sy) in 0..159 × 0..143
render_bg_pixel(sx, sy):
  wx = (SCX + sx) mod 256          # wrap in 256×256 map
  wy = (SCY + sy) mod 256
  map_x = wx / 8                   # tile column 0..31
  map_y = wy / 8                   # tile row 0..31
  fine_x = wx mod 8
  fine_y = wy mod 8

  map_base = LCDC.3 ? 0x9C00 : 0x9800
  # VRAM index relative to $8000: map_base - 0x8000
  tile_id = vram_byte(map_base + map_y * 32 + map_x)

  tile_addr = tile_data_addr(tile_id, LCDC.4)   # see below
  # each tile row = 2 bytes (lo plane, hi plane); bit7 = leftmost pixel
  lo = vram_byte(tile_addr + fine_y * 2)
  hi = vram_byte(tile_addr + fine_y * 2 + 1)
  bit = 7 - fine_x
  color_id = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1)

  framebuffer[sy * 160 + sx] = shade(color_id)

tile_data_addr(tile_id, lcdc4):
  if lcdc4 == 1:                    # unsigned $8000 method
    return 0x8000 + tile_id * 16
  else:                             # signed $8800 method; base $9000
    return 0x9000 + (i8)tile_id * 16

# when to fill the buffer (pick one v1 policy and test it)
render_frame():
  clear or overwrite all 160×144
  if !LCDC.7 or !LCDC.0: fill white (shade of ID0 or fixed white); return
  for sy in 0..143:
    for sx in 0..159:
      render_bg_pixel(sx, sy)

# optional: render during Mode 3 per line instead of whole frame —
# not required for first golden; whole-frame after 144 lines / 1 frame is fine

frame_crc = crc32(framebuffer)   # or any stable hash; document which
# fixture: fixtures/02_solid_bg.expected.txt  (crc + maybe tiny PPM dump)
```

```text
  256×256 BG map (32×32 tiles)              LCD 160×144 viewport
  ┌────────────────────────────┐            ┌──────────────────┐
  │  tile (0,0) …              │            │ (SCX,SCY) origin │
  │       ┌──────────────┐     │  scroll    │##################│
  │       │ visible 160×144    │ ─────────► │##################│
  │       │ (wraps at edges)   │            │##################│
  │       └──────────────┘     │            └──────────────────┘
  └────────────────────────────┘
```

  **Min test:**

```text
# --- solid colour screen (simplest golden) ---
ppu = new()
# LCD on + BG on + tile data $8000 + map $9800
write LCDC = 0x91                  # bits 7,4,0 (typical post-boot-ish)
write BGP  = 0xE4                  # 3,2,1,0 → classic DMG palette
write SCX=0, SCY=0

# tile 0: every pixel colour_id 3 (solid dark before BGP)
# 8 rows × (lo=0xFF, hi=0xFF) = 16 bytes of 0xFF at $8000
fill vram tile0 with 0xFF
# map: all $9800.. entries = 0 (tile 0)  — or just the 20×18 visible tiles

render_frame()                     # or tick one full frame then render
assert frame_crc == GOLDEN_SOLID   # checked-in fixtures/02_solid_bg…

# spot-check a few pixels == shade(3) under BGP 0xE4
assert framebuffer[0] == expected_grey
assert framebuffer[160*143 + 159] == expected_grey

# --- BG off → blank ---
write LCDC with bit0 clear (keep LCD on)
render_frame()
assert all pixels white / blank golden

# --- scroll smoke (optional same item or tiny follow-up) ---
# checker or two-tone tiles; SCX=8 → image shifts left by one tile
# assert crc changes and/or pixel(0,0) matches former pixel(8,0)
```

  **Done when:** `fixtures/02_solid_bg` (CRC `0x511BEFBF`) checked in and green; BG enable + BGP + `$8000` tile path proven; SCX=8 smoke test covered.

---

- [x] **Window**
  **What:** The **window** is a non-scrolling overlay tilemap drawn on top of the BG. Screen top-left of the window is `(WX - 7, WY)`. Enabled by `LCDC.5` (and requires `LCDC.0` on DMG). Tile data uses the same `$8000`/`$8800` mode as BG (`LCDC.4`); map base is `$9800` or `$9C00` via `LCDC.6`. SoftGBC v1: whole-frame `render_bg` uses `win_y = sy - WY` (no mid-frame internal-line quirks yet).  
  **Refs:** [Window](https://gbdev.io/pandocs/Window.html) · [Tile Maps](https://gbdev.io/pandocs/Tile_Maps.html) · [LCDC](https://gbdev.io/pandocs/LCDC.html) · [`fixtures/03_window_corner.expected.txt`](fixtures/03_window_corner.expected.txt)  
  **Structure:**

```text
# LCDC.5 win enable, LCDC.6 win map, LCDC.0 must be on (DMG)
# visible (v1): WY in 0..143, WX in 0..166, sy >= WY, sx >= (WX-7)

pixel(sx, sy):
  if window covers (sx, sy):
    (win_x, win_y) = (sx - (WX-7), sy - WY)
    fetch from win map + tile data → BGP shade
  else:
    BG path (SCX/SCY + BG map)

# defer: window line counter quirks, WX=0/166 bugs, mid-frame enable
```

  **Min test:**

```text
LCDC=0xF1, BGP=0xE4, WY=72, WX=87   # window at (80,72)
BG map tile0 shade0; win map tile1 shade3
render → (0,0)=0; (80,72)=3; (79,72)=0; crc == 0xAB9E8A54
window enable clear → full BG blank/zeros
```

  **Done when:** `fixtures/03_window_corner` CRC green; window overlays BG at `(WX-7, WY)`.

---

- [x] **Sprites**
  **What:** Up to **40** OBJs in OAM (`$FE00`–`$FE9F`); per scanline Mode-2 style select of ≤**10** whose Y range hits the line; Mode-3 style composite over BG/window. OBJ tiles always from `$8000` (unsigned). Colour ID **0** is transparent. Attribute bit7 = draw behind BG colours 1–3; bit4 picks OBP0/OBP1; bits 5–6 X/Y flip. SoftGBC: `render_frame(vram, oam)`.  
  **Refs:** [OAM](https://gbdev.io/pandocs/OAM.html) · [`fixtures/04_sprite_one.expected.txt`](fixtures/04_sprite_one.expected.txt)  
  **Structure:**

```text
OAM entry (4 bytes): Y(=screen_y+16), X(=screen_x+8), tile, flags
height = LCDC.2 ? 16 : 8

objs_on_line(ly): first 10 OAM entries with ly in [Y-16, Y-16+height)

for each screen pixel:
  bg_id = BG/window colour id (0 if BG off)
  shade = BGP(bg_id)
  if OBJ enable:
    among line OBJs covering this x, winner = lowest X, then earliest OAM
    if obj colour_id != 0:
      if (priority && bg_id != 0): keep BG
      else: shade = OBP(obj_id)

# defer: FIFO timing, 10-OBJ mid-line quirks beyond scan order
```

  **Min test:**

```text
LCDC=0x93, BGP=OBP0=0xE4
BG tile0 shade0; OBJ tile1 shade3 at screen (40,40)
render_frame → crc 0x6222800E; pixel (40,40)=3
OBJ tile all 0 → transparent (BG shows)
OBJ priority bit + BG id3 → BG wins
```

  **Done when:** `04_sprite_one` golden green; transparency + BG-priority smoke covered.

---

- [x] **SoftGBC owns PPU; remove** `stub_ly`
  **What:** SoftGBC uses the real `DmgPpu` on `DmgBus` for `FF40`–`FF4B` / LY. `SoftGbc::step` → `tick_m_cycles` advances the PPU; VBlank/STAT bits from `ppu.tick` OR into `IF`. No `MachineConfig::stub_ly` (FF44 writes were already ignored).  
  **Structure:**

```text
DmgBus owns Ppu
tick_t_cycles(t):
  irq = timer.tick(t).merge(ppu.tick(t))
  IF |= irq.bits          # VBlank bit0, STAT bit1, …

read/write FF40–FF4B → IoDevice on ppu
FF44 read → ly; FF44 write → no-op

MachineConfig: no stub_ly
SoftGbc::step → cpu.step + bus.tick_m_cycles
```

  **Min test:**

```text
# softgbc-core (already): tick → FF44; VBlank/STAT → IF
# soc:
SoftGbc reset → FF44 == 0
114× NOP step → FF44 == 1
tick_t_cycles(144*456) + IME+IE VBlank → step → PC == $0040
Blargg cpu_instrs still green
```

  **Done when:** SoftGbc LY/VBlank tests green; Blargg individuals pass without LY stub.

---

- [ ] `dmg-acid2` **/ visual golden**
  **What:** last accuracy gate for v1 graphics.  
  **Structure:** run ROM or compare frame CRC to reference.  
  **Min test:**

```text
run dmg-acid2 (or agreed dump)
assert framebuffer/CRC matches reference
```

  **Done when:** checklist item checked and noted in SoftGBC status.

## Out of scope

CPU opcodes · APU · MBC · SoftGBC glue before LY unit tests green · Verilog.
