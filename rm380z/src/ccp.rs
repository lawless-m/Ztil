use crate::cpm::{Cpm, TPA_BASE, BDOS_ADDR};
use crate::fcb;

/// Run the CCP (Console Command Processor).
/// Called when PC reaches CCP_ENTRY. Loops until a transient command runs or exit.
pub fn run(cpm: &mut Cpm) {
    loop {
        // Print prompt
        let drive_letter = (b'A' + cpm.disk.current_disk) as char;
        cpm.console.write_str(&format!("{}>", drive_letter));

        // Read command line
        let line = cpm.console.read_line(127);
        if line.is_empty() { continue; }

        let line_str = String::from_utf8_lossy(&line).to_uppercase();
        let line_str = line_str.trim().to_string();
        if line_str.is_empty() { continue; }

        // Split command and arguments
        let (cmd, args) = match line_str.find(' ') {
            Some(pos) => (line_str[..pos].to_string(), line_str[pos + 1..].trim().to_string()),
            None => (line_str.clone(), String::new()),
        };

        match cmd.as_str() {
            "DIR" => cmd_dir(cpm, &args),
            "TYPE" => cmd_type(cpm, &args),
            "ERA" | "ERASE" => cmd_era(cpm, &args),
            "REN" | "RENAME" => cmd_ren(cpm, &args),
            "USER" => { /* ignore */ }
            "EXIT" => { cpm.running = false; return; }
            _ => {
                // Try to load as .COM file
                if load_transient(cpm, &cmd, &args) {
                    return; // execution continues in the main loop
                }
                cpm.console.write_str(&format!("{}?\r\n", cmd));
            }
        }
    }
}

fn cmd_dir(cpm: &mut Cpm, args: &str) {
    let pattern = if args.is_empty() { "*.*" } else { args };

    // Parse pattern into name/ext
    let (name_part, ext_part) = if let Some(dot) = pattern.find('.') {
        (&pattern[..dot], &pattern[dot + 1..])
    } else {
        (pattern, "*")
    };

    let mut name = [b'?'; 8];
    let mut ext = [b'?'; 3];

    if name_part != "*" {
        for (i, &ch) in name_part.as_bytes().iter().take(8).enumerate() {
            name[i] = ch;
        }
        for i in name_part.len()..8 {
            name[i] = b' ';
        }
    }
    if ext_part != "*" {
        for (i, &ch) in ext_part.as_bytes().iter().take(3).enumerate() {
            ext[i] = ch;
        }
        for i in ext_part.len()..3 {
            ext[i] = b' ';
        }
    }

    let files = cpm.disk.search_files(&name, &ext);
    if files.is_empty() {
        cpm.console.write_str("No file\r\n");
        return;
    }

    let mut col = 0;
    for f in &files {
        // Format as CP/M style: "FILENAME.TYP"
        let padded = format!("{:<12}", f);
        cpm.console.write_str(&format!("{} ", padded));
        col += 1;
        if col >= 4 {
            cpm.console.write_str("\r\n");
            col = 0;
        }
    }
    if col > 0 {
        cpm.console.write_str("\r\n");
    }
}

fn cmd_type(cpm: &mut Cpm, args: &str) {
    if args.is_empty() {
        cpm.console.write_str("Type what?\r\n");
        return;
    }

    let path = parse_host_path(cpm, args);
    let Ok(data) = std::fs::read(&path) else {
        cpm.console.write_str("File not found\r\n");
        return;
    };

    for &ch in &data {
        if ch == 0x1A { break; } // CP/M EOF
        cpm.console.write_char(ch);
    }
}

fn cmd_era(cpm: &mut Cpm, args: &str) {
    if args.is_empty() {
        cpm.console.write_str("Erase what?\r\n");
        return;
    }
    let path = parse_host_path(cpm, args);
    if std::fs::remove_file(&path).is_ok() {
        // success, no message
    } else {
        cpm.console.write_str("File not found\r\n");
    }
}

fn cmd_ren(cpm: &mut Cpm, args: &str) {
    // Format: NEW.TYP=OLD.TYP
    let Some(eq) = args.find('=') else {
        cpm.console.write_str("Usage: REN NEW.TYP=OLD.TYP\r\n");
        return;
    };
    let new_name = args[..eq].trim();
    let old_name = args[eq + 1..].trim();
    let old_path = parse_host_path(cpm, old_name);
    let new_path = parse_host_path(cpm, new_name);
    if std::fs::rename(&old_path, &new_path).is_err() {
        cpm.console.write_str("File not found\r\n");
    }
}

/// Try to load a .COM transient command. Returns true if loaded.
fn load_transient(cpm: &mut Cpm, cmd: &str, args: &str) -> bool {
    let files = cpm.disk.search_files(
        &padded_name(cmd),
        b"COM",
    );

    if files.is_empty() { return false; }

    let path = cpm.disk.drive_a.join(&files[0]);
    let Ok(data) = std::fs::read(&path) else { return false };

    if data.len() > (BDOS_ADDR - TPA_BASE) as usize {
        cpm.console.write_str("Program too large\r\n");
        return false;
    }

    // Prepare command tail
    let tail = if args.is_empty() {
        String::new()
    } else {
        format!(" {}", args)
    };

    cpm.load_com(&data, &tail);

    // Parse first two arguments into FCBs
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
    cpm.disk.drive_a.join(filename.to_uppercase())
}
