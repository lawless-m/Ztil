use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use crate::diskimg::DiskImage;

const MAX_DRIVES: usize = 16;

/// A drive can be either a host directory or a CP/M disk image.
pub enum Drive {
    HostDir(PathBuf),
    Image(DiskImage),
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
    pub fn mount(&mut self, drive: u8, path: PathBuf) {
        if (drive as usize) >= MAX_DRIVES { return; }
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
            Drive::Image(_) => None,
        }
    }

    /// Check if a drive is a disk image.
    pub fn is_image(&self, drive: u8) -> bool {
        let d = if drive == 0 { self.current_disk } else { drive - 1 };
        matches!(self.drives.get(d as usize), Some(Some(Drive::Image(_))))
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
