use wasm_bindgen::prelude::*;
use z80::cpu::Cpu;
use std::collections::{VecDeque, HashMap};

// Memory map
const BDOS_ENTRY: u16 = 0x0005;
const BDOS_ADDR: u16 = 0xE400;
const BIOS_BASE: u16 = 0xFA00;
const BIOS_HANDLERS: u16 = BIOS_BASE + 17 * 3;
const TPA_BASE: u16 = 0x0100;

// VDU text mode
const VDU_BASE: u16 = 0xFC00;
const VDU_COLS: usize = 40;
const VDU_ROWS: usize = 24;
const VDU_SIZE: usize = VDU_COLS * VDU_ROWS;

// HRG (High Resolution Graphics)
// Both modes use 40 bytes per row, 192 rows = 7680 bytes.
// 320×192: each bit = 2px wide on screen
// 640×192: each bit = 1px wide on screen (double horizontal resolution)
const HRG_BYTES_PER_ROW: usize = 40;
const HRG_HEIGHT: usize = 192;
const HRG_SIZE: usize = HRG_BYTES_PER_ROW * HRG_HEIGHT; // 7680 bytes

/// A Plan 9-style network connection.
struct NetConn {
    proto: NetProto,
    ctl_data: Vec<u8>,     // data written to ctl (verb + url + headers)
    req_body: Vec<u8>,     // data written to data file (request body for POST/PUT)
    resp_data: Vec<u8>,    // response data (filled by JS callback)
    resp_pos: usize,       // read position in response
    state: NetState,
}

#[derive(Clone, Copy, PartialEq)]
enum NetProto { Http, WebSocket }

#[derive(Clone, Copy, PartialEq)]
enum NetState { New, CtlWritten, RequestSent, ResponseReady, Done }

#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    cursor_row: usize,
    cursor_col: usize,
    key_buffer: VecDeque<u8>,
    waiting_for_key: bool,
    waiting_for_net: Option<u8>,  // connection ID we're waiting on
    running: bool,
    hrg: Box<[u8; HRG_SIZE]>,
    hrg_enabled: bool,
    hrg_hires: bool,
    // Network (Plan 9 model)
    net_drive: Option<u8>,           // which drive letter is network (0=A, etc.)
    net_conns: HashMap<u8, NetConn>, // connection ID → connection
    net_next_id: u8,
    // FCB tracking: maps FCB address → (conn_id, file_type)
    net_fcbs: HashMap<u16, (u8, NetFileType)>,
}

#[derive(Clone, Copy, PartialEq)]
enum NetFileType { Clone, Ctl, Data, Mem, DevCpu }

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
            waiting_for_net: None,
            running: true,
            hrg: vec![0u8; HRG_SIZE].into_boxed_slice().try_into().unwrap(),
            hrg_enabled: false,
            hrg_hires: false,
            net_drive: None,
            net_conns: HashMap::new(),
            net_next_id: 0,
            net_fcbs: HashMap::new(),
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
            if self.waiting_for_key || self.waiting_for_net.is_some() { break; }
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

    // --- HRG (High Resolution Graphics) ---

    /// Get pointer to HRG framebuffer for direct JS access.
    /// 40 bytes/row × 192 rows, MSB = leftmost pixel.
    pub fn hrg_ptr(&self) -> *const u8 { self.hrg.as_ptr() }
    /// Pixel width: 320 (lores) or 640 (hires).
    pub fn hrg_width(&self) -> usize { if self.hrg_hires { 640 } else { 320 } }
    pub fn hrg_height(&self) -> usize { HRG_HEIGHT }
    pub fn hrg_enabled(&self) -> bool { self.hrg_enabled }
    pub fn hrg_is_hires(&self) -> bool { self.hrg_hires }

    /// Set a pixel. In 320 mode, x is 0-319. In 640 mode, x is 0-639.
    pub fn hrg_set_pixel(&mut self, x: usize, y: usize) {
        let w = self.hrg_width();
        if x < w && y < HRG_HEIGHT {
            let byte_idx = y * HRG_BYTES_PER_ROW + x / 8;
            let bit = 7 - (x % 8);
            self.hrg[byte_idx] |= 1 << bit;
            self.hrg_enabled = true;
        }
    }

    /// Clear a pixel.
    pub fn hrg_clear_pixel(&mut self, x: usize, y: usize) {
        let w = self.hrg_width();
        if x < w && y < HRG_HEIGHT {
            let byte_idx = y * HRG_BYTES_PER_ROW + x / 8;
            let bit = 7 - (x % 8);
            self.hrg[byte_idx] &= !(1 << bit);
        }
    }

    /// Clear entire HRG framebuffer.
    pub fn hrg_clear(&mut self) {
        for b in self.hrg.iter_mut() { *b = 0; }
    }

    /// Toggle HRG display on/off.
    pub fn hrg_toggle(&mut self, enabled: bool) {
        self.hrg_enabled = enabled;
    }

    /// Switch between 320×192 (false) and 640×192 (true) modes.
    pub fn hrg_set_hires(&mut self, hires: bool) {
        self.hrg_hires = hires;
    }

    /// Write a byte directly to HRG memory at offset.
    pub fn hrg_write(&mut self, offset: usize, value: u8) {
        if offset < HRG_SIZE {
            self.hrg[offset] = value;
            self.hrg_enabled = true;
        }
    }

    // --- Plan 9 Network Drive ---

    /// Mount the network drive on a letter (0=A, 1=B, ...).
    pub fn net_mount(&mut self, drive: u8) {
        self.net_drive = Some(drive);
    }

    /// Check if waiting for a network response.
    pub fn needs_net(&self) -> bool { self.waiting_for_net.is_some() }

    /// Get the connection ID we're waiting on.
    pub fn waiting_net_id(&self) -> i32 {
        self.waiting_for_net.map(|id| id as i32).unwrap_or(-1)
    }

    /// Check if a connection is WebSocket (true) or HTTP (false).
    pub fn net_is_ws(&self, conn_id: u8) -> bool {
        self.net_conns.get(&conn_id).map(|c| c.proto == NetProto::WebSocket).unwrap_or(false)
    }

    /// Get the request details for a pending connection (for JS to execute).
    /// For HTTP: "VERB URL\nHeaders...". For WSK: just the URL.
    pub fn net_get_request(&self, conn_id: u8) -> String {
        match self.net_conns.get(&conn_id) {
            Some(conn) if conn.state == NetState::CtlWritten => {
                let mut req = String::from_utf8_lossy(&conn.ctl_data).to_string();
                if !conn.req_body.is_empty() {
                    req.push_str("\n\n");
                    req.push_str(&String::from_utf8_lossy(&conn.req_body));
                }
                req
            }
            _ => String::new(),
        }
    }

    /// Get just the request body bytes (for POST/PUT).
    pub fn net_get_request_body(&self, conn_id: u8) -> Vec<u8> {
        self.net_conns.get(&conn_id)
            .map(|c| c.req_body.clone())
            .unwrap_or_default()
    }

    /// JS calls this to deliver the HTTP response.
    pub fn net_set_response(&mut self, conn_id: u8, data: &[u8]) {
        if let Some(conn) = self.net_conns.get_mut(&conn_id) {
            conn.resp_data = data.to_vec();
            conn.resp_pos = 0;
            conn.state = NetState::ResponseReady;
        }
        if self.waiting_for_net == Some(conn_id) {
            self.waiting_for_net = None;
        }
    }

    /// Get and clear pending WebSocket send data.
    pub fn net_ws_take_send(&mut self, conn_id: u8) -> Vec<u8> {
        if let Some(conn) = self.net_conns.get_mut(&conn_id) {
            if conn.proto == NetProto::WebSocket {
                let data = std::mem::take(&mut conn.req_body);
                return data;
            }
        }
        Vec::new()
    }

    /// JS calls this to inject text as keystrokes (Claude RUN mode).
    pub fn inject_keys(&mut self, data: &[u8]) {
        for &b in data {
            if b == b'\n' {
                self.key_buffer.push_back(0x0D); // CR for CP/M
            } else if b != b'\r' && b < 128 {
                self.key_buffer.push_back(b);
            }
        }
        self.waiting_for_key = false;
    }

    /// JS calls this to deliver WebSocket incoming data.
    pub fn net_ws_receive(&mut self, conn_id: u8, data: &[u8]) {
        if let Some(conn) = self.net_conns.get_mut(&conn_id) {
            conn.resp_data.extend_from_slice(data);
            if conn.state != NetState::ResponseReady {
                conn.state = NetState::ResponseReady;
            }
        }
        if self.waiting_for_net == Some(conn_id) {
            self.waiting_for_net = None;
        }
    }
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
            // File operations — network drive intercept
            15 => self.bdos_open(),
            16 => self.bdos_close(),
            20 => self.bdos_read_seq(),
            21 => self.bdos_write_seq(),
            22 => self.bdos_make(),
            _ => {
                self.cpu.a = 0;
                self.cpu.l = 0;
                self.cpu.h = 0;
            }
        }
    }

    // --- BDOS File Operations (with network drive support) ---

    fn fcb_name(&self, fcb: u16) -> [u8; 8] {
        let mut n = [0u8; 8];
        for i in 0..8 { n[i] = self.cpu.read8(fcb + 1 + i as u16); }
        n
    }
    fn fcb_ext(&self, fcb: u16) -> [u8; 3] {
        let mut e = [0u8; 3];
        for i in 0..3 { e[i] = self.cpu.read8(fcb + 9 + i as u16); }
        e
    }
    fn fcb_drive(&self, fcb: u16) -> u8 { self.cpu.read8(fcb) }

    fn is_net_drive(&self, fcb_drive: u8) -> bool {
        let Some(net_drv) = self.net_drive else { return false };
        let d = if fcb_drive == 0 { 0 /* TODO: current disk */ } else { fcb_drive - 1 };
        d == net_drv
    }

    /// Parse network filename: CLONE.HTTP, CLONE.WS, 0.CTL, 0.DATA, etc.
    fn parse_net_filename(&self, fcb: u16) -> Option<(NetFileType, Option<u8>, NetProto, String)> {
        let name = self.fcb_name(fcb);
        let ext = self.fcb_ext(fcb);
        let name_str: String = name.iter().map(|&b| (b & 0x7F) as char).collect::<String>();
        let name_str = name_str.trim();
        let ext_str: String = ext.iter().map(|&b| (b & 0x7F) as char).collect::<String>();
        let ext_str = ext_str.trim();
        let ext_owned = ext_str.to_string();

        if name_str == "CLONE" {
            let proto = match ext_str {
                "WWW" => NetProto::Http,
                "WSK" => NetProto::WebSocket,
                _ => return None,
            };
            return Some((NetFileType::Clone, None, proto, ext_owned));
        }

        if name_str == "MEM" && matches!(ext_str, "0" | "1" | "2" | "3" | "VDU") {
            return Some((NetFileType::Mem, None, NetProto::Http, ext_owned));
        }

        if name_str == "DEV" && ext_str == "CPU" {
            return Some((NetFileType::DevCpu, None, NetProto::Http, ext_owned));
        }

        let id: u8 = name_str.parse().ok()?;
        match ext_str {
            "CTL" => Some((NetFileType::Ctl, Some(id), NetProto::Http, ext_owned)),
            "DATA" => Some((NetFileType::Data, Some(id), NetProto::Http, ext_owned)),
            _ => None,
        }
    }

    fn bdos_open(&mut self) {
        let fcb = self.cpu.de();
        let drv = self.fcb_drive(fcb);

        if self.is_net_drive(drv) {
            if let Some((ftype, conn_id, proto, ext_s)) = self.parse_net_filename(fcb) {
                match ftype {
                    NetFileType::Clone => {
                        // Allocate new connection — ID returned on first read
                        let id = self.net_next_id;
                        self.net_next_id += 1;
                        // Pre-create the connection with the right protocol
                        self.net_conns.insert(id, NetConn {
                            proto,
                            ctl_data: Vec::new(), req_body: Vec::new(),
                            resp_data: Vec::new(), resp_pos: 0, state: NetState::New,
                        });
                        self.net_fcbs.insert(fcb, (id, NetFileType::Clone));
                        self.cpu.a = 0;
                    }
                    NetFileType::Ctl => {
                        if let Some(id) = conn_id {
                            self.net_fcbs.insert(fcb, (id, NetFileType::Ctl));
                            self.cpu.a = 0;
                        } else { self.cpu.a = 0xFF; }
                    }
                    NetFileType::Data => {
                        if let Some(id) = conn_id {
                            self.net_fcbs.insert(fcb, (id, NetFileType::Data));
                            self.cpu.a = 0;
                        } else { self.cpu.a = 0xFF; }
                    }
                    NetFileType::Mem | NetFileType::DevCpu => {
                        // Store ext in the FCB tracking so read/write knows the bank
                        self.net_fcbs.insert(fcb, (0, ftype));
                        self.cpu.a = 0;
                    }
                }
            } else {
                self.cpu.a = 0xFF;
            }
        } else {
            // Non-network: not supported in WASM (no filesystem)
            self.cpu.a = 0xFF;
        }
        self.cpu.l = self.cpu.a;
        self.cpu.h = 0;
    }

    fn bdos_close(&mut self) {
        let fcb = self.cpu.de();
        if let Some((conn_id, ftype)) = self.net_fcbs.remove(&fcb) {
            if ftype == NetFileType::Ctl {
                // Closing ctl tears down the connection
                self.net_conns.remove(&conn_id);
            }
            self.cpu.a = 0;
        } else {
            self.cpu.a = 0xFF;
        }
        self.cpu.l = self.cpu.a;
        self.cpu.h = 0;
    }

    fn bdos_read_seq(&mut self) {
        let fcb = self.cpu.de();
        if let Some(&(conn_id, ftype)) = self.net_fcbs.get(&fcb) {
            match ftype {
                NetFileType::Clone => {
                    // Read from clone returns the connection ID as text
                    let id = conn_id;
                    let id_str = format!("{}\r\n", id);
                    let mut buf = [0x1Au8; 128];
                    let bytes = id_str.as_bytes();
                    buf[..bytes.len()].copy_from_slice(bytes);
                    let dma = 0x0080u16; // default DMA
                    for i in 0..128 {
                        self.cpu.write8(dma + i as u16, buf[i]);
                    }
                    self.cpu.a = 0;
                }
                NetFileType::Data => {
                    // Read response data
                    if let Some(conn) = self.net_conns.get_mut(&conn_id) {
                        if conn.state == NetState::CtlWritten {
                            // Need to send request — signal JS
                            conn.state = NetState::RequestSent;
                            self.waiting_for_net = Some(conn_id);
                            return; // don't pop return addr, retry later
                        }
                        if conn.state == NetState::ResponseReady && conn.resp_pos < conn.resp_data.len() {
                            let mut buf = [0x1Au8; 128];
                            let remaining = conn.resp_data.len() - conn.resp_pos;
                            let n = remaining.min(128);
                            buf[..n].copy_from_slice(&conn.resp_data[conn.resp_pos..conn.resp_pos + n]);
                            conn.resp_pos += 128;
                            let dma = 0x0080u16;
                            for i in 0..128 {
                                self.cpu.write8(dma + i as u16, buf[i]);
                            }
                            self.cpu.a = 0;
                        } else if conn.state == NetState::ResponseReady {
                            self.cpu.a = 1; // EOF
                        } else {
                            // Waiting for response
                            self.waiting_for_net = Some(conn_id);
                            return;
                        }
                    } else {
                        self.cpu.a = 1;
                    }
                }
                NetFileType::Mem => {
                    let fcb_ext = self.fcb_ext(fcb);
                    let ext: String = fcb_ext.iter().map(|&b| (b & 0x7F) as char).collect::<String>();
                    let ext = ext.trim();
                    let (base, size) = match ext {
                        "0" => (0x0000usize, 0x4000usize),
                        "1" => (0x4000, 0x4000),
                        "2" => (0x8000, 0x4000),
                        "3" => (0xC000, 0x4000),
                        _ => (0xFC00, 0x0400), // VDU
                    };
                    let cr = self.cpu.read8(fcb + 32) as usize;
                    let offset = base + cr * 128;
                    if offset + 128 <= base + size {
                        let dma = 0x0080u16;
                        for i in 0..128 {
                            self.cpu.write8(dma + i as u16, self.cpu.mem[offset + i]);
                        }
                        self.cpu.a = 0;
                    } else {
                        self.cpu.a = 1;
                    }
                }
                NetFileType::DevCpu => {
                    let text = format!(
                        "A={:02X} F={:02X} BC={:04X} DE={:04X} HL={:04X}\r\nSP={:04X} PC={:04X} IX={:04X} IY={:04X}\r\n",
                        self.cpu.a, self.cpu.f, self.cpu.bc(), self.cpu.de(), self.cpu.hl(),
                        self.cpu.sp, self.cpu.pc, self.cpu.ix, self.cpu.iy
                    );
                    let dma = 0x0080u16;
                    let mut buf = [0x1Au8; 128];
                    let n = text.len().min(127);
                    buf[..n].copy_from_slice(&text.as_bytes()[..n]);
                    for i in 0..128 {
                        self.cpu.write8(dma + i as u16, buf[i]);
                    }
                    self.cpu.a = 0;
                }
                _ => { self.cpu.a = 1; }
            }
        } else {
            self.cpu.a = 1;
        }
        self.cpu.l = self.cpu.a;
        self.cpu.h = 0;
    }

    fn bdos_write_seq(&mut self) {
        let fcb = self.cpu.de();
        if let Some(&(conn_id, ftype)) = self.net_fcbs.get(&fcb) {
            match ftype {
                NetFileType::Ctl => {
                    // Write to ctl: accumulate verb + URL + headers
                    if let Some(conn) = self.net_conns.get_mut(&conn_id) {
                        let dma = 0x0080u16;
                        for i in 0..128 {
                            let b = self.cpu.read8(dma + i as u16);
                            if b == 0x1A { break; } // CP/M EOF
                            conn.ctl_data.push(b);
                        }
                        conn.state = NetState::CtlWritten;
                        self.cpu.a = 0;
                    } else { self.cpu.a = 2; }
                }
                NetFileType::Data => {
                    // Write to data: request body (for POST/PUT)
                    if let Some(conn) = self.net_conns.get_mut(&conn_id) {
                        let dma = 0x0080u16;
                        for i in 0..128 {
                            let b = self.cpu.read8(dma + i as u16);
                            if b == 0x1A { break; }
                            conn.req_body.push(b);
                        }
                        self.cpu.a = 0;
                    } else { self.cpu.a = 2; }
                }
                NetFileType::Mem => {
                    let fcb_ext = self.fcb_ext(fcb);
                    let ext: String = fcb_ext.iter().map(|&b| (b & 0x7F) as char).collect::<String>();
                    let ext = ext.trim();
                    let (base, size) = match ext {
                        "0" => (0x0000usize, 0x4000usize),
                        "1" => (0x4000, 0x4000),
                        "2" => (0x8000, 0x4000),
                        "3" => (0xC000, 0x4000),
                        _ => (0xFC00, 0x0400),
                    };
                    let cr = self.cpu.read8(fcb + 32) as usize;
                    let offset = base + cr * 128;
                    if offset + 128 <= base + size {
                        let dma = 0x0080u16;
                        for i in 0..128 {
                            self.cpu.mem[offset + i] = self.cpu.read8(dma + i as u16);
                        }
                    }
                    self.cpu.a = 0;
                }
                _ => { self.cpu.a = 2; }
            }
        } else {
            self.cpu.a = 2;
        }
        self.cpu.l = self.cpu.a;
        self.cpu.h = 0;
    }

    fn bdos_make(&mut self) {
        // Make = same as open for network files
        self.bdos_open();
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
