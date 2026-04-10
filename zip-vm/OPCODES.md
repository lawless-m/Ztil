# ZIP Z80 Opcode Inventory

Authoritative list of every Z80 instruction used by the ZIP TIL system,
extracted from `zip-til-docs/01-inner-interpreter.md` through
`07-assembler-extension.md`.

Sources are coded: **01**=inner interpreter, **02**=outer interpreter,
**03**=branch/control primitives, **04**=word definitions, **05**=secondaries,
**07**=assembler extension (instructions the assembler can emit at runtime).

Instructions from 01-04 are used directly in the ZIP kernel. Instructions
from 07 are available to user-written CODE words via the assembler.

## Kernel Instructions (files 01-05)

These are the instructions that appear in actual Z80 code blocks in the
ZIP source. Implementing these is required to run the ZIP kernel.

### Load Group

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `LD A,(BC)` | 0A | 01,02,03 | ~12 | NEXT fetch, branch offset fetch |
| `LD A,(DE)` | 1A | 03,04 | ~3 | *SYS variable access, CCONSTANT |
| `LD A,(HL)` | 7E | 02 | ~6 | NUMBER, SEARCH character fetch |
| `LD A,(nn)` | 3A nn nn | 02 | ~2 | LD A,(BASE) |
| `LD (HL),A` | 77 | 02,04 | ~10 | INLINE, +!, C+! |
| `LD (HL),n` | 36 nn | 04 | ~5 | INLINE clear, 0!, 1! |
| `LD (HL),E` | 73 | 04 | ~8 | !, C!, comma, C, |
| `LD (HL),D` | 72 | 04 | ~5 | !, comma |
| `LD (nn),A` | 32 nn nn | 02,04 | ~4 | LD (BASE),A |
| `LD (nn),HL` | 22 nn nn | 02,04 | ~4 | LD (DP),HL, (LBP),HL, (LBEND),HL |
| `LD A,n` | 3E nn | 02,03,04 | ~15 | Load immediate byte constants |
| `LD A,B` | 78 | 02 | ~1 | SEARCH test link high |
| `LD A,C` | 79 | 02,04 | ~3 | INLINE save char, $UD/ |
| `LD A,D` | 7A | 02,04 | ~5 | $ISIGN, $UD/, D/MOD |
| `LD A,E` | 7B | 02,04 | ~3 | CCONSTANT, $UD* |
| `LD A,H` | 7C | 02,04 | ~3 | SEARCH test high, $US/ |
| `LD A,L` | 7D | 02,03,04 | ~15 | Flag tests, ALU, *IF/*END |
| `LD B,A` | 47 | 02 | ~2 | SEARCH set count |
| `LD B,D` | 42 | 01 | ~1 | COLON IR=DE |
| `LD B,0` | 06 00 | 02 | ~2 | TOKEN |
| `LD B,n` | 06 nn | 02,04 | ~5 | Loop counts (8, 10h, LENGTH) |
| `LD B,(HL)` | 46 | 02 | ~1 | NUMBER get length |
| `LD C,A` | 4F | 02,04 | ~4 | INLINE, $UD* |
| `LD C,B` | 48 | 02 | ~1 | TOKEN |
| `LD C,E` | 4B | 01 | ~1 | COLON IR=DE |
| `LD C,1` | 0E 01 | 02 | ~1 | SEARCH flag true |
| `LD C,0` | 0E 00 | 02 | ~1 | SEARCH flag false |
| `LD C,n` | 0E nn | 02 | ~2 | Small constants |
| `LD D,A` | 57 | 03,04 | ~4 | *#, $UD/, D/MOD |
| `LD D,(HL)` | 56 | 01,03 | ~3 | RUN dereference, *LOOP |
| `LD E,A` | 5F | 02,03,04 | ~8 | NUMBER, *#, *C#, ASCII |
| `LD E,(HL)` | 5E | 01,02,04 | ~6 | RUN, SEARCH link, @, TYPE |
| `LD E,n` | 1E nn | 04 | ~1 | <# terminator (A0h) |
| `LD H,A` | 67 | 01,02,04 | ~6 | NEXT, logical ops |
| `LD H,B` | 60 | 04 | ~1 | D/MOD |
| `LD H,C` | 61 | 04 | ~2 | $UD*, MLOOP |
| `LD H,0` | 26 00 | 04 | ~4 | CI>, CJ>, CR> zero extend |
| `LD H,n` | 26 nn | 04 | ~4 | Zero-extend byte values |
| `LD L,A` | 6F | 01,02,03,04 | ~15 | Throughout — NEXT, *SYS, flag, etc. |
| `LD L,C` | 69 | 04 | ~2 | $ISIGN, D/MOD |
| `LD L,H` | 6C | 04 | ~1 | SINGLE |
| `LD L,0` | 2E 00 | 02 | ~1 | INLINE reset |
| `LD L,n` | 2E nn | 02,04 | ~2 | Byte constants |
| `LD C,(HL)` | 4E | 05 | ~1 | DOES> load new IR |
| `LD B,(HL)` | 46 | 02,05 | ~2 | NUMBER, DOES> |
| `LD HL,nn` | 21 nn nn | 02,03,04 | ~20 | Load addresses and constants |
| `LD HL,(nn)` | 2A nn nn | 02,04 | ~6 | LD HL,(DP), (LBP), (CURRENT) |
| `LD DE,nn` | 11 nn nn | 02,03,04 | ~10 | Address/constant loads |
| `LD BC,nn` | 01 nn nn | 02,04 | ~3 | OUTER, 0800h |
| `LD SP,nn` | 31 nn nn | 02,04 | ~3 | LD SP,STACK |
| `LD SP,HL` | F9 | 02 | ~1 | *STACK reset |
| `LD IY,nn` | FD 21 nn nn | 02,04 | ~3 | LD IY,NEXT, LD IY,RETURN |
| `LD IX,nn` | DD 21 nn nn | 02 | ~1 | LD IX,RETURN |
| `LD C,(IX+d)` | DD 4E dd | 01 | ~1 | SEMI pop low byte |
| `LD B,(IX+d)` | DD 46 dd | 01 | ~1 | SEMI pop high byte |
| `LD (IX+d),B` | DD 70 dd | 01,05 | ~2 | COLON, DOES> push IR |
| `LD (IX+d),C` | DD 71 dd | 01,05 | ~2 | COLON, DOES> push IR |
| `LD (IX+d),L` | DD 75 dd | 03,04 | ~6 | *DO, *CDO, <R, C<R |
| `LD (IX+d),H` | DD 74 dd | 03,04 | ~4 | *DO, <R, <# |
| `LD L,(IX+d)` | DD 6E dd | 04 | ~6 | R>, I>, J>, CI>, CJ>, CR> |
| `LD H,(IX+d)` | DD 66 dd | 04 | ~4 | R>, I>, J> |
| `LD A,(IX+d)` | DD 7E dd | 03 | ~4 | *LEAVE, *CLEAVE |
| `LD (IX+d),A` | DD 77 dd | 03 | ~4 | *LEAVE, *CLEAVE |
| `LD (HL),0` | 36 00 | 04 | ~2 | 0!, 1! high byte |

### Arithmetic Group

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `ADD A,C` | 81 | 02,03 | ~4 | $ELSE/$WHILE offset, $PATCH |
| `ADD A,E` | 83 | 03 | ~2 | *C+LOOP |
| `ADD A,L` | 85 | 03 | ~2 | *SYS offset |
| `ADD A,n` | C6 nn | 04 | ~3 | ASCII add bias |
| `ADD A,(HL)` | 86 | 03 | ~1 | *LOOP increment |
| `ADD A,A` | 87 | — | — | Not in kernel (used by $UD* as ADC A,A with prior ADD HL,HL) |
| `ADC A,A` | 8F | 02,04 | ~3 | NUMBER multiply shift, $UD* |
| `ADC A,C` | 89 | 04 | ~1 | $UD* carry propagate |
| `ADC A,D` | 8A | 04 | ~2 | $UD/ shift high byte |
| `ADD HL,DE` | 19 | 02,04 | ~6 | SEARCH, +, NUMBER, $UD* |
| `ADD HL,HL` | 29 | 04 | ~5 | 2*, $UD*, $US*, $UD/, $US/ |
| `ADD HL,SP` | 39 | 04 | ~1 | +SP |
| `ADD HL,BC` | 09 | 04 | ~1 | MOVE |
| `ADD IX,DE` | DD 19 | 03 | ~2 | *DO, *LOOP adjust return stack |
| `SUB n` | D6 nn | 02 | ~2 | NUMBER subtract bias |
| `SUB (HL)` | 96 | 03 | ~2 | *LOOP/*CLOOP terminator compare |
| `SUB E` | 93 | 04 | ~2 | $UD/, $US/ |
| `SBC A,A` | 9F | 03,04 | ~4 | Sign extend (FF if neg, 00 if pos) |
| `SBC A,(HL)` | 9E | 03 | ~1 | *LOOP high byte compare |
| `SBC HL,DE` | ED 52 | 02,04 | ~12 | Subtract, -, MINUS, ABS, comparisons |
| `SBC HL,BC` | ED 42 | 04 | ~2 | D*, D/MOD |
| `SBC HL,SP` | ED 72 | 02,04 | ~2 | *STACK, -SP |
| `INC HL` | 23 | 01,02,03,04,05 | ~30 | Very frequent — pointer advance |
| `INC BC` | 03 | 01,02,03 | ~10 | Advance IR |
| `INC IX` | DD 23 | 01,03,04 | ~10 | Pop return stack |
| `INC DE` | 13 | 02 | ~2 | SEARCH dictionary pointer advance |
| `INC L` | 2C | 02 | ~5 | INLINE/TOKEN buffer pointer |
| `INC E` | 1C | 04 | ~4 | Flag true (INC E from 0 to 1) |
| `INC B` | 04 | 02,03 | ~3 | TOKEN count, $ELSE page cross |
| `INC C` | 0C | 04 | ~1 | D* 2's complement bump |
| `INC A` | 3C | 02 | ~1 | NUMBER restore BASE |
| `INC (HL)` | 34 | 03 | ~2 | *LOOP/*CLOOP increment |
| `DEC HL` | 2B | 02,04 | ~4 | 2-, $PATCH |
| `DEC IX` | DD 2B | 01,03,04,05 | ~10 | Push to return stack |
| `DEC DE` | 1B | 02 | ~1 | $PATCH |
| `DEC L` | 2D | 02 | ~2 | INLINE backspace |
| `DEC B` | 05 | 02 | ~2 | NUMBER, implicit via DJNZ |
| `DEC A` | 3D | 02 | ~1 | NUMBER sign flag |
| `DEC D` | — | 03 | ~1 | *[ string length |
| `DEC E` | 1D | 04 | ~1 | TYPE length |
| `NEG` | ED 44 | 04 | ~2 | */MOD, D/MOD negate divisor |

### Logic Group

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `AND A` | A7 | 02,04 | ~15 | Reset carry flag (CY=0) |
| `AND E` | A3 | 04 | ~2 | Bitwise AND low bytes |
| `AND D` | A2 | 04 | ~2 | Bitwise AND high bytes |
| `OR H` | B4 | 02,03,04 | ~8 | Test HL for zero (with LD A,L first) |
| `OR L` | B5 | 02 | ~2 | SEARCH test link |
| `OR E` | B3 | 04 | ~2 | IOR low bytes |
| `OR D` | B2 | 04 | ~2 | IOR high bytes |
| `XOR E` | AB | 04 | ~2 | XOR low bytes |
| `XOR D` | AA | 04 | ~2 | XOR high bytes |
| `XOR B` | A8 | 04 | ~1 | $ISIGN sign XOR |
| `XOR L` | AD | 04 | ~1 | */MOD divisor sign |
| `CPL` | 2F | 04 | ~2 | D*, D/MOD complement |
| `CP n` | FE nn | 02,04 | ~10 | INLINE char tests, NUMBER range |
| `CP (HL)` | BE | 02 | ~4 | TOKEN/SEARCH char compare |
| `CP C` | B9 | 02 | ~2 | TOKEN separator compare |
| `CP E` | BB | 02 | ~1 | NUMBER valid digit test |

### Rotate/Shift Group

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `RLA` | 17 | 02,03,04 | ~5 | Sign to carry, *C#, C@, TOKEN |
| `SRA H` | CB 2C | 04 | ~1 | 2/ arithmetic shift right |
| `RR L` | CB 1D | 04 | ~1 | 2/ propagate carry |

### Bit Group

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `BIT 7,L` | CB 7D | 02 | ~1 | INLINE buffer full test |
| `BIT 7,(HL)` | CB 7E | 02 | ~1 | QUESTION terminator test |
| `BIT 7,D` | CB 7A | 04 | ~1 | ABS sign test |
| `BIT 7,(IX+d)` | DD CB dd 7E | 04 | ~1 | SIGN return stack sign |
| `RES 5,(HL)` | CB AE | 02 | ~1 | INLINE lowercase→uppercase |

### Branch Group

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `JP nn` | C3 nn nn | 02,03,04 | ~15 | JP NEXT, JP START, JP ABORT, etc. |
| `JP (HL)` | E9 | 01 | ~1 | RUN jump to code |
| `JP (IY)` | FD E9 | 01,02,03,04,05 | ~30+ | Return to NEXT (ends every primitive) |
| `JP Z,nn` | CA nn nn | 02,03 | ~3 | *IF, *END conditional |
| `JP C,nn` | DA nn nn | 03 | ~2 | *LOOP/*CLOOP branch back |
| `JP M,nn` | FA nn nn | 02,04 | ~4 | INLINE, *, $UD/, $US/ |
| `JP P,nn` | F2 nn nn | 04 | ~6 | $ISIGN, D*, */MOD, D/MOD, MAX |
| `JP NC,nn` | D2 nn nn | — | — | Not used in kernel |
| `JR n` | 18 dd | 01,02,03,04 | ~10 | Unconditional relative |
| `JR Z,n` | 28 dd | 02,04 | ~6 | Branch on zero |
| `JR NZ,n` | 20 dd | 02,03,04 | ~15 | Branch on nonzero |
| `JR C,n` | 38 dd | 02,03 | ~6 | Branch on carry |
| `JR NC,n` | 30 dd | 02,03,04 | ~8 | Branch on no carry |
| `DJNZ n` | 10 dd | 02,03,04 | ~10 | Loop control |
| `CALL nn` | CD nn nn | 02,04 | ~15 | CALL $ECHO, $KEY, $CRLF, subroutines |
| `RET` | C9 | 04 | ~5 | $CRLF, $ISIGN, $UD*, $US*, $UD/, $US/ |
| `RET P` | F0 | 04 | ~2 | $ISIGN, $OSIGN early return |

### Stack/Exchange Group

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `PUSH HL` | E5 | 01,02,03,04,05 | ~40+ | Very frequent — push result |
| `PUSH DE` | D5 | 02,04 | ~15 | Push data/result |
| `PUSH BC` | C5 | 02,04 | ~8 | Save IR or count |
| `PUSH AF` | F5 | 02 | ~1 | NUMBER push True flag |
| `PUSH IX` | DD E5 | 03,04 | ~4 | *LOOP, *CLOOP, ?RS |
| `POP HL` | E1 | 01,02,03,04 | ~40+ | Very frequent — get from stack |
| `POP DE` | D1 | 02,04 | ~20 | Get data from stack |
| `POP BC` | C1 | 02,04 | ~10 | Get from stack (TOKEN, *, etc.) |
| `EX DE,HL` | EB | 01,02,04,05 | ~15 | Exchange DE and HL |
| `EX (SP),HL` | E3 | 02,04 | ~6 | NUMBER, SWAP, LROT, RROT, MOVE |
| `EX AF,AF'` | 08 | 02,04 | ~10 | Sign flag save/restore |
| `EXX` | D9 | 02,04 | ~20 | Save/restore IR in alternate regs |

### Block Transfer Group

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `LDIR` | ED B0 | 02,04 | ~2 | TOKEN, MOVE forward |
| `LDDR` | ED B8 | 04 | ~1 | MOVE backward |

### Miscellaneous

| Instruction | Opcode(s) | Files | Count | Notes |
|------------|-----------|-------|-------|-------|
| `SCF` | 37 | 02 | ~1 | NUMBER set True flag |
| `NOP` | 00 | — | — | Not used in kernel (only in assembler vocab) |

## Kernel Instruction Summary

Total unique instruction forms in the kernel: **~105**

### By frequency tier

**Tier 1 — Core (>10 uses):** `JP (IY)`, `PUSH HL`, `POP HL`, `INC HL`,
`LD A,n`, `LD L,A`, `JR NZ,n`, `LD HL,nn`, `JP nn`, `AND A`,
`SBC HL,DE`, `CALL nn`, `EXX`, `EX DE,HL`, `LD A,(BC)`, `DJNZ`,
`CP n`, `POP DE`, `PUSH DE`, `INC BC`, `INC IX`, `DEC IX`,
`EX AF,AF'`, `ADD HL,DE`

**Tier 2 — Common (4-10 uses):** `JR C,n`, `JR NC,n`, `JR Z,n`,
`LD DE,nn`, `LD E,A`, `LD H,A`, `JP M,nn`, `JP P,nn`,
`LD (HL),E`, `LD (HL),D`, `LD (HL),A`, `ADD HL,HL`, `POP BC`,
`PUSH BC`, `LD A,L`, `LD A,D`, `LD D,(HL)`, `LD E,(HL)`,
`LD (nn),HL`, `LD HL,(nn)`, `EX (SP),HL`, `OR H`, `SBC A,A`,
`RLA`, `LD (IX+d),L`, `LD L,(IX+d)`, `INC L`, `INC E`,
`PUSH IX`

**Tier 3 — Occasional (1-3 uses):** `JP (HL)`, `JR n`, `RET`, `RET P`,
`PUSH AF`, `SCF`, `NEG`, `SRA H`, `RR L`, `RES 5,(HL)`,
`BIT 7,L`, `BIT 7,(HL)`, `BIT 7,D`, `BIT 7,(IX+d)`, `LDIR`,
`LDDR`, `CPL`, `SUB n`, `SUB (HL)`, `SUB E`, `ADD A,C`,
`ADD A,E`, `ADD A,L`, `ADD A,n`, `ADD A,(HL)`, `ADC A,A`,
`ADC A,C`, `ADC A,D`, `SBC A,(HL)`, `SBC HL,BC`, `SBC HL,SP`,
`ADD HL,SP`, `ADD HL,BC`, `ADD IX,DE`, `INC (HL)`, `LD SP,nn`,
`LD SP,HL`, `LD IY,nn`, `LD IX,nn`, `JP Z,nn`, `JP C,nn`,
`LD (nn),A`, `LD A,(nn)`, `LD A,(HL)`, `LD A,(DE)`,
`LD (HL),n`, `OR L`, `OR E`, `OR D`, `AND E`, `AND D`,
`XOR E`, `XOR D`, `XOR B`, `XOR L`, `CP (HL)`, `CP C`, `CP E`,
`DEC HL`, `DEC DE`, `DEC L`, `DEC D`, `DEC E`, `DEC A`,
`INC DE`, `INC B`, `INC C`, `INC A`

## Assembler-Emittable Instructions (file 07)

These are the Z80 instructions that CODE words can generate via the
assembler extension. They include the kernel set above plus these
additional forms. Implementing these is needed to support user-defined
CODE words.

### Additional Load Instructions

| Instruction | Opcode(s) | Assembler | Notes |
|------------|-----------|-----------|-------|
| `LD r,r'` | 40+8*r+r' | MOV, | All 49 register-register combos (excl. LD (HL),(HL)) |
| `LD r,n` | 06+8*r, n | MVI, | Immediate to any 8-bit register |
| `LD r,(IX+d)` | DD 46+8*r, d | MOV, | Indexed source |
| `LD r,(IY+d)` | FD 46+8*r, d | MOV, | Indexed source |
| `LD (IX+d),r` | DD 70+r, d | MOV, | Indexed destination |
| `LD (IY+d),r` | FD 70+r, d | MOV, | Indexed destination |
| `LD (IX+d),n` | DD 36, d, n | MVI, | Indexed immediate |
| `LD (IY+d),n` | FD 36, d, n | MVI, | Indexed immediate |
| `LD A,(BC)` | 0A | LDA, | Load A indirect |
| `LD A,(DE)` | 1A | LDA, | Load A indirect |
| `LD A,(nn)` | 3A nn nn | LDA, | Load A extended |
| `LD (BC),A` | 02 | STA, | Store A indirect |
| `LD (DE),A` | 12 | STA, | Store A indirect |
| `LD (nn),A` | 32 nn nn | STA, | Store A extended |
| `LD rp,nn` | 01/11/21/31 nn nn | DMI, | 16-bit immediate |
| `LD (nn),HL` | 22 nn nn | DSM, | 16-bit store |
| `LD (nn),rp` | ED 43/53/63/73 nn nn | DSM, | 16-bit store (non-HL) |
| `LD HL,(nn)` | 2A nn nn | DLM, | 16-bit load |
| `LD rp,(nn)` | ED 4B/5B/6B/7B nn nn | DLM, | 16-bit load (non-HL) |
| `LD SP,HL` | F9 | LSP, | Also IX/IY with prefix |
| `LD A,I` | ED 57 | LAI, | Interrupt vector register |
| `LD A,R` | ED 5F | LAR, | Refresh register |
| `LD I,A` | ED 47 | LIA, | Set interrupt vector |
| `LD R,A` | ED 4F | LRA, | Set refresh register |
| `PUSH rp` | C5/D5/E5/F5 | PSH, | Push register pair |
| `POP rp` | C1/D1/E1/F1 | POP, | Pop register pair |

### Additional Arithmetic/Logic Instructions

| Instruction | Opcode(s) | Assembler | Notes |
|------------|-----------|-----------|-------|
| `ADD A,r` | 80+r | ADD, | 8-bit add |
| `ADD A,(IX+d)` | DD 86, d | ADD, | Indexed add |
| `ADD A,(IY+d)` | FD 86, d | ADD, | Indexed add |
| `ADC A,r` | 88+r | ADC, | Add with carry |
| `SUB r` | 90+r | SUB, | 8-bit subtract |
| `SBC A,r` | 98+r | SBC, | Subtract with borrow |
| `AND r` | A0+r | AND, | Bitwise AND |
| `XOR r` | A8+r | XOR, | Bitwise XOR |
| `OR r` | B0+r | IOR, | Bitwise OR |
| `CP r` | B8+r | CMP, | Compare |
| `ADD A,n` | C6 nn | ADI, | Immediate add |
| `ADC A,n` | CE nn | ACI, | Immediate add with carry |
| `SUB n` | D6 nn | SUI, | Immediate subtract |
| `SBC A,n` | DE nn | SCI, | Immediate subtract w/ borrow |
| `AND n` | E6 nn | ANI, | Immediate AND |
| `XOR n` | EE nn | XOI, | Immediate XOR |
| `OR n` | F6 nn | ORI, | Immediate OR |
| `CP n` | FE nn | CPI, | Immediate compare |
| `INC r` | 04+8*r | INC, | 8-bit increment |
| `DEC r` | 05+8*r | DEC, | 8-bit decrement |
| `INC (IX+d)` | DD 34, d | INC, | Indexed increment |
| `DEC (IX+d)` | DD 35, d | DEC, | Indexed decrement |
| `ADD HL,rp` | 09/19/29/39 | DAD, | 16-bit add |
| `ADD IX,rp` | DD 09/19/29/39 | DAI, | Indexed 16-bit add |
| `ADD IY,rp` | FD 09/19/29/39 | DAI, | Indexed 16-bit add |
| `ADC HL,rp` | ED 4A/5A/6A/7A | DAC, | 16-bit add with carry |
| `SBC HL,rp` | ED 42/52/62/72 | DSC, | 16-bit subtract with carry |
| `INC rp` | 03/13/23/33 | DIN, | 16-bit increment |
| `DEC rp` | 0B/1B/2B/3B | DDC, | 16-bit decrement |
| `NEG` | ED 44 | NEG, | Negate A |
| `DAA` | 27 | DAA, | Decimal adjust |
| `CPL` | 2F | CPL, | Complement A |

### Additional Rotate/Shift Instructions

| Instruction | Opcode(s) | Assembler | Notes |
|------------|-----------|-----------|-------|
| `RLCA` | 07 | RLC, (A) | Rotate left circular A (1-byte form) |
| `RRCA` | 0F | RRC, (A) | Rotate right circular A (1-byte form) |
| `RLA` | 17 | RLT, (A) | Rotate left through carry A (1-byte form) |
| `RRA` | 1F | RRT, (A) | Rotate right through carry A (1-byte form) |
| `RLC r` | CB 00+r | RLC, | Rotate left circular |
| `RRC r` | CB 08+r | RRC, | Rotate right circular |
| `RL r` | CB 10+r | RLT, | Rotate left through carry |
| `RR r` | CB 18+r | RRT, | Rotate right through carry |
| `SLA r` | CB 20+r | SLR, | Shift left arithmetic |
| `SRA r` | CB 28+r | SRR, | Shift right arithmetic |
| `SRL r` | CB 38+r | SRL, | Shift right logical |
| `RLD` | ED 6F | RLD, | Rotate left digit (BCD) |
| `RRD` | ED 67 | RRD, | Rotate right digit (BCD) |

### Additional Bit Instructions

| Instruction | Opcode(s) | Assembler | Notes |
|------------|-----------|-----------|-------|
| `BIT b,r` | CB 40+8*b+r | BIT, | Test bit |
| `RES b,r` | CB 80+8*b+r | RES, | Reset bit |
| `SET b,r` | CB C0+8*b+r | SET, | Set bit |

### Additional Branch Instructions

| Instruction | Opcode(s) | Assembler | Notes |
|------------|-----------|-----------|-------|
| `JP nn` | C3 nn nn | JMP, | Unconditional absolute |
| `JP cc,nn` | C2/CA/D2/DA/E2/EA/F2/FA nn nn | JPC, | Conditional absolute |
| `JR e` | 18 dd | JPR, | Unconditional relative |
| `JR cc,e` | 20/28/30/38 dd | JRC, | Conditional relative (NZ/Z/NC/C only) |
| `JP (HL)` | E9 | JPM, | Jump indirect |
| `JP (IX)` | DD E9 | JPM, | Jump indirect indexed |
| `JP (IY)` | FD E9 | JPM, | Jump indirect indexed |
| `DJNZ e` | 10 dd | DJN, | Decrement B and jump |
| `CALL nn` | CD nn nn | CAL, | Unconditional call |
| `CALL cc,nn` | C4/CC/D4/DC/E4/EC/F4/FC nn nn | CLC, | Conditional call |
| `RET` | C9 | RET, | Unconditional return |
| `RET cc` | C0/C8/D0/D8/E0/E8/F0/F8 | RTC, | Conditional return |
| `RST n` | C7/CF/D7/DF/E7/EF/F7/FF | RST, | Restart |
| `RETI` | ED 4D | RTI, | Return from interrupt |
| `RETN` | ED 45 | RTN, | Return from NMI |

### Additional Exchange/Block Instructions

| Instruction | Opcode(s) | Assembler | Notes |
|------------|-----------|-----------|-------|
| `EX AF,AF'` | 08 | XAA, | Exchange AF pairs |
| `EXX` | D9 | XAL, | Exchange BC/DE/HL with alternates |
| `EX DE,HL` | EB | XDH, | Exchange DE and HL |
| `EX (SP),HL` | E3 | XST, | Exchange (SP) and HL |
| `EX (SP),IX` | DD E3 | XST, | Exchange (SP) and IX |
| `EX (SP),IY` | FD E3 | XST, | Exchange (SP) and IY |
| `LDI` | ED A0 | IC CLD, | Block load increment |
| `LDD` | ED A8 | DC CLD, | Block load decrement |
| `LDIR` | ED B0 | IR CLD, | Block load inc repeat |
| `LDDR` | ED B8 | DR CLD, | Block load dec repeat |
| `CPI` | ED A1 | IC CCP, | Block compare increment |
| `CPD` | ED A9 | DC CCP, | Block compare decrement |
| `CPIR` | ED B1 | IR CCP, | Block compare inc repeat |
| `CPDR` | ED B9 | DR CCP, | Block compare dec repeat |
| `INI` | ED A2 | IC CIN, | Block input increment |
| `IND` | ED AA | DC CIN, | Block input decrement |
| `INIR` | ED B2 | IR CIN, | Block input inc repeat |
| `INDR` | ED BA | DR CIN, | Block input dec repeat |
| `OUTI` | ED A3 | IC COT, | Block output increment |
| `OUTD` | ED AB | DC COT, | Block output decrement |
| `OTIR` | ED B3 | IR COT, | Block output inc repeat |
| `OTDR` | ED BB | DR COT, | Block output dec repeat |

### Additional Misc Instructions

| Instruction | Opcode(s) | Assembler | Notes |
|------------|-----------|-----------|-------|
| `NOP` | 00 | NOP, | No operation |
| `HALT` | 76 | HLT, | Halt CPU |
| `DI` | F3 | DSI, | Disable interrupts |
| `EI` | FB | ENI, | Enable interrupts |
| `IM 0` | ED 46 | IM0, | Interrupt mode 0 |
| `IM 1` | ED 56 | IM1, | Interrupt mode 1 |
| `IM 2` | ED 5E | IM2, | Interrupt mode 2 |
| `SCF` | 37 | SCF, | Set carry flag |
| `CCF` | 3F | CCF, | Complement carry flag |
| `XOR A` | AF | CLA, | Clear accumulator |
| `AND A` | A7 | RCF, | Reset carry flag |
| `IN A,(n)` | DB nn | INA, | Input from port |
| `OUT (n),A` | D3 nn | OTA, | Output to port |

## Unused Z80 Instruction Families

The following Z80 instruction families are **not used at all** in the ZIP
kernel code (files 01-04), and are only available if the assembler
extension is loaded:

| Family | Instructions | Assembler available? |
|--------|-------------|---------------------|
| Decimal adjust | `DAA` | Yes (DAA,) |
| Complement carry | `CCF` | Yes (CCF,) |
| Halt | `HALT` | Yes (HLT,) |
| Interrupts | `DI`, `EI`, `IM 0/1/2` | Yes (DSI, ENI, IM0/1/2,) |
| NMI return | `RETN` | Yes (RTN,) |
| Interrupt return | `RETI` | Yes (RTI,) |
| Restart | `RST n` | Yes (RST,) |
| I/O port | `IN A,(n)`, `OUT (n),A` | Yes (INA, OTA,) |
| I/O block | `INI/IND/INIR/INDR`, `OUTI/OUTD/OTIR/OTDR` | Yes (CIN, COT,) |
| Block compare | `CPI/CPD/CPIR/CPDR` | Yes (CCP,) |
| Block load single | `LDI`, `LDD` | Yes (CLD,) |
| Rotate accumulator short | `RLCA`, `RRCA`, `RRA` | Yes (RLC/RRC/RRT, with A) |
| BCD rotate | `RLD`, `RRD` | Yes (RLD, RRD,) |
| Shift left | `SLA r` | Yes (SLR,) |
| Shift right logical | `SRL r` | Yes (SRL,) |
| Conditional call | `CALL cc,nn` | Yes (CLC,) |
| Conditional jump abs | `JP cc,nn` (PE/PO/P/M) | Yes (JPC,) |
| Conditional return | `RET cc` (all except RET P) | Yes (RTC,) |
| Special registers | `LD A,I`, `LD A,R`, `LD I,A`, `LD R,A` | Yes (LAI, LAR, LIA, LRA,) |
| Store A indirect | `LD (BC),A`, `LD (DE),A` | Yes (STA,) |
| ED load/store | `LD (nn),rp` (non-HL), `LD rp,(nn)` (non-HL) | Yes (DSM, DLM,) |
| BIT/RES/SET general | Most bit combos | Yes (BIT, RES, SET,) |
| Indexed addressing | Most (IX+d)/(IY+d) ALU forms | Yes (via @X/@Y) |
| `NOP` | No operation | Yes (NOP,) |
| `EX (SP),IX/IY` | Indexed stack exchange | Yes (XST,) |
| `JP (IX)` | Jump indirect IX | Yes (JPM,) |

The only Z80 instruction not representable in the assembler is
`LD (HL),(HL)` (which is `HALT`, opcode 76).

## Implementation Priority

**Phase 1 — Required for kernel boot (inner interpreter + NEXT/RUN/COLON/SEMI):**
- `LD` (8-bit register-register, immediate, indirect via BC/HL/DE, indexed IX)
- `INC`/`DEC` (8-bit registers, 16-bit pairs, IX)
- `PUSH`/`POP` (HL, DE, BC, AF, IX)
- `JP nn`, `JP (HL)`, `JP (IY)`, `JR`, `JR cc`
- `EX DE,HL`, `EX AF,AF'`, `EXX`, `EX (SP),HL`
- `ADD HL,rp`, `SBC HL,rp` (DE, BC, SP)
- `ADD A,r`, `ADC A,r`, `SUB`, `SBC A,A`
- `AND`, `OR`, `XOR`, `CP` (register and immediate forms)
- `CALL nn`, `RET`, `RET P`
- `DJNZ`, `BIT`, `RES`, `RLA`, `SRA`, `RR`
- `SCF`, `NEG`, `CPL`, `AND A` (as reset carry)
- `LDIR`, `LDDR`
- `LD SP,nn`, `LD SP,HL`, `ADD HL,SP`, `SBC HL,SP`
- `LD IY,nn`, `LD IX,nn`, `ADD IX,DE`
- `LD (nn),HL`, `LD HL,(nn)`, `LD (nn),A`, `LD A,(nn)`
- `LD (HL),n` (immediate to memory)
- `INC (HL)` (memory increment)

**Phase 2 — Assembler extension (user CODE words):**
- All remaining forms from the assembler tables above
- Parameterised by register/condition code masks
