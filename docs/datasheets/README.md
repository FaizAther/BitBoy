# Datasheets & CPU references

There is **no public Nintendo/Sharp “Game Boy CPU datasheet”** PDF. The core is identified as Sharp **SM83** (same ISA family as the SM831x appliance MCUs). Use these instead:

## What to use for SoftGba / `sm83-iss`

| File / link | What it is | Use for |
|---|---|---|
| [`gbctr.pdf`](gbctr.pdf) | **GB: Complete Technical Reference** (Gekkio) — best modern SM83 + GB SoC doc | CPU microarchitecture, ISA, timing, SoC behaviour |
| [`sharp_microcomputer_databook_1996.pdf`](sharp_microcomputer_databook_1996.pdf) | Sharp *Microcomputer Data Book* (1996) — includes **SM8311/13/14/15** which embed an **SM83** CPU core | Historical Sharp description of the SM83 core (not Game Boy peripherals) |
| [Pan Docs](https://gbdev.io/pandocs/) | Community Game Boy hardware bible | Memory map, PPU, APU, cart, IO |
| [Opcode tables](https://gbdev.io/gb-opcodes/optables/) | Compact SM83 opcode chart | Implementing decode / cycles |
| [gbz80(7)](https://rgbds.gbdev.io/docs/master/gbz80.7) | Per-instruction text | What each opcode does |
| [Gekkio sm83-cpu-core](https://github.com/Gekkio/gb-research/tree/main/sm83-cpu-core) | Die / RE notes | Deep CPU core research |

## Important distinction

- **SM8311 / SM8313 / …** = whole Sharp *appliance* microcontrollers that *contain* an SM83 core + their own ROM/RAM/ADC/timers.
- **Game Boy** = custom Sharp SoC (often called LR35902 / DMG-CPU) that also uses an **SM83-compatible CPU core**, plus PPU/APU/etc. that are **not** in the SM831x datasheets.

So: same **CPU core name/ISA lineage**, different chip. Do **not** implement SoftGba peripherals from the SM831x pages.

## Sources

- gbctr: https://gekkio.fi/files/gb-docs/gbctr.pdf  
- Sharp 1996 databook: https://bitsavers.org/components/sharp/_dataBooks/1996_Sharp_Microcomputer_Data_Book.pdf  
