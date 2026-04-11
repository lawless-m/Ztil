pub use rm380z_core::vdu::{VDU_BASE, VDU_COLS, VDU_ROWS, VDU_SIZE};
use std::io::{self, Write};

/// VDU with crossterm terminal rendering.
pub struct Vdu {
    pub inner: rm380z_core::vdu::Vdu,
    dirty: bool,
}

impl Vdu {
    pub fn new() -> Self {
        Vdu { inner: rm380z_core::vdu::Vdu::new(), dirty: true }
    }

    pub fn init(&mut self, mem: &mut [u8; 0x10000]) {
        self.inner.init(mem);
        self.dirty = true;
    }

    pub fn clear(&mut self, mem: &mut [u8; 0x10000]) {
        self.inner.clear(mem);
        self.dirty = true;
    }

    pub fn write_char(&mut self, mem: &mut [u8; 0x10000], ch: u8) {
        self.inner.write_char(mem, ch);
        self.dirty = true;
    }

    pub fn write_str(&mut self, mem: &mut [u8; 0x10000], s: &str) {
        self.inner.write_str(mem, s);
        self.dirty = true;
    }

    pub fn render(&mut self, mem: &[u8; 0x10000]) {
        if !self.dirty { return; }
        self.dirty = false;

        let stdout = io::stdout();
        let mut out = stdout.lock();
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
        let _ = write!(out, "\x1b[{};{}H", self.inner.cursor_row + 1, self.inner.cursor_col + 1);
        let _ = out.flush();
    }
}

impl std::ops::Deref for Vdu {
    type Target = rm380z_core::vdu::Vdu;
    fn deref(&self) -> &Self::Target { &self.inner }
}
