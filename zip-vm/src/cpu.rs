use crate::flags;

/// Z80 CPU state. Memory is owned directly — no separate bus abstraction.
pub struct Cpu {
    // Main registers
    pub a: u8, pub f: u8,
    pub b: u8, pub c: u8,
    pub d: u8, pub e: u8,
    pub h: u8, pub l: u8,
    // Shadow registers
    pub a_: u8, pub f_: u8,
    pub b_: u8, pub c_: u8,
    pub d_: u8, pub e_: u8,
    pub h_: u8, pub l_: u8,
    // Index, stack, program counter
    pub ix: u16, pub iy: u16,
    pub sp: u16, pub pc: u16,
    // Special registers (mostly decorative for ZIP)
    pub i: u8, pub r: u8,
    pub iff1: bool, pub iff2: bool,
    pub im: u8,
    pub halted: bool,
    /// Monotonic T-state counter (not used for timing, just debug).
    pub cycles: u64,
    /// 64 KB flat memory.
    pub mem: Box<[u8; 0x10000]>,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0xFF, f: 0xFF,
            b: 0, c: 0, d: 0, e: 0, h: 0, l: 0,
            a_: 0, f_: 0, b_: 0, c_: 0, d_: 0, e_: 0, h_: 0, l_: 0,
            ix: 0, iy: 0, sp: 0, pc: 0,
            i: 0, r: 0,
            iff1: false, iff2: false, im: 0,
            halted: false, cycles: 0,
            mem: vec![0u8; 0x10000].into_boxed_slice().try_into().unwrap(),
        }
    }

    // ---- Memory access ----

    pub fn read8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }
    pub fn write8(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }
    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.mem[addr as usize] as u16;
        let hi = self.mem[addr.wrapping_add(1) as usize] as u16;
        lo | (hi << 8)
    }
    pub fn write16(&mut self, addr: u16, val: u16) {
        self.mem[addr as usize] = val as u8;
        self.mem[addr.wrapping_add(1) as usize] = (val >> 8) as u8;
    }

    // ---- Fetch (read at PC, advance PC) ----

    pub fn fetch8(&mut self) -> u8 {
        let v = self.read8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }
    pub fn fetch16(&mut self) -> u16 {
        let lo = self.fetch8() as u16;
        let hi = self.fetch8() as u16;
        lo | (hi << 8)
    }

    // ---- Stack ----

    pub fn push16(&mut self, val: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.write16(self.sp, val);
    }
    pub fn pop16(&mut self) -> u16 {
        let v = self.read16(self.sp);
        self.sp = self.sp.wrapping_add(2);
        v
    }

    // ---- Register pairs ----

    pub fn af(&self) -> u16 { (self.a as u16) << 8 | self.f as u16 }
    pub fn bc(&self) -> u16 { (self.b as u16) << 8 | self.c as u16 }
    pub fn de(&self) -> u16 { (self.d as u16) << 8 | self.e as u16 }
    pub fn hl(&self) -> u16 { (self.h as u16) << 8 | self.l as u16 }

    pub fn set_af(&mut self, v: u16) { self.a = (v >> 8) as u8; self.f = v as u8; }
    pub fn set_bc(&mut self, v: u16) { self.b = (v >> 8) as u8; self.c = v as u8; }
    pub fn set_de(&mut self, v: u16) { self.d = (v >> 8) as u8; self.e = v as u8; }
    pub fn set_hl(&mut self, v: u16) { self.h = (v >> 8) as u8; self.l = v as u8; }

    // ---- Register access by 3-bit code ----
    // 0=B 1=C 2=D 3=E 4=H 5=L 6=(HL) 7=A

    pub fn reg8(&self, r: u8) -> u8 {
        match r & 7 {
            0 => self.b, 1 => self.c,
            2 => self.d, 3 => self.e,
            4 => self.h, 5 => self.l,
            6 => self.read8(self.hl()),
            7 => self.a,
            _ => unreachable!()
        }
    }
    pub fn set_reg8(&mut self, r: u8, v: u8) {
        match r & 7 {
            0 => self.b = v, 1 => self.c = v,
            2 => self.d = v, 3 => self.e = v,
            4 => self.h = v, 5 => self.l = v,
            6 => { let addr = self.hl(); self.write8(addr, v); },
            7 => self.a = v,
            _ => unreachable!()
        }
    }

    /// Register pair by 2-bit code: 0=BC 1=DE 2=HL 3=SP
    pub fn rp(&self, p: u8) -> u16 {
        match p & 3 { 0 => self.bc(), 1 => self.de(), 2 => self.hl(), 3 => self.sp, _ => unreachable!() }
    }
    pub fn set_rp(&mut self, p: u8, v: u16) {
        match p & 3 { 0 => self.set_bc(v), 1 => self.set_de(v), 2 => self.set_hl(v), 3 => self.sp = v, _ => unreachable!() }
    }
    /// Register pair (AF variant for PUSH/POP): 0=BC 1=DE 2=HL 3=AF
    pub fn rp2(&self, p: u8) -> u16 {
        match p & 3 { 0 => self.bc(), 1 => self.de(), 2 => self.hl(), 3 => self.af(), _ => unreachable!() }
    }
    pub fn set_rp2(&mut self, p: u8, v: u16) {
        match p & 3 { 0 => self.set_bc(v), 1 => self.set_de(v), 2 => self.set_hl(v), 3 => self.set_af(v), _ => unreachable!() }
    }

    // ---- Index register helpers (for DD/FD prefix) ----

    pub fn get_idx(&self, is_ix: bool) -> u16 { if is_ix { self.ix } else { self.iy } }
    pub fn set_idx(&mut self, is_ix: bool, v: u16) { if is_ix { self.ix = v; } else { self.iy = v; } }

    /// Read (IX/IY + d) where d is a signed displacement already fetched.
    pub fn read_idx(&self, is_ix: bool, d: i8) -> u8 {
        let addr = self.get_idx(is_ix).wrapping_add(d as u16);
        self.read8(addr)
    }
    pub fn write_idx(&mut self, is_ix: bool, d: i8, v: u8) {
        let addr = self.get_idx(is_ix).wrapping_add(d as u16);
        self.write8(addr, v);
    }

    // ---- Condition codes ----
    // 0=NZ 1=Z 2=NC 3=C 4=PO 5=PE 6=P 7=M

    pub fn check_cc(&self, cc: u8) -> bool {
        match cc & 7 {
            0 => self.f & flags::Z == 0,
            1 => self.f & flags::Z != 0,
            2 => self.f & flags::C == 0,
            3 => self.f & flags::C != 0,
            4 => self.f & flags::PV == 0,
            5 => self.f & flags::PV != 0,
            6 => self.f & flags::S == 0,
            7 => self.f & flags::S != 0,
            _ => unreachable!()
        }
    }

    // ---- ALU operations ----

    pub fn alu_op(&mut self, op: u8, val: u8) {
        let a = self.a;
        match op & 7 {
            0 => { let (r, f) = flags::add8(a, val, false); self.a = r; self.f = f; }
            1 => { let (r, f) = flags::add8(a, val, self.f & flags::C != 0); self.a = r; self.f = f; }
            2 => { let (r, f) = flags::sub8(a, val, false); self.a = r; self.f = f; }
            3 => { let (r, f) = flags::sub8(a, val, self.f & flags::C != 0); self.a = r; self.f = f; }
            4 => { self.a = a & val; self.f = flags::szp(self.a) | flags::H; }
            5 => { self.a = a ^ val; self.f = flags::szp(self.a); }
            6 => { self.a = a | val; self.f = flags::szp(self.a); }
            7 => { let (_, f) = flags::sub8(a, val, false); self.f = f; } // CP: flags only
            _ => unreachable!()
        }
    }

    // ---- EX helpers ----

    pub fn ex_af(&mut self) {
        std::mem::swap(&mut self.a, &mut self.a_);
        std::mem::swap(&mut self.f, &mut self.f_);
    }
    pub fn exx(&mut self) {
        std::mem::swap(&mut self.b, &mut self.b_);
        std::mem::swap(&mut self.c, &mut self.c_);
        std::mem::swap(&mut self.d, &mut self.d_);
        std::mem::swap(&mut self.e, &mut self.e_);
        std::mem::swap(&mut self.h, &mut self.h_);
        std::mem::swap(&mut self.l, &mut self.l_);
    }
}
