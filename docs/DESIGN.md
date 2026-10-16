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
  kq8-data/           library: finds and reads game files (VOL archives,
                      models, terrain, object lists, scripts, ...)
  kq8-core/           library: the game's rules, no drawing (Connor, camera,
                      scripts and events, doors, chests, traps, items, ...);
                      named kq8-game until 2026-10-13
  kq8-cli/            the `kq8` command-line tool for checks and dumps
  kq8-viewer/         the window (uses Bevy): draws a world and runs the
                      rules; also the developer's test tool (H key, free
                      camera, `--at`, `--click`, `--aim`, detailed logs)
docs/                 DESIGN.md (this file), RESEARCH.md (what exists online),
                      FEATURES.md (what works, in detail)
docs/provenance/      where each piece of knowledge came from
docs/behaviour/       behaviour notes (session A) and QUESTIONS.md
kq8-artifacts/        logs and dumps (never committed)
reference/            third-party repos cloned for reading (never committed)
MASK/                 the game install (never committed)
```

Data flows game files → `kq8-data` → `kq8-core` → `kq8-viewer`. Open
decision (user): whether the game you play will be a separate program
later (e.g. `kq8-play`, using `kq8-data` and `kq8-core`, with the viewer
staying a test tool); B's recommendation, not decided.

More crates will come later (rendering, audio). Asset *parsing*
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

- `kq8-viewer --model <model.kqs> [--world Daventry] [--palette <file>]`: finds
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
- Viewer: `kq8-viewer [--world Daventry]` (the ground is the default; this was `--terrain` before). Textures come from the
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
  model's rest pose at its position on the ground in the ground view.
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
- Viewer (ground view) also draws the objects: each different model is
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
- Viewer (ground view): draw the 33 Daventry buildings with the same
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

#### M1 step 4 — ground lighting: light and shadow maps (planned 2026-10-14)

Rules from `docs/behaviour/terrain-lighting.md` (rule numbers below are
that note's). Ground only: objects, buildings, haze and moving lights
(rule 15) are not part of this step. Each sub-step is tested and
reported before the next starts.

What the data says (checked 2026-10-14): six worlds name map files
(Castle, Daventry, Dead City, Ice World, Snow exit, Swamp); Barren, Gnome
and Temples 1–4 have empty names. Swamp and Ice World name the same file
for the hardware and the software pair. `MASK/Options.cs` has
`useDynamicLighting 1`.

**4a. Read the maps** (`kq8-data`, `kq8-cli`; no window)

- `TerrainSettings` gains `lightScale` (default 0 when missing; stand-in,
  the note gives no default) and the four map names.
- New `kq8-data/src/lightmap.rs`:
  - `LightMap`: side = 128 × 2^lightScale + 1 (rule 6), spacing =
    2^groundScale / 2^lightScale, values row by row along +x, rows along
    +y from y = 0 (rule 7). Read from a u32 count + count × u16 (rule 5);
    a count ≠ side × side or a short file is an error (the caller then
    keeps full brightness, rule 4). `LightMap::full(side)`: every channel
    15 (rule 4).
  - A value's channels: brightness (bits 12–15), red, green, blue
    (rule 9).
  - `colour_at(x, y)`: the four surrounding points blended bilinearly in
    world coordinates (rule 11; bilinear is the note's assumption,
    **stand-in**, open question 2). Outside the map: the nearest edge
    point.
  - Which file (rules 1–2): we draw like the hardware renderers, so the
    `lightmapName`/`shadowmapName` pair; `useDynamicLighting` 1 → shadow
    map, 0 (or missing) → light map. A reader for that line of
    `Options.cs` (last line wins).
- `kq8 lightmap-verify`: every world: `lightScale`, `groundScale`, the
  four names; for each named file: where it was found, bytes, count,
  side, spacing, a histogram of each channel, whether it is grey; per
  pair, points that differ and by how much; hardware vs software red
  channel as a table of value pairs. Also `Options.cs`'s
  `useDynamicLighting` and which map is shown. Log
  `kq8-artifacts/logs/lightmap-verify.log`. Compared with the note:
  Daventry shadow map red 4 ×20,127, 6 ×4,598, 7 ×236,889, 8 ×1,555; light
  vs shadow differ in 761 points by one step; 263,169 points; file sizes
  in rule 6's table.
- Unit tests: the side rule, value order (point (i, j) = value
  j × side + i), channels, a wrong count is refused, bilinear blend at a
  point, between two points and in a cell's middle, the file choice.
  **Done 2026-10-14** (MODLOG): PASS, 24 files in 6 worlds; Daventry's
  numbers match the note. Other worlds' hardware maps use 15 for sunlit
  ground, which favours ×2 (Q65).

**4b. Colours as stored: no tone mapping** (`kq8-viewer`)

Bevy's 3D camera applies "tone mapping" by default (a curve meant for
photo-like lighting, `TonyMcMapface`): it changes the brightness and
saturation of every colour we draw, so a texture at 100% does not look
like the texture. For an eye comparison of ×1 and ×2 (4c) the screen
must show the colours as computed. Change: every camera gets
`Tonemapping::None`; logged at start. Affects everything drawn, not only
the ground (a separate step so it can be reverted alone).

**4c. Draw the ground lit** (`kq8-viewer`, `kq8-data`)

- The map is chosen and loaded with the ground (4a); missing, unnamed or
  unreadable: full brightness, logged.
- Each ground square is drawn as (2^lightScale)² small cells (Daventry
  4 × 4) whose corners are the map's points, so every map point is a
  mesh corner and gets its colour there. The cells use the square's own
  diagonal direction, so the shape of the ground does not change
  (`terrain.md` rules on triangles). Inside a cell the colour is blended
  across its two triangles: close to bilinear but not the same
  (**stand-in**, open question 2). New `Terrain::square_cells` in
  `kq8-data` (same corners and texture coordinates as
  `square_triangles`, finer).
- The colour multiplies the texture, red by red, green by green, blue by
  blue (rule 10). Factor (planned: ×2 by default; **changed to ×1 after
  the user's check, see below**):
  shown = min(1, 2 × channel / 15); ×1: channel / 15; "off": 1 (as
  before). The multiply is meant to act on the colour as shown on screen
  (an 8-bit card of 1998 has no gamma step), so the factor is turned into
  Bevy's linear colour space before it is stored (stand-in). Brightness
  setting (`setBrightness`, open question 3): ignored (**stand-in**).
- Applies to the drawn surface, the floor view (U key) and the ground
  animators' squares (Daventry's river), which are ground squares too.
- `--light x1|x2|off` and the **F7** key (not used by
  `Connor.sae`/`Camera.sae`) cycles the modes. The on-screen label
  shows the mode.
- Log (`viewer.log`): map file used and why (`useDynamicLighting`),
  count, side, spacing, red-channel histogram; the mode at start and on
  every change; the light under Connor (or the camera when flying) every
  second: raw channels and the shown percentage.
- Check: by eye against the original (terrain-lighting.md "How to
  check"): a hill's shadow on the +y side, shadowed grass about half as
  bright as sunlit grass; ×1 vs ×2 against the original decides open
  question 1.

**4b and 4c done together 2026-10-14** (user's choice; MODLOG). New
`kq8-viewer/src/light_view.rs` (mode, F7, the on-screen line "Ground
light: x2 / x1 / off (F7)" under the U line, the log every second).
Daventry draws 3.2 million corners (surface and the hidden floor view);
average frame 6.8 ms in a 300-frame run.

**User's check 2026-10-14:** ×1 matches the original better (it is very
dark and muddy). ×1 is now the default; F7 cycles ×1 → ×2 → off.
Correction request Q66. Objects and buildings are still drawn full
bright (not part of this step; their lighting is not studied yet).

#### M1 step 5 — object and building lighting (planned 2026-10-15)

Rules from `docs/behaviour/object-lighting.md` (rule numbers below are
that note's). Models get a light per corner from the world's ambient
light and sun; buildings get the per-wall light maps in their `.dil`
file. Each sub-step is tested and reported before the next starts.
Haze (rules 19–23) is **not** in this plan; the user decides later.
Point lights (rule 24, Dead City only) are read and logged but not
drawn.

What the data says (checked 2026-10-15): all 12 worlds have the same
`swtohwlt.cs` (`deleteObject KQLight`, `deleteObject KQAmbientLight`,
`loadKQ light/SunlgtHW.kq`, `loadKQ light/AmblgtHW.kq`), in `kq.vol`.
The light files are loose in `data/<World>/KQ/Light/`. The `.dil` files
are in each world's `resource.vol`, next to the `.dig` files. Only
Gnome (`gnome13.kq`, `gnome14.kq`) and Temple 4 (`temple.kq`) name an
`althwLightmapName`; no file sets `usingAltLighting`.

**5a. Read the lights and the `.dil` files** (`kq8-data`, `kq8-cli`;
nothing drawn)

- New `kq8-data/src/lights.rs`:
  - The files a world loads on hardware: the `loadKQ` lines of its
    `swtohwlt.cs` (rule 1), paths relative to `data/<World>/KQ/`.
  - From those files' sections: ambient lights (`classType=KQAmbientLight`,
    `intensity.red/green/blue`), suns (`classType=KQLight`,
    `tsLightType=1`, `tsLightAim.x/y/z`, `tsLightIntensity.*`) and point
    lights (`classType=KQPointLight`, `tsLightType=2`, `tsLightPosition.*`,
    `tsAttenuationBegin`, `tsAttenuationEnd`; only Dead City has them,
    and `swtohwlt.cs` doesn't load them). The ambient is the **sum** of all ambient lights
    (rule 3). The aim is kept as stored, **not** made unit length
    (rule 5). A missing colour value counts as 0 and is logged
    (stand-in: the note doesn't say; none is missing in the data).
  - `corner_light(normal)` = ambient + Σ sun colour × max(0, −aim ·
    normal), each channel capped to 0–1 (rules 4, 5). `flat_grey` =
    0.299 R + 0.587 G + 0.114 B, capped at 1 (rule 7, with the usual
    weights as "Do not reproduce" says). Point lights are left out
    (not drawn in this step).
- New `kq8-data/src/dil.rs` (`ITRLighting`, the layout table in the
  note): version, build id, light scale shift, the five counts, tables
  A–C skipped by size, table D (one record per surface: byte offset,
  width, height, offset x/y), table E (the texels, 16-bit values,
  4 bits each brightness, red, green, blue), the end u32. A texel
  lookup `texel(surface, i, j)` and the size check of rule 13: width =
  ⌈(offset_x + size_x + 1) ÷ 16⌉ + 1, height likewise.
- `building.rs`: keep the `.dig`'s build id (read now, thrown away).
- New CLI command `kq8 lighting-verify`, log
  `kq8-artifacts/logs/lighting-verify.log`:
  - Every world: the light files `swtohwlt.cs` loads, ambient sum, each
    sun (colour, aim, aim length), point lights, and what a model corner
    gets facing away from the sun, facing up, and facing the sun squarely
    (Daventry must log 0.30, 0.366, 0.42 as the note's table).
  - Every `.dil` in every archive: surface count, texel bytes; for
    each one with a `.dig` of the same name, whether the build ids and
    surface counts match and how many surfaces fail the size check
    (expected: 0 except `temple4/level4`, `swamp/ramp`,
    `daventry/zedrock1`, `zedrock2`, `deadcity/platform`). Red-channel
    histogram for Daventry's files; `cnnrhut.dil` must give 1,129
    surfaces, 162,170 bytes, red 0 ×16,017, 3 ×7,775, 4 ×7,344.
  - Every material list in every archive: how many materials are
    shading none / flat / smooth / no shading bits / other, and how
    many have `0x1000`, `0x2000`, `0x10000` (for 5b's log and to find
    cases the note doesn't cover).
  - Buildings' `.kq` keys `althwLightmapName`, `usingAltLighting`.
- PASS = every light file and `.dil` read, every pair's build id and
  surface count match, size-check misses only in the 5 named files.
- Unit tests: light sums and the capped corner light (Daventry's three
  numbers; Swamp's 1.02 → 1), the flat grey, the `loadKQ` lines, a
  small hand-made `.dil` (layout, texel order and channels, a short
  file refused), the size check.

**Done 2026-10-15** (MODLOG): PASS. Lights: every world's
`swtohwlt.cs` loads its two files; Daventry 0.300 / 0.366 / 0.420 as the
note. `.dil`: all 235 read, 153 of 153 pairs match the `.dig` and fit
rule 13 on every surface; `cnnrhut.dil` matches the note. Three
differences from the note's layout table, measured and asked as Q67:
one more u32 (0) after the shift; 12-byte surface records in the 5
"misfit" files (read that way they fit the size rule); 96 Castle D
surfaces with byte offset `0xFFFFFFFF` (read as "no light map").
Materials: models are 10,981 smooth, 49 flat, 515 none; no model
material has `0x1000` or `0x2000`.

**5b. Light the models** (`kq8-viewer`)

- Every model corner gets a colour by the material's lighting kind
  (rule 7): smooth `0x400` per corner with its own normal; flat `0x200`
  one grey per triangle from its first corner; none `0x100`, and any
  material with `0x1000` or `0x2000`, full brightness. Normals: the
  ones we already compute from the geometry (rule 8). The colour
  multiplies the texture, no doubling (rule 9), as the ground does
  (vertex colours, turned into Bevy's linear colour space; same
  stand-in as M1 4c). The per-object "unlit" switch is treated as off
  (rule 10).
- Normals are turned into world space with the object's rotation
  (rule 5), so the light is worked out when the object is placed, and
  again for objects that turn (Connor, objects whose movement turns
  them).
- **Changed 2026-10-15 (user's choice): the light is computed on the
  graphics card, not stored as vertex colours.** Many objects share one
  mesh while facing different ways (all copies of a model, animation
  frames, items, falling leaves), so colours stored in the mesh can't
  be right for all of them. Instead every model uses a small custom
  material (`object_light.rs` + the shader `object_light.wesl`): each
  corner carries its normal; the shader turns it into world space with
  the object's own rotation, applies rules 4–5 per corner (the colour
  is blended across the triangle, as rule 7 "smooth" says), and
  multiplies the texture. Flat triangles: all three corners carry the
  first corner's normal and the shader uses the grey of rule 7. "None"
  materials: light 1. The world's ambient and suns (up to 4) are in the
  material's settings; F7 changes the mode in every material.
- Corner normals: the average of the unit normals of the triangles
  that share the corner (same mesh node, same vertex number), computed
  in `kq8-data` (`shape.rs`) when the triangles are built.
- Falling leaves and other particles are `.kqs` models too, so they are
  lit the same way (our reading of "models are lit"; the note doesn't
  mention particles).
- Buildings keep the old full-bright material until 5c. The ground
  keeps its vertex colours (M1 4c).

- F7 also switches the object light: ×1 (as the note), ×2 (for
  comparison), off (full bright, as before).
- Log (`viewer.log`): light files used, ambient and suns; how many
  materials are smooth / flat / none; minimum, mean and maximum corner
  light of Connor and of two or three objects near his start
  (Daventry: within 0.30–0.42).
- Check: by eye, objects about as dark as the grass, the sunny side a
  little lighter (the user already confirmed this in the original).

**Done 2026-10-15** (MODLOG): lit in all 8 worlds tried, no shader
errors. Daventry: Connor and the three objects nearest his start stay
within 0.30–0.42 (log); the chest's corner normals all point outward.
Checked by eye by the user (2026-10-15): much closer to the original.
`.gitignore` now also lets `*.wesl` shader
sources under `crates/*/src/` through.

**5c. Light the buildings** (`kq8-viewer`)

- Each wall's colour = texture × its light map (rules 13–14), no sun or
  ambient (rule 16, stand-in), blended smoothly between light texels
  (bilinear, stand-in). The 5 files that miss the size rule: lookups
  clamped to the map's edge (stand-in). No `.dil`, or a build id that
  doesn't match: unlit (full brightness) and logged. `0x1000` building
  materials use the light map too (open question 7, stand-in).
- How to draw it is decided at the start of 5c, with the user. Two
  ways: (1) cut every wall into cells at the light texels and give
  each corner its colour, as the ground does (no new drawing code, but
  many more corners and fiddly cutting of each polygon); (2) put each
  building's light maps into one image and give the walls a second set
  of texture coordinates, drawn with a small custom material (the
  graphics card does the bilinear blend, as Glide did, and the corner
  count stays the same). B recommends (2).
- Log: per building, the `.dil` used, surfaces lit, surfaces failing
  the size check; Connor's hut red histogram as in 5a.
- Check: by eye, inside Connor's hut darker than outside and darker in
  the corners, but reasonably lit (the user's check in the original).

**5c plan, way (2), chosen by the user 2026-10-15** (Q67 answered the
same day):

- New `kq8-viewer/src/building_light.rs` and shader
  `building_light.wesl`: `BuildingMaterial` = the wall texture × a light
  image, no sun or ambient (rule 16, stand-in), x1 / x2 / off with F7
  like the rest.
- Per building model, one **light image** ("atlas"): every surface's
  light map packed into it side by side (rows of maps, tallest first).
  Texel 0,0 is kept white (all channels 15) for surfaces with no map.
  Each texel holds red, green, blue = channel × 17 (0–255, plain, not
  sRGB), so the card's bilinear blend gives channel ÷ 15 blended
  (rules 13, 14). Edges clamped; no mipmaps.
- Each wall corner gets a second set of picture coordinates into the
  light image. With p = the corner's texture position measured from
  the surface's own texture origin (texel position minus the surface's
  texture offset, `buildings.md` step 3), the light map coordinate is
  i = (p + offset_x) ÷ 16 (rule 13: texel i sits at 16 i − offset_x),
  and the picture coordinate is (map's x in the image + i + 0.5) ÷ image
  width, same for y. The size rule (checked in 5a for every surface)
  keeps i between 0 and width − 1, so the blend never reaches a
  neighbouring map.
- Which `.dil`: `<shape>.dil` (rule 12; no world switches the alternate
  maps on at the start). Missing, unreadable, or build id / surface count
  not matching the `.dig`: the building is drawn full bright (one white
  texel) and logged (note: edge cases). A surface whose map offset is
  `0xFFFFFFFF` and whose material is 255 is **not drawn** (Q67 c); any
  other surface without a map gets the white texel (stand-in; none in
  the data).
- `0x1000` building materials use the light map like the others (open
  question 7, stand-in).
- `.dil` reader: the closing u32 is a counted byte list (Q67); read the
  count and skip that many bytes.
- Log (`viewer.log`): per building, the `.dil` used, surfaces lit / with
  no map / not drawn, the light image size; Connor's hut's red histogram
  (must match 5a: 0 ×16,017, 3 ×7,775, 4 ×7,344).
- Unit tests: the packing (maps don't overlap, all inside the image),
  the corner coordinate rule (texel centres land on map texels; the far
  edge of a surface stays inside its map).

**Done 2026-10-15** (MODLOG): every building in the 10 worlds tried is
lit by its own map (no fallbacks, no size-rule misses); Connor's hut
matches 5a; Castle D's 96 material-255 surfaces are left out. Checked
by eye by the user (2026-10-15).

#### M1 step 6 — haze (distance fog) on models (planned 2026-10-15)

Rules from `docs/behaviour/object-lighting.md` rules 19–23 (and
`objects.md` "See-through objects" rule 1, `models-and-textures.md` flag
`0x10000`). Asked by the user, with a key to switch it on and off.

- Data: `TERRAIN.KQ` `hazeDistance`, `visibleDistance` (no terrain:
  20,000 and 25,500); `world.cs` `resetHazeColor R G B` (0–255, the
  note's assumption; last line wins); objects' `b_hazeSolid` (61 files)
  and `hazeMultiplier` (default 1); material flag `0x10000`.
- Per object, each frame, on the graphics card (in `object_light.wesl`):
  distance from the camera to the object's position (one value per
  object: the drawn part's own origin, which is the object's position for
  every model we draw); haze = (distance − hazeDistance) ÷ (end −
  hazeDistance), capped at 1, 0.01 when closer than hazeDistance; end =
  0.85 × visibleDistance unless that is not beyond hazeDistance
  (rule 19). `0x10000` halves it (rule 20). Haze 1 and not `b_hazeSolid`:
  not drawn (rule 21). Then × `hazeMultiplier` (rule 22). Solid: blended
  towards the haze colour by the haze (plain blend on screen colours,
  the note's assumption); see-through: no colour, opacity × (1 while
  haze < 0.5, else 1.2 − haze) (rule 23).
- Objects with their own `b_hazeSolid` or `hazeMultiplier` get their own
  copy of their materials (as see-through objects do).
- **F8** switches the haze on and off (free in `Connor.sae`,
  `Camera.sae` and the viewer); `--haze on|off` sets the start (default
  on). On-screen line "Haze: on / off (F8)". Logged.
- **Not hazed (stand-ins until Q68):** the ground and the buildings (the
  note covers objects only; the data's `b_hazeSolid` on 7 buildings
  hints that buildings are hazed like objects). Particles (falling
  leaves) are hazed each at its own position, like an object (stand-in,
  Q68). The data's misspelled `hazeMulitplier=2.0` (13 Gnome objects)
  is not read; the note names `hazeMultiplier` (Q68).
- Log: the haze settings at start; every second, how many placed objects
  are past the haze end from the camera (not drawn) and the nearest
  distance at which one is left out (the note's "How to check": expected
  22,950 in Daventry).
- Unit tests: the end rule, the `resetHazeColor` line, the CPU copy of
  the haze formula (start, middle, end, halving, solid, multiplier,
  see-through fade).

**Done 2026-10-15** (MODLOG): haze from the data in 7 worlds (Daventry
full at 22,950, Dead City at 10,200, as the note); from Connor's start
286 of 313 Daventry objects are left out, the nearest at 23,427. Doors
with `b_hazeSolid` get their own material copies too. Not checked by eye
yet. Ground and buildings wait for Q68.

**User's check 2026-10-15:** objects vanish in the distance, but "there
is no actual haze": in the original a very dense, dark fog hides
**everything** (ground and buildings too). Added to Q68.

**6b. Fog over the ground and buildings** (stand-in until Q68)

- The ground (everything drawn with Bevy's standard material: surface,
  floor view, flowing squares) gets Bevy's distance fog on every 3D
  camera: linear from `hazeDistance` to the haze end, by each point's
  distance from the camera, colour = the haze colour.
- Building walls get the same in `building_light.wesl` (per point, same
  distances and colour).
- The background (the clear colour) is the haze colour while the haze is
  on, so the far distance is fog, not grey.
- To match the ground, the objects' haze blend changes to the same blend
  as Bevy's fog (on Bevy's linear colours, not screen colours). Still
  one value per object, and objects are still left out at full haze.
- F8 switches all of it (fog on the cameras, building and object
  materials, background).
- All of this is a **stand-in** from the user's observation, not from
  the note; Q68 asks how the original does it.

**Done 2026-10-15** (MODLOG): checked by eye by the user. **Replaced by
6d–6f** (2026-10-16).

**6c. Draw distance for objects** (stand-in until Q69). The user saw
Dead City's torch flames shown far past the fog; in the original they
vanish at about 11,000 from Connor. By the note's rules they never
vanish (see-through, and their material halves the haze). Stand-in: no
object is drawn farther than the world's `visibleDistance` from the
camera (Dead City 12,000, Daventry 27,000), while the haze is on
(`Haze::beyond_view`, `object_light.wesl`). **Done 2026-10-15**, checked
by eye by the user. **Replaced by 6h** (2026-10-16).

**6d–6h. Replace the stand-ins with A's answers to Q68 and Q69** (planned
2026-10-16). Rules: `object-lighting.md` rules 19–23, 27–40. One step at
a time, each logged and tested on its own. F8 still switches all the
haze off (and with it the ground's edge and the switch-off by distance,
so the whole world can be seen for comparison).

- **6d. Haze colour and background from the palette; plain blend.**
  - The haze colour is the palette colour nearest to `resetHazeColor`
    (smallest sum of squared differences over entries 10–244, the first
    one on a tie; rule 23, open question 8 stand-in: all of 10–244 count
    as usable). The palette is `KQWorld::loadingPalette` from `world.cs`
    (the one `loadPalette` loads just before `resetHazeColor`), else
    `<world>.ppl`.
  - The background (clear colour) is palette entry `KQWorld::background`
    (`newSky $KQWorld::background`, rule 31), with or without the haze.
  - Objects and walls blend towards the haze colour on **screen colours**
    (rule 23), not Bevy's linear colours: colour → screen, mix, → back.
  - Log at load: haze colour index and RGB, background index and RGB.
  - Unit tests: nearest-colour pick, `KQWorld::background` line.
- **6e. The ground's own haze.** The ground (surface, floor view, flowing
  squares) gets its own small material (`GroundMaterial`, texture ×
  the light map's vertex colour, as Bevy's unlit material did), so Bevy's
  distance fog is no longer used anywhere.
  - Per corner (rule 27): 0.01 below `hazeDistance`; 0.01 + (d −
    hazeDistance) ÷ (1.02 × (visibleDistance − hazeDistance)) up to
    `visibleDistance`; 0.99 beyond. Blended across each triangle by the
    card; plain screen-colour blend.
  - Not drawn beyond `visibleDistance` from the camera (rule 28): a point
    whose distance is past it is left out, so triangles that cross it are
    cut there (open question 10: the original's exact cut was not read).
  - Log: the ground's haze numbers at load.
- **6f. Walls hazed per surface.** Each building surface's haze comes
  from its **nearest corner** to the camera, with the objects' numbers
  (end = 0.85 × visibleDistance), 0.01 nearer than `hazeDistance`, 0.99
  when the formula passes 1; at 0.99 the surface is left out unless the
  building's `.kq` has `b_hazeSolid=1` (rule 29). Done on the graphics
  card: each building's surface corners go into one list on the card,
  each wall corner carries its surface's place in that list, and the
  vertex shader finds the nearest corner. `0x10000` and `hazeMultiplier`
  are not used for walls. (Point lights on walls, rule 16: not done.)
  - Log every second: how many wall surfaces are left out by the haze.
- **6g. Particles: one haze per emitter** (rule 30), from the emitter's
  location (which moves for following emitters; snow follows the camera
  so it gets 0.01). Each emitter gets its own material copies whose
  "haze from here" point is set every frame.
- **6h. Near / far / off by distance from Connor** (rules 34–40),
  replacing 6c's "nothing beyond `visibleDistance` from the camera".
  - Every frame, for every placed object, building, door, chest,
    scripted object, animated object, item and emitter: D = distance from
    Connor to the centre of its bounding ball − the ball's radius.
    Off when D ≥ `visibleDistance`; far when D > `tooFar` ×
    `visibleDistance` (`tooFar` from its `.kq`, default 0.83); else near.
  - Ball (rule 35): the shape's box (centre = box middle turned with the
    object, radius = half the diagonal); no shape: radius 100 at the
    position; emitters: 250 round the emitter's location. "The collider's
    ball" for objects with a box collider: **stand-in**, the same ball
    from the shape's box (asked as Q70).
  - Drawn (rule 37) when the active bits have any of 0x10, 0x40, 0x80,
    0x100: near = `groupsAlways` | `groupsActive3D` (default 0x448), far
    = `groupsAlways` | `groupsActive3DFar` (default 0x440), off =
    `groupsAlways`. Read `tooFar`, `groupsActive3D`, `groupsActive3DFar`
    from the `.kq`. A hidden object keeps its own rule (`objects.md`).
  - How it hides: an object that is off by distance is moved to a
    drawing layer the camera doesn't show (Bevy's `RenderLayers`), so it
    does not fight the systems that already show and hide objects
    (hidden objects, items picked up, particles). Emitters: their
    particle material is switched off.
  - Not done (logged): the collider switch (0x8; far objects are at
    least `tooFar` × `visibleDistance` from Connor, so he can't touch
    them anyway), `makeActive`, `setActiveCamera` (cut-scene scripts).
    With no Connor (fly camera), the camera is the reference point
    (viewer only).
  - Log every second: how many objects are near / far / off; for Dead
    City's flames, the distance from Connor at which each switches off
    (rule 38: 12,100; the user's ~11,000 is open question 9).

**6d–6h done 2026-10-16** (MODLOG, one entry each), from the logs;
checked by eye by the user the same day (matches the original). Differences from the plan: the walls' corner list is
a storage buffer read in the vertex shader (`BuildingMaterial::specialize`
adds the surface attribute); the ground's material is `ground_haze.rs`;
the draw distance is `draw_distance.rs` and uses each object's own
transform (all tested entities are roots), so it is right on the frame
an object appears; it also logs objects that are near or far but not
drawn (8 Daventry props whose files set `groupsActive3DFar=0`). Items use
the default masks (stand-in). Open: Q70 (the collider's ball).

#### M1 step 7 — draw distance answers and point lights (planned 2026-10-16)

From A's answers to Q70–Q74 (`object-lighting.md` rules 14, 16, 24–26b,
35, 37, 41–43; `items.md` 5b; `terrain-lighting.md` 15). Order as STATUS
asks: the draw-distance fixes first (small), then lights on models (what
the user saw on Connor), then walls, then the ground. Each sub-step gets
its own MODLOG entry; the lighting sub-steps are planned in detail here
before they start.

- **7a (done 2026-10-16). The draw-distance ball and items** (Q70, Q71; rules 35, 43): the
  ball always comes from a `BOUNDS.KQ` box: the collision box for objects
  that have one (animated: current movement's shape, then own name),
  else by `shapeName` then own name; none: radius 100 at the position.
  Worked out once, when the object enters. Centre turned with the object
  (our choice, "Do not reproduce"). Items: `tooFar` 0.83, near 0x448,
  **far 0**: a far item is neither drawn nor clickable. Log: per item
  near / far / off counts.
- **7b (done 2026-10-16). The collider switch, `setActiveCamera`, `makeActive`** (Q74;
  rules 36, 41, 42): when an object's state changes, losing bit 0x8 takes
  its collider out (Connor, the camera and clicks pass through), gaining
  it puts it back. `setActiveCamera 0|1` (reference point Connor or the
  camera) and `makeActive <name> 0|1` (always near) as script commands,
  effective from the next frame. Log every switch.
- **7c (done 2026-10-16). Lights on models** (rules 24, 26b, 3, 37): `KQPointLight`s that
  the world's scripts load (Dead City seven at the start) and the lights
  carried by models (`.kqs` light nodes of the current frame, at most 5,
  placed by the object's placement only), each while its owner is in the
  lights group (0x400). Per model part: out of range when (distance to
  the part's ball) − radius > End; facing from the light to the ball's
  centre; falloff Begin → End. On the graphics card, as the sun is now.
  Three sub-steps (planned 2026-10-16):
  - **7c-1 (done 2026-10-16), the data.** `kq8-data` `shape.rs`: the lights of a sequence
    (its light sub-sequence's nodes, at most 5): kind (flag bit → point or
    sun-type), colour = byte ÷ 255 × scale × level × 0.2, capped at 1
    (versions 1–3: level and scale 1.0), position, Begin, End. The viewer
    keeps them per drawn frame (`Animation::frame_lights`, and for still
    models their shown frame). `KQPointLight` objects that world scripts
    `loadKQ` (`light/Plight01.kq`) are found where the sun files are found
    (`data/<World>/KQ/`) and kept by name (deletable, near / far / off).
    Logs: per world, how many point lights load and how many drawn shapes
    carry lights; `portlite.kqs`' 17 frames (position, colour) as a check
    against A's numbers (rule 26b example). Unit tests on a made-up shape.
  - **7c-2 (done 2026-10-16), gathering.** Once per frame (rule 3): every `KQPointLight` and
    every light carried by a shown object, each only while its owner is in
    the lights group (bit 0x400: near or far with the default masks);
    point positions put through the owner's placement only, sun-type aims
    as stored. Into one list on the graphics card (a storage buffer
    shared by every object material, fixed size, contents rewritten each
    frame). Log once a second: how many lights, the nearest to Connor.
  - **7c-3 (done 2026-10-16), the shader.** Each vertex carries its model part's ball
    (centre in model space = middle of the part's vertices in that frame,
    radius = the part's mesh radius; rule 24's stand-in for the centre).
    Per light: part out of range when (distance to the ball centre) −
    radius > End; facing = max(0, −(unit direction light→centre ·
    normal)); falloff from the corner's own distance: 1 to Begin, linear
    to 0 at End; colour × falloff × facing added before the cap (rule 4).
    Sun-type model lights add like suns (rule 5). Flat and none as now.
- **7d. Point lights on walls** (rule 16): when dynamic lighting is on,
  per wall surface per light: in front, switched on (`turnOffDynLighting`
  off), nearest corner/centre within Begin + End, line not blocked; then
  added to the light map per texel with the wall falloff (End = fade
  length), and the surface's haze × 0.04.
- **7e. Point lights on the ground** (rule 26): patches painted into the
  shown map every frame, put back the next.
- **7f. Q73 leftovers**: light maps bilinear, no colour key on walls or
  ground, the ground's far-edge cut at t = (visibleDistance − d₁) ÷
  (d₂ − d₁) (rule 28); check each against our code and change what
  differs. **Checked 2026-10-16:** walls' light maps are sampled
  bilinear ✓; walls and ground are opaque, texture alpha ignored, so no
  colour key ✓; the ground's cut (corner distances blended across the
  triangle, cut at `visibleDistance`) is the same cut as rule 28 ✓
  (comment updated, no code change). **Differs:** the ground's light map
  is blended per triangle (vertex colours), not bilinear per square
  (`terrain-lighting.md` rule 11, `object-lighting.md` rule 13). Planned
  as **7e-0**: the ground's light map as a texture looked up in world
  coordinates with bilinear filtering; 7e (point-light patches painted
  into the shown map each frame) then works on that texture.
- **7e-0. The ground's light map, bilinear (planned and done 2026-10-16, MODLOG).**
  `terrain-lighting.md` rules 7, 10, 11; `object-lighting.md` rule 13.
  - The shown map becomes one picture on the graphics card: one texel per
    map point (Daventry 513 × 513), red/green/blue = channel × 17 (so
    channel ÷ 15 when read), not sRGB, bilinear filtering, clamped at the
    edges (outside the map: the nearest edge point, as `colour_at`).
  - The ground shader looks it up per pixel from the world position:
    u = (x ÷ spacing + 0.5) ÷ side, v = (y ÷ spacing + 0.5) ÷ side (map
    point (i, j) at x = i × spacing, y = j × spacing, rule 7). Then the
    mode (×1 / ×2 / off, F7) and the screen-colour stand-in as before.
  - The vertex colours go; F7 changes a setting in every ground material
    instead of recolouring the meshes. The meshes stay cut into map cells
    (harmless; may be simplified later).
  - Check: unit test of the picture's texels and the lookup formula
    against `LightMap::colour_at`; viewer runs (no shader errors); the
    once-a-second "ground light under Connor" log line is unchanged (it
    already uses the bilinear `colour_at`). By eye: soft shadow edges,
    no diagonal steps along the cells' triangles.
  - 7e then paints its patches into this picture.

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

**Where the code goes.** A new crate `crates/kq8-core` (no Bevy) holds the
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
    standing, walking or running. Banking came later (2i).
  - No start/stop animations (`stopwalk`, `walkstopleft`...) yet: he goes
    straight from one movement to the next, starting the loop at its
    beginning. Which transition plays is asked as Q25. (Replaced by 2g:
    starts in 2g-1, stops in 2g-2.)
  - All frames of each movement's loop are drawn in turn, at the loop's
    speed.
  - The rules live in `kq8-core` (`connor.rs`): intent → movement, turning,
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
  - The mover's update (`kq8-core` `connor_mover.rs`):
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
    (`objects.md`, loop rule 2: `frameCount − 1` steps) (**corrected in 2d-4:** over the stored
    loop length, which already runs from the first frame to the last). The walker hears
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
  Q33). Done 2026-10-12: 2d-1 to 2d-5, 2d-3b and 2d-3c, all checked by eye
  by the user.** Every rule below is traced or read from the data; the
  stand-ins added on the way are marked in their sub-step (Q39, starting
  inside a box, the `#` comment after `colliderType`). Rules from
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
  - **2d-2, the collision tests (`kq8-core`, new `collide.rs`; still not
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
  - **2d-3b, walk held against a wall.** *Done 2026-10-12 (MODLOG).*
    The user
    saw Connor frozen on the walk's first frame while forward was held
    against a wall: each update the keys restart `walk` at frame 0 and
    the head-on stop puts `stop` back at frame 0. **The user checked the
    original:** he tries to move, stops in the idle animation (`stop`) as
    soon as the wall is hit, and doesn't try again until the forward key
    is pressed again (A measured 2.8 s of `Stop` with the key held, Q37
    d; the user's check says it lasts until the key is pressed again).
    - The walker remembers the key direction (forward or back) that was
      stopped. While that direction is still what the keys ask for, it
      counts as "not moving forward or back": with no turn key he stands
      in `stop` and its animation plays on; with a turn key he turns on
      the spot (our choice, not checked in the original).
    - Pressing that key again (a new press, not a held one) clears it,
      and so does any mode change (falling, the test lifts). Releasing the
      key alone doesn't clear it, but then nothing asks for that
      direction anyway. Pressing the other direction moves him normally;
      the stopped direction stays blocked until it is pressed again.
    - Running is not blocked (2d-4: held run runs again after the crash).
    - Log: an event line when the block starts and when a key press
      clears it; `connor.log` gets a column `blocked` (`none`, `fwd`,
      `back`).
    - Unit tests: walk into a wall, keep forward held for 45 updates:
      `stop` all the time with its phase moving on; release and press
      forward: walks again. Back works the same way.
  - **2d-3c, the wall block as traced.** *Done 2026-10-12 (MODLOG).*
    Session A
    traced why a held walk key doesn't restart walking (`connor-physics.md`
    "Walk held against a wall", Q38). It replaces 2d-3b's rules where
    they differ:
    - Two blocks, forward and back, both off at the start. A head-on
      stop that plays `stop` sets back blocked if the movement was `back`,
      otherwise forward blocked (running crashes and sets none).
    - While either block is on, he stands in `stop` whatever the keys
      ask: no walking, running, stepping back **or turning on the spot**
      (2d-3b let him run and turn).
    - Only key events clear blocks: forward released → forward; back
      released → back; forward pressed → back; back pressed → forward;
      left or right pressed → both. Releasing a turn key clears nothing.
      Falling, landing and the test lift no longer clear them (2d-3b did);
      `--at` still starts a fresh Connor.
    - So the user's check still holds (release and press forward: walks
      again), and pressing a turn key against the wall makes him walk
      again, turning as he goes, instead of turning on the spot.
    - Not done: the unknown Connor flag that clears both blocks on hits.
      Keyboard auto-repeat: Bevy reports a held key as pressed once, so a
      held turn key clears the blocks once (A's open point).
    - Log: column `blocked` (`none`, `fwd`, `back`, `both`); event lines
      when a key clears a block.
    - Unit tests: the clearing table; standing still with run or a turn
      key held while blocked; a block survives a mode change.
  - **2d-4, crash when running head-on.** *Done 2026-10-12 (MODLOG).*
    Running (`run`; the note's
    other run-like movements don't exist in our code yet) head-on into a
    wall: `crash` plays **in full** (traced, Q31 e). Its own 7 steps move
    him 441.5 straight back (63 per update; read from the data, matches
    the measurement), taken by the mover like any other step, so a wall
    behind him still stops it. Keys are ignored while it plays; then he
    stands, and with run still held he runs again (the original plays
    `stoprun` first; done since 2g-1: `stop`, `stoprun`, `run`) and can crash
    again, as measured. Not done: the head-on exceptions that need things
    we don't have yet (`KQMonster`, `backJump`, the unknown Connor flag,
    the "no crash" flags of open question 4). Unit test: run into a wall,
    `crash` for 7 updates, 441.5 back, then `run`.
    - How (the user's choice, 2026-10-12): a new walker mode `Crash`,
      like `Land`: `crash` plays once, its steps go to the mover, the
      keys are ignored, then the keys take over again. The head-on hit
      is found during an update; `crash` starts with the next one (as
      `Land` does). While it plays he is not in the Move state, so a wall
      behind him is never head-on: he slides or stops at it.
    - **Fix found on the way: how long a play-once animation lasts.** The
      walker took a one-shot's stored length to cover all its frames and
      played it for (frames − 1) / frames of it, moving that share of its
      steps. The data says the stored length already runs from the first
      frame to the last: `crash` 8 frames 0.4667 s (7/15), `jumpend` 11
      frames 0.6667 s (10/15), even `walk` 20 frames 1.2667 s (19/15). A
      measurement agrees: `crash` moved him 441 in 7 ticks of 63, the
      whole step sum (441.5) in 7/15 s. So a one-shot now lasts its stored
      length and moves its whole step sum. Side effects: `jumpend` and
      `jumpdie` last exactly 10 updates (before: 9.1, shown as 10), and
      `jumpend`'s own steps are given in full (before: 10/11).
    - Log: event lines for the head-on crash and its end; `mode` column
      `crash`.
  - **2d-5, objects.** *Done 2026-10-12 (MODLOG), checked by eye by the user.
    Changes from this plan: no `ObjectCollider` wrapper (the box
    collider got its own rough check, using the box enlarged by r and
    turned into the world, so a turned box's corner is never skipped);
    shape names are lower-cased before the lookup (stand-in, Q39);
    `colliderType=97  #World, Click, Attack` (Barren, IceWorld) is read as
    97 (our reading: what follows the number is a comment). No start-up
    script in the 12 worlds uses `collisionType` / `collision`, so they
    aren't run.*
    - How (the user's choice, 2026-10-12): a second list, of object
      boxes, next to the building list in the mover's scene; both are
      swept and the nearest hit kept. Buildings stay as they are. To
      revisit if monsters (ball stacks) make a third list awkward.
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
    - **Detailed plan (2026-10-12), with the Q34 answers:**
      - `kq8-data` `bounds.rs`: a lookup that follows the game: only keys
        `item0` … `item<nItems − 1>`; names compared with case; inside-out
        boxes fixed (smaller number = min on each axis); for the two real
        duplicates the entries the note names (`rat` = item 8,
        `razetta.kqs` = item 775). The raw list stays for
        `building-verify`.
      - `kq8-core` `collide.rs`: `ObjectCollider` (the box collider plus
        its world box for the rough check); Connor's type 4 and mask
        0x281B; the pair test.
      - `connor_mover.rs`: the scene gets an object list next to the buildings;
        every sweep tests both and keeps the contact nearest the old
        position (step 7). The rest of the hit handling is unchanged, so
        objects stop, glance, carry him (floors) and block like
        buildings. A ball that **starts inside** an object's box has no
        usable contact (not traced what follows): we treat it as no hit,
        so he can walk out, and log it.
      - Viewer `load.rs`: every object the start-up scripts load and we
        place, apart from buildings, after `move` and `setHidden` (hidden
        ones too). Objects we can't place (no model, `.anm` characters,
        a model that fails to load) get no collider; they are counted in
        the log. `KQMonster`s are ball stacks, and Connor against ball
        stacks is not traced: none in M2 (they aren't drawn yet).
      - The script commands `collisionType` / `collision`: applied only
        if a start-up script uses them (checked over all 12 worlds).
      - Log (`viewer.log`): counts of objects with a box (by class),
        without one, with type and mask 0, not tested against Connor by
        the pair rule; names that match the table only when case is
        ignored; every object collider in detail.
      - Check by eye: the signpost, trees, the stump near the wizard,
        closed doors, DeadCity's crates.
  - **Still to check or not traced** (none blocks 2d; each is logged
    where it happens):
    - **25 above floors: confirmed** (2026-10-12, Q35). In the original
      his feet are at 13,980 in Connor's house; the house's floor in our
      data is at 13,955.
    - **225 from a wall: confirmed** (2026-10-12, Q36, Q37). The sweep
      at the Tomb wall gives the measured contacts to within 0.1 unit,
      and so does the viewer with Connor's real movement (2d-3).
    - A ball that starts inside an object's box: "a hit with no usable
      contact"; what follows was not traced. We count it as no hit (he
      can walk out) and log it (2d-5).
    - ~~The inside-out `lfswdoor.kqs` box: not traced~~ **Answered (Q34
      c):** inside-out boxes are fixed on loading. Swamp's `Lfswdoor`
      uses it (2d-5 log).
    - Whether a shape's name is lower-cased before the box lookup: our
      stand-in says yes (Q39, open).
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
- **2e — jumping (planned 2026-10-12; 2e-1 to 2e-3 done 2026-10-12,
  all three checked by eye by the user).** Rules from `connor-physics.md`
  "Jumping" (traced; arcs measured), "Falling off an edge" rules 2–5,
  "Walls"; the user's checks in the original (2026-10-12, below). Three
  sub-steps, each stopped for a check.
  - **The user's checks in the original:** no jump while a start or stop
    animation plays; a jump only from standing (standing jump), full
    walking (standing jump) or full running (running jump). No turning
    during a jump or its fall; the only turning is during the landing of
    a running jump (forward let go before he lands, a turn key held).
    The running jump's measured 6,480 vs 6,356 in the data: probably not
    flat ground; no need to match. Turning on the spot, then Jump: the
    jump happens, turning stops, and starts again after the landing
    animation. A back flip into a wall or steep hill: he slides down in
    the backward falling animation until he reaches the ground (death
    rule as usual).
  - **Where the code goes.** The update that ties the walker and the
    mover together (fall events, head-on, and now jumps) moves from the
    viewer (`connor_view.rs` `connor_step`) into `kq8-core` (new
    `connor_update.rs`), so jumps can be unit-tested with both. The viewer keeps
    the keys, drawing and logging. A small move of existing code, no
    change in behaviour.
  - **2e-1, the standing jump** (`jump`, `C_jump.kqs`): *Done 2026-10-12
    (MODLOG), checked by eye by the user (also from turning on the spot,
    as in the original). Added on the way: `--jump N` test
    option.*
    - The Jump key (`Connor.sae` action `Jump`, Insert) asks for a jump at
      the next update. It is done only if (a) he is in the keys' control
      (not falling, landing, crashing or dead), no start or stop animation
      plays (user's check), and his movement is `stop`, `walk`, `run`,
      `back`, `left` or `right`; (b) he is on the ground, or within 250
      above the terrain, whose normal then counts; (c) that ground is
      walkable (normal z > 0.8), or steep but its normal doesn't point
      toward his back (not facing uphill). Otherwise it is refused and
      logged with the reason. Which jump: `runJump` from `run`,
      `backJump` from `back`, otherwise `jump`. In 2e-1 only `jump` is
      loaded, so a jump from running or stepping back is refused as "not
      loaded yet".
    - Jumping (new mover state Jump, walker mode Jump): gravity off, step
      up and down 0; the arc is the animation's own frame steps (x, y and
      z), frame by frame (as in 2g). Keys ignored, no turning. Walls,
      ceilings and the ground check still act; a hit during the jump is
      handled as glancing until 2e-3.
    - The update after the jump animation ends: gravity on, steps 325,
      the Fall state with the start height 1,280 below where he is (so
      death if he lands more than 5,120 lower), fall timer so that the
      first drop is from 0.30 to 0.36 s (396). Then `jumphold`, and
      `jumpend` or `jumpdie` as for a walk-off. No Fall request from the
      air counter during a jump (gravity is off, so it stays 0).
    - Expected on flat ground (note): 20 updates of `jump` (1.333 s),
      highest about 1,697 above the start, at the end 873 above and 1,423
      forward; then `jumphold` until he lands, then `jumpend`.
  - **2e-2, running and backward jumps.** *Done 2026-10-12 (MODLOG),
    checked by eye by the user. Added on the way (from the user's check): at a
    running jump's landing a turn key alone counts as standing, so
    `jumpstop` plays and he turns during it (Q41 d).* Load `runJump`, `runJumpHold`,
    `runJumpDie`, `jumprun`, `jumpstop`, `backJump`, `backJumpHold`,
    `backJumpEnd`, `backJumpDie`. Hold, landing and death animations by
    the jump's kind. `runJump` has no landing animation: on landing he is
    in the keys' control at once, still in `runJumpHold`, and the keys ask
    for a movement from there, so the transitions in `Conner.anm` (2g
    rules) give `jumprun` → `run` at 0.8 with run held, `jumpstop` →
    `stop` with forward let go (turning allowed, as the user saw), and
    `walk` at once with run off. Expected: `runJump` about 6,356 forward.
  - **2e-3, hitting something during a jump.** *Done 2026-10-12
    (MODLOG), checked by eye by the user. Corrected after the user's check:
    the back flip first ended at the hit (Fall at once); it now plays on
    to its end.* A steep hit (normal z <
    0.8) ends the jump: gravity on, steps 325. Down pass during `jump` or
    `runJump`: `crash`. Up or across pass: `runJump` crashes, `jump`
    stops (`stop`, standing). During a jump any surface with normal z <
    0.8 counts as head-on, and head-on acts in Jump as in Move.
    `backJump` does neither: the user saw the back flip keep going to its
    end, then `backJumpHold` until he reaches the ground, with no crash or
    stumble. So for `backJump` the hit is glancing, gravity comes on and
    steps go back to 325 (rule 5), the animation plays on (state still
    Jump, so no Fall request), then Fall as at any jump's end.
  - **Log** (`connor.log`): the mover state `jump`; event lines for a jump
    asked for, done or refused (and why), its end, the Fall it starts
    (start height, timer), the landing; the height above the jump's start.
  - **Unit tests** (made-up flat ground, the note's arc numbers): the
    standing jump's 20 updates, highest point and end point, then Fall
    with drops 396, 468, …, `jumpend`; death limit 5,120; each refusal
    (falling, during `stopwalk`, steep ground facing uphill, 300 above
    the ground); no turning while jumping.
- **2f — the following camera (planned 2026-10-11).** Moved up at the
  user's request: done right after 2b, before 2c. Rules from `camera.md`.
  - The camera logic runs in the same 15-updates-per-second schedule as
    Connor, right after him, so every per-update rate in the note (height
    10% per update, tilt 0.015 rad per update, zoom 125 per update) is used
    as traced; drawing blends between its last two updates, like Connor.
  - Rules in `kq8-core` (`camera.rs`), unit-tested: aim point 1,400 above
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
- **2g — start and stop transitions (planned 2026-10-12; done
  2026-10-12, 2g-1 and 2g-2 checked by eye by the user).** Rules from
  `connor-movement.md` "Starting and stopping" (Q25, rules 1–6, the two
  `loop=` flags, "Steps in the start animations"); walls from
  `connor-physics.md` ("Walls", "Walk held against a wall"). Two sub-steps,
  each stopped for a check.
  - **Words used here.** *Phase*: how far through its animation a
    movement is, 0 at the start, 1 at the end. A *transition* is a short
    animation played between two movements (`stopwalk` between `stop`
    and `walk`). A *one-shot* animation plays once and stops on its last
    frame; a *loop* repeats.
  - **The data** (`kq8-data` `objects.rs`, read for every movement):
    `transitionN=<from phase> <to movement> <to phase> <transition
    movement>` lines (`nTransition` of them), and the 6th `loop=` field,
    "can be cut short". The viewer keeps a transition only when both its
    `<to movement>` and its transition movement are loaded; the others are
    logged as "not loaded" and skipped (for now: water, and in 2g-1 the
    stop animations).
  - **Per-frame steps** for one-shot animations: the frames' own steps
    (frame k's step is the move from frame k−1 to k, over one frame time,
    the stored length / (frames − 1)), applied for the part of each frame
    the update covers, on both axes (x = his left, −y = forward). Loops
    (`walk`, `run`, …) keep the even spread (Q24 is only partly
    answered). This also changes `jumpend` (its steps are uneven); `crash`
    is even, so it doesn't change.
  - **The walker** (`kq8-core` `connor.rs`) gains, following rules 1–6:
    the *wanted* movement (asked for only when the keys' choice changes;
    a new request replaces the old one); a *chosen* transition waiting
    for its from point; the transition being played and its `<to
    movement>` / `<to phase>`.
    - Choosing (rule 2): of the current movement's transitions to the
      wanted one, the smallest distance `from − phase` (+1 if negative and
      the movement loops; never chosen if negative on a one-shot); ties:
      first listed; from −1 always wins on a loop.
    - Waiting (rule 3): the movement plays on; in the update where its
      phase reaches the from point it stops exactly there (only that part
      of the step), and the transition starts at phase 0. **Our reading:**
      the transition plays from the next update, so from −1 gives one
      update of `stop` with no movement, then `stopwalk`. The same reading
      gives the measured "one update of `run`, then `stoprunstop`" in 2g-2.
      Asked as Q40 (a).
    - No transition (rule 4): switch at once from phase 0 if the current
      movement can be cut short; otherwise keep the wanted one for the
      end.
    - The end (rule 6): `<to movement>` at `<to phase>`; if the wanted
      movement is now different, choose again from there at once.
    - Turning works throughout (it doesn't depend on the animation).
    - Walls: head-on during `stopwalk` is handled as for `walk` (`stop`,
      forward blocked); during `stoprun` as for `run` (`crash`). Either
      clears the transition state.
    - Falling and the test lifts clear the transition state. After a
      mode's one-shot (`crash`, `jumpend`) ends he is in `stop`, and the
      keys ask again from there, so with run held after a crash `stoprun`
      plays before `run` (as the original does, DESIGN 2d-4). For `jumpend`
      this is our choice, not traced.
  - **2g-1, starting.** *Done 2026-10-12 (MODLOG), checked by eye by
    the user, including the original's quirk that holding a turn key and
    then pressing forward starts at once with no start animation (`left`
    and `right` have no transitions and can be cut short, rule 4). Found on the way: `stoprun` adds up to 526.7 forward in the data,
    not the note's 536.7 (Q40 c).* Load `stopwalk` (`c_stpwk.kqs`) and `stoprun`
    (`c_stprn.kqs`). Expected (the note): `stopwalk` 7 updates, 51.2 left
    and 78.5 forward in all (7 × (7.32, 11.22)), then `walk` from phase
    0.85; `stoprun` 10 updates, 43.9 left and 536.7 forward with the
    note's uneven steps, then `run` from 0.867. Let go during `stopwalk`:
    it plays to its end, `walk` at 0.85, then `stop` at once (no stop
    animations loaded yet). Run switched on during `stopwalk`: `run` at
    once from 0 when it ends.
  - **2g-2, stopping.** *Done 2026-10-12 (MODLOG), checked by eye by
    the user. Added on the way: `--hold F,T,R,N` lets go after N updates, so
    a stop can be logged.* Load `walkstopleft`, `stopwalkstop`,
    `walkstopright`, `runstopright`, `runstopleft`, `stoprunstop`.
    Expected: letting go during `stoprun` gives one update of `run`, then
    `stoprunstop`; letting go while walking waits for the next from point
    (0.45, 0.90 or 0.95), then plays that point's animation.
    - State: from the moment `stop` is asked for, he is in Wait, not Move
      (the note's measurement: "state already Wait" while the walk or run
      plays on). So a wall hit while a stop waits or plays is never
      head-on: he slides or stops at the contact, with no `crash` or wall
      block. In code: "moving" follows the keys' choice, not the movement
      playing.
    - Pressing forward again while a stop waits: walking carries on (our
      reading, Q40 b). Pressed while a stop animation plays: it plays to
      its end, then `stopwalk` (rules 4 and 6).
    - Unit tests: each of the three walk stops and the run stops picked by
      the phase at release; letting go during `stopwalk` (walk at 0.85,
      then `stopwalkstop` about one update later) and during `stoprun`
      (one update of `run` with no step, then `stoprunstop`); forward
      pressed again while waiting; no head-on while stopping.
  - **Log** (`connor.log`, new columns): phase at the start of the update,
    the wanted movement, the step applied in his own axes (left,
    forward). Event lines: a request, the transition chosen (from, to,
    via, distance), its start and its end.
  - **Unit tests:** the data reader; choosing (next ahead, wrap, −1, ties,
    one-shot never wraps); `stopwalk` 7 updates and its totals, then walk
    at 0.85; `stoprun` 10 updates and its totals, then run at 0.867;
    letting go during `stopwalk`; run switched on during `stopwalk`;
    head-on during `stopwalk` and `stoprun`; turning during a start; after
    a crash with run held: `stop`, `stoprun`, `run`.

- **2h — water: the floor grid and wading (planned 2026-10-12; 2h-1 and
  2h-2 done 2026-10-12, MODLOG; not checked by eye yet).** Rules
  from `docs/behaviour/water.md` (Q43). Two steps, each tested and
  checked on its own.
  - **2h-1, the floor grid.** Worlds with a `floorBMPName` line
    (Daventry, Swamp, IceWorld, DeadCity) get a second height grid from
    that image (`terrh2.dib`), built like the drawn one. "Floor" questions
    use it; in a world without one they use the drawn grid (the
    "surface").
    - `kq8-data` `terrain.rs`: `Terrain` keeps an optional floor grid;
      `height_at` stays the surface (drawing), new `floor_at` (floor, or
      surface without one). Same triangle rule for both.
    - `kq8-data` `objects.rs`: `keepOnGround` becomes three values:
      default (−1) raise onto the **floor**, `=1` onto the **surface**,
      `=0` no raise. A `loadKQ` z of −1 = floor height, −2 = surface
      height (replaces the old "both = ground"). So Swamp's `shack2` goes
      from 6,120 to 5,355.
    - Connor's mover and the camera's "keep above the ground" use the
      floor; `--at` puts him on the floor. The ground drawn stays the
      surface (the floor is most likely never drawn, note open question
      1).
    - Log: per world, whether it has a floor grid and at how many corners
      it differs; each placement says which grid it used.
    - Expected (note "How to check"): Swamp `shack2` at 5,355; Connor
      dropped in the shack lands 765 lower than before (feet 6,468 east,
      7,332 west); Daventry objects placed with −1 on dry land unchanged.
    - Unit tests: grid choice for −1, −2, `keepOnGround` 1/0/default,
      floor fallback without a floor image.
  - **2h-2, wading.** Read each world's `kq/envInfo.kq` ground flags (the
    note's exact column rule). After every move of Connor: `submerged` =
    the square's material is liquid (water, poison, fastpoison/lava,
    swamp) and the surface ≥ feet + 600. The movement choice uses it:
    forward → `waterwalk` / `waterrun` instead of `walk` / `run`; redone
    every update while moving, a new movement asked for only when the
    chosen name changes, so the `.anm` transitions (`walkwaterwalk`,
    `runwaterrun`, `waterwalkwalk`, `waterrunrun`) play. `waterrun` and
    `runwaterrun` crash head-on like `run`. No slowing, no floating.
    - Load `waterwalk`, `waterrun` and the four transitions.
    - Log: `submerged` and the surface height in `connor.log`; an event
      line when it changes.
    - Expected: walking into the Daventry river, his feet go about 1,020
      under the drawn water; `submerged` comes on where the surface is
      ≥ 600 above his feet; the next update asks for `waterwalk`.
    - Unit tests: the envInfo line reader (stripping, the skipped first
      mark, tabs), the submerged test, the movement choice switching both
      ways, crash from `waterrun`.
  - Viewer helper (added 2026-10-12, user's request): key U cycles
    normal → both → floor only, to look at the floor grid (e.g.
    DeadCity's raised areas).
  - Not in 2h: damage from poison, lava and swamp, sinking into `ontop`
    hazards, monsters (`keepOnGround=1`), drawing the water differently.
- **2f-2 — the camera avoids walls (planned 2026-10-13; done 2026-10-13,
  MODLOG; checked in the logs and by eye).** Rules from
  `camera.md` "Obstacles" (traced 2026-10-13; three points checked by the
  user in the original). The aim height (800) and height easing (50%) the
  user set by hand stay as they are.
  - **Words.** The *view line* runs from the aim point to the camera. A
    *view blocker* is something that line may not pass through.
  - **View blockers** (built once at load, `kq8-viewer` `load.rs`):
    - the drawn ground (the surface grid, not the floor under water);
    - every building, unless its `.kq` gives a `colliderType` without bit
      0x8 or 0x1000 (buildings default to 0x28, so the default blocks);
    - objects whose collision type has bit 0x8 or 0x1000, by their
      `BOUNDS.KQ` box, turned with the object (Daventry: the doors);
      doors use their box as it stands now (`door_view`), with their
      current type. Monsters, trees and other plain objects don't block.
  - **Line tests** (`kq8-core` `collide.rs`): a building's BSP walked with
    a zero-width line (the same walk as Connor's balls, radius 0); a box
    by the slab test Connor's boxes use; the ground by stepping along the
    line every 50 units, then halving the step 20 times at the first step
    that goes from above the ground to below it (our choice: the note
    gives no method; a line that starts under the ground, e.g. the aim
    point under deep water, is not blocked until it goes back down into
    it). The nearest hit wins.
  - **The camera update** (`kq8-core` `camera.rs`), after the existing
    steps 1–5: (6) a running swing turns the camera round the aim point by
    its step, or is cancelled if the mouse moved the camera this update;
    (7) the view test; (8) if blocked: (8.1) mouse moved → back to the
    previous update's offset × 0.95 (not if that's under the minimum);
    (8.2) if avoidance is on, Connor is moving, and his feet aren't under
    the floor grid: new direction towards the point halfway between his
    aim point one update ago (A) and the hit (H), same length; start a
    swing of 0.66 × that turn per update, ending once the direction
    towards A is reached or passed; test again, and if still blocked pull
    the camera in to 100 short of the new hit (at least 1 from the aim
    point, "Do not reproduce") and cancel the swing. No height rule after
    this.
  - **"Connor is moving":** he moved (position or heading changed) in one
    of the last 3 updates, not counting the update he starts in.
  - **Not done:** updating only while Connor moves or the camera is used
    (the note's "When the camera updates at all"; our camera updates every
    tick, which only matters for the height easing; asked as a separate
    small step), switching avoidance off (nothing found that does).
  - Tests: blocked line pulls in to 100 short; swing towards the path he
    came by and its end; mouse drag into a wall creeps by 5% and never
    under the minimum; standing still nothing moves; the four line tests.
  - Log (`camera.log`): per update blocked yes/no, the hit and what hit,
    swing step and remaining angle, pulled-in distance.
- **2i — banking while running (planned 2026-10-13; done 2026-10-13,
  MODLOG; checked in the logs and by eye by the user, 2i-2 for water).** Rules from
  `connor-movement.md` "Turning" rule 3 and "Do not reproduce"
  (frame-rate-dependent banking).
  - **Words.** *Banking* (or the *lean*) is Connor tilting sideways into a
    turn, like a cyclist: a turn about his own forward axis.
  - **The rule** (`kq8-core` `connor.rs`, `Walker::bank`, radians,
    positive = leaning to his left): each update the lean moves towards a
    target by at most 0.45 rad/s × 1/15 s (= the original's 0.03 per
    update at its 15 updates per second). Target = 0.15 × the turn key
    (+1 left, −1 right) while he runs and turns, else 0. When the target
    is 0 and the lean is within 0.029 of it, it snaps to exactly 0.
  - **"Runs and turns"** (stand-ins, asked as Q56): the movement playing
    this update is `run` or `waterrun` (**2i-2**: the user saw him lean
    while wading in the original), the keys drive him (no jump, fall,
    crash or landing) and no wall block is on, and a turn key is held. Not
    the start/stop transitions (`stoprun`, `runstopleft`, `runwaterrun`...). In every other case the lean goes back to upright
    at the same rate, also during jumps and falls. Dead: left as it is.
  - **Drawing only** (`kq8-viewer` `connor_view.rs`): the lean is turned
    in **before** his heading, about his model's forward axis (−Y) through
    his feet (the model's origin; the user's check in the original), blended
    between updates like the heading. It changes nothing in collision,
    movement or the camera.
  - Tests: builds to 0.15 in 5 updates and stays there; turning the other
    way swings through 0 to −0.15 in 10; letting go of the turn key
    returns in steps of 0.03, snapping at the end; `waterrun` leans;
    walking, wading at a walk and turning on the spot never lean; a jump from a leaning run
    returns to upright during the jump.
  - Log: `connor.log` gets the lean as a new last column; `viewer.log`
    gets an event line when the lean starts building and when it is back
    at 0.
  - **2i-3, correction after Q56 (planned and done 2026-10-13).** The
    traced rule (`connor-movement.md` "Turning" rule 3) replaces the
    stand-ins: a *lean allowed* switch, set in the keys' updates only (on
    when they pick `run` or `waterrun`; with a wall block on, the pick is
    `stop`), kept in every other mode (jumps, falls, crash, landing,
    death). Each update, every mode: turning + switch on + forward held →
    one 0.03 step (0.45 rad/s) towards the turn if the lean is still short
    of 0.15 before the step, so it ends at **0.18**, on both sides (the
    original's −0.15 after swinging over is not copied, "Do not
    reproduce"); not turning or switch off → upright if within 0.029,
    else one step back; turning with the switch on but forward not held →
    unchanged. So the running start `stoprun` leans, the stops don't, and
    a jump keeps the lean going.
  - **2i-4, jumps straighten him up (2026-10-13).** The user checked in
    the original: he does **not** stay leaned through a jump, even with
    the turn key held, against the traced rule. Stand-in until Q59: the
    switch is off while jumping, falling and landing, so the lean goes
    back to upright at the usual 0.03 per update. Crash, death and the
    other states keep the note's rule.

### M3 — Daventry comes alive

**You will see:** doors opening, traps firing, chests, animated objects,
the first puzzles working.

- A: the script language (`.cs` commands like `set`, `if test`,
  `sendEvent`), the event system, and each object kind used in Daventry
  (door, trap, chest, emitter, animation lists...).
- B: script interpreter, event system, the object kinds.

Notes: `docs/behaviour/doors.md` (doors, their scripts, the minimum of the
`.cs` language), `docs/behaviour/emitters.md` (emitters, particle
catchers, ground animators).

- **3a — Daventry's doors (planned 2026-10-12; 3a-1, 3a-2, 3a-3 done
  2026-10-12, MODLOG; checked in the logs, and by eye with the mouse by
  the user).** Rules from
  `doors.md`.
  Three sub-steps, each with its own tests and MODLOG entry.
  - **Words used here.** A *script* is one of the game's `.cs` text files
    (puzzle logic). An *event* is a message sent to a named object
    (`sendEvent`), handled when its time comes. A *movement* is one of an
    object's named animations (`closed`, `open`, ...); its *phase* runs
    from 0 (first frame) to 1 (last frame). A *cue* is an action written
    in the movement (`cue0=0.0 playsound door4.aud`) that fires when the
    phase passes it.
  - **3a-1, the script runner** (`kq8-core` `script.rs`, no Bevy). Only
    what `doors.md` "The script language" lists:
    - a line is split into words (spaces/tabs), `#` starts a comment, a
      double-quoted part is one word;
    - `$1`…`$n` the call's arguments (missing ones empty), `$Name` a
      variable, replaced and split into words again; an unset or empty
      one **vanishes**, unless quoted (then one word, maybe empty);
    - `if <command…>` runs the rest of the line and takes the branch when
      `$Console::Return` is 0 or empty; `else` flips; `endif` closes;
      skipped `if`s are counted so their `else`/`endif` pair up; `else
      if …` = `else` + a nested `if` that needs its own `endif`;
    - `test a op b`: exactly 3 words (else a usage line, false); numbers
      when both look like numbers, else text with capitals counting;
      `&&`/`||` whole numbers only (error for text);
    - `set`, `echo` (to the log), a first word that names a script (with
      `.cs` added if it has no dot) runs it with the rest as arguments,
      an unknown command logs an error and fails;
    - `sendEvent <delay> <object> <type> [data…]` and
      `KQObject::collisionType <name> <±bits>` are handed to the caller;
      the later-milestone commands (`KQMonster::speak`, `doPopup`,
      `addMapIcon`, `earnTeleport`, `teleportersEarned`,
      `KQMonster::checkInventory`, `deleteObject`, `handsOff`,
      `KQMonster::setState`, `pauseAnim`, `preloadShape`,
      `KQObject::setHidden`, `loadKQ`, `setThirdPerson`, ...) are
      **stubs**: one log line, nothing else (`checkInventory` → 0).
    - Stand-ins (asked as Q45): `$Console::Return` is `0` for true or
      success and `1` for false or failure (the note gives only "0 =
      true"); a word "looks like a number" as the note says and its value
      is its leading number part. **Confirmed 2026-10-13** (`doors.md`
      "Q45" e, f; see 3r).
    - The start-up follower (`kq8-data` `world_script.rs`) is **not**
      changed in this step (it still skips a line with an unknown
      variable and ignores `if`). Moving it onto the new runner is a later
      step of its own.
    - Unit tests on made-up scripts (never copies of the game's files):
      vanishing and quoted variables, nesting, `else if` with its extra
      `endif`, `test` with numbers/text/wrong word count, `&&`/`||`,
      `$Console::Return`, a script calling another, `sendEvent` parsing.
      Viewer log: every Daventry door script read through the block
      counter (balanced `if`/`endif`).
  - **3a-2, the door and the event queue** (`kq8-core` `door.rs`,
    `events.rs`, no Bevy).
    - Door state (rule 1): closed / opening / opened / closing, closed at
      the start. `KQDoorEvent 1` opens only when closed, `2` closes only
      when opened, anything else (also no number) toggles; ignored while
      moving (rules 2–4). `KQAnimPlayEvent <movement>` starts it by name,
      state unchanged (rule 6).
    - Playing a movement: its `loop=` start phase and speed; phase moves
      by speed × (1/15 s) / play time per update; a play-once movement
      finishes at 1 (forward) or 0 (backward). When the finished
      movement is called `open`/`close` (any capitals): state opened /
      closed, the `opened` / `closed` movement, and the script with `$1`
      = door name, `$2` = `Open` / `Close` (rule 5).
    - Cues: a cue at phase c fires when c is in [lower, higher) of the
      phases before and after the update (rule 9). `playsound` is logged
      (no audio yet); anything else is logged as not done.
    - **The door's turn** (rule 24, open question 1): worked out from the
      phase every update, never added up: the shown frame is
      round(phase × (frames − 1)), as for Connor's one-shots, and the turn
      is the frame steps 1…k chained (each step in the door's own,
      already-turned axes). Measured from `door1.kqs` with a throwaway
      program: every chained step turns about the same point, (−634,
      −63) in the door's own space (its hinge edge); frames 13, 14, 15 are
      165°, 167.75°, 165°. So `close` from 0.93 (frame 14) starts 2.75°
      past `opened` (frame 15), a tiny jump, and a full cycle comes back
      to exactly 0°. The box turns with it: the door's collider is moved
      every update.
    - Events (`events.rs`): game time = updates / 15. `sendEvent` with
      delay d is due at now + d; due events are handled in time order,
      same time in send order. Unknown object or event type: logged,
      nothing sent. `KQCueEvent n1 [n2]` → the target's script with `Cue
      n1 n2` (rule 23). `KQStatsChangeRequestEvent` → logged (stats are
      M5).
    - **Order in one update** (stand-ins, Q45; confirmed 2026-10-13,
      `doors.md` "Q45" d): (1) clicks; (2) events
      due by now, including those just sent with delay 0 (open question
      4); (3) each moving door advances, except one started in this
      update, which first advances in the next (as Connor's transitions,
      Q40 a), cues fire, finishes call the script (events sent there are
      handled in the next update); (4) the door colliders are moved; then
      Connor's update as before.
    - Unit tests: the state table (1/2/0/none in every state), `open` 15
      updates and `close` from 0.93 14 updates (door1/door2; each door
      uses its own file's frames and time, Q46), `Open`/`Close` calls,
      `KQAnimPlayEvent open` from closed ends opened with `Open`, cue at
      0.0 fires in the first update, `Saradoor`-like cue at 0.95 on
      `close` never fires, turn after 10 open/close cycles is exactly 0,
      event order and delays.
  - **3a-3, clicking and the viewer** (`kq8-core` `click.rs`;
    `kq8-viewer` `door_view.rs`, `load.rs`, `main.rs`).
    - Load every `KQDoor`: its four movements (`loop=`, cues), all frames
      of its shape with their frame transforms, `script=`,
      `colliderType`, `acceptsInventory`, `acceptAllItems`,
      `tooFarAwayAction`, `doNotPointAt`, its `BOUNDS.KQ` box. Doors are
      drawn by `door_view.rs` (turned as above, the shown frame's mesh),
      no longer as static copies.
    - The play cursor: a **left click** sends a ray from the camera
      through the mouse; the nearest box hit among objects whose
      collision type has bit 0x20 is picked (rule 10). Stand-in (Q45):
      only those boxes are tested; walls, terrain and other objects do
      not hide a clickable one (confirmed, `doors.md` Q45 b). No weapons yet, so the cursor is always
      the play cursor.
    - Reach (rule 11): Connor's feet turned into the door's own axes
      (its current, turned box), only the part outside the box counts on
      each axis; in reach when that distance is < 2 × `attackDist`
      (`GAME/Kq/Connor.kq`, 2,350 → 4,700). The box as it stands, turned
      (confirmed, Q45 c).
    - In reach, nothing in hand: script `<door> ConnorAction do do`.
      Item in hand (test option only, no inventory yet): accepted when
      the door's `acceptsInventory` names its type, or `acceptAllItems=1`
      (stand-in, open question 2), then `<door> ConnorAction <item
      object> <item type>`; refused: logged "that doesn't work" remark,
      no script. Out of reach: `tooFarAwayAction` 0 → logged "too far"
      remark, nothing sent; else `ConnorAction TooFar TooFar` (rules
      12–14). A picked object that is not a door: logged, its script not
      run (only doors in this step).
    - `KQObject::collisionType` changes the runtime collision type: −32
      makes a door unclickable. Removing bit 0x1 would stop Connor's
      collision; no Daventry script does, so it is logged, not applied.
    - Not done (logged): Connor turning to face the door (rule 16, which
      states not traced), the remark lines themselves (M4), hits,
      `Die`, `ProjImmune` (no combat yet, M5), `tooFar=` (a display
      setting, not reach: Q45 a).
    - Test options: `--click NAME,N` clicks object NAME before update
      N + 1 (no mouse needed); `--item TYPE` puts an item of that type in
      Connor's hand (object name = the type).
    - Log `kq8-artifacts/logs/doors.log`: for every click the picked
      object, distance to its box, reach yes/no, the script arguments;
      every script line that does something (echo, `set`, stubs, errors);
      every `KQDoorEvent` with its value and the state before and after;
      every finished movement with its `Open`/`Close` call; cues; one
      line per update for each moving door (phase, frame, turn angle).
    - Expected (`doors.md` "How to check"): `Condoor` clicked in reach →
      `Condoor ConnorAction do do` → `KQDoorEvent 1` → opening for 15
      updates → opened → `Condoor Open`; clicked again: stays open; from
      far away: "too far", nothing sent; open door turned ~165° out of
      the doorway, still solid where it now stands.

- **3b — emitters, particle catchers, ground animators (planned
  2026-10-13; 3b-1, 3b-2, 3b-3 done 2026-10-13, MODLOG; checked in the
  logs and by eye by the user).** Rules from `docs/behaviour/emitters.md`. Three
  sub-steps, each with its own tests and MODLOG entry.
  - **Words used here.** An *emitter* is an invisible point that keeps a
    fixed number of *particles* (small models: leaves, drips) flying; each
    particle repeats the same path every *life*. A *particle catcher* hides
    one emitter's particles inside a building's box. A *ground animator*
    swaps the ground texture of some grid squares every few tenths of a
    second (flowing water). A *tick* is one game update (1/15 s).
  - **Where the code goes.** Reading the keys: `kq8-data` `effects.rs`
    (no rules, just the numbers with their defaults). The rules:
    `kq8-core` `emitter.rs` (particles and catchers) and `ground_anim.rs`
    (animators), no Bevy, unit-tested. Drawing and the tick:
    `kq8-viewer` `effects_view.rs` (emitters, catchers) and
    `ground_anim_view.rs` (animators), run in the fixed 15-per-second
    update after the doors and Connor. Logs:
    `kq8-artifacts/logs/effects.log` and `ground_anim.log`.
  - **Random numbers:** a small seeded generator of our own (uniform
    fractions, the note says the original's sequence needn't be copied).
    Same seed every run, so two runs give the same log.
  - **3b-1, emitters.** Keys and defaults from the note's table
    (`maxParticles` 200, `maxLife` 100, `translucency` 256). Position: the
    object's place after the usual placement rules (`loc`, `loadKQ`
    position); whether the ground rules apply to emitters is not in the
    note: the log lists any emitter below the ground. At the start every
    particle gets its offset, velocity, spin, sway speeds and starting age
    (rule 1) and is moved on by age × velocity. Every tick: rule 3 steps
    1–6 in that order, sway (rule 6), gravity as age × g (rule 7),
    restart after `maxLife` + 1 ticks, `delayStart` (rule 4). "Non-zero" =
    absolute value ≥ 0.000001. Spin: the turn angles add up and are used
    like an object's `dirX/Y/Z` (the note's assumption). Drawn: frame 0 of
    `shape` at the particle's position and turn, only when visible and age
    ≠ 0, with the emitter's `translucency` (likely, open question 2), blended
    between ticks except right after a restart. Always active (rule 12).
    Not done (logged): `follow` / `followMesh` (rule 11; no Daventry
    emitter uses it), dying (rule 10; no file uses it), the script
    commands (open question 4). `randomZRot` ignored (Do not reproduce).
    Tests: same path every life, restart after `maxLife` + 1 ticks with
    one hidden tick, gravity sum n(n+1)/2, sway = velocity × sin, delayStart.
    Log: per emitter its settings and particle count; for `stream1` and
    `leaf1`, particle 0's choices and then per tick its age, position and
    visibility (note "How to check").
  - **3b-2, particle catchers.** After loading, each catcher looks up its
    emitter and building by name (any case; nothing found → does nothing,
    logged; the original counts capitals, but every catcher's names match
    exactly in all worlds, 3r). Box = the building's `.dig` bounding box; top = box top +
    building z. Every tick, after the emitters: each visible particle of
    its emitter with z ≤ top is turned into the building's own axes and
    hidden if min ≤ value < max on all three axes. Catchers run after the
    emitters in the same tick (the original's order is not settled; ours
    is fine, `emitters.md` catcher rule 5). Log: each catcher's
    box and a line whenever it hides a particle.
  - **3b-3, ground animators.** Squares chosen once at load (explicit
    list if `numGridBlocks` ≠ 0, else every square whose material is an
    `origTexture`, minus the exclude list). Started when `startNow=1`
    (else logged and left alone). Changes: the first after `delay`
    seconds, entry 0; then every `delay` seconds the next entry; with
    `repeatflag=0` stop at the last. ~~Timer on the game clock (ticks /
    15), next due `delay` after the previous due time~~ (stand-in;
    **replaced in 3r-1**, Q49: continuous game time, checked every drawn
    frame, next due `delay` after the time of the change). Ground flags
    (envInfo) not changed (open question 3). Drawn: animated squares are
    taken out of the normal ground mesh's surface (their floor stays in
    it) and drawn one entity each; on a change its texture follows the new
    material, while its orientation stays that of its own loaded material
    (changed after the user's check; confirmed by Q50). The floor view
    (U) treats them as surface squares.
    Log: squares chosen per animator (expect the note's table: 86, 0, 57,
    19, 0, 17, 32, 2, 13, 4, 4), square (45, 57) never animated, and each
    change with its game time and material.

- **3b-4 — emitters that follow something (planned 2026-10-13; done
  2026-10-13, MODLOG; checked in the logs and by eye by the user).**
  `emitters.md` rule 11. In the data only IceWorld's `snow` follows
  something when a world starts (Connor); the dragon fires are made by
  the dragon heads (`fireFile=`, monsters, M5) and Temple4's `lucfire` by
  its altar script, so those two kinds are not started yet.
  - `follow=<name>`: Connor, `KQCamera` (the game camera) or a placed
    object (any case). Every update, before the particles move, the
    emitter's `loc` = the target's position + `followOffset`, the offset
    turned with the target unless its x and y are both 0 (then world
    axes). Connor's turn = his heading about z; the camera: position only,
    no turn (the original turns the offset by the camera's turn, Q52 c;
    no emitter in any world follows the camera). Target not found: the
    emitter is not started (logged; the original would crash, so keep
    this, Q52 b).
  - Velocity limits turned with the target: a particle's velocity is
    chosen in the limits, then turned by the target's turn at that
    moment (rule 1: at the start; rule 2: kept after that).
  - `followMesh`: at each restart the particle takes the target's current
    turn and a new random velocity (turned). Its value in the data is a
    mesh-part name (`fire`, `Box01`). **Answered (Q52 a), not built yet:**
    the emitter sits at that part of the target's (animated) model,
    `followOffset` unused; part missing → the emitter dies (rule 10). To
    do with the dragon heads (M5) and Temple4's `lucfire`, the only users.
  - Particles already flying don't move with the target (their positions
    are world positions); restarts happen at the moved `loc`.
  - Tests: offset in own vs world axes, turned velocity, restart at the
    moved spot, `followMesh` new velocity and turn. Log: `snow`'s `loc`
    vs Connor every second, particle 0 as before.

- **3c — animated objects play (planned 2026-10-13; done 2026-10-13,
  MODLOG; checked in the logs and by eye by the user).** Rules from
  `objects.md` "Animated objects at load" (rules 2–4, Q13), the phase
  rule the doors use (`doors.md` rules 5, 8) and the loop length Connor's
  loops use (the loop's stored length). Until now every animated object
  other than a door was drawn frozen in its start frame (the user saw the
  fountain's water, `wellwatr`, as drips stuck in the air).
  - **Which objects:** every placed object with `animLoops` that is not a
    door, whose current movement's `loop=` speed is not 0 (Daventry: the
    windmill sails, mill wheel and gears, signs, shutters, torches, lamps,
    burners, waterfalls, fish, the fountain water, …). Speed 0 stays a
    still model, as now.
  - **Playing** (`kq8-core` `anim.rs`, no Bevy): phase starts at `a`
    (`loop=` start); each update it moves by speed × (1/15 s) ÷ the loop's
    stored length; repeating loops wrap round 0..1, one-shots stop at the
    end (and stay there: `nextAnimName` is not traced). Frame shown:
    repeating round(phase × frames) mod frames (rule 2, as the start
    frame), one-shot round(phase × (frames − 1)) (as doors). `flags` bit 8:
    random start phase and speed × a random 0.9–1.1 (rule 4).
  - **Whole-object steps** (frame transforms, models-and-textures.md Q1 b):
    first measured and logged per object. Applied, if any running object
    has them, like the doors: the pose of the shown frame = its steps
    1…k chained, never added up across cycles. (Q51: the original adds
    them up and the data's cue puts the object back each cycle; the same
    picture for Gnome's rocks, the only ones, so kept, by the user's
    check. `KQBoulder` is not played: 3r-2.)
  - **Not done (logged):** cues (sounds), `nextAnimName`, `animPos`,
    scripts changing an object's movement (`KQAnimPlayEvent` to non-door
    objects). Collision boxes stay fixed. (Q51 rule 5: in the original a
    box moves with its object; "fixed" in Q33 a means its size. Only the
    rolling rocks move, and their colliders don't follow yet; not done.)
  - **Drawing** (`kq8-viewer` `anim_view.rs`): all frames of the loop
    built once per (model, loop), one entity per frame, only the shown one
    visible (as doors); hidden and unused objects keep their H-key
    visibility; see-through objects keep their opacity.
  - **Log** `kq8-artifacts/logs/anim.log`: per object its loop, frames,
    length, start phase and speed (with the random parts), its steps;
    per update for a few objects (`wellwatr`, `windprop`, `millwheel`,
    a torch) the phase and frame. (A loop's stored length is
    (frames − 1) / 15 s, so a repeating loop that runs through all its
    frames per cycle skips one frame per cycle at 15 updates per second;
    that follows from the rules, not a bug.)
  - Tests: phase and wrap, frame rule (start frame equals the still model's
    start frame, e.g. `telepad1` 0.3 × 16 → 5), one-shot stops at the end,
    random range, speed −1 runs backwards.

- **3r — review after the answers to Q45–Q52 (planned 2026-10-13; 3r-1 to
  3r-4 done 2026-10-13, MODLOG; checked in the logs and by eye by the
  user).** The
  study session answered Q45–Q52 (`doors.md` "Q45", rules 8 and 24;
  `emitters.md` emitter rules 2 and 11, catcher rule 5, ground animator
  rules 2 and 3; `objects.md` "Loops that move the whole object"). Each
  answer checked against the code and, where it could matter, against the
  data of all 12 worlds (throwaway scanner in B's scratch folder, nothing
  kept):

  | Q | Answer | Our code | Change |
  | --- | --- | --- | --- |
  | Q45 a–f | all six stand-ins right; `$Console::Return` details | matches (script call keeps the last command's result; `-.5` is text; unknown command 1) | comments only. Not copied: a false text `==` can leave −1 instead of 1 (which sign for which order is not given; both count as false; asked as Q54) |
  | Q46 | each door plays its own `.kqs` | already | none |
  | Q47 | sway and turn carry on; paths drift | already | none |
  | Q48 | catcher order not settled (ours fine); names compared with capitals counting | we ignore capitals | none: every catcher's `emitter`/`interior` name in all worlds matches exactly (Daventry 3, IceWorld 10), so both give the same result |
  | Q49 | next change = clock time **of this change** + `delay`, on continuous game time, not the 15/s tick | previous due + `delay`, checked once per tick | **3r-1** |
  | Q50 | the square's own orientation | already (the fix) | none |
  | Q51 | steps added up + the data's reset cue; user: our chained way gives the same picture, keep it | chained per frame | none for the rocks; **3r-2** for `KQBoulder` (rule 8: don't play it) |
  | Q52 a | `followMesh=<part>`: sit at that model part, `followOffset` unused, part missing → the emitter dies | name ignored | none now: only the dragon fires (IceWorld, made by monsters, M5) and `lucfire` (Temple4's altar script) use it, and neither is started. Do it when those arrive |
  | Q52 b, d | missing target: keep ours; order right | already | none |
  | Q52 c | offset turned by the target's whole turn, the camera's too | Connor heading, objects all three angles, camera no turn | none: no emitter in any world follows `KQCamera` (only `snow` → Connor, offset z only) |

  - **3r-1, ground animator clock (Q49).** `kq8-core` `ground_anim.rs`:
    after a change at clock time t the next is due at t + `delay`
    (`update` takes the clock time; the change records it). `kq8-viewer`
    `ground_anim_view.rs`: the step moves from the 15-per-second update to
    every drawn frame, with game time = the fixed clock's time + its
    overstep (the time already run since the last update), so it is
    continuous and stays in step with the other systems' game time. The
    log gives each change's game time and the gap since the last change
    in seconds; the 10-second summary gives the gaps' smallest, largest
    and average. Expected: gaps 0.300 s plus at most one drawn frame (at
    60 frames per second, 0.300–0.317 s), never the 0.267/0.333 s of the
    old tick-based timer. Unit test: changes checked at irregular frame
    times are each `delay` after the previous **change**, not after the
    previous due time.
  - **3r-2, `KQBoulder` still (Q51 rule 8).** `kq8-viewer` `load.rs`: an
    object whose `classType` is `KQBoulder` is not played (drawn still,
    frame 0: its `stop` loop starts at 0.0), logged. Only one in the game
    (Gnome `boulder`, `pushston`). Until pushable objects are studied.
  - **3r-4, a moving object's collider moves with it (Q51 rule 5).**
    Found while checking 3c against Q51: Gnome's three rolling rocks have
    `BOUNDS.KQ` boxes (`theroll.kqs`, ±900 × ±900 × 0–1,750, type 0x1,
    which Connor collides with), but our boxes stay at the rocks' start
    while the rocks roll up to 9,691 away. `kq8-viewer` `anim_view.rs`:
    every update, an object whose loop has whole-object steps puts its
    collider (found once by the object's name in the world's object
    colliders; none or several → logged, left alone) at the pose it is
    drawn with (the chained steps of the shown frame, as 3c). Runs in
    `anim_step`, which comes after Connor's update, so Connor is tested
    against the box of the previous update (one update late; the
    original's order between objects was not traced). Not done: what
    happens when a moving box runs into a standing Connor (the mover only
    sweeps Connor's own moves; a box that ends up around him counts as
    "started inside", which was not traced, `collide.rs`); the rocks
    knocking him down (not studied). Log `anim.log`: each such collider
    found, then its position every second for the first 10 s.
  - **3r-3, docs.** Comments in `script.rs`, `ground_anim.rs`,
    `ground_anim_view.rs`, `emitter.rs`/`effects_view.rs` and the plans
    above (3a, 3b, 3b-4, 3c) now say "confirmed" where they said
    "stand-in"; provenance updated; Q54 asked.
  - **Next after 3r:** traps (`traps.md`), chests (`chests.md`), items
    and picking up (`items.md`), each planned here before it is built.

- **3d — Daventry's traps (planned 2026-10-13; 3d-1 to 3d-3 done
  2026-10-13, MODLOG; checked in the logs, and by eye against the original
  by the user).** Rules from `traps.md`
  (and `connor-movement.md` rule 4, `camera.md` "Distance" rule 2 for the
  two commands). Three sub-steps, each with its own tests and MODLOG
  entry.
  - **Words.** A *trap* is an invisible box; when Connor walks into it or
    out of it, its script runs (`On` / `Off`). Its *inside list* is who is
    in it now. A *sticky* trap (`trapType=2`) has a smaller *inner* box to
    get in and the full box to get out, so music zones don't flicker at
    the edge.
  - **Data (Daventry, counted 2026-10-13):** 58 trap files, all
    `colliderMask=4` (Connor only; other worlds differ, Q57); 19 are
    templates that `newworld.cs`-style scripts place 17 times with
    `loadKQ` (positive z; their angles are ignored, the box never turns);
    8 files have `locZ=-1`; 4 sticky traps (`chrchtrp` 0.9, `mugltrap`,
    `mwiztrap`, `mwzhtrap` 0.8).
  - **3d-1, the trap rules** (`kq8-core` new `trap.rs`, no Bevy). A `Trap`:
    name, position, box (`BBMin/BBMax` added to the position, never
    turned), script, mask (`colliderMask`, default 0, stand-in: not
    stated in the note), `trapType`, `innerPercent`, inside list, shrunk
    yes/no. On entering the world: sticky → box × `innerPercent` (all six
    numbers, so it shrinks towards the position, rule 10). `check(object)`
    for one object that moved (name, type, position, its `BOUNDS.KQ` box,
    its ball stack), returning what to call:
    - who counts: the trap's mask has a bit of the object's type
      (stand-in, Q58: rule 4's two-sided pair rule would let every trap
      see Connor, whose own mask 0x281B has the trap type 0x10, and rule
      16 could not switch a trap off); otherwise nothing;
    - **touching** = one of the object's balls reaches the box (the ball's
      centre within its radius of the box; stand-in for open question 2:
      the same balls as for walls, tested where he ends the update, not
      along the way);
    - not inside and touching → added; first one in → `On <name>`, else
      `Cue 301 <name>` (rule 6); sticky → box back to full (÷
      `innerPercent`);
    - inside: still in if the centre of his box (position + middle of his
      `BOUNDS.KQ` box) is in the box (edges count) or he is touching;
      else taken off, list empty → `Off <name>`, else `Cue 302 <name>`
      (rules 7, 8); sticky and empty → shrunk again;
    - one-shot (`trapType` bit 1): gone after its `On` (rule 11; unused in
      the data);
    - mask change (`KQObject::collision`, rule 16): applied at once; no
      `Off` is sent for whoever is inside (stand-in, open question 1),
      and since the pair rule now fails, later moves neither add nor
      remove him (stand-in; logged).
    Unit tests: enter on touch, stay while the centre is in, leave only
    when both fail; sticky inner/full box and the shrink/grow numbers; a
    second object gives `Cue 301/302`; the pair rule (mask 0 never fires);
    one-shot.
  - **3d-2, traps in Daventry** (`kq8-viewer` `load.rs`, `door_view.rs`
    or a new `trap_view.rs`; `kq8-core` `active.rs`). Loading: every
    placed `KQTrap` becomes a trap: position as written (`keepOnGround`
    ignored, rule 1), z of −1 → the floor grid height there (rule 2), the
    template copies at their `loadKQ` place. The traps join the doors'
    world (`Active`): the same variables, script runner and event queue.
    At the start each trap's script gets `Init` (rule 13). Each update,
    **after Connor's update** (so the trap sees where he is now), if his
    position changed, every trap checks him; `On`/`Off`/`Cue` run the
    trap's script with `$1` = trap name (rules 6, 8). `sendEvent … <trap>
    KQCueEvent n1 n2` → `Cue n1 n2` (rule 14, as doors). `KQObject::collision
    <name> <±bits>` changes a trap's (or door's) mask (rule 16).
    `TalkComplete` and `StateChange` are M4/M5. The shared scripts
    (`walk_la.cs` ...) are in `GAME/Kq`: scripts are looked up in the
    world first, then `GAME/patch`, then `GAME/Kq`
    (`models-and-textures.md` "Which one wins"; no Daventry script has a
    game-wide twin). Log
    `kq8-artifacts/logs/traps.log`: every trap at load (name, file, box,
    shrunk box, mask, script, `Init`), and every `On`/`Off`/`Cue` with
    Connor's position, his box centre, the box used, and the script's
    lines. The main log gets a count at load and each `On`/`Off`.
  - **3d-3, what the walk/run zones do** (`kq8-core` `active.rs`, viewer).
    `KQConner::setRun 0|1` sets Connor's run switch (connor-movement.md
    rule 4); `KQCamera::setRange <min> <max>` sets the camera's range,
    refused unless min < max (camera.md). Both are handed to the viewer
    as requests and applied before the camera's update in the same tick.
    The other commands the Daventry trap scripts use and that only act
    (music, monsters, ropes, cutscene camera, `move`, `handsOff`, ...) are
    added to the stubs (one log line); commands that answer a question
    (`getloc`, `checkDistance`, `exist`, `checkSave`, ...) stay unknown
    (error line, result 1) until they are studied. `--hold F,T,R` then
    sets the run switch only at the start, so the scripts can change it.
    What you'll see: by Connor's house and the other houses he drops to a
    walk and the camera comes closer; away from them he runs again.
  - **3d-4, corrections after Q57/Q58 (planned and done 2026-10-13).**
    The study answers (`traps.md` rules 4, 6, 16) change two stand-ins:
    - **Touching is swept** (rule 6): each of the visitor's balls, from
      where its move started to where it ended, against the trap's box
      widened by the ball's radius (a slab test); a ball that starts inside
      counts. The move that enters is not also tested for leaving. For
      staying (rule 7, "still touching") the same swept test is used
      (stand-in: the note doesn't say; at most one update later `Off`).
    - **Mask changes only gate coming in** (rule 16): those inside stay
      inside and get their `Off` when they leave.
    - Q58's one-sided rule was already the stand-in: unchanged.
    Tests: a move straight through a thin trap gives `On`, the next `Off`;
    a path beside it nothing; mask removed while inside → `Off` on
    leaving, no new entry.

- **3e — chests and items (planned 2026-10-13; done 2026-10-13: 3e-1
  to 3e-5 and 3e-4b, MODLOG).** Rules from
  `chests.md` and `items.md` (and `doors.md` rules 10, 11, 13, 14 for
  clicking, `objects.md` for heights and hiding). Five sub-steps, each
  with its own tests and MODLOG entry.
  - **Words.** An *item type* is one line of `GAME/Kq/Invinfo.itl`
    (`INVITEM_Crystal`: its model, sounds, size). An *item object* is one
    thing of that type, in the world or carried, holding a *quantity*
    (one object can be 200 coins). An *inventory* is the list of item
    objects a character carries (in M3 just a list; the screen is M4).
  - **Data (Daventry, counted 2026-10-13):** 3 `KQChest`s placed
    (`smlchst3` 3 crystals, pub upstairs; `smlchst4` 200 silver coins,
    keep; `wizchest1` 1 lady bell, hidden, type 1 until a Reveal potion);
    the `jar` chest template is not loaded. 3 *scripted* chests, plain
    animated objects with a script, type 33 (`smlchst1`, `smlchst2`,
    `jar1`). Items at the start: 21 `KQMonster::inventory world …` lines
    in `WorldInv.cs` and 1 in `newWorld.cs` (`BirthdaySuit`, hidden); 23
    Daventry scripts make or give items.
  - **3e-1, item types** (`kq8-data` new `items.rs`, `kq8` CLI). Read
    `Invinfo.itl`: `[InventoryItemTypeList]`, `itemN=` lines, fields
    tab-separated in the order of the `;itemX=` header (rule 1). Keep
    the name, id, category, land and pickup sounds, shape, radius,
    translucency. Name lookup (rule 2): put `INVITEM_` in front unless
    the name starts with exactly that, then compare ignoring capitals.
    New CLI command `kq8 item-verify` (log
    `kq8-artifacts/logs/item-verify.log`): number of types (expect 135),
    each type's shape found or not (world archives, then `GAME`), its
    `BOUNDS.KQ` box (shape name lower-cased, as for objects), and every
    item type named by any world's scripts and chests that the lookup
    does not find. Unit tests: field split, the prefix rule
    (`silverCoins`, `INVITEM_Crystal`, `ladybell`, `invitem_X` gets a
    second prefix, so it is not found).
    **Done:** 135 types, all models found, 3 without a box; of 758 type
    names in the data only `INVITEM_All` (an `acceptsInventory` value) and
    one id number don't resolve (Q62). `acceptsInventory` turned out to
    hold comma-separated lists too (Q62); it matters for using items on
    doors (today: exact comparison), not for this step.
  - **3e-2, item objects and inventories** (`kq8-core` new `item.rs`,
    `active.rs`; no Bevy). An item object: name, type, quantity, third
    number −1 (rule 3), where it is (in the world at a position and turn,
    or carried by someone), hidden, script, `doNotPointAt` (default on,
    rule 4). Commands in the running world's scripts:
    - `KQMonster::inventory <who> <type> [qty] [x y z [rx ry rz]]` and
      `give <type> [qty]` (rule 9): a new object; `world` → placed (z −1
      / −2 → floor / surface height, which the caller supplies), any
      other name → carried by it; sets `NewInvItem`; unknown type →
      error line, result 1.
    - `KQMonster::checkInventory <who> <type> [var]` (rule 10): result 1
      if found, 0 if not or on error (was a stub answering 0).
    - `KQObject::setScript <item> <script|none>` and `setDontLook <item>
      0|1` on item objects; `KQObject::setHidden` and `deleteObject` on
      item objects. On other objects: as now (stub or unknown).
    - Stubs, logged: `KQObject::tossItems` (smashed barrels need weapons,
      M5; its flight is not fully traced, items.md open question 3),
      `KQMonster::dropItem` (only `rockHit.cs`, M5), `KQObject::changeBmp`,
      `KQObject::setGroupsAlways`.
    - **Name of a new item object:** not in the note. Stand-in:
      `<type without INVITEM_>.<n>`, n counting from 1 per world
      (`Crystal.1`); asked as Q60.
    - **Clicking an item** (rules 6–8): picked like any clickable object
      by its box (the type's shape in `BOUNDS.KQ`, lower-cased; our
      reading of `chests.md`'s crystal check); type 0x220 (clickable).
      Out of reach → "too far" (logged, nothing); item in hand →
      "that doesn't work"; in reach → taken out of the world, added to
      Connor's inventory, pickup sound logged (no audio yet), hidden from
      the world; then its script (if any) runs with `<item> ConnorAction
      do do`. Items never collide with Connor (type 0x220 and mask 0x809
      share no bit with his type 4 and mask 0x281B, pair rule).
    Unit tests: make in the world and carried, `NewInvItem`, unknown
    type, `give`, `checkInventory` 1/0 and its variable, pickup in and
    out of reach, script after pickup with `setScript … none` removing it
    (`Tavcoin.cs`), hidden items not clickable.
    **Done.** Also: a world command can now set script variables (the
    runner asks the world for them after each command), and
    `setHidden`/`deleteObject` on objects other than items stay stubs.
    Our choices: a world item needs x y z (else error); `setScript` and
    `setDontLook` on other objects are "not tracked" (result 1).
  - **3e-3, items at the start and in the viewer** (`kq8-data`
    `world_script.rs`; `kq8-viewer` `load.rs`, new `item_view.rs`).
    The start-up runner runs `KQMonster::inventory`, `give`,
    `KQObject::setScript`, `setDontLook` and `setHidden` on items through
    its host and sets `NewInvItem`, so `WorldInv.cs`'s lines work
    (`KQObject::setScript $NewInvItem dagger.cs`). The viewer: every
    world item gets a model (its type's shape, loaded once per type; also
    the types chests and world scripts can make later), its
    translucency, z −1/−2 → floor / surface height, its turn; hidden ones
    are not drawn and can't be clicked. Clicks go through the same
    picking as doors (nearest box). On pickup the model disappears; when
    a script makes a new world item while playing, it appears. Log
    `kq8-artifacts/logs/items.log`: every item made (type, quantity,
    name, where, `NewInvItem`), every pickup (distance, Connor's count of
    that type before and after), the start count in the main log.
    **Done.** The `KQMonster::inventory` / `give` line reader is shared
    (`item::read_make`) by the start-up runner's host (`load.rs`) and the
    running game. Daventry: 22 items (21 in `WorldInv.cs`, the hidden
    `BirthdaySuit`), models for 25 types. Item models are looked up like
    objects' (this world, then `GAME/Common`), so a type whose model only
    another world has is not drawn (Gnome's `schroomC.kqs`, not needed at
    the start there).
  - **3e-4, chests** (`kq8-core` new `chest.rs`, `active.rs`;
    `kq8-viewer` `load.rs`, `door_view.rs`). A `KQChest`: its keys (rule
    1), default type 0x8021 unless `colliderType` (rule 2; today chests
    load with the plain default 0x41, which is not clickable), movements
    `closed`/`open` played as doors' are (the phase step and cues shared
    with `door.rs`; no `opened`: it stays on the last frame, rule 3),
    states closed / open / empty. Clicks (rules 4–10): reach, "too far"
    and "doesn't work" as doors; 1st click plays `open` and makes the
    item object at `invLoc`/`invDir` (or goes straight to empty if no
    type); 2nd click moves the remembered item into Connor's inventory
    (from the world, or already carried: only the state changes, the
    note's stand-in), then the `Take` remark if any; later clicks the
    `Empty` remark (logged, lines are M4). The chest's script is not run
    on clicks; it gets `Init`, `EndLoop <movement>` when a movement
    finishes, and `Cue`. Chests are drawn like doors (frame of the
    current movement). `KQObject::collisionType` works on chests
    (`wizchest.cs`). Unit tests: the three clicks, no item type, item
    already picked up, item in hand refused, out of reach, `EndLoop
    open` after 1 s.
    **Done.** In the viewer a chest is loaded and drawn as a door (the
    loader's door list with `DoorAssets::chest` set; same box lookup and
    per-frame models); the running game keeps chests apart
    (`ActiveChest`). Daventry: `smlchst3`, `smlchst4`, `wizchest1`
    (hidden, type 0x1).
  - **3e-4b (replaced by 3g-2 after Q63), clicks need the camera's line of sight (planned and done
    2026-10-13, MODLOG).** The user's check in the original (2026-10-13): an
    object can't be clicked through a wall or the ground, but can when
    Connor is behind a wall and the camera sees the object. This
    contradicts `doors.md` Q45 b ("walls, terrain and other objects do not
    hide a clickable one"); asked as Q63. Applies to every click (doors,
    chests, items). Stand-in until Q63 is answered: the mouse ray (it
    already starts at the camera) is tested against the camera's view
    obstacles (`camera.md` "Obstacles": the drawn ground; buildings;
    objects with collision type bit 0x8 or 0x1000, including doors as
    they stand); if one is hit nearer than the picked box, the click picks
    nothing. Doors and chests block only when they are not the one picked.
    Reach (2 × `attackDist` from Connor's feet) is unchanged. Code:
    `kq8-viewer` `door_click_input` gets the ground and view blockers it
    needs (the same ones `game_camera.rs` uses); `kq8-core` `active.rs`
    `pick` takes an "is the line blocked before t?" test. Unit test: a box
    behind a wall is not picked, in front of it is; the ground blocks.
    Log: a blocked click names what blocked it and at what distance.
    Viewer check: from the pub's ground floor the upstairs chest can't be
    clicked; standing upstairs it can.
    **Done** as planned (`Active::pick_seen`, `game_camera::view_blockers`
    shared with the camera), plus a test option `--aim NAME,N` (the
    mouse's ray, aimed by the program at the middle of NAME's box).
  - **3e-5, scripted chests and baskets** (`active.rs`, viewer). A
    clickable object with a script that is not a door, chest or item:
    clicking it in reach runs `<name> ConnorAction do do` (`chests.md`
    rule 11 says these chests work with "the script runner, `items.md`
    and the commands above"; the call is the same as a door's, rule 12);
    `KQAnimPlayEvent <movement>` to it plays that movement (as doors,
    rule 6), so `smlchst1`, `smlchst2` and `jar1` open and drop their
    coins, and the mushroom baskets (`give mushroom 5`) give. These
    objects are drawn like doors. `KQMonster::speak` is logged as
    Connor's remark (M4).
    **Done.** Which objects: placed `KQAnimObject`s with a script and the
    click bit whose movement is still at load (Daventry 14); those whose
    movement plays at load (signs, fish) stay with `anim_view` and are not
    clickable yet. The movement-playing part of chests moved to
    `kq8-core` `named_anim.rs` (`NamedAnimPlayer`; first called
    `movement.rs`, renamed for clarity), used by chests and these
    objects; doors keep their own. Our stand-ins for what the notes
    describe only for doors and chests (`Init`, `EndLoop`, the click
    call, `KQAnimPlayEvent`) are asked as Q64.
  - **What you'll see after 3e:** coins, mushrooms, crystals and the
    other start items lying where `WorldInv.cs` puts them; click one near
    Connor and it vanishes into his (invisible) inventory, logged. In the
    pub upstairs the big chest opens on the first click and a crystal
    appears in it; the second click takes it; the third logs "empty".

- **3f — clicking objects that move at load, and `deleteObject`
  (planned and done 2026-10-13, MODLOG).** The user's check in the original
  (2026-10-13): clicking a swimming fish makes it disappear, Connor makes
  an eating sound and gains health. `fishies.cs` (data) does it:
  `deleteObject $1`, `KQSound::play eat.aud`, `KQStatsChangeRequestEvent
  2 0 0 0 0` to Connor, once (`Daventry::fish::full`, shared by the three
  fish). Asked as part of Q64 (d).
  - **Which objects.** Every clickable placed object with a script that
    isn't a door, chest, item or 3e-5 scripted object: the ones that move
    at load (Daventry 10: signs, fish, dead birds, `starball`,
    `girltree`) and still ones without movements. They stay drawn as now
    (`anim_view` or the still models). None of Daventry's has
    whole-object steps, so the click box stays at the placement (an object
    with steps logs that its box doesn't follow; not needed yet).
  - **Clicks** (stand-in, Q64 a): as doors and 3e-5 objects (the shared
    `script_args`): `ConnorAction do do` in reach, item in hand by
    `acceptsInventory`, out of reach by `tooFarAwayAction`. `Init` at the
    start (Q64 c). `KQAnimPlayEvent` to them: logged, not done (their
    animation belongs to `anim_view`). `KQCueEvent` → `Cue`.
  - **`deleteObject <name>`** on such an object or a 3e-5 object
    (stand-in, Q64 d): gone for good: not drawn, not clickable, no
    collision, not a view blocker; events to it are dropped (logged). On
    other placed objects it stays a stub. The viewer gets a request:
    hides the object's entities (a name tag on every placed object's
    entities), turns its collider off (moved far below the world, size 0)
    and drops its view box.
  - **Stubs:** `KQSound::play` (no audio yet); the stats event stays
    "stats are M5".
  - **Code:** `kq8-core` `active.rs` (`Clickable` gets the script and the
    click keys, `gone`; `Request::Delete`); `kq8-viewer` `load.rs` (the
    keys for `Clickable`), `terrain_view.rs`/`anim_view.rs` (name tags),
    `door_view.rs` (applies deletions). Unit tests: a scripted clickable
    runs `ConnorAction`; a fish-like script deletes itself (gone, not
    picked, request sent, a later event dropped); one without a script is
    logged as before. Viewer check: `--click fishies.01,N`: the fish is
    gone, `eat.aud` and the stats event logged; clicking it again does
    nothing; Connor can walk through where it was.
    **Done** as planned. Every placed entity now carries the object's
    name (`terrain_view::ObjectName`) so a deletion can hide it; deletions
    are applied in `trap_step` (with the other script requests).

- **3g — the clicking and script answers (Q54, Q60–Q64) (planned and done
  2026-10-16).** A answered these 2026-10-15/16 (`clicking.md`,
  `doors.md` "Q45" e, `items.md` rules 7 and 9, `chests.md` rule 8); our
  stand-ins from 3e/3f/3r are replaced one at a time. Each sub-step has
  its own unit tests and MODLOG entry.
  - **3g-1 (done 2026-10-16), `test` leaves −1** (Q54, `doors.md` Q45 e): a false text `==`
    leaves `-1` when the left word comes first in byte order, `1`
    otherwise; every other false text test leaves `1`. Unit test
    `test abc == abd` → −1, `abd == abc` → 1, `abc < abb` → 1.
  - **3g-2 (done 2026-10-16), what the click line hits** (Q63, `clicking.md` rules 1–12).
    Replaces 3e-4b's stand-in. The click line is tested against every
    clickable shape (bit 0x20) only, nearest wins: the boxes we already
    test, **plus buildings** (type 0x28 by default, so clickable, unless
    `colliderType` says otherwise: Daventry's cave, church and wizhouse
    have 136 and are not). Buildings are tested with the BSP line test the
    camera already uses. **The ground no longer blocks**, and neither do
    objects without bit 0x20. A picked building with no script: nothing
    happens, logged "swallowed, no script" (rule 10; in reach or out:
    `tooFarAwayAction` 1). With an item in hand: "that doesn't work"
    (rule 12). Buildings with a script (none in Daventry) get the calls as
    doors do (logged only for now: no Daventry building has one).
    Hidden objects stay unclickable: with default masks hiding drops
    their collider (`objects.md` "Hidden objects" 3, `object-lighting.md`
    rule 41), which is what the click line tests. Code: `kq8-core`
    `active.rs` (`pick_seen` takes "the nearest clickable building hit"
    instead of the view blockers), viewer `door_view.rs` / `game_camera.rs`
    (a list of clickable buildings). Unit tests: a box behind a clickable
    wall is not picked and the wall is; behind a 136 building it is
    picked; the ground doesn't block. Log: what the line hit first (name,
    kind, distance).
  - **3g-3 (done 2026-10-16), accept lists** (Q62, `clicking.md` rules 13–18): split at
    commas; first word `all` / `INVITEM_All` (any capitals; anywhere in the
    list, our choice from "Do not reproduce") = accept everything; other
    words are type names looked up with `items.md` rule 2 until `none`;
    unknown names skipped and logged; `acceptAllItems` non-zero = the same
    switch; compare the held item's **type**. `KQObject::setAccepts`
    adds. Unit tests: list, `INVITEM_All`, `none`, unknown name, prefix
    and capitals, `setAccepts` adds.
  - **3g-4 (done 2026-10-16), `EndLoop` through the queue, doors too, `getEndLoop`** (Q64 c,
    `clicking.md` rule 21): a finished non-repeating movement sends the
    object a cue 3333 through the event queue (arrives the next update,
    after what is already waiting); a cue 3333 becomes `EndLoop
    <movement>`, 500 is "projectile, combat" (logged). Doors get
    `EndLoop open/close` after their `Open`/`Close`. `getEndLoop <from>
    <to> 1|0`: one listener per object; when `<from>`'s non-repeating
    movement ends, `<to>` gets `Cue 300 0 <movement>`. Connor as `<from>`:
    his movements (`push`) are not played by scripts yet, so it is
    recorded and logged only. Unit tests: `EndLoop` one update later,
    door `Open` then `EndLoop open`, `getEndLoop` relays, 0 stops it.
  - **3g-5 (done 2026-10-16), `deleteObject` for everything, `NoName<n>`, chest pickup
    sound** (`clicking.md` rule 22, `items.md` rules 7 and 9, Q60, Q61):
    `deleteObject` on doors, chests and items too (deleted at once: not
    clickable, not solid, not drawn), every listed name, unknown name →
    error line and `1`; new item objects are named `NoName<n>`, one
    counter from 0 for the whole run; a chest's 2nd click logs the type's
    pickup sound. Unit tests for each.

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

### Outside the milestones: choosing the graphics API, `--renderer` (planned 2026-10-16, done 2026-10-16)

Not from the original game (its Glide / Direct3D choice is not studied):
our own launch option, asked for by the user. The viewer draws through
Bevy, which draws through **wgpu**, a layer that turns one set of drawing
calls into Vulkan, DirectX 12, Metal or OpenGL. Until now wgpu chose by
itself (Vulkan on the user's PC), and only the `WGPU_BACKEND` environment
variable could change that.

One step:

1. `--renderer auto|vulkan|dx12` (default `auto`). `auto` keeps today's
   behaviour (wgpu chooses; `WGPU_BACKEND` still works). `vulkan` or
   `dx12` tells Bevy's `RenderPlugin` to use only that API (it wins over
   `WGPU_BACKEND`). `vk`, `d3d12` and `directx` are accepted as other
   spellings. OpenGL (`gl`) is refused with a message: Bevy is built
   without its OpenGL part here (feature `gles`), and on Windows it would
   also need the ANGLE library; a test run with `WGPU_BACKEND=gl` crashed
   with "Unable to find a GPU". Adding it would be its own step.
   `viewer.log` records the API asked for at start-up and, once the
   graphics card is set up, the API actually used, the card's name and
   driver (`renderer: asked ... got ...`).
   If the chosen API is not available on the computer, Bevy stops with
   "Unable to find a GPU" (its own message); the log line written
   before that says which API was asked for.
   **Test:** run with each value and `--frames 60`; `viewer.log` must say
   `got Vulkan` / `got Dx12`; a wrong value must be refused.
   **Done** as planned (`crates/kq8-viewer/src/renderer.rs`); all values
   tested from the log, Daventry runs on both APIs; checked by the user
   2026-10-16. Not tested on Linux or another graphics card.
