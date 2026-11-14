# Datasheets & CPU references

There is **no public Nintendo/Sharp “Game Boy CPU datasheet”** PDF. The core is identified as Sharp **SM83** (same ISA family as the SM831x appliance MCUs). Use these instead.

PDFs in this folder are copied into the HTML site when you run `docs/serve.sh` (see [`../README.md`](../README.md)). SoftGBC explainers live under SoftGBC crates and are linked from that site.

## Local PDFs

| File | What it is | Use for |
|---|---|---|
| [`gbctr.pdf`](gbctr.pdf) | **Game Boy: Complete Technical Reference** (Gekkio) — best modern SM83 + GB SoC doc | CPU microarchitecture, ISA, timing, SoC behaviour |
| [`TCAGBD.pdf`](TCAGBD.pdf) | **The Cycle-Accurate Game Boy Docs** (AntonioND) | Timing intuition — **old / unmaintained**; verify against hardware/Pan Docs |
| [`sharp_microcomputer_databook_1996.pdf`](sharp_microcomputer_databook_1996.pdf) | Sharp *Microcomputer Data Book* (1996) — SM8311/13/14/15 embed an **SM83** core | Historical SM83 core description only — **not** Game Boy PPU/APU/IO |

## Online references (also on the docs site when vendored)

| Link | What | Use for |
|---|---|---|
| [Pan Docs](https://gbdev.io/pandocs/) | Community Game Boy hardware bible (CC0) | Memory map, PPU, APU, cart, IO |
| [GBEDG](https://github.com/Ashiepaws/GBEDG) | Emulator-oriented guide (MIT) | PPU, timers, DMA, MBCs with diagrams |
| [Opcode tables](https://gbdev.io/gb-opcodes/optables/) | Compact SM83 opcode chart | Decode / cycle counts |
| [gbz80(7)](https://rgbds.gbdev.io/docs/master/gbz80.7) | Per-instruction text | What each opcode does |
| [Gekkio sm83-cpu-core](https://github.com/Gekkio/gb-research/tree/main/sm83-cpu-core) | Die / RE notes | Deep CPU core research |

Browse locally after `cd SoftGBC/docs && ./serve.sh` → [datasheets on the site](http://127.0.0.1:8000/datasheets/).

## Important distinction

- **SM8311 / SM8313 / …** = Sharp *appliance* MCUs that *contain* an SM83 core + their own ROM/RAM/ADC/timers.
- **Game Boy** = custom Sharp SoC (LR35902 / DMG-CPU) with an **SM83-compatible CPU core**, plus PPU/APU/etc. that are **not** in the SM831x datasheets.

Same **CPU core name/ISA lineage**, different chip. Do **not** implement SoftGBC peripherals from the SM831x pages.

## How SoftGBC should use these

```text
Learn pages / SoftGBC READMEs  →  what we built + next milestone
GBEDG / Pan Docs               →  hardware behaviour
gbctr.pdf                      →  CPU + SoC when IRQ/timing gets weird
TCAGBD.pdf                     →  optional historical timing notes
Sharp 1996                     →  SM83 core history only
```

## Sources (upstream)

- gbctr: https://gekkio.fi/files/gb-docs/gbctr.pdf  
- TCAGBD: https://github.com/AntonioND/giibiiadvance/tree/master/docs  
- Sharp 1996 databook: https://bitsavers.org/components/sharp/_dataBooks/1996_Sharp_Microcomputer_Data_Book.pdf  
