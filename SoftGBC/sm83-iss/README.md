# sm83-iss

**Idea:** a Rust crate that is the golden **SM83 CPU** model for the Game Boy platform.

Not a full emulator. Not graphics. Just the CPU — correct, testable, and reusable as a library (nested under `SoftGBC/`, later a git submodule).

```text
.gb / hex fixture / Blargg ROM
      ↓
  softgbc_core::Bus (injected by host)
      ↓
  sm83-iss::Cpu::step()
      ↓
  (later) JSONL traces → trace-diff ← QEMU / Verilator / FPGA
```

---

## Current status *(2026-11)*

| Item | State |
|---|---|
| Registers / flags | **Done** |
| Full base ISA + `CB` prefix | **Done** |
| `EI` / `DI` / `HALT` + IRQ vectoring | **Done** |
| `Bus` via `softgbc_core::Bus` | **Done** (`Option`-based read/write) |
| Opcode smoke tests | **Done** — `tests/every_opcode.rs` (~511 tests) + in-crate unit tests |
| Hex fixtures | Early set present (`01`–`03`); ISA growth moved to white-box tests |
| Trace CLI / JSONL | **Not started** |
| Blargg via this crate alone | Prefer SoftGBC `soc` runner; CPU is ready |

```bash
cd SoftGBC
cargo test -p sm83-iss
```

Blargg individuals (except interrupts) already pass through `soc` using this CPU. Remaining SoftGBC gap is **timer**, not missing opcodes.

---

## Why Rust

- Safe, explicit state machine for an ISA-heavy codebase
- Easy library boundary (`sm83_iss` crate) for SoftGBC SoC
- Good test story (per-opcode tests + Blargg via SoC)
- Optional `no_std`-friendly core later if we care; start with `std`

## In scope

- [x] Registers: `AF BC DE HL SP PC`
- [x] Flags: `Z N H C`
- [x] Full instruction set including `CB` prefix
- [x] Interrupts, `EI` / `DI` / `HALT` / `STOP` (basic)
- [x] Injectable memory via a `Bus` trait
- [ ] Instruction tracing (JSONL) and optional cycle-accurate timing polish
- [ ] Dedicated CLI binary: `sm83-iss <rom> [--trace out.jsonl]`
- [x] Hex fixtures under [`fixtures/`](fixtures/) (early ladder; still useful)

## Instruction-by-instruction workflow

1. Add/extend a file in `fixtures/` with only the bytes you need.
2. Load via `softgbc_core` hex loader; set `PC` to `# entry:`.
3. `step()` and assert in a unit test.
4. Or extend `tests/every_opcode.rs` for a single-opcode smoke.

See [`fixtures/README.md`](fixtures/README.md).

## Out of scope (other crates / `GBA/`)

| Concern | Where |
|---|---|
| PPU / LCD | `../dmg-ppu` |
| APU / audio | SoftGBC `apu/` / `soc/` |
| MBC / cart banking | SoftGBC `cartridge/` |
| Timer / DMA / joypad | SoftGBC peripheral folders + `soc` |
| Verilog / FPGA | `GBA/rtl/` |
| LLVM / QEMU | forks under `GBA/` |

## Actual API *(today)*

```rust
// Bus lives in softgbc_core (not inside this crate):
//   fn read8(&self, addr: u16) -> Option<u8>;
//   fn write8(&mut self, addr: u16, data: u8) -> bool;

pub struct Cpu { /* a f b c d e h l sp pc ime halted … */ }

impl Cpu {
    pub fn new() -> Self;
    /// Execute one instruction; return M-cycles (`u8`).
    pub fn step(&mut self, bus: &mut impl Bus) -> u8;
}
```

`DmgBus` in `softgbc-core` is the usual host for unit tests and SoftGBC.

## Crate layout *(as implemented)*

```text
sm83-iss/
├── Cargo.toml
├── src/
│   └── lib.rs              # Cpu + decode/execute (single file for now)
├── tests/
│   └── every_opcode.rs     # one smoke test per opcode (+ invalids)
├── fixtures/
│   ├── 01_nop.hex
│   ├── 02_jr_spin.hex
│   ├── 03_ld_imm.hex
│   └── README.md
└── README.md
```

Split into `cpu.rs` / `ops.rs` / `trace.rs` when tracing or file size demands it — not required for correctness.

## Done means (v1)

1. [x] Full SM83 ISA with tests (Blargg individuals via SoftGBC nearly green)
2. [ ] SoftGBC Blargg `02-interrupts` + agreed CPU set fully green *(SoC timer, not more opcodes)*
3. [ ] Stable trace format documented (PC, regs, flags, opcode, cycles, mem writes)
4. [x] `Cargo` library usable from SoftGBC via path
5. [x] No PPU code in this crate

## Relationship to the rest of the stack

```text
Chunk B  →  this crate (software CPU oracle)     ← largely here
Chunk F  →  Verilog CPU must match these traces
Chunk H  →  FPGA HIL must match these traces
Chunk I+ →  SoftGBC + dmg-ppu run real games using this CPU
```

Build the brain here. Wire the body in SoftGBC (`soc`, timer, PPU, …).
