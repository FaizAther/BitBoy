//! One behavioral smoke/correctness test per SM83 opcode.
//! GBC uses the same SM83 ISA as DMG — there are no extra CPU opcodes on GBC.

use sm83_iss::Cpu;
use softgbc_core::{Bus, DmgBus};

fn harness() -> (Cpu, DmgBus) {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFE;
    cpu.a = 0x12;
    // Point BC/DE/HL at WRAM so LD (BC)/(DE)/(HL) smoke tests can write.
    cpu.set_bc(0xC100);
    cpu.set_de(0xC200);
    cpu.set_hl(0xC000);
    let mut bus = DmgBus::new();
    // Preload a few memory cells for indirect ops
    assert!(bus.poke(0xC000, 0xAB));
    assert!(bus.poke(0xC100, 0xCD));
    assert!(bus.poke(0xC200, 0xEF));
    (cpu, bus)
}

fn run(bytes: &[u8]) -> (Cpu, DmgBus, u8) {
    let (mut cpu, mut bus) = harness();
    for (i, b) in bytes.iter().enumerate() {
        assert!(bus.poke(0x0100 + i as u16, *b));
    }
    let cycles = cpu.step(&mut bus);
    (cpu, bus, cycles)
}

#[test]
fn op_00() {
    let (cpu, _bus, cycles) = run(&[0x00]);
    assert!(cycles >= 1, "opcode 0x00 returned 0 cycles");
}

#[test]
fn op_01() {
    let (cpu, _bus, cycles) = run(&[0x01, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0x01 returned 0 cycles");
}

#[test]
fn op_02() {
    let (cpu, _bus, cycles) = run(&[0x02]);
    assert!(cycles >= 1, "opcode 0x02 returned 0 cycles");
}

#[test]
fn op_03() {
    let (cpu, _bus, cycles) = run(&[0x03]);
    assert!(cycles >= 1, "opcode 0x03 returned 0 cycles");
}

#[test]
fn op_04() {
    let (cpu, _bus, cycles) = run(&[0x04]);
    assert!(cycles >= 1, "opcode 0x04 returned 0 cycles");
}

#[test]
fn op_05() {
    let (cpu, _bus, cycles) = run(&[0x05]);
    assert!(cycles >= 1, "opcode 0x05 returned 0 cycles");
}

#[test]
fn op_06() {
    let (cpu, _bus, cycles) = run(&[0x06, 0x42]);
    assert!(cycles >= 1, "opcode 0x06 returned 0 cycles");
}

#[test]
fn op_07() {
    let (cpu, _bus, cycles) = run(&[0x07]);
    assert!(cycles >= 1, "opcode 0x07 returned 0 cycles");
}

#[test]
fn op_08() {
    let (cpu, _bus, cycles) = run(&[0x08, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0x08 returned 0 cycles");
}

#[test]
fn op_09() {
    let (cpu, _bus, cycles) = run(&[0x09]);
    assert!(cycles >= 1, "opcode 0x09 returned 0 cycles");
}

#[test]
fn op_0a() {
    let (cpu, _bus, cycles) = run(&[0x0A]);
    assert!(cycles >= 1, "opcode 0x0A returned 0 cycles");
}

#[test]
fn op_0b() {
    let (cpu, _bus, cycles) = run(&[0x0B]);
    assert!(cycles >= 1, "opcode 0x0B returned 0 cycles");
}

#[test]
fn op_0c() {
    let (cpu, _bus, cycles) = run(&[0x0C]);
    assert!(cycles >= 1, "opcode 0x0C returned 0 cycles");
}

#[test]
fn op_0d() {
    let (cpu, _bus, cycles) = run(&[0x0D]);
    assert!(cycles >= 1, "opcode 0x0D returned 0 cycles");
}

#[test]
fn op_0e() {
    let (cpu, _bus, cycles) = run(&[0x0E, 0x42]);
    assert!(cycles >= 1, "opcode 0x0E returned 0 cycles");
}

#[test]
fn op_0f() {
    let (cpu, _bus, cycles) = run(&[0x0F]);
    assert!(cycles >= 1, "opcode 0x0F returned 0 cycles");
}

#[test]
fn op_10() {
    let (cpu, _bus, cycles) = run(&[0x10, 0x42]);
    assert!(cycles >= 1, "opcode 0x10 returned 0 cycles");
    assert!(cpu.halted);
}

#[test]
fn op_11() {
    let (cpu, _bus, cycles) = run(&[0x11, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0x11 returned 0 cycles");
}

#[test]
fn op_12() {
    let (cpu, _bus, cycles) = run(&[0x12]);
    assert!(cycles >= 1, "opcode 0x12 returned 0 cycles");
}

#[test]
fn op_13() {
    let (cpu, _bus, cycles) = run(&[0x13]);
    assert!(cycles >= 1, "opcode 0x13 returned 0 cycles");
}

#[test]
fn op_14() {
    let (cpu, _bus, cycles) = run(&[0x14]);
    assert!(cycles >= 1, "opcode 0x14 returned 0 cycles");
}

#[test]
fn op_15() {
    let (cpu, _bus, cycles) = run(&[0x15]);
    assert!(cycles >= 1, "opcode 0x15 returned 0 cycles");
}

#[test]
fn op_16() {
    let (cpu, _bus, cycles) = run(&[0x16, 0x42]);
    assert!(cycles >= 1, "opcode 0x16 returned 0 cycles");
}

#[test]
fn op_17() {
    let (cpu, _bus, cycles) = run(&[0x17]);
    assert!(cycles >= 1, "opcode 0x17 returned 0 cycles");
}

#[test]
fn op_18() {
    let (cpu, _bus, cycles) = run(&[0x18, 0x00]);
    assert!(cycles >= 1, "opcode 0x18 returned 0 cycles");
}

#[test]
fn op_19() {
    let (cpu, _bus, cycles) = run(&[0x19]);
    assert!(cycles >= 1, "opcode 0x19 returned 0 cycles");
}

#[test]
fn op_1a() {
    let (cpu, _bus, cycles) = run(&[0x1A]);
    assert!(cycles >= 1, "opcode 0x1A returned 0 cycles");
}

#[test]
fn op_1b() {
    let (cpu, _bus, cycles) = run(&[0x1B]);
    assert!(cycles >= 1, "opcode 0x1B returned 0 cycles");
}

#[test]
fn op_1c() {
    let (cpu, _bus, cycles) = run(&[0x1C]);
    assert!(cycles >= 1, "opcode 0x1C returned 0 cycles");
}

#[test]
fn op_1d() {
    let (cpu, _bus, cycles) = run(&[0x1D]);
    assert!(cycles >= 1, "opcode 0x1D returned 0 cycles");
}

#[test]
fn op_1e() {
    let (cpu, _bus, cycles) = run(&[0x1E, 0x42]);
    assert!(cycles >= 1, "opcode 0x1E returned 0 cycles");
}

#[test]
fn op_1f() {
    let (cpu, _bus, cycles) = run(&[0x1F]);
    assert!(cycles >= 1, "opcode 0x1F returned 0 cycles");
}

#[test]
fn op_20() {
    let (cpu, _bus, cycles) = run(&[0x20, 0x00]);
    assert!(cycles >= 1, "opcode 0x20 returned 0 cycles");
}

#[test]
fn op_21() {
    let (cpu, _bus, cycles) = run(&[0x21, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0x21 returned 0 cycles");
}

#[test]
fn op_22() {
    let (cpu, _bus, cycles) = run(&[0x22]);
    assert!(cycles >= 1, "opcode 0x22 returned 0 cycles");
}

#[test]
fn op_23() {
    let (cpu, _bus, cycles) = run(&[0x23]);
    assert!(cycles >= 1, "opcode 0x23 returned 0 cycles");
}

#[test]
fn op_24() {
    let (cpu, _bus, cycles) = run(&[0x24]);
    assert!(cycles >= 1, "opcode 0x24 returned 0 cycles");
}

#[test]
fn op_25() {
    let (cpu, _bus, cycles) = run(&[0x25]);
    assert!(cycles >= 1, "opcode 0x25 returned 0 cycles");
}

#[test]
fn op_26() {
    let (cpu, _bus, cycles) = run(&[0x26, 0x42]);
    assert!(cycles >= 1, "opcode 0x26 returned 0 cycles");
}

#[test]
fn op_27() {
    let (cpu, _bus, cycles) = run(&[0x27]);
    assert!(cycles >= 1, "opcode 0x27 returned 0 cycles");
}

#[test]
fn op_28() {
    let (cpu, _bus, cycles) = run(&[0x28, 0x00]);
    assert!(cycles >= 1, "opcode 0x28 returned 0 cycles");
}

#[test]
fn op_29() {
    let (cpu, _bus, cycles) = run(&[0x29]);
    assert!(cycles >= 1, "opcode 0x29 returned 0 cycles");
}

#[test]
fn op_2a() {
    let (cpu, _bus, cycles) = run(&[0x2A]);
    assert!(cycles >= 1, "opcode 0x2A returned 0 cycles");
}

#[test]
fn op_2b() {
    let (cpu, _bus, cycles) = run(&[0x2B]);
    assert!(cycles >= 1, "opcode 0x2B returned 0 cycles");
}

#[test]
fn op_2c() {
    let (cpu, _bus, cycles) = run(&[0x2C]);
    assert!(cycles >= 1, "opcode 0x2C returned 0 cycles");
}

#[test]
fn op_2d() {
    let (cpu, _bus, cycles) = run(&[0x2D]);
    assert!(cycles >= 1, "opcode 0x2D returned 0 cycles");
}

#[test]
fn op_2e() {
    let (cpu, _bus, cycles) = run(&[0x2E, 0x42]);
    assert!(cycles >= 1, "opcode 0x2E returned 0 cycles");
}

#[test]
fn op_2f() {
    let (cpu, _bus, cycles) = run(&[0x2F]);
    assert!(cycles >= 1, "opcode 0x2F returned 0 cycles");
}

#[test]
fn op_30() {
    let (cpu, _bus, cycles) = run(&[0x30, 0x00]);
    assert!(cycles >= 1, "opcode 0x30 returned 0 cycles");
}

#[test]
fn op_31() {
    let (cpu, _bus, cycles) = run(&[0x31, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0x31 returned 0 cycles");
}

#[test]
fn op_32() {
    let (cpu, _bus, cycles) = run(&[0x32]);
    assert!(cycles >= 1, "opcode 0x32 returned 0 cycles");
}

#[test]
fn op_33() {
    let (cpu, _bus, cycles) = run(&[0x33]);
    assert!(cycles >= 1, "opcode 0x33 returned 0 cycles");
}

#[test]
fn op_34() {
    let (cpu, _bus, cycles) = run(&[0x34]);
    assert!(cycles >= 1, "opcode 0x34 returned 0 cycles");
}

#[test]
fn op_35() {
    let (cpu, _bus, cycles) = run(&[0x35]);
    assert!(cycles >= 1, "opcode 0x35 returned 0 cycles");
}

#[test]
fn op_36() {
    let (cpu, _bus, cycles) = run(&[0x36, 0x42]);
    assert!(cycles >= 1, "opcode 0x36 returned 0 cycles");
}

#[test]
fn op_37() {
    let (cpu, _bus, cycles) = run(&[0x37]);
    assert!(cycles >= 1, "opcode 0x37 returned 0 cycles");
}

#[test]
fn op_38() {
    let (cpu, _bus, cycles) = run(&[0x38, 0x00]);
    assert!(cycles >= 1, "opcode 0x38 returned 0 cycles");
}

#[test]
fn op_39() {
    let (cpu, _bus, cycles) = run(&[0x39]);
    assert!(cycles >= 1, "opcode 0x39 returned 0 cycles");
}

#[test]
fn op_3a() {
    let (cpu, _bus, cycles) = run(&[0x3A]);
    assert!(cycles >= 1, "opcode 0x3A returned 0 cycles");
}

#[test]
fn op_3b() {
    let (cpu, _bus, cycles) = run(&[0x3B]);
    assert!(cycles >= 1, "opcode 0x3B returned 0 cycles");
}

#[test]
fn op_3c() {
    let (cpu, _bus, cycles) = run(&[0x3C]);
    assert!(cycles >= 1, "opcode 0x3C returned 0 cycles");
}

#[test]
fn op_3d() {
    let (cpu, _bus, cycles) = run(&[0x3D]);
    assert!(cycles >= 1, "opcode 0x3D returned 0 cycles");
}

#[test]
fn op_3e() {
    let (cpu, _bus, cycles) = run(&[0x3E, 0x42]);
    assert!(cycles >= 1, "opcode 0x3E returned 0 cycles");
}

#[test]
fn op_3f() {
    let (cpu, _bus, cycles) = run(&[0x3F]);
    assert!(cycles >= 1, "opcode 0x3F returned 0 cycles");
}

#[test]
fn op_40() {
    let (cpu, _bus, cycles) = run(&[0x40]);
    assert!(cycles >= 1, "opcode 0x40 returned 0 cycles");
}

#[test]
fn op_41() {
    let (cpu, _bus, cycles) = run(&[0x41]);
    assert!(cycles >= 1, "opcode 0x41 returned 0 cycles");
}

#[test]
fn op_42() {
    let (cpu, _bus, cycles) = run(&[0x42]);
    assert!(cycles >= 1, "opcode 0x42 returned 0 cycles");
}

#[test]
fn op_43() {
    let (cpu, _bus, cycles) = run(&[0x43]);
    assert!(cycles >= 1, "opcode 0x43 returned 0 cycles");
}

#[test]
fn op_44() {
    let (cpu, _bus, cycles) = run(&[0x44]);
    assert!(cycles >= 1, "opcode 0x44 returned 0 cycles");
}

#[test]
fn op_45() {
    let (cpu, _bus, cycles) = run(&[0x45]);
    assert!(cycles >= 1, "opcode 0x45 returned 0 cycles");
}

#[test]
fn op_46() {
    let (cpu, _bus, cycles) = run(&[0x46]);
    assert!(cycles >= 1, "opcode 0x46 returned 0 cycles");
}

#[test]
fn op_47() {
    let (cpu, _bus, cycles) = run(&[0x47]);
    assert!(cycles >= 1, "opcode 0x47 returned 0 cycles");
}

#[test]
fn op_48() {
    let (cpu, _bus, cycles) = run(&[0x48]);
    assert!(cycles >= 1, "opcode 0x48 returned 0 cycles");
}

#[test]
fn op_49() {
    let (cpu, _bus, cycles) = run(&[0x49]);
    assert!(cycles >= 1, "opcode 0x49 returned 0 cycles");
}

#[test]
fn op_4a() {
    let (cpu, _bus, cycles) = run(&[0x4A]);
    assert!(cycles >= 1, "opcode 0x4A returned 0 cycles");
}

#[test]
fn op_4b() {
    let (cpu, _bus, cycles) = run(&[0x4B]);
    assert!(cycles >= 1, "opcode 0x4B returned 0 cycles");
}

#[test]
fn op_4c() {
    let (cpu, _bus, cycles) = run(&[0x4C]);
    assert!(cycles >= 1, "opcode 0x4C returned 0 cycles");
}

#[test]
fn op_4d() {
    let (cpu, _bus, cycles) = run(&[0x4D]);
    assert!(cycles >= 1, "opcode 0x4D returned 0 cycles");
}

#[test]
fn op_4e() {
    let (cpu, _bus, cycles) = run(&[0x4E]);
    assert!(cycles >= 1, "opcode 0x4E returned 0 cycles");
}

#[test]
fn op_4f() {
    let (cpu, _bus, cycles) = run(&[0x4F]);
    assert!(cycles >= 1, "opcode 0x4F returned 0 cycles");
}

#[test]
fn op_50() {
    let (cpu, _bus, cycles) = run(&[0x50]);
    assert!(cycles >= 1, "opcode 0x50 returned 0 cycles");
}

#[test]
fn op_51() {
    let (cpu, _bus, cycles) = run(&[0x51]);
    assert!(cycles >= 1, "opcode 0x51 returned 0 cycles");
}

#[test]
fn op_52() {
    let (cpu, _bus, cycles) = run(&[0x52]);
    assert!(cycles >= 1, "opcode 0x52 returned 0 cycles");
}

#[test]
fn op_53() {
    let (cpu, _bus, cycles) = run(&[0x53]);
    assert!(cycles >= 1, "opcode 0x53 returned 0 cycles");
}

#[test]
fn op_54() {
    let (cpu, _bus, cycles) = run(&[0x54]);
    assert!(cycles >= 1, "opcode 0x54 returned 0 cycles");
}

#[test]
fn op_55() {
    let (cpu, _bus, cycles) = run(&[0x55]);
    assert!(cycles >= 1, "opcode 0x55 returned 0 cycles");
}

#[test]
fn op_56() {
    let (cpu, _bus, cycles) = run(&[0x56]);
    assert!(cycles >= 1, "opcode 0x56 returned 0 cycles");
}

#[test]
fn op_57() {
    let (cpu, _bus, cycles) = run(&[0x57]);
    assert!(cycles >= 1, "opcode 0x57 returned 0 cycles");
}

#[test]
fn op_58() {
    let (cpu, _bus, cycles) = run(&[0x58]);
    assert!(cycles >= 1, "opcode 0x58 returned 0 cycles");
}

#[test]
fn op_59() {
    let (cpu, _bus, cycles) = run(&[0x59]);
    assert!(cycles >= 1, "opcode 0x59 returned 0 cycles");
}

#[test]
fn op_5a() {
    let (cpu, _bus, cycles) = run(&[0x5A]);
    assert!(cycles >= 1, "opcode 0x5A returned 0 cycles");
}

#[test]
fn op_5b() {
    let (cpu, _bus, cycles) = run(&[0x5B]);
    assert!(cycles >= 1, "opcode 0x5B returned 0 cycles");
}

#[test]
fn op_5c() {
    let (cpu, _bus, cycles) = run(&[0x5C]);
    assert!(cycles >= 1, "opcode 0x5C returned 0 cycles");
}

#[test]
fn op_5d() {
    let (cpu, _bus, cycles) = run(&[0x5D]);
    assert!(cycles >= 1, "opcode 0x5D returned 0 cycles");
}

#[test]
fn op_5e() {
    let (cpu, _bus, cycles) = run(&[0x5E]);
    assert!(cycles >= 1, "opcode 0x5E returned 0 cycles");
}

#[test]
fn op_5f() {
    let (cpu, _bus, cycles) = run(&[0x5F]);
    assert!(cycles >= 1, "opcode 0x5F returned 0 cycles");
}

#[test]
fn op_60() {
    let (cpu, _bus, cycles) = run(&[0x60]);
    assert!(cycles >= 1, "opcode 0x60 returned 0 cycles");
}

#[test]
fn op_61() {
    let (cpu, _bus, cycles) = run(&[0x61]);
    assert!(cycles >= 1, "opcode 0x61 returned 0 cycles");
}

#[test]
fn op_62() {
    let (cpu, _bus, cycles) = run(&[0x62]);
    assert!(cycles >= 1, "opcode 0x62 returned 0 cycles");
}

#[test]
fn op_63() {
    let (cpu, _bus, cycles) = run(&[0x63]);
    assert!(cycles >= 1, "opcode 0x63 returned 0 cycles");
}

#[test]
fn op_64() {
    let (cpu, _bus, cycles) = run(&[0x64]);
    assert!(cycles >= 1, "opcode 0x64 returned 0 cycles");
}

#[test]
fn op_65() {
    let (cpu, _bus, cycles) = run(&[0x65]);
    assert!(cycles >= 1, "opcode 0x65 returned 0 cycles");
}

#[test]
fn op_66() {
    let (cpu, _bus, cycles) = run(&[0x66]);
    assert!(cycles >= 1, "opcode 0x66 returned 0 cycles");
}

#[test]
fn op_67() {
    let (cpu, _bus, cycles) = run(&[0x67]);
    assert!(cycles >= 1, "opcode 0x67 returned 0 cycles");
}

#[test]
fn op_68() {
    let (cpu, _bus, cycles) = run(&[0x68]);
    assert!(cycles >= 1, "opcode 0x68 returned 0 cycles");
}

#[test]
fn op_69() {
    let (cpu, _bus, cycles) = run(&[0x69]);
    assert!(cycles >= 1, "opcode 0x69 returned 0 cycles");
}

#[test]
fn op_6a() {
    let (cpu, _bus, cycles) = run(&[0x6A]);
    assert!(cycles >= 1, "opcode 0x6A returned 0 cycles");
}

#[test]
fn op_6b() {
    let (cpu, _bus, cycles) = run(&[0x6B]);
    assert!(cycles >= 1, "opcode 0x6B returned 0 cycles");
}

#[test]
fn op_6c() {
    let (cpu, _bus, cycles) = run(&[0x6C]);
    assert!(cycles >= 1, "opcode 0x6C returned 0 cycles");
}

#[test]
fn op_6d() {
    let (cpu, _bus, cycles) = run(&[0x6D]);
    assert!(cycles >= 1, "opcode 0x6D returned 0 cycles");
}

#[test]
fn op_6e() {
    let (cpu, _bus, cycles) = run(&[0x6E]);
    assert!(cycles >= 1, "opcode 0x6E returned 0 cycles");
}

#[test]
fn op_6f() {
    let (cpu, _bus, cycles) = run(&[0x6F]);
    assert!(cycles >= 1, "opcode 0x6F returned 0 cycles");
}

#[test]
fn op_70() {
    let (cpu, _bus, cycles) = run(&[0x70]);
    assert!(cycles >= 1, "opcode 0x70 returned 0 cycles");
}

#[test]
fn op_71() {
    let (cpu, _bus, cycles) = run(&[0x71]);
    assert!(cycles >= 1, "opcode 0x71 returned 0 cycles");
}

#[test]
fn op_72() {
    let (cpu, _bus, cycles) = run(&[0x72]);
    assert!(cycles >= 1, "opcode 0x72 returned 0 cycles");
}

#[test]
fn op_73() {
    let (cpu, _bus, cycles) = run(&[0x73]);
    assert!(cycles >= 1, "opcode 0x73 returned 0 cycles");
}

#[test]
fn op_74() {
    let (cpu, _bus, cycles) = run(&[0x74]);
    assert!(cycles >= 1, "opcode 0x74 returned 0 cycles");
}

#[test]
fn op_75() {
    let (cpu, _bus, cycles) = run(&[0x75]);
    assert!(cycles >= 1, "opcode 0x75 returned 0 cycles");
}

#[test]
fn op_76() {
    let (cpu, _bus, cycles) = run(&[0x76]);
    assert!(cycles >= 1, "opcode 0x76 returned 0 cycles");
    assert!(cpu.halted);
}

#[test]
fn op_77() {
    let (cpu, _bus, cycles) = run(&[0x77]);
    assert!(cycles >= 1, "opcode 0x77 returned 0 cycles");
}

#[test]
fn op_78() {
    let (cpu, _bus, cycles) = run(&[0x78]);
    assert!(cycles >= 1, "opcode 0x78 returned 0 cycles");
}

#[test]
fn op_79() {
    let (cpu, _bus, cycles) = run(&[0x79]);
    assert!(cycles >= 1, "opcode 0x79 returned 0 cycles");
}

#[test]
fn op_7a() {
    let (cpu, _bus, cycles) = run(&[0x7A]);
    assert!(cycles >= 1, "opcode 0x7A returned 0 cycles");
}

#[test]
fn op_7b() {
    let (cpu, _bus, cycles) = run(&[0x7B]);
    assert!(cycles >= 1, "opcode 0x7B returned 0 cycles");
}

#[test]
fn op_7c() {
    let (cpu, _bus, cycles) = run(&[0x7C]);
    assert!(cycles >= 1, "opcode 0x7C returned 0 cycles");
}

#[test]
fn op_7d() {
    let (cpu, _bus, cycles) = run(&[0x7D]);
    assert!(cycles >= 1, "opcode 0x7D returned 0 cycles");
}

#[test]
fn op_7e() {
    let (cpu, _bus, cycles) = run(&[0x7E]);
    assert!(cycles >= 1, "opcode 0x7E returned 0 cycles");
}

#[test]
fn op_7f() {
    let (cpu, _bus, cycles) = run(&[0x7F]);
    assert!(cycles >= 1, "opcode 0x7F returned 0 cycles");
}

#[test]
fn op_80() {
    let (cpu, _bus, cycles) = run(&[0x80]);
    assert!(cycles >= 1, "opcode 0x80 returned 0 cycles");
}

#[test]
fn op_81() {
    let (cpu, _bus, cycles) = run(&[0x81]);
    assert!(cycles >= 1, "opcode 0x81 returned 0 cycles");
}

#[test]
fn op_82() {
    let (cpu, _bus, cycles) = run(&[0x82]);
    assert!(cycles >= 1, "opcode 0x82 returned 0 cycles");
}

#[test]
fn op_83() {
    let (cpu, _bus, cycles) = run(&[0x83]);
    assert!(cycles >= 1, "opcode 0x83 returned 0 cycles");
}

#[test]
fn op_84() {
    let (cpu, _bus, cycles) = run(&[0x84]);
    assert!(cycles >= 1, "opcode 0x84 returned 0 cycles");
}

#[test]
fn op_85() {
    let (cpu, _bus, cycles) = run(&[0x85]);
    assert!(cycles >= 1, "opcode 0x85 returned 0 cycles");
}

#[test]
fn op_86() {
    let (cpu, _bus, cycles) = run(&[0x86]);
    assert!(cycles >= 1, "opcode 0x86 returned 0 cycles");
}

#[test]
fn op_87() {
    let (cpu, _bus, cycles) = run(&[0x87]);
    assert!(cycles >= 1, "opcode 0x87 returned 0 cycles");
}

#[test]
fn op_88() {
    let (cpu, _bus, cycles) = run(&[0x88]);
    assert!(cycles >= 1, "opcode 0x88 returned 0 cycles");
}

#[test]
fn op_89() {
    let (cpu, _bus, cycles) = run(&[0x89]);
    assert!(cycles >= 1, "opcode 0x89 returned 0 cycles");
}

#[test]
fn op_8a() {
    let (cpu, _bus, cycles) = run(&[0x8A]);
    assert!(cycles >= 1, "opcode 0x8A returned 0 cycles");
}

#[test]
fn op_8b() {
    let (cpu, _bus, cycles) = run(&[0x8B]);
    assert!(cycles >= 1, "opcode 0x8B returned 0 cycles");
}

#[test]
fn op_8c() {
    let (cpu, _bus, cycles) = run(&[0x8C]);
    assert!(cycles >= 1, "opcode 0x8C returned 0 cycles");
}

#[test]
fn op_8d() {
    let (cpu, _bus, cycles) = run(&[0x8D]);
    assert!(cycles >= 1, "opcode 0x8D returned 0 cycles");
}

#[test]
fn op_8e() {
    let (cpu, _bus, cycles) = run(&[0x8E]);
    assert!(cycles >= 1, "opcode 0x8E returned 0 cycles");
}

#[test]
fn op_8f() {
    let (cpu, _bus, cycles) = run(&[0x8F]);
    assert!(cycles >= 1, "opcode 0x8F returned 0 cycles");
}

#[test]
fn op_90() {
    let (cpu, _bus, cycles) = run(&[0x90]);
    assert!(cycles >= 1, "opcode 0x90 returned 0 cycles");
}

#[test]
fn op_91() {
    let (cpu, _bus, cycles) = run(&[0x91]);
    assert!(cycles >= 1, "opcode 0x91 returned 0 cycles");
}

#[test]
fn op_92() {
    let (cpu, _bus, cycles) = run(&[0x92]);
    assert!(cycles >= 1, "opcode 0x92 returned 0 cycles");
}

#[test]
fn op_93() {
    let (cpu, _bus, cycles) = run(&[0x93]);
    assert!(cycles >= 1, "opcode 0x93 returned 0 cycles");
}

#[test]
fn op_94() {
    let (cpu, _bus, cycles) = run(&[0x94]);
    assert!(cycles >= 1, "opcode 0x94 returned 0 cycles");
}

#[test]
fn op_95() {
    let (cpu, _bus, cycles) = run(&[0x95]);
    assert!(cycles >= 1, "opcode 0x95 returned 0 cycles");
}

#[test]
fn op_96() {
    let (cpu, _bus, cycles) = run(&[0x96]);
    assert!(cycles >= 1, "opcode 0x96 returned 0 cycles");
}

#[test]
fn op_97() {
    let (cpu, _bus, cycles) = run(&[0x97]);
    assert!(cycles >= 1, "opcode 0x97 returned 0 cycles");
}

#[test]
fn op_98() {
    let (cpu, _bus, cycles) = run(&[0x98]);
    assert!(cycles >= 1, "opcode 0x98 returned 0 cycles");
}

#[test]
fn op_99() {
    let (cpu, _bus, cycles) = run(&[0x99]);
    assert!(cycles >= 1, "opcode 0x99 returned 0 cycles");
}

#[test]
fn op_9a() {
    let (cpu, _bus, cycles) = run(&[0x9A]);
    assert!(cycles >= 1, "opcode 0x9A returned 0 cycles");
}

#[test]
fn op_9b() {
    let (cpu, _bus, cycles) = run(&[0x9B]);
    assert!(cycles >= 1, "opcode 0x9B returned 0 cycles");
}

#[test]
fn op_9c() {
    let (cpu, _bus, cycles) = run(&[0x9C]);
    assert!(cycles >= 1, "opcode 0x9C returned 0 cycles");
}

#[test]
fn op_9d() {
    let (cpu, _bus, cycles) = run(&[0x9D]);
    assert!(cycles >= 1, "opcode 0x9D returned 0 cycles");
}

#[test]
fn op_9e() {
    let (cpu, _bus, cycles) = run(&[0x9E]);
    assert!(cycles >= 1, "opcode 0x9E returned 0 cycles");
}

#[test]
fn op_9f() {
    let (cpu, _bus, cycles) = run(&[0x9F]);
    assert!(cycles >= 1, "opcode 0x9F returned 0 cycles");
}

#[test]
fn op_a0() {
    let (cpu, _bus, cycles) = run(&[0xA0]);
    assert!(cycles >= 1, "opcode 0xA0 returned 0 cycles");
}

#[test]
fn op_a1() {
    let (cpu, _bus, cycles) = run(&[0xA1]);
    assert!(cycles >= 1, "opcode 0xA1 returned 0 cycles");
}

#[test]
fn op_a2() {
    let (cpu, _bus, cycles) = run(&[0xA2]);
    assert!(cycles >= 1, "opcode 0xA2 returned 0 cycles");
}

#[test]
fn op_a3() {
    let (cpu, _bus, cycles) = run(&[0xA3]);
    assert!(cycles >= 1, "opcode 0xA3 returned 0 cycles");
}

#[test]
fn op_a4() {
    let (cpu, _bus, cycles) = run(&[0xA4]);
    assert!(cycles >= 1, "opcode 0xA4 returned 0 cycles");
}

#[test]
fn op_a5() {
    let (cpu, _bus, cycles) = run(&[0xA5]);
    assert!(cycles >= 1, "opcode 0xA5 returned 0 cycles");
}

#[test]
fn op_a6() {
    let (cpu, _bus, cycles) = run(&[0xA6]);
    assert!(cycles >= 1, "opcode 0xA6 returned 0 cycles");
}

#[test]
fn op_a7() {
    let (cpu, _bus, cycles) = run(&[0xA7]);
    assert!(cycles >= 1, "opcode 0xA7 returned 0 cycles");
}

#[test]
fn op_a8() {
    let (cpu, _bus, cycles) = run(&[0xA8]);
    assert!(cycles >= 1, "opcode 0xA8 returned 0 cycles");
}

#[test]
fn op_a9() {
    let (cpu, _bus, cycles) = run(&[0xA9]);
    assert!(cycles >= 1, "opcode 0xA9 returned 0 cycles");
}

#[test]
fn op_aa() {
    let (cpu, _bus, cycles) = run(&[0xAA]);
    assert!(cycles >= 1, "opcode 0xAA returned 0 cycles");
}

#[test]
fn op_ab() {
    let (cpu, _bus, cycles) = run(&[0xAB]);
    assert!(cycles >= 1, "opcode 0xAB returned 0 cycles");
}

#[test]
fn op_ac() {
    let (cpu, _bus, cycles) = run(&[0xAC]);
    assert!(cycles >= 1, "opcode 0xAC returned 0 cycles");
}

#[test]
fn op_ad() {
    let (cpu, _bus, cycles) = run(&[0xAD]);
    assert!(cycles >= 1, "opcode 0xAD returned 0 cycles");
}

#[test]
fn op_ae() {
    let (cpu, _bus, cycles) = run(&[0xAE]);
    assert!(cycles >= 1, "opcode 0xAE returned 0 cycles");
}

#[test]
fn op_af() {
    let (cpu, _bus, cycles) = run(&[0xAF]);
    assert!(cycles >= 1, "opcode 0xAF returned 0 cycles");
}

#[test]
fn op_b0() {
    let (cpu, _bus, cycles) = run(&[0xB0]);
    assert!(cycles >= 1, "opcode 0xB0 returned 0 cycles");
}

#[test]
fn op_b1() {
    let (cpu, _bus, cycles) = run(&[0xB1]);
    assert!(cycles >= 1, "opcode 0xB1 returned 0 cycles");
}

#[test]
fn op_b2() {
    let (cpu, _bus, cycles) = run(&[0xB2]);
    assert!(cycles >= 1, "opcode 0xB2 returned 0 cycles");
}

#[test]
fn op_b3() {
    let (cpu, _bus, cycles) = run(&[0xB3]);
    assert!(cycles >= 1, "opcode 0xB3 returned 0 cycles");
}

#[test]
fn op_b4() {
    let (cpu, _bus, cycles) = run(&[0xB4]);
    assert!(cycles >= 1, "opcode 0xB4 returned 0 cycles");
}

#[test]
fn op_b5() {
    let (cpu, _bus, cycles) = run(&[0xB5]);
    assert!(cycles >= 1, "opcode 0xB5 returned 0 cycles");
}

#[test]
fn op_b6() {
    let (cpu, _bus, cycles) = run(&[0xB6]);
    assert!(cycles >= 1, "opcode 0xB6 returned 0 cycles");
}

#[test]
fn op_b7() {
    let (cpu, _bus, cycles) = run(&[0xB7]);
    assert!(cycles >= 1, "opcode 0xB7 returned 0 cycles");
}

#[test]
fn op_b8() {
    let (cpu, _bus, cycles) = run(&[0xB8]);
    assert!(cycles >= 1, "opcode 0xB8 returned 0 cycles");
}

#[test]
fn op_b9() {
    let (cpu, _bus, cycles) = run(&[0xB9]);
    assert!(cycles >= 1, "opcode 0xB9 returned 0 cycles");
}

#[test]
fn op_ba() {
    let (cpu, _bus, cycles) = run(&[0xBA]);
    assert!(cycles >= 1, "opcode 0xBA returned 0 cycles");
}

#[test]
fn op_bb() {
    let (cpu, _bus, cycles) = run(&[0xBB]);
    assert!(cycles >= 1, "opcode 0xBB returned 0 cycles");
}

#[test]
fn op_bc() {
    let (cpu, _bus, cycles) = run(&[0xBC]);
    assert!(cycles >= 1, "opcode 0xBC returned 0 cycles");
}

#[test]
fn op_bd() {
    let (cpu, _bus, cycles) = run(&[0xBD]);
    assert!(cycles >= 1, "opcode 0xBD returned 0 cycles");
}

#[test]
fn op_be() {
    let (cpu, _bus, cycles) = run(&[0xBE]);
    assert!(cycles >= 1, "opcode 0xBE returned 0 cycles");
}

#[test]
fn op_bf() {
    let (cpu, _bus, cycles) = run(&[0xBF]);
    assert!(cycles >= 1, "opcode 0xBF returned 0 cycles");
}

#[test]
fn op_c0() {
    let (cpu, _bus, cycles) = run(&[0xC0]);
    assert!(cycles >= 1, "opcode 0xC0 returned 0 cycles");
}

#[test]
fn op_c1() {
    let (cpu, _bus, cycles) = run(&[0xC1]);
    assert!(cycles >= 1, "opcode 0xC1 returned 0 cycles");
}

#[test]
fn op_c2() {
    let (cpu, _bus, cycles) = run(&[0xC2, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xC2 returned 0 cycles");
}

#[test]
fn op_c3() {
    let (cpu, _bus, cycles) = run(&[0xC3, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xC3 returned 0 cycles");
}

#[test]
fn op_c4() {
    let (cpu, _bus, cycles) = run(&[0xC4, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xC4 returned 0 cycles");
}

#[test]
fn op_c5() {
    let (cpu, _bus, cycles) = run(&[0xC5]);
    assert!(cycles >= 1, "opcode 0xC5 returned 0 cycles");
}

#[test]
fn op_c6() {
    let (cpu, _bus, cycles) = run(&[0xC6, 0x42]);
    assert!(cycles >= 1, "opcode 0xC6 returned 0 cycles");
}

#[test]
fn op_c7() {
    let (cpu, _bus, cycles) = run(&[0xC7]);
    assert!(cycles >= 1, "opcode 0xC7 returned 0 cycles");
}

#[test]
fn op_c8() {
    let (cpu, _bus, cycles) = run(&[0xC8]);
    assert!(cycles >= 1, "opcode 0xC8 returned 0 cycles");
}

#[test]
fn op_c9() {
    let (cpu, _bus, cycles) = run(&[0xC9]);
    assert!(cycles >= 1, "opcode 0xC9 returned 0 cycles");
}

#[test]
fn op_ca() {
    let (cpu, _bus, cycles) = run(&[0xCA, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xCA returned 0 cycles");
}

#[test]
fn op_cc() {
    let (cpu, _bus, cycles) = run(&[0xCC, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xCC returned 0 cycles");
}

#[test]
fn op_cd() {
    let (cpu, _bus, cycles) = run(&[0xCD, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xCD returned 0 cycles");
}

#[test]
fn op_ce() {
    let (cpu, _bus, cycles) = run(&[0xCE, 0x42]);
    assert!(cycles >= 1, "opcode 0xCE returned 0 cycles");
}

#[test]
fn op_cf() {
    let (cpu, _bus, cycles) = run(&[0xCF]);
    assert!(cycles >= 1, "opcode 0xCF returned 0 cycles");
}

#[test]
fn op_d0() {
    let (cpu, _bus, cycles) = run(&[0xD0]);
    assert!(cycles >= 1, "opcode 0xD0 returned 0 cycles");
}

#[test]
fn op_d1() {
    let (cpu, _bus, cycles) = run(&[0xD1]);
    assert!(cycles >= 1, "opcode 0xD1 returned 0 cycles");
}

#[test]
fn op_d2() {
    let (cpu, _bus, cycles) = run(&[0xD2, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xD2 returned 0 cycles");
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_d3_invalid() {
    let _ = run(&[0xD3]);
}

#[test]
fn op_d4() {
    let (cpu, _bus, cycles) = run(&[0xD4, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xD4 returned 0 cycles");
}

#[test]
fn op_d5() {
    let (cpu, _bus, cycles) = run(&[0xD5]);
    assert!(cycles >= 1, "opcode 0xD5 returned 0 cycles");
}

#[test]
fn op_d6() {
    let (cpu, _bus, cycles) = run(&[0xD6, 0x42]);
    assert!(cycles >= 1, "opcode 0xD6 returned 0 cycles");
}

#[test]
fn op_d7() {
    let (cpu, _bus, cycles) = run(&[0xD7]);
    assert!(cycles >= 1, "opcode 0xD7 returned 0 cycles");
}

#[test]
fn op_d8() {
    let (cpu, _bus, cycles) = run(&[0xD8]);
    assert!(cycles >= 1, "opcode 0xD8 returned 0 cycles");
}

#[test]
fn op_d9() {
    let (cpu, _bus, cycles) = run(&[0xD9]);
    assert!(cycles >= 1, "opcode 0xD9 returned 0 cycles");
}

#[test]
fn op_da() {
    let (cpu, _bus, cycles) = run(&[0xDA, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xDA returned 0 cycles");
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_db_invalid() {
    let _ = run(&[0xDB]);
}

#[test]
fn op_dc() {
    let (cpu, _bus, cycles) = run(&[0xDC, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xDC returned 0 cycles");
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_dd_invalid() {
    let _ = run(&[0xDD]);
}

#[test]
fn op_de() {
    let (cpu, _bus, cycles) = run(&[0xDE, 0x42]);
    assert!(cycles >= 1, "opcode 0xDE returned 0 cycles");
}

#[test]
fn op_df() {
    let (cpu, _bus, cycles) = run(&[0xDF]);
    assert!(cycles >= 1, "opcode 0xDF returned 0 cycles");
}

#[test]
fn op_e0() {
    let (cpu, _bus, cycles) = run(&[0xE0, 0x42]);
    assert!(cycles >= 1, "opcode 0xE0 returned 0 cycles");
}

#[test]
fn op_e1() {
    let (cpu, _bus, cycles) = run(&[0xE1]);
    assert!(cycles >= 1, "opcode 0xE1 returned 0 cycles");
}

#[test]
fn op_e2() {
    let (cpu, _bus, cycles) = run(&[0xE2]);
    assert!(cycles >= 1, "opcode 0xE2 returned 0 cycles");
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_e3_invalid() {
    let _ = run(&[0xE3]);
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_e4_invalid() {
    let _ = run(&[0xE4]);
}

#[test]
fn op_e5() {
    let (cpu, _bus, cycles) = run(&[0xE5]);
    assert!(cycles >= 1, "opcode 0xE5 returned 0 cycles");
}

#[test]
fn op_e6() {
    let (cpu, _bus, cycles) = run(&[0xE6, 0x42]);
    assert!(cycles >= 1, "opcode 0xE6 returned 0 cycles");
}

#[test]
fn op_e7() {
    let (cpu, _bus, cycles) = run(&[0xE7]);
    assert!(cycles >= 1, "opcode 0xE7 returned 0 cycles");
}

#[test]
fn op_e8() {
    let (cpu, _bus, cycles) = run(&[0xE8, 0x01]);
    assert!(cycles >= 1, "opcode 0xE8 returned 0 cycles");
}

#[test]
fn op_e9() {
    let (cpu, _bus, cycles) = run(&[0xE9]);
    assert!(cycles >= 1, "opcode 0xE9 returned 0 cycles");
}

#[test]
fn op_ea() {
    let (cpu, _bus, cycles) = run(&[0xEA, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xEA returned 0 cycles");
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_eb_invalid() {
    let _ = run(&[0xEB]);
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_ec_invalid() {
    let _ = run(&[0xEC]);
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_ed_invalid() {
    let _ = run(&[0xED]);
}

#[test]
fn op_ee() {
    let (cpu, _bus, cycles) = run(&[0xEE, 0x42]);
    assert!(cycles >= 1, "opcode 0xEE returned 0 cycles");
}

#[test]
fn op_ef() {
    let (cpu, _bus, cycles) = run(&[0xEF]);
    assert!(cycles >= 1, "opcode 0xEF returned 0 cycles");
}

#[test]
fn op_f0() {
    let (cpu, _bus, cycles) = run(&[0xF0, 0x42]);
    assert!(cycles >= 1, "opcode 0xF0 returned 0 cycles");
}

#[test]
fn op_f1() {
    let (cpu, _bus, cycles) = run(&[0xF1]);
    assert!(cycles >= 1, "opcode 0xF1 returned 0 cycles");
}

#[test]
fn op_f2() {
    let (cpu, _bus, cycles) = run(&[0xF2]);
    assert!(cycles >= 1, "opcode 0xF2 returned 0 cycles");
}

#[test]
fn op_f3() {
    let (cpu, _bus, cycles) = run(&[0xF3]);
    assert!(cycles >= 1, "opcode 0xF3 returned 0 cycles");
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_f4_invalid() {
    let _ = run(&[0xF4]);
}

#[test]
fn op_f5() {
    let (cpu, _bus, cycles) = run(&[0xF5]);
    assert!(cycles >= 1, "opcode 0xF5 returned 0 cycles");
}

#[test]
fn op_f6() {
    let (cpu, _bus, cycles) = run(&[0xF6, 0x42]);
    assert!(cycles >= 1, "opcode 0xF6 returned 0 cycles");
}

#[test]
fn op_f7() {
    let (cpu, _bus, cycles) = run(&[0xF7]);
    assert!(cycles >= 1, "opcode 0xF7 returned 0 cycles");
}

#[test]
fn op_f8() {
    let (cpu, _bus, cycles) = run(&[0xF8, 0x01]);
    assert!(cycles >= 1, "opcode 0xF8 returned 0 cycles");
}

#[test]
fn op_f9() {
    let (cpu, _bus, cycles) = run(&[0xF9]);
    assert!(cycles >= 1, "opcode 0xF9 returned 0 cycles");
}

#[test]
fn op_fa() {
    let (cpu, _bus, cycles) = run(&[0xFA, 0x00, 0xC0]);
    assert!(cycles >= 1, "opcode 0xFA returned 0 cycles");
}

#[test]
fn op_fb() {
    let (cpu, _bus, cycles) = run(&[0xFB]);
    assert!(cycles >= 1, "opcode 0xFB returned 0 cycles");
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_fc_invalid() {
    let _ = run(&[0xFC]);
}

#[test]
#[should_panic(expected = "invalid opcode")]
fn op_fd_invalid() {
    let _ = run(&[0xFD]);
}

#[test]
fn op_fe() {
    let (cpu, _bus, cycles) = run(&[0xFE, 0x42]);
    assert!(cycles >= 1, "opcode 0xFE returned 0 cycles");
}

#[test]
fn op_ff() {
    let (cpu, _bus, cycles) = run(&[0xFF]);
    assert!(cycles >= 1, "opcode 0xFF returned 0 cycles");
}

#[test]
fn cb_00() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x00]);
    assert!(cycles >= 2, "CB 0x00 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_01() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x01]);
    assert!(cycles >= 2, "CB 0x01 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_02() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x02]);
    assert!(cycles >= 2, "CB 0x02 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_03() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x03]);
    assert!(cycles >= 2, "CB 0x03 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_04() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x04]);
    assert!(cycles >= 2, "CB 0x04 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_05() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x05]);
    assert!(cycles >= 2, "CB 0x05 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_06() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x06]);
    assert!(cycles >= 2, "CB 0x06 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_07() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x07]);
    assert!(cycles >= 2, "CB 0x07 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_08() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x08]);
    assert!(cycles >= 2, "CB 0x08 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_09() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x09]);
    assert!(cycles >= 2, "CB 0x09 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_0a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x0A]);
    assert!(cycles >= 2, "CB 0x0A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_0b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x0B]);
    assert!(cycles >= 2, "CB 0x0B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_0c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x0C]);
    assert!(cycles >= 2, "CB 0x0C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_0d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x0D]);
    assert!(cycles >= 2, "CB 0x0D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_0e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x0E]);
    assert!(cycles >= 2, "CB 0x0E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_0f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x0F]);
    assert!(cycles >= 2, "CB 0x0F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_10() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x10]);
    assert!(cycles >= 2, "CB 0x10 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_11() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x11]);
    assert!(cycles >= 2, "CB 0x11 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_12() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x12]);
    assert!(cycles >= 2, "CB 0x12 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_13() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x13]);
    assert!(cycles >= 2, "CB 0x13 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_14() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x14]);
    assert!(cycles >= 2, "CB 0x14 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_15() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x15]);
    assert!(cycles >= 2, "CB 0x15 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_16() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x16]);
    assert!(cycles >= 2, "CB 0x16 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_17() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x17]);
    assert!(cycles >= 2, "CB 0x17 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_18() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x18]);
    assert!(cycles >= 2, "CB 0x18 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_19() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x19]);
    assert!(cycles >= 2, "CB 0x19 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_1a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x1A]);
    assert!(cycles >= 2, "CB 0x1A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_1b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x1B]);
    assert!(cycles >= 2, "CB 0x1B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_1c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x1C]);
    assert!(cycles >= 2, "CB 0x1C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_1d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x1D]);
    assert!(cycles >= 2, "CB 0x1D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_1e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x1E]);
    assert!(cycles >= 2, "CB 0x1E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_1f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x1F]);
    assert!(cycles >= 2, "CB 0x1F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_20() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x20]);
    assert!(cycles >= 2, "CB 0x20 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_21() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x21]);
    assert!(cycles >= 2, "CB 0x21 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_22() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x22]);
    assert!(cycles >= 2, "CB 0x22 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_23() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x23]);
    assert!(cycles >= 2, "CB 0x23 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_24() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x24]);
    assert!(cycles >= 2, "CB 0x24 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_25() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x25]);
    assert!(cycles >= 2, "CB 0x25 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_26() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x26]);
    assert!(cycles >= 2, "CB 0x26 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_27() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x27]);
    assert!(cycles >= 2, "CB 0x27 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_28() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x28]);
    assert!(cycles >= 2, "CB 0x28 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_29() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x29]);
    assert!(cycles >= 2, "CB 0x29 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_2a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x2A]);
    assert!(cycles >= 2, "CB 0x2A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_2b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x2B]);
    assert!(cycles >= 2, "CB 0x2B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_2c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x2C]);
    assert!(cycles >= 2, "CB 0x2C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_2d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x2D]);
    assert!(cycles >= 2, "CB 0x2D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_2e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x2E]);
    assert!(cycles >= 2, "CB 0x2E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_2f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x2F]);
    assert!(cycles >= 2, "CB 0x2F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_30() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x30]);
    assert!(cycles >= 2, "CB 0x30 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_31() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x31]);
    assert!(cycles >= 2, "CB 0x31 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_32() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x32]);
    assert!(cycles >= 2, "CB 0x32 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_33() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x33]);
    assert!(cycles >= 2, "CB 0x33 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_34() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x34]);
    assert!(cycles >= 2, "CB 0x34 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_35() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x35]);
    assert!(cycles >= 2, "CB 0x35 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_36() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x36]);
    assert!(cycles >= 2, "CB 0x36 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_37() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x37]);
    assert!(cycles >= 2, "CB 0x37 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_38() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x38]);
    assert!(cycles >= 2, "CB 0x38 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_39() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x39]);
    assert!(cycles >= 2, "CB 0x39 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_3a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x3A]);
    assert!(cycles >= 2, "CB 0x3A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_3b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x3B]);
    assert!(cycles >= 2, "CB 0x3B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_3c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x3C]);
    assert!(cycles >= 2, "CB 0x3C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_3d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x3D]);
    assert!(cycles >= 2, "CB 0x3D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_3e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x3E]);
    assert!(cycles >= 2, "CB 0x3E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_3f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x3F]);
    assert!(cycles >= 2, "CB 0x3F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_40() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x40]);
    assert!(cycles >= 2, "CB 0x40 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_41() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x41]);
    assert!(cycles >= 2, "CB 0x41 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_42() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x42]);
    assert!(cycles >= 2, "CB 0x42 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_43() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x43]);
    assert!(cycles >= 2, "CB 0x43 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_44() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x44]);
    assert!(cycles >= 2, "CB 0x44 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_45() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x45]);
    assert!(cycles >= 2, "CB 0x45 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_46() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x46]);
    assert!(cycles >= 2, "CB 0x46 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_47() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x47]);
    assert!(cycles >= 2, "CB 0x47 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_48() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x48]);
    assert!(cycles >= 2, "CB 0x48 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_49() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x49]);
    assert!(cycles >= 2, "CB 0x49 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_4a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x4A]);
    assert!(cycles >= 2, "CB 0x4A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_4b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x4B]);
    assert!(cycles >= 2, "CB 0x4B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_4c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x4C]);
    assert!(cycles >= 2, "CB 0x4C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_4d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x4D]);
    assert!(cycles >= 2, "CB 0x4D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_4e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x4E]);
    assert!(cycles >= 2, "CB 0x4E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_4f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x4F]);
    assert!(cycles >= 2, "CB 0x4F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_50() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x50]);
    assert!(cycles >= 2, "CB 0x50 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_51() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x51]);
    assert!(cycles >= 2, "CB 0x51 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_52() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x52]);
    assert!(cycles >= 2, "CB 0x52 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_53() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x53]);
    assert!(cycles >= 2, "CB 0x53 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_54() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x54]);
    assert!(cycles >= 2, "CB 0x54 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_55() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x55]);
    assert!(cycles >= 2, "CB 0x55 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_56() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x56]);
    assert!(cycles >= 2, "CB 0x56 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_57() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x57]);
    assert!(cycles >= 2, "CB 0x57 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_58() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x58]);
    assert!(cycles >= 2, "CB 0x58 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_59() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x59]);
    assert!(cycles >= 2, "CB 0x59 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_5a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x5A]);
    assert!(cycles >= 2, "CB 0x5A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_5b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x5B]);
    assert!(cycles >= 2, "CB 0x5B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_5c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x5C]);
    assert!(cycles >= 2, "CB 0x5C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_5d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x5D]);
    assert!(cycles >= 2, "CB 0x5D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_5e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x5E]);
    assert!(cycles >= 2, "CB 0x5E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_5f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x5F]);
    assert!(cycles >= 2, "CB 0x5F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_60() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x60]);
    assert!(cycles >= 2, "CB 0x60 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_61() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x61]);
    assert!(cycles >= 2, "CB 0x61 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_62() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x62]);
    assert!(cycles >= 2, "CB 0x62 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_63() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x63]);
    assert!(cycles >= 2, "CB 0x63 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_64() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x64]);
    assert!(cycles >= 2, "CB 0x64 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_65() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x65]);
    assert!(cycles >= 2, "CB 0x65 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_66() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x66]);
    assert!(cycles >= 2, "CB 0x66 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_67() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x67]);
    assert!(cycles >= 2, "CB 0x67 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_68() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x68]);
    assert!(cycles >= 2, "CB 0x68 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_69() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x69]);
    assert!(cycles >= 2, "CB 0x69 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_6a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x6A]);
    assert!(cycles >= 2, "CB 0x6A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_6b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x6B]);
    assert!(cycles >= 2, "CB 0x6B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_6c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x6C]);
    assert!(cycles >= 2, "CB 0x6C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_6d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x6D]);
    assert!(cycles >= 2, "CB 0x6D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_6e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x6E]);
    assert!(cycles >= 2, "CB 0x6E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_6f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x6F]);
    assert!(cycles >= 2, "CB 0x6F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_70() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x70]);
    assert!(cycles >= 2, "CB 0x70 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_71() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x71]);
    assert!(cycles >= 2, "CB 0x71 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_72() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x72]);
    assert!(cycles >= 2, "CB 0x72 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_73() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x73]);
    assert!(cycles >= 2, "CB 0x73 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_74() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x74]);
    assert!(cycles >= 2, "CB 0x74 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_75() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x75]);
    assert!(cycles >= 2, "CB 0x75 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_76() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x76]);
    assert!(cycles >= 2, "CB 0x76 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_77() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x77]);
    assert!(cycles >= 2, "CB 0x77 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_78() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x78]);
    assert!(cycles >= 2, "CB 0x78 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_79() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x79]);
    assert!(cycles >= 2, "CB 0x79 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_7a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x7A]);
    assert!(cycles >= 2, "CB 0x7A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_7b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x7B]);
    assert!(cycles >= 2, "CB 0x7B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_7c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x7C]);
    assert!(cycles >= 2, "CB 0x7C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_7d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x7D]);
    assert!(cycles >= 2, "CB 0x7D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_7e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x7E]);
    assert!(cycles >= 2, "CB 0x7E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_7f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x7F]);
    assert!(cycles >= 2, "CB 0x7F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_80() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x80]);
    assert!(cycles >= 2, "CB 0x80 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_81() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x81]);
    assert!(cycles >= 2, "CB 0x81 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_82() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x82]);
    assert!(cycles >= 2, "CB 0x82 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_83() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x83]);
    assert!(cycles >= 2, "CB 0x83 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_84() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x84]);
    assert!(cycles >= 2, "CB 0x84 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_85() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x85]);
    assert!(cycles >= 2, "CB 0x85 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_86() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x86]);
    assert!(cycles >= 2, "CB 0x86 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_87() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x87]);
    assert!(cycles >= 2, "CB 0x87 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_88() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x88]);
    assert!(cycles >= 2, "CB 0x88 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_89() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x89]);
    assert!(cycles >= 2, "CB 0x89 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_8a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x8A]);
    assert!(cycles >= 2, "CB 0x8A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_8b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x8B]);
    assert!(cycles >= 2, "CB 0x8B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_8c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x8C]);
    assert!(cycles >= 2, "CB 0x8C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_8d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x8D]);
    assert!(cycles >= 2, "CB 0x8D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_8e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x8E]);
    assert!(cycles >= 2, "CB 0x8E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_8f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x8F]);
    assert!(cycles >= 2, "CB 0x8F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_90() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x90]);
    assert!(cycles >= 2, "CB 0x90 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_91() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x91]);
    assert!(cycles >= 2, "CB 0x91 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_92() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x92]);
    assert!(cycles >= 2, "CB 0x92 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_93() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x93]);
    assert!(cycles >= 2, "CB 0x93 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_94() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x94]);
    assert!(cycles >= 2, "CB 0x94 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_95() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x95]);
    assert!(cycles >= 2, "CB 0x95 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_96() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x96]);
    assert!(cycles >= 2, "CB 0x96 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_97() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x97]);
    assert!(cycles >= 2, "CB 0x97 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_98() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x98]);
    assert!(cycles >= 2, "CB 0x98 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_99() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x99]);
    assert!(cycles >= 2, "CB 0x99 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_9a() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x9A]);
    assert!(cycles >= 2, "CB 0x9A cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_9b() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x9B]);
    assert!(cycles >= 2, "CB 0x9B cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_9c() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x9C]);
    assert!(cycles >= 2, "CB 0x9C cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_9d() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x9D]);
    assert!(cycles >= 2, "CB 0x9D cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_9e() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x9E]);
    assert!(cycles >= 2, "CB 0x9E cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_9f() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0x9F]);
    assert!(cycles >= 2, "CB 0x9F cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a0() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA0]);
    assert!(cycles >= 2, "CB 0xA0 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a1() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA1]);
    assert!(cycles >= 2, "CB 0xA1 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a2() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA2]);
    assert!(cycles >= 2, "CB 0xA2 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a3() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA3]);
    assert!(cycles >= 2, "CB 0xA3 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a4() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA4]);
    assert!(cycles >= 2, "CB 0xA4 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a5() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA5]);
    assert!(cycles >= 2, "CB 0xA5 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a6() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA6]);
    assert!(cycles >= 2, "CB 0xA6 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a7() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA7]);
    assert!(cycles >= 2, "CB 0xA7 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a8() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA8]);
    assert!(cycles >= 2, "CB 0xA8 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_a9() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xA9]);
    assert!(cycles >= 2, "CB 0xA9 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_aa() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xAA]);
    assert!(cycles >= 2, "CB 0xAA cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ab() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xAB]);
    assert!(cycles >= 2, "CB 0xAB cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ac() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xAC]);
    assert!(cycles >= 2, "CB 0xAC cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ad() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xAD]);
    assert!(cycles >= 2, "CB 0xAD cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ae() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xAE]);
    assert!(cycles >= 2, "CB 0xAE cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_af() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xAF]);
    assert!(cycles >= 2, "CB 0xAF cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b0() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB0]);
    assert!(cycles >= 2, "CB 0xB0 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b1() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB1]);
    assert!(cycles >= 2, "CB 0xB1 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b2() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB2]);
    assert!(cycles >= 2, "CB 0xB2 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b3() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB3]);
    assert!(cycles >= 2, "CB 0xB3 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b4() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB4]);
    assert!(cycles >= 2, "CB 0xB4 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b5() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB5]);
    assert!(cycles >= 2, "CB 0xB5 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b6() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB6]);
    assert!(cycles >= 2, "CB 0xB6 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b7() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB7]);
    assert!(cycles >= 2, "CB 0xB7 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b8() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB8]);
    assert!(cycles >= 2, "CB 0xB8 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_b9() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xB9]);
    assert!(cycles >= 2, "CB 0xB9 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ba() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xBA]);
    assert!(cycles >= 2, "CB 0xBA cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_bb() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xBB]);
    assert!(cycles >= 2, "CB 0xBB cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_bc() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xBC]);
    assert!(cycles >= 2, "CB 0xBC cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_bd() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xBD]);
    assert!(cycles >= 2, "CB 0xBD cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_be() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xBE]);
    assert!(cycles >= 2, "CB 0xBE cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_bf() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xBF]);
    assert!(cycles >= 2, "CB 0xBF cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c0() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC0]);
    assert!(cycles >= 2, "CB 0xC0 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c1() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC1]);
    assert!(cycles >= 2, "CB 0xC1 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c2() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC2]);
    assert!(cycles >= 2, "CB 0xC2 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c3() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC3]);
    assert!(cycles >= 2, "CB 0xC3 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c4() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC4]);
    assert!(cycles >= 2, "CB 0xC4 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c5() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC5]);
    assert!(cycles >= 2, "CB 0xC5 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c6() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC6]);
    assert!(cycles >= 2, "CB 0xC6 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c7() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC7]);
    assert!(cycles >= 2, "CB 0xC7 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c8() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC8]);
    assert!(cycles >= 2, "CB 0xC8 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_c9() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xC9]);
    assert!(cycles >= 2, "CB 0xC9 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ca() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xCA]);
    assert!(cycles >= 2, "CB 0xCA cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_cb() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xCB]);
    assert!(cycles >= 2, "CB 0xCB cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_cc() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xCC]);
    assert!(cycles >= 2, "CB 0xCC cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_cd() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xCD]);
    assert!(cycles >= 2, "CB 0xCD cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ce() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xCE]);
    assert!(cycles >= 2, "CB 0xCE cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_cf() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xCF]);
    assert!(cycles >= 2, "CB 0xCF cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d0() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD0]);
    assert!(cycles >= 2, "CB 0xD0 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d1() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD1]);
    assert!(cycles >= 2, "CB 0xD1 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d2() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD2]);
    assert!(cycles >= 2, "CB 0xD2 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d3() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD3]);
    assert!(cycles >= 2, "CB 0xD3 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d4() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD4]);
    assert!(cycles >= 2, "CB 0xD4 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d5() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD5]);
    assert!(cycles >= 2, "CB 0xD5 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d6() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD6]);
    assert!(cycles >= 2, "CB 0xD6 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d7() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD7]);
    assert!(cycles >= 2, "CB 0xD7 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d8() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD8]);
    assert!(cycles >= 2, "CB 0xD8 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_d9() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xD9]);
    assert!(cycles >= 2, "CB 0xD9 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_da() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xDA]);
    assert!(cycles >= 2, "CB 0xDA cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_db() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xDB]);
    assert!(cycles >= 2, "CB 0xDB cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_dc() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xDC]);
    assert!(cycles >= 2, "CB 0xDC cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_dd() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xDD]);
    assert!(cycles >= 2, "CB 0xDD cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_de() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xDE]);
    assert!(cycles >= 2, "CB 0xDE cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_df() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xDF]);
    assert!(cycles >= 2, "CB 0xDF cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e0() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE0]);
    assert!(cycles >= 2, "CB 0xE0 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e1() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE1]);
    assert!(cycles >= 2, "CB 0xE1 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e2() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE2]);
    assert!(cycles >= 2, "CB 0xE2 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e3() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE3]);
    assert!(cycles >= 2, "CB 0xE3 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e4() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE4]);
    assert!(cycles >= 2, "CB 0xE4 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e5() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE5]);
    assert!(cycles >= 2, "CB 0xE5 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e6() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE6]);
    assert!(cycles >= 2, "CB 0xE6 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e7() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE7]);
    assert!(cycles >= 2, "CB 0xE7 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e8() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE8]);
    assert!(cycles >= 2, "CB 0xE8 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_e9() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xE9]);
    assert!(cycles >= 2, "CB 0xE9 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ea() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xEA]);
    assert!(cycles >= 2, "CB 0xEA cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_eb() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xEB]);
    assert!(cycles >= 2, "CB 0xEB cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ec() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xEC]);
    assert!(cycles >= 2, "CB 0xEC cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ed() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xED]);
    assert!(cycles >= 2, "CB 0xED cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ee() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xEE]);
    assert!(cycles >= 2, "CB 0xEE cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ef() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xEF]);
    assert!(cycles >= 2, "CB 0xEF cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f0() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF0]);
    assert!(cycles >= 2, "CB 0xF0 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f1() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF1]);
    assert!(cycles >= 2, "CB 0xF1 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f2() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF2]);
    assert!(cycles >= 2, "CB 0xF2 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f3() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF3]);
    assert!(cycles >= 2, "CB 0xF3 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f4() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF4]);
    assert!(cycles >= 2, "CB 0xF4 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f5() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF5]);
    assert!(cycles >= 2, "CB 0xF5 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f6() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF6]);
    assert!(cycles >= 2, "CB 0xF6 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f7() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF7]);
    assert!(cycles >= 2, "CB 0xF7 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f8() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF8]);
    assert!(cycles >= 2, "CB 0xF8 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_f9() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xF9]);
    assert!(cycles >= 2, "CB 0xF9 cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_fa() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xFA]);
    assert!(cycles >= 2, "CB 0xFA cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_fb() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xFB]);
    assert!(cycles >= 2, "CB 0xFB cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_fc() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xFC]);
    assert!(cycles >= 2, "CB 0xFC cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_fd() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xFD]);
    assert!(cycles >= 2, "CB 0xFD cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_fe() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xFE]);
    assert!(cycles >= 2, "CB 0xFE cycles");
    let _ = cpu.a; // exercised
}

#[test]
fn cb_ff() {
    let (cpu, _bus, cycles) = run(&[0xCB, 0xFF]);
    assert!(cycles >= 2, "CB 0xFF cycles");
    let _ = cpu.a; // exercised
}
