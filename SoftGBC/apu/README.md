# apu

DMG sound — four channels + mixer + NR regs.

**Refs:** [Pan Docs — Audio](https://gbdev.io/pandocs/Audio.html)  

## Status *(2026-11)*

Not started. Prefer **mute stub first**.

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [ ] **Mute-safe stub (first)**

  **What:** NR regs accept writes; output silence so games don’t hang.  
  **Structure:**
```text
Apu: regs[FF10..FF3F]
read/write → store/load regs (mask later)
tick(t) → no-op
mix() → (0, 0)
```
  **Min test:**
```text
write FF26 → read back LCD/APU-on bit as stored
mix stays 0 after many ticks
```
  **Done when:** a ROM that pokes NR boots without spinning.

---

- [ ] **NR block + wave RAM**

  **What:** `FF10`–`FF26` and `FF30`–`FF3F`.  
  **Min test:** wave RAM 16-byte round-trip.  
  **Done when:** all addresses routed.

---

- [ ] **Channel 2 (square)** then **1 / 3 / 4**

  **What:** duty/length/envelope/freq (ch1 +sweep; ch3 wave; ch4 LFSR).  
  **Structure:** each channel advances in `tick`; mixer sums DACs.  
  **Min test:** enable channel → mix not always zero (or freq counter moves).  
  **Done when:** audible or counter-tested per channel.

---

- [ ] **Frame sequencer / NR50–52 / host buffer**

  **What:** 512 Hz steps; master volume; sample ring for host.  
  **Min test:** length expires → channel off; buffer grows after ticks.  
  **Done when:** one known ROM sounds recognizable (manual).
