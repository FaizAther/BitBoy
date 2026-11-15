//! SM83 GDB architecture — exposed as i386 so stock Ubuntu GDB accepts XML.
//!
//! Wire layout (32-bit LE each, SM83 values zero-extended):
//! `eax=af ecx=bc edx=de ebx=hl esp=sp ebp=0 esi=0 edi=0 eip=pc eflags=ime`

use gdbstub::arch::{Arch, BreakpointKind, RegId, Registers};

pub enum Sm83Arch {}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Sm83Regs {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
}

impl Sm83Regs {
    /// 16 × u32 LE (eax..gs) — matches org.gnu.gdb.i386.core.
    pub const WIRE_LEN: usize = 64;

    fn words(&self) -> [u32; 16] {
        [
            u32::from(self.af),  // eax
            u32::from(self.bc),  // ecx
            u32::from(self.de),  // edx
            u32::from(self.hl),  // ebx
            u32::from(self.sp),  // esp
            0,                   // ebp
            0,                   // esi
            0,                   // edi
            u32::from(self.pc),  // eip
            u32::from(self.ime), // eflags
            0,                   // cs
            0,                   // ss
            0,                   // ds
            0,                   // es
            0,                   // fs
            0,                   // gs
        ]
    }

    pub fn gdb_serialize(&self, buf: &mut [u8]) -> usize {
        let mut n = 0;
        for w in self.words() {
            if n + 4 > buf.len() {
                break;
            }
            buf[n..n + 4].copy_from_slice(&w.to_le_bytes());
            n += 4;
        }
        n
    }

    pub fn gdb_deserialize(&mut self, data: &[u8]) -> bool {
        if data.len() < Self::WIRE_LEN {
            return false;
        }
        let mut words = [0u32; 16];
        for (i, slot) in words.iter_mut().enumerate() {
            let o = i * 4;
            *slot = u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        }
        self.af = words[0] as u16;
        self.bc = words[1] as u16;
        self.de = words[2] as u16;
        self.hl = words[3] as u16;
        self.sp = words[4] as u16;
        self.pc = words[8] as u16;
        self.ime = words[9] != 0;
        true
    }
}

impl Registers for Sm83Regs {
    type ProgramCounter = u32;

    fn pc(&self) -> Self::ProgramCounter {
        u32::from(self.pc)
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        let mut buf = [0u8; Self::WIRE_LEN];
        let n = Sm83Regs::gdb_serialize(self, &mut buf);
        for &b in &buf[..n] {
            write_byte(Some(b));
        }
    }

    fn gdb_deserialize(&mut self, bytes: &[u8]) -> Result<(), ()> {
        if Sm83Regs::gdb_deserialize(self, bytes) {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Sm83RegId {
    EaxAf,
    EcxBc,
    EdxDe,
    EbxHl,
    EspSp,
    Ebp,
    Esi,
    Edi,
    EipPc,
    EflagsIme,
    Seg(u8),
}

impl RegId for Sm83RegId {
    fn from_raw_id(id: usize) -> Option<(Self, Option<core::num::NonZeroUsize>)> {
        let reg = match id {
            0 => Sm83RegId::EaxAf,
            1 => Sm83RegId::EcxBc,
            2 => Sm83RegId::EdxDe,
            3 => Sm83RegId::EbxHl,
            4 => Sm83RegId::EspSp,
            5 => Sm83RegId::Ebp,
            6 => Sm83RegId::Esi,
            7 => Sm83RegId::Edi,
            8 => Sm83RegId::EipPc,
            9 => Sm83RegId::EflagsIme,
            10..=15 => Sm83RegId::Seg((id - 10) as u8),
            _ => return None,
        };
        Some((reg, core::num::NonZeroUsize::new(4)))
    }
}

#[derive(Debug)]
pub enum Sm83BreakpointKind {
    Soft1,
}

impl BreakpointKind for Sm83BreakpointKind {
    fn from_usize(_kind: usize) -> Option<Self> {
        Some(Sm83BreakpointKind::Soft1)
    }
}

impl Arch for Sm83Arch {
    type Usize = u32;
    type Registers = Sm83Regs;
    type RegId = Sm83RegId;
    type BreakpointKind = Sm83BreakpointKind;

    fn target_description_xml() -> Option<&'static str> {
        Some(
            r#"<?xml version="1.0"?>
<!DOCTYPE target SYSTEM "gdb-target.dtd">
<target version="1.0">
  <architecture>i386</architecture>
  <feature name="org.gnu.gdb.i386.core">
    <reg name="eax" bitsize="32" type="int32" regnum="0"/>
    <reg name="ecx" bitsize="32" type="int32" regnum="1"/>
    <reg name="edx" bitsize="32" type="int32" regnum="2"/>
    <reg name="ebx" bitsize="32" type="int32" regnum="3"/>
    <reg name="esp" bitsize="32" type="data_ptr" regnum="4"/>
    <reg name="ebp" bitsize="32" type="data_ptr" regnum="5"/>
    <reg name="esi" bitsize="32" type="int32" regnum="6"/>
    <reg name="edi" bitsize="32" type="int32" regnum="7"/>
    <reg name="eip" bitsize="32" type="code_ptr" regnum="8"/>
    <reg name="eflags" bitsize="32" type="int32" regnum="9"/>
    <reg name="cs" bitsize="32" type="int32" regnum="10"/>
    <reg name="ss" bitsize="32" type="int32" regnum="11"/>
    <reg name="ds" bitsize="32" type="int32" regnum="12"/>
    <reg name="es" bitsize="32" type="int32" regnum="13"/>
    <reg name="fs" bitsize="32" type="int32" regnum="14"/>
    <reg name="gs" bitsize="32" type="int32" regnum="15"/>
  </feature>
</target>"#,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sm83_regs_gdb_round_trip_i386_layout() {
        let regs = Sm83Regs {
            af: 0x01B0,
            bc: 0x0013,
            de: 0x00D8,
            hl: 0x014D,
            sp: 0xFFFE,
            pc: 0x0100,
            ime: true,
        };
        let mut buf = [0u8; 64];
        assert_eq!(regs.gdb_serialize(&mut buf), Sm83Regs::WIRE_LEN);
        let mut out = Sm83Regs::default();
        assert!(out.gdb_deserialize(&buf[..Sm83Regs::WIRE_LEN]));
        assert_eq!(out.af, regs.af);
        assert_eq!(out.pc, regs.pc);
        assert_eq!(out.ime, true);
        assert_eq!(Registers::pc(&out), 0x0100);
    }

    #[test]
    fn sm83_reg_id_maps_eip_pc() {
        let (id, sz) = Sm83RegId::from_raw_id(8).unwrap();
        assert!(matches!(id, Sm83RegId::EipPc));
        assert_eq!(sz, core::num::NonZeroUsize::new(4));
    }
}
