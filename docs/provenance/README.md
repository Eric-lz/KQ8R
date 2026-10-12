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
| ~~Engine forces first and last 8 colours to the Windows system colours~~ **Replaced 2026-10-11**: palettes are used exactly as stored | Behaviour note `models-and-textures.md` "System colours" (Q3): the swap (20 entries) happens only on a 256-colour screen | Textures coloured by a world palette that use entries 0–9/246–255: none in 11 worlds; Swamp `STN1.BMP` (35 pixels) and `WIRLPOOL.BMP` (16,512 pixels, all entry 0). Swamp `Teleport.BMP` uses entry 0 for 3,961 pixels. Entry 0 is stored blue: these now show blue where they were black. Conflict with the Q5 answer about entry 0: Q22. (Corrected 2026-10-12: `swamp.ppl` stores entry 0 black, so `WIRLPOOL.BMP` was never blue; only `Teleport.BMP`, through its own colour table, was.) |
| Entry 0 is black on any screen, in world palettes **and** in textures' own colour tables (`palette::as_shown`) | Behaviour note `models-and-textures.md` "System colours" (Q22, measured in the running original's memory, Daventry's world palette). The own-table part is our stand-in for consistency (Q42), chosen by the user | Unit tests `chunked_palette`, `riff_palette`. Export: Swamp and Daventry `Teleport.BMP` entry 0 = (0, 0, 0). Viewer log: both `Teleport.BMP` now `used_colours_differing_from_world_palette=0` (was 1). User: those pixels lie outside the round pad, not seen in game |
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
| Building lighting (`.dil`) and visibility bits (table 4) not used | Our choice for now | — |
| BSP nodes (table 2, 8 bytes): plane u16, front child i16, back child i16, 2 unused; a child below 0 is leaf −1 − child; the tree starts at node 0. Leaves (table 3, 48 bytes): first byte bit 0 = solid, the rest unused | `connor-physics.md` "Against a building" (traced, Q31, Q33 e) | Unit test on made-up bytes. `building-verify` (M2 step 2d-1), all 153 files: 30,400 nodes, 37,132 leaves; every index in range; node 0 is nobody's child; every other node has exactly one parent and is reachable from node 0; every plane normal has length 1.00000. Leaf first bytes: 0 (16,688), 1 = solid (13,865), 2 (6,579). Every leaf with 2 is one no node leads to, so it never matters for the trace; every reachable leaf is 0 or 1 with exactly one parent |

## Collision tests (M2 step 2d-2) — `crates/kq8-game/src/collide.rs`

Not used by Connor's movement yet (2d-3).

| Fact | Source | Checked how |
| --- | --- | --- |
| Ball stack from a box: r = half the larger of x/y size; n = height / 2r rounded up; ball k at min z + r + 2r·k, the last at max z − r. Connor: r 200 at 200, 600, 1,000, 1,100 | `connor-physics.md` "Collision shape" (traced, Q31 a, Q33) | Unit tests |
| Building test: no test under 0.1; own space (placement undone); each ball traced from node 0; front / behind / touching with margin r + 0.00001; split where the ball touches the plane moved by r; both children when both ends touch; a solid leaf entered across a plane is a hit, starting inside solid is not; normal = the crossed plane pointing against the move; earliest ball wins | `connor-physics.md` "Against a building" steps 1–7 (traced, Q31 b, c) | Unit tests on made-up trees (wall from both sides, floor, ceiling, lintel, start inside, exact touch, turned building). **Against the original:** `ConHouse` floor at session A's 3 points: feet at 13,980.0 (measured 13,980). `Tomb` wall: run head-on contact (237959.73, 187449.17) (measured 237959.7, 187449.2); walk head-on stop at (237909.4, 187437.8) (as measured); glancing contact y 187449.56 (measured 187449.4–187449.6). Throwaway program, Q36 |
| The touch margin is compared in 64-bit numbers | Our choice: in 32-bit, r + 0.00001 rounds back to r; the note gives the margin | — |
| Contact pulled back 25 towards the old position (the old position itself if within 25), for buildings and objects, not the terrain | `connor-physics.md` (traced, Q33 b); confirmed on floors (Q35) | Unit tests; the floor and wall checks above |
| The rough check uses the building's bounding box, placed, grown by √3 × 200 | `connor-physics.md` "First, a rough check" (traced) | Measured with a throwaway program on all 153 buildings: the BSP is open everywhere just outside the bounding box (2.3 million points, 1–400 units out, none solid), so the rough check can't change a result |
| Object test: each ball swept in the object's own space against its box enlarged by r on every face (square corners); exactly vertical moves test only the top ball (up or not moving) or the bottom ball (down); no test under 0.001 in every axis; a ball starting inside is a hit without a contact | `connor-physics.md` "Against another object" (traced, Q31 b, Q33) | Unit tests (square corner, turned object, vertical one-ball rule, start inside) |
| Inside-out boxes (min > max) give no hit | Our choice until Q34 (c) | Unit test |

## Buildings in Connor's movement (M2 step 2d-3) — `crates/kq8-game/src/{mover,connor}.rs`, `crates/kq8-viewer/src/{load,connor_view,terrain_view}.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| Up pass: the building sweep and the terrain (above the pass's end); the contact nearer the start wins; put at the contact, no slide | `connor-physics.md` "One update", "The terrain in the passes", "Exact hit handling" (traced) | Unit test: a ceiling at 1,500 cuts the up pass at feet 175 (top ball 25 short of it) |
| Across pass: buildings only (terrain not tested) | Same (traced, Q26) | Unit tests; viewer |
| Head-on: angle between this pass's horizontal movement (unit) and the reversed surface direction under 0.35 rad; never without horizontal movement; only in the Move state; by movement, not facing | "Exact hit handling" (traced, Q33 d) | Unit tests: head-on stop, backwards is head-on, not moving is never head-on. Viewer at the `Tomb`: 1.5° and 11.8° head-on, 21.6° glancing, as measured |
| Head-on (Move), up-pass hits, walkable landings (surface z ≥ 0.8): put at the contact. Otherwise: target = wanted end − n × ((wanted end − contact) · n), swept from the contact as an across pass, the same handling again for a new hit; on the 3rd hit in the update he stays at that contact; no slide if the rest is under 0.000001 in every axis | "Exact hit handling" (traced, Q33 c) | Unit tests: glancing at 21.6° and 35° slides 0.37 and 0.57 of the step (the measured factors), a corner, past the end of a wall. Viewer at the `Tomb`: run head-on contact (237959.7, 187449.2), as measured; glancing slide at y 187449.5–187449.6 from x 239937.0, as measured; walk head-on stop 210 from the face (measured: between 200 and 225 depending on the last free step) |
| Head-on outside Move counts as glancing; one count of 3 hits per update over all passes; the head-on test also on down-pass and slide hits | Our reading of the note; asked as Q37 (a)–(c) | — |
| Down pass: the building sweep and the terrain; the nearer contact wins; walkable (z > 0.8): on ground, fall timer 0; steep: gravity on, pushed off as above (buildings can now stop the push-off); the pull points down the contact's slope | "One update" step 4, "Exact hit handling" (traced) | Unit tests: a 300 step climbed (stands at 325), a 400 one stops him head-on; standing on a building floor stays 25 above it. Viewer in `ConHouse`: feet at 13,980 (measured 13,980), then walking 43 updates on the floor to a head-on stop at the inner wall |
| Ground check: the terrain only | "One update" step 5 (traced) | Unchanged |
| Walk or step back head-on: the movement switches straight to `stop` (no stop animation) | "Walls" (measured) | Unit test of the walker |
| ~~The next update the held key starts the walk again~~ ~~(2d-3b) that key direction stays blocked until pressed again; running not blocked; a mode change clears it; a turn key turns him on the spot~~ **Replaced 2026-10-12 (2d-3c):** two blocks. A head-on stop sets back blocked after `back`, forward blocked otherwise. While either is on he stands in `stop`: no walking, running, stepping back or turning on the spot. Only key events clear them: forward released → forward; back released → back; forward pressed → back; back pressed → forward; left or right pressed → both. Falling and landing don't | `connor-physics.md` "Walk held against a wall" (traced, Q38). 2d-3b was from the user's check of the original (it agrees: release and press forward walks again) | Unit tests (`a_wall_stop_blocks_until_the_key_is_released`, `while_blocked_he_neither_runs_nor_turns_on_the_spot`, `which_key_events_clear_which_block`); viewer at the `Tomb` with walk held: `stop`, `blocked fwd`, position unchanged |
| A held key is reported as pressed once (no keyboard auto-repeat), so a held turn key clears the blocks once | Our choice (Bevy's key input); the note leaves auto-repeat open | — |
| Not done: the Connor flag that clears both blocks on hits | Not traced which flag | — |
| Running head-on: stops at the contact, keeps running | Stand-in until 2d-4 (`crash`) | — |
| Every building the start-up scripts load is a collider, hidden ones too, placed as drawn | "Against a building", "When an object takes part" (hidden objects collide; traced) | Viewer log: Daventry's building colliders listed with their world boxes |
| Connor's balls from his `BOUNDS.KQ` box | "Collision shape" (traced) | Viewer log: box −200…200, −200…200, 0…1,300; balls r 200 at 200, 600, 1,000, 1,100 |

## Crash when running head-on (M2 step 2d-4) — `crates/kq8-game/src/connor.rs`, `crates/kq8-viewer/src/{connor_view,load}.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| Running (`run`) head-on into a wall: `crash` plays in full, keys ignored, then the keys take over (held run runs again and can crash again) | `connor-physics.md` "Walls" (traced, Q31 e; measured) | Unit test (`running_head_on_crashes_7_updates_441_5_back_then_runs`). Viewer at the `Tomb` (`--at 237968.9,187493.2,-0.2054 --hold 1,0,1`): contact (237959.7, 187449.2), crash ends at (238049.8, 187881.3); measured (238050.1, 187881.3). Crashes again every 10 updates with run held |
| `crash`: movement 7 of `Conner.anm`, `c_crash.KQS` loop `c_crash`, plays once, 8 frames, 0.4667 s, steps add up to (0, 441.5, 0): straight back | Observed (`GAME/Kq/Conner.anm`, `GAME/Common/c_crash.KQS`) | `viewer.log`: "connor: movement crash … 8 frames, 0.4667 s, steps (0.0, 441.5, 0.0)" |
| The crash starts with the update after the head-on hit | Our choice, the same as `Land` (the note doesn't say which update) | Viewer log: hit at update 1, `crash` updates 2–8 |
| While `crash` plays he is not in the Move state, so a wall behind him is never head-on (he slides or stops) | `connor-physics.md` "Exact hit handling" (head-on only in Move or Jump; crash is state Special) | Unit test (`a_crash_pushing_him_into_a_wall_behind_stops_there_without_head_on`) |
| Not done: the other run-like movements (`stoprun` done in 2g-1; `WaterRun`, `runwaterrun`, `runJump`, `PreBirth`), the exceptions (`KQMonster`, `backJump`, the Connor flag, the "no crash" object flag), the `crash.aud` sound and footstep cues | Not in our code yet | — |

## Objects in Connor's movement (M2 step 2d-5) — `crates/kq8-data/src/bounds.rs` (`lookup`), `crates/kq8-game/src/{collide,mover}.rs`, `crates/kq8-viewer/src/{load,terrain_view,connor_view}.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| Box lookup: keys `item0` … `item<nItems − 1>` only; names compared with case; inside-out boxes fixed on loading; duplicates `rat` = item 8, `razetta.kqs` = item 775 | `connor-physics.md` "How the table is read" (traced, Q34; duplicates read from the running game) | Unit tests (`lookup_follows_the_game`, `lookup_razetta_uses_item_775`). Swamp's `Lfswdoor` gets the fixed box (log) |
| A shape's name is lower-cased before the lookup; object names are not | **Our stand-in until Q39**: all 951 live `.kqs` entries of the table are lower case, and 25 Daventry objects (stone villagers, wagons…) write theirs with capitals | Viewer log: 25 such objects in Daventry, 60 in DeadCity, 12 in IceWorld, 7 Barren, 6 CastleD, 3 SnowExit, 1 Gnome, 1 Swamp |
| Box by the shape's name first (an animated object: its movement's shape when loaded), then the object's own name; none: never collides; fixed, not following the animation | "Where every object's box comes from" (traced, Q33 a) | Viewer log, Daventry: 295 object colliders, 24 without a box (e.g. the `Davtorch` torches, the gates `davgate1/2`, jugs) |
| Type and mask from `colliderType` / `colliderMask` in the object's section, defaults 0x41 and 0; type 0 and mask 0 never collides; a pair is tested when one's mask shares a bit with the other's type; Connor type 4, mask 0x281B | "Which pairs collide", "When an object takes part" (traced) | Unit test (`which_pairs_collide_with_connor`). Viewer log: per world, left out by type and mask (Daventry: `keepwall`, `pondclck`, type 0x20) |
| A value like `97		#World, Click, Attack` is read as 97 | Our reading (the note gives only the key) | Viewer log: Barren and IceWorld `eleswtch` type 0x61 |
| The script commands `collisionType` / `collision` | Not run: no start-up script of the 12 worlds uses them | Viewer log, "other commands" of every world |
| Hidden objects collide | "When an object takes part" (traced, not checked in the game) | Viewer log: Daventry 1 (`wizchest1`), Barren 50, Gnome 20, SnowExit 1 |
| `KQMonster`s get no box (ball stacks; Connor against them is not traced) | "Collision shape", "Not traced" | Viewer log: DeadCity 1 left out |
| Object test in the passes: the same sweep handling as buildings; over buildings and objects the contact nearest the old position | "Against a building" step 7, "Exact hit handling" (traced) | Unit tests: head-on stop 225 from an object's face, glancing slide, standing on a crate 25 above it. Viewer: `signpst.05` from east and north, stopped 225 from the turned box's face; DeadCity `crate.01` (turned 25.8°) glanced, as the 20° limit says |
| A ball that starts inside an object's box: no hit (he can walk out), logged | **Our choice** (the note: "a hit with no usable contact", what follows not traced) | Unit test (`starting_inside_an_object_is_no_hit_and_is_noted`) |
| Rough check for objects: the box enlarged by r, turned into the world | Our choice, only to save time (never skips a hit) | Unit test (`rough_check_never_skips_a_turned_box_corner`) |

## Start transitions (M2 step 2g-1) — `crates/kq8-data/src/objects.rs` (`transition_lines`), `crates/kq8-game/src/connor.rs`, `crates/kq8-viewer/src/{load,connor_view}.rs`

Behaviour note = `docs/behaviour/connor-movement.md` "Starting and
stopping" (session A, traced, Q25).

| Fact | Source | Checked how |
| --- | --- | --- |
| `transitionN=<from> <to> <to phase> <via>` lines, `nTransition` of them, in each `[movement<k>]`; `;toFramesN` lines are comments | Note; layout observed in `GAME/Kq/Conner.anm` | Unit test (`transition_lines_as_in_conner_anm`). Viewer log: `stop` → `run` via `stoprun` at 0.866667, → `walk` via `stopwalk` at 0.85 |
| `loop=` 6th field = "can be cut short"; transition animations are `0 0` | Note ("The two flags") | Viewer log: `stopwalk`, `stoprun` false; `stop`, `walk`, `run` true |
| A transition is kept only when its `<to>` and `<via>` movements are loaded; the others are skipped and logged | Our choice (water and the stop animations aren't loaded yet) | Viewer log: walk's and run's stop transitions and the water ones "skipped (movement not loaded)" |
| Rules 1–6: ask only when the keys' choice changes; next from point ahead (wrap on a loop, never on a one-shot, −1 always wins, ties to the first listed); the movement plays up to the from point; no transition → at once from 0 if it can be cut short; at the end `<to>` at `<to phase>`, then a different wanted movement is asked for again | Note, rules 1–6 (traced) | Unit tests (`choosing_the_next_from_point_ahead`, `letting_go_during_stopwalk_…`, `run_switched_on_during_stopwalk_…`) |
| The transition plays from the update **after** the one where the from point is reached (so −1 gives one update of `stop` with no step) | **Our reading** of rule 3; fits the measured "one update of `run`, then `stoprunstop`". Asked as Q40 (a) | Viewer log: update 1 `stop`, `stopwalk` from update 2 |
| Asking for the movement already playing drops a waiting transition and changes nothing | **Our choice**, not traced; asked as Q40 (b) | Unit test (`pressing_forward_again_while_a_stop_waits_walks_on`) |
| One-shot animations move by their own frame steps, frame by frame (frame k's step over frame time = play time / (frames − 1)), both axes; +x = his left | Note "Steps in the start animations" | Unit tests; viewer `--hold 1,0,0`: `stopwalk` 7 updates of (7.32 left, 11.22 forward), 51.2 / 78.5 in all, then `walk` from 0.85. `--hold 1,0,1`: `stoprun` 10 updates with the note's steps, 43.9 / **526.7**, then `run` from 0.8667 |
| `stoprun` adds up to 526.7 forward, not the note's 536.7 | **Observed** (`GAME/Common/C_stprn.kqs` frame steps; the note's own per-frame table agrees) | Viewer log frame steps; asked as Q40 (c) |
| This also changes `jumpend`: its whole step (78 forward, 291 down) is in frame 1, so it is now taken in the first landing update instead of spread over 10 | Data (`C_jstp.kqs` frame steps), same rule | Viewer log frame steps. Not checked by eye |
| Loops (`walk`, `run`, …) keep the even spread | Q24 only partly answered | Unchanged |
| Head-on during `stopwalk`: `stop`, forward blocked; during `stoprun`: `crash`; either drops the transition | Note ("Walking into a wall during `stopwalk`…"), `connor-physics.md` "Walls" | Unit test; viewer `--hold 1,0,1`: `stoprun` into `tree.06` at update 29 → `crash` |
| He turns while a transition plays | Note ("Turning during a start") | Unit test |
| Falling, a test lift or the end of a mode's one-shot (`crash`, `jumpend`) leaves him in `stop`, and the keys ask again from there | After `crash`: fits the measurement (the original plays `stoprun` before `run`, DESIGN 2d-4). After `jumpend`: **our choice**, not traced | Unit test (`after_a_crash_with_run_held_stoprun_plays_before_run`); viewer: crash ends update 19, `stop` 20, `stoprun` 21– |

## Stop transitions (M2 step 2g-2) — `crates/kq8-game/src/connor.rs`, `crates/kq8-viewer/src/{load,connor_view,main}.rs`

Same note and rules as 2g-1; this step loads the six stop animations
(`walkstopleft`, `stopwalkstop`, `walkstopright`, `runstopright`,
`runstopleft`, `stoprunstop`), so walk's and run's stop transitions are
no longer skipped.

| Fact | Source | Checked how |
| --- | --- | --- |
| Letting go waits for the next from point ahead (walk 0.45 / 0.90 / 0.95, run 0.0667 / 0.6 / 0.8667, wrapping), then plays that point's animation to its end, then `stop` from 0 | Note rules 2, 3, 5, 6 (traced) | Unit test (`the_stop_is_picked_by_the_next_from_point_after_letting_go`, all six). Viewer `--at 277744,199295,3.14 --hold 1,0,1,20`: release at run phase 0.51 → `runstopleft` at 0.6; `--hold 1,0,0,20`: release at walk 0.48 → walk to 0.9, then `stopwalkstop` |
| Let go during `stopwalk`: walk at 0.85, cut at 0.9 one update later, `stopwalkstop`. During `stoprun`: one update of `run` with no step (cut at 0.8667, distance 0), then `stoprunstop` | Note, worked examples; matches the measured "one update of `run`, then `stoprunstop`" | Unit tests; viewer `--hold 1,0,0,4` and `--hold 1,0,1,4`: as described |
| Stop animations step by their own frames; `stoprunstop` goes to his right (about 35) | Data (`c_strst.kqs` frame steps); the note's measurement ("almost straight to his right") | Viewer `--hold 1,0,1,4`: −10.45, −8.21, … left (i.e. right), 35.2 in all |
| From the moment `stop` is asked for he is in Wait, not Move, so a wall while a stop waits or plays is never head-on (no `crash`, no wall block) | Note open question 2 (measured: "state already Wait" while the walk or run plays on); `connor-physics.md` "Exact hit handling" (head-on only in Move/Jump) | Unit test (`no_head_on_while_stopping`) |
| Forward pressed during a stop animation: it plays to its end, then `stop`, `stopwalk`, `walk` | Note rules 4 and 6 | Unit test |
| `--hold F,T,R,N`: let go of forward and turn after N updates | Our test helper | — |

## The standing jump (M2 step 2e-1) — `crates/kq8-game/src/{player,connor,mover}.rs`, `crates/kq8-viewer/src/{connor_view,load,main}.rs`

Behaviour note = `docs/behaviour/connor-physics.md` "Jumping" (traced;
arcs measured) and "Falling off an edge"; the user's checks in the
original (2026-10-12).

| Fact | Source | Checked how |
| --- | --- | --- |
| The update tying walker and mover together (falls, landings, head-on) moved from the viewer to `kq8-game` `player.rs`, unchanged | Our refactor, so jumps can be unit-tested | All earlier tests pass; viewer runs as before |
| Jump key = `Connor.sae` action `Jump` (Insert) | Observed; note "Controls" | Viewer log: `jump Some(Insert)` |
| A jump only in the keys' control, with no start or stop animation playing, from `stop`, `walk`, `run` (and `back`, `left`, `right`) | **User's check in the original** (no jump during start/stop animations; standing, full walking, full running; from turning on the spot: allowed, turning stops, and starts again after the landing); `back` from rule 2; the wait before a stop: our reading, Q41 (a) | Unit test; viewer `--hold 1,0,1 --jump 4`: "refused: stoprun is playing" |
| On the ground, or within 250 above the terrain (its normal then counts); walkable, or steep and not facing uphill | Rule 1 (traced); "feet to the terrain under him" is our reading, Q41 (c) | Unit test (300 refused, 200 allowed; 45° slope facing up refused, facing down allowed) |
| `runJump` from `run`, `backJump` from `back`, else `jump` (walking too) | Rule 2 | Viewer `--hold 1,0,0 --jump 20`: `jump from walk`; from running: "runJump is not loaded yet" (2e-2) |
| Jumping: gravity off, step up and down 0, the arc = the animation's frame steps (x, y, z); keys ignored, no turning | Rule 3; no turning: **user's check** | Unit test; viewer `--jump 5`: 20 updates of `jump`, still for 9, top +1,697.0, end +872.8 and 1,423.4 forward (data: 1,423.4, 872.8) |
| The update after the animation ends: gravity on, steps 325, Fall with the death limit from 1,280 below there, fall timer 0.36 (previous 0.30), so the first drop is 396 | Rule 4; "Falling off an edge" rule 2; the timer measured in the original | Unit tests (drop 396; landing 4,873 below the end safe, 5,373 fatal); viewer: first Fall drop 396.0 |
| No Fall request from the air counter while jumping | "Falling off an edge" rule 1 ("unless he is jumping"); gravity is off, so the counter stays 0 | Unit test (20 jump updates, no Fall) |
| Hits during a jump: walls, ceilings and the ground check act; glancing only (head-on and steep hits during jumps come in 2e-3) | Rule 3 ("walls and ceilings still stop him"); the rest: stand-in until 2e-3 | — |
| `--jump N`: test option, presses Jump once | Our test helper | Log line `test --jump` |

## Running and backward jumps (M2 step 2e-2) — `crates/kq8-game/src/{connor,player}.rs`, `crates/kq8-viewer/src/load.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| `runJump` from `run`, `backJump` from `back`; hold, landing and death animations by kind (`runJumpHold`/`runJumpDie`; `backJumpHold`/`backJumpEnd`/`backJumpDie`); `runJump` has no landing animation | `connor-physics.md` "Jumping" rules 2, 4 (table) | Unit tests; viewer `--at 277744,199295,3.14 --hold 1,0,1 --jump 20`: `runJump` 20 updates, top +1,858.8 (note 1,859); `--at 277744,199295,0 --hold -1,0,0 --jump 10`: `backJump` 18 updates, 2,565.7 back, top +2,156.1, end +788.1 (note 2,566, 2,156, 788), `backJumpEnd` 11 updates |
| After a running jump lands he is in the keys' control at once, still in `runJumpHold`; the keys' choice is asked for from there, so `Conner.anm`'s transitions give `jumprun` (run held) → `run` at 0.8, `jumpstop` (forward let go) → `stop`, `walk` at once (run off) | Note's measurement (open question 3: `jumprun` 3 updates, `jumpstop` 13 updates ≈ 1.0 s); the 2g rules; **our reading**, the landing itself not traced | Unit tests; viewer: `jumprun` updates 43–45 then `run`; with forward let go (`--hold 1,0,1,30`) `jumpstop` updates 43–57 then `stop` |
| At that landing a turn key alone counts as standing: `jumpstop` plays and he turns during it, then the turn on the spot | **User's check in the original** (turning only during the running jump's landing, forward let go, a turn key held); how the game does it: Q41 (d) | Unit test (`running_jump_lands_into_jumpstop_with_forward_let_go_and_turns_there`) |

## Hits during a jump (M2 step 2e-3) — `crates/kq8-game/src/{mover,player,connor}.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| Head-on acts while jumping (state Jump) as in Move, and any surface with normal z < 0.8 counts as head-on; never during `backJump` | `connor-physics.md` "Walls" (traced) | Unit tests |
| Up-pass hits during a jump are tested too (a vertical move is never head-on by angle, so only the steep rule; a ceiling counts) | "Exact hit handling" (head-on is tested on up and across passes, Q37 c) | Unit tests (through the wall tests) |
| The first steep hit (normal z < 0.8) in any pass ends the jump: gravity on, steps 325 | "Jumping" rule 5 (traced) | Unit tests |
| Down-pass steep hit during `jump` or `runJump`: `crash`. Up or across: `runJump` crashes, `jump` stands (`stop`, forward blocked as for a walking head-on stop) | Rule 5; the forward block from "Walk held against a wall" (Q38) | Unit tests (wall, 63° hill); viewer `--at 277744,199295,0 --jump 3`: `jump` hits `tree.06` (across), stands, falls 5 updates later, `jumpend`; `--at 277744,201500,0 --hold 1,0,1 --jump 13`: `runJump` hits it, `crash` |
| `backJump` into a wall or steep hill: no head-on (it slides), gravity on and steps 325 (rule 5), but the animation plays on to its end (state still Jump), then Fall as at any jump's end (`backJumpHold` until the ground, usual death rule) | **User's check in the original** (no crash, no stumble, the flip keeps going, then the falling animation; Q41 b). Gravity on during the rest of the flip: rule 5 as traced, not visible to the user | Unit test (`a_back_flip_slides_along_the_wall_and_plays_on`); viewer `--at 277744,199295,3.14 --hold -1,0,0 --jump 2`: hits `tree.06` at update 13, `backJump` plays on to update 20, then `backJumpHold`, `backJumpEnd` |

## Collision boxes — `crates/kq8-data/src/bounds.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| `GAME/patch/BOUNDS.KQ`, `[BoundingBoxes]`: `nItems`, then `itemN=<name> xmin ymin zmin xmax ymax zmax`; `;` lines are comments; fields separated by one or more spaces | `connor-physics.md` "Where every object's box comes from" (traced, Q33 a); spacing observed | Unit test. `building-verify`: 1,111 entries, `nItems=1111`, numbered 0–1,110 each once, no unreadable line. `Connor` = −200 −200 0 200 200 1,300 (two older 1,600 lines are `;` comments) |
| `#item968=...` is not an entry (its key isn't `itemN`) | Our reading of the format; the note says only that `;` lines are comments | The file also has a normal `item968=Althillman` line; the `#` one is an older box for the same name |
| Every entry is kept; lookups return all entries with the name, ignoring case | Our choice until Q34 (which duplicate wins, case) | Duplicates in the data: `henchman`/`Henchman`, `rat`, `razetta.kqs` (boxes differ); `sprigan`, `trip2`–`trip5` (same). Inside-out boxes (min > max): `lfswdoor.kqs` and `iceshard.kqs` |

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
| Fall timer +0.06 per update, drop `10000 × (t² − t_prev²)` (in code `GRAVITY` 20,000 with `/ 2`), at most 5,000; timer 0 on walkable ground | `connor-physics.md` "Gravity" (strength measured in the running original, Q30; was 5,000 traced until 2026-10-12, step G1) | Unit test: drops 36, 108, 180 |
| Fall state after 5 updates in the air (timer left running on, since step G2 2026-10-12; before, it was set to 0.36, which the note now gives for jumps only), stuck count (< 50 per update, more than 5), ends on ground / stuck / moving up; death if more than 3,840 below the start | `connor-physics.md` "Falling off an edge" | Unit tests (`walking_off_a_ledge_keeps_the_fall_timer`: drops 36 … 324, then 396, 468). Viewer `--lift 5000` after G1 + G2: Fall at update 6, lands at update 12 after 4,100, dies. (Before G1: `--lift 2000`: Fall at update 5, lands at update 14 after 1,856, safe; `--lift 5000`: lands at update 22 after 4,856, dies) |
| Steep ground (normal z ≤ 0.8): not on ground, pull down the slope, gravity on | `connor-physics.md` steps 4–5 | Unit test of the downhill direction; sliding itself is step 2c |
| Test keys G / B (lift 5,000 / 2,000), C (camera behind him), `--lift N`. The lift keeps "on ground" as it was (since Q29), so the first update after it isn't counted as in the air | Our test helpers | `--lift 2000`: Fall at update 6, safe after 1,775; `--lift 5000`: dies after 4,775 |

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
| ~~No start/stop transitions, new loop starts at 0~~ **Replaced by 2g-1** (start transitions, below) | Our choice until Q25 | — |
| All frames of the current movement drawn in turn | Frame layout from the shape (`models-and-textures.md`) | Not checked by eye yet |
| Terrain only in step 5 (ground check), not in the down pass | `connor-physics.md` as written; asked as Q26 | Downhill running: in the air every other update by about 1.4 units; steep slopes half-climbed (sliding is step 2c). **Replaced in 2c** (below) |
| F: camera follows (keeps its offset); `--hold F,T,R` test option | Our test helpers | — |

## Slopes and falls (M2 step 2c, falling part of 2e)

Code: `crates/kq8-game/src/mover.rs`, `crates/kq8-game/src/connor.rs` (`Mode`, one-shot animations), `crates/kq8-viewer/src/connor_view.rs`, `crates/kq8-viewer/src/load.rs` (`CONNOR_MOVEMENTS`, `load_animation`), `crates/kq8-viewer/src/main.rs` (`--at`).

| What | From | Checked |
| --- | --- | --- |
| Terrain tested at the end of the up and down passes (a hit when it is above where the pass ended; contact = end x, y at terrain height; normal = terrain's), not in the across pass | `connor-physics.md` "The terrain in the passes" (Q26) | Unit tests; Daventry hill (`--at 286169,190211,0.291 --hold 1,0,1`): steep down-pass hit every update on the slope |
| Up-pass hit: he stops at the terrain, nothing else | Our reading: the note gives no response outside jumps | Unit test (started 1,000 below the ground); never seen in the viewer runs |
| Walkable down-pass hit: on the contact point, on ground, fall timer 0 | `connor-physics.md` "One update" step 4 | Unit tests |
| Steep down-pass hit: pull down the slope, gravity on, push-off = rest of the move (end − contact) with its part along the normal removed, as an across move (terrain ignored) | `connor-physics.md` step 4, "The terrain in the passes", "Walls" (glancing) | Numbers measured before the 4× gravity (G1, 2026-10-12), not redone: unit test at 45°: gains 79.5, 64.9, 52.2, 39.5, 26.7, then Fall and slide back. Daventry hill (normal z 0.610): climbs 60, 60, 45, 31, 17, then Fall at update 17, slides back, lands after 214 units |
| ~~Downhill rule~~ **removed 2026-10-12**: with the measured gravity it isn't needed. Going down a walkable slope he is in the air for an update or a few and lands again, never 5 in a row | `connor-physics.md` open questions 6 and 7 (measured in the original), Q32 | Unit test at 5°, 20°, 30°, 36°, walking and running: air count under 5, no Fall. Daventry 36° slope (`--at 293994,218295,-1.222`): walking, off every other update, air count at most 1; running, off up to 3 in a row, at most 3. The original: 1, and 1–4. (Before removal: 21 snaps, air counter always 0) |
| Gravity skipped (timer 0) when the pull is less than 30° below horizontal; the vertical drop d projected onto the pull (d·sinθ along a slope) | `connor-physics.md` "Gravity" (projection: Q28 a) | Unit test at 45° (since G1, 2026-10-12): gains 66, 30, −6, −42 from the note's rule 168·cos²θ − d·sinθ·cosθ, d = 36, 108, 180, 252; back on the flat in update 5, no Fall. (Before G1: 79.5, 70.5, 61.5, 52.5, 43.5, the note's old example) |
| Ground check no longer resets the fall timer | `connor-physics.md` "Gravity" (last rule) | Unit test |
| Fall asked for at the start of the update after 5 updates in a row that ended not on the ground (only walkable ground counts), on every update from then on; accepted in normal play; no speed or height condition | `connor-physics.md` "Falling off an edge" rule 1 (Q29) | Unit test: lifted, Fall at update 6. Daventry hill, walking head-on: 1 fall (the first approach, 5 updates in the air) in 101 updates, then in and out of the slope with the walk playing |
| Fall decided before the animation's step is taken, so its first update uses `jumphold`'s step, not the walk's | Our choice (the order is not in the note). Without it, the walk step of that update lifted him up the hill, "moved up" ended the fall at once, and he crept up the hill forever (seen in a unit test run) | Unit test of the 45° hill |
| `jumphold` (repeats) while falling, `jumpend` once on a safe landing, `jumpdie` once then the Die state on its last frame; keys (also turning) ignored meanwhile | `connor-physics.md` "Falling off an edge" rules 2, 5; ignoring the keys and turning: our stand-in (asked as Q28 c) | Unit tests; `--lift 2000 --hold 1,0,0`: jumphold at 5, landing at 14 (1,875), jumpend 10 updates, walk again. `--lift 5000`: lands at 22 (4,856), jumpdie 10 updates, Die |
| One-shot animations play first to last frame (`frameCount − 1` steps). ~~Over `frameCount − 1` of `frameCount` frame times of the stored length~~ **Corrected 2026-10-12 (2d-4):** over the whole stored loop length, moving the whole step sum | `objects.md` "Animated objects at load" loop rule 2 (steps); the length is Observed in the data (every loop's stored length is (frames − 1) / 15 s: `crash` 8 frames 0.4667 s, `jumpend` 11 frames 0.6667 s, `walk` 20 frames 1.2667 s) and Measured (`crash`: 441 in 7 ticks of 63, `connor-physics.md` "Walls") | Unit tests; viewer: `crash` 7 updates, 441.5 |
| A loop name missing in the model: the first loop plays | `objects.md` Q13 (b) | `C_jhld.kqs` has no loop `C_jhld`; the log says which loop is used |
| `jumpdie` steps not applied: the mover stops at death | Our choice (asked as Q28 b) | — |
| `--at X,Y,H` start option; spots from a throwaway terrain scan (not kept) | Our test helper | Log line `test --at: put on the ground at ...` |

## The following camera (M2 step 2f)

Code: `crates/kq8-game/src/camera.rs`, `crates/kq8-viewer/src/game_camera.rs`, `crates/kq8-viewer/src/load.rs` (`script_range`, camera start).

| What | Source | Check |
| --- | --- | --- |
| Aim 1,400 above Connor's base; normal mode stays put inside min..max, pulled to max − 1 / pushed to min + 1 (3D) | `camera.md` "What the camera looks at", "Distance (normal mode)" | Unit tests. `--hold 1,0,1`: distance 4,999.0 while he runs away |
| Range 1,500–5,000 (the note's value before any script sets one), the same walking and running | `camera.md` "Distance" rule 2; `walk.cs`/`run.cs` are **not** tied to the run switch: the user saw no difference in the original (Q27 e) | — |
| Height: higher of terrain under the camera + 1,400 and Connor's base + 1,400; within 100 jump, else 10% of the gap per update | `camera.md` "Height above the ground" | Unit test; run log: never below 1,400 above the terrain |
| Right mouse: swing 3π/4 per unit round the aim point, zoom 5,000 per unit (in range, never past the aim), tilt 0.75 per unit in fixed mode or with Shift; first update after pressing uses no movement | `camera.md` "Swinging and zooming with the mouse" | Unit tests. Mouse units = pixels ÷ 600 is a stand-in (Q27 a). Directions checked by eye by the user against the original: mouse left = swing left, mouse down = zoom in. Mouse tilt (Shift or fixed mode): mouse down tilts like Page Down, also checked by eye |
| Page Up/Down tilt 0.015 rad per update (fixed mode: 125 closer/farther), ±1.57, Home resets, snaps back when Connor moves more than 10 in normal mode; F11 fixed distance | `camera.md` "Tilt", "Distance (fixed mode)" | Unit tests. What the tilt turns (we pitch the view by tilt − 0.2) is a stand-in: Q27 b |
| Keys from `MASK/Camera.sae` (F11, `prior`/`next` = Page Up/Down, Home) and `Connor.sae` (right mouse `MoveCamera`, Shift `ModifyDn`) | Observed in the data | Log: keys found |
| Start where the world script loads `cam3rd.kq` (`KQCamera`); follows Connor although Daventry's script ends with `follow none` | Observed in the data; Q27 d | Daventry starts at (275026, 197541, 14930), 3,147 from the aim point |
| Camera logic at 15 updates per second after Connor, drawn blended | `docs/DESIGN.md` M2 "Physics rate" | — |
| Not done: obstacle check (2d), swing back behind Connor (Q27 c), first person, shake, cutscene cameras | — | — |

