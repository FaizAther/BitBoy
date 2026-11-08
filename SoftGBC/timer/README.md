# timer

`DIV` / `TIMA` / `TMA` / `TAC`.

## Checklist

- [ ] `DIV` (`FF04`) increments; write resets
- [ ] `TIMA` (`FF05`) / `TMA` (`FF06`) / `TAC` (`FF07`)
- [ ] TAC enable + clock select
- [ ] TIMA overflow → reload TMA + request timer IRQ
- [ ] DIV-edge timing quirks as required by tests
- [ ] Tick from `soc` in T-cycles or M-cycles consistently
- [ ] Pass agreed timer test ROMs
