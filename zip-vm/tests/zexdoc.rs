use zip_vm::cpu::Cpu;

const ZEXDOC: &[u8] = include_bytes!("data/zexdoc.com");

/// Minimal CP/M stub for ZEXDOC:
/// - Load .COM at 0100h
/// - BDOS call at 0005h: C=2 (print char E), C=9 (print $ string at DE)
/// - RST 0 (opcode C7 at 0000h) = warm boot = exit
fn run_zexdoc() -> (String, bool) {
    let mut cpu = Cpu::new();

    // Load COM file at 0100h
    cpu.mem[0x0100..0x0100 + ZEXDOC.len()].copy_from_slice(ZEXDOC);

    // CP/M stub setup:
    // 0000h: JP 0 (warm boot — we intercept at HALT or BDOS C=0)
    cpu.mem[0x0000] = 0x76; // HALT at warm boot
    // 0005h: RET (BDOS entry — we intercept PC=0005 before executing)
    cpu.mem[0x0005] = 0xC9;
    // 0006h-0007h: BDOS address = top of TPA. Programs do LD HL,(6); LD SP,HL
    // to set their stack. Must be nonzero!
    cpu.write16(0x0006, 0xF000);

    cpu.pc = 0x0100;
    cpu.sp = 0xF000;

    let mut output = String::new();
    let mut has_error = false;
    let mut bdos_calls = 0u64;
    let max_cycles: u64 = 500_000_000_000;

    loop {
        if cpu.halted {
            break;
        }
        if cpu.cycles > max_cycles {
            output.push_str("\n*** TIMEOUT ***\n");
            has_error = true;
            break;
        }

        // Intercept BDOS call at 0005h
        if cpu.pc == 0x0005 {
            bdos_calls += 1;
            match cpu.c {
                0 => { break; } // Warm boot = exit
                1 => { cpu.a = 0; } // Console input: return 0 (no input)
                2 => {
                    let ch = cpu.e;
                    if ch == 0x0A { output.push('\n'); }
                    else if ch != 0x0D { output.push(ch as char); }
                }
                9 => {
                    let mut addr = cpu.de();
                    let mut count = 0;
                    loop {
                        let ch = cpu.read8(addr);
                        if ch == b'$' || count > 500 { break; }
                        if ch == 0x0A { output.push('\n'); }
                        else if ch != 0x0D { output.push(ch as char); }
                        addr += 1;
                        count += 1;
                    }
                }
                _ => {}
            }
            // Simulate RET from BDOS
            cpu.pc = cpu.pop16();
            continue;
        }

        cpu.step();
    }

    if output.contains("ERROR") {
        has_error = true;
    }
    // If no "OK" at all, that's also a failure (tests didn't complete)
    let ok_count = output.matches("OK").count();
    let err_count = output.matches("ERROR").count();
    if ok_count == 0 && err_count == 0 {
        has_error = true;
    }

    eprintln!("ZEXDOC: {} cycles, {} BDOS calls, {} OK, {} ERROR, halted={}",
        cpu.cycles, bdos_calls, ok_count, err_count, cpu.halted);

    (output, has_error)
}

#[test]
fn zexdoc_all_pass() {
    let (output, has_error) = run_zexdoc();

    // Print the output for CI visibility
    for line in output.lines() {
        eprintln!("{}", line);
    }

    if has_error {
        // Count errors vs OKs
        let ok_count = output.matches(" OK").count();
        let err_count = output.matches("ERROR").count();
        eprintln!("\n--- ZEXDOC SUMMARY: {} OK, {} ERROR ---", ok_count, err_count);
        panic!("ZEXDOC had {} errors", err_count);
    }
}
