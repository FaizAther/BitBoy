# dmg-ppu

**Idea:** a Rust crate that is the golden **DMG PPU** (Game Boy graphics) model.

Pair with `../sm83-iss` under SoftGba to run real games in software. This repo does not own the CPU.

```text
sm83-iss ── MMIO / VRAM / OAM ──►  dmg-ppu  ── framebuffer / CRC ──► host
```

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
- APU, MBC, joypad (SoftGba)
- SoftGba SoC glue (`../soc`)
- Verilog (lives in `GBA/rtl/ppu`)

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
2. Usable as a path/submodule dependency from SoftGba `soc/`
3. No CPU opcode logic in this repo

## Build order vs CPU

Start `sm83-iss` first enough to pass CPU tests. Stand up this crate in parallel once the bus/MMIO conventions in the platform README are settled — or even earlier with a fake CPU tick harness.
