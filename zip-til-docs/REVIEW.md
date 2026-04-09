# ZIP TIL Extraction — Review (post-PDF pass)

This review covered the 8 extracted documents in `zip-til-docs/` against
CONTENTS.md and the stated "Next Steps" (stitch into a single
assembleable source file, build a Z80 VM in Rust, boot ZIP). A second
pass was made directly against the book PDF to close the gaps; the PDF
has since been removed from the repo.

## What's in the pack

8 markdown files reconstructing the Z80 source of Loeliger's ZIP Forth
from *Threaded Interpretive Languages* (Byte Books, 1981). Structure
matches CONTENTS.md:

- Hex-validated inner interpreter at `$0100–$012F`
  (`01-inner-interpreter.md`)
- The 8 Chapter 5 outer interpreter routines
  (`02-outer-interpreter.md`)
- 14 branch/control primitives + 4 literal handlers
  (`03-branch-control-primitives.md`)
- Chapter 6 word definitions with 6 arithmetic subroutines
  (`04-word-definitions.md`)
- Secondaries, compiler directives, system variables/constants/messages,
  dictionary header format, memory map, I/O subroutine specs
  (`05-secondaries-system.md`)
- Extraction inventory with OCR error catalogue
  (`06-extraction-inventory.md`)
- Chapter 7 Z80 assembler extension with ~60-mnemonic Zilog
  cross-reference table (`07-assembler-extension.md`)

## Gaps from the first review — now resolved

1. **`*ENDF` turned out to be an OCR artifact.** Figure 5.2 actually
   reads `*END F3` (primitive `*END` with relative offset `F3` = -13),
   not a phantom `*ENDF` primitive with offset `03`. The outer
   interpreter threaded code in `02-outer-interpreter.md` has been
   corrected, along with the branch-target math in the annotations.

2. **`?SEARCH` flag polarity is consistent.** FALSE = 0 = success
   (found / is a number) throughout. The earlier confusion was a
   stale comment in `02-outer-interpreter.md`: it described `*IF, 0B`
   as "if True (not found), skip to ?NUMBER" when in fact `*IF`
   branches forward on **FALSE** and the target is `?EXECUTE`. The
   comments have been rewritten to match the actual byte-level math.

3. **Missing secondaries extracted from Ch.6:**
   - `0=` (Relational primitive) → `04-word-definitions.md`
   - `<#`, `#>` (I/O primitives) → `04-word-definitions.md`
   - `ASCII` (I/O primitive) → `04-word-definitions.md`
   - `D/MOD` (Arithmetic primitive) → `04-word-definitions.md`
   - `DO,`, `END,` (System secondaries) → `05-secondaries-system.md`

4. **Inventory/doc mismatches corrected.** `MIN`, `MAX`, `MOD`, `D*`,
   `*/`, `*/MOD`, `ABORT`, `CVARIABLE`, `*C+LOOP`, `*CLEAVE` were all
   in the book and have been added. `#S` was already present in 05 but
   mis-marked NOTED in the inventory; fixed. `ABORT` is `JP START`,
   referenced by `$PATCH` as the total-panic recovery.

5. **SYS block layout committed.** The book deliberately leaves this
   implementation-defined ("only 20 thru 30 bytes are used"). A concrete
   16-byte layout is now specified in `05-secondaries-system.md`, with
   BASE/MODE/STATE at offsets 0/1/2 (MODE and STATE adjacent so the
   single 16-bit zero-store in Listing 5.1 clears both).

6. **Dictionary header format documented concretely** from Figure 2.1:
   6 bytes total = length (bit 7 = immediate) + 3 name chars + 2-byte
   link. Body starts at +6 with the 2-byte code address; that code
   address slot's address is the keyword's "word address". See
   `05-secondaries-system.md`, "Dictionary Header Format" section.

7. **System-specific values pinned down** from Figure 3.2 and Listings
   5.1/5.2:
   - `STACK = FC00h` (data stack initial SP)
   - `RETURN = FA00h` (return stack initial IX) — the previous
     extraction said `FB00h`, which contradicted both the memory map
     figure and the "512 bytes for data stack" text. Fixed.
   - `SYS = F880h` (chosen: start of system-variable block)
   - `LBADD = F800h` (page-aligned, required by INLINE)
   - `LENGTH = 80h` (128 bytes)
   - `LBEND = F880h` (where the 8080h dual terminators go)
   - ASCII control codes: CR=0Dh, BS=08h; LD (line delete) is
     system-specific and set to 18h (ASCII CAN) here.

## Still not extracted (intentional)

These are documented in the book but out of scope for this repo's
goal of booting a ZIP VM. CONTENTS.md already enumerates them:

- Chapter 7 virtual memory system (BLOCK, RESIDENT, BUFFER, GETIT,
  PUTIT, SAVE, UPDATE) — requires disk hardware we won't emulate.
- Chapter 7 editor (Listing 7.1) — depends on virtual memory.
- Chapter 7 floating point — format discussion only, no impl code.
- Chapter 7 cross-compilation — design discussion only, no code.
- `ADUMP`, `DUMP`, `FORGET`, `.R` — non-essential for boot.

## Readiness for the stated "Next Steps"

**Stitching into a single assembleable source file — now viable.** With
gaps 1-7 closed, the remaining work is mechanical:
- Assign absolute addresses starting from `0100h` for the inner
  interpreter, then lay primitives and secondaries contiguously.
- Build the headered-keyword link chain per the format in
  `05-secondaries-system.md`, terminating at `0000h`.
- Emit the SYS block at `F880h` and initialise BASE=0, MODE=0, STATE=0
  before jumping to START.

**Rust Z80 VM — ready to start.** Memory map, register conventions,
dictionary format, `$KEY`/`$ECHO` boundary, and the inner interpreter
hex are all pinned down. The VM can be developed in parallel with the
assembly pass; it should exist first so the ROM can be validated
incrementally (inner interpreter → primitives → outer interpreter).
