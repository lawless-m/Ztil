use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Console handles keyboard input ONLY. All display output goes through the VDU.
pub struct Console {}

impl Console {
    pub fn new() -> Self {
        Console {}
    }

    /// Check if a key is available (non-blocking).
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
            return 0x1A; // EOF
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
}
