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
| LY / VBlank | **You implement** (checklist below) |
| SoftGBC | still `stub_ly = 144` |

Scanline = **456 T**. Frame = **154** lines (`0..143` visible, `144..153` VBlank).

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

- [ ] **VBlank IRQ edge at LY=144**
  **What:** one request when LY *becomes* 144 (not every tick while on 144).  
  **Structure:**

```text
when ly becomes 144: vblank_irq = true

take_vblank_irq():
  was = vblank_irq
  vblank_irq = false
  return was
```

  **Min test:**

```text
tick(143*456) → take == false
tick(456)     → ly==144, take==true, take again==false
```

  **Done when:** edge-only behaviour locked.

---

- [ ] **IO** `FF40`**–**`FF4B`
  **What:** LCD regs; `FF44` reads as LY.  
  **Structure:**

```text
read_io / write_io
FF44 read → ly
(start: ignore FF44 writes)
```

  **Min test:**

```text
write FF40 LCD-on → read back
read FF44 == ly()
```

  **Done when:** round-trip for LCDC; LY readable.

---

- [ ] **Modes 0–3 / STAT**
  **What:** Mode 2 OAM search → 3 draw → 0 HBlank per visible line; Mode 1 = VBlank. STAT can request `IF` bit 1 (see Pan Docs / GBEDG).  
  **Structure:**

```text
line_cycles also drives mode:
  0..79   → mode 2
  80..…   → mode 3 (length varies)
  rest    → mode 0
LY >= 144 → mode 1

take_stat_irq on enabled edges (mode enter / LYC==LY)
```

  **Min test:**

```text
at T=0 of visible line → mode 2
enable LYC==LY in STAT, set LYC=ly → take_stat once
```

  **Done when:** mode readable via STAT; one STAT path sets `IF` bit 1 when wired.

---

- [ ] **Background → framebuffer + CRC**
  **What:** draw BG tiles into 160×144; hash for goldens.  
  **Structure:**

```text
for each pixel (or per frame):
  map SCX/SCY + LCDC → tile in VRAM
  colour via BGP → framebuffer[y*160+x]
frame_crc = hash(framebuffer)
```

  **Min test:**

```text
one solid tile, BG on, BGP set
one frame of time
assert frame_crc == GOLDEN
```

  **Done when:** golden file checked in (e.g. `02_solid_bg`).

---

- [ ] **Window**
  **What:** WY/WX select window layer over BG.  
  **Structure:**

```text
if window enabled and LY >= WY and X past WX:
  fetch from window tile map instead of BG
```

  **Min test:**

```text
known WY/WX + one tile → pixel (x,y) or CRC matches golden
```

  **Done when:** window-only scene matches golden.

---

- [ ] **Sprites**
  **What:** OAM objects composited with BG (priority / transparency).  
  **Structure:**

```text
mode 2: collect up to 10 OBJs for this line
mode 3: merge OBJ pixels with BG per priority bit
```

  **Min test:**

```text
one OBJ on known tile → non-BG colour at expected spot
```

  **Done when:** sprite golden or acid2 progress.

---

- [ ] **SoftGBC owns PPU; remove** `stub_ly`
  **What:** real LY on the bus; VBlank into `IF`.  
  **Structure:**

```text
DmgBus owns Ppu
tick_t_cycles(t):
  timer.tick(t)
  ppu.tick(t)
  if ppu.take_vblank_irq(): IF |= 0x01
  if ppu.take_stat_irq():   IF |= 0x02
FF44 → ppu.ly()
stub_ly = none
```

  **Min test:**

```text
SoftGbc steps → FF44 changes
IME+IE VBlank → PC can reach $0040
```

  **Done when:** Blargg/LCD waits work without stub.

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
