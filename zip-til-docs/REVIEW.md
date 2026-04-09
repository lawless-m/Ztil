# ZIP TIL Extraction — Review

Review of the 8 extracted documents in `zip-til-docs/` against
CONTENTS.md's claims and the stated "Next Steps" (stitch into a single
assembleable source file, build a Z80 VM in Rust, boot ZIP).

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
  I/O subroutine specs (`05-secondaries-system.md`)
- Extraction inventory with OCR error catalogue
  (`06-extraction-inventory.md`)
- Chapter 7 Z80 assembler extension with ~60-mnemonic Zilog
  cross-reference table (`07-assembler-extension.md`)

## Quality

Solid. Hex in Table 3.3 is internally consistent: addresses, the JR
displacement `E1` → 0112, and the T-state totals all add up
(`01-inner-interpreter.md:50-110`). OCR corrections are catalogued per
file and rolled up in `06-extraction-inventory.md:172-186`. Register,
memory-map, and dictionary-entry conventions are pinned down up-front.
Symbolic labels are used consistently across files.

## Real gaps — beyond what CONTENTS.md's "Not Yet Extracted" already concedes

1. **`*ENDF` appears undefined.** Used in the outer interpreter threaded
   list at `02-outer-interpreter.md:18` (`DW *ENDF, 03`) but never
   defined in `03-branch-control-primitives.md` or the inventory. Looks
   like it needs to be a "branch forward if TRUE" primitive (mirror of
   `*IF`, which branches forward if FALSE). No such primitive exists in
   the extraction.

2. **Flag polarity vs. `*IF` is inconsistent.** `SEARCH`
   (`02-outer-interpreter.md:196-198`) returns `BC=1 = True (not found)`
   / `BC=0 = False (found)`. `*IF`
   (`03-branch-control-primitives.md:56-60`) branches forward on
   **false**. But the comment at `02-outer-interpreter.md:16` says
   "if True (not found), skip to ?NUMBER". Either the comment is wrong
   or `?SEARCH` inverts the flag — it's a headerless wrapper whose body
   isn't shown.

3. **Words used by secondaries but never defined:** `<#`, `#>`, `0=`,
   `ASCII`, `D/MOD`, `DO,`, `END,` — all appear in `.`, `#`, `#S`, and
   the compiler directives of `05-secondaries-system.md` but have no
   Z80 source or threaded-code definition anywhere.

4. **Inventory/doc mismatch:** the inventory lists `MIN`, `*/`, `*/MOD`,
   `D*`, `D/MOD`, `#>` as DONE, but none of them appear in
   `04-word-definitions.md`. Conversely `MAX`, `MOD`, `*C+LOOP`,
   `*CLEAVE`, `CVARIABLE`, `ABORT` are marked NOTED, but `ABORT` is
   *referenced* by `START/RESTART` and `$PATCH` so it must exist.

5. **SYS block layout is undefined.** `*SYS`
   (`03-branch-control-primitives.md:240-247`) uses a 1-byte offset into
   a `SYS` block, but the concrete offsets for BASE/MODE/STATE/DP/LBP/
   CURRENT/CONTEXT/COMPILER are not given anywhere.

6. **Dictionary header preamble is described in prose but never
   concretely assembled.** Listings show
   `; Header: DATA #5,T,O,K / DATA "LINK"` with `LINK` as a placeholder,
   so link-chain ordering and addresses aren't fixed.

7. **System-specific values left symbolic:** `LBADD`, `LBEND`, `LENGTH`,
   `STACK` (FC00h known), `RETURN` (FB00h known), and the ASCII control
   codes `"LD"`, `"BS"`, `"CR"` in INLINE
   (`02-outer-interpreter.md:75-87`).

## Readiness for the stated "Next Steps"

**Stitching into a single assembleable source file — not there yet.**
Needs:

- An address layout for all primitives (only the inner interpreter at
  `$0100–$012F` has absolute addresses today).
- The missing primitives/secondaries listed in gaps (1), (3), (4).
- The SYS block layout (gap 5).
- A real link chain between dictionary headers (gap 6).
- Concrete values for the symbolic constants in gap 7.

**Rust Z80 VM — independently viable and not blocked on the above.**
Memory map, register conventions, dictionary format, and the
`$KEY`/`$ECHO` boundary are documented clearly enough to start, and the
VM should exist before the assembled ROM is finalised in any case so
that the ROM can be validated against it.

## Suggested next direction

Two reasonable paths:

- **(a)** File the gaps above as issues/notes and iterate on the
  extraction (track down `*ENDF`, the missing primitives, and the SYS
  block layout) before touching code.
- **(b)** Start the Rust Z80 VM skeleton in parallel, since it's
  blocked on almost none of the above. The VM can then validate the
  inner interpreter in isolation while the rest of the assembly is
  completed.
