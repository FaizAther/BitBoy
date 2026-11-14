# Serial

Link cable (`SB` / `SC`). Low priority for games; **high value for headless test ROMs**.

**Implementation:** [`serial.rs`](serial.rs) on `DmgBus` (`FF01`/`FF02`).  
**Refs:** [Pan Docs — Serial](https://gbdev.io/pandocs/Serial_Data_Transfer_(Link_Cable).html) · [`irq.rs`](irq.rs) `IF_SERIAL`  

## Status *(2026-11)*

| Item | State |
|---|---|
| `FF01`/`FF02` R/W | Done |
| Capture `SB` on `$81` start (instant complete) | Done |
| SoftGbc stops on serial `Passed`/`Failed` | Done |
| Transfer-complete → `IF` bit 3 | Open |

```text
IRQ fires on transfer COMPLETE (clear SC.7 + IF bit 3 / $0058)
NOT when you write $81 to start
```

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [x] **Reads/writes `FF01` / `FF02`**  
  **Done when:** no hang on Blargg serial traffic.

- [x] **Capture output on internal-clock start**  
  **Done when:** `serial_output()` gets Blargg text.

- [x] **Wired through `DmgBus`**  
  **Done when:** not a Blargg-only bus.

---

- [ ] **Transfer complete → clear `SC.7` + `IF` bit 3**

  **What:** finishing a byte clears busy and requests serial IRQ. Instant stub may complete in the same step as start.  
  **Structure:**
```text
on complete (instant: same as start+$81):
  append SB to host log (optional)
  SC.7 = 0
  request IF bit 3

bus:
  IF = IF | 0x08
```
  **Min test:**
```text
write SB = 'A'
write SC = $81
assert SC.7 == 0
assert (IF & 0x08) != 0
```
  **Done when:** assert above; optional: IME+IE → PC lands on `$0058`.

---

- [ ] **Timed transfer via `tick` (~128 T)**

  **What:** 8 bits × 16 T; complete in `tick`, not in the start write.  
  **Structure:**
```text
on start: busy=1, remain=128
tick(t):
  remain -= t
  if remain <= 0: complete()   # clear SC.7, IF bit 3
```
  **Min test:**
```text
start transfer
tick(127) → still busy, IF bit 3 clear
tick(1)   → SC.7=0, IF bit 3 set
```
  **Done when:** timing test green; Blargg still captures text.

---

- [ ] **Loopback or null peer**

  **What:** after shift, `SB` is echo or `$FF` if no peer.  
  **Structure:**
```text
on complete:
  SB = loopback ? old_SB : 0xFF
```
  **Min test:**
```text
SB=0x42, start, complete
assert SB == 0x42 (loopback) or 0xFF (null)
```
  **Done when:** policy documented and tested.

---

- [ ] **Real peer / file-backed link (later)**

  **What:** bit-level peer for link-cable games.  
  **Structure:**
```text
peer.shift_bit(out_bit) → in_bit
repeat 8 times per byte
```
  **Min test:** mock peer records 8 outbound bits.  
  **Done when:** one known exchange matches.
