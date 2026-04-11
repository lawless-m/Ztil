use crate::cpu::Cpu;
use crate::flags;

/// Z80 opcode decoder. Uses the standard x/y/z bit-field decomposition.
/// Undocumented opcodes (IXH/IXL, SLL, DDCB/FDCB undocumented) are not
/// implemented — they panic. ZEXDOC does not test them.
impl Cpu {
    /// Execute one instruction. Returns T-states consumed.
    pub fn step(&mut self) -> u32 {
        if self.halted { return 4; }
        self.r = self.r.wrapping_add(1) & 0x7F;
        let op = self.fetch8();
        let t = self.execute(op);
        self.cycles += t as u64;
        t
    }

    fn execute(&mut self, op: u8) -> u32 {
        match op {
            0xCB => return self.exec_cb(),
            0xDD => return self.exec_indexed(true),
            0xED => return self.exec_ed(),
            0xFD => return self.exec_indexed(false),
            _ => {}
        }
        let x = (op >> 6) & 3;
        let y = (op >> 3) & 7;
        let z = op & 7;
        let p = y >> 1;
        let q = y & 1;

        match x {
            0 => self.exec_x0(y, z, p, q),
            1 => {
                if op == 0x76 { self.halted = true; return 4; }
                let v = self.reg8(z);
                self.set_reg8(y, v);
                if y == 6 || z == 6 { 7 } else { 4 }
            }
            2 => {
                let v = self.reg8(z);
                self.alu_op(y, v);
                if z == 6 { 7 } else { 4 }
            }
            3 => self.exec_x3(y, z, p, q),
            _ => unreachable!()
        }
    }

    // ---- x=0 block ----

    fn exec_x0(&mut self, y: u8, z: u8, p: u8, q: u8) -> u32 {
        match z {
            0 => match y {
                0 => 4, // NOP
                1 => { self.ex_af(); 4 }
                2 => { // DJNZ d
                    let d = self.fetch8() as i8;
                    self.b = self.b.wrapping_sub(1);
                    if self.b != 0 { self.pc = self.pc.wrapping_add(d as u16); 13 } else { 8 }
                }
                3 => { // JR d
                    let d = self.fetch8() as i8;
                    self.pc = self.pc.wrapping_add(d as u16);
                    12
                }
                4..=7 => { // JR cc, d (cc = y-4, only NZ/Z/NC/C)
                    let d = self.fetch8() as i8;
                    if self.check_cc(y - 4) { self.pc = self.pc.wrapping_add(d as u16); 12 } else { 7 }
                }
                _ => unreachable!()
            }
            1 => {
                if q == 0 { // LD rp, nn
                    let v = self.fetch16();
                    self.set_rp(p, v);
                    10
                } else { // ADD HL, rp
                    let hl = self.hl();
                    let rp = self.rp(p);
                    let (result, f) = flags::add16(hl, rp);
                    self.set_hl(result);
                    // Preserve S, Z, PV; merge C, H, clear N
                    self.f = (self.f & (flags::S | flags::Z | flags::PV)) | f;
                    11
                }
            }
            2 => {
                match (p, q) {
                    (0, 0) => { let addr = self.bc(); self.write8(addr, self.a); 7 } // LD (BC),A
                    (1, 0) => { let addr = self.de(); self.write8(addr, self.a); 7 } // LD (DE),A
                    (2, 0) => { let addr = self.fetch16(); self.write16(addr, self.hl()); 16 } // LD (nn),HL
                    (3, 0) => { let addr = self.fetch16(); self.write8(addr, self.a); 13 }  // LD (nn),A
                    (0, 1) => { let addr = self.bc(); self.a = self.read8(addr); 7 }  // LD A,(BC)
                    (1, 1) => { let addr = self.de(); self.a = self.read8(addr); 7 }  // LD A,(DE)
                    (2, 1) => { let addr = self.fetch16(); let v = self.read16(addr); self.set_hl(v); 16 } // LD HL,(nn)
                    (3, 1) => { let addr = self.fetch16(); self.a = self.read8(addr); 13 }  // LD A,(nn)
                    _ => unreachable!()
                }
            }
            3 => {
                if q == 0 { // INC rp
                    let v = self.rp(p).wrapping_add(1);
                    self.set_rp(p, v);
                } else { // DEC rp
                    let v = self.rp(p).wrapping_sub(1);
                    self.set_rp(p, v);
                }
                6
            }
            4 => { // INC r
                let v = self.reg8(y);
                let (result, f) = flags::inc8(v);
                self.set_reg8(y, result);
                self.f = f | (self.f & flags::C); // preserve C
                if y == 6 { 11 } else { 4 }
            }
            5 => { // DEC r
                let v = self.reg8(y);
                let (result, f) = flags::dec8(v);
                self.set_reg8(y, result);
                self.f = f | (self.f & flags::C); // preserve C
                if y == 6 { 11 } else { 4 }
            }
            6 => { // LD r, n
                let n = self.fetch8();
                self.set_reg8(y, n);
                if y == 6 { 10 } else { 7 }
            }
            7 => match y {
                0 => { // RLCA
                    let bit7 = self.a >> 7;
                    self.a = (self.a << 1) | bit7;
                    self.f = (self.f & (flags::S | flags::Z | flags::PV)) | (self.a & flags::F53) | bit7;
                    4
                }
                1 => { // RRCA
                    let bit0 = self.a & 1;
                    self.a = (self.a >> 1) | (bit0 << 7);
                    self.f = (self.f & (flags::S | flags::Z | flags::PV)) | (self.a & flags::F53) | bit0;
                    4
                }
                2 => { // RLA
                    let old_c = self.f & flags::C;
                    let bit7 = self.a >> 7;
                    self.a = (self.a << 1) | old_c;
                    self.f = (self.f & (flags::S | flags::Z | flags::PV)) | (self.a & flags::F53) | bit7;
                    4
                }
                3 => { // RRA
                    let old_c = self.f & flags::C;
                    let bit0 = self.a & 1;
                    self.a = (self.a >> 1) | (old_c << 7);
                    self.f = (self.f & (flags::S | flags::Z | flags::PV)) | (self.a & flags::F53) | bit0;
                    4
                }
                4 => { // DAA
                    let a = self.a;
                    let n = self.f & flags::N != 0;
                    let hf = self.f & flags::H != 0;
                    let cf = self.f & flags::C != 0;
                    // Compute full correction from original A (Z80 determines both
                    // nibble corrections in one step, not sequentially)
                    let mut correction: u8 = 0;
                    let mut new_cf = cf;
                    if hf || (a & 0x0F) > 9 { correction |= 0x06; }
                    if cf || a > 0x99 { correction |= 0x60; new_cf = true; }
                    self.a = if n { a.wrapping_sub(correction) } else { a.wrapping_add(correction) };
                    self.f = flags::szp(self.a)
                        | (self.f & flags::N)
                        | if new_cf { flags::C } else { 0 }
                        | ((a ^ self.a) & flags::H);
                    4
                }
                5 => { // CPL
                    self.a = !self.a;
                    self.f = (self.f & (flags::S | flags::Z | flags::PV | flags::C)) | (self.a & flags::F53) | flags::H | flags::N;
                    4
                }
                6 => { // SCF
                    self.f = (self.f & (flags::S | flags::Z | flags::PV)) | (self.a & flags::F53) | flags::C;
                    4
                }
                7 => { // CCF
                    let old_c = self.f & flags::C;
                    self.f = (self.f & (flags::S | flags::Z | flags::PV)) | (self.a & flags::F53)
                        | if old_c != 0 { flags::H } else { flags::C };
                    4
                }
                _ => unreachable!()
            }
            _ => unreachable!()
        }
    }

    // ---- x=3 block ----

    fn exec_x3(&mut self, y: u8, z: u8, p: u8, q: u8) -> u32 {
        match z {
            0 => { // RET cc
                if self.check_cc(y) { self.pc = self.pop16(); 11 } else { 5 }
            }
            1 => {
                if q == 0 { // POP rp2
                    let v = self.pop16();
                    self.set_rp2(p, v);
                    10
                } else {
                    match p {
                        0 => { self.pc = self.pop16(); 10 } // RET
                        1 => { self.exx(); 4 } // EXX
                        2 => { self.pc = self.hl(); 4 } // JP (HL)
                        3 => { self.sp = self.hl(); 6 } // LD SP,HL
                        _ => unreachable!()
                    }
                }
            }
            2 => { // JP cc, nn
                let addr = self.fetch16();
                if self.check_cc(y) { self.pc = addr; }
                10
            }
            3 => match y {
                0 => { self.pc = self.fetch16(); 10 } // JP nn
                1 => unreachable!(), // CB prefix handled above
                2 => { // OUT (n),A — stub
                    let _port = self.fetch8();
                    11
                }
                3 => { // IN A,(n) — stub
                    let _port = self.fetch8();
                    self.a = 0xFF;
                    11
                }
                4 => { // EX (SP),HL
                    let sp_val = self.read16(self.sp);
                    let hl = self.hl();
                    self.write16(self.sp, hl);
                    self.set_hl(sp_val);
                    19
                }
                5 => { // EX DE,HL
                    let de = self.de(); let hl = self.hl();
                    self.set_de(hl); self.set_hl(de);
                    4
                }
                6 => { self.iff1 = false; self.iff2 = false; 4 } // DI
                7 => { self.iff1 = true; self.iff2 = true; 4 }   // EI
                _ => unreachable!()
            }
            4 => { // CALL cc, nn
                let addr = self.fetch16();
                if self.check_cc(y) { self.push16(self.pc); self.pc = addr; 17 } else { 10 }
            }
            5 => {
                if q == 0 { // PUSH rp2
                    let v = self.rp2(p);
                    self.push16(v);
                    11
                } else {
                    match p {
                        0 => { // CALL nn
                            let addr = self.fetch16();
                            self.push16(self.pc);
                            self.pc = addr;
                            17
                        }
                        1 | 2 | 3 => unreachable!(), // DD, ED, FD handled above
                        _ => unreachable!()
                    }
                }
            }
            6 => { // ALU n
                let n = self.fetch8();
                self.alu_op(y, n);
                7
            }
            7 => { // RST y*8
                self.push16(self.pc);
                self.pc = (y as u16) * 8;
                11
            }
            _ => unreachable!()
        }
    }

    // ---- CB prefix: bit/rotate/shift ----

    fn exec_cb(&mut self) -> u32 {
        self.r = self.r.wrapping_add(1) & 0x7F;
        let op = self.fetch8();
        let x = (op >> 6) & 3;
        let y = (op >> 3) & 7;
        let z = op & 7;
        let val = self.reg8(z);
        let is_hl = z == 6;

        match x {
            0 => { // Rotate/shift
                let result = match y {
                    0 => { let c = val >> 7; self.f = c; ((val << 1) | c, c) }.0, // RLC
                    1 => { let c = val & 1; self.f = c; ((val >> 1) | (c << 7), c) }.0, // RRC
                    2 => { // RL
                        let old_c = self.f & flags::C;
                        let c = val >> 7;
                        let r = (val << 1) | old_c;
                        self.f = c;
                        r
                    }
                    3 => { // RR
                        let old_c = self.f & flags::C;
                        let c = val & 1;
                        let r = (val >> 1) | (old_c << 7);
                        self.f = c;
                        r
                    }
                    4 => { let c = val >> 7; self.f = c; (val << 1, c).0 }  // SLA
                    5 => { let c = val & 1; self.f = c; ((val as i8 >> 1) as u8, c).0 } // SRA
                    6 => { let c = val >> 7; self.f = c; ((val << 1) | 1, c).0 } // SLL (undocumented, but encode for completeness)
                    7 => { let c = val & 1; self.f = c; (val >> 1, c).0 } // SRL
                    _ => unreachable!()
                };
                self.f = flags::szp(result) | (self.f & flags::C);
                self.set_reg8(z, result);
                if is_hl { 15 } else { 8 }
            }
            1 => { // BIT y, r
                let bit = val & (1 << y);
                // Undocumented: for registers, bits 3,5 from the register value;
                // for (HL), from high byte of address. We approximate with val for registers.
                let f53_src = if is_hl { (self.hl() >> 8) as u8 } else { val };
                self.f = (self.f & flags::C) | flags::H
                    | if bit == 0 { flags::Z | flags::PV } else { 0 }
                    | (bit & flags::S)
                    | (f53_src & flags::F53);
                if is_hl { 12 } else { 8 }
            }
            2 => { // RES y, r
                self.set_reg8(z, val & !(1 << y));
                if is_hl { 15 } else { 8 }
            }
            3 => { // SET y, r
                self.set_reg8(z, val | (1 << y));
                if is_hl { 15 } else { 8 }
            }
            _ => unreachable!()
        }
    }

    // ---- ED prefix: extended instructions ----

    fn exec_ed(&mut self) -> u32 {
        self.r = self.r.wrapping_add(1) & 0x7F;
        let op = self.fetch8();
        let y = (op >> 3) & 7;
        let _z = op & 7;
        let p = y >> 1;
        let _q = y & 1;

        match op {
            // Block transfer
            0xA0 => { self.ldi(); 16 }
            0xA8 => { self.ldd(); 16 }
            0xB0 => { self.ldi(); if self.bc() != 0 { self.pc = self.pc.wrapping_sub(2); 21 } else { 16 } } // LDIR
            0xB8 => { self.ldd(); if self.bc() != 0 { self.pc = self.pc.wrapping_sub(2); 21 } else { 16 } } // LDDR
            // Block compare
            0xA1 => { self.cpi(); 16 }
            0xA9 => { self.cpd(); 16 }
            0xB1 => { self.cpi(); if self.bc() != 0 && self.f & flags::Z == 0 { self.pc = self.pc.wrapping_sub(2); 21 } else { 16 } }
            0xB9 => { self.cpd(); if self.bc() != 0 && self.f & flags::Z == 0 { self.pc = self.pc.wrapping_sub(2); 21 } else { 16 } }
            // Block I/O — stubs (NOP behavior)
            0xA2 | 0xAA | 0xB2 | 0xBA | 0xA3 | 0xAB | 0xB3 | 0xBB => 16,
            // NEG
            0x44 | 0x4C | 0x54 | 0x5C | 0x64 | 0x6C | 0x74 | 0x7C => {
                let a = self.a;
                let (r, f) = flags::sub8(0, a, false);
                self.a = r;
                self.f = f;
                8
            }
            // RETI / RETN
            0x4D => { self.pc = self.pop16(); 14 } // RETI
            0x45 | 0x55 | 0x65 | 0x75 | 0x5D | 0x6D | 0x7D => {
                self.iff1 = self.iff2;
                self.pc = self.pop16();
                14
            }
            // IM
            0x46 | 0x66 => { self.im = 0; 8 }
            0x56 | 0x76 => { self.im = 1; 8 }
            0x5E | 0x7E => { self.im = 2; 8 }
            // LD I,A / LD R,A / LD A,I / LD A,R
            0x47 => { self.i = self.a; 9 }
            0x4F => { self.r = self.a; 9 }
            0x57 => { // LD A,I
                self.a = self.i;
                self.f = (self.f & flags::C) | flags::szp(self.a) & !(flags::PV)
                    | if self.iff2 { flags::PV } else { 0 };
                // Clear parity from szp, set PV = IFF2
                self.f = (self.f & flags::C) | (self.a & flags::S) | if self.a == 0 { flags::Z } else { 0 }
                    | if self.iff2 { flags::PV } else { 0 };
                9
            }
            0x5F => { // LD A,R
                self.a = self.r;
                self.f = (self.f & flags::C) | (self.a & flags::S) | if self.a == 0 { flags::Z } else { 0 }
                    | if self.iff2 { flags::PV } else { 0 };
                9
            }
            // RLD / RRD
            0x6F => { // RLD
                let hl = self.hl();
                let mem = self.read8(hl);
                let new_mem = (mem << 4) | (self.a & 0x0F);
                self.a = (self.a & 0xF0) | (mem >> 4);
                self.write8(hl, new_mem);
                self.f = flags::szp(self.a) | (self.f & flags::C);
                18
            }
            0x67 => { // RRD
                let hl = self.hl();
                let mem = self.read8(hl);
                let new_mem = (self.a << 4) | (mem >> 4);
                self.a = (self.a & 0xF0) | (mem & 0x0F);
                self.write8(hl, new_mem);
                self.f = flags::szp(self.a) | (self.f & flags::C);
                18
            }
            _ if (op & 0xCF) == 0x43 => { // LD (nn),rp  — ED 43/53/63/73
                let addr = self.fetch16();
                let v = self.rp(p);
                self.write16(addr, v);
                20
            }
            _ if (op & 0xCF) == 0x4B => { // LD rp,(nn)  — ED 4B/5B/6B/7B
                let addr = self.fetch16();
                let v = self.read16(addr);
                self.set_rp(p, v);
                20
            }
            _ if (op & 0xCF) == 0x4A => { // ADC HL,rp
                let hl = self.hl();
                let rp = self.rp(p);
                let (result, f) = flags::adc16(hl, rp, self.f & flags::C != 0);
                self.set_hl(result);
                self.f = f;
                15
            }
            _ if (op & 0xCF) == 0x42 => { // SBC HL,rp
                let hl = self.hl();
                let rp = self.rp(p);
                let (result, f) = flags::sbc16(hl, rp, self.f & flags::C != 0);
                self.set_hl(result);
                self.f = f;
                15
            }
            // IN r,(C) / OUT (C),r — stubs
            _ if (op & 0xC7) == 0x40 => { // IN r,(C)
                self.set_reg8(y, 0xFF);
                self.f = flags::szp(0xFF) | (self.f & flags::C);
                12
            }
            _ if (op & 0xC7) == 0x41 => { // OUT (C),r
                12
            }
            // Everything else in ED space is a NOP on real Z80
            _ => 8
        }
    }

    // ---- Block transfer/compare helpers ----

    fn ldi(&mut self) {
        let val = self.read8(self.hl());
        self.write8(self.de(), val);
        self.set_hl(self.hl().wrapping_add(1));
        self.set_de(self.de().wrapping_add(1));
        self.set_bc(self.bc().wrapping_sub(1));
        // Undocumented: n = A + val; F5 = bit 1 of n, F3 = bit 3 of n
        let n = self.a.wrapping_add(val);
        self.f = (self.f & (flags::S | flags::Z | flags::C))
            | if self.bc() != 0 { flags::PV } else { 0 }
            | (n & flags::F3)
            | if n & 0x02 != 0 { flags::F5 } else { 0 };
    }
    fn ldd(&mut self) {
        let val = self.read8(self.hl());
        self.write8(self.de(), val);
        self.set_hl(self.hl().wrapping_sub(1));
        self.set_de(self.de().wrapping_sub(1));
        self.set_bc(self.bc().wrapping_sub(1));
        let n = self.a.wrapping_add(val);
        self.f = (self.f & (flags::S | flags::Z | flags::C))
            | if self.bc() != 0 { flags::PV } else { 0 }
            | (n & flags::F3)
            | if n & 0x02 != 0 { flags::F5 } else { 0 };
    }
    fn cpi(&mut self) {
        let val = self.read8(self.hl());
        let (result, f) = flags::sub8(self.a, val, false);
        self.set_hl(self.hl().wrapping_add(1));
        self.set_bc(self.bc().wrapping_sub(1));
        // Undocumented: n = A - val - HF; F5 = bit 1 of n, F3 = bit 3 of n
        let n = result.wrapping_sub(if f & flags::H != 0 { 1 } else { 0 });
        self.f = (f & !(flags::C | flags::PV | flags::F53)) | (self.f & flags::C)
            | if self.bc() != 0 { flags::PV } else { 0 }
            | (n & flags::F3)
            | if n & 0x02 != 0 { flags::F5 } else { 0 };
    }
    fn cpd(&mut self) {
        let val = self.read8(self.hl());
        let (result, f) = flags::sub8(self.a, val, false);
        self.set_hl(self.hl().wrapping_sub(1));
        self.set_bc(self.bc().wrapping_sub(1));
        let n = result.wrapping_sub(if f & flags::H != 0 { 1 } else { 0 });
        self.f = (f & !(flags::C | flags::PV | flags::F53)) | (self.f & flags::C)
            | if self.bc() != 0 { flags::PV } else { 0 }
            | (n & flags::F3)
            | if n & 0x02 != 0 { flags::F5 } else { 0 };
    }

    // ---- DD/FD prefix: IX/IY indexed ----

    fn exec_indexed(&mut self, is_ix: bool) -> u32 {
        self.r = self.r.wrapping_add(1) & 0x7F;
        let op = self.fetch8();

        if op == 0xCB { return self.exec_indexed_cb(is_ix); }

        let x = (op >> 6) & 3;
        let y = (op >> 3) & 7;
        let z = op & 7;
        let p = y >> 1;
        #[allow(unused_variables)]
        let q = y & 1;

        // Only certain opcodes are affected by the DD/FD prefix.
        // Anything referencing HL (16-bit) or (HL) is redirected.
        // H/L individual refs are undocumented IXH/IXL — we fall through to unprefixed.

        match op {
            // LD IX/IY, nn
            0x21 => { let v = self.fetch16(); self.set_idx(is_ix, v); 14 }
            // LD (nn), IX/IY
            0x22 => { let addr = self.fetch16(); let v = self.get_idx(is_ix); self.write16(addr, v); 20 }
            // LD IX/IY, (nn)
            0x2A => { let addr = self.fetch16(); let v = self.read16(addr); self.set_idx(is_ix, v); 20 }
            // INC IX/IY
            0x23 => { let v = self.get_idx(is_ix).wrapping_add(1); self.set_idx(is_ix, v); 10 }
            // DEC IX/IY
            0x2B => { let v = self.get_idx(is_ix).wrapping_sub(1); self.set_idx(is_ix, v); 10 }
            // ADD IX/IY, rp
            0x09 | 0x19 | 0x29 | 0x39 => {
                let idx = self.get_idx(is_ix);
                let rp = match p {
                    0 => self.bc(),
                    1 => self.de(),
                    2 => self.get_idx(is_ix), // ADD IX,IX
                    3 => self.sp,
                    _ => unreachable!()
                };
                let (result, f) = flags::add16(idx, rp);
                self.set_idx(is_ix, result);
                self.f = (self.f & (flags::S | flags::Z | flags::PV)) | f;
                15
            }
            // PUSH IX/IY
            0xE5 => { let v = self.get_idx(is_ix); self.push16(v); 15 }
            // POP IX/IY
            0xE1 => { let v = self.pop16(); self.set_idx(is_ix, v); 14 }
            // EX (SP), IX/IY
            0xE3 => {
                let sp_val = self.read16(self.sp);
                let idx = self.get_idx(is_ix);
                self.write16(self.sp, idx);
                self.set_idx(is_ix, sp_val);
                23
            }
            // JP (IX/IY)
            0xE9 => { self.pc = self.get_idx(is_ix); 8 }
            // LD SP, IX/IY
            0xF9 => { self.sp = self.get_idx(is_ix); 10 }

            // INC (IX/IY+d)
            0x34 => {
                let d = self.fetch8() as i8;
                let addr = self.get_idx(is_ix).wrapping_add(d as u16);
                let v = self.read8(addr);
                let (result, f) = flags::inc8(v);
                self.write8(addr, result);
                self.f = f | (self.f & flags::C);
                23
            }
            // DEC (IX/IY+d)
            0x35 => {
                let d = self.fetch8() as i8;
                let addr = self.get_idx(is_ix).wrapping_add(d as u16);
                let v = self.read8(addr);
                let (result, f) = flags::dec8(v);
                self.write8(addr, result);
                self.f = f | (self.f & flags::C);
                23
            }
            // LD (IX/IY+d), n
            0x36 => {
                let d = self.fetch8() as i8;
                let n = self.fetch8();
                let addr = self.get_idx(is_ix).wrapping_add(d as u16);
                self.write8(addr, n);
                19
            }

            // LD r, (IX/IY+d)  — opcodes 0x46,0x4E,0x56,0x5E,0x66,0x6E,0x7E
            _ if x == 1 && z == 6 && y != 6 => {
                let d = self.fetch8() as i8;
                let v = self.read_idx(is_ix, d);
                self.set_reg8(y, v);
                19
            }
            // LD (IX/IY+d), r  — opcodes 0x70-0x75, 0x77
            _ if x == 1 && y == 6 && z != 6 => {
                let d = self.fetch8() as i8;
                let v = self.reg8(z);
                self.write_idx(is_ix, d, v);
                19
            }
            // ALU A, (IX/IY+d)  — opcodes 0x86,0x8E,...,0xBE
            _ if x == 2 && z == 6 => {
                let d = self.fetch8() as i8;
                let v = self.read_idx(is_ix, d);
                self.alu_op(y, v);
                19
            }

            // --- Undocumented IXH/IXL/IYH/IYL instructions ---

            // INC/DEC IXH/IXL/IYH/IYL
            _ if x == 0 && z == 4 && (y == 4 || y == 5) => {
                let v = self.idx_reg8(is_ix, y);
                let (result, f) = flags::inc8(v);
                self.set_idx_reg8(is_ix, y, result);
                self.f = f | (self.f & flags::C);
                8
            }
            _ if x == 0 && z == 5 && (y == 4 || y == 5) => {
                let v = self.idx_reg8(is_ix, y);
                let (result, f) = flags::dec8(v);
                self.set_idx_reg8(is_ix, y, result);
                self.f = f | (self.f & flags::C);
                8
            }
            // LD IXH/IXL/IYH/IYL, n
            _ if x == 0 && z == 6 && (y == 4 || y == 5) => {
                let n = self.fetch8();
                self.set_idx_reg8(is_ix, y, n);
                11
            }
            // LD r, IXH/IXL (and LD IXH/IXL, r)
            _ if x == 1 && y != 6 && z != 6 && (y == 4 || y == 5 || z == 4 || z == 5) => {
                let v = self.idx_reg8(is_ix, z);
                self.set_idx_reg8(is_ix, y, v);
                8
            }
            // ALU A, IXH/IXL/IYH/IYL
            _ if x == 2 && (z == 4 || z == 5) => {
                let v = self.idx_reg8(is_ix, z);
                self.alu_op(y, v);
                8
            }

            // Everything else: execute as unprefixed (prefix is consumed)
            _ => self.execute(op)
        }
    }

    // ---- DDCB/FDCB prefix: indexed bit/rotate/shift ----

    fn exec_indexed_cb(&mut self, is_ix: bool) -> u32 {
        // Byte order: DD CB dd op (displacement BEFORE opcode)
        let d = self.fetch8() as i8;
        let op = self.fetch8();
        let x = (op >> 6) & 3;
        let y = (op >> 3) & 7;
        #[allow(unused_variables)]
        let z = op & 7;

        let addr = self.get_idx(is_ix).wrapping_add(d as u16);
        let val = self.read8(addr);

        match x {
            0 => { // Rotate/shift (IX/IY+d)
                let result = match y {
                    0 => { let c = val >> 7; self.f = c; (val << 1) | c }
                    1 => { let c = val & 1; self.f = c; (val >> 1) | (c << 7) }
                    2 => { let old_c = self.f & flags::C; let c = val >> 7; self.f = c; (val << 1) | old_c }
                    3 => { let old_c = self.f & flags::C; let c = val & 1; self.f = c; (val >> 1) | (old_c << 7) }
                    4 => { let c = val >> 7; self.f = c; val << 1 }
                    5 => { let c = val & 1; self.f = c; (val as i8 >> 1) as u8 }
                    6 => { let c = val >> 7; self.f = c; (val << 1) | 1 } // SLL undocumented
                    7 => { let c = val & 1; self.f = c; val >> 1 }
                    _ => unreachable!()
                };
                self.f = flags::szp(result) | (self.f & flags::C);
                self.write8(addr, result);
                // For z != 6, documented behavior stores to register too — undocumented, skip
                23
            }
            1 => { // BIT y, (IX/IY+d)
                let bit = val & (1 << y);
                self.f = (self.f & flags::C) | flags::H
                    | if bit == 0 { flags::Z | flags::PV } else { 0 }
                    | (bit & flags::S)
                    | ((addr >> 8) as u8 & flags::F53);
                20
            }
            2 => { // RES y, (IX/IY+d)
                self.write8(addr, val & !(1 << y));
                23
            }
            3 => { // SET y, (IX/IY+d)
                self.write8(addr, val | (1 << y));
                23
            }
            _ => unreachable!()
        }
    }
}
