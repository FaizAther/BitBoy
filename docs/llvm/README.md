# LLVM SM83 toolchain — approach (Chunks C–D)

Build a freestanding LLVM target that turns C (then Rust/Zig through LLVM IR)
into SM83 objects and Game Boy Color ROMs. The compiler owns code generation;
the GBC SDK owns hardware policy and game APIs.

**Refs:** repo [`README.md`](../../README.md) Chunks C–D ·
[`../qemu/README.md`](../qemu/README.md) · SoftGBC
[`sm83-iss`](../../SoftGBC/sm83-iss/README.md) · LLVM in `llvm-project/`

## Outcome

```text
 C / LLVM IR
      │
      ▼
 clang / llc  --target=sm83-unknown-none
      │
      ▼
 ELF32 SM83 objects ──► ld.lld + GBC linker script
      │                         │
      │                         ▼
      └──────────────────► linked ELF ──► ROM packer ──► .gbc
                                                   │
                         ┌─────────────────────────┼──────────────┐
                         ▼                         ▼              ▼
                     SoftGBC                 QEMU SM83       RTL / FPGA
```

The first supported source language is freestanding C. Other frontends become
supported when they can honor the same target data layout, ABI, and runtime
contracts; they do not get separate machine ABIs.

## Boundaries

| Layer | Owns |
|---|---|
| LLVM target | registers, instructions, MC, relocations, lowering, ABI |
| LLD/ROM packer | sections, relocation, flat image, cartridge header/checksums |
| compiler runtime | arithmetic/memory helpers required by generated code |
| GBC SDK | startup policy, interrupts, PPU/APU/input APIs, assets, game loop |
| SoftGBC/QEMU | execution and differential validation |

The CPU target is **SM83**, not “GBC.” Use the triple
`sm83-unknown-none`; select GBC hardware through SDK sysroot/linker profiles.

## Initial technical contract

```text
endianness:       little
address width:    16-bit
pointer width:    16-bit
legal core types: i8, i16
C char:           8-bit
C short/int:      16-bit
C long:           32-bit
object container: ELF32 little-endian (local provisional machine id)
execution:        freestanding; no hosted libc or OS
```

These values become ABI once released. Exact DataLayout, register preservation,
relocations, and section rules are frozen in [`00-contracts.md`](00-contracts.md)
and [`04-abi.md`](04-abi.md).

## Bring-up order

```text
00 contracts / test oracle
        ↓
01 MC: asm parser + printer + encoder + disassembler
        ↓
02 ELF objects + relocations + LLD + ROM image
        ↓
03 TargetMachine + legalization + instruction selection
        ↓
04 calling convention + frames + calls
        ↓
05 compiler-rt subset + driver/sysroot integration
        ↓
06 differential validation + optimization
```

MC comes before C codegen. Every instruction the backend emits must already
round-trip through assembler/disassembler and execute under SoftGBC.

## Source layout

```text
llvm-project/
├── llvm/lib/Target/SM83/
├── llvm/lib/TargetParser/          # triple registration as required
├── llvm/test/MC/SM83/
├── llvm/test/CodeGen/SM83/
├── clang/lib/Basic/Targets/SM83.*
├── clang/test/CodeGen/sm83-*
└── lld/ELF/                        # SM83 relocation support

runtime/
├── abi/
├── compiler-rt/
├── crt/
├── linker/
└── tools/                          # ELF → .gbc packer if not objcopy
```

Generated TableGen files stay in build directories. GBC SDK headers and device
libraries do not live under `llvm/lib/Target/SM83`.

## Correctness authorities

```text
encoding authority:  LLVM MC round-trip + documented SM83 table
CPU semantics:       SoftGBC sm83-iss
ABI:                 versioned docs + ABI conformance ROMs
ROM layout:          linker-map tests + cartridge-header validator
end-to-end:          SoftGBC, then QEMU, then RTL/FPGA
```

Compiler tests are layered:

1. `llvm-mc` bytes and diagnostics;
2. `llc` assembly/FileCheck;
3. object relocation and linker layout;
4. executable ROM state/trace tests;
5. GBC SDK examples.

## Checklist pages

| Page | Deliverable |
|---|---|
| [`00-contracts.md`](00-contracts.md) | Triple, data layout, object/ABI version, test matrix |
| [`01-mc.md`](01-mc.md) | Registers, TableGen instructions, asm/disasm, fixups |
| [`02-object-link.md`](02-object-link.md) | ELF, relocations, LLD, sections, ROM packing |
| [`03-codegen.md`](03-codegen.md) | TargetMachine, legalization, selection, branches |
| [`04-abi.md`](04-abi.md) | Calling convention, frames, globals, interrupts |
| [`05-runtime-driver.md`](05-runtime-driver.md) | Clang target, sysroot, crt, compiler helpers |
| [`06-validation.md`](06-validation.md) | MC/codegen/runtime parity and optimization gates |

## Toolchain exit gate

- [ ] Hand assembly round-trips and links into a header-valid 32 KiB ROM.
- [ ] `clang --target=sm83-unknown-none` compiles the supported freestanding C subset.
- [ ] ABI conformance ROMs pass in SoftGBC and QEMU.
- [ ] `sum(20,22)` writes `$2E` to `$FF80` and exits through the test profile.
- [ ] Unsupported language features fail clearly or lower to documented helpers.
- [ ] The SDK can consume the installed compiler/sysroot without source-tree paths.

The first release prioritizes stable correctness and diagnostics over code size.
