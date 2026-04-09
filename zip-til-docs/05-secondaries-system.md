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

## System Variables

All system variables are in the SYS block and accessed via the *SYS
generic code. Each has a 1-byte offset into the SYS block.

| Variable | Function |
|----------|----------|
| BASE | Current number base (default 10h on start) |
| MODE | 0=execute mode, nonzero=compile mode |
| STATE | Compiler immediate state flag |
| DP | Dictionary pointer (next free address) |
| LBP | Line buffer pointer (current scan position) |
| CURRENT | Address of current vocabulary for new definitions |
| CONTEXT | Address of vocabulary to search |
| COMPILER | Address of compiler vocabulary |

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
