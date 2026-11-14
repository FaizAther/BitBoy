# boot

Optional DMG boot ROM.

**Refs:** [Pan Docs — Power Up](https://gbdev.io/pandocs/Power_Up_Sequence.html)  

## Status *(2026-11)*

SoftGBC **skip-boot**: `PC=$0100`, `A=$01`. Real boot optional.

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [x] **Skip-boot `PC=$0100`**  
- [x] **Default SoftGBC = skip-boot**

---

- [ ] **Load boot image**

  **What:** 256-byte DMG boot dump.  
  **Structure:**
```text
BootRom.data[0x100]
from_file(path)
```
  **Min test:** first byte matches known dump (often `0x31`).  
  **Done when:** load succeeds in a test.

---

- [ ] **Map `0000`–`00FF` at reset**

  **What:** CPU sees boot instead of cart in that range.  
  **Structure:**
```text
if boot_mapped and addr < 0x100:
  read → BootRom[addr]
else:
  read → cart
```
  **Min test:** after reset with boot, `read(0)==boot[0]`.  
  **Done when:** cart header at `$0104` still cart.

---

- [ ] **Unmap on `$FF50` write**

  **What:** boot disables itself.  
  **Structure:**
```text
write FF50 → boot_mapped = false
```
  **Min test:** after unmap, `read(0)` is cart ROM.  
  **Done when:** can run from `$0000` through unmap.

---

- [ ] **Document post-boot regs**

  **What:** table of AF/BC/…/IO defaults if skip-boot must match.  
  **Min test:** SoftGbc config matches table for regs you care about.  
  **Done when:** linked from SoftGBC README.
