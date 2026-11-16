//! gdbstub [`Target`] over a shared SoftGBC machine.

use super::arch::{Sm83Arch, Sm83BreakpointKind, Sm83RegId, Sm83Regs};
use gdbstub::common::Signal;
use gdbstub::target::ext::base::single_register_access::{
    SingleRegisterAccess, SingleRegisterAccessOps,
};
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadResumeOps, SingleThreadSingleStep,
    SingleThreadSingleStepOps,
};
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::breakpoints::{
    Breakpoints, BreakpointsOps, SwBreakpoint, SwBreakpointOps,
};
use gdbstub::target::{Target, TargetError, TargetResult};
use softgbc_core::Bus;
use soc::SoftGbc;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecMode {
    Step,
    Continue,
}

/// Shared SoftGBC core used by play + GDB.
pub struct GdbMachine {
    pub gb: SoftGbc,
    pub exec_mode: ExecMode,
    pub breakpoints: HashSet<u16>,
    /// When true, the play frame thread must not advance the CPU.
    pub gdb_halted: bool,
}

impl GdbMachine {
    pub fn new(gb: SoftGbc) -> Self {
        Self {
            gb,
            exec_mode: ExecMode::Continue,
            breakpoints: HashSet::new(),
            gdb_halted: false,
        }
    }

    pub fn from_regs(cpu: &sm83_iss::Cpu) -> Sm83Regs {
        Sm83Regs {
            af: cpu.af(),
            bc: cpu.bc(),
            de: cpu.de(),
            hl: cpu.hl(),
            sp: cpu.sp,
            pc: cpu.pc,
            ime: cpu.ime,
        }
    }

    pub fn apply_regs(cpu: &mut sm83_iss::Cpu, regs: &Sm83Regs) {
        cpu.set_af(regs.af);
        cpu.set_bc(regs.bc);
        cpu.set_de(regs.de);
        cpu.set_hl(regs.hl);
        cpu.sp = regs.sp;
        cpu.pc = regs.pc;
        cpu.ime = regs.ime;
    }

    /// One CPU instruction + peripheral tick. Returns true if a breakpoint hit *after* the step.
    pub fn step_insn(&mut self) -> bool {
        let _ = self.gb.step();
        self.breakpoints.contains(&self.gb.cpu.pc)
    }

    /// One LCD frame of machine time. The framebuffer is the visible picture
    /// at the VBlank edge (see [`crate::controller::run_lcd_frame`]).
    pub fn advance_frame(&mut self) {
        let GdbMachine {
            gb,
            breakpoints,
            gdb_halted,
            ..
        } = self;
        crate::controller::run_lcd_frame(gb, |gb| {
            if breakpoints.contains(&gb.cpu.pc) {
                *gdb_halted = true;
                true
            } else {
                false
            }
        });
    }
}

pub struct SoftGbcGdbTarget {
    pub machine: Arc<Mutex<GdbMachine>>,
}

impl SoftGbcGdbTarget {
    pub fn new(machine: Arc<Mutex<GdbMachine>>) -> Self {
        Self { machine }
    }
}

impl Target for SoftGbcGdbTarget {
    type Arch = Sm83Arch;
    type Error = &'static str;

    #[inline(always)]
    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for SoftGbcGdbTarget {
    fn read_registers(&mut self, regs: &mut Sm83Regs) -> TargetResult<(), Self> {
        let g = self.machine.lock().map_err(|_| TargetError::Fatal("lock"))?;
        *regs = GdbMachine::from_regs(&g.gb.cpu);
        Ok(())
    }

    fn write_registers(&mut self, regs: &Sm83Regs) -> TargetResult<(), Self> {
        let mut g = self.machine.lock().map_err(|_| TargetError::Fatal("lock"))?;
        GdbMachine::apply_regs(&mut g.gb.cpu, regs);
        Ok(())
    }

    #[inline(always)]
    fn support_single_register_access(
        &mut self,
    ) -> Option<SingleRegisterAccessOps<'_, (), Self>> {
        Some(self)
    }

    fn read_addrs(&mut self, start_addr: u32, data: &mut [u8]) -> TargetResult<usize, Self> {
        let g = self.machine.lock().map_err(|_| TargetError::Fatal("lock"))?;
        let mut n = 0;
        for (i, slot) in data.iter_mut().enumerate() {
            let addr = start_addr.wrapping_add(i as u32);
            if addr > 0xFFFF {
                break;
            }
            *slot = g.gb.bus.read8(addr as u16).unwrap_or(0xFF);
            n += 1;
        }
        Ok(n)
    }

    fn write_addrs(&mut self, start_addr: u32, data: &[u8]) -> TargetResult<(), Self> {
        let mut g = self.machine.lock().map_err(|_| TargetError::Fatal("lock"))?;
        for (i, &byte) in data.iter().enumerate() {
            let addr = start_addr.wrapping_add(i as u32);
            if addr > 0xFFFF {
                break;
            }
            let _ = g.gb.bus.write8(addr as u16, byte);
        }
        Ok(())
    }

    #[inline(always)]
    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadResume for SoftGbcGdbTarget {
    fn resume(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        if signal.is_some() {
            return Err("continuing with signal not supported");
        }
        let mut g = self.machine.lock().map_err(|_| "lock")?;
        g.exec_mode = ExecMode::Continue;
        g.gdb_halted = false;
        Ok(())
    }

    #[inline(always)]
    fn support_single_step(&mut self) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadSingleStep for SoftGbcGdbTarget {
    fn step(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        if signal.is_some() {
            return Err("stepping with signal not supported");
        }
        let mut g = self.machine.lock().map_err(|_| "lock")?;
        g.exec_mode = ExecMode::Step;
        g.gdb_halted = false;
        Ok(())
    }
}

impl SingleRegisterAccess<()> for SoftGbcGdbTarget {
    fn read_register(
        &mut self,
        _tid: (),
        reg_id: Sm83RegId,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let g = self.machine.lock().map_err(|_| TargetError::Fatal("lock"))?;
        let regs = GdbMachine::from_regs(&g.gb.cpu);
        let val: u32 = match reg_id {
            Sm83RegId::EaxAf => u32::from(regs.af),
            Sm83RegId::EcxBc => u32::from(regs.bc),
            Sm83RegId::EdxDe => u32::from(regs.de),
            Sm83RegId::EbxHl => u32::from(regs.hl),
            Sm83RegId::EspSp => u32::from(regs.sp),
            Sm83RegId::EipPc => u32::from(regs.pc),
            Sm83RegId::EflagsIme => u32::from(regs.ime),
            Sm83RegId::Ebp | Sm83RegId::Esi | Sm83RegId::Edi | Sm83RegId::Seg(_) => 0,
        };
        if buf.len() < 4 {
            return Err(TargetError::Fatal("reg buf"));
        }
        buf[..4].copy_from_slice(&val.to_le_bytes());
        Ok(4)
    }

    fn write_register(
        &mut self,
        _tid: (),
        reg_id: Sm83RegId,
        val: &[u8],
    ) -> TargetResult<(), Self> {
        if val.len() < 4 {
            return Err(TargetError::Fatal("reg val"));
        }
        let w32 = u32::from_le_bytes([val[0], val[1], val[2], val[3]]);
        let w = w32 as u16;
        let mut g = self.machine.lock().map_err(|_| TargetError::Fatal("lock"))?;
        match reg_id {
            Sm83RegId::EaxAf => g.gb.cpu.set_af(w),
            Sm83RegId::EcxBc => g.gb.cpu.set_bc(w),
            Sm83RegId::EdxDe => g.gb.cpu.set_de(w),
            Sm83RegId::EbxHl => g.gb.cpu.set_hl(w),
            Sm83RegId::EspSp => g.gb.cpu.sp = w,
            Sm83RegId::EipPc => g.gb.cpu.pc = w,
            Sm83RegId::EflagsIme => g.gb.cpu.ime = w32 != 0,
            Sm83RegId::Ebp | Sm83RegId::Esi | Sm83RegId::Edi | Sm83RegId::Seg(_) => {}
        }
        Ok(())
    }
}

impl Breakpoints for SoftGbcGdbTarget {
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for SoftGbcGdbTarget {
    fn add_sw_breakpoint(
        &mut self,
        addr: u32,
        _kind: Sm83BreakpointKind,
    ) -> TargetResult<bool, Self> {
        if addr > 0xFFFF {
            return Ok(false);
        }
        let mut g = self.machine.lock().map_err(|_| TargetError::Fatal("lock"))?;
        g.breakpoints.insert(addr as u16);
        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: u32,
        _kind: Sm83BreakpointKind,
    ) -> TargetResult<bool, Self> {
        if addr > 0xFFFF {
            return Ok(false);
        }
        let mut g = self.machine.lock().map_err(|_| TargetError::Fatal("lock"))?;
        Ok(g.breakpoints.remove(&(addr as u16)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm83_iss::Cpu;

    #[test]
    fn regs_from_cpu_match_pairs() {
        let mut cpu = Cpu::new();
        cpu.set_af(0x01B0);
        cpu.set_bc(0x1234);
        cpu.pc = 0x0150;
        cpu.sp = 0xFFFE;
        let regs = GdbMachine::from_regs(&cpu);
        assert_eq!(regs.af, 0x01B0);
        assert_eq!(regs.bc, 0x1234);
        assert_eq!(regs.pc, 0x0150);
        assert_eq!(regs.sp, 0xFFFE);
    }

    #[test]
    fn memory_peek_via_target_reads_rom_entry() {
        let mut raw = vec![0u8; 0x200];
        raw[0x0100] = 0x00;
        raw[0x0101] = 0x76;
        let gb = SoftGbc::from_rom_bytes(&raw);
        let machine = Arc::new(Mutex::new(GdbMachine::new(gb)));
        let mut target = SoftGbcGdbTarget::new(machine);
        let mut buf = [0u8; 2];
        let n = match target.read_addrs(0x0100, &mut buf) {
            Ok(n) => n,
            Err(_) => panic!("read_addrs failed"),
        };
        assert_eq!(n, 2);
        assert_eq!(buf, [0x00, 0x76]);
    }

    #[test]
    fn step_hits_software_breakpoint_at_next_pc() {
        let mut raw = vec![0u8; 0x200];
        raw[0x0100] = 0x00; // NOP
        raw[0x0101] = 0x00; // NOP
        let gb = SoftGbc::from_rom_bytes(&raw);
        let mut m = GdbMachine::new(gb);
        m.breakpoints.insert(0x0101);
        assert!(m.step_insn()); // after NOP at 100, PC=101 → break
        assert_eq!(m.gb.cpu.pc, 0x0101);
    }

    /// ROM: one OBJ over a blank BG, then the VBlank handler turns the LCD off
    /// and hides the sprite. Hardware showed that picture for the whole visible
    /// frame; a snapshot taken after the handler is a blank LCD-off frame.
    fn lcd_off_in_vblank_rom() -> Vec<u8> {
        let mut rom = vec![0u8; 0x200];
        // $0040 VBlank: LCD off + sprite Y = 0 (would drop the OBJ if we render late).
        let handler = [
            0x3E, 0x00, // LD A, 0
            0xE0, 0x40, // LDH (LCDC), A
            0xAF, // XOR A
            0xEA, 0x00, 0xFE, // LD ($FE00), A
            0xD9, // RETI
        ];
        rom[0x0040..0x0040 + handler.len()].copy_from_slice(&handler);
        // $0100 setup, then HALT until VBlank.
        let entry = [
            0x31, 0xFE, 0xFF, // LD SP, $FFFE
            0x3E, 0x38, // LD A, 56        ; screen Y = 40
            0xEA, 0x00, 0xFE, // LD ($FE00), A
            0x3E, 0x30, // LD A, 48        ; screen X = 40
            0xEA, 0x01, 0xFE, // LD ($FE01), A
            0x3E, 0x01, // LD A, 1         ; tile 1
            0xEA, 0x02, 0xFE, // LD ($FE02), A
            0xAF, // XOR A                 ; flags 0
            0xEA, 0x03, 0xFE, // LD ($FE03), A
            0x3E, 0xFF, // LD A, $FF       ; tile 1 solid colour 3
            0x21, 0x10, 0x80, // LD HL, $8010
            0x06, 0x10, // LD B, 16
            0x22, // LD (HL+), A
            0x05, // DEC B
            0x20, 0xFC, // JR NZ, fill
            0x3E, 0xE4, // LD A, $E4
            0xE0, 0x47, // LDH (BGP), A
            0xE0, 0x48, // LDH (OBP0), A
            0x3E, 0x01, // LD A, 1
            0xE0, 0xFF, // LDH (IE), A     ; VBlank
            0x3E, 0x93, // LD A, $93       ; LCD + OBJ + BG + $8000
            0xE0, 0x40, // LDH (LCDC), A
            0xFB, // EI
            0x00, // NOP                    ; IME arms after this
            0x76, // HALT
            0x18, 0xFD, // JR HALT
        ];
        rom[0x0100..0x0100 + entry.len()].copy_from_slice(&entry);
        rom
    }

    #[test]
    fn advance_frame_presents_visible_frame_before_lcd_off_blank() {
        let mut m = GdbMachine::new(SoftGbc::from_rom_bytes(&lcd_off_in_vblank_rom()));
        m.advance_frame();
        // Handler ran: LCD is off and the sprite Y was cleared. A render *now*
        // would blank. The framebuffer must still be the completed visible frame.
        assert_eq!(m.gb.bus.ppu.LCDC & 0x80, 0, "VBlank handler turned the LCD off");
        assert_eq!(m.gb.bus.oam[0], 0, "handler hid the sprite after the frame");
        assert_eq!(
            m.gb.bus.ppu.framebuffer[40 * 160 + 40],
            3,
            "OBJ pixel from the completed visible frame"
        );
        assert_eq!(m.gb.bus.ppu.framebuffer[0], 0, "BG colour 0 stays shade 0");

        // LCD stays off for the next budget: the screen really is blank.
        m.advance_frame();
        assert!(
            m.gb.bus.ppu.framebuffer.iter().all(|&p| p == 0),
            "LCD off for the whole frame blanks"
        );
    }
}
