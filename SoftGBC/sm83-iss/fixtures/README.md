# CPU hex fixtures

Hand-written **machine code** programs for building `sm83-iss` **one opcode at a time**.

These are **not** Game Boy cartridge images and **not** Intel HEX. They are tiny commented byte lists you grow as you implement instructions.

Full `.gb` / Blargg ROMs come later. Pokemon Crystal is the wrong tool for this phase.

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

1. Pick the next opcode to implement.
2. Add or extend a fixture that **only** needs opcodes you already have + the new one.
3. Run the ISS on that fixture; assert PC / regs / memory.
4. When green, move to the next opcode.

## Suggested build order (fixtures)

| File | Opcodes exercised | Goal |
|---|---|---|
| `01_nop.hex` | `NOP` | PC advances |
| `02_jr_spin.hex` | `NOP`, `JR e` | Branch works |
| `03_ld_imm.hex` | `LD r,n` | Immediate loads |
| `04_ld_reg.hex` | `LD r,r` | Reg-reg moves |
| `05_alu_add.hex` | `ADD`, flags | ALU + `F` |
| … | … | Keep adding |

Rename / add files as you go; keep each fixture **minimal**.
