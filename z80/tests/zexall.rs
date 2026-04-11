use z80::cpu::Cpu;

const ZEXALL: &[u8] = include_bytes!("data/zexall.com");

/// Minimal CP/M stub for ZEXALL — same harness as ZEXDOC but tests all flag bits.
fn run_zexall() -> (String, bool) {
    let mut cpu = Cpu::new();

    cpu.mem[0x0100..0x0100 + ZEXALL.len()].copy_from_slice(ZEXALL);

    cpu.mem[0x0000] = 0x76; // HALT at warm boot
    cpu.mem[0x0005] = 0xC9;
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

        if cpu.pc == 0x0005 {
            bdos_calls += 1;
            match cpu.c {
                0 => { break; }
                1 => { cpu.a = 0; }
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
            cpu.pc = cpu.pop16();
            continue;
        }

        cpu.step();
    }

    if output.contains("ERROR") {
        has_error = true;
    }
    let ok_count = output.matches("OK").count();
    let err_count = output.matches("ERROR").count();
    if ok_count == 0 && err_count == 0 {
        has_error = true;
    }

    eprintln!("ZEXALL: {} cycles, {} BDOS calls, {} OK, {} ERROR, halted={}",
        cpu.cycles, bdos_calls, ok_count, err_count, cpu.halted);

    (output, has_error)
}

#[test]
fn zexall_all_pass() {
    let (output, has_error) = run_zexall();

    for line in output.lines() {
        eprintln!("{}", line);
    }

    if has_error {
        let ok_count = output.matches(" OK").count();
        let err_count = output.matches("ERROR").count();
        eprintln!("\n--- ZEXALL SUMMARY: {} OK, {} ERROR ---", ok_count, err_count);
        panic!("ZEXALL had {} errors", err_count);
    }
}
