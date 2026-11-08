# apu

DMG APU — four channels + mixer + NR registers.

## Checklist

- [ ] Register block `FF10`–`FF26`
- [ ] Wave RAM `FF30`–`FF3F`
- [ ] Channel 1 (square + sweep)
- [ ] Channel 2 (square)
- [ ] Channel 3 (wave)
- [ ] Channel 4 (noise)
- [ ] Frame sequencer / length / envelope / sweep timing
- [ ] DAC enable / mixer / NR50–NR52
- [ ] Sample output buffer for host audio
- [ ] Mute-safe stub mode (games run with silence)
- [ ] Basic channel tests / listen-check on a known ROM
