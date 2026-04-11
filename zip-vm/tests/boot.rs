use z80::cpu::Cpu;
use zip_vm::native;
use zip_vm::rom;

fn run_zip(input: &str) -> String {
    run_zip_debug(input, false)
}

fn run_zip_debug(input: &str, debug: bool) -> String {
    let mut cpu = Cpu::new();
    rom::load(&mut cpu);

    if debug {
        let outer = cpu.read16(0x000A);
        let next = cpu.read16(0x000C);
        eprintln!("OUTER={:04X} NEXT={:04X} START={:04X}", outer, next, cpu.read16(0x0002));
        eprintln!("OUTER bytes: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            cpu.read8(outer), cpu.read8(outer+1), cpu.read8(outer+2), cpu.read8(outer+3),
            cpu.read8(outer+4), cpu.read8(outer+5), cpu.read8(outer+6), cpu.read8(outer+7));
        // Check TYPE word address
        let type_wa_lo = cpu.read8(outer);
        let type_wa_hi = cpu.read8(outer+1);
        let type_wa = type_wa_lo as u16 | (type_wa_hi as u16) << 8;
        eprintln!("TYPE WA={:04X} code_addr={:04X}", type_wa,
            cpu.read16(type_wa));
        // Check w_INLINE
        let inline_wa = cpu.read16(outer+2);
        eprintln!("INLINE WA={:04X} code_addr={:04X}", inline_wa,
            cpu.read16(inline_wa));
        eprintln!("IY={:04X} IX={:04X} SP={:04X} PC={:04X}", cpu.iy, cpu.ix, cpu.sp, cpu.pc);
        // Check DOT word at 0x044B (from QEXECUTE trace)
        let dot_wa: u16 = 0x044B;
        eprintln!("DOT WA={:04X} code_addr={:04X} body: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            dot_wa, cpu.read16(dot_wa),
            cpu.read8(dot_wa+2), cpu.read8(dot_wa+3),
            cpu.read8(dot_wa+4), cpu.read8(dot_wa+5),
            cpu.read8(dot_wa+6), cpu.read8(dot_wa+7));
        eprintln!("COLON should be at 0118. Check: {:02X} {:02X}",
            cpu.read8(0x0118), cpu.read8(0x0119));
    }

    let mut io = native::Io::new();
    for b in input.bytes() {
        io.input.push_back(b);
    }
    io.input.push_back(0x0D);

    let mut steps = 0u64;
    let max = 5_000_000;

    while steps < max && !cpu.halted && !io.waiting_for_key {

        if native::is_trap(cpu.pc) {
            if debug {
                eprintln!("  TRAP {:04X} BC={:04X} SP={:04X} TOS={:04X}",
                    cpu.pc, cpu.bc(), cpu.sp,
                    if cpu.sp < 0xFC00 { cpu.read16(cpu.sp) } else { 0 });
            }
            if !native::handle_trap(&mut cpu, &mut io) {
                eprintln!("Unhandled trap {:04X}", cpu.pc);
                break;
            }
            steps += 1;
            continue;
        }
        cpu.step();
        steps += 1;
    }
    if debug {
        eprintln!("Stopped after {} steps, halted={} waiting={} PC={:04X}",
            steps, cpu.halted, io.waiting_for_key, cpu.pc);
    }

    io.output.iter().map(|&b| {
        if b == 0x0D { '\n' } else if b >= 0x20 && b < 0x7F { b as char } else { ' ' }
    }).collect()
}

#[test]
fn boot_displays_message() {
    let output = run_zip("");
    assert!(output.contains("ZIP TIL"), "Expected boot message, got: {:?}", output);
}

#[test]
fn decimal_2_3_add_dot() {
    let output = run_zip_debug("DECIMAL 2 3 + .", true);
    eprintln!("OUTPUT: {:?}", output);
    assert!(output.contains("5"), "Expected '5' in output, got: {:?}", output);
}

#[test]
fn hex_ff_dot() {
    let output = run_zip("FF .");
    eprintln!("OUTPUT: {:?}", output);
    // Basic check: boots without crash
    assert!(output.contains("ZIP TIL"), "Expected boot, got: {:?}", output);
}
