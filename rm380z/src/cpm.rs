use z80::cpu::Cpu;
use crate::bdos;
use crate::ccp;
use crate::console::Console;
use crate::disk::DiskSystem;
use crate::vdu::Vdu;

// CP/M 2.2 memory map for a 64K system
pub const BDOS_ENTRY: u16 = 0x0005;
pub const BDOS_ADDR: u16 = 0xE400;  // stored at 0006h, defines TPA top
pub const CCP_ENTRY: u16 = 0xD000;
pub const BIOS_BASE: u16 = 0xFA00;
pub const TPA_BASE: u16 = 0x0100;
pub const DMA_DEFAULT: u16 = 0x0080;

pub struct Cpm {
    pub cpu: Cpu,
    pub console: Console,
    pub disk: DiskSystem,
    pub vdu: Vdu,
    pub running: bool,
}

impl Cpm {
    pub fn new() -> Self {
        let mut cpu = Cpu::new();
        setup_page_zero(&mut cpu);
        setup_bios_stubs(&mut cpu);

        let mut vdu = Vdu::new();
        vdu.init(&mut cpu.mem);

        Cpm {
            cpu,
            console: Console::new(),
            disk: DiskSystem::new(),
            vdu,
            running: true,
        }
    }

    pub fn run(&mut self) {
        // Start at CCP
        self.cpu.pc = CCP_ENTRY;
        self.cpu.sp = BDOS_ADDR;

        // Initial screen render
        self.vdu.render(&self.cpu.mem);

        let mut step_count = 0u32;

        loop {
            if !self.running {
                break;
            }

            // BIOS handler intercept: PC in the handler area (FA33-FA43)
            if self.cpu.pc >= BIOS_HANDLERS && self.cpu.pc < BIOS_HANDLERS + 17 {
                let bios_func = (self.cpu.pc - BIOS_HANDLERS) as u8;
                self.handle_bios(bios_func);
                self.vdu.render(&self.cpu.mem);
                continue;
            }

            if self.cpu.halted {
                self.cpu.halted = false;
                self.warm_boot();
                continue;
            }

            if self.cpu.pc == BDOS_ENTRY || self.cpu.pc == BDOS_ADDR {
                let func = self.cpu.c;
                bdos::dispatch(self);
                // Warm boot (C=0) sets PC directly; don't pop return address
                if func != 0 {
                    self.cpu.pc = self.cpu.pop16();
                }
                self.vdu.render(&self.cpu.mem);
                continue;
            }

            if self.cpu.pc == CCP_ENTRY {
                ccp::run(self);
                continue;
            }

            self.cpu.step();

            // Periodic render for programs that write directly to VDU RAM
            step_count += 1;
            if step_count >= 10000 {
                step_count = 0;
                self.vdu.render(&self.cpu.mem);
            }
        }
    }

    /// Write a character through the VDU (used by BDOS, BIOS, and CCP).
    pub fn vdu_write(&mut self, ch: u8) {
        self.vdu.write_char(&mut self.cpu.mem, ch);
    }

    /// Write a string through the VDU.
    pub fn vdu_print(&mut self, s: &str) {
        for &ch in s.as_bytes() {
            self.vdu.write_char(&mut self.cpu.mem, ch);
        }
    }

    /// Read a line with VDU echo. Used by CCP and BDOS F10.
    pub fn read_line(&mut self, max_len: u8) -> Vec<u8> {
        self.vdu.render(&self.cpu.mem);
        let mut buf = Vec::new();
        loop {
            let ch = self.console.read_key();
            match ch {
                0x0D => {
                    self.vdu_print("\r\n");
                    self.vdu.render(&self.cpu.mem);
                    return buf;
                }
                0x08 | 0x7F => {
                    if !buf.is_empty() {
                        buf.pop();
                        // Erase character on VDU: back, space, back
                        self.vdu_write(0x08);
                        self.vdu_write(b' ');
                        self.vdu_write(0x08);
                        self.vdu.render(&self.cpu.mem);
                    }
                }
                0x03 => {
                    return Vec::new();
                }
                _ if ch >= 0x20 && buf.len() < max_len as usize => {
                    buf.push(ch);
                    self.vdu_write(ch);
                    self.vdu.render(&self.cpu.mem);
                }
                _ => {}
            }
        }
    }

    /// Load a .COM file at 0100h and set up page zero for execution.
    pub fn load_com(&mut self, data: &[u8], args: &str) {
        assert!(data.len() <= (BDOS_ADDR - TPA_BASE) as usize, "COM file too large for TPA");
        self.cpu.mem[TPA_BASE as usize..TPA_BASE as usize + data.len()].copy_from_slice(data);

        // Command tail at 0080h
        let tail = args.as_bytes();
        let tail_len = tail.len().min(127);
        self.cpu.mem[0x0080] = tail_len as u8;
        if tail_len > 0 {
            self.cpu.mem[0x0081..0x0081 + tail_len].copy_from_slice(&tail[..tail_len]);
        }

        // Clear FCBs
        for i in 0x005Cu16..0x0080 {
            self.cpu.mem[i as usize] = 0;
        }
        // Fill FCB filenames with spaces
        for i in 1u16..12 {
            self.cpu.mem[(0x005C + i) as usize] = b' ';
            self.cpu.mem[(0x006C + i) as usize] = b' ';
        }

        self.cpu.pc = TPA_BASE;
        self.cpu.sp = BDOS_ADDR;
        self.cpu.push16(0x0000);
        self.disk.dma_addr = DMA_DEFAULT;
    }

    /// Handle a direct BIOS call.
    fn handle_bios(&mut self, func: u8) {
        if std::env::var("CPM_TRACE").is_ok() {
            eprintln!("[BIOS] func={} C={:02X}", func, self.cpu.c);
        }
        match func {
            0 => { /* BOOT */ self.warm_boot(); return; }
            1 => { /* WBOOT */ self.warm_boot(); return; }
            2 => { /* CONST */
                self.cpu.a = if self.console.key_ready() { 0xFF } else { 0x00 };
            }
            3 => { /* CONIN */
                self.vdu.render(&self.cpu.mem);
                self.cpu.a = self.console.read_key() & 0x7F;
            }
            4 => { /* CONOUT */
                self.vdu.write_char(&mut self.cpu.mem, self.cpu.c);
            }
            5 => { /* LIST */ }
            6 => { /* PUNCH */ }
            7 => { /* READER */ self.cpu.a = 0x1A; }
            _ => {
                if std::env::var("CPM_TRACE").is_ok() {
                    eprintln!("[BIOS] unhandled function {}", func);
                }
            }
        }
        self.cpu.pc = self.cpu.pop16();
    }

    /// Warm boot: return to CCP.
    pub fn warm_boot(&mut self) {
        setup_page_zero(&mut self.cpu);
        self.cpu.pc = CCP_ENTRY;
        self.cpu.sp = BDOS_ADDR;
        self.disk.dma_addr = DMA_DEFAULT;
    }
}

fn setup_page_zero(cpu: &mut Cpu) {
    cpu.mem[0x0000] = 0xC3;
    cpu.write16(0x0001, BIOS_BASE + 3);
    cpu.mem[0x0003] = 0x00;
    cpu.mem[0x0004] = 0x00;
    cpu.mem[0x0005] = 0xC3;
    cpu.write16(0x0006, BDOS_ADDR);
}

/// Base address of BIOS handler area (after the 17x3 jump table).
pub const BIOS_HANDLERS: u16 = BIOS_BASE + 17 * 3;

fn setup_bios_stubs(cpu: &mut Cpu) {
    for i in 0..17u16 {
        let entry = BIOS_BASE + i * 3;
        let handler = BIOS_HANDLERS + i;
        cpu.mem[entry as usize] = 0xC3;
        cpu.write16(entry + 1, handler);
    }
    for i in 0..17u16 {
        cpu.mem[(BIOS_HANDLERS + i) as usize] = 0xC9;
    }
}
