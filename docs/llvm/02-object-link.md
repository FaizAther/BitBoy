# 02 — ELF objects, relocations, linking, and ROM packing

Turn MC output into linkable artifacts, then into deterministic cartridge
images. Keep ELF for symbols/debugging; produce `.gbc` only at the final step.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **SM83 ELF object writer**

  **What:** Emit ELF32 little-endian objects with the target/ABI identity from
  [`00-contracts.md`](00-contracts.md). Preserve sections, symbols, binding,
  visibility, and relocation addends needed by LLD.

  **Structure:**
```text
MC fragments
  → ELF32 sections (.text/.rodata/.data/.bss/…)
  → symbol table
  → SM83 relocations
  → ABI identity
```

  **Min test:**
```text
llvm-readobj --file-headers --sections --symbols object.o
checks class, endianness, machine, ABI, sizes, symbols
```

  **Done when:** LLVM tools inspect objects without treating them as generic or
  corrupt ELF.

---

- [ ] **Relocation set**

  **What:** Define the smallest complete relocation ABI: absolute 8/16-bit and
  PC-relative 8-bit first. Add bank/far-call relocations only with the SDK
  banking design; do not overload absolute relocations with hidden bank state.

  **Structure:**
```text
R_SM83_NONE
R_SM83_8
R_SM83_16
R_SM83_PCREL_8

future, explicit:
  bank number / banked address / far-call thunk relocations
```

  **Min test:**
```text
llvm-readobj --relocations shows type/symbol/addend
LLD applies positive/negative PC-relative values
overflow for each relocation fails with symbol and range
```

  **Done when:** relocation names, formulas, ranges, and overflow behavior are
  documented and tested in MC and LLD.

---

- [ ] **LLD target support**

  **What:** Teach ELF LLD the SM83 machine, relocation expressions, address
  width checks, and output identity. Use normal linker-script placement rather
  than a target-specific hard-coded Game Boy layout.

  **Structure:**
```text
input SM83 ELF objects
  → validate machine/ABI
  → scan/apply relocations
  → layout from script
  → linked ELF + map
```

  **Min test:**
```text
two objects call/reference each other and link
undefined symbol, duplicate symbol, relocation overflow, ABI mismatch fail
```

  **Done when:** `ld.lld -m sm83elf` (exact spelling to freeze) links normal
  sections and emits useful map/symbol data.

---

- [ ] **ROM-only linker script**

  **What:** Create the first `gbc-rom32` script: vectors/header reservations,
  code/rodata in ROM, initialized-data load image in ROM, runtime data/BSS in
  WRAM, stack/HRAM reservations, and exported startup symbols.

  **Structure:**
```text
ROM $0000-$7FFF:
  vectors / entry / header reservation
  .text .rodata
  .data load image

WRAM:
  .data runtime
  .bss
  stack bounds

exports:
  __data_load/start/end
  __bss_start/end
  __stack_top
```

  **Min test:**
```text
link map asserts no overlap with cartridge header/vectors
.data has distinct ROM load and WRAM runtime addresses
oversized ROM/WRAM fails at link time with region name
```

  **Done when:** startup can initialize data entirely from linker symbols.

---

- [ ] **Cartridge header and ROM packer**

  **What:** Convert linked ELF to a padded `.gbc`, inject/validate entry and
  cartridge header fields, then compute header/global checksums. Keep game
  metadata in an SDK manifest; do not require source to hand-author header
  bytes.

  **Structure:**
```text
linked ELF + manifest
  → extract loadable ROM sections
  → pad with deterministic byte
  → write logo/entry/title/CGB/cart/size fields
  → compute checksums
  → emit game.gbc + map + symbol file
```

  **Min test:**
```text
known tiny ELF → deterministic SHA-256
header validator checks entry, CGB flag, ROM size, checksums
bad/overlapping load segment fails
```

  **Done when:** repeated builds produce byte-identical ROMs and SoftGBC loads
  the image at `$0100`.

---

- [ ] **Debug and symbol preservation**

  **What:** Keep linked ELF as the debugger artifact while `.gbc` is deployment
  output. Addresses and symbols must correspond after packing; optional DWARF
  can arrive incrementally.

  **Structure:**
```text
build outputs:
  game.elf   symbols/debug
  game.gbc   cartridge image
  game.map   section/memory report
```

  **Min test:**
```text
symbol main address in ELF/map matches ROM byte location
QEMU GDB breakpoint at symbol reaches expected source/assembly address
```

  **Done when:** packaging does not discard the artifact needed for debugging.

---

- [ ] **Banked-ROM extension boundary**

  **What:** Defer banked linking until ROM32 is stable. Then add explicit
  sections/regions and thunk policy for MBC5; ordinary 16-bit pointers cannot
  represent arbitrary banked code/data.

  **Structure:**
```text
fixed bank 0:
  vectors, runtime, bank manager, common code

switchable banks:
  attributed/manifest-assigned sections
  far-call/far-data handles
  generated map with bank + address
```

  **Min test:**
```text
two functions in different banks link through documented thunk
linker rejects direct 16-bit relocation across incompatible banks
```

  **Done when:** `gbc-mbc5` is additive and cannot silently corrupt ROM32 ABI.
