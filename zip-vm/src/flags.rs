/// Z80 flag bit positions in the F register.
pub const C: u8 = 0x01; // Carry
pub const N: u8 = 0x02; // Subtract
pub const PV: u8 = 0x04; // Parity/Overflow
pub const H: u8 = 0x10; // Half-carry
pub const Z: u8 = 0x40; // Zero
pub const S: u8 = 0x80; // Sign

/// Parity lookup: true if even number of set bits.
pub fn parity(v: u8) -> bool {
    v.count_ones() & 1 == 0
}

/// Sign, Zero, and Parity flags from an 8-bit result.
pub fn szp(v: u8) -> u8 {
    let mut f = 0u8;
    if v == 0 { f |= Z; }
    f |= v & S; // sign = bit 7
    if parity(v) { f |= PV; }
    f
}

/// 8-bit addition with optional carry-in. Returns (result, flags).
pub fn add8(a: u8, b: u8, carry: bool) -> (u8, u8) {
    let ci = carry as u16;
    let full = a as u16 + b as u16 + ci;
    let result = full as u8;
    let mut f = 0u8;
    if result == 0 { f |= Z; }
    f |= result & S;
    if full > 0xFF { f |= C; }
    if (a & 0xF) + (b & 0xF) + ci as u8 > 0xF { f |= H; }
    // Overflow: both operands same sign, result differs
    if (a ^ result) & (b ^ result) & 0x80 != 0 { f |= PV; }
    (result, f)
}

/// 8-bit subtraction with optional borrow. Returns (result, flags).
pub fn sub8(a: u8, b: u8, carry: bool) -> (u8, u8) {
    let ci = carry as u16;
    let full = (a as u16).wrapping_sub(b as u16).wrapping_sub(ci);
    let result = full as u8;
    let mut f = N;
    if result == 0 { f |= Z; }
    f |= result & S;
    if full > 0xFF { f |= C; } // borrow
    let half = (a & 0xF) as i16 - (b & 0xF) as i16 - ci as i16;
    if half < 0 { f |= H; }
    // Overflow: operands differ in sign, result differs from a
    if (a ^ b) & (a ^ result) & 0x80 != 0 { f |= PV; }
    (result, f)
}

/// 16-bit addition. Returns (result, partial_flags).
/// Only sets C, H, clears N. Caller merges with existing S/Z/PV.
pub fn add16(a: u16, b: u16) -> (u16, u8) {
    let full = a as u32 + b as u32;
    let result = full as u16;
    let mut f = 0u8;
    if full > 0xFFFF { f |= C; }
    if (a & 0xFFF) + (b & 0xFFF) > 0xFFF { f |= H; }
    (result, f)
}

/// 16-bit add with carry. Returns (result, full_flags).
pub fn adc16(a: u16, b: u16, carry: bool) -> (u16, u8) {
    let ci = carry as u32;
    let full = a as u32 + b as u32 + ci;
    let result = full as u16;
    let mut f = 0u8;
    if result == 0 { f |= Z; }
    if result & 0x8000 != 0 { f |= S; }
    if full > 0xFFFF { f |= C; }
    if (a & 0xFFF) + (b & 0xFFF) + ci as u16 > 0xFFF { f |= H; }
    if (a ^ result) & (b ^ result) & 0x8000 != 0 { f |= PV; }
    (result, f)
}

/// 16-bit subtract with borrow. Returns (result, full_flags).
pub fn sbc16(a: u16, b: u16, carry: bool) -> (u16, u8) {
    let ci = carry as u32;
    let full = (a as u32).wrapping_sub(b as u32).wrapping_sub(ci);
    let result = full as u16;
    let mut f = N;
    if result == 0 { f |= Z; }
    if result & 0x8000 != 0 { f |= S; }
    if full > 0xFFFF { f |= C; }
    let half = (a & 0xFFF) as i32 - (b & 0xFFF) as i32 - ci as i32;
    if half < 0 { f |= H; }
    if (a ^ b) & (a ^ result) & 0x8000 != 0 { f |= PV; }
    (result, f)
}

/// INC r flags. C is preserved (not in returned flags).
pub fn inc8(v: u8) -> (u8, u8) {
    let result = v.wrapping_add(1);
    let mut f = 0u8;
    if result == 0 { f |= Z; }
    f |= result & S;
    if v & 0xF == 0xF { f |= H; }
    if v == 0x7F { f |= PV; } // overflow: 7F→80
    (result, f)
}

/// DEC r flags. C is preserved (not in returned flags).
pub fn dec8(v: u8) -> (u8, u8) {
    let result = v.wrapping_sub(1);
    let mut f = N;
    if result == 0 { f |= Z; }
    f |= result & S;
    if v & 0xF == 0 { f |= H; } // borrow from bit 4
    if v == 0x80 { f |= PV; } // overflow: 80→7F
    (result, f)
}
