# memory

On-console RAM / ROM buffers (`memory.rs`).

**Refs:** [Pan Docs — Memory map](https://gbdev.io/pandocs/Memory_Map.html)  

## Status *(2026-11)*

ROM / WRAM / HRAM done on `DmgBus`. Echo / shared VRAM-OAM policies still to firm up.

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [x] **WRAM 8KiB**  
- [x] **HRAM**  
- [x] **Load helpers**

---

- [ ] **Echo RAM `E000`–`FDFF`**

  **What:** mirrors `C000`–`DDFF`.  
  **Structure:**
```text
access E000..FDFF → same as addr - 0x2000 in WRAM
```
  **Min test:**
```text
write C000 = 0x42 → read E000 == 0x42
write E001 = 0x99 → read C001 == 0x99
```
  **Done when:** both directions pass.

---

- [ ] **VRAM shared with PPU**

  **What:** one 8KiB buffer CPU and PPU both use.  
  **Structure:** bus (or Ppu) owns `[0x2000]`; both paths use it.  
  **Min test:** CPU write `8000` visible to PPU fetch.  
  **Done when:** BG test can see poked tiles.

---

- [ ] **OAM shared with PPU/DMA**

  **What:** 160 bytes at `FE00`.  
  **Structure:** single buffer; DMA writes; PPU reads.  
  **Min test:** write/DMA then read back.  
  **Done when:** sprite path uses same bytes.

---

- [ ] **Dump helpers (optional)**

  **What:** debug hex dump of WRAM/VRAM.  
  **Min test:** after write, dump contains expected bytes.  
  **Done when:** useful in a failing test session.
