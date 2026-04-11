use wasm_bindgen::prelude::*;
use z80::cpu::Cpu;
use rm380z_core::vdu::Vdu;
use rm380z_core::page_zero::{self, BDOS_ENTRY, BDOS_ADDR, BIOS_HANDLERS, CCP_ENTRY};
use rm380z_core::bdos as bdos_core;
use rm380z_core::net::NetState;
use std::collections::{VecDeque, HashMap};

// HRG
const HRG_BYTES_PER_ROW: usize = 40;
const HRG_HEIGHT: usize = 192;
const HRG_SIZE: usize = HRG_BYTES_PER_ROW * HRG_HEIGHT;

#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    vdu: Vdu,
    key_buffer: VecDeque<u8>,
    waiting_for_key: bool,
    waiting_for_net: Option<u8>,
    waiting_for_claude: bool,
    claude_inject_keys: bool, // true = CLAUDE.RUN mode (inject as keystrokes)
    running: bool,
    hrg: Box<[u8; HRG_SIZE]>,
    hrg_enabled: bool,
    hrg_hires: bool,
    net_drive: Option<u8>,
    net: NetState,
    /// FCB tracking: maps FCB address → (conn_id_or_0, file_type)
    net_fcbs: HashMap<u16, (u8, &'static str)>,
    files: HashMap<String, Vec<u8>>,
    /// Open file handles: FCB addr -> (filename, position)
    open_handles: HashMap<u16, (String, usize)>,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        let mut cpu = Cpu::new();
        page_zero::setup_page_zero(&mut cpu);
        page_zero::setup_bios(&mut cpu);

        let mut vdu = Vdu::new();
        vdu.init(&mut cpu.mem);

        let mut emu = Emulator {
            cpu, vdu,
            key_buffer: VecDeque::new(),
            waiting_for_key: false,
            waiting_for_net: None,
            waiting_for_claude: false,
            claude_inject_keys: false,
            running: true,
            hrg: vec![0u8; HRG_SIZE].into_boxed_slice().try_into().unwrap(),
            hrg_enabled: false,
            hrg_hires: false,
            net_drive: None,
            net: NetState::new(),
            net_fcbs: HashMap::new(),
            files: HashMap::new(),
            open_handles: HashMap::new(),
        };
        emu.vdu.write_str(&mut emu.cpu.mem, "RM 380Z CP/M 2.2\r\n\r\nA>");
        emu.running = false; // idle — JS CCP handles the prompt
        emu
    }

    pub fn load_com(&mut self, data: &[u8]) {
        page_zero::load_com(&mut self.cpu, data, "");
        self.running = true;
        self.waiting_for_key = false;
        self.waiting_for_claude = false;
        self.waiting_for_net = None;
        for i in 0..rm380z_core::vdu::VDU_SIZE {
            self.cpu.mem[rm380z_core::vdu::VDU_BASE as usize + i] = b' ';
        }
        self.vdu.cursor_row = 0;
        self.vdu.cursor_col = 0;
    }

    pub fn run(&mut self, max_steps: u32) -> u32 {
        if !self.running { return 0; }
        let mut steps = 0u32;
        while steps < max_steps {
            if self.waiting_for_key || self.waiting_for_net.is_some() || self.waiting_for_claude { break; }
            if !self.running { break; }

            if self.cpu.pc >= BIOS_HANDLERS && self.cpu.pc < BIOS_HANDLERS + 17 {
                let func = (self.cpu.pc - BIOS_HANDLERS) as u8;
                self.handle_bios(func);
                if self.waiting_for_key { break; }
                continue;
            }

            if self.cpu.halted {
                self.cpu.halted = false;
                self.go_idle();
                break;
            }

            if self.cpu.pc == BDOS_ENTRY || self.cpu.pc == BDOS_ADDR {
                let func = self.cpu.c;
                self.handle_bdos(func);
                if func != 0 && !self.waiting_for_key && self.waiting_for_net.is_none() {
                    self.cpu.pc = self.cpu.pop16();
                }
                if self.waiting_for_key || self.waiting_for_net.is_some() || self.waiting_for_claude { break; }
                continue;
            }

            self.cpu.step();
            steps += 1;
        }
        steps
    }

    pub fn key_press(&mut self, ch: u8) {
        self.key_buffer.push_back(ch);
        self.waiting_for_key = false;
    }

    pub fn inject_keys(&mut self, data: &[u8]) {
        for &b in data {
            if b == b'\n' { self.key_buffer.push_back(0x0D); }
            else if b != b'\r' && b < 128 { self.key_buffer.push_back(b); }
        }
        self.waiting_for_key = false;
    }

    pub fn vdu_ptr(&self) -> *const u8 {
        &self.cpu.mem[rm380z_core::vdu::VDU_BASE as usize] as *const u8
    }
    /// Store a .COM file for CCP to find.
    pub fn add_file(&mut self, name: &str, data: &[u8]) {
        self.files.insert(name.to_uppercase(), data.to_vec());
    }

    /// Get a stored file's contents.
    pub fn get_file(&self, name: &str) -> Option<Vec<u8>> {
        self.files.get(&name.to_uppercase()).cloned()
    }

    /// List stored file names.
    pub fn list_files(&self) -> Vec<String> {
        let mut names: Vec<String> = self.files.keys().cloned().collect();
        names.sort();
        names
    }

    /// Load a .COM by name with optional arguments. Returns true if found.
    pub fn load_com_by_name(&mut self, name: &str, args: &str) -> bool {
        let key = name.to_uppercase();
        let key = if key.ends_with(".COM") { key } else { format!("{}.COM", key) };
        if let Some(data) = self.files.get(&key).cloned() {
            let tail = if args.is_empty() { String::new() } else { format!(" {}", args) };
            page_zero::load_com(&mut self.cpu, &data, &tail);
            self.running = true;
            self.waiting_for_key = false;
            self.waiting_for_claude = false;
            self.waiting_for_net = None;
            // Parse args into FCBs
            let parts: Vec<&str> = args.split_whitespace().collect();
            if let Some(arg1) = parts.first() {
                rm380z_core::fcb::parse_into(&mut self.cpu, 0x005C, arg1);
            }
            if let Some(arg2) = parts.get(1) {
                rm380z_core::fcb::parse_into(&mut self.cpu, 0x006C, arg2);
            }
            // Clear VDU for fresh program
            for i in 0..rm380z_core::vdu::VDU_SIZE {
                self.cpu.mem[rm380z_core::vdu::VDU_BASE as usize + i] = b' ';
            }
            self.vdu.cursor_row = 0;
            self.vdu.cursor_col = 0;
            true
        } else {
            false
        }
    }

    /// Write a character to the VDU (for JS-side CCP echo).
    pub fn vdu_write(&mut self, ch: u8) {
        self.vdu.write_char(&mut self.cpu.mem, ch);
    }

    /// Write a string to the VDU.
    pub fn vdu_print(&mut self, s: &str) {
        self.vdu.write_str(&mut self.cpu.mem, s);
    }

    pub fn cursor_row(&self) -> usize { self.vdu.cursor_row }
    pub fn cursor_col(&self) -> usize { self.vdu.cursor_col }
    pub fn needs_key(&self) -> bool { self.waiting_for_key }
    pub fn needs_claude(&self) -> bool { self.waiting_for_claude }
    pub fn is_running(&self) -> bool { self.running }

    /// Get the pending Claude prompt (for JS to send via WebSocket).
    pub fn claude_get_prompt(&self) -> String {
        self.net.get_prompt()
    }

    /// JS delivers Claude's response. If inject mode, feeds as keystrokes.
    pub fn claude_set_response(&mut self, text: &str) {
        if self.claude_inject_keys {
            self.inject_keys(text.as_bytes());
        } else {
            let mut data = text.replace('\n', "\r\n").into_bytes();
            data.extend_from_slice(b"\r\n");
            self.net.set_claude_response(data);
        }
        self.waiting_for_claude = false;
    }

    // --- HRG ---
    pub fn hrg_ptr(&self) -> *const u8 { self.hrg.as_ptr() }
    pub fn hrg_width(&self) -> usize { if self.hrg_hires { 640 } else { 320 } }
    pub fn hrg_height(&self) -> usize { HRG_HEIGHT }
    pub fn hrg_enabled(&self) -> bool { self.hrg_enabled }
    pub fn hrg_is_hires(&self) -> bool { self.hrg_hires }
    pub fn hrg_set_pixel(&mut self, x: usize, y: usize) {
        let w = self.hrg_width();
        if x < w && y < HRG_HEIGHT {
            self.hrg[y * HRG_BYTES_PER_ROW + x / 8] |= 1 << (7 - (x % 8));
            self.hrg_enabled = true;
        }
    }
    pub fn hrg_clear_pixel(&mut self, x: usize, y: usize) {
        let w = self.hrg_width();
        if x < w && y < HRG_HEIGHT {
            self.hrg[y * HRG_BYTES_PER_ROW + x / 8] &= !(1 << (7 - (x % 8)));
        }
    }
    pub fn hrg_clear(&mut self) { for b in self.hrg.iter_mut() { *b = 0; } }
    pub fn hrg_toggle(&mut self, enabled: bool) { self.hrg_enabled = enabled; }
    pub fn hrg_set_hires(&mut self, hires: bool) { self.hrg_hires = hires; }
    pub fn hrg_write(&mut self, offset: usize, value: u8) {
        if offset < HRG_SIZE { self.hrg[offset] = value; self.hrg_enabled = true; }
    }

    // --- Network ---
    pub fn net_mount(&mut self, drive: u8) { self.net_drive = Some(drive); }
    pub fn needs_net(&self) -> bool { self.waiting_for_net.is_some() }
    pub fn waiting_net_id(&self) -> i32 { self.waiting_for_net.map(|id| id as i32).unwrap_or(-1) }
    pub fn net_is_ws(&self, conn_id: u8) -> bool {
        self.net.conns.get(&conn_id).map(|_| false).unwrap_or(false)
        // TODO: track protocol in NetState
    }
    pub fn net_get_request(&self, conn_id: u8) -> String {
        self.net.conns.get(&conn_id)
            .map(|c| String::from_utf8_lossy(&c.ctl_data).to_string())
            .unwrap_or_default()
    }
    pub fn net_get_request_body(&self, conn_id: u8) -> Vec<u8> {
        self.net.conns.get(&conn_id).map(|c| c.req_body.clone()).unwrap_or_default()
    }
    pub fn net_set_response(&mut self, conn_id: u8, data: &[u8]) {
        if let Some(conn) = self.net.conns.get_mut(&conn_id) {
            conn.resp_data = data.to_vec();
            conn.resp_pos = 0;
            conn.ready = true;
        }
        if self.waiting_for_net == Some(conn_id) { self.waiting_for_net = None; }
    }
    pub fn net_ws_receive(&mut self, conn_id: u8, data: &[u8]) {
        if let Some(conn) = self.net.conns.get_mut(&conn_id) {
            conn.resp_data.extend_from_slice(data);
            conn.ready = true;
        }
        if self.waiting_for_net == Some(conn_id) { self.waiting_for_net = None; }
    }
    pub fn net_ws_take_send(&mut self, conn_id: u8) -> Vec<u8> {
        self.net.conns.get_mut(&conn_id)
            .map(|c| std::mem::take(&mut c.req_body))
            .unwrap_or_default()
    }
}

// --- Internal ---

impl Emulator {
    fn go_idle(&mut self) {
        self.running = false;
        self.vdu.write_str(&mut self.cpu.mem, "\r\nA>");
    }

    fn is_net_drive(&self, fcb_drive: u8) -> bool {
        let Some(net_drv) = self.net_drive else { return false };
        let d = if fcb_drive == 0 { 0 } else { fcb_drive - 1 };
        d == net_drv
    }

    fn handle_bios(&mut self, func: u8) {
        match func {
            0 | 1 => { self.go_idle(); return; }
            2 => { self.cpu.a = if self.key_buffer.is_empty() { 0 } else { 0xFF }; }
            3 => {
                if let Some(ch) = self.key_buffer.pop_front() {
                    self.cpu.a = ch & 0x7F;
                } else { self.waiting_for_key = true; return; }
            }
            4 => { self.vdu.write_char(&mut self.cpu.mem, self.cpu.c); }
            7 => { self.cpu.a = 0x1A; }
            _ => {}
        }
        if !self.waiting_for_key { self.cpu.pc = self.cpu.pop16(); }
    }

    fn handle_bdos(&mut self, func: u8) {
        match func {
            0 => { self.go_idle(); }
            1 => {
                if let Some(ch) = self.key_buffer.pop_front() {
                    self.vdu.write_char(&mut self.cpu.mem, ch);
                    bdos_core::set_return(&mut self.cpu, ch & 0x7F);
                } else { self.waiting_for_key = true; }
            }
            2 => {
                self.vdu.write_char(&mut self.cpu.mem, self.cpu.e);
                bdos_core::set_return(&mut self.cpu, 0);
            }
            6 => {
                let e = self.cpu.e;
                if e == 0xFF {
                    self.cpu.a = self.key_buffer.pop_front().unwrap_or(0);
                } else if e == 0xFE {
                    self.cpu.a = if self.key_buffer.is_empty() { 0 } else { 0xFF };
                } else { self.vdu.write_char(&mut self.cpu.mem, e); }
                self.cpu.l = self.cpu.a; self.cpu.h = 0;
            }
            9 => {
                let mut addr = self.cpu.de();
                loop {
                    let ch = self.cpu.read8(addr);
                    if ch == b'$' { break; }
                    self.vdu.write_char(&mut self.cpu.mem, ch);
                    addr = addr.wrapping_add(1);
                }
                bdos_core::set_return(&mut self.cpu, 0);
            }
            10 => {
                let buf_addr = self.cpu.de();
                let max_len = self.cpu.read8(buf_addr);
                let mut line = Vec::new();
                loop {
                    if let Some(ch) = self.key_buffer.pop_front() {
                        if ch == 0x0D {
                            self.vdu.write_str(&mut self.cpu.mem, "\r\n");
                            break;
                        } else if ch == 0x08 || ch == 0x7F {
                            if !line.is_empty() {
                                line.pop();
                                self.vdu.write_char(&mut self.cpu.mem, 0x08);
                                self.vdu.write_char(&mut self.cpu.mem, b' ');
                                self.vdu.write_char(&mut self.cpu.mem, 0x08);
                            }
                        } else if ch >= 0x20 && line.len() < max_len as usize {
                            line.push(ch);
                            self.vdu.write_char(&mut self.cpu.mem, ch);
                        }
                    } else {
                        self.waiting_for_key = true;
                        for &ch in line.iter().rev() { self.key_buffer.push_front(ch); }
                        return;
                    }
                }
                self.cpu.write8(buf_addr + 1, line.len() as u8);
                for (i, &ch) in line.iter().enumerate() {
                    self.cpu.write8(buf_addr + 2 + i as u16, ch);
                }
                bdos_core::set_return(&mut self.cpu, 0);
            }
            11 => {
                self.cpu.a = if self.key_buffer.is_empty() { 0 } else { 0xFF };
                self.cpu.l = self.cpu.a; self.cpu.h = 0;
            }
            12 => { self.cpu.a = 0x22; self.cpu.l = 0x22; self.cpu.h = 0; }
            13 => { bdos_core::set_return(&mut self.cpu, 0); } // reset disk
            14 => { bdos_core::set_return(&mut self.cpu, 0); } // select disk
            15 => self.bdos_open(),
            16 => self.bdos_close(),
            17 => self.bdos_search_first(),
            18 => self.bdos_search_next(),
            19 => self.bdos_delete(),
            20 => self.bdos_read_seq(),
            21 => self.bdos_write_seq(),
            22 => self.bdos_make(),
            25 => { self.cpu.a = 0; self.cpu.l = 0; self.cpu.h = 0; } // get disk
            26 => { /* set DMA - we always use 0x0080 */ }
            _ => { bdos_core::set_return(&mut self.cpu, 0); }
        }
    }

    fn bdos_open(&mut self) {
        let fcb = self.cpu.de();
        let drv = self.cpu.read8(fcb);
        if self.is_net_drive(drv) {
            if let Some((ftype, _ext)) = bdos_core::parse_net_fcb(&self.cpu, fcb) {
                match ftype {
                    "clone" => { let id = self.net.clone_conn(); self.net_fcbs.insert(fcb, (id, "clone")); self.cpu.a = 0; }
                    "ctl" => { let id = bdos_core::parse_conn_id(&self.cpu, fcb).unwrap_or(0); self.net_fcbs.insert(fcb, (id, "ctl")); self.cpu.a = 0; }
                    "data" => { let id = bdos_core::parse_conn_id(&self.cpu, fcb).unwrap_or(0); self.net_fcbs.insert(fcb, (id, "data")); self.cpu.a = 0; }
                    "mem" | "devcpu" => { self.net_fcbs.insert(fcb, (0, ftype)); self.cpu.a = 0; }
                    _ => { self.cpu.a = 0; }
                }
            } else { self.cpu.a = 0xFF; }
        } else {
            // Drive A: file store
            let name = self.get_fcb_filename(fcb);
            if self.files.contains_key(&name) {
                self.open_handles.insert(fcb, (name, 0));
                // Clear extent/record fields
                for i in 12..16 { self.cpu.write8(fcb + i, 0); }
                self.cpu.a = 0;
            } else { self.cpu.a = 0xFF; }
        }
        self.cpu.l = self.cpu.a; self.cpu.h = 0;
    }

    fn bdos_make(&mut self) {
        let fcb = self.cpu.de();
        let drv = self.cpu.read8(fcb);
        if self.is_net_drive(drv) {
            self.bdos_open(); return;
        }
        let name = self.get_fcb_filename(fcb);
        self.files.insert(name.clone(), Vec::new());
        self.open_handles.insert(fcb, (name, 0));
        for i in 12..16 { self.cpu.write8(fcb + i, 0); }
        self.cpu.a = 0; self.cpu.l = 0; self.cpu.h = 0;
    }

    fn bdos_delete(&mut self) {
        let fcb = self.cpu.de();
        let name = self.get_fcb_filename(fcb);
        if self.files.remove(&name).is_some() {
            self.cpu.a = 0;
        } else { self.cpu.a = 0xFF; }
        self.cpu.l = self.cpu.a; self.cpu.h = 0;
    }

    fn bdos_search_first(&mut self) {
        let fcb = self.cpu.de();
        let name = self.get_fcb_filename(fcb);
        // Collect matching files
        let mut results: Vec<String> = if name.contains('?') || name == "????????.???" {
            self.files.keys().cloned().collect()
        } else {
            self.files.keys().filter(|k| **k == name).cloned().collect()
        };
        results.sort();
        self.net.claude_prompt.clear(); // reuse as search state storage
        // Store search results as newline-separated in a temp
        let joined = results.join("\n");
        self.net.claude_prompt = joined.into_bytes();
        self.net.claude_resp_pos = 0; // reuse as search index
        self.bdos_search_next();
    }

    fn bdos_search_next(&mut self) {
        let results: Vec<&str> = std::str::from_utf8(&self.net.claude_prompt)
            .unwrap_or("").split('\n').filter(|s| !s.is_empty()).collect();
        let idx = self.net.claude_resp_pos;
        if idx < results.len() {
            let filename = results[idx].to_string();
            self.net.claude_resp_pos += 1;
            // Write dir entry at DMA (0x0080)
            rm380z_core::fcb::write_dir_entry(&mut self.cpu, 0x0080, &filename);
            self.cpu.a = 0;
        } else { self.cpu.a = 0xFF; }
        self.cpu.l = self.cpu.a; self.cpu.h = 0;
    }

    fn bdos_close(&mut self) {
        let fcb = self.cpu.de();
        if let Some((id, ftype)) = self.net_fcbs.remove(&fcb) {
            if ftype == "ctl" { self.net.close_conn(id); }
        }
        self.open_handles.remove(&fcb);
        self.cpu.a = 0; // always succeed (CP/M convention)
        self.cpu.l = 0; self.cpu.h = 0;
    }

    fn bdos_read_seq(&mut self) {
        let fcb = self.cpu.de();
        if let Some(&(id, ftype)) = self.net_fcbs.get(&fcb) {
            match ftype {
                "clone" => {
                    let id_str = format!("{}\r\n", id);
                    let mut buf = [0x1Au8; 128];
                    buf[..id_str.len()].copy_from_slice(id_str.as_bytes());
                    let dma = 0x0080u16;
                    for i in 0..128 { self.cpu.write8(dma + i as u16, buf[i]); }
                    self.cpu.a = 0;
                }
                "data" => {
                    if let Some(conn) = self.net.conns.get(&id) {
                        if !conn.ready && !conn.ctl_data.is_empty() {
                            self.waiting_for_net = Some(id);
                            return;
                        }
                    }
                    if let Some(buf) = self.net.read_data(id) {
                        let dma = 0x0080u16;
                        for i in 0..128 { self.cpu.write8(dma + i as u16, buf[i]); }
                        self.cpu.a = 0;
                    } else { self.cpu.a = 1; }
                }
                "mem" => {
                    let ext = self.get_fcb_ext(fcb);
                    if bdos_core::read_mem_bank(&mut self.cpu, &ext, fcb, 0x0080) {
                        self.cpu.a = 0;
                    } else { self.cpu.a = 1; }
                }
                "devcpu" => {
                    let text = bdos_core::cpu_dump(&self.cpu);
                    let mut buf = [0x1Au8; 128];
                    let n = text.len().min(127);
                    buf[..n].copy_from_slice(&text.as_bytes()[..n]);
                    let dma = 0x0080u16;
                    for i in 0..128 { self.cpu.write8(dma + i as u16, buf[i]); }
                    self.cpu.a = 0;
                }
                "models" => {
                    let text = self.net.get_model_info();
                    let mut buf = [0x1Au8; 128];
                    let n = text.len().min(127);
                    buf[..n].copy_from_slice(&text.as_bytes()[..n]);
                    let dma = 0x0080u16;
                    for i in 0..128 { self.cpu.write8(dma + i as u16, buf[i]); }
                    self.cpu.a = 0;
                }
                "claude" | "cli" | "run" => {
                    if let Some(buf) = self.net.read_claude() {
                        let dma = 0x0080u16;
                        for i in 0..128 { self.cpu.write8(dma + i as u16, buf[i]); }
                        self.cpu.a = 0;
                    } else if !self.net.get_prompt().is_empty() && !self.waiting_for_claude {
                        // First read after prompt written — signal JS to ask Claude
                        self.claude_inject_keys = ftype == "run";
                        self.waiting_for_claude = true;
                        return; // don't pop return addr, retry after response
                    } else if self.waiting_for_claude {
                        return; // still waiting
                    } else {
                        self.cpu.a = 1; // EOF, no prompt
                    }
                }
                _ => { self.cpu.a = 1; }
            }
        } else if let Some((name, pos)) = self.open_handles.get(&fcb).cloned() {
            // Drive A: file read
            if let Some(data) = self.files.get(&name) {
                if pos < data.len() {
                    let mut buf = [0x1Au8; 128];
                    let n = (data.len() - pos).min(128);
                    buf[..n].copy_from_slice(&data[pos..pos + n]);
                    let dma = 0x0080u16;
                    for i in 0..128 { self.cpu.write8(dma + i as u16, buf[i]); }
                    self.open_handles.get_mut(&fcb).unwrap().1 = pos + 128;
                    let cr = self.cpu.read8(fcb + 32);
                    self.cpu.write8(fcb + 32, cr.wrapping_add(1));
                    self.cpu.a = 0;
                } else { self.cpu.a = 1; } // EOF
            } else { self.cpu.a = 1; }
        } else { self.cpu.a = 1; }
        self.cpu.l = self.cpu.a; self.cpu.h = 0;
    }

    fn bdos_write_seq(&mut self) {
        let fcb = self.cpu.de();
        if let Some(&(id, ftype)) = self.net_fcbs.get(&fcb) {
            let dma = 0x0080u16;
            let data: Vec<u8> = (0..128).map(|i| self.cpu.read8(dma + i)).collect();
            match ftype {
                "ctl" => { self.net.write_ctl(id, &data); }
                "data" => { self.net.write_data(id, &data); }
                "claude" | "cli" | "run" => { self.net.write_claude(&data); }
                "apikey" => { self.net.set_api_key(&data); }
                "setmodel" => { self.net.set_model(&data); }
                "mem" => {
                    let ext = self.get_fcb_ext(fcb);
                    bdos_core::write_mem_bank(&mut self.cpu, &ext, fcb, 0x0080);
                }
                _ => {}
            }
            self.cpu.a = 0;
        } else if let Some((name, pos)) = self.open_handles.get(&fcb).cloned() {
            // Drive A: file write
            let dma = 0x0080u16;
            let data: Vec<u8> = (0..128).map(|i| self.cpu.read8(dma + i)).collect();
            let file = self.files.entry(name).or_insert_with(Vec::new);
            // Extend file if needed
            while file.len() < pos + 128 {
                file.push(0x1A);
            }
            file[pos..pos + 128].copy_from_slice(&data);
            self.open_handles.get_mut(&fcb).unwrap().1 = pos + 128;
            let cr = self.cpu.read8(fcb + 32);
            self.cpu.write8(fcb + 32, cr.wrapping_add(1));
            self.cpu.a = 0;
        } else { self.cpu.a = 2; }
        self.cpu.l = self.cpu.a; self.cpu.h = 0;
    }

    fn get_fcb_ext(&self, fcb: u16) -> String {
        let ext: String = (0..3).map(|i| (self.cpu.read8(fcb + 9 + i) & 0x7F) as char).collect();
        ext.trim().to_string()
    }

    fn get_fcb_filename(&self, fcb: u16) -> String {
        let name: String = (0..8).map(|i| (self.cpu.read8(fcb + 1 + i) & 0x7F) as char).collect();
        let ext = self.get_fcb_ext(fcb);
        let name = name.trim();
        if ext.is_empty() { name.to_string() } else { format!("{}.{}", name, ext) }
    }
}
