# ZIP TIL Extraction — Document Contents

## Start Here

Read the files in numerical order. Together they constitute the complete
extracted and corrected Z80 source code from Loeliger's book, ready to
be assembled and run in a VM.

## Files

| File | Description |
|------|-------------|
| `CONTENTS.md` | This file |
| `01-inner-interpreter.md` | **Start here.** Table 3.3 fully reconstructed with hex validation. SEMI, NEXT, RUN, COLON, EXECUTE. Register assignments and memory map. |
| `02-outer-interpreter.md` | Chapter 5 routines: START/RESTART, INLINE, TOKEN, SEARCH, NUMBER, QUESTION, *STACK, $PATCH. The outer interpreter threaded code list. |
| `03-branch-control-primitives.md` | All branch/loop primitives: $ELSE, $WHILE, *IF, *END, *DO, *CDO, *LOOP, *CLOOP, *+LOOP, *LEAVE. Literal handlers: *#, *C#, *[, *SYS. |
| `04-word-definitions.md` | All Chapter 6 primitives with Z80 code: stack ops, memory ops, arithmetic (including $ISIGN/$OSIGN/$UD*/$US*/$UD//$US/ subroutines), relationals, logicals, I/O, interstack, system, utility, defining words. |
| `05-secondaries-system.md` | Secondary (threaded code) definitions, compiler directives, system variables, system constants, system messages, I/O subroutine specs. |
| `06-extraction-inventory.md` | Complete inventory of all extracted keywords with status. OCR error catalogue. List of system-specific routines that the VM must provide. |
| `07-assembler-extension.md` | Chapter 7 Z80 assembler: ~60 mnemonic keywords, defining words, structured constructs (IF/THEN, BEGIN/END, DO/LOOP), CODE/NEXT vocabulary management. Full mnemonic cross-reference table. |

## Coverage

- **Inner interpreter**: Complete, hex-validated
- **Outer interpreter**: Complete (8 routines)
- **Branch/control**: Complete (14 primitives + 4 literal handlers)
- **Stack operations**: Complete (DROP, DUP, 2DUP, SWAP, OVER, LROT, RROT, 2OVER, 2SWAP)
- **Memory operations**: Complete (!, @, C!, C@, +!, C+!, comma, C,, 0!, 1!)
- **Arithmetic**: Complete including all 6 subroutines and 10+ operators
- **Relational/Logical**: Complete (<, =, >, AND, IOR, XOR, NOT)
- **I/O**: Complete (TYPE, ECHO, KEY, SPACE, DISPLAY, SIGN, $CRLF)
- **Interstack**: Complete (<R, R>, I>, J>, CI>, CJ>, C<R, CR>)
- **Defining words**: Complete (CREATE, CONSTANT, CCONSTANT, VARIABLE, CVARIABLE, <BUILDS, DOES>)
- **System**: Complete (EXECUTE, HEX, DECIMAL, OCTAL, BINARY, SINGLE, +SP, -SP, ?RS, ?SP, SCODE)
- **Compiler directives**: Complete (IF, THEN, ELSE, BEGIN, END, WHILE, DO, LOOP, LEAVE, :, ;)

## Not Yet Extracted

- Chapter 7 virtual memory system (BLOCK, RESIDENT, BUFFER, GETIT,
  PUTIT, SAVE, UPDATE) — requires disk hardware we won't have
- Chapter 7 editor (Listing 7.1, 2 screens of TIL source) — depends on
  virtual memory
- Chapter 7 floating point — format discussion only, no implementation code
- Chapter 7 cross-compilation — design discussion only, no code
- A few secondary definitions marked NOTED in the inventory
  (ADUMP, DUMP, FORGET, .R — all non-essential for boot)
- Some words described only in prose without code (MOD standalone,
  MAX standalone) — can be reconstructed from descriptions

## Next Steps (for Claude Code)

1. Stitch all primitives into a single assembleable source file with
   absolute addresses, resolving all symbolic references.
2. Build the Z80 VM in Rust (CPU decode, 64K memory, console I/O).
3. Assemble to binary, load into VM, boot ZIP.
