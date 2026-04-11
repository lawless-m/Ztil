use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;
use std::time::Duration;

/// Console handles keyboard input. Supports injected keystrokes (from Claude RUN).
pub struct Console {
    injected: VecDeque<u8>,
}

impl Console {
    pub fn new() -> Self {
        Console { injected: VecDeque::new() }
    }

    /// Inject a key into the buffer (for Claude RUN mode).
    pub fn inject_key(&mut self, ch: u8) {
        self.injected.push_back(ch);
    }

    /// Check if a key is available (non-blocking).
    pub fn key_ready(&self) -> bool {
        !self.injected.is_empty() || event::poll(Duration::from_millis(0)).unwrap_or(false)
    }

    /// Read one key (blocking). Injected keys take priority.
    pub fn read_key(&mut self) -> u8 {
        if let Some(ch) = self.injected.pop_front() {
            return ch;
        }
        // Fallback for non-TTY (piped input)
        if !crossterm::terminal::is_raw_mode_enabled().unwrap_or(false) {
            use std::io::Read;
            let mut buf = [0u8; 1];
            if std::io::stdin().read(&mut buf).unwrap_or(0) == 1 {
                if buf[0] == b'\n' { return 0x0D; }
                return buf[0];
            }
            return 0x1A;
        }
        loop {
            if let Some(ch) = self.injected.pop_front() {
                return ch;
            }
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

    /// Read one key (non-blocking).
    pub fn try_read_key(&mut self) -> Option<u8> {
        if let Some(ch) = self.injected.pop_front() {
            return Some(ch);
        }
        if event::poll(Duration::from_millis(0)).unwrap_or(false) {
            Some(self.read_key())
        } else {
            None
        }
    }
}
