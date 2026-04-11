use std::collections::HashMap;

/// Plan 9-style network connection state (platform-independent).
/// The actual I/O (HTTP, WebSocket, subprocess) is handled by the platform layer.
pub struct NetState {
    pub conns: HashMap<u8, Connection>,
    pub next_id: u8,
    pub claude_prompt: Vec<u8>,
    pub claude_response: Vec<u8>,
    pub claude_resp_pos: usize,
    pub claude_active: bool,
    pub api_key: String,
    pub model: String,
}

pub struct Connection {
    pub ctl_data: Vec<u8>,
    pub req_body: Vec<u8>,
    pub resp_data: Vec<u8>,
    pub resp_pos: usize,
    pub ready: bool,
}

impl NetState {
    pub fn new() -> Self {
        NetState {
            conns: HashMap::new(),
            next_id: 0,
            claude_prompt: Vec::new(),
            claude_response: Vec::new(),
            claude_resp_pos: 0,
            claude_active: false,
            api_key: String::new(),
            model: "claude-sonnet-4-5".to_string(),
        }
    }

    pub fn clone_conn(&mut self) -> u8 {
        let id = self.next_id;
        self.next_id += 1;
        self.conns.insert(id, Connection {
            ctl_data: Vec::new(), req_body: Vec::new(),
            resp_data: Vec::new(), resp_pos: 0, ready: false,
        });
        id
    }

    pub fn write_ctl(&mut self, id: u8, data: &[u8]) {
        if let Some(conn) = self.conns.get_mut(&id) {
            for &b in data { if b == 0x1A { break; } conn.ctl_data.push(b); }
        }
    }

    pub fn write_data(&mut self, id: u8, data: &[u8]) {
        if let Some(conn) = self.conns.get_mut(&id) {
            for &b in data { if b == 0x1A { break; } conn.req_body.push(b); }
        }
    }

    pub fn read_data(&mut self, id: u8) -> Option<[u8; 128]> {
        let conn = self.conns.get_mut(&id)?;
        if conn.resp_pos >= conn.resp_data.len() { return None; }
        let mut buf = [0x1Au8; 128];
        let remaining = conn.resp_data.len() - conn.resp_pos;
        let n = remaining.min(128);
        buf[..n].copy_from_slice(&conn.resp_data[conn.resp_pos..conn.resp_pos + n]);
        conn.resp_pos += 128;
        Some(buf)
    }

    pub fn close_conn(&mut self, id: u8) {
        self.conns.remove(&id);
    }

    pub fn open_claude(&mut self) {
        self.claude_prompt.clear();
        self.claude_response.clear();
        self.claude_resp_pos = 0;
        self.claude_active = true;
    }

    pub fn write_claude(&mut self, data: &[u8]) {
        for &b in data { if b == 0x1A { break; } self.claude_prompt.push(b); }
    }

    /// Read claude response. Returns None if no response yet or EOF.
    /// The platform layer must call set_claude_response() first.
    pub fn read_claude(&mut self) -> Option<[u8; 128]> {
        if self.claude_resp_pos >= self.claude_response.len() { return None; }
        let mut buf = [0x1Au8; 128];
        let remaining = self.claude_response.len() - self.claude_resp_pos;
        let n = remaining.min(128);
        buf[..n].copy_from_slice(&self.claude_response[self.claude_resp_pos..self.claude_resp_pos + n]);
        self.claude_resp_pos += 128;
        Some(buf)
    }

    pub fn set_claude_response(&mut self, data: Vec<u8>) {
        self.claude_response = data;
        self.claude_resp_pos = 0;
    }

    pub fn get_prompt(&self) -> String {
        String::from_utf8_lossy(&self.claude_prompt).trim().to_string()
    }

    pub fn close_claude(&mut self) {
        self.claude_active = false;
    }

    pub fn set_api_key(&mut self, data: &[u8]) {
        let key: String = data.iter().take_while(|&&b| b != 0x1A && b != b'\r' && b != b'\n')
            .map(|&b| b as char).collect();
        self.api_key = key.trim().to_string();
    }

    pub fn set_model(&mut self, data: &[u8]) {
        let model: String = data.iter().take_while(|&&b| b != 0x1A && b != b'\r' && b != b'\n')
            .map(|&b| b as char).collect();
        let model = model.trim().to_string();
        if !model.is_empty() { self.model = model; }
    }

    pub fn get_model_info(&self) -> String {
        format!("{}\r\n", self.model)
    }
}
