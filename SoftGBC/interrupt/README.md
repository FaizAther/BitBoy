# interrupt

`IF` (`FF0F`), `IE` (`FFFF`), priority, CPU IRQ entry.

**Refs:** [Pan Docs — Interrupts](https://gbdev.io/pandocs/Interrupts.html) · SoftGBC [`Timer`](../peripherals/src/Timer.README.md) · [`dmg-ppu`](../dmg-ppu/README.md)  

## Status *(2026-11)*

| Piece | State |
|---|---|
| `IF` / `IE` on bus | Done |
| Priority + vectoring in CPU | Done |
| Clear `IF` bit on entry | Done |
| Timer → `IF` bit 2 | Done (Blargg `02`) |
| VBlank / STAT / serial / joypad → `IF` | Open |
| Dedicated helpers in this folder | Optional |

```text
IF bit:  0        1       2       3        4
         VBlank   STAT    Timer   Serial   Joypad
vector:  $0040    $0048   $0050   $0058    $0060
```

Hardware **requests** set `IF` bits. CPU **services** only if `IME` and matching `IE`. Priority = bit 0 highest … bit 4 lowest.

## Checklist

Pattern per item: **What → Structure → Min test → Done when** (pseudocode).

---

- [x] **Readable/writable `IF` and `IE`**  
  **Done when:** `FF0F` / `FFFF` on bus; `IF` often inits with upper bits 1 (`0xE0`).

- [x] **Priority when multiple pending**  
  **Done when:** CPU services lowest set bit of `(IF & IE)`.

- [x] **Dispatch when `IME` and masked bit set**  
  **Done when:** push PC, clear IME, jump to vector (~5 M-cycles, +1 from HALT).

- [x] **Clear corresponding `IF` bit on service**

- [x] **`HALT` wake-up (basic)**  
  **Done when:** `(IF & IE)` exits HALT even if `IME=0`.

---

- [ ] **Source: timer → `IF` bit 2**

  **What:** TIMA overflow requests timer IRQ. SoftGBC already does this — re-verify.  
  **Structure:**
```text
timer.tick(t):
  if TIMA overflows:
    TIMA = TMA
    request bit 2

bus:
  IF = IF | 0x04
```
  **Min test:**
```text
enable TAC, seed TIMA near 0xFF
advance enough T for overflow
assert (IF & 0x04) != 0
```
  **Done when:** Blargg `02` still green / assert above passes.

---

- [ ] **Source: VBlank → `IF` bit 0**

  **What:** when LY becomes 144, PPU requests VBlank.  
  **Structure:**
```text
after ppu.tick(t):
  if ppu.take_vblank_irq():   # edge only
    IF = IF | 0x01
```
  **Min test:**
```text
real PPU on bus (no stub_ly)
advance ~144 * 456 T
assert (IF & 0x01) != 0
```
  **Done when:** IF bit 0 sets at first VBlank without poking `FF44` by hand.

---

- [ ] **Source: STAT → `IF` bit 1**

  **What:** STAT mode / LYC coincidence edges request LCD STAT IRQ.  
  **Structure:**
```text
if ppu.take_stat_irq():
  IF = IF | 0x02
```
  **Min test:**
```text
enable LYC==LY in STAT
force LY == LYC
assert (IF & 0x02) != 0
```
  **Done when:** STAT edge sets bit 1 once per event (not every tick).

---

- [ ] **Source: serial → `IF` bit 3**

  **What:** IRQ fires on **transfer complete**, not on `$81` start.  
  **Structure:**
```text
on transfer complete:
  clear SC.7
  IF = IF | 0x08
```
  **Min test:**
```text
write SB = 'X'
write SC = $81
assert (IF & 0x08) != 0
assert SC.7 == 0
```
  **Done when:** see also [`Serial.README`](../peripherals/src/Serial.README.md).

---

- [ ] **Source: joypad → `IF` bit 4**

  **What:** selected-line high→low can request joypad IRQ.  
  **Structure:**
```text
on press edge while group selected:
  IF = IF | 0x10
```
  **Min test:**
```text
select buttons on FF00
press A
assert (IF & 0x10) != 0
```
  **Done when:** see [`Joypad.README`](../peripherals/src/Joypad.README.md).

---

- [ ] **Optional helpers in this folder**

  **What:** constants + “highest pending” — does **not** own registers.  
  **Structure:**
```text
IF_VBLANK=0x01 … IF_JOYPAD=0x10

highest_pending(IF, IE):
  pending = IF & IE & 0x1F
  for bit = 0..4:                 # 0 highest
    if pending has bit:
      return bit, vector[bit]     # $40,$48,$50,$58,$60
  return none
```
  **Min test:**
```text
highest_pending(0x05, 0x05) → bit 0, $0040   # VBlank over timer
```
  **Done when:** unit test passes; bus/CPU still own storage.

---

- [ ] **Integration: VBlank + timer priority**

  **What:** both bits can be set; CPU still takes VBlank first.  
  **Structure:**
```text
SoftGbc.step:
  m = cpu.step(bus)
  bus.tick(m * 4)     # may set several IF bits
  # next irq check uses priority
```
  **Min test:**
```text
IF = 0x05, IE = 0x05, IME = on
service one interrupt
assert PC == $0040          # not $0050
```
  **Done when:** priority matches Pan Docs bit order.
