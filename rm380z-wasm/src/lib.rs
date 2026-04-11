use wasm_bindgen::prelude::*;
use z80::cpu::Cpu;
use std::collections::VecDeque;

// Memory map
const BDOS_ENTRY: u16 = 0x0005;
const BDOS_ADDR: u16 = 0xE400;
const BIOS_BASE: u16 = 0xFA00;
const BIOS_HANDLERS: u16 = BIOS_BASE + 17 * 3;
const TPA_BASE: u16 = 0x0100;

// VDU
const VDU_BASE: u16 = 0xFC00;
const VDU_COLS: usize = 40;
const VDU_ROWS: usize = 24;
const VDU_SIZE: usize = VDU_COLS * VDU_ROWS;

#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    cursor_row: usize,
    cursor_col: usize,
    key_buffer: VecDeque<u8>,
    waiting_for_key: bool,
    running: bool,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        let mut cpu = Cpu::new();
        setup_page_zero(&mut cpu);
        setup_bios(&mut cpu);

        // Init VDU with spaces
        for i in 0..VDU_SIZE {
            cpu.mem[VDU_BASE as usize + i] = b' ';
        }

        let mut emu = Emulator {
            cpu,
            cursor_row: 0,
            cursor_col: 0,
            key_buffer: VecDeque::new(),
            waiting_for_key: false,
            running: true,
        };

        // Print banner
        emu.vdu_print("RM 380Z CP/M 2.2\r\n\r\nA>");
        emu
    }

    /// Load a .COM file into memory at 0100h.
    pub fn load_com(&mut self, data: &[u8]) {
        let len = data.len().min((BDOS_ADDR - TPA_BASE) as usize);
        self.cpu.mem[TPA_BASE as usize..TPA_BASE as usize + len].copy_from_slice(&data[..len]);

        // Command tail: empty
        self.cpu.mem[0x0080] = 0;

        // Clear FCBs
        for i in 0x005Cu16..0x0080 {
            self.cpu.mem[i as usize] = 0;
        }
        for i in 1u16..12 {
            self.cpu.mem[(0x005C + i) as usize] = b' ';
            self.cpu.mem[(0x006C + i) as usize] = b' ';
        }

        self.cpu.pc = TPA_BASE;
        self.cpu.sp = BDOS_ADDR;
        self.cpu.push16(0x0000);
        self.waiting_for_key = false;

        // Clear VDU
        for i in 0..VDU_SIZE {
            self.cpu.mem[VDU_BASE as usize + i] = b' ';
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Run up to max_steps Z80 instructions. Stops early if waiting for key input.
    /// Returns number of steps executed.
    pub fn run(&mut self, max_steps: u32) -> u32 {
        if !self.running { return 0; }

        let mut steps = 0u32;
        while steps < max_steps {
            if self.waiting_for_key { break; }
            if !self.running { break; }

            // BIOS handler intercept
            if self.cpu.pc >= BIOS_HANDLERS && self.cpu.pc < BIOS_HANDLERS + 17 {
                let func = (self.cpu.pc - BIOS_HANDLERS) as u8;
                self.handle_bios(func);
                if self.waiting_for_key { break; }
                continue;
            }

            if self.cpu.halted {
                self.cpu.halted = false;
                self.warm_boot();
                continue;
            }

            // BDOS intercept
            if self.cpu.pc == BDOS_ENTRY || self.cpu.pc == BDOS_ADDR {
                let func = self.cpu.c;
                self.handle_bdos(func);
                if func != 0 && !self.waiting_for_key {
                    self.cpu.pc = self.cpu.pop16();
                }
                if self.waiting_for_key { break; }
                continue;
            }

            self.cpu.step();
            steps += 1;
        }
        steps
    }

    /// Push a key into the input buffer.
    pub fn key_press(&mut self, ch: u8) {
        self.key_buffer.push_back(ch);
        self.waiting_for_key = false;
    }

    /// Get pointer to VDU memory for direct JS access.
    pub fn vdu_ptr(&self) -> *const u8 {
        &self.cpu.mem[VDU_BASE as usize] as *const u8
    }

    pub fn cursor_row(&self) -> usize { self.cursor_row }
    pub fn cursor_col(&self) -> usize { self.cursor_col }
    pub fn needs_key(&self) -> bool { self.waiting_for_key }
    pub fn is_running(&self) -> bool { self.running }
}

// --- Internal methods (not exported) ---

impl Emulator {
    fn handle_bdos(&mut self, func: u8) {
        match func {
            0 => { self.warm_boot(); }
            1 => {
                // CONIN: read key with echo
                if let Some(ch) = self.key_buffer.pop_front() {
                    self.vdu_char(ch);
                    self.cpu.a = ch & 0x7F;
                    self.cpu.l = self.cpu.a;
                    self.cpu.h = 0;
                } else {
                    self.waiting_for_key = true;
                    // Don't pop return address — we'll retry on next run()
                }
            }
            2 => {
                // CONOUT
                self.vdu_char(self.cpu.e);
                self.cpu.a = 0;
                self.cpu.l = 0;
                self.cpu.h = 0;
            }
            6 => {
                // Direct I/O
                let e = self.cpu.e;
                if e == 0xFF {
                    self.cpu.a = self.key_buffer.pop_front().unwrap_or(0);
                } else if e == 0xFE {
                    self.cpu.a = if self.key_buffer.is_empty() { 0 } else { 0xFF };
                } else {
                    self.vdu_char(e);
                }
                self.cpu.l = self.cpu.a;
                self.cpu.h = 0;
            }
            9 => {
                // Print $-terminated string
                let mut addr = self.cpu.de();
                loop {
                    let ch = self.cpu.read8(addr);
                    if ch == b'$' { break; }
                    self.vdu_char(ch);
                    addr = addr.wrapping_add(1);
                }
                self.cpu.a = 0;
                self.cpu.l = 0;
                self.cpu.h = 0;
            }
            10 => {
                // Read line — simplified: check if we have a CR in the buffer
                let buf_addr = self.cpu.de();
                let max_len = self.cpu.read8(buf_addr);
                // Collect chars until CR
                let mut line = Vec::new();
                loop {
                    if let Some(ch) = self.key_buffer.pop_front() {
                        if ch == 0x0D {
                            self.vdu_print("\r\n");
                            break;
                        } else if ch == 0x08 || ch == 0x7F {
                            if !line.is_empty() {
                                line.pop();
                                self.vdu_char(0x08);
                                self.vdu_char(b' ');
                                self.vdu_char(0x08);
                            }
                        } else if ch >= 0x20 && line.len() < max_len as usize {
                            line.push(ch);
                            self.vdu_char(ch);
                        }
                    } else {
                        self.waiting_for_key = true;
                        // Put chars back for retry
                        for &ch in line.iter().rev() {
                            self.key_buffer.push_front(ch);
                        }
                        return;
                    }
                }
                self.cpu.write8(buf_addr + 1, line.len() as u8);
                for (i, &ch) in line.iter().enumerate() {
                    self.cpu.write8(buf_addr + 2 + i as u16, ch);
                }
                self.cpu.a = 0;
                self.cpu.l = 0;
                self.cpu.h = 0;
            }
            11 => {
                // Console status
                self.cpu.a = if self.key_buffer.is_empty() { 0 } else { 0xFF };
                self.cpu.l = self.cpu.a;
                self.cpu.h = 0;
            }
            12 => {
                // Version
                self.cpu.a = 0x22;
                self.cpu.l = 0x22;
                self.cpu.h = 0x00;
            }
            _ => {
                // Unhandled — return 0
                self.cpu.a = 0;
                self.cpu.l = 0;
                self.cpu.h = 0;
            }
        }
    }

    fn handle_bios(&mut self, func: u8) {
        match func {
            0 | 1 => { self.warm_boot(); return; }
            2 => {
                // CONST
                self.cpu.a = if self.key_buffer.is_empty() { 0 } else { 0xFF };
            }
            3 => {
                // CONIN
                if let Some(ch) = self.key_buffer.pop_front() {
                    self.cpu.a = ch & 0x7F;
                } else {
                    self.waiting_for_key = true;
                    return; // don't pop return address
                }
            }
            4 => {
                // CONOUT
                self.vdu_char(self.cpu.c);
            }
            7 => { self.cpu.a = 0x1A; } // READER: EOF
            _ => {}
        }
        if !self.waiting_for_key {
            self.cpu.pc = self.cpu.pop16();
        }
    }

    fn warm_boot(&mut self) {
        setup_page_zero(&mut self.cpu);
        // For WASM, just stop — the JS frontend handles restart
        self.running = false;
    }

    fn vdu_char(&mut self, ch: u8) {
        match ch {
            0x0D => { self.cursor_col = 0; }
            0x0A => {
                self.cursor_row += 1;
                if self.cursor_row >= VDU_ROWS {
                    self.scroll_up();
                    self.cursor_row = VDU_ROWS - 1;
                }
            }
            0x08 => {
                if self.cursor_col > 0 { self.cursor_col -= 1; }
            }
            0x09 => {
                let next = (self.cursor_col + 8) & !7;
                while self.cursor_col < next && self.cursor_col < VDU_COLS {
                    let addr = VDU_BASE as usize + self.cursor_row * VDU_COLS + self.cursor_col;
                    self.cpu.mem[addr] = b' ';
                    self.cursor_col += 1;
                }
                if self.cursor_col >= VDU_COLS {
                    self.cursor_col = 0;
                    self.cursor_row += 1;
                    if self.cursor_row >= VDU_ROWS {
                        self.scroll_up();
                        self.cursor_row = VDU_ROWS - 1;
                    }
                }
            }
            0x20..=0x7E => {
                let addr = VDU_BASE as usize + self.cursor_row * VDU_COLS + self.cursor_col;
                self.cpu.mem[addr] = ch;
                self.cursor_col += 1;
                if self.cursor_col >= VDU_COLS {
                    self.cursor_col = 0;
                    self.cursor_row += 1;
                    if self.cursor_row >= VDU_ROWS {
                        self.scroll_up();
                        self.cursor_row = VDU_ROWS - 1;
                    }
                }
            }
            _ => {}
        }
    }

    fn vdu_print(&mut self, s: &str) {
        for &ch in s.as_bytes() {
            self.vdu_char(ch);
        }
    }

    fn scroll_up(&mut self) {
        let base = VDU_BASE as usize;
        for row in 1..VDU_ROWS {
            let src = base + row * VDU_COLS;
            let dst = base + (row - 1) * VDU_COLS;
            for col in 0..VDU_COLS {
                self.cpu.mem[dst + col] = self.cpu.mem[src + col];
            }
        }
        let last = base + (VDU_ROWS - 1) * VDU_COLS;
        for col in 0..VDU_COLS {
            self.cpu.mem[last + col] = b' ';
        }
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

fn setup_bios(cpu: &mut Cpu) {
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
