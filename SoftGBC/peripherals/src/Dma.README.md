# DMA

OAM DMA via `FF46`.

**Refs:** [Pan Docs — OAM DMA](https://gbdev.io/pandocs/OAM_DMA_Transfer.html) · [GBEDG — DMA](https://github.com/Ashiepaws/GBEDG/blob/master/dma/index.md) · [`dma.rs`](dma.rs)  

## Status *(2026-11)*

**Instant OAM DMA wired** on `DmgBus`: write `FF46` copies 160 bytes into OAM immediately.
Timed DMA + CPU bus limits still open.

```text
write FF46 = src_hi
  → copy 160 bytes from (src_hi << 8) → OAM FE00..FE9F
  SoftGBC: instant (Dma::run_instant / bus.run_oam_dma_instant)
real HW ≈ 160 M-cycles with CPU bus limits
```

### Allowed sources (SoftGBC)

| Source | SoftGBC |
|---|---|
| WRAM `$C000`–`$DFFF` | **Supported** (common game path) |
| ROM / VRAM / other via `bus.read8` | Whatever the bus returns (no extra reject) |
| Echo WRAM `$E000`–`$FDFF` | Follows bus echo mapping |

Games typically DMA from WRAM. We do not yet emulate source-region restrictions beyond the bus map.

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [x] **Write `FF46` starts transfer**

  **What:** writing the high byte of the source address kicks off OAM DMA.  
  **Structure:**
```text
on write FF46:
  src = data << 8
  start_dma(src)
```
  **Min test:**
```text
mem[C000] = 0xAB
write FF46 = 0xC0
assert read(FE00) == 0xAB
```
  **Done when:** one-byte smoke test passes (`dma` + `oam_dma_ff46_copies_wram_to_oam`).

---

- [x] **Copy 160 bytes to OAM**

  **What:** full sprite attribute table refresh.  
  **Structure:**
```text
for i in 0..159:
  oam[i] = read(src + i)
```
  **Min test:**
```text
fill C000..C09F with i
DMA 0xC0
for i in 0..159: assert oam[i] == i
```
  **Done when:** all 160 bytes match.

---

- [x] **Document allowed sources**

  **What:** WRAM/`C000` common; some regions restricted (Pan Docs).  
  **Structure:** comment + reject/ignore bad sources if needed.  
  **Min test:** DMA from `C000` works.  
  **Done when:** README notes which sources you support (table above).

---

- [ ] **Timed DMA (~160 M-cycles)**

  **What:** spread copy over time instead of instant.  
  **Structure:**
```text
Dma: active, src, index

tick(m):
  while active and m left:
    oam[index] = read(src + index)
    index += 1
    if index == 160: active = false
```
  **Min test:**
```text
start → index 0
tick(1) → one byte written
after 160 → inactive
```
  **Done when:** timing test green.

---

- [ ] **CPU access limits during DMA**

  **What:** during DMA, CPU mostly reliable for HRAM only.  
  **Structure:**
```text
if dma.active and addr not HRAM:
  read → 0xFF   # or documented open-bus
```
  **Min test:** mid-DMA WRAM read returns chosen value.  
  **Done when:** policy locked in a test.

---

- [x] **OAM usable by PPU after DMA**

  **What:** sprite data visible to PPU.  
  **Structure:** PPU reads same OAM buffer.  
  **Min test:** DMA then PPU/OBJ sample (after sprites exist).  
  **Done when:** one sprite shows from DMA’d OAM (`oam_dma_then_ppu_sees_sprite`).
