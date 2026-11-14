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
  halt_bug/ / halt_bug.gb
  interrupt_time/
```

## SoftGBC status *(2026-11)*

**Stack today:** `SoftGbc` (`sm83-iss` + `DmgBus` with timer/serial/VRAM/OAM) + optional stub `LY=144`.  
Blargg ROMs are exercised by `cargo test -p soc --test blargg_cpu_instrs`.  
**No** real PPU, MBC, or DMA yet.

| ROM | Fit in 32K? | SoftGBC result | Notes |
|---|---|---|---|
| `cpu_instrs/01-special.gb` | yes | **Passed** | |
| `02-interrupts.gb` | yes | **Passed** | Timer + IRQ path |
| `03-op sp,hl.gb` | yes | **Passed** | |
| `04-op r,imm.gb` | yes | **Passed** | |
| `05-op rp.gb` | yes | **Passed** | |
| `06-ld r,r.gb` | yes | **Passed** | |
| `07-jr,jp,call,ret,rst.gb` | yes | **Passed** | |
| `08-misc instrs.gb` | yes | **Passed** | |
| `09-op r,r.gb` | yes | **Passed** | |
| `10-bit ops.gb` | yes | **Passed** | |
| `11-op a,(hl).gb` | yes | **Passed** | |
| `cpu_instrs.gb` (combined) | **no** (64K) | Not run | Needs banking or larger ROM window |
| `instr_timing.gb` | yes | Not run | Needs accurate M-cycle counts end-to-end |
| `mem_timing/*` | mostly yes | Not run | Memory / timing accuracy |
| `halt_bug.gb` | yes | Not run | HALT bug behaviour |
| `interrupt_time.gb` | yes | Not run | IRQ timing |

**Next:** combined `cpu_instrs.gb` (banking) or `instr_timing` / PPU work.

## How to run

```bash
cd SoftGBC
# generic runner (any ROM; serial Passed/Failed → exit code)
cargo run -p soc -- tests/roms/blargg/cpu_instrs/01-special.gb
# full Blargg individuals as unit tests
cargo test -p soc --test blargg_cpu_instrs
```

Exit codes (CLI): `0` serial Passed, `1` serial Failed, `3` max steps, `2` load/usage error.

## How Blargg reports results

Writes to serial (`FF01`/`FF02`) and/or shows a string on the “screen”. SoftGBC headless path:

1. On `FF02` write with transfer-start bit, capture `FF01` into a string  
2. Success contains `Passed`  
3. Failure contains `Failed`

## Source

```text
https://github.com/retrio/gb-test-roms
```
