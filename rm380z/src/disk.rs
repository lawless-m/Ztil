use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use crate::diskimg::DiskImage;

const MAX_DRIVES: usize = 16;

/// A drive can be a host directory, a CP/M disk image, or a network drive.
pub enum Drive {
    HostDir(PathBuf),
    Image(DiskImage),
    Network(NetDrive),
}

/// Plan 9-style network drive. Connections created via CLONE.WWW files.
pub struct NetDrive {
    conns: HashMap<u8, NetConn>,
    pub next_id: u8,
    pub claude: Option<ClaudeConn>,
    pub api_key: String,
    pub model: String,
}

struct NetConn {
    ctl_data: Vec<u8>,
    req_body: Vec<u8>,
    resp_data: Vec<u8>,
    resp_pos: usize,
    state: NetState,
}

/// Claude AI conversation state.
pub struct ClaudeConn {
    prompt: Vec<u8>,
    resp_data: Vec<u8>,
    resp_pos: usize,
    responded: bool,
}

#[derive(PartialEq)]
enum NetState { New, CtlWritten, ResponseReady }

#[derive(Clone, Copy, PartialEq)]
pub enum NetFileType { Clone, Ctl, Data }

impl NetDrive {
    pub fn new() -> Self {
        NetDrive {
            conns: HashMap::new(), next_id: 0, claude: None,
            api_key: String::new(),
            model: "claude-sonnet-4-5".to_string(),
        }
    }

    /// Open CLAUDE.AI — creates a new conversation.
    pub fn open_claude(&mut self) {
        self.claude = Some(ClaudeConn {
            prompt: Vec::new(), resp_data: Vec::new(), resp_pos: 0, responded: false,
        });
    }

    /// Write prompt text to CLAUDE.AI.
    pub fn write_claude(&mut self, data: &[u8]) {
        if let Some(c) = &mut self.claude {
            for &b in data {
                if b == 0x1A { break; }
                c.prompt.push(b);
            }
        }
    }

    /// Read response from CLAUDE.AI. Executes the API call on first read.
    pub fn read_claude(&mut self) -> Option<[u8; 128]> {
        let claude = self.claude.as_mut()?;
        if !claude.responded {
            // Execute the API call
            let prompt = String::from_utf8_lossy(&claude.prompt).trim().to_string();
            if prompt.is_empty() { return None; }

            let api_key = &self.api_key;
            if api_key.is_empty() {
                claude.resp_data = b"ERROR: No API key. Write key to N:CLAUDE.KEY\r\n".to_vec();
            } else {
                let body = format!(
                    r#"{{"model":"{}","max_tokens":1024,"messages":[{{"role":"user","content":"{}"}}]}}"#,
                    self.model,
                    prompt.replace('\\', "\\\\").replace('"', "\\\"")
                );
                let result = ureq::post("https://api.anthropic.com/v1/messages")
                    .set("x-api-key", api_key)
                    .set("anthropic-version", "2023-06-01")
                    .set("content-type", "application/json")
                    .send_string(&body);

                match result {
                    Ok(resp) => {
                        let text = resp.into_string().unwrap_or_default();
                        // Extract the text content from the JSON response
                        if let Some(start) = text.find("\"text\":\"") {
                            let start = start + 8;
                            if let Some(end) = text[start..].find("\"") {
                                let content = &text[start..start + end];
                                let content = content.replace("\\n", "\r\n").replace("\\\"", "\"");
                                claude.resp_data = content.into_bytes();
                            } else {
                                claude.resp_data = text.into_bytes();
                            }
                        } else {
                            claude.resp_data = text.into_bytes();
                        }
                    }
                    Err(e) => {
                        claude.resp_data = format!("ERROR: {}\r\n", e).into_bytes();
                    }
                }
            }
            // Add CP/M line ending
            claude.resp_data.extend_from_slice(b"\r\n");
            claude.resp_pos = 0;
            claude.responded = true;
        }

        if claude.resp_pos >= claude.resp_data.len() { return None; }
        let mut buf = [0x1Au8; 128];
        let remaining = claude.resp_data.len() - claude.resp_pos;
        let n = remaining.min(128);
        buf[..n].copy_from_slice(&claude.resp_data[claude.resp_pos..claude.resp_pos + n]);
        claude.resp_pos += 128;
        Some(buf)
    }

    /// Close CLAUDE.AI conversation.
    pub fn close_claude(&mut self) {
        self.claude = None;
    }

    /// Set API key (from CLAUDE.KEY file write).
    pub fn set_api_key(&mut self, data: &[u8]) {
        let key: String = data.iter().take_while(|&&b| b != 0x1A && b != b'\r' && b != b'\n')
            .map(|&b| b as char).collect();
        self.api_key = key.trim().to_string();
    }

    /// Get available model names.
    pub fn get_models(&self) -> String {
        format!("Current: {}\r\nclaude-haiku-4-5\r\nclaude-sonnet-4-5\r\nclaude-sonnet-4-6\r\nclaude-opus-4-6\r\n", self.model)
    }

    /// Set the model (from CLAUDE.MDL file write).
    pub fn set_model(&mut self, data: &[u8]) {
        let model: String = data.iter().take_while(|&&b| b != 0x1A && b != b'\r' && b != b'\n')
            .map(|&b| b as char).collect();
        let model = model.trim().to_string();
        if !model.is_empty() { self.model = model; }
    }

    pub fn clone_conn(&mut self) -> u8 {
        let id = self.next_id;
        self.next_id += 1;
        self.conns.insert(id, NetConn {
            ctl_data: Vec::new(), req_body: Vec::new(),
            resp_data: Vec::new(), resp_pos: 0, state: NetState::New,
        });
        id
    }

    pub fn write_ctl(&mut self, id: u8, data: &[u8]) {
        if let Some(conn) = self.conns.get_mut(&id) {
            for &b in data {
                if b == 0x1A { break; }
                conn.ctl_data.push(b);
            }
            conn.state = NetState::CtlWritten;
        }
    }

    pub fn write_data(&mut self, id: u8, data: &[u8]) {
        if let Some(conn) = self.conns.get_mut(&id) {
            for &b in data {
                if b == 0x1A { break; }
                conn.req_body.push(b);
            }
        }
    }

    /// Execute the HTTP request. Called when the program reads from data.
    pub fn execute_if_needed(&mut self, id: u8) -> bool {
        let Some(conn) = self.conns.get_mut(&id) else { return false };
        if conn.state != NetState::CtlWritten { return conn.state == NetState::ResponseReady; }

        // Parse ctl: "VERB URL\nHeaders..."
        let ctl = String::from_utf8_lossy(&conn.ctl_data).to_string();
        let lines: Vec<&str> = ctl.lines().collect();
        let first = lines.first().unwrap_or(&"");
        let (verb, url) = first.split_once(' ').unwrap_or(("GET", first));

        let mut req = match verb.to_uppercase().as_str() {
            "POST" => ureq::post(url),
            "PUT" => ureq::put(url),
            "DELETE" => ureq::delete(url),
            "PATCH" => ureq::patch(url),
            "HEAD" => ureq::head(url),
            _ => ureq::get(url),
        };

        // Add headers from ctl lines 1+
        for line in lines.iter().skip(1) {
            let line = line.trim();
            if line.is_empty() { break; }
            if let Some((key, val)) = line.split_once(':') {
                req = req.set(key.trim(), val.trim());
            }
        }

        // Execute
        let result = if !conn.req_body.is_empty() && matches!(verb.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") {
            req.send_bytes(&conn.req_body)
        } else {
            req.call()
        };

        match result {
            Ok(resp) => {
                let mut body = Vec::new();
                let _ = resp.into_reader().read_to_end(&mut body);
                conn.resp_data = body;
            }
            Err(e) => {
                conn.resp_data = format!("ERROR: {}\r\n", e).into_bytes();
            }
        }
        conn.resp_pos = 0;
        conn.state = NetState::ResponseReady;
        true
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
}

pub struct DiskSystem {
    pub drives: [Option<Drive>; MAX_DRIVES],
    pub dma_addr: u16,
    pub current_disk: u8,
    pub open_files: HashMap<u16, OpenFile>,
    pub search_results: Vec<String>,
    pub search_index: usize,
    search_drive: u8,
}

pub struct OpenFile {
    pub host_path: PathBuf,
    pub handle: File,
    pub position: u32,
    /// For disk image files: track record position instead of file handle.
    pub img_name: Option<([u8; 8], [u8; 3])>,
    pub img_record: u32,
}

impl DiskSystem {
    pub fn new() -> Self {
        DiskSystem {
            drives: Default::default(),
            dma_addr: 0x0080,
            current_disk: 0,
            open_files: HashMap::new(),
            search_results: Vec::new(),
            search_index: 0,
            search_drive: 0,
        }
    }

    /// Mount a path as a drive. Auto-detects directory vs .dsk file.
    /// Mount a network drive.
    pub fn mount_net(&mut self, drive: u8) {
        if (drive as usize) < MAX_DRIVES {
            self.drives[drive as usize] = Some(Drive::Network(NetDrive::new()));
        }
    }

    /// Get mutable reference to network drive.
    pub fn net_drive(&mut self, drive: u8) -> Option<&mut NetDrive> {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        match self.drives.get_mut(d as usize)?.as_mut()? {
            Drive::Network(n) => Some(n),
            _ => None,
        }
    }

    pub fn mount(&mut self, drive: u8, path: PathBuf) {
        if (drive as usize) >= MAX_DRIVES { return; }
        if path.to_str() == Some("net") {
            self.mount_net(drive);
            return;
        }
        if path.is_dir() {
            self.drives[drive as usize] = Some(Drive::HostDir(path));
        } else if path.extension().map(|e| e == "dsk" || e == "DSK").unwrap_or(false) {
            if let Some(img) = DiskImage::open(&path) {
                self.drives[drive as usize] = Some(Drive::Image(img));
            }
        } else if !path.exists() && path.extension().map(|e| e == "dsk" || e == "DSK").unwrap_or(false) {
            // Create new disk image
            if let Some(img) = DiskImage::create(&path) {
                self.drives[drive as usize] = Some(Drive::Image(img));
            }
        } else {
            // Try as directory anyway
            self.drives[drive as usize] = Some(Drive::HostDir(path));
        }
    }

    /// Unmount a drive.
    pub fn unmount(&mut self, drive: u8) {
        if (drive as usize) < MAX_DRIVES {
            self.drives[drive as usize] = None;
        }
    }

    /// Check if a drive is mounted.
    pub fn is_mounted(&self, drive: u8) -> bool {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        self.drives.get(d as usize).map(|d| d.is_some()).unwrap_or(false)
    }

    /// Get the directory for a host-dir drive. Drive 0 = default = current disk.
    pub fn drive_path(&self, drive: u8) -> Option<&PathBuf> {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        match self.drives.get(d as usize)?.as_ref()? {
            Drive::HostDir(p) => Some(p),
            Drive::Image(_) | Drive::Network(_) => None,
        }
    }

    /// Check if a drive is a disk image.
    pub fn is_image(&self, drive: u8) -> bool {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        matches!(self.drives.get(d as usize), Some(Some(Drive::Image(_))))
    }

    /// Check if a drive is a network drive or disk image (not a host dir).
    pub fn is_image_or_net(&self, drive: u8) -> bool {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        matches!(self.drives.get(d as usize), Some(Some(Drive::Image(_) | Drive::Network(_))))
    }

    pub fn login_vector(&self) -> u16 {
        let mut vec = 0u16;
        for i in 0..MAX_DRIVES {
            if self.drives[i].is_some() { vec |= 1 << i; }
        }
        vec
    }

    pub fn cpm_to_host_path(&self, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> Option<PathBuf> {
        let dir = self.drive_path(drive)?;
        let name_str: String = name.iter().map(|&b| (b & 0x7F) as char).collect::<String>().trim_end().to_string();
        let ext_str: String = ext.iter().map(|&b| (b & 0x7F) as char).collect::<String>().trim_end().to_string();
        let filename = if ext_str.is_empty() { name_str } else { format!("{}.{}", name_str, ext_str) };
        Some(dir.join(filename))
    }

    pub fn search_files(&self, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> Vec<String> {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        match self.drives.get(d as usize) {
            Some(Some(Drive::HostDir(dir))) => search_host_dir(dir, name, ext),
            Some(Some(Drive::Image(_))) => {
                // Need mutable access — caller should use search_start for images
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// Start a search — works for both host dirs and disk images.
    pub fn search_start(&mut self, drive: u8, name: &[u8; 8], ext: &[u8; 3]) {
        self.search_drive = drive;
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        self.search_results = match &mut self.drives[d as usize] {
            Some(Drive::HostDir(dir)) => search_host_dir(dir, name, ext),
            Some(Drive::Image(img)) => img.search_files(name, ext),
            Some(Drive::Network(_)) => Vec::new(),
            None => Vec::new(),
        };
        self.search_index = 0;
    }

    pub fn open(&mut self, fcb_addr: u16, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        match &mut self.drives[d as usize] {
            Some(Drive::HostDir(dir)) => {
                let path = host_file_path(dir, name, ext);
                let actual_path = find_file_ci(dir, &path);
                let Some(actual_path) = actual_path else { return false };
                let Ok(handle) = File::options().read(true).write(true).open(&actual_path)
                    .or_else(|_| File::open(&actual_path)) else { return false };
                self.open_files.insert(fcb_addr, OpenFile {
                    host_path: actual_path, handle, position: 0,
                    img_name: None, img_record: 0,
                });
                true
            }
            Some(Drive::Image(img)) => {
                if img.open_file(name, ext) {
                    self.open_files.insert(fcb_addr, OpenFile {
                        host_path: PathBuf::new(), handle: File::open("/dev/null").unwrap(),
                        position: 0, img_name: Some((*name, *ext)), img_record: 0,
                    });
                    true
                } else { false }
            }
            None => false,
            Some(Drive::Network(_)) => false,
        }
    }

    pub fn make(&mut self, fcb_addr: u16, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        match &mut self.drives[d as usize] {
            Some(Drive::HostDir(dir)) => {
                let path = host_file_path(dir, name, ext);
                let Ok(handle) = File::create(&path) else { return false };
                self.open_files.insert(fcb_addr, OpenFile {
                    host_path: path, handle, position: 0,
                    img_name: None, img_record: 0,
                });
                true
            }
            Some(Drive::Image(img)) => {
                if img.make_file(name, ext) {
                    self.open_files.insert(fcb_addr, OpenFile {
                        host_path: PathBuf::new(), handle: File::open("/dev/null").unwrap(),
                        position: 0, img_name: Some((*name, *ext)), img_record: 0,
                    });
                    true
                } else { false }
            }
            Some(Drive::Network(_)) => false,
            None => false,
        }
    }

    pub fn read_seq(&mut self, fcb_addr: u16, mem: &mut [u8; 0x10000]) -> bool {
        let file = self.open_files.get(&fcb_addr);
        let is_img = file.map(|f| f.img_name.is_some()).unwrap_or(false);

        if is_img {
            let file = self.open_files.get(&fcb_addr).unwrap();
            let (name, ext) = file.img_name.unwrap();
            let record = file.img_record;
            let d = self.current_disk as usize;
            let data = match &mut self.drives[d] {
                Some(Drive::Image(img)) => img.read_record(&name, &ext, record),
                _ => None,
            };
            if let Some(buf) = data {
                let dma = self.dma_addr as usize;
                mem[dma..dma + 128].copy_from_slice(&buf);
                self.open_files.get_mut(&fcb_addr).unwrap().img_record += 1;
                true
            } else { false }
        } else {
            let Some(file) = self.open_files.get_mut(&fcb_addr) else { return false };
            let dma = self.dma_addr as usize;
            let _ = file.handle.seek(SeekFrom::Start(file.position as u64));
            let mut buf = [0x1Au8; 128];
            let n = file.handle.read(&mut buf).unwrap_or(0);
            if n == 0 { return false; }
            mem[dma..dma + 128].copy_from_slice(&buf);
            file.position += 128;
            true
        }
    }

    pub fn write_seq(&mut self, fcb_addr: u16, mem: &[u8; 0x10000]) -> bool {
        let file = self.open_files.get(&fcb_addr);
        let is_img = file.map(|f| f.img_name.is_some()).unwrap_or(false);

        if is_img {
            let file = self.open_files.get(&fcb_addr).unwrap();
            let (name, ext) = file.img_name.unwrap();
            let record = file.img_record;
            let dma = self.dma_addr as usize;
            let mut buf = [0u8; 128];
            buf.copy_from_slice(&mem[dma..dma + 128]);
            let d = self.current_disk as usize;
            let ok = match &mut self.drives[d] {
                Some(Drive::Image(img)) => img.write_record(&name, &ext, record, &buf),
                _ => false,
            };
            if ok { self.open_files.get_mut(&fcb_addr).unwrap().img_record += 1; }
            ok
        } else {
            let Some(file) = self.open_files.get_mut(&fcb_addr) else { return false };
            let dma = self.dma_addr as usize;
            let _ = file.handle.seek(SeekFrom::Start(file.position as u64));
            let ok = file.handle.write_all(&mem[dma..dma + 128]).is_ok();
            if ok { file.position += 128; }
            ok
        }
    }

    pub fn close(&mut self, fcb_addr: u16) -> bool {
        self.open_files.remove(&fcb_addr).is_some()
    }

    pub fn delete(&mut self, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        match &mut self.drives[d as usize] {
            Some(Drive::HostDir(dir)) => {
                let path = host_file_path(dir, name, ext);
                let actual = find_file_ci(dir, &path);
                actual.map(|p| std::fs::remove_file(p).is_ok()).unwrap_or(false)
            }
            Some(Drive::Image(img)) => img.delete_file(name, ext),
            Some(Drive::Network(_)) => false,
            None => false,
        }
    }
}

fn host_file_path(dir: &PathBuf, name: &[u8; 8], ext: &[u8; 3]) -> PathBuf {
    let name_str: String = name.iter().map(|&b| (b & 0x7F) as char).collect::<String>().trim_end().to_string();
    let ext_str: String = ext.iter().map(|&b| (b & 0x7F) as char).collect::<String>().trim_end().to_string();
    let filename = if ext_str.is_empty() { name_str } else { format!("{}.{}", name_str, ext_str) };
    dir.join(filename)
}

fn search_host_dir(dir: &PathBuf, name: &[u8; 8], ext: &[u8; 3]) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut results = Vec::new();
    for entry in entries.flatten() {
        let fname = entry.file_name().to_string_lossy().to_uppercase();
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) { continue; }
        if matches_cpm_pattern(&fname, name, ext) { results.push(fname); }
    }
    results.sort();
    results
}

fn matches_cpm_pattern(host_filename: &str, name: &[u8; 8], ext: &[u8; 3]) -> bool {
    let (hname, hext) = if let Some(dot) = host_filename.rfind('.') {
        (&host_filename[..dot], &host_filename[dot + 1..])
    } else { (host_filename, "") };
    let pat_name: String = name.iter().map(|&b| (b & 0x7F) as char).collect::<String>();
    let pat_ext: String = ext.iter().map(|&b| (b & 0x7F) as char).collect::<String>();
    match_wild(hname, pat_name.trim_end()) && match_wild(hext, pat_ext.trim_end())
}

fn match_wild(s: &str, pattern: &str) -> bool {
    if pattern.is_empty() { return true; }
    let s = s.to_uppercase();
    let pattern = pattern.to_uppercase();
    if s.len() > pattern.len() { return false; }
    for (sc, pc) in s.chars().zip(pattern.chars()) {
        if pc != '?' && sc != pc { return false; }
    }
    for pc in pattern.chars().skip(s.len()) {
        if pc != '?' && pc != ' ' { return false; }
    }
    true
}

fn find_file_ci(dir: &PathBuf, target: &PathBuf) -> Option<PathBuf> {
    let target_name = target.file_name()?.to_string_lossy().to_uppercase();
    if target.exists() { return Some(target.clone()); }
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        if entry.file_name().to_string_lossy().to_uppercase() == target_name {
            return Some(entry.path());
        }
    }
    None
}
