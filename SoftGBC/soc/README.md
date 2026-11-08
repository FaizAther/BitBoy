# soc

SoftGba system glue — builds the machine and runs ROMs.

Wires [`sm83-iss`](../sm83-iss/) and [`dmg-ppu`](../dmg-ppu/) directly with the other SoftGba peripherals.

Do this **in order**. Each step should leave you with something runnable or testable before you start the next.

```text
workspace → cart header → bus/mem → CPU steps → IRQs/timer/DMA
    → PPU frames → MBC/joypad → APU/window → full game
```

---

## Step 0 — Cargo workspace + empty SoC

**Goal:** `softgba` builds and prints “hello”.

1. Create a Cargo workspace at `SoftGba/` with members: `soc`, `sm83-iss`, `dmg-ppu` (others as you add them).
2. Add a `soc` package with a binary `softgba`.
3. Define an empty `struct SoftGba { }` and `fn main()` that parses `--help`.

**Done when:** `cargo run -p softgba -- --help` works from `SoftGba/`.

- [ ] Workspace + `soc` package
- [ ] `softgba` binary stub

---

## Step 1 — Load a ROM (cartridge only)

**Goal:** open a `.gb` and print the header title / cart type. No CPU yet.

1. Implement [`cartridge`](../cartridge/) load + header parse.
2. `SoftGba::load_rom(path)` stores the cart.
3. CLI: `softgba <rom>` prints title and MBC type, then exits.

**Done when:** you can identify Tetris / a test ROM from the CLI.

- [ ] `load_rom`
- [ ] Header dump on CLI

---

## Step 2 — Bus + memory map (no PPU/APU yet)

**Goal:** CPU-shaped reads/writes work for ROM + WRAM + HRAM.

1. Implement [`memory`](../memory/) (WRAM, HRAM; VRAM/OAM buffers can exist empty).
2. Implement [`bus`](../bus/) routing:
   - `0000`–`7FFF` → cartridge ROM (ROM-only is enough)
   - `C000`–`DFFF` (+ echo) → WRAM
   - `FF80`–`FFFE` → HRAM
   - `FFFF` → stub `IE` (or hold in interrupt stub)
3. Stub other IO reads as `0xFF` so nothing panics.

**Done when:** unit tests read/write WRAM and read bytes from a loaded ROM.

- [ ] Memory regions
- [ ] Bus decode for ROM/WRAM/HRAM
- [ ] Safe stubs for unmapped IO

---

## Step 3 — Plug in the CPU (`sm83-iss`)

**Goal:** execute instructions against the SoftGba bus.

1. Flesh out [`sm83-iss`](../sm83-iss/) enough for NOP / basic ops (or bring it up in parallel before this step).
2. SoC owns `Cpu` + implements / wraps SoftGba `Bus` for the ISS `Bus` trait.
3. Add `reset()` (PC = `0x0100` skip-boot for now; post-boot regs optional).
4. Add `step()` → one CPU instruction.
5. CLI: `softgba <rom> --steps N` prints PC each step (or a short trace).

**Done when:** a tiny hand ROM or first bytes of a test ROM advance PC sanely.

- [ ] CPU held by SoC
- [ ] `reset` / `step`
- [ ] `--steps` trace

---

## Step 4 — Interrupts controller (minimal)

**Goal:** `IF` / `IE` exist and can jump the CPU later.

1. Implement [`interrupt`](../interrupt/) (`FF0F`, `FFFF`).
2. Bus routes those registers.
3. After each `step`, SoC asks interrupt module whether to service; call into CPU IRQ entry when ready.
4. Keep sources stubbed (nothing raises yet) except manual `IF` writes for tests.

**Done when:** writing `IF`/`IE` in a test can force an interrupt service (even with a tiny crafted ROM).

- [ ] `IF` / `IE` on bus
- [ ] CPU IRQ dispatch hook from SoC

---

## Step 5 — Timer

**Goal:** time passes and timer IRQs fire.

1. Implement [`timer`](../timer/).
2. SoC ticks timer by the cycles returned from `cpu.step()`.
3. Timer requests IRQ through `interrupt`.

**Done when:** a known timer test ROM (or unit test) overflows `TIMA` and sets `IF` timer bit.

- [ ] Timer wired to bus
- [ ] Cycle-accurate enough tick from SoC
- [ ] Timer → interrupt

---

## Step 6 — OAM DMA

**Goal:** `FF46` copies sprites into OAM.

1. Implement [`dma`](../dma/).
2. SoC runs DMA progress alongside CPU (CPU restricted as required).
3. Bus writes to `FF46` start transfer into `memory` OAM.

**Done when:** write `FF46`, wait ~160 M-cycles, OAM matches source.

- [ ] DMA start + copy
- [ ] SoC advances DMA with CPU

---

## Step 7 — Headless CPU CI path

**Goal:** SoftGba runs Blargg without a window.

1. Define exit convention (e.g. classic Blargg serial/`0xFF01` or a magic HRAM store — pick one and document it).
2. CLI: `softgba <rom> --headless --max-cycles N` → exit code 0/1.
3. Optional: dump ISS-compatible traces for `trace-diff`.

**Done when:** Blargg `cpu_instrs` passes under SoftGba.

- [ ] Headless runner
- [ ] Blargg green
- [ ] (Optional) trace output

---

## Step 8 — Plug in the PPU (`dmg-ppu`)

**Goal:** frames exist.

1. Bring up [`dmg-ppu`](../dmg-ppu/) enough for LY + BG (or full as available).
2. SoC shares VRAM/OAM buffers with PPU.
3. Bus routes `FF40`–`FF4B` to PPU.
4. After CPU steps, SoC `ppu.tick(t_cycles)` using the same cycle budget.
5. PPU VBlank/STAT → `interrupt`.
6. CLI: `softgba <rom> --frames N --crc` prints frame CRC (no window yet).

**Done when:** LY advances; a simple ROM produces a stable CRC or non-zero framebuffer.

- [ ] PPU owned by SoC
- [ ] VRAM/OAM shared
- [ ] PPU IO on bus
- [ ] VBlank/STAT IRQs
- [ ] `--frames` / CRC

---

## Step 9 — MBC + real carts

**Goal:** banked commercial/homebrew ROMs load correctly.

1. Extend [`cartridge`](../cartridge/) with MBC1 (then MBC3/5 as needed).
2. Bus banking through cartridge for `0000`–`7FFF` / `A000`–`BFFF`.
3. Optional `.sav` load/store later (can wait until Phase audio/polish).

**Done when:** a banked test ROM or known MBC1 game reads correct banks.

- [ ] MBC1
- [ ] SRAM enable/window
- [ ] MBC3/5 when required by your target game

---

## Step 10 — Joypad

**Goal:** you can control a game.

1. Implement [`joypad`](../joypad/) on `FF00`.
2. SoC API: `set_button(Button, pressed)`.
3. CLI or window backend feeds keys → joypad.
4. Optional joypad IRQ.

**Done when:** input changes `P1` reads as expected in a test; in-game movement works once a window exists.

- [ ] Joypad on bus
- [ ] Host input injection

---

## Step 11 — Windowed frontend

**Goal:** see the game.

1. Pick a host (SDL2 / winit+egui / pixels — keep it thin).
2. Each frame: run SoC until VBlank; upload `dmg-ppu` framebuffer; poll keys into joypad.
3. CLI default becomes windowed; keep `--headless` for CI.

**Done when:** a homebrew or commercial ROM shows a recognizable screen and accepts input.

- [ ] Present 160×144
- [ ] Keyboard → joypad
- [ ] `--headless` still works

---

## Step 12 — APU

**Goal:** sound.

1. Implement [`apu`](../apu/) (or stub mixer first).
2. Bus routes NR regs + wave RAM.
3. SoC ticks APU with CPU cycles; pushes samples to host audio.

**Done when:** audible output on a known music ROM (or clean silence with no crashes if muted).

- [ ] APU on bus
- [ ] Host audio stream

---

## Step 13 — Polish (optional order)

Do these when you need them; they are not on the critical path to “see a game”:

1. [`boot`](../boot/) — real boot ROM vs skip-boot register init  
2. [`serial`](../serial/) — stub so games don’t spin forever on link  
3. Save RAM to disk  
4. `run_frame()` convenience + debugger hooks (breakpoints, mem peek)  
5. Trace format aligned with RTL/FPGA `trace-diff`

- [ ] Boot skip documented / optional boot ROM
- [ ] Serial stub
- [ ] `.sav` files
- [ ] Debugger niceties

---

## Step 14 — Victory condition

**Done when:**

- [ ] One full Game Boy game runs playably (video + input; audio nice-to-have)
- [ ] Blargg CPU still green headless in CI
- [ ] At least one PPU visual test (CRC or `dmg-acid2`) in CI

---

## Suggested SoC API (target shape)

```rust
pub struct SoftGba { /* cart, bus, cpu, ppu, timer, dma, irqs, apu, joypad, ... */ }

impl SoftGba {
    pub fn new() -> Self;
    pub fn load_rom(&mut self, path: &Path) -> Result<()>;
    pub fn reset(&mut self);
    pub fn step(&mut self) -> u32;          // cycles
    pub fn run_frame(&mut self);            // until VBlank
    pub fn framebuffer(&self) -> &[u32];
    pub fn set_button(&mut self, btn: Button, pressed: bool);
}
```

## Cycle discipline (keep consistent)

Whatever unit `sm83-iss::step` returns (M-cycles or T-cycles), convert **once** in SoC and feed the same budget to timer, DMA, PPU, and APU. Document the choice in code comments and do not mix units.
