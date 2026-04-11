use std::io::{self, Write};

/// 380Z VDU base address in Z80 memory.
pub const VDU_BASE: u16 = 0xFC00;
pub const VDU_COLS: usize = 40;
pub const VDU_ROWS: usize = 24;
pub const VDU_SIZE: usize = VDU_COLS * VDU_ROWS;

/// RM 380Z Video Display Unit — memory-mapped 40×24 character display.
/// The VDU buffer lives in cpu.mem at FC00h. This struct tracks cursor
/// state and handles rendering to the host terminal.
pub struct Vdu {
    pub cursor_row: usize,
    pub cursor_col: usize,
    dirty: bool,
}

impl Vdu {
    pub fn new() -> Self {
        Vdu {
            cursor_row: 0,
            cursor_col: 0,
            dirty: true,
        }
    }

    /// Fill VDU RAM with spaces and home cursor.
    pub fn init(&mut self, mem: &mut [u8; 0x10000]) {
        for i in 0..VDU_SIZE {
            mem[VDU_BASE as usize + i] = b' ';
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.dirty = true;
    }

    /// Clear screen: fill with spaces, home cursor.
    pub fn clear(&mut self, mem: &mut [u8; 0x10000]) {
        self.init(mem);
    }

    /// Write a character at cursor position, advance cursor.
    pub fn write_char(&mut self, mem: &mut [u8; 0x10000], ch: u8) {
        match ch {
            0x0D => {
                // CR: return to column 0
                self.cursor_col = 0;
            }
            0x0A => {
                // LF: move down one line, scroll if needed
                self.cursor_row += 1;
                if self.cursor_row >= VDU_ROWS {
                    self.scroll_up(mem);
                    self.cursor_row = VDU_ROWS - 1;
                }
            }
            0x08 => {
                // BS: move cursor left
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            0x07 => {
                // BEL: just ignore (or could beep)
            }
            0x09 => {
                // TAB: advance to next 8-column boundary
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
            0x0C => {
                // FF: form feed = clear screen
                self.clear(mem);
            }
            0x20..=0x7E => {
                // Printable: write to VDU RAM and advance
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
            _ => {} // ignore other control chars
        }
        self.dirty = true;
    }

    /// Scroll the entire screen up by one line.
    fn scroll_up(&mut self, mem: &mut [u8; 0x10000]) {
        let base = VDU_BASE as usize;
        // Copy rows 1..23 to rows 0..22
        for row in 1..VDU_ROWS {
            let src = base + row * VDU_COLS;
            let dst = base + (row - 1) * VDU_COLS;
            for col in 0..VDU_COLS {
                mem[dst + col] = mem[src + col];
            }
        }
        // Clear bottom row
        let last = base + (VDU_ROWS - 1) * VDU_COLS;
        for col in 0..VDU_COLS {
            mem[last + col] = b' ';
        }
        self.dirty = true;
    }

    /// Render VDU RAM to the crossterm terminal.
    pub fn render(&mut self, mem: &[u8; 0x10000]) {
        if !self.dirty { return; }
        self.dirty = false;

        let stdout = io::stdout();
        let mut out = stdout.lock();

        // Move to top-left
        let _ = write!(out, "\x1b[H");

        let base = VDU_BASE as usize;
        for row in 0..VDU_ROWS {
            for col in 0..VDU_COLS {
                let ch = mem[base + row * VDU_COLS + col];
                let c = if ch >= 0x20 && ch <= 0x7E { ch } else { b' ' };
                let _ = out.write_all(&[c]);
            }
            if row < VDU_ROWS - 1 {
                let _ = write!(out, "\r\n");
            }
        }

        // Position cursor
        let _ = write!(out, "\x1b[{};{}H", self.cursor_row + 1, self.cursor_col + 1);
        let _ = out.flush();
    }
}
