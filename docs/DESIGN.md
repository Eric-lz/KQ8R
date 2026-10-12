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
  - gravity: fall timer +0.06 per update, drop `5000 × (t² − t_prev²) / 2`
    (changed to 4 times that on 2026-10-12, see the follow-up under 2c),
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
- **2c + falling part of 2e — slopes and falls (planned 2026-10-11).**
  Done together at the user's request, after Q26 and Q23 were answered.
  Rules from `connor-physics.md`: "One update of the mover", "The terrain in
  the passes" (with the recommended rule for walkable downhill), "Gravity",
  "Slopes", "Falling off an edge". The terrain is still the only solid
  thing (buildings and objects come in 2d).
  - The mover's update (`kq8-game` `mover.rs`):
    1. Start: as before. It also remembers whether he began the update on
       walkable ground (for the downhill rule).
    2. Up pass: step up + the animation's height change. At its end, if
       the terrain under him is higher than where he ended, that is a hit:
       he stops at the terrain (in effect, he is raised to it). The note
       says nothing else happens outside a jump. This should be rare,
       because each update ends with him at or above the terrain. Logged.
    3. Across pass: the wanted x, y. The terrain is not tested (Q26).
    4. Down pass: this update's gravity drop along the pull, then step
       down 325. The fall timer is +0.06 per update. If the pull points
       less than 30° below horizontal, the timer goes back to 0 and there
       is no drop. At the end the terrain is tested:
       - **walkable hit** (normal z > 0.8): he is put on the contact point,
         on ground, fall timer 0;
       - **steep hit**: not on ground, the pull points down the slope,
         gravity is switched on, and he is **pushed off**. The rest of the
         move, from the contact point to where the pass ended, is
         projected onto the surface and done as an across move (terrain
         ignored). With no walls yet that is always one try; the
         3-try limit is kept for 2d;
       - **no hit: the downhill rule** (**removed 2026-10-12**, not needed
         with the measured gravity; see MODLOG) (the note's recommendation, our own
         design, not traced): if he began the update on walkable ground and
         there is walkable terrain no more than 325 + this update's drop
         below where the pass ended, he is put on it (on ground, fall
         timer 0). Logged as `snap`.
    5. Ground check: as before (at or below the terrain → on it, 0.8 test),
       but it **no longer resets the fall timer** (the note: only the down
       pass does).
  - The Fall state: asked for on **every** update from 5 updates in the air
    on (before, only on the 5th), and accepted in normal play. There is no
    "controls taken away" flag yet. Then: fall timer 0.36, the stuck
    count, the end rules, death if more than 3,840 below the start (as in
    2a).
  - Animations: Fall plays `jumphold` (repeats). Its own steps replace the
    walk's, so his forward movement stops, as the user saw in the
    original. The keys are ignored while falling, including turning (the
    note doesn't say; a stand-in, logged). On a safe landing, `jumpend`
    plays once with the keys ignored, then he goes back to the movement
    the keys ask for. On death, `jumpdie` plays once, then the Die state:
    he stays on its last frame. The `jumpdie.aud` sound isn't played (no
    sound yet). One-shot animations run from the first frame to the last
    (`objects.md`, loop rule 2: `frameCount − 1` steps). The walker hears
    about a fall starting, a landing or a death from the mover's report
    and switches on the next update (1/15 s later).
  - Test helper: `--at X,Y,H` starts Connor on the ground at X, Y, facing
    heading H (radians), instead of the script's spot. Combined with
    `--hold`, it lets a steep hillside or a steep walkable slope be tried
    the same way each time. Spots found with a throwaway terrain scan
    (in the scratch folder, not committed).
  - Log (`connor.log`), new columns per update: began on walkable ground,
    up-pass contact, down-pass result (`none` / `walkable` / `steep` /
    `snap`) and its contact normal (x y z), push-off length, ground-check
    result, pull z, animation mode (`free` / `fall` / `land` / `dying` /
    `dead`). The air counter and on-ground flag are logged as before.
  - Unit tests on made-up ground (flat, planes at set angles, a ledge):
    flat ground unchanged; running into a 45° hill: forward gain shrinks
    each update, Fall after 5 updates, then a slide down; running down
    20°, 30° and 36° planes: the air counter never reaches 5; ledge 5,000:
    Fall, then death; ledge 300: followed with no fall; the ground check
    leaves the fall timer alone.
  - Known difference: the note's 45° example (80, 71, 62, 53, 44 units)
    assumes the gravity drop goes straight down. The note's own Gravity
    rules put it along the pull after a steep contact. We follow the rules
    and log the numbers; the difference goes to `QUESTIONS.md`.
  - **Follow-up after Q28 (a) and Q29 (planned 2026-10-11):**
    - the gravity drop d is projected onto the pull: he moves d·(−pull z)
      along the pull (d·sinθ on a slope of angle θ; the full d when the
      pull is straight down). The 45° unit test then expects the note's
      80, 71, 62, 53, 44;
    - the test lift (G, B, `--lift`) no longer clears "on ground". The
      update before the lift ended on the ground, so the first update after
      it doesn't count as in the air. Fall then starts after 5 real updates
      in the air (at update 6, not 5). The count for real updates already
      matches Q29 and is unchanged;
    - still open, no change: Q28 (b), (c) and why he gains so little height
      on steep slopes (`connor-physics.md` open question 7).
  - **Follow-up after the fall-speed measurement (planned 2026-10-12).**
    Two steps, each separately revertable. Rules from `connor-physics.md`
    "Gravity" and "Falling off an edge" rule 2 (Q30).
    - **G1, gravity strength:** the drop per update becomes
      `10000 × (t² − t_prev²)` (36, 108, 180, … units; 4 times the old
      drop), as measured in the running original. Same fall timer
      (+0.06 per update), same 5,000 cap, same projection onto the pull.
      In code: `GRAVITY` from 5,000 to 20,000 (the formula keeps its
      `/ 2`, so `20000 × (t² − t_prev²) / 2` is exactly the note's rule).
      Unit tests: first three drops 36, 108, 180; the 45° hill test's
      expected numbers recomputed from the note's own rule
      (168·cos²θ − d·sinθ·cosθ, now with d = 36, 108, …), so the gains
      turn negative after two updates (he slides back sooner). Done: the
      gains are 66, 30, −6, −42, and in update 5 the slide carries him
      back onto the flat, so only 4 updates end in the air and **no Fall**
      starts there (the old test expected one).
    - **G2, entering Fall from walking or standing leaves gravity alone:**
      no more setting the fall timer to 0.36 when Fall starts; it runs on
      from the 5 updates in the air (0.30). The 0.36 head start belongs
      to jumps only and comes with 2e (kept as a constant for then).
      Unit test: a walk-off's drops are 36, 108, 180, 252, 324, then 396,
      468 in Fall (no jump from 324 to 468).
    - Log: no new columns. `connor.log` already has the fall timer and
      the drop per update, so both steps can be checked from the numbers.
- **2d — walls (plan rewritten 2026-10-12 from the traced rules, Q31 and
  Q33; waiting for the user's OK).** Every rule below is traced or read
  from the data; there are no stand-ins of our own. Rules from
  `connor-physics.md`: "Where the values live", "Collision shape" (the
  ball stack, the building test steps 1–7, the object test, the 25-unit
  pull-back), "The terrain in the passes" (nearest contact wins), "Walls"
  (head-on, "Exact hit handling", the measured part). Data:
  `GAME/patch/BOUNDS.KQ` (every collision box, by name), `.dig` tables 2,
  3 and 8 (BSP nodes, leaves, planes), `Conner.anm` (`crash`). Five
  sub-steps, each separately revertable and each stopped for a check.
  - **Words used here.** A *BSP tree* ("binary space partition") is the
    tree of flat planes stored in each building file: each branch point
    (*node*) splits space by one plane into a front side and a back side,
    and each end of the tree (*leaf*) is a region that is either solid
    (inside a wall) or open. A *sweep* moves a shape from its old to its
    new position and finds the first point along the way where it would
    touch something.
  - **Connor's shape** (traced, and read from the running game): his box
    is the `Connor` entry of `BOUNDS.KQ`, −200…200, −200…200, 0…1,300
    (**not** the 1,600-tall `[ConnerCollider]` in `Connor.kq`, which an
    earlier version of this plan used). It becomes a stack of balls of
    radius 200 on his vertical axis, at heights 200, 600, 1,000 and 1,100
    (the top one at the box top − r, so it overlaps the one below). The
    balls are built from the box by the note's rule, not typed in, so the
    same code serves monsters later. They don't turn with him; turning is
    never checked.
  - **2d-1, the data (nothing moves differently yet).** *Done
    2026-10-12 (MODLOG). Found on the way: names used twice in
    `BOUNDS.KQ`, a `#item968` line and a second inside-out box
    (`iceshard.kqs`), asked as Q34 (only matters for 2d-5); unused BSP
    leaves with first byte 2.*
    - `kq8-data`: a reader for `BOUNDS.KQ` (`[BoundingBoxes]`,
      `itemN=<name> xmin ymin zmin xmax ymax zmax`, lines starting with `;`
      are comments). Lookup by name, ignoring case. Read from
      `GAME/patch/` (the patch copy, which differs from `GAME/Kq/Bounds.kq`
      only in two door entries).
    - `building.rs`: read table 2 (nodes: plane number u16, front child
      i16, back child i16, 2 unused bytes) and table 3 (leaves: 48 bytes;
      only the first byte's bit 0 = solid is used). Child below 0 = leaf
      number −1 − child. The tree starts at node 0 (traced).
    - Checks with `kq8 building-verify`, on all 153 `.dig` files: every
      child and plane number in range; no node lists node 0 as a child;
      every node reachable from node 0; counts of solid and open leaves.
      And `BOUNDS.KQ`: 1,111 entries read, the `Connor` entry as above.
    - Unit tests for both readers on made-up bytes.
  - **2d-2, the collision tests (`kq8-game`, new `collide.rs`; still not
    used by the mover).** *Done 2026-10-12 (MODLOG). Checked against
    the original's measurements (house floor, Tomb wall: Q36). Added on
    the way: the touch test's 0.00001 margin is done in 64-bit numbers,
    because in 32-bit `200 + 0.00001` is exactly 200. Inside-out object
    boxes are refused (no hit) until Q34 (c) is answered.*
    - **Ball stack from a box** (the rule above).
    - **Building test**, exactly the note's steps 1–7: no test for a move
      under 0.1; work in the building's own space (its placement undone:
      the same rotation, position and "raise to the ground" rule as
      drawing); each of the four balls traced from node 0, sorted at each
      node as in front / behind / touching with the margin r + 0.00001;
      split where the ball first touches the plane moved by r; both
      children when both ends touch, keeping the nearer hit; a solid leaf
      entered across a plane is a hit, starting inside solid is not; the
      hit's surface direction is the crossed plane, flipped to point
      against the move (walls block from both sides); the earliest of
      the balls' hits counts; the contact is that hit minus the ball's
      offset, back in world space, then **pulled back 25** toward the old
      position (the old position itself if within 25).
    - **Object test** (used in 2d-5): each ball swept in the object's own
      space (the box turns with the object) against the box **enlarged
      by r on every face** (a bigger box with square corners, not a
      rounded one); earliest hit; the hit face's axis turned back into the
      world as the surface direction; the same 25 pull-back. An exactly
      vertical move tests only the top ball going up (or not moving) and
      the bottom ball going down. Moves under 0.001 in every axis are not
      tested.
    - **Hit handler:** over everything hit in one pass, the contact
      nearest the old position is kept. Against the terrain (up and down
      passes only), whichever is nearer the pass's start wins; terrain
      contacts get no pull-back.
    - Unit tests on a made-up one-wall building (one node, a solid back
      leaf): walking at it from either side stops with his centre 225
      from the wall; a move along the wall doesn't hit; a move that only
      touches the plane; a ball starting inside the wall: no hit; a floor:
      feet stop 25 above it; a ceiling: top ball stops 25 below it; the
      earliest of four balls wins when only the top one hits (a low
      lintel). Object test: square-cornered enlarged box, rotated object,
      the vertical one-ball rule.
  - **2d-3, buildings in the passes; glance and stop.** *Done
    2026-10-12 (MODLOG). Changes from this plan: the rough check covers
    the balls' full reach instead of the game's √3 rule (whose starting
    point isn't traced; it can't change a result). Four readings of the
    note asked as Q37: head-on outside Move counts as glancing; one count
    of 3 hits per update over all passes; the head-on test also on
    down-pass and slide hits; with the walk key held against a wall the
    movement alternates walk/stop each update. Running head-on stops at
    the contact but keeps running until 2d-4 (crash).*
    - The up, across and down passes, and every slide retry, sweep Connor
      against the buildings near the move (a rough check first, by each
      building's bounding box grown by the note's √3 × his largest
      half-size, only to save time). The terrain stays as it is: tested
      only at the end of the up and down passes, never in the across pass.
      The ground check (`keepOnGround`) stays terrain-only.
    - "Exact hit handling" as traced. With the kept contact (already
      pulled back 25):
      - **Head-on:** the angle between this pass's horizontal movement
        (wanted end − start, unit length) and the reversed surface
        direction is under 0.35 rad; zero horizontal movement is never
        head-on. His movement, not his facing, so stepping backwards into
        a wall is head-on. Acts only in state Move (or Jump, 2e), so not
        while standing. Walking or stepping back head-on: he is put at the
        contact and the movement switches straight to `stop` (state Wait,
        no stop animation; measured). With the key still held he moves
        again next update and is stopped again.
      - **Up-pass hits and walkable landings** (down pass, surface
        direction z ≥ 0.8): put at the contact, no slide.
      - **Everything else** (glancing, steep floors or roofs in the down
        pass): target = wanted end − n × ((wanted end − contact) · n);
        set at the contact and swept to the target as an across pass. A
        hit there runs the same handling again (its own head-on test, with
        the slide as the movement); on the 3rd hit in one update he stays
        at that contact. The steep terrain push-off already works this way
        (the same formula, with no pull-back for the terrain); it moves
        into this shared code so buildings can stop it.
    - So, by the traced rules: he stands 25 above building floors and
      stops with his centre 225 from a wall. **Neither is checked in the
      original yet** (see "Still to check").
    - Unit tests on made-up buildings: the head-on stop (walking: `stop`;
      backwards: also head-on); glancing at 21.6° and 35° from the normal
      slides 0.37 and 0.57 of the step (the measured factors); into a
      corner, stops at the 3rd hit; past the end of a wall, full step; a
      325 step climbed, a 400 one blocked; under a low ceiling the up
      pass is cut short; standing on a building floor stays 25 above it
      update after update.
    - Log per update (`connor.log`), new columns: for each pass, what it
      hit (`terrain`, a building's name, later an object's name) and the
      surface direction; the across result (`free` / `glance` /
      `head-on` / `3 hits`), hits used, the head-on angle; on a building
      floor, his height above it.
  - **2d-4, crash when running head-on.** Running (`run`; the note's
    other run-like movements don't exist in our code yet) head-on into a
    wall: `crash` plays **in full** (traced, Q31 e). Its own 7 steps move
    him 441.5 straight back (63 per update; read from the data, matches
    the measurement), taken by the mover like any other step, so a wall
    behind him still stops it. Keys are ignored while it plays; then he
    stands, and with run still held he runs again (the original plays
    `stoprun` first; we have no start animations yet, Q25) and can crash
    again, as measured. Not done: the head-on exceptions that need things
    we don't have yet (`KQMonster`, `backJump`, the unknown Connor flag,
    the "no crash" flags of open question 4). Unit test: run into a wall,
    `crash` for 7 updates, 441.5 back, then `run`.
  - **2d-5, objects.**
    - Box: from `BOUNDS.KQ` by name when the object enters the world:
      first its shape's name (for an animated object, the shape of the
      movement it has then, e.g. `signpst1.kqs`), then its own name. Used
      as is, in the object's own space, fixed (doesn't follow the
      animation). **Not in the table: never collides** (e.g. Daventry's
      `Davtorch`).
    - Type and mask: `colliderType` / `colliderMask` from its `.kq` (lines
      starting with `#` are switched off), defaults type 0x41, mask 0;
      the script commands `collisionType` and `collision` (a negative
      number clears bits). Type 0 and mask 0: never collides. A pair is
      tested when Connor's mask 0x281B shares a bit with its type, or its
      mask shares a bit with his type 4.
    - **Hidden objects still collide** (traced).
    - Log: per world, objects with a box, without one, and with type and
      mask 0; which object stopped him.
    - Unit tests on made-up objects; a check in Daventry (the signpost:
      box −250…−10, 0…250, 0…2,420).
  - **Still to check or not traced** (none blocks 2d; each is logged
    where it happens):
    - **25 above floors: confirmed** (2026-10-12, Q35). In the original
      his feet are at 13,980 in Connor's house; the house's floor in our
      data is at 13,955.
    - **225 from a wall: confirmed** (2026-10-12, Q36, Q37). The sweep
      at the Tomb wall gives the measured contacts to within 0.1 unit,
      and so does the viewer with Connor's real movement (2d-3).
    - A ball that starts inside an object's box: "a hit with no usable
      contact"; what follows was not traced. Logged if it happens.
    - The inside-out `lfswdoor.kqs` box (xmin 10 > xmax −1,520): not
      traced; logged if a world uses it (to be checked in 2d-5).
    - Walkable is "> 0.8" in the slope rules and "≥ 0.8" in the hit
      handling. They differ only at exactly 0.8, which doesn't happen in
      practice; we use each as written.
  - **Not in 2d:** the camera's obstacle check (after 2d), jumping (2e),
    Connor against monsters (ball stack against ball stack, not traced;
    no monsters move in M2), landing on a monster (pushed 250 sideways),
    water, ropes, pushing objects.
  - **How the user will test 2d:** spots next to Daventry buildings
    found by a throwaway scan, given as `--at` and `--hold` commands: a
    straight wall to walk and run into head-on, at about 30°, and
    backwards; a doorway; steps; the inside of a building; the signpost.
    Compared with the original by eye and with `connor.log`.
- **2e** — jumping, crash, the rest of the falling animations (after jumps).
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
