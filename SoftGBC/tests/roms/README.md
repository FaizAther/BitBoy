# Test ROMs

Downloaded from [retrio/gb-test-roms](https://github.com/retrio/gb-test-roms) (Blargg).

## Layout

```text
blargg/
  cpu_instrs/          # instruction correctness
    01-special.gb … 11-op a,(hl).gb   # 32 KiB each — prefer these
    cpu_instrs.gb                       # 64 KiB combined (needs 2 ROM banks)
  instr_timing/
  mem_timing/
  halt_bug.gb
  interrupt_time/
```

## What can run on SoftGBC *today*?

Current stack: SM83 ISS + `DmgBus` (32 KiB ROM window, WRAM, HRAM). **No** timer, PPU, serial device, or IRQ service yet.

| ROM | Fit in 32K? | Likely runnable soon? | Notes |
|---|---|---|---|
| `cpu_instrs/01`…`11` (except `02`) | yes | **best next target** | Need serial pass/fail + IO stubs; header says MBC1 but image is 32K |
| `02-interrupts.gb` | yes | after IRQ + timer | Needs `IF`/`IE` + timer interrupt |
| `cpu_instrs.gb` (combined) | **no** (64K) | after 64K/MBC1 or larger ROM | |
| `instr_timing.gb` | yes | after cycle-accurate timing | Strict M-cycle counts |
| `mem_timing/*` | 32K singles yes | after mem timing | |
| `halt_bug.gb` | yes | after HALT bug behavior | |
| `interrupt_time.gb` | yes | after IRQ timing | |

**Start with:** individual `cpu_instrs/01-special.gb`, then `04`, `06`, `09`, `10`, `11` (pure ALU/LD/bit). Leave `02-interrupts` until the SoC has interrupts.

## How Blargg reports results

Writes to serial (`FF01`/`FF02`) and/or shows a string on the “screen”. For headless:

1. Stub serial: on `FF02` write with transfer start, capture `FF01` bytes into a string  
2. Success contains `Passed`  
3. Failure contains `Failed`

## Source

```text
https://github.com/retrio/gb-test-roms
```
