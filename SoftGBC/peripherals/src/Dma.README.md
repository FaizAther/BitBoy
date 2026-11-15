# DMA

OAM DMA via `FF46`.

**Refs:** [Pan Docs — OAM DMA](https://gbdev.io/pandocs/OAM_DMA_Transfer.html) · [GBEDG — DMA](https://github.com/Ashiepaws/GBEDG/blob/master/dma/index.md) · [`dma.rs`](dma.rs)  

## Status *(2026-11)*

**Timed OAM DMA wired** on `DmgBus`: write `FF46` starts a 160 M-cycle transfer
(1 byte / M via `tick_t_cycles` / `tick_m_cycles`). While active, CPU access outside
HRAM / `FF46` is blocked (reads `$FF`, writes ignored).

```text
write FF46 = src_hi
  → active, index = 0, src = src_hi << 8
each M-cycle while active:
  oam[index] = read(src + index); index++
  if index == 160: active = false
```

### Allowed sources (SoftGBC)

| Source | SoftGBC |
|---|---|
| WRAM `$C000`–`$DFFF` | **Supported** (common game path) |
| ROM / VRAM / other via bus raw read | Whatever the map returns |
| Echo WRAM `$E000`–`$FDFF` | Follows bus echo mapping |

### CPU during DMA

| Region | Read | Write |
|---|---|---|
| HRAM `$FF80`–`$FFFE` | ok | ok |
| `FF46` | ok (page) | restarts DMA |
| Everything else | `$FF` | ignored |

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

- [x] **Timed DMA (~160 M-cycles)**

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
  **Done when:** timing test green (`timed_dma_one_byte_per_m_cycle`, bus ticks).

---

- [x] **CPU access limits during DMA**

  **What:** during DMA, CPU mostly reliable for HRAM only.  
  **Structure:**
```text
if dma.active and addr not HRAM:
  read → 0xFF   # or documented open-bus
```
  **Min test:** mid-DMA WRAM read returns chosen value.  
  **Done when:** policy locked in a test (`oam_dma_cpu_access_limited_to_hram`).

---

- [x] **OAM usable by PPU after DMA**

  **What:** sprite data visible to PPU.  
  **Structure:** PPU reads same OAM buffer.  
  **Min test:** DMA then PPU/OBJ sample (after sprites exist).  
  **Done when:** one sprite shows from DMA’d OAM (`oam_dma_then_ppu_sees_sprite`).
