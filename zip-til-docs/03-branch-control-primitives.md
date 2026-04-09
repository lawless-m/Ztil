# ZIP Branch and Control Primitives — Reconstructed from Chapter 6

Source: Loeliger, *Threaded Interpretive Languages*, Byte Books, 1981, Ch.6

All branch primitives are headerless (system-only). They manipulate the
instruction register (BC) to implement relative jumps within threaded code.

## Branch Mechanism

Threaded code stores a 1-byte relative offset after branch word addresses.
Forward branches add a positive offset; backward branches add a negative
(two's complement) offset. Because BC is a 16-bit register pair but the offset
is 8-bit, page-crossing must be handled.

## $ELSE — Forward Relative Branch (unconditional)

Used by *IF for the forward jump when condition is false.

```z80
$ELSE:  LD   A,(BC)            ; get jump byte (offset)
        ADD  A,C               ; add to IR low byte
        LD   C,A               ; reset IR low
        JR   NC,OUT            ; past page boundary?
        INC  B                 ; yes — increment high byte
OUT:    JP   (IY)              ; return to NEXT
```

Bytes: 10

## $WHILE — Backward Relative Branch (unconditional)

Used by *END, *LOOP, *CLOOP for backward jumps.

```z80
$WHILE: LD   A,(BC)            ; get jump byte (negative offset)
        ADD  A,C               ; add to IR low byte
        LD   C,A               ; reset IR low
        JR   C,OUT             ; past page boundary? (carry = no borrow)
        DEC  B                 ; yes — decrement high byte
OUT:    JP   (IY)              ; return to NEXT
```

Bytes: 10

Note: For backward jumps the offset byte is negative (two's complement).
Adding a negative value to C will NOT set carry if we crossed a page
boundary downward, so `JR C,OUT` skips the DEC B when no page cross.

OCR corrections: `IR C,OUT` → `JR C,OUT`; `JP {IV}` → `JP (IY)`.

## *IF — Conditional Forward Branch

If TOS is zero (false), branch forward. Otherwise step past offset byte.

```z80
*IF:    POP  HL                ; get the flag
        LD   A,L               ; are all bits 0
        OR   H                 ; or false?
        JP   Z,$ELSE           ; if 0, jump forward
        INC  BC                ; else bump IR past offset byte
```

Bytes: 11. Jump to $ELSE evokes the relative forward branch.

## *END — Conditional Backward Branch

If TOS is zero (false), branch backward. Otherwise step past offset byte.

```z80
*END:   POP  HL                ; get the flag
        LD   A,L               ; are all bits 0
        OR   H                 ; or false?
        JP   Z,$WHILE          ; if 0, jump backward
        INC  BC                ; else bump IR past offset byte
```

Bytes: 11. Jump to $WHILE evokes the relative backward branch.

## *DO — Word Loop Initialisation

Pushes terminator and start index to return stack for DO..LOOP.

```z80
*DO:    POP  HL                ; get start index
        LD   (IX-4),L          ; move to return stack
        LD   (IX-3),H          ;   as top entry (index)
        POP  HL                ; get terminator
        LD   (IX-2),L          ; move to return stack
        LD   (IX-1),H          ;   as 2nd entry (limit)
        LD   DE,-4             ; reset return
        ADD  IX,DE             ;   stack pointer (4 bytes down)
```

Return stack layout after *DO: IX+0,+1 = index; IX+2,+3 = terminator.

## *CDO — Byte Loop Initialisation

Pushes terminator and start index (byte-length) to return stack.

```z80
*CDO:   POP  HL                ; get start index (byte)
        LD   (IX-2),L          ; to return stack top
        POP  HL                ; get terminator (byte)
        LD   (IX-1),L          ; to return stack second
        DEC  IX                ; reset return
        DEC  IX                ;   stack pointer
```

Return stack layout: IX+0 = index byte; IX+1 = terminator byte.

## *LOOP — Word Loop Test

Increments index, compares to terminator. Branches back if not done.

```z80
*LOOP:  PUSH IX                ; get return stack pointer
        POP  HL                ;   to HL
        LD   A,1               ; get increment (1)
SLOOP:  ADD  A,(HL)            ; increment index low byte
        LD   (HL),A            ; restore low index
        INC  HL                ; bump to index high byte
        JR   NC,PAGE           ; past page?
        INC  (HL)              ; bump page (carry into high byte)
PAGE:   LD   D,(HL)            ; get index high byte
        INC  HL                ; bump to terminator low
        SUB  (HL)              ; index - terminator (low)
        LD   A,D               ; index high to A
        INC  HL                ; bump to terminator high
        SBC  A,(HL)            ; index - terminator - CY (high)
        JP   C,$WHILE          ; if CY=1 (index < term), jump back
        LD   DE,4              ; else drop index and terminator
        ADD  IX,DE             ;   from return stack
        INC  BC                ; increment IR past offset byte
```

Bytes: 30. The SLOOP entrance is shared with *+LOOP.

## *CLOOP — Byte Loop Test

Increments byte index, compares to byte terminator.

```z80
*CLOOP: PUSH IX                ; get return stack pointer
        POP  HL                ;   to HL
        INC  (HL)              ; increment index byte
SCLOOP: LD   A,(HL)            ; get index
        INC  HL                ; point to terminator
        SUB  (HL)              ; index - terminator
        JP   C,$WHILE          ; if CY=1 (index < term), jump back
        INC  IX                ; else drop index
        INC  IX                ;   and terminator
        INC  BC                ; increment IR past offset byte
```

OCR correction: `IP C,$WHILE` → `JP C,$WHILE`.

## *+LOOP — Word Loop with Variable Increment

Gets increment from stack, then uses SLOOP code from *LOOP.

```z80
*+LOOP: PUSH IX                ; get return stack pointer
        POP  HL                ;   to HL
        POP  DE                ; get increment byte from stack
        LD   A,E               ; to the A register
        JP   SLOOP             ; jump to *LOOP code
```

Bytes: 10. Note: *+LOOP has a code address but not a return address.
Increments must be in the range -128 < I < 127.

## *LEAVE — Force Loop Exit

Replaces loop index with terminator to force exit on next test.

```z80
*LEAVE: LD   A,(IX+3)          ; get terminator low byte
        LD   (IX+1),A          ; to index low byte
        LD   A,(IX+2)          ; get terminator high byte
        LD   (IX+0),A          ; to index high byte
```

Bytes: 16.

## Literal Handlers

### *# — Word Literal

Pushes the 16-bit word at IR to the stack, advances IR by 2.

```z80
*#:     LD   A,(BC)            ; get byte at IR
        LD   E,A               ; move to DE low
        INC  BC                ; bump IR
        LD   A,(BC)            ; get byte at IR
        LD   D,A               ; move to DE high
        INC  BC                ; bump IR
        PUSH DE                ; push word to stack
```

Bytes: 11.

OCR correction: `ID A,(BC)` → `LD A,(BC)`.

### *C# — Byte Literal

Pushes the byte at IR (sign-extended to 16 bits) to stack, advances IR by 1.

```z80
*C#:    LD   A,(BC)            ; get byte at IR
        LD   E,A               ; to E
        RLA                    ; sign to carry
        SBC  A,A               ; FF if neg else 00
        LD   D,A               ; sign extend to D
        INC  BC                ; bump IR
        PUSH DE                ; push sign-extended byte
```

### *[ — Inline String Display

Displays a counted string embedded in threaded code.

```z80
*[:     LD   A,(BC)            ; BC=IR, (IR)=length
        LD   D,A               ; save length
SLOOP:  INC  BC                ; bump IR
        LD   A,(BC)            ; get character at IR
        CALL $ECHO             ; echo character to display
        DEC  D                 ; decrement length
        JR   NZ,SLOOP          ; loop until length=0
        INC  BC                ; adjust IR past string
```

OCR correction: `IR NZ,SLOOP` → `JR NZ,SLOOP`.

### *SYS — System Variable Access

Generic code for system variables in the SYS user block.

```z80
*SYS:   LD   A,(DE)            ; DE=WA, (WA)=offset byte
        LD   HL,SYS            ; start of SYS block
        ADD  A,L               ; add offset
        LD   L,A               ; variable address low byte
        PUSH HL                ; address to stack
        JP   (IY)              ; jump to NEXT
```

All system variables (BASE, MODE, STATE, DP, LBP, CURRENT, CONTEXT, etc.)
are defined as entries with *SYS as their code address and a 1-byte offset
as their code body.
