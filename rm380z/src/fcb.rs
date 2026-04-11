use z80::cpu::Cpu;

/// Read the 8-byte filename from an FCB in Z80 memory.
pub fn name(cpu: &Cpu, fcb: u16) -> [u8; 8] {
    let mut n = [0u8; 8];
    for i in 0..8 {
        n[i] = cpu.read8(fcb + 1 + i as u16);
    }
    n
}

/// Read the 3-byte extension from an FCB.
pub fn ext(cpu: &Cpu, fcb: u16) -> [u8; 3] {
    let mut e = [0u8; 3];
    for i in 0..3 {
        e[i] = cpu.read8(fcb + 9 + i as u16);
    }
    e
}

/// Get the drive byte (0=default, 1=A, 2=B...).
pub fn drive(cpu: &Cpu, fcb: u16) -> u8 {
    cpu.read8(fcb)
}

/// Clear the extent/record fields of an FCB (bytes 12-15 only).
/// Only clears the 4 extent bytes, not the full record area, because
/// the default FCBs at 005C and 006C overlap in CP/M's page zero.
pub fn clear(cpu: &mut Cpu, fcb: u16) {
    for i in 12..16 {
        cpu.write8(fcb + i, 0);
    }
}

/// Parse a CP/M filename string ("NAME.TYP" or "D:NAME.TYP") into an FCB.
pub fn parse_into(cpu: &mut Cpu, fcb: u16, filename: &str) {
    let filename = filename.trim().to_uppercase();

    // Drive prefix?
    let (drv, rest) = if filename.len() >= 2 && filename.as_bytes()[1] == b':' {
        let d = filename.as_bytes()[0] - b'A' + 1;
        (d, &filename[2..])
    } else {
        (0u8, filename.as_str())
    };

    cpu.write8(fcb, drv);

    // Split name.ext
    let (name_part, ext_part) = if let Some(dot) = rest.find('.') {
        (&rest[..dot], &rest[dot + 1..])
    } else {
        (rest, "")
    };

    // Write name (8 bytes, space-padded)
    for i in 0..8u16 {
        let ch = name_part.as_bytes().get(i as usize).copied().unwrap_or(b' ');
        cpu.write8(fcb + 1 + i, ch);
    }
    // Write extension (3 bytes, space-padded)
    for i in 0..3u16 {
        let ch = ext_part.as_bytes().get(i as usize).copied().unwrap_or(b' ');
        cpu.write8(fcb + 9 + i, ch);
    }

    clear(cpu, fcb);
}

/// Write a CP/M directory entry (32 bytes) into Z80 memory at addr.
/// Used by search first/next to return results in the DMA buffer.
pub fn write_dir_entry(cpu: &mut Cpu, addr: u16, filename: &str) {
    // User number
    cpu.write8(addr, 0x00);

    let filename_upper = filename.to_uppercase();
    let (name_part, ext_part) = if let Some(dot) = filename_upper.rfind('.') {
        (&filename_upper[..dot], &filename_upper[dot + 1..])
    } else {
        (filename_upper.as_str(), "")
    };

    // Name (8 bytes, space-padded)
    for i in 0..8u16 {
        let ch = name_part.as_bytes().get(i as usize).copied().unwrap_or(b' ');
        cpu.write8(addr + 1 + i, ch);
    }
    // Extension (3 bytes, space-padded)
    for i in 0..3u16 {
        let ch = ext_part.as_bytes().get(i as usize).copied().unwrap_or(b' ');
        cpu.write8(addr + 9 + i, ch);
    }
    // Fill rest of 32-byte entry with zeros
    for i in 12..32u16 {
        cpu.write8(addr + i, 0);
    }
}
