use z80::cpu::Cpu;
use crate::bdos;
use crate::ccp;
use crate::console::Console;
use crate::disk::DiskSystem;
use std::path::PathBuf;

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
    pub running: bool,
}

impl Cpm {
    pub fn new(drive_a: PathBuf) -> Self {
        let mut cpu = Cpu::new();
        setup_page_zero(&mut cpu);
        setup_bios_stubs(&mut cpu);

        Cpm {
            cpu,
            console: Console::new(),
            disk: DiskSystem::new(drive_a),
            running: true,
        }
    }

    pub fn run(&mut self) {
        // Start at CCP
        self.cpu.pc = CCP_ENTRY;
        self.cpu.sp = BDOS_ADDR;

        loop {
            if !self.running {
                break;
            }

            // BIOS handler intercept: PC in the handler area (FA33-FA43)
            if self.cpu.pc >= BIOS_HANDLERS && self.cpu.pc < BIOS_HANDLERS + 17 {
                let bios_func = (self.cpu.pc - BIOS_HANDLERS) as u8;
                self.handle_bios(bios_func);
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
                continue;
            }

            if self.cpu.pc == CCP_ENTRY {
                ccp::run(self);
                continue;
            }

            self.cpu.step();
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
        // Push warm boot address so RET = JP 0000h (standard CP/M convention)
        self.cpu.push16(0x0000);
        self.disk.dma_addr = DMA_DEFAULT;
    }

    /// Handle a direct BIOS call. Programs like MBASIC read the BIOS jump
    /// table and call BIOS functions directly for performance.
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
                self.cpu.a = self.console.read_key() & 0x7F;
            }
            4 => { /* CONOUT */
                self.console.write_char(self.cpu.c);
            }
            5 => { /* LIST */ } // printer — ignore
            6 => { /* PUNCH */ } // paper tape — ignore
            7 => { /* READER */ self.cpu.a = 0x1A; } // EOF
            _ => {
                if std::env::var("CPM_TRACE").is_ok() {
                    eprintln!("[BIOS] unhandled function {}", func);
                }
            }
        }
        // BIOS functions were CALLed, so RET to return
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
    // 0000h: JP BIOS_BASE+3 (warm boot entry)
    cpu.mem[0x0000] = 0xC3;
    cpu.write16(0x0001, BIOS_BASE + 3);

    // 0003h: IOBYTE
    cpu.mem[0x0003] = 0x00;
    // 0004h: current disk/user (drive A, user 0)
    cpu.mem[0x0004] = 0x00;

    // 0005h: JP BDOS_ADDR (BDOS entry — intercepted before execution)
    cpu.mem[0x0005] = 0xC3;
    cpu.write16(0x0006, BDOS_ADDR);
}

/// Base address of BIOS handler area (after the 17×3 jump table).
pub const BIOS_HANDLERS: u16 = BIOS_BASE + 17 * 3; // FA33

fn setup_bios_stubs(cpu: &mut Cpu) {
    // BIOS jump table: 17 entries × 3 bytes, each JP to a handler.
    // Programs read these JP targets to call BIOS functions directly.
    for i in 0..17u16 {
        let entry = BIOS_BASE + i * 3;
        let handler = BIOS_HANDLERS + i;
        cpu.mem[entry as usize] = 0xC3; // JP
        cpu.write16(entry + 1, handler);
    }
    // Handler area: each is a single RET (0xC9).
    // We intercept PC in this range before execution.
    for i in 0..17u16 {
        cpu.mem[(BIOS_HANDLERS + i) as usize] = 0xC9; // RET
    }
}
