use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;

pub struct DiskSystem {
    pub drive_a: PathBuf,
    pub dma_addr: u16,
    pub current_disk: u8,
    pub open_files: HashMap<u16, OpenFile>,  // keyed by FCB address
    pub search_results: Vec<String>,
    pub search_index: usize,
}

pub struct OpenFile {
    pub host_path: PathBuf,
    pub handle: File,
    pub position: u32,
}

impl DiskSystem {
    pub fn new(drive_a: PathBuf) -> Self {
        DiskSystem {
            drive_a,
            dma_addr: 0x0080,
            current_disk: 0,
            open_files: HashMap::new(),
            search_results: Vec::new(),
            search_index: 0,
        }
    }

    /// Convert a CP/M filename (8+3, space-padded, uppercase) to a host path.
    pub fn cpm_to_host_path(&self, name: &[u8; 8], ext: &[u8; 3]) -> PathBuf {
        let name_str: String = name.iter()
            .map(|&b| (b & 0x7F) as char)
            .collect::<String>()
            .trim_end()
            .to_string();
        let ext_str: String = ext.iter()
            .map(|&b| (b & 0x7F) as char)
            .collect::<String>()
            .trim_end()
            .to_string();

        let filename = if ext_str.is_empty() {
            name_str
        } else {
            format!("{}.{}", name_str, ext_str)
        };
        self.drive_a.join(filename)
    }

    /// List files matching a CP/M pattern (supports ? wildcards).
    pub fn search_files(&self, name: &[u8; 8], ext: &[u8; 3]) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(&self.drive_a) else { return Vec::new() };
        let mut results = Vec::new();

        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_uppercase();
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            if matches_cpm_pattern(&fname, name, ext) {
                results.push(fname);
            }
        }
        results.sort();
        results
    }

    /// Open a file by FCB address. Returns true if found.
    pub fn open(&mut self, fcb_addr: u16, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let path = self.cpm_to_host_path(name, ext);
        // Try case-insensitive match
        let actual_path = find_file_ci(&self.drive_a, &path);
        let Some(actual_path) = actual_path else { return false };

        let Ok(handle) = File::options().read(true).write(true).open(&actual_path) else {
            // Try read-only
            let Ok(handle) = File::open(&actual_path) else { return false };
            self.open_files.insert(fcb_addr, OpenFile { host_path: actual_path, handle, position: 0 });
            return true;
        };
        self.open_files.insert(fcb_addr, OpenFile { host_path: actual_path, handle, position: 0 });
        true
    }

    /// Create a new file.
    pub fn make(&mut self, fcb_addr: u16, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let path = self.cpm_to_host_path(name, ext);
        let Ok(handle) = File::create(&path) else { return false };
        self.open_files.insert(fcb_addr, OpenFile { host_path: path, handle, position: 0 });
        true
    }

    /// Read 128 bytes sequentially. Returns true if OK, false if EOF.
    pub fn read_seq(&mut self, fcb_addr: u16, mem: &mut [u8; 0x10000]) -> bool {
        let Some(file) = self.open_files.get_mut(&fcb_addr) else { return false };
        let dma = self.dma_addr as usize;

        // Seek to position
        let _ = file.handle.seek(SeekFrom::Start(file.position as u64));

        let mut buf = [0x1Au8; 128]; // pad with CP/M EOF
        let n = file.handle.read(&mut buf).unwrap_or(0);
        if n == 0 { return false; } // EOF

        mem[dma..dma + 128].copy_from_slice(&buf);
        file.position += 128;
        true
    }

    /// Write 128 bytes sequentially. Returns true if OK.
    pub fn write_seq(&mut self, fcb_addr: u16, mem: &[u8; 0x10000]) -> bool {
        let Some(file) = self.open_files.get_mut(&fcb_addr) else { return false };
        let dma = self.dma_addr as usize;

        let _ = file.handle.seek(SeekFrom::Start(file.position as u64));
        let ok = file.handle.write_all(&mem[dma..dma + 128]).is_ok();
        if ok { file.position += 128; }
        ok
    }

    /// Close a file by FCB address.
    pub fn close(&mut self, fcb_addr: u16) -> bool {
        self.open_files.remove(&fcb_addr).is_some()
    }

    /// Delete file(s) matching name.
    pub fn delete(&mut self, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let path = self.cpm_to_host_path(name, ext);
        let actual = find_file_ci(&self.drive_a, &path);
        if let Some(p) = actual {
            std::fs::remove_file(p).is_ok()
        } else {
            false
        }
    }
}

/// Check if a host filename matches a CP/M pattern (? = any single char).
fn matches_cpm_pattern(host_filename: &str, name: &[u8; 8], ext: &[u8; 3]) -> bool {
    // Split host filename into name.ext
    let (hname, hext) = if let Some(dot) = host_filename.rfind('.') {
        (&host_filename[..dot], &host_filename[dot + 1..])
    } else {
        (host_filename, "")
    };

    let pat_name: String = name.iter().map(|&b| (b & 0x7F) as char).collect::<String>();
    let pat_name = pat_name.trim_end();
    let pat_ext: String = ext.iter().map(|&b| (b & 0x7F) as char).collect::<String>();
    let pat_ext = pat_ext.trim_end();

    match_with_wildcards(hname, pat_name) && match_with_wildcards(hext, pat_ext)
}

fn match_with_wildcards(s: &str, pattern: &str) -> bool {
    if pattern.is_empty() { return s.is_empty() || true; } // empty pattern matches anything
    let s = s.to_uppercase();
    let pattern = pattern.to_uppercase();
    if s.len() > pattern.len() { return false; }
    for (sc, pc) in s.chars().zip(pattern.chars()) {
        if pc != '?' && sc != pc { return false; }
    }
    // Remaining pattern chars must be '?' or spaces
    for pc in pattern.chars().skip(s.len()) {
        if pc != '?' && pc != ' ' { return false; }
    }
    true
}

/// Find a file case-insensitively.
fn find_file_ci(dir: &PathBuf, target: &PathBuf) -> Option<PathBuf> {
    let target_name = target.file_name()?.to_string_lossy().to_uppercase();
    // First try exact path
    if target.exists() { return Some(target.clone()); }
    // Case-insensitive search
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        if entry.file_name().to_string_lossy().to_uppercase() == target_name {
            return Some(entry.path());
        }
    }
    None
}
