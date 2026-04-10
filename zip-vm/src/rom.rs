use crate::asm::Asm;
use crate::native as n;

pub const STACK: u16 = 0xFC00;
pub const RETURN: u16 = 0xFA00;
pub const SYS: u16 = 0xF882; // After LBEND terminators at F880-F881
pub const LBADD: u16 = 0xF800;
pub const LENGTH: u8 = 0x80;
pub const LBEND: u16 = 0xF880;

pub const SYS_BASE: u8 = 0x00;
pub const SYS_MODE: u8 = 0x01;
pub const SYS_STATE: u8 = 0x02;
pub const SYS_DP: u8 = 0x04;
pub const SYS_LBP: u8 = 0x06;
pub const SYS_CURRENT: u8 = 0x08;
pub const SYS_CONTEXT: u8 = 0x0A;
pub const SYS_COMPILER: u8 = 0x0C;

pub fn build() -> Asm {
    let mut a = Asm::new(0x0100);
    let mut prev: Option<&'static str> = None;

    // ======== INNER INTERPRETER (real Z80, Table 3.3) ========

    a.label("SEMI");
    a.dw_val(0x0102);
    a.emit(&[0xDD,0x4E,0x00, 0xDD,0x23, 0xDD,0x46,0x00, 0xDD,0x23]);

    a.label("NEXT");
    a.emit(&[0x0A, 0x6F, 0x03, 0x0A, 0x67, 0x03]);

    a.label("RUN");
    a.emit(&[0x5E, 0x23, 0x56, 0x23, 0xEB, 0xE9]);

    a.label("COLON");
    a.emit(&[0xDD,0x2B, 0xDD,0x70,0x00, 0xDD,0x2B, 0xDD,0x71,0x00,
             0x4B, 0x42, 0xFD,0xE9]);

    // ======== EXECUTE (headered, real Z80) ========

    a.label("hdr_EXECUTE");
    a.header("EXECUTE", None, false);
    a.label("EXECUTE");
    a.emit_code_addr();
    a.emit(&[0xE1]); // POP HL
    a.jr("RUN");
    prev = Some("hdr_EXECUTE");

    // ======== START/RESTART ========

    a.label("START");
    a.emit(&[0x11]); a.dw_label("RSTMSG");
    a.emit(&[0x3A]); a.dw_val(SYS + SYS_BASE as u16);
    a.emit(&[0xA7]);
    a.jr_nz("ABORT_ENTRY");
    a.emit(&[0x3E, 0x10]);
    a.emit(&[0x32]); a.dw_val(SYS + SYS_BASE as u16);
    a.emit(&[0x11]); a.dw_label("SRTMSG");
    a.label("ABORT_ENTRY");
    a.emit(&[0x31]); a.dw_val(STACK);
    a.emit(&[0xD5]);
    a.emit(&[0x21, 0x00, 0x00]);
    a.emit(&[0x22]); a.dw_val(SYS + SYS_MODE as u16);
    a.emit(&[0xFD, 0x21]); a.dw_label("NEXT");
    a.emit(&[0xDD, 0x21]); a.dw_val(RETURN);
    a.emit(&[0x21, 0x80, 0x80]);
    a.emit(&[0x22]); a.dw_val(LBEND);
    a.emit(&[0x01]); a.dw_label("OUTER");
    a.jp("NEXT");

    // ======== NATIVE PRIMITIVES (trap word entries) ========

    macro_rules! prim {
        ($name:expr, $trap:expr) => {{
            let lbl: &'static str = Box::leak(format!("hdr_{}", $name).into_boxed_str());
            a.label(lbl);
            a.header($name, prev, false);
            a.label($name);
            a.dw_val($trap);
            prev = Some(lbl);
        }};
    }
    macro_rules! prim_i {
        ($name:expr, $trap:expr) => {{
            let lbl: &'static str = Box::leak(format!("hdr_{}", $name).into_boxed_str());
            a.label(lbl);
            a.header($name, prev, true);
            a.label($name);
            a.dw_val($trap);
            prev = Some(lbl);
        }};
    }

    prim!("DROP", n::DROP); prim!("DUP", n::DUP); prim!("2DUP", n::DUP2);
    prim!("SWAP", n::SWAP); prim!("OVER", n::OVER);
    prim!("LROT", n::LROT); prim!("RROT", n::RROT);

    prim!("!", n::STORE); prim!("@", n::FETCH);
    prim!("C!", n::CSTORE); prim!("C@", n::CFETCH);
    prim!("+!", n::PSTORE); prim!("C+!", n::CPSTORE);
    prim!(",", n::COMMA); prim!("C,", n::CCOMMA);
    prim!("0!", n::ZEROSTORE); prim!("1!", n::ONESTORE);

    prim!("+", n::ADD); prim!("-", n::SUB);
    prim!("MINUS", n::MINUS); prim!("ABS", n::ABS);
    prim!("2*", n::DOUBLE); prim!("2+", n::TWOPLUS);
    prim!("2-", n::TWOMINUS); prim!("2/", n::HALVE);
    prim!("*", n::MUL); prim!("/", n::DIV);
    prim!("/MOD", n::DIVMOD); prim!("D/MOD", n::DMOD);
    prim!("MOD", n::MOD_P);

    prim!("<", n::LT); prim!("=", n::EQ); prim!(">", n::GT);
    prim!("0=", n::ZEROEQ); prim!("0<", n::ZEROL);
    prim!("NOT", n::NOT);
    prim!("AND", n::AND); prim!("IOR", n::IOR); prim!("XOR", n::XOR);

    prim!("TYPE", n::TYPE); prim!("ECHO", n::ECHO);
    prim!("KEY", n::KEY); prim!("SPACE", n::SPACE);
    prim!("DISPLAY", n::DISPLAY); prim!("SIGN", n::SIGN);
    prim!("<#", n::LHASH); prim!("#>", n::HASHR);
    prim!("ASCII", n::ASCII_CONV);

    prim!("<R", n::TO_R); prim!("R>", n::R_FROM);
    prim!("I>", n::I_FETCH); prim!("J>", n::J_FETCH);
    prim!("CI>", n::CI_FETCH); prim!("CJ>", n::CJ_FETCH);
    prim!("C<R", n::C_TO_R); prim!("CR>", n::CR_FROM);

    prim!("+SP", n::PLUSSP); prim!("-SP", n::MINUSSP);
    prim!("?RS", n::QRS); prim!("?SP", n::QSP);
    prim!("SINGLE", n::SINGLE);
    prim!("HEX", n::HEX); prim!("DECIMAL", n::DECIMAL);
    prim!("OCTAL", n::OCTAL); prim!("BINARY", n::BINARY);
    prim!("ABORT", n::ABORT_P);
    prim!("COSET", n::COSET);
    prim!("DEFINITIONS", n::DEFINITIONS);
    prim!("MOVE", n::MOVE_P);

    // ======== HEADERLESS WORD ENTRIES ========

    a.label("w_IF");    a.dw_val(n::STAR_IF);
    a.label("w_END");   a.dw_val(n::STAR_IF);    // same handler as *IF
    a.label("w_ELSE");  a.dw_val(n::STAR_BR);
    a.label("w_WHILE"); a.dw_val(n::STAR_BR);    // same handler as *ELSE
    a.label("w_DO");    a.dw_val(n::STAR_DO);
    a.label("w_CDO");   a.dw_val(n::STAR_CDO);
    a.label("w_LOOP");  a.dw_val(n::STAR_LOOP);
    a.label("w_CLOOP"); a.dw_val(n::STAR_CLOOP);
    a.label("w_PLOOP"); a.dw_val(n::STAR_PLOOP);
    a.label("w_LEAVE"); a.dw_val(n::STAR_LEAVE);
    a.label("w_NUM");   a.dw_val(n::STAR_NUM);
    a.label("w_CNUM");  a.dw_val(n::STAR_CNUM);
    a.label("w_BRAK");  a.dw_val(n::STAR_BRAK);
    a.label("w_SYS");   a.dw_val(n::STAR_SYS);
    a.label("w_STACK");    a.dw_val(n::STAR_STACK);
    a.label("w_QUESTION"); a.dw_val(n::QUESTION);
    a.label("w_PATCH");    a.dw_val(n::DOLLAR_PATCH);
    a.label("w_INLINE");   a.dw_val(n::INLINE);
    a.label("w_TOKEN");    a.dw_val(n::TOKEN);
    a.label("w_SEARCH");   a.dw_val(n::SEARCH_P);
    a.label("w_NUMBER");   a.dw_val(n::NUMBER);
    a.label("w_COSET");    a.dw_val(n::COSET);
    a.label("w_C1SET");    a.dw_val(n::C1SET);
    a.label("w_CONST");    a.dw_val(n::CONST_CODE);
    a.label("w_CCONST");   a.dw_val(n::CCONST_CODE);
    a.label("w_VAR");      a.dw_val(n::VAR_CODE);
    a.label("w_QSEARCH");  a.dw_val(n::QSEARCH);
    a.label("w_QNUMBER");  a.dw_val(n::QNUMBER);
    a.label("w_QEXECUTE"); a.dw_val(n::QEXECUTE);

    // ======== SYSTEM VARIABLES ========

    macro_rules! sysvar {
        ($name:expr, $off:expr) => {{
            let lbl: &'static str = Box::leak(format!("hdr_{}", $name).into_boxed_str());
            a.label(lbl);
            a.header($name, prev, false);
            a.label($name);
            a.dw_val(n::STAR_SYS);
            a.db($off);
            prev = Some(lbl);
        }};
    }

    sysvar!("BASE", SYS_BASE); sysvar!("MODE", SYS_MODE);
    sysvar!("STATE", SYS_STATE); sysvar!("DP", SYS_DP);
    sysvar!("LBP", SYS_LBP); sysvar!("CURRENT", SYS_CURRENT);
    sysvar!("CONTEXT", SYS_CONTEXT); sysvar!("COMPILER", SYS_COMPILER);

    // ======== CONSTANTS ========

    {
        let lbl: &'static str = "hdr_ASPACE";
        a.label(lbl); a.header("ASPACE", prev, false);
        a.label("ASPACE"); a.dw_val(n::CCONST_CODE); a.db(0x20);
        prev = Some(lbl);
    }
    {
        let lbl: &'static str = "hdr_ZERO";
        a.label(lbl); a.header("0", prev, false);
        a.label("ZERO"); a.dw_val(n::CONST_CODE); a.dw_val(0);
        prev = Some(lbl);
    }
    {
        let lbl: &'static str = "hdr_ONE";
        a.label(lbl); a.header("1", prev, false);
        a.label("ONE"); a.dw_val(n::CONST_CODE); a.dw_val(1);
        prev = Some(lbl);
    }

    // ======== SECONDARIES ========

    // HERE — DP @
    {
        let lbl: &'static str = "hdr_HERE";
        a.label(lbl); a.header("HERE", prev, false);
        a.label("HERE"); a.dw_label("COLON");
        a.dw_label("DP"); a.dw_label("@");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // ENTRY — CURRENT @ @
    {
        let lbl: &'static str = "hdr_ENTRY";
        a.label(lbl); a.header("ENTRY", prev, false);
        a.label("ENTRY"); a.dw_label("COLON");
        a.dw_label("CURRENT"); a.dw_label("@"); a.dw_label("@");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // . (period) — <# ABS #S SIGN #>
    {
        let lbl: &'static str = "hdr_DOT";
        a.label(lbl); a.header(".", prev, false);
        a.label("DOT"); a.dw_label("COLON");
        a.dw_label("<#"); a.dw_label("ABS");
        a.dw_label("#S"); a.dw_label("SIGN"); a.dw_label("#>");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // # — 0 BASE C@ D/MOD ASCII SWAP
    {
        let lbl: &'static str = "hdr_HASH";
        a.label(lbl); a.header("#", prev, false);
        a.label("HASH"); a.dw_label("COLON");
        a.dw_label("ZERO"); a.dw_label("BASE"); a.dw_label("C@");
        a.dw_label("D/MOD"); a.dw_label("ASCII"); a.dw_label("SWAP");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // #S — BEGIN # DUP 0= END DROP
    {
        let lbl: &'static str = "hdr_HASHS";
        a.label(lbl); a.header("#S", prev, false);
        a.label("#S"); a.dw_label("COLON");
        a.label("_hs_loop"); // target for backward branch
        a.dw_label("HASH"); a.dw_label("DUP"); a.dw_label("0=");
        a.dw_label("w_END");
        // offset: backward to _hs_loop
        // BC at offset byte, target = _hs_loop
        // offset = _hs_loop - offset_byte_addr
        a.jr_target("_hs_loop"); // reuse JR fixup — same math
        a.dw_label("DROP");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // ======== COMPILER DIRECTIVES ========

    // IF — *# w_IF DO, 0 C,
    {
        let lbl: &'static str = "hdr_IF";
        a.label(lbl); a.header("IF", prev, true);
        a.label("IF"); a.dw_label("COLON");
        a.dw_label("w_NUM"); a.dw_label("w_IF");
        a.dw_label("DO,"); a.dw_label("ZERO"); a.dw_label("C,");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // DO, — , HERE
    {
        let lbl: &'static str = "hdr_DO,";
        a.label(lbl); a.header("DO,", prev, false);
        a.label("DO,"); a.dw_label("COLON");
        a.dw_label(","); a.dw_label("HERE");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // END, — , HERE - C,
    {
        let lbl: &'static str = "hdr_END,";
        a.label(lbl); a.header("END,", prev, false);
        a.label("END,"); a.dw_label("COLON");
        a.dw_label(","); a.dw_label("HERE"); a.dw_label("-"); a.dw_label("C,");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // THEN — HERE OVER - SWAP C!
    {
        let lbl: &'static str = "hdr_THEN";
        a.label(lbl); a.header("THEN", prev, true);
        a.label("THEN"); a.dw_label("COLON");
        a.dw_label("HERE"); a.dw_label("OVER"); a.dw_label("-");
        a.dw_label("SWAP"); a.dw_label("C!");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // : — CURRENT @ CONTEXT ! CREATE *# COLON , MODE C1SET
    {
        let lbl: &'static str = "hdr_COLON_DEF";
        a.label(lbl); a.header(":", prev, false);
        a.label("COLON_DEF"); a.dw_label("COLON");
        a.dw_label("CURRENT"); a.dw_label("@");
        a.dw_label("CONTEXT"); a.dw_label("!");
        a.dw_label("CREATE");
        a.dw_label("w_NUM"); a.dw_label("COLON");
        a.dw_label(",");
        a.dw_label("MODE"); a.dw_label("w_C1SET");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // ; — *# SEMI , MODE COSET
    {
        let lbl: &'static str = "hdr_SEMI_DEF";
        a.label(lbl); a.header(";", prev, true);
        a.label("SEMI_DEF"); a.dw_label("COLON");
        a.dw_label("w_NUM"); a.dw_label("SEMI");
        a.dw_label(",");
        a.dw_label("MODE"); a.dw_label("w_COSET");
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    // CREATE — ENTRY ASPACE TOKEN HERE CURRENT @ ! *C# 4 DP +! , HERE 2+ ,
    {
        let lbl: &'static str = "hdr_CREATE";
        a.label(lbl); a.header("CREATE", prev, false);
        a.label("CREATE"); a.dw_label("COLON");
        a.dw_label("ENTRY");
        a.dw_label("ASPACE"); a.dw_label("w_TOKEN");
        a.dw_label("HERE");
        a.dw_label("CURRENT"); a.dw_label("@"); a.dw_label("!");
        a.dw_label("w_CNUM"); a.db(4); // advance by 4 (len+3 chars)
        a.dw_label("DP"); a.dw_label("+!");
        a.dw_label(","); // enclose link
        a.dw_label("HERE"); a.dw_label("2+"); a.dw_label(","); // enclose code addr
        a.dw_label("SEMI");
        prev = Some(lbl);
    }

    let _last_header = prev.unwrap();

    // ======== OUTER INTERPRETER (threaded code, exact offsets from book) ========

    a.label("OUTER");
    a.dw_label("TYPE");       // 0-1
    a.dw_label("w_INLINE");   // 2-3
    a.dw_label("ASPACE");     // 4-5
    a.dw_label("w_TOKEN");    // 6-7
    a.dw_label("w_QSEARCH");  // 8-9
    a.dw_label("w_IF"); a.db(0x0B);   // 10-12: if FALSE(found) → byte 23
    a.dw_label("w_QNUMBER");  // 13-14
    a.dw_label("w_END"); a.db(0xF3u8 as u8); // 15-17: if FALSE(number) → byte 4
    a.dw_label("w_QUESTION"); // 18-19
    a.dw_label("w_WHILE"); a.db(0xEAu8 as u8); // 20-22: → byte 0 (TYPE)
    a.dw_label("w_QEXECUTE"); // 23-24
    a.dw_label("w_WHILE"); a.db(0xE9u8 as u8); // 25-27: → byte 4 (ASPACE)

    // ======== MESSAGES ========

    a.label("SRTMSG");
    let m = b"ZIP TIL"; a.db(m.len() as u8); a.emit(m);
    a.label("RSTMSG");
    let m = b"OK"; a.db(m.len() as u8); a.emit(m);
    a.label("OKMSG");
    let m = b" OK"; a.db(m.len() as u8); a.emit(m);
    a.label("QMSG");
    let m = b"?"; a.db(m.len() as u8); a.emit(m);

    a.label("rom_end");
    a.resolve();

    // Store lookup addresses at fixed low memory (0x0000-0x001F).
    // Native handlers use these to find messages and entry points.
    // We'll patch them into CPU memory after loading the ROM.

    a
}

/// Load ROM into CPU memory and set up fixed-address lookup table.
pub fn load(cpu: &mut crate::cpu::Cpu) {
    let asm = build();
    asm.load_into(&mut cpu.mem);

    // Fixed lookup table at 0x0000-0x001F
    cpu.write16(0x0002, asm.addr("START"));
    cpu.write16(0x0004, asm.addr("OKMSG"));
    cpu.write16(0x0006, asm.addr("RSTMSG"));
    cpu.write16(0x0008, asm.addr("w_PATCH"));
    cpu.write16(0x000A, asm.addr("OUTER"));
    cpu.write16(0x000C, asm.addr("NEXT"));
    cpu.write16(0x000E, asm.addr("w_NUM"));   // *# word address
    cpu.write16(0x0010, asm.addr("w_CNUM"));  // *C# word address

    // Initialize SYS block (DP, CURRENT, CONTEXT, COMPILER)
    let rom_end = asm.addr("rom_end");
    cpu.write16(SYS + SYS_DP as u16, rom_end);

    // Last header = head of the main vocabulary
    // Find it from the `prev` chain — it's the last header we emitted.
    // We stored it as the label of the last headered word.
    // For now, search for the pattern: the highest header address.
    // Actually, we can just use the label directly.
    let last_hdr = asm.addr("hdr_CREATE"); // last headered word we emitted

    // CURRENT and CONTEXT point to a vocabulary variable whose value
    // is the address of the latest header.
    // For simplicity, put the vocabulary variable at a fixed address.
    let vocab_addr: u16 = 0x0020;
    cpu.write16(vocab_addr, last_hdr); // vocab head → CREATE header
    cpu.write16(SYS + SYS_CURRENT as u16, vocab_addr);
    cpu.write16(SYS + SYS_CONTEXT as u16, vocab_addr);
    cpu.write16(SYS + SYS_COMPILER as u16, vocab_addr); // same for now

    // Set up CPU state for boot
    cpu.pc = asm.addr("START");
    cpu.sp = STACK;
    cpu.ix = RETURN;
    cpu.iy = asm.addr("NEXT");
}
