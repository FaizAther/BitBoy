# cartridge

ROM image + MBC + external battery RAM.

## Current status *(2026-11)*

**Stub crate** (`lib.rs` placeholder). SoftGBC loads `.gb` files today via **`softgbc-core::DmgBus::from_rom_file`** (first 32 KiB into the ROM window). No header dump CLI and no MBC.

Needed soon for:

- Combined Blargg `cpu_instrs.gb` (64 KiB) — banking or larger ROM
- Any commercial/homebrew that banks

## Checklist

- [x] Load `.gb` into a ROM image *(via `softgbc-core` for now)*
- [ ] Move / wrap load behind this crate’s API
- [ ] Parse header (title, cart type, ROM/RAM size)
- [ ] ROM-only cart type
- [ ] MBC1 (ROM/RAM banking)
- [ ] MBC3 (optional RTC later)
- [ ] MBC5
- [ ] External SRAM read/write when enabled
- [ ] Persist SRAM to disk (`.sav`)
- [ ] Bus hooks for `0000`–`7FFF` and `A000`–`BFFF`
- [ ] Reject / warn on unsupported cart types
