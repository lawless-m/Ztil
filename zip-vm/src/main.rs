use std::env;
use std::fs;
use zip_vm::cpu::Cpu;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: zipcli <rom.bin> [load_addr_hex]");
        std::process::exit(1);
    }

    let rom = fs::read(&args[1]).expect("failed to read ROM file");
    let load_addr = if args.len() > 2 {
        u16::from_str_radix(&args[2], 16).expect("invalid hex load address")
    } else {
        0x0100 // CP/M .COM default
    };

    let mut cpu = Cpu::new();
    cpu.mem[load_addr as usize..load_addr as usize + rom.len()].copy_from_slice(&rom);
    cpu.pc = load_addr;
    cpu.sp = 0xFC00; // ZIP data stack top

    println!("Loaded {} bytes at {:04X}, starting execution.", rom.len(), load_addr);

    loop {
        if cpu.halted {
            println!("\nHALT at PC={:04X} after {} cycles", cpu.pc, cpu.cycles);
            break;
        }
        cpu.step();
    }
}
