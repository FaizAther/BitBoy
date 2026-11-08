//! SM83 CPU instruction-set simulator.

use softgbc_core::Bus;

pub const PC_START: u16 = 0x0100;
pub const SP_START: u16 = 0xFFFE;

const FLAG_Z: u8 = 0x80;
const FLAG_N: u8 = 0x40;
const FLAG_H: u8 = 0x20;
const FLAG_C: u8 = 0x10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cpu {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
    pub halted: bool,
    ei_delay: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: SP_START,
            pc: PC_START,
            ime: false,
            halted: false,
            ei_delay: false,
        }
    }

    pub fn af(&self) -> u16 {
        u16::from(self.a) << 8 | u16::from(self.f & 0xF0)
    }
    pub fn bc(&self) -> u16 {
        u16::from(self.b) << 8 | u16::from(self.c)
    }
    pub fn de(&self) -> u16 {
        u16::from(self.d) << 8 | u16::from(self.e)
    }
    pub fn hl(&self) -> u16 {
        u16::from(self.h) << 8 | u16::from(self.l)
    }

    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val as u8) & 0xF0;
    }
    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8;
    }
    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8;
    }
    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8;
    }

    pub fn z_flag(&self) -> bool {
        self.f & FLAG_Z != 0
    }
    pub fn n_flag(&self) -> bool {
        self.f & FLAG_N != 0
    }
    pub fn h_flag(&self) -> bool {
        self.f & FLAG_H != 0
    }
    pub fn c_flag(&self) -> bool {
        self.f & FLAG_C != 0
    }

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.f = 0;
        if z {
            self.f |= FLAG_Z;
        }
        if n {
            self.f |= FLAG_N;
        }
        if h {
            self.f |= FLAG_H;
        }
        if c {
            self.f |= FLAG_C;
        }
    }

    fn read8(&self, bus: &impl Bus, addr: u16) -> u8 {
        bus.read8(addr)
            .unwrap_or_else(|| panic!("bus read failed at {:04X}", addr))
    }

    fn write8(&self, bus: &mut impl Bus, addr: u16, data: u8) {
        // ROM / unmapped writes are ignored on real hardware.
        let _ = bus.write8(addr, data);
    }

    fn read_r(&self, bus: &impl Bus, r: u8) -> u8 {
        match r & 7 {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.read8(bus, self.hl()),
            7 => self.a,
            _ => unreachable!(),
        }
    }

    fn write_r(&mut self, bus: &mut impl Bus, r: u8, val: u8) {
        match r & 7 {
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => self.write8(bus, self.hl(), val),
            7 => self.a = val,
            _ => unreachable!(),
        }
    }

    fn fetch8(&mut self, bus: &mut impl Bus) -> u8 {
        let b = self.read8(bus, self.pc);
        self.pc = self.pc.wrapping_add(1);
        b
    }

    fn fetch16(&mut self, bus: &mut impl Bus) -> u16 {
        let lo = self.fetch8(bus) as u16;
        let hi = self.fetch8(bus) as u16;
        hi << 8 | lo
    }

    fn push16(&mut self, bus: &mut impl Bus, val: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.write8(bus, self.sp, (val >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.write8(bus, self.sp, val as u8);
    }

    fn pop16(&mut self, bus: &mut impl Bus) -> u16 {
        let lo = self.read8(bus, self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = self.read8(bus, self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        hi << 8 | lo
    }

    fn alu_add(&mut self, value: u8, carry_in: bool) {
        let c_in = u8::from(carry_in);
        let result = u16::from(self.a) + u16::from(value) + u16::from(c_in);
        let half = (self.a & 0x0F) + (value & 0x0F) + c_in > 0x0F;
        self.a = result as u8;
        self.set_flags(self.a == 0, false, half, result > 0xFF);
    }

    fn alu_sub(&mut self, value: u8, carry_in: bool, store: bool) {
        let a = self.a;
        let c_in = u8::from(carry_in);
        let result = a.wrapping_sub(value).wrapping_sub(c_in);
        let half = (a & 0x0F) < (value & 0x0F).wrapping_add(c_in);
        let carry = (a as u16) < (value as u16) + (c_in as u16);
        if store {
            self.a = result;
        }
        self.set_flags(result == 0, true, half, carry);
    }

    fn alu_and(&mut self, value: u8) {
        self.a &= value;
        self.set_flags(self.a == 0, false, true, false);
    }

    fn alu_xor(&mut self, value: u8) {
        self.a ^= value;
        self.set_flags(self.a == 0, false, false, false);
    }

    fn alu_or(&mut self, value: u8) {
        self.a |= value;
        self.set_flags(self.a == 0, false, false, false);
    }

    fn add_hl(&mut self, value: u16) {
        let hl = self.hl();
        let result = u32::from(hl) + u32::from(value);
        let half = (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;
        self.set_hl(result as u16);
        self.set_flags(self.z_flag(), false, half, result > 0xFFFF);
    }

    fn daa(&mut self) {
        let mut a = self.a;
        let mut adjust = 0u8;
        let mut c = false;
        if self.n_flag() {
            if self.h_flag() {
                adjust |= 0x06;
            }
            if self.c_flag() {
                adjust |= 0x60;
                c = true;
            }
            a = a.wrapping_sub(adjust);
        } else {
            if self.h_flag() || a & 0x0F > 0x09 {
                adjust |= 0x06;
            }
            if self.c_flag() || a > 0x99 {
                adjust |= 0x60;
                c = true;
            }
            a = a.wrapping_add(adjust);
        }
        self.a = a;
        self.set_flags(a == 0, self.n_flag(), false, c);
    }

    fn rlc(&mut self, val: u8) -> u8 {
        let c = val & 0x80 != 0;
        let res = val.rotate_left(1);
        self.set_flags(res == 0, false, false, c);
        res
    }

    fn rrc(&mut self, val: u8) -> u8 {
        let c = val & 0x01 != 0;
        let res = val.rotate_right(1);
        self.set_flags(res == 0, false, false, c);
        res
    }

    fn rl(&mut self, val: u8) -> u8 {
        let c = val & 0x80 != 0;
        let res = (val << 1) | u8::from(self.c_flag());
        self.set_flags(res == 0, false, false, c);
        res
    }

    fn rr(&mut self, val: u8) -> u8 {
        let c = val & 0x01 != 0;
        let res = (val >> 1) | (u8::from(self.c_flag()) << 7);
        self.set_flags(res == 0, false, false, c);
        res
    }

    fn sla(&mut self, val: u8) -> u8 {
        let c = val & 0x80 != 0;
        let res = val << 1;
        self.set_flags(res == 0, false, false, c);
        res
    }

    fn sra(&mut self, val: u8) -> u8 {
        let c = val & 0x01 != 0;
        let res = (val >> 1) | (val & 0x80);
        self.set_flags(res == 0, false, false, c);
        res
    }

    fn srl(&mut self, val: u8) -> u8 {
        let c = val & 0x01 != 0;
        let res = val >> 1;
        self.set_flags(res == 0, false, false, c);
        res
    }

    fn swap(&mut self, val: u8) -> u8 {
        let res = val.rotate_left(4);
        self.set_flags(res == 0, false, false, false);
        res
    }

    /// Execute one instruction. Returns M-cycles.
    pub fn step(&mut self, bus: &mut impl Bus) -> u8 {
        if self.ei_delay {
            self.ime = true;
            self.ei_delay = false;
        }

        if self.halted {
            return 1;
        }

        let opcode = self.fetch8(bus);
        self.execute(bus, opcode)
    }

    fn execute(&mut self, bus: &mut impl Bus, opcode: u8) -> u8 {
        match opcode {
            0x00 => 1, // NOP

            0x01 => {
                let v = self.fetch16(bus);
                self.set_bc(v);
                3
            }
            0x11 => {
                let v = self.fetch16(bus);
                self.set_de(v);
                3
            }
            0x21 => {
                let v = self.fetch16(bus);
                self.set_hl(v);
                3
            }
            0x31 => {
                self.sp = self.fetch16(bus);
                3
            }

            0x02 => {
                self.write8(bus, self.bc(), self.a);
                2
            }
            0x12 => {
                self.write8(bus, self.de(), self.a);
                2
            }
            0x22 => {
                let hl = self.hl();
                self.write8(bus, hl, self.a);
                self.set_hl(hl.wrapping_add(1));
                2
            }
            0x32 => {
                let hl = self.hl();
                self.write8(bus, hl, self.a);
                self.set_hl(hl.wrapping_sub(1));
                2
            }

            0x0A => {
                self.a = self.read8(bus, self.bc());
                2
            }
            0x1A => {
                self.a = self.read8(bus, self.de());
                2
            }
            0x2A => {
                let hl = self.hl();
                self.a = self.read8(bus, hl);
                self.set_hl(hl.wrapping_add(1));
                2
            }
            0x3A => {
                let hl = self.hl();
                self.a = self.read8(bus, hl);
                self.set_hl(hl.wrapping_sub(1));
                2
            }

            0x03 => {
                self.set_bc(self.bc().wrapping_add(1));
                2
            }
            0x13 => {
                self.set_de(self.de().wrapping_add(1));
                2
            }
            0x23 => {
                self.set_hl(self.hl().wrapping_add(1));
                2
            }
            0x33 => {
                self.sp = self.sp.wrapping_add(1);
                2
            }
            0x0B => {
                self.set_bc(self.bc().wrapping_sub(1));
                2
            }
            0x1B => {
                self.set_de(self.de().wrapping_sub(1));
                2
            }
            0x2B => {
                self.set_hl(self.hl().wrapping_sub(1));
                2
            }
            0x3B => {
                self.sp = self.sp.wrapping_sub(1);
                2
            }

            0x09 => {
                self.add_hl(self.bc());
                2
            }
            0x19 => {
                self.add_hl(self.de());
                2
            }
            0x29 => {
                self.add_hl(self.hl());
                2
            }
            0x39 => {
                self.add_hl(self.sp);
                2
            }

            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                let r = (opcode >> 3) & 7;
                let old = self.read_r(bus, r);
                let v = old.wrapping_add(1);
                self.write_r(bus, r, v);
                self.set_flags(v == 0, false, (old & 0x0F) == 0x0F, self.c_flag());
                if r == 6 { 3 } else { 1 }
            }

            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                let r = (opcode >> 3) & 7;
                let old = self.read_r(bus, r);
                let v = old.wrapping_sub(1);
                self.write_r(bus, r, v);
                self.set_flags(v == 0, true, (old & 0x0F) == 0, self.c_flag());
                if r == 6 { 3 } else { 1 }
            }

            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                let r = (opcode >> 3) & 7;
                let v = self.fetch8(bus);
                self.write_r(bus, r, v);
                if r == 6 { 3 } else { 2 }
            }

            0x07 => {
                // RLCA — Z always cleared
                let c = self.a & 0x80 != 0;
                self.a = self.a.rotate_left(1);
                self.set_flags(false, false, false, c);
                1
            }
            0x0F => {
                let c = self.a & 0x01 != 0;
                self.a = self.a.rotate_right(1);
                self.set_flags(false, false, false, c);
                1
            }
            0x17 => {
                let c = self.a & 0x80 != 0;
                self.a = (self.a << 1) | u8::from(self.c_flag());
                self.set_flags(false, false, false, c);
                1
            }
            0x1F => {
                let c = self.a & 0x01 != 0;
                self.a = (self.a >> 1) | (u8::from(self.c_flag()) << 7);
                self.set_flags(false, false, false, c);
                1
            }

            0x08 => {
                let addr = self.fetch16(bus);
                self.write8(bus, addr, self.sp as u8);
                self.write8(bus, addr.wrapping_add(1), (self.sp >> 8) as u8);
                5
            }

            0x10 => {
                let _ = self.fetch8(bus); // STOP reads one extra byte
                self.halted = true;
                1
            }

            0x18 => {
                let e = self.fetch8(bus) as i8;
                self.pc = self.pc.wrapping_add(e as u16);
                3
            }
            0x20 => self.jr_cc(bus, !self.z_flag()),
            0x28 => self.jr_cc(bus, self.z_flag()),
            0x30 => self.jr_cc(bus, !self.c_flag()),
            0x38 => self.jr_cc(bus, self.c_flag()),

            0x27 => {
                self.daa();
                1
            }
            0x2F => {
                self.a = !self.a;
                self.set_flags(self.z_flag(), true, true, self.c_flag());
                1
            }
            0x37 => {
                self.set_flags(self.z_flag(), false, false, true);
                1
            }
            0x3F => {
                self.set_flags(self.z_flag(), false, false, !self.c_flag());
                1
            }

            0x76 => {
                self.halted = true;
                1
            }

            0x40..=0x7F => {
                let src = opcode & 7;
                let dst = (opcode >> 3) & 7;
                let v = self.read_r(bus, src);
                self.write_r(bus, dst, v);
                if src == 6 || dst == 6 { 2 } else { 1 }
            }

            0x80..=0x87 => {
                let v = self.read_r(bus, opcode & 7);
                self.alu_add(v, false);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0x88..=0x8F => {
                let v = self.read_r(bus, opcode & 7);
                self.alu_add(v, self.c_flag());
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0x90..=0x97 => {
                let v = self.read_r(bus, opcode & 7);
                self.alu_sub(v, false, true);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0x98..=0x9F => {
                let v = self.read_r(bus, opcode & 7);
                self.alu_sub(v, self.c_flag(), true);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0xA0..=0xA7 => {
                let v = self.read_r(bus, opcode & 7);
                self.alu_and(v);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0xA8..=0xAF => {
                let v = self.read_r(bus, opcode & 7);
                self.alu_xor(v);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0xB0..=0xB7 => {
                let v = self.read_r(bus, opcode & 7);
                self.alu_or(v);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0xB8..=0xBF => {
                let v = self.read_r(bus, opcode & 7);
                self.alu_sub(v, false, false);
                if opcode & 7 == 6 { 2 } else { 1 }
            }

            0xC1 => {
                let v = self.pop16(bus);
                self.set_bc(v);
                3
            }
            0xD1 => {
                let v = self.pop16(bus);
                self.set_de(v);
                3
            }
            0xE1 => {
                let v = self.pop16(bus);
                self.set_hl(v);
                3
            }
            0xF1 => {
                let v = self.pop16(bus);
                self.set_af(v);
                3
            }

            0xC5 => {
                self.push16(bus, self.bc());
                4
            }
            0xD5 => {
                self.push16(bus, self.de());
                4
            }
            0xE5 => {
                self.push16(bus, self.hl());
                4
            }
            0xF5 => {
                self.push16(bus, self.af());
                4
            }

            0xC6 => {
                let v = self.fetch8(bus);
                self.alu_add(v, false);
                2
            }
            0xCE => {
                let v = self.fetch8(bus);
                self.alu_add(v, self.c_flag());
                2
            }
            0xD6 => {
                let v = self.fetch8(bus);
                self.alu_sub(v, false, true);
                2
            }
            0xDE => {
                let v = self.fetch8(bus);
                self.alu_sub(v, self.c_flag(), true);
                2
            }
            0xE6 => {
                let v = self.fetch8(bus);
                self.alu_and(v);
                2
            }
            0xEE => {
                let v = self.fetch8(bus);
                self.alu_xor(v);
                2
            }
            0xF6 => {
                let v = self.fetch8(bus);
                self.alu_or(v);
                2
            }
            0xFE => {
                let v = self.fetch8(bus);
                self.alu_sub(v, false, false);
                2
            }

            0xC3 => {
                self.pc = self.fetch16(bus);
                4
            }
            0xC2 => self.jp_cc(bus, !self.z_flag()),
            0xCA => self.jp_cc(bus, self.z_flag()),
            0xD2 => self.jp_cc(bus, !self.c_flag()),
            0xDA => self.jp_cc(bus, self.c_flag()),
            0xE9 => {
                self.pc = self.hl();
                1
            }

            0xCD => {
                let target = self.fetch16(bus);
                self.push16(bus, self.pc);
                self.pc = target;
                6
            }
            0xC4 => self.call_cc(bus, !self.z_flag()),
            0xCC => self.call_cc(bus, self.z_flag()),
            0xD4 => self.call_cc(bus, !self.c_flag()),
            0xDC => self.call_cc(bus, self.c_flag()),

            0xC9 => {
                self.pc = self.pop16(bus);
                4
            }
            0xD9 => {
                self.pc = self.pop16(bus);
                self.ime = true;
                4
            }
            0xC0 => self.ret_cc(bus, !self.z_flag()),
            0xC8 => self.ret_cc(bus, self.z_flag()),
            0xD0 => self.ret_cc(bus, !self.c_flag()),
            0xD8 => self.ret_cc(bus, self.c_flag()),

            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let target = (opcode & 0x38) as u16;
                self.push16(bus, self.pc);
                self.pc = target;
                4
            }

            0xE0 => {
                let n = self.fetch8(bus) as u16;
                self.write8(bus, 0xFF00 | n, self.a);
                3
            }
            0xF0 => {
                let n = self.fetch8(bus) as u16;
                self.a = self.read8(bus, 0xFF00 | n);
                3
            }
            0xE2 => {
                self.write8(bus, 0xFF00 | u16::from(self.c), self.a);
                2
            }
            0xF2 => {
                self.a = self.read8(bus, 0xFF00 | u16::from(self.c));
                2
            }

            0xEA => {
                let addr = self.fetch16(bus);
                self.write8(bus, addr, self.a);
                4
            }
            0xFA => {
                let addr = self.fetch16(bus);
                self.a = self.read8(bus, addr);
                4
            }

            0xE8 => {
                let e = self.fetch8(bus) as i8 as i16;
                let sp = self.sp;
                let result = (sp as i16).wrapping_add(e) as u16;
                let sp_lo = sp as u8;
                let e_u = e as u8;
                let half = (sp_lo & 0x0F) + (e_u & 0x0F) > 0x0F;
                let carry = sp_lo as u16 + e_u as u16 > 0xFF;
                self.sp = result;
                self.set_flags(false, false, half, carry);
                4
            }
            0xF8 => {
                let e = self.fetch8(bus) as i8 as i16;
                let sp = self.sp;
                let result = (sp as i16).wrapping_add(e) as u16;
                let sp_lo = sp as u8;
                let e_u = e as u8;
                let half = (sp_lo & 0x0F) + (e_u & 0x0F) > 0x0F;
                let carry = sp_lo as u16 + e_u as u16 > 0xFF;
                self.set_hl(result);
                self.set_flags(false, false, half, carry);
                3
            }
            0xF9 => {
                self.sp = self.hl();
                2
            }

            0xF3 => {
                self.ime = false;
                self.ei_delay = false;
                1
            }
            0xFB => {
                self.ei_delay = true;
                1
            }

            0xCB => {
                let cb = self.fetch8(bus);
                self.execute_cb(bus, cb)
            }

            // Unused on SM83 / Game Boy (including GBC CPU)
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                panic!("invalid opcode {:02X}", opcode);
            }

            other => panic!("unimplemented inst {:02X}", other),
        }
    }

    fn execute_cb(&mut self, bus: &mut impl Bus, op: u8) -> u8 {
        let r = op & 7;
        let cycles = if r == 6 { 4 } else { 2 };

        match op {
            0x00..=0x07 => {
                let v = self.rlc(self.read_r(bus, r));
                self.write_r(bus, r, v);
            }
            0x08..=0x0F => {
                let v = self.rrc(self.read_r(bus, r));
                self.write_r(bus, r, v);
            }
            0x10..=0x17 => {
                let v = self.rl(self.read_r(bus, r));
                self.write_r(bus, r, v);
            }
            0x18..=0x1F => {
                let v = self.rr(self.read_r(bus, r));
                self.write_r(bus, r, v);
            }
            0x20..=0x27 => {
                let v = self.sla(self.read_r(bus, r));
                self.write_r(bus, r, v);
            }
            0x28..=0x2F => {
                let v = self.sra(self.read_r(bus, r));
                self.write_r(bus, r, v);
            }
            0x30..=0x37 => {
                let v = self.swap(self.read_r(bus, r));
                self.write_r(bus, r, v);
            }
            0x38..=0x3F => {
                let v = self.srl(self.read_r(bus, r));
                self.write_r(bus, r, v);
            }
            0x40..=0x7F => {
                let bit = (op >> 3) & 7;
                let val = self.read_r(bus, r);
                let z = val & (1 << bit) == 0;
                self.set_flags(z, false, true, self.c_flag());
                return if r == 6 { 3 } else { 2 };
            }
            0x80..=0xBF => {
                let bit = (op >> 3) & 7;
                let val = self.read_r(bus, r) & !(1 << bit);
                self.write_r(bus, r, val);
            }
            0xC0..=0xFF => {
                let bit = (op >> 3) & 7;
                let val = self.read_r(bus, r) | (1 << bit);
                self.write_r(bus, r, val);
            }
        }
        cycles
    }

    fn jr_cc(&mut self, bus: &mut impl Bus, take: bool) -> u8 {
        let e = self.fetch8(bus) as i8;
        if take {
            self.pc = self.pc.wrapping_add(e as u16);
            3
        } else {
            2
        }
    }

    fn jp_cc(&mut self, bus: &mut impl Bus, take: bool) -> u8 {
        let target = self.fetch16(bus);
        if take {
            self.pc = target;
            4
        } else {
            3
        }
    }

    fn call_cc(&mut self, bus: &mut impl Bus, take: bool) -> u8 {
        let target = self.fetch16(bus);
        if take {
            self.push16(bus, self.pc);
            self.pc = target;
            6
        } else {
            3
        }
    }

    fn ret_cc(&mut self, bus: &mut impl Bus, take: bool) -> u8 {
        if take {
            self.pc = self.pop16(bus);
            5
        } else {
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use softgbc_core::DmgBus;

    fn load(bus: &mut DmgBus, at: u16, bytes: &[u8]) {
        for (i, b) in bytes.iter().enumerate() {
            assert!(bus.poke(at.wrapping_add(i as u16), *b));
        }
    }

    fn cpu_at(pc: u16) -> Cpu {
        let mut cpu = Cpu::new();
        cpu.pc = pc;
        cpu
    }

    #[test]
    fn test_new() {
        let cpu = Cpu::new();
        assert_eq!(cpu.pc, PC_START);
        assert_eq!(cpu.sp, SP_START);
    }

    #[test]
    fn test_nop() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x00, 0x00, 0x00]);
        let mut cpu = Cpu::new();
        assert_eq!(cpu.step(&mut bus), 1);
        assert_eq!(cpu.pc, 0x0101);
    }

    #[test]
    fn test_ld_r_d8() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x3E, 0x77]);
        let mut cpu = Cpu::new();
        cpu.step(&mut bus);
        assert_eq!(cpu.a, 0x77);
    }

    #[test]
    fn test_ld_rr_d16() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x01, 0x34, 0x12]);
        let mut cpu = Cpu::new();
        cpu.step(&mut bus);
        assert_eq!(cpu.bc(), 0x1234);
    }

    #[test]
    fn test_ld_r_r() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x78]);
        let mut cpu = cpu_at(0x0100);
        cpu.b = 0xAB;
        cpu.step(&mut bus);
        assert_eq!(cpu.a, 0xAB);
    }

    #[test]
    fn test_ld_hl_mem() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x36, 0x42, 0x7E]); // LD (HL),0x42 ; LD A,(HL)
        let mut cpu = Cpu::new();
        cpu.set_hl(0xC000);
        cpu.step(&mut bus);
        assert_eq!(bus.read8(0xC000), Some(0x42));
        cpu.step(&mut bus);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_ld_a_indirect() {
        let mut bus = DmgBus::new();
        bus.write8(0xC000, 0x99);
        load(&mut bus, 0x0100, &[0x0A]); // LD A,(BC)
        let mut cpu = cpu_at(0x0100);
        cpu.set_bc(0xC000);
        cpu.step(&mut bus);
        assert_eq!(cpu.a, 0x99);
    }

    #[test]
    fn test_ldi_ldd() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x22, 0x3A]); // LD (HL+),A ; LD A,(HL-)
        let mut cpu = Cpu::new();
        cpu.a = 0x55;
        cpu.set_hl(0xC000);
        cpu.step(&mut bus);
        assert_eq!(bus.read8(0xC000), Some(0x55));
        assert_eq!(cpu.hl(), 0xC001);
        bus.write8(0xC001, 0x66);
        cpu.step(&mut bus);
        assert_eq!(cpu.a, 0x66);
        assert_eq!(cpu.hl(), 0xC000);
    }

    #[test]
    fn test_inc_dec_r() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x04, 0x05]);
        let mut cpu = cpu_at(0x0100);
        cpu.b = 0x0F;
        cpu.step(&mut bus);
        assert_eq!(cpu.b, 0x10);
        assert!(cpu.h_flag());
        cpu.step(&mut bus);
        assert_eq!(cpu.b, 0x0F);
        assert!(cpu.n_flag());
    }

    #[test]
    fn test_inc_dec_rr() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x03, 0x0B]);
        let mut cpu = cpu_at(0x0100);
        cpu.set_bc(0x00FF);
        cpu.step(&mut bus);
        assert_eq!(cpu.bc(), 0x0100);
        cpu.step(&mut bus);
        assert_eq!(cpu.bc(), 0x00FF);
    }

    #[test]
    fn test_add_hl() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x09]);
        let mut cpu = cpu_at(0x0100);
        cpu.set_hl(0x1000);
        cpu.set_bc(0x0100);
        cpu.step(&mut bus);
        assert_eq!(cpu.hl(), 0x1100);
    }

    #[test]
    fn test_jr_jp() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x18, 0x02]);
        let mut cpu = Cpu::new();
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0x0104);

        load(&mut bus, 0x0200, &[0xC3, 0x00, 0x30]);
        cpu.pc = 0x0200;
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0x3000);
    }

    #[test]
    fn test_jr_z() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x28, 0xFE]);
        let mut cpu = Cpu::new();
        cpu.set_flags(true, false, false, false);
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0x0100);
    }

    #[test]
    fn test_alu() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x80, 0x90, 0xA0, 0xA8, 0xB0, 0xB8]);
        let mut cpu = cpu_at(0x0100);
        cpu.a = 0x10;
        cpu.b = 0x01;
        cpu.step(&mut bus); // ADD
        assert_eq!(cpu.a, 0x11);
        cpu.step(&mut bus); // SUB
        assert_eq!(cpu.a, 0x10);
        cpu.a = 0xF0;
        cpu.b = 0x0F;
        cpu.step(&mut bus); // AND
        assert_eq!(cpu.a, 0x00);
        cpu.a = 0xF0;
        cpu.b = 0x0F;
        cpu.step(&mut bus); // XOR
        assert_eq!(cpu.a, 0xFF);
        cpu.a = 0xF0;
        cpu.b = 0x0F;
        cpu.step(&mut bus); // OR
        assert_eq!(cpu.a, 0xFF);
        cpu.a = 0x10;
        cpu.b = 0x10;
        cpu.step(&mut bus); // CP
        assert!(cpu.z_flag());
    }

    #[test]
    fn test_push_pop() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xC5, 0xC1]);
        let mut cpu = Cpu::new();
        cpu.set_bc(0xABCD);
        cpu.sp = 0xFFFE;
        cpu.step(&mut bus);
        assert_eq!(cpu.sp, 0xFFFC);
        cpu.set_bc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.bc(), 0xABCD);
        assert_eq!(cpu.sp, 0xFFFE);
    }

    #[test]
    fn test_call_ret() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xCD, 0x00, 0x20]); // CALL 0x2000
        load(&mut bus, 0x2000, &[0xC9]); // RET
        let mut cpu = Cpu::new();
        cpu.sp = 0xFFFE;
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0x2000);
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0x0103);
    }

    #[test]
    fn test_rst() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xCF]); // RST 08H
        let mut cpu = Cpu::new();
        cpu.sp = 0xFFFE;
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0x0008);
    }

    #[test]
    fn test_ldh() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xE0, 0x80, 0xF0, 0x80]); // LDH (80h),A ; LDH A,(80h)
        let mut cpu = Cpu::new();
        cpu.a = 0x5A;
        cpu.step(&mut bus);
        assert_eq!(bus.read8(0xFF80), Some(0x5A));
        cpu.a = 0;
        cpu.step(&mut bus);
        assert_eq!(cpu.a, 0x5A);
    }

    #[test]
    fn test_ld_a_nn() {
        let mut bus = DmgBus::new();
        bus.write8(0xC000, 0x11);
        load(&mut bus, 0x0100, &[0xFA, 0x00, 0xC0, 0xEA, 0x01, 0xC0]);
        let mut cpu = Cpu::new();
        cpu.step(&mut bus);
        assert_eq!(cpu.a, 0x11);
        cpu.a = 0x22;
        cpu.step(&mut bus);
        assert_eq!(bus.read8(0xC001), Some(0x22));
    }

    #[test]
    fn test_add_sp_e_ld_hl_sp_e() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xE8, 0x02, 0xF8, 0x02]);
        let mut cpu = Cpu::new();
        cpu.sp = 0xFFF0;
        cpu.step(&mut bus);
        assert_eq!(cpu.sp, 0xFFF2);
        cpu.sp = 0xFFF0;
        cpu.step(&mut bus);
        assert_eq!(cpu.hl(), 0xFFF2);
    }

    #[test]
    fn test_ld_sp_hl() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xF9]);
        let mut cpu = cpu_at(0x0100);
        cpu.set_hl(0x1234);
        cpu.step(&mut bus);
        assert_eq!(cpu.sp, 0x1234);
    }

    #[test]
    fn test_rotates_a() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x07, 0x0F]);
        let mut cpu = cpu_at(0x0100);
        cpu.a = 0x80;
        cpu.step(&mut bus); // RLCA
        assert_eq!(cpu.a, 0x01);
        assert!(cpu.c_flag());
        cpu.step(&mut bus); // RRCA
        assert_eq!(cpu.a, 0x80);
    }

    #[test]
    fn test_daa_cpl_scf_ccf() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x27, 0x2F, 0x37, 0x3F]);
        let mut cpu = cpu_at(0x0100);
        cpu.a = 0x0A; // needs DAA adjust after ADD-like state
        cpu.set_flags(false, false, false, false);
        cpu.step(&mut bus); // DAA
        assert_eq!(cpu.a, 0x10);
        cpu.step(&mut bus); // CPL
        assert_eq!(cpu.a, !0x10);
        assert!(cpu.n_flag() && cpu.h_flag());
        cpu.step(&mut bus); // SCF
        assert!(cpu.c_flag());
        cpu.step(&mut bus); // CCF
        assert!(!cpu.c_flag());
    }

    #[test]
    fn test_di_ei() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xFB, 0x00, 0xF3]);
        let mut cpu = Cpu::new();
        assert!(!cpu.ime);
        cpu.step(&mut bus); // EI (delayed)
        assert!(!cpu.ime);
        cpu.step(&mut bus); // NOP — IME enables after previous EI delay
        assert!(cpu.ime);
        cpu.step(&mut bus); // DI
        assert!(!cpu.ime);
    }

    #[test]
    fn test_halt() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0x76]);
        let mut cpu = Cpu::new();
        cpu.step(&mut bus);
        assert!(cpu.halted);
        let pc = cpu.pc;
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, pc); // stays halted
    }

    #[test]
    fn test_cb_rlc_bit_res_set() {
        let mut bus = DmgBus::new();
        load(
            &mut bus,
            0x0100,
            &[
                0xCB, 0x07, // RLC A
                0xCB, 0x47, // BIT 0,A
                0xCB, 0x87, // RES 0,A
                0xCB, 0xC7, // SET 0,A
                0xCB, 0x37, // SWAP A
            ],
        );
        let mut cpu = Cpu::new();
        cpu.a = 0x80;
        cpu.step(&mut bus);
        assert_eq!(cpu.a, 0x01);
        assert!(cpu.c_flag());
        cpu.step(&mut bus); // BIT 0,A — set
        assert!(!cpu.z_flag());
        cpu.step(&mut bus); // RES 0
        assert_eq!(cpu.a, 0x00);
        cpu.step(&mut bus); // SET 0
        assert_eq!(cpu.a, 0x01);
        cpu.a = 0x1F;
        cpu.step(&mut bus); // SWAP
        assert_eq!(cpu.a, 0xF1);
    }

    #[test]
    fn test_cb_sla_sra_srl() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xCB, 0x27, 0xCB, 0x2F, 0xCB, 0x3F]);
        let mut cpu = Cpu::new();
        cpu.a = 0x81;
        cpu.step(&mut bus); // SLA A
        assert_eq!(cpu.a, 0x02);
        assert!(cpu.c_flag());
        cpu.a = 0x81;
        cpu.step(&mut bus); // SRA A
        assert_eq!(cpu.a, 0xC0);
        cpu.a = 0x81;
        cpu.step(&mut bus); // SRL A
        assert_eq!(cpu.a, 0x40);
    }

    #[test]
    fn test_jp_hl() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xE9]);
        let mut cpu = cpu_at(0x0100);
        cpu.set_hl(0xABCD);
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0xABCD);
    }

    #[test]
    fn test_reti() {
        let mut bus = DmgBus::new();
        load(&mut bus, 0x0100, &[0xCD, 0x00, 0x20]);
        load(&mut bus, 0x2000, &[0xD9]);
        let mut cpu = Cpu::new();
        cpu.sp = 0xFFFE;
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0x0103);
        assert!(cpu.ime);
    }
}
