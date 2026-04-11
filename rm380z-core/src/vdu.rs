/// 380Z VDU base address in Z80 memory.
pub const VDU_BASE: u16 = 0xFC00;
pub const VDU_COLS: usize = 40;
pub const VDU_ROWS: usize = 24;
pub const VDU_SIZE: usize = VDU_COLS * VDU_ROWS;

/// RM 380Z Video Display Unit — memory-mapped 40×24 character display.
/// The VDU buffer lives in cpu.mem at FC00h.
pub struct Vdu {
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl Vdu {
    pub fn new() -> Self {
        Vdu { cursor_row: 0, cursor_col: 0 }
    }

    pub fn init(&mut self, mem: &mut [u8; 0x10000]) {
        for i in 0..VDU_SIZE {
            mem[VDU_BASE as usize + i] = b' ';
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    pub fn clear(&mut self, mem: &mut [u8; 0x10000]) {
        self.init(mem);
    }

    pub fn write_char(&mut self, mem: &mut [u8; 0x10000], ch: u8) {
        match ch {
            0x0D => { self.cursor_col = 0; }
            0x0A => {
                self.cursor_row += 1;
                if self.cursor_row >= VDU_ROWS {
                    self.scroll_up(mem);
                    self.cursor_row = VDU_ROWS - 1;
                }
            }
            0x08 => {
                if self.cursor_col > 0 { self.cursor_col -= 1; }
            }
            0x07 => {}
            0x09 => {
                let next = (self.cursor_col + 8) & !7;
                while self.cursor_col < next && self.cursor_col < VDU_COLS {
                    let addr = VDU_BASE as usize + self.cursor_row * VDU_COLS + self.cursor_col;
                    mem[addr] = b' ';
                    self.cursor_col += 1;
                }
                if self.cursor_col >= VDU_COLS {
                    self.cursor_col = 0;
                    self.cursor_row += 1;
                    if self.cursor_row >= VDU_ROWS {
                        self.scroll_up(mem);
                        self.cursor_row = VDU_ROWS - 1;
                    }
                }
            }
            0x0C => { self.clear(mem); }
            0x20..=0x7E => {
                let addr = VDU_BASE as usize + self.cursor_row * VDU_COLS + self.cursor_col;
                mem[addr] = ch;
                self.cursor_col += 1;
                if self.cursor_col >= VDU_COLS {
                    self.cursor_col = 0;
                    self.cursor_row += 1;
                    if self.cursor_row >= VDU_ROWS {
                        self.scroll_up(mem);
                        self.cursor_row = VDU_ROWS - 1;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn write_str(&mut self, mem: &mut [u8; 0x10000], s: &str) {
        for &ch in s.as_bytes() {
            self.write_char(mem, ch);
        }
    }

    fn scroll_up(&mut self, mem: &mut [u8; 0x10000]) {
        let base = VDU_BASE as usize;
        for row in 1..VDU_ROWS {
            let src = base + row * VDU_COLS;
            let dst = base + (row - 1) * VDU_COLS;
            for col in 0..VDU_COLS {
                mem[dst + col] = mem[src + col];
            }
        }
        let last = base + (VDU_ROWS - 1) * VDU_COLS;
        for col in 0..VDU_COLS {
            mem[last + col] = b' ';
        }
    }
}
