# boot

Optional DMG boot ROM.

## Checklist

- [ ] Optional load of boot ROM image
- [ ] Map over `0000`–`00FF` at reset
- [ ] Unmap when boot writes unmap register
- [ ] Skip-boot mode: start PC at `0100` with post-boot register state
- [ ] Document which mode SoftGba defaults to
