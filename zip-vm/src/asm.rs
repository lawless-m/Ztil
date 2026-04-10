use std::collections::HashMap;

/// Minimal two-pass Z80 assembler — just enough to build the ZIP ROM.
/// Labels, absolute 16-bit fixups, and relative 8-bit fixups (JR/DJNZ).
pub struct Asm {
    pub code: Vec<u8>,
    pub origin: u16,
    labels: HashMap<String, u16>,
    fixups_abs: Vec<(usize, String)>,
    fixups_rel: Vec<(usize, String)>,
}

impl Asm {
    pub fn new(origin: u16) -> Self {
        Asm {
            code: Vec::new(),
            origin,
            labels: HashMap::new(),
            fixups_abs: Vec::new(),
            fixups_rel: Vec::new(),
        }
    }

    /// Current emission address.
    pub fn pc(&self) -> u16 {
        self.origin + self.code.len() as u16
    }

    /// Emit raw bytes.
    pub fn emit(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    /// Emit a single byte.
    pub fn db(&mut self, b: u8) {
        self.code.push(b);
    }

    /// Emit a 16-bit value, little-endian.
    pub fn dw_val(&mut self, v: u16) {
        self.code.push(v as u8);
        self.code.push((v >> 8) as u8);
    }

    /// Define a label at the current position.
    pub fn label(&mut self, name: &str) {
        let pc = self.pc();
        self.labels.insert(name.to_string(), pc);
    }

    /// Get a label's address (panics if undefined — call after resolve()).
    pub fn addr(&self, name: &str) -> u16 {
        *self.labels.get(name).unwrap_or_else(|| panic!("undefined label: {}", name))
    }

    /// Emit a 16-bit label reference (forward references OK — resolved later).
    pub fn dw_label(&mut self, name: &str) {
        let pos = self.code.len();
        self.code.push(0); self.code.push(0); // placeholder
        self.fixups_abs.push((pos, name.to_string()));
    }

    /// Emit `*+2` — a code address field pointing to the next byte.
    pub fn emit_code_addr(&mut self) {
        let ca = self.pc() + 2;
        self.dw_val(ca);
    }

    /// Emit a relative jump placeholder to a label (for JR/DJNZ).
    /// Caller must have already emitted the opcode byte.
    pub fn jr_target(&mut self, name: &str) {
        let pos = self.code.len();
        self.code.push(0); // placeholder displacement
        self.fixups_rel.push((pos, name.to_string()));
    }

    /// Emit JR opcode + displacement to label.
    pub fn jr(&mut self, name: &str) {
        self.db(0x18);
        self.jr_target(name);
    }

    /// Emit JR NZ + displacement.
    pub fn jr_nz(&mut self, name: &str) {
        self.db(0x20);
        self.jr_target(name);
    }

    /// Emit JR Z + displacement.
    pub fn jr_z(&mut self, name: &str) {
        self.db(0x28);
        self.jr_target(name);
    }

    /// Emit JR C + displacement.
    pub fn jr_c(&mut self, name: &str) {
        self.db(0x38);
        self.jr_target(name);
    }

    /// Emit JR NC + displacement.
    pub fn jr_nc(&mut self, name: &str) {
        self.db(0x30);
        self.jr_target(name);
    }

    /// Emit DJNZ + displacement.
    pub fn djnz(&mut self, name: &str) {
        self.db(0x10);
        self.jr_target(name);
    }

    /// Emit JP nn to label.
    pub fn jp(&mut self, name: &str) {
        self.db(0xC3);
        self.dw_label(name);
    }

    /// Emit JP Z,nn to label.
    pub fn jp_z(&mut self, name: &str) {
        self.db(0xCA);
        self.dw_label(name);
    }

    /// Emit JP P,nn to label.
    pub fn jp_p(&mut self, name: &str) {
        self.db(0xF2);
        self.dw_label(name);
    }

    /// Emit JP M,nn to label.
    pub fn jp_m(&mut self, name: &str) {
        self.db(0xFA);
        self.dw_label(name);
    }

    /// Emit CALL nn to label.
    pub fn call(&mut self, name: &str) {
        self.db(0xCD);
        self.dw_label(name);
    }

    /// Emit a dictionary header: length byte, 3 name chars, link to previous header.
    /// Returns nothing; the word address follows at current position after this.
    pub fn header(&mut self, name: &str, link_label: Option<&str>, immediate: bool) {
        let len = name.len() as u8 | if immediate { 0x80 } else { 0 };
        self.db(len);
        // First 3 chars, space-padded
        let name_bytes = name.as_bytes();
        for i in 0..3 {
            self.db(if i < name_bytes.len() { name_bytes[i] } else { 0x20 });
        }
        // Link
        if let Some(prev) = link_label {
            self.dw_label(prev);
        } else {
            self.dw_val(0x0000); // end of chain
        }
    }

    /// Resolve all forward references. Call once after all code is emitted.
    pub fn resolve(&mut self) {
        for (pos, name) in &self.fixups_abs {
            let addr = *self.labels.get(name.as_str())
                .unwrap_or_else(|| panic!("unresolved label: {}", name));
            self.code[*pos] = addr as u8;
            self.code[*pos + 1] = (addr >> 8) as u8;
        }
        for (pos, name) in &self.fixups_rel {
            let addr = *self.labels.get(name.as_str())
                .unwrap_or_else(|| panic!("unresolved label: {}", name));
            let from = self.origin as i32 + *pos as i32 + 1; // PC after displacement byte
            let disp = addr as i32 - from;
            assert!(disp >= -128 && disp <= 127,
                "relative jump to {} out of range: {} (from {:04X} to {:04X})",
                name, disp, from, addr);
            self.code[*pos] = disp as u8;
        }
    }

    /// Load the assembled code into a CPU's memory at the origin address.
    pub fn load_into(&self, mem: &mut [u8; 0x10000]) {
        let start = self.origin as usize;
        mem[start..start + self.code.len()].copy_from_slice(&self.code);
    }
}
