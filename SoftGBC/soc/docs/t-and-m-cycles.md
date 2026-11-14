# T-cycles and M-cycles

Same clock, two rulers.

| Name | Meaning | SoftGBC use |
|---|---|---|
| **T-cycle** | One tick of ~4.19 MHz master clock | Peripherals (`tick(t)`), scanline = **456 T** |
| **M-cycle** | CPU “machine” cycle | Opcode length tables (`NOP` = 1 M) |

```text
1 M-cycle  =  4 T-cycles
```

## Simple example

`NOP` takes **1 M-cycle**.

```text
CPU:  execute NOP          → charges 1 M
Bus:  tick_t_cycles(1 * 4) → advance timer/PPU by 4 T
```

A scanline is **456 T** = **114 M**. So after `114` NOPs of wall time (if nothing else), LY could advance by one — in reality games mix longer instructions, but the **sum of T** is what matters.

## Tiny snippet (idea)

```rust
/// Convert CPU M-cycles from step() into peripheral T-cycles.
fn advance_peripherals(bus: &mut DmgBus, m_cycles: u8) {
    bus.tick_t_cycles(u32::from(m_cycles) * 4);
}
```

## Emphasize

- Do **not** add `456` to an M-cycle counter without converting.  
- PPU fixtures use **T** (`456`, `70224`).  
- CPU tests often speak **M**.

## References

- SoftGBC overview — [`../../README.md`](../../README.md) master clock section  
- [GBEDG concepts](https://github.com/Ashiepaws/GBEDG/blob/master/concepts/index.md)  
- [gbctr.pdf](../../../docs/datasheets/gbctr.pdf) — timing chapters  
- Pan Docs — [Specifications](https://gbdev.io/pandocs/Specifications.html)  
- Index: [`../../../docs/REFERENCES.md`](../../../docs/REFERENCES.md)
