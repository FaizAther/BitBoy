# cartridge

ROM image + MBC + battery RAM.

**Refs:** [Pan Docs — Header](https://gbdev.io/pandocs/The_Cartridge_Header.html) · [GBEDG — MBCs](https://github.com/Ashiepaws/GBEDG/blob/master/mbcs/index.md)  

## Status *(2026-11)*

Stub. SoftGBC loads `.gb` via `DmgBus::from_rom_file` (32 KiB window).

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [x] **Load `.gb` bytes** *(via softgbc-core today)*

---

- [ ] **Cartridge API wrap**

  **What:** this crate owns load + header.  
  **Structure:**
```text
Cartridge { data, header }
from_path(path) → Cartridge
```
  **Min test:** load Blargg `01-special.gb` → data length ≥ 0x150.  
  **Done when:** SoftGBC can construct from this API.

---

- [ ] **Parse header**

  **What:** title, cart type `0x0147`, size codes.  
  **Structure:**
```text
Header: title, cart_type, rom_size_code, ram_size_code
```
  **Min test:** known ROM → expected type / non-empty title.  
  **Done when:** fields match file bytes.

---

- [ ] **ROM-only mapping**

  **What:** type `$00` — flat map `0000–7FFF`.  
  **Structure:** read → `data[addr]`; write ROM ignored.  
  **Min test:** read `0x0100` == file byte.  
  **Done when:** Blargg individuals still run.

---

- [ ] **MBC1 banking**

  **What:** writes select ROM/RAM banks (GBEDG MBC1).  
  **Structure:**
```text
on write to low ROM:
  update rom_bank / ram_enable / mode
read 4000–7FFF from banked region of data
```
  **Min test:** switch bank → `read(0x4000)` changes on 64K+ image.  
  **Done when:** combined Blargg or target game banks correctly.

---

- [ ] **SRAM + `.sav`**

  **What:** `A000–BFFF` when enabled; persist file.  
  **Structure:** enable → RAM r/w; save/load path.  
  **Min test:** write RAM → save → reload → same byte.  
  **Done when:** round-trip works for one cart type.

---

- [ ] **Bus hooks + reject unsupported**

  **What:** `DmgBus` uses Cartridge; warn on bad type.  
  **Min test:** unsupported type → error or clear warning.  
  **Done when:** documented behaviour.
