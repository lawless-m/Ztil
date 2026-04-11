use crate::cpm::Cpm;
use crate::fcb;

/// Dispatch a BDOS call based on the function number in register C.
pub fn dispatch(cpm: &mut Cpm) {
    let func = cpm.cpu.c;
    match func {
        0 => sys_reset(cpm),
        1 => con_read(cpm),
        2 => con_write(cpm),
        6 => direct_io(cpm),
        9 => print_string(cpm),
        10 => read_line(cpm),
        11 => con_status(cpm),
        12 => version(cpm),
        13 => reset_disk(cpm),
        14 => select_disk(cpm),
        15 => open_file(cpm),
        16 => close_file(cpm),
        17 => search_first(cpm),
        18 => search_next(cpm),
        19 => delete_file(cpm),
        20 => read_seq(cpm),
        21 => write_seq(cpm),
        22 => make_file(cpm),
        25 => get_disk(cpm),
        26 => set_dma(cpm),
        32 => get_set_user(cpm),
        _ => {
            eprintln!("[BDOS] unhandled function {}", func);
            cpm.cpu.a = 0;
            cpm.cpu.l = 0;
        }
    }
}

// --- Console I/O ---

fn sys_reset(cpm: &mut Cpm) {
    cpm.warm_boot();
}

fn con_read(cpm: &mut Cpm) {
    let ch = cpm.console.read_key();
    cpm.console.write_char(ch);
    cpm.cpu.a = ch & 0x7F;
}

fn con_write(cpm: &mut Cpm) {
    cpm.console.write_char(cpm.cpu.e);
}

fn direct_io(cpm: &mut Cpm) {
    let e = cpm.cpu.e;
    if e == 0xFF {
        // Input: return char or 0
        cpm.cpu.a = cpm.console.try_read_key().unwrap_or(0);
    } else if e == 0xFE {
        // Status (CP/M 3 but some programs use it)
        cpm.cpu.a = if cpm.console.key_ready() { 0xFF } else { 0x00 };
    } else {
        cpm.console.write_char(e);
    }
}

fn print_string(cpm: &mut Cpm) {
    let mut addr = cpm.cpu.de();
    loop {
        let ch = cpm.cpu.read8(addr);
        if ch == b'$' { break; }
        cpm.console.write_char(ch);
        addr = addr.wrapping_add(1);
    }
}

fn read_line(cpm: &mut Cpm) {
    let buf_addr = cpm.cpu.de();
    let max_len = cpm.cpu.read8(buf_addr);
    let line = cpm.console.read_line(max_len);
    cpm.cpu.write8(buf_addr + 1, line.len() as u8);
    for (i, &ch) in line.iter().enumerate() {
        cpm.cpu.write8(buf_addr + 2 + i as u16, ch);
    }
}

fn con_status(cpm: &mut Cpm) {
    cpm.cpu.a = if cpm.console.key_ready() { 0xFF } else { 0x00 };
}

// --- System ---

fn version(cpm: &mut Cpm) {
    // CP/M 2.2
    cpm.cpu.h = 0x00;
    cpm.cpu.l = 0x22;
    cpm.cpu.a = 0x22;
    cpm.cpu.b = 0x00;
}

fn reset_disk(cpm: &mut Cpm) {
    cpm.disk.current_disk = 0;
    cpm.disk.dma_addr = 0x0080;
    cpm.cpu.a = 0;
}

fn select_disk(cpm: &mut Cpm) {
    let disk = cpm.cpu.e;
    if disk == 0 {
        cpm.disk.current_disk = 0;
        cpm.cpu.a = 0;
    } else {
        cpm.cpu.a = 0xFF; // only drive A supported
    }
}

fn get_disk(cpm: &mut Cpm) {
    cpm.cpu.a = cpm.disk.current_disk;
}

fn set_dma(cpm: &mut Cpm) {
    cpm.disk.dma_addr = cpm.cpu.de();
}

fn get_set_user(cpm: &mut Cpm) {
    let e = cpm.cpu.e;
    if e == 0xFF {
        cpm.cpu.a = 0; // always user 0
    }
    // else: set user (ignored, always user 0)
}

// --- File I/O ---

fn open_file(cpm: &mut Cpm) {
    let fcb_addr = cpm.cpu.de();
    let name = fcb::name(&cpm.cpu, fcb_addr);
    let ext = fcb::ext(&cpm.cpu, fcb_addr);
    if cpm.disk.open(fcb_addr, &name, &ext) {
        fcb::clear(&mut cpm.cpu, fcb_addr);
        cpm.cpu.a = 0;
    } else {
        cpm.cpu.a = 0xFF;
    }
}

fn close_file(cpm: &mut Cpm) {
    let fcb_addr = cpm.cpu.de();
    cpm.disk.close(fcb_addr);
    cpm.cpu.a = 0;
}

fn search_first(cpm: &mut Cpm) {
    let fcb_addr = cpm.cpu.de();
    let name = fcb::name(&cpm.cpu, fcb_addr);
    let ext = fcb::ext(&cpm.cpu, fcb_addr);
    cpm.disk.search_results = cpm.disk.search_files(&name, &ext);
    cpm.disk.search_index = 0;
    search_next(cpm);
}

fn search_next(cpm: &mut Cpm) {
    if cpm.disk.search_index < cpm.disk.search_results.len() {
        let filename = cpm.disk.search_results[cpm.disk.search_index].clone();
        cpm.disk.search_index += 1;
        // Write directory entry at DMA buffer
        let dma = cpm.disk.dma_addr;
        fcb::write_dir_entry(&mut cpm.cpu, dma, &filename);
        cpm.cpu.a = 0; // dir code 0 = entry at DMA+0
    } else {
        cpm.cpu.a = 0xFF; // no more
    }
}

fn delete_file(cpm: &mut Cpm) {
    let fcb_addr = cpm.cpu.de();
    let name = fcb::name(&cpm.cpu, fcb_addr);
    let ext = fcb::ext(&cpm.cpu, fcb_addr);
    if cpm.disk.delete(&name, &ext) {
        cpm.cpu.a = 0;
    } else {
        cpm.cpu.a = 0xFF;
    }
}

fn read_seq(cpm: &mut Cpm) {
    let fcb_addr = cpm.cpu.de();
    let mem: &mut [u8; 0x10000] = &mut cpm.cpu.mem;
    if cpm.disk.read_seq(fcb_addr, mem) {
        // Advance FCB record counter
        let cr = cpm.cpu.read8(fcb_addr + 32);
        cpm.cpu.write8(fcb_addr + 32, cr.wrapping_add(1));
        cpm.cpu.a = 0;
    } else {
        cpm.cpu.a = 1; // EOF
    }
}

fn write_seq(cpm: &mut Cpm) {
    let fcb_addr = cpm.cpu.de();
    let mem: &[u8; 0x10000] = &cpm.cpu.mem;
    if cpm.disk.write_seq(fcb_addr, mem) {
        let cr = cpm.cpu.read8(fcb_addr + 32);
        cpm.cpu.write8(fcb_addr + 32, cr.wrapping_add(1));
        cpm.cpu.a = 0;
    } else {
        cpm.cpu.a = 2; // disk full
    }
}

fn make_file(cpm: &mut Cpm) {
    let fcb_addr = cpm.cpu.de();
    let name = fcb::name(&cpm.cpu, fcb_addr);
    let ext = fcb::ext(&cpm.cpu, fcb_addr);
    if cpm.disk.make(fcb_addr, &name, &ext) {
        fcb::clear(&mut cpm.cpu, fcb_addr);
        cpm.cpu.a = 0;
    } else {
        cpm.cpu.a = 0xFF;
    }
}
