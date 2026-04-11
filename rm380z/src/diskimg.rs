use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use std::collections::HashMap;

/// CP/M disk image — a flat file of sectors.
/// Standard 380Z format: 40 tracks, 10 sectors/track, 512 bytes/sector = 200KB.
/// Tracks 0-1 reserved (system), directory starts at track 2.

const SECTOR_SIZE: usize = 512;
const SECTORS_PER_TRACK: usize = 10;
const TRACKS: usize = 40;
const RESERVED_TRACKS: usize = 2;
const BLOCK_SIZE: usize = 1024; // 1KB allocation blocks
const SECTORS_PER_BLOCK: usize = BLOCK_SIZE / SECTOR_SIZE; // 2
const DIR_ENTRIES: usize = 64;
const DIR_ENTRY_SIZE: usize = 32;
const RECORDS_PER_EXTENT: usize = 128; // 128 × 128 bytes = 16KB per extent

pub struct DiskImage {
    file: File,
    path: PathBuf,
}

/// A CP/M directory entry (32 bytes).
#[derive(Clone)]
struct DirEntry {
    user: u8,           // 0-15, 0xE5 = deleted
    name: [u8; 8],      // space-padded
    ext: [u8; 3],       // high bits are attribute flags
    extent: u8,         // EX
    _s1: u8,
    _s2: u8,
    rc: u8,             // record count in this extent
    alloc: [u8; 16],    // allocation block numbers (8-bit for small disks)
}

impl DiskImage {
    /// Open or create a disk image.
    pub fn open(path: &PathBuf) -> Option<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path).ok()?;
        Some(DiskImage { file, path: path.clone() })
    }

    /// Create a new empty formatted disk image.
    pub fn create(path: &PathBuf) -> Option<Self> {
        let total = TRACKS * SECTORS_PER_TRACK * SECTOR_SIZE;
        let mut data = vec![0xE5u8; total]; // fill with 0xE5 (standard CP/M blank)
        // Zero the reserved tracks
        for i in 0..RESERVED_TRACKS * SECTORS_PER_TRACK * SECTOR_SIZE {
            data[i] = 0;
        }
        let mut file = File::create(path).ok()?;
        file.write_all(&data).ok()?;
        file.flush().ok()?;
        drop(file);
        Self::open(path)
    }

    /// Read a 512-byte sector.
    fn read_sector(&mut self, track: usize, sector: usize) -> [u8; SECTOR_SIZE] {
        let offset = (track * SECTORS_PER_TRACK + sector) * SECTOR_SIZE;
        let mut buf = [0u8; SECTOR_SIZE];
        let _ = self.file.seek(SeekFrom::Start(offset as u64));
        let _ = self.file.read_exact(&mut buf);
        buf
    }

    /// Write a 512-byte sector.
    fn write_sector(&mut self, track: usize, sector: usize, data: &[u8; SECTOR_SIZE]) {
        let offset = (track * SECTORS_PER_TRACK + sector) * SECTOR_SIZE;
        let _ = self.file.seek(SeekFrom::Start(offset as u64));
        let _ = self.file.write_all(data);
    }

    /// Read a 1KB allocation block.
    fn read_block(&mut self, block: usize) -> [u8; BLOCK_SIZE] {
        let data_start_sector = RESERVED_TRACKS * SECTORS_PER_TRACK;
        let abs_sector = data_start_sector + block * SECTORS_PER_BLOCK;
        let track = abs_sector / SECTORS_PER_TRACK;
        let sector = abs_sector % SECTORS_PER_TRACK;
        let mut buf = [0u8; BLOCK_SIZE];
        let s1 = self.read_sector(track, sector);
        let s2 = self.read_sector(track + (sector + 1) / SECTORS_PER_TRACK,
                                   (sector + 1) % SECTORS_PER_TRACK);
        buf[..SECTOR_SIZE].copy_from_slice(&s1);
        buf[SECTOR_SIZE..].copy_from_slice(&s2);
        buf
    }

    /// Write a 1KB allocation block.
    fn write_block(&mut self, block: usize, data: &[u8; BLOCK_SIZE]) {
        let data_start_sector = RESERVED_TRACKS * SECTORS_PER_TRACK;
        let abs_sector = data_start_sector + block * SECTORS_PER_BLOCK;
        let track = abs_sector / SECTORS_PER_TRACK;
        let sector = abs_sector % SECTORS_PER_TRACK;
        let mut s1 = [0u8; SECTOR_SIZE];
        let mut s2 = [0u8; SECTOR_SIZE];
        s1.copy_from_slice(&data[..SECTOR_SIZE]);
        s2.copy_from_slice(&data[SECTOR_SIZE..]);
        self.write_sector(track, sector, &s1);
        self.write_sector(track + (sector + 1) / SECTORS_PER_TRACK,
                          (sector + 1) % SECTORS_PER_TRACK, &s2);
    }

    /// Read all directory entries.
    fn read_directory(&mut self) -> Vec<(usize, DirEntry)> {
        // Directory occupies the first 2 allocation blocks (2KB = 64 × 32 bytes)
        let mut entries = Vec::new();
        for i in 0..DIR_ENTRIES {
            let block = i * DIR_ENTRY_SIZE / BLOCK_SIZE;
            let offset = (i * DIR_ENTRY_SIZE) % BLOCK_SIZE;
            let blk = self.read_block(block);
            let e = &blk[offset..offset + DIR_ENTRY_SIZE];
            let mut alloc = [0u8; 16];
            alloc.copy_from_slice(&e[16..32]);
            entries.push((i, DirEntry {
                user: e[0],
                name: e[1..9].try_into().unwrap(),
                ext: e[9..12].try_into().unwrap(),
                extent: e[12],
                _s1: e[13],
                _s2: e[14],
                rc: e[15],
                alloc,
            }));
        }
        entries
    }

    /// Write a directory entry back to disk.
    fn write_dir_entry(&mut self, index: usize, entry: &DirEntry) {
        let block_num = index * DIR_ENTRY_SIZE / BLOCK_SIZE;
        let offset = (index * DIR_ENTRY_SIZE) % BLOCK_SIZE;
        let mut blk = self.read_block(block_num);
        blk[offset] = entry.user;
        blk[offset + 1..offset + 9].copy_from_slice(&entry.name);
        blk[offset + 9..offset + 12].copy_from_slice(&entry.ext);
        blk[offset + 12] = entry.extent;
        blk[offset + 13] = entry._s1;
        blk[offset + 14] = entry._s2;
        blk[offset + 15] = entry.rc;
        blk[offset + 16..offset + 32].copy_from_slice(&entry.alloc);
        self.write_block(block_num, &blk);
    }

    // --- Public CP/M file operations ---

    /// List files matching a pattern. Returns filenames as "NAME.EXT".
    pub fn search_files(&mut self, name: &[u8; 8], ext: &[u8; 3]) -> Vec<String> {
        let dir = self.read_directory();
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        for (_, e) in &dir {
            if e.user == 0xE5 || e.user > 15 { continue; }
            if !match_pattern(&e.name, name) || !match_pattern_ext(&e.ext, ext) { continue; }
            let fname = format_filename(&e.name, &e.ext);
            if seen.insert(fname.clone()) {
                results.push(fname);
            }
        }
        results.sort();
        results
    }

    /// Open a file — returns true if found. Tracks position internally.
    pub fn open_file(&mut self, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let dir = self.read_directory();
        dir.iter().any(|(_, e)| e.user != 0xE5 && e.user <= 15
            && names_equal(&e.name, name) && exts_equal(&e.ext, ext))
    }

    /// Read 128 bytes at a given record position. Returns None if past EOF.
    pub fn read_record(&mut self, name: &[u8; 8], ext: &[u8; 3], record: u32) -> Option<[u8; 128]> {
        let extent_num = (record / RECORDS_PER_EXTENT as u32) as u8;
        let rec_in_extent = (record % RECORDS_PER_EXTENT as u32) as u8;

        let dir = self.read_directory();
        let entry = dir.iter().find(|(_, e)| e.user != 0xE5 && e.user <= 15
            && names_equal(&e.name, name) && exts_equal(&e.ext, ext)
            && e.extent == extent_num)?;

        if rec_in_extent >= entry.1.rc { return None; } // past EOF in this extent

        // Which allocation block and offset within it?
        let records_per_block = (BLOCK_SIZE / 128) as u8; // 8
        let block_index = rec_in_extent / records_per_block;
        let rec_in_block = rec_in_extent % records_per_block;

        let alloc_block = entry.1.alloc[block_index as usize];
        if alloc_block == 0 { return None; }

        let blk = self.read_block(alloc_block as usize);
        let offset = rec_in_block as usize * 128;
        let mut buf = [0x1Au8; 128];
        buf.copy_from_slice(&blk[offset..offset + 128]);
        Some(buf)
    }

    /// Write 128 bytes at a given record position. Allocates blocks as needed.
    pub fn write_record(&mut self, name: &[u8; 8], ext: &[u8; 3], record: u32, data: &[u8; 128]) -> bool {
        let extent_num = (record / RECORDS_PER_EXTENT as u32) as u8;
        let rec_in_extent = (record % RECORDS_PER_EXTENT as u32) as u8;
        let records_per_block = (BLOCK_SIZE / 128) as u8;
        let block_index = rec_in_extent / records_per_block;
        let rec_in_block = rec_in_extent % records_per_block;

        let dir = self.read_directory();
        let used_blocks = self.used_blocks(&dir);

        // Find or create the directory entry for this extent
        let entry_idx = if let Some((idx, _)) = dir.iter().find(|(_, e)| e.user != 0xE5 && e.user <= 15
            && names_equal(&e.name, name) && exts_equal(&e.ext, ext)
            && e.extent == extent_num) {
            *idx
        } else {
            // Create new extent entry
            let free_slot = dir.iter().find(|(_, e)| e.user == 0xE5).map(|(i, _)| *i);
            let Some(slot) = free_slot else { return false; }; // directory full
            let new_entry = DirEntry {
                user: 0, name: *name, ext: *ext, extent: extent_num,
                _s1: 0, _s2: 0, rc: 0, alloc: [0; 16],
            };
            self.write_dir_entry(slot, &new_entry);
            slot
        };

        // Re-read the entry we're modifying
        let dir = self.read_directory();
        let (_, mut entry) = dir[entry_idx].clone();

        // Allocate block if needed
        if entry.alloc[block_index as usize] == 0 {
            let total_blocks = (TRACKS - RESERVED_TRACKS) * SECTORS_PER_TRACK / SECTORS_PER_BLOCK;
            let free_block = (2..total_blocks).find(|b| !used_blocks.contains(b));
            let Some(block) = free_block else { return false; }; // disk full
            entry.alloc[block_index as usize] = block as u8;
        }

        // Write data to the block
        let alloc_block = entry.alloc[block_index as usize] as usize;
        let mut blk = self.read_block(alloc_block);
        let offset = rec_in_block as usize * 128;
        blk[offset..offset + 128].copy_from_slice(data);
        self.write_block(alloc_block, &blk);

        // Update record count
        if rec_in_extent >= entry.rc {
            entry.rc = rec_in_extent + 1;
        }
        self.write_dir_entry(entry_idx, &entry);
        true
    }

    /// Create a new file (empty directory entry).
    pub fn make_file(&mut self, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        // Delete any existing file first
        self.delete_file(name, ext);
        let dir = self.read_directory();
        let free_slot = dir.iter().find(|(_, e)| e.user == 0xE5).map(|(i, _)| *i);
        let Some(slot) = free_slot else { return false; };
        let entry = DirEntry {
            user: 0, name: *name, ext: *ext, extent: 0,
            _s1: 0, _s2: 0, rc: 0, alloc: [0; 16],
        };
        self.write_dir_entry(slot, &entry);
        true
    }

    /// Delete all extents of a file.
    pub fn delete_file(&mut self, name: &[u8; 8], ext: &[u8; 3]) -> bool {
        let dir = self.read_directory();
        let mut found = false;
        for (idx, e) in &dir {
            if e.user != 0xE5 && e.user <= 15
                && names_equal(&e.name, name) && exts_equal(&e.ext, ext) {
                let mut deleted = e.clone();
                deleted.user = 0xE5;
                self.write_dir_entry(*idx, &deleted);
                found = true;
            }
        }
        found
    }

    /// Get set of used allocation blocks.
    fn used_blocks(&self, dir: &[(usize, DirEntry)]) -> std::collections::HashSet<usize> {
        let mut used = std::collections::HashSet::new();
        used.insert(0); // directory block 0
        used.insert(1); // directory block 1
        for (_, e) in dir {
            if e.user == 0xE5 || e.user > 15 { continue; }
            for &b in &e.alloc {
                if b != 0 { used.insert(b as usize); }
            }
        }
        used
    }
}

fn names_equal(a: &[u8; 8], b: &[u8; 8]) -> bool {
    a.iter().zip(b.iter()).all(|(&x, &y)| (x & 0x7F) == (y & 0x7F))
}

fn exts_equal(a: &[u8; 3], b: &[u8; 3]) -> bool {
    a.iter().zip(b.iter()).all(|(&x, &y)| (x & 0x7F) == (y & 0x7F))
}

fn match_pattern(val: &[u8; 8], pat: &[u8; 8]) -> bool {
    val.iter().zip(pat.iter()).all(|(&v, &p)| p == b'?' || (v & 0x7F) == (p & 0x7F))
}

fn match_pattern_ext(val: &[u8; 3], pat: &[u8; 3]) -> bool {
    val.iter().zip(pat.iter()).all(|(&v, &p)| p == b'?' || (v & 0x7F) == (p & 0x7F))
}

fn format_filename(name: &[u8; 8], ext: &[u8; 3]) -> String {
    let n: String = name.iter().map(|&b| (b & 0x7F) as char).collect::<String>().trim_end().to_string();
    let e: String = ext.iter().map(|&b| (b & 0x7F) as char).collect::<String>().trim_end().to_string();
    if e.is_empty() { n } else { format!("{}.{}", n, e) }
}
