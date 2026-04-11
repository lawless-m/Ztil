use rm380z_wasm::Emulator;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut emu = Emulator::new();

    // Load .COM file from argument or default to HELLO.COM
    if args.len() > 1 {
        let data = std::fs::read(&args[1]).expect("Cannot read file");
        emu.load_com(&data);
    } else {
        emu.load_com(include_bytes!("../../disks/a/HELLO.COM"));
    }

    // Feed keyboard input from remaining args
    if args.len() > 2 {
        for ch in args[2..].join(" ").bytes() {
            emu.key_press(ch);
        }
        emu.key_press(0x0D);
    }

    // Run
    for _ in 0..1000 {
        let steps = emu.run(100000);
        if steps == 0 && !emu.needs_key() && !emu.needs_net() { break; }
        if emu.needs_key() && args.len() <= 2 { break; }
    }

    // Dump VDU (non-empty lines only)
    let ptr = emu.vdu_ptr();
    for row in 0..24 {
        let mut line = String::new();
        for col in 0..40 {
            let ch = unsafe { *ptr.add(row * 40 + col) };
            line.push(if ch >= 0x20 && ch <= 0x7E { ch as char } else { ' ' });
        }
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            println!("{}", trimmed);
        }
    }
}
