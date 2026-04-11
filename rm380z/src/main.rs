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

    // Usage: rm380z [--drive-a <dir>] [file.com [args...]]
    let mut drive_a = PathBuf::from(".");
    let mut com_file: Option<String> = None;
    let mut com_args = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--drive-a" | "-a" => {
                i += 1;
                if i < args.len() { drive_a = PathBuf::from(&args[i]); }
            }
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

    let mut cpm = Cpm::new(drive_a.clone());

    // If a .COM file was specified, load it directly
    if let Some(ref com) = com_file {
        let path = if PathBuf::from(com).exists() {
            PathBuf::from(com)
        } else {
            drive_a.join(com)
        };

        match std::fs::read(&path) {
            Ok(data) => {
                let tail = if com_args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", com_args)
                };
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

    eprint!("\x1b[2J\x1b[H"); // clear screen
    eprintln!("RM 380Z CP/M 2.2\r");
    eprintln!("\r");

    cpm.run();
}
