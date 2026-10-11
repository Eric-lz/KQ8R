# Provenance

Where each piece of knowledge in this project came from. Modelled on IW4L's
`docs/provenance/`. Add an entry whenever code is written from a new source.

Source types:

- **Observed**: worked out by reading the player's own game data files.
- **Documented (permissive)**: from a third-party source under a licence
  that allows reuse (MIT, Apache, ...).
- **Documented (GPL)**: from a GPL project. Used only for *facts about file
  layouts*, never code or wording, so our code is not a derivative of it.
- **Disassembly**: from studying `Mask.exe`. Not used yet. See the working
  method in `docs/DESIGN.md` before using this source.

## VOL archives — `crates/kq8-data/src/vol.rs`

| Fact | Source |
| --- | --- |
| `RESLIST\0` signature, 20-byte name, u32 next-entry offset, PKWARE DCL data | Documented (permissive): `kqmoevol.c` by Nico Bendlin, MIT, <https://svn.nicode.net/kqmoevol/> (read 2026-10-11, trunk) |
| Data is PKWARE DCL | Observed: `Mask.exe` contains the "PKWARE Inc." copyright string; the first entry of `Daventry/kq.vol` starts `00 04` (DCL binary mode, 2 KB window); every entry decompressed without error |
| Name may contain a space (`Ogre01 .BMP`) | Documented (permissive): kqmoevol's name check special-cases it. Observed: present once in `Barren/8bit.vol` |
| Last entry's next offset equals archive size; no padding between entries | Observed: `kq8 vol-verify` reports 0 size slack across all 48 archives |

Our code was written from scratch from these facts; no lines were copied.
Decompression is done by the `explode` crate (MIT), not by our code.

## Bitmaps and palettes — `crates/kq8-data/src/{chunks,bitmap,palette}.rs`

"GPL notes" = `libkq8fpc/doc/kq8pbmp.txt` and `kq8ppal.txt` (Nico Bendlin,
GPL-3.0 repository, r34). Facts only; code and wording are our own.

| Fact | Source | Checked how |
| --- | --- | --- |
| Chunk = 4-byte tag, u32 size, payload | GPL notes | Observed: all 6,495 bitmaps and 44 `.ppl` split cleanly to end of file (`kq8 bitmap-verify`) |
| `PBMP` / `PPAL` size field is meaningless; skip the 8-byte marker | GPL notes | Observed: `daventry.pbm` PBMP size is 0x4B020, file is 0x4B038 bytes |
| `head` = u32 (24-bit chunk count, 8-bit version), width, height, bits, flags | GPL notes | Observed: `daventry.pbm` head gives 640×480×8 and the data chunk is exactly 307,200 bytes |
| Head chunk count can be too high for bitmaps | GPL notes | Observed: 74 bitmaps say 3, have 2 |
| Palette head count is 1–2 lower than the real chunk count | Observed (new) | 34 palettes say 3/have 4, 8 say 4/have 5, 2 say 4/have 6. What the engine does with this is unknown |
| Flag bits: 1 transparent, 2 dither, 4 translucent, 8 owns memory | GPL notes | Observed: values seen are only 0x8 and 0x9. Meaning of bits not verified |
| Unmipmapped rows padded to 4 bytes, top row first; 24-bit is blue-green-red | GPL notes | Observed: no unmipmapped 24-bit chunked files exist, so the 24-bit part is untested on real data |
| Mipmapped data: levels back to back, each ¼ of the previous, count in `DETL` (1..9), data padded to 4 | GPL notes | Observed: 128×64 file holds 10,924 bytes = 7 levels + 2 padding; all 1,173 mipmapped files parse; counts seen 3,5,6,7,8,9 |
| Palette `data` = 256 × (red, green, blue, flags) | GPL notes | Observed: entries 1–8 of `Menus.ppl` are the Windows system colours in that order |
| Engine forces first and last 8 colours to the Windows system colours | GPL notes | **Not verified** against `Mask.exe`. Changes 1 entry in `Menus.ppl`/`daventry.ppl` and 15 in the loading palettes; `daventry.pbm` uses none of those colours, so it does not affect its output |
| `.dib` and most archive `.bmp` are plain Windows BMP | Observed | 3,382 `.bmp` + 31 `.dib` start with `BM` and parse as uncompressed BMP; 8 of them are 24-bit |
| `.pal` is a Windows RIFF palette, 237 or 256 colours | Observed | 6 files. Which colours the engine uses from a 237-entry one is unknown |
| `GAME/8Gui/daventry.pbm` uses the loading palette (`GAME/8Gui/l*.ppl`) | Observed (measurement), confirmed by eye by the user 2026-10-11 | Neighbour colour change with every palette in the game: 18.4 with all 16 `GAME/8Gui/l*.ppl` (identical colour tables), 44.5 with `daventry.ppl`, 107.5 with `Menus.ppl`. The user confirmed `daventry.ppl` looks wrong (dark, speckled). Which `l*.ppl` file the engine loads is unknown, but all 16 have the same colours |

## Formats read but not yet implemented

`libkq8fpc/doc/*.txt`: notes on `.msg`, `.kqs`, `.pba`, `.pft` and others
were read for an overview. Each fact taken from them will be listed here
when implemented.
