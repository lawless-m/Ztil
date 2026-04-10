use crate::cpu::Cpu;
use crate::rom;
use std::collections::VecDeque;

pub const TRAP_BASE: u16 = 0x0040;
pub const TRAP_END: u16 = 0x0100;

pub fn is_trap(pc: u16) -> bool { pc >= TRAP_BASE && pc < TRAP_END }

// ---- Trap IDs ----
pub const DROP: u16      = 0x40;
pub const DUP: u16       = 0x41;
pub const SWAP: u16      = 0x42;
pub const OVER: u16      = 0x43;
pub const STORE: u16     = 0x44; // !
pub const FETCH: u16     = 0x45; // @
pub const CSTORE: u16    = 0x46; // C!
pub const CFETCH: u16    = 0x47; // C@
pub const PSTORE: u16    = 0x48; // +!
pub const COMMA: u16     = 0x49; // ,
pub const CCOMMA: u16    = 0x4A; // C,
pub const ADD: u16       = 0x4B;
pub const SUB: u16       = 0x4C;
pub const MINUS: u16     = 0x4D;
pub const ABS: u16       = 0x4E;
pub const LT: u16        = 0x4F;
pub const EQ: u16        = 0x50;
pub const GT: u16        = 0x51;
pub const ZEROEQ: u16    = 0x52;
pub const AND: u16       = 0x53;
pub const IOR: u16       = 0x54;
pub const XOR: u16       = 0x55;
pub const NOT: u16       = 0x56;
pub const TYPE: u16      = 0x57;
pub const ECHO: u16      = 0x58;
pub const KEY: u16       = 0x59;
pub const SPACE: u16     = 0x5A;
pub const DISPLAY: u16   = 0x5B;
pub const SIGN: u16      = 0x5C;
pub const LHASH: u16     = 0x5D; // <#
pub const HASHR: u16     = 0x5E; // #>
pub const ASCII_CONV: u16= 0x5F;
pub const TO_R: u16      = 0x60; // <R
pub const R_FROM: u16    = 0x61; // R>
pub const I_FETCH: u16   = 0x62; // I>
pub const J_FETCH: u16   = 0x63; // J>
pub const HEX: u16       = 0x64;
pub const DECIMAL: u16   = 0x65;
pub const SINGLE: u16    = 0x66;
pub const ABORT_P: u16   = 0x67;
pub const MOVE_P: u16    = 0x68;
pub const STAR_IF: u16   = 0x69; // *IF / *END (conditional branch)
pub const STAR_BR: u16   = 0x6A; // *WHILE / *ELSE (unconditional branch)
pub const STAR_DO: u16   = 0x6B;
pub const STAR_LOOP: u16 = 0x6C;
pub const STAR_LEAVE: u16= 0x6D;
pub const STAR_PLOOP: u16= 0x6E; // *+LOOP
pub const STAR_NUM: u16  = 0x6F; // *#
pub const STAR_CNUM: u16 = 0x70; // *C#
pub const STAR_BRAK: u16 = 0x71; // *[
pub const STAR_SYS: u16  = 0x72; // *SYS
pub const STAR_STACK: u16= 0x73;
pub const QUESTION: u16  = 0x74;
pub const DOLLAR_PATCH: u16 = 0x75;
pub const COSET: u16     = 0x76;
pub const INLINE: u16    = 0x77;
pub const TOKEN: u16     = 0x78;
pub const SEARCH_P: u16  = 0x79;
pub const NUMBER: u16    = 0x7A;
pub const CONST_CODE: u16= 0x7B;
pub const CCONST_CODE:u16= 0x7C;
pub const VAR_CODE: u16  = 0x7D;
pub const DMOD: u16      = 0x7E;
pub const QSEARCH: u16   = 0x7F; // ?SEARCH (native outer interp component)
pub const QNUMBER: u16   = 0x80; // ?NUMBER
pub const QEXECUTE: u16  = 0x81; // ?EXECUTE
pub const DUP2: u16      = 0x82;
pub const LROT: u16      = 0x83;
pub const RROT: u16      = 0x84;
pub const OCTAL: u16     = 0x85;
pub const BINARY: u16    = 0x86;
pub const STAR_CDO: u16  = 0x87;
pub const STAR_CLOOP: u16= 0x88;
pub const C1SET: u16     = 0x89;
pub const CPSTORE: u16   = 0x8A;
pub const MUL: u16       = 0x8B;
pub const DIV: u16       = 0x8C;
pub const DIVMOD: u16    = 0x8D;
pub const DOUBLE: u16    = 0x8E;
pub const TWOPLUS: u16   = 0x8F;
pub const TWOMINUS: u16  = 0x90;
pub const HALVE: u16     = 0x91;
pub const CI_FETCH: u16  = 0x92;
pub const CJ_FETCH: u16  = 0x93;
pub const C_TO_R: u16    = 0x94;
pub const CR_FROM: u16   = 0x95;
pub const PLUSSP: u16     = 0x96;
pub const MINUSSP: u16   = 0x97;
pub const QRS: u16        = 0x98;
pub const QSP: u16        = 0x99;
pub const DEFINITIONS: u16= 0x9A;
pub const MOD_P: u16      = 0x9B;
pub const ZEROL: u16      = 0x9C; // 0<
pub const ZEROSTORE: u16  = 0x9D;
pub const ONESTORE: u16   = 0x9E;

pub struct Io {
    pub output: Vec<u8>,
    pub input: VecDeque<u8>,
    pub waiting_for_key: bool,
}

impl Io {
    pub fn new() -> Self { Io { output: Vec::new(), input: VecDeque::new(), waiting_for_key: false } }
}

/// Branch helper: read signed offset byte at BC, set BC = BC + offset.
/// Does NOT advance BC first (matches Z80 $ELSE/$WHILE behavior).
fn do_branch(cpu: &mut Cpu) {
    let offset = cpu.read8(cpu.bc()) as i8;
    cpu.set_bc(cpu.bc().wrapping_add(offset as i16 as u16));
}

/// Skip the offset byte: advance BC by 1.
fn skip_offset(cpu: &mut Cpu) {
    cpu.set_bc(cpu.bc().wrapping_add(1));
}

/// Handle a native trap. Returns true if handled.
pub fn handle_trap(cpu: &mut Cpu, io: &mut Io) -> bool {
    let trap = cpu.pc;
    match trap {
        DROP => { cpu.dpop(); }
        DUP  => { let v = cpu.dpeek(); cpu.dpush(v); }
        DUP2 => { let v = cpu.dpeek(); cpu.dpush(v); cpu.dpush(v); }
        SWAP => { let a = cpu.dpop(); let b = cpu.dpop(); cpu.dpush(a); cpu.dpush(b); }
        OVER => { let a = cpu.dpop(); let b = cpu.dpeek(); cpu.dpush(a); cpu.dpush(b); }
        LROT => { let c = cpu.dpop(); let b = cpu.dpop(); let a = cpu.dpop();
                   cpu.dpush(b); cpu.dpush(c); cpu.dpush(a); }
        RROT => { let c = cpu.dpop(); let b = cpu.dpop(); let a = cpu.dpop();
                   cpu.dpush(c); cpu.dpush(a); cpu.dpush(b); }

        STORE  => { let ad = cpu.dpop(); let v = cpu.dpop(); cpu.write16(ad, v); }
        FETCH  => { let ad = cpu.dpop(); let v = cpu.read16(ad); cpu.dpush(v); }
        CSTORE => { let ad = cpu.dpop(); let v = cpu.dpop(); cpu.write8(ad, v as u8); }
        CFETCH => { let ad = cpu.dpop(); cpu.dpush(cpu.read8(ad) as i8 as i16 as u16); }
        PSTORE => { let ad = cpu.dpop(); let v = cpu.dpop();
                    cpu.write16(ad, cpu.read16(ad).wrapping_add(v)); }
        CPSTORE=> { let ad = cpu.dpop(); let v = cpu.dpop() as u8;
                    cpu.write8(ad, cpu.read8(ad).wrapping_add(v)); }
        COMMA  => { let v = cpu.dpop(); enclose_word(cpu, v); }
        CCOMMA => { let v = cpu.dpop() as u8; enclose_byte(cpu, v); }
        ZEROSTORE => { let ad = cpu.dpop(); cpu.write16(ad, 0); }
        ONESTORE  => { let ad = cpu.dpop(); cpu.write16(ad, 1); }

        ADD   => { let b = cpu.dpop(); let a = cpu.dpop(); cpu.dpush(a.wrapping_add(b)); }
        SUB   => { let b = cpu.dpop(); let a = cpu.dpop(); cpu.dpush(a.wrapping_sub(b)); }
        MINUS => { let a = cpu.dpop(); cpu.dpush(0u16.wrapping_sub(a)); }
        ABS   => { let a = cpu.dpop() as i16; cpu.dpush(a.unsigned_abs()); }
        DOUBLE=> { let a = cpu.dpop(); cpu.dpush(a << 1); }
        TWOPLUS => { let a = cpu.dpop(); cpu.dpush(a.wrapping_add(2)); }
        TWOMINUS => { let a = cpu.dpop(); cpu.dpush(a.wrapping_sub(2)); }
        HALVE => { let a = cpu.dpop() as i16; cpu.dpush((a >> 1) as u16); }
        MUL   => { let b = cpu.dpop() as i8 as i32; let a = cpu.dpop() as i16 as i32;
                   cpu.dpush((a * b) as u16); }
        DIV   => { let b = cpu.dpop() as i8 as i16; let a = cpu.dpop() as i16;
                   cpu.dpush(if b == 0 { 0 } else { (a / b) as u16 }); }
        DIVMOD=> { let b = cpu.dpop() as i8 as i16; let a = cpu.dpop() as i16;
                   if b == 0 { cpu.dpush(0); cpu.dpush(0); } else {
                   cpu.dpush((a / b) as u16); cpu.dpush((a % b) as u16); } }
        DMOD  => { let d = cpu.dpop() as i8 as i32; let h = cpu.dpop() as i8 as i32;
                   let l = cpu.dpop() as u16 as i32; let n = (h << 16) | l;
                   if d == 0 { cpu.dpush(0); cpu.dpush(0); } else {
                   cpu.dpush((n / d) as u16); cpu.dpush((n.abs() % d.abs()) as u16); } }
        MOD_P => { let b = cpu.dpop() as i8 as i16; let a = cpu.dpop() as i16;
                   cpu.dpush(if b == 0 { 0 } else { (a % b) as u16 }); }

        LT    => { let b = cpu.dpop(); let a = cpu.dpop();
                   cpu.dpush(if (a as i16) < (b as i16) { 1 } else { 0 }); }
        EQ    => { let b = cpu.dpop(); let a = cpu.dpop();
                   cpu.dpush(if a == b { 1 } else { 0 }); }
        GT    => { let b = cpu.dpop(); let a = cpu.dpop();
                   cpu.dpush(if (a as i16) > (b as i16) { 1 } else { 0 }); }
        ZEROEQ=> { let a = cpu.dpop(); cpu.dpush(if a == 0 { 1 } else { 0 }); }
        ZEROL => { let a = cpu.dpop(); cpu.dpush(if a as i16 >= 0 { 0 } else { 1 }); }
        AND   => { let b = cpu.dpop(); let a = cpu.dpop(); cpu.dpush(a & b); }
        IOR   => { let b = cpu.dpop(); let a = cpu.dpop(); cpu.dpush(a | b); }
        XOR   => { let b = cpu.dpop(); let a = cpu.dpop(); cpu.dpush(a ^ b); }
        NOT   => { let a = cpu.dpop(); cpu.dpush(if a == 0 { 1 } else { 0 }); }

        TYPE => {
            let addr = cpu.dpop();
            let len = cpu.read8(addr);
            for i in 1..=len as u16 { io.output.push(cpu.read8(addr + i)); }
        }
        ECHO  => { io.output.push(cpu.dpop() as u8); }
        KEY   => {
            if let Some(ch) = io.input.pop_front() {
                cpu.dpush(ch as u16);
            } else {
                io.waiting_for_key = true;
                // Rewind: the caller will re-attempt this trap next time
                // PC is still at the trap address, so it will fire again.
                return true; // don't advance to NEXT
            }
        }
        SPACE   => { io.output.push(0x20); }
        DISPLAY => {
            loop { let v = cpu.dpop() as u8;
                   io.output.push(v & 0x7F);
                   if v & 0x80 != 0 { break; } }
        }
        SIGN => {
            let sign = cpu.mem[cpu.ix as usize];
            if sign & 0x80 != 0 { cpu.dpush(0x2D); } // '-'
        }
        LHASH => { // <#
            let n = cpu.dpop();
            cpu.dpush(0x00A0); // terminator (space | 0x80)
            cpu.dpush(n);
            cpu.rpush_byte((n >> 8) as u8);
        }
        HASHR => { // #>
            cpu.rpop_byte();
            loop { let v = cpu.dpop() as u8;
                   io.output.push(v & 0x7F);
                   if v & 0x80 != 0 { break; } }
        }
        ASCII_CONV => {
            let n = cpu.dpop() as u8;
            cpu.dpush(if n < 10 { b'0' + n } else { b'A' + n - 10 } as u16);
        }

        TO_R     => { let v = cpu.dpop(); cpu.rpush(v); }
        R_FROM   => { let v = cpu.rpop(); cpu.dpush(v); }
        I_FETCH  => { let v = cpu.read16(cpu.ix); cpu.dpush(v); }
        J_FETCH  => { let v = cpu.read16(cpu.ix.wrapping_add(4)); cpu.dpush(v); }
        CI_FETCH => { cpu.dpush(cpu.mem[cpu.ix as usize] as u16); }
        CJ_FETCH => { cpu.dpush(cpu.mem[cpu.ix.wrapping_add(2) as usize] as u16); }
        C_TO_R   => { let v = cpu.dpop() as u8; cpu.rpush_byte(v); }
        CR_FROM  => { let v = cpu.rpop_byte(); cpu.dpush(v as u16); }

        PLUSSP  => { let a = cpu.dpop(); cpu.dpush(a.wrapping_add(cpu.sp)); }
        MINUSSP => { let a = cpu.dpop(); cpu.dpush(a.wrapping_sub(cpu.sp)); }
        QRS     => { cpu.dpush(cpu.ix); }
        QSP     => { cpu.dpush(cpu.sp); }
        SINGLE  => { let a = cpu.dpop() as i16;
                     cpu.dpush(if a >= -128 && a <= 127 { 1 } else { 0 }); }
        HEX     => { cpu.write8(rom::SYS + rom::SYS_BASE as u16, 16); }
        DECIMAL => { cpu.write8(rom::SYS + rom::SYS_BASE as u16, 10); }
        OCTAL   => { cpu.write8(rom::SYS + rom::SYS_BASE as u16, 8); }
        BINARY  => { cpu.write8(rom::SYS + rom::SYS_BASE as u16, 2); }
        ABORT_P => { cpu.pc = cpu.read16(0x0002); return true; }
        COSET   => { let ad = cpu.dpop(); cpu.write8(ad, 0); }
        C1SET   => { let ad = cpu.dpop(); cpu.write8(ad, 1); }
        DEFINITIONS => {
            let c = cpu.read16(rom::SYS + rom::SYS_CONTEXT as u16);
            cpu.write16(rom::SYS + rom::SYS_CURRENT as u16, c);
        }
        MOVE_P => {
            let dest = cpu.dpop(); let end = cpu.dpop(); let start = cpu.dpop();
            if end >= start {
                let n = (end - start + 1) as usize;
                if dest <= start {
                    for i in 0..n { let v = cpu.read8(start + i as u16); cpu.write8(dest + i as u16, v); }
                } else {
                    for i in (0..n).rev() { let v = cpu.read8(start + i as u16); cpu.write8(dest + i as u16, v); }
                }
            }
        }

        // ---- Branch primitives ----
        // *IF and *END are identical: conditional branch if TOS is FALSE (0)
        STAR_IF => {
            let flag = cpu.dpop();
            if flag == 0 { do_branch(cpu); } else { skip_offset(cpu); }
        }
        // *WHILE and *ELSE are identical: unconditional branch
        STAR_BR => { do_branch(cpu); }

        STAR_DO => { let s = cpu.dpop(); let t = cpu.dpop(); cpu.rpush(t); cpu.rpush(s); }
        STAR_CDO => { let s = cpu.dpop() as u8; let t = cpu.dpop() as u8;
                      cpu.rpush_byte(t); cpu.rpush_byte(s); }
        STAR_LOOP => {
            let idx = cpu.read16(cpu.ix).wrapping_add(1);
            cpu.write16(cpu.ix, idx);
            let term = cpu.read16(cpu.ix.wrapping_add(2));
            if (idx as i16) < (term as i16) { do_branch(cpu); }
            else { skip_offset(cpu); cpu.ix = cpu.ix.wrapping_add(4); }
        }
        STAR_CLOOP => {
            let idx = cpu.mem[cpu.ix as usize].wrapping_add(1);
            cpu.mem[cpu.ix as usize] = idx;
            let term = cpu.mem[cpu.ix.wrapping_add(1) as usize];
            if idx < term { do_branch(cpu); }
            else { skip_offset(cpu); cpu.ix = cpu.ix.wrapping_add(2); }
        }
        STAR_PLOOP => {
            let inc = cpu.dpop() as i16;
            let idx = (cpu.read16(cpu.ix) as i16).wrapping_add(inc) as u16;
            cpu.write16(cpu.ix, idx);
            let term = cpu.read16(cpu.ix.wrapping_add(2));
            if (idx as i16) < (term as i16) { do_branch(cpu); }
            else { skip_offset(cpu); cpu.ix = cpu.ix.wrapping_add(4); }
        }
        STAR_LEAVE => {
            let t = cpu.read16(cpu.ix.wrapping_add(2));
            cpu.write16(cpu.ix, t);
        }

        // ---- Literal handlers ----
        STAR_NUM  => { let v = cpu.ir_fetch16(); cpu.dpush(v); }
        STAR_CNUM => { let v = cpu.ir_fetch8() as i8 as i16 as u16; cpu.dpush(v); }
        STAR_BRAK => { let n = cpu.ir_fetch8();
                       for _ in 0..n { io.output.push(cpu.ir_fetch8()); } }
        STAR_SYS  => { let de = cpu.de();
                       let off = cpu.read8(de);
                       cpu.dpush(rom::SYS + off as u16); }
        STAR_STACK=> { if cpu.sp > rom::STACK {
                       io.output.extend_from_slice(b"\nSTACK UNDERFLOW\n");
                       cpu.sp = rom::STACK; } }

        // ---- Outer interpreter components (native) ----
        QUESTION => {
            let dp = cpu.read16(rom::SYS + rom::SYS_DP as u16);
            let ch2 = cpu.read8(dp + 1);
            if ch2 & 0x80 != 0 {
                // Terminator — push OK message
                cpu.dpush(cpu.read16(0x0004));
            } else {
                // Unknown token — echo it + "?"
                let len = cpu.read8(dp);
                io.output.push(b'\n');
                for i in 1..=len as u16 { io.output.push(cpu.read8(dp + i)); }
                io.output.extend_from_slice(b" ?\n");
                // Restart outer interpreter
                cpu.sp = rom::STACK;
                cpu.dpush(cpu.read16(0x0006)); // restart msg
                cpu.set_bc(cpu.read16(0x000A)); // outer interpreter addr
                // Fall through to NEXT
            }
        }
        DOLLAR_PATCH => {
            cpu.write8(rom::SYS + rom::SYS_MODE as u16, 0);
            cpu.sp = rom::STACK;
            cpu.dpush(cpu.read16(0x0006)); // restart msg
            cpu.set_bc(cpu.read16(0x000A));
        }
        INLINE => {
            io.output.push(0x0D); io.output.push(0x0A);
            for i in 0..rom::LENGTH as u16 { cpu.write8(rom::LBADD + i, 0x20); }
            cpu.write16(rom::LBEND, 0x8080);
            cpu.write16(rom::SYS + rom::SYS_LBP as u16, rom::LBADD);
            let mut pos: u8 = 0;
            loop {
                let ch = if let Some(c) = io.input.pop_front() { c } else {
                    io.waiting_for_key = true;
                    // Can't block — for now, inject CR
                    0x0D
                };
                if ch == 0x0D {
                    io.output.push(0x20);
                    break;
                } else if ch == 0x08 && pos > 0 {
                    pos -= 1;
                    cpu.write8(rom::LBADD + pos as u16, 0x20);
                    io.output.extend_from_slice(&[0x08, 0x20, 0x08]);
                } else if pos < rom::LENGTH {
                    let uc = if ch >= b'a' && ch <= b'z' { ch - 32 } else { ch };
                    cpu.write8(rom::LBADD + pos as u16, uc);
                    io.output.push(uc);
                    pos += 1;
                }
            }
            cpu.write16(rom::SYS + rom::SYS_LBP as u16, rom::LBADD);
        }
        TOKEN => {
            let sep = cpu.dpop() as u8;
            let mut lbp = cpu.read16(rom::SYS + rom::SYS_LBP as u16);
            let dp = cpu.read16(rom::SYS + rom::SYS_DP as u16);
            // Skip leading separators (relies on terminator 0x80 ≠ 0x20 to stop)
            if sep == 0x20 {
                while cpu.read8(lbp) == 0x20 { lbp += 1; }
            }
            // Copy token characters until separator or terminator (bit 7)
            let mut count: u8 = 0;
            loop {
                let ch = cpu.read8(lbp);
                if ch == sep || ch & 0x80 != 0 { break; }
                count += 1;
                cpu.write8(dp + count as u16, ch);
                lbp += 1;
            }
            cpu.write8(dp, count);
            // Copy the stopping character to dp+1 so QUESTION can see
            // the terminator sentinel (bit 7 set) for end-of-line detection
            if count == 0 {
                cpu.write8(dp + 1, cpu.read8(lbp));
            }
            // Advance past separator (but not past terminator)
            if cpu.read8(lbp) == sep { lbp += 1; }
            cpu.write16(rom::SYS + rom::SYS_LBP as u16, lbp);
        }
        SEARCH_P => {
            let mut hdr = cpu.dpop();
            let dp = cpu.read16(rom::SYS + rom::SYS_DP as u16);
            let tok_len = cpu.read8(dp) & 0x7F;
            let max_cmp = std::cmp::min(tok_len, 3) as u16;
            while hdr != 0 {
                let hdr_len = cpu.read8(hdr) & 0x7F;
                if hdr_len == tok_len {
                    let mut ok = true;
                    for i in 0..max_cmp {
                        if cpu.read8(dp + 1 + i) != cpu.read8(hdr + 1 + i) { ok = false; break; }
                    }
                    if ok {
                        cpu.dpush(hdr + 6); // word address
                        cpu.dpush(0);       // FALSE = found
                        cpu.pc = cpu.iy; return true;
                    }
                }
                hdr = cpu.read16(hdr + 4);
            }
            cpu.dpush(1); // TRUE = not found
        }
        NUMBER => {
            let dp = cpu.read16(rom::SYS + rom::SYS_DP as u16);
            let len = cpu.read8(dp) as usize;
            let base = cpu.read8(rom::SYS + rom::SYS_BASE as u16) as u32;
            if len == 0 || base < 2 {
                cpu.dpush(0); cpu.pc = cpu.iy; return true;
            }
            let mut neg = false;
            let mut start = 1usize;
            if cpu.read8(dp + 1) == b'-' { neg = true; start = 2; }
            if start > len { cpu.dpush(0); cpu.pc = cpu.iy; return true; }
            let mut result: i32 = 0;
            for i in start..=len {
                let ch = cpu.read8(dp + i as u16);
                let d = if ch >= b'0' && ch <= b'9' { (ch - b'0') as u32 }
                        else if ch >= b'A' && ch <= b'Z' { (ch - b'A' + 10) as u32 }
                        else { cpu.dpush(0); cpu.pc = cpu.iy; return true; };
                if d >= base { cpu.dpush(0); cpu.pc = cpu.iy; return true; }
                result = result * base as i32 + d as i32;
            }
            if neg { result = -result; }
            cpu.dpush(0);               // FALSE = success
            cpu.dpush(result as u16);   // the number
        }

        // ---- ?SEARCH, ?NUMBER, ?EXECUTE (native outer-interp secondaries) ----
        QSEARCH => {
            let dp = cpu.read16(rom::SYS + rom::SYS_DP as u16);
            let tok_len = cpu.read8(dp) & 0x7F;
            let max_cmp = std::cmp::min(tok_len, 3) as u16;

            // Search CONTEXT vocabulary
            let ctx_ptr = cpu.read16(rom::SYS + rom::SYS_CONTEXT as u16);
            let ctx_head = cpu.read16(ctx_ptr);
            if let Some(wa) = search_vocab(cpu, ctx_head, dp, tok_len, max_cmp) {
                cpu.dpush(wa);
                cpu.dpush(0); // FALSE = found
                cpu.pc = cpu.iy; return true;
            }

            // Not found: check MODE
            let mode = cpu.read8(rom::SYS + rom::SYS_MODE as u16);
            if mode == 0 {
                // Execute mode: not found
                cpu.dpush(1); // TRUE
                cpu.pc = cpu.iy; return true;
            }

            // Compile mode: search COMPILER vocabulary
            let comp_ptr = cpu.read16(rom::SYS + rom::SYS_COMPILER as u16);
            let comp_head = cpu.read16(comp_ptr);
            if let Some(wa) = search_vocab(cpu, comp_head, dp, tok_len, max_cmp) {
                cpu.dpush(wa);
                cpu.dpush(0); // FALSE = found
                cpu.write8(rom::SYS + rom::SYS_STATE as u16, 1); // STATE = 1
                cpu.pc = cpu.iy; return true;
            }

            cpu.dpush(1); // TRUE = not found
        }

        QNUMBER => {
            let dp = cpu.read16(rom::SYS + rom::SYS_DP as u16);
            let len = cpu.read8(dp) as usize;
            let base = cpu.read8(rom::SYS + rom::SYS_BASE as u16) as u32;

            if len == 0 || base < 2 {
                cpu.dpush(1); // TRUE = not a number (terminator or invalid)
                cpu.pc = cpu.iy; return true;
            }

            // Check for terminator (bit 7 set on char 2)
            if len >= 1 && cpu.read8(dp + 1) & 0x80 != 0 {
                cpu.dpush(1); // TRUE = terminator
                cpu.pc = cpu.iy; return true;
            }

            let mut neg = false;
            let mut start = 1usize;
            if cpu.read8(dp + 1) == b'-' { neg = true; start = 2; }
            if start > len { cpu.dpush(1); cpu.pc = cpu.iy; return true; }

            let mut result: i32 = 0;
            for i in start..=len {
                let ch = cpu.read8(dp + i as u16);
                let d = if ch >= b'0' && ch <= b'9' { (ch - b'0') as u32 }
                        else if ch >= b'A' && ch <= b'Z' { (ch - b'A' + 10) as u32 }
                        else { cpu.dpush(1); cpu.pc = cpu.iy; return true; };
                if d >= base { cpu.dpush(1); cpu.pc = cpu.iy; return true; }
                result = result * base as i32 + d as i32;
            }
            if neg { result = -result; }
            let n = result as u16;

            let mode = cpu.read8(rom::SYS + rom::SYS_MODE as u16);
            if mode != 0 {
                // Compile mode: enclose literal handler + number in dictionary
                let is_byte = result >= -128 && result <= 127;
                if is_byte {
                    enclose_word(cpu, cpu.read16(0x0010)); // *C# word address
                    enclose_byte(cpu, n as u8);
                } else {
                    enclose_word(cpu, cpu.read16(0x000E)); // *# word address
                    enclose_word(cpu, n);
                }
                cpu.dpush(0);  // FALSE = success (no number on stack in compile mode)
            } else {
                // Execute mode: push number, then flag on top
                cpu.dpush(n);  // the number (below flag)
                cpu.dpush(0);  // FALSE = success (TOS — tested by *END)
            }
        }

        QEXECUTE => {
            let wa = cpu.dpop();  // word address from ?SEARCH
            let state = cpu.read8(rom::SYS + rom::SYS_STATE as u16);
            let mode = cpu.read8(rom::SYS + rom::SYS_MODE as u16);
            cpu.write8(rom::SYS + rom::SYS_STATE as u16, 0); // clear STATE
            if state == mode {
                // Execute the word
                cpu.dpush(wa);
                // Simulate EXECUTE: pop WA to HL, jump to RUN
                let hl = cpu.dpop();
                cpu.set_hl(hl);
                cpu.pc = 0x0112; // RUN address
                return true; // don't go to NEXT — go to RUN
            } else {
                // Compile: enclose word address in dictionary
                enclose_word(cpu, wa);
            }
        }

        CONST_CODE => { let de = cpu.de(); cpu.dpush(cpu.read16(de)); }
        CCONST_CODE => { let de = cpu.de(); cpu.dpush(cpu.read8(de) as i8 as i16 as u16); }
        VAR_CODE => { cpu.dpush(cpu.de()); }

        _ => { eprintln!("Unknown trap {:04X}", trap); return false; }
    }

    cpu.pc = cpu.iy; // JP (IY) → NEXT
    true
}

fn search_vocab(cpu: &Cpu, mut hdr: u16, dp: u16, tok_len: u8, max_cmp: u16) -> Option<u16> {
    while hdr != 0 {
        let hdr_len = cpu.read8(hdr) & 0x7F;
        if hdr_len == tok_len {
            let mut ok = true;
            for i in 0..max_cmp {
                if cpu.read8(dp + 1 + i) != cpu.read8(hdr + 1 + i) { ok = false; break; }
            }
            if ok { return Some(hdr + 6); }
        }
        hdr = cpu.read16(hdr + 4);
    }
    None
}

fn enclose_word(cpu: &mut Cpu, val: u16) {
    let dp = cpu.read16(rom::SYS + rom::SYS_DP as u16);
    cpu.write16(dp, val);
    cpu.write16(rom::SYS + rom::SYS_DP as u16, dp + 2);
}

fn enclose_byte(cpu: &mut Cpu, val: u8) {
    let dp = cpu.read16(rom::SYS + rom::SYS_DP as u16);
    cpu.write8(dp, val);
    cpu.write16(rom::SYS + rom::SYS_DP as u16, dp + 1);
}
