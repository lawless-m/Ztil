use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io::{self, Write};
use std::time::Duration;

pub struct Console {
    // Nothing stateful yet — all I/O goes directly through crossterm/stdout
}

impl Console {
    pub fn new() -> Self {
        Console {}
    }

    /// Print a single character to the terminal.
    pub fn write_char(&mut self, ch: u8) {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        if ch == 0x0D {
            let _ = out.write_all(b"\r\n");
        } else if ch >= 0x20 || ch == 0x07 || ch == 0x08 || ch == 0x09 || ch == 0x0A {
            let _ = out.write_all(&[ch]);
        }
        let _ = out.flush();
    }

    /// Print a string (for convenience).
    pub fn write_str(&mut self, s: &str) {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = out.write_all(s.as_bytes());
        let _ = out.flush();
    }

    /// Check if a key is available (non-blocking). Returns true if ready.
    pub fn key_ready(&self) -> bool {
        event::poll(Duration::from_millis(0)).unwrap_or(false)
    }

    /// Read one key (blocking). Returns ASCII code.
    pub fn read_key(&mut self) -> u8 {
        // Fallback for non-TTY (piped input)
        if !crossterm::terminal::is_raw_mode_enabled().unwrap_or(false) {
            use std::io::Read;
            let mut buf = [0u8; 1];
            if std::io::stdin().read(&mut buf).unwrap_or(0) == 1 {
                if buf[0] == b'\n' { return 0x0D; }
                return buf[0];
            }
            return 0x1A; // EOF → Ctrl-Z
        }
        loop {
            if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    match code {
                        KeyCode::Char('c') => return 0x03,
                        KeyCode::Char('z') => return 0x1A,
                        KeyCode::Char('s') => return 0x13,
                        KeyCode::Char(c) => return (c as u8) & 0x1F,
                        _ => {}
                    }
                }
                match code {
                    KeyCode::Enter => return 0x0D,
                    KeyCode::Backspace => return 0x08,
                    KeyCode::Esc => return 0x1B,
                    KeyCode::Tab => return 0x09,
                    KeyCode::Char(c) if c.is_ascii() => return c as u8,
                    _ => {}
                }
            }
        }
    }

    /// Read one key (non-blocking). Returns Some(ascii) or None.
    pub fn try_read_key(&mut self) -> Option<u8> {
        if self.key_ready() {
            Some(self.read_key())
        } else {
            None
        }
    }

    /// BDOS function 10: buffered line input.
    /// Buffer at addr: [max_len] [actual_len] [chars...]
    /// We handle this entirely in Rust for simplicity.
    pub fn read_line(&mut self, max_len: u8) -> Vec<u8> {
        let mut buf = Vec::new();
        loop {
            let ch = self.read_key();
            match ch {
                0x0D => {
                    self.write_char(0x0D);
                    return buf;
                }
                0x08 | 0x7F => {
                    if !buf.is_empty() {
                        buf.pop();
                        self.write_str("\x08 \x08");
                    }
                }
                0x03 => {
                    // Ctrl-C: return empty (warm boot signal)
                    return Vec::new();
                }
                _ if ch >= 0x20 && buf.len() < max_len as usize => {
                    buf.push(ch);
                    self.write_char(ch);
                }
                _ => {}
            }
        }
    }
}
