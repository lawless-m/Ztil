use crossterm::terminal;
use rm380z::cpm::Cpm;
use std::path::PathBuf;

struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Usage: rm380z [-a <dir>] [-b <dir>] [-c <dir>] [-d <dir>] [file.com [args...]]
    let mut drives: Vec<(u8, PathBuf)> = Vec::new();
    let mut com_file: Option<String> = None;
    let mut com_args = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--drive-a" | "-a" => { i += 1; if i < args.len() { drives.push((0, PathBuf::from(&args[i]))); } }
            "--drive-b" | "-b" => { i += 1; if i < args.len() { drives.push((1, PathBuf::from(&args[i]))); } }
            "--drive-c" | "-c" => { i += 1; if i < args.len() { drives.push((2, PathBuf::from(&args[i]))); } }
            "--drive-d" | "-d" => { i += 1; if i < args.len() { drives.push((3, PathBuf::from(&args[i]))); } }
            _ => {
                if com_file.is_none() {
                    com_file = Some(args[i].clone());
                } else {
                    if !com_args.is_empty() { com_args.push(' '); }
                    com_args.push_str(&args[i]);
                }
            }
        }
        i += 1;
    }

    // Default: mount current directory as A: if no drives specified
    if drives.is_empty() {
        drives.push((0, PathBuf::from(".")));
    }

    let mut cpm = Cpm::new();
    for (drv, path) in &drives {
        cpm.disk.mount(*drv, path.clone());
    }

    if let Some(ref com) = com_file {
        let path = if PathBuf::from(com).exists() {
            PathBuf::from(com)
        } else if let Some(dir) = cpm.disk.drive_path(1) { // drive A = index 0+1
            dir.join(com)
        } else {
            PathBuf::from(com)
        };

        match std::fs::read(&path) {
            Ok(data) => {
                let tail = if com_args.is_empty() { String::new() } else { format!(" {}", com_args) };
                cpm.load_com(&data, &tail);
            }
            Err(e) => {
                eprintln!("Cannot load {}: {}", path.display(), e);
                std::process::exit(1);
            }
        }
    }

    let _guard = if terminal::enable_raw_mode().is_ok() {
        Some(RawModeGuard)
    } else {
        None
    };

    eprint!("\x1b[2J\x1b[H");
    cpm.vdu_print("RM 380Z CP/M 2.2\r\n\r\n");
    cpm.run();
}
