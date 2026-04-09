# ZIP Z80 Assembler Extension — Reconstructed from Chapter 7

Source: Loeliger, *Threaded Interpretive Languages*, Byte Books, 1981, Ch.7.2

The Z80 assembler is implemented entirely in TIL — no new Z80 machine code
primitives are needed. It defines ~60 mnemonic keywords in an ASSEMBLER
vocabulary using defining words and secondaries. Total size ~1800 bytes.

The assembler is used in execution mode (not compile mode) to define new
primitives. Example usage:

```
CODE DUP                    ( create primitive header, switch to ASSEMBLER vocab )
  HL PSH,                   ( PUSH HL )
  HL PSH,                   ( PUSH HL )
  NEXT                      ( JP (IY), switch back to CORE vocab )
```

## Notation

In the book, `B` between tokens represents a space separator. So
`:B8*B2*B2*B2*B;` means `: 8* 2* 2* 2* ;`. All definitions below
are shown with spaces for clarity.

## 7.2.3.1 Operands

### Utility

```forth
: 8* 2* 2* 2* ;             ( shift register mask to b5b4b3 position )
```

### 8-Bit Register Constants

Defined as CCONSTANT values (register mask in bits b2b1b0):

```forth
0 CCONSTANT B               ( register mask 000 )
1 CCONSTANT C               ( register mask 001 )
2 CCONSTANT D               ( register mask 010 )
3 CCONSTANT E               ( register mask 011 )
4 CCONSTANT H               ( register mask 100 )
5 CCONSTANT L               ( register mask 101 )
6 CCONSTANT M               ( register mask 110 = (HL) indirect )
7 CCONSTANT A               ( register mask 111 )
```

### 16-Bit Register Pair Constants

```forth
00 CCONSTANT BC              ( register pair mask )
10 CCONSTANT DE
20 CCONSTANT HL
30 CCONSTANT AF
30 CCONSTANT SP              ( same mask as AF in some contexts )
```

### Condition Code Constants

```forth
00 CCONSTANT NZ              ( not zero )
08 CCONSTANT Z               ( zero )
10 CCONSTANT NC              ( no carry )
18 CCONSTANT CY              ( carry — CY not C to avoid register clash )
20 CCONSTANT PO              ( parity odd )
28 CCONSTANT PE              ( parity even )
30 CCONSTANT P               ( positive )
38 CCONSTANT N               ( negative — N not M to avoid register clash )
```

### Indexed Register Operands

@X and @Y enclose the DD/FD prefix byte and leave a negative-valued mask
(8007h) on the stack. The negative value signals indexed mode to instruction
keywords.

```forth
: @X DD C, 8007 ;           ( IX indexed: emit DD, push mask )
: @Y FD C, 8007 ;           ( IY indexed: emit FD, push mask )
```

### Index Register Pair Keywords

```forth
: IX DD C, HL ;              ( emit DD prefix, push HL mask )
: IY FD C, HL ;              ( emit FD prefix, push HL mask )
```

## 7.2.3.2 Constants (Fixed Instructions)

### 1-Byte Instructions

Defining word:
```forth
: 1BYTE <BUILDS C, DOES> C@ C, ;
```

Keywords:
```forth
3F 1BYTE CCF,                ( complement carry flag )
AF 1BYTE CLA,                ( clear accumulator — XOR A )
2F 1BYTE CPL,                ( complement accumulator, 1's )
27 1BYTE DAA,                ( decimal adjust accumulator )
F3 1BYTE DSI,                ( disable interrupts )
FB 1BYTE ENI,                ( enable interrupts )
76 1BYTE HLT,                ( halt )
00 1BYTE NOP,                ( no operation )
A7 1BYTE RCF,                ( reset carry flag — AND A )
37 1BYTE SCF,                ( set carry flag )
C9 1BYTE RET,                ( return from subroutine )
08 1BYTE XAA,                ( exchange AF and AF' )
D9 1BYTE XAL,                ( exchange BC,DE,HL with alternates )
EB 1BYTE XDH,                ( exchange DE and HL )
```

### 2-Byte Instructions (ED prefix)

Defining word:
```forth
: 2BYTES <BUILDS C, DOES> ED C, C@ C, ;
```

Keywords:
```forth
46 2BYTES IM0,               ( interrupt mode 0 )
56 2BYTES IM1,               ( interrupt mode 1 )
5E 2BYTES IM2,               ( interrupt mode 2 )
44 2BYTES NEG,               ( negate A, 2's complement )
4D 2BYTES RTI,               ( return from interrupt )
45 2BYTES RTN,               ( return from non-maskable interrupt )
6F 2BYTES RLD,               ( rotate left digit )
67 2BYTES RRD,               ( rotate right digit )
57 2BYTES LAI,               ( A = I )
5F 2BYTES LAR,               ( A = R )
4F 2BYTES LRA,               ( R = A )
47 2BYTES LIA,               ( I = A )
```

## 7.2.3.3 8-Bit Move Group

### MOV, — Register to register move

Usage: `r r' MOV,` — moves r' to r.
For indexed: `d @X r MOV,` or `r d @X MOV,`

```forth
: MOV, OVER 8* OVER + 40 + C, + 0< IF C, THEN ;
```

The `OVER 8*` shifts destination to b5b4b3. `OVER +` combines both masks.
`40 +` adds the MOV opcode base. `+ 0<` tests for indexed mode (negative
mask means @X/@Y was used, so enclose displacement byte).

### MVI, — Move immediate to register

Usage: `d r n MVI,` (d only for @X/@Y)

```forth
: MVI, OVER 8* 06 + C, SWAP 0< IF SWAP C, THEN C, ;
```

### LDA, — Load accumulator from memory

Usage: `rp LDA,` (for BC/DE) or `addr LDA,` (extended addressing)

```forth
: BCORDE 2DUP BC = SWAP DE = OR ;
: LDA, BCORDE IF 0A + C, ELSE 3A C, , THEN ;
: STA, BCORDE IF 02 + C, ELSE 32 C, , THEN ;
```

## 7.2.3.4 16-Bit Move Group

### DMI, — Double move immediate

Usage: `rp n DMI,` — loads 16-bit value to register pair.

```forth
: DMI, SWAP 01 + C, , ;
```

### DSM, — Double store to memory

Usage: `n rp DSM,` — stores register pair to memory address.

```forth
: DSM, DUP HL = IF 22 C, DROP ELSE ED C, 43 + C, THEN , ;
```

### DLM, — Double load from memory

Usage: `rp n DLM,` — loads register pair from memory address.

```forth
: DLM, SWAP DUP HL = IF 2A C, DROP ELSE ED C, 4B + C, THEN , ;
```

### PSH, and POP, — Push/Pop register pairs

Defining word:
```forth
: 1MASK <BUILDS C, DOES> C@ SWAP 8* + C, ;
```

Keywords:
```forth
C5 1MASK PSH,                ( push register pair )
C1 1MASK POP,                ( pop register pair )
```

## 7.2.3.5 Arithmetic and Logic Group

### 8-Bit ALU Operations (register operand)

Defining word:
```forth
: 8ALG <BUILDS C, DOES> C@ OVER + C, 0< IF C, THEN ;
```

Keywords:
```forth
80 8ALG ADD,                 ( add register to A )
88 8ALG ADC,                 ( add with carry )
90 8ALG SUB,                 ( subtract register from A )
98 8ALG SBC,                 ( subtract with borrow )
A0 8ALG AND,                 ( AND register with A )
A8 8ALG XOR,                 ( XOR register with A )
B0 8ALG IOR,                 ( OR register with A )
B8 8ALG CMP,                 ( compare register with A )
```

### 8-Bit Immediate Operations

Defining word:
```forth
: 8IM <BUILDS C, DOES> C@ C, C, ;
```

Keywords:
```forth
C6 8IM ADI,                  ( add immediate to A )
CE 8IM ACI,                  ( add immediate with carry )
D6 8IM SUI,                  ( subtract immediate )
DE 8IM SCI,                  ( subtract immediate with borrow )
E6 8IM ANI,                  ( AND immediate )
EE 8IM XOI,                  ( XOR immediate )
F6 8IM ORI,                  ( OR immediate )
FE 8IM CPI,                  ( compare immediate )
```

### 8-Bit Increment/Decrement

```forth
: INC, DUP 8* 04 + C, IF C, THEN ;
: DEC, DUP 8* 05 + C, IF C, THEN ;
```

### 16-Bit Arithmetic

```forth
: DAD, 09 + C, ;                          ( add rp to HL )
: DAI, SWAP OVER = IF -1 DP +! THEN DAD, ; ( add rp to IX/IY )
: DAC, ED C, 4A + C, ;                    ( add with carry to HL )
: DSC, ED C, 42 + C, ;                    ( subtract with carry from HL )
: DSB, ED A7 , 42 + C, ;                  ( subtract without carry — macro )
```

### 16-Bit Increment/Decrement

```forth
03 1MASK DIN,                ( increment register pair )
0B 1MASK DDC,                ( decrement register pair )
```

## 7.2.3.6 Rotate and Shift Group

Defining word:
```forth
: RSG <BUILDS C, DOES> CB C, C@ DUP 20 - 0<
    IF OVER 07 = IF -1 DP +! THEN THEN OVER 0<
    IF LROT C, THEN + C, ;
```

The complex generic code handles: (1) dropping the CB prefix for the four
1-byte rotate instructions (codes < 20h with register A), and (2) inserting
the displacement byte for indexed modes.

Keywords:
```forth
00 RSG RLC,                  ( rotate left circular )
08 RSG RRC,                  ( rotate right circular )
10 RSG RLT,                  ( rotate left through carry )
18 RSG RRT,                  ( rotate right through carry )
20 RSG SLR,                  ( shift left register )
28 RSG SRR,                  ( shift right arithmetic )
38 RSG SRL,                  ( shift right logical )
```

## 7.2.3.7 Bit Addressing

Defining word:
```forth
: BITAD <BUILDS C, DOES> CB C, C@ LROT 8*
    + OVER + SWAP 0< IF SWAP C, THEN C, ;
```

Usage: `b r BIT,` or `d b @X BIT,`

Keywords:
```forth
40 BITAD BIT,                ( test bit )
80 BITAD RES,                ( reset bit )
C0 BITAD SET,                ( set bit )
```

## 7.2.3.8 Block-Directed Instructions

Condition operands:
```forth
00 CCONSTANT IC              ( increment )
01 CCONSTANT DC              ( decrement )
10 CCONSTANT IR              ( increment and repeat )
11 CCONSTANT DR              ( decrement and repeat )
```

Defining word:
```forth
: BDIR <BUILDS C, DOES> ED C, C@ + C, ;
```

Usage: `condition CLD,` — e.g. `IR CLD,` assembles LDIR.

Keywords:
```forth
A0 BDIR CLD,                ( block load: LDI, LDD, LDIR, LDDR )
A1 BDIR CCP,                ( block compare: CPI, CPD, CPIR, CPDR )
A2 BDIR CIN,                ( block input: INI, IND, INIR, INDR )
A3 BDIR COT,                ( block output: OUTI, OUTD, OTIR, OTDR )
```

## 7.2.3.9 Miscellaneous Instructions

```forth
: RST, 8* C7 + C, ;         ( restart — usage: n RST, )
DB 8IM INA,                  ( input from port — usage: port INA, )
D3 8IM OTA,                  ( output to port — usage: port OTA, )
: XST, E3 C, DROP ;         ( exchange (SP) with rp )
: JPM, E9 C, DROP ;         ( jump to address in rp )
: LSP, F9 C, DROP ;         ( load SP from rp )
```

## 7.2.3.10 Call and Return Group

```forth
: LABEL ' 2+ HERE SWAP ! ;  ( save current addr to a CONSTANT )
: CAL, CD C, , ;             ( unconditional call — usage: addr CAL, )
```

Conditional call/return defining word:
```forth
: CCODE <BUILDS C, DOES> C@ + C, , ;
```

Keywords:
```forth
C4 CCODE CLC,               ( conditional call — usage: addr cc CLC, )
C0 CCODE RTC,               ( conditional return — usage: cc RTC, )
```

## 7.2.3.11 Jump Instructions

```forth
: JMP, C3 C, , ;             ( unconditional absolute jump )
C2 CCODE JPC,                ( conditional absolute jump )
18 8IM JPR,                   ( unconditional relative jump )
: JRC, 10 + C, C, ;          ( conditional relative jump — Z,NZ,CY,NC only )
10 8IM DJN,                   ( decrement B, jump if not zero )
```

## 7.2.4 Structured Assembly Constructs

### Unconditional Condition

```forth
-1 CCONSTANT U               ( unconditional "condition" for END )
```

### Helper Keywords

```forth
: JRC DUP 0< IF DROP JPR, ELSE JRC, THEN ;
: JAC DUP 0< IF DROP JMP, ELSE JPC, THEN ;
```

JRC/JAC select unconditional vs conditional based on the condition code sign.

### BEGIN ... END Loop

```forth
: BEGIN HERE ;
: END DUP 20 - 0< IF
    OVER HERE 2 + - DUP 80 - 0< IF
      2SWAP DROP JRC
    ELSE
      DROP JAC
    THEN
  ELSE JAC THEN ;
```

END automatically selects relative vs absolute jump based on distance
and condition code. PE, PO, P, N conditions always use absolute jumps
(no relative form exists).

### IF ... ELSE ... THEN

Relative forms:
```forth
: RIF 0 SWAP JRC, HERE ;
: RTHEN OVER - SWAP 1- C! ;
: RRELSE HERE 2+ RTHEN 18 , HERE ;
```

Absolute forms:
```forth
: AIF H0 SWAP JPC, HERE 2+ ;
: ATHEN HERE SWAP ! ;
: RAELSE HERE 3 + RTHEN 0 JMP, HERE 2- ;
: ARELSE ATHEN 18 , HERE ;
: AAELSE ATHEN 0 JMP, HERE 2- ;
```

Note: True/False code bodies are reversed from TIL IF/THEN — the Z80
jumps when the condition IS met, so the "fall through" case is when
the condition fails.

### WHILE

```forth
: RWHILE SWAP U END HERE RTHEN ;
: AWHILE SWAP U END HERE ATHEN ;
```

### DO ... LOOP (uses DJNZ)

```forth
: DO HERE B LROT MVI, ;     ( save addr, load B register with count )
: LOOP HERE 2 + - DJN, ;    ( compute offset, assemble DJNZ )
```

Usage: `count DO ... LOOP` — repeats body count times (1-256, 0=256).

## 7.2.5 Assembler Vocabulary Management

### CODE — Begin primitive definition

```forth
: CODE CREATE HEX ASSEMBLER ;
```

Creates a primitive header, sets hex base, switches to ASSEMBLER vocabulary.

### NEXT — End primitive definition (ASSEMBLER version)

```forth
: NEXT IY JPM, DEFINITIONS ;
```

Encloses `JP (IY)` instruction, switches back to CORE vocabulary.
This is a different keyword from the CORE vocabulary NEXT.

## Mnemonic Cross-Reference

Loeliger uses personal mnemonics. Here's a mapping to standard Zilog:

| ZIP Mnemonic | Zilog Equivalent | Notes |
|-------------|-----------------|-------|
| MOV, | LD r,r' | 8-bit register to register |
| MVI, | LD r,n | 8-bit immediate |
| LDA, | LD A,(rp) or LD A,(nn) | Load accumulator |
| STA, | LD (rp),A or LD (nn),A | Store accumulator |
| DMI, | LD rp,nn | 16-bit immediate |
| DSM, | LD (nn),rp | 16-bit store to memory |
| DLM, | LD rp,(nn) | 16-bit load from memory |
| PSH, | PUSH rp | Push register pair |
| POP, | POP rp | Pop register pair |
| ADD, | ADD A,r | 8-bit add |
| ADC, | ADC A,r | 8-bit add with carry |
| SUB, | SUB r | 8-bit subtract |
| SBC, | SBC A,r | 8-bit subtract with borrow |
| AND, | AND r | Bitwise AND |
| XOR, | XOR r | Bitwise XOR |
| IOR, | OR r | Bitwise OR |
| CMP, | CP r | Compare |
| ADI, | ADD A,n | Immediate add |
| ACI, | ADC A,n | Immediate add with carry |
| SUI, | SUB n | Immediate subtract |
| SCI, | SBC A,n | Immediate subtract with borrow |
| ANI, | AND n | Immediate AND |
| XOI, | XOR n | Immediate XOR |
| ORI, | OR n | Immediate OR |
| CPI, | CP n | Immediate compare |
| INC, | INC r | 8-bit increment |
| DEC, | DEC r | 8-bit decrement |
| DAD, | ADD HL,rp | 16-bit add |
| DAI, | ADD IX/IY,rp | Indexed 16-bit add |
| DAC, | ADC HL,rp | 16-bit add with carry |
| DSC, | SBC HL,rp | 16-bit subtract with carry |
| DSB, | AND A / SBC HL,rp | Macro: subtract without carry |
| DIN, | INC rp | 16-bit increment |
| DDC, | DEC rp | 16-bit decrement |
| RLC, | RLC r | Rotate left circular |
| RRC, | RRC r | Rotate right circular |
| RLT, | RL r | Rotate left through carry |
| RRT, | RR r | Rotate right through carry |
| SLR, | SLA r | Shift left |
| SRR, | SRA r | Shift right arithmetic |
| SRL, | SRL r | Shift right logical |
| BIT, | BIT b,r | Test bit |
| RES, | RES b,r | Reset bit |
| SET, | SET b,r | Set bit |
| CLD, | LDI/LDD/LDIR/LDDR | Block load |
| CCP, | CPI/CPD/CPIR/CPDR | Block compare |
| CIN, | INI/IND/INIR/INDR | Block input |
| COT, | OUTI/OUTD/OTIR/OTDR | Block output |
| RST, | RST n | Restart |
| INA, | IN A,(n) | Input from port |
| OTA, | OUT (n),A | Output to port |
| XST, | EX (SP),rp | Exchange with stack top |
| JPM, | JP (rp) | Jump to address in register |
| LSP, | LD SP,rp | Load stack pointer |
| CAL, | CALL nn | Unconditional call |
| CLC, | CALL cc,nn | Conditional call |
| RTC, | RET cc | Conditional return |
| JMP, | JP nn | Unconditional absolute jump |
| JPC, | JP cc,nn | Conditional absolute jump |
| JPR, | JR e | Unconditional relative jump |
| JRC, | JR cc,e | Conditional relative jump |
| DJN, | DJNZ e | Decrement B and jump |
| CCF, | CCF | Complement carry |
| CLA, | XOR A | Clear accumulator |
| CPL, | CPL | Complement accumulator |
| DAA, | DAA | Decimal adjust |
| DSI, | DI | Disable interrupts |
| ENI, | EI | Enable interrupts |
| HLT, | HALT | Halt CPU |
| NOP, | NOP | No operation |
| RCF, | AND A | Reset carry flag |
| SCF, | SCF | Set carry flag |
| RET, | RET | Return from subroutine |
| XAA, | EX AF,AF' | Exchange AF pair |
| XAL, | EXX | Exchange all alternate regs |
| XDH, | EX DE,HL | Exchange DE and HL |
| NEG, | NEG | Negate A (2's complement) |
| RTI, | RETI | Return from interrupt |
| RTN, | RETN | Return from NMI |
| RLD, | RLD | Rotate left digit |
| RRD, | RRD | Rotate right digit |

## Notes

- The assembler requires ~1800 bytes of dictionary space.
- All definitions live in the ASSEMBLER vocabulary, linked to CORE.
- Register names A-E shadow hex digits in the ASSEMBLER vocabulary;
  use leading zeros (0A, 0B, etc.) for hex numbers when assembler is active.
- The @X/@Y indexed mode uses a negative mask (8007h) as a sentinel;
  instruction keywords test for negative to decide whether to emit a
  displacement byte.
- No error checking beyond stack underflow. Invalid operand combinations
  produce garbage silently.
- Structured constructs (IF/THEN/ELSE, BEGIN/END, DO/LOOP) manage
  addresses on the stack, eliminating the need for a symbol table.
