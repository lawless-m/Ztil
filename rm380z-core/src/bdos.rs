use z80::cpu::Cpu;
use crate::fcb;
use crate::page_zero::DMA_DEFAULT;

/// Parse a network/device filename from an FCB.
/// Returns (file_type, ext_string).
pub fn parse_net_fcb(cpu: &Cpu, fcb: u16) -> Option<(&'static str, String)> {
    let name: String = (0..8).map(|i| (cpu.read8(fcb + 1 + i) & 0x7F) as char).collect();
    let ext: String = (0..3).map(|i| (cpu.read8(fcb + 9 + i) & 0x7F) as char).collect();
    let name = name.trim();
    let ext = ext.trim();
    let ext_owned = ext.to_string();

    match (name, ext) {
        ("CLONE", "WWW" | "WSK") => Some(("clone", ext_owned)),
        ("CLAUDE", "AI") => Some(("claude", ext_owned)),
        ("CLAUDE", "KEY") => Some(("apikey", ext_owned)),
        ("CLAUDE", "CLI") => Some(("cli", ext_owned)),
        ("CLAUDE", "RUN") => Some(("run", ext_owned)),
        ("CLAUDE", "MNS") => Some(("models", ext_owned)),
        ("CLAUDE", "MDL") => Some(("setmodel", ext_owned)),
        ("MEM", "0" | "1" | "2" | "3" | "VDU") => Some(("mem", ext_owned)),
        ("DEV", "CPU") => Some(("devcpu", ext_owned)),
        _ => {
            let id: u8 = name.parse().ok()?;
            match ext {
                "CTL" => Some(("ctl", ext_owned)),
                "DATA" => Some(("data", ext_owned)),
                _ => None,
            }
        }
    }
}

/// Get the connection ID from an FCB name field (e.g. "0", "1").
pub fn parse_conn_id(cpu: &Cpu, fcb: u16) -> Option<u8> {
    let name: String = (0..8).map(|i| (cpu.read8(fcb + 1 + i) & 0x7F) as char).collect();
    name.trim().parse().ok()
}

/// Memory bank base and size for a MEM.x file.
pub fn mem_bank(ext: &str) -> (usize, usize) {
    match ext {
        "0" => (0x0000, 0x4000),
        "1" => (0x4000, 0x4000),
        "2" => (0x8000, 0x4000),
        "3" => (0xC000, 0x4000),
        "VDU" => (0xFC00, 0x0400),
        _ => (0, 0),
    }
}

/// Read 128 bytes from a memory bank into DMA.
pub fn read_mem_bank(cpu: &mut Cpu, ext: &str, fcb: u16, dma: u16) -> bool {
    let (base, size) = mem_bank(ext);
    let cr = cpu.read8(fcb + 32) as usize;
    let offset = base + cr * 128;
    if offset + 128 <= base + size {
        cpu.mem.copy_within(offset..offset + 128, dma as usize);
        true
    } else {
        false
    }
}

/// Write 128 bytes from DMA into a memory bank.
pub fn write_mem_bank(cpu: &mut Cpu, ext: &str, fcb: u16, dma: u16) {
    let (base, size) = mem_bank(ext);
    let cr = cpu.read8(fcb + 32) as usize;
    let offset = base + cr * 128;
    if offset + 128 <= base + size {
        cpu.mem.copy_within(dma as usize..dma as usize + 128, offset);
    }
}

/// Generate a CPU register dump as text.
pub fn cpu_dump(cpu: &Cpu) -> String {
    format!(
        "A={:02X} F={:02X} BC={:04X} DE={:04X} HL={:04X}\r\nSP={:04X} PC={:04X} IX={:04X} IY={:04X}\r\n",
        cpu.a, cpu.f, cpu.bc(), cpu.de(), cpu.hl(),
        cpu.sp, cpu.pc, cpu.ix, cpu.iy
    )
}

/// Set the standard BDOS return convention: A=L=val, H=B=0.
pub fn set_return(cpu: &mut Cpu, val: u8) {
    cpu.a = val;
    cpu.l = val;
    cpu.h = 0;
    cpu.b = val;
}
