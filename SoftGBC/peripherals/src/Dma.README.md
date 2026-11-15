# DMA

OAM DMA via `FF46` — a hardware memcpy that refreshes the sprite attribute table
while the CPU is mostly locked out of the bus.

**Refs:** [Pan Docs — OAM DMA](https://gbdev.io/pandocs/OAM_DMA_Transfer.html) · [GBEDG — DMA](https://github.com/Ashiepaws/GBEDG/blob/master/dma/index.md) · [`dma.rs`](dma.rs) · bus wiring in `softgbc-core`

## Status *(2026-11)*

**Timed OAM DMA wired** on `DmgBus`: write `FF46` starts a **160 M-cycle** transfer
(1 byte per M via `tick_t_cycles` / `tick_m_cycles`). While `active`, CPU access
outside HRAM / `FF46` is blocked (reads `$FF`, writes ignored). Source fetches use
an unrestricted bus path so the copy still sees real WRAM.

```text
                    SoftGBC OAM DMA (big picture)
  ┌──────────┐  write FF46=src_hi       ┌─────────────┐
  │   CPU    │ ──────────────────────► │  Dma device │
  │ (HRAM OK │                         │ page,active │
  │  mid-DMA)│                         │ index 0..160│
  └────┬─────┘                         └──────┬──────┘
       │ blocked                               │ 1 byte / M-cycle
       │ (else → $FF)                          ▼
       │                               ┌──────────────┐
       │                               │  source page │
       │                               │  src_hi<<8   │──read──► WRAM/ROM/…
       │                               └──────┬───────┘
       │                                      │ write
       │                                      ▼
       │                               ┌──────────────┐
       └──── after done, PPU reads ─── │ OAM FE00..9F │
                                       │  160 bytes   │
                                       └──────────────┘
```

```text
write FF46 = src_hi
  → active=true, index=0, src = src_hi << 8
each M-cycle while active:
  oam[index] = read_raw(src + index); index++
  if index == 160: active = false
```

### Allowed sources (SoftGBC)

| Source | SoftGBC |
|---|---|
| WRAM `$C000`–`$DFFF` | **Supported** (common game path) |
| ROM / VRAM / other via bus raw read | Whatever the map returns |
| Echo WRAM `$E000`–`$FDFF` | Follows bus echo mapping |

### CPU during DMA

| Region | Read | Write |
|---|---|---|
| HRAM `$FF80`–`$FFFE` | ok | ok |
| `FF46` | ok (page) | restarts DMA |
| Everything else | `$FF` | ignored |

## Checklist

What → Structure → Min test → Done when (pseudocode + ASCII).

---

- [x] **Write `FF46` starts transfer**

  **What:** Games do not poke OAM byte-by-byte every frame. They build a 160-byte
  shadow table in WRAM, then write the **high byte** of that address to `FF46`.
  That single write **arms** DMA: SoftGBC sets `page`, `active=true`, `index=0`.
  No bytes move until the SoC ticks M-cycles (see timed item). Hardware and SoftGBC
  both treat `FF46` as “start from `page<<8`.”

  **Structure:**
```text
FF46 write (data = src_hi):

     CPU write data ──► Dma.page = data
                        Dma.active = true
                        Dma.index  = 0
                        src_base   = data << 8   # e.g. C0 → $C000

     timeline:
     ────●────────────────────────────► time
         FF46 write
         (armed, 0 bytes copied yet)
```
```text
on write FF46:
  page   = data
  active = true
  index  = 0
  # copy happens later in tick_m / bus.tick_*
```

  **Min test:**
```text
mem[C000] = 0xAB
write FF46 = 0xC0          # arm
tick 1 M-cycle             # first byte lands
assert oam_buffer[0] == 0xAB
# (CPU read FE00 mid-DMA may still be $FF — see access limits)
```

  **Done when:** write arms DMA (`active`, `index==0`, `page` stored); first tick
  copies the first source byte (`dma` unit tests + `oam_dma_ff46_copies_wram_to_oam`).

---

- [x] **Copy 160 bytes to OAM**

  **What:** OAM is exactly **160 bytes** (`$FE00`–`$FE9F`): 40 objects × 4 bytes
  (Y, X, tile, flags). One DMA pass replaces the **entire** table. SoftGBC writes
  into `DmgBus.oam[]`, which is the same buffer the PPU reads in `render_frame`.

  **Structure:**
```text
  source page (e.g. $C000)              OAM ($FE00)
  ┌────┬────┬────┬────┬───┐            ┌────┬────┬────┬────┬───┐
  │ Y0 │ X0 │ T0 │ F0 │… │  ──DMA──►  │ Y0 │ X0 │ T0 │ F0 │… │
  │obj0│    │    │    │   │   160 B    │obj0│    │    │    │   │
  ├────┼────┼────┼────┤   │            ├────┼────┼────┼────┤   │
  │ Y1 │ …  │    │    │   │            │ Y1 │ …  │    │    │   │
  └────┴────┴────┴────┴───┘            └────┴────┴────┴────┴───┘
       index 0 ……… 159                      index 0 ……… 159

  for i in 0..159:
    oam[i] = read_raw(src_base + i)
```

  **Min test:**
```text
fill C000..C09F with i     # byte i holds value i
write FF46 = 0xC0
tick 160 M-cycles
for i in 0..159:
  assert oam[i] == i
```

  **Done when:** all 160 bytes match after a full transfer
  (`instant_dma_copies_full_160_bytes`, bus test after 160 ticks).

---

- [x] **Document allowed sources**

  **What:** Pan Docs list regions that are awkward or unused as DMA sources.
  SoftGBC does **not** specially reject addresses: the DMA engine calls the bus
  **raw** read map. In practice games use **WRAM `$C000+`**. ROM/VRAM “work” only
  insofar as `read8_raw` returns something for that address. Echo WRAM follows
  the same echo rules as normal CPU reads.

  **Structure:**
```text
  address space (simplified)          DMA source?
  $0000─┬─ ROM                        yes (raw)
  $8000─┼─ VRAM                       yes (raw)
  $C000─┼─ WRAM  ◄── usual path       yes ★
  $E000─┼─ echo WRAM                  yes (via echo map)
  $FE00─┼─ OAM                        odd; raw still reads oam[]
  $FF00─┴─ IO/HRAM                    rarely used as src page

  SoftGBC policy: no extra reject list — document what we support (★ WRAM).
```

  **Min test:**
```text
DMA from C000 works (same as copy tests)
README table lists WRAM / raw / echo
```

  **Done when:** this README’s “Allowed sources” table matches behaviour.

---

- [x] **Timed DMA (~160 M-cycles)**

  **What:** Real hardware takes about **160 M-cycles** (one byte per M-cycle), not
  an instant memcpy. SoftGBC mirrors that: `Dma::tick_m` copies `min(m, remaining)`
  bytes. The bus converts T→M as `t_cycles/4` inside `tick_t_cycles`, so every
  `SoftGbc::step` that ticks the bus advances DMA in lockstep with the CPU.

  **Structure:**
```text
  M-cycle:   0    1    2         …        159   160
  oam[]:   [ ]  [b0] [b0 b1]  ………  [b0..b158] [full]  active=false
             ▲
           FF46 write (index=0, active=true)

  Dma:
    page, active, index

  tick_m(m, read_raw, oam):
    while active and m > 0:
      oam[index] = read_raw(src_base + index)
      index += 1
      m -= 1
      if index == 160: active = false

  Bus:
    tick_t_cycles(t):
      tick_oam_dma(t / 4)     # M-cycles worth of bytes
      timer.tick(t); ppu.tick(t); …
```

  **Min test:**
```text
start → index 0, active
tick(1) → oam[0] written, index 1, still active
tick(159) → index 160, active false
further ticks → no-op
```

  **Done when:** `timed_dma_one_byte_per_m_cycle` and bus `oam_dma_ff46_copies_wram_to_oam`
  (1 tick then 159) are green.

---

- [x] **CPU access limits during DMA**

  **What:** While DMA owns the external bus, the CPU can still run from **HRAM**
  (and can read/write `FF46`). Other reads return SoftGBC’s chosen open-bus stand-in
  **`$FF`**; other writes are **ignored**. Critical: DMA’s own source fetches must
  **not** go through that block — they use `read8_raw` / `read8_unrestricted`, or
  the copy would fill OAM with `$FF`.

  **Structure:**
```text
  CPU access while dma.active:

       addr
         │
         ▼
    ┌────────────────┐
    │ HRAM or FF46?  │──yes──► normal read/write
    └────────┬───────┘
             │ no
             ▼
        read  → return $FF
        write → ignore (ok)

  DMA engine path (parallel, not CPU):

       src_base+index ──► read8_raw ──► oam[index]
                          (never blocked)
```
```text
if dma.active and not cpu_ok_during_dma(addr):
  read  → 0xFF
  write → no-op
# cpu_ok = FF80..FFFE or FF46
```

  **Min test:**
```text
mem[C000]=0xAB; mem[FF80]=0x11
write FF46=0xC0; tick 1
assert read(C000) == 0xFF      # blocked
assert read(FF80) == 0x11      # HRAM ok
write C001=0x99                # ignored
tick until done
assert read(C000) == 0xAB      # WRAM visible again
assert read(C001) == 0x00      # write did not land
```

  **Done when:** `oam_dma_cpu_access_limited_to_hram` locks the policy.

---

- [x] **OAM usable by PPU after DMA**

  **What:** DMA only matters if the PPU composites the **same** OAM buffer. After
  160 M-cycles, `bus.oam` holds sprite Y/X/tile/flags; `ppu.render_frame(&vram, &oam)`
  must show an object where those bytes say. SoftGBC keeps one `oam: [u8; 0xA0]` on
  the bus — DMA writes it, PPU reads it. No second shadow copy.

  **Structure:**
```text
  WRAM shadow          DMA (160 M)         bus.oam           PPU
  ┌─────────┐         ═══════════►       ┌─────────┐      ┌─────────┐
  │ Y X T F │                            │ Y X T F │ ───► │ sprite  │
  │ …       │                            │ …       │      │ at (x,y)│
  └─────────┘                            └─────────┘      └─────────┘

  after DMA inactive:
    ppu.render_frame(vram, bus.oam)
    assert pixel at sprite screen pos == expected shade
```

  **Min test:**
```text
# tile 1 solid id 3; OAM in WRAM: Y=56,X=48,tile=1 → screen (40,40)
write FF46=0xC0; tick 160 M
render_frame
assert pixel(40,40) == shade 3
assert pixel(0,0) == shade 0   # BG empty
```

  **Done when:** `oam_dma_then_ppu_sees_sprite` is green.
