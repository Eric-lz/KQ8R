# What exists for KQ8 (research notes)

Searched October 2026. "Not verified" means I saw the resource but did not
check its claims against the game files myself.

## Format documentation and tools

### nicode.net (Nico Bendlin) — the main resource

Site: <https://kqmoe.nicode.net/>

| Resource | License | What it is | Status |
| --- | --- | --- | --- |
| `kqmoevol` — <https://svn.nicode.net/kqmoevol/> | MIT | VOL archive extractor in C, includes its own PKWARE DCL decompressor. The GOG install script (`kqmoepfd.cmd`, `kqmoevol.exe` in the game folder) comes from here. | **Used.** Our VOL reader is based on the format it documents. Verified against all 48 archives. |
| `libkq8fpc` — <https://svn.nicode.net/libkq8fpc/> | GPL-3.0 | Free Pascal library: bitmaps, bitmap arrays, palettes, fonts. | Not used as code (license would force GPL on us). |
| `libkq8fpc/doc/` | Lives in the GPL-3.0 repo | Plain-text notes on 12 formats: `aud` (sound), `dml` (material list), `gui` (dialogs), `kqs` (3D shapes, the biggest note), `lip` (lip sync), `msg` (dialogue text), `pbma` (bitmap arrays), `pbmp` (bitmaps), `pers` (the shared object header), `pfon` (fonts), `ppal` (palettes), `tag` (GUI text tags). | We read these for *facts about the formats* and write our own code. We do not copy their text into the repo. Not verified yet beyond VOL. |
| `kqmoefix` shim | Not stated | Run-time patch for the "TalkComplete" random cutscene freeze. | Worth studying later: tells us a real bug in the original we should not reproduce. |
| `kqmoepal` | Not stated | Palette viewer. | Not looked at. |

### KQ8FixWin11 (SegMash)

<https://github.com/SegMash/KQ8FixWin11> — Windows 11 compatibility fix
(dgVoodoo setup, replaced video playback). No separate license file; says it
follows kqmoefix's license. Does not document formats. Useful tip from its
README: `aviPlayed=1` in `main.kq` skips the intro videos.
Latest release: `10_25` "KQ8 Windows 11 patch" (published 2025-10-28);
last push to the repo 2026-01-08. The `kq8fix.exe`/`kq8fix.dll` in our
install come from it.

### ScummVM

Not supported. The development compatibility list (updated 2026-10-08)
covers King's Quest I–VII only; KQ8 is a 3D Dynamix-engine game, not SCI.

## Decompilation / reimplementation projects

**None found.** No KQ8 decomp, source port, or engine remake turned up in
web searches, the "awesome-decomp-games" list, Wikipedia's list of game
engine recreations, or
[GameDecompLibrary](https://github.com/solarfren69420/GameDecompLibrary)
(304 projects, commit `0e1357d`, 2026-10-07; no Sierra, Dynamix or King's
Quest entries). Not proof that none exists, but nothing public and
findable.

## Game version

- Install: GOG.com, product ID 1207661053; `Version.txt` says "Version 1.3"
  (the AGENTS.md "1.03").
- GOGdb lists one Galaxy build (9171, 2015-06-08) and one offline installer
  (500.2 MB, no version number; re-uploaded 2026-06-23). No newer game
  version exists that we know of.
- Single-player only. No online mode, no anti-cheat, no DRM in the GOG build.

## The game's licence (EULA.txt in the install)

Flagged, because it restricts what this project does. Not legal advice.

- Forbids reverse engineering, decompiling, disassembling, modifying, or
  creating derivative works of the program.
- Says any "New Game Materials" made public are owned by Activision.
- Forbids modifying the game's EXE/DLL files.

How the project limits its exposure: we never modify or redistribute game
files; the repo holds only our own code and plain-language notes; we prefer
learning from data files and community docs over disassembly; and nothing
is published without a check. Whether the EULA clauses are enforceable
depends on the country (some laws allow reverse engineering for
interoperability). Ask a lawyer before publishing.

## Engine lineage (lead, partly verified)

KQ8 is built on Dynamix's in-house engine, the family that also powers
*Earthsiege 2*, *Starsiege* and *Starsiege: Tribes* (1998):

- Verified: `Mask.exe` contains the build path `dynamix\sim\code\simbase.cpp`.
- Verified: `*.cs` files are Dynamix console scripts (`set`, `alias`,
  `loadKQ`, `KQGame::...`).
- Verified: `Mask.exe` embeds the PKWARE DCL library (used for VOLs).
- From Nico's notes: class names like `TS::CelAnimMesh`,
  `Persistent::Base`, `GFXBitmap`, `GFXPalette` match Dynamix naming.

So Tribes-community format knowledge (DTS shapes, PBM/PPL, DML, PFT) may
partly apply. **Not verified**: KQ8 is older or a fork, and its VOL format
already differs from Tribes' `PVOL`. Check each format against KQ8's files.

## Reference project for structure

IW4L — <https://github.com/vladtrc/iw4L> (Apache-2.0). Rust + Bevy/wgpu
runtime for *Call of Duty: MW2* that reads the player's own install. We
copy its *layout ideas* only: Cargo workspace under `crates/`, asset readers
feeding the engine, an `*-artifacts/` folder for logs/caches, docs folder,
credits for every source. No code copied.

## Useful original-behaviour facts noticed so far

- `console.cs` sets `KQGame::FPSLimit 15` and `KQGame::BitDepth 8`.
- `mask.cs` builds the world's search volumes from `KQWorld::Name`,
  `KQGame::BitDepth` and `KQGame::Language` (e.g. `Daventry/8 Bit`).
