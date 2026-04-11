use z80::cpu::Cpu;
use crate::vdu::Vdu;
use crate::fcb;
use crate::page_zero::{TPA_BASE, BDOS_ADDR};

/// Platform-specific operations the CCP needs.
pub trait CcpPlatform {
    /// Search for a .COM file. Returns the file data if found.
    fn find_com(&self, name: &str) -> Option<Vec<u8>>;
    /// List files matching a pattern. Returns filenames.
    fn list_files(&self, pattern: &str) -> Vec<String>;
    /// Read a file's contents (for TYPE command).
    fn read_file(&self, name: &str) -> Option<Vec<u8>>;
}

/// Run the CCP loop. Reads lines, dispatches commands.
/// Returns when a .COM is loaded (caller should run the CPU)
/// or when exit is requested.
pub enum CcpResult {
    /// A .COM was loaded at 0100h, ready to execute.
    RunProgram,
    /// User requested exit.
    Exit,
}

pub fn run_ccp(
    cpu: &mut Cpu,
    vdu: &mut Vdu,
    current_disk: u8,
    key_reader: &mut dyn FnMut(&mut Vdu, &mut [u8; 0x10000]) -> Vec<u8>,
    platform: &dyn CcpPlatform,
) -> CcpResult {
    loop {
        let drive_letter = (b'A' + current_disk) as char;
        vdu.write_str(&mut cpu.mem, &format!("{}>", drive_letter));

        let line = key_reader(vdu, &mut cpu.mem);
        if line.is_empty() { continue; }

        let raw_line = String::from_utf8_lossy(&line).trim().to_string();
        if raw_line.is_empty() { continue; }

        let (cmd, args) = match raw_line.find(' ') {
            Some(pos) => (raw_line[..pos].to_uppercase(), raw_line[pos + 1..].trim().to_string()),
            None => (raw_line.to_uppercase(), String::new()),
        };

        match cmd.as_str() {
            "DIR" => {
                let pattern = if args.is_empty() { "*.*".to_string() } else { args.to_uppercase() };
                let files = platform.list_files(&pattern);
                if files.is_empty() {
                    vdu.write_str(&mut cpu.mem, "No file\r\n");
                } else {
                    let mut col = 0;
                    for f in &files {
                        vdu.write_str(&mut cpu.mem, &format!("{:<12} ", f));
                        col += 1;
                        if col >= 3 { vdu.write_str(&mut cpu.mem, "\r\n"); col = 0; }
                    }
                    if col > 0 { vdu.write_str(&mut cpu.mem, "\r\n"); }
                }
            }
            "TYPE" => {
                if args.is_empty() {
                    vdu.write_str(&mut cpu.mem, "Type what?\r\n");
                } else if let Some(data) = platform.read_file(&args.to_uppercase()) {
                    for &ch in &data {
                        if ch == 0x1A { break; }
                        vdu.write_char(&mut cpu.mem, ch);
                    }
                } else {
                    vdu.write_str(&mut cpu.mem, "File not found\r\n");
                }
            }
            "EXIT" => return CcpResult::Exit,
            _ => {
                // Try to load as .COM
                if let Some(data) = platform.find_com(&cmd) {
                    if data.len() > (BDOS_ADDR - TPA_BASE) as usize {
                        vdu.write_str(&mut cpu.mem, "Program too large\r\n");
                        continue;
                    }
                    let tail = if args.is_empty() { String::new() } else { format!(" {}", args) };
                    crate::page_zero::load_com(cpu, &data, &tail);

                    // Parse args into FCBs
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    if let Some(arg1) = parts.first() {
                        fcb::parse_into(cpu, 0x005C, arg1);
                    }
                    if let Some(arg2) = parts.get(1) {
                        fcb::parse_into(cpu, 0x006C, arg2);
                    }
                    return CcpResult::RunProgram;
                }
                vdu.write_str(&mut cpu.mem, &format!("{}?\r\n", cmd));
            }
        }
    }
}
