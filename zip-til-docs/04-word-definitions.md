# ZIP Word Definitions — Primitives with Z80 Code

Source: Loeliger, *Threaded Interpretive Languages*, Byte Books, 1981, Ch.6
All primitives end with `JP (IY)` to return to NEXT unless noted otherwise.

## Stack Operations

### DROP — Discard TOS
```z80
        POP  HL                ; drop top
```
Bytes: 11.

### DUP — Duplicate TOS
```z80
        POP  HL                ; get top word
        PUSH HL                ; restore top
        PUSH HL                ; and push it again
```
Bytes: 13.

### 2DUP — Duplicate TOS twice
```z80
        POP  HL                ; get top
        PUSH HL                ; restore
        PUSH HL                ; dup 1
        PUSH HL                ; dup 2
```

### SWAP — Exchange top two entries
```z80
        POP  HL                ; get top
        EX   (SP),HL           ; top to 2nd, 2nd to HL
        PUSH HL                ; 2nd to top
```
Bytes: 13.

### OVER — Copy second entry to top
```z80
        POP  HL                ; get top
        POP  DE                ; get 2nd
        PUSH DE                ; restore 2nd as 3rd
        PUSH HL                ; restore top as 2nd
        PUSH DE                ; push 2nd to top
```

### LROT — Left rotate top 3 (A B C → B C A)
```z80
        POP  DE                ; get top (C)
        POP  HL                ; get 2nd (B)
        EX   (SP),HL           ; exchange 3rd(A) and 2nd(B)
        PUSH DE                ; push old top (C)
        PUSH HL                ; push old 3rd (A)
```
Bytes: 15.

### RROT — Right rotate top 3 (A B C → C A B)
```z80
        POP  HL                ; get top (C)
        POP  DE                ; get 2nd (B)
        EX   (SP),HL           ; top(C) to 3rd, 3rd(A) to HL
        PUSH HL                ; 3rd(A) to 2nd
        PUSH DE                ; 2nd(B) to top
```
Bytes: 15.

### 2OVER — Copy 3rd entry to top
```z80
        EXX                    ; save IR
        POP  HL                ; get top
        POP  DE                ; get 2nd
        POP  BC                ; get 3rd
        PUSH BC                ; restore 3rd
        PUSH DE                ; restore 2nd
        PUSH HL                ; restore top
        PUSH BC                ; push 3rd to top
        EXX                    ; restore IR
```

### 2SWAP — Exchange top and 3rd entries
```z80
        POP  HL                ; get top
        POP  DE                ; get 2nd
        EX   (SP),HL           ; top to 3rd, 3rd to HL
        PUSH DE                ; restore 2nd
        PUSH HL                ; 3rd to top
```

## Memory Operations

### ! — Store word at address
```z80
        POP  HL                ; get address
        POP  DE                ; get data
        LD   (HL),E            ; store low byte
        INC  HL                ; bump address
        LD   (HL),D            ; store high byte
```
Bytes: 15.

### @ — Fetch word from address
```z80
        POP  HL                ; get address
        LD   E,(HL)            ; low byte at address
        INC  HL                ; bump address
        LD   D,(HL)            ; high byte at address
        PUSH DE                ; push contents
```
Bytes: 15.

### C! — Store byte at address
```z80
        POP  HL                ; get address
        POP  DE                ; get byte
        LD   (HL),E            ; store low byte only
```
Bytes: 13.

### C@ — Fetch byte from address (sign-extended)
```z80
        POP  HL                ; get address
        LD   E,(HL)            ; get byte at address
        LD   A,E               ; get the byte
        RLA                    ; sign to carry
        SBC  A,A               ; FF if neg else 00
        LD   D,A               ; set sign extension
        PUSH DE                ; push 16-bit word
```
Bytes: 17.

### +! — Add to word in memory
```z80
        POP  HL                ; get address
        POP  DE                ; get increment/decrement
        LD   A,(HL)            ; get low byte
        ADD  A,E               ; add increment low
        LD   (HL),A            ; store back
        INC  HL                ; step to high byte
        LD   A,(HL)            ; get high byte
        ADC  A,D               ; add increment high + carry
        LD   (HL),A            ; store back
```

### C+! — Add to byte in memory
```z80
        POP  HL                ; get address
        POP  DE                ; get byte
        LD   A,(HL)            ; get byte at address
        ADD  A,E               ; add byte
        LD   (HL),A            ; store at address
```

### , (comma) — Enclose word in dictionary
```z80
        POP  DE                ; get word
        LD   HL,(DP)           ; get dictionary pointer
        LD   (HL),E            ; store low byte
        INC  HL
        LD   (HL),D            ; store high byte
        INC  HL
        LD   (DP),HL           ; update DP
```

### C, — Enclose byte in dictionary
```z80
        POP  DE                ; get byte
        LD   HL,(DP)           ; get dictionary pointer
        LD   (HL),E            ; store byte
        INC  HL
        LD   (DP),HL           ; update DP
```

### 0! — Store zero at address
```z80
        POP  HL                ; get address
        LD   (HL),0            ; zero low byte
        INC  HL
        LD   (HL),0            ; zero high byte
```

### 1! — Store one at address
```z80
        POP  HL                ; get address
        LD   (HL),1            ; one to low byte
        INC  HL
        LD   (HL),0            ; zero high byte
```

## Arithmetic

### + — Add
```z80
        POP  HL                ; get 1st word
        POP  DE                ; get 2nd word
        ADD  HL,DE             ; add them
        PUSH HL                ; push sum
```

### - — Subtract (2nd - TOS)
```z80
        POP  DE                ; get TOS (B)
        POP  HL                ; get 2nd (A)
        AND  A                 ; reset carry
        SBC  HL,DE             ; form A-B
        PUSH HL                ; push result
```
Bytes: 16.

### MINUS — Negate (two's complement)
```z80
        LD   HL,0              ; get zero
        POP  DE                ; get number
        AND  A                 ; reset carry
        SBC  HL,DE             ; 0 - number
        PUSH HL                ; push two's complement
```
Bytes: 18.

### ABS — Absolute value
```z80
        POP  DE                ; get number
        BIT  7,D               ; if positive, Z=1
        JR   Z,OUT             ; if Z=1, it's OK
        LD   HL,0              ; else get zero
        AND  A                 ; reset carry
        SBC  HL,DE             ; zero - number
        EX   DE,HL             ; positive result to DE
OUT:    PUSH DE                ; push positive number
```
Bytes: 23.

### 2* — Double
```z80
        POP  HL                ; get word
        ADD  HL,HL             ; double it
        PUSH HL                ; restore
```

### 2+ — Add two
```z80
        POP  HL                ; get word
        INC  HL                ; +1
        INC  HL                ; +2
        PUSH HL
```

### 2- — Subtract two
```z80
        POP  HL
        DEC  HL                ; -1
        DEC  HL                ; -2
        PUSH HL
```

### 2/ — Arithmetic shift right (divide by 2)
```z80
        POP  HL                ; get word
        SRA  H                 ; arithmetic shift right high byte
        RR   L                 ; propagate carry to low byte
        PUSH HL                ; push word/2
```

### * — Signed multiply (16×8 → 16)
```z80
        EXX                    ; save IR
        POP  BC                ; get first (8 bit)
        POP  DE                ; get second (16 bit)
        CALL $ISIGN            ; field input signs
        CALL $UD*              ; multiply 16×8
        CALL $OSIGN            ; justify result
        PUSH HL                ; result to stack
        EXX                    ; restore IR
```
Bytes: 24.

### D* — Signed 16×8 → 24-bit multiply
Produces the full 24-bit signed product of the 16-bit second stack
entry and the low-order byte of TOS. Replaces both entries with the
16 least-significant bits (TOS) and 8 most-significant bits (2nd),
sign-extended.
```z80
        EXX                    ; save IR
        POP  BC                ; get 8-bit number
        POP  DE                ; get 16-bit number
        CALL $ISIGN            ; field input signs
        CALL $UD*              ; multiply 16×8
        EX   AF,AF'            ; retrieve sign flag
        JP   P,OUTS            ; if +, it's OK
        LD   A,C               ; move 8 most-significant
        CPL                    ; complement
        LD   C,A               ; restore
        EX   DE,HL             ; move 16 least
        LD   HL,0              ; get zero
        SBC  HL,DE             ; negate 16 least
        JR   NZ,OUTS           ; if not zero, OK
        INC  C                 ; else 2's comp most
OUTS:   PUSH HL                ; 16 least to stack
        PUSH BC                ; 8 most to stack
        EXX                    ; restore IR
```
Bytes: 39.

### */ — Multiply then divide (truncated)
Computes `(third * 2nd_byte) / top_byte` returning only the 16-bit
quotient. Uses `*/MOD` as a subroutine via an IY trick.
```z80
        LD   IY,RETTO          ; change NEXT return vector
        JP   $*/MOD            ; do the */MOD code
RETTO:  POP  HL                ; drop the remainder
        LD   IY,NEXT           ; restore normal NEXT return
```
Bytes: 22. Illustrates re-using a primitive as a subroutine: the
`JP (IY)` inside `$*/MOD` jumps to RETTO instead of NEXT, which
then restores IY before returning.

### */MOD — Multiply then divide with remainder
Computes `(third * 2nd_byte) / top_byte` leaving 16-bit quotient (2nd)
and 8-bit remainder (TOS). Uses the full 24-bit intermediate product.
```z80
$*/MOD: POP  HL                ; divisor to L
        EXX                    ; save IR and divisor
        POP  BC                ; multiplicand (8)
        POP  DE                ; multiplier (16)
        CALL $ISIGN            ; field * sign
        CALL $UD*              ; do 16×8 multiply
        EXX                    ; get divisor and IR
        EX   AF,AF'            ; get / sign flag
        XOR  L                 ; XOR in divisor sign
        EX   AF,AF'            ; save result sign
        LD   A,L               ; get divisor
        EXX                    ; save IR again
        AND  A                 ; test divisor sign
        JP   P,SKIPN           ; if +, it's OK
        NEG                    ; make divisor positive
SKIPN:  LD   D,C               ; move high 8 bits of 24
        LD   E,A               ; move divisor
        CALL $UD/              ; do 24×8 divide
        CALL $OSIGN            ; justify result
        PUSH HL                ; quotient to stack
        PUSH BC                ; remainder to stack
        EXX                    ; restore IR
```
Bytes: 43. The `$*/MOD` entrance is re-used by `*/`.

### / — Signed divide (16÷8 → 8 quotient)
```z80
        EXX                    ; save IR
        POP  DE                ; get divisor (8 bits)
        POP  BC                ; get dividend (16 bits)
        CALL $ISIGN            ; field input signs
        CALL $US/              ; divide 16×8
        CALL $OSIGN            ; justify result
        PUSH HL                ; quotient to stack
        EXX                    ; restore IR
```

### /MOD — Signed divide with remainder
```z80
        EXX                    ; save IR
        POP  DE                ; get divisor (8 bits)
        POP  BC                ; get dividend (16 bits)
        CALL $ISIGN            ; field input signs
        CALL $US/              ; divide 16×8
        CALL $OSIGN            ; justify result
        PUSH HL                ; quotient to stack
        PUSH BC                ; remainder to stack
        EXX                    ; restore IR
```
Bytes: 25.

### D/MOD — Signed 24÷8 divide with remainder
Divides a signed 24-bit dividend (3rd entry = 16 LSBs, 2nd entry = 8 MSBs)
by the low-order byte of TOS. Replaces with 16-bit quotient (2nd) and
positive 8-bit remainder expanded to 16 bits (TOS).
```z80
        EXX                    ; save IR
        POP  HL                ; get 8-bit divisor
        POP  DE                ; get 8 most-sig of dividend
        POP  BC                ; get 16 least-sig of dividend
        LD   A,H               ; divisor sign
        XOR  D                 ; result sign
        EX   AF,AF'            ; save sign flag
        LD   A,L               ; get divisor magnitude
        AND  A                 ; test sign
        JP   P,MOV1            ; if +, it's OK
        NEG                    ; make positive
MOV1:   LD   D,A               ; store divisor
        LD   H,B               ; get 16 least
        LD   L,C               ;   to HL
        LD   A,E               ; get 8 most
        AND  A                 ; test sign
        JP   P,MOV2            ; if +, it's OK
        CPL                    ; complement high 8
        LD   HL,0              ; zero
        SBC  HL,BC             ; negate low 16
        JR   NZ,MOV2           ; if non-zero, OK
        INC  A                 ; else bump high
MOV2:   LD   D,A               ; move high 8
        CALL $UD/              ; divide 24×8
        CALL $OSIGN            ; justify result
        PUSH HL                ; quotient to stack
        PUSH BC                ; remainder to stack
        EXX                    ; restore IR
```
Bytes: 48. Does not validate that TOS is a valid 8-bit number, nor that
the quotient fits in 16 bits.

### MIN — Signed minimum
Replaces the top two stack entries with the smaller (signed) value.
```z80
        POP  DE                ; get top
        POP  HL                ; get 2nd
        PUSH HL                ; assume 2nd smaller
        AND  A                 ; reset carry
        SBC  HL,DE             ; 2nd - top
        JP   M,OUT             ; 2nd smaller, exit
        POP  HL                ; drop 2nd
        PUSH DE                ; push top
OUT:    JP   (IY)              ; return to NEXT
```
Bytes: 21.

### MAX — Signed maximum
Replaces the top two stack entries with the larger (signed) value.
```z80
        POP  DE                ; get top
        POP  HL                ; get 2nd
        PUSH HL                ; assume 2nd greater
        AND  A                 ; reset carry
        SBC  HL,DE             ; 2nd - top
        JP   P,OUT             ; 2nd greater, exit
        POP  HL                ; drop 2nd
        PUSH DE                ; push top
OUT:    JP   (IY)              ; return to NEXT
```
Bytes: 21.

### MOD — Signed remainder
Replaces the top two stack entries with the 8-bit remainder of
`2nd / top_byte`, sign-extended to 16 bits.
```z80
        EXX                    ; save IR
        POP  DE                ; get 8-bit divisor
        POP  BC                ; get 16-bit dividend
        CALL $ISIGN            ; field input signs
        CALL $US/              ; divide 16×8
        PUSH BC                ; push remainder
        EXX                    ; restore IR
```
Bytes: 21. No test for a valid 8-bit divisor.

### S* — Signed 8×8 multiply
```z80
        EXX                    ; save IR
        POP  BC                ; get first 8 bits
        POP  DE                ; get 2nd 8 bits
        CALL $ISIGN            ; field input signs
        CALL $US*              ; multiply 8×8
        CALL $OSIGN            ; justify result
        PUSH HL                ; product to stack
        EXX                    ; restore IR
```
Bytes: 24.

## Relational

### < — Less than (signed)
```z80
        POP  DE                ; get top
        POP  HL                ; get 2nd
        AND  A                 ; reset carry
        SBC  HL,DE             ; 2nd - top
        LD   DE,0              ; set flag false
        JP   P,PUSHIT          ; if positive (2nd >= top), false
        INC  E                 ; set flag true
PUSHIT: PUSH DE                ; flag to stack
```
Bytes: 23. PUSHIT label is shared by =, >, <.

### = — Equal
```z80
        POP  HL                ; get top
        POP  DE                ; get 2nd
        AND  A                 ; reset carry
        SBC  HL,DE             ; top - 2nd
        LD   DE,0              ; set flag false
        JR   NZ,PUSHIT         ; if not equal, push false
        INC  E                 ; set flag true
PUSHIT: PUSH DE                ; flag to stack
```
Bytes: 22.

### > — Greater than (signed)
```z80
        POP  HL                ; get top
        POP  DE                ; get 2nd
        AND  A                 ; reset carry
        SBC  HL,DE             ; top - 2nd
        LD   DE,0              ; set flag false
        JP   P,PUSHIT          ; if positive (top >= 2nd), false
        INC  E                 ; set flag true
PUSHIT: PUSH DE                ; flag to stack
```
Bytes: 23.

### 0= — Zero equality
Replaces the top with TRUE if it was zero, FALSE otherwise.
```z80
        POP  HL                ; get word
        LD   A,L               ; low byte
        OR   H                 ; OR in high byte
        LD   DE,0              ; flag false
        JR   NZ,OUT            ; nonzero → push false
        INC  DE                ; else flag true
OUT:    PUSH DE                ; push flag
```
Bytes: 20.

## Logical

### AND — Bitwise AND
```z80
        POP  HL                ; get top
        POP  DE                ; get 2nd
        LD   A,L               ; AND low bytes
        AND  E
        LD   L,A
        LD   A,H               ; AND high bytes
        AND  D
        LD   H,A
        PUSH HL                ; result to stack
```
Bytes: 19.

### IOR — Bitwise inclusive OR
```z80
        POP  HL
        POP  DE
        LD   A,L
        OR   E
        LD   L,A
        LD   A,H
        OR   D
        LD   H,A
        PUSH HL
```

### XOR — Bitwise exclusive OR
```z80
        POP  HL
        POP  DE
        LD   A,L
        XOR  E
        LD   L,A
        LD   A,H
        XOR  D
        LD   H,A
        PUSH HL
```
Bytes: 19.

### NOT — Invert flag
```z80
        POP  HL                ; get the flag
        LD   A,L               ; move low byte
        OR   H                 ; OR in high byte
        LD   DE,0              ; assume false result
        JR   NZ,OUT            ; if nonzero, false
        INC  E                 ; make true
OUT:    PUSH DE                ; flag to stack
```
Bytes: 20.

## Interstack

### <R — Push word to return stack
```z80
        POP  HL                ; get word
        DEC  IX                ; push to return stack
        LD   (IX+0),H          ; high byte
        DEC  IX
        LD   (IX+0),L          ; low byte
```

### R> — Pop word from return stack
```z80
        LD   L,(IX+0)          ; get return low byte
        INC  IX                ; adjust RSP
        LD   H,(IX+0)          ; get return high byte
        INC  IX                ; adjust RSP
        PUSH HL                ; push to data stack
```
Bytes: 21.

### I> — Get word loop index (innermost)
```z80
        LD   L,(IX+0)          ; get low index
        LD   H,(IX+1)          ; get high index
        PUSH HL                ; index to stack
```

### J> — Get word loop index (second level)
```z80
        LD   L,(IX+4)          ; get low index
        LD   H,(IX+5)          ; get high index
        PUSH HL                ; index to stack
```

### CI> — Get byte loop index (innermost)
```z80
        LD   L,(IX+0)          ; get index byte
        LD   H,0               ; zero extend
        PUSH HL
```

### CJ> — Get byte loop index (second level)
```z80
        LD   L,(IX+2)          ; get index byte
        LD   H,0
        PUSH HL
```

### C<R — Push byte to return stack
```z80
        POP  HL                ; get word
        DEC  IX
        LD   (IX+0),L          ; push low byte only
```

### CR> — Pop byte from return stack
```z80
        LD   L,(IX+0)          ; get byte
        LD   H,0               ; zero extend
        INC  IX
        PUSH HL
```

## I/O

### TYPE — Display counted string
```z80
        POP  HL                ; get string address
        LD   E,(HL)            ; get length
LOOP:   INC  HL                ; bump pointer
        LD   A,(HL)            ; get character
        CALL $ECHO             ; display it
        DEC  E                 ; decrement length
        JR   NZ,LOOP           ; loop until done
```
Bytes: 20.

### ECHO — Display low byte of TOS
```z80
        POP  HL                ; get top
        LD   A,L               ; get low-order byte
        CALL $ECHO             ; display it
```
Bytes: 15.

### KEY — Input character from keyboard
```z80
        CALL $KEY              ; get character in A
        LD   L,A               ; to L
        PUSH HL                ; push to stack
```
Bytes: 15. Note: H is undefined (caller should use C@ or mask).

### SPACE — Display a space
```z80
        LD   A,20h             ; get ASCII space
        CALL $ECHO             ; display it
```
Bytes: 15.

### DISPLAY — Output stack string (from #>)
```z80
$DISPLAY:
        EXX                    ; save IR
DLOOP:  POP  HL                ; get top stack word
        LD   A,L               ; low byte
        CALL $ECHO             ; display it
        AND  A                 ; test for bit 7 (terminator)
        JP   P,DLOOP           ; if positive, continue loop
        EXX                    ; restore IR
```
Bytes: 21.

### SIGN — Add minus sign to number string
```z80
        BIT  7,(IX+0)          ; get return stack sign bit
        JR   Z,OUT             ; if zero (positive), exit
        LD   L,2Dh             ; ASCII minus sign code
        PUSH HL                ; push to stack
OUT:    JP   (IY)              ; return to NEXT
```
Bytes: 19.

### <# — Begin number conversion
Saves number sign on the return stack and leaves a string terminator
(ASCII space with bit 7 set, A0h) under the original number on the
data stack. Must be paired with `#>` or `CR>` within a definition.
```z80
        POP  HL                ; get the number
        LD   E,A0h             ; space with bit 7 = 1
        PUSH DE                ; push string stop
        PUSH HL                ; restore number
        DEC  IX                ; decrement RSP
        LD   (IX+0),H          ; sign byte to return stack
```
Bytes: 18. The sign is the high byte H — its top bit indicates sign.
Note that E is loaded with A0h but D is untouched, so the pushed word's
high byte is whatever D was on entry. (Quirk of the book's listing —
for the terminator to work correctly, the caller must treat only the
low byte as meaningful, which DISPLAY does.)

### #> — End number conversion
Discards the saved sign byte on the return stack and jumps into
`$DISPLAY` to emit the number string that was built on the data stack.
```z80
        INC  IX                ; drop return stack sign byte
        JP   $DISPLAY          ; go display string (no return)
```
Bytes: 13. Has no return address — enters DISPLAY's code body, which
does its own `JP (IY)` to NEXT.

### ASCII — Convert binary digit to ASCII character
Converts the low byte of TOS (a value 0..35) to its ASCII character
representation ('0'..'9', 'A'..'Z').
```z80
        POP  HL                ; get binary
        LD   A,30h             ; ASCII '0'
        ADD  A,L               ; add binary
        CP   3Ah               ; letter?
        JR   C,OUT             ; CY=1 → it's a digit
        ADD  A,7               ; add letter bias (':'+7='A')
OUT:    LD   L,A               ; back to L
        PUSH HL                ; code to stack
```
Bytes: 22.

## System

### +SP — Add SP to TOS
```z80
        POP  HL                ; get number
        ADD  HL,SP             ; add stack pointer
        PUSH HL                ; push result
```
Bytes: 13.

### -SP — Subtract SP from TOS
```z80
        POP  HL                ; get number
        AND  A                 ; reset carry
        SBC  HL,SP             ; subtract stack pointer
        PUSH HL                ; push result
```
Bytes: 15. OCR note: text has `SBC SP` — should be `SBC HL,SP`.

### ?RS — Push return stack pointer
```z80
        PUSH IX                ; push return stack pointer
```
Bytes: 12.

### ?SP — Push data stack pointer (with underflow check)
```z80
        LD   HL,0              ; get stack
        ADD  HL,SP             ; pointer to HL
        EX   DE,HL             ; save in DE
        LD   HL,STACK          ; get end of stack
        AND  A                 ; reset carry
        SBC  HL,DE             ; end - SP
        JR   NC,SKIP           ; NC = stack OK
        LD   SP,STACK          ; else reset stack
SKIP:   PUSH DE                ; push prior SP
```
Bytes: 27.

### SINGLE — Test for valid byte-length number
```z80
        POP  HL                ; get word
        PUSH HL                ; restore word
        LD   L,H               ; if single, H=00 or FF
        LD   A,H               ; get high byte
        AND  A                 ; test it
        JR   Z,OUT             ; if zero, push false (HL=0000)
        INC  HL                ; FFFF+1=0000 only if H was FF
OUT:    PUSH HL                ; push flag
```
Bytes: 18.

### ABORT — Jump to START/RESTART
Unconditionally restarts the system via the ABORT entry point in
START/RESTART (keeps current BASE, resets stacks and flags).
```z80
        JP   START             ; to START/RESTART
```
Bytes: 11. No return address. Referenced by `$PATCH` and used as the
"total panic" recovery.

### EXECUTE — Execute word from stack
```z80
        POP  HL                ; get keyword word address
        JP   RUN               ; execute it
```
Bytes: 12. Note: no return address (no JP (IY)); jumps directly to RUN.

### HEX — Set base to 16
```z80
        LD   A,10h             ; get 16 decimal
        LD   (BASE),A          ; store at BASE
```
Bytes: 15.

### DECIMAL — Set base to 10
```z80
        LD   A,0Ah             ; get 10 decimal
        LD   (BASE),A
```
Bytes: 15.

### OCTAL — Set base to 8
```z80
        LD   A,8               ; get 8 decimal
        LD   (BASE),A
```
Bytes: 15.

### BINARY — Set base to 2
```z80
        LD   A,2               ; get 2 decimal
        LD   (BASE),A
```
Bytes: 15.

## Utility

### MOVE — Block move (handles overlap)
```z80
        EXX                    ; save IR
        POP  DE                ; new starting address
        POP  HL                ; old ending address
        POP  BC                ; old starting address
        AND  A                 ; reset carry
        SBC  HL,BC             ; count-1
        PUSH BC                ; save old starting addr
        EX   (SP),HL           ; save count-1, get old start
        POP  BC                ; BC = count-1
        EX   DE,HL             ; HL = new starting addr
        PUSH HL                ; save new start
        AND  A                 ; reset carry
        SBC  HL,DE             ; move from top?
        POP  HL                ; get new start back
        JR   NC,BOTTOM         ; no, move from bottom
        EX   DE,HL             ; HL = old start
        INC  BC                ; BC = count
        LDIR                   ; move block forward
OUTM:   EXX                    ; restore IR
        JP   (IY)              ; return to NEXT
BOTTOM: ADD  HL,BC             ; new ending address
        EX   DE,HL             ; old starting address
        ADD  HL,BC             ; old ending address
        INC  BC                ; BC = count
        LDDR                   ; move block backward
        JR   OUTM              ; return
```
Bytes: 40. Correctly handles overlapping blocks.

OCR correction: `JP {IV}` → `JP (IY)`.

## Arithmetic Subroutines

### $CRLF — Carriage return / line feed
```z80
$CRLF:  LD   A,0Dh             ; get CR
        CALL $ECHO             ; issue it
        LD   A,0Ah             ; get LF
        CALL $ECHO             ; issue it
        RET
```

### $ISIGN — Field input signs
```z80
$ISIGN: LD   A,D               ; sign of 1st number
        XOR  B                 ; XOR sign of 2nd
        EX   AF,AF'            ; result sign to AF'
        LD   A,D               ; sign of 1st
        AND  A                 ; test sign, CY=0
        JP   P,TST2            ; if positive, OK
        LD   HL,0
        SBC  HL,DE             ; make 1st positive
        EX   DE,HL
TST2:   LD   H,B               ; move 2nd high byte
        LD   L,C               ; move 2nd low byte
        LD   A,B               ; sign of 2nd
        AND  A                 ; test sign, CY=0
        RET  P                 ; if positive, return
        LD   HL,0
        SBC  HL,BC             ; make 2nd positive
        RET
```

### $OSIGN — Justify result sign
```z80
$OSIGN: EX   AF,AF'            ; retrieve sign flags
        RET  P                 ; if positive, sign OK
        EX   DE,HL             ; result to DE
        LD   HL,0
        SBC  HL,DE             ; negate result
        RET
```
Result in HL on entrance and exit.

### $UD* — Unsigned 16×8 multiply → 24-bit result
```z80
$UD*:   LD   A,L               ; multiplicand to A
        LD   BC,0800h          ; B=8 (count), C=0 (dummy)
        LD   H,C               ; zero high result
        LD   L,C               ; zero low result
D*LOOP: ADD  HL,HL             ; shift result left 1
        ADC  A,A               ; shift multiplicand left 1
        JR   NC,SKADD          ; if CY=0, skip add
        ADD  HL,DE             ; add multiplier
        ADC  A,C               ; propagate carry
SKADD:  DJNZ D*LOOP            ; loop 8 times
        LD   C,A               ; high 8 bits in C
        RET                    ; low 16 in HL
```
Bytes: 16. Entrance: L=8-bit multiplicand, DE=16-bit multiplier.
Exit: C:HL = 24-bit product.

### $US* — Unsigned 8×8 multiply → 16-bit result
```z80
$US*:   LD   H,L               ; multiplicand to H
        LD   L,0               ; zero result low
        LD   D,L               ; multiplier high = 0
        LD   B,8               ; multiply count
S*LOOP: ADD  HL,HL             ; shift result and multiplicand
        JR   NC,SKPAD          ; if CY=0, skip add
        ADD  HL,DE             ; add multiplier
SKPAD:  DJNZ S*LOOP            ; loop 8 times
        RET                    ; result in HL
```
Bytes: 13. Entrance: L and E = 8-bit unsigned integers.

### $UD/ — Unsigned 24÷8 divide → 16-bit quotient, 8-bit remainder
```z80
$UD/:   LD   B,10h             ; divide count = 16
D/LOOP: ADD  HL,HL             ; shift low 16
        LD   A,D               ; get high 8
        ADC  A,D               ; shift high 8
        LD   D,A               ; restore high
        SUB  E                 ; subtract divisor
        JP   M,SKIP            ; too much, skip
        INC  L                 ; set result low bit = 1
        LD   D,A               ; decrease dividend
SKIP:   DJNZ D/LOOP            ; loop 16 times
        LD   C,D               ; remainder to C
        RET                    ; quotient in HL
```
Bytes: 16. Entrance: D:HL = 24-bit dividend, E = 8-bit divisor.

OCR correction: `IP M,SKIP` → `JP M,SKIP`.

### $US/ — Unsigned 16÷8 divide → 8-bit quotient, 8-bit remainder
```z80
$US/:   LD   B,8               ; divide count = 8
S/LOOP: ADD  HL,HL             ; shift dividend
        LD   A,H               ; get high byte
        SUB  E                 ; subtract divisor
        JP   M,SKP             ; too much, skip
        INC  L                 ; set result low bit
        LD   H,A               ; decrease dividend
SKP:    DJNZ S/LOOP            ; loop 8 times
        LD   C,H               ; remainder in C
        LD   H,B               ; result high = 0
        RET                    ; result in HL (quotient in L)
```
Bytes: 15. Entrance: HL = 16-bit dividend, E = 8-bit divisor.

## Defining Words (Primitive portions)

### CONSTANT — Create word constant
```z80
        ; Secondary preamble: CREATE, , (comma), SCODE
        ; Generic code (executed when constant is referenced):
        EX   DE,HL             ; word address to HL
        LD   E,(HL)            ; get low byte from code body
        INC  HL
        LD   D,(HL)            ; get high byte
        PUSH DE                ; number to stack
        JP   (IY)              ; return to NEXT
```
Bytes: 21.

### CCONSTANT — Create byte constant
```z80
        ; Secondary preamble: CREATE, C,, SCODE
        ; Generic code:
        LD   A,(DE)            ; get byte from code body (DE=WA)
        LD   L,A               ; to L
        RLA                    ; sign to carry
        SBC  A,A               ; FF if neg else 00
        LD   H,A               ; sign extend
        PUSH HL                ; push 16-bit word
        JP   (IY)              ; return to NEXT
```
Bytes: 22. OCR correction: `JP {IV}` → `JP (IY)`.

### VARIABLE — Create word variable
```z80
        ; Secondary preamble: CONSTANT (creates header + stores value),
        ;   then SCODE replaces code address
        ; Generic code:
        PUSH DE                ; push word address (DE from RUN)
        JP   (IY)              ; return to NEXT
```
Bytes: 15.

### CVARIABLE — Create byte variable
```z80
        ; Secondary preamble: CCONSTANT (creates header + stores byte),
        ;   then SCODE replaces code address
        ; Generic code:
        PUSH DE                ; push word address
        JP   (IY)              ; return to NEXT
```
Bytes: 15. Formal: `: CVARIABLE CCONSTANT ;CODE ....` — same shape
as VARIABLE except the initialiser uses CCONSTANT (one byte) instead
of CONSTANT (one word). Used by BASE, MODE, STATE in the SYS block.
