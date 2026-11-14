# soc

SoftGBC system glue — **generic** machine + `softgbc` CLI.

Wires [`sm83-iss`](../sm83-iss/) with [`softgbc-core`](../softgbc-core/) (`DmgBus` + timer/serial). Blargg (and any other suite) is just a test or a ROM path, not a separate machine type.

```text
SoftGbc { cpu, bus: DmgBus }
  step()  = cpu.step + bus.tick_m_cycles
  run()   = loop until serial Passed/Failed or max steps
```

---

## Current status *(2026-11)*

| Piece | State |
|---|---|
| Package / binary | **`soc`** crate, binary **`softgbc`** |
| Machine | [`src/machine.rs`](src/machine.rs) — `SoftGbc`, `MachineConfig`, `RunStatus` |
| CLI | `softgbc <rom.gb> [max_steps]` — any ROM; prints serial |
| Blargg | [`tests/blargg_cpu_instrs.rs`](tests/blargg_cpu_instrs.rs) — 11/11 Passed |
| Next | Real PPU (drop `stub_ly`); then window/joypad |

```bash
cargo run -p soc -- tests/roms/blargg/cpu_instrs/01-special.gb
cargo test -p soc --test blargg_cpu_instrs
```

`MachineConfig::default()` sets skip-boot `A=0x01`, `IF=0xE0`, and **`LY=144` stub** so LCD waits return without a PPU.

---

## Step 0 — Cargo workspace + SoC — **DONE**

- [x] Workspace + `soc` package
- [x] `softgbc` binary

---

## Step 1 — Load a ROM — **PARTIAL**

- [x] ROM file → `DmgBus` / `SoftGbc::from_rom_file`
- [ ] Header dump CLI mode
- [ ] `cartridge` crate API

---

## Step 2 — Bus + memory — **DONE (minimal)**

In **`softgbc-core`**: ROM, WRAM (+ echo), HRAM, VRAM, OAM, IO/`IF`, `IE`, timer, serial.

- [x] Memory regions
- [x] Bus decode
- [x] Peripheral-owned timer + serial

---

## Step 3 — CPU — **DONE**

- [x] `SoftGbc` holds `Cpu` + `DmgBus`
- [x] Skip-boot reset (`MachineConfig`)
- [x] `step` / `run`
- [ ] Optional `--steps` opcode trace

---

## Step 4 — Interrupts — **PARTIAL**

- [x] `IF` / `IE` on bus; CPU vectors
- [x] Timer raises `IF` bit 2
- [ ] Dedicated interrupt module; PPU sources

---

## Step 5 — Timer — **DONE (coarse)**

- [x] `peripherals::Timer` on `DmgBus`
- [x] Master-clock tick from `SoftGbc::step`
- [x] Blargg `02-interrupts` green

Details: [`../peripherals/src/Timer.README.md`](../peripherals/src/Timer.README.md).

---

## Step 6 — OAM DMA

- [ ] Instant memcpy on `FF46`
- [ ] Later: cycle-spread DMA

---

## Step 7 — Headless CI — **DONE**

Generic serial convention (`Passed` / `Failed`), not a Blargg-only runner.

- [x] `SoftGbc::run` + CLI
- [x] Blargg individuals as `soc` integration tests
- [ ] Combined 64K `cpu_instrs.gb` (banking)
- [ ] Optional JSONL traces

---

## Step 8 — PPU (`dmg-ppu`)

- [ ] PPU owned / ticked by machine
- [ ] Remove `stub_ly` once LY is real
- [ ] BG + CRC / PPM
- [ ] VBlank/STAT → `IF`

---

## Steps 9–14

MBC, joypad, window, APU, polish — unchanged intent; see SoftGBC root README.

---

## Suggested API (now)

```rust
pub struct SoftGbc {
    pub cpu: Cpu,
    pub bus: DmgBus,
}

impl SoftGbc {
    pub fn from_rom_file(path) -> Result<Self>;
    pub fn reset(&mut self, cfg: &MachineConfig);
    pub fn step(&mut self) -> u8;           // M-cycles
    pub fn run(&mut self, max_steps) -> RunStatus;
    pub fn serial_output(&self) -> &str;
}
```

## Master clock

```text
m = cpu.step(&mut bus);
bus.tick_m_cycles(m);   // timer (+ later ppu/dma/apu)
```

T-cycles vs M-cycles: [`../README.md`](../README.md#t-cycles-vs-m-cycles).
