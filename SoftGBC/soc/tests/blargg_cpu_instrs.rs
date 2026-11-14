//! Blargg `cpu_instrs` individuals — thin tests over the generic [`soc::SoftGbc`] machine.

use soc::{RunStatus, SoftGbc};
use std::path::PathBuf;

fn rom_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/roms/blargg/cpu_instrs")
        .join(name)
}

fn run_blargg(name: &str) {
    let path = rom_path(name);
    assert!(
        path.is_file(),
        "missing test ROM {} (see SoftGBC/tests/roms)",
        path.display()
    );
    let mut gb = SoftGbc::from_rom_file(&path).expect("load ROM");
    let status = gb.run(50_000_000);
    let out = gb.serial_output();
    assert_eq!(
        status,
        RunStatus::SerialPassed,
        "{name}: status={status:?} serial={out:?}"
    );
    assert!(out.contains("Passed"), "{name}: serial={out:?}");
}

#[test]
fn blargg_01_special() {
    run_blargg("01-special.gb");
}

#[test]
fn blargg_02_interrupts() {
    run_blargg("02-interrupts.gb");
}

#[test]
fn blargg_03_op_sp_hl() {
    run_blargg("03-op sp,hl.gb");
}

#[test]
fn blargg_04_op_r_imm() {
    run_blargg("04-op r,imm.gb");
}

#[test]
fn blargg_05_op_rp() {
    run_blargg("05-op rp.gb");
}

#[test]
fn blargg_06_ld_r_r() {
    run_blargg("06-ld r,r.gb");
}

#[test]
fn blargg_07_jr_jp_call_ret_rst() {
    run_blargg("07-jr,jp,call,ret,rst.gb");
}

#[test]
fn blargg_08_misc_instrs() {
    run_blargg("08-misc instrs.gb");
}

#[test]
fn blargg_09_op_r_r() {
    run_blargg("09-op r,r.gb");
}

#[test]
fn blargg_10_bit_ops() {
    run_blargg("10-bit ops.gb");
}

#[test]
fn blargg_11_op_a_hl() {
    run_blargg("11-op a,(hl).gb");
}
