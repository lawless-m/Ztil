use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use std::io::{self, Write};
use z80::cpu::Cpu;
use zip_vm::native;
use zip_vm::rom;

fn main() {
    let mut cpu = Cpu::new();
    rom::load(&mut cpu);
    let mut io_ctx = native::Io::new();

    // Batch mode: if args provided, feed as input line
    let args: Vec<String> = std::env::args().collect();
    let batch = args.len() > 1;
    if batch {
        let input = args[1..].join(" ");
        for b in input.bytes() {
            io_ctx.input.push_back(b);
        }
        io_ctx.input.push_back(0x0D);
    }

    if batch {
        run_batch(&mut cpu, &mut io_ctx);
    } else {
        run_interactive(&mut cpu, &mut io_ctx);
    }
}

fn run_batch(cpu: &mut Cpu, io: &mut native::Io) {
    let max_steps: u64 = 50_000_000;
    for _ in 0..max_steps {
        if cpu.halted { break; }
        flush_output(io);
        if native::is_trap(cpu.pc) {
            if !native::handle_trap(cpu, io) {
                eprintln!("\nUnhandled trap {:04X}", cpu.pc);
                break;
            }
            if io.waiting_for_key && io.input.is_empty() { break; }
            continue;
        }
        cpu.step();
    }
    flush_output(io);
}

fn run_interactive(cpu: &mut Cpu, io: &mut native::Io) {
    terminal::enable_raw_mode().expect("failed to enable raw mode");
    let _guard = RawModeGuard; // ensures cleanup on drop

    loop {
        // Run CPU until it needs input or halts
        let mut steps = 0u64;
        loop {
            if cpu.halted {
                flush_output(io);
                eprintln!("\r\nHALT at PC={:04X}", cpu.pc);
                return;
            }
            if native::is_trap(cpu.pc) {
                if !native::handle_trap(cpu, io) {
                    flush_output(io);
                    eprintln!("\r\nUnhandled trap {:04X}", cpu.pc);
                    return;
                }
                flush_output(io);
                if io.waiting_for_key {
                    break; // need input from user
                }
                continue;
            }
            cpu.step();
            steps += 1;
            if steps > 10_000_000 {
                flush_output(io);
                eprintln!("\r\nRunaway execution");
                return;
            }
        }

        // Wait for a keypress
        loop {
            if let Ok(evt) = event::read() {
                match evt {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        return;
                    }
                    Event::Key(KeyEvent { code, .. }) => {
                        let ch = match code {
                            KeyCode::Char(c) => c as u8,
                            KeyCode::Enter => 0x0D,
                            KeyCode::Backspace => 0x08,
                            KeyCode::Esc => 0x18, // line delete
                            _ => continue,
                        };
                        io.input.push_back(ch);
                        io.waiting_for_key = false;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn flush_output(io: &mut native::Io) {
    if io.output.is_empty() { return; }
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for &b in &io.output {
        match b {
            0x0D => { let _ = out.write_all(b"\r\n"); }
            0x0A => {} // LF after CR already handled
            0x08 => { let _ = out.write_all(b"\x08 \x08"); } // backspace
            0x20..=0x7E => { let _ = out.write_all(&[b]); }
            _ => {}
        }
    }
    let _ = out.flush();
    io.output.clear();
}

/// RAII guard to restore terminal on exit/panic.
struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        print!("\r\n");
    }
}
