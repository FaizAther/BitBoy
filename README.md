# SM83 Platform

An end-to-end, open-source Game Boy–class development platform:

**C / Rust / Zig → LLVM IR → SM83 machine code → QEMU / SameBoy / Verilog / FPGA**

The project has four major components:

1. **SM83 LLVM backend** — compile high-level languages to Game Boy CPU code.
2. **QEMU SM83 target** — run and debug compiled programs in a system emulator.
3. **SM83 Verilog/SystemVerilog RTL** — implement the CPU and Game Boy-compatible SoC.
4. **FPGA implementation and verification** — run the same ROMs on real hardware and prove RTL behaviour matches the software models.

```text
C / Rust / Zig frontend
        ↓
      LLVM IR
        ↓
SM83 LLVM backend
        ↓
SM83 assembly / ELF / .gb ROM
        ↓
QEMU SM83 target  ←  SameBoy / mGBA  ←  Verilog simulation  ←  FPGA hardware
                                                      ↑
                                              FPGA verification
                                         (HIL, trace capture, compare)
```

## Goals

- Implement an **SM83 / Game Boy CPU** target for LLVM.
- Implement a custom **QEMU system target** for SM83.
- Write a **cycle-accurate SM83 CPU core** in Verilog or SystemVerilog.
- Build a Game Boy–compatible SoC: CPU, memory map, PPU, APU, timers, interrupts, DMA, and cartridge interface.
- Validate every layer using the same test ROMs and compiled programs.
- **Verify RTL on FPGA**, not only in simulation: same tests, captured traces, and host-side compare against ISS/QEMU.
- Run the final design on an FPGA with video and audio output.

## Non-goals

- Not trying to make a commercially compatible Nintendo product.
- Not aiming for perfect compatibility with every commercial Game Boy game in the first releases.
- Not implementing a new high-level language; the first frontend is C through LLVM.
- Not optimising for minimum FPGA resource usage before correctness and debuggability.

---

## Repositories

Work is split across repos so CPU, graphics, compiler, and SoC can move in parallel. The **platform** repo wires them together (bus, MBCs, APU, FPGA, CI) and consumes the small libraries as **git submodules**.

```text
GBA/   (platform root — everything lives under here)
├── SoftGba/              # software Game Boy
│   ├── sm83-iss/         # nested git repo — Rust CPU
│   ├── dmg-ppu/          # nested git repo — Rust PPU
│   ├── apu/ bus/ …       # other virtual hardware
│   └── soc/              # wires ISS + PPU + peripherals
├── rtl/                  # Verilog CPU + PPU + SoC
├── runtime/
├── tools/
├── tests/
├── llvm-project/         # optional local LLVM fork
└── qemu/                 # optional local QEMU fork
```

| Path under `GBA/` | Role | Own git repo? |
|---|---|---|
| **.** (platform) | Umbrella: SoftGba, RTL, FPGA HIL, tools | yes |
| **`SoftGba/`** | Software DMG machine | no — lives here |
| **`SoftGba/sm83-iss/`** | SM83 CPU ISS crate | yes — submodule later |
| **`SoftGba/dmg-ppu/`** | DMG PPU crate | yes — submodule later |
| **`llvm-project/`** | SM83 LLVM backend (Chunks C–D) | yes — heavy fork, optional |
| **`qemu/`** | SM83 QEMU target (Chunk E) | yes — heavy fork, optional |

### How a full game runs (software path)

```text
.gb ROM
   ↓
 SoftGba/soc
   ├── SoftGba/sm83-iss
   ├── SoftGba/dmg-ppu
   ├── bus, memory, cartridge, timer, dma, interrupt, joypad, apu, …
   ↓
 framebuffer / audio / input
```

See [`SoftGba/README.md`](SoftGba/README.md) for the full component list.

### Nested repos → submodules later

`SoftGba/sm83-iss/` and `SoftGba/dmg-ppu/` are their own git repos. When the umbrella is published:

```bash
# from GBA/, after remotes exist
git submodule add git@github.com:<you>/sm83-iss.git SoftGba/sm83-iss
git submodule add git@github.com:<you>/dmg-ppu.git  SoftGba/dmg-ppu
git submodule update --init --recursive
```

---

## Project chunks

The work is split into **chunks**: scoped packages that can ship independently, with clear owners, dependencies, and “done” definitions. Chunks reduce the risk of trying to build the whole diagram at once.

```text
Chunk A  Tooling & harness          ──────────────────────────────┐
Chunk B  Reference ISS              depends on A                  │
Chunk C  LLVM assembler / MC        depends on A                  │
Chunk D  LLVM codegen + runtime     depends on B, C               │
Chunk E  QEMU SM83 target           depends on B                  │
Chunk F  RTL CPU core               depends on A, B               │
Chunk G  RTL memory / bus / DMA     depends on F                  │
Chunk H  FPGA verification (CPU)    depends on A, B, F            │  ← gate before SoC
Chunk I  PPU / APU / peripherals    depends on G, H               │
Chunk J  Full SoC + FPGA bring-up   depends on D/E, I, H          ┘
```

| Chunk | Name | Delivers | Depends on | Parallel with |
|---|---|---|---|---|
| **A** | Tooling & harness | Build scripts, ROM format, `trace-diff`, CI skeletons | — | everything later |
| **B** | Reference ISS | Trusted SM83 instruction-set simulator + traces | A | C, F (after A) |
| **C** | LLVM assembler / MC | `llvm-mc`, encodings, object/ROM output | A | B |
| **D** | LLVM codegen + runtime | C → SM83, crt0, linker scripts, helpers | B, C | E |
| **E** | QEMU SM83 | `qemu-system-sm83`, TCG, debug exit MMIO | B | D, F |
| **F** | RTL CPU core | Cycle-accurate SM83 in Verilog/SV + Verilator/cocotb | A, B | E |
| **G** | RTL memory system | Address map, WRAM/HRAM/VRAM/OAM, MBC stub, DMA | F | — |
| **H** | FPGA verification (CPU) | Synthesizable CPU harness, HIL tests, hardware traces | A, B, F | G (after F green in sim) |
| **I** | Graphics / audio / I/O | PPU, APU, timers, joypad, serial | G, H | — |
| **J** | Full FPGA SoC | Board support, video/audio out, end-to-end ROMs | D or E, I, H | — |

### Chunk A — Tooling & harness

**Scope:** shared infrastructure only. No CPU semantics beyond loading bytes.

- Repo layout, CMake/Ninja wrappers, lint/format hooks.
- Canonical test-ROM container (header, entry point, exit MMIO convention).
- `tools/trace-diff`: compare PC/regs/flags/memory-write streams.
- CI: “build empty targets + run harness smoke”.

**Done when:** a hand-assembled ROM loads in a stub runner and `trace-diff` can compare two identical synthetic traces.

### Chunk B — Reference ISS (`sm83-iss` repo)

**Scope:** software golden model for SM83 — **separate repository**, consumed as a submodule.

- Registers, flags, injectable bus, base ALU, CB prefix, interrupts, `HALT`/`STOP`.
- Instruction-level and optional cycle-level tracing.
- Blargg `cpu_instrs` + Mooneye CPU subset.
- No PPU/APU here.

**Done when:** ISS is the authority for CPU tests; all later backends must match its traces (or document intentional differences).

### Chunk C — LLVM assembler / MC

**Scope:** assemble and emit SM83 machine code; no high-level codegen yet.

- Target registration, registers, TableGen encodings.
- Asm parser/printer, MC emitter, flat ROM or ELF + linker script.

**Done when:** `llvm-mc` turns hand-written SM83 asm into a ROM the ISS runs.

### Chunk D — LLVM codegen + runtime

**Scope:** compile simple C to correct SM83.

- Instruction selection, calling convention, stack frames, runtime helpers.
- `runtime/crt0`, minimal libc stubs, linker scripts.

**Done when:** the sample `sum(20,22)` program writes `0x2E` to `0xFF80` under the ISS.

### Chunk E — QEMU SM83

**Scope:** fast system emulator + debugger for the same ROMs.

- `target/sm83` TCG, minimal machine, ROM load, debug-exit device.
- Trace hooks compatible with Chunk A.

**Done when:** LLVM-built (or hand) ROMs that pass ISS also pass QEMU with matching exit status / key state.

### Chunk F — RTL CPU core

**Scope:** Verilog/SystemVerilog SM83 core only; Verilator + cocotb.

- Regfile, ALU, fetch/decode/execute, memory-cycle sequencing, interrupts.
- Same CPU test ROMs as ISS; cycle/trace compare via `trace-diff`.

**Done when:** Verilator RTL matches ISS on Blargg CPU tests (or the agreed CPU subset).

### Chunk G — RTL memory system

**Scope:** Game Boy address map around the CPU.

- Decoder, WRAM/HRAM/VRAM/OAM/IO, cartridge ROM port, OAM DMA, MBC1 first.

**Done when:** banked ROM load + DMA behave correctly in simulation vs ISS/QEMU where modelled.

### Chunk H — FPGA verification (CPU)

**Scope:** prove the **same Verilog** that passes Verilator also passes on silicon — before investing in PPU/APU.

This chunk is mandatory, not optional bring-up polish.

Deliverables:

1. **Synthesizable CPU test harness** (`rtl/fpga/cpu_harness`)
   - Soft CPU + tiny memory (BRAM) + ROM image + exit/status MMIO.
   - Host link: UART (default) or SPI/USB-FIFO for commands and trace upload.
   - Optional: LED/HEX for coarse pass/fail; always keep a machine-readable protocol.

2. **Host-side FPGA runner** (`tools/fpga-runner`)
   - Load bitstream (or use pre-programmed flash).
   - Upload test ROM into BRAM / cartridge emulation window.
   - Start / reset / step (if supported) / collect exit code.
   - Stream or dump execution traces (PC, regs, flags, selected bus writes).

3. **Hardware-in-the-loop (HIL) compare**
   - For each test ROM: ISS trace → FPGA trace → `trace-diff`.
   - Fail CI/nightly if hardware diverges from ISS beyond documented allowances (e.g. reset latency).

4. **Synthesis / timing gate**
   - Yosys/nextpnr or vendor flow for a chosen board.
   - Timing report checked into CI artifacts (or a “last known good” job).
   - Resource utilisation baseline so later SoC work has a budget.

5. **Debug observability**
   - Lightweight ILA/embedded logic analyser *or* a cycle-counter + last-N PC ring buffer readable over UART.
   - Document how to capture a failing test from the board in under 15 minutes.

**Done when:** the Blargg/Mooneye CPU set that passes Verilator also passes on FPGA with traces matching the ISS (same pass criteria as Chunk F).

**Why this comes before PPU:** PPU timing bugs are expensive; locking CPU+bus on hardware first keeps later failures localised.

### Chunk I — PPU / APU / peripherals

**Scope:** remaining SoC blocks, still simulation-first, then extend FPGA verification.

- **Software PPU** lives in the separate **`dmg-ppu`** repo (submodule); platform wires it to `sm83-iss` for playable ROMs.
- RTL PPU/APU stay under `rtl/` in this platform repo (or a future `dmg-ppu-rtl` split if needed).
- APU channels, joypad, serial, timers start in the platform.
- Reuse Chunk H harness patterns: framebuffer/audio dump over the host link where feasible.

**Done when:** `dmg-acid2` (and agreed audio/timer tests) pass in software (`dmg-ppu`) and Verilator; FPGA HIL extended for at least frame CRC / golden framebuffer compare.

### Chunk J — Full FPGA SoC bring-up

**Scope:** playable/demo path on board.

- Clocks/reset/PLL, BRAM map, ROM load from SPI/SD/host, HDMI/VGA/audio, timing closure.
- End-to-end: LLVM (or hand) ROM → QEMU → Verilator → FPGA, visible match.

**Done when:** one LLVM-compiled demo ROM shows equivalent behaviour on QEMU, RTL sim, and FPGA.

### Suggested build order

1. **A → B** (golden model).
2. **C → D** in parallel with **F** once B can score CPU tests.
3. **E** when you need a faster debugger than the ISS.
4. **F green in sim → H immediately** (FPGA CPU verification).
5. **G → I → J**, extending HIL as each block lands.

Do not start Chunk I until Chunk H’s CPU HIL is green.

---

## Repository layout

```text
GBA/                        # Everything under this root
├── SoftGba/
│   ├── sm83-iss/           # nested git repo → CPU ISS
│   ├── dmg-ppu/            # nested git repo → PPU
│   ├── apu/ bus/ memory/ … # other virtual hardware
│   └── soc/                # SoftGba glue / runner
├── rtl/                    # FPGA / simulation RTL
│   ├── cpu/
│   ├── bus/
│   ├── memory/
│   ├── ppu/
│   ├── apu/
│   ├── peripherals/
│   ├── top/
│   ├── fpga/
│   └── tb/
├── runtime/
├── tests/
├── tools/
├── docs/
├── llvm-project/
├── qemu/
├── .github/workflows/
├── .gitmodules
├── repos.txt
├── LICENSE
├── CONTRIBUTING.md
└── README.md
```

LLVM/QEMU forks clone **into** `GBA/` when you need them (see `repos.txt`); they are optional until Chunks C–E.

## Current status

| Component | Chunk | Status |
|---|---|---|
| Tooling & harness | A | Planned |
| SM83 reference ISS | B | Planned |
| LLVM assembler / MC layer | C | Planned |
| LLVM code-generation backend | D | Planned |
| QEMU SM83 CPU target | E | Planned |
| QEMU Game Boy machine model | E | Planned |
| Verilog SM83 CPU | F | Planned |
| Verilog memory system | G | Planned |
| FPGA CPU verification (HIL) | H | Planned |
| PPU | I | Planned |
| APU | I | Planned |
| Full FPGA SoC bring-up | J | Planned |

## Milestones

Milestones map onto chunks. Prefer finishing a chunk’s exit criteria over starting the next diagram box.

### M0 — Foundation (Chunk A)

**Goal:** establish the toolchain and a minimal executable test path.

- [ ] Set up LLVM build environment.
- [ ] Set up QEMU build environment.
- [ ] Set up Verilator, cocotb, GTKWave/Surfer.
- [ ] Set up FPGA toolchain for the chosen board (synthesis + program).
- [ ] Create a minimal SM83 test ROM format.
- [ ] Add a trace-diff tool for comparing emulator, RTL, and FPGA execution.
- [ ] Define the UART (or SPI) HIL protocol sketch used later by Chunk H.

**Exit criteria:** build scripts work; a hand-written SM83 test ROM can be assembled and loaded by a test harness; `trace-diff` runs on fixture traces.

---

### M1 — SM83 instruction-set simulator (Chunk B)

**Goal:** create a trusted software reference model.

- [ ] Implement CPU registers and flags.
- [ ] Implement memory bus interface.
- [ ] Implement base instructions: loads, ALU, jumps, stack operations.
- [ ] Implement 16-bit operations.
- [ ] Implement `CB` prefix instructions.
- [ ] Implement interrupts, `EI`, `DI`, `HALT`, and `STOP`.
- [ ] Add instruction tracing and state snapshots.

**Exit criteria:** the ISS passes Blargg `cpu_instrs` and relevant Mooneye CPU tests.

---

### M2 — LLVM assembler and object output (Chunk C)

**Goal:** assemble and link SM83 code using LLVM tools.

- [ ] Add SM83 target registration.
- [ ] Define SM83 registers and register classes.
- [ ] Define instruction encodings in TableGen.
- [ ] Implement assembly parser and printer.
- [ ] Implement MC code emitter.
- [ ] Add ELF or flat-binary ROM output.
- [ ] Add a linker script for a minimal Game Boy memory map.

**Exit criteria:** `llvm-mc` can assemble a hand-written SM83 program into a valid ROM image.

---

### M3 — Minimal LLVM backend (Chunk D)

**Goal:** compile simple C functions to correct SM83 code.

- [ ] Add `SM83TargetMachine`.
- [ ] Define 8-bit and 16-bit integer types.
- [ ] Define calling convention.
- [ ] Implement instruction selection for basic operations.
- [ ] Implement loads, stores, branches, calls, and returns.
- [ ] Implement stack frames.
- [ ] Add runtime helpers for unsupported operations.
- [ ] Add LLVM regression tests.

**Exit criteria:** compile a small C program, link it into a ROM, and execute it successfully in the SM83 ISS.

Example target program:

```c
#include <stdint.h>

uint8_t sum(uint8_t a, uint8_t b) {
    return a + b;
}

void main(void) {
    volatile uint8_t *result = (uint8_t *)0xFF80;
    *result = sum(20, 22);

    while (1) {
        /* idle */
    }
}
```

Expected result: memory address `0xFF80` contains `0x2E` (46).

---

### M4 — QEMU SM83 target (Chunk E)

**Goal:** run compiled programs in QEMU.

- [ ] Add `target/sm83` CPU architecture.
- [ ] Implement SM83 CPU state and TCG translation.
- [ ] Implement memory access and MMIO dispatch.
- [ ] Add a minimal `gbc-test` machine.
- [ ] Add ROM loading.
- [ ] Add a debug exit MMIO device.
- [ ] Add QEMU tracing and register inspection.
- [ ] Add QEMU tests driven by the same ROMs used for RTL and FPGA.

**Exit criteria:** `qemu-system-sm83` executes an LLVM-compiled ROM and reports the expected result.

---

### M5 — Verilog SM83 CPU (Chunk F)

**Goal:** implement and verify the CPU in RTL simulation.

- [ ] Define the CPU/bus interface.
- [ ] Implement register file and flags.
- [ ] Implement ALU.
- [ ] Implement fetch/decode/execute control FSM.
- [ ] Implement memory-cycle sequencing.
- [ ] Implement stack, calls, jumps, and returns.
- [ ] Implement interrupt handling.
- [ ] Add cocotb testbench.
- [ ] Compare RTL traces against the ISS and QEMU.
- [ ] Run CPU test ROMs in Verilator.

**Exit criteria:** RTL passes the same CPU test ROMs as the ISS and produces matching traces in simulation.

---

### M5b — FPGA verification of the CPU (Chunk H)

**Goal:** prove the Verilog CPU on real hardware before building the rest of the SoC.

- [ ] Choose a primary FPGA board and document pin/clock constraints.
- [ ] Create a synthesizable CPU harness with BRAM, ROM slot, reset, and exit MMIO.
- [ ] Add a host link (UART recommended) for ROM load, start, status, and trace dump.
- [ ] Implement `tools/fpga-runner` against that protocol.
- [ ] Synthesize the harness; close timing at the target clock (or documented clock-enable rate).
- [ ] Run the same CPU test ROMs on FPGA as in Verilator.
- [ ] Capture hardware traces and diff against ISS (and optionally QEMU).
- [ ] Add a nightly or manual CI job that programs the board and publishes pass/fail + artefacts.
- [ ] Document failure triage: how to pull waveforms / PC history from the board.

**Exit criteria:** FPGA HIL passes the same CPU ROM set as Verilator, with traces matching the ISS under the agreed compare rules.

---

### M6 — Game Boy memory system (Chunk G)

**Goal:** build the complete memory-mapped system.

- [ ] Implement 64 KiB address-space decoder.
- [ ] Implement WRAM, HRAM, VRAM, OAM, and I/O regions.
- [ ] Implement cartridge ROM interface.
- [ ] Implement MBC1.
- [ ] Implement MBC3 and MBC5.
- [ ] Implement OAM DMA.
- [ ] Add memory access arbitration between CPU, PPU, and DMA.
- [ ] Extend the FPGA harness memory map and re-run CPU + DMA HIL tests.

**Exit criteria:** banked ROMs load correctly and DMA does not corrupt CPU-visible state in simulation **and** on the FPGA harness.

---

### M7 — PPU and video output (Chunk I, video)

**Goal:** render graphics.

- [ ] Implement LCD timing: 154 lines, 456 T-cycles per line.
- [ ] Implement PPU modes 0–3.
- [ ] Implement background rendering.
- [ ] Implement window rendering.
- [ ] Implement sprite rendering.
- [ ] Implement VRAM/OAM access restrictions.
- [ ] Add HDMI, VGA, or framebuffer output.
- [ ] Validate with `dmg-acid2` and test ROMs in Verilator.
- [ ] Extend FPGA verification: dump frame CRC or framebuffer over the host link; compare to golden.

**Exit criteria:** a test ROM renders the expected image in simulation and on FPGA (pixel-perfect or CRC-agreed).

---

### M8 — APU, joypad, serial, and integration (Chunk I, rest)

**Goal:** complete the Game Boy-compatible SoC.

- [ ] Implement APU channels 1–4.
- [ ] Implement audio output.
- [ ] Implement joypad input.
- [ ] Implement serial link interface.
- [ ] Integrate CPU, memory, PPU, APU, timers, DMA, and cartridge bridge.
- [ ] Run a small LLVM-compiled game or demo in sim and QEMU.
- [ ] Extend FPGA HIL for audio register/stream checks where practical.
- [ ] Run selected commercial/homebrew ROMs.

**Exit criteria:** the SoC runs a playable homebrew ROM with video, audio, and input in simulation; FPGA path at least boots the same ROM to a known frame.

---

### M9 — Full FPGA SoC bring-up (Chunk J)

**Goal:** deploy the integrated design to hardware with production-like I/O.

- [ ] Finalise target FPGA board support package.
- [ ] Add clocking, reset, and PLL/clock-enable infrastructure.
- [ ] Map internal memories to BRAM.
- [ ] Add ROM loading from SPI flash, SD card, or host interface.
- [ ] Add video output.
- [ ] Add audio output.
- [ ] Run timing closure for the full SoC.
- [ ] Compare hardware behaviour against QEMU and RTL simulation (traces + frame/audio checks).
- [ ] Keep the Chunk H CPU harness alive as a regression bitstream alongside the full SoC.

**Exit criteria:** the same LLVM-compiled ROM produces equivalent visible output on QEMU, RTL simulation, and FPGA hardware; CPU HIL suite still passes on the dedicated harness bitstream.

## Verification strategy

Every layer uses the same test assets:

```text
Test ROM / C program
        ↓
   SM83 ISS          (software golden model)
        ↓
 QEMU SM83 target    (fast functional / debug)
        ↓
 Verilog simulation  (Verilator + cocotb)
        ↓
 FPGA HIL            (same ROM, hardware traces / CRC)
```

For each execution, compare:

- Program counter
- CPU registers and flags
- Memory writes
- Interrupt requests and acknowledgement timing
- PPU scanline and frame counters (once PPU exists)
- Frame CRC or framebuffer digest (sim + FPGA)
- Audio register state (and optional sample CRC)
- Final exit status

### Simulation vs FPGA verification

| Layer | What it proves | What it misses |
|---|---|---|
| ISS | Architectural intent | Cycle micro-effects you have not modelled |
| QEMU | Speed, debugger UX, rough SoC | Exact GB timing unless you invest heavily |
| Verilator | RTL logic vs ISS | Vendor primitives, I/O, real timing, metastability at boundaries |
| **FPGA HIL** | **Synthesized netlist + board reality** | Harder debug; needs a deliberate observability story |

FPGA verification rules for this project:

1. **Same stimulus:** ROMs and MMIO exit conventions identical to sim.
2. **Same oracle:** ISS (primary) and QEMU (secondary) generate expected traces/results.
3. **Early and often:** CPU HIL (M5b / Chunk H) before PPU/APU investment.
4. **Machine checkable:** `fpga-runner` + `trace-diff` must be scriptable; no “looks fine on HDMI” as the only gate for CPU.
5. **Two bitstreams:** a small **CPU harness** for fast HIL regression, and a **SoC** bitstream for integration demos.
6. **CI stance:** Verilator on every PR; FPGA HIL on nightly/manual runners when a board is attached.

Primary test suites:

- Blargg `cpu_instrs`
- Mooneye Test Suite
- Mealybug Tearoom Tests
- `dmg-acid2` for PPU validation
- LLVM regression tests for compiler correctness
- Custom integration ROMs generated by the LLVM backend
- FPGA HIL suite mirroring the CPU (then PPU) ROM sets

## Toolchain

| Area | Tool |
|---|---|
| Compiler | LLVM / Clang |
| CPU emulation | QEMU with custom SM83 target |
| Reference emulator | SameBoy and/or mGBA |
| RTL simulation | Verilator |
| Python testbench | cocotb |
| Waveform viewer | GTKWave or Surfer |
| Formal verification | SymbiYosys |
| FPGA synthesis | Yosys / nextpnr for open FPGA targets, or vendor tools |
| FPGA programming / HIL | openFPGALoader (or vendor), `tools/fpga-runner` |
| Host link | UART (default), optional SPI / USB-FIFO |
| Documentation | Pan Docs |

## Getting started

### Prerequisites

- CMake
- Ninja
- Python 3.11+
- LLVM development tools
- QEMU build dependencies
- Verilator
- cocotb
- GTKWave or Surfer
- FPGA toolchain for your chosen board
- Serial terminal / `pyserial` (for FPGA HIL)

### Build the reference ISS

```bash
cmake -S tools/iss -B build/iss
cmake --build build/iss
```

### Run a test ROM

```bash
./build/iss/sm83-iss tests/roms/cpu_instrs.gb
```

### Build the RTL simulation

```bash
cmake -S rtl -B build/rtl
cmake --build build/rtl
```

### Run RTL tests

```bash
pytest rtl/tests
```

### Run FPGA CPU HIL (after Chunk H)

```bash
# Build / program the CPU harness bitstream for your board, then:
./tools/fpga-runner \
  --port /dev/ttyUSB0 \
  --rom tests/roms/cpu_instrs.gb \
  --trace out/fpga.jsonl

./tools/trace-diff out/iss.jsonl out/fpga.jsonl
```

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — overall system architecture
- [`docs/chunks.md`](docs/chunks.md) — chunk contracts and dependency graph
- [`docs/llvm-backend.md`](docs/llvm-backend.md) — SM83 LLVM backend design
- [`docs/qemu-target.md`](docs/qemu-target.md) — QEMU target and machine model
- [`docs/rtl-design.md`](docs/rtl-design.md) — RTL microarchitecture and bus design
- [`docs/fpga.md`](docs/fpga.md) — FPGA integration and bring-up
- [`docs/fpga-verification.md`](docs/fpga-verification.md) — HIL protocol, harnesses, trace compare

## References

- [`docs/datasheets/`](docs/datasheets/) — local PDFs (Gekkio GBCTR + Sharp 1996 SM83 databook)
- [Pan Docs — Game Boy technical reference](https://gbdev.io/pandocs/)
- [SM83 opcode tables](https://gbdev.io/gb-opcodes/optables/)
- [LLVM: Writing an LLVM Backend](https://llvm.org/docs/WritingAnLLVMBackend.html)
- [LLVM-MOS: LLVM backend for MOS 6502](https://github.com/llvm-mos/llvm-mos)
- [SameBoy](https://github.com/LIJI32/SameBoy)
- [mGBA](https://mgba.io/)
- [Game Boy test ROM collection](https://github.com/c-sp/game-boy-test-roms)
- [VerilogBoy](https://github.com/zephray/Verilogboy)

## Contributing

Contributions are welcome, especially in these areas:

- LLVM instruction selection and register allocation
- QEMU TCG frontend implementation
- Verilog CPU microarchitecture
- FPGA verification harnesses and `fpga-runner`
- PPU timing and rendering
- cocotb testbenches
- FPGA board support
- Documentation and test ROMs

Please open an issue before starting a large change. Keep each pull request focused on **one chunk** (or one milestone inside a chunk).

Suggested contribution order for new contributors:

1. Chunk A tooling / tests
2. Chunk B ISS instructions
3. Chunk F RTL + sim tests
4. Chunk H FPGA HIL (if you have board access)

## License

Choose a license before publishing code:

- **MIT or Apache-2.0** — simple permissive licensing.
- **GPL-3.0** — strong copyleft.
- **CERN-OHL-S / CERN-OHL-W** — if you want hardware-oriented licensing for RTL.

Recommended initial choice: **Apache-2.0** for software and **CERN-OHL-S** for RTL, if you want separate software and hardware terms.
