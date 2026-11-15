# References (PDFs, guides, SoftGBC)

## SoftGBC drivers (edit these)

| Checklist | Path |
|---|---|
| Roadmap | `SoftGBC/README.md` |
| PPU | `SoftGBC/dmg-ppu/README.md` |
| Timer | `SoftGBC/peripherals/src/Timer.README.md` |
| Serial | `SoftGBC/peripherals/src/Serial.README.md` |
| SoC | `SoftGBC/soc/README.md` |

## LLVM SM83 / Chunks C–D (edit these)

| Doc | Path |
|---|---|
| Approach | [`llvm/README.md`](llvm/README.md) |
| Target contracts | [`llvm/00-contracts.md`](llvm/00-contracts.md) |
| MC / assembler | [`llvm/01-mc.md`](llvm/01-mc.md) |
| Object / linker | [`llvm/02-object-link.md`](llvm/02-object-link.md) |
| Code generation | [`llvm/03-codegen.md`](llvm/03-codegen.md) |
| ABI | [`llvm/04-abi.md`](llvm/04-abi.md) |
| Runtime / driver | [`llvm/05-runtime-driver.md`](llvm/05-runtime-driver.md) |
| Validation | [`llvm/06-validation.md`](llvm/06-validation.md) |

## QEMU SM83 / Chunk E (edit these)

| Doc | Path |
|---|---|
| Approach (big picture) | [`qemu/README.md`](qemu/README.md) |
| Semantics contract | [`qemu/semantics.md`](qemu/semantics.md) |
| 00 Harness / JSONL | [`qemu/00-harness.md`](qemu/00-harness.md) |
| 01 Skeleton | [`qemu/01-skeleton.md`](qemu/01-skeleton.md) |
| 02 Helper ISA | [`qemu/02-helper-isa.md`](qemu/02-helper-isa.md) |
| 03 Interrupts | [`qemu/03-interrupts.md`](qemu/03-interrupts.md) |
| 04 Board | [`qemu/04-board.md`](qemu/04-board.md) |
| 05 Parity | [`qemu/05-parity.md`](qemu/05-parity.md) |
| 06 TCG / gdb / LLVM | [`qemu/06-tcg-debug.md`](qemu/06-tcg-debug.md) |

## GBC game SDK (edit these)

| Doc | Path |
|---|---|
| Approach | [`sdk/README.md`](sdk/README.md) |
| Project / CLI | [`sdk/00-project-cli.md`](sdk/00-project-cli.md) |
| Runtime | [`sdk/01-runtime.md`](sdk/01-runtime.md) |
| Graphics / assets | [`sdk/02-graphics-assets.md`](sdk/02-graphics-assets.md) |
| Input / loop | [`sdk/03-input-loop.md`](sdk/03-input-loop.md) |
| Audio / save / banking | [`sdk/04-audio-save-banking.md`](sdk/04-audio-save-banking.md) |
| Agent workflow | [`sdk/05-agent-workflow.md`](sdk/05-agent-workflow.md) |
| Validation / examples | [`sdk/06-validation-examples.md`](sdk/06-validation-examples.md) |

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
