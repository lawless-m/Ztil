use zip_vm::cpu::Cpu;
use zip_vm::native;
use zip_vm::rom;

fn main() {
    let mut cpu = Cpu::new();
    rom::load(&mut cpu);

    let mut io = native::Io::new();

    // If args provided, feed them as input (batch mode)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let input = args[1..].join(" ");
        for b in input.bytes() {
            io.input.push_back(b);
        }
        io.input.push_back(0x0D); // CR to end the line
    }

    let mut max_steps: u64 = 10_000_000;

    loop {
        if cpu.halted {
            flush_output(&mut io);
            eprintln!("\nHALT at PC={:04X} after {} cycles", cpu.pc, cpu.cycles);
            break;
        }

        if max_steps == 0 {
            flush_output(&mut io);
            eprintln!("\nExecution limit reached. PC={:04X} cycles={}", cpu.pc, cpu.cycles);
            break;
        }
        max_steps -= 1;

        // Check for native trap
        if native::is_trap(cpu.pc) {
            if !native::handle_trap(&mut cpu, &mut io) {
                flush_output(&mut io);
                eprintln!("\nUnhandled trap {:04X} at cycle {}", cpu.pc, cpu.cycles);
                break;
            }
            flush_output(&mut io);

            // If waiting for key and no input available, we're done in batch mode
            if io.waiting_for_key {
                flush_output(&mut io);
                break;
            }
            continue;
        }

        cpu.step();
    }

    flush_output(&mut io);
}

fn flush_output(io: &mut native::Io) {
    if !io.output.is_empty() {
        let s: String = io.output.iter().map(|&b| {
            if b == 0x0D { '\n' }
            else if b >= 0x20 && b < 0x7F { b as char }
            else if b == 0x0A { ' ' } // LF after CR already handled
            else { ' ' }
        }).collect();
        print!("{}", s);
        io.output.clear();
    }
}
