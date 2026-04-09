# ZIP Outer Interpreter Routines — Reconstructed from Chapter 5

Source: Loeliger, *Threaded Interpretive Languages*, Byte Books, 1981, Ch.5

## Outer Interpreter Threaded Code (Figure 5.2)

The outer interpreter is a secondary — a threaded code list. START/RESTART
initialises BC to point to OUTER (the TYPE entry in this list) and jumps to NEXT.

```
OUTER:  DW  TYPE        ; display message (start/restart/error)
        DW  INLINE      ; get a line of input
        DW  ASPACE      ; push space separator
        DW  TOKEN        ; extract next token from line buffer
        DW  ?SEARCH      ; search vocabularies    → (WA,False) or (True)
        DW  *IF, 0B      ; if True (not found), skip to ?NUMBER
        DW  ?NUMBER      ; try to convert as number → (N,True) or (False)
        DW  *ENDF, 03    ; if False, skip to QUESTION
        DW  QUESTION     ; handle terminator or unknown token
        DW  *WHILE, EA   ; loop back to TYPE (unconditional)
        DW  ?EXECUTE     ; execute or compile the found word
        DW  *WHILE, E9   ; loop back to TYPE (unconditional)
```

Note: ?SEARCH, ?NUMBER, ?EXECUTE are headerless secondaries that wrap
SEARCH, NUMBER, EXECUTE with compile-mode logic.

## Listing 5.1 — START/RESTART

```z80
START:
        LD   DE,RSTMSG        ; restart message address to DE
        LD   A,(BASE)          ; get system base
        AND  A                 ; test for zero
        JR   NZ,ABORT          ; if nonzero, it's a restart
        LD   A,10h             ; else get hex base (16 decimal)
        LD   (BASE),A          ; store it at BASE
        LD   DE,SRTMSG         ; start message address to DE
ABORT:
        LD   SP,STACK          ; set data stack pointer (FC00h)
        PUSH DE                ; push message address for TYPE
        LD   HL,0
        LD   (MODE),HL         ; set MODE=0, STATE=0 (adjacent bytes)
        LD   IY,NEXT           ; IY = address of NEXT (010Ch)
        LD   IX,RETURN         ; set return stack pointer (FB00h)
        LD   HL,8080h
        LD   (LBEND),HL        ; two terminators at end of line buffer
        LD   BC,OUTER          ; IR = address of TYPE in outer interp
        JP   NEXT              ; jump to inner interpreter
```

Notes:
- OUTER is the address of the TYPE word address in the outer interpreter
  threaded list, so the first thing executed is TYPE (displays the message).
- BASE=0 on cold start distinguishes start from restart.
- MODE and STATE are adjacent bytes, zeroed by a single 16-bit store.
- 8080h = two bytes with bit 7 set = terminators for line buffer.

## Listing 5.2 — INLINE

Input line editor. 128-byte buffer on page boundary (L register = offset).

```z80
INLINE: ; *+2 (primitive code address)
        PUSH BC                ; save IR
START:  CALL $CRLF             ; issue CR-LF
        LD   HL,LBADD          ; get start of line buffer
        LD   (LBP),HL          ; reset line buffer pointer
        LD   B,LENGTH          ; set buffer length (128)
CLEAR:  LD   (HL),20h          ; load space to buffer
        INC  HL                ; bump buffer pointer
        DJNZ CLEAR             ; loop to clear buffer
ZERO:   LD   L,0               ; back to first buffer location
INKEY:  CALL $KEY              ; input a character
        CP   "LD"              ; is it a line delete?  [Note: system-specific code]
        JR   NZ,TSTBS          ; if not, skip LD code
        CALL $ECHO             ; else echo line delete char
        JP   START             ; and start over
TSTBS:  CP   "BS"              ; is it a backspace?   [Note: system-specific code]
        JR   NZ,TSTCR          ; if not, skip BS code
        DEC  L                 ; decrement buffer pointer
        JP   M,ZERO            ; reset to zero if negative (underflow)
        LD   (HL),20h          ; load space to buffer (erase char)
ISSUE:  CALL $ECHO             ; display the character
        JP   INKEY             ; and return for next
TSTCR:  CP   "CR"              ; is it a carriage return?  [0Dh]
        JR   Z,LAST1           ; if so, go to exit
        BIT  7,L               ; at 129th place? (buffer full)
        JR   NZ,END            ; do buffer-end task at 129
SAVEIT: LD   (HL),A            ; save character in buffer
        CP   61h               ; is it less than lowercase 'a'?
        JR   C,NOTLC           ; if so, skip lowercase code
        CP   7Bh               ; is it more than lowercase 'z'?
        JR   NC,NOTLC          ; if so, skip lowercase code
        RES  5,(HL)            ; else convert lowercase to uppercase
NOTLC:  INC  L                 ; bump pointer
        JR   ISSUE             ; go echo character
END:    DEC  L                 ; back up to 128th place
        LD   C,A               ; save the input character
        LD   A,"BS"            ; get backspace character
        CALL $ECHO             ; move cursor left
        LD   A,C               ; restore original character
        JR   SAVEIT            ; go put it at 128th place
LAST1:  LD   A,20h             ; replace CR by a space
        CALL $ECHO             ; and echo it
        POP  BC                ; restore IR
        JP   (IY)              ; return to NEXT
```

Notes:
- "LD", "BS", "CR" are system-specific ASCII control codes for line delete,
  backspace, and carriage return. Typically: CR=0Dh, BS=08h, LD=18h (CAN).
- Buffer sits on a page boundary so L register alone serves as offset.
- Lowercase is converted to uppercase by RES 5 (clears bit 5).
- Buffer length is 128 bytes. Two terminators (80h) sit at LBEND after buffer.

## Listing 5.3 — TOKEN

Extracts next token from line buffer to dictionary free space.

```z80
        ; Header: DATA #5,T,O,K  / DATA "LINK"
TOKEN:  ; *+2 (primitive code address)
        EXX                    ; save IR in alternate BC
        LD   HL,(LBP)          ; get pointer to current position in buffer
        LD   DE,(DP)           ; get pointer to free dictionary space
        POP  BC                ; separator in C, B is zero (from stack)
        LD   A,20h             ; space code to A
        CP   C                 ; is separator a space?
        JR   NZ,TOK            ; if not, start of token
IGNLB:  CP   (HL)              ; is current char a space?
        JR   NZ,TOK            ; if not, start of token
        INC  L                 ; bump pointer (skip leading spaces)
        JR   IGNLB             ; try next character
TOK:    PUSH HL                ; save token start address
COUNT:  INC  B                 ; increment count
        INC  L                 ; bump pointer
        LD   A,(HL)            ; get next character
        CP   C                 ; is it the separator?
        JR   Z,ENDTOK          ; if so, token end
        RLA                    ; bit 7 to carry (terminator check)
        JR   NC,COUNT          ; if CY=0, not at end, continue
        DEC  L                 ; back up 1 if a terminator
ENDTOK: INC  L                 ; step past separator
        LD   (LBP),HL          ; update LBP for next call
        LD   A,B               ; move count to A
        LD   (DE),A            ; length to dictionary
        INC  DE                ; bump dictionary address
        POP  HL                ; get token start address
        LD   C,B               ; get count to BC
        LD   B,0
        LDIR                   ; move token to dictionary
        EXX                    ; restore IR
        JP   (IY)              ; return to NEXT
```

## Listing 5.4 — SEARCH

Searches linked list for keyword matching token at dictionary free space.

```z80
        ; Header: DATA #6,S,E,A / DATA "LINK"
SEARCH: ; *+2 (primitive code address)
        EXX                    ; save IR
        POP  HL                ; get 1st header address (from stack)
TESTIT: PUSH HL                ; save start of header
        LD   DE,(DP)           ; get dictionary pointer
        LD   C,0               ; C used with B as False flag
        LD   A,(DE)            ; get dictionary token length
        CP   (HL)              ; same as keyword length?
        JR   NZ,NXTHDR         ; if not, go to next header
        CP   4                 ; is length over 3?
        JR   C,BEL04           ; if not, skip 3-set code
        LD   A,3               ; set length to 3 (max chars to compare)
BEL04:  LD   B,A               ; save length as count
NXTCH:  INC  HL                ; bump header pointer
        INC  DE                ; bump dictionary pointer
        LD   A,(DE)            ; get next dictionary character
        CP   (HL)              ; match keyword character?
        JR   NZ,NXTHDR         ; if not, go to next header
        DJNZ NXTCH             ; else test next character
        ; Match found
        POP  HL                ; start of found header
        LD   DE,6
        ADD  HL,DE             ; header + 6 = word address
        PUSH HL                ; push WA
        JR   FLAG              ; BC=0 = False flag (found)
NXTHDR: POP  HL                ; get start of current header
        LD   DE,4
        ADD  HL,DE             ; + 4 = link address location
        LD   E,(HL)            ; get link address
        INC  HL
        LD   D,(HL)
        EX   DE,HL             ; HL = next header address
        LD   A,H               ; test link address for zero
        OR   L
        JR   NZ,TESTIT         ; if not 0, test next header
        LD   C,1               ; flag = 1 = True (not found)
FLAG:   PUSH BC                ; push flag
        EXX                    ; restore IR
        JP   (IY)              ; return to NEXT
```

Returns: (WA, 0=False) if found, or (1=True) if not found.

## Listing 5.5 — NUMBER

Converts ASCII token at dictionary free space to binary number.
Most complex routine in the system.

```z80
NUMBER: ; headerless primitive, code address only
        EXX                    ; save IR
        LD   HL,(DP)           ; get pointer to dictionary
        LD   B,(HL)            ; get length of token (count)
        INC  HL                ; bump pointer
        LD   A,(HL)            ; get first character
        CP   2Dh               ; is it a minus sign?
        LD   A,0               ; set sign flag to false
        JR   NZ,SKPSAV         ; if positive, skip to flag save
        DEC  A                 ; make sign flag true (A=FFh)
        DEC  B                 ; decrease count by 1
        INC  HL                ; bump past minus sign
SKPSAV: EX   AF,AF'            ; save sign flag in AF'
        LD   DE,0              ; zero DE
        PUSH DE                ; save as flag (False=0)
        PUSH DE                ; save as result (0)
NLOOP:  LD   A,(HL)            ; get next character
        SUB  30h               ; subtract numbers bias
        JR   C,NOTNO           ; if CY=1, not a number (<0)
        CP   0Ah               ; less than 10?
        JR   C,NUMB            ; if CY=1, it's a digit
        CP   11h               ; is it a letter (>=11h)?
        JR   C,NOTNO           ; if not, it's not a number
        SUB  7                 ; subtract additional letters bias (A=10..F=15)
NUMB:   LD   E,A               ; save binary number in E
        LD   A,(BASE)          ; get system number base
        DEC  A                 ; valid set is {0, BASE-1}
        CP   E                 ; is the binary number valid?
        JR   NC,ANUMB          ; cheers, it's a valid number
NOTNO:  POP  HL                ; pop result, leaving False on stack
        EXX                    ; restore IR
        JP   (IY)              ; return — stack has (False)
ANUMB:  EX   (SP),HL           ; get result & save pointer
        EX   DE,HL             ; result to DE as multiplicand
        PUSH BC                ; save count
        PUSH HL                ; save new binary number
        LD   BC,0800h          ; B=8 (multiply count), C=0
        INC  A                 ; restore BASE in A (multiplier)
MLOOP:  LD   L,C               ; zero HL as product area
        LD   H,C
        ADD  HL,HL             ; shift product left 1 bit
        ADC  A,A               ; shift multiplier left 1 bit
        JR   NC,SKPADD         ; if CY=0, skip add
        ADD  HL,DE             ; else add multiplicand
SKPADD: DJNZ MLOOP             ; loop to complete multiply
        POP  DE                ; get binary number back
        ADD  HL,DE             ; result = product + number
        POP  BC                ; restore count
        EX   (SP),HL           ; get pointer & save result
        INC  HL                ; bump pointer
        DJNZ NLOOP             ; loop for all characters
        POP  DE                ; get final result
        POP  HL                ; the False flag (a zero)
        EX   AF,AF'            ; get sign flag from AF'
        AND  A                 ; is it zero? (also CY=0)
        JR   Z,DONE            ; skip complement if positive
        SBC  HL,DE             ; complement result (0-result)
        EX   DE,HL             ; final result to DE
DONE:   PUSH DE                ; result to stack
        SCF                    ; make AF true (nonzero)
        PUSH AF                ; push True flag
        EXX                    ; restore IR
        JP   (IY)              ; return — stack has (result, True)
```

Returns: (number, True) if valid, or (False) if not valid.

## Listing 5.6 — QUESTION

Handles end-of-line (terminator) or unknown token.

```z80
QUESTION: ; *+2 (primitive code address)
        LD   HL,(DP)           ; get pointer to dictionary
        INC  HL                ; step over token length
        BIT  7,(HL)            ; if bit set, it's a terminator
        JR   Z,ERROR           ; if not set, it's an error
        LD   DE,OK             ; put OK message address in DE
        JP   (IY)              ; return to NEXT
ERROR:  CALL $CRLF             ; issue CR-LF before unknown token
        LD   IY,RETURN         ; set IY to return to this routine
        DEC  HL                ; back-up to token length
        JP   TYPE              ; go echo unknown token
RETURN: LD   DE,MSG?           ; ? message address to DE
        JP   $PATCH            ; go patch system before restart
```

## Listing 5.7 — *STACK

Stack underflow detection.

```z80
*STACK: ; *+2 (primitive code address)
        LD   HL,STACK          ; get top of stack address (FC00h)
        AND  A                 ; reset carry flag
        SBC  HL,SP             ; subtract current SP
        JR   NC,OK             ; if CY=0, no underflow
        ADD  HL,SP             ; else restore top address
        LD   SP,HL             ; and reset stack pointer
        LD   DE,STKMSG         ; stack error message address
        JP   $PATCH            ; go patch system before restart
OK:     JP   (IY)              ; return to NEXT if no underflow
```

## Listing 5.8 — $PATCH

Compile-mode error recovery. Delinks aborted dictionary entry.

```z80
$PATCH:
        LD   A,(MODE)          ; get MODE variable
        AND  A                 ; is it zero (execute mode)?
        JP   Z,ABORT           ; if so, go to restart
        PUSH DE                ; else save message address
        LD   HL,(CURRENT)      ; get vocabulary address
        LD   E,(HL)            ; it points to the latest
        INC  HL                ;   entry, which was aborted
        LD   D,(HL)            ; this is where DP should point
        EX   DE,HL
        LD   (DP),HL           ; restore DP
        LD   A,5               ; bump pointer to the
        ADD  A,L               ;   link address of the aborted
        LD   L,A               ;   keyword by adding 5
        JR   NC,SKIP
        INC  H
SKIP:   LD   A,(HL)            ; move link address to the
        LD   (DE),A            ;   CURRENT vocabulary as
        DEC  HL                ;   the pointer to its
        DEC  DE                ;   latest entry
        LD   A,(HL)
        LD   (DE),A
        POP  DE                ; restore message address
        JP   ABORT             ; and exit to restart
```
