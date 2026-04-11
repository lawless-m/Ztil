use crate::cpm::Cpm;
use rm380z_core::page_zero::{TPA_BASE, BDOS_ADDR};
use crate::fcb;

/// Run the CCP (Console Command Processor).
pub fn run(cpm: &mut Cpm) {
    loop {
        let drive_letter = (b'A' + cpm.disk.current_disk) as char;
        cpm.vdu_print(&format!("{}>", drive_letter));
        cpm.vdu.render(&cpm.cpu.mem);

        let line = cpm.read_line(127);

        if line.is_empty() { continue; }

        let raw_line = String::from_utf8_lossy(&line).trim().to_string();
        if raw_line.is_empty() { continue; }

        // Uppercase the command but preserve original case for args (paths)
        let (cmd, args, raw_args) = match raw_line.find(' ') {
            Some(pos) => (
                raw_line[..pos].to_uppercase(),
                raw_line[pos + 1..].trim().to_uppercase(),
                raw_line[pos + 1..].trim().to_string(),
            ),
            None => (raw_line.to_uppercase(), String::new(), String::new()),
        };

        match cmd.as_str() {
            "DIR" => cmd_dir(cpm, &args),
            "TYPE" => cmd_type(cpm, &args),
            "ERA" | "ERASE" => cmd_era(cpm, &args),
            "REN" | "RENAME" => cmd_ren(cpm, &args),
            "MOUNT" => cmd_mount(cpm, &raw_args),
            "USER" => {}
            "EXIT" => { cpm.running = false; return; }
            _ => {
                // Drive switch: "B:" or "A:" etc.
                if cmd.len() == 2 && cmd.as_bytes()[1] == b':' {
                    let drv = cmd.as_bytes()[0] - b'A';
                    if cpm.disk.is_mounted(drv + 1) {
                        cpm.disk.current_disk = drv;
                    } else {
                        cpm.vdu_print("Invalid drive\r\n");
                    }
                } else if load_transient(cpm, &cmd, &args) {
                    return;
                } else {
                    cpm.vdu_print(&format!("{}?\r\n", cmd));
                }
            }
        }
    }
}

fn cmd_dir(cpm: &mut Cpm, args: &str) {
    let pattern = if args.is_empty() { "*.*" } else { args };
    let (name_part, ext_part) = if let Some(dot) = pattern.find('.') {
        (&pattern[..dot], &pattern[dot + 1..])
    } else {
        (pattern, "*")
    };

    let mut name = [b'?'; 8];
    let mut ext = [b'?'; 3];
    if name_part != "*" {
        for (i, &ch) in name_part.as_bytes().iter().take(8).enumerate() { name[i] = ch; }
        for i in name_part.len()..8 { name[i] = b' '; }
    }
    if ext_part != "*" {
        for (i, &ch) in ext_part.as_bytes().iter().take(3).enumerate() { ext[i] = ch; }
        for i in ext_part.len()..3 { ext[i] = b' '; }
    }

    let files = cpm.disk.search_files(0, &name, &ext);
    if files.is_empty() {
        cpm.vdu_print("No file\r\n");
        return;
    }

    let mut col = 0;
    for f in &files {
        cpm.vdu_print(&format!("{:<12} ", f));
        col += 1;
        if col >= 3 { // 3 columns for 40-char display
            cpm.vdu_print("\r\n");
            col = 0;
        }
    }
    if col > 0 {
        cpm.vdu_print("\r\n");
    }
}

fn cmd_type(cpm: &mut Cpm, args: &str) {
    if args.is_empty() {
        cpm.vdu_print("Type what?\r\n");
        return;
    }
    let path = parse_host_path(cpm, args);
    let Ok(data) = std::fs::read(&path) else {
        cpm.vdu_print("File not found\r\n");
        return;
    };
    for &ch in &data {
        if ch == 0x1A { break; }
        cpm.vdu_write(ch);
    }
}

fn cmd_era(cpm: &mut Cpm, args: &str) {
    if args.is_empty() {
        cpm.vdu_print("Erase what?\r\n");
        return;
    }
    let path = parse_host_path(cpm, args);
    if std::fs::remove_file(&path).is_err() {
        cpm.vdu_print("File not found\r\n");
    }
}

fn cmd_ren(cpm: &mut Cpm, args: &str) {
    let Some(eq) = args.find('=') else {
        cpm.vdu_print("Usage: REN NEW.TYP=OLD.TYP\r\n");
        return;
    };
    let new_name = args[..eq].trim();
    let old_name = args[eq + 1..].trim();
    let old_path = parse_host_path(cpm, old_name);
    let new_path = parse_host_path(cpm, new_name);
    if std::fs::rename(&old_path, &new_path).is_err() {
        cpm.vdu_print("File not found\r\n");
    }
}

fn cmd_mount(cpm: &mut Cpm, args: &str) {
    // MOUNT B: /path/to/dir   or   MOUNT B: disk.dsk
    let parts: Vec<&str> = args.splitn(2, ' ').collect();
    if parts.len() < 2 || parts[0].len() != 2 || !parts[0].ends_with(':') {
        cpm.vdu_print("Usage: MOUNT B: path\r\n");
        return;
    }
    let drv = parts[0].as_bytes()[0].to_ascii_uppercase() - b'A';
    let path = std::path::PathBuf::from(parts[1].trim());
    cpm.disk.mount(drv, path);
    if cpm.disk.is_mounted(drv + 1) {
        cpm.vdu_print(&format!("{}:=OK\r\n", (b'A' + drv) as char));
    } else {
        cpm.vdu_print("Mount failed\r\n");
    }
}

fn load_transient(cpm: &mut Cpm, cmd: &str, args: &str) -> bool {
    let files = cpm.disk.search_files(0, &padded_name(cmd), b"COM");
    if files.is_empty() { return false; }

    let Some(dir) = cpm.disk.drive_path(0) else { return false };
    let path = dir.join(&files[0]);
    let Ok(data) = std::fs::read(&path) else { return false };

    if data.len() > (BDOS_ADDR - TPA_BASE) as usize {
        cpm.vdu_print("Program too large\r\n");
        return false;
    }

    let tail = if args.is_empty() { String::new() } else { format!(" {}", args) };
    cpm.load_com(&data, &tail);

    let parts: Vec<&str> = args.split_whitespace().collect();
    if let Some(arg1) = parts.first() {
        fcb::parse_into(&mut cpm.cpu, 0x005C, arg1);
    }
    if let Some(arg2) = parts.get(1) {
        fcb::parse_into(&mut cpm.cpu, 0x006C, arg2);
    }

    true
}

fn padded_name(name: &str) -> [u8; 8] {
    let mut buf = [b' '; 8];
    for (i, &ch) in name.as_bytes().iter().take(8).enumerate() {
        buf[i] = ch.to_ascii_uppercase();
    }
    buf
}

fn parse_host_path(cpm: &Cpm, filename: &str) -> std::path::PathBuf {
    let dir = cpm.disk.drive_path(0).cloned().unwrap_or_default();
    dir.join(filename.to_uppercase())
}
