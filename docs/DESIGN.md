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
   `docs/SESSIONS.md` (kept locally, not published).
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

Who does what (see `docs/SESSIONS.md`, kept locally):
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

#### M1 step 3c — place only what the world scripts load (planned 2026-10-11)

Rules from `docs/behaviour/objects.md` ("Where a world's objects come
from", Q16; "See-through objects", Q14; "Follow-ups", Q13) and the
corrections to `buildings.md` rule 2 and material flag `0x10000`.

- `kq8-data/world_script.rs`: follow the world's `newWorld.cs` line by
  line. A line whose first word is a `.cs` file runs that script at that
  point (`WorldVar.cs`, `WorldInv.cs`, `permwrld.cs`); `#` starts a
  comment; `loadKQ <file> [<name> [x y z [dirX dirY dirZ]]]` is recorded.
  Other commands are counted, not run. `if … endif` blocks are not
  evaluated; `loadKQ` lines inside them are counted and logged (Daventry:
  none).
- Placement per `loadKQ`: the object is the file's first section. Name:
  `same` or missing = keep, `@` = none, else rename. With x y z, the
  position replaces `locX/Y/Z` and the angles are the given ones or
  0 0 0 (the `.kq`'s `dirX/Y/Z` are ignored); with only a file name, the
  `.kq`'s own position and angles. The ground rule applies the same way.
  Files with several sections that have a position are logged.
- The viewer places only these objects (no more "all `.kq` files" and
  no more "(0, 0, 0) = template" guess).
- `translucency=<n>` below 256: every material of that object drawn at
  opacity n/255. The distance fade needs haze, which we do not draw yet.
- Material flag `0x10000` no longer means see-through (it is fog).
- Tests: script parsing (comments, nested scripts, name rules, partial
  positions); log: objects loaded per script, skipped and why.

#### M1 step 3d — `loadKQ` z of −1/−2 = on the ground (planned 2026-10-11)

Rule from `docs/behaviour/objects.md` "Ground heights −1 and −2" (Q17).
Small fix: a `loadKQ` line whose z is exactly −1 or −2 puts the object at
the ground height under (x, y), up or down, whatever `keepOnGround` says.
A `.kq` file's own `locZ` keeps the old rule (raise only, with
`keepOnGround`). `Cottage2–4` and `wateran1–9` stay skipped (no model).
Log: how many objects were put on the ground this way.

#### M1 step 3e — script variables, `getLoc` and `move` (planned 2026-10-11)

Rules from `docs/behaviour/objects.md` "Script variables, `getLoc` and
`move`" (Q20) and the Q17 correction.

- `kq8-data/world_script.rs` keeps a table of variables while the scripts
  run, in order:
  - `set <name> <words…>`: the variable holds the words joined by spaces
    (after their own `$` replacement).
  - A script run as `<file>.cs a b` gets `$1` = a, `$2` = b.
  - A `$name` word that is not the first word of a line is replaced by
    its text, which is split into words again. Words in double quotes stay
    one word. An unknown variable makes the line unreadable (logged).
  - `getLoc <name>` / `KQObject::getLoc <name>` asks the caller for the
    object's current position and angles, then sets `$KQObject::loc`
    (six numbers) and `$KQObject::locOnly` (three).
  - `move <who> <flags> x y z [rx ry rz]` (missing numbers 0) is passed to
    the caller.
  - The runner talks to the caller through a small interface ("host"):
    read a script, place a `loadKQ`, tell where a named object is, move
    one. The data library stays free of drawing code.
- `loadKQ` with 4 or 5 numbers: x y z, extra numbers ignored, angles
  0 0 0 (Q20 e).
- `.kq` values that are `$variables` are replaced the same way before the
  object is read (e.g. the anvil's `locX=$Barren::Seller::xPos`).
- Viewer: the ground rules run when the object is loaded, so `getLoc`
  sees the result. Every loaded object is remembered by name (also those
  with no model, like the Weirdling). `move` changes the remembered
  object, then what is drawn:
  - flags 0 to 3 as in the note (absolute or relative position and turn);
  - a relative position on an object that is turned: world axes (Q21);
    these moves are counted in the log;
  - a relative turn: when the object and the turn are both only about
    the vertical axis, the two angles add up (true in any order; all 9
    cases in the game). Any other relative turn is not applied (not
    traced); counted and logged;
  - `move` does not apply the −1/−2 rule (Q17 correction); moves with
    z −1/−2 are counted and logged.
- Log: variables set, unknown variables, `getLoc` and `move` lines (done,
  object not found, not applied and why). Tests: substitution, quotes,
  `$1`, `getLoc` → `loadKQ`, 4-number lines, `move` flags.

#### M1 step 3f — hidden objects (planned 2026-10-11)

Rules from `docs/behaviour/objects.md` "Hidden objects" (Q18). Small fix.

- `hidden=<n>`: any whole number other than 0 = hidden. Scripts switch it
  with `KQObject::setHidden <name> 0|1`; the start-up scripts' lines are
  followed.
- A hidden object is not drawn at the start, unless its `groupsAlways`
  (default 6) has bit 0x40 (the "drawn" bit; inferred by session A, not
  traced). In the viewer, such objects are shown with the H key, like
  unused objects.
- Log: hidden objects, `setHidden` lines, objects drawn although hidden.

### M2 — Walk Connor around Daventry

**You will see:** Connor walking, running and turning, the camera following,
not falling through the ground or walking through walls.

- B: animations (`.anm`), Connor's model.
- A: Connor's movement (speeds, turning, jumping, slopes), camera behaviour,
  collision rules, controls of the original.
- B: implement from those notes.

Notes: `docs/behaviour/connor-movement.md`, `connor-physics.md`,
`camera.md`.

**Physics rate (decided 2026-10-11).** The game logic runs at a **fixed
15 updates per second**, as the original does (`KQGame::FPSLimit 15`).
Every per-update value is then used exactly as traced: the fall timer
grows by 0.06 s per update, step up/down are 325 per update, the air
counter counts updates. This is the simplest way to match the original's
jumps, falls and steps (including its gravity running at 0.9 × real time;
see `connor-physics.md` "Do not reproduce"). Drawing runs at the screen's
rate: Connor is drawn **between his last two updates** (blended by the time
since the last update), so motion looks smooth. The blend is display only
and never changes the logic. If a higher rate is wanted later, it is a
separate, logged change.

**Where the code goes.** A new crate `crates/kq8-game` (no Bevy) holds the
game rules: Connor's mover first. It asks the world for the ground under a
point through a small interface, so it can be unit-tested without the
game files. The viewer runs it at 15 updates per second.

Steps (each separately revertable, each stopped for an eye check):

- **2a — Connor stands in Daventry, gravity on (planned 2026-10-11).**
  Connor is read from `GAME/Kq/Connor.kq` when the world script loads
  `connor.kq` (it is not in the world archives), placed by that line
  (Daventry: 277744, 199295, z −1 = on the ground), drawn in the first frame
  of his `stop` movement (`Conner.anm` → `c_stop.kqs`), coloured with
  `KQWorld::connorPalette`. The mover runs at 15 updates per second with
  **gravity on** (`connor-physics.md` "Is gravity on at the start?":
  recommendation) and only the ground (terrain), no buildings or objects
  yet:
  - each update: up pass (step up 325), across (no movement yet), down pass
    (gravity drop + step down 325), ground check (`keepOnGround`) with the
    0.8 walkable test;
  - gravity: fall timer +0.06 per update, drop `5000 × (t² − t_prev²) / 2`,
    at most 5,000 per update; timer back to 0 on walkable ground;
  - air counter, the Fall state (start at 5 updates in the air, timer
    0.36, the "stuck" count, end rules) and the death test (more than
    3,840 below the start). Animations for falling come in a later step;
    here they are logged only.
  - Steep ground (normal z ≤ 0.8): logged as steep; sliding comes in 2c.
  - Test keys: **G** lifts Connor 5,000 above where he is (should die on
    landing), **B** lifts him 2,000 (safe landing). **C** puts the camera
    behind Connor.
  - Log (`kq8-artifacts/logs/connor.log`), one line per update: time,
    position, on ground, gravity, fall timer, air counter, state, ground
    height and normal; state changes and deaths also in the main log.
- **2b — walk, run, back and turn (planned 2026-10-11).** Rules from
  `connor-movement.md`.
  - Keys read from `MASK/Connor.sae` (new reader in `kq8-data`): the keys
    whose `make` sends `MoveY 1` / `MoveY -1` / `RotateZ 1` / `RotateZ -1`
    / `Run`. As the note recommends ("Do not reproduce"), the intent comes
    from the keys **held**: of two opposite keys held, the one pressed last
    wins, and releasing it gives the other back. (No setting for the
    original behaviour for now, as the user asked.) `Run` toggles; Connor
    starts with run on (Daventry's scripts do not set it).
  - Movement chosen from the intent by the note's table (`run`, `walk`,
    `back`, `left`, `right`, `stop`), each from `Conner.anm`.
  - Speed from the animation: the loop's per-frame steps added up, divided
    by the loop's length, times the playback speed (`back` = walk at −1).
    The steps are **spread evenly** over the loop (the note says this is
    close; whether the game applies them per frame is asked as Q24).
  - Turning: 1.75 rad/s × 1/15 s per update while a turn key is held,
    standing, walking or running. No banking yet.
  - No start/stop animations (`stopwalk`, `walkstopleft`...) yet: he goes
    straight from one movement to the next, starting the loop at its
    beginning. Which transition plays is asked as Q25.
  - All frames of each movement's loop are drawn in turn, at the loop's
    speed.
  - The rules live in `kq8-game` (`connor.rs`): intent → movement, turning,
    the wanted step for the mover. Unit tests: the table, speeds, turn
    rate.
  - Viewer: F makes the camera follow Connor (keeps its offset; a
    stand-in until the real camera, 2f).
  - Log per update: keys' intent, movement, heading, loop position, wanted
    step; the measured speeds of walk and run at load.
- **2c** — slopes: sliding on steep ground, climbing up to 36.9°.
- **2d** — walls: buildings and objects stop him or let him slide along
  (collision shapes need a question: `connor-physics.md` open question 2).
- **2e** — jumping, falling with its animations, crash, death animation.
- **2f — the following camera (planned 2026-10-11).** Moved up at the
  user's request: done right after 2b, before 2c. Rules from `camera.md`.
  - The camera logic runs in the same 15-updates-per-second schedule as
    Connor, right after him, so every per-update rate in the note (height
    10% per update, tilt 0.015 rad per update, zoom 125 per update) is used
    as traced; drawing blends between its last two updates, like Connor.
  - Rules in `kq8-game` (`camera.rs`), unit-tested: aim point 1,400 above
    Connor's base; normal mode keeps the camera where it is unless it is
    farther than the maximum (pulled to max − 1) or closer than the minimum
    (pushed to min + 1), in 3D; fixed mode (F11) keeps its distance, zoom
    changes it within min + 1 … max − 1; height = the higher of terrain
    under the camera + 1,400 and Connor's base + 1,400, jumped to within
    100, else 10% of the gap per update; right mouse held: sideways swings
    round the aim point (3π/4 per mouse unit), forward/back zooms (5,000 per
    unit, kept in range, never past the aim point) or tilts in fixed mode or
    with Shift (0.75 per unit); the first update after pressing uses no
    movement; Page Up/Down tilt 0.015 rad per update, limits ±1.57, Home
    resets; tilt snaps back when Connor moves more than 10 (|Δx| + |Δy|)
    in normal mode.
  - Keys from `MASK/Camera.sae` (F11, Page Up/Down, Home) and `Connor.sae`
    (right mouse, Shift). The letter keys in `Camera.sae` (development
    moves) are not used.
  - Range: 1,500–5,000, the same walking and running (changed after the
    user compared with the original: no difference; Q27 e).
  - Start: where the world script loads `cam3rd.kq` (`KQCamera`, class
    `KQCamMover`; Daventry 275026, 197541, 14930), else behind Connor.
    Daventry's script ends with `KQCamera::follow none` (probably its
    opening scene); we follow Connor anyway (Q27 d).
  - Not done here: the obstacle check (needs 2d), the automatic swing back
    behind Connor (not decoded: Q27 c), first person (needs Connor's
    first-person models), shake and cutscene cameras.
  - Stand-ins until Q27 is answered, all logged: mouse value = pixels ÷
    600 (a); the view looks exactly at the aim point at the default tilt,
    and other tilts pitch it by the difference (b); mouse forward = zoom in
    (f).
  - Viewer: F switches between the game camera (default when Connor is
    there) and the free flying camera. Log: camera position, distance to
    the aim point, height above the terrain, tilt, mode, range, per update.

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
