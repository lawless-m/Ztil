use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;

/// CP/M supports drives A-P (0-15).
const MAX_DRIVES: usize = 16;

pub struct DiskSystem {
    pub drives: [Option<PathBuf>; MAX_DRIVES],
    pub dma_addr: u16,
    pub current_disk: u8,
    pub open_files: HashMap<u16, OpenFile>,
    pub search_results: Vec<String>,
    pub search_index: usize,
    /// Which drive the current search is on.
    search_drive: u8,
}

pub struct OpenFile {
    pub host_path: PathBuf,
    pub handle: File,
    pub position: u32,
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

    /// Mount a host directory as a CP/M drive (0=A, 1=B, etc.)
    pub fn mount(&mut self, drive: u8, path: PathBuf) {
        if (drive as usize) < MAX_DRIVES {
            self.drives[drive as usize] = Some(path);
        }
    }

    /// Get the directory for a drive. Drive 0 = default = current disk.
    pub fn drive_path(&self, drive: u8) -> Option<&PathBuf> {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        self.drives.get(d as usize).and_then(|p| p.as_ref())
    }

    /// Get the login vector (bitmap of mounted drives).
    pub fn login_vector(&self) -> u16 {
        let mut vec = 0u16;
        for i in 0..MAX_DRIVES {
            if self.drives[i].is_some() { vec |= 1 << i; }
        }
        vec
    }

    /// Convert a CP/M filename to a host path on the given drive.
    pub fn cpm_to_host_path(&self, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> Option<PathBuf> {
        let dir = self.drive_path(drive)?;
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
        let filename = if ext_str.is_empty() { name_str } else { format!("{}.{}", name_str, ext_str) };
        Some(dir.join(filename))
    }

    /// List files matching a CP/M pattern on a drive.
    pub fn search_files(&self, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> Vec<String> {
        let Some(dir) = self.drive_path(drive) else { return Vec::new() };
        let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
        let mut results = Vec::new();
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_uppercase();
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) { continue; }
            if matches_cpm_pattern(&fname, name, ext) {
                results.push(fname);
            }
        }
        results.sort();
        results
    }

    /// Start a search on a specific drive.
    pub fn search_start(&mut self, drive: u8, name: &[u8; 8], ext: &[u8; 3]) {
        self.search_drive = drive;
        self.search_results = self.search_files(drive, name, ext);
        self.search_index = 0;
    }

    pub fn open(&mut self, fcb_addr: u16, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let Some(path) = self.cpm_to_host_path(drive, name, ext) else { return false };
        let Some(dir) = self.drive_path(drive) else { return false };
        let actual_path = find_file_ci(dir, &path);
        let Some(actual_path) = actual_path else { return false };

        let Ok(handle) = File::options().read(true).write(true).open(&actual_path) else {
            let Ok(handle) = File::open(&actual_path) else { return false };
            self.open_files.insert(fcb_addr, OpenFile { host_path: actual_path, handle, position: 0 });
            return true;
        };
        self.open_files.insert(fcb_addr, OpenFile { host_path: actual_path, handle, position: 0 });
        true
    }

    pub fn make(&mut self, fcb_addr: u16, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let Some(path) = self.cpm_to_host_path(drive, name, ext) else { return false };
        let Ok(handle) = File::create(&path) else { return false };
        self.open_files.insert(fcb_addr, OpenFile { host_path: path, handle, position: 0 });
        true
    }

    pub fn read_seq(&mut self, fcb_addr: u16, mem: &mut [u8; 0x10000]) -> bool {
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

    pub fn write_seq(&mut self, fcb_addr: u16, mem: &[u8; 0x10000]) -> bool {
        let Some(file) = self.open_files.get_mut(&fcb_addr) else { return false };
        let dma = self.dma_addr as usize;
        let _ = file.handle.seek(SeekFrom::Start(file.position as u64));
        let ok = file.handle.write_all(&mem[dma..dma + 128]).is_ok();
        if ok { file.position += 128; }
        ok
    }

    pub fn close(&mut self, fcb_addr: u16) -> bool {
        self.open_files.remove(&fcb_addr).is_some()
    }

    pub fn delete(&mut self, drive: u8, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let Some(path) = self.cpm_to_host_path(drive, name, ext) else { return false };
        let Some(dir) = self.drive_path(drive) else { return false };
        let actual = find_file_ci(dir, &path);
        if let Some(p) = actual {
            std::fs::remove_file(p).is_ok()
        } else {
            false
        }
    }
}

fn matches_cpm_pattern(host_filename: &str, name: &[u8; 8], ext: &[u8; 3]) -> bool {
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
