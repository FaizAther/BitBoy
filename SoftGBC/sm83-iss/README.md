# sm83-iss

**Idea:** a Rust crate that is the golden **SM83 CPU** model for the Game Boy platform.

Not a full emulator. Not graphics. Just the CPU — correct, traceable, and reusable as a library (nested under `SoftGba/`, later a git submodule).

```text
.gb / test ROM
      ↓
  Bus trait (injected by host)
      ↓
  sm83-iss  ──JSONL traces──►  trace-diff  ◄── QEMU / Verilator / FPGA
      ↓
  step() / run()
```

## Why Rust

- Safe, explicit state machine for an ISA-heavy codebase
- Easy library boundary (`sm83_iss` crate) for the platform SoC harness
- Good test story (unit tests per opcode family + Blargg ROM integration tests)
- Optional `no_std`-friendly core later if we care; start with `std`

## In scope

- Registers: `AF BC DE HL SP PC`
- Flags: `Z N H C`
- Full instruction set including `CB` prefix
- Interrupts, `EI` / `DI` / `HALT` / `STOP`
- Injectable memory via a `Bus` trait (read8 / write8)
- Instruction tracing (and optional cycle-accurate timing later)
- CLI binary: `sm83-iss <rom> [--trace out.jsonl]`
- **Hex fixtures** under [`fixtures/`](fixtures/) — build opcodes one at a time

## Instruction-by-instruction workflow

1. Add/extend a file in `fixtures/` with only the bytes you need.
2. Implement a hex loader + that opcode in the CPU yourself.
3. Set `PC` to `# entry:`, `step()` and assert.
4. Uncomment the next instruction in the fixture when ready.

See [`fixtures/README.md`](fixtures/README.md).

## Out of scope (other crates / `GBA/`)

| Concern | Where |
|---|---|
| PPU / LCD | `../dmg-ppu` |
| APU / audio | SoftGba `apu/` / `soc/` |
| MBC / cart banking | SoftGba `cartridge/` |
| Verilog / FPGA | `GBA/rtl/` |
| LLVM / QEMU | forks under `GBA/` |

## Sketch API

```rust
pub trait Bus {
    fn read8(&mut self, addr: u16) -> u8;
    fn write8(&mut self, addr: u16, data: u8);
}

pub struct Cpu<B: Bus> { /* regs, ime, halted, bus */ }

impl<B: Bus> Cpu<B> {
    pub fn new(bus: B) -> Self;
    pub fn reset(&mut self);
    /// Execute one instruction; return machine cycles (M-cycles).
    pub fn step(&mut self) -> u32;
    pub fn set_tracer(&mut self, tracer: Option<Box<dyn Tracer>>);
}
```

Flat 64KiB RAM+ROM stub implements `Bus` for CPU unit tests. The real Game Boy map lives in SoftGba (`bus/`, `memory/`, `soc/`) and wraps this CPU + `dmg-ppu`.

## Crate layout (when implemented)

```text
sm83-iss/
├── Cargo.toml          # lib + bin
├── src/
│   ├── lib.rs
│   ├── cpu.rs
│   ├── decode.rs
│   ├── ops.rs          # ALU, loads, jumps, CB
│   ├── interrupts.rs
│   ├── trace.rs
│   └── bin/sm83-iss.rs
├── tests/
│   ├── opcode_smoke.rs
│   └── blargg.rs       # needs tests/roms/
└── README.md
```

## Done means (v1)

1. Blargg `cpu_instrs` passes
2. Stable trace format documented (PC, regs, flags, opcode, cycles, mem writes)
3. `Cargo` library usable from SoftGba via path or submodule
4. No PPU code in this repo

## Relationship to the rest of the stack

```text
Chunk B  →  this crate (software CPU oracle)
Chunk F  →  Verilog CPU must match these traces
Chunk H  →  FPGA HIL must match these traces
Chunk I+ →  SoftGba + dmg-ppu run real games using this CPU
```

Build the brain here. Wire the body in SoftGba.
