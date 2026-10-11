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
   `docs/behaviour/`. Session B, a fresh chat with no
   access to the disassembly, writes the Rust code from that description
   only. Not a legal safe harbour, but much better than one session
   doing both. Full rules and the prompts for each session:
   [docs/SESSIONS.md](SESSIONS.md).
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
  kq8-viewer/         window that shows one model (uses Bevy)
docs/                 DESIGN.md (this file), RESEARCH.md (what exists online)
docs/provenance/      where each piece of knowledge came from
docs/behaviour/       behaviour notes (session A) and QUESTIONS.md
kq8-artifacts/        logs and dumps (never committed)
reference/            third-party repos cloned for reading (never committed)
MASK/                 the game install (never committed)
```

More crates will come later (rendering, audio, game logic). Asset *parsing*
stays in `kq8-data` so the engine never touches raw bytes directly. The
viewer follows that rule: `kq8-data` turns a model into a plain list of
triangles (positions, texture coordinates, material), and only the
viewer's `main.rs` knows about Bevy.

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

## Roadmap

The goal is a fully playable game. That is a multi-month project; the
roadmap is built around **milestones you can see and test**, each made of
small, separately revertable steps. Steps are planned in detail only when
a milestone starts.

Who does what (see [SESSIONS.md](SESSIONS.md)):
- **A** = study session: works out behaviour from `Mask.exe`, writes
  `docs/behaviour/*.md`. No code.
- **B** = implementation session: writes Rust from data files, community
  format notes and behaviour notes. Never looks at `Mask.exe`.

Where the game keeps things (found so far):
- **Data files** say *what* exists: models, textures, terrain, object lists
  (`.kq` files are plain-text object descriptions: class, model, position,
  settings).
- **Scripts** (`.cs`) hold puzzle logic, e.g. "this switch opens that door
  and marks the puzzle solved". We run them with our own interpreter.
- **`Mask.exe`** holds *how* each kind of object behaves (monsters, traps,
  doors, Connor's movement, combat, camera). That needs study session A.

### M0 — Read the game's files (done)

- Step 1: VOL archives. 48 archives, 14,444 files, all match. (done)
- Step 2: images and palettes. 6,495 images, 50 palettes, PNG export,
  confirmed by eye. (done)

### M1 — See Daventry

**You will see:** a window showing Daventry's terrain and objects, with a
free-flying camera.

- Graphics library: **Bevy** (decided 2026-10-11; IW4L uses it). Keep game
  logic in Bevy's ECS (the part that stores game objects and runs their
  logic) and rendering separate, so the drawing part can be
  swapped later if Bevy's renderer ever becomes too slow (ai-game-modding-guides,
  guide 3).
- B: 3D models (`.kqs`) and material lists (`.dml`) → textured model in a
  window. Planned in detail below ("M1 step 1").
- B: terrain (`kq/Terrain`, `.dib` images) → Daventry ground. Planned in
  detail below ("M1 step 2"). Correction: `.dig`/`.dil` are not terrain;
  they are named after buildings (`pub`, `castle`, `wndmill`, ...) and go
  with the 33 `KQInterior` objects, so they belong to step 3.
- B: place objects from the `.kq` files.
- A, only if the data does not explain it: terrain size/scale, how textures
  and palettes are applied, lighting (`.ppl` shade/haze tables).

#### M1 step 1 — one textured model in a window (planned 2026-10-11)

No behaviour note exists. Sources: the game's files and the libkq8fpc
notes `kq8kqs.txt`, `kq8dml.txt`, `kq8pers.txt` (facts only, see
`docs/provenance/`). Split into two parts you can revert separately:

**1a. Read models and material lists** (`kq8-data`, no window yet)

- `pers.rs`: the "PERS" wrapper every model and material list sits in
  (a tag, a size, a class name like `KQ8::Shape`, a version number).
- `material.rs`: `TS::MaterialList` (`.dml`, and the copy at the end of
  every `.kqs`): per material its kind (texture / colour / none),
  shading, colour and texture file name.
- `shape.rs`: `KQ8::Shape` with its `TS::CelAnimMesh` meshes. A model is
  a set of *mesh nodes* (a mesh plus a placement), grouped into
  *sequences* (one per animation frame). Vertices are stored as bytes
  0–254 and turned into positions with a per-frame scale and origin.
- A function that builds the triangles of one frame (positions, texture
  coordinates, material) without any graphics library, so the engine never
  touches raw bytes.
- `kq8 shape-verify`: parses every `.kqs` and `.dml` in the game and logs
  counts, versions, failures, and checks: face indices in range, the two
  bounding-box vertices, transforms that rotate, texture names that cannot
  be found, the range of texture coordinates, which axis is tallest.

**1b. Show it** (new crate `kq8-viewer`, uses Bevy)

- `kq8-viewer <model.kqs> [--world Daventry] [--palette <file>]`: finds
  the model and its textures in that world's archives, builds frame 0,
  shows it with a camera you can orbit with the mouse.
- Textures use the world palette (`resource/<world>.ppl`) by default.
  Reason: measured on 3 Daventry textures, `daventry.ppl` gives the
  smoothest pictures (same as `8bit/daventry.pal`; `daventr1.pal` is worse).
- Drawn without lighting (texture colours as they are), because the
  lighting rules are not known yet. Back faces hidden, front = clockwise
  corners (answer to Q2, `docs/behaviour/models-and-textures.md`).
- The game's axes are converted to Bevy's (Bevy has Y up). Which game axis
  is up is decided from the `shape-verify` numbers.
- Logs to `kq8-artifacts/logs/viewer.log`: vertex/triangle counts, the
  bounding box, every texture found or missing, frame times.
  `--frames N` closes the window after N frames, for a quick check.

Not in this step: animation, lighting, transparency, the normal table
(we compute our own normals instead of copying the game's table).

#### M1 step 2 — Daventry's ground (planned 2026-10-11)

No behaviour note exists. Sources: the game's files only (no community
notes cover terrain). What the files say (observed):

- `kq/Terrain/TERRAIN.KQ` is plain text (`key=value`): it names the
  pieces. Daventry: height image `terrh.dib`, a second one `terrh2.dib`
  (`floorBMPName`, almost the same pixels), material image `terrm.dib`,
  material directions `terrain/daventry.bin`, material list
  `davennew.dml`, plus `groundScale=12`, `heightBMPScale=255`, distances,
  light settings and light/shadow maps (`*.bin`, 526,342 bytes each).
- `terrh.dib`: 129×129 grey values 0–254, i.e. one height per corner of
  128×128 squares. `terrm.dib`: 128×128 values, one material per square.
- `daventry.bin`: a count (255) then 255 records of 28 bytes: own index,
  16-byte texture name (same names, same order as `davennew.dml`), a value
  that is always 0, a value 0–7 (probably the texture's direction).

Size, heights, orientation and the triangle split come from the
behaviour note `docs/behaviour/terrain.md` (answers Q6, Q7). Still open:
how a material's texture covers a square and which way it turns (Q8),
holes, and the second height image `terrh2.dib`.

**2a. Read the terrain files** (`kq8-data`, no window; not blocked)

- `kq_text.rs`: reader for the game's `[section]` / `key=value` text
  files (`TERRAIN.KQ` now, the object `.kq` files in step 3).
- `terrain.rs`: the `TERRAIN.KQ` settings, the height and material grids
  (from the `.dib` images, using the existing bitmap reader), the
  direction records, and the height lookup from `terrain.md`: square =
  2^groundScale, height = value × heightBMPScale, bottom image row = y 0,
  two flat triangles per square with a chessboard diagonal.
- `kq8 terrain-verify`: reads the terrain of every world and logs sizes,
  value ranges, how often each material is used, and checks (grid sizes
  match, every material number exists in the list, direction-file names
  match the list, every texture exists in the world's archives), and the
  ground height at every placed object compared with its own height.
  (Done 2026-10-11.)

**2b. Draw the ground** (planned 2026-10-11, after Q2 and Q8 were answered)

Rules from `docs/behaviour/terrain.md` ("Ground textures") and
`models-and-textures.md` (Q2):

- `kq8-data/terrain.rs` gains `ground_triangles()`: two triangles per
  square, the same diagonals as the height lookup, corners listed
  clockwise seen from above (the game's front-face rule, same as models,
  so the viewer treats both the same way). Each triangle carries its
  square's material number and texture coordinates. Texture coordinates
  follow the note's table for orientation codes 0–7: with u = west→east
  and v = north→south inside the square, code 0 uses (u, v), 1 (1−u, v),
  2 (u, 1−v), 3 (1−u, 1−v), 4 (v, 1−u), 5 (1−v, 1−u), 6 (v, u), 7 (1−v, u).
  One texture covers exactly one square.
- Unit tests: every triangle's centre lies on the height lookup's
  surface; every triangle is clockwise from above; the eight codes give
  the eight different turns and mirrors.
- Viewer: `kq8-viewer --terrain [--world Daventry]`. Textures come from the
  world's archives, coloured with the palette that `world.cs` names in
  `KQWorld::terrainPalette` (falls back to `<world>.ppl` and says so).
  Textures stop at the square's edge instead of repeating. Squares whose
  material has no texture (kind "null") are not drawn and are counted.
  Drawn without lighting, back faces hidden, mipmaps not used (level 0
  only).
- Camera for the ground: fly with W/A/S/D, Q/E down/up, Shift for faster;
  hold the right mouse button to look around. Starts above the middle of
  the grid looking north.
- Log (`kq8-artifacts/logs/viewer.log`): squares drawn and skipped,
  triangles, textures found and missing, the palette used, and every 2 s
  the camera position in game units with the ground height below it.

Not in this step: lighting/shadow maps, haze, holes, `terrh2.dib`, the
buildings (`.dig`/`.dil`) and objects (step 3). (Built 2026-10-11.)

#### M1 step 3 — Daventry's objects (planned 2026-10-11, not started)

Known from the data (observed while checking the terrain): Daventry's
`kq.vol` holds ~430 `.kq` files; 249 sections have a non-zero position
(`locX/locY/locZ`, game units). Classes include `KQObject` (153),
`KQAnimObject`, `KQTrap`, `KQInterior` (33, the buildings), `KQMonster`,
`KQDoor`, `KQChest`. Objects name their model in `shapeName`; buildings
use `.dig`/`.dil` files instead of `.kqs`.

**3a. Place models** (no new format needed)

- Read every `.kq` section with a position and a `shapeName`; draw that
  model's rest pose at its position on the ground in the `--terrain` view.
- Log: objects placed, skipped (no model, model missing, unknown class),
  and per class counts.
- Placement rules from `docs/behaviour/objects.md` (answers Q9, Q10):
  angles `dirX/Y/Z` in radians, right-hand rule, turned about Y first,
  then X, then Z (row vectors: `RotY × RotX × RotZ`); no scale; position
  `locX/Y/Z` used exactly as written (`isGroundRelative` is never read,
  `keepOnGround` only matters for moving objects). A model point is
  placed as: mesh node transform, then that rotation, then the position.
- `kq8-data/objects.rs`: read placements from every `.kq` section, build
  the rotation, unit tests for each axis and the order.
- Viewer `--terrain` also draws the objects: each different model is
  built once and reused. Skipped and counted: sections at (0, 0, 0)
  (probably templates), buildings (`.dig`, step 3b), missing models.
- Check from the numbers: for each object, log the gap between the
  placed model's lowest point and the ground under it; most objects
  outdoors should rest on the ground.
- Added while building (observed in the data): animated objects (doors,
  chests, portals, `KQAnimObject`) name their model through
  `animLoops`/`currAnimName`; shown in their first frame until Q12 says
  which frame is right. Monsters (`.anm`) and emitters wait for M2/M3.
  (Built 2026-10-11.)

**3b. Buildings** (`KQInterior`, `.dig`/`.dil`; planned 2026-10-11)

Rules from `docs/behaviour/buildings.md` (answers Q11 and the Q10
follow-up). A building = `<name>.dig` (shape), `<name>.dil` (lighting,
not decoded, not used yet) and the material list named by `material=`
(Daventry: `interior.dml`, shared by all buildings).

- `kq8-data/building.rs`: read the `.dig` (PERS block, class
  `ITRGeometry`, versions 2 and 4): header (texture scale, bounding box,
  8 counts), the 8 tables with record sizes 20, 8, 48, 1, 4, 12, 8, 16
  bytes. Keep surfaces, corners, points, texture coordinates and planes;
  skip the BSP and visibility tables (not needed to draw). Turn each
  surface into triangles (a fan from its first corner) with texture
  coordinates `(offset + uv × (size + 1)) / texture width or height`,
  textures repeating.
- Front faces are **anticlockwise**, the opposite of models, so the
  viewer flips them the other way. Back faces hidden.
- `kq8 building-verify`: read every `.dig` in the game (153 files) and
  log counts, versions, index checks, and whether each surface's corner
  order agrees with its plane and the 0x80 "flipped" bit.
- Viewer `--terrain`: draw the 33 Daventry buildings with the same
  placement rule as objects (rotation, then position, raised onto the
  ground if below it). Palette from `KQWorld::interiorPalette`. Log per
  building: lowest point vs ground, textures found or missing.
- Not in this step: building lighting (`.dil`), inside/outside visibility
  (BSP), doors between inside and outside. (Built 2026-10-11.)

Not in this step: animation, doors opening, monsters moving (M2/M3).

### M2 — Walk Connor around Daventry

**You will see:** Connor walking, running and turning, the camera following,
not falling through the ground or walking through walls.

- B: animations (`.anm`), Connor's model.
- A: Connor's movement (speeds, turning, jumping, slopes), camera behaviour,
  collision rules, controls of the original.
- B: implement from those notes.

### M3 — Daventry comes alive

**You will see:** doors opening, traps firing, chests, animated objects,
the first puzzles working.

- A: the script language (`.cs` commands like `set`, `if test`,
  `sendEvent`), the event system, and each object kind used in Daventry
  (door, trap, chest, emitter, animation lists...).
- B: script interpreter, event system, the object kinds.

### M4 — Talk, listen, use the menus

**You will see and hear:** dialogue text, speech with lip sync, the
inventory and menus.

- B: messages (`.msg`), audio (`.aud`), fonts (`.pft`), GUI files.
- A: conversations, inventory rules, menu behaviour.

### M5 — Fight

**You will see:** monsters that move, attack and die; Connor's health and
stats.

- A: monster AI per type, combat rules and numbers, stats, death.
- B: implement.

### M6 — The whole game

Save/load, moving between worlds, all 12 worlds, cutscene videos, music,
options. A and B as needed per topic.

### M7 — Match the original, then improve

Play the original next to ours, list differences, fix them. After that,
the improvements this project was started for: controls, graphics,
gameplay changes.
