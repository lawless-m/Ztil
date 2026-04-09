# ZIP Secondaries and System Layout

Source: Loeliger, *Threaded Interpretive Languages*, Byte Books, 1981, Ch.6

## Secondary Definitions

Secondaries are threaded code lists. Each begins with a code field pointing
to COLON (0118h) and ends with the word address of SEMI.

### . (period) — Display signed number
```
Code:   <#              ; initialise conversion
        ABS             ; take absolute value
        #S              ; convert all digits
        SIGN            ; add minus sign if needed
        #>              ; display result
```
Formal: `: . <# ABS #S SIGN #> ;`

### # — Convert one digit
```
Code:   0               ; 24-bit number extension
        BASE            ; number base address
        C@              ; get number base
        D/MOD           ; remainder then quotient
        ASCII           ; remainder to ASCII character
        SWAP            ; remainder to number string
```
Bytes: 22. Formal: `: # 0 BASE C@ D/MOD ASCII SWAP ;`

### #S — Convert all digits
```
Code:   #               ; convert 1 character
        DUP             ; duplicate quotient
        0=              ; is it zero?
        *END xx         ; loop back if not zero
        DROP            ; drop zero quotient
```
Formal: `: #S BEGIN # DUP 0= END DROP ;`

### ? — Display word at address
```
Code:   @               ; get the number
        .               ; display it
```
Formal: `: ? @ . ;`

### C? — Display byte at address
```
Code:   C@              ; get the byte
        .               ; display it
```

### CA! — Store code address of latest entry
```
Code:   ENTRY           ; address of latest header
        *C# 6           ; literal 6
        +               ; header + 6 = word address location
        !               ; store code address
```
Formal: `: CA! ENTRY 6 + ! ;`

### ENTRY — Get latest header address
```
Code:   CURRENT         ; CURRENT vocabulary address
        @               ; vocabulary link address
        @               ; header address
```
Formal: `: ENTRY CURRENT @ @ ;`

### HERE — Get next free dictionary address
```
Code:   DP              ; dictionary pointer address
        @               ; get its contents
```
Formal: `: HERE DP @ ;`

### DO, — Enclose program control directive word address
Used by immediate compiler directives that need to both enclose a
directive word address and save the reserved byte location for later
patching.
```
Code:   ,               ; store directive word address
        HERE            ; push the new free address
```
Bytes: 14. Formal: `: DO, , HERE ;`
Used by: IF, ELSE, DO.

### END, — Enclose directive + backward offset
Used by immediate compiler directives that need to enclose a directive
word address plus a relative jump byte computed from a previously saved
address at TOS.
```
Code:   ,               ; store directive word address
        HERE            ; current free address
        -               ; compute relative offset
        C,              ; enclose low byte
```
Bytes: 18. Formal: `: END, , HERE - C, ;`
Used by: END, WHILE, LOOP, C+LOOP, +LOOP, CLOOP.

### CREATE — Create dictionary header
```
Code:   ENTRY           ; pointer to latest header
        ASPACE          ; set separator (space)
        TOKEN           ; token to dictionary space
        HERE            ; points to the token
        CURRENT         ; address of CURRENT vocabulary
        @               ; vocabulary link
        !               ; update link to new token
        *C# 4           ; four identifier characters
        DP              ; dictionary pointer
        +!              ; enclose four chars (advance DP by 4)
        ,               ; add link address to new header
        HERE            ; word address of new header
        2+              ; points to code body
        ,               ; store at word address
```
Bytes: 39.

### ERASE — Fill memory with spaces
```
Code:   1+              ; bump last address for looping
        SWAP            ; get loop order correct
        *DO             ; initialise loop
        ASPACE          ; get space code
        I>              ; index = memory address
        C!              ; space to memory
        *LOOP F8        ; loop until done
```
Formal: `: ERASE 1+ SWAP DO ASPACE I> C! LOOP ;`

### FILL — Fill memory with specified byte
```
Code:   1+              ; bump last address
        SWAP            ; get right loop order
        *DO             ; initialise loop
        DUP             ; duplicate byte
        I>              ; get memory address
        C!              ; store byte
        *LOOP F8        ; loop until done
        DROP            ; remove byte from stack
```
Formal: `: FILL 1+ SWAP DO DUP I> C! LOOP DROP ;`

### <BUILDS — Initiate high-level defining word
```
Code:   0               ; initial value
        CONSTANT        ; creates a constant keyword
```
Formal: `: <BUILDS 0 CONSTANT ;`

### SCODE — Replace code address (used by defining words)
```
Code:   R>              ; get return address
        CA!             ; store it as code address
```
Formal: `: SCODE R> CA! ;`

### DOES> — Terminate compile-time code of defining word
```z80
        ; Mixed secondary/primitive:
Code:   R>              ; get top return address
        ENTRY           ; latest header address
        *C# 8           ; plus 8
        +               ; points to code body
        !               ; store return address to code body
        SCODE           ; replace code address and return
        ; Then assembly code (generic execution code):
        DEC  IX
        LD   (IX+0),B   ; push IR to return stack
        DEC  IX
        LD   (IX+0),C
        EX   DE,HL      ; WA register to HL
        LD   C,(HL)     ; load new IR from word address
        INC  HL
        LD   B,(HL)
        INC  HL
        PUSH HL         ; push parameter field pointer
```
Bytes: 39. Formal: `: DOES> ENTRY 8 + ! ;CODE ....`

## Compiler Directives (Immediate Secondaries)

These words execute during compilation to build control structures.

### IF — Begin conditional
```
Code:   *# XX           ; word address of *IF (literal)
        DO,             ; store address and push here
        0               ; reserve byte for offset
        C,              ; enclose it
```
Formal: `: IF XX DO, 0 C, ; IMMEDIATE`

### THEN — Resolve forward reference from IF
```
Code:   HERE            ; current dictionary pointer
        OVER            ; copy IF's saved address
        -               ; compute offset
        SWAP            ; get IF's address
        C!              ; patch the offset byte
```
Formal: `: THEN HERE OVER - SWAP C! ; IMMEDIATE`

### ELSE — Alternative branch in IF..ELSE..THEN
```
Code:   *# XX           ; word address of *ELSE (literal)
        DO,             ; store and push
        0               ; reserve offset byte
        C,              ; enclose it
        SWAP            ; get IF's saved address
        THEN            ; resolve the IF's forward ref
```
Formal: `: ELSE XX DO, 0 C, SWAP THEN ; IMMEDIATE`

### BEGIN — Start of loop
```
Code:   HERE            ; push current address
```
Formal: `: BEGIN HERE ; IMMEDIATE`

### END — Conditional loop termination
```
Code:   *# XX           ; word address of *END (literal)
        END,            ; compute and store backward offset
```
Formal: `: END XX END, ; IMMEDIATE`

### WHILE — Unconditional loop back
```
Code:   *# XX           ; word address of *WHILE (literal)
        END,            ; compute and store backward offset
```
Formal: `: WHILE XX END, ; IMMEDIATE`

### DO — Begin counted loop
```
Code:   *# XX           ; word address of *DO (literal)
        DO,             ; store address and push here
```
Formal: `: DO XX DO, ; IMMEDIATE`

### LOOP — End counted loop
```
Code:   *# XX           ; word address of *LOOP (literal)
        END,            ; compute and store backward offset
```
Formal: `: LOOP XX END, ; IMMEDIATE`

### LEAVE — Force loop exit (compiled)
```
Code:   *# XX           ; word address of *LEAVE (literal)
        ,               ; enclose it
```
Formal: `: LEAVE XX , ; IMMEDIATE`

### : (colon) — Begin new secondary definition
```
Code:   CURRENT         ; current vocabulary address
        @               ; get vocabulary link
        CONTEXT         ; context variable address
        !               ; set context = current
        ...             ; (additional setup)
        MODE            ; mode variable address
        C1SET           ; set compile mode
```
Formal: `: : CURRENT @ CONTEXT ! ... MODE C1SET ;`

### ; (semicolon) — End secondary definition
The semicolon is an immediate word that:
1. Encloses the word address of SEMI in the dictionary
2. Resets MODE to execute mode (C0SET)

## Dictionary Header Format (Figure 2.1)

Each headered keyword consists of a 6-byte header followed by a body.
The body starts with the 2-byte **code address** — and the address of
that code-address slot is what the book calls the keyword's **word
address (WA)**. All inter-keyword references use word addresses.

```
Offset  Size  Field
------  ----  ---------------------------------------
  +0     1    Length byte: bit 7 = immediate flag;
                             bits 0-6 = character count
  +1     3    First 3 characters of the name, ASCII,
              space-padded if fewer than 3 chars
  +4     2    Link: address of the previous entry's
              first header byte (0000h terminates chain)
  +6     2    Code address = start of the code body
              (this address IS the word address)
  +8   ...    Code body: machine code (primitive) or
              list of word addresses (secondary)
```

SEARCH only compares the length byte plus up to 3 name characters, so
`DROP` and `DROX` collide, but `DROP` and `DROPIT` do not (different
lengths).

A new definition's link field is set to the previous CURRENT-vocabulary
head, and CURRENT's head pointer is updated to point at the new header.
See CREATE (defined above) for the full sequence.

Vocabularies are themselves just VARIABLE-typed keywords whose value
is the header address of the most-recently-defined word in that
vocabulary. The system variable CURRENT holds the address of the
vocabulary receiving new definitions; CONTEXT holds the address of
the vocabulary being searched.

## Link Chain Initialisation

At cold start the root vocabulary must already contain every headered
primitive and secondary, linked in definition order. The link field of
the very first keyword (the one loaded lowest in memory) is 0000h.
Each subsequent header's link field points back to the previous
header's byte 0.

For the assembly, this is done by emitting each keyword with a
`LINK` label equ'd to the previous keyword's header start, and the
vocabulary variable initialised to the last keyword's header start.

## System Variables — SYS Block Layout

All system variables are in the SYS block and accessed via the `*SYS`
generic code. Each variable is a dictionary header whose code address
points to `*SYS` and whose 1-byte code body is the offset into the SYS
block. The book deliberately leaves the layout implementation-defined:

> "All system variables defined in the system block contain *SYS as
> their code address followed by a 1-byte offset as their code body.
> The offset points to the system variable, relative to the start of
> the block. A full 256-byte block is not reserved for system variables
> (only 20 thru 30 bytes are used)."
> — Loeliger, Ch.6, *SYS

The layout below is the one this implementation commits to. MODE and
STATE occupy adjacent bytes so `LD (MODE),HL` with HL=0 zeroes both
(as required by START/RESTART, Listing 5.1). BASE is a CVARIABLE (byte)
but is placed first so that the start-vs-restart test (`LD A,(BASE)`
then test for zero) works on a clean memory image.

| Offset | Size | Variable | Type      | Function |
|--------|------|----------|-----------|----------|
| 00h    | 1    | BASE     | CVARIABLE | Current number base (10h after cold start) |
| 01h    | 1    | MODE     | CVARIABLE | 0=execute, 1=compile |
| 02h    | 1    | STATE    | CVARIABLE | Immediate state flag |
| 03h    | 1    | —        | pad       | Keeps next word on even offset |
| 04h    | 2    | DP       | VARIABLE  | Dictionary pointer (next free address) |
| 06h    | 2    | LBP      | VARIABLE  | Line buffer pointer (scan position) |
| 08h    | 2    | CURRENT  | VARIABLE  | Vocabulary to which new defs are linked |
| 0Ah    | 2    | CONTEXT  | VARIABLE  | Vocabulary to be searched |
| 0Ch    | 2    | COMPILER | VARIABLE  | Compiler vocabulary |
| 0Eh    | 2    | LBEND    | (const)   | End-of-line-buffer address (terminators go here) |

Total: 16 bytes used. `*SYS` masks its offset to the low byte of the
SYS block address (`LD L,A` after `ADD A,L`), so the full block must
sit on a 256-byte page. The concrete base address is `SYS = F880h` —
immediately after the 128-byte line buffer (see Memory Map below).

## Memory Map (Figure 3.2)

Reproduced from the book's Figure 3.2. Addresses grow upward in the
table; each stack grows downward in memory.

| Address range | Purpose | Size |
|---------------|---------|------|
| FA00h – FC00h | Data stack (initial SP = FC00h) | 512 bytes |
| F900h – FA00h | Return stack (initial IX = FA00h) | 256 bytes |
| F880h – F900h | System variables (SYS block) | 128 bytes |
| F800h – F880h | Line buffer + terminators | 128 bytes |

Symbolic constants referenced by Listings 5.1 and 5.2:

| Name   | Value | Origin |
|--------|-------|--------|
| STACK  | FC00h | Data stack initial SP (Figure 3.2) |
| RETURN | FA00h | Return stack initial IX (Figure 3.2) |
| SYS    | F880h | Start of system-variable block (chosen) |
| LBADD  | F800h | Line buffer start (chosen; must be page-aligned — INLINE uses `LD L,0` to reset) |
| LENGTH | 80h   | Line buffer length = 128 (INLINE's `BIT 7,L` exit condition) |
| LBEND  | F880h | Line buffer end — address of first terminator (INLINE stores 8080h here) |

## ASCII Control Codes Used by INLINE

The book names these symbolically as `"LD"`, `"BS"`, `"CR"`:

| Symbol | Value | Meaning |
|--------|-------|---------|
| CR     | 0Dh   | Carriage return — end of line (ASCII standard) |
| BS     | 08h   | Backspace — one char left (ASCII standard) |
| LD     | 18h   | Line delete — restart input. Not an ASCII standard; the book leaves this system-specific. We use CAN (0x18), which is the ASCII cancel control. |

$CRLF additionally uses LF (0Ah).

## System Constants

| Constant | Value | Notes |
|----------|-------|-------|
| ASPACE | 20h | ASCII space |
| 0 | 0000h | Zero |
| 1 | 0001h | One |

## System Messages

Messages are in TYPE format: a length byte followed by that many ASCII characters.
May contain embedded control codes.

| Message | Content | Usage |
|---------|---------|-------|
| SRTMSG | "HELLO I'M A TIL" | Cold start (preceded by clear-screen code) |
| RSTMSG | "TIL RESTART" | Warm restart (preceded by CR-LF code) |
| OK | "OK" | Displayed after successful line execution |
| MSG? | "?" | Displayed after unknown token |
| STKMSG | "STACK" | Stack underflow error |

## I/O Subroutines (System-Specific — Must Be Provided)

### $KEY
Resets keyboard, waits for next input character.
Returns character in A register. Must preserve instruction register (BC).

### $ECHO
Outputs character in A register to display device.
Must preserve instruction register (BC).

For the VM implementation, these map directly to console stdin/stdout.
