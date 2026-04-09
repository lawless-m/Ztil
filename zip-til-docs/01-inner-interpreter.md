# ZIP Inner Interpreter — Reconstructed from Table 3.3

Source: Loeliger, *Threaded Interpretive Languages*, Byte Books, 1981, p.36

## Register Assignments (Table 3.2)

| Register | Usage |
|----------|-------|
| AF | 8-bit accumulator and program status word |
| BC | Instruction Register (IR) — points into threaded code |
| DE | Word Address register / scratch |
| HL | Scratch / 16-bit accumulator |
| IX | Return Stack Pointer (grows downward) |
| IY | Address of NEXT (for fast JP (IY) return) |
| SP | Data Stack Pointer (grows downward) |
| AF', BC', DE', HL' | Scratch (used by EXX-saving primitives) |

## Memory Map (Figure 3.2)

| Address | Contents |
|---------|----------|
| 0100+ | TIL code (inner interpreter, primitives, dictionary) |
| F800 | Line buffer (128 bytes + 2 terminators) |
| F900 | System variables |
| FB00 | Return stack pointer init (grows down toward F900) |
| FC00 | Data stack pointer init (grows down toward FB00) |

## Dictionary Entry Format (Figure 2.1)

```
Offset  Size  Contents
  +0     1    Name length (1 byte). Bit 7 = immediate flag.
  +1     3    First 3 chars of name (ASCII)
  +4     2    Link address (points to previous header, 0000 = end)
  +6     2    Code field address (CFA) — points to code body
  +8     n    Code body (machine code for primitives, threaded
              addresses for secondaries)
```

For primitives: CFA = WA+2 (points to code immediately after itself).
For secondaries: CFA = address of COLON (0118h).

## Reconstructed Code

All hex opcodes validated against Z80 instruction encoding.
Notation: standard Zilog, (IX+d) for indexed addressing.

```
; =====================================================================
; SEMI — Return from secondary (headerless, no dictionary entry)
; Pops return address from return stack into BC (instruction register)
; =====================================================================
; Address  Bytes
  0100:    02 01          ; DW 0102h — code field points to 0102
  0102:    DD 4E 00       ; LD C,(IX+0)      ; low byte of return addr   [19T]
  0105:    DD 23          ; INC IX           ; pop return stack           [10T]
  0107:    DD 46 00       ; LD B,(IX+0)      ; high byte of return addr  [19T]
  010A:    DD 23          ; INC IX           ; pop return stack           [10T]
                          ; falls through to NEXT

; =====================================================================
; NEXT — Fetch next word address from threaded code stream
; BC (IR) points to next entry in threaded code; fetches 16-bit WA
; =====================================================================
  010C:    0A             ; LD A,(BC)        ; low byte of word address    [7T]
  010D:    6F             ; LD L,A                                         [4T]
  010E:    03             ; INC BC           ; advance IR                  [6T]
  010F:    0A             ; LD A,(BC)        ; high byte of word address   [7T]
  0110:    67             ; LD H,A           ; HL = word address            [4T]
  0111:    03             ; INC BC           ; advance IR                  [6T]
                          ; falls through to RUN                     [=34T]

; =====================================================================
; RUN — Dereference code field at word address, jump to code
; HL = word address; code field at (HL) points to executable code
; =====================================================================
  0112:    5E             ; LD E,(HL)        ; low byte of code address    [7T]
  0113:    23             ; INC HL                                         [6T]
  0114:    56             ; LD D,(HL)        ; high byte of code address   [7T]
  0115:    23             ; INC HL           ; HL now = parameter field    [6T]
  0116:    EB             ; EX DE,HL         ; HL=code addr, DE=param fld  [4T]
  0117:    E9             ; JP (HL)          ; jump to code               [4T]
                          ;                                          [=34T]

; =====================================================================
; COLON — Enter secondary (push current IR, set IR to parameter field)
; Called when RUN jumps to COLON's address as the code body
; DE = parameter field address (from RUN), BC = current IR
; =====================================================================
  0118:    DD 2B          ; DEC IX           ; make room on return stack  [10T]
  011A:    DD 70 00       ; LD (IX+0),B      ; push high byte of IR      [19T]
  011D:    DD 2B          ; DEC IX                                        [10T]
  011F:    DD 71 00       ; LD (IX+0),C      ; push low byte of IR       [19T]
  0122:    4B             ; LD C,E           ; IR = parameter field        [4T]
  0123:    42             ; LD B,D           ;   (from DE set by RUN)     [4T]
  0124:    FD E9          ; JP (IY)          ; jump to NEXT               [8T]
                          ;                                          [=74T]

; =====================================================================
; EXECUTE — Execute word whose address is on data stack
; First dictionary entry (link = 0000, end of chain)
; =====================================================================
  0126:    07             ; DB 7             ; name length = 7 (EXECUTE)
  0127:    45 58 45       ; DB 'E','X','E'   ; first 3 chars
  012A:    00 00          ; DW 0000h         ; link = 0 (end of chain)
  012C:    2E 01          ; DW 012Eh         ; CFA → 012E (code body)
  012E:    E1             ; POP HL           ; get execution token from stack
  012F:    18 E1          ; JR 0112h         ; jump to RUN
                          ; (displacement E1h = -31, 012F+2-31 = 0112)
```

## Timing Summary

| Sequence | T-states |
|----------|----------|
| NEXT + RUN | 34 + 34 = 68 |
| Primitive overhead | NEXT + RUN + body + JP (IY) = 76 + body |
| COLON entry | 74 |
| SEMI exit | 58 |
| Secondary call overhead | NEXT+RUN+COLON + ... + NEXT+RUN+SEMI = 200+ |

## OCR Corrections Applied

| Raw text | Corrected | Error type |
|----------|-----------|------------|
| `01 0A` | `010A` | Spaces in hex address |
| `01 0D`, `01 0E` | `010D`, `010E` | Spaces in hex address |
| `01 OF` | `010F` | Letter O for digit 0 |
| `01 1 1` | `0111` | Extra space in address |
| `01 1 A` | `011A` | Extra space in address |
| `01 ID` | `011D` | Letter I for digit 1 |
| `01 IF` | `011F` | Letter I for digit 1 |
| `LD D# {H L}` | `LD D,(HL)` | # for comma, space in HL |
| `El` | `E1` | Lowercase L for digit 1 |
| `1 8E1` | `18 E1` | Space in wrong position |
| Garbled `LD/00/II/o` | (T-state data: `10`) | Column extraction failure |
| Garbled `00/ll/CD` | (T-state data: `6`) | Column extraction failure |
