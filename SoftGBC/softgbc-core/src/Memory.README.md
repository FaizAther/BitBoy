# memory

On-console RAM regions and buffers shared with PPU/DMA.

## Checklist

- [ ] WRAM 8KiB (`C000`–`DFFF`)
- [ ] Echo RAM behaviour (`E000`–`FDFF`)
- [ ] HRAM (`FF80`–`FFFE`)
- [ ] VRAM 8KiB buffer
- [ ] OAM 160-byte buffer
- [ ] Reset / clear helpers
- [ ] Optional dump helpers for debugging


## Testing

RUST_BACKTRACE=1 rustc --test memory/memory.rs -o /tmp/ram_tests