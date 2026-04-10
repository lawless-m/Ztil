use zip_vm::cpu::Cpu;
use zip_vm::flags;

fn cpu_with(code: &[u8]) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.mem[..code.len()].copy_from_slice(code);
    cpu.pc = 0;
    cpu.sp = 0xFFF0;
    cpu
}

#[test]
fn nop_advances_pc() {
    let mut cpu = cpu_with(&[0x00]); // NOP
    cpu.step();
    assert_eq!(cpu.pc, 1);
}

#[test]
fn ld_a_immediate() {
    let mut cpu = cpu_with(&[0x3E, 0x42]); // LD A, 42h
    cpu.step();
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn ld_hl_immediate() {
    let mut cpu = cpu_with(&[0x21, 0x34, 0x12]); // LD HL, 1234h
    cpu.step();
    assert_eq!(cpu.hl(), 0x1234);
}

#[test]
fn push_pop_hl() {
    let mut cpu = cpu_with(&[
        0x21, 0xCD, 0xAB, // LD HL, ABCDh
        0xE5,             // PUSH HL
        0x21, 0x00, 0x00, // LD HL, 0
        0xE1,             // POP HL
    ]);
    cpu.step(); cpu.step(); cpu.step(); cpu.step();
    assert_eq!(cpu.hl(), 0xABCD);
}

#[test]
fn add_a_b() {
    let mut cpu = cpu_with(&[
        0x3E, 0x10, // LD A, 10h
        0x06, 0x20, // LD B, 20h
        0x80,       // ADD A, B
    ]);
    cpu.step(); cpu.step(); cpu.step();
    assert_eq!(cpu.a, 0x30);
    assert!(cpu.f & flags::Z == 0);
    assert!(cpu.f & flags::C == 0);
}

#[test]
fn add_a_overflow() {
    let mut cpu = cpu_with(&[
        0x3E, 0x7F, // LD A, 7Fh
        0xC6, 0x01, // ADD A, 1
    ]);
    cpu.step(); cpu.step();
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.f & flags::S != 0, "sign flag should be set");
    assert!(cpu.f & flags::PV != 0, "overflow flag should be set");
}

#[test]
fn sub_a_carry() {
    let mut cpu = cpu_with(&[
        0x3E, 0x10, // LD A, 10h
        0xD6, 0x20, // SUB 20h
    ]);
    cpu.step(); cpu.step();
    assert_eq!(cpu.a, 0xF0);
    assert!(cpu.f & flags::C != 0, "carry (borrow) should be set");
    assert!(cpu.f & flags::N != 0, "subtract flag should be set");
}

#[test]
fn jp_nn() {
    let mut cpu = cpu_with(&[
        0xC3, 0x10, 0x00, // JP 0010h
    ]);
    cpu.step();
    assert_eq!(cpu.pc, 0x0010);
}

#[test]
fn jr_forward_backward() {
    let mut cpu = cpu_with(&[
        0x18, 0x02, // JR +2 (skip next 2 bytes)
        0x00, 0x00, // skipped
        0x18, 0xFC, // JR -4 (back to address 2) — but we won't run this
    ]);
    cpu.step();
    assert_eq!(cpu.pc, 4); // 0 + 2 (instruction length) + 2 (displacement)
}

#[test]
fn jr_nz_taken_and_not_taken() {
    let mut cpu = cpu_with(&[
        0x3E, 0x01, // LD A, 1
        0xD6, 0x01, // SUB 1 — A=0, Z flag set
        0x20, 0x02, // JR NZ, +2 — NOT taken (Z is set)
        0x00,       // NOP (we land here)
        0x00,       // NOP
    ]);
    cpu.step(); cpu.step(); cpu.step();
    assert_eq!(cpu.pc, 6); // JR was not taken, PC at next instruction
}

#[test]
fn call_and_ret() {
    // CALL 0010h at addr 0, subroutine at 0010h does RET
    let mut cpu = cpu_with(&[
        0xCD, 0x10, 0x00, // CALL 0010h
    ]);
    cpu.mem[0x10] = 0xC9; // RET
    cpu.step(); // CALL
    assert_eq!(cpu.pc, 0x0010);
    cpu.step(); // RET
    assert_eq!(cpu.pc, 0x0003);
}

#[test]
fn ex_de_hl() {
    let mut cpu = cpu_with(&[
        0x21, 0x34, 0x12, // LD HL, 1234h
        0x11, 0x78, 0x56, // LD DE, 5678h
        0xEB,             // EX DE, HL
    ]);
    cpu.step(); cpu.step(); cpu.step();
    assert_eq!(cpu.hl(), 0x5678);
    assert_eq!(cpu.de(), 0x1234);
}

#[test]
fn exx_swaps_shadow() {
    let mut cpu = cpu_with(&[
        0x01, 0x11, 0x11, // LD BC, 1111h
        0x11, 0x22, 0x22, // LD DE, 2222h
        0x21, 0x33, 0x33, // LD HL, 3333h
        0xD9,             // EXX
    ]);
    cpu.step(); cpu.step(); cpu.step(); cpu.step();
    // After EXX, main regs should be 0 (from init), shadow should have old values
    assert_eq!(cpu.bc(), 0);
    assert_eq!(cpu.b_, 0x11);
    assert_eq!(cpu.c_, 0x11);
}

#[test]
fn sbc_hl_de() {
    let mut cpu = cpu_with(&[
        0x21, 0x00, 0x10, // LD HL, 1000h
        0x11, 0x01, 0x00, // LD DE, 0001h
        0xA7,             // AND A (clear carry)
        0xED, 0x52,       // SBC HL, DE
    ]);
    cpu.step(); cpu.step(); cpu.step(); cpu.step();
    assert_eq!(cpu.hl(), 0x0FFF);
    assert!(cpu.f & flags::C == 0);
}

#[test]
fn inc_dec_register() {
    let mut cpu = cpu_with(&[
        0x3E, 0xFF, // LD A, FFh
        0x3C,       // INC A → 00, Z set
    ]);
    cpu.step(); cpu.step();
    assert_eq!(cpu.a, 0);
    assert!(cpu.f & flags::Z != 0);
}

#[test]
fn djnz_loop() {
    // Count B from 3 down to 0
    let mut cpu = cpu_with(&[
        0x3E, 0x00, // LD A, 0
        0x06, 0x03, // LD B, 3
        0x3C,       // INC A (at addr 4)
        0x10, 0xFD, // DJNZ -3 (back to addr 4)
    ]);
    cpu.step(); cpu.step(); // LD A,0; LD B,3
    for _ in 0..3 { cpu.step(); cpu.step(); } // 3 iterations of INC A + DJNZ
    assert_eq!(cpu.a, 3);
}

#[test]
fn ldir_block_copy() {
    let mut cpu = cpu_with(&[
        0x21, 0x00, 0x80, // LD HL, 8000h (source)
        0x11, 0x00, 0x90, // LD DE, 9000h (dest)
        0x01, 0x04, 0x00, // LD BC, 4 (count)
        0xED, 0xB0,       // LDIR
    ]);
    cpu.mem[0x8000] = 0xAA;
    cpu.mem[0x8001] = 0xBB;
    cpu.mem[0x8002] = 0xCC;
    cpu.mem[0x8003] = 0xDD;
    cpu.step(); cpu.step(); cpu.step();
    // LDIR re-executes via PC-2 until BC=0, each byte is one step()
    for _ in 0..4 { cpu.step(); }
    assert_eq!(cpu.mem[0x9000], 0xAA);
    assert_eq!(cpu.mem[0x9001], 0xBB);
    assert_eq!(cpu.mem[0x9002], 0xCC);
    assert_eq!(cpu.mem[0x9003], 0xDD);
    assert_eq!(cpu.bc(), 0);
}

#[test]
fn ix_indexed_load() {
    let mut cpu = cpu_with(&[
        0xDD, 0x21, 0x00, 0x80, // LD IX, 8000h
        0xDD, 0x7E, 0x05,       // LD A, (IX+5)
    ]);
    cpu.mem[0x8005] = 0x42;
    cpu.step(); cpu.step();
    assert_eq!(cpu.a, 0x42);
}

#[test]
fn jp_iy() {
    let mut cpu = cpu_with(&[
        0xFD, 0x21, 0x00, 0x10, // LD IY, 1000h
        0xFD, 0xE9,             // JP (IY)
    ]);
    cpu.step(); cpu.step();
    assert_eq!(cpu.pc, 0x1000);
}

#[test]
fn bit_7_d() {
    let mut cpu = cpu_with(&[
        0x16, 0x80, // LD D, 80h (bit 7 set)
        0xCB, 0x7A, // BIT 7, D
    ]);
    cpu.step(); cpu.step();
    assert!(cpu.f & flags::Z == 0, "bit 7 is set so Z should be clear");
}

#[test]
fn cycles_accumulate() {
    let mut cpu = cpu_with(&[0x00, 0x00, 0x00]); // 3 NOPs
    cpu.step(); cpu.step(); cpu.step();
    assert_eq!(cpu.cycles, 12); // 3 * 4 T-states
}
