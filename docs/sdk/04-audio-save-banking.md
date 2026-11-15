# 04 — Audio, persistent saves, and banked ROM

Add larger-game facilities only after ROM32 graphics/input games are stable.
All three features cross timing or address-space boundaries and require explicit
handles rather than ordinary pointers.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Audio command model**

  **What:** Game code enqueues music/SFX commands; a deterministic tick updates
  APU registers. Public APIs never busy-wait on audio hardware and report queue/
  channel conflicts through a documented priority policy.

  **Structure:**
```text
main context:
  play song / stop / play SFX(volume, priority)
        ↓
fixed command queue
        ↓ timer/VBlank tick
sequencer → APU register writes
```

  **Min test:**
```text
play/stop sequence emits expected timestamped register-write trace
queue overflow and SFX priority are deterministic
```

  **Done when:** SoftGBC can validate audio register streams before audible
  output is perfect.

---

- [ ] **Audio asset conversion**

  **What:** Convert a constrained tracker/song format and WAV samples into
  compact instrument/pattern/wave data. Reject unsupported rates/channels and
  report ROM/RAM cost.

  **Structure:**
```text
source song/SFX
  → validate SDK-supported subset
  → quantize notes/envelopes/wave data
  → typed descriptor + bankable data
```

  **Min test:**
```text
tiny scale/song has exact converted bytes and register trace
unsupported effect reports pattern/row
```

  **Done when:** example game references named audio assets without raw register
  tables.

---

- [ ] **Save schema and SRAM driver**

  **What:** Persist a versioned fixed-size save record to cartridge RAM with
  magic, schema version, payload length, and checksum. Use two slots or another
  documented commit strategy so interrupted writes retain one valid copy.

  **Structure:**
```text
slot:
  magic | schema | generation | length | payload | checksum

load:
  validate slots → choose newest valid → migrate/default

save:
  write inactive slot payload/checksum → commit generation last
```

  **Min test:**
```text
empty RAM → defaults
valid save round-trip
corrupt newest slot → older slot recovered
simulated interruption at each write step
```

  **Done when:** save loss/corruption behavior is tested and never reads beyond
  declared payload.

---

- [ ] **Typed game save API**

  **What:** Let a project declare one save struct/schema and explicit migration
  functions. Avoid serializing compiler padding/pointers accidentally; generated
  code defines field encoding.

  **Structure:**
```text
manifest save schema
  → generated encode/decode declarations

game:
  gbc_save_load(&state)
  gbc_save_commit(&state)
```

  **Min test:**
```text
golden bytes for every field type
schema v1 → v2 migration
unsupported pointer/variable array rejected at build time
```

  **Done when:** save bytes remain stable across optimization levels/tool hosts.

---

- [ ] **MBC5 bank manager**

  **What:** Add `gbc-mbc5` profile with one owner of ROM/RAM bank registers.
  Bank changes occur in short critical sections and restore the prior bank when
  using scoped access.

  **Structure:**
```text
bank manager state:
  current ROM bank
  current RAM bank
  nesting/scoped token policy

select bank → MBC5 MMIO → update shadow
```

  **Min test:**
```text
select low/high ROM bank values
nested scoped switch restores previous bank
interrupt during protected transition cannot observe half-written selection
```

  **Done when:** no library writes MBC registers outside the manager.

---

- [ ] **Far asset handles**

  **What:** Represent banked data as `{bank, address, length/type}` descriptors.
  Ordinary C pointers remain 16-bit and valid only in the currently mapped
  window. APIs copy/stream through a scoped bank selection.

  **Structure:**
```c
typedef struct {
    uint16_t bank;
    uint16_t address;
    uint16_t size;
} gbc_far_asset_t;
```
```text
far asset → select bank → bounded copy/use → restore bank
```

  **Min test:**
```text
two same-address assets in different banks return distinct expected bytes
out-of-range length/invalid bank rejected in debug profile
```

  **Done when:** generated asset APIs never cast far handles to ordinary pointers.

---

- [ ] **Banked code and far-call thunks**

  **What:** Keep runtime/ISRs/common code in fixed bank. Optional banked
  functions call through linker-generated or SDK thunks that switch bank,
  invoke, and restore. Recursion/nesting policy is explicit.

  **Structure:**
```text
fixed-bank caller
  → far-call thunk(bank, address)
      save prior bank
      select target
      indirect-call trampoline
      restore prior bank
  → caller
```

  **Min test:**
```text
call functions in two banks with nested normal calls
interrupt during far call executes fixed-bank ISR safely
direct illegal cross-bank relocation fails at link time
```

  **Done when:** banked calls pass ABI traces and map report names every thunk.

---

- [ ] **Cross-feature stress ROM**

  **What:** Exercise streamed banked graphics, music/SFX, and periodic save
  commits together. This catches ownership and timing conflicts absent from unit
  tests.

  **Structure:**
```text
scripted scene:
  banked map transition
  music continues + SFX
  save checkpoint
  reset/reload
```

  **Min test:**
```text
replay reaches expected visual/audio-register/save hashes
no queue/bank assertion or lost save
```

  **Done when:** MBC5 SDK profile has one integrated deterministic demo.
