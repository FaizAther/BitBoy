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
| Modes / STAT | **You implement** (checklist below) |
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

- [x] **VBlank IRQ edge at LY=144**  
  **Done when:** edge-only raise when LY becomes 144 (not every tick on 144).

---

- [x] **IO** `FF40`**–**`FF4B`  
  **Done when:** LCDC round-trip; `FF44` == `ly`; `FF44` writes ignored; bus routes via `IoDevice`.

---

- [ ] **Modes 0–3 / STAT**
  **What:** Each visible scanline walks Mode 2 (OAM search) → Mode 3 (draw) → Mode 0 (HBlank). Lines `LY >= 144` stay in Mode 1 (VBlank). `STAT` (`FF41`) exposes the current mode in bits 0–1, the LYC=LY flag in bit 2, and interrupt enables in bits 3–6. On an **enabled edge** (enter mode, or LYC becomes equal), raise STAT IRQ (`IF` bit 1) once — same edge style as VBlank.  
  **Refs:** [Pan Docs — STAT](https://gbdev.io/pandocs/STAT.html) · [GBEDG — PPU timing](https://github.com/Ashiepaws/GBEDG/blob/master/ppu/index.md)  
  **Structure:**

```text
# STAT (FF41)
#  bit7     unused (often reads 1)
#  bit6     LYC=LY interrupt select   (R/W)
#  bit5     Mode 2 interrupt select   (R/W)
#  bit4     Mode 1 interrupt select   (R/W)
#  bit3     Mode 0 interrupt select   (R/W)
#  bit2     LYC==LY flag              (R)  live from (ly == lyc)
#  bit1-0   PPU mode                  (R)  live from timing

# v1 fixed budgets (real Mode 3 length varies with sprites; lock constants for tests)
MODE2_DOTS = 80          # line_cycles 0..79
MODE3_DOTS = 172         # line_cycles 80..251   (simplistic)
# remainder → Mode 0     # line_cycles 252..455
# LY 144..153            → Mode 1 for the whole line

Ppu:
  … existing …
  mode: 0..3             # or compute from ly + line_cycles
  prev_mode
  lyc_equal_prev
  stat_irq: bool         # or return via IrqLines like VBlank

mode_now():
  if ly >= 144: return 1
  if line_cycles < 80: return 2
  if line_cycles < 80 + 172: return 3
  return 0

read STAT:
  writable = stat & 0x78          # keep select bits from last write
  mode_bits = mode_now() & 0x03
  lyc_flag  = 0x04 if (ly == lyc) else 0
  return writable | lyc_flag | mode_bits   # (+ optional bit7=1)

write STAT:
  store only interrupt-select bits (mask 0x78); ignore writes to 0–2

tick / after LY or line_cycles change:
  m = mode_now()
  if m != prev_mode:
    if select_bit_for(m) enabled:   # bit3 for mode0, bit4 mode1, bit5 mode2
      raise STAT irq
    prev_mode = m
  lyc_eq = (ly == lyc)
  if lyc_eq and not lyc_equal_prev and (stat bit6 set):
    raise STAT irq
  lyc_equal_prev = lyc_eq

# IRQ return: either IrqLines with IF_STAT from tick(), or take_stat_irq()
# Bus later: IF |= 0x02  (same merge path as VBlank)

# defer: LCDC.7 off → mode reads 0; Mode 3 length vs OBJ count; STAT blocking
```

  **Min test:**

```text
# --- mode from timing (visible line LY=0) ---
ppu = new()
assert mode_now() == 2                  # T=0 → Mode 2
assert (read(FF41) & 0x03) == 2

tick(79)
assert mode_now() == 2
assert (read(FF41) & 0x03) == 2

tick(1)                                 # line_cycles == 80
assert mode_now() == 3
assert (read(FF41) & 0x03) == 3

tick(172)                               # into HBlank (252)
assert mode_now() == 0
assert (read(FF41) & 0x03) == 0

tick(456 - 252)                         # end of line → LY=1, Mode 2 again
assert ly == 1
assert mode_now() == 2

# --- Mode 1 during VBlank ---
ppu = new()
tick(144 * 456)
assert ly == 144
assert mode_now() == 1
assert (read(FF41) & 0x03) == 1

# --- LYC==LY flag (STAT bit 2), no IRQ yet ---
ppu = new()
write(FF45, 0)                          # LYC = 0
assert (read(FF41) & 0x04) != 0         # equal at LY=0
write(FF45, 5)
assert (read(FF41) & 0x04) == 0
tick(5 * 456)
assert ly == 5
assert (read(FF41) & 0x04) != 0

# --- STAT write: only select bits stick; mode bits stay live ---
ppu = new()
write(FF41, 0xFF)
assert (read(FF41) & 0x78) == 0x78      # bits 3–6 kept
assert (read(FF41) & 0x03) == 2         # mode still from timing, not 3

# --- LYC STAT IRQ: edge once ---
ppu = new()
write(FF41, 0x40)                       # bit6 LYC interrupt enable
write(FF45, 1)                          # fire when LY becomes 1
irq = tick(456)                         # LY 0 → 1
assert irq has STAT (IF bit 1)          # or take_stat_irq() == true
assert take_stat / tick again → no second STAT until LYC edge repeats

# --- optional: Mode 2 enter IRQ ---
ppu = new()
write(FF41, 0x20)                       # bit5 Mode 2 select
# first enter Mode 2 at reset may or may not fire — pick one policy and test it
tick(456)                               # leave line 0, enter Mode 2 on line 1
assert STAT irq once on that enter
```

  **Done when:** `FF41` bits 0–1 match `mode_now()`; bit 2 tracks `LY==LYC`; at least the LYC path raises STAT once on the edge (ready for bus `IF |= 0x02`).

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
