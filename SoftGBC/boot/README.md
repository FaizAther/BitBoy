# boot

Optional DMG boot ROM.

## Current status *(2026-11)*

**Not started.** SoftGBC defaults to **skip-boot**: `PC = 0x0100` (Blargg runner also sets `A = 0x01` as DMG id). Real boot ROM can wait.

## Checklist

- [ ] Optional load of boot ROM image
- [ ] Map over `0000`–`00FF` at reset
- [ ] Unmap when boot writes unmap register
- [x] Skip-boot mode: start PC at `0100` (used by SoftGBC today)
- [ ] Document full post-boot register init if games need it
- [x] SoftGBC default = skip-boot
