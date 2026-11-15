# bus

16-bit address decode for CPU (and later DMA).

**Implementation:** `bus.rs` — `Bus` + `DmgBus`.  
**Refs:** [Pan Docs — Memory map](https://gbdev.io/pandocs/Memory_Map.html)  

## Status *(2026-11)*

| Region | State |
|---|---|
| ROM / WRAM / echo / HRAM / IE | Done |
| VRAM / OAM buffers | Done (raw) |
| Timer + serial | Done |
| `tick` → timer → `IF` | Done |
| PPU LCD regs / LY / VBlank+STAT→IF | Done |
| Joypad / DMA | Open |

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [x] **`read8` / `write8`**  
- [x] **ROM / WRAM / HRAM / IE**  
- [x] **VRAM / OAM buffers**  
- [x] **Timer + serial routing**  
- [x] **`tick` → timer → `IF`**

---

- [x] **PPU-owned LCD regs**

  **What:** `FF40`–`FF4B` / LY live on `Ppu`, not flat IO.  
  **Structure:**
```text
DmgBus.ppu
read FF40..FF4B → ppu IoDevice
tick(t): IF |= timer.tick.merge(ppu.tick)
```
  **Min test:**
```text
tick(456) → read(FF44) == 1
VBlank / LYC STAT → IF bits 0 / 1
```
  **Done when:** stub_ly gone; LY moves under SoftGbc (soc tests).

---

- [ ] **Joypad + DMA routing**

  **What:** `FF00` → Joypad; `FF46` → DMA.  
  **Structure:** same decode style as timer.  
  **Min test:** see Joypad / Dma READMEs.  
  **Done when:** those crate tests pass through the bus.

---

- [ ] **Open-bus policy**

  **What:** unused IO reads return a defined value (often `$FF`).  
  **Structure:**
```text
unmapped IO read → 0xFF
```
  **Min test:** read unused IO → `0xFF`.  
  **Done when:** documented + tested.
