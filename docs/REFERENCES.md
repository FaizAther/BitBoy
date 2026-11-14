# References (PDFs, guides, SoftGBC)

## SoftGBC drivers (edit these)

| Checklist | Path |
|---|---|
| Roadmap | `SoftGBC/README.md` |
| PPU | `SoftGBC/dmg-ppu/README.md` |
| Timer | `SoftGBC/peripherals/src/Timer.README.md` |
| Serial | `SoftGBC/peripherals/src/Serial.README.md` |
| SoC | `SoftGBC/soc/README.md` |

## PDFs (`docs/datasheets/`)

| File | Use |
|---|---|
| [`datasheets/gbctr.pdf`](datasheets/gbctr.pdf) | SM83 + SoC (Gekkio) |
| [`datasheets/TCAGBD.pdf`](datasheets/TCAGBD.pdf) | Old timing notes (AntonioND) |
| [`datasheets/sharp_microcomputer_databook_1996.pdf`](datasheets/sharp_microcomputer_databook_1996.pdf) | SM83 core history only |

See [`datasheets/README.md`](datasheets/README.md).

## Third-party (`docs/vendor/`)

| Guide | Path | License |
|---|---|---|
| GBEDG | `vendor/gbedg/` | MIT |
| Pan Docs | `vendor/pandocs/` | CC0 |

```bash
cd docs && ./serve.sh
```
