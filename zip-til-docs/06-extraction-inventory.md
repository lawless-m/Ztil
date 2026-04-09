# ZIP TIL Code Extraction Inventory

## Status Summary

### Chapter 3 — Inner Interpreter
| Routine | Status | Type | Has Hex | Notes |
|---------|--------|------|---------|-------|
| SEMI | DONE | Headerless primitive | Yes | Table 3.3, fully validated |
| NEXT | DONE | Headerless primitive | Yes | Table 3.3, fully validated |
| RUN | DONE | Headerless primitive | Yes | Table 3.3, fully validated |
| COLON | DONE | Headerless primitive | Yes | Table 3.3, fully validated |
| EXECUTE | DONE | Primitive (first in dict) | Yes | Table 3.3, fully validated |

### Chapter 5 — Outer Interpreter Routines
| Routine | Status | Type | Has Hex | Notes |
|---------|--------|------|---------|-------|
| START/RESTART | DONE | Machine code | No | Listing 5.1, symbolic only |
| INLINE | DONE | Primitive | No | Listing 5.2, symbolic |
| TOKEN | DONE | Primitive | No | Listing 5.3, symbolic |
| SEARCH | DONE | Primitive | No | Listing 5.4, symbolic |
| NUMBER | DONE | Primitive | No | Listing 5.5, most complex routine |
| QUESTION | DONE | Primitive | No | Listing 5.6 |
| *STACK | DONE | Primitive | No | Listing 5.7 |
| $PATCH | DONE | Machine code | No | Listing 5.8 |

### Chapter 6 — Word Definitions (Primitives with Z80 Code)
| Keyword | Status | Class | Notes |
|---------|--------|-------|-------|
| ! (store) | DONE | Memory | POP HL; POP DE; LD (HL),E; INC HL; LD (HL),D |
| # | DONE | I/O | Secondary |
| #> | DONE | I/O | Partial primitive |
| $CRLF | DONE | Subroutine | LD A,0D; CALL $ECHO; LD A,0A; CALL $ECHO; RET |
| $ECHO | NOTED | Subroutine | System-specific, must be provided |
| $ISIGN | DONE | Subroutine | Sign fielding for arithmetic |
| $OSIGN | DONE | Subroutine | Result sign correction |
| $KEY | NOTED | Subroutine | System-specific, must be provided |
| $UD* | DONE | Subroutine | 16x8 multiply, 24-bit result |
| $US* | DONE | Subroutine | 8x8 multiply, 16-bit result |
| $UD/ | DONE | Subroutine | 24/8 divide |
| $US/ | DONE | Subroutine | 16/8 divide |
| * (multiply) | DONE | Arithmetic | Signed 16x8 multiply |
| */ | DONE | Arithmetic | Sneaky IY trick |
| */MOD | DONE | Arithmetic | Most complex arithmetic |
| *# (word literal) | DONE | Literal handler | Headerless |
| *C# (byte literal) | DONE | Literal handler | Headerless |
| *CDO | DONE | Program control | Byte loop init |
| *CLOOP | DONE | Program control | Byte loop test |
| *C+LOOP | NOTED | Program control | Uses SLOOP entrance |
| *CLEAVE | NOTED | Program control | |
| *DO | DONE | Program control | Word loop init |
| *ELSE | DONE | Program control | Forward branch |
| *END | DONE | Program control | Conditional backward branch |
| *IF | DONE | Program control | Conditional forward branch |
| *LEAVE | DONE | Program control | Force loop exit |
| *LOOP | DONE | Program control | Word loop test |
| *SYS | DONE | System | System variable access |
| *WHILE | DONE | Program control | Unconditional backward branch |
| *[ (string literal) | DONE | Literal handler | Inline string display |
| + (add) | DONE | Arithmetic | POP HL; POP DE; ADD HL,DE; PUSH HL |
| +! | DONE | Memory | Increment word in memory |
| +SP | DONE | System | Add SP to TOS |
| , (comma) | DONE | System | Enclose word in dictionary |
| - (subtract) | DONE | Arithmetic | POP DE; POP HL; SBC HL,DE; PUSH HL |
| -SP | DONE | System | Subtract SP from TOS |
| . (period) | DONE | I/O | Secondary: <# ABS #S SIGN #> |
| / | DONE | Arithmetic | Signed 16/8 divide |
| /MOD | DONE | Arithmetic | Signed divide with remainder |
| 0! | DONE | Memory | Store zero |
| 1! | DONE | Memory | Store one |
| 2* | DONE | Arithmetic | ADD HL,HL |
| 2+ | DONE | Arithmetic | INC HL; INC HL |
| 2- | DONE | Arithmetic | DEC HL; DEC HL |
| 2/ | DONE | Arithmetic | SRA H; RR L |
| 2DUP | DONE | Stack | |
| 2OVER | DONE | Stack | Uses EXX |
| 2SWAP | DONE | Stack | EX (SP),HL trick |
| < | DONE | Relational | Signed compare |
| <# | DONE | I/O | Number conversion init |
| <R | DONE | Interstack | Push to return stack |
| = | DONE | Relational | Equality test |
| > | DONE | Relational | Signed compare |
| ? | DONE | I/O | Secondary: @ . |
| ?RS | DONE | System | Push return stack pointer |
| ?SP | DONE | System | Push data stack pointer |
| @ (fetch) | DONE | Memory | POP HL; LD E,(HL); INC HL; LD D,(HL); PUSH DE |
| ABS | DONE | Arithmetic | |
| AND | DONE | Logical | Bitwise AND |
| C! | DONE | Memory | Store byte |
| C+! | DONE | Memory | Increment byte in memory |
| C, | DONE | System | Enclose byte in dictionary |
| C@ | DONE | Memory | Fetch byte, sign-extended |
| CCONSTANT | DONE | Defining | CREATE + C, + SCODE + generic code |
| CONSTANT | DONE | Defining | CREATE + , + SCODE + generic code |
| CREATE | DONE | Defining | Secondary |
| D* | DONE | Arithmetic | 24-bit product |
| D/MOD | DONE | Arithmetic | 24/8 divide |
| DISPLAY | DONE | I/O | Loop displaying stack chars |
| DROP | DONE | Stack | POP HL |
| DUP | DONE | Stack | POP HL; PUSH HL; PUSH HL |
| ECHO | DONE | I/O | POP HL; LD A,L; CALL $ECHO |
| EXECUTE | DONE | System | POP HL; JP RUN |
| IOR | DONE | Logical | Bitwise OR |
| KEY | DONE | I/O | CALL $KEY; LD L,A; PUSH HL |
| LROT | DONE | Stack | Left rotate top 3 |
| MAX | NOTED | Arithmetic | Signed max |
| MIN | DONE | Arithmetic | Signed min |
| MINUS | DONE | Arithmetic | Negate (twos complement) |
| MOD | NOTED | Arithmetic | Remainder only |
| MOVE | DONE | Utility | Block move with overlap handling |
| NOT | DONE | Logical | Invert flag |
| OVER | DONE | Stack | POP HL; POP DE; PUSH DE; PUSH HL |
| R> | DONE | Interstack | Pop from return stack |
| RROT | DONE | Stack | Right rotate top 3 |
| S* | DONE | Arithmetic | Signed 8x8 multiply |
| SCODE | DONE | Program control | Secondary: R> CA! |
| SIGN | DONE | I/O | Add minus sign to number string |
| SINGLE | DONE | System | Test for valid byte |
| SPACE | DONE | I/O | LD A,20; CALL $ECHO |
| SWAP | DONE | Stack | POP HL; EX (SP),HL; PUSH HL |
| TYPE | DONE | I/O | Display counted string |
| VARIABLE | DONE | Defining | CONSTANT + SCODE + generic code |
| XOR | DONE | Logical | Bitwise XOR |
| WAIT | NOTED | System | System-specific delay |

### Chapter 6 — Secondaries (threaded code only)
| Keyword | Status | Notes |
|---------|--------|-------|
| #S | NOTED | BEGIN # DUP 0= END DROP |
| .R | NOTED | Right-justified number display |
| : (colon) | NOTED | CURRENT @ CONTEXT ! ... MODE C1SET |
| ;CODE | NOTED | Compiler directive |
| <BUILDS | DONE | 0 CONSTANT |
| ABORT | NOTED | Jump to START/RESTART |
| ADUMP | NOTED | ASCII memory dump |
| ASPACE | DONE | Constant (hex 20) |
| BASE | NOTED | System variable via *SYS |
| BEGIN | DONE | HERE ; IMMEDIATE |
| BINARY | DONE | LD A,2; LD (BASE),A |
| C? | DONE | C@ . |
| CA! | DONE | ENTRY 6 + ! |
| CDO | DONE | *# XX DO, ; IMMEDIATE |
| CLOOP | DONE | *# XX END, ; IMMEDIATE |
| COMPILER | NOTED | System variable via *SYS |
| CONTEXT | NOTED | System variable via *SYS |
| CORE | NOTED | Vocabulary |
| CVARIABLE | NOTED | Defining word |
| DECIMAL | DONE | LD A,10; LD (BASE),A |
| DEFINITIONS | NOTED | |
| DO | DONE | *# XX DO, ; IMMEDIATE |
| DO, | NOTED | Store address and push |
| DOES> | DONE | Complex defining word support |
| DUMP | NOTED | Hex memory dump |
| ELSE | NOTED | Compiler directive |
| END | NOTED | XX END, ; IMMEDIATE |
| END, | NOTED | Compiler directive |
| ENTRY | DONE | CURRENT @ @ |
| ERASE | DONE | Fill with spaces |
| FILL | DONE | Fill with specified byte |
| FORGET | NOTED | Remove dictionary entries |
| HERE | NOTED | DP @ |
| HEX | DONE | LD A,16; LD (BASE),A |
| IF | NOTED | Compiler directive |
| IMMEDIATE | NOTED | Set immediate bit |
| LEAVE | NOTED | XX , ; IMMEDIATE |
| LOOP | NOTED | XX END, ; IMMEDIATE |
| NEXT (keyword) | DONE | Enclose JP (IY) |
| OCTAL | DONE | LD A,8; LD (BASE),A |
| THEN | NOTED | Compiler directive |
| VOCABULARY | NOTED | |
| WHILE | NOTED | Compiler directive |

## OCR Error Catalogue
| Pattern | Actual | Type | Frequency |
|---------|--------|------|-----------|
| O (letter) | 0 (zero) | In hex addresses/values | Common |
| I (letter) | 1 (digit) | In hex addresses | Common |
| l (lowercase L) | 1 (digit) | In hex values | Common |
| Spaces in hex | No spaces | Address/opcode splitting | Very common |
| # | , (comma) | In LD instructions | Occasional |
| Space in register pairs | No space | e.g. {H L} → {HL} | Occasional |
| JP {IV} | JP (IY) | V↔Y confusion | 5 occurrences |
| IR (start of line) | JR | I↔J confusion | 3 occurrences |
| IP (start of line) | JP | I↔J confusion | 2 occurrences |
| ID (start of line) | LD | I↔L confusion | Occasional |
| SBC SP | SBC HL,SP | Missing register | Occasional |
| Column data mixing | T-states garbage | Multi-column extraction failure | In Table 3.3 |

## System-Specific Routines (Must Be Provided for VM)
- $KEY — keyboard input, return char in A
- $ECHO — display output, char in A
- Memory map and port I/O model
