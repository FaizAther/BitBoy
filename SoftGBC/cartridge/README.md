# cartridge

ROM image + MBC + external battery RAM.

## Checklist

- [ ] Load `.gb` / `.gbc` file into memory
- [ ] Parse header (title, cart type, ROM/RAM size)
- [ ] ROM-only cart type
- [ ] MBC1 (ROM/RAM banking)
- [ ] MBC3 (optional RTC later)
- [ ] MBC5
- [ ] External SRAM read/write when enabled
- [ ] Persist SRAM to disk (`.sav`)
- [ ] Bus hooks for `0000`–`7FFF` and `A000`–`BFFF`
- [ ] Reject / warn on unsupported cart types
