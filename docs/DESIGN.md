# Design

How this project works, in plain language. Updated when the architecture
changes, not on every commit.

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* (KQ8, Sierra 1998) that
reads every asset from the player's own installed copy at runtime. The repo
contains only our own source code; no game files, no decompiled code.

## Ground rules that shape the design

- **Game data is read, never written.** The game folder (`./MASK/` by
  default) is opened read-only. The original game unpacks the current world
  from `data/<World>/*.vol` onto the hard disk; we do not do that. We read
  the `.vol` archives directly into memory instead.
- **Everything we produce goes to `kq8-artifacts/`** (logs, debug dumps).
  That folder is not in the `.gitignore` whitelist, so it is never committed.
  (Same idea as IW4L's `iw4l-artifacts/`.)
- **Log, don't look.** Every tool writes numbers (counts, sizes, offsets,
  timings) to a log so results can be checked without looking at the screen.
- **Match the original.** When a format detail is unclear, check it against
  the real files (and, later, the original `Mask.exe` in a disassembler kept
  outside the repo) rather than guessing.

## Working method for studying the original

Adopted from ai-game-modding-guides, guides 3 and 13 (not legal advice).

1. **Prefer the data files.** Learn formats from the game's own files and
   from community documentation before looking at `Mask.exe`.
2. **Record where every fact came from** in `docs/provenance/`.
3. **If disassembly is needed, split the work across two sessions.**
   Session A studies `Mask.exe` (in Ghidra, with all output kept outside the
   repo) and writes a behaviour description in its own words into
   `docs/formats/` or `docs/behaviour/`. Session B, a fresh chat with no
   access to the disassembly, writes the Rust code from that description
   only. Not a legal safe harbour, but much better than one session
   doing both.
4. **Never transcribe.** Describe *what* the original does (sizes, units,
   rules, timings), never *how its code is laid out* (struct field order,
   function structure).
5. **Do not reproduce the original's bugs, dead code, or unused fields**,
   such as the "TalkComplete" cutscene freeze that kqmoefix works around.
   Where we deliberately differ, say so in a code comment and in
   `docs/provenance/`.

## Repository layout

Modelled on IW4L (<https://github.com/vladtrc/iw4L>): a Cargo *workspace*
(one repo holding several Rust packages, called *crates*) under `crates/`.

```text
Cargo.toml            workspace definition
crates/
  kq8-data/           library: finds and reads game files (VOL archives, ...)
  kq8-cli/            the `kq8` command-line tool for checks and dumps
docs/                 DESIGN.md (this file), RESEARCH.md (what exists online)
docs/provenance/      where each piece of knowledge came from
kq8-artifacts/        logs and dumps (never committed)
reference/            third-party repos cloned for reading (never committed)
MASK/                 the game install (never committed)
```

More crates will come later (rendering, audio, game logic). Asset *parsing*
stays in `kq8-data` so the engine never touches raw bytes directly.

## Game data layout (as found in the GOG 1.03 install)

| Where | What |
| --- | --- |
| `data/<World>/{kq,resource,8bit,8gui}.vol` | Packed world data, 12 worlds, 48 archives, 133 MB |
| `data/<World>/{English,KQ,Sound}/` | Loose files next to the archives (speech, messages, ...) |
| `GAME/` | Shared data used by all worlds |
| `<world>/{kq,resource,8bit,8gui}/` | Copies unpacked by the GOG/kqmoevol install script. Used by us **only** as a reference to check our VOL reader |

## VOL archive format

Learned from Nico Bendlin's MIT-licensed `kqmoevol` extractor (see
`CREDITS.md`) and confirmed against the real files:

- 8-byte signature: `RESLIST` followed by a zero byte.
- Then entries, one after the other. Each entry is:
  - 20-byte file name (ASCII, zero-terminated),
  - 4-byte little-endian offset of the *next* entry (from start of file),
  - the file data, compressed with PKWARE DCL "implode" (an old 1980s/90s
    compression method), running up to the next entry.
- The last entry's "next" offset equals the archive size.

Decompression uses the `explode` crate (MIT), a Rust port of Mark Adler's
`blast.c`.

## Plan

Each step is small and separately revertable.

1. **Prove the data loads** (done).
   `kq8 vol-verify` opens all 48 VOL archives, decompresses every entry in
   memory, and compares each one byte-for-byte with the already-unpacked copy
   in `./<world>/<archive>/`. It logs per-archive counts and a final total.
   Success = every entry decompresses and every comparison matches.
2. Read one real asset format on top of that: a `.pbm` bitmap + `.ppl`
   palette, converted to a PNG in `kq8-artifacts/` for checking (done).
   - `kq8-data`: `bitmap.rs` (image size, bits per pixel, flags, mipmap
     count, pixels) and `palette.rs` (256 colours plus a list of the extra
     lighting/haze tables, which are not decoded yet).
   - `kq8 bitmap-png <file> [--palette <file>]` writes
     `kq8-artifacts/png/<name>.png` and logs size, format and colour use.
     `<file>` is either a path in the game folder (`GAME/8Gui/daventry.pbm`)
     or a file inside an archive (`vol:Daventry/8bit/NAME.BMP`).
   - `kq8 bitmap-verify` parses every image and palette in the game (all
     archives and loose files) and logs counts per variant and every
     failure. Success = 0 failures.
   - PNG writing uses the `png` crate (MIT/Apache-2.0) and lives in the
     CLI, so `kq8-data` stays free of output formats.
3. Messages (`.msg`) so text can be checked as plain strings.
4. Models (`.kqs`), then a first window that draws one model (needs a
   graphics library decision, will ask before choosing).

Steps after 4 will be planned here when we get there.
