# interrupt

`IF` (`FF0F`), `IE` (`FFFF`), priority, CPU IRQ entry.

## Checklist

- [ ] Readable/writable `IF` and `IE`
- [ ] Sources: VBlank, STAT, timer, serial, joypad
- [ ] Priority order when multiple pending
- [ ] CPU interrupt dispatch when `IME` and masked bit set
- [ ] Clear corresponding `IF` bit on service
- [ ] Interaction with `HALT` wake-up
- [ ] Integration tests with PPU VBlank + timer
