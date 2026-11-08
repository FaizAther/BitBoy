# bus

16-bit address decode and routing for CPU and DMA.

## Checklist

- [ ] `read8` / `write8` API used by CPU and DMA
- [ ] Route `0000`–`7FFF` → cartridge ROM
- [ ] Route `8000`–`9FFF` → VRAM
- [ ] Route `A000`–`BFFF` → cartridge SRAM (if any)
- [ ] Route `C000`–`DFFF` → WRAM (+ echo `E000`–`FDFF`)
- [ ] Route `FE00`–`FE9F` → OAM
- [ ] Route `FF00`–`FF7F` → IO (joypad, serial, timer, IF, APU, PPU, DMA, …)
- [ ] Route `FF80`–`FFFE` → HRAM
- [ ] Route `FFFF` → `IE`
- [ ] Open-bus / unmapped behaviour documented
- [ ] DMA-vs-CPU access restrictions hook
