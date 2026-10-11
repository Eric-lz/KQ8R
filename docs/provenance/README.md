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
| ~~Engine forces first and last 8 colours to the Windows system colours~~ **Replaced 2026-10-11**: palettes are used exactly as stored | Behaviour note `models-and-textures.md` "System colours" (Q3): the swap (20 entries) happens only on a 256-colour screen | Textures coloured by a world palette that use entries 0–9/246–255: none in 11 worlds; Swamp `STN1.BMP` (35 pixels) and `WIRLPOOL.BMP` (16,512 pixels, all entry 0). Swamp `Teleport.BMP` uses entry 0 for 3,961 pixels. Entry 0 is stored blue: these now show blue where they were black. Conflict with the Q5 answer about entry 0: Q22 |
| `.dib` and most archive `.bmp` are plain Windows BMP | Observed | 3,382 `.bmp` + 31 `.dib` start with `BM` and parse as uncompressed BMP; 8 of them are 24-bit |
| `.pal` is a Windows RIFF palette, 237 or 256 colours | Observed | 6 files. Which colours the engine uses from a 237-entry one is unknown |
| `GAME/8Gui/daventry.pbm` uses the loading palette (`GAME/8Gui/l*.ppl`) | Observed (measurement), confirmed by eye by the user 2026-10-11 | Neighbour colour change with every palette in the game: 18.4 with all 16 `GAME/8Gui/l*.ppl` (identical colour tables), 44.5 with `daventry.ppl`, 107.5 with `Menus.ppl`. The user confirmed `daventry.ppl` looks wrong (dark, speckled). Which `l*.ppl` file the engine loads is unknown, but all 16 have the same colours |

## Models and material lists — `crates/kq8-data/src/{pers,material,shape}.rs`

No behaviour note was used (none exists for this topic). "GPL notes" =
`libkq8fpc/doc/kq8kqs.txt`, `kq8dml.txt`, `kq8pers.txt` (Nico Bendlin,
GPL-3.0 repository, downloaded 2026-10-11, directory revision 22). Facts
about file layouts only; code and wording are our own. "Checked" numbers
come from `kq8 shape-verify` on the GOG 1.03 install
(`kq8-artifacts/logs/shape-verify.log`).

| Fact | Source | Checked how |
| --- | --- | --- |
| PERS block: tag, 31-bit size + 1 alignment bit, i16 class name length, name padded to even, then i32 version | GPL notes | Observed: all 4,573 `KQ8::Shape` files and 25 `.dml` files parse with 0 unread bytes and 0 bytes after the block |
| `KQ8::Shape` field order and per-version differences (v1–v4: light nodes from v2, attenuation start v3, intensity v4; v1 sequences have 2 junk bytes) | GPL notes | Observed: versions seen v1 29, v2 55, v3 3,921, v4 568; all parse exactly to the end |
| `TS::CelAnimMesh` layout, v2 mesh-wide scale/origin vs v3 per-frame scale/origin | GPL notes | Observed: 56 v2 and 23,097 v3 meshes parse exactly; no mesh had 0 frames |
| Vertex = 3 bytes (0..254) × scale + origin, plus 1 byte normal index | GPL notes | Observed: frame vertex 0 / 1 equal the min / max of the remaining vertices in all 471,186 frames |
| Face vertex indices are relative to the frame's first vertex | GPL notes | Observed: indices range 2..1153, never 0 or 1 (the bounding-box slots), never past the frame |
| Texture coordinates for a node start at `texture_frame × texture vertices per frame` | Our reading of the GPL notes | Observed: 0 out-of-range indices in the rest pose of every model. All coordinates lie in 0..1 |
| Parts are drawn with their mesh node transform only; the sequence's frame transform is **not** used for drawing (it is the step the whole object makes during an animation) | Behaviour note `docs/behaviour/models-and-textures.md`, Q1 (b) (session A, from data, high confidence) | Replaces our earlier choice "node, then sequence". Observed: `kq8 shape-verify` rest-pose numbers for all 4,573 models are identical before and after the change (frame 0 has a zero step, as the note says). Using the frame transform for moving objects is for M2/M3 |
| Transform = point as a row vector times the 3×3 matrix, plus translation | **Measured**, then confirmed by behaviour note `models-and-textures.md` Q1 (a) | 412,731 of 578,976 transforms rotate, so it matters. In 2,739 multi-part models, vertices of one part touching another part: 4,891 this way vs 2,761 transposed; clear wins 245 vs 7, 2,487 undecided. By eye: the chest's lid sits closed on the box (with the transposed matrix the measurement says its parts would not meet). Strong evidence, not proof (question Q1) |
| The game's Z axis is up | **Measured** (new) | Tallest axis: z in 2,950 models, y 1,014, x 590. Lowest point at z ≈ 0 in 3,177 models (y 90, x 29) |
| The rest pose is the first sequence of the first loop | Our choice | Observed: 4,571 models have 1 loop, 2 have 2 |
| Material: **u32** flags (the GPL note says u8), f32 alpha, i32 palette, 4 colour bytes, name of 16 (v1) or 32 (v2) bytes | GPL notes, corrected by observation | Observed: `davennew.dml` = 38-byte header + 255 × 32 bytes = 8,198 bytes exactly; flag values like 0x10103 need more than 8 bits |
| Material kind in bits 0..3 (3 = texture, 2 = colour, 0 = none), shading in bits 8..11 | GPL notes | Observed: 12,997 texture, 1 colour, 223 none. Flags seen: 0x403 11,221, 0x103 1,538, 0x10103 142, 0x203 75, 0x1403 20, 0xF000 31, 0x0 192. Meaning of 0x1000 and 0x10000 unknown |
| Texture names refer to files in a world's archives; models in `GAME/Common` have no textures next to them | Observed | 7,355 texture references found in the same world, 5,613 only in another world (mostly `GAME/Common` models), 0 in `GAME/`, 29 names missing everywhere (listed in the log) |
| Textures use the world palette (`resource/daventry.ppl` for Daventry) | Measured | Neighbour colour change on `wood2`, `Hay`, `g2g1str`: 39.3 / 77.1 / 20.4 with `daventry.ppl` and `8bit/daventry.pal` (identical), 46.3 / 77.1 / 35.9 with `daventr1.pal` (question Q3) |
| `TS::Shape` v2 in 4 Daventry tomb files | GPL notes (says unused leftovers) | Counted and skipped, not parsed |
| Normals: we compute them from the triangles | **Deliberate difference** | The game stores a normal index per vertex into a 256-entry table. The table is in the GPL notes; we do not copy it. We do not need it while models are drawn without lighting |

## Model viewer — `crates/kq8-viewer/src/{load,main}.rs`

| Fact or choice | Source | Checked how |
| --- | --- | --- |
| Game (x, y, z) → Bevy (x, z, −y), a rotation not a mirror | Our choice, from the measured "Z is up" | Confirmed by eye by the user 2026-10-11: `smlchest.KQS` (lid closed on the box, texture correct) and `alcmsign.KQS` look right. Mirroring not ruled out: the user saw nothing wrong, but we did not confirm that either model has readable lettering. Handedness is still question Q2 |
| Divide game units by 100 for display | Our choice | Display only; says nothing about the game's units |
| Model at rest = first sequence of first loop | Our choice | `alcmsign.KQS` (alchemist's sign): its loop is 31 sequences over 2.0 s. The sign tilts 10° in sequence 0, 0° in sequence 6, 10° in 15, 0° in 23, i.e. it swings. So the first sequence is not always a resting pose; the user saw the sign tilted, as expected from this |
| Textures use the world palette `<World>.ppl`; Windows-BMP textures use their own colour table | Measured | Viewer log: in `ITEMS22`, `HAY`, `alchy`, `TORCHY` (Daventry) every used colour of the own table equals `DAVENTRY.PPL`, so both give the same picture |
| Textures not in the current world are taken from another world | Our choice (question Q4) | Logged per texture when it happens |
| Front faces are the clockwise ones; back faces hidden; no mirror | Behaviour note `models-and-textures.md`, Q2 (session A: 98.9% of 1.49M triangles clockwise seen from the front; back-face removal on in the Glide renderer) | Viewer reverses each triangle's corners (Bevy's front is counter-clockwise) and keeps Bevy's back-face hiding. Observed: 30 of 32 chest triangles then face away from its centre (old order would give 2 of 32). Checked by eye by the user 2026-10-11: `smlchest.KQS` and `alcmsign.KQS` look right from all sides |
| No lighting, linear filtering, texture repeats | Our choice while lighting is not studied | — |

## Terrain — `crates/kq8-data/src/{kq_text,terrain}.rs`

Behaviour note = `docs/behaviour/terrain.md` (session A, answers Q6/Q7).
"Checked" numbers come from `kq8 terrain-verify`
(`kq8-artifacts/logs/terrain-verify.log`).

| Fact | Source | Checked how |
| --- | --- | --- |
| `.kq` files are `[section]` + `key=value` lines, `;` comments, keys in mixed case | Observed | All 17 terrain files parse; object files read for the height check |
| Terrain files live loose in `data/<World>/KQ/Terrain/` (not in `kq.vol`); images, `.dml` and light maps are in `resource.vol` | Observed | `terrain-verify` finds every named file there (except below) |
| Missing setting defaults (groundScale 12, heightBMPScale 60, `terrain.dml`, `terrm.dib`, `terrh.dib`) | Behaviour note | No terrain file leaves these out, so not exercised |
| Square = 2^groundScale units; grid corner at (0, 0); pixel column = x, bottom image row = y 0; height = value × heightBMPScale | Behaviour note | Observed: objects exactly on our ground (within 0.5 units): Daventry 11, Barren 11, Ice World 11, Dead City 7, Snow exit 2. Daventry's 11 span heights 6,630–30,600. Daventry's scale is exactly 255 per grey step (the note's 256.2 was objects standing a little above the ground; many stand exactly +5 or +25) |
| Two flat triangles per square; diagonal (0,0)–(1,1) when i + j is even, (1,0)–(0,1) when odd | Behaviour note | Unit tests for both diagonals. Observed: `St_gdad` stands between grid points and matches to 0.1 units |
| Height image has a plain grey colour table | Behaviour note | Observed: true for every terrain image |
| Direction table: u32 count, then per record u32 index, 16-byte name, u32 (always 0), u32 0..7 | Observed | Exact file sizes; names match the material list in the same order for every world (e.g. 255/255 Daventry, 164/164 Swamp, 170/170 Dead City). Meaning of the 0..7 value open (Q8) |
| Material image is one value per square; no value past the list | Observed | 128×128 in every world; 0 squares out of range. Meaning not confirmed (Q8): Daventry's most used value is 159 = `stone1.bmp` on 15,224 of 16,384 squares |
| `terrain2.kq` in temple1–4 names `ice.dml`, which those worlds do not have | Observed | Reported as a note; those grids are flat (all heights 0) |
| No holes | Our choice until the note answers it (terrain.md open question 3) | — |
| Ground drawn as two triangles per square with the height lookup's diagonals, corners clockwise from above | Behaviour notes `terrain.md` (diagonals) and `models-and-textures.md` Q2 (front faces) | Unit tests: every triangle's centre lies on the height lookup's surface; every triangle is clockwise from above |
| Material image value = direct index into the material list and the direction table; one texture per square; orientation codes 0–7 as in the note's table (u west→east, v north→south) | Behaviour note `terrain.md` "Ground textures" (answers Q8; session A measured it from edge matching) | Unit tests: the 8 codes are 8 different turns/mirrors; texture coordinates follow the code. Viewer log: 16,384 of 16,384 Daventry squares drawn, 30 textures, all 128×128 mipmapped, none missing. Checked by eye by the user 2026-10-11: Daventry's ground looks right, textures join up |
| Ground textures coloured with the palette `world.cs` names in `KQWorld::terrainPalette` | Behaviour note `models-and-textures.md` Q3 | Read from `world.cs` in the world's archives: Daventry gives `daventry.ppl` |
| Ground textures stop at the square's edge (no repeat); mipmaps not used; no lighting | Our choice (display only) | — |

## Object placement — `crates/kq8-data/src/objects.rs`, `crates/kq8-viewer/src/{load,terrain_view}.rs`

Behaviour note = `docs/behaviour/objects.md` (session A, answers Q9/Q10).

| Fact | Source | Checked how |
| --- | --- | --- |
| Position `locX/Y/Z` used as written; `isGroundRelative` and `keepOnGround` do not affect placement | Behaviour note (Q10) | Viewer log: of 104 placed Daventry objects, 21 rest within 50 units of the ground, 81 are higher (tables, hanging, inside buildings not drawn yet), 2 lower: `millwheel` 601 below (dips into the stream), `keg` 422 below |
| Angles `dirX/Y/Z` in radians, right-hand rule, turned about Y, then X, then Z (`RotY × RotX × RotZ`, row vectors); no scale | Behaviour note (Q9) | Unit tests: each axis and the order. Viewer self-check: Bevy transform vs the game's rule agree to 0.035 game units for all 104 objects |
| Bevy rotation = C · Rᵀ · Cᵀ (C = the axis change of `to_bevy`) | Our derivation | Same self-check |
| ~~Sections at (0, 0, 0) are skipped~~ **Replaced**: only objects loaded by the world's start-up scripts are placed | Behaviour note `objects.md` "Where a world's objects come from" (Q16) | `newWorld.cs` → `WorldVar.cs`, `WorldInv.cs`, `permwrld.cs`: 446 `loadKQ` lines, 0 inside if-blocks, 0 unreadable. 347 objects placed (was 170). Unit tests for script order, comments, names, positions, angles. Checked by eye by the user 2026-10-11: furniture complete, nothing misplaced or doubled |
| `loadKQ <file> [<name> [x y z [angles]]]`: `same`/missing keep the name, `@` none, else rename; a given position replaces `locX/Y/Z`, angles given or 0 0 0; the object is the file's first section | Behaviour note (Q16 rule 2); "first section" is our reading of the file layout | Observed: no loaded file has more than one section with a position |
| Ground rule with `loadKQ` positions: a z of exactly −1 or −2 is replaced by the ground height at (x, y), whatever `keepOnGround` says; other values as written, then the `keepOnGround` raise. A `.kq` file's own `locZ` does not take the −1/−2 rule | Behaviour note `objects.md` "Ground heights −1 and −2" (Q17) | Unit test. Viewer log: 79 Daventry objects put on the ground this way (was ~8,000 under it); "more than 500 below the ground" went from 83 to 4 (`Dock`, `Dock2`, `look`, `millwheel`: posts, foundation and wheel reaching into water or hillside) |
| Invisible objects shown as green (no model) or orange (`.anm` character) 1 m cubes, toggled with H, off at start | Our choice (viewer only; not a game rule) | Viewer log: 72 green, 5 orange in Daventry. `hidden=1` (`wizchest1`) is still drawn: Q18 |
| Missing `locX`/`locY`/`locZ` = 0, each on its own; then the usual ground rules (so a missing `locZ` with `keepOnGround` on = the ground height, if above 0) | Behaviour note `objects.md` "Missing locX / locY / locZ" (Q19) | Unit test. All 12 worlds: placed objects unchanged except Barren +3: `lookout` and `lookout2` (no `locZ`) both raised to 11,592, as the note says; `sparks16` (no `locX`, `keepOnGround=0`, z −1) ends up at x 0, about 20,000 under the ground. Objects with no position keys at all have no model (invisible-object cubes at the map corner) |
| Script variables: `set` (words joined by spaces), `$1`… script arguments, `$name` words after the first replaced and split again (quoted: one word); unknown variable = line not run (a `.cs` call still runs without it); `.kq` values that are `$variables` replaced the same way | Behaviour note `objects.md` "Script variables, getLoc and move" (Q20 b, c) | Unit test. All 12 worlds: 0 unreadable lines (was 184 in DeadCity, 3 Barren, 2 Swamp, 1 IceWorld). Barren: 18 `.kq` values replaced (anvil, coals, grinder) |
| `getLoc` / `KQObject::getLoc` set `$KQObject::loc` (x y z dirX dirY dirZ) and `$KQObject::locOnly` from the object's place after the ground rules; objects placed while the scripts run | Behaviour note (Q20 a); "after the `keepOnGround` lift" is not traced (no difference for the Weirdling) | Unit test. Barren: anvil, coals, grinder at the Weirdling's (144476, 131239, 1744), dirZ −1.43. DeadCity: 90 `getLoc` lines |
| `move <who> <flags> x y z [rx ry rz]`, flags 0–3 = absolute/relative position and turn; missing numbers 0; no −1/−2 rule | Behaviour note (Q20 d; Q17 correction) | DeadCity: 184 moves, e.g. `flamelh.1a` 14,890 → 16,890. IceWorld: `move Henchman` comes before Henchman is loaded (logged, not moved) |
| Relative move of a turned object: along world axes | Behaviour note `objects.md` Q20 (d), answer to Q21 (traced) | 17 DeadCity cases (torch flames `flamedt.*`, pots `pot.prs*`), logged |
| Relative turn about Z on an object turned only about Z: angles added | Our derivation (two turns about one axis add up in any order) | 9 DeadCity cases (`block12–15`, `DeadHal2`, doors `ltddoor4/6`, `rtddoor4/6`) |
| `loadKQ` with 4 or 5 numbers: x y z, extra ignored, angles 0 0 0 | Behaviour note (Q20 e) | Unit test; Swamp `swmp.01`, `swmp.02` placed |
| `hidden=<n>` (not 0 = hidden) and `KQObject::setHidden <name> 0\|1` in the start-up scripts; a hidden object is not drawn at the start unless `groupsAlways` (default 6) has bit 0x40 | Behaviour note `objects.md` "Hidden objects" (Q18); 0x40 = "drawn" is session A's inference, not traced | Unit test. Hidden at the start (shown only with H): Daventry 1 (`wizchest1`), Gnome 20 (bridges `rotbrdg`, `obridge*`, `gemfuse`, `gemlite.01–13`), Barren 50 (docks, rails, lava tiles), Snowexit 1 (`portal4`). Daventry: 1 `setHidden $NewInvItem 1` not run (variable set by an inventory command we do not run) |
| Unused objects (`.kq` files no start-up script loads, not at (0, 0, 0)) drawn with their own position and angles, hidden until H | Our choice (viewer only; the game does not have them at the start, objects.md Q16) | Viewer log: 16 in Daventry (`tap01/02/04` pictures, `pubart1`, `easel2`, `potion`, `Pyramid`...); each line says which event scripts name the file |
| `Cottage2–4` (`KQParticleCatcher`) and `wateran1–9` (`KQTerrainAnimator`, flowing river textures) are not drawn | Behaviour note (Q17 rule 5): no model, skip for M1 | Already skipped as "no position" / "no model" |
| Animated objects: `animLoops` → `[list]` `MovementK` names → `[movementK]` `loop=<model> ...`; model of the `currAnimName` movement | Observed in the data (e.g. `alchdoor.kq`) | Unit test; 30 Daventry objects found this way (17 doors, 8 animated objects, 3 chests, 2 portals) |
| Animated objects drawn in their model's first frame | Replaced 2026-10-11 by the next rows. (By eye before: most doors looked closed) | — |
| `loop=<model> <sequence> <start> <speed> <repeat> <fourth>`; start frame = round(start × frames), wraps if repeating, else stops at the last frame; no usable `currAnimName` → movement 0; `curAnimName` (one r) not read | Behaviour note `objects.md` "Animated objects at load" (Q12) | Unit tests. Viewer log: 140 objects placed (was 104); 35 start in movement 0; doors' `closed` = frame 0; only 2 objects start past frame 0 (`Pyramid` frame 63/70, `telepad1` frame 5/16) |
| Sequence name not found in the model → first loop | Behaviour note `objects.md` "Follow-ups (Q13)" (b): name looked up, sequence 0 if not found (traced) | 20 Daventry objects (before script placement) |
| Frame steps before a start frame are not applied | Behaviour note `objects.md` Q13 (a): a jump, likely without steps (not traced) | Only `Pyramid` affected (logged as WARNING) |
| `keepOnGround` (default on): an object below the ground is raised onto it, never lowered | Behaviour note `buildings.md` (Q10 follow-up) | Unit test. Daventry objects raised: 0 (the two below the ground, mill wheel and keg, have `keepOnGround=0`). Matters for buildings (step 3b) |
| ~~Material flag 0x10000 = 50% see-through~~ **Withdrawn**: 0x10000 halves the fog (haze) on that material | Behaviour note `models-and-textures.md` "Material flags" (corrected by session A, Q14) | Ignored until haze is drawn |
| Object `translucency=<n>` below 256: every material at opacity n/255 | Behaviour note `objects.md` "See-through objects" (Q14) | Viewer log: 9 Daventry objects (waterfalls 200, `water` 200, `wellwatr` 160, 5 torches 165). Distance fade not done (no haze yet). Teleport pad: still unknown (Q14 b) | 51 objects with `animLoops` but no `currAnimName` are skipped until Q12 is answered |
| Object palette from `KQWorld::objectPalette` in `world.cs` | Behaviour note `models-and-textures.md` (Q3) | Daventry: `daventry.ppl` |

## Buildings — `crates/kq8-data/src/building.rs`, `crates/kq8-viewer/src/load.rs`

Behaviour note = `docs/behaviour/buildings.md` (session A, answers Q11 and
the Q10 follow-up). "Checked" numbers come from `kq8 building-verify`
(`kq8-artifacts/logs/building-verify.log`).

| Fact | Source | Checked how |
| --- | --- | --- |
| `.dig` = PERS block `ITRGeometry`: build id, texture scale, bounding box, 8 counts, 8 tables (records 20, 8, 48, 1, 4, 12, 8, 16 bytes), +1 u32 if version > 3, +1 if version 4 | Behaviour note | Observed: all 153 files parse to exactly the end of their block. Versions 4 (151) and 2 (2); texture scale 0.125 in all |
| A PERS block that ends the data may lack its padding byte | **Observed** (new; the community PERS note says blocks are padded) | 61 of 153 `.dig` files have an odd size and end exactly at size + 8. `shape-verify` unchanged: all 4,573 models still parse |
| Surface record: flags (0x80 = faces opposite to its plane), material, texture size − 1, texture offset, plane, first corner, lighting ref, corner count | Behaviour note | Observed: corner order (anticlockwise = front) agrees with plane and 0x80 bit in 172,780 of 172,783 surfaces; corners within 0.50 units of their plane; texture coordinates all in 0..1; no index out of range |
| Texel = offset + uv × (size + 1); divided by the texture's size; textures repeat | Behaviour note ("likely", not traced) | Checked by eye by the user 2026-10-11: textures placed correctly |
| Front faces are **clockwise** seen from the visible side, as for models | Observed; behaviour note `buildings.md` rule 2 since corrected to match (Q15) | Drawn anticlockwise first: user saw every wall inside-out. `building-verify`: surfaces on the sides of each building's box have their right-hand-rule normal pointing into the box in 99.7% (e.g. roofs 6,069 vs 0). After switching to clockwise, checked by eye by the user 2026-10-11: outside and inside look right |
| Building palette `KQWorld::interiorPalette`; materials from the `material=` list (Daventry `interior.dml`, 52) | Behaviour note | Daventry: `daventry.ppl`; no texture missing; no material index past 52 in Daventry's buildings |
| Below-ground buildings raised onto the ground | Behaviour note (Q10 follow-up) | 19 Daventry objects raised (all buildings at `locZ` −1/−2). Lowest points well below ground: docks (posts in the water), `look`, `fenc3` |
| `ruins1/3/4.dig` named by Daventry objects but missing from the game | Observed | Not found in any archive or folder; skipped and logged |
| Building lighting (`.dil`) and visibility tables not used | Our choice for now | — |

## Formats read but not yet implemented

`libkq8fpc/doc/*.txt`: notes on `.msg`, `.kqs`, `.pba`, `.pft` and others
were read for an overview. Each fact taken from them will be listed here
when implemented.

## Connor: standing and gravity (M2 step 2a)

Code: `crates/kq8-game/src/mover.rs`, `crates/kq8-viewer/src/connor_view.rs`, `crates/kq8-viewer/src/load.rs` (`place_connor`).

| What | Source | Check |
| --- | --- | --- |
| Connor read from `GAME/Kq/Connor.kq` when the world script loads `connor.kq` (not in the world archives) | Behaviour note `connor-physics.md` "Data used"; the script line from Daventry `newWorld.cs` line 21 | Daventry: placed at (277744, 199295), z −1 → ground 13,633 |
| Standing model: movement `stop` of `GAME/Kq/Conner.anm` (same list format as objects' `animLoops`) → `c_stop.kqs` first frame; `KQWorld::connorPalette` | `connor-movement.md` (movement names), `models-and-textures.md` Q3 (palettes) | Log: `c_stop.kqs`, texture `CON0.BMP` found |
| Game logic at a fixed 15 updates per second; drawing blended between the last two updates | Our choice, recorded in `docs/DESIGN.md` M2 "Physics rate" (`connor-physics.md` "Do not reproduce") | 26 updates in 1.77 s of a test run |
| Gravity on at the start | `connor-physics.md` "Is gravity on at the start?" (recommendation; not traced) | Log line `gravity on` |
| One update: start (on ground → reset pull and air count, else count), up pass 325 + wanted z, across, down pass (gravity drop along the pull, then 325), ground check with the 0.8 walkable test | `connor-physics.md` "One update of the mover" | Unit tests; standing: z constant 13,633.5, on ground every update (ground normal z 0.996). With gravity off this rule hovers, which contradicts the note: Q23 |
| Fall timer +0.06 per update, drop `5000 × (t² − t_prev²) / 2`, at most 5,000; timer 0 on walkable ground | `connor-physics.md` "Gravity" | Unit test: drops 9, 27, 45 |
| Fall state after 5 updates in the air (timer set to 0.36), stuck count (< 50 per update, more than 5), ends on ground / stuck / moving up; death if more than 3,840 below the start | `connor-physics.md` "Falling off an edge" | Unit tests. Viewer `--lift 2000`: Fall at update 5, lands at update 14 after 1,856, safe. `--lift 5000`: lands at update 22 after 4,856, dies |
| Steep ground (normal z ≤ 0.8): not on ground, pull down the slope, gravity on | `connor-physics.md` steps 4–5 | Unit test of the downhill direction; sliding itself is step 2c |
| Test keys G / B (lift 5,000 / 2,000), C (camera behind him), `--lift N` | Our test helpers | — |

## Connor: walking, running, turning (M2 step 2b)

Code: `crates/kq8-data/src/action_map.rs`, `crates/kq8-game/src/connor.rs`, `crates/kq8-viewer/src/connor_view.rs`, `crates/kq8-viewer/src/load.rs` (`load_animation`, `place_connor`).

| What | Source | Check |
| --- | --- | --- |
| `.sae` action map: `[KEY]/[MOU] make\|break <key>`, `[ACTION] <name> <value> <n>`, `[ACTIONFLAG] <flag>` | Observed in `MASK/Connor.sae`; bindings in `connor-movement.md` "Controls" | Unit test; `Connor.sae`: 19 bindings, forward Up, back Down, left Left, right Right, run End |
| Intent from the keys **held**, last pressed wins | `connor-movement.md` "Do not reproduce" (recommended); no setting for the original behaviour (user's choice) | — (needs a keyboard) |
| Run toggles; starts on | `connor-movement.md` rules 3–4; Daventry scripts do not call `setRun` | — |
| Movement table (`run`, `walk`, `back`, `left`, `right`, `stop`) | `connor-movement.md` "Which movement plays" | Unit test |
| Speed = loop's frame steps added up / loop length × playback speed; spread evenly over the loop | `connor-movement.md` "Forward speed comes from the animation"; even spread: Q24 | Measured at load: walk 1,126.6, run 2,518.1, back −1,126.6 units/s (note: ≈ 1,127 and ≈ 2,518). `--hold 1,0,0`: 11,191 in 149 updates (= 1,126.6 × 149/15) |
| Turning 1.75 rad/s while a turn key is held | `connor-movement.md` "Turning" | Unit test; `--hold 0,1,0`: 3.383 rad in 29 updates |
| No start/stop transitions, new loop starts at 0 | Our choice until Q25 | — |
| All frames of the current movement drawn in turn | Frame layout from the shape (`models-and-textures.md`) | Not checked by eye yet |
| Terrain only in step 5 (ground check), not in the down pass | `connor-physics.md` as written; asked as Q26 | Downhill running: in the air every other update by about 1.4 units; steep slopes half-climbed (sliding is step 2c) |
| F: camera follows (keeps its offset); `--hold F,T,R` test option | Our test helpers | — |

