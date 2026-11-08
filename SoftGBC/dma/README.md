# dma

OAM DMA via `FF46`.

## Checklist

- [ ] Write to `FF46` starts transfer
- [ ] Copy 160 bytes to OAM
- [ ] Source from ROM/WRAM (document blocked regions)
- [ ] Takes ~160 M-cycles
- [ ] CPU memory access restrictions during DMA
- [ ] Bus arbitration with PPU/CPU documented
- [ ] Test: sprite data appears in OAM after DMA
