use z80::cpu::Cpu;

pub const BDOS_ENTRY: u16 = 0x0005;
pub const BDOS_ADDR: u16 = 0xE400;
pub const CCP_ENTRY: u16 = 0xD000;
pub const BIOS_BASE: u16 = 0xFA00;
pub const BIOS_HANDLERS: u16 = BIOS_BASE + 17 * 3;
pub const TPA_BASE: u16 = 0x0100;
pub const DMA_DEFAULT: u16 = 0x0080;

pub fn setup_page_zero(cpu: &mut Cpu) {
    cpu.mem[0x0000] = 0xC3;
    cpu.write16(0x0001, BIOS_BASE + 3);
    cpu.mem[0x0003] = 0x00;
    cpu.mem[0x0004] = 0x00;
    cpu.mem[0x0005] = 0xC3;
    cpu.write16(0x0006, BDOS_ADDR);
}

pub fn setup_bios(cpu: &mut Cpu) {
    for i in 0..17u16 {
        let entry = BIOS_BASE + i * 3;
        let handler = BIOS_HANDLERS + i;
        cpu.mem[entry as usize] = 0xC3;
        cpu.write16(entry + 1, handler);
    }
    for i in 0..17u16 {
        cpu.mem[(BIOS_HANDLERS + i) as usize] = 0xC9;
    }
}

pub fn load_com(cpu: &mut Cpu, data: &[u8], args: &str) {
    let len = data.len().min((BDOS_ADDR - TPA_BASE) as usize);
    cpu.mem[TPA_BASE as usize..TPA_BASE as usize + len].copy_from_slice(&data[..len]);

    let tail = args.as_bytes();
    let tail_len = tail.len().min(127);
    cpu.mem[0x0080] = tail_len as u8;
    if tail_len > 0 {
        cpu.mem[0x0081..0x0081 + tail_len].copy_from_slice(&tail[..tail_len]);
    }

    for i in 0x005Cu16..0x0080 {
        cpu.mem[i as usize] = 0;
    }
    for i in 1u16..12 {
        cpu.mem[(0x005C + i) as usize] = b' ';
        cpu.mem[(0x006C + i) as usize] = b' ';
    }

    cpu.pc = TPA_BASE;
    cpu.sp = BDOS_ADDR;
    cpu.push16(0x0000);
}
