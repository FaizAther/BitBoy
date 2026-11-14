# dmg-ppu

**Idea:** a Rust crate that is the golden **DMG PPU** (Game Boy graphics) model.

Pair with `../sm83-iss` under SoftGBC to run real games in software. This crate does not own the CPU.

```text
sm83-iss ── MMIO / VRAM / OAM ──►  dmg-ppu  ── framebuffer / CRC ──► host
```

---

## Current status *(2026-11)*

| Item | State |
|---|---|
| Crate in workspace | **Yes** (empty `lib.rs` stub) |
| LCD timing / modes | Not started |
| BG / window / sprites | Not started |
| Framebuffer / CRC | Not started |
| SoftGBC integration | SoC fakes `LY=144` for Blargg only — **not** this crate |

**Do not start serious PPU work until** SoftGBC Blargg `02-interrupts` is green (timer + IRQ path). Then follow SoftGBC roadmap step 8: LY → BG → PPM/CRC → window/sprites → `dmg-acid2`.

---

## Why a separate crate

- CPU and graphics can advance in parallel
- RTL/FPGA PPU can match frame CRCs the same way CPU RTL matches ISS traces
- Keeps `sm83-iss` small

## In scope

- LCD timing (154 lines × 456 T-cycles)
- Modes 0–3
- Background, window, sprites
- `0xFF40`–`0xFF4B` IO registers
- 160×144 framebuffer + frame CRC
- STAT / VBlank IRQ signals for the SoC to raise CPU interrupts

## Out of scope

- SM83 CPU (`sm83-iss`)
- APU, MBC, joypad (SoftGBC)
- SoftGBC SoC glue (`../soc`)
- Verilog (lives in `GBA/rtl/ppu`)

## Checklist (v1)

- [ ] `Ppu` struct + `tick(t_cycles)`
- [ ] LY / mode FSM (456 T-cycles × 154 lines)
- [ ] IO regs `FF40`–`FF4B` read/write
- [ ] Background render → framebuffer
- [ ] Window
- [ ] Sprites
- [ ] `frame_crc()` / PPM dump hook from SoC
- [ ] `take_vblank_irq` / `take_stat_irq`
- [ ] SoftGBC `soc` owns PPU + shares VRAM/OAM
- [ ] `dmg-acid2` (or golden CRC) matches

## Sketch API

```rust
pub struct Ppu { /* lcdc, stat, ly, fb, ... */ }

impl Ppu {
    pub fn new(vram: SharedVram, oam: SharedOam) -> Self;
    pub fn tick(&mut self, t_cycles: u32);
    pub fn read_io(&self, addr: u16) -> u8;
    pub fn write_io(&mut self, addr: u16, data: u8);
    pub fn framebuffer(&self) -> &[u32; 160 * 144];
    pub fn frame_crc(&self) -> u32;
    pub fn take_vblank_irq(&mut self) -> bool;
    pub fn take_stat_irq(&mut self) -> bool;
}
```

## Done means (v1)

1. `dmg-acid2` CRC (or golden framebuffer) matches
2. Usable as a path/submodule dependency from SoftGBC `soc/`
3. No CPU opcode logic in this repo

## Build order vs CPU

`sm83-iss` is already ISA-complete enough for Blargg. Stand up this crate **after** SoftGBC timer/`02-interrupts`, starting with LY-only + a fake CPU tick harness if needed.
