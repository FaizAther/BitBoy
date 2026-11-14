# CPU hex fixtures

Hand-written **machine code** programs for building / debugging `sm83-iss` **one opcode at a time**.

These are **not** Game Boy cartridge images and **not** Intel HEX. They are tiny commented byte lists.

Full `.gb` / Blargg ROMs live under [`../../tests/roms/`](../../tests/roms/). Pokemon Crystal is the wrong tool for CPU bring-up.

---

## Current status

| Fixture | Opcodes | Status |
|---|---|---|
| `01_nop.hex` | `NOP` | Present (early ladder) |
| `02_jr_spin.hex` | `NOP`, `JR e` | Present |
| `03_ld_imm.hex` | `LD r,n` | Present |
| `04+` | … | **Not required** — ISA finished via `tests/every_opcode.rs` + unit tests |

The fixture ladder was the learning path. New opcodes (if any quirks) can still get a tiny hex file, but Blargg + white-box tests are the authority now.

Loader: `softgbc_core::hex_load` / `DmgBus::from_hex_file` (see `softgbc-core`).

---

## Format

```text
# comment
# requires: NOP, JR
# entry: 0100

@0100          # next bytes are placed at this address (hex)
00             # NOP
00             # NOP
18 FE          # JR -2  (infinite loop at this JR)
```

Rules:

- `#` starts a comment (rest of line ignored)
- `@AAAA` sets the load address for following bytes
- Tokens are hex bytes (`00`, `FF`, `18`) separated by whitespace
- Multiple bytes per line are fine
- Default load address is `0000` if no `@` yet
- `# requires:` documents which opcodes the fixture needs (human checklist)
- `# entry:` documents where PC should start (usually `0100`)

## How to use

1. Pick a bug or missing quirk to exercise.
2. Keep the fixture **minimal** (only opcodes you need).
3. Load it, set `PC` to `# entry:`, `step()` and assert.
4. Prefer Blargg / `every_opcode` for broad coverage.

## Suggested build order (historical)

| File | Opcodes exercised | Goal |
|---|---|---|
| `01_nop.hex` | `NOP` | PC advances |
| `02_jr_spin.hex` | `NOP`, `JR e` | Branch works |
| `03_ld_imm.hex` | `LD r,n` | Immediate loads |
| `04_ld_reg.hex` | `LD r,r` | Reg-reg moves *(skipped — covered by tests)* |
| `05_alu_add.hex` | `ADD`, flags | ALU + `F` *(skipped — covered by tests)* |

Rename / add files as you go; keep each fixture **minimal**.
