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
| A model whose start frame has no triangles (Daventry `Map1.KQS` frame 0, the Wizard's magic map) loads with nothing drawn in that frame; its box is every frame's together; no triangles in any frame: no model (2026-10-24) | Observed (the data); the box is **ours** | Viewer: `magicmap` loaded, "box from all 40 frames" (before: skipped) |

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

## Graphics API choice, `--renderer` — `crates/kq8-viewer/src/{renderer,main}.rs`

| Fact or choice | Source | Checked how |
| --- | --- | --- |
| `--renderer auto\|vulkan\|dx12`, default `auto` (wgpu chooses; `WGPU_BACKEND` still read) | Our choice, asked for by the user; nothing from the original game | `viewer.log` 2026-10-16: auto → Vulkan, vulkan → Vulkan, dx12 → Dx12, auto with `WGPU_BACKEND=dx12` → Dx12, vulkan with `WGPU_BACKEND=dx12` → Vulkan. Checked by the user 2026-10-16 (one NVIDIA card, Windows only) |
| OpenGL refused | Our choice: Bevy built without `gles`; `WGPU_BACKEND=gl` crashed with "Unable to find a GPU" | Run 2026-10-16 |

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

## Ground lighting maps (M1 step 4a) — `crates/kq8-data/src/{lightmap,terrain}.rs`, `crates/kq8-cli/src/lightmap_verify.rs`

Behaviour note = `docs/behaviour/terrain-lighting.md` (session A,
2026-10-13); rule numbers are the note's. "Checked" numbers come from
`kq8 lightmap-verify` (`kq8-artifacts/logs/lightmap-verify.log`).

| Fact | Source | Checked how |
| --- | --- | --- |
| `TERRAIN.KQ` names four maps: `lightmapName`/`shadowmapName` (hardware), `SWlightmapName`/`SWshadowmapName` (software); one pair is used | Note rule 1 | 6 worlds name files, 6 have empty names; all 24 named files found in `resource.vol` |
| `useDynamicLighting 1` → shadow map, 0 → light map; the game starts at 0 | Note rule 2 | `Options.cs` has `useDynamicLighting 1` (read by `lightmap::use_dynamic_lighting`, last line wins; unit test) |
| Empty name or unusable file → every channel 15 | Note rule 4 | `LightMap::full`; unit test |
| File = u32 count + count × u16, little-endian; count must be side × side | Note rule 5 | All 24 files: count = side², file size = 4 + 2 × count; unit test refuses a wrong count and a short file |
| side = 128 × 2^lightScale + 1; spacing = 2^groundScale / 2^lightScale | Note rule 6 | Daventry and Dead City 513 (spacing 1,024 / 512), Swamp 257, the rest 129, as in the note's table |
| Value j × side + i = point (i, j) at (i × spacing, j × spacing) | Note rules 7–8 | Unit test; the order itself is the note's (checked by session A against the height map's shadows) |
| Value = brightness ¦ red ¦ green ¦ blue, 4 bits each | Note rule 9 | Unit test. Daventry shown (shadow) map red: 4 ×20,127, 6 ×4,598, 7 ×236,889, 8 ×1,555, exactly the note's "How to check" numbers; light vs shadow differ in 761 points, as rule 3 says |
| Between points: bilinear in world coordinates | Note rule 11, **stand-in** (open question 2: the original's filtering not traced) | Unit test of the blend (`colour_at`) |
| Missing `lightScale` → 0 | **Our stand-in** (the note gives no default) | Every world's file has the line |

## Ground drawn lit (M1 steps 4b, 4c) — `crates/kq8-data/src/terrain.rs` (`square_cells`), `crates/kq8-viewer/src/{light_view,terrain_view,ground_anim_view,load,main}.rs`

Behaviour note = `docs/behaviour/terrain-lighting.md` (rule numbers are
the note's).

| Fact | Source | Checked how |
| --- | --- | --- |
| Hardware pair; shadow map when `useDynamicLighting 1` | Note rules 1–2 | Viewer log: Daventry `daveshdw.bin`, Swamp, Dead City, Ice World shadow maps; Barren: no name, full brightness |
| Texture × light colour, channel by channel (÷ 15); top 4 bits not used | Note rule 10 | Viewer log at Connor: Daventry start 7 → 93% (x2) / 47% (x1); a hill shadow (208,896, 284,672) 4 → 53% |
| **×1 (channel ÷ 15) by default; ×2 (capped at 100%) and off on F7 / `--light`** | **User's check by eye** (2026-10-14: ×1 matches the original, which is very dark and muddy), against the note's suggested ×2 (open question 1) and the data in Q65; correction request Q66. Still a stand-in until session A confirms | Unit test of the three modes |
| Map points are mesh corners; inside a cell the colour is blended over the cell's two triangles | **Stand-in** for the note's assumed bilinear filtering (open question 2) | Unit test: the cells are the same ground (n = 1 equals the two triangles; corners and centres on the height lookup; each cell inside its triangle; texture coordinates follow the code). Viewer: walking, the light under Connor changes smoothly (8 → 7 → 6.2) |
| Multiply on the colour as shown (sRGB), turned into Bevy's linear space per corner | **Our choice / stand-in** (a 1998 card has no gamma step; sRGB of a product is not exactly the product of sRGBs) | — |
| Brightness setting (`setBrightness`) ignored | **Stand-in** (open question 3) | — |
| No tone mapping (`Tonemapping::None` on every camera) | **Our choice** (display only: colours as computed, so ×1 / ×2 can be compared by eye) | — |
| Ground animators' squares (Daventry's river) and the U key's floor view are lit like the rest of the ground | Note rule 10 ("when a ground square is drawn") | Viewer log: 872 lit meshes in Daventry, including the animators' |

## Object and building lights read (M1 step 5a) — `crates/kq8-data/src/{lights,dil,building}.rs`, `crates/kq8-cli/src/lighting_verify.rs`

Behaviour note = `docs/behaviour/object-lighting.md` (rule numbers are
the note's). Numbers from `kq8 lighting-verify`
(`kq8-artifacts/logs/lighting-verify.log`).

| Fact | Source | Checked how |
| --- | --- | --- |
| On hardware the lights are the files that `swtohwlt.cs` loads (`loadKQ` lines) | Note rule 1 | Observed: all 12 worlds' `swtohwlt.cs` load `light/SunlgtHW.kq` and `light/AmblgtHW.kq`; both found in every world |
| Ambient = sum of all ambient lights; suns = `KQLight` with `tsLightType=1`; point lights = `KQPointLight` with `tsLightType=2` | Note rules 3, 24 | Unit tests; log: one ambient and one sun per world; Dead City's 10 point lights are in its `Light` folder but not loaded by `swtohwlt.cs` |
| Corner light = ambient + sun colour × max(0, −aim · normal), capped 0–1 per channel; aim not made unit length | Note rules 4, 5 | Unit tests; log: Daventry 0.300 / 0.366 / 0.420 (away / up / into the sun) = the note's table; Swamp into the sun 1.02 → 1.000 |
| Flat grey 0.299 R + 0.587 G + 0.114 B, capped at 1 | Note rule 7 and "Do not reproduce" (usual weights) | Unit test |
| A missing colour or aim value counts as 0 and is logged | **Stand-in** (the note doesn't say) | Log: no light file has a missing value |
| `.dil` header: build id, light scale shift, 5 counts, tables A–E (16, 8, 24, 48, 1 bytes), end u32 0 | Note "`.dil` layout" | Log: all 235 files read |
| **One more u32 after the shift (0 in every file)** | **Observed** (measured; not in the note's table): Q67 (a) | Without it no file reads; with it 230 files end exactly at the end u32 |
| **Table D records are 12 bytes in 5 files** (`zedrock1`, `zedrock2`, `platform`, `ramp`, `level4`), worked out from the block size | **Observed**: Q67 (b) | Only D × 12 + E + 4 fits those blocks; read that way they pass rule 13 |
| Byte offset `0xFFFFFFFF` = the surface has no light map | **Observed / stand-in**: Q67 (c) | Castle D `castle.dil`: 96 of 5,318 surfaces; no other file |
| Bytes after the PERS block are ignored | Observed (as for `.dig`, `buildings.md`) | 4 files |
| Texels: 16-bit, 4 bits each brightness, red, green, blue | Note "`.dil` layout" | `cnnrhut.dil`: 1,129 surfaces, 162,170 bytes, red 0 ×16,017, 3 ×7,775, 4 ×7,344 = the note's numbers |
| `.dil` build id and surface count equal the `.dig`'s | Note "`.dil` layout" | 153 of 153 pairs match |
| Size check: width = ⌈(offset_x + size_x + 1) ÷ 16⌉ + 1, height likewise | Note rule 13 | Unit test; every surface of 153 of 153 pairs fits (the note's 148 + the 5 read with 12-byte records) |

## Models lit (M1 step 5b) — `crates/kq8-data/src/{shape,material}.rs`, `crates/kq8-viewer/src/{object_light.rs,object_light.wesl,load,main,terrain_view,anim_view,door_view,connor_view,item_view,effects_view,light_view}.rs`

Behaviour note = `docs/behaviour/object-lighting.md` (rule numbers are
the note's).

| Fact | Source | Checked how |
| --- | --- | --- |
| The lights are the files `swtohwlt.cs` loads | Note rule 1 | Viewer log, every world: the two `*HW.kq` files |
| Corner light = ambient + sun colour × max(0, −aim · normal), capped 0–1; aim not unit length | Note rules 4, 5 | Shader (`object_light.wesl`); the same rules on the CPU for the log (`light_stats`, unit test `suns_in_bevy_space`). Log: Daventry Connor 0.300–0.419, `tree.06` 0.300–0.404, `stngrl` 0.300–0.412, `lamp01.13` 0.300–0.420 (note: 0.30–0.42) |
| Normal turned into world space with the object's own rotation | Note rule 5 | Done on the graphics card per drawn copy (Bevy's mesh transform) |
| Smooth: light per corner, blended across the triangle; flat: first corner's light as one grey (usual weights); none: 1; `0x1000`/`0x2000` force none | Note rule 7 and "Do not reproduce" | Unit test `lighting_kind`; log: Daventry 721 smooth, 4 flat, 365 none materials |
| Other shading bits: full brightness | **Stand-in** (the note doesn't say; no model material in the data has them, `kq8 lighting-verify`) | — |
| Corner normal = average of the unit normals of the triangles that share the vertex (same mesh node, same vertex number) | Note rule 8 (compute from the geometry) | Unit test `corner_normals_average_the_triangles_at_a_vertex`; `--model smlchest.KQS`: 96 of 96 corner normals point away from the model centre |
| Texture × light, no doubling | Note rule 9 | Shader |
| Per-object "unlit" switch off | Note rule 10 | — |
| Multiply on the colour as shown (sRGB curve on the light) | **Stand-in**, as the ground (M1 4c) | — |
| Particles (falling leaves) lit like models | **Our reading** (they are `.kqs` models; the note doesn't mention particles) | — |
| Light computed on the graphics card, not stored in the meshes | **Our choice** (user, 2026-10-15): copies of one mesh face different ways | Viewer runs in 8 worlds, no shader errors |
| Buildings full bright until M1 5c | Plan (DESIGN M1 step 5) | — |
| Point lights, haze, brightness setting not drawn | Not done yet (rules 17, 19–24) | — |

## Buildings lit (M1 step 5c) — `crates/kq8-viewer/src/{building_light.rs,building_light.wesl,load,main,terrain_view}.rs`, `crates/kq8-data/src/dil.rs`

Behaviour note = `docs/behaviour/object-lighting.md` (rule numbers are
the note's; layout corrected by Q67).

| Fact | Source | Checked how |
| --- | --- | --- |
| A building uses `<shape>.dil` | Note rule 12 (alternates are switched only by scripts) | Viewer log: every building in 10 worlds finds its `.dil`, build id and surface count match |
| Wall colour = texture × light map colour (channel ÷ 15), no doubling, no sun or ambient | Note rules 14, 16 (16 is the note's recommendation, **stand-in**) | Shader (`building_light.wesl`) |
| Light texel (i, j) at texture position (16 i − offset_x, 16 j − offset_y), from the surface's own texture origin | Note rule 13, `buildings.md` step 3 | Unit test `corner_coordinates` (a texel's position lands on its centre; the far edge stays in the map); every surface passes the size rule (log: 0 misses) |
| Blend between light texels: bilinear | Note rule 13 (assumption, **stand-in**) | The card's linear filter on the light picture |
| All maps of a building packed into one picture, white texel at 0,0 | **Our choice** (way 2, user 2026-10-15) | Unit test `packing_keeps_maps_apart` |
| No `.dil` or not matching the `.dig`: full bright | Note "Edge cases" | No such building in the 10 worlds tried |
| Surfaces with map offset 0xFFFFFFFF and material 255 not drawn | Q67 c (inferred from data by A) | Castle D: 96 of 5,318 surfaces not drawn (log) |
| `0x1000` building materials use the light map | Note open question 7 (**stand-in**) | — |
| Closing u32 of a `.dil` = a counted byte list | Q67 | `kq8 lighting-verify` PASS |
| Multiply on the colour as shown (sRGB curve on the light) | **Stand-in**, as the ground and the objects | — |

## Haze on models (M1 step 6) — `crates/kq8-data/src/{haze,terrain,objects}.rs`, `crates/kq8-viewer/src/{object_light.rs,object_light.wesl,load,main,terrain_view,anim_view,door_view}.rs`

Behaviour note = `docs/behaviour/object-lighting.md` (rule numbers are
the note's).

| Fact | Source | Checked how |
| --- | --- | --- |
| `hazeDistance`, `visibleDistance` from `TERRAIN.KQ`; no terrain: 20,000 and 25,500 | Note rule 19 | Log: Daventry 18,000 / 27,000, Dead City 9,000 / 12,000 |
| end = 0.85 × visibleDistance, unless not beyond hazeDistance, then visibleDistance | Note rule 19 | Unit test `end_rule` (Daventry 22,950, Dead City 10,200, as the note); viewer log the same |
| haze = (distance − start) ÷ (end − start), capped at 1, 0.01 nearer than the start; distance from the camera to the object's position | Note rule 19 | Unit test `amount`; shader (per drawn part, from its origin = the object's position for every model we draw) |
| `0x10000` halves the haze | Note rule 20, `models-and-textures.md` | Unit test `material_and_object_rules`; log: Daventry 6 materials |
| Haze 1 and not `b_hazeSolid`: not drawn | Note rule 21 | Unit test; log: from Connor's start, 286 of 313 placed objects left out, the nearest at 23,427 (end 22,950) |
| × `hazeMultiplier` after that test | Note rule 22 | Unit test |
| Capped at 1 after the multiplier | **Stand-in** (the note doesn't say; a blend can't pass the haze colour) | Unit test |
| Solid: plain blend towards the haze colour on screen colours | Note rule 23 (traced 2026-10-16, Q68 e; M1 6d) | Shader (`haze_blend`) |
| See-through: no colour, opacity × (1 below 0.5, else 1.2 − haze) | Note rule 23, `objects.md` rule 1 | Unit test |
| Haze colour = `resetHazeColor R G B` in `world.cs`, 0–255, snapped to the nearest colour of the `KQWorld::loadingPalette` palette among entries 10–244 (smallest sum of squared differences) | Note rules 18, 23 (traced, Q68 e). Entries 10–244 all usable and first-on-a-tie: **stand-in** (open question 8) | Unit tests `haze_colour_line`, `nearest_palette_colour`; log: Daventry entry 17 = (4, 8, 12), Ice World entry 64 |
| Keys `b_hazeSolid`, `hazeMultiplier` | Note rules 21, 22 | Unit test `reads_a_section`; the data's misspelled `hazeMulitplier` (13 Gnome files) not read (Q68 d) |
| ~~Particles hazed each at its own position~~ **Replaced (M1 6g)**: all particles of an emitter share one haze, from the emitter's location; the emitter's `b_hazeSolid` / `hazeMultiplier` apply | Note rule 30 (Q68 c) | Viewer runs (Daventry, Ice World); not measurable in the log |
| ~~No object drawn farther than `visibleDistance` from the camera~~ **Replaced (M1 6h)** by the near / far / off rule below | was a stand-in (Q69) | — |
| F8 / `--haze` switches it | **Our choice** (user's request) | `--haze off` logged; F8 by key press not tested |
| ~~Ground and building walls fogged per point; background = haze colour; linear-colour blend~~ **Replaced (M1 6d–6f)** by the rules below | was a stand-in (Q68) | — |

## Haze on the ground, walls and background; draw distance (M1 steps 6d–6h) — `crates/kq8-data/src/{haze,objects}.rs`, `crates/kq8-viewer/src/{ground_haze.rs,ground_haze.wesl,building_light.rs,building_light.wesl,object_light.rs,object_light.wesl,draw_distance.rs,effects_view,item_view,door_view,anim_view,terrain_view,ground_anim_view,load,main}.rs`

Behaviour note = `docs/behaviour/object-lighting.md` rules 23, 27–40
(session A's answers to Q68 and Q69, 2026-10-16), and `objects.md`
"Hidden objects" rule 3.

| Fact | Source | Checked how |
| --- | --- | --- |
| Background = palette entry `newSky n` (`newSky $KQWorld::background` → `set KQWorld::background n`), flat, no haze | Note rule 31 | Unit test `sky_line`; log: Daventry entry 17 (4, 8, 12), Dead City 10 (black) |
| Ground haze per corner: 0.01 below the start, 0.01 + (d − start) ÷ (1.02 × (visible − start)), capped 0.99; start = `hazeDistance`, or 0.6 × `visibleDistance` if that is not beyond it | Note rule 27 | Unit test `ground`; log: Daventry 0.5 at 22,500 |
| Ground not drawn beyond `visibleDistance` from the camera | Note rule 28 | Shader (`ground_haze.wesl`) |
| Where a ground triangle crosses `visibleDistance`: corner distances blended across it, every point past the limit left out | **Stand-in** (open question 10: the original's cut was not read) | Shader |
| Wall haze per surface from its nearest corner, objects' numbers; ≥ 0.99 left out unless `b_hazeSolid` (then 0.99); `0x10000` and `hazeMultiplier` not used | Note rule 29 | Unit test `wall_haze`; log: Daventry start, 16,824 of 19,660 surfaces left out, 1,533 kept (the tomb) |
| Bevy's own unlit material and distance fog no longer used for the ground | **Our choice** (the ground needs its own haze curve and edge) | Viewer runs, no shader errors |
| Near / far / off: D = distance from Connor to the ball centre − radius; off at ≥ `visibleDistance`, far beyond `tooFar` (default 0.83) × `visibleDistance` | Note rules 34–36 | Unit test `near_far_off`; Dead City log: `flame.05` off at 12,169, back at 12,098, near at 10,054 |
| Ball: shape's box middle (turned with the object) and half its diagonal; no `shapeName` in the file: radius 100 at the position; emitters 250 | Note rule 35 and "Do not reproduce" (turn the centre) | Unit test `near_far_off`; log: flame radius 100 |
| Objects with a box collider use the shape's ball too | **Stand-in**, Q70 | — |
| Group bits: near = always \| `groupsActive3D` (0x448), far = always \| `groupsActive3DFar` (0x440), off = always; drawn if any of 0x10, 0x40, 0x80, 0x100 | Note rule 37, `objects.md` "Hidden objects" rule 3 | Unit tests `near_far_off`, `hidden_objects_...`; log: 8 Daventry props with `groupsActive3DFar=0` far and not drawn |
| Items lying around: default `tooFar` and group masks | **Stand-in** (item types' settings not read) | — |
| Collider switch (0x8), `makeActive`, `setActiveCamera` | Not done (note rules 34, 36, 37) | — |
| No Connor (fly camera): the camera is the reference point | **Our choice** (viewer only) | Log says "from the camera" |
| Not drawn = on a drawing layer the camera doesn't show (`RenderLayers`, passed to children) | **Our choice** (how, not what) | Log: hidden meshes drop when objects come back |

## Object placement — `crates/kq8-data/src/objects.rs`, `crates/kq8-viewer/src/{load,terrain_view}.rs`

Behaviour note = `docs/behaviour/objects.md` (session A, answers Q9/Q10).

| Fact | Source | Checked how |
| --- | --- | --- |
| Position `locX/Y/Z` used as written; `isGroundRelative` and `keepOnGround` do not affect placement | Behaviour note (Q10) | Viewer log: of 104 placed Daventry objects, 21 rest within 50 units of the ground, 81 are higher (tables, hanging, inside buildings not drawn yet), 2 lower: `millwheel` 601 below (dips into the stream), `keg` 422 below |
| Angles `dirX/Y/Z` in radians, right-hand rule, turned about Y, then X, then Z (`RotY × RotX × RotZ`, row vectors); no scale | Behaviour note (Q9) | Unit tests: each axis and the order. Viewer self-check: Bevy transform vs the game's rule agree to 0.035 game units for all 104 objects |
| Bevy rotation = C · Rᵀ · Cᵀ (C = the axis change of `to_bevy`) | Our derivation | Same self-check |
| ~~Sections at (0, 0, 0) are skipped~~ **Replaced**: only objects loaded by the world's start-up scripts are placed | Behaviour note `objects.md` "Where a world's objects come from" (Q16) | `newWorld.cs` → `WorldVar.cs`, `WorldInv.cs`, `permwrld.cs`: 446 `loadKQ` lines, 0 inside if-blocks, 0 unreadable. 347 objects placed (was 170). Unit tests for script order, comments, names, positions, angles. Checked by eye by the user 2026-10-11: furniture complete, nothing misplaced or doubled |
| `loadKQ <file> [<name> [x y z [angles]]]`: `same`/missing keep the name, `@` none, else rename; a given position replaces `locX/Y/Z`, angles given or 0 0 0; the object is the file's first section | Behaviour note (Q16 rule 2); "first section" is our reading of the file layout | Observed: no loaded file has more than one section with a position |
| ~~Ground rule with `loadKQ` positions: a z of exactly −1 or −2 is replaced by the ground height at (x, y)~~ **Changed 2026-10-12 (M2 2h-1): −1 = the floor height, −2 = the surface height (`water.md`), whatever `keepOnGround` says; other values as written, then the `keepOnGround` raise. A `.kq` file's own `locZ` does not take the −1/−2 rule | Behaviour note `objects.md` "Ground heights −1 and −2" (Q17) | Unit test. Viewer log: 79 Daventry objects put on the ground this way (was ~8,000 under it); "more than 500 below the ground" went from 83 to 4 (`Dock`, `Dock2`, `look`, `millwheel`: posts, foundation and wheel reaching into water or hillside) |
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
| `keepOnGround` (default on): an object below the ground is raised onto it, never lowered. **Changed 2026-10-12 (M2 2h-1): default (−1) onto the floor grid, `=1` onto the surface, `=0` none (`water.md`); a value other than 0/±1 counts as the surface (our reading; the data only uses 0 and 1)** | Behaviour note `buildings.md` (Q10 follow-up) | Unit test. Daventry objects raised: 0 (the two below the ground, mill wheel and keg, have `keepOnGround=0`). Matters for buildings (step 3b) |
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

## Collision tests (M2 step 2d-2) — `crates/kq8-core/src/collide.rs`

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

## Buildings in Connor's movement (M2 step 2d-3) — `crates/kq8-core/src/{connor_mover,connor}.rs`, `crates/kq8-viewer/src/{load,connor_view,terrain_view}.rs`

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
| ~~A held key is reported as pressed once (no keyboard auto-repeat), so a held turn key clears the blocks once~~ **Replaced 2026-10-12:** keyboard auto-repeat. The movement key pressed last, while still held, counts as pressed again before every update (only that key, as in Windows; letting it go ends it). So a held turn key keeps clearing both blocks: against a wall he keeps turning (bumping head-on a few times) until he glances off. No start delay (Windows waits about 0.5 s before repeating; the game's own timing is not known) | User's check in the original (2026-10-12): with forward or back held against a wall, a held turn key turns him all the way; after letting it go he stands until a turn key is pressed again. The note leaves auto-repeat open (Q38) | Unit test (`at_a_wall_a_held_turn_key_turns_him_only_while_it_repeats`); viewer at `signpst.05`: `--at 173802,269850,-1.5708 --hold 1,0,0 --turn 40,-1` (before: turned 0.35 rad and stood; after: turns every update, glances off at update 49) and `--at 171300,269800,-1.5708 --hold -1,0,0 --turn 40,-1` (backing in: glances off at update 44) |
| Not done: the Connor flag that clears both blocks on hits | Not traced which flag | — |
| Running head-on: stops at the contact, keeps running | Stand-in until 2d-4 (`crash`) | — |
| Every building the start-up scripts load is a collider, hidden ones too, placed as drawn | "Against a building", "When an object takes part" (hidden objects collide; traced) | Viewer log: Daventry's building colliders listed with their world boxes |
| Connor's balls from his `BOUNDS.KQ` box | "Collision shape" (traced) | Viewer log: box −200…200, −200…200, 0…1,300; balls r 200 at 200, 600, 1,000, 1,100 |

## Crash when running head-on (M2 step 2d-4) — `crates/kq8-core/src/connor.rs`, `crates/kq8-viewer/src/{connor_view,load}.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| Running (`run`) head-on into a wall: `crash` plays in full, keys ignored, then the keys take over (held run runs again and can crash again) | `connor-physics.md` "Walls" (traced, Q31 e; measured) | Unit test (`running_head_on_crashes_7_updates_441_5_back_then_runs`). Viewer at the `Tomb` (`--at 237968.9,187493.2,-0.2054 --hold 1,0,1`): contact (237959.7, 187449.2), crash ends at (238049.8, 187881.3); measured (238050.1, 187881.3). Crashes again every 10 updates with run held |
| `crash`: movement 7 of `Conner.anm`, `c_crash.KQS` loop `c_crash`, plays once, 8 frames, 0.4667 s, steps add up to (0, 441.5, 0): straight back | Observed (`GAME/Kq/Conner.anm`, `GAME/Common/c_crash.KQS`) | `viewer.log`: "connor: movement crash … 8 frames, 0.4667 s, steps (0.0, 441.5, 0.0)" |
| The crash starts with the update after the head-on hit | Our choice, the same as `Land` (the note doesn't say which update) | Viewer log: hit at update 1, `crash` updates 2–8 |
| While `crash` plays he is not in the Move state, so a wall behind him is never head-on (he slides or stops) | `connor-physics.md` "Exact hit handling" (head-on only in Move or Jump; crash is state Special) | Unit test (`a_crash_pushing_him_into_a_wall_behind_stops_there_without_head_on`) |
| Not done: the other run-like movements (`stoprun` done in 2g-1; `WaterRun`, `runwaterrun`, `runJump`, `PreBirth`), the exceptions (`KQMonster`, `backJump`, the Connor flag, the "no crash" object flag), the `crash.aud` sound and footstep cues | Not in our code yet | — |

## Objects in Connor's movement (M2 step 2d-5) — `crates/kq8-data/src/bounds.rs` (`lookup`), `crates/kq8-core/src/{collide,connor_mover}.rs`, `crates/kq8-viewer/src/{load,terrain_view,connor_view}.rs`

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

## Start transitions (M2 step 2g-1) — `crates/kq8-data/src/objects.rs` (`transition_lines`), `crates/kq8-core/src/connor.rs`, `crates/kq8-viewer/src/{load,connor_view}.rs`

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

## Stop transitions (M2 step 2g-2) — `crates/kq8-core/src/connor.rs`, `crates/kq8-viewer/src/{load,connor_view,main}.rs`

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

## The standing jump (M2 step 2e-1) — `crates/kq8-core/src/{connor_update,connor,connor_mover}.rs`, `crates/kq8-viewer/src/{connor_view,load,main}.rs`

Behaviour note = `docs/behaviour/connor-physics.md` "Jumping" (traced;
arcs measured) and "Falling off an edge"; the user's checks in the
original (2026-10-12).

| Fact | Source | Checked how |
| --- | --- | --- |
| The update tying walker and mover together (falls, landings, head-on) moved from the viewer to `kq8-core` `connor_update.rs`, unchanged | Our refactor, so jumps can be unit-tested | All earlier tests pass; viewer runs as before |
| Jump key = `Connor.sae` action `Jump` (Insert) | Observed; note "Controls" | Viewer log: `jump Some(Insert)` |
| A jump only in the keys' control, with no start or stop animation playing, from `stop`, `walk`, `run` (and `back`, `left`, `right`) | **User's check in the original** (no jump during start/stop animations; standing, full walking, full running; from turning on the spot: allowed, turning stops, and starts again after the landing); `back` from rule 2; the wait before a stop: our reading, Q41 (a) | Unit test; viewer `--hold 1,0,1 --jump 4`: "refused: stoprun is playing" |
| On the ground, or within 250 above the terrain (its normal then counts); walkable, or steep and not facing uphill | Rule 1 (traced); "feet to the terrain under him" is our reading, Q41 (c) | Unit test (300 refused, 200 allowed; 45° slope facing up refused, facing down allowed) |
| `runJump` from `run`, `backJump` from `back`, else `jump` (walking too) | Rule 2 | Viewer `--hold 1,0,0 --jump 20`: `jump from walk`; from running: "runJump is not loaded yet" (2e-2) |
| Jumping: gravity off, step up and down 0, the arc = the animation's frame steps (x, y, z); keys ignored, no turning | Rule 3; no turning: **user's check** | Unit test; viewer `--jump 5`: 20 updates of `jump`, still for 9, top +1,697.0, end +872.8 and 1,423.4 forward (data: 1,423.4, 872.8) |
| The update after the animation ends: gravity on, steps 325, Fall with the death limit from 1,280 below there, fall timer 0.36 (previous 0.30), so the first drop is 396 | Rule 4; "Falling off an edge" rule 2; the timer measured in the original | Unit tests (drop 396; landing 4,873 below the end safe, 5,373 fatal); viewer: first Fall drop 396.0 |
| No Fall request from the air counter while jumping | "Falling off an edge" rule 1 ("unless he is jumping"); gravity is off, so the counter stays 0 | Unit test (20 jump updates, no Fall) |
| Hits during a jump: walls, ceilings and the ground check act; glancing only (head-on and steep hits during jumps come in 2e-3) | Rule 3 ("walls and ceilings still stop him"); the rest: stand-in until 2e-3 | — |
| `--jump N`: test option, presses Jump once | Our test helper | Log line `test --jump` |

## Running and backward jumps (M2 step 2e-2) — `crates/kq8-core/src/{connor,connor_update}.rs`, `crates/kq8-viewer/src/load.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| `runJump` from `run`, `backJump` from `back`; hold, landing and death animations by kind (`runJumpHold`/`runJumpDie`; `backJumpHold`/`backJumpEnd`/`backJumpDie`); `runJump` has no landing animation | `connor-physics.md` "Jumping" rules 2, 4 (table) | Unit tests; viewer `--at 277744,199295,3.14 --hold 1,0,1 --jump 20`: `runJump` 20 updates, top +1,858.8 (note 1,859); `--at 277744,199295,0 --hold -1,0,0 --jump 10`: `backJump` 18 updates, 2,565.7 back, top +2,156.1, end +788.1 (note 2,566, 2,156, 788), `backJumpEnd` 11 updates |
| After a running jump lands he is in the keys' control at once, still in `runJumpHold`; the keys' choice is asked for from there, so `Conner.anm`'s transitions give `jumprun` (run held) → `run` at 0.8, `jumpstop` (forward let go) → `stop`, `walk` at once (run off) | Note's measurement (open question 3: `jumprun` 3 updates, `jumpstop` 13 updates ≈ 1.0 s); the 2g rules; **our reading**, the landing itself not traced | Unit tests; viewer: `jumprun` updates 43–45 then `run`; with forward let go (`--hold 1,0,1,30`) `jumpstop` updates 43–57 then `stop` |
| At that landing a turn key alone counts as standing: `jumpstop` plays and he turns during it, then the turn on the spot | **User's check in the original** (turning only during the running jump's landing, forward let go, a turn key held); how the game does it: Q41 (d) | Unit test (`running_jump_lands_into_jumpstop_with_forward_let_go_and_turns_there`) |

## Hits during a jump (M2 step 2e-3) — `crates/kq8-core/src/{connor_mover,connor_update,connor}.rs`

| Fact | Source | Checked how |
| --- | --- | --- |
| Head-on acts while jumping (state Jump) as in Move, and any surface with normal z < 0.8 counts as head-on; never during `backJump` | `connor-physics.md` "Walls" (traced) | Unit tests |
| Up-pass hits during a jump are tested too (a vertical move is never head-on by angle, so only the steep rule; a ceiling counts) | "Exact hit handling" (head-on is tested on up and across passes, Q37 c) | Unit tests (through the wall tests) |
| The first steep hit (normal z < 0.8) in any pass ends the jump: gravity on, steps 325 | "Jumping" rule 5 (traced) | Unit tests |
| Down-pass steep hit during `jump` or `runJump`: `crash`. Up or across: `runJump` crashes, `jump` stands (`stop`, forward blocked as for a walking head-on stop). **Replaced 2026-10-17** (Q82 stand-in, then A's corrected rule 5; see "A jump that hits something steep" below) | Rule 5 as it read then; the forward block from "Walk held against a wall" (Q38) | Unit tests (wall, 63° hill); viewer `--at 277744,199295,0 --jump 3`: `jump` hits `tree.06` (across), stands, falls 5 updates later, `jumpend`; `--at 277744,201500,0 --hold 1,0,1 --jump 13`: `runJump` hits it, `crash` |
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

Code: `crates/kq8-core/src/connor_mover.rs`, `crates/kq8-viewer/src/connor_view.rs`, `crates/kq8-viewer/src/load.rs` (`place_connor`).

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

Code: `crates/kq8-data/src/action_map.rs`, `crates/kq8-core/src/connor.rs`, `crates/kq8-viewer/src/connor_view.rs`, `crates/kq8-viewer/src/load.rs` (`load_animation`, `place_connor`).

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

Code: `crates/kq8-core/src/connor_mover.rs`, `crates/kq8-core/src/connor.rs` (`Mode`, one-shot animations), `crates/kq8-viewer/src/connor_view.rs`, `crates/kq8-viewer/src/load.rs` (`CONNOR_MOVEMENTS`, `load_animation`), `crates/kq8-viewer/src/main.rs` (`--at`).

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

Code: `crates/kq8-core/src/camera.rs`, `crates/kq8-viewer/src/game_camera.rs`, `crates/kq8-viewer/src/load.rs` (`script_range`, camera start).

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

## Water: the floor grid and wading (M2 step 2h) — `crates/kq8-data/src/{terrain,objects,envinfo}.rs`, `crates/kq8-core/src/{water,connor_mover,connor,connor_update}.rs`, `crates/kq8-viewer/src/{load,connor_view}.rs`

| What | From | Checked how |
| --- | --- | --- |
| A world with `floorBMPName` gets a floor grid from that image, built like the surface (value × `heightBMPScale`, same triangles); without one, floor questions get the surface | `water.md` "Two ground grids" (traced) | Unit test. Viewer log: Daventry 120 corners lower, 0 higher; Swamp 739 lower, 12 higher (as A measured) |
| The drawn ground stays the surface; the floor is not drawn | `water.md` rule 5 (not traced, "most likely") | — |
| Connor's mover (terrain height and normal), the camera's "keep above the ground" and `--at` use the floor | `water.md` "Which grid answers" (Connor has the default −1) | Swamp shack: Connor lands at 6,468 (east) and 7,332 (west), 765 lower than before, as predicted. Daventry river: feet at 5,610 under a surface of 6,630 (1,020) |
| Placement: `loadKQ` z −1 floor / −2 surface; `keepOnGround` default floor, 1 surface, 0 none (see the changed rows above). **The `keepOnGround` raise also follows a −1/−2 (added 2026-10-12)** from the user's check in the original (DeadCity portal by the flame tower; Q44): only `portsa7`, `portsb7`, `porttp7`, `portaa7` moved, 0 → 2,784, over all 12 worlds | `water.md`, `objects.md` Q17 rule 2 (corrected) | Unit tests. Before/after over Daventry, Swamp, IceWorld, DeadCity: Daventry and IceWorld unchanged; Swamp `shack2` 6,120 → 5,355, `fientowr` 6,120 → 5,994, `proftree` 6,885 → 6,871; DeadCity `DeadHal2`, `platfrml`, `porttow3.01` and the doors moved up 2,784 onto floor squares above the drawn ground (the note's open question 3, not checked in the original); `Pool1`–`Pool5` (click targets) down to the floor |
| `envInfo.kq` ground flags: strip spaces and tabs, skip the first character, tabs move columns, letters/digits set them; column k = flag 1 << k; liquid = 0x1C8 | `water.md` "Ground flags from envInfo.kq" (traced) | Unit tests. Viewer log: Daventry 201 water squares (4 also swamp), as A counted |
| The flags of a position = its square's material (`terrm.dib`) flags | Same | Unit test |
| After every move: `submerged` = liquid square and surface ≥ feet + 600; the next update's movement choice uses it | `water.md` "The water test", "Wading animations" rule 3 (traced) | Unit tests; Daventry river: on at update 67 (surface 6,630, feet 6,017), off at update 192 |
| Forward in water: `waterrun` / `waterwalk` instead of `run` / `walk`; other movements unchanged; the `.anm` transitions play (`walkwaterwalk`, `waterwalkwalk`, `runwaterrun`, `waterrunrun`); no slowing, no floating | `water.md` "Wading animations" rules 1, 2, 4 | Unit test; Daventry river, walking: `walk` → `walkwaterwalk` (from 0.2) → `waterwalk` → `waterwalkwalk` → `walk`; running: `run` → `runwaterrun` → `waterrun` → `waterrunrun` → `run` |
| `waterrun` and `runwaterrun` crash head-on like `run` | `water.md` rule 5; `connor-physics.md` "Walls" | Unit test |
| ~~No jump from `waterwalk`/`waterrun`~~ **Changed 2026-10-12:** from `waterwalk` and `waterrun` the Jump key gives the standing `jump` (not during the transitions into or out of water, like other start/stop animations) | User's check in the original (2026-10-12): walking or running in water, he stops and does a standing jump | Unit test; Daventry river `--at 141312,198656,0 --hold 1,0,1 --jump 60`: `jump from waterrun` at update 61, lands at 83, `jumpend`, then `waterrun` again |
| Not done: damage (poison, lava, swamp), sinking into `ontop` hazards, monsters with `keepOnGround=1` | `water.md` Edge cases | — |

## Doors and their scripts (M3 step 3a) — `crates/kq8-core/src/{script,door,events,click,active}.rs`, `crates/kq8-data/src/objects.rs` (`movement_loops`, cues), `crates/kq8-viewer/src/{door_view,load,terrain_view,main}.rs`

Behaviour note = `docs/behaviour/doors.md` (session A, traced unless
marked). No other source; the game's scripts are read at runtime, never
copied (unit tests use made-up scripts).

| Fact | Source | Checked how |
| --- | --- | --- |
| Script lines split at spaces/tabs, `#` comment, `"…"` one word; `$1`… arguments (missing empty); `$Name` variables split into words again, an unset or empty one vanishes, quoted stays one (maybe empty) word | doors.md "Words and arguments" | Unit tests (`unset_variables_vanish_quoted_ones_stay`) |
| `if <command>` takes the branch when `$Console::Return` is 0 or empty; `else`; `endif`; skipped `if`s counted; `else if` = else + nested if with its own `endif` | doors.md "`if`, `else`, `endif`" | Unit tests (`else_if_needs_its_own_endif`, `skipped_ifs_are_counted_not_run`). All 174 Daventry `.cs` balance under this rule (throwaway program); viewer log: every door script balanced in Daventry, Swamp, DeadCity, Barren, IceWorld |
| `test a op b`: 3 words or usage + false; numbers when both look like numbers (digit first, or `-`/`.` then a digit), else text with case; `&&`/`||` whole numbers only | doors.md "`test`" | Unit test (`test_compares_numbers_and_text`); `KeepDoor.cs`'s `test $Console::Return >1` (two words) logs usage and is false |
| `$Console::Return` = `0` for true/success, `1` for false/failure; a number's value is its leading number part | Our stand-ins (Q45 e, f), **confirmed 2026-10-13** (`doors.md` "Q45" e, f; unknown command 1, a script call keeps its last command's result, `-.5` is text). Not copied: a false text `==` may leave −1 (Q54) | Unit test (`console_return_and_if_on_any_command`) |
| Commands matched exactly; a first word naming a script (`.cs` added without a dot) runs it with the rest as arguments; unknown command: error, fails | doors.md "Words and arguments" | Unit test; IceWorld `control.cs` logs `getEndLoop` as unknown |
| Stubs (log only, result 0): `KQMonster::speak`, `doPopup`, `addMapIcon`, `earnTeleport`, `teleportersEarned`, `KQMonster::checkInventory`, `deleteObject`, `handsOff`, `KQMonster::setState`, `pauseAnim`, `preloadShape`, `KQObject::setHidden`, `loadKQ` | doors.md command table ("stand-ins: log them, checkInventory → 0") | Viewer: `condoor.cs` logs `KQMonster::speak` and `loadKQ ravencnr.kq` |
| Door states, `KQDoorEvent` 1/2/other (no number = toggle), ignored while moving; every door starts closed | doors.md rules 1–4 | Unit test (`the_state_table`, 14 cases) |
| `KQAnimPlayEvent` starts a movement by name, state unchanged; finishing `open`/`close` (any case) → opened/closed + `opened`/`closed` + script `Open`/`Close` | doors.md rules 5–6 | Unit tests; viewer: `Condoor` `open` finished update 20, `condoor.cs Condoor Open` |
| Phase moves speed × (1/15 s) / loop length per update; one-shots finish at 1 or 0 | doors.md rules 5, 8 (and `objects.md` loop rules) | Viewer: `Condoor` 15 moving updates (16 frames, 1.0 s) |
| A movement started by an event first moves in the next update | Our stand-in (as Connor's transitions, Q40 a), **confirmed** by `doors.md` "Q45" d (reasoned from the queue, not measured) | Viewer: event update 5, first step update 6 |
| Cue at c fires when c is in [lower, higher) of the update's phases; `playsound` logged (no audio) | doors.md rule 9 | Unit tests (cue 0.0 fires in the first moving update; 0.95 on a close from 0.93 never); viewer: `playsound door4.aud` at update 6 |
| `cueN=<phase> <action>` lines, `nCue` of them; `#`-commented ones don't count | Observed (Daventry door `.kq` files; `tombdor2.kq` comments its cues out) | Unit test (`every_movement_with_its_cues`) |
| The door turns with its animation: frame shown = round(phase × (frames − 1)); turn = frame steps 1…k chained (each in the door's already-turned axes); the box turns with it | doors.md rule 24 (inferred, confirmed by the user in the original) and open question 1's recommendation; frame rule as Connor's one-shots | Throwaway program on `door1.kqs`: every chained step turns about the same hinge (−634, −63); 165° at frame 13, 167.75° at 14, 165° at 15. Unit test: 10 open/close cycles end exactly closed. Viewer: closed door stops Connor 225 from its face; opened, he walks through the doorway into `ConHouse`; walking into the open leaf stops him head-on (normal turned 165°) |
| Door turn angles and timing from the data: `door1`/`door2` 165° in 1.0 s; `tombdoor`, `outdor` 90°; `keepdrl` 180°, `keepdrr` −180°; `door3` (`tombdor`) 25 frames, 1.6 s, 180° | **Observed** (`.kqs` frame steps and loop lengths) | Viewer log per door. `door3` differs from doors.md rule 8 (Q46) |
| `sendEvent <delay> <object> <type> [data]`: due at now + delay, time order then send order; fewer than 4 words usage; unknown object or event type: error, not sent | doors.md command table (traced) | Unit tests (`time_order_then_send_order`, `unknown_objects_and_event_types_are_not_sent`) |
| Events handled at the start of each update, including those sent with delay 0 in this update's clicks or events; events sent by `Open`/`Close` calls wait for the next update | Our stand-in (doors.md open question 4), **confirmed** by `doors.md` "Q45" d | Viewer: click and `KQDoorEvent` in the same update 5 |
| `KQCueEvent n1 [n2]` → script `Cue n1 n2` (`$5` only if non-zero) | doors.md rule 23 | Unit test: `.07` s later = two updates later |
| `KQObject::collisionType <name> <±bits>`: add / remove bits, 0 refused | doors.md command table | Unit test; viewer: keep doors `0x1021 → 0x1001`, then "not clickable now" |
| Script `Init` call for each door as the world starts | doors.md rule 22 | Viewer: `doors.log` update 0 |
| Click: nearest box hit by the camera ray among objects whose type has bit 0x20 | doors.md rule 10; "only clickable boxes block": our stand-in, **confirmed** (`doors.md` "Q45" b) | Unit tests (`the_ray_meets_the_turned_box`); mouse not tested |
| Reach: Connor's feet into the door's axes, per-axis part outside the box, < 2 × `attackDist` (`Connor.kq` 2,350 → 4,700); the box as it stands (turned) | doors.md rule 11; "turned box": our stand-in, **confirmed** (`doors.md` "Q45" c) | Unit test (`distance_is_measured_in_the_doors_own_axes`); viewer: 1,000.0 from 1,000 in front; 26,913.5 from the start spot → too far |
| `ConnorAction do do`; item: accepted when `acceptsInventory` = its type or `acceptAllItems=1` → `<object> <type>`, else refused; out of reach: `tooFarAwayAction` 0 → remark, nothing sent, else `TooFar TooFar` | doors.md rules 12–14; the accept rule is **our stand-in** (open question 2) | Unit tests; viewer with `--item INVITEM_KeepKey`: both keep doors open |
| Not done: turning Connor to face the door (rule 16), the remarks' lines, hits / `Die` / `ProjImmune` (no combat yet), `tooFar=` (Q45 a), clicks on objects other than doors (logged only), changing Connor's collision when a script removes bit 0x1 (none does in Daventry) | — | Logged |

## Emitters (M3 step 3b-1) — `crates/kq8-data/src/effects.rs`, `crates/kq8-core/src/emitter.rs`, `crates/kq8-viewer/src/{effects_view,load,terrain_view}.rs`

Behaviour note = `docs/behaviour/emitters.md` "Emitters" (session A,
traced). No other source; unit tests use made-up sections.

| Fact | Source | Checked how |
| --- | --- | --- |
| Keys `shape`, `radius`, `forceMin/Max`, `gravity`, `rotMin/Max`, `maxSinPeriod`, `followOffset` as `X/Y/Z` triples (missing 0); `maxLife` 100, `maxParticles` 200 by default; `delayStart` only zero or not; `follow`, `followMesh` | Note "Emitter keys" | Unit tests (`emitter_keys_and_defaults`, `unreadable_numbers_are_reported`). Over all 12 worlds (throwaway count): 43 emitter files; no `dying`, `deadCount` or `frameCount` |
| Position: the object's place after the usual placement rules | Our reading (the note gives only `loc`); no difference in Daventry | Viewer log: all 14 Daventry emitters unchanged by the rules and above both grids |
| At the start: offset in the ±radius box, velocity, spin (only if a rot limit is non-zero), sway speeds, starting age ⌊fraction × maxLife⌋, each uniform and independent; moved on by age × velocity | Rule 1 | Unit test (`choices_lie_within_their_limits`) |
| Per tick in this order: age + 1; sway angles + speeds; position + velocity (velocity × sin(angle) on an axis whose angle is not 0); turn + spin; position + age × gravity; past `maxLife` restart at loc + offset, visible, turn and sway angle kept | Rules 3, 6, 7 | Unit tests (`a_life_is_max_life_plus_one_ticks_with_one_hidden`, `gravity_adds_up_to_n_n_plus_1_over_2`, `swaying_steps_are_velocity_times_sine`). Viewer: `stream1` particle 0 restarts every 41 updates, 1 not drawn, rises to +277, falls to 7,834 (580 below the ground near its end; rule 5: no collision) |
| Sway speed = fraction × P × 6.28 ÷ maxLife; "non-zero" = absolute value ≥ 0.000001 | Rule 6, "Numbers and units" | Unit test |
| Same path every life | Rule 2 | **Only on axes without sway.** With rule 3.6 (sway angle not reset), `leaf1` particle 0's x differs from its first life by 2,787 in the second life and 5,400 in the third (viewer log); y and z repeat exactly. Asked as Q47 |
| `delayStart`: all hidden until their first restart | Rule 4 | Unit test (`delay_start_hides_…`) |
| Drawn: frame 0 of `shape` at the particle's position and turn (turn used like `dirX/Y/Z`), only when visible and age ≠ 0 | Rules 4, 9 (turn convention: the note's assumption, not checked) | Not checked by eye |
| The emitter's `translucency` on every particle | Rule 9 ("likely, not traced", open question 2) | `stream1–4` drawn at 150/255 |
| Always updated | Rule 12 (recommendation) | — |
| Random numbers: our own seeded generator (SplitMix64), same seed every run | Note "Do not reproduce" (no need to match the C library's) | Unit test (`random_fractions_stay_in_range`) |
| Drawing blended between updates, not across a restart | Our choice (display only, as Connor) | — |
| Not done: `follow`/`followMesh` (rule 11; IceWorld `draglfir`, `dragrfir`, `snow`, Temple4 `lucfire`), dying (rule 10), script commands (open question 4), arrow emitters (`emitter=` in projectiles, combat) | — | Logged as "not started" |

## Particle catchers (M3 step 3b-2) — `crates/kq8-data/src/effects.rs` (`CatcherSettings`), `crates/kq8-core/src/{emitter,collide}.rs`, `crates/kq8-viewer/src/{effects_view,load,terrain_view}.rs`

Behaviour note = `docs/behaviour/emitters.md` "Particle catchers" (session
A, traced).

| Fact | Source | Checked how |
| --- | --- | --- |
| Keys `emitter`, `interior` | Rule 1 | Unit test (`catcher_keys`) |
| Names looked up once after the start-up scripts; nothing found → the catcher does nothing | Rule 2 | Viewer log: Daventry's 3 catchers find `leaf1–3` and `Cottage` |
| Names compared without case | **Our stand-in** (Q48: the original counts capitals) | Throwaway scanner (scratchpad, 2026-10-13): all 13 catchers in the game (Daventry 3, IceWorld 10) name their emitter and building with exactly matching capitals, so both ways give the same result |
| Box = the building's `.dig` bounding box (`BuildingCollider::bounds`) and its placement; top = box top z + building z | Rule 3 | Viewer log: `Cottage` (`house1.dig`, dirZ −1.57) box (−6,272, −3,936, 0)..(3,072, 9,184, 5,120), top 18,915 |
| Every tick: particles at or below the top, turned into the building's own axes, hidden if min ≤ value < max on all axes; visible again at the next restart | Rules 4, 5; emitter rule 4 | Unit test (`a_catcher_hides_particles_inside_the_turned_box_until_their_restart`: a turned box, the roof edge itself not inside, visible again after the restart). Viewer (about 620 updates): `Cottage2` hid `leaf1` particles 19 times, at x ≈ 280,190–280,250 (the box's west side is at 280,181), `Cottage4` hid `leaf3` 4 times, `Cottage3` none (`leaf2` is 13,000 west of the cottage) |
| Catchers run after all emitters in the same tick | **Our stand-in**; the original's order is not settled (`emitters.md` catcher rule 5: "B's order is fine") | — |

## Ground animators (M3 step 3b-3) — `crates/kq8-data/src/{effects,terrain}.rs`, `crates/kq8-core/src/ground_anim.rs`, `crates/kq8-viewer/src/{ground_anim_view,load,terrain_view,main}.rs`

Behaviour note = `docs/behaviour/emitters.md` "Ground animators" (session
A, traced; square choice checked against the data).

| Fact | Source | Checked how |
| --- | --- | --- |
| Keys `numOrigTextures`/`origTextureK`, `numExclude`/`excludeXK`/`excludeYK`, `numGridBlocks`/`gridBlockXK`/`gridBlockYK`, `numMaterialIndex`/`materialIndexK`, `numMaterialIndexII`/…, `delay`, `repeatflag`, `startNow`, `frameCount`; only the first `numX` entries read; with `numGridBlocks` ≠ 0 the texture and exclude keys are not read | Note "Ground animator keys" | Unit tests (`animator_keys`, `grid_blocks_replace_textures_and_excludes`). `#numMaterialIndex=` lines (`fullstrm.kq`) are other keys, so not read |
| A missing `delay`, `repeatflag`, `startNow` counts as 0 | Our reading (every file of the 12 worlds sets all three) | — |
| Squares chosen once, from the materials as loaded: the list, or every square with an `origTexture` minus the excluded | Rule 1 | Unit tests. Viewer log, Daventry: 86, 0, 57, 19, 0, 17, 32, 2, 13, 4, 4 squares, as the note's table; (45, 57) animated by none, (45, 53)–(45, 56) by `FullStream` only |
| Each change sets every chosen square to the same material; first change `delay` after the start shows entry 0, then 1, 2, …; `repeatflag=1` wraps to 0, `0` stops at the last | Rules 2–4 | Unit tests (`changes_every_delay_on_the_15_per_second_clock`, `played_once_keeps_the_last_material`). Viewer: first change at update 5, entry 0 |
| ~~Timer on game time (updates / 15); next due = previous due + `delay`~~ **Replaced 2026-10-13 (3r-1):** checked every drawn frame on continuous game time (fixed clock + overstep); next due = time of this change + `delay` | `emitters.md` ground animator rule 3 (Q49, traced) | Unit test (`each_change_is_delay_after_the_previous_change`); viewer, Daventry 8.5 s: 297 gaps 0.300–0.306 s, average 0.304 s (frames about 5.7 ms) |
| `startNow=0`: never started | Note keys table; the commands that would start it are unused (open question 4) | IceWorld `pond` logged as not started |
| `frameCount` ≠ 0 not applied; envInfo flags not changed | Rule 5 (no file sets `frameCount`); open question 3 | Logged if seen |
| ~~The drawn square follows its material: texture and orientation code from the direction table~~ **Changed 2026-10-13:** the texture follows the new material, the orientation stays that of the square's own (loaded) material | Rule 6 read as "the new material's code" first; **user's check of our viewer** (Daventry shores all turned alike, DeadCity blood frames turned 90°, Swamp squares flipping) and the direction tables (turned variants are separate materials of one texture; animation frames mostly code 0). Stand-in, asked as Q50; **confirmed** (rule 2: the square's own original orientation) | Viewer log: squares where it makes a difference: Daventry 116, DeadCity 42 (`Blood2` 4, `Blood3` 38), Swamp 376. Throwaway script (scratchpad) listing each animator's own and cycle codes. Not yet re-checked by eye |
| Animated squares drawn as their own entities; their surface left out of the ground mesh, their floor kept; U view: hidden in "floor only", see-through in "both" over a different floor | Our choice (display only) | Viewer log: Daventry 234 squares, 43 materials; Swamp 25 animators, 676 squares; DeadCity 8, 543; IceWorld 7 (6 running), 263 |
| `Grid::from_rows`, `Grid::set`, `Terrain::square_triangles` (one square's triangles, now also used by `ground_triangles`) | Refactor | Existing terrain tests pass unchanged |

## Animated objects play (M3 step 3c) — `crates/kq8-core/src/anim.rs`, `crates/kq8-viewer/src/{anim_view,load,terrain_view,main}.rs`

Behaviour notes: `objects.md` "Animated objects at load" (traced, Q12,
Q13), `doors.md` rules 5, 8 (phase), `connor-movement.md` (a loop's
length is its stored length).

| Fact | Source | Checked how |
| --- | --- | --- |
| Every placed non-door object with `animLoops` whose current movement's speed is not 0 plays it; speed 0 stays still | `objects.md` rule 2 (`b` = speed, 0 = holds the start frame) | Viewer log: Daventry 72 objects, 40 loops; Swamp 426, DeadCity 213, Barren 135, Gnome 234, IceWorld 23, CastleD 8 |
| Phase starts at `a`, moves speed × 1/15 s ÷ the loop's stored length per update; repeating wraps, one-shots stop at their end and stay | Rule 2; doors.md rules 5, 8 (same phase rule); the stored length as for Connor's loops | Unit tests (`a_repeating_loop_wraps_once_per_stored_length`, `a_one_shot_stops_…`, `speed_zero_holds`). Viewer: `wellwatr` 66 frames / 4.333 s, one frame per update |
| Frame while playing: repeating round(phase × frames) mod frames; one-shot round(phase × (frames − 1)) | Rule 2 gives the repeating formula for the start frame; using it while playing is **our reading** (Q51; `objects.md` Q51 rule 6: keep it); one-shot as doors | Unit test (`the_start_frame_is_the_still_models`: `telepad1` 0.3 × 16 → 5, as before) |
| A 16-frame loop of stored length 15/15 s skips one frame per cycle | Follows from the two rules above | Unit test |
| `flags` bit 8: random start phase, speed × random 0.9–1.1 | Rule 4 | Unit test (`random_start_and_speed_stay_in_range`). Viewer: Daventry 10 (`Davtorch.01–05`, dead birds, fish) |
| `nextAnimName` (one-shot end), `animPos`, cues, script changes of the movement | Not done (rule 4: `nextAnimName` not traced) | — |
| Whole-object steps applied like doors (pose = steps 1…k of the shown frame chained, never added across cycles) | Our way (doors.md rule 24 applied to loops). The original adds the steps up and the data's `cue1=1.0 sendEvent @ KQMoveEvent 0 <start>` puts the rock back each cycle; same picture, **kept by the user's check** (`objects.md` Q51 rules 1–7) | Only Gnome `rrock1–3` (`theroll`) have any: up to 9,691 units, 57 units off the start at the last frame |
| ~~Collision boxes stay fixed~~ **Changed 2026-10-13 (3r-4):** an object whose loop moves it takes its collider along | `objects.md` Q51 rule 5 (the box goes with the object; Q33 a's "fixed" is its size) | See "Review after Q45–Q52" below |
| Drawn: one entity per frame, only the shown one visible; H-key and opacity kept | Our choice (display), as doors | Not checked by eye |

## Emitters that follow something (M3 step 3b-4) — `crates/kq8-core/src/emitter.rs`, `crates/kq8-viewer/src/{effects_view,terrain_view,game_camera}.rs`

Behaviour note = `docs/behaviour/emitters.md` emitter rule 11 (traced).

| Fact | Source | Checked how |
| --- | --- | --- |
| A following emitter's `loc` = target position + `followOffset`, the offset in the target's own axes unless its x and y are both 0 (then world axes); moved with the target | Rule 11 | Unit test (`a_following_emitter_sits_at_its_target_plus_the_offset`). IceWorld viewer, Connor running: `snow`'s `loc` stays 4,000 above him every second of the log |
| Velocity limits turned with the target at the particle's start; kept afterwards; restarts at the moved `loc` | Rule 11, rules 1–2 | Unit test (`following_turns_the_velocity_and_restarts_at_the_moved_spot`) |
| `followMesh`: a restart takes the target's turn and a new random velocity, turned | Rule 11 | Unit test (`follow_mesh_restarts_…`). Its value (a mesh-part name) not used yet; the answer (rule 11 a: sit at that model part, offset unused, part missing → dies) is for when the dragon heads / `lucfire` arrive. No running emitter has it (scanner: only `draglfir`, `dragrfir`, `lucfire` in the game) |
| Targets: `Connor` (turn = his heading about z), `KQCamera` (position, no turn), placed objects and markers by name, any case; no target → not started | No target: our choice, **kept** (Q52 b: the original would crash). Camera with no turn: differs from Q52 c (the camera's turn), but no emitter in any world follows the camera (scanner) | IceWorld: `snow` follows Connor |
| The emitter follows before its particles move, after Connor's update | Our stand-in, **confirmed** (Q52 d) | — |
| IceWorld's 10 snow catchers now find their emitter | Catchers rule 2 | Viewer: 296 flakes hidden in about 140 s (Connor starts inside `Lvator`) |

## Review after Q45–Q52 (M3 step 3r) — `crates/kq8-core/src/ground_anim.rs`, `crates/kq8-viewer/src/{ground_anim_view,anim_view,load,main}.rs`

Behaviour notes: `emitters.md` ground animator rule 3 (Q49), `objects.md`
"Loops that move the whole object" rules 5 and 8 (Q51). The other answers
only confirmed code already written (rows above updated).

| Fact | Source | Checked how |
| --- | --- | --- |
| 3r-1: a ground animator is checked every drawn frame on continuous game time; the next change is due `delay` after the time of this change; at most one change per check | `emitters.md` ground animator rule 3 (Q49, traced) | Unit test with uneven frame times; viewer Daventry: 297 gaps 0.300–0.306 s |
| 3r-2: an object of class `KQBoulder` doesn't play its movement (drawn still, its `stop` loop starts at 0.0) | `objects.md` Q51 rule 8 (the user saw it still in the original) | Viewer Gnome: `boulder` logged as not played; 233 objects play (was 234), the rolling rocks still play |
| 3r-4: an object whose loop has whole-object steps moves its collider to the pose it is drawn with, every update, after Connor's update | `objects.md` Q51 rule 5 (the box goes with the object). The order after Connor is **our choice** (not traced) | Viewer Gnome: `rrock1–3` find their colliders (`theroll.kqs`, type 0x1); `rrock1`'s box goes from y 111,710 to about 102,000 and back each 3.53 s |

## The camera avoids walls (M2 step 2f-2) — `crates/kq8-core/src/{camera,collide}.rs`, `crates/kq8-viewer/src/{game_camera,load,terrain_view}.rs`

Behaviour note = `docs/behaviour/camera.md` "Obstacles" (session A,
traced 2026-10-13; three points checked by the user in the original).
The aim height (800) and height easing (50%) the user set by hand are
unchanged. Checked by eye by the user on 2026-10-13 ("looks great").

| Fact | Source | Checked how |
| --- | --- | --- |
| View blockers: the drawn ground; buildings (type default 0x28) and objects whose type has bit 0x8 or 0x1000, by `BOUNDS.KQ` box turned with the object; doors as they stand, with their current type; not monsters, trees or plain objects | Note "What blocks the view" | Viewer log per world: Daventry 26/26 buildings, 0 object boxes + 17 doors; Barren `smlrock` (0x21) doesn't block; DeadCity 41 boxes (crates, blocks), Gnome 9 (bridges), IceWorld 8 |
| Buildings: the BSP walk with a zero-width line; starting inside solid is not a hit | Note ("buildings by their BSP"); the zero-width walk is **our reading** (Q53 b) | Unit test (`a_line_stops_at_the_wall_face_and_not_from_inside`) |
| Boxes: the slab test | Note ("objects by their box") | Unit test (`a_line_meets_a_turned_box`) |
| Ground: stepped every 50 units, halved 20 times, only a step from above to below counts | **Our method** (Q53 a) | Unit test (`the_ground_blocks_where_the_line_goes_into_it`) |
| Update steps 6–9: running swing (cancelled by the mouse); view test; mouse into a wall → previous offset × 0.95 unless under the minimum; when moving, avoidance on and feet not under the floor grid: towards halfway between the last aim point and the hit, swing 0.66 × the first turn per update until the direction to the last aim point is reached or passed; still blocked → 100 short of the new hit (at least 1, "Do not reproduce"), swing cancelled; no height rule after | Note steps 6–9, "Obstacle numbers" | Unit tests (`standing_still_behind_a_wall_nothing_moves`, `walking_with_a_wall_behind_pulls_the_camera_in_100_short`, `the_camera_swings_round_to_look_along_his_path`, `dragging_into_a_wall_creeps_in_5_percent_never_under_the_minimum`, `a_hit_closer_than_100_keeps_the_camera_in_front`). Viewer, Connor walking into his house: blocked at update 39, turned 0.084 rad, swing 0.056 per update ending 0.039 rad past his path at update 54. Three runs near the house: 0 updates with the camera inside a building's solid |
| "Connor is moving": position or heading changed, from the update after he starts until 3 updates without movement | Note ("Connor is moving"; heading counts: the note's assumption) | Unit test (the first moving update doesn't count) |
| The new direction is 3D (towards M) | Note as written; asked (Q53 c) | — |
| Not done: the camera updating only while Connor moves or the camera is used; switching avoidance off | Note "When the camera updates at all" (Q53 d) | — |

## Banking while running (M2 step 2i) — `crates/kq8-core/src/connor.rs` (`Walker::bank`, `update_bank`), `crates/kq8-viewer/src/connor_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Lean into the turn while running and turning, at most 0.15 rad | `connor-movement.md` "Turning" rule 3 | Unit test; `--hold 1,1,1,45`: 0.03 per update from update 12 (first `run` update after `stoprun`), 0.150 from update 16 |
| Lean speed 0.45 rad/s × elapsed time (not 0.03 per update) | `connor-movement.md` "Do not reproduce" (frame-rate-dependent banking) | Unit test (0.03 per 1/15 s update) |
| Back to upright at the same rate, snap within 0.029 | `connor-movement.md` "Turning" rule 3 | Unit test; log: upright 5 updates after the keys are let go |
| Walking never leans | `connor-movement.md` "Turning" rule 3 | Unit test; `--hold 1,1,0,40`: lean 0 in all updates |
| `waterrun` leans too | The user's check in the original (2026-10-13), Q56 a | Unit test; `--at 141312,198656,0 --hold 1,0,1 --turn 60,1`: wading from update 44, lean from 61 |
| Otherwise only `run`, keys in control, no wall block; not the start/stop animations; upright during jumps and falls; dead: unchanged | **Our stand-in**, asked as Q56 a | Unit tests |
| Lean always 0.15 × the turn's sign | **Our stand-in**, Q56 d (half turns); the size checked by eye against the original by the user | By eye (user) |
| Through 0 to the other side at the same rate, snap only towards upright | **Our stand-in**, Q56 c | Unit test |
| Pivot through his feet (model origin), turned in before the heading | The user's check in the original (2026-10-13), Q56 b | By eye (user) |
| Drawing only | **Our stand-in**, Q56 e | — |
| Sign: positive lean = turn about the model's Y (forward axis), head towards +X (his left) | `objects.rs` rotation convention (unit test "positive dirY turns +Z towards +X"); `connor.rs`: +X is his left, turn +1 = left | `--hold 1,-1,1,30`: lean −0.150 turning right |

## Trap rules (M3 step 3d-1) — `crates/kq8-core/src/trap.rs`

| What | Source | Check |
| --- | --- | --- |
| Box = `BBMin/BBMax` + position, never turned; never solid | `traps.md` rules 1, 5 | Unit tests |
| Enter on touch: first in `On <who>`, later ones `Cue 301 <who>` | `traps.md` rule 6 | Unit tests |
| Stay while the box centre is in (edges count) or still touching; leave when both fail: last out `Off <who>`, others `Cue 302 <who>` | `traps.md` rules 7–9 | Unit tests |
| Sticky (`trapType` bit 2): shrunk to `innerPercent` towards the position while empty, full while someone is in | `traps.md` rule 10 | Unit test (0.9: ±300 → ±270, z 0–1,000 → 0–900) |
| One-shot (bit 1): gone after its `On` | `traps.md` rule 11 | Unit test (unused in the data) |
| Who counts: only the trap's mask against the visitor's type | `traps.md` rule 4, `connor-collision.md` rule 2 (Q58: our stand-in confirmed) | Unit test |
| ~~"Touching" at the end of the move~~ **Replaced in 3d-4:** each ball's path against the box widened by its radius (swept, starting inside counts) | `traps.md` rule 6 (Q57 b) | Unit tests |
| No `colliderMask` → 0; no `innerPercent` → full box | **Our stand-in** (not in the note; every trap in the data has a mask, every sticky one an `innerPercent`) | — |

## Traps in Daventry (M3 step 3d-2) — `crates/kq8-core/src/active.rs`, `crates/kq8-viewer/src/{load,door_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| Every placed `KQTrap` becomes a trap; position as written, `keepOnGround` ignored; `locZ=-1` → floor grid height | `traps.md` rules 1, 2 | Daventry: 52 traps, 4 sticky (note: 50; asked, not settled) |
| Final position after the start-up scripts' `move` lines | Our choice, as emitters (3b-1) | — |
| `Init` for each trap at the start, after the doors' | `traps.md` rule 13; the order is **our stand-in** | Log: `script roofTrap.cs roofTrap Init` ... |
| Checked after Connor's update whenever his position changed | `traps.md` rules 6, 7 ("every time an object inside moves"); the order in the update is **our stand-in** | `--at 268280,223800,3.1416 --hold 1,0,1,30`: `Conwalk.tp` On at update 15, Off at 21 |
| Sticky boxes in play | `traps.md` rule 10 | `--at 186500,198000,1.5708 --hold 1,0,1,90`: `mwiztrap` On at x 189,041 (inner edge 189,101 − 200), Off at x 198,106 (full edge 197,857 + 200) |
| `KQCueEvent` to a trap → `Cue n1 n2 [n3]` | `traps.md` rule 14 (as doors rule 23) | Unit test |
| `KQObject::collision <name> <±bits>` changes a trap's or door's mask | `traps.md` rule 16 | Unit test; log: `jumpa.cs` `collision jumptp -4: mask 0x4 -> 0x0` |
| ~~Whoever is inside when the mask changes: no `Off`~~ **Replaced in 3d-4:** they stay inside and get `Off` when they leave; the mask only gates coming in | `traps.md` rule 16 (Q57 d) | Unit tests |
| Scripts looked up in the world, then `GAME/patch`, then `GAME/Kq` | `models-and-textures.md` "Which one wins" (world folders before `GAME/`; patch before Kq: the note's likely reading) | 174 world + 30 game-wide scripts; no name in both |

## What the walk/run zones do (M3 step 3d-3) — `crates/kq8-core/src/{active,script}.rs`, `crates/kq8-viewer/src/{door_view,connor_view,game_camera}.rs`

| What | Source | Check |
| --- | --- | --- |
| `KQConner::setRun 0\|1` sets Connor's run switch | `connor-movement.md` rule 4 | Unit test; `--at 270813,225311,-1.5708 --hold 1,0,0,80 --click Condoor,2`: `Conrun.tp` On at update 18 → run on, `walk -> run` at 19; `Conwalk.tp` On at 26 → run off, `run -> walk` at 27 |
| Any number but 0 counts as on | **Our reading** (the data only uses 0 and 1) | — |
| `KQCamera::setRange <min> <max>`, refused unless min < max | `camera.md` "Distance" rule 2 | Unit test; same run: range (1,000, 2,500) in the house, (1,500, 5,000) outside |
| Applied after the traps' check, before the camera's update in the same tick | **Our stand-in** (order not traced) | — |
| Action-only commands of Daventry's trap scripts are stubs (one log line, result 0); question commands (`getloc`, `checkDistance` (built since, see below), `exist`, `checkSave`, `getConInventory`, `getEndLoop`, `KQConner::getDoMode`) stay unknown (result 1) | **Our choice** until they are studied | Log |
| `--hold F,T,R`: the run switch only at the start | Our test helper (so scripts can change it) | Same run |

## Trap corrections after Q57/Q58 (M3 step 3d-4) — `crates/kq8-core/src/{trap,active}.rs`, `crates/kq8-viewer/src/door_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Entering: swept ball paths against the box widened by the radius; the entering move is not also tested for leaving | `traps.md` rule 6 | Unit test: x 9,000 → 11,000 through a box 9,700–10,300 gives `On`, the next move `Off`; a path 600 to the side nothing |
| Staying: the same swept test for "still touching" | **Our stand-in** (rule 7 doesn't say) | Unit test; viewer: every `Off` one update later than in 3d-3 (`Conrun.tp` 25, `Conwalk.tp` 39, `mwiztrap` 78), `On` unchanged |
| Mask only gates coming in | `traps.md` rule 16 | Unit tests (trap and `active.rs`) |

## Banking corrected after Q56 (M2 step 2i-3) — `crates/kq8-core/src/connor.rs`

Replaces the stand-ins in the 2i table above (which movements lean, the end value, what jumps do).

| What | Source | Check |
| --- | --- | --- |
| "Lean allowed" switch: set only in the keys' updates, on when they pick `run`/`waterrun`; kept in crash, death and other states | `connor-movement.md` "Turning" rule 3 (traced) | Unit tests; `--hold 1,1,1,45`: leans from update 1 (`stoprun`) |
| **2i-4:** switch off while jumping, falling and landing, so he straightens up | **The user's check in the original** (2026-10-13), against the traced rule; asked as Q59 | Unit test; `--hold 1,1,1 --jump 20`: 0.180 at the jump (update 21), upright by 26 |
| Step 0.03 per update (0.45 rad/s) while short of 0.15 before the step: ends at 0.18 rad | Same rule; per second: "Do not reproduce" | Unit test; viewer: 0.030 … 0.180 by update 6 |
| 0.18 on both sides, also after swinging over | "Do not reproduce" (the original's −0.15 there is float rounding) | Unit test |
| Returning: within 0.029 → upright, else one step; the check before the step | Same rule | Unit test (0.04 → 0.01 → 0) |
| Turning, switch on, forward not held: unchanged | Same rule | Unit test |
| With a wall block on, the pick is `stop` (switch off) | **Our reading** (our wall block makes the keys pick `stop`) | — |
| First-person view and `allowLeaning` switch it off | Same rule | Not done (no first-person view yet; nothing in the data changes `allowLeaning`) |


## Item types (M3 step 3e-1) — `crates/kq8-data/src/items.rs`, `crates/kq8-cli/src/item_verify.rs`

| What | Source | Check |
| --- | --- | --- |
| `GAME/Kq/Invinfo.itl`: `[InventoryItemTypeList]`, `numItems`, one `itemN=` line per type, fields tab-separated | `items.md` rule 1; **Observed** in the file (the commented `;itemX=` header names the 23 fields) | `kq8 item-verify`: 135 types read, `numItems=135`, 0 problems, every class `KQInventoryItemType`, no name twice |
| Field order: class, `IDName`, id, …, category (10th), …, land sound (16th), pickup sound (17th), use sound, shape (19th), cursor, radius, translucency, extra data | `items.md` rule 1 (same order as the file's header) | Unit test; log: `INVITEM_Crystal` id 3020, `HealthItem`, `crystl.KQS`, radius 250, translucency 200, sounds `potnland.aud` / `energy.aud` |
| A sound field `none` = no sound | `items.md` "Edge cases" | Unit test |
| Type names: `INVITEM_` put in front unless the name starts with exactly that, then compared ignoring capitals | `items.md` rule 2 | Unit tests; `item-verify`: of 758 uses of a type name in all 12 worlds' scripts and object files, all resolve except `INVITEM_All` (35, `acceptsInventory`) and one id number (`2050`), asked as Q62 |
| Kept: name, id, category, sounds, shape, radius, translucency; the rest (stats, menus) not read yet | `items.md` rule 1 ("for M3 only … matter") | — |
| Each type's model is found in a world's archives or `GAME/Common`, and its box in `BOUNDS.KQ` with the shape name lower-cased | Where the viewer looks for models (M1); lower-cased lookup: `connor-physics.md` Q39 | `item-verify`: all 135 models found; 3 without a box (`truthkey.kqs`, `tablet2.kqs`, `orcxbow.kqs`; Q62 c) |
| `acceptsInventory` read as a comma-separated list in the check | **Observed** in the data (19 files); what the game does with it is Q62 a | — |

## Item objects and inventories (M3 step 3e-2) — `crates/kq8-core/src/{item,active,script}.rs`

| What | Source | Check |
| --- | --- | --- |
| Item object = type, quantity, third number −1; clickable type 0x220, mask 0x809, `doNotPointAt` on | `items.md` rules 3, 4 | Unit test |
| Never collides with Connor | **Our reading** of `items.md` rule 4 with the pair rule (`connor-physics.md`): 0x220/0x809 share no bit with his type 4 / mask 0x281B | — |
| `KQMonster::inventory <who> <type> [qty] [x y z [rx ry rz]]`: `world` → placed, anyone else → carried; `give <type> [qty]` = for Connor; `NewInvItem`; unknown type → error, 1 | `items.md` rule 9 | Unit tests |
| World item z −1 / −2 → floor / surface height | `items.md` rule 9 → `objects.md` Q17 rule 2 | Unit test |
| A world item without a position, a position of 1–2 numbers, or an unknown character → error, nothing made | **Our choice** (every such line in the data gives x y z) | Unit test |
| New item name `<type without INVITEM_>.<n>`, n from 1 in the order made | **Our stand-in**, asked as Q60 | Unit test |
| `KQMonster::checkInventory <who> <type> [var]`: 1 found / 0 not or error; `var` = the object's name | `items.md` rule 10 | Unit test |
| `KQObject::setScript <item> <script\|none>`, `setDontLook`, `setHidden`, `deleteObject` on items | `items.md` rules 4, 7; `chests.md` rule 11 (`none` removes the script); `objects.md` "Hidden objects" (hidden = not drawn, not touchable) | Unit tests |
| On other objects: `setHidden`, `deleteObject` stay stubs; `setScript`, `setDontLook` "not tracked", result 1 | **Our choice** until those objects need them | Unit test |
| `tossItems`, `dropItem`, `changeBmp`, `setGroupsAlways` are stubs | **Our choice**: tossing needs weapons and its flight is not fully traced (`items.md` rule 12, open question 3); `dropItem` only in `rockHit.cs` (M5) | — |
| Clicking a world item: out of reach (≥ 2 × `attackDist`) "too far"; item in hand "doesn't work"; in reach → carried by Connor, pickup sound logged, then its script with `ConnorAction do do` | `items.md` rules 6–8 | Unit tests |
| Picked and measured by its type's shape box in `BOUNDS.KQ` (lower-cased), turned with the item; hidden items not picked | **Our reading** of `chests.md`'s crystal check (no box rule in `items.md`); Q62 c for types without a box | Unit tests |
| World commands can set script variables (`take_vars`) | Our code structure (the runner owns the variables) | Unit tests |

## Items at the start and in the viewer (M3 step 3e-3) — `crates/kq8-data/src/world_script.rs`, `crates/kq8-core/src/item.rs` (`read_make`), `crates/kq8-viewer/src/{load,door_view,item_view,terrain_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| The start-up scripts run `KQMonster::inventory`, `give`, `KQObject::setScript`, `setDontLook` (and `setHidden` on items); `$NewInvItem` set after a make | `items.md` rule 9; `objects.md` "Where a world's objects come from" (the start-up scripts run line by line) | Unit test; Daventry: 35 item lines, 22 items (21 from `WorldInv.cs`, `BirthdaySuit` hidden from `newWorld.cs`) |
| One reader for `KQMonster::inventory` / `give` lines, used at load and while playing | Our code structure | Unit tests (3e-2) |
| Fix 2026-10-22: a start-up `KQObject::setScript <character> <file\|none>` replaces the script of the character placed under that name (`Placer::set_character_script`) | **From the data** (Daventry `newWorld.cs` line 166's comment: the tomb's spriggan "needs a special cs file"); no behaviour note, **Q154** | Viewer: `items.log` "character script now Some("sprigan2.cs")"; killing `sprigan` runs `sprigan2.cs`, then `knight.cs` Cue 10 → `birthx` → speech → `TalkComplete` |
| Lines inside `if` blocks at load are run anyway (the start-up runner does not evaluate `if`) | Same as `loadKQ` (M1 3c) | Daventry, Swamp, Gnome, DeadCity: 0 such item lines |
| z −1 / −2 at load → floor / surface height | `objects.md` Q17 rule 2 | Daventry: 8 mushrooms at −1 → 7,650 and 13,770 |
| Drawn with the type's shape at its place and turn; nothing else moves it | `items.md` rules 1, 5 | Not checked by eye yet |
| See-through by the type's translucency, as objects (n/255, 256 solid) | **Our reading**: `items.md` rule 1 lists translucency; the formula is `objects.md` Q14's | Crystal 200 → 0.78; not checked by eye |
| Hidden items not drawn | `objects.md` "Hidden objects" | `BirthdaySuit.22` hidden |
| Models loaded for every type the world's data can make or hold (start items, `inventory`/`give`/`tossItems` lines, chests' `invItemTypeName`), looked up in the world and `GAME/Common` | Same lookup as objects (M1) | Daventry 25 types, none failed; Gnome: `schroomC.kqs` only in Swamp's archives, so not loaded (no Gnome start item needs it) |
| Picking up in the viewer: the clicks of 3a-3 (mouse ray or `--click NAME,N`) | `items.md` rule 6 | `--at 266011,221900,0 --click Mushroom.14,10`: in reach (507), picked up 0 → 1; `Crystal.20` too far (32,820); `--at 228081,202000,0 --click SilverCoins.10,10`: `Farmcoin.cs` runs, removes its script |

## Chests (M3 step 3e-4) — `crates/kq8-core/src/{chest,door,active}.rs`, `crates/kq8-viewer/src/{load,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| Keys `invItemTypeName`, `invQuantity` (default 1), `invLocX/Y/Z`, `invDirX/Y/Z`, `Take…`/`Empty…` (six numbers) | `chests.md` rule 1 | Viewer log: `smlchst3` 3 crystals at (202336, 263155, 11550), Empty [0, 109, 14, 33, 1, 1] |
| No `invLoc` → the chest's own place | **Our choice** (every chest in the data has one) | — |
| Default type 0x8021 unless `colliderType` | `chests.md` rule 2 | `wizchest1` 0x1, the others 0x8021 |
| `closed` / `open` played as doors' movements (shared phase step); no `opened`: stays on the last frame; `EndLoop <movement>` to the script when it finishes | `chests.md` rules 3, 6; the step as `doors.md` rule 9 | Unit tests; viewer: `open` 15 updates (11–25), `EndLoop open` |
| Placed by the shown frame's steps, as doors | **Our reading** (`objects.md` Q51 for whole-object steps); `chest2.kqs`/`wizchest.kqs` turn 0° | Log |
| Click: reach, "too far", item in hand "doesn't work" as doors; the script is not run on clicks | `chests.md` rules 4–6 | Unit tests |
| 1st click: `open` plays, the item made at `invLoc`/`invDir` (z −1/−2 as `loadKQ`), remembered, state open; no type → empty | `chests.md` rule 7 | Unit tests; viewer: `Crystal.23` x3 |
| Unknown item type → counts as holding nothing | **Our choice** | — |
| 2nd click: remembered item into Connor's inventory; already carried → only the state; `Take` remark if Seq ≠ 0; state empty | `chests.md` rule 8 and "Edge cases" | Unit tests; viewer: 0 → 3 |
| 2nd click: no pickup sound | **Our stand-in**, asked as Q61 | — |
| Later clicks: `Empty` remark if Seq ≠ 0 | `chests.md` rule 9 | Unit test; viewer |
| Connor turns before a remark unless `doNotPointAt`, not on the 1st click | `chests.md` rules 8–10 | Logged only (turning not done, as doors) |
| `Init` after the doors', before the traps' | `chests.md` rule 6; the order is **our stand-in** | Log |
| `KQCueEvent` → `Cue`; other events not handled | `chests.md` rule 6 (Cue); others not in the note | — |
| `collisionType` and `setHidden` on chests (`wizchest.cs` Reveal) | `chests.md` "Edge cases"; `objects.md` "Hidden objects" | Unit test |
| Hidden chests still collide with Connor | As other hidden objects (M2 2d-5) | — |

## Clicks need the camera's line of sight (M3 step 3e-4b) — `crates/kq8-core/src/active.rs` (`pick_seen`, `target_point`), `crates/kq8-viewer/src/{door_view,game_camera,main}.rs`

Replaces "walls, terrain and other objects do not hide a clickable one" (`doors.md` Q45 b) in the doors' table above.

| What | Source | Check |
| --- | --- | --- |
| No clicks through walls or the ground; from behind a wall a click works if the camera sees the object | **The user's check in the original** (2026-10-13), against `doors.md` Q45 b; asked as Q63 | Viewer `--aim` runs below |
| What blocks: the camera's view obstacles (drawn ground, buildings and objects with type bit 0x8 or 0x1000, doors as they stand) | **Our stand-in** (Q63 a): `camera.md` "Obstacles" reused; plain objects (tables, trees) don't block | — |
| A blocked click clicks nothing; reach unchanged | **Our stand-in** (Q63 b, c) | Unit test |
| The line is tested up to 1 unit short of the picked box (an object never blocks itself) | Our method | Unit test (wall at the box's face: seen); viewer: `Condoor` seen |
| `--aim NAME,N`: a ray from the camera to the middle of NAME's box at update N, through the same test as the mouse | Our test helper (`--click` skips the ray) | `--at 202336,262000,1.5708 --aim smlchst3,20`: blocked by building `Pub` at 5,319 (chest at 5,974); `--at 266011,221900,0 --aim Mushroom.14,20`: seen, picked up; `--at 270813,225311,-1.5708 --aim Condoor,20`: seen, clicked |

## Scripted objects (M3 step 3e-5) — `crates/kq8-core/src/{named_anim,chest,active}.rs` (`named_anim.rs` was `movement.rs`), `crates/kq8-viewer/src/{load,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| Which: placed `KQAnimObject`s with `script=`, the click bit 0x20, and a still movement at load; one that plays stays with `anim_view` (not clickable yet) | **Our choice** (Q64 d) | Daventry 14 (`smlchst1/2`, `jar1`, 6 baskets, `coffnlid`, `Bricks`, `MillTree`, `wizbook`, `wizquill`); 10 left to `anim_view` (signs, fish, birds, `starball`, `girltree`) |
| Movements played by a shared player (phase step and cues as doors, play-once stops on its last frame); chests use it too | `doors.md` rules 6, 9; `chests.md` rule 3 | Unit tests (`named_anim.rs`); chest tests unchanged |
| Click: `ConnorAction do do` in reach; item in hand by `acceptsInventory`/`acceptAllItems`; out of reach by `tooFarAwayAction`; turn unless `doNotPointAt` (logged) | `chests.md` rule 11 ("the script runner and the commands above"), as `doors.md` rules 12–16; **our reading**, Q64 a | Unit tests; viewer: `smlchst1` (35 coins at 300 beside it, `Tavcoin.cs`), `basket1` (`give mushroom 5`) |
| `KQAnimPlayEvent <movement>` plays it | `doors.md` rule 6, applied to these objects (Q64 b) | Unit test; viewer `open` |
| `Init` at the start (after chests), `EndLoop <movement>` when a play-once movement ends, `KQCueEvent` → `Cue` | `chests.md` rule 6, applied to these objects (**our stand-in**, Q64 c) | Unit tests; viewer: `smlchst1.cs smlchst1 EndLoop open` |
| `collisionType`, `setHidden` on them | As chests (3e-4) | — |
| Box moves with the shown frame's steps (collider updated each update) | As doors and chests | — |
| Click shares the doors' argument rules (`script_args`) | Our code structure (doors unchanged) | Door tests unchanged |

## Clicking objects that move at load, `deleteObject` (M3 step 3f) — `crates/kq8-core/src/{active,script}.rs`, `crates/kq8-viewer/src/{load,terrain_view,anim_view,door_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| Clicking the swimming fish: it disappears, Connor makes an eating sound and gains health | **The user's check in the original** (2026-10-13); `fishies.cs` (data) does it | Viewer below |
| Every clickable placed object with a script (not a door, chest, item or 3e-5 object) is clicked like a door: `ConnorAction`, `acceptsInventory`, `tooFarAwayAction`, `doNotPointAt` | **Our stand-in** (Q64 a, d) | Unit test; Daventry: `fishies.01/02` |
| Their script gets `Init` at the start (after the 3e-5 objects) and `Cue`; `KQAnimPlayEvent` to them is logged, not done | **Our stand-in** (Q64 b, c) | Log: 118 `Init` calls in Daventry; Gnome/IceWorld: one unknown command each in an `Init` branch (`KQMonster::clearSpeak`, `getEndLoop`) |
| `deleteObject` on such an object or a 3e-5 object: gone for good (not drawn, not clickable, no collision, no view box; events to it dropped); on other placed objects still a stub | **Our stand-in** (Q64 d) | Unit tests; viewer: `fishies.01` "1 collider(s) switched off, hidden", its cue 1 s later "deleted; dropped" |
| A switched-off collider: size 0, 10,000,000 below the world | Our method (the collider list keeps its order) | — |
| `KQSound::play` is a stub (no audio yet) | Our choice | Log |

## `test` leaves −1 (M3 step 3g-1) — `crates/kq8-core/src/script.rs` (`test_return`)

| What | Source | Check |
| --- | --- | --- |
| A false text `==` leaves `-1` when the left word comes first in byte order (a word that runs out first comes first), `1` when the right one does; every other false test (`!=`, `<`, `<=`, `>`, `>=`, numbers) leaves `1`; true leaves `0` | `doors.md` "Q45" (e), Q54 a (traced by A) | Unit test `a_false_text_equal_leaves_minus_one_when_the_left_word_comes_first` |
| `set` and `echo` leave `0` | `doors.md` "Q45" (e), Q54 b (traced by A); the code already did this | Existing tests |

## What the click line hits (M3 step 3g-2) — `crates/kq8-core/src/active.rs` (`ActiveBuilding`, `ClickWalls`, `pick_line`, `click_building`), `crates/kq8-viewer/src/{load,door_view,game_camera}.rs`

Replaces the 3e-4b stand-in (clicks blocked by the camera's view obstacles).

| What | Source | Check |
| --- | --- | --- |
| Only shapes with bit 0x20 are tested, nearest wins; boxes as they stand, buildings by their BSP line test | `clicking.md` rules 2, 5 (traced by A, Q63) | Unit test `a_clickable_wall_in_front_takes_the_click` |
| Buildings default to type 0x28 (clickable); `colliderType=136` ones are not tested | `clicking.md` rules 3, 4 | Log: Daventry 23 of 26 buildings, not Church, Cave, WizHouse |
| The ground is never tested; objects without 0x20 never stop the line | `clicking.md` rules 5, 6 | Unit test |
| A picked building with no script: nothing (no remark, no turn), in reach or out (`tooFarAwayAction` default 1 for buildings); with an item in hand in reach: "that doesn't work"; with a script: the door calls | `clicking.md` rules 9–12 | Unit test `a_building_without_a_script_swallows_the_click`; viewer `--aim smlchst3,20` from the pub's ground floor: "hits building Pub first, at 5319 units; it hides smlchst3 at 5974" → Swallowed |
| Hidden objects stay unclickable | `objects.md` "Hidden objects" 3 with `object-lighting.md` rule 41 (hiding drops the collider), which `clicking.md` rule 7's "not skipped by the line" does not override | Existing tests |
| Click line length 1,000,000 | **Our stand-in** (`clicking.md` open question 1) | — |
| Reach to a building: distance to its world bounding box | **Our stand-in**, asked as Q75 | Log: Connor in the pub 22.6 from its box |

## Accept lists (M3 step 3g-3) — `crates/kq8-core/src/item.rs` (`Accepts`), `crates/kq8-core/src/active.rs` (`accepts_entry`, `set_accepts`, `script_args`)

| What | Source | Check |
| --- | --- | --- |
| `acceptsInventory` split at commas; `all` / `INVITEM_All` (any capitals) = accept everything; other words are type names (rule-2 lookup) until `none`; `acceptAllItems` non-zero = the same switch; the held item's **type** is compared | `clicking.md` rules 13–15 (traced by A, Q62 a) | Unit test `accept_lists_split_at_commas_and_compare_types`; viewer: keep doors accept `INVITEM_KeepKey` (also given as `keepkey`), refuse a crystal |
| `all` honoured anywhere in the list; unknown type names skipped and logged | **Our choice** from `clicking.md` "Do not reproduce" | Unit test |
| Read when the object enters the world (before `Init`), or when first asked | `clicking.md` rules 14, 21 | Log: Daventry 7 objects with a list |
| `KQObject::setAccepts <object> <value>` adds, never empties | `clicking.md` rule 16 | Unit test `set_accepts_adds_to_what_the_door_takes` |
| `$4` of an accepted click is the type's own spelling from `Invinfo.itl` | `clicking.md` rule 18 ("its type name") | Viewer: `--item keepkey` → `ConnorAction keepkey INVITEM_KeepKey`, `KeepDoor.cs` opens both doors |

## `EndLoop` through the queue, `getEndLoop` (M3 step 3g-4) — `crates/kq8-core/src/active.rs` (`movement_ended`, `get_end_loop`, `call_named`, `cue_args`)

| What | Source | Check |
| --- | --- | --- |
| A play-once movement's end: the object cues itself 3333 through the event queue (only if it has a script), and its script gets `EndLoop <movement>` | `clicking.md` rule 21 (traced by A, Q64 c) | Unit tests; viewer: `basket1` "movement 'open' ended" in update 11, "EndLoop open" in update 12 |
| Doors get `EndLoop open` / `close` too, after `Open` / `Close` | `clicking.md` rule 21 | Unit test `a_door_gets_end_loop_after_open_and_get_end_loop_relays` |
| `getEndLoop <from> <to> 1|0`: one listener per `<from>`, a new one replaces the old; `<to>` gets `Cue 300 0 <movement>` at the same moment | `clicking.md` rule 21 (what is sent: traced; bookkeeping: inferred by A) | Unit test |
| Connor as `<from>` (`bricks.cs`): recorded, but nothing relays yet (his movements aren't played by scripts) | Our limit until Connor's script states exist | Log line |
| Cue 500 = projectile: logged, nothing done | `clicking.md` rule 21 (combat not studied) | Unit test |
| A cue's third value is passed as `$5` when it is a word or a non-zero number | `doors.md` rule 23 ("only if non-zero") with `clicking.md` rule 21 (`$5` = the movement's name); **our reading** of "non-zero" for a word | Unit test (`Cue 300 0 open`) |

## `deleteObject` for every object, `NoName<n>`, the pickup sound on every add (M3 step 3g-5) — `crates/kq8-core/src/{active,item}.rs`, `crates/kq8-viewer/src/door_view.rs`

Replaces the 3f stand-in for `deleteObject` and the 3e-2 stand-in for item names.

| What | Source | Check |
| --- | --- | --- |
| `deleteObject` marks any object deleted at once (items, doors, chests, scripted and clickable objects, traps, any other placed object): not clicked, not solid, not drawn; events to it dropped; `0` if something was deleted; an unknown name prints `deleteObject: Object "<name>" doesn't exist` and leaves `1` | `clicking.md` rule 22 (traced by A) | Unit tests `delete_object_on_a_scripted_object_and_on_other_objects`, `delete_object_on_doors_and_chests_and_several_names`; viewer: `fishies.01` deleted, its cue dropped, a later click does nothing |
| Every listed name is deleted (the original stops after the first that exists) | **Our choice**, `clicking.md` "Do not reproduce" | Unit test |
| Removed "at the end of the update" in our terms: the viewer applies the request after the scripts of that update | `clicking.md` rule 22 ("for us: remove it at the end of the current update") | Log: "applied: fishies.01 deleted" |
| A deleted building is no longer clicked; its solid space for Connor and the camera stays | **Our limit** (none of Daventry's deleted names is a building) | Log line says so |
| New item objects are named `NoName<n>`, one counter from 0 | `items.md` rule 9 (Q60, traced) | Unit tests; viewer: Daventry's start items `NoName0` … `NoName21` |
| The counter lives with the world's item list (no world travel or saving yet) | **Our limit until M6** | — |
| The type's pickup sound on every add to Connor's inventory (pickup, chest 2nd click, `give` / `inventory Connor`), not placed | `items.md` rule 7, `chests.md` rule 8 (Q61, traced) | Viewer: `smlchst3` 2nd click "pickup sound energy.aud (no audio yet)" |

## The draw-distance ball and items (M1 step 7a) — `crates/kq8-viewer/src/{draw_distance,terrain_view,anim_view,door_view,item_view,load}.rs`

Replaces the 6h stand-in (every ball from the model's geometry).

| What | Source | Check |
| --- | --- | --- |
| The ball is always a `BOUNDS.KQ` box: the collision box (an animated object's current movement's shape; doors, chests and scripted objects their movement's shape), else `shapeName` lower-cased, then the own name; none: radius 100 at the position | `object-lighting.md` rule 35 (Q70, traced by A) | Log "balls by source": Daventry 297 by shape, 24 radius 100; Dead City 928 by shape, 1 by own name, 173 radius 100 |
| Worked out once, when the object is spawned | Rule 35 | — |
| The centre is turned with the object | **Our choice**, rule 35 / "Do not reproduce" (the original doesn't turn it) | — |
| Buildings keep the ball round their geometry | **Our stand-in**, asked as Q76 (only 6 buildings of the game are in `BOUNDS.KQ`) | Log: Daventry 26, Dead City 35 |
| Items: their type's shape in `BOUNDS.KQ`, else 100; `tooFar` 0.83, `groupsAlways` 0x2, near 0x448, far **0** (not drawn when far) | `object-lighting.md` rule 43, `items.md` 5b (Q71, traced by A) | Log: Daventry from the start, 3 items "Far but not drawn … far 0x0" |
| Dead City's flames unchanged (no entry: radius 100) | Rule 38 | `--world DeadCity --at 130800,172122,-1.5708 --hold 1,0,0,90`: `flame.05` Off → Far at 12,097, Far → Near at 10,041 |

## The collider switch, `setActiveCamera`, `makeActive` (M1 step 7b) — `crates/kq8-core/src/active.rs` (`switch_collider`, `is_switched_off`, `is_hidden`, `Request::ActiveCamera` / `MakeActive`), `crates/kq8-viewer/src/{draw_distance,door_view,anim_view,game_camera,item_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| When an object's groups lose bit 0x8 its collider leaves the collision world (Connor, the camera's view line and clicks pass through); when they gain it, it comes back where the object is; only a change of near / far / off switches | `object-lighting.md` rule 41 (Q74, traced by A) | Unit test `a_switched_off_collider_is_not_clicked_and_scripts_ask_for_the_reference_point`; viewer: Daventry 335 switches at the start, `spinweel` out at update 0 (far, `tooFar` 0.3), back at update 15 walking in |
| A hidden object is always off, so (with default masks) not solid and not clickable | `object-lighting.md` rule 36, `objects.md` "Hidden objects" 3 | Log (changes M2's "hidden objects still collide") |
| `setActiveCamera <n>`: reference point Connor (0) or the camera, from the next frame; usage line if not 2 words | Rule 42 | Unit test; not run in the viewer (only Dead City scripts use it) |
| `makeActive <name> <n>`: always near (≠0) or back to the test; a hidden object stays off | Rule 42 | Unit test; not run in the viewer |
| The test runs every drawn frame; the switches are applied at the start of the next game update | **Our timing** (the original does both in its one frame loop) | Log: "update N: collider switch" |
| Saved boxes are put back as they were; doors, chests and moving objects are then moved by their own update; a deleted object stays out | Our method | — |
| Buildings: no longer clicked while out; their BSP stays solid for Connor and the camera | **Our limit** (a building is only out beyond `visibleDistance` from Connor) | — |

## Lights carried by models and `KQPointLight` objects, the data (M1 step 7c-1) — `crates/kq8-data/src/shape.rs` (`ModelLight`, `sequence_lights`), `crates/kq8-cli/src/lighting_verify.rs`, `crates/kq8-viewer/src/load.rs`

| What | Source | Check |
| --- | --- | --- |
| A frame's lights: the nodes of its sequence's light sub-sequence, at most 5; none for −1 | `object-lighting.md` rule 26b (traced by A) | Unit test `a_sequence_names_its_lights` |
| Colour = byte ÷ 255 × intensity scale × level × 0.2, capped at 1; versions 1–3: level and scale 1.0 | Rule 26b | Unit test; `lighting-verify`: `portlite.kqs` colour (0.200, 0.200, 0.013) → (0.000, 0.200, 0.111), A's (1.0, 1.0, 0.07) → (0, 1.0, 0.56) × 0.2 |
| Point light when flag bit 0x1 is set (else sun-type) | Rule 26b says "flag bit 1"; **our reading** of which bit from the data (flags only 1 and 5) and A's example; asked as Q77 a | `lighting-verify`: flags {1: 12,216, 5: 615} |
| `KQPointLight` objects loaded by world scripts (`loadKQ light/Plight01.kq`) are found in `data/<World>/KQ/` like the sun files; kept by name | Rules 1, 24, 25 | Viewer Dead City: all 7 load (`PLight01`, `Facelt03`–`08`), positions and colours as rule 25 |
| Their place in the draw-distance test: the light's position, radius 100 | **Our stand-in**, asked as Q77 b | — |

## Point lights on models (M1 steps 7c-2, 7c-3) — `crates/kq8-viewer/src/{scene_lights,object_light,object_light.wesl,draw_distance,terrain_view,anim_view,door_view,main}.rs`, `crates/kq8-data/src/shape.rs` (`Triangle::part_ball`)

| What | Source | Check |
| --- | --- | --- |
| All lights gathered once per frame into one shared list | `object-lighting.md` rule 3 | Log "scene lights" once a second |
| A `KQPointLight` and a model's carried lights shine only while their owner has group bit 0x400 (near or far with default masks; not off, not hidden, not deleted) | Rules 26b, 37 | Dead City from the start: 2 of 7 point lights + portal (the others are off, > 12,000 from Connor) |
| A carried point light's position: node position through the object's placement only (not the frame's step); sun-type aims as stored in world axes | Rule 26b (traced by A) | Dead City by the portal: the light moves 853–888 from Connor and changes colour second by second |
| Per model part: out of range when (distance to the part's ball centre) − radius > End; facing = max(0, −(unit light→centre · corner normal)); falloff from the corner's own distance: 1 to Begin, straight to 0 at End; added before the cap | Rule 24 (traced by A) | Unit test `falloff_on_models`; log: Connor 877 from the portal light gets (0.200, 0.200, 0.013) |
| Part ball centre = middle of the part's vertices in that frame; radius = the mesh's stored radius (not scaled by the node's transform) | Rule 24's stand-in for the centre; the radius as the note says, **our reading** that it is not scaled | — |
| Flat materials: the first corner's normal at all three corners, each corner's own position | Rule 7 with rule 24; **our choice** for the position | — |
| At most 128 lights per frame on the card (more logged) | **Our limit** | Log (Daventry 5, Dead City 2 from the start) |
| Lights carried by items and Connor: not gathered | **Our limit** (none known to carry lights) | — |

## Q73 leftovers checked (M1 step 7f) — `crates/kq8-viewer/src/{ground_haze.wesl,building_light.rs,building_light.wesl,light_view.rs}`

| What | Source | Check |
| --- | --- | --- |
| Wall light maps sampled bilinear | `object-lighting.md` rule 13 (traced 2026-10-17) | Code: `ImageSamplerDescriptor::linear()` (already so) |
| No colour key on walls or the ground: pure black is drawn black | Rule 14 (traced) | Code: both opaque, texture alpha unused (already so) |
| The ground's far-edge cut | Rule 28 (traced): t = (visibleDistance − d₁) ÷ (d₂ − d₁) along each edge, the rest blended with t | Our per-pixel cut on blended corner distances is the same line; comment updated, no code change |
| The ground's light map between points: per triangle (vertex colours), **not** bilinear | Differs from `terrain-lighting.md` rule 11 / `object-lighting.md` rule 13 (now traced bilinear) | Planned as M1 7e-0 |

## The ground's light map, bilinear (M1 step 7e-0) — `crates/kq8-viewer/src/{light_view,ground_haze,terrain_view,ground_anim_view}.rs`, `ground_haze.wesl`, `crates/kq8-data/src/lightmap.rs` (`with_size`, `set`)

Replaces the M1 4c stand-in (vertex colours blended per triangle).

| What | Source | Check |
| --- | --- | --- |
| The shown map is one picture, texel (i, j) = map point (i, j), red/green/blue = channel × 17 (read back as channel ÷ 15), not sRGB | `terrain-lighting.md` rules 7, 9, 10 | Unit test `the_picture_holds_the_map_and_the_lookup_hits_its_points` |
| Looked up per pixel from the world position, u = (x ÷ spacing + 0.5) ÷ side, bilinear, clamped at the edges | Rule 11 (recommended: world coordinates, bilinear); `object-lighting.md` rule 13 (traced 2026-10-17: the light-map texture unit is bilinear) | Unit test (point centres, half-way = texel edge = `colour_at`'s average); viewer: Daventry 513, Swamp 257, Barren 129 (no file: full) run without shader errors |
| F7 modes ×1 / ×2 / off applied in the shader; the sRGB-curve multiply stand-in as before | M1 4c (unchanged) | — |
| The meshes stay cut into map cells | Our choice (harmless; may be simplified) | — |

## Point lights on the ground (M1 step 7e) — `crates/kq8-data/src/{lightmap,objects}.rs` (`paint_light`, `add_light`, `fade_falloff`, `turn_off_dyn_lighting`), `crates/kq8-viewer/src/{light_view,scene_lights,load,terrain_view,anim_view,door_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| Only with `useDynamicLighting 1` (shown map = shadow map) | `object-lighting.md` rule 26, `terrain-lighting.md` rules 2, 15 | Log says "(useDynamicLighting 0: none)" otherwise |
| Each frame: last frame's patches put back first, then each light painted on the current values | Rule 26 (traced by A) | Log: "357 point(s) painted, 357 put back" |
| Which lights: point lights of the gathered list (lights group, not deleted); carried lights of an object with `turnOffDynLighting=1` don't reach the ground | Rules 26, 26b | — (none of Dead City's start lights have the key) |
| h = light z − drawn ground below; h > End: nothing; square of points within √(End² − h²) on x and y | Rule 26 | Unit test; Dead City `PLight01` h 1,386 → half-width 3,752, 225 points |
| A light more than End **below** the ground paints nothing | **Our choice** (the square would have no size) | Unit test |
| d = √(dx² + dy² + h²); falloff 1 to Begin, 1 − (d − Begin) ÷ End, ≥ 0 (End a fade length); channel ÷ 15 + colour × falloff, capped at 1, stored ⌊× 15.9⌋; brightness bits kept | Rule 26; **our reading** that the brightness bits stay (the note names only the colour channels) | Unit test `point_lights_paint_the_ground` |
| At most 256 patches and 65,536 points a frame | Rule 26 | Unit test (points) |
| Only touched texels of the picture are rewritten | Our method (7e-0's picture) | — |

## Point lights on building surfaces (M1 step 7d) — `crates/kq8-data/src/{dil,wall_light}.rs`, `crates/kq8-cli/src/lighting_verify.rs` (`rect_check`), `crates/kq8-viewer/src/{wall_lights,building_light,building_light.wesl,terrain_view,load,main}.rs`

All from `object-lighting.md` rule 16 (rewritten 2026-10-16, Q78) unless marked.

| What | Source | Check |
| --- | --- | --- |
| Table D bytes 12–47: P0, U, V (floats, building space); none in 12-byte records | Rule 16, `.dil` layout | Unit test `layout_and_texels`; `lighting-verify`: U and V equal rule 13's 16 × (width − 1) texels |
| Every `.dig` surface counts (walls, floors, ceilings, roofs) | Rule 16 | Log: Connor's hut, floor and wall surfaces lit by `fireset.01` |
| Lights: `KQPointLight`s and point lights carried by models in the lights group, not `turnOffDynLighting=1`; only with dynamic lighting on | Rules 16, 26b, 37 | Log: "(dynamic lighting off: none)", 0 lit with `Options.cs` 0 / `--dynamic off` |
| Every frame: last frame's surfaces back to their `.dil` texels, then lights added one after another on the rounded result | Rule 16 | Log: "63 surface(s) lit, 62 put back" (the fire animates) |
| Lights turned into the building's own axes (the collider's placement) | Rule 16 | — |
| Facing = plane normal if byte-0 bit 0x80, else reversed | Rule 16, `buildings.md` (corrected) | Unit test; `lighting-verify`: agrees with the corner order on 164,835 of 164,838 surfaces; viewer log: all of Daventry's |
| In front: facing · unit(P0 − light) < −0.1 | Rule 16 | Unit test `in_front_is_measured_to_p0` |
| Light exactly on P0: not in front | **Our choice** (direction undefined) | Unit test |
| Reach: nearest of the 4 corners and centre < Begin + End | Rule 16 | Unit test `reach_uses_the_four_corners_and_the_centre` |
| Line test from the light to 64 units from that spot towards the light, against this building's BSP only (`BuildingCollider::line_hit`) | Rule 16 | Log: `fireset.01` 20 surfaces blocked, 24 lit |
| Falloff 1 to Begin, then 1 − (d − Begin) ÷ End; channel ÷ 15 + colour × falloff, capped, ⌊× 15.9⌋; brightness bits kept | Rules 16, 26 (the ground's `fade_falloff`, `add_light`) | Unit test `painting_adds_and_rounds_like_the_ground` |
| Texel positions: true spacing U ÷ (width − 1), V ÷ (height − 1); one column / row at P0 | Rule 16 "Do not reproduce" (not the original's U ÷ width) | Unit test `texels_sit_at_the_true_spacing` |
| Texels start ¼ step **before** P0 (P0 lies 4 texture texels past rule 13's texel (0, 0)) | **Measured** (`lighting-verify`: 164,685 of 164,838 surfaces within 1 unit); the note says to use rule 13's positions. Asked as **Q79** | Unit test |
| Haze × 0.04 per light that passes all tests, after rule 29's cap (past 1 → 0.99), before the "0.99 or more: not drawn" test | Rules 16, 29 | Not checked by eye |
| All surfaces of a drawn building copy are tested, not only those in view | **Our simplification** (a surface out of view can't be seen) | — |
| Quick skip: light ≥ Begin + End + 256 from the `.dig` box | **Our method** (only saves time) | Log "skipped" |
| Each building copy has its own light picture and haze factors | **Our method** | Log: 26 Daventry copies, 35 Dead City |
| `--dynamic on|off` overrides `Options.cs` | **Viewer helper** (the game's files are never written) | Log: "ground light: … (--dynamic on)" |

## `KQPointLight` always shines (M1 step 7g) — `crates/kq8-viewer/src/{scene_lights,terrain_view}.rs`

Replaces the M1 7c stand-in (the light's position, radius 100, default masks).

| What | Source | Check |
| --- | --- | --- |
| A `KQPointLight` is not in the near / far / off test; it stays in the lights group from when it is added until it is removed, and shines every frame however far Connor is | `object-lighting.md` rule 37 (corrected 2026-10-16, Q77 b) | Dead City from the start: "scene lights: 8 (7 KQPointLight, …)" (before: 1) |
| A deleted `KQPointLight` gives no light | Rule 37 ("until it is removed"); `deleteObject` as for other objects | — (no script deletes one in the data we ran) |

## A building's draw-distance ball (M1 step 7h) — `crates/kq8-viewer/src/{draw_distance,terrain_view}.rs`

The code was already this way (a building `Model`'s box is the `.dig` header box); relabelled from "Q76 stand-in".

| What | Source | Check |
| --- | --- | --- |
| Buildings never use `BOUNDS.KQ`; radius = half the `.dig` header box's diagonal | `object-lighting.md` rule 40 (traced, Q76) | Log: Daventry `csbridge` 4,398, `Church` (`nchurch.dig`) 18,990 (note: 4,400, 19,000) |
| Centre = the box middle turned with the building | Rule 40 "Do not reproduce" recommendation; **the original uses the building's position** | Log: `csbridge`'s box middle 13,992 from its position (note: 14,000) |

## Reach to a clicked building (M1 step 7i) — `crates/kq8-core/src/active.rs` (`ActiveBuilding::bounds`, `placement`, `click_building`), `crates/kq8-viewer/src/load.rs`

Replaces the M3 3g-2 stand-in (distance to the axis-aligned world box around the turned building box).

| What | Source | Check |
| --- | --- | --- |
| Reach to a picked building: Connor's feet turned into the building's own axes, distance to the nearest point of its `.dig` header box (0 inside), against 2 × `attackDist`; not where the click line met the wall | `clicking.md` rule 9 (traced, Q75) | Unit test `reach_to_a_building_is_in_its_own_axes` (a point inside the old world box is 424 outside the turned box); viewer: Pub 22.6, in reach, "that doesn't work" with a crystal |

## F9: dynamic lighting on / off (M1 step 7j) — `crates/kq8-viewer/src/{light_view,load,main}.rs`

A viewer helper (the user's request), standing in for the original's video option.

| What | Source | Check |
| --- | --- | --- |
| Dynamic lighting on = the ground shows the shadow map, off = the light map | `terrain-lighting.md` rule 2 | Log: both maps load (Daventry `davelght.bin` / `daveshdw.bin`, Dead City `dedlight.bin` / `dedshado.bin`, all 513 × 513) |
| Point lights paint the ground and building surfaces only while on; models keep theirs | `object-lighting.md` rules 16, 24, 26 | Unit test `f9_swaps_the_maps_and_forgets_the_patches`; walls follow the same switch (`light_walls`) |
| Switching while running: the whole picture rewritten, last frame's patches forgotten, ground lookup updated | **Our method** | Not tested by a test option (key press) |

## Light-map placement check (M1 step 7k) — `crates/kq8-cli/src/lighting_verify.rs` (`rect_check`)

Checks only; no game behaviour changed.

| What | Source | Check |
| --- | --- | --- |
| Each corner's light texel position (u × (size + 1) + map offset) ÷ 16, before the clamp: none below 0 or above width − 1 / height − 1 | `object-lighting.md` rule 13 ("How the drawing places it", Q79), "How to check" light-map placement | Log: 0 of 656,181 corners outside on 164,838 surfaces (matches). Surfaces with a corner at exactly 0: 365 (x and y) / 7,276 (x or y), note says 7,511: **Q80 open** |
| P0 minus the drawn texel (0, 0), in texture texels (P0 put into texture texels through the surface's placement) | Rule 16 (P0 a quarter light texel past texel (0, 0), Q79), "How to check" | Log: mean (4.000, 4.000) (matches); within 0.05 of (4, 4) on 164,640, note says 160,649: **Q80 open** |
| Our point-light painting (`wall_light::texel_position`) against the drawn texel positions (0, 0) and last, maps 1 texel wide or high counted apart | "Do not reproduce" (wall light texels), rule 13 | Log: 164,528 of 164,838 within 0.05 texture texel (the rest are surfaces whose stored P0 is itself off); no `.dil` has a map 1 texel wide or high. `wall_light.rs` unchanged |

## Sound files (M3 step 3h-1) — `crates/kq8-data/src/aud.rs`, `crates/kq8-cli/src/aud_verify.rs`

"Community note" = `libkq8fpc/doc/kq8aud.txt` (Nico Bendlin, GPL-3.0
repository, read 2026-10-17 from <https://svn.nicode.net/libkq8fpc/doc/>).
Facts only; code and wording are our own. Numbers checked with
`kq8 aud-verify` (`kq8-artifacts/logs/aud-verify.log`).

| What | Source | Check |
| --- | --- | --- |
| SOL accepted when bytes 0–1 `0D 0C`, 2–5 `SOL\0`, format has no bits outside 0x1D and has 0x04 or 0x08; else WAV (tag 1, 1–2 channels, 8 or 16 bits, `data` chunk); else refused | `sound.md` rule 12 | Unit test `acceptance_rules`; log: 1,749 SOL + 1 WAV of 1,750 `.aud`, 0 refused (matches) |
| Header: rate u16 at 6, format at 8, data length u32 at 9; byte 1 ignored, data at byte 14 | `sound.md` rules 2, 10; community note (layout) | Log: header + data = file length in all 4,598 SOL files (matches) |
| 0x01 compressed, 0x10 stereo; stereo bytes alternate left, right | `sound.md` rules 3, 7 | Log: 92 stereo `.aud`, `FOREST.AUD` 2 ch 178.5 s (matches) |
| Compressed: low 7 bits pick a step, 0x80 subtracts, each channel from 0, 16-bit wrap-round | `sound.md` rules 7, 8 | Unit tests; log: 0 wrap-rounds, largest peak 32,768 (matches) |
| The 128 step numbers | Community note (`sound.md` rule 7: same as the game's); our code builds them from their pattern | Unit test `step_table_matches_the_community_note` (19 entries, one or two per stretch) |
| Samples per channel = data length ÷ channels; uncompressed SOL = signed 16-bit | `sound.md` rule 9 | Log: `AXEDOOR.AUD` 11,847, `NONE.AUD` 8 (matches); uncompressed: unit test only (no game file) |
| WAV: read only the needed fields | `sound.md` "Do not reproduce" 6 | `hm_iced.aud`: 22,050 Hz, mono, 16-bit (matches rule 1) |
| A 14-byte WAV format chunk (no bits field) takes its bits from the block size; a cut-off file decodes what is there | **Our choice** (no game file has either) | Unit test `a_cut_off_file_decodes_what_is_there` |
| `--wav`: writes a plain 16-bit PCM WAV to `kq8-artifacts/wav/` | **Our method** (a listening aid) | Python's `wave` module reads the three files written (`AXEDOOR`, `FOREST`, `hm_iced`) with the right rate, channels and length |

## Playing sound effects (M3 step 3h-2) — `crates/kq8-data/src/aud.rs` (`find`, `name_variants`), `crates/kq8-core/src/{sound,active,script}.rs`, `crates/kq8-audio/src/lib.rs`, `crates/kq8-viewer/src/sound_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Name lookup: as given, then `.AUD`, then `.WAV`; a name with a folder from the game folder | `sound.md` rules 13, 14 | Unit test `name_variants_follow_rule_13`; `aud-verify`: `raven.snd` → `daventry/sound/RAVEN.AUD`, `sprgouch.wav` → `GAME/SOUND/SPRGOUCH.AUD`, `poffup.aud` not found (matches); `chstopen`, `door4.aud`, `dirt1` found in `GAME/SOUND` (observed) |
| Search order: the world's loose folders, then `GAME`'s, `patch` first | `models-and-textures.md` "Where texture names are looked up" | Order inside `GAME` past `patch` is **our choice** (not traced; only `SOUND` folders hold `.aud`) |
| Six groups; sliders from `Options.cs` `setSfxVol` (0, 1, 2), `setSpeechVol` (3), `setMusicVol` (4, 5); 1 before | `sound.md` rules 50–52 | Unit test; `sound.log`: 0.14 / 0.34 / 0.14 from the shipped `MASK/Options.cs` |
| Loudness = own × group, at most 1 | Rule 21 | Unit tests `shipped_options_give_014_for_effects` (a door next to Connor 0.14), `loudness_is_capped_at_1` |
| Distance factor 1 − D ÷ (max − min), 1 inside min, 0 beyond max | Rule 29, "Do not reproduce" 1 (kept, the user's decision) | Unit test with the note's numbers (4,999 / 5,000 / 10,000 / 20,000; fountain 1,500 / 9,250 / 18,500) |
| Distance from Connor in 3D; position copied at the start | Rule 28 | `sound.log`: Condoor 2,200 from Connor at (270813, 225311, 13770) |
| Balance 0.82 × s ÷ √(s² + f²) in the camera's axes; centre straight above/below | Rule 31, "Do not reproduce" 4 | Unit test `balance_follows_the_camera` |
| Pan law: p > 0 left × (1 − p), p < 0 right × (1 + p) | Rule 23 | Unit tests `pan_law` (rules) and `mono_goes_to_both_sides_at_their_gains` (mixer) |
| Beyond max: one-shot stopped; loop paused, unless it started beyond max and was never heard | Rule 30 | Unit tests `a_one_shot_stops_beyond_max`, `a_loop_pauses_beyond_max_and_resumes` |
| Mix limit 8: more than 8 playing → the oldest non-looping sound of the lowest group stops; only loops → none | Rules 26, 27 | Unit tests (12 one-shots: 0, 1, 2 stop; groups; loops spared) |
| Finished one-shots leave the list on the 200 ms step | Rule 20 | Unit test `one_shots_finish_on_the_housekeeping_step` |
| Missing or broken file: skipped, logged once | "Do not reproduce" 2 | Unit test `a_missing_file_is_skipped_and_logged_once` |
| `playsound` cue → placed one-shot 5,000 / 25,000, group 0, at the object | Rules 35, 49 | Unit tests (Condoor `door4.aud`, `chstopen`); `sound.log` for Condoor and `smlchst3` |
| Item pickup → plain one-shot, group 0, every add to Connor's inventory | Rule 42, `items.md` rule 7 | Unit test; `sound.log`: `energy.aud` on the chest's second click |
| `KQSound::play` → plain one-shot, group 0 | Rule 40 | Unit test (`fishies.cs` `eat.aud`); `sound.log` for both fish |
| Rate conversion by a straight line between samples; sum clipped | **Our method** (rule 18: any rate is fine) | Unit tests `rate_conversion_keeps_the_length`, `the_sum_is_clipped` |
| Log line when a playing placed sound's loudness or balance moved by 0.05 | **Our method** (a log aid) | — |
| Listener and camera from the drawn camera; the camera is the listener when there is no Connor | **Our choice** for the viewer's free camera and worlds without Connor | — |

## Sound objects (M3 step 3h-3) — `crates/kq8-core/src/sound.rs` (`Audible`), `crates/kq8-viewer/src/{load,sound_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| `KQAudibleObject` keys `soundFileName`, `isSound3D`, `minDistance`, `maxDistance`, position; `deltaX/Y/Z` ignored | `sound.md` rules 43, 44, "Do not reproduce" 7 | `viewer.log`: the 5 objects Daventry's start-up scripts load, with the note's files and distances |
| Looping, group 0, placed when `isSound3D=1`, else plain | Rule 45 | Unit test `a_plain_sound_object_and_a_missing_file` |
| On while near or far (ball radius 100, off from `visibleDistance`), restarted from the beginning when it comes back; on at once when it enters the world | Rules 46, 47; `object-lighting.md` rules 34–36 | Unit test (off at 27,100, restart = a new start); viewer: fountain off at 27,169, mill at 27,196, water wheel at 27,191, windmill on at 27,046 |
| Started beyond max: plays silently until first heard; later beyond max: paused | Rule 30 | Viewer: water wheel started at 26,385 silently, heard from 19,245 in, paused at 20,049 |
| `deleteObject` stops it for good | Rule 48 | Unit test only (`bricks.cs` not reached in our runs) |
| Distance measured from Connor only, not the camera after `setActiveCamera`; `makeActive` not applied | **Not done yet** (`object-lighting.md` rule 42 says both apply to every object) | No Daventry script uses them on a sound object |

## Connor's animation cues and footsteps (M3 step 3h-4) — `crates/kq8-core/src/{connor,sound}.rs`, `crates/kq8-viewer/src/{connor_view,load,sound_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| A cue fires when its phase is in [lower, higher) of the update's phases | `doors.md` rule 9 | Unit test `cue_ranges_follow_rule_9_and_the_wrap`; `connor.log`: walk cues at 0.1 and 0.6 fire once per cycle (0.0605 → 0.1132, 0.5868 → 0.6395) |
| Forward loop wrap: [old, 1.0001) then [0, new) | `objects.md` "Loops" rule 4 | Unit test |
| Backward loop wrap: [0, old) then [new, 1.0001) | **Stand-in**, mirror of rule 4 (Q81 c) | Unit test |
| Cue action names ignore capitals; `PlaySound` = placed one-shot at the object (Connor too) | `sound.md` rules 34, 35 | `connor.log` |
| `FootStep` only for characters; plain one-shot, group 0 | Rules 36, 37 | `sound.log`: plain, loudness 0.140 |
| Flags → set: 0x1 dirt, 0x2 grass, 0x4 rock, 0x8/0x100/0x40 water, 0x20 weirdb, else none | Rule 37, "Do not reproduce" 5 (kept) | Unit test `footstep_sets_and_files` |
| Terrain: the square's `envInfo.kq` flags | Rule 37, `water.md` "Ground flags" | Walk near the fountain: dirt and water steps |
| Building: 2ⁿ for `footsoundType=n` (0–8), else 0x1 | Rule 39 | Bridge `bigbrig2` (2): 15 rock steps; `cav2` (4): silent (unit test) |
| Building without `footsoundType`: 0, so 0x1; on an object: the terrain under him | Rule 39, 39a (Q81 a, b answered; were stand-ins) | Pub upper floor: dirt steps "building without footsoundType: 0, so 0x1" |
| In the air: a remembered building kept 2 updates (gravity on; gravity off nothing counts), forgotten on the 3rd, then the terrain under him now (was: the last ground) | Rule 39a (Q81 d), fixed 2026-10-17 | Unit test `steps_remember_a_building_for_2_updates_in_the_air`; `connor.log`: off the pub floor, in the air from update 49, forgotten at 51 |
| File `<set>1/2/3` at 25 / 50 / 25 % | Rule 38 | Unit test `the_dice_give_25_50_25` (40,000 rolls within 5 %) |
| The random number source (xorshift, fixed start) | **Our method** | — |

## World music (M3 step 3h-5) — `crates/kq8-core/src/{music,sound,active,script}.rs`, `crates/kq8-viewer/src/{load,sound_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| Fades: own level × loudness, 0.75 per second in the 200 ms steps, stop exactly at the target, wait while paused, at 0 the sound stops | `sound.md` rules 21, 33, 57g | Unit test `fades_step_with_housekeeping_wait_while_paused_and_stop_at_0`; `sound.log`: steps 0.15 apart, done 1.2–1.4 s after the start |
| The list from `music.kq` (`nMusic`, `music<n>`), `music0` at the bottom; names ignore capitals | Rules 57, 57a | `sound.log`: "music object Music with [forest.aud]" |
| Each tune a looping plain sound, group 5 | Rule 57a | `sound.log`: `forest.aud` group 5, loudness 0.140 |
| World start: the top at full loudness, no fade | Rule 57b | Unit test `the_world_tune_starts_at_full_loudness_and_loops`; `sound.log` frame 1 |
| push: nothing if on the list; old top fades out and stays; new one starts at 0 and fades in; empty list → full at once | Rule 57c | Unit tests `push_*`, `pop_of_an_unknown_or_the_only_tune`; wizard trap run |
| pop: not on list nothing; top fades out and leaves, the one below fades in from its beginning (still fading out: from where it is); only one → silence; lower one leaves at once | Rule 57d | Unit tests `pop_*`; wizard trap run: `forest.aud` restarted (a new sound) |
| play: top fades out and leaves, new one fades in, no name check | Rule 57e | Unit test `play_replaces_the_top_without_a_name_check` (no script uses it) |
| Script push / pop 0.2 s of game time later through the event queue, in order; play at once; usage line; create / delete obsolete | Rule 57f | Unit test `music_push_and_pop_act_02_s_later_in_order_play_at_once`; `traps.log` "due at" |
| No music object: requests ignored (we write a `sound.log` line; the original says nothing) | Rule 57f; the log line is **our choice** | — |
| The queue target name `<KQMusic>` | **Our method** (no object has that name) | — |

## A jump that hits something steep (Q82, corrected rule) — `crates/kq8-core/src/connor_update.rs`

Replaced the Q82 stand-in on 2026-10-17 (the stand-in: "every non-back jump hit crashes", from the user's check in the original).

| What | Source | Check |
| --- | --- | --- |
| During `jump` or `runJump`, a steep hit (normal z < 0.8) on the **down pass** crashes | `connor-physics.md` "Jumping" rule 5, first bullet | Unit tests `landing_on_a_steep_hill_during_a_jump_crashes`, `only_head_on_or_down_pass_jump_hits_crash` |
| An **up, across or slide** hit crashes only when the head-on test said head-on; during a jump every steep surface is head-on and `jump` and `runJump` are both in the crash list (`jump_hit_outcome`) | Rule 5, second bullet, and "Walls" (corrected 2026-10-17, Q82) | Unit tests `a_standing_jump_into_a_wall_crashes_head_on`, `a_running_jump_into_a_wall_crashes`, `only_head_on_or_down_pass_jump_hits_crash`; `connor.log`: "jump hit tree.06 (steep, across pass): the jump ends, crash (head-on; …)" at Daventry's start |
| A steep up/across hit that is **not** head-on (a `KQMonster`, the Connor flag; neither exists in our code yet): the jump ends (gravity on, steps 325), no crash, the animation plays on as after a `backJump` hit | Jump ends: rule 5. What follows: **STAND-IN**, asked as Q84 (a) | Unit test `only_head_on_or_down_pass_jump_hits_crash` (not reachable in a run) |
| A fall starting during the crash switches to the hold animation at once (the stumble is cut short) | **User's check in the original** (2026-10-17, Q82 b); rule 5 says it is not traced | `connor.log`: crash updates 24–27, `jumphold` at 28, `jumpend` at 31 |
| The crash ignores the keys (no turning while it plays), as for the run crash | Unchanged; rule 5 now says "turning stays allowed, and a turn key still held is passed on again", unclear to B: asked as Q84 (b) | — |

## Scripts start movements: the waiting movement (M3 step 3i-1) — `crates/kq8-core/src/{named_anim,door,active}.rs`, `crates/kq8-viewer/src/{load,door_view}.rs`

All from `docs/behaviour/anim-scripts.md` (session A, traced unless the note says "data"). Doors, chests and scripted objects now share one player (`NamedAnimPlayer`); the door's four states stay on top (`doors.md`).

| What | Source | Check |
| --- | --- | --- |
| Current movement name (as asked), one waiting slot, the running animation's "can be cut short" (6th field of `loop=`) | Rules 1, 4; `connor-movement.md` "two flags" | Unit tests in `named_anim.rs` |
| No `currAnimName`: movement 0 shows, the name stays empty; on the first update the waiting movement (`nextAnimName`) starts | Rule 3 | Unit test `no_curr_anim_name_lets_next_anim_name_start_on_the_first_update`; `doors.log` update 0: `smlchst1`, baskets, `jar1` start `stop`; `smlchst2`'s `closed` names no movement, movement 0 stays (as the note's data count says) |
| `KQAnimPlayEvent`: waits; starts at once if the running one can be cut short or has ended (≥ 0.9999 forwards, ≤ 0 backwards), else at its end | Rule 4 | Unit tests `a_cut_short_movement_is_replaced_at_once`, `a_play_once_that_*` |
| Transitions: none built for objects | Rule 4.2 (no placed object has a `transitionN=` line, data) | — |
| Starting: ignore capitals, first match; unknown name → movement 0 under the asked name; the movement's own `loop=` start and speed; afresh even if playing | Rule 5 | Unit tests `an_unknown_name_plays_movement_0_under_that_name`, `the_playing_movement_restarts_from_its_start`; active test with `fly` |
| A stopped movement that can't be cut short: a request waits forever (logged "waiting forever (rule 6)") | Rule 6; logging from "Do not reproduce" | Unit test `a_stopped_pose_that_cant_be_cut_short_waits_forever` |
| At the end of a play-once movement the waiting one starts first, then `EndLoop` / `Cue 300` name the ended one; a repeating one never ends | Rules 8, 10 | Unit tests `a_play_once_that_cant_be_cut_short_finishes_then_the_waiting_one_starts`, `a_repeating_movement_that_cant_be_cut_short_never_lets_a_request_start` |
| "Has ended" at 0.9999 also ends a movement's play (phase then set to 1, as before) | Rule "Has ended" (traced in `objects.md`); setting 1.0 is our earlier choice (3a-2) | Door and chest unit tests unchanged |
| Chests answer `KQAnimPlayEvent` too | Rule 8 ("doors, chests and the rest") | — (no Daventry script sends one) |
| A door whose `open`/`close` ends while a movement waits: the waiting one starts, then the door rule shows `opened`/`closed` over it, logged | **Our choice**: no note covers it, no data does it | — |
| The `.kq` keys `currAnimName`, `nextAnimName` read for doors, chests and scripted objects | Observed (data) and rule 2 | `doors.log` above |

## Objects that play once and report (M3 step 3i-2) — `crates/kq8-core/src/{named_anim,active}.rs`

| What | Source | Check |
| --- | --- | --- |
| A `KQAnimOnce`'s movement end: script gets `Die`, then the listener `Cue 300 0 <movement>`, then deletion if `setDeleteWhenDone` asked; no `EndLoop`, no waiting movement | `anim-scripts.md` rule 11 | Unit tests `a_kq_anim_once_reports_die_tells_its_listener_and_is_deleted`, `a_kq_anim_once_keeps_its_last_frame_and_never_starts_the_waiting_one` |
| `setDeleteWhenDone <name> 1|0` only on `KQAnimOnce`; else "could not find object" | Rule 13 | Same unit test |
| Its usage line's wording, and `$Console::Return` 1 for the failures | **Our choice** (the note says only "prints its usage line"; 1 = failure as `doors.md` Q45 f) | — |

## Pausing an animation (M3 step 3i-3) — `crates/kq8-core/src/{named_anim,active,script}.rs`

| What | Source | Check |
| --- | --- | --- |
| `pauseAnim <name>` toggles; while paused no phase change, no cues, no end, no waiting start; logs `<name>, <phase>`; unknown name → `pauseAnim:  could not find anim <name>` | `anim-scripts.md` rule 14 | Unit tests `a_paused_animation_doesnt_move_fire_cues_or_end`, `pause_anim_toggles_and_set_anim_pos_sets_the_phase` |
| `setAnimPos <name> <phase>` sets the phase, same log line | Rule 15 | Same unit test |
| A movement requested while paused starts (its phase is set) but doesn't move until resumed | **Our reading** of rules 4 and 14 together (the note doesn't say) | Same unit test |
| Return values (0 done, 1 failed) and the usage lines | **Our choice** | — |

## Scripts reach every animated object (M3 step 3i-4) — `crates/kq8-core/src/{active,door,named_anim}.rs`, `crates/kq8-viewer/src/{anim_view,load,terrain_view,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| Every animated object (not only doors, chests, scripted clickables) keeps current/waiting movement and follows rules 1–10 | `anim-scripts.md` rules 1–10 ("Every animated object") | Unit test `scripts_reach_the_other_animated_objects`; star globe run (`doors.log`) |
| Still objects with more than one movement are played too (a script may switch them); still single-movement objects stay plain models | **Our choice** (nothing could change those) | `viewer.log`: 75 objects, 8 with several movements |
| Repeating frame: round(phase × frames), wrapping | `objects.md` rule 2 (as 3c) | Unit test `cues_at_a_wrap_fire_once_including_one_at_exactly_1`; `anim.log` start lines unchanged |
| Cue ranges at a wrap: [old, 1.0001) and [0, new) forwards; [0, old) and [new, 1.0001) backwards | `objects.md` "Loops" rule 4 | Same unit test |
| `flags` bit 8: random phase, speed × 0.9–1.1, same random numbers in the same order as 3c | `objects.md` rule 4 | `anim.log` start lines identical to before |
| A movement started later runs at its own speed (no random factor) | `anim-scripts.md` rule 5 ("its own `loop=` line") | Unit test `a_random_start_survives_the_first_update_restart_q85_stand_in` |
| Rule 3's first-update start keeps the random start of the movement shown at load, and uses up that update | **Stand-in**, asked as Q85 (the two rules meet on `davtorch`, the fish, the dead birds) | `anim.log`: `Davtorch.01` phase 0.2819 at update 1 |
| A movement with no length: a loop stays, a play-once one has ended | Phase rule (it can't move); play-once as before 3i | Same unit test |
| Unused objects (H key only) keep a player of their own, outside the world's scripts | **Our choice** (they are not part of the world) | — |

## The script reader: single quotes, `;`, `( … )`, 40 words (M3 step 3i-5) — `crates/kq8-core/src/script.rs`

| What | Source | Check |
| --- | --- | --- |
| Single quotes dropped, `$` replacement off until the next one, words still split at spaces | `anim-scripts.md` rule 17 | Unit test `single_quotes_semicolons_brackets_and_the_word_limit` |
| `;` ends a command; `( … )` one word without brackets; more than 40 words → `Console: Too many arguments`, the command fails | Rule 18 | Same unit test |
| The count is taken after variables are replaced; `;` inside single quotes ends a command; an open quote ends with the line | **Our choice** (not in the note; no data line has more than 29 words or a `;` in single quotes) | Data survey of all `.cs` files |

## Stats (M3 step 3i-6) — `crates/kq8-core/src/{stats,active}.rs`, `crates/kq8-viewer/src/{load,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| Five numbers in `stats=` order (health first, fourth armour, fifth experience), added; fourth capped at 100; health below 0 → `Die` | `anim-scripts.md` rule 19, `clicking.md` rule 24 | Unit tests `numbers_are_added_and_the_fourth_capped`, `stats_events_add_and_an_object_below_0_gets_die` |
| Connor: health cap 10 × level; experience over 10 × 2^(level − 1) (150,000 from 15) → level + 1, health = new max, third + 5, fourth + 1 (cap 100), second + 2 | Rule 19 | Unit test `connor_levels`; star globe run |
| Connor starts with `stats=10 10 0 0 0`, `level=1` | Observed (`GAME/Kq/Connor.kq`) | `viewer.log` |
| Doors without `stats=`: health 10,000 | `doors.md` rule 18 | — |
| "Passes" = more than; one level check per event; cap at the old level first; other objects start at 0; `Die` without `$3` | **Stand-ins**, asked as Q86 | Unit test `connor_levels` |

## Connor's Special state (M3 step 3i-7) — `crates/kq8-core/src/{connor,active,script}.rs`, `crates/kq8-viewer/src/{load,connor_view,door_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| `setState Connor special <list>`: items left to right; `AnimList=`, `AnimListOn`/`Off`, `Inv <item>`, `KillConnor`, any other word a movement played to its end; then standing | `anim-scripts.md` rule 16 | Unit tests `a_special_list_is_worked_out_left_to_right`, `special_plays_each_movement_to_its_end_then_stands`; `connor.log` (`--special`) |
| Movement names compared ignoring capitals | Rule 16 | Same |
| His play-once movement ends go to his `getEndLoop` listener as `Cue 300 0 <movement>` | Rule 16, `clicking.md` rule 21 | Unit test `set_state_connor_special_is_handed_over_and_his_ends_are_relayed` |
| Which movements to load: those the world's scripts name in `setState Connor special` lines | **Our method** (data survey at load) | `viewer.log`: 87 movements, no problems |
| An item naming a movement that isn't in the list: skipped, logged | **Stand-in** (not in the note) | `connor.log` |
| `Inv <item>`: logged only | Rule 16 says "not traced further" | — |
| `KillConnor`: our Dead mode (fall-death pose) | **Stand-in** (his Die state is M5) | — |
| The Special lists' transitions are not used | Rule 16 says nothing of them; objects have none (rule 4.2) | `viewer.log` detail lines |
| `--special ITEMS,N` | **Our test option** | — |

## Pushing: the check and the Push state (M3 step 3j-1) — `crates/kq8-core/src/{collide,connor_mover,connor,connor_update,active}.rs`, `crates/kq8-viewer/src/load.rs`

| What | Source | Check |
| --- | --- | --- |
| A touch with type bit 0x100 goes through the push check, else a wall | `pushing.md` rule 1, `connor-collision.md` rule 4 | Unit tests `walking_into_a_pushable_*`, `the_push_check_fails_*` |
| The six conditions: Connor moving, on the ground or pushing, empty hands (hand mode 4), across pass, facing cos ≥ 0.5 (into the surface / towards the object), `walk`/`run`/`push` | Rule 2 | Same; `connor.log` (hay bale) |
| Facing off while pushing → Move; wrong movement → standing state | Rules 2.5, 2.6 | Unit test `the_push_check_fails_*` |
| Push state: `push` (from `Conner.anm`), counter: a touch sets 0; < 4 with a key +1; < 4 no key → Wait; ≥ 4 → Move or Wait | Rules 6, 7 | Unit test `the_push_state_asks_for_push_and_ends_after_4_updates_without_a_touch`; `connor.log` |
| `push`'s stop transitions `pushright` / `pushleft` play when he lets go | Observed (`Conner.anm`), `connor-movement.md` rules 2–6 | `connor.log`: `pushright` 132–146 |
| "On the ground" = began the update on the ground, mover not jumping or falling | **Our reading** of rule 2.2 (open question 3 of the note) | — |
| The standing state of rule 2.6 changes the state only, not the animation | **Our reading** (the note says "puts him in the standing state") | `connor.log` |
| Turning in the Push state as when walking | **Stand-in**, Q87 d | — |
| "The object's centre" = its position | **Stand-in**, Q87 e | — |
| An accepted touch leaves him at the pass start until the object's part exists | Rule 4.3 (the "didn't move" case) | Unit test `walking_into_a_pushable_passes_the_push_check_and_stays_at_the_pass_start` |
| Colliders carry type, mask, shape, keepOnGround; scripted objects' values refreshed every update | **Our method** | — |

## Pushing physical objects (M3 step 3j-2) — `crates/kq8-core/src/connor_mover.rs`, `crates/kq8-viewer/src/{connor_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| The object is moved by Connor's horizontal step of the pass, if more than 0.000001 on some axis | `pushing.md` rule 4, 4.1 | Unit test `a_ball_stack_pushable_slides_ahead_and_stops_at_a_wall`; `connor.log` (hay bale) |
| Its own sweep ignores Connor; walls and objects its mask makes obstacles stop it | Rule 4.1, `connor-collision.md` rules 2, 3, 12 | Same |
| Any change of height → put back, counts as not moved | Rule 4.2 | Unit test `a_box_shaped_pushable_or_one_on_a_slope_stays` |
| Connor at the pass start + 0.95 × the object's movement | Rule 4.3 | Unit test; `connor.log`: Connor ends behind the bale |
| The sweep uses the object's ball stack (`collisionShape=1`, built from its box like Connor's) | `connor-collision.md` "Ball stack against ball stack" (who is a ball stack); the sweep against buildings and boxes is the one Connor uses | — |
| Box-shaped pushables not moved | **Stand-in**, Q87 a | `connor.log` (barrel) |
| Stops at the first contact (no slide) | **Stand-in**, Q87 b | — |
| keepOnGround after the move: exactly on the floor grid | **Stand-in**, Q87 c | Unit test (slope) |
| Buildings count as obstacles when the object's mask has 0x8 (their types 0x28 / 0x88) | `camera.md` "Obstacles" (building type), `connor-collision.md` rule 3 | — |
| The viewer moves its colliders and drawn parts after Connor's update | **Our method** | `connor.log` "pushed haybail.01 … moved" |

## Pushing scripted objects (M3 step 3j-3) — `crates/kq8-core/src/{connor_mover,active}.rs`, `crates/kq8-viewer/src/{connor_view,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| Scripted pushable (0x100 + 0x10000): the touch is not a hit; the script runs with `$1` its name, `$2` `Push`, no `$3`; again at every later pushing touch | `pushing.md` rule 3 | Unit tests `walking_into_a_pushable_*`, `a_scripted_pushable_gets_push_and_takes_control_or_becomes_a_wall`; `doors.log` (bricks) |
| He goes on through it for the rest of that move | Rule 3 ("does not stop or slide him"), edge case "walks into it" | Same unit test |
| The script runs right after his move in the same update, before the scripts' requests are applied | **Our order** (rule 3 says mid-move; what follows a mid-move `move Connor` is not traced, `connor-collision.md`) | `doors.log` update 13 |
| Still a pushable after its `Push` call → a wall for him from then on, logged | "Do not reproduce" of `pushing.md` | Unit test |
| Two contacts at exactly the same distance: the pushable's first, the other found after passing through it | **Stand-in**, Q88 (bricks and table) | `connor.log` update 13 |

## `move` while playing (M3 step 3k-1) — `crates/kq8-core/src/{active,script}.rs`, `crates/kq8-data/src/world_script.rs`, `crates/kq8-viewer/src/{connor_view,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| `move <who> <flags> x y z [rx ry rz]`: up to six numbers, missing 0; flags 0 absolute, 1 position relative, 2 turn relative, 3 both; relative position along the world axes | `objects.md` "(d)", Q21 | Unit test `move_sets_or_adds_along_the_world_axes`; `doors.log` (bricks) |
| Set directly, no collision passes | `objects.md` "(d)" (traced) | `connor.log`: Connor put at the spot, then falls to the floor |
| Relative turns added only about Z | Q21 answer (all 9 cases in the game) | Same unit test |
| Connor turns only about Z (rx, ry ignored, logged) | **Our choice** (his heading is his only turn) | — |
| The viewer moves Connor, the camera (3k-3) and plain placed objects; core moves its own objects | **Our method** | — |

## `setHidden` on plain placed objects (M3 step 3k-2) — `crates/kq8-core/src/active.rs`, `crates/kq8-viewer/src/draw_distance.rs`

| What | Source | Check |
| --- | --- | --- |
| A hidden object keeps only `groupsAlways`: not drawn, collider out, until `setHidden <name> 0` | `objects.md` "Hidden objects" rules 1–3, `object-lighting.md` rule 41 | Unit test `set_hidden_on_a_plain_placed_object_overrides_its_file`; `viewer.log` (Table) |
| A script's value wins over the file's `hidden=` | `objects.md` rule 1 ("scripts switch it") | Same unit test |
| Done through the draw-distance test (it already turns "hidden" into off) | **Our method** | — |

## Camera and controls in a cutscene (M3 step 3k-3) — `crates/kq8-core/src/{active,script}.rs`, `crates/kq8-viewer/src/{game_camera,connor_view,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| `KQCamera::follow Connor|none`; following none, the camera stays where it is | `camera.md` rule 1 and "Following `none`" | `traps.log` (bricks) |
| `move KQCamera`: position set or added (flags as `move`) | `objects.md` "(d)" | `traps.log` |
| Its view: keeps looking at its last aim point; the angles are not used | **Stand-in**, Q89 c | — |
| `handsOff 1`: movement keys and Jump count as let go, clicks ignored; `0` restores | **Stand-in**, Q89 a | `connor.log`: stands from 64 to 124 with forward held |
| `setThirdPerson`: logged only | **Stand-in**, Q89 b | — |

## `loadKQ` while playing (M3 step 3k-4) — `crates/kq8-core/src/{active,sound,script}.rs`, `crates/kq8-viewer/src/{load,anim_view,sound_view,door_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| An object loaded while playing is set up as at the world's start; no script call; a sound object starts at once | **Stand-in**, Q90 a–c (`objects.md` describes `loadKQ` at the start only) | `doors.log`, `sound.log` (bricks) |
| Prepared at the start: files a script names in `loadKQ <file>` that no start-up script loads, with a place of their own | **Our method** (they were already loaded as H-key copies, M1) | `viewer.log`: 17 dormant copies |
| A deleted sound object stops at once; the new one starts | `sound.md` rule 48; stand-in above | `sound.log` frame 665 |
| A copy that comes in has no collider and isn't clickable | **Not done** (the H-key copies have none) | — |
| Fix 2026-10-22: a prepared **character** gets its click box at the start, switched off (no `Init` at the start, not clickable); the `loadKQ` switches it on, then its `Init` runs as before | `objects.md` "Where a world's objects come from" rule 6 (comes in as at the start: a character placed at the start is clickable, `doors.md` rule 10); the same as prepared traps (`traps.md` rules 1, 2, 13) | Unit test `a_character_loaded_while_playing_gets_init_then_and_can_be_clicked`; viewer `--begin`: click `Ravens` → `ravensrh.cs ConnorAction`, "Begone, raven! Leave her be!", `fromsarah`, deleted |

## The answers to Q84–Q93 (M3 step 3l)

### 3l-1, no pop-up at the bricks — `crates/kq8-core/src/connor_mover.rs`, `crates/kq8-viewer/src/{connor_view,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| A script's `move` of Connor: no collision tests, then one gravity step (if gravity is on) and the terrain raise | `objects.md` "(d)", refined 2026-10-17 (Q92) | Unit test `a_scripted_move_from_outside_his_update_drops_one_gravity_step`; `connor.log` update 124 |
| A `move Connor` from a `Push` call is followed by the rest of his update (a gravity step, the down pass with its 325 step-down, the ground check), so he lands in the same update | `pushing.md` rule 11 (Q92) | Unit test `a_scripted_move_inside_his_update_lands_him_at_once`; `connor.log` updates 13–64 at z 14837 |
| The gravity step uses the down pass's fall timer, straight down | **Our method** (the note gives "about 44" for the original's real time; ours is 36 at 15 updates per second) | — |
| The requests a `Push` call makes count as "inside his update"; the objects he pushed stay passed through | **Our method** (the script still runs after his move, our order) | — |

### 3l-2, the watermill cutscene — `crates/kq8-core/src/{connor_auto,connor,connor_update,active,script}.rs`, `crates/kq8-viewer/src/{connor_view,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| `walkTo` / `runTo x y <who> <c> <d>`: handsOff on, Wait, turn to face, then Move straight ahead (walk or run) until the flat distance from the walk's start is gone; Wait, `KQCueEvent c d` to `<who>`; early at a head-on hit or `walkExit`; never switches handsOff off | `anim-scripts.md` rule 24 | Unit tests in `connor_auto.rs`, `walk_to_point_and_cue_and_walk_exit_become_requests`; `traps.log` (watermill) |
| `pointAndCue x y z <who> <c> <d>`: turn to face (x, y), z ignored; the cue when the turn ends, at once if already facing | `anim-scripts.md` rule 25 | Same; `connor.log` update 70 |
| The automatic turn: 3.5 rad/s, the shorter way, exactly onto the heading once the rest is under one update's turn; ~~a player turn cancels it~~ (corrected in 4a-7: only a turn key press) | `connor-movement.md` "Turning" rule 2 | Unit test `the_automatic_turn_is_3_5_per_second_the_short_way_and_lands_exactly` |
| No time-out if a cue never comes | `anim-scripts.md` rule 26 | — |
| A head-on hit during a scripted walk: he stands at once, no crash, no wall block; a wall block already on wins; `stop` plays during the turn; no object named, no cue | **Stand-ins**, Q94 | — |
| The walk's end is checked after each move, so it ends up to one step past the distance | **Our method** | `connor.log`: 2783 of 2736 |

### 3l-3, cutscene control — `crates/kq8-core/src/{camera,active,script}.rs`, `crates/kq8-viewer/src/{game_camera,connor_view,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| `handsOff 1`: movement, turn and run keys, Jump, the mouse look and clicks ignored; presses swallowed, releases counted; fixed camera mode back to normal; `N` logged only | `anim-scripts.md` rule 21 | `connor.log`, `traps.log` (bricks) |
| `handsOff 0`: everything back at once; every movement key still held is sent again as a press | `anim-scripts.md` rule 22 | `traps.log` update 124: "held keys sent again: Forward" |
| `move KQCamera`: flags bit 1 adds the position, bit 2 the angles; rx tilt (negative down), rz heading (0 along +Y, anticlockwise), ry roll; view (−sin rz cos rx, cos rz cos rx, sin rx); taken at once | `camera.md` "Shake and cutscene cameras" | Unit test `cutscene_angles_give_the_view_and_back`; `camera.log` |
| Roll not drawn, logged | **Our choice** (0 in Daventry's two cutscenes) | — |
| `follow Connor` from where the camera is: range at once, more than 0.4 rad off → 1/3 per update, snap under 0.063 (one snap on both sides), tilt at once; also when it already followed | `camera.md` "Shake and cutscene cameras", "Do not reproduce" | Unit test `follow_turns_back_by_a_third_per_update_then_snaps`; `traps.log` |
| `follow Connor` while already following runs the same setup | **Our reading** (the note describes the command, not a state change) | `traps.log` (watermill cue 6) |
| `setThirdPerson`: a paired toggle; in third person it flips the first-person key's lock (logged; we have no first person) | `camera.md` "First person" rule 7 | Unit test `set_third_person_flips_the_key_lock_and_hands_off_logs_its_frames` |

### 3l-4, pushing as traced — `crates/kq8-core/src/connor_mover.rs`, `crates/kq8-viewer/src/{terrain_view,connor_view,door_view}.rs`

This replaces the Q87 a–c and Q88 stand-ins of 3j-2 and 3j-3 above.

| What | Source | Check |
| --- | --- | --- |
| Every object touched in a sweep is told at once (the push check runs there); tested in the order they entered the world; a contact replaces the kept one when at most as far (tie: the later-tested); an accepted push is not recorded | `connor-physics.md` "Against a building" step 7 (Q88) | Unit test `every_object_touched_is_told_and_a_tie_keeps_the_later_tested`; `connor.log` (bricks) |
| Buildings tested before the objects | **Our reading** (the note orders the objects; buildings are loaded before them) | — |
| The load order: by where each collider's name comes in the start-up scripts' load order | **Our method** | `viewer.log`: 295 colliders, already in that order |
| Box-shaped pushables (no `collisionShape`) are never stopped | `pushing.md` rule 4.1 (Q87 a) | Unit test `a_box_shaped_pushable_goes_through_walls_and_the_ground_only_raises`; `connor-3l4-barrel.log` |
| Ball-stack pushables slide: the hit and 3 retries, normal z in (0, 0.7) = vertical wall, no head-on stop, contact pulled back 25 | `pushing.md` rule 4.1 (Q87 b) | Unit tests `a_ball_stack_pushable_pushed_at_an_angle_slides_along_the_wall`, `a_ball_stack_pushable_slides_ahead_and_stops_at_a_wall`; `connor-3l4-bale2.log` |
| Ground after the move: raise only, with `keepOnGround`; any height change puts it back | `pushing.md` rules 4.1–4.2 (Q87 c) | Same unit test (uphill put back, downhill floats); barrel log from update 81 |
| Facing test: from his position at the start of the update to the object's position | `pushing.md` rule 2.5 (Q87 e) | — |
| Two physical pushables in one sweep: he follows the last one's movement | **Our choice** (not described) | — |

### 3l-5, turning during `crash`; glancing jump hits — `crates/kq8-core/src/{connor,connor_update,connor_mover}.rs`

| What | Source | Check |
| --- | --- | --- |
| Turn keys turn him at 1.75 rad/s while `crash` plays (run and jump crashes); forward, back and run wait | `connor-physics.md` "Jumping" rule 5, "Walls" (Q84 b; user's check 2026-10-18) | Unit test `running_head_on_crashes_7_updates_441_5_back_then_runs`; `connor.log` updates 8–14 |
| No turning from a jump's start to its landing's end | `connor-physics.md` "Jumping" rule 5 (the user's checks, Q41) | Existing jump tests |
| A down-pass crash straight out of a jump turns too | The note's stand-in (not settled) | — |
| A steep hit during a jump that is not a crash (all `backJump` hits, not head-on): glancing, gravity stays off, steps 0, the jump plays on, then Fall | `connor-physics.md` "Jumping" rule 5 (Q84 a) | Unit test `a_back_flip_slides_along_the_wall_and_plays_on` |

### 3l-6, animated objects' first update, levels, `Die`'s `$3` — `crates/kq8-core/src/{named_anim,stats,active}.rs`, `crates/kq8-viewer/src/anim_view.rs`

This replaces the Q85 and Q86 stand-ins of 3i-4 and 3i-6.

| What | Source | Check |
| --- | --- | --- |
| First update: no current name and a waiting movement → that one starts plainly (no random part); else the current movement with `animPos` and the bit 8 random part; the phase moves on in that same update | `anim-scripts.md` rule 3 (Q85) | Unit tests `without_curr_anim_name_the_random_start_is_lost_q85`, `no_curr_anim_name_lets_next_anim_name_start_on_the_first_update`; `doors.log` update 0 |
| Bit 8: start frame round(random × (frames − 1)), phase = frame ÷ frames; random speed 0.9–1.1 replaces the `loop=` speed | `anim-scripts.md` rule 3, `objects.md` rule 4 | Unit test `with_curr_anim_name_the_random_frame_and_speed_replace_the_loop_line` |
| Levels: strictly more than the threshold, once per event, cap at the old level then a full heal, second +5, third +2, fourth +1 | `anim-scripts.md` rule 19 (Q86 a–c, correction) | Unit test `connor_levels`; `doors.log` update 13 |
| "Below 0" checked right after the adding; the event's fourth number cut to 100 when read | `anim-scripts.md` rule 19 | Unit test `stats_events_add_and_an_object_below_0_gets_die` |
| `Die` always has `$3`: the last weapon-hit kind, `Normal` if never hit (always `Normal` until M5) | `anim-scripts.md` rule 20 (Q86 e) | Same test and `a_kq_anim_once_reports_die_tells_its_listener_and_is_deleted` |

### 3l-7, `loadKQ` while playing — `crates/kq8-core/src/{script,active}.rs`, `crates/kq8-viewer/src/load.rs`

This replaces the Q90 b and d stand-ins of 3k-4.

| What | Source | Check |
| --- | --- | --- |
| The new object's script gets `$1=<name> $2=Init` at once, during the `loadKQ` line, before the next line | `objects.md` "Where a world's objects come from" rule 6 (Q90 b) | Unit test `load_kq_while_playing_calls_init_at_once_and_refuses_a_name_in_use`; `doors.log` update 63 |
| A name a live object still has, or deleted during the same event: error logged, load refused (the original's fatal error) | `objects.md` rule 6, `clicking.md` rule 22 (Q90 d; "for us: log and refuse") | Same unit test |
| A world command asks the script runner to run a script at once (`take_calls`); the runner reports the end of an event's run (`event_done`) | **Our method** | — |
| Fix 2026-10-24: an event for an object a `loadKQ` brought in, sent before the viewer hands it over (after the update), is held and handled at the hand-over, before drawing (`Active::loaded`), so `sendEvent 0` on the next line reaches it (the Wizard's `magicmap` `Map1`) | `objects.md` "Where a world's objects come from" rule 6 (added to the world at once); `anim-scripts.md` rule 8 (delay 0 never waits for the next update) | Unit test `an_event_to_an_object_still_coming_in_waits_for_it`; `doors.log`: `Map1` held at update 217, played from the hand-over at 218 |

### `KQConner::getDoMode` — `crates/kq8-core/src/active.rs`

| What | Source | Check |
| --- | --- | --- |
| Answers the hand mode: sword 0, bow 1, empty hands 4 | `pushing.md` "Words used here" (the game's own help text) | Unit test `get_do_mode_answers_empty_hands`; `traps.log` (windmill) |
| Always 4: no weapons yet, `setDoMode` a stub | **Our state** (until M5) | — |

## The answers to Q94–Q96 and the camera flights (M3 step 3m)

### 3m-1, the windmill tumble — `crates/kq8-core/src/{active,connor,connor_update,script}.rs`, `crates/kq8-viewer/src/{connor_view,door_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| `KQConner::JumpDir`: 1 in the Jump state with `backJump`, 0 with `jump`/`runJump`, 0 and the usage text outside a jump | `anim-scripts.md` rule 29 (Q96 a) | Unit test `jump_dir_and_end_jump_answer_from_connors_jump`; `traps-3m1-windmill.log` update 38 |
| `KQConner::endJump`: in the Jump state gravity on, steps 325, cut short, the jump finished, no state change, result 1; else the usage text and 0 | `anim-scripts.md` rule 28 (Q96 b) | Same unit test; `end_jump_then_special_replaces_the_jump_with_no_fall`; `connor-3m1-windmill.log` |
| An unfinished jump refuses another state; with `handsOff` on a state request finishes it first; once Special replaces Jump no Fall follows | `anim-scripts.md` rule 28, `connor-physics.md` "Jumping" rule 4 | Unit tests `with_hands_off_a_state_request_finishes_the_jump_by_itself`, `end_jump_then_special_replaces_the_jump_with_no_fall` |
| A refused state request is tried again at each later update | **Stand-in** (when the original tries again was not traced) | — |
| `setState … special` and a scripted walk (it puts him in Wait) count as state requests | **Our reading** of "any state request" | — |
| A trap's script runs inside Connor's move: its `move Connor` gets the rest of his update | `anim-scripts.md` rule 30 | `connor-3m1-windmill.log` update 38; watermill update 1 |
| The scripts see Connor's jump as it was at the end of his last update (our scripts run after his move) | **Our method** | — |

### 3m-2, a push and a wall in the same sweep — `crates/kq8-core/src/connor_mover.rs`

This replaces the "end the pass at the 0.95 point" of 3l-4.

| What | Source | Check |
| --- | --- | --- |
| The push happens at the pushed object's turn in the list; his position and sweep become P = the sweep's start + 0.95 × its movement; contacts kept from earlier in the list stay | `pushing.md` rule 4.3 (Q95 a) | Unit test `a_wall_before_the_barrel_in_the_list_stops_him_but_not_the_barrel`; `connor-3m2-barrel-pub.log` update 54 |
| The kept hit afterwards: wanted end P, movement 0, never head-on; P projected onto the plane through the contact; put at the contact and slid there | `pushing.md` rule 4.3 (Q95 b), `connor-physics.md` "Against a building" step 7 | Same |
| A push inside a slide moves the object by the slide's step; P from the slide's start | `pushing.md` rule 4.3 | — |
| Objects later in the list after a push are not tested | **Stand-in** (rule 4.3: "not traced"; buildings test nothing under 0.1) | Unit test `after_a_push_later_objects_in_the_list_are_not_tested` |
| Only updates whose sweep reaches the barrel are pushing touches; 4 without one end Push | `pushing.md` rules 4.3 (Q95 c), 7 | `connor-3m2-barrel-pub.log` updates 55–64 |

### 3m-3, scripted walks — `crates/kq8-core/src/{connor_auto,connor,connor_update,active}.rs`

This replaces the Q94 stand-ins of 3l-2.

| What | Source | Check |
| --- | --- | --- |
| During the automatic turn: update 1 `Stop`; update 2 (no wall block) `Left`/`Right` (Move); from update 3 `Walk`/`Run` with the start and distance recorded; at the turn's end Move afresh and the distance measured again | `anim-scripts.md` rule 24 (Q94 c, corrected) | Unit tests `during_the_turn_stop_then_left_then_walk_and_the_distance_is_measured_again`, `a_scripted_walk_plays_stop_left_walk_and_starts_even_with_a_wall_block`; `connor-3m3-watermill.log` updates 18–20 |
| The automatic turn's direction counts as a turn key for the movement; the walker's own key turning stays off meanwhile | **Our method** for "reacts to it as to a key" (the turn rate stays the automatic 3.5 rad/s) | Same |
| `pointAndCue`: `Left`/`Right` until the turn ends, then Wait | `anim-scripts.md` rule 25 | `connor-3m3-watermill.log` updates 58–64 |
| A wall block on at the start: `Stop` through the turn, the walk starts anyway, the block stays | `anim-scripts.md` rule 24 (Q94 b) | Unit test above |
| Head-on during the walk (also while it walks the rest of the turn): put at the contact, cue, Wait (`Stop`), no crash, no block | `anim-scripts.md` rule 24 (Q94 a) | — |
| `walkTo x y <who>` without both cue numbers: console line, the walk, no cue; `walkTo x y`: no cue | `anim-scripts.md` rule 24 (Q94 d) | Unit test `walk_to_point_and_cue_and_walk_exit_become_requests` |
| "Move starts afresh" re-measures the distance only; the walk animation goes on | **Our reading** | — |

### 3m-4, camera flights and cuts — `crates/kq8-core/src/{camera_flight,active,script}.rs`, `crates/kq8-viewer/src/{game_camera,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| `addCamLoc x y z [newlist]`: newlist non-zero empties the list; fly and cut use the spot nearest the camera (a tie: the first added) | `camera.md` "Camera flights and cuts" rules 1–2 | Unit test `the_list_and_the_nearest_spot` |
| N = whole(seconds × 15) + 1 updates; no seconds (or ≤ 0): distance ÷ 5,000 per second | Rule 4 | Unit test `flight_lengths`; `camera-3m4-watermill.log` (31), `camera-3m4-wheel.log` (46) |
| Straight line, the same step every update, exactly on the spot at N; roll in equal steps to `endRoll` (only with five numbers), else 0 | Rule 4 | Unit tests `the_watermill_flight`, `the_roll_goes_evenly_to_the_end_roll_or_to_0`, `cam_loc_fly_and_cut_become_requests` |
| A flight stops following, avoidance off, tilt and heading offsets to the defaults, "places" off; at N "places" on and pulled onto the range around the point 1,400 above the aim, along the line to it | Rule 4 | `camera-3m4-watermill.log` flight update 31: (188999.0, 229341.5, 10863.4) |
| Aiming: tilt −0.2 + the angle up to that point each update; heading locked within 0.02, else a third of the rest, one snap angle 0.063 on both sides | Rule 5, "Do not reproduce" | Unit test `aiming_turns_a_third_then_snaps_and_locks`; heading 2.153, tilt +0.016 |
| The cut: on the spot, heading toward (x, y), roll, one follow update (places → range; aims → aim) | Rule 3 | Unit test `a_cut_goes_to_the_spot_and_aims_at_once` |
| "Places"/"aims" set by `follow` (Connor: both on, avoidance on; none: both off) | Rule intro, open question 4 | `camera-3m4-wheel.log` (both on from play) |
| An empty list: nothing, an error logged | "Do not reproduce" | — |
| The first step is in the update after the command | **Our reading** of rule 7 (cue 5, 30 updates after cue 4, one update before the end) | `camera-3m4-watermill.log` |
| The height rule is not applied on the last update or after a cut; the wanted height is logged | **Stand-in** (not settled; Q97) | Same log: "would want z 11400.0" |
| `follow Connor` during a flight ends it | **Stand-in** (the original keeps adding the steps until N; no Daventry case) | — |
| Roll kept (also from `move KQCamera`), logged, not drawn | **Our choice** (0 in Daventry's scenes) | — |


## The answers to Q97–Q99: when the camera updates (M3 step 3n)

### 3n-1, the camera updates only when it should — `crates/kq8-core/src/camera.rs`, `crates/kq8-viewer/src/{game_camera,door_view}.rs`

This replaces "the camera updates every tick" (Q53 d).

| What | Source | Check |
| --- | --- | --- |
| "Connor moved": position change > 0.1 on an axis, or a facing change (an entry of the turn matrix by > 0.0001; we test cos and sin of the heading) | `camera.md` "When the camera updates at all" item 1 (Q99 b) | Unit test `moved_needs_more_than_0_1_on_an_axis_or_a_turn`; `camera-3n1-wheel.log` tick 19 (landing, no update) |
| A camera update only for: Connor moved, `follow` (once, aim counts as not moved), every tick while the "turn back" switch is on, the player's camera controls, flights and cuts | Items 1–5 | `camera-3n1-walk.log`: updates every tick while he walks, none from 1 tick after he stops |
| "Turn back" switch: on when `follow` finds the view > 0.4 rad off; off when a later `follow` finds the camera beyond the maximum, or a flight ends; logged | Item 3, "Do not reproduce" | — (no Daventry script sets it) |
| When one `follow` finds both, the switch ends up on | **Our reading** (the note only says "a later `follow`" clears it) | — |
| All causes in one tick give one camera update | **Our method** (the original handles one message per committed move: one per update in normal play) | — |
| The update for a script's `move Connor` comes after the script's `move KQCamera` | Item 1, "When the message arrives" (Q99 a); in our build the camera's system runs after the traps' and cues' scripts | `camera-3n1-wheel.log` ticks 17–18 |
| "Connor is moving" and the previous aim point count game updates without a message; after 3 the previous aim point is reset to where he stands | "Connor is moving" | Unit test `still_after_3_updates_without_a_message_and_the_aim_record_reset` |
| One update on the first tick | **Stand-in** (Q27 d: Daventry's start-up script says `KQCamera::follow none`; how the original starts following is not traced) | `camera-3n1-walk.log` tick 1 |

### 3n-2, the refined height rule — `crates/kq8-core/src/camera.rs`, `crates/kq8-viewer/src/game_camera.rs`

This replaces "snap within 100, else 0.5 of the gap every update" (the 0.5 was hand-tuned).

| What | Source | Check |
| --- | --- | --- |
| Order of an update: leash (max) or fixed distance, height, push out to the minimum | `camera.md` "Each camera update" steps 3–5 | Unit test `pulled_in_on_a_leash_and_pushed_out` |
| Lift at once to ground + 1,400 if under it by at most the maximum; deeper: where the line from the update's start to the new spot meets the floor grid, wanted height taken again there | "Height above the ground" rule 2.1 | Unit tests `height_lifted_at_once_…`, `height_eases_from_the_start_…` |
| Deeper and the line meets no ground: lifted to ground + 1,400 | **Stand-in** (the note doesn't cover a line that starts under the ground) | — |
| Within 100 of the wanted height: snapped | Rule 2.2 | Unit test `height_lifted_at_once_…` |
| Moved (aim point > 20 since the previous record): start height + 10% × (wanted − height after 1–2); not moved: kept; mouse swing and lifted earlier: snapped | Rule 2.3 | Unit test `height_eases_10_percent_…`; `camera-3n2-wheel.log` tick 18: 9,750 → 9,627.3 (sum 1,109.6, wanted 8,523.1) |
| "Moved" by \|Δx\| + \|Δy\| + \|Δz\|, not the signed sum | "Do not reproduce" | Unit test `height_eases_10_percent_…` (+300 / −300) |
| 10% per update as 1 − 0.9^(15 × seconds) | "Do not reproduce" (per-update rates) | Same unit test |

### 3n-3, `move KQCamera` while following — `crates/kq8-viewer/src/game_camera.rs`

| What | Source | Check |
| --- | --- | --- |
| The script's place and view stand until the next camera update (no update of its own) | `camera.md` "`move KQCamera` while following" (Q99 d), built by 3n-1 | `camera-3n3-wheel.log`: cue 4's shot unchanged until the flight |
| In that update: the tilt set at once (the view looks at the aim point), the heading not snapped | Same | `camera-3n3-wheel.log` tick 18: heading kept −0.300, +0.259 off Connor, tilt −0.846 |
| The heading stays kept in later updates until the next `follow`, flight or cut | **Stand-in** (the swing-behind rules, open question 7, are not decoded; Q100 a) | — (no Daventry case) |

### 3n-4, the flight's last update and the cut (Q97) — `crates/kq8-core/src/camera_flight.rs`, `crates/kq8-viewer/src/game_camera.rs`

This replaces the 3m-4 stand-in "the height rule is not applied on the last update".

| What | Source | Check |
| --- | --- | --- |
| A flight's last update and a cut's one update: the height rule with the aim point as not moved: lifted at once if under the ground + 1,400, snapped within 100 of the wanted height (the higher of ground + 1,400 and aim point + 1,400), otherwise kept; never eased | `camera.md` "Height above the ground" rule 2, "Camera flights and cuts" rules 3, 4 (Q97) | Unit tests `after_a_flight_or_cut_the_height_is_lifted_or_snapped_never_eased`, `the_watermill_flight`; `camera-3n4-watermill.log`: kept at 10,863.4, ground 7,650.0; `camera-3n4-wheel.log`: kept at 8,800 |

### 3n-5, the end of a scripted walk, logs only (Q98) — `crates/kq8-viewer/src/connor_view.rs`

No rule changed: A's answer to Q98 says the build already matches `anim-scripts.md` rules 24–25.

| What | Source | Check |
| --- | --- | --- |
| In the update a scripted walk ends: the walk's phase, its stop transitions (from points) and the next from that phase; when the stop transition has ended: its updates, its own steps in the data, the distance along his path, the heading change | `anim-scripts.md` rules 24–25, "How to check", open question 6 | `connor-3n5-watermill.log` updates 56 and 63 |

### A script's `move Connor` and the traps (fix after the user's check, 2026-10-18) — `crates/kq8-viewer/src/door_view.rs`

| What | Source | Check |
| --- | --- | --- |
| After a script's `move Connor` the traps' swept test starts again from the new spot; the jump's path is not swept | **Stand-in** (`traps.md` rule 5: placing by `move` not traced; `camera.md` calls `move Connor` "a jump to the new spot"; Q102) | `run-wheel2.txt`: `wheeltp2` alone, `Off` at update 133; `run-wheel1.txt` unchanged |

## Remarks and speech (M4 step 4a)

From `docs/behaviour/talking.md` (session A, 2026-10-18). The byte
layouts are also in the community notes `kq8msg.txt` and `kq8lip.txt`
(libkq8fpc, GPL-3.0; facts about layouts only, code and wording our own).

### 4a-1, reading the files — `crates/kq8-data/src/msg.rs`, `crates/kq8-cli/src/msg_verify.rs`

| What | Source | Check |
| --- | --- | --- |
| Message file: 2-byte header (0x0F / 0x8F, 0), version 5010, data size, last id, count; 11-byte records (noun, verb, case, sequence, talker, u16 text offset from the version field, 4 unused); zero-ended texts | `talking.md` rule 2; `kq8msg.txt` | `kq8 msg-verify`: all 12 files parse, version 5010, 2,080 lines; unit test `a_message_file_gives_its_lines` (made-up file) |
| Stage directions `( … )` taken out, `#` inside = "no stop", `[n]` = camera number; the space after the bracket kept | `talking.md` rule 3 | `msg-verify`: 476 lines with `#` (Daventry 100, `0.MSG` 21); the alchemist's sign shows " The alchemist's shop. …" |
| Voice / lip-sync names: `A`/`S` + file (3) + noun (2) + verb (2) + `.` + case (2) + sequence (1), base 36 | `talking.md` rule 4 | Unit test (the note's `A0RS1J00.001`); `msg-verify`: lines without a voice file match the note in every folder |
| `TALKERS.SH`: `(define NAME n)`, object name = lower case with a capital first letter; the ten generic talkers | `talking.md` rule 5 | `msg-verify`: Connor is 4; Daventry's lines per talker match the note |
| `<last>` 0: the unbroken run of sequences from `<first>` | `talking.md` rule 6 | Unit test |
| Lip-sync file: 2-byte header (0x8E / 0x0E, 0), records (u16 time, u16 shape) until time 0xFFFF | `kq8lip.txt` (layout); `talking.md` rule 17 (1/60 s, shapes 0–7) | `msg-verify`: 1,538 files read, times in order; 1,992 changes have a shape above 7 (open question 2) |
| Texts read as 8-bit Latin-1 | **Our choice** (the files are Windows text; not traced) | — |

### 4a-2, the queues and the script commands — `crates/kq8-core/src/{talk,active,script}.rs`, `crates/kq8-viewer/src/{talk_view,door_view,connor_view,load,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| `KQMonster::speak` words, `<last>` 0, `nofade` only as the 10th word, the issuer must be a `KQMonster` or Connor | `talking.md` rule 6 | Unit test `connor_turns_to_the_listener_only_in_wait_or_move`; `talk.log` of the sign run: `1000 55 0 0 1…1 (<last> 0)` |
| `<to>` another object: Connor turns with the scripted turn (`pointAndCue`'s, no cue), only in Wait or Move; the lines 0.5 s later; one waiting slot per issuer | `talking.md` rule 7, "Edge cases" | `connor.log` updates 32–42: the turn; `talk.log`: speak at 2.000 s, line at 2.533 s (the first update at or after 2.5 s); unit test `the_wait_after_turning_has_one_slot` |
| A monster issuer's turn not done (monsters don't move yet); the 0.5 s wait kept | ~~**Stand-in** (Q103 d)~~ `talking.md` rule 7 (Q103 d, 2026-10-18): it always turns, the same 0.5 s; the turn is logged only (4a-8) | — |
| One queue per character; the duplicate check by file, noun, verb, case and first sequence; the next line starts at once | `talking.md` rules 9, 11 | Sign run: three clicks during the line "duplicate: not queued"; a click after it ended plays it again (line #1); unit test `lines_play_in_turn_…` |
| A line: looked up (missing: dropped), speaker = talker object (generic: the issuer), voice next to the message file, caption if `Options::Captions` is true, ducking unless `nofade`, mouth changes from the lip-sync file, lasts its voice's length | `talking.md` rules 5, 10, 17 | Sign run: `A0RS1J00.001` 3.542 s, 39 mouth changes, caption with the leading space, ends at 6.133 s |
| Line time counted in game time (15 updates per second) | **Our choice** (the voice plays in real time; the same while the game keeps up) | — |
| A line with no voice or a missing talker: 0.06 s per character, at least 1 s | `talking.md` "Do not reproduce" (the note's example) | Unit test `talkers_speakers_and_lines_that_cant_be_played` |
| `TalkComplete f n v c s` after every line: the issuer's own script at once (not Connor's), the `stateChangeNotify` group through the event queue (the next update); the issuer registered for itself is skipped | `talking.md` rule 13, "Do not reproduce" (no reused numbers; every line has its own) | Unit test `talk_complete_reaches_the_notify_group_a_moment_later`; sign run: "Connor has no script … nobody" |
| `clearSpeak`: queue emptied, the line stopped, no `TalkComplete` | `talking.md` rule 15 | Unit test `clear_speak_stops_everything_without_talk_complete` |
| Pending 0.5 s waits survive `clearSpeak` | ~~**Stand-in** (Q103 b)~~ `talking.md` rule 15 (Q103 b, 2026-10-18: confirmed) | — |
| `testMessage`: 9 or 11 words, 0 when equal; the file as 16 bits, the rest as bytes | `talking.md` rule 14 | Unit test `test_message_takes_9_or_11_words` |
| `eval <command…>` runs the command and keeps its result | ~~**Stand-in** (no note; Q103 a)~~ `anim-scripts.md` rule 18 (Q103 a, 2026-10-18: confirmed for the data's uses). Two details of rule 18 are **not** built (no data uses them; code unchanged at the user's request): `eval` with nothing after it gives 1 (the note: 0), and the words are not read again as a new line (no second `$` replacement, quotes and `;` not counted again) | Unit test (the listener script) |
| Message files searched in the world's `English` folder, then `GAME/ENGLISH` | `talking.md` rule 1 (the data's reading, not traced) | Viewer log: 4 files for Daventry |
| `Options::Captions` from the `set` lines of `MASK/Options.cs` | `talking.md` rule 10 | Viewer log: `Options.cs sets Options::Captions = 'true'` |
| Monsters: placed `KQMonster` objects with their place after `move` and their `script=`; their script gets `TalkComplete` (not `Init`, M5) | `talking.md` rules 6, 13 | Viewer log: Girl, Wizard, Ravenw, Shadow |

### 4a-3, hearing it — `crates/kq8-core/src/sound.rs`, `crates/kq8-viewer/src/{talk_view,sound_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| Speech plays in group 3 (speech slider, not ducked); Connor's plain, others placed at the speaker when the line starts (copied once, not following the speaker: confirmed by Q103 c, 2026-10-18), min 2,000, max 30,000 | `talking.md` rule 10, `sound.md` rules 50, 58 | Unit test `speech_ducks_effects_and_world_music_not_itself`; sign run `sound.log`: group 3 plain, loudness 0.340 |
| While a line without `nofade` plays, groups 0, 2, 5 × 0.2; back to 1 when no line ducks | `talking.md` rules 10–11, `sound.md` rule 53 | Sign run `sound.log`: 0.028 during the line, 0.140 after |
| `clearSpeak` stops the voice at once | `talking.md` rule 15 | Code path only (not reached in a run) |
| A non-Connor speaker with no known place: plain voice | **Our choice** (not in the note) | — |

### 4a-4, captions — `crates/kq8-viewer/src/{talk_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| The line's text (stage directions out) shown while its queue talks, if `Options::Captions` is true; hidden when that queue is empty | `talking.md` rules 3, 10–11 | Sign run `viewer.log`: shown at update 38, hidden after the line |
| The panel's look: Bevy's font, 20 px, white on a 60% black band, 80% wide, 40 px above the bottom | **Stand-in** (`talking.md` open question 4: the GUI is not studied) | — |

### 4a-5, Connor's own remarks; cutting a line short — `crates/kq8-core/src/{talk,active}.rs`, `crates/kq8-viewer/src/{connor_view,door_view,sound_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| Out of reach with `tooFarAwayAction` 0: 0 0 0 1 1 through Connor's queue, no turn | `talking.md` rule 19, `clicking.md` rule 10 | Too-far run: `A0000000.011`, 1.736 s |
| An item not taken: 0 0 0 4 1, no turn | `talking.md` rule 19, `clicking.md` rule 12 | Code path only (shares `connor_remark` with "too far") |
| A chest's `Take…`/`Empty…` remark: `Msg Noun Verb Case Seq` and `End` as the last sequence; Connor turns to the chest unless `doNotPointAt`, then 0.5 s | `talking.md` rule 19, `chests.md` rules 8–10 | Chest run: turn from update 82, "'Tis empty." at 5.867 s; unit test (empty remark wording) |
| ~~`doNotPointAt`, or Connor not in Wait or Move: no turn and no wait~~ **Replaced in 4a-9**: only `doNotPointAt` gives no turn and no wait; the chest turns him in any state | ~~**Our reading** (Q103 e)~~ `talking.md` rule 19, `chests.md` rule 10 (Q103 e) | See 4a-9 |
| A press of forward, back, a turn key or Jump, or a click in the world, stops Connor's line as `clearSpeak Connor` unless it has `#`; releases don't count; `handsOff` swallows them | `talking.md` rule 16, `anim-scripts.md` rule 21 | Tombstone run: cut at 1.667 of 11.971 s; unit test `only_lines_without_the_mark_are_cut_short` |
| The test options `--turn`, `--jump`, `--click`, `--aim` count as presses / clicks | **Our choice** (test tools) | Tombstone run |

### 4a-6, the mouth — `crates/kq8-viewer/src/{load,connor_view,talk_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| Connor's face texture `baseLip=cf0.bmp` is replaced by `lip<n>` of the `lips=` section (`[ConnerLips]` `CF0`–`CF7.bmp`) at each mouth change; a shape of 8 or more uses `lip0` | `talking.md` rule 18; data `GAME/Kq/Connor.kq` | Sign run `connor.log`: lip7, lip6 … on 122 face materials |
| At the end of a line the mouth goes to rest (shape 0 → `lip0`) | `talking.md` rule 11 | Sign run: lip0 at update 92 |
| A change to the shape already shown is not applied again | **Our choice** (same picture) | 36 swaps for 39 changes |
| Mouth changes are applied once per update (15 per second) | **Our choice** (the game logic runs at 15 per second) | — |

## A's answers to Q104 and Q103 (M4 steps 4a-7 to 4a-9)

From `docs/behaviour/connor-movement.md` "Turning" rule 2,
`clicking.md` rules 25–33, `talking.md` rules 7, 10, 15, 19,
`chests.md` rule 10, `anim-scripts.md` rule 18 and "Edge cases"
(session A, 2026-10-18; the user checked Q104 in the original).

### 4a-7, turn keys are events — `crates/kq8-core/src/{turn_keys,connor_auto,connor,connor_update}.rs`, `crates/kq8-viewer/src/connor_view.rs`

| What | Source | Check |
| --- | --- | --- |
| The turn direction changes only on key events: down sets ±1, up sets 0 | `connor-movement.md` "Turning" rule 2 (Q104 a) | Unit tests in `turn_keys.rs`; `connor-4a7-sign-held-left.log` |
| Only a down cancels an automatic turn; a held key neither cancels it nor adds to its 3.5 rad/s | Same | `connor-4a7-sign-held-left.log`: left held from update 10, turn 18–25 at 0.233 per update; `connor-4a7-sign-press-during.log`: right pressed at 20, cancelled at 21; unit test `a_held_key_neither_cancels_nor_changes_the_turn_and_forward_is_kept` |
| When an automatic turn ends the direction is 0; a held key does not take over | Same; checked by the user in the original | `connor-4a7-sign-held-left.log`: heading 2.4958 from update 25 on with left held |
| Letting go of one turn key while the other is held sends the other as a down (cancels) | Same | Unit test `letting_go_of_one_key_sends_the_other_again_as_a_down` |
| Forward/back and the run switch stay the player's during a turn without a walk (click, `speak`, chest, `pointAndCue`) | Same | Unit test (forward and run kept) |
| Standing, the turn still plays `Stop`, then `Left`/`Right` | `anim-scripts.md` rules 24–25 (unchanged) | Same runs: update 18 `stop`, then `left` |
| `handsOff`: presses swallowed, releases counted; when it goes off the held turn key goes down again | `anim-scripts.md` rules 21–22 | Unit test `hands_off_swallows_presses_but_not_releases`; code path for "sent again" only |
| A cancelled turn with a cue: "cue lost" in the log | `anim-scripts.md` "Edge cases" | Code path only (every `pointAndCue` in Daventry is under `handsOff`) |
| Keyboard auto-repeat still clears the wall blocks but is not a down for the turn | **Our reading**: the note says nothing re-sends a held key, and the user saw a held key neither cancel nor resume the turn | Runs above |
| `--turn N,dir` = pressed at N and held; `--hold`'s turn pressed at the start, let go at its end | **Our choice** (test tools) | Runs above |

### 4a-8, the turn after a click — `crates/kq8-core/src/{active,connor}.rs`, `crates/kq8-viewer/src/{connor_view,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| After every `ConnorAction` call (`do`, an accepted item, `TooFar`; `Special` not made yet) on doors, scripted objects, scripted clickables, buildings with a script and items with a script, Connor is asked to turn to the object's placement point | `clicking.md` rules 25, 31; `doors.md` rule 16 | Unit test `a_click_call_asks_for_a_turn_to_the_object_after_the_script`; `connor-4a8-tmbstn1.log`, `connor-4a8-wizbook.log` |
| No call, no turn: no script, "too far" with `tooFarAwayAction` 0, an item not taken; a `TooFar` call (`tooFarAwayAction` non-zero) turns him from any distance | `clicking.md` rule 26 | Same unit test |
| The request comes after the script's own requests; Connor's state is looked at when it is applied: Wait or Move turn, every other state of ours doesn't | `clicking.md` rules 27–28 | Unit test `cam_loc_fly_and_cut_become_requests` (the turn is the last request); runs: "state after the script Wait" |
| ~~His state is the one after his own next update (requests are applied after it in our build)~~ **Replaced 2026-10-18 (4c-5)**: his state at the click, or the state the click's script asked for and got (a `setState` or scripted walk applied before the turn) | `clicking.md` rule 35 (A's answer to Q105 b) | `connor-4c5-fish.log`: "state after the script Wait (his state at the click, rule 35)"; the "asked for" path: no Daventry click script sets Connor's state (not tested by a run) |
| `doNotPointAt` stops it; items default to 1; the key is looked up by its whole name, so `doNotPointAtMe` does nothing | `clicking.md` rule 29 | `doors-4a8-urn.log`: `urn1` "doNotPointAt: no turn"; `connor-4a8-wizbook.log`: the book turns him (updates 46–60); unit test `a_picked_up_item_does_not_turn_him_by_default` |
| ~~`doNotPointAt` not read for buildings~~ **Fixed 2026-10-18 (4c-5)**: read like for any object | `clicking.md` rule 36 (Q105 c) | Unit test `a_building_without_a_script_swallows_the_click` (a building with a script and `doNotPointAt` gives no turn); no Daventry building sets it |
| An object deleted by its own click script still turns him (to where it stood) | `clicking.md` rule 34 (A's answer to Q105 a: our stand-in was right) | Unit test `a_scripted_clickable_eats_itself_once`; `connor-4c5-fish.log` |
| One automatic turn: a cue-less turn asked for during a running turn (not a walk) only replaces its target, a `pointAndCue` keeps its cue | `clicking.md` rule 32, `anim-scripts.md` "Edge cases" | Unit test `a_second_turn_request_only_replaces_the_target`; `connor-4a8-tmbstn2.log`: "one automatic turn: the running turn (update 0) gets this target" |
| `speak Connor Connor`: the line at once, the click's turn during it; `speak Connor $1`: one turn, the line 0.5 s after the speak | `clicking.md` rule 32 | `talk-4a8-tmbstn1.log`: line at 1.333 s, turn updates 22–32; `talk-4a8-tmbstn2.log`: line at 1.867 s |
| Chests: no click turn (only before their remark) | `clicking.md` rule 33 | Log line "chest: no ConnorAction call" |
| Another character's `speak` turn: always, no state check, same 0.5 s; only logged (monsters don't move yet) | `talking.md` rule 7 (Q103 d) | Code path only (no character speaks outside conversations) |

### 4a-9, the chest remark turns him in any state — `crates/kq8-core/src/active.rs`

| What | Source | Check |
| --- | --- | --- |
| A chest's `Take`/`Empty` remark: the turn to the chest and the 0.5 s wait go together, in any state of Connor's; only `doNotPointAt` gives no turn and no wait (queued at once) | `talking.md` rule 19, `chests.md` rule 10 (Q103 e) | Unit test `a_chest_opens_then_gives_its_item_then_is_empty` (not in Wait or Move: turn and wait; `doNotPointAt`: at once); `connor-4a9-chest-jump.log`: clicked during a jump (update 40), turn 42–48, "'Tis empty." at 3.200 s |
| Q103 (a)–(c): `eval`, `clearSpeak` during the wait, the placed voice | `anim-scripts.md` rule 18, `talking.md` rules 10, 15 | Rows of 4a-2 and 4a-3 updated; no code changed |

## The interface's files (M4 step 4b)

From the game's data and the community notes `kq8pfon.txt`,
`kq8pbma.txt`, `kq8gui.txt`, `kq8tag.txt` (libkq8fpc, GPL-3.0
repository, read 2026-10-18 from <https://svn.nicode.net/libkq8fpc/doc/>;
facts about file layouts only, code and wording our own). No behaviour
note yet (A is studying the interface).

### 4b-1, fonts — `crates/kq8-data/src/font.rs`, `crates/kq8-cli/src/font_verify.rs`

| What | Source | Check |
| --- | --- | --- |
| `PFON` block: 31-bit size and an alignment bit (2 or 4), then version 2, the 44-byte font info (font flags, text flags, glyph count, height, width, text and back colour, baseline, two 16.16 scales, spacing), character count and first code, the character map (−1: no glyph), 8-byte glyph entries (picture, left, top, width, height, baseline shift, 2 spare) | `kq8pfon.txt` | `kq8 font-verify`: all 75 fonts read to their exact end; unit test `a_tiny_font_gives_its_settings_map_and_glyph_pixels` (made-up font) |
| Bitmap array `PBMA`: `head` (chunk count, version, bitmap count), then other chunks, then one `PBMP` bitmap per entry (our existing bitmap reader); the `PBMA`/`PBMP` size fields are skipped | `kq8pbma.txt` | `font-verify`: bitmap count = the head's in all 75 |
| The `rmap` chunk is present in 71 of the 75 fonts (not in `console1`/`console2`); it is read and ignored | Observed (the note says it is not used by KQMoE) | `font-verify` "array chunks" |
| A bitmap ends at the next `PBMP` tag or at the first chunk that is not `head`, `data`, `DETL` or `PPAL` | **Our reading** (the note doesn't say how to find a bitmap's end) | Every font then ends exactly with the palette flag |
| Font flags `0x41` (bit 6 not in the note) in all fonts but the two `Resource` console fonts (`0x46`: monospaced, monochrome) | Observed | `font-verify` |
| Glyph pixels are palette indices with many values per font (shaded letters); 0 = no ink; no font has its own palette | Observed | `font-verify` pixel values; which palette colours them: Q106 |
| `14.pft` (and each world's `14*.pft`) covers only the space and the digits 0–9 | Observed | `font-verify`: 11 glyphs |

### 4b-2, layouts and tags — `crates/kq8-data/src/gui.rs`, `crates/kq8-cli/src/gui_verify.rs`

| What | Source | Check |
| --- | --- | --- |
| A layout is one block: tag, size, version, id, flags, rectangle (left, top, right, bottom); then the fields of its kind, each kind's fields after those of the kind it builds on (`UIDL` child counts and children; `UICD` caption y, font, justification, text; `CCBD` / `CCAD` a picture; `UIBM` picture; `UITX` font, justification, text; `CCAB` / `CCBB` text + picture; the unused kinds too) | `kq8gui.txt` | `kq8 gui-verify`: the 9 layouts read to their exact end; unit test `a_dialog_with_a_bitmap_button_and_a_text` |
| The block's size counts the bytes after the 8-byte tag and size; the field sizes (4 bytes each) | Observed (the note gives types, not sizes) | Every block of the 9 files ends exactly where its size says |
| The tag file: `[TAG]`, then `id name <q>string<q>` lines, `//` comments, `[END]` | `kq8tag.txt` | `gui-verify`: 739 tags, no line refused; all 241 block ids have a tag |
| Children's rectangles are relative to their dialog | Observed (they fit their parent's size, e.g. the meters' bars inside the 256×14 strip) | `gui-verify-4b2.log`; Q107 (c) |
| The caption box: `IDDLG_CAPTION` (542) in `Gplayscr.gui`, a `CCBD` at (320, 400)-(640, 416), caption y 2, font `console.pft` (the world's own, 15 high), justification 0x9, picture `stontran.pbm` (the world's `8gui` folder) | Data | `gui-verify-4b2.log`; how it is drawn: Q107 |
| The in-game panels (meters, health slots, inventory bars, minimap, map window) and their pictures are in the same file | Data | `gui-verify-4b2.log` |

### 4b-3, captions in the original font — `crates/kq8-core/src/caption.rs`, `crates/kq8-viewer/src/{talk_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| The caption's font (`console.pft`, the world's `8gui` copy), strip picture (`stontran.pbm`), line height 16 and text 2 pixels down | Data: `IDDLG_CAPTION` in `Gplayscr.gui` (4b-2) | `viewer-4b3-sign.log`: "caption: IDDLG_CAPTION (320, 400)-(640, 416), caption y 2, font Daventry/8gui/console.pft …" |
| At the bottom of the screen (no status bar is drawn yet), text centred, the strip growing upwards one line per text line | The user's check in the original (2026-10-18) | `viewer-4b3-sign.log`: 2 lines, 640x32; `viewer-4b3-tomb.log`: 4 lines, 640x64 |
| Wrapped by words at 400 pixels | **Stand-in** (Q107 a; the user: "mostly the correct word, but not exactly" on several tombstones, left for A): the user's line break (after "something") puts the width between 365 and 418 pixels in this font, not the layout's 320 | Unit test `words_are_added_while_they_fit`; sign: lines 365 and 104 wide, the break where the user saw it |
| Font and strip coloured by the world's palette, index 0 see-through | **Stand-in** (Q106; the user saw the colours as right, 2026-10-18), chosen by measurement: with the world palette the font's 8 colours are black (its most common, an edge) and golds (232, 200, 128 …), the strip's are greys 56-104 (stone); the menus palette gives a pale yellow and red strip. The user describes gold letters on a stone strip | — |
| ~~The strip is drawn across the whole screen width~~ **Replaced 2026-10-18**: the strip covers the widest line plus 16 pixels on each side, centred, tiled from its own left edge | The user's check in the original ("at most around 2 letters" beyond the text on each side); 16 pixels is **our estimate** (about two lower-case letters in `console.pft`; Q107 a) | Unit test `the_picture_has_one_strip_line_per_text_line_and_centred_text`; `viewer-4b4-sign.log`: strip x 121..518 for the 365-pixel line |
| The 640x480 picture is stretched to the window, nearest pixels | **Our choice** | — |
| Without the layout, font or palette: the old 4a-4 text panel | **Our choice** | Log line "the 4a-4 text panel is used" |

### 4c-1, the caption strip as the original's — `crates/kq8-core/src/caption.rs`, `crates/kq8-viewer/src/talk_view.rs`

Replaces the 4b-3 stand-ins above (wrap at 400 by words, the strip fitted to the text, a 640×480 picture, 16 pixels per line).

| What | Source | Check |
| --- | --- | --- |
| Rows cut letter by letter at the caption box's width: glyph width × horizontal scale, spacing after each letter that fits; a letter that would pass the width ends the row at the last space before it (dropped); spaces that fit at a row's end kept; spaces count like letters (the sign's leading space too, `talking.md` rule 3) | `interface.md` rule 47 | Unit test `rows_break_at_the_last_space_before_the_letter_that_does_not_fit`; `viewer-4c1-sign.log`: rows 371 and 104 wide, the break after "something" |
| A row with no space to break at is cut at the box's edge | `interface.md` "Edge cases" (the note's "do instead") | Same unit test |
| Height rows × font height (15) + 4, growing upwards; each row centred in the box, 2 pixels down in its own 15-pixel slot | `interface.md` rules 48–49; caption y 2 from the layout | `viewer-4c1-sign.log`: box 400x34 |
| The stone covers the whole box, not the text | `interface.md` rule 50 (traced; the user's check: a short line's strip is as wide as a long one's) | Unit test `the_stone_covers_the_whole_box_and_rows_are_centred` |
| The box's bottom = the top of the highest bottom panel (none drawn yet: the screen's bottom); its centre half-way between the map's right edge and the screen's right edge; the map is always away (no map yet), its right edge at −50/640 of the screen | `interface.md` rules 9, 51 (traced; the user's check: a little left of the middle with the map closed) | Unit test `the_box_sits_on_the_bottom_panel_left_of_the_middle_with_the_map_away`; `viewer-4c1-sign.log`: left 168.75, centre 368.75 of 800 |
| The screen is 800 × 600 when `Options.cs` has `set800mode 1`, the layout scaled by 1.25: box 400 wide; fonts at their own size | `interface.md` rule 47 (**inferred** by A, fits the user's measured 365–418); **stand-in** until Q108 | Viewer log: "set800mode true: screen 800x600 (layout x 1.25), box 400 px wide" |
| The stone picture stretched with the layout (nearest pixel) and tiled from the box's top-left | **Stand-in** (`interface.md` open question 6, Q108) | — |
| (2026-10-27, replaces the row above for the stone's size) The stone tiled from the box's top-left (rule 20a) with each of its pixels a whole number of window pixels: the true scale (800 × 600 layout × the window's 4:3 frame × Windows' display scaling) rounded, at least 1; the caption picture made at window pixels (letters as before, enlarged nearest pixel) and shown at a whole window pixel at its own size | Rule 20a: tiled "at its own size" at 640 × 480; the 800 × 600 scale is open (`interface.md` open question 1). Data: `stontran.pbm` is a one-pixel checkerboard, 2,048 of 4,096 pixels colour 0 (see-through). **Ours:** the whole-pixel rounding, so the checkerboard stays even on any window (an uneven stretch, × 1.25 and × 1.2 at 1280 × 720, turned it into stripes and blocks: the user's report) | Unit test `a_checkerboard_stone_stays_even_at_any_window_size` (800 × 600, 1280 × 720, 1920 × 1080). Viewer, 1280 × 720: "box 400x34 made at 480x41 window pixels (x 1.200); each of its pixels 2 window pixel(s) square", shown 480 × 41 at (363, 583) |

### 4c-2, the always-visible bottom bar — `crates/kq8-core/src/{panels,meters,gui_draw,inventory}.rs`, `crates/kq8-viewer/src/{gui_view,talk_view,door_view,load}.rs`, `crates/kq8-data/src/{items,font}.rs`

| What | Source | Check |
| --- | --- | --- |
| The panels and what each is: bottom left, eight slots, meters, bottom right, status icons, by their `Gplayscr.gui` blocks | `interface.md` rule 7; data | `gui.log` |
| Panel groups: the bottom bar = bottom left, slots, bottom right and meters together; the meters alone | `interface.md` rule 8 | Unit tests in `panels.rs` |
| Sliding in 0.2 s of real time for the whole distance; a panel already going that way is left alone; bottom panels off the bottom edge, the top bar 1/480 above its height, the map 50/640 beyond its width; sounds `intrup`/`intrdown` (top), `map`/`mapbgone` | `interface.md` rule 9 | Unit tests `space_slides…`, `hidden_places`; `gui.log`: hide at 2.501 s, fully hidden at 2.707 s |
| Space (row 12 of `movekeys.kq`, `KeyChar12=32`) toggles the bottom bar | `interface.md` rule 10; data | `gui.log` "movekeys.kq row 12: KeyChar12=Some(32) KeyWord12=space" |
| `handsOff 1`: a shown bottom bar is remembered and slides away, the map hidden, the spinning mask on; `handsOff 0`: back only if it was shown | `interface.md` rule 15 (the user's check 7) | Unit test `hands_off_remembers…`; `gui-4c2-bricks*.log` |
| Conversations put the bar away and bring it back, not during handsOff (built, nothing calls it yet: conversations are not studied) | `interface.md` rule 16 | Unit test `conversations_put_the_bar_away…` |
| `setLowerGui`, `setUpperGui`, `setMapGui` 0\|1, `getGuiStatus` | `interface.md` rule 15 | Unit test `connor_stats_events_are_counted_and_panel_commands_handed_over` |
| The screen opens with the bottom bar shown, the top bar, map and popup hidden | `interface.md` rule 19; the user's check 3 in the original | `gui.log` "the screen opens" |
| Bars: width × percent / 100 cut to whole pixels, at least 1 when not 0, percent kept in 0–100; health % = health × 100 / (10 × level), ≥ 1 if health isn't 0; experience % = experience × 100 / threshold, sent only when it changed; all values after every stats event | `interface.md` rules 20–24 | Unit tests in `meters.rs`; bricks run: "health 100 % (bar 107 of 107 px), experience 200 % (bar 108 of 108 px), level 2" |
| A health % sent while the meters are hidden pops them up alone for 1.5 s (real time); bringing the bar back clears the timer; a new % while popped renews it | `interface.md` rule 25 (the renewal: "inferred" by A) | Unit test `the_meters_pop_up_alone…`; `gui-4c2-bricks-bar-away.log`: shown at 1.302 s, hidden at 2.807 s |
| The three numbers are Connor's stats 4, 2, 3; a number that changed flashes `fx.pba` once over its slot | `interface.md` rules 27, 31, 33, 36 | Bricks run: flashes armour, hand, ranged |
| Slot pictures: the worn item's `GUIBitmap` (via the tag file), else the layout's empty one; the armour picture by rule 35 | `interface.md` rules 34–35 | Unit test `the_slots_show_what_he_wears`; `gui.log` "files [2armr0.pbm, 2sord0.pbm, 2rng0.pbm]" |
| With `GodArmor` its picture wins, then a full suit; `NoArmor` counts as no body armour beside a suit | **Stand-in** (not traced; `inventory.md` "Edge cases") | Same unit test |
| Connor's start inventory from `[ConnerInventory]` (`inventory=` of his section, `numItems`, `ItemName<k>` → `itemType`, `quantity`, `userId`), named after their sections (no `NoName` number used) | `inventory.md` rule 4; data | `items.log`: "Connor's start inventory: Mask1a: INVITEM_Piece1A …" (5 items) |
| `Invinfo.itl` fields 3–7 (stats), 8 (`Comb`), 10 (`GUIBitmap`), 11 (message), 12 (mode), 13 (`Dur`), 17 (use sound), 19 (cursor) | `inventory.md` rule 1; the file's own `;itemX=` header | Unit test `reads_the_fields_by_position` |
| `.pba` files read with the bitmap-array reader of 4b-1 | `kq8pbma.txt` (as 4b-1) | `fx.pba` 17 frames, `mskicn.pba` 16 |
| Children's rectangles relative to their dialog; a text with right < left anchored at (left, top); justification bits | `interface.md` rules 5–6 | Unit test `numbers_anchored_at_a_point_or_aligned_in_their_box` |
| A dialog's children drawn last first (the first listed on top) | **Our reading of the data**: every panel lists its numbers before its pictures | — |
| Index 0 see-through only for pictures whose flags have bit 1 (only `stontran.pbm` among the panels') | `kq8pbmp.txt` ("transparent" bit); data: the flags read 8 or 9 | — |
| The layout's own placeholder texts ("0", "3Dfx") not drawn; the red alternative numbers and the level-up flame not drawn | `interface.md` rules 26, 32 (M5 / not traced) | — |
| A click on a shown panel is not a world click | **Our choice** (the bars cover the world; how the original passes clicks was not described) | `doors.log` "mouse click on the panel …" |
| Pictures × 1.25 (nearest pixel), tiled when smaller than their box; fonts at their own size; animations 15 frames per second | **Stand-ins** (Q108) | — |
| The caption box's bottom follows the highest of bottom left, slots and meters as they slide | `interface.md` rule 51 | Sign run: "caption box at left 168.75 top 486.00 bottom 520.00" |

### 4c-3, the inventory bars — `crates/kq8-core/src/connor_inventory.rs`, `crates/kq8-core/src/{active,item,panels}.rs`, `crates/kq8-viewer/src/{gui_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| Places: a full suit covers body armour, pants, gloves, boots (not the helmet); the chain-mail suit pants, gloves, boots; `GodArmor` its own; each weapon and armour category one place | `inventory.md` rule 3 | Unit test `equipment_pushes_out_what_shares_its_place` |
| Going in: `HealthItem` / `Money` / `Rock` with `Comb` 1 join the entry he has (the new object deleted); others are new entries; equipment takes off what shares a place (placeholders deleted, others put into the world where the new one lay) and is put on | `inventory.md` rule 8 (the user's check 4 in the original) | Unit tests `stacks_join…`, `equipment_pushes_out…`; `gui-4c3-coins.log` (7 then 12, one entry); `gui-4c3-dagger.log` ("placeholder Fist1 (INVITEM_Fists) deleted", hand 10 → 15) |
| Every way in goes through these rules: a pickup, a chest's item, `give` / `KQMonster::inventory Connor` | `inventory.md` rule 8 ("picking up, `give`, chests") | `items.log` "inventory (picked up)", "inventory (basket1.cs line 4: give)" |
| An old piece pushed out by a `give` drops at Connor's feet | **Stand-in** (`inventory.md` "Edge cases" infers the new object's own position, which a `give` doesn't have) | Same unit test |
| Applying: only if not applied; a stack loses 1; its five numbers added through one stats event; `Dur` 0 stays, `Dur` > 0 a timer then the numbers taken back, `Dur` −1 un-applied at once keeping the numbers; a stack at 0 leaves | `inventory.md` rules 13–15 | Unit tests `stacks_join_and_health_items_heal_for_good`, `a_potion_runs_out_and_its_numbers_go_back`; mushroom run: 5 → 4 → 3 → 2 with `healing.aud` |
| "Applied" kept in `userId` as 0 for Connor; the potions' timers in game time | **Stand-ins** (the original keeps an object id; rule 14 doesn't say which clock) | — |
| A left click on a slot: ignored during handsOff; quest items, money, rocks, rope "would be held" (4c-4); the mask piece: the main menu (none yet, logged); everything else applied with its `useSnd` if it wasn't applied | `inventory.md` rule 9; `interface.md` rule 41 | Unit tests; `items.log` "would be held", "opens the main menu", "apply Fist1" |
| A right click: the description, `0.MSG` with the type's message numbers, its voice played plain, a new one stopping the last; an empty slot: nothing | `inventory.md` rule 10; `talking.md` rule 20 | `items.log` "describes it: 0.MSG 0 47 0 0 1: GAME/ENGLISH/A0001b00.001"; `sound.log` starts it in group 3, no ducking |
| Keys 1–8 (and the keypad's) use slots 1–8: `Mushroom`, `Crystal`, `SacredWater`, `ElixerOfLife`, `Invulnerable`, `Strength`, `Clarity`, `Invisible` | `interface.md` rules 38, 40; `inventory.md` rule 11 | Mushroom run (`--key 1,40`) |
| The slots' counts (`14.pft`, right/top at their anchors); a stack at 0 shows the empty picture | `interface.md` rule 39 | `gui.log` "slots [5, 0, …]" |
| Counts of 0 not written (gold, rocks, slots) | **Stand-in** (Q109) | — |
| The quest strip: quest items in the order they went in, 7 from a first-shown index, the arrows ±1 kept in 0..count − 7 | `inventory.md` rules 5–6 (rule 5 "inferred") | `gui-4c3-scroll.log`: strip ["Mask1a", "NoName5"]; arrow: "first shown 0 -> 0 of 1" |
| The gold picture by count (0, 1–24, 25–49, 50 or more); the mask button the mask piece's picture; the rope `2rope1` when carried | `interface.md` rules 41, 43, 45 | `gui-4c3-coins.log` |
| The rocks picture: the rock type's own (`2rocks3`) when he has any | **Stand-in** (Q109; `interface.md` open question 4) | — |
| A new item or a higher count blinks: picture and empty picture every 0.25 s, 5 times (shown, empty, shown, empty, shown) | `interface.md` rule 44; the order of the five is **our reading** (Q109) | `gui.log` "blinking ["NoName5"]" |
| A new quest item or mask piece pops the top bar for 1.5 s; `intrup.aud` / `intrdown.aud` | `interface.md` rules 9, 13; `inventory.md` rule 8 | `gui-4c3-scroll.log`: shown at 1.178 s, hidden at 2.684 s; `sound.log` both sounds |
| The top bar comes down when the pointer reaches the window's top row, goes up below y 60 of 480 unless popped; not while the right button is held or it is sliding | `interface.md` rule 12 | Not tested by a run (needs the mouse) |
| A click on a bar goes to the control under it (first listed on top); texts and the flashes parked at y 900 are not targets | Our reading of the layout (rule 5) | `--key left:NAME` / `right:NAME` runs |
| `--key left:NAME,N` / `right:NAME,N` | **Our test option** | — |

### 4c-4, the item in hand — `crates/kq8-core/src/{connor_inventory,active}.rs`, `crates/kq8-viewer/src/{gui_view,door_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| A left click on a quest item, the coins, the rocks or the rope makes it the hand; the same item again puts it back; another one replaces it (the first goes back) | `inventory.md` rule 9 | Unit tests `quest_items_money_are_held_the_order_kept`, `the_hand_empties…`; coins run: "now the item in hand, the pointer IDBMA_GOLDCURS", then "put back" |
| The pointer: the held type's `cursor` picture, else the play pointer `IDBMA_CURS04` (`Gplayscr.kq` `cursor=`); drawn as the window's cursor image, scaled in whole steps like the screen, index 0 see-through (the cursors' bit 1) | `inventory.md` rule 9; `interface.md` rule 1; data | `gui.log` "pointer: IDBMA_GOLDCURS (goldcurs.pba, 32x32 …)" |
| The click point at the picture's top-left | **Stand-in** (Q110) | — |
| A held quest item leaves the strip, the others close the gap; the arrows' limit still counts it; a click on the empty place it left puts it back | `inventory.md` rules 7, 12 (rule 7 checked from memory by the user) | — (needs 8+ quest items) |
| `INVITEM_Scroll` can't be taken: Connor reads it, at most every 3 s and not while he talks | `inventory.md` rule 9 | Scroll run: "line 0 92 14 0 1 starts … PERMANENT SPELL OF MIGHT"; a second click 0.67 s later: "less than 3 s ago: nothing" |
| The note's `0.MSG (0, 92, 14, 0)` and `(0, 0, 58, 37)` read as file, noun, verb, case with sequence 1 | **Our reading**, checked against the data: as noun, verb, case, sequence the scroll's line is not in `0.MSG`; read this way it is the scroll's spell and "I am not near anything to climb." (Q110) | Scroll and rope runs (`talk.log`) |
| A type with cursor `none` can't be held | `inventory.md` rule 9 | — |
| The rope: held, and away from a rope place (none yet) the remark | `inventory.md` rule 9; holding **and** the remark is a **stand-in** (Q110) | Rope run: "now the item in hand, the pointer IDBMA_CURS06; … 'I am not near anything to climb.'" |
| A world click with the hand is the click of `clicking.md` rules 12–18 (accept lists, `$3` = the item's name, `$4` its type); accepted → the hand is emptied, except coins while `goldCoinPersists 1`; refused or out of reach → it stays; the item itself is not removed by this | `inventory.md` rule 12 (the user's check 4: a refused item stays on the pointer) | Unit test `the_hand_empties_when_accepted_except_coins_that_persist`; coins run ("not accepted … it stays in hand"); star globe run ("accepted: the hand is emptied … taken by the script") |
| An accepted item at a building without a script (no call) counts as accepted | **Our reading** of rule 12 ("accepted" = the accept question said yes) | — |
| `goldCoinPersists 0\|1`; 0 also empties the hand at once | `inventory.md` rule 12 | — (Gnome's apothecary) |
| A held item a script deleted or took: the hand is emptied | **Our choice** (the note doesn't say; the user: no item is ever lost) | Same unit test |
| `--item TYPE`: a real item named `TestItem`, carried, in the hand | **Our test option** (replaces the 3e fake held item named after its type) | Star globe run |
| Escape still quits; the original opens the main menu and pauses the game | `interface.md` rule 14 (the user's check); no menu yet | — |

### 4c-6, a 4:3 screen in the middle of the window — `crates/kq8-core/src/panels.rs`, `crates/kq8-viewer/src/{gui_view,talk_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| The panels, the caption, the pointer's screen position, the click areas and the pointer's size are on the largest 4:3 rectangle centred in the window; the world fills the whole window | **The user's choice** (2026-10-18: the original runs at a fixed 4:3 and its panels are laid out for it; "a virtual 4:3 screen in the middle") | Unit test `the_screen_is_4_3_in_the_middle_of_the_window`; `viewer.log` "screen: window 1280x720: the game's 4:3 screen at left 160.0 top 0.0, 960.0x720.0" |
| The top bar comes down with the pointer at the 4:3 screen's top row | **Our choice** (follows the screen) | — (needs the real mouse) |

### 4c-7, the user's corrections — `crates/kq8-core/src/panels.rs`, `crates/kq8-viewer/src/gui_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Space plays `intrup.aud` as the bottom bar starts to show and `intrdown.aud` as it starts to hide; handsOff, scripts and the meters' pop-up play nothing | **The user's check in the original** (2026-10-18), against `interface.md` rule 9 ("no sound of their own"); which sound with which way is a **stand-in** (Q111 a) | Unit tests in `panels.rs`; `gui-4c7-bricks.log` "Space: intrdown.aud", "Space: intrup.aud"; `sound-4c7-bricks.log`; handsOff at 1.370 s: no sound |
| The status icons at their own size (48 × 48 on the 800 × 600 screen), the spinning mask's bottom right corner at the screen's | **The user's check in the original** ("bottom right corner, a little smaller"); the exact place and size are a **stand-in** (Q111 b) | `gui-4c7-bricks.log` "status icons at [752, 552, 1088, 600] of 800x600" |
| The meter bars and the empty green bar unchanged | `interface.md` rules 20, 22 and the layout; the user sees them differ (Q111 c, d) | — |

### 4c-8, the user's second check — `crates/kq8-core/src/{panels,gui_draw}.rs`, `crates/kq8-viewer/src/gui_view.rs`

| What | Source | Check |
| --- | --- | --- |
| ~~Space: `intrup.aud` as the bar shows, `intrdown.aud` as it hides~~ **Replaced**: `intrup.aud` as Space hides the bottom bar, `intrdown.aud` as it shows it (the top bar's stay as they were) | **The user's check in the original** (2026-10-18, second check; Q111 a) | Unit tests in `panels.rs`; `gui-4c8-bricks.log` "Space: intrup.aud" at 0.000 s (hide), "Space: intrdown.aud" at 1.101 s (show) |
| The spinning mask drawn whole at its pictures' own size (57 × 57, larger than its 48 × 48 layout box), its bottom right corner at the screen's; only the mask is drawn of the status icons | **The user's check** (it was cut off on its right, Q111 b); `mskicn.pba`'s size from the data | `gui-4c8-bricks.log` "picture mskicn.pba: 16 frame(s), 57x57", "spinning mask at [743, 543, 800, 600] of 800x600" |
| Black behind the health and experience bars (the part a bar doesn't cover is black, not see-through) | **The user's check** (Q111 d) | Unit test `pictures_fill_their_box…`; `gui.log` lists no see-through pixels in `IDCTL_METERS` any more |
| `gui.log` lists each picture's size and frames when loaded, and any see-through pixels of a drawn panel | **Our log** | `gui-4c8-bricks.log` |

### 4c-9, the gap beside the meters; the mask on the bar — `crates/kq8-core/src/gui_draw.rs`, `crates/kq8-viewer/src/{gui_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| A panel's picture is drawn where it stands: every edge (the panel's and its pieces') is rounded at its screen place, so the picture has exactly the rows and columns of its place (the meters: 466 to 480 × 1.25 = 582.5 to 600, 17 rows; they were drawn 18 and squeezed, which made the user's gap and its flicker while sliding) | **Our fix** (a drawing error of ours; the user's check found it) | Unit test `pictures_fill_their_box…` (`screen_size` 320 × 17); `gui-4c9-bricks.log` "engine: IDCTL_METERS … picture 320x17" (before: "picture 320x18" in a 20.4-pixel place) |
| The spinning mask stands on the bottom bar at the right and rises and falls with it (the screen's bottom when the bar is away) | **The user's check in the original** (as the caption, `interface.md` rule 51) | `gui-4c9-bricks.log` "spinning mask at [743, 465, 800, 522]" (on the bar, its top at 520) down to "[743, 543, 800, 600]" |
| `gui.log` lists where the engine put each bottom panel (physical pixels, after its rounding) and its picture's size | **Our log** | `gui-4c9-bricks.log` "engine: …" |

### 4c-10, the meters' gap for good; the mask not on the meters alone — `crates/kq8-viewer/src/{gui_view,talk_view,main,terrain_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| Each panel's picture and the caption's fill their box exactly (`NodeImageMode::Stretch`); Bevy's default kept the picture's proportions and centred it, so the 320 × 17 meters in a 384 × 20 box were drawn 376 wide, leaving 4 pixels of world at each end (and jumping as the box went 20 / 21 high while sliding: the flicker) | **Our fix** (an engine setting of ours; the user's check found it) | A screenshot read as numbers (temporary code, removed): before, the meters' first stone pixels at x 452 and world colours at 448–451 and 828–831; after, stone from 448 to 831, next to the side panels |
| The panels' edges are not smoothed by the engine (`UiAntiAlias::Off` on the camera) | **Our choice** (pixel pictures laid edge to edge; it only changed the slots' fractional bottom row, not the gap) | Same screenshots |
| The spinning mask stands on the bottom bar (bottom left, slots) but not on the meters when they pop up alone; the caption still does | **The user's check in the original** | `gui-4c10-bricks-bar-away.log`: the meters pop up at 1.287 s, the mask stays "at [743, 543, 800, 600]" |

### 4c-11, corner panels (an improvement) — `crates/kq8-viewer/src/{gui_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| With the option on, the bottom bar's left panel at the window's left edge, its right panel and the spinning mask at the window's right edge; the slots, meters, top bar and caption stay on the centred 4:3 screen; clicks follow the panels; `--corners on\|off` (default off), F6 | **Not in the original: the user's idea** (2026-10-18, "the third idea") | `gui-4c11-corners.log` (window 1280 × 720): "engine: IDCTL_INVBARL at left 0.00 … right 288.00", "IDCTL_INVBARR at left 992.00 … right 1280.00", the slots and meters at 448–832 |

### The world palette for the bars and captions (Q118) — `crates/kq8-viewer/src/{load,gui_view,talk_view}.rs`

Replaces the 4b-3 / 4c-2 guess `<folder>/resource/<folder>.ppl`, which found no palette in eight worlds, so their bars and caption strip were not drawn.

| What | Source | Check |
| --- | --- | --- |
| The bars, the caption strip and the fonts use the palette named by `set KQWorld::objectPalette` in the world's `world.cs` | `interface.md` rules 1, 1c, 1g | `viewer.log` per world, e.g. DeadCity "gui: … palette DeadCity/resource/newdead.ppl (world.cs: KQWorld::objectPalette newdead.ppl …)"; all 12 worlds found theirs |
| If that file is missing: `loadingPalette`, then `interiorPalette`, `terrainPalette`, `connorPalette` | Rule 1f (the panels keep the last palette that loaded) for `loadingPalette` and rule 1c ("may fall back to `loadingPalette`, then to the other three"); the order of the last three is **our choice** | Not tested (every shipped world has its `objectPalette`) |
| Each name looked up in the world's `resource` folder, then `GAME/Resource`; capitals ignored | Rule 1e | CastleD: `Castle.ppl` finds `castle.ppl` |

### The back flip at a wall (2026-10-18) — `crates/kq8-core/src/connor.rs`

| What | Source | Check |
| --- | --- | --- |
| Stopped by a wall while stepping back (movement `stop`, the back block on, so back still held), Jump starts `backJump`, not the standing jump | **The user's check in the original**; the rule behind it is a **stand-in** (Q112): `connor-physics.md` "Jumping" rule 2 alone gives the standing jump | Unit test `a_wall_stop_blocks_until_the_key_is_released` (extended); `connor-backflip-wall.log`: "head-on into a wall: movement back -> stop; blocked: back" at update 7, "jump: backJump from stop" at 26, glancing up the Pub's wall to z 10,316, `backJumpEnd` at 46 |

## Closing M3: the answers to Q27, Q53, Q100, Q102 (M3 step 3o)

### 3o-1, the view heading — `crates/kq8-core/src/camera.rs`, `crates/kq8-viewer/src/game_camera.rs`

This replaces three stand-ins: the heading kept after `move KQCamera` while following (3n-3), the viewer's own turn back after `follow` (3l-3), and "a `follow` 0.4 rad or less off looks at Connor at once".

| What | Source | Check |
| --- | --- | --- |
| The camera's position never swings behind Connor; only the view's heading turns, by four states (idle, catching up, locked, turning back), every camera update | `camera.md` "Turning the view toward Connor" (Q27 c, Q100 a) | `camera-3o1-side.log` |
| Error = angle from the view heading to the direction to the aim point (+ the mouse offset), wrapped; positive = to the left | Same, "Words used here" | Unit tests below |
| Idle → catching up when the error is over 0.25 and Connor moves, or over 0.5 | Rule 1 | Unit test `the_view_holds_until_a_quarter_radian_then_catches_up_and_locks`; `camera-3o1-side.log` update 19: "idle->catching_up error +0.258" |
| Rule 1 is applied only from idle | **Stand-in** (Q113 a: the worked mill wheel example has a locked view stay locked) | — |
| Catching up: only while he moves or the error is 0.5 or more and he hasn't just switched forward/backward (else idle); under 0.02 → locked; over 0.5 → cut to 0.25 on the same side; otherwise, only when the error grew, a turn of 1.25 × the growth | Rule 4 | Unit tests `the_view_holds_…`, `a_big_error_is_cut_to_a_quarter_radian_at_once_then_held`; `camera-3o1-side.log` updates 19–95, locked at 96 |
| Locked: exactly on the aim point + offset; unlocked (idle) when no drag, switch off, and he is still or just switched; the exact aim comes first | Rule 4; the order is a **stand-in** (Q113 b) | Unit tests `the_view_holds_…` (unlocks on the 3rd still update), `stepping_back_while_locked_unlocks_the_view` |
| A drag: locked with the offset that keeps the present angle (taken after the swing of that update); after the drag: offset 0, idle | Rule 2 | Unit test `a_drag_locks_the_view_at_its_angle_and_the_end_of_it_goes_idle` |
| The "turn back" switch: offset 0, state turning back (or stays locked); a third of the rest per update, one snap angle 0.063 on both sides; under 0.02 → locked | Rules 3–4, "Do not reproduce" (lopsided snap) | Unit test `the_turn_back_switch_turns_a_third_per_update_then_locks_and_stays` |
| "Switched": the move since the previous camera update against the facing then; more than 0.1 of its length along his back = backward; measured only while catching up or locked and moving | "Switched between going forward and going backward" | Unit test `stepping_back_while_locked_unlocks_the_view` |
| "Error at the end of the previous update" is measured after that update's turn | Rule 5 | `camera-3o1-side.log` (each "left" is the next "error" minus his drift) |
| Flights and cuts keep their own aiming (3m-4) and write the heading the follow view goes on from; `move KQCamera` writes it too; the state is not changed by any of them | **Stand-in** (Q113 d) | `camera-3o1-watermill.log`: flight unchanged, cue 6 `follow` leaves the view 0.121 off, idle |
| The view is drawn from the camera's own heading and tilt (blended between updates, the heading the short way round), no longer "look at the aim point, then turn" | **Our method** | Viewer runs above |
| At the start-up update the view first looks straight at Connor | **Stand-in** (part of the Q27 d start stand-in; `--at` moves him after the camera is made) | `camera-3o1-side.log` update 1: error 0.000 |

### 3o-2, the tilt and the aim height — `crates/kq8-core/src/{camera,camera_flight}.rs`, `crates/kq8-viewer/src/game_camera.rs`

This replaces the stand-in "the view looks straight at the aim point, pitched by the tilt's difference from the default" and the hand-set aim height of 800.

| What | Source | Check |
| --- | --- | --- |
| The aim point is 1,400 above the followed object's base (back from the hand-set 800; the leash, the height rule and the view test use it) | `camera.md` "What the camera looks at" rule 2 | Unit tests (all camera tests now at 1,400) |
| The view tilt = the player's tilt + the angle up from the camera to the aim point, set every update; the player's tilt is −0.2 by default (negative looks down), Page Up adds 0.015 per update | Rule 3 (Q27 b), "Tilt" rules 1–2 | Unit test `the_view_looks_0_2_below_the_aim_point_and_passes_connor_at_his_chest`; `camera-3o2-wheel.log` update 18: −0.832 (the note: about −0.83) |
| One default tilt and one aim height for the follow camera and the flights | **Our method** (same numbers in both rules) | `the_watermill_flight` unchanged; `camera-3o2-watermill.log` flight 31/31 as before |
| `camera.log` logs where the view's centre line passes Connor (height above his feet) | **Our log** (the user's check of the hips) | `camera-3o2-walkzone.log`: 893 at 2,499 away; `camera-3o2-walk.log`: 782 at 3,046, 390 at 4,999 |

### 3o-3, the mouse — `crates/kq8-core/src/camera.rs`, `crates/kq8-viewer/src/game_camera.rs`

This replaces the hand-set "800 pixels = one mouse unit".

| What | Source | Check |
| --- | --- | --- |
| One mouse unit = one window width sideways (a swing of 3π/4) and one window height forward/back (a zoom of 5,000) | `camera.md` "Swinging and zooming" rule 1 (Q27 a) | — (not tested in play: needs the real mouse) |
| The movement is the mouse's own (raw) movement in window pixels, not the cursor's place, so a drag doesn't stop at the window's edge | **Our method** (the original reads the cursor's place; its recentring at 0.9 / 0.5, rule 6, is not settled and not built) | — |
| A mouse movement during a drag is a cause for a camera update; holding the button still is not | Rule 6, "When the camera updates at all" item 5; one update per game update with the movement since the last (**our method**, as the original at 15 frames per second) | `camera-3o3-walkzone.log` identical to `camera-3o2-walkzone.log` (no mouse) |
| Held without a change: the offset from the aim point set back to the previous update's (before the leash, the height and the obstacle rules) | Rule 6 | Unit test `holding_the_button_still_keeps_the_camera_s_place_relative_to_connor` |
| After `move KQCamera`, a flight or a cut, the previous offset is forgotten (holding the button then doesn't pull the camera back) | **Our choice** (the note's rule assumes the previous offset is the follow camera's own) | Same unit test |
| Directions: mouse left swings the camera left, mouse down zooms in; with Shift, mouse down tilts like Page Down | Rule 7 (the user's check in the original) | As before |

### 3o-4, the view line against the ground — `crates/kq8-core/src/camera.rs`, `crates/kq8-viewer/src/game_camera.rs`

This replaces "stepped every 50 units, halved 20 times; only a line going from above to below counts".

| What | Source | Check |
| --- | --- | --- |
| Exact, per triangle: the line is cut at every square edge and diagonal it crosses (the chessboard's diagonals all lie on x − y or x + y = an even number of squares) and at its end; the first cut on the other side of the drawn ground than the start gives the triangle; the hit is the exact crossing with that triangle's plane | `camera.md` "Obstacles" item 1 (Q53 a); triangles from `terrain.md` (as `Terrain::height_at`) | Unit test `the_ground_test_is_exact_per_triangle_and_any_change_of_side_counts` (hits lie on the ground; nothing nearer) |
| Any change of side counts: a line that starts under the surface is blocked where it comes up out of it | Same (Q53 a) | Same unit test |
| Hole squares passed through | Same; **not built**: no holes are known in our data (`terrain.md` open question 3) | — |
| Points outside the grid are skipped (no ground there) | **Our method** (the note doesn't cover lines leaving the grid) | Same unit test |
| The deep lift keeps the old stepped floor-grid test until 3o-5 | **Kept for one step** (its rule changes in 3o-5) | — |

### 3o-5, small camera points (Q100 b, c) — `crates/kq8-core/src/{camera,camera_flight}.rs`, `crates/kq8-viewer/src/{game_camera,door_view}.rs`

This replaces two stand-ins of 3n: the deep lift's line against the floor grid ("no hit: ground + 1,400") and "one `follow` finding both conditions leaves the switch on".

| What | Source | Check |
| --- | --- | --- |
| The deep lift (gap over the range's maximum): the line from the update's start to the new spot is tested against the **drawn surface** with the view test's exact ground test; the camera is put **on** the hit point; the wanted height is then the floor grid's there + 1,400 | `camera.md` "Height above the ground" rule 2.1 (Q100 b) | Unit test `height_eases_from_the_start_of_the_update_and_lifts_along_the_line_when_deep` |
| The line meets nothing: the camera stays at its deep spot, "lifted" still remembered | Same | Same unit test |
| The same for a flight's last update and a cut (they use the same height rule) | Same, "Camera flights and cuts" rules 3–4 | Flight unit tests unchanged; `camera-3o5-watermill.log` flight 31/31 as before |
| One `follow` finding the view over 0.4 rad off **and** the camera beyond the maximum: the switch ends **off** (the 0.4 test first, then the distance test); neither: left as it was | "When the camera updates at all" item 3 (Q100 c) | Unit test `one_follow_finding_both_leaves_the_turn_back_switch_off` |

### 3o-6, a script's `move` and the trap he was in (Q102 b) — `crates/kq8-core/src/{trap,active}.rs`, `crates/kq8-viewer/src/door_view.rs`

This replaces the stand-in "the old trap gets `Off` at his next move".

| What | Source | Check |
| --- | --- | --- |
| When a script's `move Connor` is applied, every trap he is inside runs its stay test (rule 7) at the new spot: `Off` (or `Cue 302`) then, in the same update | `traps.md` rule 5 "Placed by a script" (b) (Q102 b) | Unit tests `placed_by_a_script_only_the_stay_test_runs`, `a_script_s_move_out_of_a_trap_gives_off_at_once`; `traps-3o6-wheel2.log`: "wheeltp2: Off Connor (stay test right after a script's move, rule 5)" in the tick of cue 2 |
| No trap is entered by the move; his next ordinary move's sweep starts from the new spot | Rule 5 (a) (as built before) | Same unit tests |
| In our build the stay test runs when the viewer applies the move, after all the scripts of that update (the note: right after the script that did it) | **Our order** (requests are applied after the update's scripts; what the `Off` script asks waits for the next update) | — (no visible difference in Daventry: `wheeltp2.cs` does nothing on `Off`) |

### 3o-7, the right button (the user's first check) — `crates/kq8-core/src/camera.rs`, `crates/kq8-viewer/src/game_camera.rs`

| What | Source | Check |
| --- | --- | --- |
| "Dragging" (heading rule 2) = the right button held, moving or not: locked at Connor's present angle on screen while held; letting go: offset 0, idle (he may drift) | **The user's check in the original** (2026-10-19: a click alone lets him drift afterwards; held still he keeps his spot on screen); fits `camera.md` rule 1 of "Swinging and zooming" (the mouse value is the cursor's place, so not 0 while held); our 3o-1 read "not 0" as "moved" | Unit test `a_drag_locks_the_view_at_its_angle_and_the_end_of_it_goes_idle` (rewritten); asked A to confirm in Q113 e |
| The angle kept is measured before that update's swing | **The user's check** (a fast drag only rotates round Connor); the note's "keeps its present angle" | Same unit test (half a window per update: Connor stays 0.1 off) |
| Pressing and letting go of the button each run a camera update | **Our method** (so the lock starts and ends at the click, not at Connor's next move) | — (needs the real mouse) |

## Characters (M4 step 4d) — `docs/behaviour/characters.md`

### 4d-1, reading characters — `crates/kq8-data/src/characters.rs`, `crates/kq8-cli/src/character_verify.rs`

| What | Source | Check |
| --- | --- | --- |
| The character classes `KQMonster` and `KQGeometryTalker` | `characters.md` rule 1 | `character-verify`: Daventry 22 + 2 files, as the note; all worlds 154 + 7 (the note says 155 `KQMonster`: one is not in a world's archives as a `.kq` with that first section; not looked for) |
| Monster-list line: 20 tab-separated fields in the order of rule 3; `@` = the type's value; quotes dropped; `type(a.bmp,b.bmp)` = skin swap; z −1 = ground; movement `none` / `loop n …` / `patrol n …` / `random` | Rules 3, 4; field order also in the file's own `;monsterX=` header line (data) | Unit tests `monster_lines_are_read_field_by_field`, `bad_lines_are_reported`; Daventry 52 lines, 0 problems |
| Type list `Monstats.mtl`: 15 fields, `KQFile` the 7th | Rule 4; field order from the file's `;typeX=` header (data) | Unit test `types_name_their_files`; 81 types, 0 problems |
| A type name matched exactly, then ignoring capitals | **Our choice** (the note does not say); counted in the log | Daventry needs none; Swamp 1, DeadCity 9, temple3 2 match only ignoring capitals (later worlds, M6) |
| Key defaults: `flags` 0x10, gravity on, `wanders` 1, `shadowMask` 2, `birthDelay` 0, `noTurn` 0 | Rules 6, 8, 13, 36, "Numbers and units" | Unit test `keys_have_the_note_defaults` |
| `lips=` → `KQLips` section in the same file, face = `baseLip` else `lip0`, shape 8+ → `lip0` | Rule 25 | Unit test `lips_give_the_face_and_shapes`; Daventry's Wizard, Girl, Lady: the face texture is in their `Stop` model |
| `.anm` list read like `Conner.anm` (first section = the list); looked up in the world's archives, else `GAME/Kq` | Rule 6; the search order is the one 3i-7 uses for `AnimList=` files (**our choice**, not in the note) | Unit test `anm_lists_give_movements_and_the_current_one`; frame counts as the note: `wizstop` 31, `chk_stop` 30, `UG_STOP` 40, `Ravnstp` 46 |
| Box: `BOUNDS.KQ` by type / `monsterType`, else by name, exact | Rule 9 | All 52 Daventry lines find a box (the Knight by name) |
| Geometry mouth movement names; gesture names | Rules 27, 28 | Daventry's two geometry talkers have all 8 mouth movements |

### 4d-2, characters drawn where they stand — `crates/kq8-viewer/src/{load,anim_view,draw_distance,terrain_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| `loadKQ monsters.kq`: every line places its type's file (world archives), renamed, the line's position and heading; an unknown type makes nothing | `characters.md` rules 3, 4 | `characters.log`: Daventry 52 of 52 lines placed |
| A line's z −1 is put on the floor grid like a `loadKQ` −1; any other z (the chickens' and bats' −2) as written, then `keepOnGround` | Rule 4, "Edge cases" | `characters.log`: `chicken1` at z 13,932 (raised), `bat2` at 6,790 (as written) |
| The `.kq` file's own position, `dirX`/`dirY` kept for a list line; only `dirZ` replaced | Rule 4 ("the position and heading replace those of the file") | — |
| A character's `animFile` list attached as if the file carried it; every movement drawn as for animated objects, one model per movement | Rules 2, 6 | 57 characters in `characters.log`, 0 markers left for characters |
| Characters whose list has `Birth` and no `PreBirth`: not drawn (only with H), groups `groupsAlways` + 1 only | Rule 16 | 21 hidden in Daventry: all the zombies |
| Draw-distance ball from the box by type (list) or `monsterType`, else by name, exact; no box: radius 100 (as for objects) | Rules 9, 33; radius 100 from `object-lighting.md` rule 35 | `characters.log`: the Shadow has no box (radius 100) |
| Skin swap: the texture named first is drawn as the second, names compared ignoring capitals | Rule 4; capitals: **our reading**, as rule 25 infers for the mouth | `viewer.log`: `test2.bmp` drawn as `test3.bmp` in 17 loads (the boss spriggan's models) |
| Characters are clickable when their type has bit 0x20, with the rule-9 box; never solid for Connor yet | `clicking.md` / `doors.md` rule 10 for clicks; rule 9 for the box; not solid: **our stand-in** until M5 (rule 9: ball stacks) | Daventry: Girl, Wizard, Ravenw, Knight, Uglybeast clickable; `--click Uglybeast`: `ugly.cs` plays `pivot`, at its end `move Uglybeast 3` and `stand` |
| `KQGeometryTalker`s count as `KQMonster`s for `speak` | Rule 1 ("a `KQMonster` in every way") | — |
| A drawn animated object follows its place in core (a script's `move`); until now only its collider moved | `objects.md` Q20 d (a `move` moves the object) | `--click Uglybeast`: the body moves with `move Uglybeast 3` (log; not seen by eye) |
| Characters loaded while playing (the Lady, three ravens, the Unicorn) are prepared like other dormant objects (3k-4) | As 3k-4 | Not tested: no run reaches their scripts |

### 4d-3, the states — `crates/kq8-core/src/{character,active,named_anim}.rs`, `crates/kq8-viewer/src/{anim_view,draw_distance,door_view}.rs`

| What | Source | Check |
| --- | --- | --- |
| A character updates only while near or far (not off, not hidden); the viewer's draw-distance test tells core each frame | `characters.md` rule 11 | Unit test `nothing_happens_while_off`; `characters.log` "by state": 52 "not updated yet" at the Wizard's house |
| First update: a waiting first movement with no current name starts, no state; else movement 0 (or `currAnimName`) **starts** (so it has a name), then Birth or Wait after `birthDelay` | Rules 12, 14, 15 | Unit tests `first_update_then_wait_asks_for_stop_when_movement_0_ends`, `a_waiting_first_movement_means_no_state`, `birth_list_means_birth_state`, `birth_delay_puts_off_wait`; Shadow: rule 12.1 |
| Wait through the queue: at the next update at the earliest (delay 0) | Rule 12.2 ("at once, through the event queue") | Wizard: `attack` at update 0, `Stop` at update 1 |
| Wait asks for `Stop` the ordinary way; random frame and speed 0.9–1.1 with `flags` bit 0x10, also when it starts later at the end of movement 0 | Rule 13 | Ravenw: `Stop` at update 44 at random frame 40; chickens out of step |
| In Wait, any one-shot ending (not `Stop`, not `gesture…`) → `Stop` again, always random | Rule 18 | Unit test `in_wait_a_one_shot_is_followed_by_stop_but_not_a_gesture` |
| `setState` by name: Wait, Special (words joined, then items one at a time, then Wait), Unborn; Move / Die logged (M5); Connor-only states logged and ignored; unknown "cannot find state" | Rule 22, "Do not reproduce" | Unit tests `special_lists_play_in_turn_then_wait`, `other_states_are_logged` |
| `AnimList=`, `Inv`, `KillConnor` in a character's Special list: skipped and logged | **Our stand-in** (none in Daventry) | — |
| The state-allowed check ("only if the current one allows it") | Not built: only Talk's list is given and there is no Talk state yet | — |
| A state set before the first update is kept (no restart, no Wait) | **Stand-in**, asked as Q121 (`sprdump`) | Unit test `a_state_set_before_the_first_update_is_kept` |
| `setUseGravity`, `setMonUseGravity`, `setCanTurn` stored; `changeAnmFile` logged | Rules 8, 22, 23 (used from 4d-4 / 4d-6); `changeAnmFile` open question 4 | — |
| Each character's random draws from its own generator seeded by its name | **Our choice** (the original's random source is not described) | Same numbers every run |
| A dormant character (loaded by a script while playing) gets its draw-distance test when it comes in | As 3k-4 | Not tested (no run reaches those scripts) |

### 4d-4, turning — `crates/kq8-core/src/{character,active}.rs`

| What | Source | Check |
| --- | --- | --- |
| `speak` with a listener turns the issuing character toward the listener's point; `noTurn` (and `setCanTurn 0`) ignores the request | `characters.md` rule 23; `talking.md` rule 7 (monsters always) | Unit test `no_turn_blocks_turns`; viewer: the Uglybeast's "too far" line: "blocked: noTurn" |
| 3.5 rad/s the short way, landing exactly when less than one update's turn is left; a newer request replaces the target | Rule 23 (Connor's automatic turn, `connor_auto.rs`) | Unit test `a_turn_goes_at_3_5_per_second_and_lands_exactly` (a quarter turn in 7 updates) |
| No turning movement: the current movement goes on | Rule 23 | Same unit test (`stop` keeps playing) |
| The heading is where the model's −Y points, as Connor's; the turn is about the world's z, after the rotation it has (a script's `move` turn kept) | **Our reading**: rule 23 says "the same automatic turn as Connor's"; the models' forward axis is not stated for characters | Unit test `heading_of_matches_dir_z`; not seen by eye |
| The turn only runs while the character updates (near or far) | Rule 11 | — |
| Talk state's turn every 4th update | Rule 24, `conversation.md` rule 17: not built (no conversations yet) | — |
| A clickable character's click box keeps the rotation it was loaded with | **Not done** in 4d-4; done in 4d-5c (the box follows the character) | — |

### 4d-5, the face and the hands — `crates/kq8-core/src/{character,active}.rs`, `crates/kq8-viewer/src/{anim_view,load,door_view,main}.rs`

| What | Source | Check |
| --- | --- | --- |
| Codes 0–7: mouth shape; the character's own copies of every material with its face texture (`baseLip`, else `lip0`) get `lip<n>`; kept across movements; 0 at a line's end | `characters.md` rule 25 | `characters.log` (Wizard, `--command`): "lip6 drawn on 15 face material(s)" … "lip0" at the end |
| Face texture names compared ignoring capitals, by file name | Rule 25 ("inferred") | Wizard `WF0.bmp`, 15 materials found |
| Geometry talker: 0/16 `rest`, 1–7 `f m r w ih ah e`, started at once (not a request); 8–15 and above 16 ignored | Rule 27, "Do not reproduce" | Unit test `a_geometry_talker_starts_mouth_movements_at_once`; Uglybeast: 16 codes, each a movement |
| Codes 8–15 `gesturea`…`gestureh`, 16 `stop`, asked the ordinary way, only when the movement playing can be cut short or has ended and the list has it; nobody walks yet; above 16 nothing; the mouth unchanged | Rule 28 (Q115), `conversation.md` rule 18 | Unit test `mouth_codes_shape_gesture_or_nothing`; Wizard line 1000/4/0/8: `gesturee`, `gesturea`, `gesturee`, `gesturec`, then code 16 `stop` |
| After the last line of a queue the speaker goes to Wait, from no state or Wait only | Rules 15, 27; **stand-in** for the details (Q117) | Wizard: "last line of its queue: the speaker goes to Wait" at update 154 |
| Mouth movements in Wait bring `Stop` back (rule 18 read literally) | Rule 18; asked as Q120 | Uglybeast: `f` at 67, `Stop` at 68 |
| `--command "WORDS",N` test option | **Our test tool** (not a game rule) | Used for the runs above |
| Connor's own codes 8–16 (Q114: no mouth change, gestures) | Not changed in this step: Connor's mouth is `talking.md`'s (4a-6); left for the conversations step | — |

### 4d-5b, `makeTranslucent` — `crates/kq8-core/src/active.rs`, `crates/kq8-viewer/src/anim_view.rs`

| What | Source | Check |
| --- | --- | --- |
| `makeTranslucent <name> <n>`: at once, no fade; every material of every part at opacity n / 255; 256 = solid | `characters.md` rule 22; `objects.md` "See-through objects" rule 2 | Unit test `make_translucent_sets_the_value_on_an_animated_object`; viewer `--command "makeTranslucent Wizard 150",20 --command "makeTranslucent Wizard 256",60`: 3,347 parts at 0.588, then solid |
| Only for animated objects and characters; others (Connor in Dead City) logged as not done | **Our scope** (Daventry only uses it on the Wizard) | Unit test (an unknown name) |
| The object's own material copies; the mouth's copies kept, so the mouth still changes | **Our method** | `characters.log`: 64 own materials |

### 4d-5c, a character's click box follows it — `crates/kq8-core/src/active.rs`

| What | Source | Check |
| --- | --- | --- |
| A clickable character's click box is placed where the character is drawn, every update (after a script's `move` or a turn) | `doors.md` rule 10 / `clicking.md`: an object is picked by its box at its place; the box is the character's (`characters.md` rule 9) | Unit test `a_character_s_click_box_follows_it` |

### 4d-6a and 4d-7a, measurements and the shadow rule — `crates/kq8-viewer/src/load.rs`, `crates/kq8-data/src/characters.rs`

| What | Source | Check |
| --- | --- | --- |
| "gravity check": each character's z against the floor and surface grids under it (a log only; no gravity built) | `characters.md` rule 8; **measurement for Q122** | `characters.log`: 46 of 57 on the floor grid |
| Casts a shadow when (world mask AND `shadowMask`) is not 0, `allowShadows` is on and `translucency` is 256 | Rule 36 | Unit test `shadows_follow_the_masks_and_solidity`; Daventry: 51 cast, not the chickens, the pig, the Knight, the Shadow (as the note) |
| World mask: the last `setShadowMask` in `MASK/Options.cs` (15), else the built-in 1 | Rule 36 ("`MASK/Options.cs` sets 15 at start") | Same unit test (Options.cs-like text) |
| `allowShadows` taken as on at load (its default; Daventry's `world.cs` sets 1); not changed while playing | Rule 36 (default on); **not read from scripts yet** | — |
| Shadows not drawn | Waits for Q123 | — |

### Q114, the corrected `talking.md` rules 9 and 17–18 — `crates/kq8-core/src/talk.rs`, `crates/kq8-viewer/src/talk_view.rs`

| What | Source | Check |
| --- | --- | --- |
| The duplicate check is on the range's **last** line (after 0 is replaced): nothing of the range is added when that line is waiting or playing | `talking.md` rule 9 (corrected, Q114) | Unit tests `the_duplicate_check_is_on_the_range_s_last_line`, `lines_play_in_turn_for_their_voice_length_and_duplicates_are_dropped` (changed: 1…2 during line 2) |
| Connor's lip-sync codes above 7 don't change his mouth (8–16 are gestures, above 16 nothing); was: `lip0` | Rules 17, 18 (corrected) | Not tested in the viewer (no reachable Connor line with such codes found); logged in `connor.log` when it happens |
| Connor's gestures (codes 8–16) | `conversation.md` rule 18: not built; left for the conversations step | — |

## M4 4e, conversations

From `docs/behaviour/conversation.md` (session A, 2026-10-19), unless
marked otherwise.

### 4e-1, reading — `crates/kq8-data/src/conv_info.rs`, `characters.rs`, `crates/kq8-cli/src/conv_verify.rs`

| What | Source | Check |
| --- | --- | --- |
| `KQConvInfo` section `<partner>Info`: `convLocX/Y/Z`, `nBox` + `box<n>` (4 corners x y), `nCam` + `cam<n>` (x y z rx ry rz); `box0` start, `box1` stay, `box2`… special areas | Rule 3; **Observed** in `wizinfo.kq`, `uglyinfo.kq` (the file's own comments name the boxes the same way) | Unit test `reads_the_spot_areas_and_cameras`; `kq8 conv-verify`: Wizard 3 areas 7 cameras, Uglybeast 2 and 7 |
| A bad or missing `box<n>` / `cam<n>` ends its list (logged) | **Our choice** (not in the data) | Unit test `other_classes_and_bad_lists` |
| Inside: the corners' angles seen from the point add up to 6.1–6.4 rad; on a corner = outside; corner order doesn't matter | Rule 3 | Unit test `inside_by_the_angle_sum_corners_outside_order_free`; the note's three Daventry points land in the note's areas |
| `GBMsg…GBSeq`, `WBMsg…WBSeq`: a missing key 0; file and noun both 0 = no line | Rule 20 | Unit test `goodbye_and_welcome_back_keys`; `conv-verify`: 19 characters |
| `conv-verify`: `$1` in a script is the object whose `script=` is that file, the one with an info object first (`wizard.cs` is also `magicmap`'s) | **Our method** for the count | Every partner's largest camera number below its `nCam` |

### 4e-2, the conversation queue — `crates/kq8-core/src/talk.rs`

| What | Source | Check |
| --- | --- | --- |
| A second queue per partner (`<name>Conv`), issuer the partner; lines play as `speak` lines; `TalkComplete` from the partner | Rules 2, 12 | Unit test `a_conversation_plays_its_lines_with_cameras_and_reports_its_end` |
| A line's `[n]` reported when it starts, conversation lines only | Rule 14 | Same test (a `speak` line of the same tuple reports none) |
| "The conversation's last line": a conversation line after which nothing waits in its queue | Rule 15 (**our reading** of "last line" when a welcome back and older lines are in the queue) | Same test |
| Goodbye: with the line playing and at least one waiting, and a goodbye line, it goes in front and starts at once, cutting the line playing (back to the head of the waiting lines, no `TalkComplete` for it); at its end captions off, sounds up, the queue pauses, the caller told (`TalkExit`); no `TalkComplete` | Rule 22; "Edge cases" (the cut line: treat as cut) | Unit test `walking_away_plays_the_goodbye_and_coming_back_the_welcome_back` |
| A goodbye whose line can't be played pauses the queue at once | **Our choice** (not in the note) | — |
| Told to start while paused with lines waiting: welcome back in front, then the waiting lines, then the new ones with the duplicate check; a goodbye not played yet dropped | Rule 21 | Same test: welcome back, line 2 from its start, line 3; the range not added twice |
| Walking away on the last line: no goodbye, the line plays on with its `TalkComplete` | Rule 22 | Unit test `no_goodbye_on_the_last_line_and_clear_speak_empties_the_conversation` |
| `clearSpeak` on the conversation queue: emptied at once, no `TalkComplete`, pause forgotten | Rule 23 | Same test |

### 4e-3, the command and the two talk states — `crates/kq8-core/src/conversation.rs`, `connor.rs`, `character.rs`, `active.rs`; viewer `door_view.rs`, `load.rs`

| What | Source | Check |
| --- | --- | --- |
| `KQConner::conversation`: 9 words or the usage text; the partner must be a `KQMonster` (or geometry talker); `<last>` 0 = the run; `<enemy>` ≠ 0 keeps the weapon | Rule 1 | Unit test `a_missing_info_object_and_no_case_are_treated_as_there` (bad commands); Wizard and Uglybeast runs |
| At once: partner's `speak` queue emptied, conversation queue made with GB/WB, partner's movement made cuttable + `Stop` + Talk; Connor "sent the start message", handled after the update's scripts | Rule 2 | `conversation.log` updates 30/31 of the Wizard run |
| The start message waits while his jump is unfinished (tried again each update) | `anim-scripts.md` rule 28 used for this state too; **stand-in** (open question 3 of the note) | Unit test (`Err("jumping")`) |
| Entering: Connor's `speak` queue emptied; the bottom bar away (the panels' own handsOff rule); the case: stay area and under 125 (3D) "there", stay area "close", start area "walk" | Rules 2, 6; `interface.md` rule 16 | Unit test `the_case_comes_from_the_areas_and_the_distance`; Wizard from (193000, 198000): 834, "close"; Uglybeast: 1, "there" |
| "Close": put on the spot as `move Connor 2` (heading kept) | Rule 9 | Placed at (192171, 198091, 6700) as the note |
| "Walk": automatic turn toward the spot, `stopwalk`; at its end put 440 short on the line from where he stood, at the spot's height, facing it, `walkstopright`, partner told to start; at its end the automatic turn to the partner | Rule 10 | Unit test `the_walk_in_puts_him_440_short_facing_the_spot`; viewer from (195000, 200000): placed (192536, 198337), `walkstopright` ends 29 from the spot, turns to −1.725 |
| The keys are ignored during the approach movements; no Jump (not "in the keys' control") and no push (only Wait/Move/Push accept one) in the talk state | **Stand-ins (Q125)** | — |
| No weapons: `away` / `out` never play | Rule 7; weapons are M5 | Logged "none drawn" |
| First person not built: nothing to switch or restore | Rules 6, 15 | Logged |
| Stay test every update once the spot is reached; outside `box1` = walked away, Connor leaves at once | Rule 13 | Unit test `walking_out_of_the_stay_area_ends_it_and_sends_talk_exit_without_a_goodbye`; viewer: out at update 47 |
| The end: Move if moving, else `Stop` and Wait; camera to `cam0` looking at the partner, then `follow Connor`; partner to Wait; bar back; `TalkExit` before the last line's `TalkComplete` | Rules 15, 16 | Unit test `a_conversation_enters_plays_and_ends_with_talk_exit_before_the_last_talk_complete`; Wizard: update 1694; Uglybeast: update 1094 (`TalkExit`, its cue 2 in 0.5 s, then `TalkComplete 6 11`, as the note) |
| The camera looks at the partner's position + 1,400 | **Stand-in (Q126)** | Logged |
| Connor's state changes one step after the core decides (this update, before his next) | **Our order** | — |
| No info object or no case: "there" (no cameras; no stay test without an info object); walking away with no goodbye: `TalkExit` anyway | "Do not reproduce" | Unit tests |
| Partner's Talk: every 4th update a turn toward Connor's current position unless `noTurn`; no return to `Stop`; Wait at the end | Rule 17; `characters.md` rules 19, 24 | Unit test `in_talk_it_turns_toward_connor_every_4th_update_and_wait_ends_it`; Wizard and Uglybeast: "noTurn" |
| `speak` of Connor or the partner ignored while either is in its talk state; keys and clicks don't cut lines short | `talking.md` rule 8; rule 12 | Unit test (`speak_ignored`) |
| `clearSpeak <partner>` empties its conversation queue too; partner to Wait; Connor's mouth to rest, his notify group `TalkComplete 0 0 0 0 0`; his talk state goes on | Rule 23 | Not run (the ring, candle, ashes and horn scenes need items and the story) |
| Info objects read from the world's `loadKQ` (no model); a character's GB/WB keys on its `Monster` | Rule 3; data (`permwrld.cs` lines 374–376) | `viewer.log`: "2 conversation info object(s)" |

### 4e-4, clicks — `crates/kq8-core/src/active.rs` (`click`, `click_partner`)

| What | Source | Check |
| --- | --- | --- |
| A character with an info object: no reach test; special area (`box2`…) → `ConnorAction Special Special`; start area → in reach (`do do`, or the held item through the accept list); else `TooFar TooFar`, `tooFarAwayAction` not looked at; then the click turn | Rule 5 | Unit test `a_click_on_a_partner_goes_by_the_areas_not_the_reach`; viewer: the note's three Daventry points give `do`, `Special` ("Come around, lad…"), `TooFar` ("Hello, there, sir wizard!") |
| In the talk state a world click does nothing, except on the partner with an item: `InvConv <item type>` to its script, Connor leaves (with `TalkExit`), then the ordinary click; the queue is not emptied | Rule 19 | Unit test `during_the_talk_state_only_an_item_on_the_partner_counts`; viewer with `--item INVITEM_SilverCoins`: `InvConv`, `TalkExit`, `ConnorAction TestItem INVITEM_SilverCoins`, conversation 8/86 queued behind 13 waiting lines |

### 4e-5, conversation cameras — `crates/kq8-core/src/conversation.rs`, `active.rs`; viewer `game_camera.rs`, `door_view.rs`

| What | Source | Check |
| --- | --- | --- |
| A conversation line with `[n]` starting while Connor is in his talk state: once he has reached his spot he turns to face the partner; the camera stops following and cuts to `cam<n>` (as `move KQCamera 0 …`) | Rule 14 | Unit test `camera_numbers_cut_the_camera_until_the_right_button`; viewer: the Wizard's first meeting cuts to 2 3 3 5 2 1 1 1 1 3 5 2 3 6, the note's list |
| `n` not below `nCam`: logged and ignored | "Do not reproduce" (the original does not check) | Same unit test |
| The right button held during the stay test: the camera placed once at `cam0` aimed at the partner, swung and zoomed around it; camera numbers ignored for the rest of the conversation | Rule 14 (the user's check in the original) | Same unit test; viewer `--right 200`: at update 200 the camera at `cam0`, then cameras 3, 5, 2, 1 ignored |
| The swing around the partner is the following camera's with the partner's position in place of Connor's feet (range, height rule, obstacle check, aim 1,400 above); Connor's own moves don't move it; `follow` ends it | **Stand-in (Q126)** | `camera.log` "swings around (190807, 198274, 6630)" |
| `--right N`: the right button counts as held from update N on | **Our test tool** | Used above |
| The right button's lock released on entering the talk state | Rule 6 | **Not built** (our lock is the button itself; nothing to release when it is up) |
| Only this update's line starts are looked at | **Our method** (the event list is emptied by the viewer each update) | Unit test (it failed before the fix) |

### 4e-6, goodbye and welcome back in the viewer

| What | Source | Check |
| --- | --- | --- |
| Walk away with lines left: goodbye, pause, `TalkExit`; click again: welcome back, the cut line from its start, the rest; the range not added twice | Rules 21, 22; the note's "How to check" (Wizard) | Viewer run of 4e-6 (MODLOG): goodbye at 46–108, welcome back at 151–267, line 1 again at 267 |

### 4e-7, Connor's gestures — `crates/kq8-core/src/connor.rs`, `active.rs`; viewer `door_view.rs`, `load.rs`

| What | Source | Check |
| --- | --- | --- |
| Connor's codes 8–15 play `gesturea`…`gestureh`, 16 `stop`, at once from phase 0; the mouth unchanged; only when he stands still (Wait, or the talk state outside its approach; no start or stop movement playing), no weapon drawn, not in first person | Rule 18 | Unit test `gestures_play_only_while_he_stands`; viewer: line 1 of the first meeting plays `gesturee`, `gestured`, `gestureg`, `gesturef`, `stop` |
| "Stands still" read as: the keys choose and choose standing, and no transition plays | **Our reading** of rule 18 | Same test |
| His eight gesture movements loaded from `Conner.anm` | Data (`Conner.anm` Movement13–20) | `viewer.log` "connor: movement gesturea = movement 13 …" |

### 4e-8, the right-button camera's centre — `crates/kq8-core/src/conversation.rs`

| What | Source | Check |
| --- | --- | --- |
| With the right button held, the camera is aimed at and swings around the point half-way between the partner and Connor (where he stands when the button is pressed), 1,400 above | **The user's check in the original** (2026-10-19); the exact point is our stand-in (Q126) | Unit test `camera_numbers_cut_the_camera_until_the_right_button` |

### 4e-9, a 0 s loop never ends — `crates/kq8-core/src/door.rs`

| What | Source | Check |
| --- | --- | --- |
| A movement whose model loop lasts 0 s (or less) does not move and never ends, whatever its speed, loop or play-once: no `EndLoop`, Wait brings no `Stop` back | `anim-scripts.md` rule 10a (Q120) | Unit test `a_zero_length_mouth_pose_holds_in_wait` |

### 4e-10, delay-0 events before drawing — `crates/kq8-core/src/active.rs`

| What | Source | Check |
| --- | --- | --- |
| After the objects' update, the queue is handled again until nothing is due now; what those handlers send with delay 0 is handled in the same pass | `anim-scripts.md` rule 8 (Q119) | Unit test `what_an_end_loop_script_sends_with_delay_0_happens_before_drawing`; the four older timing tests changed |
| Scripts run after Connor's update (traps, `trap_step`): their delay-0 events wait for the start of the next update | **Our order**, not changed in this step | — |

### 4d-8, the names start as `Stop` — `crates/kq8-core/src/named_anim.rs`, `crates/kq8-viewer/src/anim_view.rs`, `door_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Animated objects and characters start with current and waiting movement `Stop`; `currAnimName` / `nextAnimName` replace them; the current movement is found by name, else movement 0 under that name | `anim-scripts.md` rule 3 (correction 2026-10-19), `characters.md` rules 12, 14 | Unit test `the_names_start_as_stop_q121`; `viewer.log` "names Stop" lines (Daventry: 55) |
| `flags=8` objects without `currAnimName` get the random start | `anim-scripts.md` rule 3 (correction) | `anim.log`: `fishies.01`–`.03` at random speeds |
| A play-once first movement that ends lets the waiting `Stop` start | `anim-scripts.md` rule 3 and 8.1 (`tombdoor`) | Same unit test |
| Doors and chests keep the old load names | **Our choice** until Q127 | — |

### 4d-9, a character's start-up — `crates/kq8-core/src/character.rs`, `active.rs`

| What | Source | Check |
| --- | --- | --- |
| The start-up runs once: at the first update, or before the first movement request (`KQAnimPlayEvent`, lip-sync code, a state's request, a conversation), whatever the range or `hidden` | `characters.md` rule 12 (Q121) | Unit test `the_start_up_runs_at_the_first_movement_request_whatever_the_range`; sprdump run |
| Only with no state: Birth, or Wait `birthDelay` s later in game time through the queue; a state set first is kept and nothing is queued | Rule 12 (Q116, Q121) | Unit tests `the_start_up_queues_wait_handled_before_the_first_picture`, `no_state_until_the_queued_wait`; goblin18 run (Wait at update 15) |
| The queued Wait is checked after each pass over the event queue (start and end of the update), for every character, near or not | Rule 12 ("through the event queue") with `anim-scripts.md` rule 8 (Q119); applying it out of range is **our reading** of "through the queue" | Same tests |
| The current movement is not restarted at the start-up: it was set at load and has not moved yet | Rule 12.1, with our load (4d-8) | — |
| Replaces the 4d-3 rows "a state set before the first update is kept" (Q121 stand-in) and the "empty name: no state" step | Rule 12 (corrected) | — |

### 4d-10, the end of a `speak` — `crates/kq8-core/src/character.rs`, `active.rs`

| What | Source | Check |
| --- | --- | --- |
| On the line marked last of a `speak` in a character's queue (for a conversation, the partner's), before its `TalkComplete`: in Wait nothing; otherwise `Stop` asked for and Wait, accepted from no state, Talk, and Special once the list's last movement started | `talking.md` rule 13a (Q117) | Unit test `the_end_of_a_speak_goes_to_wait_but_not_from_wait`; Uglybeast and Wizard runs |
| Birth, Move, Die: `Stop` asked for, state kept | **Our reading**: the note names only no state, Wait, Talk and Special | — |
| Wait entered through Wait's own start (rule 13's random `Stop` with `flags` 0x10) | **Our reading** of "`Stop` is asked for and it is put into Wait": the same picture either way | — |
| Replaces the 4d-5 row "after the last line of a queue the speaker goes to Wait" (Q117 stand-in) | Rule 13a | — |

### 4d-11, transitions on characters — `crates/kq8-core/src/named_anim.rs`, `door.rs`, viewer `anim_view.rs`, `door_view.rs`, `load.rs`

| What | Source | Check |
| --- | --- | --- |
| A request chooses the transition whose from point comes next (−1 first; passed on a play-once: never; tie: first listed); a newer request chooses again or drops it | `connor-movement.md` rules 2, 4 (Q40 b), `anim-scripts.md` rule 4.2 | Unit tests `a_transition_waits_for_its_from_point_then_its_target_starts`, `a_from_point_already_passed_on_a_play_once_is_never_reached` |
| The update is cut at the from point; the transition movement plays from 0 at speed 1, once; at its end the target starts at its phase, another wanted movement is asked for again | `connor-movement.md` rules 3, 5, 6 | Same tests |
| The end of the transition movement is an ordinary end (`EndLoop`, `Cue 300`, in Wait `Stop` again at a random frame) | `characters.md` rule 20 (Q124 a) | Unit test `in_wait_the_end_of_a_transition_restarts_stop_at_a_random_frame` |
| The transition and its target first move in the next update | As Connor's walker (`connor.rs`, rule 3 "first advanced in the next update"); for the target **our reading** (the rule does not say) | Same tests |
| Backwards: the distance measured the other way | `connor-movement.md` rule 2 ("the other way round"); no data has one | — |
| The `endLoop` cue not built | `characters.md` rule 20 (Q124 c: M5) | — |

### 4d-6b-1, the mover's switches — `crates/kq8-core/src/connor_mover.rs`

| What | Source | Check |
| --- | --- | --- |
| No Fall state and no "steep ground switches gravity on" without collision type bit 4 | `characters.md` rule 8, `connor-physics.md` "Other characters on the same mover" (Q122 d) | Unit test `a_character_falls_and_lands_with_no_fall_state_q122` |
| The terrain in the up and down passes only with mask bit 0x800 (`colliderMask=0`: only the ground check) | Rule 8 (Q122 a, b) | Unit test `mask_0_skips_the_terrain_in_the_passes_but_the_ground_check_catches_it` |
| `keepOnGround=1`: the ground check onto the water surface (flat), where it is above the floor | Rule 8, `water.md` | Unit test `keep_on_ground_1_stands_on_the_water_surface` |
| A character's ball stack from its box; steps 325 or its file's | Rule 8, rule 9 | Same tests |

### 4d-6b-2, characters fall — `crates/kq8-data/src/characters.rs`, `crates/kq8-core/src/character.rs`, `active.rs`, viewer `anim_view.rs`, `door_view.rs`, `draw_distance.rs`, `main.rs`

| What | Source | Check |
| --- | --- | --- |
| A character's type and mask: `colliderType` / `colliderMask`, else 0x42 / 0x281F | `characters.md` rule 8 (Q122 a) | Wizard's log line: terrain no, buildings no (mask 0) |
| Its mover: Connor's, with its `BOUNDS.KQ` box (by type, else name) as ball stack, steps 325 or its file's, `keepOnGround` | Rules 8, 9 | Windmill and ruins runs |
| Runs only while near or far, not in Birth; standing still; a script's `move` sets its start | Rules 8 (Q122 e), 11, 16 | Unit test `a_character_in_range_falls_onto_the_ground_and_says_so` |
| Buildings count when its mask has 0x8 (every building type in the data has it) | Rule 8 (pair rule); the building types are **data** (0x28, 0x88) | Ruins: lands on Castle |
| Objects, other characters, Connor: not tested | Waits for Q128 | Log line "not tested (Q128)" |
| The mover runs in a step right after the characters' own update, not inside it | **Our order** (the update's parts are the same) | — |

### 4d-6b-3, the flying mover — `crates/kq8-core/src/connor_mover.rs`, `character.rs`

| What | Source | Check |
| --- | --- | --- |
| `flyingObject=1`: steps 0, gravity off, the ordinary move, then down by up to 150 (stopped as the down pass), then up by up to 300 if the ground is above its feet within its height | `characters.md` rule 8 (Q122 c; the note: "not sure of every detail") | Unit test `a_flyer_settles_150_per_update_and_rises_up_to_300`; bat3 run |
| The raise tests the terrain only | **Our cut**: how a building surface "above its feet" is found is not described (Q129 b) | — |
| The ground check (in the ordinary move) comes before the drop, so over water a `keepOnGround=1` flyer sits 150 under the surface | Literal order of rule 8; asked as Q129 (a) | bat2 run: settles at 6,480 |

### 4d-7b-1, shadow geometry — `crates/kq8-core/src/shadow.rs`

| What | Source | Check |
| --- | --- | --- |
| Direction: the first sun's aim made length 1, else (1, 1, −1) made length 1 | `characters.md` rule 37 (Q123 a) | Unit test `the_direction_is_the_suns_made_length_1_else_1_1_minus_1` |
| Each sun-facing model triangle projected along the sun onto each receiving triangle's plane, cut to it, each piece on its own (overlaps darken twice) | Rule 37 "The copy", rule 36 (user's check) | Unit tests `a_triangle_straight_under_the_sun_lands_lifted_20_with_its_own_area`, `a_shadow_is_cut_to_the_receiving_triangle`, `only_the_part_above_the_plane_casts_and_overlaps_stay_two_layers` |
| The receiving plane lifted 20 toward its front | Rule 37 | Same tests |
| Only the part of a model triangle above the receiving plane is projected | Rule 37 "Over water" (its suggestion) | Same tests |
| "Facing the sun" = the front (clockwise, Q2) turned toward the sun; a receiver facing away from the sun gets nothing | **Our reading** ("probably the side facing the sun"; projecting onto a back face would run toward the sun) | Same tests |
| Terrain: the drawn ground's triangles under the box moved along the sun, none beyond 1,500 above or below the ground | Rule 37 (1) and "the 1,500 limit is a cut" | First used in 4d-7b-2 |

### 4d-7b-2, characters' shadows on the terrain — `crates/kq8-viewer/src/anim_view.rs`, `load.rs`, `main.rs`

| What | Source | Check |
| --- | --- | --- |
| Only characters that cast (rule 36) and are drawn (near or far, not hidden) | `characters.md` rules 36, 37 "When" | `characters.log` shadow lines |
| The shown model frame's triangles, every frame, through `kq8_core::shadow` onto the terrain | Rule 37 | Same |
| The terrain triangles searched: under the model's own corners moved along the sun (the box's are not used) | Rule 37 (1) says "its box"; **our stand-in**: the model's points cover the same shadow | — |
| Drawn unlit, multiply blending at 0.5 (two layers: a quarter), no haze, no fading | Rule 37 "How dark" (0.5; "most likely" halves; blending not traced) and the user's check (half as bright, a quarter on overlaps) | Not checked by eye yet |
| 0.5 of the brightness as shown (sRGB 0.5, linear 0.214), not of linear light | **Our reading** (2026-10-21): an 8-bit screen halves the shown value; Bevy blends in linear light, where 0.5 left about 73 %. The user's check: ours was lighter than the original | Not checked by eye yet |

### 4d-7b-2b, Connor's shadow — `crates/kq8-viewer/src/anim_view.rs`, `connor_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Connor's shown model frame cast like a character's (direction, pieces, 20 lift, 1,500 cut, 0.5) | `characters.md` rule 37 and its user's checks (his shadow at a jump, wading) | `characters.log` "Connor: shadow" lines |
| He casts whenever his body is shown | **Stand-in**, asked as Q130 (his `shadowMask`, `allowShadows`) | — |
| A shadow with nothing to draw is hidden, not given an empty mesh | **Ours** (Bevy warns about empty meshes) | Run at the ruins: no warning |
| The original's shadow flicker and gaps are not reproduced | **The user's decision** (2026-10-20, after comparing with the original) | The user's check |

### 4d-7b-4, plain objects' shadows — `crates/kq8-data/src/objects.rs`, viewer `shadow_view.rs`, `draw_distance.rs`, `load.rs`, `terrain_view.rs`, `anim_view.rs`

| What | Source | Check |
| --- | --- | --- |
| `shadowMask` default 8 for plain objects; casts when (world mask) AND it is not 0 and it is solid | `characters.md` rule 36 | `shadows.log`: 77 of 617 cast in Daventry |
| The same drawing as characters (sun, pieces, 20 lift, 0.5, terrain within 1,500), only while drawn | Rule 37 "Plain objects" (Q123 d), "When" | `shadows.log` counts |
| The sun's aim made length 1 for objects too | Rule 37 says objects use it "not made length 1, which only changes how far the terrain search reaches"; our terrain search uses the model's points, so it makes no difference | — |
| Buildings only receive (no shadow of their own) | Rule 37 "What it lies on" (buildings receive); **our reading** that they don't cast | — |
| Worked out again only when its shown parts or place change | **Ours** (the same picture, less work) | 0.03 ms a frame |
| `allowShadows` taken as on | **Stand-in**, as for characters (4d-7a) | — |
| Which objects cast, compared with the original | The user's check (2026-10-20) | Pub barrels: only `barrel.07` casts, in both |

### 4d-7b-3b, which building surfaces receive a shadow — `crates/kq8-core/src/shadow.rs` (`building_receivers`)

The stand-ins below were **replaced by A's traced answer in 4d-7b-5a** (next sections).

| What | Source | Check |
| --- | --- | --- |
| Buildings receive shadows, only near or far ones (the caller's filter) | `characters.md` rule 37 (2) | — (viewer, 4d-7b-3c) |
| Within reach: the sum of the two sizes plus 500 | Rule 37 (2) | Unit test `a_building_receives_only_within_the_sum_of_sizes_plus_500` |
| The sizes: centre-to-centre distance; the building's ball as our draw distance has it; the caster's ball = the box around its shown model (middle, half diagonal) | **Stand-in**, asked as Q135 (a) | Same test |
| Band: any part of a surface between 2 × the caster's height below its feet and 100 above its top | Rule 37 (2) says "a little above the top" and "several times its size below"; the numbers are a **stand-in**, Q135 (b) | Unit test `the_band_runs_from_2_heights_below_the_feet_to_100_above_the_top` |
| Area: a square 1,800 across on x and y, around the caster's middle moved along the sun down to its feet; surfaces reaching into it receive whole | Rule 37 (2) ("at most 1,800 across centred on the shadow"); the shape, centre and "no cut" are a **stand-in**, Q135 (c). Data: 17,295 of 33,962 floor surfaces are wider than 1,800 (`kq8 building-verify`) | Unit test `the_area_is_1800_across_around_the_shadows_middle_on_the_floor` |
| The chosen surfaces get the same drawing as the terrain (pieces, 20 lift, part above the plane) | Rule 37 "The copy" | Unit test `the_chosen_floor_receives_the_shadow` |

### 4d-7b-3c, shadows drawn on buildings — viewer `terrain_view.rs`, `shadow_view.rs`, `anim_view.rs`, `draw_distance.rs`

| What | Source | Check |
| --- | --- | --- |
| "Active" buildings = near or far in the draw-distance test this frame | `characters.md` rule 37 (2) ("near or far (active)"), `object-lighting.md` rules 36, 40 | `characters.log`: "N of M active buildings in reach" |
| A building's surfaces taken once at load, placed in the world (buildings don't move) | **Ours** (no Daventry script moves a building; not checked in other worlds) | `viewer.log` "shadows on buildings": 26 copies, 19,660 surfaces |
| Only drawn surfaces receive | **Stand-in**, Q135 (d) | Daventry: 0 surfaces left out |
| Characters, Connor and plain objects all receive on buildings | Rule 37 (2) for characters; "Plain objects": "the same"; Connor: rule 36 (Q130, the same test); confirmation asked in Q135 | Barn wall run: Connor 143 of 364 pieces on the barn; windmill run: one plain object's shadow on 8 building surfaces (`shadows.log`) |
| Shadows on buildings compared with the original | The user's check (2026-10-20): as expected; shadows also show inside a building when Connor stands outside, in both | The user's check |
| No cut at the edge of the 1,800 area (whole surfaces receive) | **The user's decision** (2026-10-20): the original cuts some shadows off; not reproduced | The user's check |
| A plain object's shadow is worked out again when the set of active buildings changes | **Ours** (as its other "worked out again" rule) | `shadows.log` counts |

### 4d-7b-5a, the search box on buildings as traced — `crates/kq8-core/src/shadow.rs` (`Caster`, `search_box`, `building_receivers`)

| What | Source | Check |
| --- | --- | --- |
| The caster's size: its shown model's stored `.kqs` ball (radius, centre in its own space); s = 0.287 × radius | `characters.md` rule 37 (2) (Q135, traced) | Unit test `a_building_is_searched_within_its_radius_plus_s_plus_500` (Connor 879 → s 252) |
| Reach: building ball centre (turned `.dig` box middle) to the caster's ball centre in the world ≤ building radius + s + 500 | Rule 37 (2) (a) | Same test |
| Band (own space): s above to 6 × s below the ball's centre; over 1,800 cut around its middle | Rule 37 (2) (b) | Unit tests `the_band_runs_from_6_s_below_to_s_above_the_balls_centre` (1,079 / 687), `a_band_over_1800_is_cut_around_its_middle` (885 / 915) |
| Area (own space, turns with the heading): the rectangle around a 2 × s square at the band's top and its copy moved along the sun to height 0 | Rule 37 (2) (c) | Unit tests `the_area_spans_the_square_and_its_copy_along_the_sun_uncut` (505 × 2,139), `the_box_turns_with_the_casters_heading` |
| The area's sides are **not** cut to 1,800 (the original cuts them) | **The user's decision** (2026-10-21): the cut left a wall without shadow while the ground shadow slid under it; the user doesn't want the original's cut shadows | Barn walk: the wall receives from update 23 (30 with the cut) |
| The copy of the square is moved along the sun from the top of the model's ball, not from the band's top (the original's) | **The user's decision** (2026-10-21, same reason): the head's shadow went under the wall before showing on it | Barn walk: the wall's surfaces are picked before the shadow reaches them (update 18); standing box 505 × 3,089 |
| "The band's top" for the square = the top after the 1,800 cut | **Our reading** (A's example only covers an uncut band) | — |
| No sun with a downward part: no building search | Rule 37 (2) (c) | Unit test `no_downward_sun_means_no_search` |
| A surface counts when any part of it is inside the box; taken whole, the shadow not cut at the box | Rule 37 (2) "Which surfaces count" ("Do"; the original's looser test not reproduced) and the user's decision (no cut) | Unit tests above, `the_chosen_floor_receives_the_shadow` |
| Kind-0 (never drawn) surfaces skipped | Rule 37 (2) (d) "Do instead" | `viewer.log` (Daventry: none) |
| A caster without a stored ball: the box around its points (world axes, ball = half the box diagonal) | **Our fallback** (not in the note) | Unit test `the_fallback_caster_is_the_box_around_the_points`; until 4d-7b-5b every caster uses it |

### 4d-7b-5b, the model's stored ball in the viewer — `load.rs`, `shadow_view.rs` (`ModelBall`, `caster`), `anim_view.rs`, `connor_view.rs`, `door_view.rs`, `terrain_view.rs`

| What | Source | Check |
| --- | --- | --- |
| The ball is the shown model's: each movement's `.kqs` (radius, centre after the version), on each shown frame; still objects' own model | `characters.md` rule 37 (2) "The character's size" (Q135); the ball's place in the file: GPL notes (`kq8kqs.txt`), already read by `shape.rs` | `characters.log`: Connor standing s 252 (879), walking s 318 |
| The caster's own space = the shown frame's place (position, heading) | Rule 37 (2) (a)–(c) | `characters.log`: Connor facing +x gets a 1,800 × 505 box, facing +y 505 × 1,800 |
| Items lying in the world have no ball on their entities: our fallback box | **Ours** (items not checked; no stored ball passed) | `shadows.log` counts fallbacks (0 at the windmill) |

### 4d-12-1, the cylinder test — `crates/kq8-core/src/collide.rs` (`CylinderCollider`)

| What | Source | Check |
| --- | --- | --- |
| Each of the mover's balls swept against an upright cylinder on the other's position, widened by the ball's radius sideways and at both ends; the earliest hit counts | `connor-collision.md` rule 18 | Unit tests |
| Side hit: level direction out from the axis through the hit point; top / bottom: straight up / down | Rule 18 | Unit tests |
| Moves under 0.000001 in every axis not tested | Rule 18 | Unit test |
| A ball starting inside: no hit for type bit 2 or 4; otherwise a hit at the start, direction against the movement | Rule 19 ("along the movement"; **our reading** of the sign: against it, like every other contact; the slide comes out the same either way) | Unit test |
| The 25 pull-back on the contact | `connor-physics.md` "Collision shape" (every object hit, Q33 b) | Unit test |
| Radius = the other's ball radius, height = its box height, foot on its position; every ball tested on a vertical move | **Stand-in** (rule 18 "not pinned down"), asked as Q131 (a), (b) | — |

### 4d-12-2, characters in Connor's scene — `crates/kq8-core/src/connor_mover.rs`, `character.rs`, `active.rs`

| What | Source | Check |
| --- | --- | --- |
| The scene's character cylinders, tested after the objects; the nearest contact over everything is kept | `connor-collision.md` rule 18; `connor-physics.md` "Against a building" step 7 | Unit tests |
| A character hit is never head-on, also while jumping (always the glancing handling: slide, or stay if nothing is left) | `connor-physics.md` "Exact hit handling" (never against a `KQMonster`; the jump rule does not win over it); `characters.md` rule 1 (every character is a `KQMonster`) | Unit tests |
| Solid for Connor: not deleted, has a box, not in Birth, collider on now, his mask covers its type apart from 0x10 | `connor-collision.md` rules 2, 4 (he is told; ignores 0x10); `characters.md` rules 9, 16, 33 | Unit test |
| Not a push: no character type has 0x100 (Daventry: 0x42, 0xC021, 0xC022) | Rule 4; data | — |
| The cylinder follows the character's position each update, after its gravity | Rule 18 ("standing on the other's position") | Unit test |
| Landing on a character's top is an ordinary landing | **Stand-in**; the 250 sideways push is asked as Q131 (c) | — |
| `characters.log`: solid / not solid and why, with the cylinder | AGENTS.md (log, don't look) | Unit test |

### 4d-12-3, characters solid for Connor in the viewer — `crates/kq8-viewer/src/draw_distance.rs`, `door_view.rs`, `connor_view.rs`, `anim_view.rs`, `main.rs`

| What | Source | Check |
| --- | --- | --- |
| Solid only while its collider bit (0x8) is on: near yes, far, off and hidden no; unborn zombies never (their groups have no collider bit) | `characters.md` rules 16, 33; `object-lighting.md` rule 37 | `characters.log` "solid for Connor" lines |
| Each character's type, mask (`colliderType`, `colliderMask`, defaults 0x42 / 0x281F) and `BOUNDS.KQ` box (by type, else by name) | `characters.md` rules 8, 9 | Same |
| The list rebuilt each update after the characters' own update and gravity, before Connor's move | `connor-collision.md` rule 21 (order not traced; **ours**, as for objects) | — |
| Every hit on a character logged (who, where, distance, surface direction, angle, slid or stayed) | AGENTS.md (log, don't look) | `characters.log` |

**4d-12 checked by the user against the original (2026-10-20):** the goblin's stopping distance (stand-in cylinder size, Q131 a) and the slide round it, walking away, the Wizard and his conversation all match.

### 5a-1, what the movement rules read — `crates/kq8-data/src/characters.rs` (`ChaseMode`, `AiSettings`), `crates/kq8-cli/src/character_verify.rs`, viewer `load.rs`, `anim_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Monster list fields: team (3rd), chase mode (16th), attack distance (19th); type list: chase mode (11th), attack distance (14th) | Data: the files' own header lines (`;monsterX=`, `;typeX=`) | Unit tests; `character-verify.log` |
| `chase a b` = home radius a, chase radius b; `guard` / `none` = both 0 | `character-movement.md` rule 27 | Unit test |
| Only the first number of `attackDist` (`slime`: `5000 0.4 3.0`) | Data; **ours** (the other numbers belong to the fight study) | Unit test |
| The list's (or type's, for `@`) movement, alarm radius, chase mode and attack distance replace the `.kq` keys; the team is the line's | `character-movement.md` rules 9, 23, 27; `characters.md` rule 31 | Unit test; goblin22 alarm 4,000, chase 3,000 / 6,000 (type `goblin2`) |
| `.kq` keys `homeRadius`, `chaseRadius`, `attackDist` (default 1,250), `team`, `canAlwaysSeeConner`, `wanderAndStop`, `doMode` | `character-movement.md` "Data used", rule 25 | Unit test |
| A `.kq` character without `alarmRadius` / `homeRadius` / `chaseRadius`: 0 | **Ours** (no default in the notes; every Daventry character loaded this way has `wanders=0`) | — |
| `KQWorld::MaxMonsters` from `world.cs` | `character-movement.md` rule 20; data (Daventry 6, Ice World 10) | `character-verify.log` |

### 5a-2, walking — `crates/kq8-core/src/character.rs` (`Walk`, Move), `connor_mover.rs` (`object_slide`, `line_blocked`, `drawn_line_hit`), `collide.rs` (`CylinderCollider::line_hit`), `named_anim.rs` (`loops_move_base`), `active.rs`, viewer `anim_view.rs`

| What | Source | Check |
| --- | --- | --- |
| A character moves only by its movement's steps: a repeating movement's frame steps added up × the phase moved, spread evenly (as Connor's loops), turned by its rotation, through its mover | `character-movement.md` rule 1; `connor-movement.md` "Forward speed comes from the animation" | Unit test (917 / 3,068 units/s); viewer: goblin22 917 walking, 3,068 running |
| The drawing does not add a character's repeating movement's shift (a turn in its frames stays) | **Ours**, following from rule 1 (otherwise the model would run ahead and jump back at each loop) | Unit test (placement = place) |
| Entering Move: `Stop` at a random frame and speed, then at once `walk` / `run`; a walk or run that ends is asked for again | Rule 4 | Unit test; `characters.log` |
| `walk` / `run` and their fallbacks | Rule 3 | Unit test |
| Steering: the automatic turn (3.5 rad/s) aimed anew each update, walking on; `noTurn` blocks it | Rule 5; `characters.md` rule 23 | Unit test |
| A character's across pass: the ordinary slide of an animated object (normal z 0–0.7 upright; the hit and 3 retries, then stays), never head-on, no push checks | Rule 6; `pushing.md` rule 4.1 | Unit test; viewer: goblin22 slides along Sara's cottage |
| The stuck check: after 1 s, then every 0.5 s; moved < 200 (3D) → no steering for 3 s, the first free of four diagonals 1,250 out (blocked by the drawn ground and mask 0xB: objects, characters, buildings), one further round each time, a snap turn, `run` | Rule 7 | Unit tests; viewer: goblin22 at the cottage |
| The diagonals are the world's, in the order π/4, 3π/4, …; the line 1 unit above its feet | **Stand-in**, asked as Q132 (a), (b) | — |
| All four blocked: turned round | **Stand-in**, the note's open question 1 | Unit test |
| `KQMonster::setState <name> Move`: walks straight on (`walk`) | Rule 2's edge case ("put there by a script"); `walk` is **our reading** | Viewer run |
| No gestures while walking | `characters.md` rule 28 | — |
| `characters.log`: Move, once a second while moving (place, heading, movement, distance in the last second, steering), every hit of its own move, every stuck event with the diagonals tried | The note's "How to check" (our logs) | Viewer run |

### 5a-3, a character's own touches — `crates/kq8-core/src/active.rs` (`characters_gravity`), `character.rs` (`OWN_TOUCH_OBSTACLES`), viewer `door_view.rs`

| What | Source | Check |
| --- | --- | --- |
| A character's own move meets, as obstacles, whatever its mask covers with type bits 0x100, 0x8, 0x1, 0x2, 0x4, 0x2000: plain objects, buildings, other characters, Connor; traps (0x10) nothing; in every pass (up stops, across slides, down lands) | `character-movement.md` rule 33 (Q128) | Unit test (cart, trap, Connor, another goblin) |
| Other characters as their cylinders where they stand now, only while solid (collider on, not in Birth, with a box) | Rule 33 with `connor-collision.md` rules 18–19 and `characters.md` rules 16, 33; "where they stand now" (already moved in this pass) is **ours** | Unit test |
| Connor as the upright cylinder of his box (radius 200, height 1,300) | **Stand-in**, asked as Q132 (c) | Unit test; viewer: goblin22 stops 1,000 from him |
| `colliderMask=0`: nothing but the ground check | Rule 35 | Unit test |
| Nobody is pushed by a character's move | Rule 34 | — (nothing moves the other) |

### 5a-4, sight and the active list — `crates/kq8-core/src/character_ai.rs` (new), `character.rs`, `active.rs`, viewer `anim_view.rs`, `door_view.rs`, `load.rs`

| What | Source | Check |
| --- | --- | --- |
| The sight test every 15th update: a line from the top of its box to 1,250 above Connor, blocked by the drawn ground, plain objects (bit 1) and buildings (bit 8), not characters or Connor; no distance limit; `canAlwaysSeeConner` | `character-movement.md` rule 20 | Unit tests; viewer: goblin22 blocked by the cottage, chicken1 (eye 400) sees him |
| "Every 15th update" = the world's update number a multiple of 15 | **Our reading** (the note does not say from when it counts) | Unit test |
| The count of updates since it last saw him | Rule 20 | — (used from 5a-7) |
| In Wait, not active and seeing him → joins the list if there is room (`KQWorld::MaxMonsters`, from `world.cs`); a team member takes the place of the **first** one without a team (**our reading**) | Rule 20 | Unit test |
| Off (not near or far) or pushed off → leaves; put home at once if on its way home or Connor > 2 × alarm radius (plain 3D distance: **stand-in**, the note's open question 2); `hide` → Birth; Move → Wait | Rule 21 | Unit tests |
| `characters.log`: every change of sight (and what blocked it), joins and leaves, a status line per character every 15 updates | The note's "How to check" | Viewer run |

### 5a-5, wandering — `crates/kq8-core/src/character_ai.rs` (`make_route`, `wander`, `set_home`), `character.rs`, `script.rs`, viewer `anim_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Only active wanderers wander; in Wait they start walking (`walk`) the way they face | `character-movement.md` rules 3, 9, 11, 12 | Unit test; viewer: chicken1 |
| The points: `loop` in order; `patrol` there and back (`A B C D C B A`, then round again); only the first n read | Rule 10 | Unit test |
| `random` / no points without `Birth`: own position + 3 at 2–3 × alarm radius, ≥ 0.79 rad apart; the exact spread of the other directions is **ours** | Rule 10 | Unit test |
| Reached within 250 (x, y) → next point, turn toward it; re-aim only when distance² > closest² + 12,500² | Rule 11 | Unit test |
| Steering ends once it faces the point (it then walks straight) | Rule 5 | Unit test |
| After the stuck escape a wanderer takes its next point | Rule 7 | Viewer: chicken1 |
| `wanderAndStop`: no more wandering at the end of the points; its cue not traced | Rule 11 | — (no Daventry file has it) |
| `KQMonster::setHome <name> x y z`: home only, never moves it | Rule 13 | Unit test |

### 5a-6, idling — `crates/kq8-core/src/character_ai.rs` (`idle`), viewer `load.rs` (`movement_animation`, the idle list)

| What | Source | Check |
| --- | --- | --- |
| `idleAnimFile`'s movements loaded with their models, kept in the character's player as `idle:<name>` | `character-movement.md` rules 15, 18 ("played from that list"); keeping both lists in one player under a prefix is **ours** (the result is the same: a fidget, then `Stop`; the note's open question 3) | Viewer: goblin22's `react` / `attack` |
| Only active, not on its way home; the first fidget as soon as it is active; a walking wanderer goes back to Wait for it | Rule 16 | Unit test; viewer |
| Picked at random, all equally likely; an ordinary request | Rule 18 | Unit test |
| The next due 2 + round(random × 10) s after it ended; any movement ending during it counts as its end (we take: the movement shown is no longer the fidget, or it has ended) | Rule 17 | Unit test |
| Then Wait's `Stop` at a random frame and speed | Rule 18; `characters.md` rule 18 | Unit test |
| Wanderers don't start walking while a fidget plays or is due | Rule 12 | Unit test |

### 5a-7, alarm, chase, giving up, going home — `crates/kq8-core/src/character_ai.rs` (`alarm_rules`, `alarm`, `spread_alarm`, `give_up`, `way_home`, `monster_ai_command`), `active.rs`, `script.rs`

| What | Source | Check |
| --- | --- | --- |
| Alarmed when active, not fidgeting, in Wait or walking its points, seeing Connor closer than its alarm radius (straight distance), `goHome` off, radius above 0 | `character-movement.md` rule 23 | Unit test; viewer: goblin22 at 3,175 |
| A random 0–1 s before it reacts (wandering carries on; no new fidget once alarmed); chase timer 5 s; no alarm line (none in Daventry's files) | Rules 16, 23 | Unit test |
| The walk / run coin at each start of a chase move | Rules 3, 23 | Viewer |
| The alarm spreads to active characters (alarm radius > 0) that see Connor or whose team does, and to the alarmed one's team (made active if needed, **ours**: even past `MaxMonsters`); being told does not tell others again (**ours**) | Rule 24 | — (windmill spriggans not run yet) |
| Standing: turns to him (or the last seen spot if more than 500 away); moves after him when out of sight or beyond its attack distance and its chase radius is above 0; `guard` (chase 0) only turns | Rule 25 | Unit test |
| Moving: steers at him each update; seen and within its attack distance → Wait facing him, "the attack would start" logged | Rule 25; the stop in every case is a **stand-in** until the fight study (Q133 b) | Unit test; viewer |
| Not seen for 45 updates → gives up, with the team exceptions | Rule 26 | — |
| After the chase timer: no team and beyond 2 × alarm radius, or beyond its chase radius from home (x, y: **ours**) → gives up (`run` for the second) | Rule 27 | Unit test; viewer: goblin22 at 6,007 from home |
| Giving up: target forgotten, on its way home, `hide` → Birth playing `hide`; steers home, within 100 in x and y → Wait; takes up the chase again as rule 28 says | Rule 28 (the loop it gives at the leash is asked as Q133 a) | Unit test; viewer |
| `KQMonster::goHome <name> [0]`, `setAlarmRadius`, `setChaseRadius`, `setHomeRadius` (no longer stubs) | Rules 23, 27, 29 | Unit test |

### 5a-8, the zombies rising — `crates/kq8-core/src/character_ai.rs` (`wake`, `rise`), `character.rs` (`unborn`), viewer `anim_view.rs`, `draw_distance.rs`

| What | Source | Check |
| --- | --- | --- |
| In Birth the sight test runs; not active, seen, within its alarm radius → active (if there is room) and alarmed | `character-movement.md` rule 30 | Unit test; viewer: the road zombies |
| `birthDelay` s later: waits until Connor is farther than its ball + his (200), then `birth` at once, drawn and solid, gravity on, turned toward him, in Wait | Rule 30; Connor's 200 from `BOUNDS.KQ` | Unit test; viewer: RoadZ2–4 at once, RoadZ1 after 3 s, RoadZ5 not while he stands on it |
| The chase waits until `birth` has ended | **Stand-in**, asked as Q134 | Viewer |
| Giving up with `hide`: Birth, `hide`; when it ends, underground again (not drawn, not solid); the reset to home with `hide` (rule 21) at once | Rules 21, 28; `characters.md` rule 16 | Unit test |
| The viewer shows the body and gives back its own draw / collider groups while the running game says it has risen | `characters.md` rules 16, 33 | `characters.log` "risen: drawn" |

### 5a-9, the flying mover corrected — `crates/kq8-core/src/connor_mover.rs` (`fly_update`, `line_hit`), `character.rs`

| What | Source | Check |
| --- | --- | --- |
| Steps 0, gravity off; the ordinary move with the `keepOnGround` raise | `character-movement.md` rule 32 | Unit test |
| The drop of 150 as a down pass, cancelled completely if it hits anything | Rule 32 (corrects `characters.md` rule 8, Q129) | Unit test (stays 100 above land) |
| A line down from the top of its shape to its feet against the drawn ground and buildings; raised onto what it finds, at most 300, as an up pass, cancelled if that hits something | Rule 32; the up pass not testing the terrain it is put on is **ours** | Unit test; viewer: bat2 ends on the lake's surface, 6,630 |

### 5a fix: `getLoc` while playing — `crates/kq8-core/src/active.rs` (`get_loc`, `angles_of`)

| What | Source | Check |
| --- | --- | --- |
| `KQObject::getLoc` / `getLoc` sets `$KQObject::loc` (x y z and the angles worked back from the turn) and `$KQObject::locOnly` (x y z), from the object's current place | `objects.md` "Script variables, getLoc and move" rule 1 (Q20) | Unit test; viewer: `zomtrap.cs` sets the graveyard zombies' homes |
| Connor's turn as 0 0 0 | **Ours** (his turn is not in core's world) | — |

### 5a fix: zombies rise again — `crates/kq8-core/src/character_ai.rs` (`alarm_rules`, `rise`)

| What | Source | Check |
| --- | --- | --- |
| A sunk zombie (Birth, on its way home) takes up the chase again as rule 28 says, then rises after `birthDelay` as rule 30 | `character-movement.md` rules 28, 30, read together (**our reading**); the user saw zombies rise again in the original; asked in Q134 | Unit test; viewer: the road zombies |
| It rises only once a `hide` under way has ended | **Stand-in**, asked in Q134 | Viewer |

### 5b-1, the exact cylinder — `crates/kq8-core/src/collide.rs` (`CylinderCollider::from_box`, `sweep`)

| What | Source | Check |
| --- | --- | --- |
| Radius = the ball radius; bottom = position + box's lowest z; height = box height with more than one ball, else 2 × radius; whole units | `connor-collision.md` rule 18 (Q131 a; replaces the 4d-12-1 stand-in) | Unit test: goblin 0–1,600, Connor 0–1,300, a bat from 1,170 |
| A purely vertical move tests one ball: the top one going up, the bottom one going down | Rule 18 (Q131 b) | Unit test |

### 5b-2, landing on a character's top — `crates/kq8-core/src/connor_mover.rs` (down pass, `MONSTER_TOP_PUSH`)

| What | Source | Check |
| --- | --- | --- |
| He lands on a character's top as on any surface facing up more than 0.8, then is pushed 250 level from the contact, away from its axis (exactly over it: toward his back), as an across move (3 slides), in any state; Connor only | `connor-physics.md` "landing on a `KQMonster`" (Q131 c; replaces the ordinary-landing stand-in) | Unit test |

### 5b-3, the stuck escape corrected — `crates/kq8-core/src/character.rs` (`after_move`, `stuck_escape`, `enter_move`)

| What | Source | Check |
| --- | --- | --- |
| The check is on only once `walk` / `run` really plays; first check 1 s after that | `character-movement.md` rule 7 (Q132) | Unit test; viewer: chicken1 |
| The world's diagonals NE, NW, SW, SE, ±1,250 in x and y; the round starts at NE on each start of moving, one further per stuck event | Rule 7 (Q132 a; replaces our π/4-first, 1,250-long stand-in) | Unit test |
| Level from its feet, lifted 1 | Rule 7 and its "Do not reproduce" (Q132 b) | — |
| All four blocked: face north (+y) | Rule 7 (replaces "turned round") | Unit test |
| Connor never blocks a diagonal (the line's mask 0xB has no bit 4), nor a box the line starts in | Rule 7 | — |

### 5b-4, the chase corrected — `crates/kq8-core/src/character_ai.rs` (`alarm_rules`, `way_home`, `idle`, `set_home`), `active.rs` (`connor_move_key`), viewer `door_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Within reach, a melee character stops only while Connor has no forward / back key (a scripted walk sends none); otherwise it follows him at reach | `character-movement.md` rule 25 (Q133 b); the user's check in the original | Unit test; viewer: goblin22 follows a walking Connor |
| Where the attack would be due it stops and logs it (the attack itself is the fight study) | **Stand-in** until the fight study | — |
| `doMode=1`: stop within 0.75 ×, start beyond 1.25 × the attack distance | Rule 25 | — (none in Daventry's moving characters) |
| Taking up the chase on the way home: only while moving, seeing him; no coin (keeps its movement); keeps heading home during its reaction delay; new chase timer | Rules 3, 28 (Q133 a) | Viewer: "no coin, it keeps 'run'" |
| Standing leash also with a chase radius of 0; the home distance 3D; standing give-up goes home at a `run`, a guard stays | Rules 27, 28 | — |
| `setHome` z −1 / −2 placed on the floor / surface | Rule 13 | — |
| A fidget named `attack` / `react` ending sets the chase timer to 5 s | Rule 18 | — |

### 5b-5, the zombies as traced — `crates/kq8-core/src/character_ai.rs` (`give_up`, `sunk`, `spread_alarm`, `alarm_rules`), `active.rs`

| What | Source | Check |
| --- | --- | --- |
| Sinking: `hide` where it gave up, still drawn, solid and active; when `hide` ends: off the active list, underground (not drawn, not solid), gravity off, on its home point, no longer on its way home | `character-movement.md` rule 30 (Q134; replaces the 5a fix's re-take underground) | Unit test; viewer: the road zombies |
| No alarm during `hide`; told by a team mate during it: underground at home at once | Rule 30 | Viewer: RoadZ2 told by RoadZ3 |
| Rising again only by the rising test, from home | Rule 30 | Unit test; viewer |
| Solid for others and for Connor while not underground (a sinking zombie stays solid) | Rule 30; `characters.md` rule 16 | Unit test |
| The chase during `birth`: Move at once, its walk waiting for `birth` (the stand-in "stay in Wait" removed); stuck check from the walk's start | Rules 7, 30 | Viewer |

**5a / 5b checked by the user against the original (2026-10-20):** the creatures' movement and routes, the chase, following a walking Connor at reach and stopping when he stands, the leash sending a goblin home, and the zombies sinking and rising again from their graves all match.

## The fight (M5 5c) — `docs/behaviour/fighting.md`

Written from `fighting.md` (A, 2026-10-20; rules 6 and 31–32 checked by
the user in the original) and the game's data; no executable looked at.

### 5c-1, reading the fight's numbers — `crates/kq8-data/src/{characters,items}.rs` (`ItemEntry`, `item_entries`, `FightSettings`, `ItemType::extra`), `crates/kq8-cli/src/character_verify.rs`, `crates/kq8-core/src/{fight,character,active}.rs`, viewer `load.rs`, `anim_view.rs`

| What | Source | Check |
| --- | --- | --- |
| Monster list columns 4–7 `xtraHealth xtraDamage xtraProtect xtraExp` (`@` = 0), 14–15 `invHold invDrop`, 20 `attackDelay` (`@` = the type's) | `fighting.md` rules 2, 31, "Data used"; data: the file's own `;monsterX=` header | Unit test; `character-verify` |
| Type list columns `Health Damage Protect Experience invHold invDrop attackDelay` | Same; data: `Monstats.mtl`'s `;typeX=` header | Unit test |
| The attack distance's 2nd and 3rd numbers are `tooClose farEnough` | Rule 37 | Unit test (`slime` `5000 0.4 3.0`) |
| Item lists: comma separated, quotes, `none` / `@` = nothing, `SilverCoins(3)` = 3 | Rules 2, 31 | Unit test |
| Item list's last column (`extraData`) = a `LongWeapon`'s projectile file | Rule 36; data (`INVITEM_SmCrossbow` → `Prscbolt.kq`) | `character-verify` |
| Stats = type + extra; `Damage` fills hand and ranged; protection capped at 100 (also after each item); held items (type's, then line's) add health and protection; a `LongWeapon` adds ranged damage and makes it ranged, others add hand damage; items' experience not added; Flame Sword (id 1110) → fire | Rules 1, 2, 14, 34 | Unit test; all of rule 5's table matches `character-verify` and `fight.log` |
| Drops = the line's `invDrop`, then the type's | Rule 31 | `character-verify` |
| Keys `deadDts`, `idedKqs`, `projdKqs`, `ouchSound`, `normalReactPercent`, `specialReactAnim`, `specialReactPercent`, `likelyToFlee`, `mustFlee` | "Data used" | `character-verify` |
| `reactPause` read and logged only | Data (3 files outside Daventry); not in the note: **Q136** | — |
| Only monster-list characters fight (`.kq`-only characters get no fight part) | Rule 2 covers only the list; **ours** for the rest (Daventry: the scripted Knight, Henchman are list lines; the others never fight) | `fight.log`: 52 fighters |
| `fight.log`: one line per fighter at load | AGENTS.md (log, don't look) | Viewer run |

### 5c-2, the `interruptable` cue — `crates/kq8-core/src/named_anim.rs` (`interruptable_cues`), `active.rs` (`cue_sound`, log text)

| What | Source | Check |
| --- | --- | --- |
| A cue `interruptable <n>` the phase passes sets "can be cut short" to n, for every movement player (characters, objects) | `anim-scripts.md` rule 7 (data: only in `.anm` files) | Unit test (goblin2's `attack`: on at 0, off at 0.1875) |
| Two such cues passed in one update: the later in the file wins | **Ours** (Daventry's lists have them in phase order, and never two in one update) | — |
| The cue at phase 0 fires in the movement's first moving update (the update after its start, as every movement) | Our player's existing rule (M3 3i) | Unit test |

### 5c-3, characters attack — `crates/kq8-core/src/{fight,character,character_ai,active}.rs` (`Fight::attack_time`, `start_attack`, `attack_ended`, `attack_cut_short`, `attack_now`, `character_do`), viewer `door_view.rs` (`connor_back_key`)

Replaces the 5a / 5b stand-in (within reach a character only stopped or followed).

| What | Source | Check |
| --- | --- | --- |
| The attack time starts at 0 (first attack at once), never reset by an alarm | `fighting.md` rule 6 | Unit test; viewer: goblin22's first swing on arrival |
| Standing (Wait, alarmed, past its reaction): sees him, closer than reach (ranged 1.25 ×), due, movement can be cut short or ended → `Attack` | Rule 7 | Viewer |
| Moving: the same with 1 × reach, whatever Connor does; before it is due, 5b-4's stop rule | Rule 7 with `character-movement.md` rule 25 | Unit test; viewer (from `run`) |
| While `Attack` plays from Move, no stop rule: it stays in Move until the swing ends | Rule 11b, "Do not reproduce" (**the user's decision**: the swallowed swing is not reproduced) | Unit test (no "stops" between the start and the end) |
| `Attack` asked for as an ordinary request (any capitals) | Rule 8; `anim-scripts.md` rules 4–5 | — |
| While it plays: attack time "never", chase timer 5 s from its start, stuck check off until `walk` / `run` plays again; steering / turning go on; no step (the models have none, and only repeating movements step) | Rule 9 | Viewer: the swing at Connor 1,154 away stays there |
| Its end (any movement named `Attack` / `Attack2`, idle fidgets too): attack time now + `attackDelay`, chase timer 5 s | Rule 10; `character-movement.md` rule 18 | Unit test; viewer: one swing every 30 updates (1 s + 1 s) |
| Cut short by something else: put right at the next entry into Wait (`set_state`, the start-up's Wait) or Move (`enter_move`) | Rule 11 (the hit case comes with 5c-6) | — |
| The `Do` cue of a fighter: logged only (hit test in 5c-4) | Rule 13 | Viewer |
| `fight.log`: per attack who, distance, angle off its heading, his back key, attack time before / after, the movement replaced | The note's "How to check" (our logs) | Viewer |
| His back key (forward key −1) handed to core each update | Rule 13 (used in 5c-4) | — |

### 5c-4, the swing lands on Connor — `crates/kq8-core/src/{fight,active,connor}.rs` (`HitTest`, `Damage`, `character_do`, `hit_connor`, `Request::ConnorHurt` / `ConnorDies`, `Walker::death`), viewer `connor_view.rs` (`hurt`, `die_from_hit`), `door_view.rs`, `load.rs` (`CONNOR_MOVEMENTS`, `ConnorFight`)

| What | Source | Check |
| --- | --- | --- |
| At `Do`: the target is Connor when the character is alarmed; none otherwise (nothing happens) | `fighting.md` rule 13 ("it needs a target") | — |
| In reach: 3D distance feet to feet ≤ reach, 1.25 × while his back key is held | Rule 13 | Unit test; viewer: hits at 1,642 / 1,503 with the key held (reach 1,875) |
| In front: the line to him less than 90° off its facing (3D dot product > 0; exactly 90° is behind) | Rule 13 | Unit test (Connor behind a `noTurn` goblin: misses) |
| A miss logs the original's line ("missing Connor, too far away") plus why; nothing else happens | Rule 13 | Unit test |
| A hit: its hand damage, `Fire` with a Flame Sword | Rule 14 | Viewer |
| Roll on Connor round(*D* × (0.25 + r × 0.75)) ≥ 1; protection 100 or *D* 0: no loss; *P'* = round(*P* × (0.75 + r × 0.5)) ≤ 100; loss cut(*R* × (100 − *P'*) / 100) ≥ 1; "round" = halves up | Rule 16 | Unit test: rule 19's examples (2 → 1–2, 15 → 4–15, fists 10 on a character → 8–12) |
| One random generator for the fight's rolls, seeded at start | **Ours** (the note says only "random") | — |
| Below 0 (0 alive) → his dying state; hits ignored from then on | Rules 18, 29 | Unit test; viewer: dies at −2 after 8 hits |
| Health kept at most 10 × level; the bars refreshed (stats event count) | Rule 29; `interface.md` rule 24 | — |
| The two one-off tips: logged only | Rule 29 (not traced further) | Viewer |
| Flinch: *R* > 0.9 *D* (his `normalReactPercent` default), his movement can be cut short, last flinch ended over 2 s ago → `react`; else `ouchSound` (`conrow3.aud`, placed one-shot) at most once a second, a flinch starting that second too | Rules 23, 24, 29 | Viewer: flinch at updates 169 and 229, refused at 199 (1.20 s after the last ended) |
| "His movement can be cut short" = the keys choose it (Free, Push, talk), no start / stop transition plays, and its `loop=` line says so or it has ended | **Ours**, from Connor's movement rules (his `interruptable` cues come with his swings, 5c-5) | — |
| The flinch played as a one-item Special list (keys ignored while it plays) | **Stand-in**: how his own flinch state works was not traced beyond rule 29 | — |
| Death: `die` (Normal, Projectile), `dropweapon` then `firedie` / `icedie` (Fire, Ice), then his Die state holding the last frame; during a jump the death waits like any state request (rule 28) | Rule 29; **stand-in**: game over is another study, he stays dead | Viewer |
| `Connor.kq`: `ouchSound`, `minDamageFactor`, `maxDamageFactor`, `ShortAnimList` (weapon → `.anm`) | Rules 24, 39–41; data | Viewer log line at load |

### 5c-5, Connor hits back — `crates/kq8-core/src/{fight,connor_inventory,active}.rs` (`connor_do`, `hit_character`, `cut_short_at`, `SWINGS`, `ready_hand_weapon`, `hand_weapon`, `weapon_ready`, `combat_level`), viewer `connor_view.rs` (`Swing`, `swing_click`, `swing_step`, `can_be_cut_short`, the `do` cue), `door_view.rs` (`SwingClick`, `SwingAt`), `load.rs` (the weapon lists' swings), `main.rs` (`--swing`)

| What | Source | Check |
| --- | --- | --- |
| Clicking the hand-weapon slot readies the weapon (`INVITEM_Fists` too) | `inventory.md` rule 9; `fighting.md` rule 39 (manual) | Unit test |
| A second click on the ready weapon's slot puts it away | **Stand-in** (not traced) | Unit test |
| With a weapon ready, a left click in the world swings; Ctrl held gives the plain click; Shift the big swing | Rule 39 (**manual only**: open question 1; stand-in, logged) | Viewer (`--swing`) |
| The clicked character becomes his target; a click on no character: the point where the click line meets the level of his feet | Rule 39; **stand-in** for the point (the ground is not tested) | — |
| He turns toward the click (the automatic turn, 3.5 rad/s); more than 90° off: the swing waits for the turn, otherwise it starts at once and he turns during it | Rule 39 ("turning round first if it was behind him"); **stand-in** for the details | Viewer: 130° off → turn, then `attack` 10 updates later |
| `Attack` / `AttackB` 50/50, Shift `Attack2`; nothing while a swing plays; only when his movement can be cut short (its `loop=` line and `interruptable` cues) | Rule 40; `anim-scripts.md` rule 7 | Viewer: the click at update 92 refused; unit test of the cues |
| The swings come from the ready weapon's list in `Connor.kq`'s `ShortAnimList` (`INVITEM_Fists` → `confists.anm`), loaded as `<list>/<movement>`; his other movements stay `Conner.anm`'s | Data; **stand-in** (switching his whole set of movements belongs to the Connor-controls study) | Viewer |
| The swing plays as a one-item Special list (keys ignored while it plays) | **Stand-in** | — |
| His `Do`: rule 13's test with his reach (`attackDist` 2,350) and his target; misses say "missing … too far away"; no target: dead air | Rule 41 | Unit test; viewer |
| Damage: his hand damage (stats, weapon included); Shift × a random factor from `minDamageFactor` to `maxDamageFactor` (0.75 to 4), cut | Rule 41; data (`Connor.kq`; the manual's 5 is not used: "Do not reproduce") | Unit test; viewer (× 3.804 → 38) |
| `Fire` when the ready weapon is the Flame Sword | Rules 14, 41 | — |
| On a character: roll 75–125 % (85–125 % below combat level 1.0), its protection, its health (its reaction: 5c-6; dying: 5c-7) | Rule 16 | Unit test; viewer (loss 11 then 8) |

### 5c-6, characters being hit — `crates/kq8-core/src/{fight,character,character_ai}.rs` (`hit_character`, `run_alarm`, `hit_unalarmed`, `hunt`, `Fight::{last_kind, must_flee, flinch_end, last_ouch, flinching}`, `Ai::{hit_count, hunting}`)

| What | Source | Check |
| --- | --- | --- |
| Rule 22's order: kind; active (if room); chase timer 5 s; hit count +1; must flee (`likelyToFlee` > 0.5); damage (below 0: the rest skipped); attack time "never" → now + `attackDelay`; flinch or ouch; where Connor is; the alarm with "run" | `fighting.md` rule 22 | Viewer (goblin21), unit tests |
| From the second counted hit it always runs when it starts moving (the coin skipped); cleared at home | Rules 22.4, 27; `character-movement.md` rules 3, 28 | Viewer: "hit count 7" |
| No knock-back | Rule 20 | — |
| Flinch: *R* > `normalReactPercent` (0.9) × *D*, movement can be cut short or ended, last flinch ended > 2 s ago → `React` as an ordinary request in its state; a special flinch when `specialReactAnim` and *R* > `specialReactPercent` × *D* | Rule 23 | Unit test (never two within 2 s of an end); viewer: refused while its `Attack` can't be cut short |
| The flinch's end: the 2 s pause starts, chase timer 5 s | Rule 23 | Viewer: update 102 |
| The 2 s pause is fixed; `reactPause` not used | Rule 23; **Q136** asks about `reactPause` (none in Daventry) | — |
| Ouch: `ouchSound` (default `sword.aud`), placed one-shot at its position, at most once a second (a flinch starts that second too) | Rule 24 | Viewer: "the last was 0.60 s ago" |
| Surprise lines: not built (no Daventry file has the keys) | Rule 25 | — |
| The alarm with "run": every active character with an alarm radius that sees Connor (or its team does), the hit one included: alarmed if it had no target (rule 23, its delay, team); hit count 2, `run` at once if moving, chase timer 10 s | Rule 26 (as written: no distance limit) | Viewer: goblin21 plus Brdgsprig and Fallsprig (active, seeing him); unit test |
| Hit while not alarmed and standing: no team and a chase radius → runs with no target: alarmed (no delay) within its alarm radius; after its chase timer gives up beyond 4 × alarm radius or runs home beyond its chase radius; heads for the point at his distance 0.52 rad anticlockwise off the line to him; otherwise alarmed at once | Rule 28 with its "Do not reproduce" (the original's point is about twice the angle) | Unit test |
| A hunting character neither wanders nor fidgets | **Ours** (rule 28 replaces its wandering) | — |

### 5c-7, dying — `crates/kq8-core/src/{fight,active,character,character_ai}.rs` (`character_dies`, `death_ended`, `drop_items`, `Corpse`, `die_calls`), `crates/kq8-data/src/characters.rs` (`FightSettings::type_name`), viewer `load.rs` (corpse shapes), `item_view.rs` (`CorpseDraw`, `corpse_draw`), `terrain_view.rs`, `main.rs`

| What | Source | Check |
| --- | --- | --- |
| Below 0: health 0; its die movement by the last hit's kind (`Die`; `projdie` if it has one, else `Die`; `firedie`; `icedie`) started at once; off the active list; gravity off; collision type and mask 0 (not solid); its state never changes again (`setState` and Move refused); no movement rules | `fighting.md` rule 30 | Unit test; viewer (goblin22) |
| A conversation it was in ends: logged only | Rule 30 (inferred in the note; none in Daventry) | — |
| `setState <name> Die now`: no movement, straight to rule 32 (kind: its last hit's, else `Normal`) | Rule 30 | Unit test |
| A hit on a dying character: health taken, others alarmed (rule 26), nothing else | Rule 21 | Unit test |
| When the die movement ends: Connor's experience as a stats event (fifth number; his levels apply), whoever killed it | Rule 32.1; `anim-scripts.md` rule 19 | Unit test; viewer: experience 0 → 3 at update 125, not at the death (94) |
| A corpse: `deadDts`; `Projectile`: `projdKqs` else `deadDts`; `Ice`: `idedKqs` else none; `Fire`: none; where it stood, turned the same way | Rule 32.2 | Unit test; viewer: `g2dead.kqs` drawn |
| The corpse is drawn by the viewer from shapes loaded with each fighter at the start; no shadow, not solid, no draw-distance test; **the skin swap is not applied** (only `bossprig` in Daventry) | Rule 32.2; **ours** for the draw-distance test and the skin (a known gap) | Viewer |
| Its script gets `Die <kind>` (after the update's characters, by its name) | Rule 32.3 | Unit test ("no script") |
| Removed: gone in core, the viewer hides it like a `deleteObject` | Rule 32.4 | Viewer |
| First kill not of type `bat`, `chicken`, `chicken2`: a tip 1.5 s later, logged only | Rule 33 with its Do instead (`chicken2`, not the typo `checken2`) | Unit test |
| The drops at the start of dying: logged here, thrown in 5c-8 | Rule 31 | Viewer |

### 5c-8, drops thrown — `crates/kq8-core/src/{item,fight,active}.rs` (`Flight`, `TOSS_*`, `drop_items`, `flights_step`), viewer `load.rs` (the drops' item models), `item_view.rs` (a flying item moves)

| What | Source | Check |
| --- | --- | --- |
| At the start of the dying, each drop entry (the line's `invDrop`, then the type's) becomes a new item (its quantity: `SilverCoins(3)` = one item of 3) at its feet (offset 0) | `fighting.md` rule 31 | Viewer: two mushrooms at update 94 |
| Thrown: heading 0–360°, 75°–85° up, 5,000–6,500 units/s | `items.md` rule 12; rule 31 (checked by the user in the original) | Unit test |
| The flight: position += velocity × 1/15 s, then gravity 8,085 units/s²; lands when it comes down to the floor-grid height, where it stays; its type's land sound | `items.md` rule 12 for the values; **stand-in** for the stepping (not traced: **Q137**) | Unit test; viewer: landed after 24–25 updates with `mushland.aud` |
| Clickable in the air (its click box is where it is); picked up in the air: the flight ends | Rule 31 (the user's check), `items.md` rule 4 | — (not run) |
| An entry naming an object already in the world: not handled (none in Daventry) | Rule 31 (not followed in the note) | — |
| The drops' item models are loaded at the start | Ours (as the scripts' items) | Viewer |

### 5c-9, the bow spriggans — `crates/kq8-core/src/{fight,active,character_ai}.rs` (`shoot`, `lead`, `Bolt`, `bolts_step`, `connor_was`, doMode 1), `crates/kq8-data/src/characters.rs` (`ProjectileKeys`), viewer `load.rs`, `door_view.rs`, `item_view.rs` (`bolt_draw`), `main.rs`

| What | Source | Check |
| --- | --- | --- |
| Holding a `LongWeapon` makes it ranged: doMode 1 (rule 25's distances) | `fighting.md` rule 34 | `fight.log` at load |
| Standing, it shoots within 1.25 × reach (5c-3) | Rules 7, 35 | Viewer: `sprigwnd1` at 3,046 (limit 7,500) |
| The projectile file (`Prscbolt.kq`): `shapeName`, `velocity` 8,000, `maxDistance` 18,000 | Rule 36; data | Unit test; viewer |
| At `Do`: a bolt at its target (Connor when alarmed); no reach or front test | Rule 36 | Viewer: first shot 177° off its heading |
| Aimed ahead of him: his speed (from his last two positions) and the bolt's give the meeting point | Rule 36 | Unit test; viewer (speed (91, −1,938, 0) used) |
| Leaves from 3/4 of the shooter's box height, aimed at 3/4 of his (1,300) | **Stand-in** (rule 36: default offset unknown, aiming height "not sure") | — |
| The flight: straight on, no gravity; the first character (not its shooter, solid, not dying), Connor, building, plain object or drawn ground in its way; gone after `maxDistance` | **Stand-in** (open question 2: the flight was not traced) | Viewer: hits Connor; hits the windmill's wall and a hay bale |
| The hit: `Projectile`, the shooter's ranged damage (`sprigwnd1` 11) | Rule 36 ("inferred from the original's log line") | Viewer |
| "Immune to projectiles" flag and `ProjImmune`: not built (which bit it is was not given; none in Daventry known) | Rule 36 | — |
| Drawn with its shape, turned to its heading only (no pitch) | **Stand-in** | — |
| Keeping distance / fleeing (rule 37): **not built** (only ranged characters with a chase radius; Daventry's three are guards) | Rules 37, 38 | — |

### 5c-10, the combat level — `crates/kq8-data/src/world_script.rs` (`Host::apply_combat_level`), `crates/kq8-core/src/{fight,active}.rs` (`combat_factor`, `Fight::apply_combat_level`, `set_combat_level`, scaling in `add_anims`), viewer `load.rs`, `anim_view.rs`, `door_view.rs`, `terrain_view.rs`, `main.rs` (`--combat-level`)

| What | Source | Check |
| --- | --- | --- |
| Factor 1.0 by default; `setCombatLevel 1|2|3` → 0.66 / 1.0 / 1.5 | `fighting.md` rule 3 | Unit test |
| `applyCombatLevel 1` in `newworld.cs` (run at load, before the objects): characters loaded after it get the factor when they enter the world: health, hand and ranged damage × factor, cut toward zero; protection and experience not; Connor never | Rule 3 | Unit test; viewer: goblin22 `[15, 2, 2, 0, 3]` → `[22, 3, 3, 0, 3]` at level 3, sprgnbow's −3 → −4 |
| The roll on characters 85–125 % below level 1.0 | Rule 16 (built in 5c-5) | — |
| `--combat-level 1|2|3` sets it before the characters enter | **Stand-in** for the menu (open question 3) | Viewer |

### 5c fix 1, the ready stance and the sword pointer — viewer `load.rs` (`weapon_sets`, the movement loop), `connor_view.rs` (`find`, `drawn`, `ready_set`, `weapon_set`), `door_view.rs` (`weapon_list`), `gui_view.rs` (`SWORD_POINTER`), `crates/kq8-core/src/connor.rs` (`MovementLoop::frame_at`)

After the user's check in the original (2026-10-20): readying the fists
did not show his fists-up stance nor the sword pointer.

| What | Source | Check |
| --- | --- | --- |
| With a hand weapon ready the pointer is a sword; Ctrl held gives the play pointer | `fighting.md` rule 39 (manual), `doors.md` rule 10; the user's check | Viewer: `gui.log` pointer `IDBMA_CURS03` at the ready, `IDBMA_CURS04` after |
| The sword is `IDBMA_CURS03` (`curs03.pba`) | `interface.md` rule 12 lists the attack cursors; **picked by its picture** (`kq8 bitmap-png`): `curs05` an arrow, `curs07` a rock or glove; **Q138 (d)** | — |
| An item in hand keeps its own pointer even with a weapon ready | **Ours** (Q138 d) | — |
| Each hand weapon's list in `Connor.kq`'s `ShortAnimList` whose weapon is in the world (`Fists` always) is loaded as a whole set, `<file>/<movement>`, with its transitions; its `draw` and `undraw` too | Data (`confists.anm` has 59 movements); **stand-in** for what readying does (Q138 a) | `viewer.log`: `confists.anm` loaded, 11 of his movements not in it (`push`, gestures) |
| With a weapon ready every movement the walker asks for is drawn and timed from the weapon's version where it has one (his flinch and deaths too) | The user's check (fists up when readied); **stand-in** (Q138 a) | `connor.log`: `walk (drawn: confists.anm/walk)`, `stop (drawn: confists.anm/stop)` |
| Readying plays the list's `draw`, putting away its `undraw`, as a one-item Special list, only from Wait / Move (else only the set switches); the set switches at the start of either | **Stand-in** (Q138 a, b) | `fight.log`: `draw` at update 30 (walking: he stops, then walks on in the fists set), `undraw` at 150 |
| A one-shot with speed −1 shows its frames last to first; its start phase is not used | `anim-scripts.md` rule 5 (start phase and speed from the `loop=` line), our walker starts at 0 (Q138 e) | Unit test |

### 5c fix 2, no readying during a jump; the put-away jitter — viewer `door_view.rs` (the refusal), `gui_view.rs` (the pointer), `connor_view.rs` (`phase_step`)

| What | Source | Check |
| --- | --- | --- |
| Readying or putting away asked during a jump is refused: the click is undone, the weapon stays as it was | The user's check in the original (2026-10-20, Q138 b) | `fight.log`: update 32 "asked in mode jump: refused" |
| The same while falling, landing, dying or dead | **Ours** (Q138 b) | — |
| The sword pointer follows what Connor took (his weapon set), not the click: one update later | **Ours** (no sword flash on a refused click) | `gui.log`: pointer at update 91 for a click at 90 |
| A one-shot's blend between updates goes forward even when it plays backwards | Fix of our own drawing (the user saw `undraw` jitter) | — (by eye) |

### 5c fix 3, the landing and the stumble; the weapon away before a conversation — viewer `connor_view.rs` (`may_ready`, `ready_set` hand mode), `door_view.rs` (the refusal, the plain click on a peaceful character), `load.rs` (`away`), `crates/kq8-core/src/conversation.rs` (`Stage::Away`, `Stage::Out`, `weapon_away`, `conversation_case`), `active.rs` (`getDoMode`), `connor.rs` (`SWORD_MODE`)

| What | Source | Check |
| --- | --- | --- |
| No readying or putting away during a stumble (`crash`) or a jump's landing (`jumpend`, and `jumprun` / `jumpstop`, which play with the keys choosing again) | The user's check in the original (2026-10-20) | `fight.log`: update 54 "the landing jumprun plays: refused" |
| Readying sets his hand mode to sword (0), putting away back to empty hands (4); `KQConner::getDoMode` answers it | `pushing.md` "Words used here" (sword 0, bow 1, empty 4), `inventory.md` rule 9 (`SwordMode`, the fists too) | — |
| A conversation with `<enemy>` 0 and his weapon drawn: `away` plays (the weapon's version), then bare-hand mode (the weapon no longer ready), `out`, then the case (rules 8–10); our 150-update guard on each | `conversation.md` rule 7; the user's check (the Wizard) | Unit test; `conversation.log`: `away` 32–38, `out` 39–44, the Wizard starts at 44 |
| With a weapon ready, a left click that picks a character with no fight part (the Wizard) is the plain click, not a swing | The user's check in the original (the Wizard); **ours** for which characters count (Q138) | — |
| His hand mode now also stops pushing and gestures while a weapon is ready | `pushing.md`, `conversation.md` rule 18 (already built; the mode was always empty) | The user's check: no pushing with a weapon drawn |

### 5c fix 4, the attack click's target — `crates/kq8-core/src/active.rs` (`pick_fighter`, `target_point` for characters), viewer `door_view.rs`

| What | Source | Check |
| --- | --- | --- |
| With a hand weapon ready the click line also meets the boxes (`BOUNDS.KQ`, as they stand) of characters that fight and are not dying, whatever their collision type; the nearest is his target unless a building is between the camera and it; else the old rule (a clickable character picked) | `fighting.md` rule 39 ("the clicked character becomes his target"); **stand-in** (Q139): by `clicking.md` rule 1 and `characters.md` rule 8 no creature is clickable | Viewer (`--aim goblin22`): hits for 11 and 8, goblin22 dies |
| `--aim NAME` also aims at a character's box | Test option (ours) | Viewer |
| A click near a creature but not on it misses | The user's check in the original (Q139 b) | — |

### 5c fix 5, Connor's hits on objects (the barrels) — `crates/kq8-core/src/active.rs` (`pick_attack`, `get_loc`), `fight.rs` (`object_at`, `connor_hits_object`), viewer `load.rs` (`object_colliders`)

| What | Source | Check |
| --- | --- | --- |
| Objects with a script are kept with their box even when not clickable (the barrels, type 0x141): they get `Init` and can be hit; the plain click still needs bit 0x20 | `clicking.md` rule 21 (every object with a script gets `Init`), rule 1 | Unit test; `viewer.log`: `Init` for the four barrels and the boards |
| The attack click also meets objects with a script and a box (not gone, collider in) | **Stand-in** (Q139 e); `anim-scripts.md` rule 12 (barrels die when hit in a fight) | Unit test; viewer: barrel.07 |
| Connor's swing on an object: rule 13's reach and front test; damage = his hand damage (× the big swing's factor), rolled 75–125 % with its 4th stat as protection, at least 1 | `fighting.md` rules 13, 41; the roll is a **stand-in** (`doors.md` rule 19: "random, at least 1") | Unit test |
| Taken off its health (`stats=`, all 0 without one); below 0 its script gets `Die <kind>` (again on every hit while below 0) | `doors.md` rules 19–20, `anim-scripts.md` rules 19–20 | Unit test; viewer: barrel.07 0 → −9, `barelup.cs` gets `Die Normal` |
| `KQObject::getLoc` also finds those box objects | `objects.md` "Script variables, getLoc and move" (any object) | Viewer: `getLoc barrel.07: 199536 262152 8185` |
| Fix 2026-10-22: a door with a script is also a target for the swing's hit (and a bolt's), at its position; same damage and `Die` as above, doors without `stats=` at 10,000 | `doors.md` rules 18–20; `ranged-weapons.md` rule 32 (doors, type bit 0x20, become the target) | Unit test `connor_s_hit_on_a_door_runs_die_below_0`; viewer: Daventry `tombdor2` health 1 → −24, `tombdor2.cs` opens it |

### 5c fix 6, a thrown item drawn where it landed — viewer `item_view.rs` (`item_draw`)

| What | Source | Check |
| --- | --- | --- |
| A world item's model follows every change of its place, not only while it flies (the update it lands in also ends its flight: it stayed drawn at its last place in the air, above its click box) | Fix of our own drawing (the user's check) | — (by eye) |

### 5c fix 7, the tumble — `crates/kq8-core/src/item.rs` (`Flight::spin`, `with_tumble`, `angles`, `TUMBLE_RATE`), `fight.rs` (`flights_step`), `active.rs` (`flight_rest`)

| What | Source | Check |
| --- | --- | --- |
| A thrown item tumbles slowly while it flies and lands in its normal resting pose | The user's check in the original (2026-10-20) | Unit test |
| Each axis turns at a random 90–180° per second, either way | **Ours** (Q137 e) | Unit test |

## Objects loaded while playing (M5 5d) — `docs/behaviour/objects.md`, `anim-scripts.md`, `items.md`

Plan in `docs/DESIGN.md` "5d". Written from the notes and the data; no
executable looked at.

### 5d-1, `KQAnimOnce` ends — `crates/kq8-core/src/active.rs` (the other animated objects' loop, `set_delete_when_done`), `named_anim.rs` (`delete_when_done`), viewer `load.rs` (`AnimatedAssets::once`), `anim_view.rs`

| What | Source | Check |
| --- | --- | --- |
| `classType=KQAnimOnce` marks the object's player as play-once (no waiting movement starts at its end) | `anim-scripts.md` rule 11 | — |
| At its movement's end, also for the other animated objects (`world.anims`, where runtime loads come in): `Die Normal` to its script, `Cue 300` to its `getEndLoop` listener, no `EndLoop`; deleted if `setDeleteWhenDone 1` | `anim-scripts.md` rules 11, 13, 20 | Unit test |
| `setDeleteWhenDone` finds a `KQAnimOnce` among them too | Rule 13 | Unit test |

### 5d-2, the name and place rules — `crates/kq8-core/src/active.rs` (`Template`, `TemplateKind`, `set_templates`, `load_copy`, `Request::Spawn`), `item.rs` (`next_name`)

| What | Source | Check |
| --- | --- | --- |
| The name word: missing or `same` = the file's section name; `@` = `NoName<n>` from the game-wide counter, given at the line; anything else = that name | `objects.md` "Where a world's objects come from" rule 2; `anim-scripts.md` rule 12; `items.md` rule 9 | Unit test |
| With x y z the place replaces the file's; the angles given, or 0 0 0; z −1 / −2 = the floor / surface height | `objects.md` rules 2, "Ground heights −1 and −2" 1–2 | Unit test (place, angles) |
| A name in use (live, or deleted in this same event) refused with an error line | `objects.md` rule 6 ("for us: log an error and refuse") | Unit test |
| `$ObjectName` = the new object's name | `anim-scripts.md` rule 12 | Unit test |
| The object is in core's world at once (an animated one with the template's player; a still one with a box as a box object) and its script gets `Init` during the line | `objects.md` rule 6 | Unit test |
| A file with a prepared copy and no name or place on the line still uses that copy (M3 3k-4) | Ours (kept until a later step) | Existing tests |
| The viewer is asked to draw it (`Request::Spawn`); not drawn yet in 5d-2 | Ours | — |

### 5d-3, still copies drawn — viewer `load.rs` (`TemplateAssets`, `template_files`), `door_view.rs` (`spawn_doors`: the still templates to core; `DoorSim::spawns`), `terrain_view.rs`, new `copies_view.rs` (`keep_templates`, `spawn_copies`)

| What | Source | Check |
| --- | --- | --- |
| A file named on a world script's `loadKQ` line and not loaded at the start with no place of its own is read and built at the start (hidden) as a template | `objects.md` rule 6 (the same steps as at the start); which files: data (B's `load_targets`) | `viewer.log`: 7 templates in Daventry (the wreck, three bursts, the boards' two and their wreck) |
| Its box from `BOUNDS.KQ` (by shape, then name), its collision type and mask as for any object | `clicking.md` rule 8, `characters.md` / `connor-physics.md` pair rule | `viewer.log` template lines |
| A copy is drawn from the template's model at its place; with a box and a type that collides with Connor it gets a collider, tested after all others | `objects.md` rule 6; the load order of the sweep (`connor-physics.md`, Q88) | `--command "loadKQ barelded.kq @ …"`: drawn, collider |
| A copy has no draw-distance test, lights, see-through settings or camera box of its own | **Ours** (stand-in until needed) | — |

### 5d-4, animated copies — viewer `anim_view.rs` (`Recipe`, `AnimSim::spawn_copy`, `spawn_animated` returns the unused players), `door_view.rs` (`spawn_doors`: animated templates), `copies_view.rs`, `terrain_view.rs`

| What | Source | Check |
| --- | --- | --- |
| An animated template's player as built at the start (not yet played) is core's template; a copy gets it with its own name and place and starts its movement as at the start | `objects.md` rule 6 ("an animated object starts its movement as at the start"), `anim-scripts.md` rule 3 | Viewer: barrel.07 |
| The copy is drawn from the template's frames (meshes and materials kept), played by core's object, at its place | Ours (drawing) | `viewer.log`: `copy NoName22 of barrelsm: drawn, 1 movement(s), 24 part(s)` |
| Its end reaches its script (5d-1), whose `deleteObject $1` hides it and `loadKQ barelded.kq @ $KQObject::loc` makes the wreck (5d-3) | `anim-scripts.md` rules 11–12 | `traps.log`: update 65 burst, 88 wreck |
| A copy of an unused character is not made (none in the data's runtime loads) | Ours | — |

### 5d-5, `KQObject::tossItems` — `crates/kq8-core/src/active.rs` (`toss_items`, `place_of`), `fight.rs` (`toss`, shared with the drops), `script.rs` (off the stub list); viewer `door_view.rs` (`--swing` at an object)

| What | Source | Check |
| --- | --- | --- |
| One item (quantity 1) per listed type, comma separated, `INVITEM_` optional, thrown from the object's place raised by `offsetZ`, each with its own flight | `items.md` rule 12 | Unit test; viewer: barrel.12's four crystals fly and land 1,271–2,304 away |
| An object that doesn't exist: an error line, nothing made | `items.md` "Edge cases" | Unit test |
| The object's place as `getLoc` finds it (any kind of object, also a copy loaded the line before) | `objects.md` (getLoc) | Viewer: `tossItems NoName22` (the burst) |
| `--swing NAME,N` can aim at an object (the middle of its box) | Test option (ours) | Viewer |

### 5d fix 8, a pushed object's own box moves — viewer `connor_view.rs` (`push_apply`)

| What | Source | Check |
| --- | --- | --- |
| A pushed object's box in the world's list (clicks, Connor's swings, `getLoc`) moves with its collider and model | Fix of ours (the user's check: a pushed barrel was hit, and burst, where it had stood); `pushing.md` (the object moves) | Viewer: barrel.04 pushed to 208,225, hit there, burst and wreck there |

### 5d fix 9, the dagger's (and every weapon's) movement set — viewer `load.rs` (`weapon_sets`, `out` loaded), `connor_view.rs` (`ready_set`)

| What | Source | Check |
| --- | --- | --- |
| A hand weapon's whole set is also loaded when a world script names the weapon (Connor is loaded before `WorldInv.cs` makes the dagger) | Ours (the user's check: the dagger had no stance); which sets: data | `viewer.log`: dagger, small axe, Lake sword, fists |
| Readying plays the list's `draw`, else `out`; putting away its `undraw`, else `away` | **Stand-in** (Q138 a): data (`out` = `cd_draw` / `ca_draw`, `away` = `cd_shth` / `Ca_shth`, with their sounds); `conversation.md` rule 7 (`away` puts away) | Viewer: the dagger picked up, `condag.anm/out`, `stop` drawn from `condag.anm`, `away` |
| The alchemist's boards refuse the fists and the dagger | `anim-scripts.md` rule 12 (`boardup.cs`); the user's check | — |

### 5d fix 10, drawing and putting away in two parts; picking a weapon up while ready; no clicks meanwhile — viewer `connor_view.rs` (`ready_set`, `switch_set`, `weapon_step`, `weapon_target`, `may_ready`), `door_view.rs`, `gui_view.rs`; `crates/kq8-core/src/connor_inventory.rs` (`receive`)

| What | Source | Check |
| --- | --- | --- |
| A weapon other than the fists is put away with its `away`, then his own `out`; drawn with his own `away`, then its `out`; from one weapon to another: the old one's put-away part, then the new one's draw part; the movement set switches when the second part starts | `conversation.md` rule 7 (`away`, bare hands, `out`), data (`c_shth` forwards / backwards in `conner.anm`); **stand-in** (Q138 a); the user's check (the dagger's draw and sheath were cut short) | `connor.log`: put away 150–174, drawn 210–232 |
| The fists keep `draw` / `undraw` | The user's check (fix 1) | — |
| His own `away` and `out` (`Conner.anm`) are loaded in every world (they were loaded only where a script named them: Daventry's `urn1.cs`, `wheeltp1.cs`), so every weapon's draw and put-away has both parts | The user's check (2026-10-21: Swamp's sword and hammer started with the hand at the holster) | Viewer, Swamp: `fight.log` `["away", "conkng.anm/out"]`, `["conkng.anm/away", "out"]`, the same for `conhamr.anm` |
| A hand weapon picked up while one is ready becomes the ready one (it replaces the old one in its slot, rule 8), straight into its stance: nothing plays (fix 11) | The user's checks in the original | `items.log`: "INVITEM_Dagger is ready now"; `fight.log`: "at once, nothing plays" |
| No slot click while a draw or put-away plays (undone like one during a jump) | The user's check in the original | `fight.log`: update 215 refused |
| The sword pointer follows the weapon asked for (at the click) | Ours | — |

### Test switch: AI off — `crates/kq8-core/src/character_ai.rs` (`alarm_rules`, `alarm`), `fight.rs` (`hit_unalarmed`), `active.rs` (`ai_off`); viewer `door_view.rs` (`AiOff`, `toggle_ai`, `flip_ai`), `gui_view.rs` (`--key f4`), `main.rs` (`--no-ai`)

| What | Source | Check |
| --- | --- | --- |
| With the switch on no character is alarmed (seeing Connor, being told, being hit); an alarmed or hunting one gives up as rule 28 has it (home, or back into the ground) | Ours (a test switch, not in the original); giving up: `fighting.md` rule 28 | Viewer: goblin22 hits Connor 9 times with it off, 0 with `--no-ai`; `--key f4,100`: it gives up at update 101, no hits to update 759 |
| F4 flips it; `--no-ai` starts with it on | Ours | Viewer |
| An on-screen line under the F8 line: "AI: on (F4)" in white, "AI: off, creatures ignore Connor (F4)" in yellow | Ours | The user's check (2026-10-20) |

### Test switch: god mode — `crates/kq8-core/src/fight.rs` (`hit_connor`), `connor_mover.rs` (`no_fall_death`), `active.rs` (`god_mode`); viewer `door_view.rs` (`GodStart`, `toggle_god`, `flip_god`, the on-screen line), `gui_view.rs` (`--key f3`), `main.rs` (`--god`)

| What | Source | Check |
| --- | --- | --- |
| With it on, a hit on Connor (a swing or a bolt) does nothing: no health lost, no flinch, no death | Ours (a test switch, not in the original) | Viewer: goblin22's 4 hits logged as "nothing, god mode is on" |
| A fall past the death limit (`connor-physics.md`, 3,840) is a safe landing | Ours | Unit test; viewer: a 7,100 fall kills him without it, a safe landing with it |
| F3 flips it, `--god` starts with it on; a line under the AI line shows it (yellow when on) | Ours | The user's check (2026-10-20) |

### 5e-1, thrown items: the arc and what stops it — `crates/kq8-core/src/item.rs` (`Flight`, `FlightStep`, `ITEM_OBSTACLE_BITS`), `fight.rs` (`toss`, `flights_step`), `active.rs` (no longer flies them in the update); viewer `door_view.rs` (`character_gravity` flies them)

Replaces 5c-8's stand-in flight. The tumble stays fix 7's stand-in until 5e-3.

| What | Source | Check |
| --- | --- | --- |
| Launch: heading 0–6.28 rad, 75°–85°, 5,000–6,500 units/s; launch point kept, clock from 0 | `thrown-items.md` rule 3 | Unit test |
| Each update (1/15 s) the wanted position is the exact arc from the launch point: height = vz·t − ½·8,085·t² | Rules 4, 5 | Unit test (on the arc within 0.5 until the first touch) |
| The 25-unit first step | Rule 6 | — (never reached at these speeds) |
| "Coming down" once the wanted height is below the height at the start of the update | Rule 8 | `items.log` |
| The move is the characters' mover (three passes, the slide of an animated object, 3 retries, never head-on), with the item's `BOUNDS.KQ` box, type 0x220, mask 0x809, steps 0, gravity off in the arc, the floor grid in the passes, the raise onto the floor grid after each move | Rules 7, 11, 12.1, 15; `items.md` rule 4 | Unit tests (wall, building floor) |
| Up-pass hits stop and don't slide (the mover's rule for everyone); the note's 12.1 says a non-upward surface in the up/down part slides | **Ours**: kept the mover's `connor-physics.md` rule; the arc ends at that touch anyway, so it only changes where gravity takes over within one update | — |
| What it meets: the floor grid, every building, objects whose type has bit 0x1 or 0x100; not characters, Connor or other items | Rule 11 | Viewer: crystals touch `Pub` and `terrain` only |
| The land sound at the first touch of anything, once | Rule 12.2 | Unit test; `items.log` (`potnland.aud`) |
| Every touch ends the arc (marked landed and coming down, ordinary gravity on); scripted / animated objects throw it again in 5e-2 | Rule 12.3 ("anything else"); **5e-1 only** for scripted objects | Viewer: NoName25 hits the pub at z 9,817 and drops straight down to 8,160 |
| The thrower's own check: coming down with the wanted height below the floor grid ends the arc | Rule 14 | Unit test (no-box item); viewer |
| After the arc: wanted = where it is, the movers' ordinary gravity | Rule 15 | Viewer: drops 36, 108, 180, … per update |
| The flight ends at the start of an update when coming down and unchanged in whole units since the last, or after 10 s; "whole units" cut toward zero | Rule 16; the rounding is **ours** (the note doesn't say) | Unit tests; viewer: ends one update after it is held |
| No `BOUNDS.KQ` box: nothing tested on the way (not even the floor grid in the passes) | "Edge cases" | Unit test |
| Flights run after the characters' movers, with the same buildings and objects | Ours (the note gives no order) | Viewer |

### 5e-2, thrown again — `crates/kq8-core/src/item.rs` (`Flight::throw_again`, `launch_velocity`), `fight.rs` (`flights_step`: what throws again)

| What | Source | Check |
| --- | --- | --- |
| A touch on anything with a script, or on any animated object, throws it again from the touch point: fresh heading, elevation, speed and tumble, clock 0, not coming down, landed flag cleared, the arc's gravity again | `thrown-items.md` rule 12.3 | Unit test; viewer: crystals on barrel.13 |
| "Animated": doors, chests, the scripted and animated objects' lists (also play-once copies); "with a script": also clickables and buildings with `script=` | Rule 12.3; `anim-scripts.md` rule 8 (doors, chests and the rest are animated objects) | — |
| Touches in one update are handled in the order the mover found them; a later touch in the same update can end the new arc | Rule 12 ("at a touch"); the order within one update is **ours** (the note's edge case says the rest of the move still runs) | — |
| The land sound stays once per flight (not again after a re-throw) | Rule 12.2 ("only the first time ... in its whole flight") | Unit test |
| The new tumble is fix 7's stand-in until 5e-3 | Ours (stand-in) | — |

### 5e-3, the tumble — `crates/kq8-core/src/item.rs` (`TUMBLE`, `Flight::spin`, `turn`, `FlightStep::angles`), `fight.rs` (`flights_step`, `toss`), `active.rs` (`flight_rest` gone)

Replaces 5c fix 7's stand-in (three axes, 90–180° per second, back to the turn it had before).

| What | Source | Check |
| --- | --- | --- |
| A thrown item starts with no turn at all (0, 0, 0) | `thrown-items.md` rule 2 | Unit test |
| The tumble is one random number, −0.5 to +0.5 rad **per update** (not scaled by time), drawn at each throw and each throw again | Rules 3, 9, 12.3 | Unit test |
| Each update until the flight ends, also after the first touch, it turns by the tumble about its own x axis (angle 0 of `[x, y, z]`; with y and z 0 that is a turn about the model's own x axis) | Rule 9; the angle order is `kq8_data::objects::rotation` | Unit test; `items.log` ("turned … rad") |
| In an update with a throw again, the turn uses the new tumble | **Ours** (the note gives no order within the update) | — |
| When the flight ends: angles 0, 0, 0 | Rule 10 | Unit test |

## Ranged weapons (M5 5f) — `docs/behaviour/ranged-weapons.md`

Plan in `docs/DESIGN.md` "5f". Written from the note (A, 2026-10-20,
checked by the user in the original), `connor-collision.md` rules 18–19,
`connor-physics.md` and the data; no executable looked at.

### 5f-1, the bolt's flight — `crates/kq8-core/src/fight.rs` (`Bolt`, `aim_direction`, `bolt_kind`, `shoot`, `launch_bolt`, `bolts_step`, `bolt_hits`, `object_takes`), `active.rs` (`immune_calls`), `character_ai.rs`, `crates/kq8-data/src/characters.rs` (`ProjectileKeys`), viewer `load.rs` (`load_projectile`), `item_view.rs` (`bolt_draw`)

Replaces 5c-9's stand-in flight. 5c-9's rows for the shot itself (doMode 1, 1.25 × reach, no reach or front test, the lead) are confirmed by rule 25 ("led for Connor's movement").

| What | Source | Check |
| --- | --- | --- |
| A projectile file's keys: `velocity` (default 2,000), `maxDistance` (8,000), `colliderMask` (0x1F), `weaponType` (2 Fire, 3 Ice, else `Projectile`), `meshName`; its own type 1 | Rules 26, 30 | Unit test; `characters.log` at load (`Prscbolt.kq`: 8,000, 18,000, 0x1F, part `arrow`) |
| A character aims at its target + 0.75 × the top of the target's box: Connor's `BOUNDS.KQ` box top 1,300 → 975 | Rule 25 (the formula); the note's example says 1,600 → 1,200, the data says 1,300: **Q140 b** | `fight.log`: "Connor + 975" |
| The bolt is turned once to the aim point: heading from the level direction, pitch = asin(height difference ÷ straight distance), no roll; kept for good | Rule 27 | Unit test (pitch −11.31° for 1,000 down over 5,000); viewer: −3.8° at the windmill |
| Each update it moves speed ÷ 15 along it (8,000 → 533), no gravity | Rule 28 | Unit test; `fight.log` |
| Its own shape: its `BOUNDS.KQ` box as a ball stack (`scbolt.kqs` → one ball, radius 235, 235 above it); no box: a point | Rule 29 ("its model's `BOUNDS.KQ` box", "an ordinary collision move"); `connor-collision.md` rule 18 (the movers' balls); **our reading: Q140 a** | Unit test; `fight.log` at each shot |
| It meets the buildings (mask bit 0x8, by their `.dig` solids), the objects and characters whose type shares a bit with its mask (boxes, cylinders), Connor (type 4); never its shooter; switched-off objects and characters whose collider is off are not met | Rule 29; the switched-off / collider-off part is **ours** (as for the other movers) | Unit tests; viewer: the windmill's wall, Connor |
| A start inside a character's cylinder (or Connor's) is a hit; a start inside an object's box too | Rule 29; `connor-collision.md` rule 19 ("Projectiles always count a start inside"); `connor-physics.md` (a box: "counts as a hit") | — |
| The nearest touch along the move counts; the bolt is put at the touch point (before the 25 pull-back) and removed | Rule 30 | Unit test (touches gob at y −1,035) |
| After a free move: at or below the floor grid → removed, no damage; then more than `maxDistance` travelled → removed (after 34 updates for 18,000) | Rules 28, 29 | Unit tests |
| A character or object with type bit 0x4000: a hit of 0 and `ProjImmune` to its script; otherwise a character gets a hit of the bolt's damage and kind (`fighting.md` rules 16–33), Connor rule 29 | Rule 30 | Unit test (0xC021) |
| An object without the immune bit: the barrel stand-in of 5c fix 5 with the bolt's damage, only for objects with a script; others: nothing | Rule 30 (it gets the hit); how objects take damage: **stand-in, Q139 e** | — |
| No stick, no bounce, no sound from the bolt | Rule 30 | — |
| A bolt's damage is the shooter's ranged damage at the shot | Rule 22 | `fight.log` (`sprigwnd1` 11) |
| Drawn turned by pitch then heading, its model's front (+y) along the flight | Rule 27 (the model's front +y inferred by A from `BOUNDS.KQ`) | — (not looked at) |
| It still leaves from 3/4 of the shooter's eye height | **Stand-in** until 5f-2 | — |

### 5f-2, where a character's bolt starts — `crates/kq8-data/src/shape.rs` (`Shape::part_places`), `crates/kq8-core/src/door.rs` (`DoorMovement::frame_parts`), `fight.rs` (`Fight::shoot_offset`, `shoot`), viewer `load.rs` (`Animation::frame_parts`), `anim_view.rs`

Replaces 5f-1's stand-in start (3/4 of its eye height).

| What | Source | Check |
| --- | --- | --- |
| A model part is a mesh node with a name; its place in a frame is the node's translation in that sequence (before any frame step) | Rule 24 ("the part's translation in the `.kqs` frame, before any frame step"); the node / name layout from the `.kqs` format notes (models provenance above) | `anim.log`: `sprgnbow` `attack` frame 2 = (−55, −1,367, 954), the note's number |
| The part looked for is the bolt file's `meshName` (`arrow`), any capitals | Rule 24 | — |
| The frame used is the one at the `do` cue's own phase (`spbattck`: 0.125 × 18 → frame 2), not the frame shown one update later (frame 3) | Rule 24's example value; **our reading, Q140 c** | Unit test; `fight.log`: "frame 2 of 'Attack'" |
| Found once, kept for every later shot; not found: its own `shootOffsets` (not read: none in Daventry), 0, 0, 0, and looked for again at the next shot | Rule 24, "Do not reproduce" (keep it); looking again until found is **our reading** of "0, 0, 0 until the first shot finds the part" | Unit test |
| Start = its position + the offset turned by its rotation (row vector, our usual) | Rule 23 | Unit test (gob facing −y: start (−55, −1,367, 1,054)) |
| A start inside an object's box is a hit at once (the first shot from `sprigwnd1`, still turning, starts inside the fence `Rfence`) | 5f-1's rule | `fight.log` |

### 5f-3, weapon modes and the swap — new `crates/kq8-core/src/weapons.rs` (`WeaponMode`, `Ask`, `decide`, `take_weapon_request`, `set_weapon`, `ranged_weapon`), `active.rs` (`weapon_mode`, `last_away_ranged`, `weapon_requests`, `KQConner::setDoMode` / `getDoMode`), `connor_inventory.rs` (`bar_click`), `conversation.rs`, `script.rs` (no longer a stub); viewer `connor_view.rs` (`Controls`, `ready_set`, `switch_set`, `may_ready`), `door_view.rs` (`weapon_list`, the requests), `gui_view.rs` (the pointer, `--key delete` / `alt`)

Replaces 5c fixes 2–3 and 5d fixes 9–10's stand-ins (Q138) where the note differs.

| What | Source | Check |
| --- | --- | --- |
| Three modes, hand 0, ranged 1, none 4; start none | Rule 1, "Words used here" | Unit tests |
| A weapon slot asks for its item's mode column (`SwordMode` hand, `BowMode` ranged, `HandMode` none, `NoModeChange` nothing); Delete = `Draw`, Alt = `Away` (keys from `Connor.sae`); all ignored under `handsOff` | Rule 1 | Unit tests; viewer `--key delete,N`, `--key alt,N`, `--key left:IDCTL_INVBAR_SHORT,N` |
| Delete: hand → ranged only with a real ranged weapon; ranged → hand; none → the one put away last (hand at the start) | Rule 2 | Unit tests; viewer: update 60, "nothing" |
| Alt: put away, remembering if it was the ranged one; nothing ready: nothing. Any change to none remembers the same | Rule 3; "rule 6's putting away" in rule 2 | Unit tests |
| The mode already ready asked again → none | Rule 4 | Unit test (slots) |
| A request for ranged without a real ranged item asks for the hand weapon instead (not the original's `NoRanged` counting) | Rule 4, "Do not reproduce" (the Do instead) | Unit tests |
| Taken only in Wait / Move (our Free mode, not during the landings `jumprun` / `jumpstop` nor while a swap plays); otherwise dropped, not kept | Rule 5; the landings: the user's checks (5c fixes 2–3) | Unit test; viewer: Alt during a jump "dropped (mode jump)" |
| A request while a swing plays waits for the swing's end | **Ours**: open question 1 (cut or wait: not traced); our swings play in the Special state, the original's in Wait / Move | — |
| The swap: none → Fists: list first, then `draw`; Fists → none: `undraw`, then the list; from a hand weapon with Shift: `away2` (else `away`); otherwise `away` in the list in use, the switch, `out` from the new list; then standing | Rule 6 (replaces 5d fix 10's "draw / undraw whenever the list has them") | Viewer: case 1 and case 2 logged; cases 3–4 in 5f-4 |
| The movement playing is asked again by name in the new list (our movements are looked up by name in the set in use) | Rule 8 | — |
| The pointer by the mode at the switch: hand `IDBMA_CURS03`, ranged `IDBMA_CURS05`, none `IDBMA_CURS04`; Ctrl with a weapon ready: the play pointer; an item on the pointer: its own | Rules 11, 12 (the names from their order: open question 4) | — |
| `KQConner::setDoMode 0|1|4`: a request that needs no state (he is put into the swap from Special, Push or Talk too); the optional movement word is logged, not used; `getDoMode` gives the mode's number | Rule 9 (no Daventry script gives the word) | Unit test; viewer `--command "KQConner::setDoMode 4",150` |
| A jump under way is not ended by `setDoMode` (the set switches at once instead) | **Ours** (rule 9 says the jump is ended: not built) | — |
| A conversation's put-away, a weapon picked up while one is ready, the `--swing` test: the mode follows the item, the set follows (5d fix 11: at once) | `conversation.md` rule 7; the user's check (5d fix 11) | Viewer: `--swing goblin22` run |

### 5f-4, the crossbow readied — `crates/kq8-core/src/weapons.rs` (`set_connor_fight`, `take_ranged_item`), `fight.rs` (`ConnorFight::long_lists`, `shoot_offsets`, `start_projectile`, `start_offset`), `active.rs` (`connor_projectile`, `connor_shoot_offset`), `connor_inventory.rs` (`receive`); viewer `load.rs` (`Connor.kq`'s keys, `weapon_sets`, `special_movements`, `load_corpse_shape`, `item_models`), `door_view.rs` (`weapon_list`)

| What | Source | Check |
| --- | --- | --- |
| With the ranged mode ready his movement list is the ranged item's from `Connor.kq`'s `LongAnimList` by its name (`SmCrossBow` → `conscbow.anm`); a movement the list lacks still falls back to his own (as for the hand weapons, 5d fix 9) | Rule 10 ("the whole list is replaced"); the fallback is **ours** (kept from 5c / 5d for movements like `push` and the gestures, which the weapon lists don't have) | `viewer.log`: `conscbow.anm/stop` and the rest loaded |
| The list is loaded when the world's files name the item (`MONSTERS.KQ`'s drop), a script names it, or it is placed; `NoRanged`'s list is his own | Ours (what to load ahead) | `viewer.log`: "weapon set conscbow.anm (SmCrossBow)" |
| The swap's movements from that list: `away`, `away2`, `out`; its `attack` for 5f-5 | Rules 6, 20; data (`conscbow.anm`) | `fight.log`: `conscbow.anm/away`, `conscbow.anm/out` |
| A `LongWeapon` going into his inventory: its projectile file (the item list's last column) becomes his; its `shootOffsets` line (any capitals) his offset; no line: kept; `NoRanged`: no bolt, `INVITEM_All` (no line) | Rules 17, 21 | `fight.log`: Small Crossbow → `Prscbolt.kq`, (−315, −327, 1,027) |
| His start values: `projectileFile`, `shootOffsetX/Y/Z`; `shootDamage` not read | Rule 21, "Do not reproduce" | `viewer.log` |
| Picked up while the ranged mode is ready: the new item is ready, its list at once, no swap | "Edge cases" | — (not run) |
| The bolt files and shapes of every ranged weapon that can appear in the world are read at load | Ours (as for the characters' bolts) | `characters.log`: "[Connor's INVITEM_SmCrossbow]: shoots Prscbolt.kq" |
| Bow pointer `IDBMA_CURS05` with the ranged mode ready | Rule 11 (5f-3) | `gui.log`: CURS05 at update 37 |

### 5f-5, the standing shot — `crates/kq8-core/src/weapons.rs` (`mask_pick`, `steepness_limit`, `connor_shoot`, the pick constants), viewer `door_view.rs` (`ShotClick`, the click, the shot in `door_step`), `connor_view.rs` (`Shot`, `shot_click`, `ask_shot`, `shot_step`, `part_at`, `turn_matrix`, `DrawnMovement::parts`, the `do` cue), `main.rs` (`--aim-point`)

| What | Source | Check |
| --- | --- | --- |
| A world click: an item on the pointer is used whatever is ready (also now for the swing click); then hand → the swing click, ranged → the shot click, none → the plain click; Ctrl with a weapon ready → the plain click | Rules 12–14 | Viewer runs |
| The talk pick: the click line against characters, objects and buildings whose type has bit 0x40, 0x20 or 0x08; the first is a character with bit 0x8000 (Wizard, Knight 0xC021) or an object named `icewater` → the plain click | Rule 15 | Viewer: `--aim Wizard`: "talk pick meets Wizard (type 0xC021): the plain click", the conversation starts |
| The aim pick: types with bit 0x40 or 0x08 (characters' `BOUNDS.KQ` boxes, objects' boxes, all buildings by their solids); else the drawn ground; else the line's far end (1,000,000, our click line length); a character: its x, y and the height where the line met its box | Rule 16 | Viewer: goblin22, the windmill (type 0x28), the drawn ground |
| The right button held (the camera's kept point): not done | Rule 16, open question 2 | — |
| The steepness limit: *h* = aim height − (feet + the shoot offset's z), *d* the level distance; *d* < tan(1) × \|*h*\| → *d* = tan(1) × \|*h*\| (same direction, same height) | Rule 18 | Unit test (≈ 32.7° down); viewer: "moved out" |
| He turns toward the aim point with the automatic turn (3.5 rad/s; we are always in third person); more than 45° off: the shot is asked when 3 updates of turn or fewer are left (≤ 0.70 rad), lost if the turn is replaced or cancelled; within 45°: at once, he keeps turning | Rule 20 | Viewer: −133° off, asked with 0.69 rad left |
| Standing: `attack` from the ranged list only when his movement can be cut short (a click during `attack` does nothing, but the click's turn still happens); when `attack` ends, `stop`; no extra delay; no ammunition | Rule 20 | Viewer: a second click during `attack`: "can't be cut short: no shot" |
| A click while moving: not a shot yet (5f-6) | Rule 20 "Moving" | — |
| At the `do` cue the bolt starts at his position + his full turn (lean, then heading) × the bolt file's part (`arrow`) in the frame at the cue's phase (`conscbow.anm/attack` frame 2: (−231, −832, 1,067)), which is kept as his shoot offset; no part: his shoot offset; no bolt file (`NoRanged`): no shot | Rules 21, 23, 24; the frame at the cue's phase as for characters (Q140 c) | Unit test; viewer: (−231, −832, 1,067), the note's number |
| Aimed at the aim point, no lead; his ranged damage at the shot (5 with the Small Crossbow at level 1); the bolt is his (`Connor`): never hits him | Rules 22, 25, 29 | Unit test; viewer: goblin22 15 → 11 → 7 |
| The crossbow's sound is its `attack`'s own `playsound Crossbow.aud` cue | Rule 20 (data) | — |
| `--aim-point X,Y,Z,N`: a click along the camera's line to a point | Ours (a test option) | — |

### 5f-6, the moving shot — viewer `connor_view.rs` (`ask_shot`'s moving branch, `MOVING_SHOT_DELAY`, `next_moving_shot`, `run_back_step`, `moving_aim`), `door_view.rs` (the use sound, the moving shot's bolt)

| What | Source | Check |
| --- | --- | --- |
| Moving = in our Free mode with a forward / back key (the original's Move state: walking or running with the keys) | Rule 20 "Moving" | Viewer `--hold` |
| No `attack`; the shot's turn and its wait (more than 45°) as standing | Rule 20 (the turning part comes before "Standing" / "Moving") | Viewer: the shot at update 63 after a turn |
| If his `attackDelay` (0.5 s, the default; `Connor.kq` has none) has passed since the last shot, the bolt leaves at once from the part in the frame playing now (`walk` frame 4: (−135, −588, 868)); the next moving shot 0.5 s later; otherwise nothing | Rule 20; data (`scbwalk.KQS`'s part) | Viewer: update 70 "less than 0.5 s ago: nothing"; the offset within the note's range |
| The ranged item's use sound (`crossbow.aud`, the item list's second sound column) as a flat one-shot | Rule 20 | `sound.log`: "Connor's moving shot", plain |
| A moving shot while running switches the run off (he walks); it comes back on 1 s after the last moving shot | Rule 20; the simpler way allowed by "Do not reproduce" | Viewer: walk at update 72, run on again at 87 |
| The end of a standing `attack` sets the next moving shot 0.5 s later | Rule 20 | — |
| Our clock for the delay: his updates ÷ 15 | Ours | — |

### 5f-7, the swing click's target — `crates/kq8-core/src/weapons.rs` (`SWING_PICK`, `mask_pick`), viewer `door_view.rs` (the swing click, `SwingClick`, `--swing`), `connor_view.rs` (`swing_click`, `swing_step`, `melee_target`, `SWING_TURN_UPDATES`)

Replaces 5c fixes 3–5's stand-ins (Q139): the fighters' boxes on the line, "a character that doesn't fight gets the plain click", "behind (90°) waits for the whole turn".

| What | Source | Check |
| --- | --- | --- |
| The click line against characters, objects and buildings whose type has bit 0x40 or 0x20 (characters 0x42, plain objects 0x41, barrels 0x141, clickables 0x21, the clickable buildings 0x28, not 0x88) | Rule 32 | Viewer: goblin22, barrel.12 |
| A character with bit 0x8000 (Wizard, Knight) or `icewater`: the plain click | Rules 15, 32 | — (as 5f-5's talk pick) |
| Nothing or a building: he turns toward the click point and swings at once, no new target | Rule 32 | Viewer: update 90, "nothing or a building picked: his target stays goblin22" |
| Where "the click point" is when the line meets nothing: where it meets the level of his feet | **Stand-in** (kept from 5c-5; the note doesn't say) | — |
| Anything else: its position within his `attackDist` (2,350, straight 3D distance between the two positions) → his target; he turns toward its position; more than 45° off: the swing waits until the turn has 8 updates or fewer to go (≈ 1.87 rad), else it starts at once | Rule 32 | Viewer: 130° off, the swing starts with 1.80 rad left |
| The target stays until another pick within reach replaces it; the hit test at `Do` is `fighting.md` rule 41's with it | Rule 32 | Viewer: the second swing hits goblin22 |
| Objects get the hit only through the barrel stand-in (objects with a script) | Q139 e (not traced) | Viewer: barrel.12 bursts |
| An item on the pointer wins over the swing click | Rule 13 (5f-5) | — |
| Fix 2026-10-24: the swing pick (and the shot / aim picks) also test the click boxes the plain click tests (doors, chests, animated objects, clickables, items as 0x21; `box_hits`, now shared with `pick`), so an object with a click box but no solid box (Daventry's `bell`, 0x8021, brought in by `loadKQ`) is met | Rule 32 (clickables 0x21 are in the swing pick); `clicking.md` rule 2 | Unit test `the_swing_pick_meets_a_click_box_with_no_solid_box`; viewer: "the swing pick meets object bell (type 0x8021)", the fist hits it, `bell.cs` gets `Die` |

### After 5f: the axe cutscene — viewer `load.rs` (`dormant_traps`), `door_view.rs`, `connor_view.rs` (`ready_set`); `crates/kq8-core/src/active.rs` (`load_kq`, `start`)

| What | Source | Check |
| --- | --- | --- |
| A trap prepared for a script's `loadKQ` is built at load as any trap (box, mask, script), switched off; the `loadKQ` switches it on; its `Init` comes at the `loadKQ`, not at the start | `objects.md` rule 6 (a runtime `loadKQ` does the same as at the start); `traps.md` rules 1, 2, 13 | Unit test; viewer: `Henchtp: On Connor` at update 41 |
| A script's weapon request while a scripted walk runs: the swap plays and the walk waits; it starts when the swap is over | Q141, answered by the user's check in the original (2026-10-21: "he should stop, draw the axe and then walk outside"); fits `ranged-weapons.md` rule 7 (the swap can't be left until it ends) | Viewer: draw at 42–58, the walk 58–84, the cutscene to its end |
| A flight's last update adds its step like the others (exactly on the spot when undisturbed; within half a unit taken as the spot); after a `move KQCamera` during the flight it ends away from the spot | `camera.md` rules 4, 6 (replaces our snap to the spot on the last update) | Unit test; viewer: the axe cutscene's flight ends at (137350, 249919, 16445) |

## The command console — `crates/kq8-viewer/src/console.rs`

Our own developer tool (the user's request, 2026-10-21); not from the
original game and from no behaviour note. It only hands typed lines to
the script commands already built (each keeps its own source above:
`give` from `items.md` rule 9, `move` from `objects.md` "(d)"
and `anim-scripts.md`, and so on). The results shown are
`$Console::Return` as those commands already set it (`doors.md` "Q45"
(e)). `help` and `where` are ours. The typing line uses the `crossterm`
library (MIT) and standard terminal control codes (scrolling region,
save / restore cursor).

| Console word (`console_words.rs`) | Goes through | Source of that rule |
| --- | --- | --- |
| `list items` / `weapons` / ... | The item list's `category` column | `Invinfo.itl` (observed), `items.md` |
| `heal`, `xp <n>` | `sendEvent 0 Connor KQStatsChangeRequestEvent` | `anim-scripts.md` rule 19 (level-ups: the stats rules already built) |
| `god`, `ai` | The F3 / F4 test switches | Ours |
| `goto <name>` | `move Connor 0`, 1,000 away, 500 up, turned toward it | `objects.md` "(d)"; heading as `connor_auto.rs` |
| `spawn <item>` | `KQMonster::inventory world` | `items.md` rule 9 |
| `kill` (`fight.rs` `console_kill`) | The death of a hit (die movement, drop, body, experience), without the hit's alarm | `fighting.md` rules 30–32 |
| `help` examples: `KQDoorEvent 1`/`2`, `setCombatLevel 1|2|3` | Existing commands | `doors.md` rules 2–3, `fighting.md` rule 3 |

## M5 5g: health items and potions — `crates/kq8-core/src/connor_inventory.rs`, `potions.rs`; viewer `gui_view.rs`

Source: `docs/behaviour/potions.md` (A, 2026-10-20; checks 2–6 done by
the user in the original), with `inventory.md` rules 8–18 and
`interface.md` rules 20–33. No other source.

| What | Source | Check |
| --- | --- | --- |
| 5g-1: a slot click or key 1–8 is refused only in handsOff and while Connor is dying or dead; nothing, no sound, no remark | `potions.md` rules 2, 5 | Unit test `the_last_dose_runs_at_count_0_and_dying_refuses` |
| 5g-1: the same potion while it runs: nothing (no sound, count kept, timer untouched) | Rule 7 | Unit test; viewer: `--key 6` twice, `items.log` "already running" |
| 5g-1: the use sound plays only when the item takes effect, not placed | Rule 4 | Viewer: `healing.aud`, `strength.aud` in `items.log` |
| 5g-1: the last dose stays at count 0 until its time is up; its slot is empty and not clickable; a new one picked up joins it | Rule 8 (the join inferred there) | Unit test |
| 5g-1: a health item at full health is still used (one taken, sound) | Rule 6 | Viewer: second mushroom at health 10 |
| 5g-2 (`potions.rs`): whole seconds from `Dur`, one off every 15 updates, over when it would drop below 1 (exactly `Dur` s) | Rule 33 | Unit test; viewer: Strength from update 30 to 330 |
| 5g-3 (`potions.rs` `take_place`, viewer `status_specs`): a potion's icon (`extraData`) in the first free status place, `IDCTL_TIMER_1` first; freed at the end | Rule 35; place order from `Gplayscr.gui` (observed: `_1` at x 288, the right-most) | Unit test; viewer: Strength `_1`, shield `_2` |
| 5g-3: the icon drawn in two parts split at percent = left × 100 / `Dur`: the lower rows (48 × percent / 100) from the `.pba`'s frame 1, the rest from frame 0; redrawn when the percent changes | Rule 35 and the user's check (black background, gold outline, cut from the top); frames observed in `clar/inv/str/vul.pba` (2 frames each: 0 = black square with gold outline, 1 = the picture) | Viewer: `gui.log` 100, 95, ... 5 %, then empty. **Which frame is which part: our reading, Q143 d** |
| 5g-3: the spinning mask takes a status place too (first free), at 10 frames a second | Rule 35 (the mask is an icon place; 10 a second); **first free: stand-in, Q143** | Viewer: `handsOff 1` with Strength running: the mask in `_2` |
| 5g-3: while Strength runs the hand number is drawn in `14red.pft` at `IDCTL_INVBAR_SHORT_ALTTXT` and the white one not; the shield the armour number; the switches go with each stats event | Rules 37–39; `interface.md` rule 32 | Viewer: `gui.log` "red armour true, hand true" |
| 5g-4 (`potions.rs` `effect_on`/`effect_off`): the shield sets protection to p + ⌊p/2⌋, 50 to 100, at the start (before the count drops and the stats event) | Rules 13, 17 | Unit tests (rule 17's six examples); viewer: 2 → 50 |
| 5g-4: at the end, protection less exactly what the shield added, cut to 0–100 | **Do not reproduce 1** (the original restores the old value) | Unit test; viewer: gloves during the shield kept (51 → 3) |
| 5g-4: `setConnorFlag InTemple4 1`; then each hit on Connor with the shield uses old + ⌊added/2⌋, at most 85; the shown number unchanged | Rule 19 | Unit test (60 used as 50) |
| 5g-4 (viewer `potion_view.rs`): `orbit1.kq` (`KQOrbitObj`: `shapeName`, `orbitRadius`, `orbitStartAng*`, `orbitStepAng*`, `orbitZoffset`, `translucency`) | Rule 20; keys observed in `GAME/Kq/orbit1.kq` (parser `OrbitKeys`, unit test) | — |
| 5g-4: the orbit at Connor's `body` part − 820 + 500, radius 500, +0.2 rad per update from 0, removed at the end; see-through 165/255 as other objects | Rule 20 | Viewer: 301 places logged, updates 30–330 |
| 5g-4: angle 0 points along +x; no `body` part → his feet | **Ours** (the note gives neither) | — |
| 5g-4 fix: three orbits, `orbit1.kq`–`orbit3.kq` (start angles 0, 2.09, −2.09) | The user's check in the original (three shields); files observed in `GAME/Kq` (the note says they don't exist: Q145) | Viewer: 3 orbits, 120° apart |
| 5g-4 fix: each shape turned by its orbit angle + ¼ turn, its flat side (the model's −y) facing out | The user's check (ours was 90° off); the model's shape observed (`invulnfx.kqs`: 404 wide, 57 thin, 657 tall); which side faces out: ours (Q145 b) | The user's check: correct |
| 5g-5: Strength: hand number + ⌊hand/2⌋, the half remembered and taken off at the end; ranged unchanged | Rules 21–22 | Viewer: 10 → 15 → 10 |
| 5g-5 (kq8-data `SkinList`): `KQSkinList` sections, `skinN=<file> <bits>`, `invItemN`; item N is bit 2^N | Rules 25, 30 name the bits (Strength 2048, Clarity 4, Strength 8); **item N = bit 2^N is our reading of `Connor.kq`**, which fits all four | Unit test |
| 5g-5: a list's bits = the items of its `invItem` lines Connor has applied (worn, or a potion running) | Rules 25, 30 (potions are applied while they run, `inventory.md` rule 14); generalised from the note's examples | Viewer: bits 0, 8, 12, 4 |
| 5g-5: a missing face combination → the entry whose bits are all wanted with the most bits, a tie to the higher value | "Do not reproduce" 3 and its example (helmet + Strength → `cironf`); **the tie rule is ours**, chosen to give that example | Unit test |
| 5g-5 (viewer `potion_view.rs` `look_step`): while Strength runs, materials drawn with `con0.bmp` (skin list entry 0, `curSkin=0`) show `coniron.bmp`; normal at the end | Rule 25 | Viewer: 448 materials |
| 5g-5: face pictures `<set>0`–`<set>7` of the face list's entry; the current mouth shown at once | Rule 30, `talking.md` rule 18 | Viewer: `cf` → `cironf` → `cclif` → `cclf` |
| 5g-5: face pictures read only from the world's own archives (`chf`, `cclhf` exist in no world) | Ours (observed: no `chf0.bmp` in any world) | Viewer: sets found 8/0/8/8/8/0/8/8 |
| 5g-6 (`active.rs` `connor_reported`): while Invisible runs characters are told Connor is 100,000 below; sight, alarm, chase, leash and "lose active" use it; hits on him and a rising character's "is he standing on me" use his true place | Rule 26 (which uses take the report: our reading of "characters know where Connor is from the reports") | Unit test; viewer: goblin22 "can't see Connor (blocked by the ground)", gives up after 45 updates |
| 5g-6: `KQConner::checkInvisible` → 1 / 0 | Rule 28 | — (Dead City only) |
| 5g-6 (viewer): each of his textures → its `I_` version if this world has one (first two letters replaced), entry 0 of it a hole | Rule 29; holes at entry 0 observed in the `I_` files (3 of 4 pixels are 0) | Viewer: 445 materials |
| 5g-6: otherwise a copy with 3 of 4 pixels holes, kept where (x + 2y) mod 4 = 0 | Rule 29 (pattern as the game's `I_` files, observed) | Viewer: 585 materials |
| 5g-6: holes drawn by a shader switch (`cutout`: alpha below one half is discarded) on his materials while it runs; converted again when his skin or face changes | Rule 29; the shader switch is ours | — |
| 5g-7: Clarity: every object whose `groupsAlways` has bit 2048 (collected at load, any class) gets its script with `Reveal True` at the start, `Reveal False` at the end; the calls run at the next update | Rules 31–32; the bit observed in `keepwall.kq` (2054) and `wizchest.kq` (2062); the one-update delay is ours (the click comes between updates) | Unit test; viewer: keepwall and wizchest1 |
| 5g-7: `KQObject::dither <name> 0|1`: half see-through (opacity 0.5) | Rule 31 says only "becomes see-through"; **stand-in, Q144** | Viewer: `dither: {"keepwall"} now; 1 parts drawn half see-through` |
| 5g-7: a tracked object's (scripted object, chest, item) own hidden state wins over its file's `hidden=` in the drawing | `objects.md` "Hidden objects" (setHidden shows it); fixes our check, which kept a revealed `wizchest1` undrawn | Viewer: `[wizchest1] Off -> Near (drawn)` |
| 5g-8: each time one is added: +25 ring of the dead hero, ashes, candle, +20 lady bell, +10 rope and hook, 0.3 s later; +1,000 feather, +5,000 green mushroom piece, +6,000 mold, +10,000 hearing horn, +25,000 deciphering amulet, +50,000 ladle at once; through the stats event | Rules 40–41; type names matched to the note's names in `Invinfo.itl` (observed) | Unit test; viewer: candle at update 15 (added at 10), feather at 20 |
| 5g-8: help pop-ups 0x83 (first health item or potion, 3 s), 0x84 (first quest item), 0x80 (rope and hook, 4.5 s): logged only | Rule 41 (the pop-ups are not studied: open question 3) | Viewer: `items.log` |
| 5g-9: while Connor's handsOff is on (any `handsOff 1`, also `walkTo`'s), characters are told he is 100,000 below; each change logged | Rule 26 ("during cutscenes (handsOff with the pointer hidden)"); every handsOff counted: **our reading, Q146 b**; the user's go-ahead | Unit test; viewer: goblin22 gives up at 209, sees him again at 315 |
| 5g-2: game time = our updates; they go on when the window loses focus (Bevy's default keeps an unfocused window updating at up to 60 frames a second) | Rule 34 (the user's check in the original); the Bevy part is ours | Not tested by switching windows |

## M6 6a: the Rope and Hook — `crates/kq8-core/src/rope.rs`; `active.rs`; viewer `gui_view.rs`, `door_view.rs`

Source: `docs/behaviour/ropeandhook.md` (A, 2026-10-21; checked by the
user in the original, whose checks win over the note's reading of the
code), with `potions.md` rule 35 (status icon places) and `interface.md`
rule 15. Data: `Invinfo.itl` (the hook item types and their pictures),
`Gplayscr.gui` (observed: `hookup.pba` in `IDCTL_TIMER_5`, `hookdown.pba`
in `_6`), Daventry's `ropetrp1–4.cs` and `.kq`. No other source.

| What | Source | Check |
| --- | --- | --- |
| 6a-1: `KQConner::setRope None\|Top\|Bottom` (any case; 1, 4 or 7 words, else the usage line); accepted only if Connor carries an item of category `Rope`; place and direction stored only when accepted; the `setRopeCam` spots cleared on every `setRope`; the game time noted (logged only) | Rules 2, 4, 5 | Unit tests `set_rope_needs_the_rope_…`, `set_rope_takes_one_four_or_seven_words`; viewer: `ropetrp1` On/Off with and without the rope (`rope.log`) |
| 6a-1: `setRopeCam` (a spot set only with no 0 coordinate), `setRopeCam2`, `setRopeCamAngle` (kept, unused), `setRopeCamTopLimit` / `BottomLimit` (0 = none); an accepted `setRope None` clears both limits and puts a held rope back | Rules 3, 6; "Do not reproduce" (the angle test) | Unit test `camera_spots_limits_and_their_clearing` |
| 6a-1: `HookUp` at the foot, `HookDown` at the top, none for None, without the rope or on the rope; the picture is the hook item type's `extraData`, drawn whole; a picture of several frames would animate at 10 a second | Rule 7; potions.md rule 35 (last point); **the first free place: stand-in, Q147 a** | Unit test `the_hook_icon_follows_the_status`; viewer: `gui.log` "IDCTL_TIMER_1: the hook icon HookUp, hookup.pba" (1 frame) |
| 6a-2: the rope button: ignored in the rope state; held → put back (the rope's first-person lock released, the camera stays: the user's check); status none → `0.MSG` 0/0/58/37/1 only, **not held** (replaces the Q110 stand-in); bottom → held (`IDBMA_CURS06`); top → third person forced (logged: no first person), the lock, the camera to `setRopeCam2` looking 1,000 above Connor, then held | Rules 8, 9 (Q110 b answered there); the camera placed as a `move KQCamera` while following: **stand-in, Q147 c** | Unit test `the_rope_button_by_status`; viewer: `ropetrp2` top (camera to (151584, 210996, 8520), no camera update after), `ropetrp1` foot, and no place |
| 6a-3: with the rope in hand any world click (whatever it meets, the sky too) starts the climb; ignored in handsOff and while dying or dead; the rope leaves the pointer; no camera aim test | Rules 10–12; "Do not reproduce" (the aim test always passes) | Viewer: `--aim-point` sky clicks at the foot and at the top (`rope.log`) |
| 6a-3: the rope state (walker mode Rope, keys ignored, no turning; fall, landing, push and wall events don't change it): the camera stays, the `walkTo` turn to the rope's place, handsOff on (pointer kept), the hand mode remembered, `away` (the list in use), empty hands; foot: `draw`, gravity off, `btoss`, step heights saved and 0, placed at the place and direction, the rope model (`rope`, drawn at his place, its two sounds), `stop`; the model's end: the aim point, the camera spot, gravity off, icons hidden, +5,000, `bon`, the range up, hanging at frame 0; top: `drawdown`, placed, the camera spot aimed at the place, gravity off, `ttoss`, `ton`, the range down, the half turn, hanging at the last frame | Rules 12–21, 28, 31; the camera during `btoss` stays: open question 2's stand-in (the user's instruction); handsOff as `handsOff 1` with the pointer kept: **stand-in, Q147 b** | Unit tests `from_the_foot_to_hanging_in_order`, `from_the_top_to_hanging_with_the_half_turn`; viewer runs (`rope.log`, `connor.log`) |
| 6a-3: the camera spots: `setRopeCam` 1 and 2 if 1 is set (the nearer to the camera, 2 when equal), else the place + (±2,500, 2,500, 4,800) at the foot / (±5,000, −5,000, −2,000) at the top, turned by the rope's direction as Connor's heading turns his own steps (x his left, y his back) | Rules 18, 20; the frame's axes: our reading of "x, y along the rope's own axes" with Connor's model axes (`connor-movement.md`) | Unit test `the_spots_the_game_makes_turn_with_the_rope`; viewer: foot spot 2 (5,025 vs 5,200), top spot 1 |
| 6a-3: the camera's height range: from the foot lowest = its height, highest = +5,000 or the first terrain / building / object box straight above, −250, at most the top limit; from the top the mirror, at least the bottom limit | Rules 19, 21 | Unit test `the_height_ranges_and_their_limits`; viewer: 12,600..17,350 (foot), 7,216..10,820 (top, the terrain at 6,966) |
| 6a-3: each placing by the rope state counts for the traps as a script's `move` (stay test at once), then the enter test at the new place: the foot trap's `Off`, then the top's `On` | **Stand-in, Q147 e** (the note: leaving a trap during the climb "just changes the status") | Viewer: update 162 "ropetrap Off Connor; ropetrp2 On Connor" |
| 6a-3: the rope animations' own steps: only `taway` has one (405 forward); others 0 | Observed (`Conrope.anm` shapes); whether `taway`'s moves him: **Q147 d** | `viewer.log` load lines |
| 6a-4: hanging: forward plays `climb` forwards (up), back backwards (down), any other key or none leaves it frozen; his place never changes; the climb ends at the end it plays toward | Rules 22–23; only forward and back: the user's check (over the note's reading of the code) | Unit test `the_climb_moves_only_with_forward_and_back`; viewer: up 201–230, frozen with a turn key 231–245 (heading unchanged), down 261–275 |
| 6a-4: the camera's height and the aim point move 750 per second with the climb, the height within its range, x and y kept, the camera re-aimed each update | Rule 22; per second, not per update: "Do not reproduce" | Unit test `the_climb_camera_moves_750_a_second_within_its_range`; viewer: +50 per update (12,600 → 14,100) |
| 6a-4 test option: `--hold F,T,R,FROM,N` (held from update FROM to N), repeatable | Ours | The runs above |
| 6a-5: the climb's end: the hook icon may show, turning on, handsOff on; top: `toff`, the camera follows, `taway`; foot: moved 1,000 toward his back and 4,900 down (his frame), gravity on, `boff`, `baway`, the camera follows | Rules 23, 24 | Unit tests `up_from_the_foot_and_off_at_the_top_…`, `down_from_the_top_and_off_at_the_foot_…`; viewer: top end at 277, foot end at 297 |
| 6a-5: `taway` / `baway` end: gravity on, the saved step heights back (only if saved and not 0), the camera follows; `out` from the weapon's list only if he held a weapon at the start, else none; then handsOff off, the lock off, standing | Rules 25, 26; the weapon only if held: the user's check (over the note's reading of the code) | Viewer: empty hands → nothing drawn, standing at 332 on the ledge; fists ready → `confists.anm/out`, standing at 382 |
| 6a-6: in the rope state: hits ignored completely (no damage, sound or reaction); every state request (a script's special list, walks, dying) refused and dropped; weapon requests (keys, slots, scripts) ignored with no remark; characters told he is 100,000 below (`potions.md` rule 26's code, reason "the rope state"); the rope button ignored; an item click while hanging picks it up (handsOff is off then); no click turn (Rope is not Wait or Move) | Rules 27–30 (clicks pick items up: the user's check); rock throws: not built, so ignored by construction | Unit test `on_the_rope_hits_weapons_are_ignored_and_characters_lose_him`; viewer: `setState` refused at 206, the hand slot ignored at 210, a crystal picked up at 225 while hanging, `characters.log` "the rope state" from 91 |
| 6a fix: a movement the rope state starts after his update is drawn from its start at once (was its last frame for one update) | Ours (the user's check: snaps at each switch) | Viewer: `rope.log` "drawn: … from phase 0.000 (was drawing phase 1.000 …)" at each switch |
| 6a fix: the camera at the end of `btoss`: 250 behind him, 1,400 above his feet, tilted up 1.27 rad, looking the way he faces | Open question 2: the user's check in the original (close, eye level, looking straight up at the rope) and the note's 250 and 1.27 rad and half turn; the 1,400 and the combination are **ours, Q147 f** | Unit test `the_throw_camera_looks_up_the_way_he_faces`; viewer: camera at (147896, 207586, 9200), tilt 1.270 |
| 6a, Q147 answered by A: the hook icon in the first free place (a); the rope's handsOff a plain `handsOff 1` (b); the top camera skips Connor's first move, the first-person lock goes then (c); `taway`'s 405 step moves him (d); a placing runs only the traps' stay test (e); the rope model 2,000 along the rope's +y, the throw camera at the place + (0, 2,250, 350), tilt 1.27, heading + 3.14, the grab's tie the + spot (f) | `ropeandhook.md` rules 7, 9, 13, 15, 18, 24 (traced; c–f measured in the original by A); the tie's 0.5 tolerance ours | Unit tests with A's church numbers; viewer runs (MODLOG 2026-10-21) |

## M6 6b: Azriel's Hammer — `crates/kq8-data/src/characters.rs` (`ProjectileKeys`); viewer `door_view.rs`, `load.rs`

Source: `docs/behaviour/azriel-hammer.md` (A, 2026-10-20; checked by the
user in the original), with `ranged-weapons.md` rules 17, 21, 26. Data:
`<world>/kq/prhammer.kq`, `Invinfo.itl` (`INVITEM_AzrielHammer`'s last
column), `Connor.kq` (`shootOffsets1`), Swamp's `constart.cs`. No other
source.

| What | Source | Check |
| --- | --- | --- |
| 6b-1: a `KQHammer` file is read like a `KQProjectile` (same keys and defaults), plus `modifier`, `minDamage`, `maxDamage` (default 0) and its own default mask 0x221F | Rule 6 | Unit test `a_hammer_file_gives_its_damage_keys` (Daventry's file, Barren's defaults); viewer, Swamp: `characters.log` speed 30,000, range 20,000, mask 0x221F, 30 / 0 / 300 |
| 6b-1: Connor's fight is set up after the items the start-up scripts gave him, so a ranged weapon given at load (Swamp's hammer) gives his bolt file and shoot offset | `ranged-weapons.md` rules 17, 21 (a carried `LongWeapon` gives both); the order was our bug | Viewer, Swamp: `fight.log` "carried at the start: INVITEM_AzrielHammer: shoots Prhammer.kq; shoot offset (-41, -605, 1226)" (before: `PRarrow.kq`, (-300, -502, 1032)) |
| 6b-2: the hammer flies as a bolt on its way out (speed / 15 per update, its box as a ball stack, its mask, never its thrower); its hits are always `Projectile` | Rules 6, 7; `ranged-weapons.md` rules 28, 29 | Unit tests `the_hammer_hits_one_character_flies_back_and_is_caught`, `the_hammer_turns_back_at_its_range_and_at_the_ground` |
| 6b-2: it turns back at its first touch, past `maxDistance` or at / below the floor grid: put back at its place at the start of that update, direction reversed, distance 0, range back = the distance flown out − speed / 15, only Connor stops it; the way back ends touching Connor, past its range or at the floor grid: removed, the thrower told (`hammers_back`) | Rule 9; **the same "put back" for the range and the floor grid: B's reading, Q148 a** | Same tests: Daventry's file level: out 14,400, back 12,800, caught at update 19; at the ground: back at once; Connor on its line: passed out, touched back |
| 6b-2: the touch on the way out: `lever1` / `tmblrcyl` → `lever1`'s script with `ConnorAction` (no damage); type bit 0x4000 → a hit of 0, no `ProjImmune`; another character or scripted object → min(`modifier` + ranged damage, `maxDamage`), then at least `minDamage`; a building nothing; one thing only | Rule 8, "Do not reproduce" (the order); `lever1.cs` (data: its comment says the hammer sends `ConnorAction`); objects' damage: the 5c fix 5 stand-in (Q139 e) | Unit tests `the_hammer_s_damage_is_capped_then_raised`, `the_hammer_on_the_lever_runs_its_script_and_spares_immune_ones` |
| 6b-3: a Throw mode for Connor's walker (like the rope's: the caller's movement plays, keys ignored, no key turning; fall, landing, push and wall events don't change it) | Rule 3 ("can't be left before `catch` has ended") | Unit test `the_throw_state_plays_its_movements_and_ignores_the_keys` |
| 6b-3: a click with the hammer ready (a `KQHammer` bolt file), after `ranged-weapons.md` rules 13-18: only while standing or moving (as readying: not in a jump, its landing, a stumble or a swap), not kept otherwise; the automatic turn toward the aim point, no waiting; the item on the pointer let go; handsOff on; `attack` | Rule 2 (the refused states: the user's check) | Viewer: Swamp (`--aim-point`), Daventry (`--aim`, `--aim-point`): `fight.log` "no waiting (rule 2)", "the hammer-throw state" |
| 6b-3: the hammer leaves at `attack`'s `do` cue from his shoot offset (-41, -605, 1,226), never the `hammer` part, aimed at the aim point, no lead | Rule 5 (the fallback: the user's check) | Viewer: "start offset (-41, -605, 1226)" in both worlds |
| 6b-3: `attack` ends -> `wait` (repeating); the hammer back -> `catch` at once from `wait`; `catch` ends -> handsOff off (held keys sent again), standing | Rule 3; `wait` / `catch` loaded from `Conhamr.anm` (data) | Viewer: Swamp: click 60, `do` 71, back 86 (`catch` at once), standing 97 |
| 6b-3: back before `attack` ended: `catch` right after `attack` (no `wait`) | **Ours, Q148 c**; replaced by the next row | Viewer: Daventry, a close goblin and the ground: back at 93, `catch` 98, standing 109 |
| 6b fix: back while `attack` plays: `catch` at once, cutting `attack` short; each throw movement drawn from its start at once (as the rope's 6a fix) | Q148 c: **the user's check in the original** (against a wall he catches it the moment it leaves his hand) | Viewer: Daventry against the cottage wall: `do` 91, back 93, `attack` cut at 0.722, `catch` 93, standing 104 (1.60 s) |
| 6b-3: state requests during the throw (special lists, scripted walks, the dying state) refused and tried again every update, so they happen after the catch | Rule 3; retrying: **stand-in as for a jump (`anim-scripts.md` rule 28), Q148 d** | Not run in the viewer (no hit took him below 0 during a throw) |
| 6b-3: the throw's handsOff blocks keys and clicks only: the panels, the spinning mask and the pointer unchanged, and characters still see him | Rule 3 ("no pointer change"); `potions.md` rule 26 (a cutscene's handsOff hides the pointer); the panels: **stand-in, Q148 e** | Viewer: no `gui.log` handsOff line during a throw |
| 6b-3: the bolt files of every ranged weapon whose file the world has are read at load (a console `give AzrielHammer` in Daventry) | Rule 10 (Daventry has `prhammer.kq`) | Viewer: Daventry `characters.log` "shoots Prhammer.kq ... speed 24000, range 15000" |

## M6 6c: a weapon's movement set loaded when it is given — viewer `load.rs` (`connor_set`, `LateSets`, `read_set`), `connor_view.rs` (`late_set_step`, `spawn_frames`), `potion_view.rs` (`ConnorLooks::add`)

Not from a behaviour note: when the original loads a weapon's movements
is not studied. The user's check (2026-10-21: the Lake Sword given in
Dead City had no stance) and the user's choice (load when given, not
every set at start-up: +2.2 s per load, measured).

| What | Source | Check |
| --- | --- | --- |
| 6c-1: one set's movements read by one function, at start-up as before | Ours (refactor) | `viewer.log` "connor:" lines of Daventry and Swamp identical to the earlier build |
| 6c-2: the sets of weapons no file of the world names are listed at start-up; when Connor carries one, its set is read (the world's archives opened again) and drawn as at start-up; its materials join the potion looks (Strength, Invisible, with this world's `I_` pictures) and the mouth | Ours; the drawing and the looks as 5d fix 9, M5 5g-5, 5g-6 | Viewer, Dead City, `give LakeSword`: "conSword.anm ... loaded while playing: 39 movements, ... 157 ms"; after the draw `consword.anm/stop` drawn; looks redone (823 -> 900 materials). Daventry's first-frame light line identical to the earlier build |

## Connor's levels checked, start-up lines — `crates/kq8-core/src/connor_start.rs`, `active.rs` (`set_level`); `kq8-data` `world_script.rs` (`connor_line`); viewer `load.rs`, `door_view.rs`

Source: `anim-scripts.md` rule 19, `interface.md` rules 21–30,
`fighting.md` rule 4, `inventory.md` rules 8 and 14 (the check), and the
data: every world's `conStart.cs` (`KQConner::setLevel N`, then
`sendEvent 0 Connor KQStatsChangeRequestEvent ...`). No other source.

| What | Source | Check |
| --- | --- | --- |
| The level rules as built in M3 3l-6 are right: thresholds 10 × 2^(level − 1) (150,000 from 15), strictly more, one level per event, +10 health with a full heal, +5 / +2 / +1, health capped at 10 × level | The notes above; the data agrees (`conStart.cs`: health added = 10 × (N − 1) in all 11 worlds) | Viewer: Daventry, events of 10, 1, 20, 1,000, 0, 0, −5, 500 experience / health by `--command`; `viewer.log`, `gui.log` (levels 1 → 7, percents) |
| L-1: items, `setLevel` and stats events to Connor in the start-up scripts are kept in script order and done when play starts; items by the rule of a `give` (put on, numbers added) | `inventory.md` rule 8; the scripts' order | Viewer: Dead City level 8, [80, 60, 19, 11, 0]; Swamp level 15, [150, 110, 38, 21, 0]; Daventry unchanged |
| L-1: `KQConner::setLevel N` sets only the level number; the meters are sent again | **Stand-in, Q149 a** (the data: the stats event after it always brings the health) | Unit test `set_level_sets_only_the_level` |
| L-1: built-in pickup experience of items given by a start-up script counts (temple 1: two level-ups before `setLevel 43`) | `potions.md` rule 41 as written; **Q149 e** | Viewer: temple1 `items.log` |
| L-2: the level-up flame (`IDCTL_METERS_LEVELUP`, `flame14.pba`, layout place) starts when the level sent to the meters goes up, plays once at 15 frames per second, then is hidden; not at a world's first sending | `interface.md` rules 23, 26 (picture, place); **the user's check** (it plays on a level-up); speed, once, hidden: **stand-ins, Q149 b / Q108 d** | Viewer: Daventry `gui.log` (start at update 21, played out after 1,142 ms); Dead City: none |
| L-3: on a level-up `GAME/SOUND/LEVEL.AUD` plays once, plain, in the effects group | **The user's check in the original** (2026-10-21): the sound is `LEVEL.AUD` (Q149 c answered; was a stand-in) | Unit test (stats event); viewer `sound.log` (Daventry, level 1 → 2) |

## `checkDistance` (Q142) — `crates/kq8-core/src/active.rs` (`check_distance`)

Source: `anim-scripts.md` rule 31 (Q142 answered). No other source.

| What | Source | Check |
| --- | --- | --- |
| Three forms by word count (object–object, object–point, point–point); straight 3D distance between positions (Connor's feet, an object's current place, found as `getLoc` finds it); 1 if at most the max, else 0; unknown name or wrong word count: an error line and 0 | `anim-scripts.md` rule 31 | Unit test `check_distance_is_straight_3d_and_inclusive`; viewer: `wiztrap.cs` at (193000, 193000) 5,839 > 5,000 → the Wizard says "Come closer, lad!" (`1000 4 0 119`), at (192500, 196500) 2,452 → silent; the axe from the windmill floor 2,388 > 1,700 → the Henchman's line, no axe |
| The distance is written to the log (the original prints it to its console) | `anim-scripts.md` rule 31 | `traps.log`, `doors.log` |

## M6 6d: the magic map — `crates/kq8-data/src/{magic_map,pvol}.rs`, `crates/kq8-cli/src/map_verify.rs`, `crates/kq8-core/src/magic_map.rs`, viewer `map_view.rs`, `load.rs` (`take_map`)

Source: `docs/behaviour/magic-map.md` (A, 2026-10-21; "How to check"
1–11 run by the user in the original, all as described), with
`interface.md` rules 8–11, 15, 19, 44, 45 and `inventory.md`. The data:
`GAME/Kq/*map.kq`, `*tele.kq`, the user's saved games (read only).

| What | Source | Check |
| --- | --- | --- |
| 6d-1: a `KQMap` file's keys (box, `useFog`, `fogRadius`, `holdMapLoc`, `isActive`, pictures, `mapName`, `earnedTeleport`, teleport script and portal, icons, `rle<n>`); map index = `mapName` − 38 | `magic-map.md` "Data used", rules 3, 21 | `kq8 map-verify`: all 16 map files; Daventry index 0, 409.6 units per pixel |
| 6d-1: a `KQPortal` file's keys (`world`, enter / exit animations and files, `exitScript`, `destConnorLoc/Dir`, `destCamLoc/Dir`, `noLoc`) | `magic-map.md` "Data used", rule 40 | `kq8 map-verify`: the six `*tele.kq` |
| 6d-1: fog radius in map pixels = 460 × `fogRadius` ÷ (east − west), whole number; missing → the terrain's `hazeDistance`, else 20,000 | Rule 13 | `kq8 map-verify`: Daventry 24, Dead City 15, Swamp 24, Gnome 14, Barren 20, Ice 19, Castle 41, temples 16 / 18 / 17 / 31, Snow exit 131 (`hazeDistance` 18,000): all as the note's list |
| 6d-1: the fog's run lengths: 460 rows of 464 places, hidden run first, alternating, −1 ends; unused places ignored on reading, written as fog; nothing revealed = no runs (`rle0=-1`) | Rule 16, "Do not reproduce" 7 | Unit tests; `kq8 map-verify`: 29 saved maps in 22 saves: Daventry and Dead City runs written back identical; Castle 705 runs with 63 unused places revealed (column 460, rows 194–256, the note's "Do not reproduce" 2 at the right edge), written back as fog, the same fog |
| 6d-1: the saved-game archive `PVOL`: `vols` name table, `voli` index of 17-byte entries (name offset, `VBLK` offset, size, compression), each file in a `VBLK`; the index follows the table at the next even offset | Observed (the user's saves; the layout matches what the Tribes community describes for `PVOL`, docs/RESEARCH.md "Engine lineage") | Unit test; `kq8 map-verify` reads all 22 saves (save2's table ends at an odd offset) |
| 6d-1: the current world's map is stored in a binary `PERS` form in a save | Observed | Listed, not read (no note describes it) |
| 6d-2: the world's map is the `<x>map.kq` its start-up scripts `loadKQ`, read from `GAME/Kq` (not in the world's archives); one at a time | Rule 3 | Viewer: Daventry `map.log` "GAME/Kq/davmap.kq [davMap]: index Some(0)" (before: "davmap.kq not found") |
| 6d-2: the map pictures (`blankMap`, `fullMap`) and icons read from `GAME/Resource`, 8-bit, used only at 460 × 460 | Rules 4, 7, 19, "Data used" | `map.log`: davmap0.pbm, davmap.pbm 460x460 |
| 6d-2: active when the `MagicMap` item (category `MagicMap`) arrives in Connor's inventory, off when it leaves; `KQMap::activate` / `deactivate`; "KQMap not loaded" without a map | Rule 2 | Unit test `script_commands_and_the_update`; viewer: `give MagicMap` at update 5 → active at update 6 |
| 6d-2: map pixel = ⌊(x − west) ÷ (east − west) × 460⌋, ⌊(1 − (y − south) ÷ |north − south|) × 460⌋, towards zero, no clamp | Rule 4 | Unit test `world_to_map_pixel_north_up`; viewer: (193000, 193000) → (191, 368) |
| 6d-2: each update while active: a new pixel reveals the disc (rows cy − r + 1 to cy + r, w = ⌊√(r² − e²)⌋, columns cx − w to cx + w − 1, cut at the edges), marks the map visited on Connor and draws the icons again; the arrow frame always | Rules 9–12, 33; "Do not reproduce" 2 (marks = copied columns) | Unit tests `the_disc_rows`, `reveal_copies_and_marks_the_same_columns`; viewer: 1,696 pixels at the first pixel, 46 per row walking north |
| 6d-2: Connor's place is given once per update after his movement and the scripts' moves | **Stand-in** (rule 9 reacts to every change of position; DESIGN 6d) | — |
| 6d-2: arrow frame = round((π − heading) ÷ 2π × 16) wrapped into 0–15 | Rule 17 | Unit test `arrow_frames`; viewer: turning right steps frames 0 → 11 one by one |
| 6d-2: `KQMap::stopLoc`, `useFog` (shown pixels marked), `setFogRadius`; `addMapIcon file x y [picture]` (not twice; drawn if on the shown picture, the `fullMap` name), `removeMapIcon x y` (painted back blank, then full where revealed) | Rules 13–15, 20, 21; "Do not reproduce" 3. That icons are kept under the `fullMap` name: the saved maps name it | Unit tests; viewer: `stopLoc 1` at 60, `0` at 80: no reveal between, then 368 pixels at once |
| 6d-2: the map's keys as they would be saved (runs from the fog, icons, flags) | Rule 16 | Unit test `saved_fog_comes_back` (nothing saves yet) |
| 6d-3: the map window drawn from its layout block: borders, `4black.pbm` area, the page arrows and the unlit "Go" button (first frames), the name plate | Rule 5 (layout `Gplayscr.gui`) | Viewer `gui.log`: the `8gui` pictures read; "IDCTL_MAPWIN drawn" |
| 6d-3: the map picture at the view (from the area's centre), nearest pixel, cut to the area; the arrow on top, its top-left at the map pixel × shown size ÷ 460 + the picture's top-left − (5, 5), cut to the area, index 0 see-through | Rules 6, 18; the arrow's file has the see-through flag (0x9) | Unit test `the_map_at_its_view_with_the_arrow_cut_to_the_area`; viewer: pixel (191, 368) → arrow top-left (−34.0, 81.5) |
| 6d-3: the start view (−152.83, −152.11)–(145.44, 146.16) stored on Connor, copied to the big map and the mini-map when the map is set up | Rules 22, 27; the copy at set-up also after a world change: **stand-in**, open question 3 | Unit test `the_start_view_and_the_arrow` |
| 6d-3: the view and the arrow in 640 × 480 layout pixels, scaled × 1.25 with the screen | **Stand-in** (open question 6, Q108) | — |
| 6d-3: the name plate: `500.MSG` 0 0 `mapName`; a line break → two lines in `NAME_TEXT2` / `TEXT3`, else `TEXT1` | Rule 8 | Viewer `gui.log`: "500.MSG 0 0 38: [\"Kingdom of Daventry\"] (1 line(s))" |
| 6d-3: the map key (`movekeys.kq` row 8, Tab) and a left click on `IDCTL_INVBAR_MINIMAP` toggle the window when the map is active; nothing else checked (handsOff, conversations); `handsOff 1` hides it, `handsOff 0` doesn't bring it back (built in 4c-2) | Rule 30; `interface.md` rules 10, 11, 15 | Viewer `gui.log`: Tab before the map: nothing; Tab: shown; `handsOff 1`: hidden; Tab during handsOff: shown; `handsOff 0`: stays; the slot click: hidden |
| 6d-4: left drag on the map area moves the picture by the mouse's movement, the area's centre kept on the picture | Rule 23 | Unit test `pan_keeps_the_centre_on_the_picture`; viewer `--key mappan:5000:0`: left edge stops at 0.00 |
| 6d-4: right drag sideways multiplies the size by 2^(dx ÷ 64) about the area's centre, the shown width kept in 115–3,680; up/down nothing | Rule 24 | Unit test `zoom_doubles_per_64_and_stops_at_the_limits`; viewer `--key mapzoom:192`: 298.3 → 2,386.2 |
| 6d-4: mouse movement turned into 640 × 480 layout pixels (÷ 1.25 at 800 × 600) | **Stand-in** (open question 6) | — |
| 6d-4: when Connor's map pixel changes the big map is marked to follow; at each draw of the shown window up to 6 steps of (distance past) ÷ 10 + 1 (whole number) move the picture until the arrow's top-left corner is at least 20 layout pixels inside the 296 × 296 area on all four sides; inside clears the mark | Rule 25; the margin on all four sides: **the user's check and the task's stand-in** (open question 2; the original's limits 20–280 / 20–340 were in untraced coordinates) | Unit test `follow_brings_the_arrow_inside_the_margins`; viewer: after a pan at × 8 the arrow from (1133.8, …) back to (275.8, 275.9) in 8 draws |
| 6d-4: the view is stored on Connor after each drag or follow | Rule 22 | — (no world change yet) |
| 6d-4: a drag that starts on the map area is not a world click and does not swing the camera (right button) | Ours: the map takes the mouse (rules 23, 24 give it the buttons) | **Not tested by hand** |
| 6d-5: the bottom-right slot `IDCTL_INVBAR_MINIMAP` draws the current map while it is active, at the mini-map's own view size, Connor's map pixel at the slot's middle, with his arrow; `2mapblnk.pbm` (the layout's picture) while not active | Rule 27; `interface.md` rule 45 | Viewer `gui.log`: "IDCTL_INVBARR drawn … shown 298.3 x 298.3 … arrow top-left (−5.0, −5.0)" while the big map is at 596.5 |
| 6d-5: the mini-map keeps its own zoom (a big-map zoom does not change it) | Rule 27 (**the user's check in the original**, 2026-10-21) | Viewer: `--key mapzoom:64` → big 596.5, mini 298.3 |
| 6d-5: the slot blinks when the item arrives: the map and the empty picture every 0.25 s, 5 times | Rule 2; `interface.md` rule 44 | Viewer (map given at update 60): map at 6,014 ms, empty, map at 6,510, empty, map at 7,009 |
| 6d-5: where the map doesn't cover the slot (Connor near the map's edge) the empty picture shows under it | Ours (not in the note) | — |
| 6d-6: the page arrows step through the 16 map indexes from the one shown, wrapping, to the first marked visited (this world's counts); none in 16 steps: no change | Rule 32 | Unit test `paging_to_visited_maps_and_back`; viewer: 0 → 1 → 14 → 0, left from 0 → 14 |
| 6d-6: another world's map: its own picture (blank + full where out of its fog, or its full picture without fog, its own icons), this world's icons kept for that picture, then each of its 256 palette entries replaced by the nearest (squared RGB) of this world's palette (`GAME/Resource/<name>.ppl` by map index); its name on the plate; no arrow | Rule 34. That the icons are drawn before the colour match (so they are matched too) is **our reading** (the note lists the match first, the icons after; the icons are drawn in their own world's colours: `dv_` icons on Daventry's map) | Unit tests `palettes_match_to_the_nearest_colour`, `paging_to_visited_maps_and_back`; viewer: Dead City (save18) 24 icons, 237 of 256 entries moved |
| 6d-6: Connor's map pixel changing while another world's map is shown snaps back to this world's | Rules 9, 34 | Viewer: "back to this world's map" at update 72 |
| 6d-6: a visited world with no kept map is shown from its shipped file (`GAME/Kq`, no fog revealed); the four unused map files are never used | **Stand-in** (open question 5: how the original finds the kept map); "Do not reproduce" 6 | — |
| 6d-6: `--kept-maps SAVE`: other worlds' maps from an original saved game (text form, read only), marked visited | Ours (a test option until moving between worlds exists) | Viewer: save18: Dead City 53,055 pixels, 24 icons; Castle |
| 6d-7: `onTeleport` / `offTeleport` set and clear "on the pad" (not saved) | Rule 36 | Viewer: Daventry's keep pad trap `GnomeTeleport` → "Connor is on the pad" |
| 6d-7: `earnTeleport`: the current map's `earnedTeleport`, and for map indexes 0–7 the mark on Connor; with an argument (its text not looked at): Barren visited, Barren earned, `SageGnomeEarnedTeleport`; `teleportersEarned` = the count of marks (0–8) | Rule 37 | Unit test `teleport_marks_and_the_go_button` |
| 6d-7: the "Go" button lit (`goon.pba`, the layout's off-screen control's picture, over `gooff.pba`) only with another world's map shown, Connor on the pad and that world's teleporter earned (its map's `earnedTeleport`, Barren also by the Sage's mark); worked out again on every change of the shown map or the pad | Rule 38; "Do not reproduce" 1 (also after a snap-back) | Unit test; viewer: lights at update 30 with Barren shown on the pad |
| 6d-7: a click on the unlit button: help 151 | Rule 38; help popups are not built: **logged only** | Viewer `map.log` |
| 6d-7: a click on the lit button runs the current world's `TeleportScript` with `$1` = the shown world's `TeleportPortalKQ` | Rule 39 | Viewer: "runs Davtele.cs with $1 = bartele.kq"; the script's `loadKQ … @` (no template while playing) and `setState Connor Portal` are logged as not done |
| 6d-7: the world's map is handed to the rules first in each update (before scripts and traps) | The original loads it in `newWorld.cs` (rule 3); found by the keep pad's trap saying `onTeleport` at update 1 | Viewer: "onTeleport: Connor is on the pad" (before: "KQMap not loaded") |

## M6 6e: moving between worlds — viewer `world_build.rs`, `log.rs`, `main.rs`

Source: `docs/behaviour/world-change.md` (A, 2026-10-21; the user's check
in the original 2026-10-22). Plan: `docs/DESIGN.md` "6e".

| Fact / rule | Source | Checked how |
| --- | --- | --- |
| 6e-1: a world is read, its resources put in and its start-up systems run as one path (`read_world`, `insert_world`, the `BuildWorld` schedule), used by the launch | **Ours** (how our viewer is split up); the original also builds each world inside the running program (rules 7–8) | `cargo play --world Daventry` / `Swamp`, 300 frames, before and after: every log's common part the same apart from lines tied to real time (the same between two runs of one build) |
| 6e-1: every entity spawned with a `Transform` or a `Node` is marked as the world's (`WorldEntity`), unless marked `Persistent`; the per-world resources are listed in one place | **Ours** | Viewer: "world built: Daventry: … entities 132882 (132449 marked WorldEntity) … per-world resources 50 of 50", the same in two runs |
| 6e-1: each log is emptied the first time it is opened in a run and added to after that | **Ours** (DESIGN "6e": logs keep their lines across worlds) | Each log name is opened once at launch today, so no change yet |
| 6e-2: a world is taken down by despawning every `WorldEntity` and removing the 50 per-world resources; the next one is read two frames later (so Bevy frees the old world's assets first); the game's systems don't run meanwhile | **Ours**; the order (old world down, then the new one read) follows rule 7: the old world is taken down before the new one is built | `reload` three times in Daventry, twice in Swamp: after each take-down 448 entities, 15 (Swamp 35) meshes, 12 images, 0 world materials; after each rebuild the same counts every time |
| 6e-2: `reload` (console word, ours) builds the same world again, Connor put on the ground where he stood with his heading | **Ours** (a developer tool, like the original's debug aliases) | Swamp: "Connor back at 176863 84355, turn -1.7000"; arrival "put on the ground at (176863, 84355, 8251)" |
| 6e-2: values the viewer's systems keep between frames (shadows, the caption picture, status icon places, the dither state, the H markers) start again with each new world | **Ours** | Meshes left after take-down no longer grow (15 → 24 → 33 before; 15, 15, 15 after) |
| 6e-2: a `== world change: A -> B (why) ==` line in every log | **Ours** (DESIGN "6e") | 11 logs checked: one line each after one reload |
| 6e-2: the process's working set and private memory in `viewer.log` (Windows `K32GetProcessMemoryInfo`) | **Ours** | Daventry: private 2,541 / 2,905 / 2,961 MB after three take-downs |
| 6e-3: the journey: Connor's stats, his item objects as they are (counts, applied), the hand, the ready weapon and mode, running potions with their time left, flags, map marks and view, the combat level, the `NoName` counter, the game-wide variables | Rules 17–19, 13 (variables named after the world or `KQWorld` stay with it) | Unit test `taken_and_given_back_unchanged`; viewer: `world.log` "Connor leaving Daventry" = "Connor arriving in Swamp" |
| 6e-3: every `setConnorFlag` flag kept on Connor (only `InTemple4` changes a rule) | Rule 17 | — (no shipped flag is read back by a script we run) |
| 6e-3: on a world change `conStart.cs` is not run and the `NoName` counter goes on from the carried one before the new world's start scripts | Rules 18, 21 | Viewer: "NoName counter from 25 … conStart.cs not run" |
| 6e-3: potions' timers moved onto the new world's clock with the same time left | **Ours** (rule 19 says the time left carries; how is ours) | Viewer: Strength 15 s left on leaving, 11 s at Swamp update 54 |
| 6e-3: the rope's settings start empty in the new world; other worlds' maps and the test switches go along | **STAND-IN** (rule 19: not traced where the rope lives) | — |
| 6e-3: `goworld <World> [x y heading]` (console word, ours) | **Ours** | Daventry → Swamp (150000, 80000, heading 1.0) → Daventry |
| 6e-4a: the next world is read on another thread; the window draws the loading screen meanwhile; the game's systems don't run (keys and clicks do nothing) | Rules 24–25 (the original loads on the same thread and redraws the meter at each 1 %); the thread is **ours** | Viewer: "loading screen: 558 frames in 3100 ms, longest frame 7.9 ms" (Swamp) |
| 6e-4a: the loading picture `GAME/8Gui/<world>.pbm` in `GAME/8Gui/L<first 7 letters>.ppl`, the black bar `4black.pbm` at (0, 462)–(640, 480), the red meter `4red.pbm` 630 × 8 at (5, 468), stretched to the window; read by name, never copied | Rules 22–23; "Do not reproduce" 1–2 (no CD-copy phase); the stretch is **ours** | Unit test `file_names_follow_rule_22`; viewer: "GAME/8Gui/Swamp.pbm in GAME/8Gui/lSwamp.ppl" |
| 6e-4a: the meter follows the scripts' `setLoadProgress <percent>` (no number: one step on) | Rule 24 | Viewer: Swamp 0 → 20 → 60 → 100 %; Daventry 0 → 13 → 18 … 100 % |
| 6e-4a: the sound system is made again with each world (every playing sound stops) | Rule 7.4 | — |
| 6e-4b: arriving through a portal: Connor at `destConnorLoc` / `Dir` (z −1: on the ground), the camera at `destCamLoc` / `Dir`, `handsOff 1`, the `exitAnim` (his own list, or the `exitAnmFile` named to the Special state); when it ends the `exitScript` with `$1` = the portal, `$2` = `Exit`, then `handsOff 0` | Rules 6, 20; first person and the time step: logged only (we have neither). Placing by `move` commands before the first update: **ours** | Viewer: `Davtele.kq` → keep pad (217338, 283698, 18931), `teleportup` 22 frames, control back at update 23; `davgate4.kq` → Dead City on the ground (182262, 45000, 864), `portalx`, `shadbat.cs davgate4 Exit` |
| 6e-4b: `teleportdown`, `teleportup`, `out`, `portal`, `portalx`, `walkend` loaded from `Conner.anm` with his other movements | The portal files' `enterAnim` / `exitAnim` (data); rules 4, 20 | Viewer: "movement teleportup = movement 52 … 22 frames, 1.4000 s" |
| 6e-4b: `goworld portal <file>` (console word, ours): reads the portal from `GAME/Kq`, else the current world's archives | **Ours** | As above |
| 6e-5: `world.cs <World> Begin` on the first update of every world arrived in (before the portal arrival); at launch only with `--begin` | Rule 9 (`Begin` on every arrival); the launch default and `--begin`: **ours** (no main menu or new game yet) | Viewer: Swamp → Daventry → Swamp → Daventry: "world.cs Daventry Begin … run" each time; the opening scene's `move Connor 0 277744 199029` both times (variables not kept yet: 6e-8) |
| 6e-6a: `KQMonster::setState Connor Portal <portal> [<trap>]`: the portal's keys (from its file), Connor plays `stop`, the question opens (not twice) | Rules 1–2 | Unit tests `no_moves_connor_to_no_loc_and_ends_the_state`, `yes_with_and_without_an_enter_movement`; viewer: `portal1` from Daventry |
| 6e-6a: the popup: `IDCTL_POPUP` with `buywin4.pbm`, text 114 cut into ≤ 38-character lines (≤ 7) in TEXT1…, Yes (29) left and No (14) right, the highlighted one in `buyhibtn.pba`, the other in `yesbtn.pba`; the "turn off help" box and the third button hidden | Rule 26; the looks per button from the layout (BTN1 highlighted, BTN2 plain): **our reading**; the button places: **our reading** (Q151 c); No highlighted at first: **stand-in** (Q151) | Unit test `the_question_is_cut_into_38_character_lines`; viewer `gui.log`: "IDCTL_POPUP drawn 463x250", slid in over 0.2 s |
| 6e-6a: while it is open the game's updates (and the world's drawing) stop; the panels and the sounds go on | Rule 26 | Viewer: 700 frames, the game stayed at update 31 |
| 6e-6a: Left / Right move the highlight, Enter chooses it, Escape is No (the main menu: not built, logged), a click on a button chooses it; other keys nothing; Space still moves the bottom bar | Rule 26 (the user's check) | Viewer: `--key popup:left`, `--key popup:no` (test keys) |
| 6e-6a: No: the portal's script `Decline`, Connor moved to `noLoc` unless (−1, −1, −1) (his turn kept), `stop`, Wait | Rule 3 | Viewer: `portal1`: Decline ran, Connor at (158510, 307740, 9926) |
| 6e-6b: Yes: the portal's script `Accept`, `handsOff 1`, the `enterAnim` (his own list, or the `enterAnmFile` named to the Special state); when it ends, or at once without one, the hand-over: the world change to the portal's `world` with its places | Rules 4–6; Connor taken off the trap without `Off` (rule 5.3) is implied: the old world is taken down; the full-armour list swap (rule 5.2) not built | Viewer: `davgate4` Yes: `portal` plays, Dead City, `portalx`, `shadbat.cs davgate4 Exit`; `portal1` Left + Enter: Castle Daventry, `out` |
| 6e-6c: a script's `loadKQ <portal file> @ x y z` while playing: the `GAME/Kq` `KQPortal` files (the six teleporters') are known to the rules; the portal gets its name (`@`: the next `NoName<n>`), `$ObjectName` is set; not drawn, no collider | `magic-map.md` rule 39; the portal's place on the line is not used (its keys carry the destination) | Viewer: Daventry keep pad, Barren shown, Go: "Davtele.cs line 17: loadKQ bartele.kq: the portal NoName23 to Barren.kq", Yes, `teleportdown`, Barren at (183000, 95500), `teleportup` |
| 6e-7: walking exits need nothing of their own: their traps' and collision scripts run and start the Portal state | Rule 1 | Viewer: Daventry, walk north from `noLoc` into `portaltp`: On → `portaltp.cs` → the question; No → `noLoc`, Off; walk in again: On, the question again (rule 3); Yes → Castle Daventry; west into `portalt2` → Yes → Daventry |
| 6e-7: Connor's `stop` in the Portal state is not played as a Special movement (the game is paused; after No he is back in Wait) | Rules 2–3; **ours** (a repeating `stop` in the Special state never ends: he could not walk after No) | As above |
| 6e fix: keys let go while the game was paused (the popup open, a world loading) count as let go on the next update: movement keys and the turn keys' own record (direction 0) | **Ours** (the pause, rule 26, stops the updates that would see the release; the user's check: Forward, then Left, stuck after No) | Not tested with real keys (the test keys don't go through the keyboard) |
| 6e-7: the movements of portals' own lists are loaded: this world's portals' `enterAnmFile` / `enterAnim`, and the arriving portal's `exitAnmFile` / `exitAnim` | Rules 4, 20.3 (data: `Elevator.anm`, `elevat2.anm`, `levator2.anm`, `poolport.anm`, `Snail.anm`) | Viewer: Barren `portal6`: `elevator.anm/c_goup`; Ice world: `elevat2.anm/c_comnup`, `eleport.cs portal6 Exit` |
| 6e-8: leaving a world keeps its variables named after it or `KQWorld` (not `SaveGameName`), the names of objects that left it (deleted, carried or used up by Connor, dead or gone characters) and its map's saved keys; arriving there again sets the variables over the start values (before `Init` and `Begin`), does not load those objects, and starts its map from the kept keys; the kept maps win over the shipped ones on the map's pages; arriving in Temple 1 drops the kept worlds, not the maps | Rules 8.1, 12, 13, 14 (variables after `WorldVar.cs`), 16; "Do not reproduce" 3 (in memory). Bodies of the dead not coming back: **a known gap** (rule 12 keeps them) | Viewer (`--begin`): Daventry → Castle → Daventry: "91 variables set", `FirstTime == 0` (no opening scene), "wizbook not loaded again", Daventry's map 1,696 → 2,078 pixels out of the fog (kept + new); to Temple 1: "1 kept worlds dropped, the maps kept" |
| K-1 (2026-10-26): the animated (not characters) and scripted objects whose movement name, place or collision type changed are kept with their movement, phase, place and collision type; **ours:** unchanged ones are not kept (the original keeps every object; they load as on a first visit) | Rules 12, 15; data: the user's `save10.vol` kept `Daventry.vol` (`water`: `currAnimName=blocked animPos=0.0625`; `Bricks`: `currAnimName=down animPos=1.0`, place moved by `bricks.cs`' `move`, `colliderType=65537`) | Unit test `only_objects_whose_movement_place_or_collision_changed_are_kept` |
| K-2 (2026-10-26): on a return a kept object's `currAnimName`, `nextAnimName`, `animPos` and (scripted objects) `colliderType` replace its file's values before it is read; its place and turn are the kept ones, no ground rule applied again (**ours**: the kept place is already final); `animPos` read for animated and scripted objects: past the movement's `loop=` start it becomes the phase | Rules 12, 15; `objects.md` "Animated objects at load" rules 3, 4 (`animPos`); data: `save10.vol` (the saved form holds these keys, `nextAnimName` empty for both) | Unit test `anim_pos_past_the_start_becomes_the_phase`. Viewer: bricks pushed, Daventry → Castle → Daventry: "water 'blocked' 0.5625; Bricks 'down' 1.0000 at (156652, 303400, 14562) type 0x10001", on return `water` movement 1 (`water2`) at phase 0.5625, `Bricks` frame 50 of 51 at the moved place, its box there, type 0x10001 |
| 6e-8 addition (2026-10-26): a kept world also keeps, in order, the prepared files a script's `loadKQ <file>` brought in while playing whose object is still there; on return they are loaded again right after the start script, as objects placed at the start (their `Init` with the others), from their files | Rules 12 (objects loaded by scripts while playing are in the kept world; deleted ones are not), 14.5 (one `loadKQ <name>.kq` per kept object, after the permanent objects), 15 (`Init` on entering). Their saved state (rule 12's "whole current state"): **a known gap**, they start as in their files | Unit tests `load_kq_while_playing_brings_a_prepared_copy_in_once`, `a_copy_brought_in_then_deleted_is_not_kept_with_the_world`. Viewer: bricks pushed, Daventry → Castle → Daventry: "2 loaded while playing ["waterfa2.kq", "watrfsn2.kq"]", both loaded again, `waterfall`/`WaterfallSound` not; Connor moved beside the falls: `waterfall2` "Off -> Near (drawn)", `watrfal2.aud` starts |

## Q153: the frame shown for a phase — `crates/kq8-data/src/shape.rs` (`frame_at_phase`), `crates/kq8-core/src/{anim,door}.rs`

Replaces the round() rules in the rows above (start frame, 3c, doors, 4d).

| Fact | Source | Checked by |
| --- | --- | --- |
| Frame = floor(phase × frames + 0.001), never rounded; the same for the start frame and while playing, repeating or not; at or past the end: repeating → 0, play-once → last; below 0: repeating → last, play-once → 0 | `objects.md` "Loops that move the whole object" rule 6 and "Animated objects at load" rule 2 (traced, Q153) | Unit tests `frame_at_phase_cuts_down`, `the_start_frame_is_the_still_models` (`telepad1` 0.3 × 16 → 4, was 5; King's portrait 0.99 × 46 → 45); viewer `doors.log`: `kingart` `stop2` "at phase 0.9900 (frame 45) speed 0" |
| Connor's own frames (`connor::MovementLoop::frame_at`) not changed | — (a separate step) | — |

## A plain object loaded by a script gets its click box — viewer `load.rs` (`object_colliders`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Every object prepared for a script's `loadKQ` (not only characters) gets its `BOUNDS.KQ` click box at the start, switched off until the load (core switches it on, as for the ravens) | `objects.md` "Where a world's objects come from" rule 6 (traced: the same steps as at the start); `clicking.md` rules 2, 8 | Viewer, Castle Daventry: `--aim keepkeys` before the push "meets nothing clickable"; after it, the key is taken (`INVITEM_KeepKey` 0 → 1, `keepkeys` deleted, +30 experience). Daventry: `bell`, `clicker2`, `MillTre2`, `potion`, `waterfall2` prepared the same way |
| Their solid box (Connor bumping into them) and view-blocking box are not made | **Stand-in** (not done yet) | — |

## Script events to Connor and to characters (the Shadow scene) — `crates/kq8-core/src/{active,connor}.rs`; viewer `connor_view.rs` (`script_anim`), `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQAnimPlayEvent <movement>` to Connor: asked of his walker as the keys ask (transition, cut short, else kept for the end), afresh even when already playing; the keys' choice is left alone, so they ask again only when it changes; only in his keys' states | `anim-scripts.md` rules 4-5 (every animated object, Connor included); `connor-movement.md` "Starting and stopping" rules 1, 4 | Unit test `a_script_request_plays_once_and_holds_while_the_keys_dont_change`; viewer `connor.log`: `portal` at update 25, `portalx` at 436 |
| A name not in his list: nothing | **Stand-in** (rule 5 gives movement 0 for objects; not checked for his list) | — |
| Outside his keys' states (Special, jumps, …): not done | **Stand-in** (not described) | Unit test |
| `KQCueEvent` to a character (or animated object) with a script: its script gets `Cue …` | `anim-scripts.md` rule 37 (the ravens' `Cue 10`), as for objects (`clicking.md` rule 21) | Viewer `doors.log`: the Shadow gets Cue 10, 30, 32, 300 × 13, 20, 21, 25 |

## `StateChange` messages (A-1, Q156) — `crates/kq8-core/src/{character,active}.rs`; viewer `connor_view.rs` (`state_number`), `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Every state change of a watched character (Connor too) sends `StateChange <who> <old> <new>` to each registered recipient's script, through the queue with no delay | `characters.md` rules 14-15 (traced, Q156) | Unit test `a_watched_state_change_reaches_the_recipient_with_old_and_new`; viewer `doors.log`: `StateChange Connor 32 4 to sdwtrap1` |
| State numbers: Birth 1, Wait 2, Move 4, Die 8, SwapWeapons 32, Jump 64, Push 128, Special 256, ThrowRock 512, Talk 1,024, Tile 2,048, rope 4,096, hammer throw 8,192, Portal 32,768, Choice 524,288 | `characters.md` rule 16 | As above |
| Connor: a weapon swap playing is SwapWeapons; Free mode is Move when the keys chose a movement, else Wait | Rule 16; `connor-movement.md` (Wait / Move by the keys' pick) | As above |
| Falling and landing count as Jump, the crash as Wait; a character's "no state" sends nothing; changes are looked for once per update (two in one update are one message) | **Stand-in** (our modes the rule doesn't name; no script tests them) | — |

## Connor's Choice state (A-2, Q155) — `crates/kq8-core/src/choice.rs`, `active.rs` (`set_state`); viewer `connor_view.rs` (`enter_choice`, `choice_step`), `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `setState Connor Choice <list>` must be four words; the list is a script, then `<number>,<x>,<y>` triples | `characters.md` "Connor's Choice state" rules 1-2 | Unit test `the_list_is_a_script_then_triples` |
| Entered at once from any state (an unfinished jump ended first); held keys cleared, `Stop`, the script gets `StartChoice` at once; points measured after it | Rules 3-5 | Viewer `doors.log` / `connor.log`: `StartChoice`'s `move` applied, then "points measured from (228239, 183179)" |
| All points on one circle of radius = the level distance to the first point | Rule 6 | Unit test `daventry_s_roof_points_and_picks` (R 8,239; point 1 ≈ (236,478, 183,277)) |
| Forward / back picks the point nearest a probe 200 ahead / behind (tie: the earlier); turn keys pivot; never walks; nothing drawn | Rules 7-8 | Unit test; viewer: forward at heading −1.57 chooses 0 |
| `DoChoice <the triple's number>` once (not every update) | Rule 10; "Do not reproduce" (fired every update in the original) | Viewer: one `DoChoice 0` |
| Before the choice: a weapon swap is taken and ends in a fresh Choice (`StartChoice` again, points again); script walks, Special lists, dying and the jump are refused and dropped | Rules 3, 11 (a death lost there: inferred by A) | Viewer: the `setDoMode 0` run |
| After the choice any other state ends it; it never ends by itself | Rule 12 | Viewer: "leaves the Choice state" when the walk starts |
| State number 524,288 while in it | Rule 16 | — |
| `StartChoice` after one of his movements ends in Choice (rule 9) not done; a key held from before entering doesn't choose after `handsOff 0` (rule 13, inferred) | **Gap / stand-in** | — |

## `loadObject` (A-3, Q157) — `crates/kq8-core/src/active.rs` (`load_object`), `fight.rs` (`Corpse::loaded`); viewer `load.rs`, `item_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `loadObject <name> <file> [x y z [rx ry rz]]`: up to six numbers, missing = 0; `@` → `NoName<n>`; no duplicate check | `anim-scripts.md` rule 34 (traced, Q157) | Unit test `load_object_makes_a_still_object_that_can_be_deleted` |
| `.kqs` / `.dts` → a plain object, one still pose, no script, no sound, not solid, not clickable (drawn like `fighting.md` rule 32's corpses); `.dig` → building (not done); other → "unknown object", nothing | Rule 34 (still pose: data; not solid: inferred by A) | Unit test; viewer `viewer.log`: "corpse of deadsprg: sp_cadth.kqs drawn" |
| z −1 / −2 → the ground height | Rule 34 (**inferred** by A, not used by any shipped line) | — |
| The shapes are made ready at load from the world's scripts' `loadObject` lines; the object is a world object by its name (`deleteObject` hides it) | **Ours** (the original loads the file when the command runs; `apothgnm.cs` deletes its `mortar`) | Viewer: "loadObject shapes made ready"; `deleteObject deadsprg` → hidden |

## `setScript` and `Init` (A-4, Q154) — `crates/kq8-core/src/active.rs` (`set_script`, `Active::start`)

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQObject::setScript <name> <script>` works on any object with a script slot; the new script gets `Init` at once, inside the command; `none` removes it with no call; an unknown name does nothing | `anim-scripts.md` rule 33 (traced, Q154) | Unit test `set_script_gives_the_new_script_init_at_once_and_none_calls_nothing` |
| Every character and item with a script gets `Init` when it enters the world (at the start: after the clickables) | Rule 33 ("the same call an object gets when it is added to the world"); `clicking.md` rule 21; `characters.md` rule 4 | Viewer `doors.log`: `bossprig.cs bossprig Init`, the start-up items' `Init` |
| A character given a script at the start gets only the new script's `Init`; its click box takes that script | **Stand-in** (rule 33: the type's `Init` first, then the new one's; no Daventry type script acts on `Init`) | — |
| `StateChange` from "no state": old number 0 | **Stand-in** (Q163) | Viewer: `StateChange bossprig 0 8` |

## Cue events and `random` (A-5, Q158) — `crates/kq8-core/src/active.rs` (`cue_event`, `cue_action`, `random`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A movement's cue `sendEvent <target> <Event> <numbers…>`: no delay word, queued for now; with `@`, everything before it is dropped and `@` is the object playing the movement | `anim-scripts.md` rule 35 (traced, Q158 a) | Unit test `a_send_event_cue_with_at_goes_to_the_object_playing_it_now`; viewer `doors.log`: the raven's `Cue 10` every pass |
| `random` 0–100, `random max` 0–max, `random min max`; whole numbers (decimals ignored); min + round(u × (max − min)); four or more words: usage, 0 | Rule 36 (traced; the 0 after the usage line inferred by A) | Unit test `random_gives_whole_numbers_with_the_ends_half_as_often` |
| An unknown target or event type in a cue: not sent, logged | **Ours** (as the script `sendEvent`) | Unit test |

## The experience sound and the level number on the meters (A-6, Q159, Q149) — `crates/kq8-core/src/stats.rs`, `active.rs` (`set_level`); viewer `gui_view.rs`

Replaces the L-2 / L-3 rows' stand-ins (DESIGN "Connor's levels checked").

| Fact | Source | Checked by |
| --- | --- | --- |
| `STATS.AUD` plays when the experience percent is set and is not 0 and at least the last percent shown; the last percent starts at 0 when the program starts and is never reset | `interface.md` rule 26a (traced, Q159; the file: the user's check) | Unit test `plays_when_not_0_and_not_lower_than_the_last`; viewer `gui.log`: 50 %, 80 % play, 15 % after the level-up doesn't |
| The percent is set after a stats change that changed experience, and when Connor is added to a world (the screen opens) | Rules 22, 26a | Viewer: "experience 0 % set" at the start, silent |
| The flame and `LEVEL.AUD` play when the level the meters show changes (any direction), not on a level-up as such | Rule 26 (traced, Q149 b) | Viewer: level 1 → 2 |
| `setLevel` sets only the number, sends nothing; the next stats change shows it | Rule 30a (traced, Q149 a) | Viewer Dead City: "nothing sent to the meters" |
| Not when the screen opens; the flame 17 frames at 15 per second, once; both sounds in group 0 | **Stand-in** (not traced) | — |

## Connor's frame for a phase (A-7, Q153) — `crates/kq8-core/src/connor.rs` (`MovementLoop::frame_at`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Frame = floor(phase × frames + 0.001) for his movements too (repeating or not); a play-once movement at or past the end shows its last frame; one played backwards (`undraw`) last to first | `objects.md` "Loops that move the whole object" rule 6 (traced, Q153; rule 1: the same path for Connor) | Unit test `his_frames_are_cut_down_as_every_object_s` |
| Repeating speed (N − 1)/(N × duration) and the wrap to exactly 0: **not built** (for any loop) | Rule 6 disagrees with `character-movement.md`'s speed and Connor's measured speed: **Q164** | A build of it slowed walkers by 1/N (reverted) |

## The hammer's way back, Connor's waiting request (A-8, Q148) — `crates/kq8-core/src/fight.rs` (`hammer_step`); viewer `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| At the turn-back (touch, range, floor grid) it is put back where it was at the start of that update; range back = its count (which includes that update's step) − one step = the distance to its start point | `azriel-hammer.md` rules 7, 9 (traced, Q148 a; corrected 2026-10-22) | Unit test `the_hammer_turns_back_at_its_range_and_at_the_ground` (14,400 / 14,400 / 20 updates, the note's worked example); viewer Swamp: range back 14,000 |
| One waiting request: a later refused one replaces it; it is tried before the new requests | "Connor's waiting request" (traced, Q148 d) | Viewer log line "kept as his waiting request" |
| Tried again every update; the "strong states" exception not done | **Stand-in** | — |

## Who sees Connor (A-9, Q146) — `crates/kq8-core/src/active.rs` (`connor_hidden_why`); viewer `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Characters are told he is 100,000 below while: the Invisible potion runs, `handsOff` hides the pointer (scripts, `walkTo` / `runTo`; not the hammer throw's), he is in his talk state (not remarks outside a conversation), or on the rope | `potions.md` rule 26 (traced, Q146) | Unit test `invisible_reports_him_100000_below`; viewer `characters.log` with the Wizard |

## The portal popup (A-10a, Q151) — viewer `gui_view.rs` (`popup_step`, `popup_line_box`, `popup_button_box`)

Replaces the 6e popup rows' stand-ins (highlight, Escape, rows, lines).

| Fact | Source | Checked by |
| --- | --- | --- |
| Yes is highlighted when the popup opens; Left / Right move between the two | `world-change.md` rule 26 "Details" (traced, Q151) | Viewer `gui.log`: "Yes highlighted", then "No highlighted" |
| Enter answers on release, only if pressed while the popup was open (one answer per press) | Rule 26 (a) | As built in 6e (unchanged) |
| Escape: nothing in `handsOff`; otherwise No and the main menu | Rule 26 (b); "Do not reproduce" (the original answers the other button) | — |
| Rows: highlighted 145–183, the other 146–182; lines at 12 + k × 18 + the offset table, 18 high, 360 wide, centred | Rule 26 (c), (d) (traced) | Viewer: lines at 49 and 75 |
| Pressed: frame 1 of the button's own picture (the highlighted one `buyhibtn.pba`), label 1 right and 1 down; a mouse press doesn't move the highlight | Rule 26 (e) (traced) | — |
| Button left edges 76 / 196 (W = 360, ours); no click sound on Left / Right | **Stand-in** (A inferred W = 370; the sound not named) | — |

## The loading meter's slot (A-10b, Q152) — viewer `loading_screen.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| The filled part is the meter's picture stretched to (630 × percent / 100, whole pixels, at least 1 if not 0) × 8; the unfilled rest of the slot is `4black.pbm`, stretched; the picture's painted frame stays (no full-width black bar) | `world-change.md` rule 23 (traced; the bar settled by the user's check, Q152) | Unit test `the_filled_part_is_cut_to_whole_pixels_at_least_1`; viewer `viewer.log` |

## Map icons and the mini-map (A-10c, Q150) — `crates/kq8-core/src/magic_map.rs` (`show_other`, `remove_icon`); viewer `gui_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Another world's map: its colours matched to this world's palette, then the icons drawn with their own indexes (not matched) | `magic-map.md` rule 34 "Order" (traced, Q150 c) | Unit test `paging_to_visited_maps_and_back` |
| `removeMapIcon`: the first icon at exactly x, y on the picture shown; the rectangle blank, then full where the fog is cleared, the fog tested in 8-column groups; other icons not redrawn (cut) | Rule 21 (traced, Q150 a) | Unit test `icons_over_the_fog_and_removed` |
| The mini-map slot never blinks: straight from the empty picture to the live map | Rule 27, `interface.md` rule 44 (traced, Q150 d) | Viewer `gui.log` |
| The removed icon's list slot is compacted, not left empty | **Ours** (only the saved numbering differs) | — |

## Status icon places (A-10d, Q143) — viewer `gui_view.rs` (`status_specs`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Each status icon (potions, the mask, the hook) is drawn in its own place, clipped to it; the 57 × 57 mask is shrunk to its 48 × 48 place | `potions.md` rule 35 "Details" (clipping traced; shrinking from the user's check, Q143 c) | Not tested (needs the user's look) |
| The shrink picks the nearest source pixel | **Ours** (how the original scales: not traced) | — |

## Dithered objects (A-11a, Q144) — `crates/kq8-core/src/active.rs` (`dither`), `kq8-data/src/objects.rs` (`dithered`); viewer `potion_view.rs` (`dither_step`, `dither_rgba`), `door_view.rs`

Replaces the 5g-7 stand-in (half see-through).

| Fact | Source | Checked by |
| --- | --- | --- |
| Modes 0 none, 1 dithered, 2 justTrans, 3 stored without a material change; unknown name: "cannot find" | `objects.md` "Dithered objects" rules 1, 2 (traced, Q144) | Viewer `viewer.log` |
| Mode 1: colour key on (black not drawn), full brightness, the texture rewritten so each row keeps every 4th texel, the next row shifted by 2; mode 2: colour key only; 0: back to its own | Rules 3-5 | Unit test `dithering_keeps_one_texel_in_four_rows_shifted_by_two`; viewer: the keep wall on and off |
| `dithered=` in the file: chests and animated objects drawn so from the start (they apply it at every movement start) | Rule 8 | Viewer: `wizchest1` 2 parts |
| Kept through a movement: a part whose material the drawing swaps back at a new frame is dithered again in the same drawn frame (the user's look: the chest lost it while opening) | Rule 8; the per-frame pass is **ours** | Viewer: 16 re-dithers of `wizchest1`'s 2 parts during `open` |
| The "2 extra texels per row", shared-texture holes; doors, characters, plain objects with the key | **Not done** (rule 6 inferred; Q165) | — |

## The shield orbits' path (A-11b, Q145) — `crates/kq8-core/src/potions.rs` (`orbit_place`); viewer `potion_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| The offset is (0, `orbitRadius`, 0) turned by the angle in world axes (0 = world +y); each update the angle grows by the step; the shape faces along the offset with the angle + π | `potions.md` rule 20 (traced, Q145 c) | Unit test in `potions.rs`; viewer `items.log`: `orbit1` 500 north of the centre at angle 0 |
| Three shields (`orbit1`-`3`) | Rule 20 (Q145 a) | As built in 5g-4 |
| x / y step angles not applied | **Not done** (all shipped files have 0) | — |

## Texture colours (A-11c, Q42) — viewer `load.rs` (`to_rgba`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A texture's own colour table is thrown away; every 8-bit texture is drawn through the world palette in use (entry 0 black) | `models-and-textures.md` "System colours" (traced, Q42) | Viewer `viewer.log`: 88 of 198 Daventry textures with their own table now differ in one colour from before |
| The teleport pad is drawn solid (no part of it see-through) | `objects.md` "See-through objects" rule 4 (traced; the user's screenshot, Q14 b) | Nothing to change: no `translucency` key |
| 24-bit textures as stored | **Not traced** (as before) | — |

## The bolt's start frame (A-12a, Q140 c) — `crates/kq8-core/src/fight.rs` (`character_do`); viewer `connor_view.rs`, `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| The `arrow` part is read from the frame the movement is on after the update's step (the queued `do` is handled after every object moved), for Connor and for characters | `ranged-weapons.md` rule 24 (traced, Q140 c) | Unit test `a_character_s_bolt_starts_at_its_arrow_part_found_once`; viewer `fight.log`: Connor's `attack` at 0.25, frame 2 |

## Connor's movements start at their `loop=` start (A-12b, Q138 e) — `crates/kq8-core/src/connor.rs` (`start_phase`, `fresh`); viewer `load.rs`, `connor_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Every movement of his starts at its `loop=` start and speed (`undraw`: frame 7 of 8, backwards, about 0.41 s) | `anim-scripts.md` rule 5 (traced, Q138 e) | Unit test `a_movement_starts_at_its_loop_start`; viewer `connor.log` |
| Transitions start at 0 | `connor-movement.md` "Starting and stopping" rule 5 | Unchanged |
| The start applied at the update after the movement is chosen (one drawn frame at phase 0 before) | **Ours** | — |

## A swap during a swing (A-12c, Q138) — viewer `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Standing: a weapon request waits for the swing's end; moving: taken at once, the swap's `away` / `out` cuts the swing (no hit before `do`) | `ranged-weapons.md` open question 1, answered in Q138 (traced) | Viewer `fight.log`: "moving: the swap cuts the swing … short" |
| "Moving" = a movement key held (our swing is a Special list, not inside Wait / Move) | **Ours** | — |

## Which jump (A-12d, Q112) — `crates/kq8-core/src/connor.rs` (`jump_kind`), `connor_update.rs` (`can_jump`)

Replaces the Q112 stand-in row (the wall block).

| Fact | Source | Checked by |
| --- | --- | --- |
| The jump comes from the movement the held keys ask for: back → `backJump` (at a wall, in water too), forward + running → `runJump` (water: `jump`), else `jump` | `connor-physics.md` "Jumping" rule 2 (traced, Q112) | Unit tests `a_wall_stop_blocks_until_the_key_is_released`, `movement_table`; viewer `connor.log`: `backJump` from `back` |
| Forward at a wall with running on: `runJump` while standing | Rule 2 (**inferred** by A) | Unit test only |

## Turn keys during the approach (A-12e, Q125) — `crates/kq8-core/src/connor.rs` (`update`)

| Fact | Source | Checked by |
| --- | --- | --- |
| In the talk state the turn keys turn him whatever movement plays (the approach too); a press cancels the automatic turn; forward / back do nothing until the stay-area test | `conversation.md` rule 13 (traced, Q125 a) | Unit test `gestures_play_only_while_he_stands`; viewer `connor.log` with the Wizard |

## The conversation camera's swing centre (A-12f, Q126) — `crates/kq8-core/src/conversation.rs` (`camera_moved_by_player`)

| Fact | Source | Checked by |
| --- | --- | --- |
| The centre is half-way between the partner's feet (taken at the start) and Connor's talking spot (`convLocX/Y/Z`), + 1,400; it doesn't move when he walks within the stay area | `conversation.md` rule 14 (traced, Q126; the user's check in the original) | Unit test `camera_numbers_cut_the_camera_until_the_right_button` |
| No info object: where Connor stands | **Ours** (never happens in the data) | — |

## `getConInventory` (B-1, Q166) — `crates/kq8-core/src/active.rs` (`get_con_inventory`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Nine variables `ConInv::Long`, `Short`, `Armor`, `FullArmor`, `UnderArmor`, `Pants`, `Gloves`, `Helmet`, `Boots` (= `FootWear`), in that order | `inventory.md` rule 20 (traced) | Unit test `get_con_inventory_gives_each_place_s_id` |
| Each = the `ID #` of the first carried item whose category is exactly that place, else 0; what he carries, not what is drawn; placeholders count (fists 1280, no ranged 1310, no armour 2120); a full suit only in `FullArmor` | Rules 21–24 (traced + data) | Unit test; viewer `items.log`: Short 1280 → 1020 after `give SmallAx` |
| `$Console::Return` always 0; nine lines `ConInv::Long:       1310` (to our log, which is our console) | Rules 25–26 (traced) | Unit test |
| "can't find Connor" / "…Connor's inventory" errors not made: our Connor and his list always exist | **Ours** | — |

## `exist` (B-2, Q162) — `crates/kq8-core/src/active.rs` (`exist`; the dead removed after `Die`)

| Fact | Source | Checked by |
| --- | --- | --- |
| `exist <name>`: `$Console::Return` 0 if the name is a world object now, 1 if not; `if` takes its branch on 0 (as before) | `anim-scripts.md` rule 38 (traced) | Unit test `exist_gives_0_when_the_object_is_there`; viewer `doors.log`: "exist sprgnbow: there: 0" |
| World objects = our `known` names (placed objects, characters, Connor, items, `loadKQ` objects once loaded, named sound objects) less the deleted ones; a name deleted in the same event still counts | Rule 38 (traced; the list's members partly inferred by A) | Unit test |
| A dead character is removed right after its own `Die` script (its corpse has no name of its own): `exist` gives 1 afterwards, 0 while dying | Rule 38, `fighting.md` rule 32 | Unit test `a_character_dies_then_gives_experience_a_corpse_and_goes` |
| Wrong word count: "usage: exist Object" and 0; prints nothing otherwise (we log every answer) | Rule 38 | Unit test |
| A word tried as an object's internal number | Rule 38 (no data uses it): **not made** | — |

## Loop speed (B-3, Q164) — `crates/kq8-core/src/door.rs` (`repeat_rate`, `repeat_advance`, `DoorMovement::step_frames`, `loop_ground_speed`), `named_anim.rs`, `character.rs` (`movement_step`), `connor.rs` (`MovementLoop::frame_rule`, `phase_rate`, `repeat_advance`, `ground_speed`); viewer `load.rs` (`Animation::first_in_file`), `anim_view.rs`, `connor_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| A repeating loop of N frames moves (N − 1)/(N × length) in phase per second: one cycle = N × length/(N − 1) | `objects.md` "Loops that move the whole object" rule 6 (traced, Q153/Q164) | Unit test `a_framed_loop_moves_by_rule_6a` |
| A forward wrap sets the phase to exactly 0 (the overshoot dropped); backwards below 0: the last frame, phase (N − 1)/N | Rule 6 | Unit test |
| An update adds the steps of the frames it enters; entering frame 0 forwards adds the **last** frame's step when the loop starts at the file's first frame; backwards the frames' steps are taken off, no swap | Rule 6a (traced, Q164) | Unit test; viewer `anim.log` "character speeds" = `character-movement.md` rule 2's table |
| Applies to Connor's and characters' loops | Rules 6 (every `.kqs` animation), `connor-movement.md` "Forward speed" rules 3–4, `character-movement.md` rules 1–2 | `kq8-artifacts/logs/b3-connor-speeds.log`: loaded 1,124.0 / 2,558.4 / −1,070.3; measured 1,119–1,122 / 2,562 / 1,066 |
| "Below 0" backwards is read on the frame (floor(p × N + 0.001) < 0), so float sums a hair under 0 still show frame 0 | **Ours** (a plain p < 0 skipped frame 0 every cycle in a test: back came out 1,127) | Unit test |
| Loops with no per-frame steps (tests only) keep the even spread over 1/length | **Ours** (test scaffolding) | Existing tests |
| Plain animated objects, chests and doors keep the old speed and wrap | **Gap**: rule 6 covers them too; not in this step's scope (the user's B-3: Connor and characters) | — |

## `setState <name> Wait` (B-4, Q160) — `crates/kq8-core/src/connor.rs` (`Walker::enter_wait`, the end of a movement in Wait), `character.rs` (`set_state`), `active.rs` (`Request::ConnorWait`); viewer `connor_view.rs` (`enter_wait`), `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| From Wait or Move the change is at once: a fresh Wait that asks for `Stop` the ordinary way, even when already in Wait (Connor and characters) | `characters.md` rule 22 "`setState <name> Wait` in detail" (traced) | Unit test `set_state_wait_re_enters_wait_and_asks_for_stop`; viewer `connor.log` "setState Wait (shadow.cs line 31): from Wait" |
| From Special only once the list has no words left; meanwhile kept waiting and the list goes on (it ends in Wait by itself) | Rule 22 (traced) | Unit tests (Connor; character `special_lists_play_in_turn_then_wait`) |
| In Wait, a movement other than `Stop` / `gesture…` that ends asks for `Stop` (Connor: from its start, an ordinary request) | Rule 22 (traced; characters already had it as rule 18) | Unit test `a_script_request_plays_once_then_wait_asks_for_stop`; viewer: "portalx ended in Wait: asks for stop" |
| Connor's Wait = our Free mode with the keys' choice `stop`; a held key asks for a walk again at the next update | **Ours** (as `state_number`) | — |
| `setState Connor Wait` from his other states (jump, push, rope, talk, throw): nothing, logged; during a jump or the rope it goes through the same refusals as `setState special`; before a Choice it is refused | **Gap / stand-in**: the rule names only Wait, Move and Special | — |

## `setHaze` (B-5, Q161) — `crates/kq8-core/src/active.rs` (`Request::SetHaze`); viewer `object_light.rs` (`SetHaze`, `apply_set_haze`), `door_view.rs`, `main.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `setHaze <n>`: `hazeDistance` = n, `visibleDistance` = 1.5 n, at once; `$Console::Return` 0; wrong word count: "usage: setHaze hazeVal", nothing changes | `object-lighting.md` rule 33a (traced) | Unit test `set_haze_hands_the_number_over_and_answers_0`; viewer `viewer.log`: "Haze = 5000; Visible = 7500" |
| The ground's haze and end reset at once; objects' haze and the switch-off distance from the next frame | Rule 33a | Viewer: 2,174 object, 123 ground, 176 building materials changed; objects off nearer than 27,000 afterwards |
| Four console lines `Perspective`, `Haze`, `Visible`, `FarPlane` (to our log); `perspectiveDistance` not kept by us, so printed as unchanged | Rule 33a; **ours** for the missing value | Viewer log |
| The camera's far limit: our projection keeps its own far plane (2 × the terrain's size, as at load); the ground shader ends at `visibleDistance` and objects switch off there | **Ours / stand-in** (the far plane is logged, not set) | — |
| Kept until the next `setHaze` or a world load (the new world's `TERRAIN.KQ`); saved games: not traced | Rule 33a; **open** | — |

## `dithered=` on every class (B-6, Q165) — viewer `door_view.rs` (the file values at load, the templates); `crates/kq8-core/src/active.rs` (`Template::dithered`, `load_copy`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Every class applies its file's `dithered=` when added to the world: at load (placed objects, doors, chests, animated objects, `KQAnimOnce`, characters, plain objects) and a `loadKQ` copy when it is made | `objects.md` "Dithered objects" rule 8 (traced, Q165) | Viewer: Daventry `arroswt1`-`3` (doors) 2, `wizchest1` 1; Gnome `ironlock`, `Irnlckx`, `Mystic`, `Lucreto` 2; unit test `load_kq_while_playing_makes_new_copies_by_the_name_and_place_rules` |
| Objects that play movements apply it again at each movement start: our drawing puts the stored mode back whenever a part's material changes (A-11a's `dither_step`) | Rule 8 (traced; doors inferred) | Viewer Gnome: "7 parts drawn dithered" again as frames change |
| The values (mode 1 only `wizchest`, `snowmane`; the rest 2) are read from the files as they are | Rule 9 (data) | Viewer log |

## Panel pictures (B-7, Q108 b) — `crates/kq8-core/src/gui_draw.rs` (`Piece::Picture`, `Piece::Bar`, `Canvas::picture`, `bar`, `picture_lower`), `caption.rs` (`Strip::at`); viewer `gui_view.rs` (`Spec::Pic`, `Spec::Bar`)

Replaces the "smaller pictures tiled" stand-in (4c-1, 4c-8).

| Fact | Source | Checked by |
| --- | --- | --- |
| A picture whose layout modifier has 0x8000 is stretched to its box (width and height separately, nearest pixel); without it, drawn once at its own size from the top-left; never tiled | `interface.md` rule 20a (traced) | Unit test `pictures_are_stretched_or_drawn_once_bars_end_in_black_and_the_first_piece_is_on_top` |
| A bar: its picture stretched over the shown width × the bar's height, `4black.pbm` (tag 351) stretched over the rest; nothing if either is missing | Rule 20 (traced; replaces 4c-8's plain black fill) | Unit test; viewer `gui.log`: "bar 4green.pbm stretched over 81 of 108 px (first pixel [0, 72, 0]), 4black.pbm over the rest (first pixel [0, 0, 0])" |
| Attribute kind (lowest 3 bits) 1 = colour 0 not drawn, for pictures from layout controls; others keep their own flag's bit 1 | Rule 20a (traced kinds; the layout's attribute being the number used: inferred by A) | Unit test |
| ~~The caption's stone stretched to the whole caption box~~ **Undone the same day**: back to tiled (the 4c-1 stand-in) | Rule 20a (inferred by A for the caption dialog); the user saw the stretched stone broken and checkered, the tiled one matched the original: **Q168** | The user's look |
| The status icons (potions, mask, hook) and the popup's buttons: stretched (their controls have 0x8000), see-through from the picture's own flag | **Ours** (their attribute is 0x8 in the layout; not switched, to keep the user-checked look) | — |
| Rectangles × 1.25 at 800 × 600, fonts at their own size | **Stand-in**, Q108 a, c (open) | — |

## "No state" is 0 (B-8, Q163) — `crates/kq8-core/src/active.rs` (`state_changes`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Every character starts in "no state", numbered 0; leaving it sends `$4` = 0 (start-up Wait 0 → 2; killed before its first update 0 → 8) | `characters.md` rule 16a (traced) | Unit test `killed_before_its_first_update_it_reports_leaving_no_state_as_0`; viewer `doors.log`: "StateChange bossprig 0 8" |
| A character our once-an-update look has never seen counts as 0 (was: nothing sent) | Rule 16a; **ours** for how we look | Unit test (failed before the fix) |
| A change **to** no state is never sent: we ignore requests for unknown states | Rule 16a, "Do not reproduce" | — |
| Connor: unchanged (no first message); the rule speaks of characters | **Ours** | — |

## `getLoc Connor` gives his turn (fix after B-2, 2026-10-24) — `crates/kq8-core/src/active.rs` (`place_of`, `connor_heading`); viewer `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `$KQObject::loc` holds all six numbers of the object's place, turn included; for Connor the turn is his heading (the number `move Connor … rz` sets) | `objects.md` "Script variables, `getLoc` and `move`" rule 1, `anim-scripts.md` (all six); heading = rz: **ours** (as `move_to` uses it) | Unit test `get_loc_while_playing_feeds_set_home`; viewer: `deadsprg` turned −1.570 |

## Laying an item on a model part (C-1) — `crates/kq8-data/src/shape.rs` (`part_transforms`), `objects.rs` (`angles_of`); `crates/kq8-core/src/item.rs` (`laid_on`, `Items::body`); viewer `load.rs` (item bodies); CLI `shape_parts.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| A laid item's own model part `body` is put exactly on the named part (place and full turn) | `fighting.md` rule 31a, `anim-scripts.md` rule 16 (traced) | Unit test `a_laid_items_body_sits_exactly_on_the_part` |
| Row vectors; combining "first A, then B" = A.M × B.M, A.p × B.M + B.p | `models-and-textures.md` Q1 a (confirmed) | (same test) |
| A model part's placement is its mesh node's matrix and translation | `models-and-textures.md` Q1 b; layout: libkq8fpc `kq8kqs.txt` (facts only) | `kq8 shape-parts` |
| An item's three angles: turned about Y, then X, then Z; `angles_of` turns a plain turn back into them | `objects.md` Q9 rule 4; the inverse is **ours** (maths) | Unit test `angles_of_gives_back_the_angles` |
| The part and the item's `body` have the same scale in the data (crossbow 1 and 1, broadsword 0.885 and 0.885), so the laid turn is a plain one | Observed (`kq8 shape-parts` on `spbdie.kqs`, `scbow.kqs`, `skbsdie.kqs`, `bsword.kqs`) | Unit test (scale within 1 %) |
| An item is drawn with its model's rest frame, so `body` is read from that frame | **Ours** (as the item drawing) | `items.log`: "part body read for 23 shapes" |
| A model without a `body` part (`pouch.kqs`): its own origin goes on the part | **Ours** (no data case for a laid item) | — |

## A dying character's weapon is laid, not thrown (C-2, Q169) — `crates/kq8-core/src/fight.rs` (`drop_items`, `lay_kept_drop`, `laid_checks_step`, `throw_made`); `active.rs` (`NewInvItem` after a death); viewer `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Each drop entry (quotes and `INVITEM_` off, capitals ignored) is compared with the named parts of the pose at the dying start; one that names a part is kept, the last one wins, earlier ones are lost | `fighting.md` rule 31 (traced) | Unit test `a_dying_characters_weapon_is_laid_where_the_death_leaves_it`; viewer `items.log` |
| That pose is the die movement's first frame | Rule 31 (inferred by A; matches the user's check) | `items.log`: "parts of 'Die' frame 0: [body, SmCrossbow, arrow, blood]" |
| The other entries are thrown as before (`tossItems`) | Rule 31 (traced, user-checked) | `items.log`: `skeleton4`'s coins thrown, its sword kept |
| At the die movement's end, first of the end steps: the item's `body` on the same-named part of the **die movement's** last frame, from the character's place and turn; no such part: its place and turn, 750 up | Rule 31a.1 (traced) | Unit tests (with a waiting movement started at the end); `items.log` |
| The die movement's last frame even when a waiting movement (`Stop`) has just started in its place | Rule 31a.1 ("the character's last frame"); reading it as the die movement's: **ours** (found in Dead City, where `Stop` had started) | Unit test (fails with the shown pose) |
| Line test from 1,000 above the item to 1,000 above the feet; blocked: thrown from the feet instead; free: stays, no fall | Rule 31a.2, 31a.3 (traced; "never falls later" inferred by A) | Unit tests `a_dying_characters_weapon_is_laid_where_the_death_leaves_it`, `a_laid_weapon_behind_a_wall_is_thrown_instead` |
| What blocks: terrain, buildings, objects and characters of any collision type, not the dying one | Rule 31a.2 says "any solid object"; the types: **stand-in** (asked: Q170) | — |
| The line test runs in the thrown items' step of the same update (it needs the collision scene) | **Ours** (timing) | `items.log`: laid at update 113, tested at 114 (the counter has moved on in that step, as for the flights) |
| `NewInvItem` is set to the item a death made last (thrown or laid), before the `Die` calls | Rules 31, 31a ("as for them"); when it is set: **ours** | Unit test; `doors.log`: "NewInvItem = NoName22" |
| `Die now`: the same steps with the pose shown | Rule 31a: "not sure"; **stand-in**, logged | — |

## `Inv <item>` in Connor's Special list lays a real item (C-3, Q167) — `crates/kq8-core/src/connor.rs` (`SpecialStep::Inv`, `Walker::inv_laid`), `fight.rs` (`lay_inv_item`, `laid_place`); viewer `connor_view.rs` (`take_inv_laid`), `door_view.rs`, `load.rs` (models of `Inv` items)

| Fact | Source | Checked by |
| --- | --- | --- |
| `Inv <item>` puts a real item in the world (not in his hands, not in his inventory) | `anim-scripts.md` rule 16 (traced) | Unit test `inv_in_connors_special_list_lays_a_real_item_on_the_named_part`; viewer `items.log` |
| Handled when the movement before it ended: his pose is that movement's last frame; the next items follow at once | Rule 16 (traced; "last frame" inferred by A) | Unit test `special_plays_each_movement_to_its_end_then_stands`; `connor.log` |
| The item's `body` on the part named like the item (capitals ignored), from his place and turn; no such part: his place and turn, 750 up | Rule 16 (traced; the name match ignoring capitals inferred from the data) | Unit test; `items.log`: "on part 'SmCrossBow' of 'sprgnbow.anm/hdthree' frame 54 ... scale 1.000" |
| His place and turn = where he is drawn: his position and his full turn (lean, then heading) | **Ours** (as `part_now`, the shield's orbit) | — |
| No throw, no fall, no line test, `NewInvItem` not set | Rule 16 (traced) | Unit test |
| An item named in a Special list's `Inv` has its model loaded | **Ours** (the loader's scan of the scripts) | — |

## The level-up flame and the slot flashes see-through again (fix after B-7, 2026-10-25) — viewer `gui_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| The flame (`flame14.pba`) and the flashes (`fx.pba`) leave colour 0 undrawn by their file's transparent flag (flags 0x9), not by the attribute's kind (0x8, solid by `interface.md` rule 20a) | **The user's check in the original** (see-through; ours were black); file flags observed (`kq8 bitmap-png`); asked: Q171 | `gui.log`: the flame and three flashes play at a level-up; not looked at by eye |

## Rule 6's speed and wrap for every animated object (2026-10-25) — `crates/kq8-core/src/door.rs` (`step_with`), `named_anim.rs`, `anim.rs` (`LoopPlayer::advance`)

| Fact | Source | Checked by |
| --- | --- | --- |
| The same frame player for every `.kqs` animation: doors, chests, plain animated objects, characters, Connor | `objects.md` "Loops" rule 6 (traced, Q164) | — |
| A repeating loop moves (N − 1)/(N × length) a second; a forward wrap lands on exactly 0; backwards below 0 on the last frame, phase (N − 1)/N | Rule 6 (traced) | Unit tests `a_repeating_loop_shows_every_frame_and_wraps_to_exactly_0`, `cues_at_a_wrap_fire_once_including_one_at_exactly_1`; `anim.log`: `windprop` 0.0164 a frame per update, `Davtorch.01` wraps to 0.0000 |
| After a forward wrap a cue at 0.0 fires in the next update | Rule 4 (traced, Q158 c) | Unit test (same) |
| A one-frame repeating loop keeps the old timing (1/length a second, wrap carrying the overshoot) | **Ours** (the rule gives rate 0, so its cues would never fire) | — |
| Objects are not moved by their loops' steps (only characters) | **Ours** for now (rule 6a's adding up for objects waits for M6) | — |

## Smooth joins: partners across a change of movement (J-1, 2026-10-25) — `crates/kq8-data/src/anim_pack.rs` (`layout`, `name_hash`, `NodeFrame::name`/`layout`, `Pack::partners`, `Pack::join_share`); viewer `world_build.rs` (the log line)

| Fact | Source | Checked by |
| --- | --- | --- |
| Each of Connor's and the characters' movements is its own `.kqs` file, listed by `loop=` lines in an `.anm` list | Observed (`Conner.anm`, the worlds' `.anm` files) | — |
| Two meshes in different files can be mixed point by point only if they have the same number of points and the same triangles over the same point numbers; the order of triangles, corners and of parts in a frame doesn't matter; the part's name must match | Observed (measured 2026-10-25: Connor's `body` is one 196-point layout in 64 of 70 `Conner.anm` files; zombies' meshes have equal sizes but other point numbers) | Unit tests `a_layout_ignores_the_order_of_triangles_but_not_the_point_numbers`, `a_new_frames_parts_find_their_partners_by_name_and_layout`; `viewer.log` (Daventry): Connor's own set 2,653 of 3,192 changes with every part paired |
| Drawing a change of movement mixed at all, partners taken in order, one old node each | **Ours** (drawing only, DESIGN "smooth joins"; the game's rules unchanged) | — |

## Smooth joins: the tag and the join buffer (J-2, 2026-10-25) — viewer `anim_gpu.rs` (`tag`, `join_data`, `AnimBuffers::joins`), `object_light.rs` (binding 7), `object_light.wesl` (`model_corner`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Tag split 18/6/8 bits; 255 join records written every frame; a joined pose drawn as a straight-line mix from the old part's pose | **Ours** (how we draw; DESIGN "smooth joins") | Unit tests `a_tag_holds_the_node_frame_the_amount_and_the_join`, `the_join_buffer_keeps_its_size_and_record_0_is_none`; `viewer.log` Daventry: 72,255 node-frames, frame time unchanged |

## Smooth joins: a change of movement drawn mixed (J-3, 2026-10-25) — viewer `anim_gpu.rs` (`Slots::start_join`, `join_over`, `end_join`, `join_tags`, `JoinReport`), `smooth_view.rs` (records, `join_length`, `eased`, `write_joins`), `console_words.rs` (`smooth join`), `connor_view.rs`, `anim_view.rs` (logs)

| Fact | Source | Checked by |
| --- | --- | --- |
| A change of movement drawn mixed from the old pose over 0.15 s, eased; the old pose frozen at the change; a change during a join starts a new one | **Ours** (drawing only; the game's movements, frames, cues and steps unchanged) | `connor.log` / `characters.log` join lines (Daventry); not looked at by eye |
| Connor's `back` is `walk`'s animation played backwards | Observed (`Conner.anm`: `back` = `c_walk.kqs` at speed −1) | `connor.log`: "stop -> back" |
| `c_run.kqs`'s body is `c_walk.kqs`'s with about a dozen points numbered otherwise | Observed (scratch check of the two files' texture places) | `connor.log`: "walk -> run: 0 of 2 parts" |

## Fix 2026-10-25: an object hidden by its file is drawn once shown — viewer `draw_distance.rs` (`update_activity`; first in `anim_view.rs`, moved the same day)

| Fact | Source | Checked by |
| --- | --- | --- |
| An object with `hidden=1` (character, animated or plain) is drawn after a script's `setHidden <name> 0`; hiding it again is the drawing layer's (rule 36) | `objects.md` "Hidden objects" rule 3; `characters.md` rules 34 and 22 (`setHidden`, the `sprdump` example) | `viewer.log` "shown by a script": Daventry `sprdump` (door scene; its shadow from update 33), Barren `rail1l`, Gnome `gemfuse`, temple4 `Lucreto` (console); `sprdump` hidden again: Near -> Off; not looked at by eye |

## Fix 2026-10-27: a scripted object with no box keeps its script (the Shadow fight in Dead City) — viewer `load.rs` (the collision loader's `script_only`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Every object with a script gets `Init` and its `Cue` calls, whatever it collides with | `clicking.md` rule 21 | `doors.log`: "script ShadBat.cs ShadBat Init"; "KQCueEvent 10 to ShadBat" |
| A portal's `exitScript` runs when the `exitAnim` ends, `$1` = the portal, `$2` = `Exit` (`shadbat.cs`) | `world-change.md` rule 20.4 (already built in 6e-4b) | `doors.log` "script shadbat.cs davgate4 Exit" |
| Such an object (type 0 and mask 0, or no `BOUNDS.KQ` box) is kept with an empty box, so it is never clicked, hit or picked as an attack target | **Ours** (the note gives it no box; the click and attack picks test the box) | — |

## 2026-10-27: `makeTranslucent Connor` (the Shadow fight) — core `active.rs` (`make_translucent`, `connor_translucency`); viewer `potion_view.rs` (`translucent_step`)

| Fact | Source | Checked by |
| --- | --- | --- |
| `makeTranslucent <name> <n>` sets the see-through value at once, no fade; 256 = solid; every material of every part at n / 255 | `characters.md` rule 22, `objects.md` "See-through objects" rule 2 | unit test `make_translucent_sets_the_value_on_an_animated_object`; `connor.log` "makeTranslucent 180: opacity 1.000 -> 0.706" |
| In Dead City's fight the Shadow is part of Connor's model (`bodyshadow`, `shadowbody`, `shadow`, `pixibody` in `c_p2sm*.kqs`), so `makeTranslucent Connor` makes it see-through too | Observed (`kq8 shape-parts deadcity/resource/c_p2sm<n>.kqs`, `ShadBat.cs`) | — |
| Connor's value is not carried to another world; the haze fade of rule 2 is not applied to Connor | **Ours** (not traced; the scene ends solid) | — |

## Fix 2026-10-27: Dead City's Paddle shows on its pedestal — viewer `load.rs` (`no_place`), `door_view.rs` (`Request::Move` of a copy not drawn yet); core `active.rs` (`move_command`: a box object's click box)

| Fact | Source | Checked by |
| --- | --- | --- |
| A file with no `locX/Y/Z` starts at (0, 0, 0), then the `keepOnGround` raise | `objects.md` "Missing locX / locY / locZ" rules 1–3 | `viewer.log` "template vol:DeadCity/kq/paddle2.kq[paddle2]: kept" |
| `loadKQ` while playing runs the same steps as at the start, whatever the file's position | `objects.md` "Where a world's objects come from" rule 6 | `doors.log` "loadKQ paddle2.kq: a new copy paddle2" |
| `move` sets an object's place directly; a copy loaded in the same update is drawn at the moved place, and a box object's click box moves with it | `objects.md` "(d)" (`move`); the split between core and viewer is **ours** | `traps.log` "moved paddle2 (a new copy, not drawn yet) to (175006, 58451, 1177)"; `doors.log` "distance to its box 1223.1" |
| What pushing a `KQPaddleObject` does | **Not built**: Q173 (`pushing.md` open question 1) | — |

## P-1 2026-10-27: a cut during a camera flight — viewer `game_camera.rs` (`cut_to`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A `cutToCamLoc` does not end a running flight: its remaining steps (and roll steps) are added from the cut's spot; its last update switches "places" on and pulls the camera onto the range around the cut's aim point + 1,400, then the height rule; "places" then stays on | `camera.md` "Camera flights and cuts" rules 4 and 6 (Q172) | `camera.log` "flight 13/20" … "flight 20/20"; `traps.log` "cut to the spot (182221, 45750, 2000), pulled onto the range from 795.1" |

## P-2 2026-10-27: the Paddle's push — core `active.rs` (`paddle_push`, `PADDLE_*`, `Template::class`); viewer `load.rs` (`padpush.anm` preloaded), `door_view.rs` (the template's class); `console_words.rs` (`set`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A `KQPaddleObject`'s pushing touch sends no `Push`; the class runs its own push at once | `pushing.md` rule 11a | unit test `a_paddle_runs_its_own_push_by_connors_side`; `doors.log` "paddle2 (KQPaddleObject push …)" |
| `handsOff` on, third person locked, Connor to Wait | rule 11b | `traps.log` (our Push state refuses Wait: logged; the Special right after is taken) |
| Side A / B by the side of the line through the paddle along its facing (plain test); fixed world offsets by side and west/east (x < 182,000); turn ± 1.5708; the paddle's height | rules 11c, 11d, "Do not reproduce" | unit test; `doors.log` side A (174406, 57881), side B (175576, 57941) |
| Camera: `addCamLoc` spot ±2,000 on x, `cutToCamLoc` at Connor's new place, roll 0, follow Connor | rule 11e; `follow Connor` setting both switches on is **ours** | `traps.log` "cut to the spot (173006, 58451, 1177)" |
| The paddle's type to 0x21, `Scary2`–`Scary5` to 0 (stored type too) | rule 11f, "Do not reproduce" | `doors.log` "collisionType paddle2 -65792: 0x10121 -> 0x21" |
| Special `padpush.anm` with `pushccw` / `pushcw`, or `pushoff` / `pushoff2` when `DeadCity::Paddle::Used` = 3; then `padpush.cs` / `padpush2.cs` with `$1` = the paddle | rule 11g | unit test; `doors.log` |
| `padpush.anm`'s four movements preloaded when a script-loaded file of the world has `classType=KQPaddleObject` (no script line names them) | **Ours** (how we load; rule 11g says the class asks for them) | `viewer.log` "special padpush.anm/pushccw" |
| Console `set <variable> <value>` | **Ours** (a test helper) | `console.log` "DeadCity::Paddle::Used = '3' (was '0')" |

## Fix 2026-10-27: a reloaded name finds the live object (the 2nd–4th Paddle) — viewer `door_view.rs` (`Request::Move`, `is_switched_off`); core `active.rs` (`live_clickable`, `anim_index`, `object_index`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A deleted object's name is free again once its event has finished; a later `loadKQ` makes a new object of that name | `objects.md` "Where a world's objects come from" rule 6, `clicking.md` rule 22 | `doors.log` "copy paddle2 of paddle2.kq drawn at (175006, 66474, 1177)" |
| Commands naming it act on the new, live object (not a deleted one still in our lists) | Follows from the rules above; how we keep deleted objects listed is **ours** | `doors.log` "getLoc UniqueTikiName2: 175006 66474 1177"; "the paddle at (175006, 66474, 1177)" |

## SC-1 2026-10-27: `kq8 script-coverage` — `crates/kq8-cli/src/script_coverage.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| What the scripts use: commands, Connor's states, event kinds, `$2` calls, classes; start-up scripts = `newWorld.cs` and the scripts it names | Observed (the game's `kq.vol` archives and `GAME/Kq`); the start-up rule from `objects.md` "Where a world's objects come from" rule 1 | Its output, `docs/SCRIPT-COVERAGE.md` |
| Built / stub / missing: one probe line per command in an empty world, read from our own log | **Ours** (a tool; says nothing about how the original behaves) | — |

## Batch 2 (Dead City), D-1 2026-10-30: `KQMoveEvent` on objects — core `active.rs` (event delivery, `EVENT_TYPES`)

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQMoveEvent <flags> x y z rx ry rz [extra]` to an object = `move` with the same flags (bit 1 add the position on world axes, bit 2 combine the turn); only 7 numbers read | `objects.md` "Loops that move the whole object" rule 3 (batch 2) | unit test `a_move_event_to_an_object_moves_it_with_the_move_flags`; `doors.log` |
| A relative `KQMoveEvent` sent by an object's own movement cue is not applied | **Ours (stand-in, Q176)**: we draw a loop's pose from its phase, so the leftover the cue cancels does not exist here | `doors.log` "not applied, our loops leave no leftover" |

## Batch 2, D-2 2026-10-30: `tileExit`, `deadHallTeleport` — core `active.rs` (`tile_command`, `Request::TileExit`, `Request::HallTeleport`); viewer `door_view.rs`, `connor_view.rs` (`tile_exit`)

| Fact | Source | Checked by |
| --- | --- | --- |
| `tileExit 0|1` sets the tile state's exit switch (1: Wait at once); nothing in other states; the usage line | `tile-puzzle.md` rule 21 | `traps.log` |
| `deadHallTeleport <from> <to>`: Connor and the camera moved by `to` − `from` (positions only, heading and aim kept); the "cannot find object" and usage lines | `tile-puzzle.md` rule 22 | `traps.log`, `connor.log` "offset (55043, 39207, 1920)" |
| The newest building of a name is used (a hall loaded again by a script) | **Ours** (how we keep buildings) | — |
| Near/far worked out again at the next frame (ours does it every frame); the 0.5 s notice not done | rule 22; open question 2 | — |

## Batch 2, D-3 2026-10-30: the death-hall stones' rules — core `tile_puzzle.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Grid: `tile<r><c>` at x 230,333 + 2,000 c, y 124,297 + 2,000 r, z 2,871 + 500 r; extra spot rows −2, −1, 8 with the nearest stone row's z | `tile-puzzle.md` rules 3, 5 | unit tests |
| Connor's spot: area x 229,333–245,333, y 119,297–139,297; column and row by whole 2,000s; else south / north / west / east | rule 6 | unit test `spots_by_rule_6` |
| Stones read by name when the state starts; a missing one skipped and named (the original crashes) | rule 2, "Do not reproduce" | unit test |
| Jump target: the neighbour nearest his facing (or straight behind), columns clamped, not his own spot, no back diagonals from the start strip; straight / diagonal movements | rules 9, 11 | unit test |
| Order D `tile07`, E `tile63`, A `tile21`, T `tile56`, H `tile10`; put-on stones and camera points; start and its camera point; +1,000 / +2,000 | rules 4, 13, 14 (kept as a table: "Do not reproduce") | unit test |
| Landing: right / door (row −2) / start strip / wrong | rule 12 | unit test |
| Lines: condition 3 on the first teleport, 133 the first time sent back after a right stone | rule 15 | unit test |

## Batch 2, D-4 2026-10-30: Connor's `Tile` state — viewer `connor_tile.rs` (`ConnorSim::enter_tile`, `tile_step`, `tile_exit`); `connor_view.rs`, `door_view.rs`, `copies_view.rs`, `load.rs`; core `active.rs` (`Request::ConnorTile`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Entering: stones read, expects D, exit switch off, a drawn weapon put away (`away`) else `stop` | `tile-puzzle.md` rule 7 | `connor.log` "enters the Tile state: 64 of 64 stones" |
| Keys: turn on the spot; a jump on a change of the wanted movement, cleared at each landing (holding forward keeps jumping); keys ignored during a jump or teleport; walking south of the area | rule 8 | `connor.log` (three strip jumps from one held key) |
| The turn to the target (or its mirror for a back jump) before the jump movement | rule 11 | `connor.log` "turns to (…) first" |
| The turn is our scripted turn (`pointAndCue`'s); the jump and teleport movements are one-item Special lists | **Ours** (how we play them) | — |
| Landing: right → `teleportdown`, put on the next stone keeping his heading, camera point and flight, `teleportup`; wrong → the start, D again, no damage; row −2 → the door; the start strip → nothing | rules 12–14 | `connor.log` solve run (`d4-tile-solve-connor.log`) |
| The door: its script's `ConnorAction do do`, **without the reach test** | rule 12 says "as if the player clicked it"; the reach test left out is **ours** (he is about 6,000 from the door) | `connor.log` "ltddoor4 gets ConnorAction do do: its script ran", then `tileExit 1` |
| The camera flight after a teleport: 0.5 s; following again after `teleportup` | **Stand-ins**, open questions 4 and 1 | — |
| Lines 2000/2/0/3 and /133 after `teleportup` | rule 15 | `connor.log` |
| H: exit switch on, `DeadCity::Azriel::TrapOn` true, Wait | rule 13 | `connor.log` "SOLVED" |
| Other states refused and dropped while the exit switch is off; the weapon swap refused with 0/0/37/23 | rules 18, 20 | not run |
| The Jump key is ignored in the state | **Ours** (rule 8 lists no Jump) | — |
| `tilejump1`, `tilejump2`, `tilebackjump`, `tilebackjump2` always loaded with Connor | **Ours** (how we load) | `connor.log` |
| A copy a script loads while playing that is animated gets a solid box when its type collides with Connor (as still copies did) | **Ours**, from the pair rule (`connor-physics.md`) and `objects.md` rule 6 (a loaded object is like any other) | `viewer.log` "copy tile07 … collider (…, type 0x41)" |

## Batch 2, D-5 2026-10-30: the lit stone — viewer `connor_tile.rs` (`unlit_step`), `main.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| The stone a forward jump would reach (and a back jump's target while it starts) is drawn with the per-object "unlit" switch on: smooth materials at full brightness | `tile-puzzle.md` rule 10, `object-lighting.md` rule 10 | `viewer.log` "unlit: None -> Some(\"tile04\"): 1 part(s) at full brightness (… lighting kinds [2] …)" |
| Done with per-part material copies, put on again when the drawing puts the own material back (as `dither`) | **Ours** (how we draw) | `viewer.log` "30 put on again after a new frame" |

## Batch 2, D-7 2026-10-30: the small commands — core `active.rs` (`react_command`, `required_weapon_mode_command`, `inv_click`, `light_on_fire`, `cam_rel`, `walk_slope`), `fight.rs`, `character.rs`; viewer `connor_view.rs` (`hurt`), `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Connor's `normalReactPercent` is 0.5 (not 0.9) | `fighting.md` rule 29 (corrected 2026-10-28) | unit test `batch_2_small_commands` |
| `setReactPercent <who> <p>` (−1 = default), `setSpecialReact <who> <movement> <p>` (`none` / `null` clear); short lines refused; a special react that rolls plays instead of `react` | `fighting.md` rule 23, "Do not reproduce" | unit test; `fight.log` |
| `setRequiredWeaponMode <mode> [script]`: a different mode is swapped to at once (as `setDoMode`); remembered; the wrong-swap script kept only | `ranged-weapons.md` rule 9 (its swap-back not traced) | `fight.log` "setDoMode 0 (rule 9): taken" |
| `invClick <type> N V C S`: the item's right-click message, each 0–255, the item looked up by name | `inventory.md` rule 10, "Do instead" | `items.log` "'76 0 0 1' -> '76 0 0 1'" |
| `lightOnFire <obj> <0|1>`: the group bit kept, nothing visible | `object-lighting.md` rule 37 | unit test |
| `setCamRelConner <dist> <zDir>` / `setCamRelObj <obj> <dist> <zDir>`: `$KQCamera::newLoc` and `$KQCamera::pointAt`; the camera does not move; the point at his own height | `camera.md` "setCamRelConner"; the height is the note's "probably" | unit test; `doors.log` |
| `walkSlope <name> <degrees>`: degrees converted properly, 90 = every surface; kept only (our characters' mover has no slope limit) | `character-movement.md` "Do not reproduce" | — |

## Batch 2, F-1 2026-10-30: doors moved by a start-up script — viewer `load.rs` (`Drawn::Door`, `move_object`, `place_door`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A start-up script's `move` of a door, chest or scripted object changes where it stands and its heading, as for any other model | `objects.md` Q20 d (`move` moves the object); the data: `newWorld.cs` lines 850–872 | `viewer.log` "doors: [ltddoor4] at (240510, 117979, 3044) dirZ 3.14" |

## Batch 2, F-2 2026-10-30: Death's trap armed at the landing on H — viewer `connor_tile.rs` (`tile_land`, `tile_step`)

| Fact | Source | Checked by |
| --- | --- | --- |
| On H the exit switch and `DeadCity::Azriel::TrapOn` are set at the landing, before `teleportdown`; Wait only after `teleportup` | `tile-puzzle.md` rule 13 (corrects D-4, which set both after `teleportup`) | `connor.log` update 832; `traps.log` "trap azrieltp: On Connor" at update 855 |

## Batch 2, F-3 2026-10-30: gravity off while a tile jump plays — core `connor_mover.rs` (`start_tile_jump`, `end_tile_jump`); viewer `connor_tile.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| While a tile jump movement plays: gravity off, step up and down 0; when it ends: gravity on, steps 325 | `connor-physics.md` "Jumping" rules 3 and 5 (rule 5 names the tile jumps); **a reading**, not said in `tile-puzzle.md`: Q177 asks to confirm | `connor.log` solve run: z rises during `tilejump1`, landings on top of the stones |
| No Jump state for a tile jump (no Fall after it, no crash) | `tile-puzzle.md` rule 12 (`stop` at the end); `connor-physics.md` crash list (tile jumps not in it) | `connor.log` |

## Batch 2, F-4 2026-10-30: Death as a character — data `characters.rs` (`Class::Death`); core `character.rs` (`mouth_moves`, `mouth_code`), `active.rs`; viewer `anim_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQDeath` is loaded and run as an ordinary character (drawn, his script, `speak`) | `talking.md` rule 18a ("otherwise he is an ordinary character run by `death.cs`") | `characters.log` "Death: start-up"; `doors.log` TalkComplete 1–12 |
| His lip-sync codes 0–7 start `rest`, `f`, `m`, `r`, `w`, `ih`, `ah`, `e` | `talking.md` rule 18a; the names in `Death.anm` (data) | unit test `death_plays_his_mouth_shapes_as_movements`; `characters.log` "movement 'e' starts at once" |
| Started at once for every code, even the same shape | **Stand-in**, as `characters.md` rule 27 (18a: not traced); Q179 (c) | unit test |
| Codes above 7 as for an ordinary character (rule 28) | `talking.md` rule 18a, `characters.md` rule 28; Q179 (b) | unit test |
| End of a line: shape 0, no movement started | **Stand-in** (not traced), Q179 (a) | `characters.log` |

## Batch 2, F-5 2026-10-30: camera command numbers — core `active.rs` (`cam_loc`)

| Fact | Source | Checked by |
| --- | --- | --- |
| `flyToCamLoc x y z seconds endRoll` | `camera.md` rule 4 (already built) | unit test `cam_loc_fly_and_cut_become_requests` |
| A word such as `-.6` counts as a number (−0.6) for `addCamLoc`, `flyToCamLoc`, `cutToCamLoc` | **Ours** (the notes do not say how these commands read numbers); Q180 | unit test |

## Batch 2, F-6 2026-10-30: templates from start-up placements — viewer `load.rs` (`playing_templates`, `used_no_place`), `door_view.rs`, `anim_view.rs`, `terrain_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| A `loadKQ <file> <name> x y z` while playing makes a new copy of the file | `objects.md` "Where a world's objects come from" rules 2, 6 (as built in M5 5d-3) | `traps.log` "a new copy torchrd.1a" |
| The copy's template may come from a placement made at the start, when the file has no place of its own and a script run while playing loads it | **Ours** (how we keep templates; the original reads the file again) | `viewer.log` "(F-6)"; `doors.log` "copy flamerd.1a … drawn" |

## Batch 2, D-6 2026-10-30: `changeBmp` and the Reveal letters — core `active.rs` (`change_bmp`, `bmp_swaps`, `reveal_objects_here`), `potions.rs`, `script.rs`; viewer `load.rs` (`swap_looks`), `anim_view.rs`, `connor_tile.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `changeBmp <obj> <old> <new>`: the object's own copy; textures whose original name is `old` drawn as `new`; matched against the original names (so `old old` puts it back); `normal` drops it; only the last pair kept; unknown object: a console message | `characters.md` rule 4a | unit test `change_bmp_keeps_the_last_pair_and_normal_drops_it`; `anim.log` "tile07: changeBmp: tiled.bmp -> tiledcl.bmp on 1 material(s)" |
| The `new` pictures read at load from the scripts' lines; drawn on animated objects only for now | **Ours** (how we draw) | `viewer.log` "changeBmp: 10 picture(s) … 10 read" |
| `tileshow.kq` gets `Reveal True` / `False`; `tileshow.cs` swaps the five letters | `tile-puzzle.md` rule 17, `potions.md` rule 31 | `doors.log` "tileshow.cs line 3: KQObject::changeBmp tile21" |
| A reveal object prepared for a script's `loadKQ` is called once it is in the world; not before it comes in | `potions.md` rules 31, 32 (every reveal object in the world at the start and the end) | unit test `clarity_tells_the_reveal_objects_at_start_and_end` |
| The lit stone's bright copy shows the swapped picture | `tile-puzzle.md` rule 10 with rule 17 (both at once) | not checked by eye |

## Fixes G-1 2026-10-30: the Tile state's weapon — core `weapons.rs` (`put_weapon_away`); viewer `door_view.rs` (`Request::ConnorTile`, the weapon requests)

| Fact | Source | Checked by |
| --- | --- | --- |
| Entering `Tile` with a weapon drawn: `away`, his weapon mode becomes empty hands (the game's ready weapon too, not a request) | `tile-puzzle.md` rule 7 | unit test `the_tile_state_puts_the_weapon_away_itself`; `fight.log` "put away by the state"; `connor.log` "the set switches now (None, none)" |
| In the state the weapon swap is refused with his line 0/0/37/23 | `tile-puzzle.md` rule 18 | `talk.log` update 350 |
| Bottom-bar slot clicks are refused too, with the line; a script's request without it | **Ours** (rule 18 names the swap key; rule 20 refuses other states); asked in Q181 (c) | `fight.log` "refused in the Tile state" |
| With a weapon out he says 0/0/37/23 on entering | **The user's check in the original**; asked in Q181 | `talk.log` update 250 |

## Fixes G-2 2026-10-30: the camera's roll drawn — viewer `game_camera.rs` (`camera_draw`, `roll_before`)

| Fact | Source | Checked by |
| --- | --- | --- |
| The roll (radians) turns the view about its line of sight; a flight steps it evenly to its end roll | `camera.md` "`move KQCamera`" (ry = roll), "Camera flights and cuts" rules 3–4 | `camera.log` "roll … -0.600" |
| A negative roll turns the picture anticlockwise | **The user's check in the original** (`azrieltp.cs` Cue 2); asked in Q182. G-5: the sign of G-2's turn was wrong (the user's check) and is reversed | not checked by eye |
| Drawn blended between updates; a `move KQCamera` sets it at once | **Ours** (as the camera's place is drawn) | not checked by eye |

## Fixes G-3 2026-10-30: copies move their collider — viewer `anim_view.rs` (`spawn_copy`, `anim_step`)

| Fact | Source | Checked by |
| --- | --- | --- |
| An animated object's whole-object steps move it through its mover, its collision box with it; copies loaded while playing too | `objects.md` "Loops that move the whole object" rule 1 | `connor.log`: Connor's z on `tile73` 6,636–6,751, every 31 updates |
| The box is placed from the object's current place (after a `move` or `KQMoveEvent`) | `objects.md` same rule 1 ("onto the object's current placement"), rule 3 (the stones' relative move) | as above |
| A stone whose shape has no `BOUNDS.KQ` entry has no box | `connor-physics.md` "Where every object's box comes from" | `anim.log` "[tile70] has no collider" (`tilem.kqs` not in the table) |

## Fixes G-4 2026-10-30: a loaded object starts afresh — viewer `anim_view.rs` (`at_load`, `bring_in`)

| Fact | Source | Checked by |
| --- | --- | --- |
| An animated object a script loads while playing starts its movement as at the world's start | `objects.md` "Where a world's objects come from" rule 6 | `doors.log` update 2040–2095: `keycycle` `stop`, `gone`, the key given, deleted |
| Kept as a copy of the player built at load; the unused object's own player still plays for the H key | **Ours** (how we prepare objects) | as above |

## Fixes G-5 2026-10-30: the camera's roll reversed — viewer `game_camera.rs` (`camera_draw`)

| Fact | Source | Checked by |
| --- | --- | --- |
| G-2 turned the picture the opposite way to the original's; the camera now turns by minus the roll | **The user's check in the original** (`azrieltp.cs` Cue 2); Q182 | not checked by eye |

## Fixes H-2 2026-10-30: running out of deep water — core `connor.rs` (`end_transition`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A transition ends into its `to` at its phase (`waterrun` → `run` at 1.000000) | `connor-movement.md` "Starting and stopping" rule 6; `CONNER2.anm` | `connor.log` "waterrunrun ended: run from phase 1" |
| Phase 1 of a repeating loop is its phase 0 (the frame drawn there is frame 0) | **Ours** (keeps our phase in 0..1, as `objects.md` "Loops" rule 6's frame) | unit test; `connor.log` update 111: 172 forward |

## Fixes H-1 2026-10-30: the armour skins — viewer `load.rs` (`PotionLooks::armour`), `potion_view.rs` (`look_step`)

| Fact | Source | Checked by |
| --- | --- | --- |
| His body skin is the `ConnerSkinList` entry for the items applied to him (item N = bit 2^N) | `potions.md` rules 25, 29, 30; `Connor.kq` `ConnerSkinList`; asked in Q184 | `items.log` "skin list bits 7: body skin drawn as con7.bmp" |
| Equipment is applied while carried | `inventory.md` rules 8, 14 | `items.log` "apply … put on" |
| A missing combination takes the entry with most of the wanted bits | `potions.md` "Do not reproduce" 3 (for the face list); **ours** for the skin list (Q184) | not checked against the original |
| Strength's entry wins over any armour, and over `con65.bmp` on a model | `potions.md` rule 25 | `items.log` update 200 |
| An entry whose picture this world lacks: his own skin | **Ours** (Q184 c) | `items.log` update 151 |

## Fixes I-1 2026-10-31: tile jumps — core `connor_mover.rs` (`start_tile_jump`, `end_tile_jump`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A tile jump switches gravity and keep-on-ground off, leaves step up / down at 325, and switches both back on when it ends | `tile-puzzle.md` rule 11 (Q177) | `connor.log`: the north jump onto `tile56` |

## Fixes I-3, I-4 2026-10-31: the teleport flight, the hold switch — viewer `connor_tile.rs`, `game_camera.rs`, `door_view.rs`; core `camera.rs` (`Camera::hold`)

| Fact | Source | Checked by |
| --- | --- | --- |
| The flight after a teleport is a `flyToCamLoc` with no time (distance ÷ 5,000 a second), starting when `teleportdown` ends | `tile-puzzle.md` rule 13 "The camera flight" (Q178); `camera.md` "Camera flights and cuts" rule 4 | `camera.log` N = 54 for 17,883 |
| When the flight ends the camera follows Connor again, the hold switch on again (not after H); the keys work from `teleportup`'s end | `tile-puzzle.md` rule 13 | `camera.log` "Connor is followed again" |
| The hold switch: on entering, off for each flight, on at its end; while on the follow keeps its height and the tilt isn't aimed at Connor | `tile-puzzle.md` rule 7a (Q178) | `camera.log` updates 291–305: z 4,465, tilt −0.200 |

## Fixes I-5 2026-10-31: the stones' weapon refusals and the Jump key — viewer `door_view.rs`, `connor_tile.rs` (`tile_step`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Every weapon change in the Tile state is refused with 0/0/37/23, scripts' too; Alt with empty hands does nothing | `tile-puzzle.md` rule 18 (Q181) | `fight.log` update 330 (script); Alt not tested |
| The Jump key starts a forward jump; ignored while a jump, landing or teleport is under way | `tile-puzzle.md` rule 8 and its "Do not reproduce" (Q181) | `connor.log` update 290 |

## Fixes I-6 2026-10-31: Death's mouth — core `character.rs` (`mouth_code`), `active.rs` (line end)

| Fact | Source | Checked by |
| --- | --- | --- |
| Death's codes 0–7 start their movements at once, an equal code again; 16 starts `rest`; 8–15 and above 16 do nothing | `talking.md` rule 18a (Q179) and its "Do not reproduce" | unit test `death_plays_his_mouth_shapes_as_movements`; `characters.log` |
| At the end of a line nothing happens to his pose | `talking.md` rule 18a (Q179) | `characters.log` "line ended: nothing" |

## Fixes I-7 2026-10-31: camera command numbers — core `active.rs` (`c_atof`, `cam_loc`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Every number of `addCamLoc`, `cutToCamLoc`, `flyToCamLoc` is read as C's `atof`; no number = 0, no message | `camera.md` rule 4b (Q180) | unit test `camera_numbers_are_read_as_c_atof` |
| `addCamLoc` exactly 3 or 4 numbers, the list word whole; cut's roll only with 4, fly's end roll only with 5; seconds below 0 = 0 | `camera.md` rule 4b | `camera.log`: Death's flight "roll 0.000 -> -0.600" |

## Fixes I-8 2026-10-31: the stones' movements follow each other in one update — viewer `connor_tile.rs` (`tile_follow_on`), `connor_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| The landing starts `teleportdown`, its end the put and `teleportup`, with nothing between | `tile-puzzle.md` rules 12–14; **ours**: done in the same update, the user's check (no standing frame) | `connor.log` updates 432, 454 |

## Fixes I-9 2026-10-31: script movements drawn from their start — viewer `connor_view.rs` (`script_anim`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A movement started at once starts at its phase 0 (its `loop=` start) | `anim-scripts.md` rules 4–5; **ours**: the drawing takes that phase in the same update | `connor.log` "drawn from phase 0.0000" at 2819, 2864 |

## Batch 2 D-8a 2026-10-31: `terAnimSwitch` — core `ground_anim.rs` (`switch`), `active.rs`; viewer `ground_anim_view.rs` (`GroundAsks`), `door_view.rs`, `load.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Swaps the animator's cycle and second cycle (`materialIndexII…`) only when both have materials | `emitters.md` "`terAnimSwitch`" | unit tests `ter_anim_switch_*`; `ground_anim.log` "switched to cycle [161]" |
| A step past the new cycle's end goes to 0; one word does not start or stop it; the change clock is kept | `emitters.md` "`terAnimSwitch`"; **ours**: "current step" read as our next entry (Q187) | `ground_anim.log`: Blood1 161 at 2.2075 s, 0.3055 s after its last change |
| With a second word: step 0, repeat = that word read as a number (not the original's misread) | `emitters.md` "`terAnimSwitch`", "Do not reproduce" | unit test |
| The second cycle's textures are loaded for the animated squares | **ours** (needed to draw it) | viewer.log: no MISSING |

## Batch 2 D-8b 2026-10-31: `matchHeights` — core `terrain_drain.rs`, `active.rs`; viewer `ground_drain_view.rs`, `ground_anim_view.rs` (`meshes_in`), `door_view.rs`, `main.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Five whole numbers; corner column/row = coordinate ÷ square size, the max one not included | `terrain.md` "`matchHeights`" | unit test `the_rectangle_leaves_out_the_max_corner…`; `ground_drain.log` columns (89, 93) for 184000–190500 |
| The surface corners move to the floor corners in `frames` even steps, one per update; after the last they equal the floor | `terrain.md` "`matchHeights`" | unit tests; `ground_drain.log`: all five middles surface = floor at the step |
| Up to 32 at once | `terrain.md` "`matchHeights`"; **ours**: a 33rd is refused with a log line (not in the note) | — |
| First step in the update after the call | **ours** (the note gives only one step every 1/15 s) | `ground_drain.log` updates 31 → 32 |
| In-between steps rounded to whole pixel values; `frames` below 1 does nothing; no floor grid does nothing | **ours** (the grids are whole values; not in the data) | unit tests |
| The live terrain is changed (water depth, camera, shadows read it) and the drawn surface and animated squares are put back on it | **ours** (drawing) | `ground_drain.log` "39704 vertices moved" |

## Batch 2 D-9a 2026-10-31: the trap `Lookup` call — core `trap.rs` (`lookup_call`), `active.rs` (`post_lookup`, `TRAP_LOOKUP`); viewer `load.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| A trap entering the world (start, after its `Init`; or `loadKQ`) posts itself a lookup notice through the event queue; on arrival `lookupCall=1` with a script runs `<trap> Lookup` | `traps.md` rule 15a | unit test `a_trap_with_lookup_call_gets_lookup_after_init_through_the_queue`; `traps.log` `trptrp12`–`15` Lookup at update 0, Cue 1 at update 2 |
| Delay 0 (handled with the first update's events); a trap being loaded gets it when it is handed over (held as any event) | **ours**: the note says "after the current load or script step (the exact moment: not sure)" | — |
| Not done: the same notice after a saved game is restored | `traps.md` rule 15a (no saving yet) | — |

## Batch 2 D-9b 2026-10-31: spirit traps — core `trap.rs` (`spirit_damage`, `spirit_here`), `active.rs`, `fight.rs` (`spirit_drain`); viewer `load.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQSpiriTrap` is a trap with a box and a `damage` share | `traps.md` "Spirit traps" | `viewer.log`: `spirtp1`–`4` "spirit damage Some(0.2)" |
| Cue 1 = a spirit is passing, Cue 0 = it has left | `traps.md` "Spirit traps" | `traps.log` |
| **Cue 2 also = it has left** | **the user's choice 2026-10-31** (the note: Cue 2 is ignored, a possible original bug; the data never sends Cue 0) | unit test `a_spirit_trap_drains_connor_only_while_a_spirit_passes`; `traps.log` "Cue 2 counted as left" |
| Connor entering first while a spirit passes, or Cue 1 while he is inside: health − damage × maximum, not below 1 % of maximum, then a 1-point hit that makes him flinch | `traps.md` "Spirit traps" | `fight.log`: 80 → 64, then the hit 64 → 63, `react` |
| Shares rounded to whole health; the hit is the ordinary one (rule 16, so it costs at least 1 and can kill); nothing while dying, on the rope or in god mode | **ours, stand-in** (Q188) | unit test |

## Batch 2 D-10a 2026-10-31: `ConnorCollision` from Connor's moves — core `connor_mover.rs` (`Scene::touched`), `active.rs` (`connor_touch`, `direction_number`); viewer `connor_view.rs`, `door_view.rs`, `load.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| An object with a script, `conCollisionAction` ≠ 0 and its own mask covering 4 is told about Connor touching it, from his walking, running or falling into it | `connor-collision.md` rules 2, 7 | unit test `connor_collision_runs_once_per_direction…`; Daventry `bricks` (mask commented out) never told |
| Every object contact found as it is tested counts, in every pass | `connor-collision.md` rule 7, `connor-physics.md` step 7 (Q88: told at once as tested) | `doors.log` waterfall |
| `$3` = direction number: 1 any upward part, 6 any downward part, 2–5 by x and y | `connor-collision.md` rule 9; **ours**: the x = 0 bump to 4 / 6 not copied ("Do not reproduce") | unit test `direction_numbers_follow_the_surface_direction` |
| Per object six clocks starting at 0; a call only when more than 0.2 s since the last touch in that direction; every touch sets it | `connor-collision.md` rule 10 | unit test; waterfall: one call at update 35, none while pressing on to update 70 |
| The script runs right after his move | **ours** (the note: mid-move; as the scripted `Push`) | — |
| Not built: an object moving into Connor (Gnome's rocks, M6) | `connor-collision.md` rules 5, 9 | — |

## Batch 2 D-11a 2026-10-31: `KQAttackEvent` — core `active.rs` (`attack_event`, `EVENT_TYPES`), `fight.rs` (`object_takes` shared)

| Fact | Source | Checked by |
| --- | --- | --- |
| Text `<attacker> <kind> <damage> <flag>`; on Connor the shared hurt rule; kind 0 normal, 1 projectile, 2 fire, 3 ice; the flag unused | `connor-collision.md` rules 16–17 (= `fighting.md` rules 16–18) | unit test `an_attack_event_hurts_connor_by_the_shared_rule`; `fight.log`: `rivrtrap.03` 5000 → health 80 → −1298, `die` |
| On an ordinary object: its hit by its stats, the kind remembered for its `Die` | `sound.md` rule 41, `doors.md` rules 19–20 | **not tested** (no data sends one to an object); its ouch sound not built |
| On a character: logged only; other kind numbers count as Normal | **ours**, Q190 | — |

## Fix J-1 2026-10-31: a character's `shapeName` is not used — viewer `load.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| A character's model comes from its `animFile` movement list; there is no shape key for characters | `characters.md` rule 6 (traced) | `viewer.log` "animated … [Sylph]"; `characters.log` Barren `Witchdr` shown with `Docstop.kqs` (its file names the Sylph's `sylfstop.kqs`) |
| So `move` reaches the Sylph as any character | `objects.md` / the scripts' `move` (as built) | `traps.log`: update 2 `move Sylph 0` → (0, 0, 0); `--command "move Sylph 2 70662 105677 1584"` → back at the well |

## T-1 2026-10-31: the tower skeleton is a character — data `characters.rs` (`Class::TowerSkeleton`, keys, `tower_alarm`); core `tower_skeleton.rs`, `character.rs`, `active.rs` (Cue 201); viewer `anim_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQTowerSkeleton` is a character with its own states | `fighting.md` 7a | `characters.log` "[Skeltowr] KQTowerSkeleton … 11 movements" |
| Keys `realAlarm`, `stunPause` (default 3), `reactPause` (default 2), `moveTime` (starts 0), `timeToRun` (enter + 5 s, kept only) | `fighting.md` 7a "Keys" | unit test `defaults_and_the_data` |
| On entering the world its alarm radius is 0; Cue 201 sets it to `realAlarm` | `fighting.md` 7a | `fight.log` "alarm radius 0 -> 5000" |
| It enters at game time 0 (it is in the start's monster list) | **ours** | — |
| Who sends Cue 201: not in the data or the note | Q195 | — |

## T-1b and T-2 2026-10-31: `PreBirth`; the tower skeleton standing and moving — core `character.rs` (`pre_birth`, the flinch end), `character_ai.rs` (`rise`, `tower_rules`, the probes), `fight.rs` (`flinch_gap`), `tower_skeleton.rs`; viewer `anim_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| A list with `PreBirth`: in Birth it loops `prebirth` visibly, and rises when it notices Connor as the others | `characters.md` rule 16 | `characters.log` "'prebirth' loops until it rises", "rises (rule 30)" at update 46 |
| No target: hit or on its way home → runs after Connor; not active → joins; within its alarm radius → alarmed at once, no sight test, no delay | `fighting.md` 7a "Standing" | — (not run: hit before alarmed) |
| With a target: 45 updates unseen → gives up (no team exception); no leash; walks after him only after `moveTime`, chase radius > 0 and ground within 1,000 below the point 1,000 ahead | 7a "Standing" | `characters.log` "after Connor, walking" at update 46 |
| Moving: no ground ahead → stops; within attack distance: attacks unless Connor is on a rope, stops either way | 7a "Moving" | update 139 "no ground … ahead: stops", then attacks (updates 140, 175, 210) |
| `reactback` (its stagger): `moveTime` = now + `stunPause` each update; at its end the next flinch gap is `reactPause` if moving, else 2 s; the probe 400 behind: nothing → it falls | 7a, `fighting.md` rule 23 | **not run** (needs Connor to hit it) |
| The probes: a line straight down from 100 above its feet to 1,000 below, against the drawn ground and the buildings | **ours** (the note gives the distances, not the test) | — |
| Its fall is the ordinary death until T-3 | **ours, stand-in** | — |

## T-2b 2026-10-31: play-once movements move characters — core `door.rs` (`once_travel`), `named_anim.rs`, `character.rs` (`movement_step`)

| Fact | Source | Checked by |
| --- | --- | --- |
| A character moves only by the steps of the movement it plays, as Connor (no exception for play-once movements) | `character-movement.md` rule 1; `connor-collision.md` rule 14 (`reactback` moves Connor 903 back through his mover) | `characters.log`: `reactback` takes the skeleton from y 125,849 to 125,445 and it stays there |
| Each update adds the steps of every frame it enters; backwards they are taken off | `objects.md` rule 6a | — |
| Its drawing no longer adds those steps (they moved its place) | **ours** (drawing), as for its walk | the user's look: not yet |

## T-3 2026-10-31: the tower skeleton's fall — core `tower_skeleton.rs` (stages, offsets, `own_axes`, `look_angles`), `fight.rs` (`tower_fall_start`), `active.rs` (`tower_fall_next`, `tower_cue50`, `run_tower_lines`), `character_ai.rs` (the ground behind it)

| Fact | Source | Checked by |
| --- | --- | --- |
| No ground behind it during / after `reactback`: it falls: gravity off, Connor third person, Wait, controls off; the scream (2000/16/12/22 seq 1); `teeter` | `fighting.md` 7a "Dying" step 1, rule 23 | `fight.log` update 444 |
| `teeter` ends: camera 2,000 behind and 5,000 below, looking at the point 0.9 of the way; Cue 50 2.18 s later; `die` | 7a step 2; **ours**: "0.9 of the way" read as from the camera to it, "behind" its +Y | `fight.log` update 473; `doors.log` `move KQCamera 0 186077 128438 4593 1.1903 0 3.0550` |
| `die` ends: camera 3,000 to its side and 2,000 up from the ground point 2,000 behind, looking at it; moved 600 back, 5,000 above that ground; `die2` | 7a step 3 (5,000: "not sure"); **ours**: "side" = its +X | update 526 |
| `die2` ends: on the ground 1,400 behind; drops what it carries; `fallend` | 7a step 4 | update 551: `INVITEM_ChainMailShirt` thrown |
| `fallend` ends: Cue 50 3 s later; first Cue 50: the scream's second part; second: camera back to Connor, controls on, the dropped item gets `chainmal.cs`, the ordinary finish (rule 32) | 7a steps 2, 5; **ours**: "back behind Connor" = `KQCamera::follow Connor` | update 603: experience 4000, `chainmal.cs` |
| The ground heights: a line straight down against the drawn ground and the buildings | **ours** | — |

## T-4a 2026-10-31: the tower skeleton's lines — core `tower_skeleton.rs`, `active.rs` (`tower_hears`, `tower_greeting_step`)

| Fact | Source | Checked by |
| --- | --- | --- |
| It hears Connor's state changes (`towSklTp.cs` asks); the first time he starts climbing a rope: controls off, line 2000/16/0/18; when it ends, controls on and the music `mtower.aud` | `fighting.md` 7a | unit test `the_tower_skeleton_greets_connor_once_on_a_rope_and_hears_a_ranged_weapon` |
| Connor readies a ranged weapon: line 2000/16/0/102 | `fighting.md` 7a; **ours**: "readies" = a swap (32) ending with the ranged mode | `fight.log` update 57 "Connor readies a ranged weapon: its line"; `talk.log` 16 0 102 |
| The music started with `KQMusic::push` | **ours** (the trap's `Off` pops it) | unit test |
| Not built: the greeting's camera on it | Q195 c | — |

## K 2026-10-31: `KQMonster::add` while playing — viewer `load.rs` (`prepare_script_adds`, `place_monster_line`), `anim_view.rs` (`copy_prototype`, `spawn_character_copy`, prototype recipes), `door_view.rs`, `copies_view.rs`; core `active.rs` (`monster_add`, `set_add_prototypes`, `ADD_PROTOTYPE`, `command_event`)

| Fact | Source | Checked by |
| --- | --- | --- |
| The command takes the 20 fields (words joined with tabs, read as one monster-list line) or `name type`; the line's values on top of its type's file, as a list line | `characters.md` rules 3–4 | `characters.log` "rat1's prototype prepared" ×15 |
| A name in use: the original stops with a fatal error | `objects.md` (as `loadKQ`) | `doors.log` update 30 "refused" |
| One prototype per line, made at the start under `<name>@add`, copied by every add | **ours** (how to make it; every use in the data is a respawn) | `doors.log` updates 22 and 50: two copies |
| A copy is drawn from the prototype's recipe and counts for the draw distance; not copied: its mouth pictures and shadow | **ours** (drawing) | `viewer.log` "copy rat1 of rat1@add: drawn … its draw-distance test" |
| `--command` ends as a script run (a name it deleted is free afterwards) | **ours** (test harness) | — |

## U-1 2026-11-01: the tower skeleton's fall order — core `tower_skeleton.rs`, `active.rs` (`tower_fall_next`); viewer `anim_view.rs` (`birth_shown` for every character)

| Fact | Source | Checked by |
| --- | --- | --- |
| The order is `teeter`, `die`, `fallend`, `die2` | `fighting.md` 7a "Dying", correction Q196 | `fight.log` updates 473, 526, 533, 558 |
| After `die`: camera G + (3,000 side, 0, 2,000 up) looking at G; moved 600 behind, at the camera's height + 3,000 | 7a step 3 | update 526: (185956, 127043, 7898), 5,000 above G |
| After `fallend`: on the ground 1,400 behind; drops its items; `die2` there | 7a step 4 | update 533: on the ground (z 2898), chain mail thrown |
| After `die2`: hidden (not drawn, not solid); Cue 50 3 s later | 7a step 5 | update 558; `characters.log` 559 "not drawn" |
| The ground height measured once when the fall starts (the point 2,000 behind = 600 + 1,400) | **ours** | — |

## U-2 2026-11-01: Cue 201 and the tower skeleton's greeting camera — core `active.rs` (`camera_flight_ended`, Connor's pass-on, `tower_cue201`, `tower_greeting_step`, `tower_hears`, `tower_cue50`, `run_lines`), `tower_skeleton.rs`; viewer `game_camera.rs` (`flights_ended`), `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| The end of any camera flight sends Cue 201 to Connor; he passes it on to his state-change list unless in a Tile state | `fighting.md` 7a "Cue 201" (Q195 a) | `doors.log` update 37 "passed on … Skeltowr" |
| Sent one update after the flight's last update | **ours** (the viewer runs the flights) | update 36 → 37 |
| Cue 201 acts only after the greeting: follow Connor, controls on, first person allowed, alarm `realAlarm`; 2000/16/0/98 if Connor is invisible | 7a "Cue 201" (Q195 a, d) | unit test `the_greeting_flies_the_camera_back_and_cue_201_from_connor_wakes_the_tower_skeleton` |
| Greeting: third person, controls off, `Stop`, the camera's place remembered, cut to it + (0, −500, 1,200) in its own axes looking at it; at the line's end a flight back with no time, aimed at it; the music 0.2 s later | 7a (Q195 c) | unit tests |
| The cuts as `move KQCamera` with a fixed view; the flight as `addCamLoc … 1` + `flyToCamLoc <its position>` | **ours** (how) | — |
| No camera place known: controls on at the line's end | **ours** | unit test |
| Second Cue 50: cut to Connor + (0, −5,000, 1,000) in his own axes, looking at him, then follow | 7a (Q195 b); which side: Q197 | — |

## U-3 2026-11-01: far characters and animated objects frozen — viewer `draw_distance.rs` (`FrozenNow`, `TICK_BIT`, `DistanceTest::animated`), `anim_view.rs`, `door_view.rs`; core `active.rs` (`frozen`), `character_ai.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Bit 0x1 is the per-update tick; animated objects and characters get it added to their near set, not their far set; `groupsActive3DFar` 1089 keeps it when far | `object-lighting.md` rule 37 (corrected Q185) | — |
| Far = drawn, frozen in its last pose, not solid, no updates; off = off the list and the reset home | `character-movement.md` rule 21 (Q185), the user's check in the original | Daventry `gobcon1`: frozen at update 54, moving again at 152, home at 302 (`viewer.log`, `characters.log`) |
| Objects without a distance test are never frozen; doors and chests unchanged | **ours** | — |

## U-4 2026-11-01: the spirit drain in whole numbers — core `fight.rs` (`spirit_numbers`, `spirit_drain`, `hit_connor_with`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Loss = whole part of (level × damage × 10); below 1 % of the maximum: whole part of (level × 0.1), slightly short (0 for levels 1–10, 1 for 11–20) | `traps.md` "Spirit traps" (Q188 a) | unit test `the_spirit_drain_in_whole_numbers` |
| Then the trap's ordinary hit: Normal, 1, must react | Q188 b | unit test `a_spirit_trap_drains_connor_only_while_a_spirit_passes` |
| The health change in any state; the hit part ignored while dying, nothing on the rope | Q188 c | — |
| The first-hit tip only for a character's hit or a projectile | Q188 c (`fighting.md` rule 29) | the same unit test |

## U-5 2026-11-01: `KQAttackEvent` on a character — core `active.rs` (`attack_event`)

| Fact | Source | Checked by |
| --- | --- | --- |
| The ordinary hit of rules 16–29 (roll, protection, flinch, alarm) | `fighting.md` rule 29a (Q190) | Dead City `fight.log` update 30: Skelking 800 → 799, flinches |
| Damage 0 with a script: `ProjImmune` instead | rule 29a | — |
| Kinds other than 1–3 count as Normal | rule 32a "Do not reproduce" | unit test `an_attack_event_on_a_character_is_the_ordinary_hit` |

## U-6 2026-11-01: ground flags follow the current material — data `terrain.rs` (`material_flags`, `set_square_material`); viewer `ground_anim_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| A square's flags are looked up from the material it has now; an animator's change changes them at once | `water.md` "The flags follow the square's current material" (Q192) | Dead City `ground_anim.log`: "ground flags 0x40 -> 0x4" on all 120 blood squares |
| Footsteps follow too (inferred in the note) | `water.md` (Q192 b) | `connor.log`: water steps to update 32, rock steps from 35 |

## U-7 2026-11-01: liquid damage (Connor) — core `water.rs` (`timer_name`, `damage`, `kind_number`, `deep`, `LiquidSample`), `connor_update.rs`, `connor_mover.rs` (`liquid`), `active.rs` (`connor_liquid`), `fight.rs` (`hit_connor_full`); data `envinfo.rs`; viewer `door_view.rs`, `connor_view.rs` (`hurt`), `load.rs` (`damage_timers`)

| Fact | Source | Checked by |
| --- | --- | --- |
| After every move; poison → `PoisonTime`, fastpoison/lava → `FastPoisonTime`, swamp → `SwampTime`; 0 or unset = 1,000 | `water.md` "Liquid damage" rules 1–2 (Q191) | unit test `the_liquid_damage_numbers` |
| x = 0.66 × Connor's level ÷ value; ≥ 1: whole part every update; else 1 when k (second's fraction × 15, rounded) is a multiple of n = whole part of 1/x, at most 8 | rule 3 | the unit test (levels 1, 2, 3, 4, 8, 16) |
| Only deep (liquid, surface ≥ feet + 250), or `ontop` with the surface at most 100 below the feet | rule 6 | `fight.log`: blood 288 deep, level 8: −1 every update while he moves, nothing once he stands |
| Kind Fire / Ice / Normal by the flags; the ordinary hit; no ouch, `poison<n>.aud` at most every 2.5 s | rule 7 | `doors.log` updates 12, 50 |
| Which of `poison1`–`3`: a roll | **ours** | — |
| The timers read from `world.cs`'s text at load (its `Init` is not run as a script by the viewer) | **ours** (as `MaxMonsters`) | `items.log` "terrain damage timers from world.cs" |

## U-8a 2026-11-01: animated objects drawn from their movements — viewer `load.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| A `KQAnimObject` never loads its `shapeName`; it is drawn from its current movement's model | `objects.md` "Animated objects at load" rule 3 (Q189 a) | Dead City `viewer.log`: `scale`, `scalelt`, `scalert` loaded (were skipped) |
| Box: current movement's model in `BOUNDS.KQ`, then its own name | rule 4 (Q189 b) | `viewer.log` "doors: [scalelt] box item289=scalelt (−2750, −590, −2175)..(−1725, 570, −2005)" |
| Only the class `KQAnimObject`; a failed movement lookup keeps `shapeName` | **ours** | — |

## U-8b 2026-11-01: the `Scales` state — viewer `connor_scales.rs`, `connor_view.rs`, `connor_tile.rs` (`tile_holds`), `door_view.rs`, `load.rs`; core `active.rs` (`Request::ConnorScales`)

| Fact | Source | Checked by |
| --- | --- | --- |
| Entering: a drawn weapon put away; `conscale.anm` (`onscale1`, `onscale2`, `offscale1`, `offscale2`) | `tile-puzzle.md` rule 23 | `connor.log` update 23 |
| Keys: turn on the spot; forward takes one step | rule 24; **ours**: one per press (Q198 b) | — |
| Floor (x < 219,000): nearest of −1.57 (away: start spot, walk to (209,500, 49,065)) / 2.207 / 0.933 (a dish: start spot, `onscale1`, `onscale2`, turned 180°, +200) | rules 25, 26a | updates 40–111, 330–444 |
| Dish: y > 50,000 → −0.933, < → −2.207, = → nothing; `offscale1`, `offscale2` | rule 26 | updates 170–194 |
| Gravity and keep-on-ground off for `onscale1` / `offscale1`, back on for the second | rule 26a (Q189) | `connor.log` z: 3,966 at the end of `onscale1`, then settles on the dish |
| Other states and weapon swaps refused as in Tile | "The scales" intro | — |
| Camera spots not built; the state left before the walk; the turn off a dish at once; state number Tile's; the tie at 1.57 → 2.207 | **ours** (Q198) | — |

## U-9a 2026-11-01: `KQMonster::dropItem` — core `active.rs` (`drop_item_command`), `script.rs` (no longer a stub); viewer `load.rs` (`item_models`)

| Fact | Source | Checked by |
| --- | --- | --- |
| One item taken from `who`'s stack; a new one (count 1) put still at the point; z −1 / −2 = floor / surface; "is not carrying" otherwise | `ranged-weapons.md` 8f (Q186 e) | unit test `drop_item_takes_one_from_the_stack_and_lays_a_new_one`; Dead City `doors.log` update 30 |
| An emptied stack leaves the inventory | **ours** | the unit test |

## U-9b 2026-11-01: the thrown rock's flight — core `fight.rs` (`RockFlight`, `launch_rock`, `rock_step`, `rock_touch`), `active.rs` (`rocks_after_flight`); data `characters.rs` (`parabola`, `peakHeightOffset`, `upVelocity`, `script`); viewer `load.rs` (the `Rock` item's file loaded)

| Fact | Source | Checked by |
| --- | --- | --- |
| Start: the `rock` part at `throw`'s `do` frame: 1,849 up, 223 ahead | `ranged-weapons.md` 8c (data `C_rktoss.kqs`) | unit test: first position z 1,349 (500 below the hand) |
| M = min(`maxDistance`, horizontal distance to the aim); T grows by `upVelocity` to M/3, then `velocity`; height start − H + h, H = 500 + 1,849, the two quarter-ellipses | 8c | unit test: the top at M/3 is the hand height |
| Ends on the floor grid; a side touch (normal z < 0.707) drops it straight down; a top touch hits an object (Connor's ranged damage) and ends it; past M: drops | 8c | unit test "reaches the floor grid" |
| The drop's pull 8,000 | 8c ("not sure") | — |
| The flying rock is a type-1 visitor for the traps; deleted at its end without a move, it never leaves (no `Off`) | 8d, `traps.md` 8a | unit test: `triptrp2` On, still inside |
| `Die`: `rockHit.cs`'s `dropItem Connor INVITEM_Rock` at the landing point, run as a line | 8c "Die", 8f; **ours**: run directly (no object for its `getLoc`) | — |
| Kind 4 treated as Normal | `fighting.md` rule 32a | — |

## U-9c 2026-11-01: the rock click and the `ThrowRock` state — viewer `connor_rock.rs`, `connor_view.rs`, `connor_tile.rs`, `door_view.rs`, `load.rs`; core `active.rs` (`rock_click`, `box_distance`, `Request::ConnorThrowRock`, `rock_aim`)

| Fact | Source | Checked by |
| --- | --- | --- |
| The aim: the click line's point; an accepting object nearer than `maxDistance` (nearest box point): its position, and `ConnorAction` with the rock at once; no walk, no remark; else a throw at the point; the rock leaves the pointer | `ranged-weapons.md` 8a | Dead City `fight.log` update 40 |
| On entry: weapon mode remembered, `away`, empty hands, `throw`; `do` (0.7778) launches; after: the old mode back, `out`, Stop, Wait | 8b | `connor.log` updates 41–78 (empty hands), 81–139 (the crossbow out) |
| Its state number 512 in `StateChange` | `characters.md` rule 16 | — |
| The turn before the throw; a script's ThrowRock with no click: 2,000 ahead; the old mode asked back through `setDoMode` | **ours** | `fight.log` update 138 |

## U-10 2026-11-20: Dead City's river-dipping boxes (`RiverControl`) — core `active.rs`, `talk.rs`, `connor_inventory.rs`, `weapons.rs`; viewer `load.rs`, `door_view.rs`, `connor_view.rs`, `world_build.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Four `RiverControl` files loaded per `newWorld.cs` lines 312–315 (`rivercon`, `rivercn2`–`4`); exact position, no `keepOnGround`; `acceptsInventory=INVITEM_BrokenShield,INVITEM_IronLock`, `stretchDist` default `1,300` | `objects.md` "River controls" rules 1, 6e, 6g (data: `newWorld.cs` loads `rivercon` too) | Dead City `viewer.log` (`4 RiverControl boxes`) |
| Click: empty -> `2000/13/14/0/1`; other item -> nothing (stays in hand); distance >= 1,300 -> `0/0/0/14/1`; accepted within reach -> `2000/8/7/16/1` and hand emptied | `objects.md` rules 2, 6g; `0.MSG` | unit test; Dead City `doors.log`, `talk.log`, `items.log` |
| End of `2000/8/7/16/1`: turn toward `(cx, cy ± 1,300)` (`-` when `locY <= cy`), `handsOff 1`, camera cut to Connor + `(200, 3,700, 1,400)` tilt `-0.2` roll `0` heading `3.0`, following resumes | `objects.md` rules 3, 6a, 6b | unit test; Dead City `doors.log` update 27, `camera.log` update 27 |
| +0.5 s: `Special` `AnimList=river.anm,away,AnimListOn,dipShield|dipLock,AnimListOff,out` (`away`/`out` from active weapon list when weapon out); `corrode` cue; +3.5 s: `2000/8/7/16/2`, item swap to `RustedBrokenShield`/`RustedLock`, +2,000 XP once per box (`rustedOne`, kept across world changes); `handsOff 0` on `out` end | `objects.md` rules 3, 6c, 6d, 6f; `river.anm` | unit test; Dead City `connor.log` updates 35–98, `items.log` update 80, `sound.log` (`CORRODE.AUD`, `STATS.AUD`, `LEVEL.AUD`) |

## U-11 2026-11-20: Dead City's weeper draining bolt (`KQWeepBolt`) — data `characters.rs`; core `connor.rs`, `character.rs`, `sound.rs`, `portal.rs`, `active.rs`, `fight.rs`; viewer `load.rs`, `connor_view.rs`, `door_view.rs`, `sound_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `PRweeper.kq` (`KQWeepBolt`, collider `weepbolt.kqs` never drawn, `duration=3`, `suckFile=lifesuck.kq`, `suckEnd=lifesuk2.kq`, `suckSound=lifesuck.wav`, `fireAnim=suck`, `hurtAnim=magicReact`, `doneAnim=suckstop`, `shootOffset=1400`, `hitOffset=1000`); `weeper`/`weeperup` hold `INVITEM_WeeperEyes`; `weeper.kq` has `shootOffsets0=INVITEM_All 0 0 1500` | `ranged-weapons.md` section 9 (Q194 a, b) and `MASK/data/DeadCity/kq/{PRweeper.kq,lifesuck.kq,lifesuk2.kq,weeper.kq,weeper.anm}` | unit tests (`shoot_offsets_and_weep_bolt_keys_are_read`, `active::tests::weep_*`); Dead City `characters.log` |
| Bolt flies straight (speed 8,000, range 10,000, mask 0xD, invisible mesh); on hitting a character other than shooter, vanishes if target is already in `Special`; otherwise spawns `max(1, floor(len/250) − 2)` chain pieces at 250-unit steps (`lifesuck.kq`, last `lifesuk2.kq`), starts looping `suckSound`, holds shooter in `fireAnim` (`suck`) and target in `hurtAnim` (`magicReact`) | `ranged-weapons.md` section 9 (Q194 b, d, e); `MASK/data/GAME/Kq/Conner.anm` movement 27 | unit test `weep_bolt_drains_connor_bypasses_protection_and_ends_cleanly`; Dead City `fight.log` updates 29–33, `viewer.log` (`weep_suck_1..6`), `sound.log` (`LIFESUCK.AUD`, `MAGICRCT.AUD`) |
| Drain counter > 0.25 s resets to 0 and subtracts 1 plain health (no roll, protection, flinch or ouch); at duration end or when shooter/target dies or vanishes: stops loop sound, deletes chain pieces, living shooter goes to `Wait` then `Special(doneAnim)` (`suckstop`) with `attack_at = now + attack_delay`, living target goes to `Wait`; lethal drain kills via `ConnorDies`/`character_dies` without returning target to `Wait` | `ranged-weapons.md` section 9 (Q194) | unit tests `weep_bolt_drains_connor_bypasses_protection_and_ends_cleanly`, `weep_drain_ends_when_shooter_dies_and_kills_connor_below_zero`; Dead City `fight.log` updates 36–77 (`80 -> 69`), `connor.log` updates 77–78, `characters.log` updates 77–89 |

## B1-1 2026-11-20: Dead City polish (`Q197` tower skeleton, `Q198` scales) — core `character.rs`, `active.rs`; viewer `connor_view.rs`, `connor_scales.rs`, `game_camera.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `StateChange` numbers: `TILE2` 16,384, `TILE3` 65,536, `SCALES` 131,072, `FALL` 262,144 | `characters.md` rule 16, `tile-puzzle.md` "The scales" (Q198) | unit test in `active.rs` (`state_number::*`) |
| Scales step away: camera `follow none` + cut to `(216,360, 52,250, 7,260)` `(-0.716, 0, 2.644)`, `handsOff 1`, walk to `(209,500, 49,034)` | `tile-puzzle.md` rule 25 (Q198) | unit test `scales_constants_match_q198`; Dead City `connor.log` / `camera.log` update 5 / 102 |
| Scales north/south dish cuts `(216,115, 53,490 / 44,790, 5,560)`, on-dish cut `(216,500, 49,140, 4,500)` `(-0.1, 0, -1.57)`, off-dish cut `(214,420, 49,340, 4,190)` `(-0.2, 0, -1.64)` + `follow Connor`; clearing remembered forward after `onscale2`/`offscale2` | `tile-puzzle.md` rules 24–26 (Q198) | Dead City `connor.log` / `camera.log` updates 5, 76, 77, 101 (held forward immediately starts `offscale1` at update 77) |
| Tower skeleton second `Cue 50`: `KQMusic::pop mtower.aud` queued 0.2 s later (`MUSIC_TARGET` event survives `death_ended`), and `follow Connor` immediately after cut snaps frame 0 onto the followed spot (`<= 4,999`) | `fighting.md` 7a (Q197) | unit test `tower_cue50_second_cuts_to_connor_follows_and_pops_music_after_death`; Dead City `camera.log` update 21 (`follow,snap_after_cut`) |

## B1-2 2026-11-20: Script `;` separator, `inc`/`setcat`/`alias`, `getConnorFlag`, `mouseAction`, loading & screen hints — data `world_script.rs`; core `script.rs`, `active.rs`; viewer `load.rs`, `door_view.rs`; cli `script_coverage.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `;` separates statements outside `'...'`/`"..."` (never a comment; fixes `Snowexit/worldinv.cs`'s 11 `;KQMonster::inventory` mushroom lines); `#` starts comments | `script-commands.md` rule 1; `MASK/Snowexit/kq/worldinv.cs` | unit test `semicolon_separates_statements_in_startup_scripts`; Snowexit `items.log` (11 world mushrooms at `z = 26,910`) |
| `inc <var>` (C `atoi` prefix / non-numeric -> `1`); `setcat` (no separator, 2 words `""`, 512-byte cap); `alias` (case-sensitive word-0, max 20 expansions, `;` in body, single quotes block expansion; `newWorld.cs` definitions passed to `Scripts::aliases`) | `script-commands.md` rules 7–9 | unit tests `inc_setcat_and_alias_follow_batch1_rules`, `world_script::follows_scripts_in_order` |
| `getConnorFlag` case-insensitive across the 7 `CONNOR_FLAGS` -> `"1"`/`"0"` (error + `"0"` on unknown); `mouseAction` hides Top panel for `y > 60` without moving OS pointer; `KQGame::selectScreen` `0..=8`; 14 loading/cache/world-start hints return `"0"` | `script-commands.md` rules 10, 12, 13; `interface.md` rule 3a | unit test `get_connor_flag_mouse_action_select_screen_and_loading_hints`; Daventry `doors.log`, `viewer.log`, `gui.log` |

## B1-3 2026-11-20: `allowShadows 0|1` world load and live toggle — core `active.rs`, `journey.rs`; viewer `load.rs`, `terrain_view.rs`, `shadow_view.rs`, `anim_view.rs`, `door_view.rs`, `world_build.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `world.cs` sets initial `allowShadows 0\|1` per world (`0` in CastleD, Gnome, Temple1–4; `1` else); explicit `world.cs` setting takes precedence over `Journey` carry | `characters.md` rule 36; `MASK/*/kq/world.cs` | unit test `taken_and_given_back_unchanged`; `CastleD` `viewer.log` (`allowShadows 0 (world.cs)`) |
| Live `allowShadows 0\|1` toggles plain-object, Connor, and character shadows cleanly off and back on | `characters.md` rules 36–37 | unit test `allow_shadows_toggles_state_and_queues_request`; `DeadCity` `doors.log`, `shadows.log`, `characters.log` |

## B1-4 2026-11-20: `loadAnim` dynamic `KQAnimOnce` spawning — core `active.rs`; viewer `load.rs`; cli `script_coverage.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `loadAnim` spawns live `KQAnimOnce` (`once = true`, `deleteWhenDone = 0` initially), sets `$ObjectName` (`@` -> `NoName<n>`), snaps `z == -1/-2` synchronously, supports `.kqs`/`.dts` (`animPos`, `timeScale`, `cyclic`, `interruptable`) and `.anm` (`loopName`) forms | `script-commands.md` rule 11 & Edge cases; `anim-scripts.md` rule 11 | unit test `load_anim_spawns_kq_anim_once_for_shape_and_anm_forms`; Swamp `doors.log` & `viewer.log` (`swmpgas.kqs`) |
| Script-targeted `.kqs`/`.dts`/`.anm` files preloaded from world and `GAME/Kq` scripts as animated `Template`s; `loadAnim` removed from `LOADING_HINTS` | `script-commands.md` rule 11; `MASK/swamp/kq/fienfir*.cs`, `MASK/GAME/Kq/swmpfrbl.cs` | Swamp `viewer.log` (`loadAnim templates: 2 preloaded: ["endfrbl.kqs", "swmpgas.kqs"]`) & `anim.log` |

## B1-5 2026-11-20: `KQCamera::shake` — core `camera.rs`, `active.rs`; viewer `door_view.rs`, `game_camera.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQCamera::shake <dur> <xMax> <xPeriod> <zMax> <zPeriod>`: 6 words, smooth tent envelope `1 - |t / half - 1|` (`dur <= 0` cancels), `x` wave `sin(period * (elapsed + period / 2) * pi) * xMax`, `z` wave `sin(period * elapsed * pi) * zMax` | `camera.md` "Shake and cutscene cameras" | unit tests `camera_shake_envelope_wave_and_reset`, `camera_shake_parses_six_words_and_checks_usage` |
| Follow camera sets `tilt = -0.200 + pitch` and adds yaw through `aim_view`, guards `MOVE_RESETS_TILT` while shaking, and restores `-0.200` tilt / `0` yaw when finished; fixed camera adds deltas without undo; first-person helper `first_person_angles` adds `zWave` / `xWave` | `camera.md` "Shake and cutscene cameras" | unit tests; Daventry `doors.log` & `camera.log` (`update 10`..`41`: 30 active `shake` ticks, `shake_end` clean restore) |

## B1-6 2026-11-20: `KQHazer` proximity haze zones — core `hazer.rs`, `active.rs`; viewer `load.rs`, `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQHazer` `.kq` keys (`radius`, `innerRadius`, `hazeCenterFactor`, `colliderMask` default `0x221f`, `locZ == -1` surface snap) and `[-radius, +radius]` cube / `move` / `deleteObject` support | `object-lighting.md` rules 23a, 23d; `MASK/DeadCity/kq/litehaze.kq` (`15000`, `3000`, `2.00`, `mask=4`) | unit tests in `hazer.rs` and `active.rs`; `DeadCity` `viewer.log` (`1 KQHazer`, `30000 x 30000 x 30000` marker) |
| 3D distance `<= radius` enters and saves terrain `hazeDistance` (default `20,000`) as `origHazeEdge`/`hazeEdge`; 2D XY distance ramps linearly over `innerRadius.min(radius)..radius` between `hazeEdge * factor` and `hazeEdge`, setting `h` and `1.5 * h` (`visibleDistance`); leaving cube restores `origHazeEdge` | `object-lighting.md` rules 23b–23c | unit tests; `DeadCity` `viewer.log` & `doors.log` (`9000 -> 18000 -> 13500 -> 9000`) |

## B1-7 2026-11-20: Help popups (`doPopup`, `notStartingHelp`, `stopHelp`, F1 help book, built-in tips) — core `help.rs`, `active.rs`, `script.rs`, `connor_inventory.rs`, `fight.rs`, `journey.rs`; data `world_script.rs`; cli `script_coverage.rs`; viewer `talk_view.rs`, `load.rs`, `world_build.rs`, `door_view.rs`, `gui_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `doPopup <n>` queues `1.5 s` in game time; defers `+1.0 s` on `handsOff`, non-Wait/Move Connor state, Connor speaking, or `< 3.0 s` cooldown, and `+3.0 s` if another popup is open (clean FIFO queue without original index-0 removal bug) | `interface.md` rules 53–54 & Do not reproduce | `help::tests::cue_delay_deferrals_cooldown_and_fifo_queue`; `Daventry` `doors.log` (`update 5 -> 28`) |
| Starting help `< 170` gated on `notStartingHelp` (`Options.cs` `1` default; checkbox `0 <-> 1`, never `2`), `>= 170` always shown without checkbox; `^<char>` placeholders resolve via `movekeys.kq` `KeyName0..12` `#<n>` -> `GAME/ENGLISH/500.MSG`; `38 × 7` line wrap; OK `#148` at `142..167` above `IDCTL_POPUP_CHECKBOX*` `#147` (or `145..183` without checkbox); `intrface.aud` on open and gameplay paused while open | `interface.md` rules 55, 57; `world-change.md` rule 26 (`Q151`); `MASK/Options.cs`, `MASK/movekeys.kq`, `MASK/GAME/ENGLISH/500.MSG` | `help::tests`, `active::talk_tests::do_popup_not_starting_help_and_stop_help`; `Daventry` `viewer.log`, `sound.log`, `gui.log` (`[Insert]`, `intrface.aud`, `notStartingHelp 0 -> 1`) |
| `stopHelp` sets `0x7f` muting the 7 built-in one-off tips (`#128` rope `4.5 s`, `#130` half health, `#131` health item `3.0 s`, `#132` quest item `0.0 s`, `#137` first kill `2.0 s`, `#138` level 2 `1.5 s`, `#140` empty-handed hit `1.5 s`); `conStart.cs` runs `stopHelp` before starting inventory; `Journey` preserves `tip_bits` via bitwise OR | `interface.md` rule 58; `fighting.md` rules 29, 33; `potions.md` rule 41 | `DeadCity` `viewer.log` & `doors.log` (`conStart.cs line 2: stopHelp -> tip_bits=0x7f`, suppressing `#131`, `#128`, `#132`); `active::tests::its_swings_hurt_connor_until_he_dies` (`#140`) |
| F1 Help Book browses the 24 `500.MSG` entries (`127–146, 149–151, 153`) with `<<`, `Close` (`#155`), and `>>`, Left/Right arrows wrapping `0 <-> 23`, and Enter/F1/Escape closing (`Escape` cleared before `fly_input`) | `interface.md` rule 56; `MASK/GAME/ENGLISH/500.MSG` | `help::tests::not_starting_help_and_stop_help_and_book_wrapping`; `Daventry` `viewer.log` & `gui.log` (`1/24 #127 -> 24/24 #153 [Tab] -> 1/24 #127 -> 2/24 #128 -> close`) |

## B1-F1 2026-11-21: Help popup button order, Portal highlight/click sound, and instant close — core `panels.rs`; viewer `gui_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| F1 Help Book buttons ordered left-to-right `Close` (`#155`), `<<`, `>>` (`IDCTL_POPUP_BTN1..3`); highlight starts on `0` (`Close`) and matches `Portal` (`buyhibtn.pba`, Left/Right arrows move highlight `0..=2` preserved across page turns, Enter presses/activates highlighted button, mouse presses clicked button without moving highlight) | User check in the original (2026-11-20); `world-change.md` rule 26 (`Q151`); `gui-verify.log` (`IDCTL_POPUP_BTN1..3`) | `Daventry` `viewer.log` & `gui.log` (`highlight 0 "Close" -> 1 "<<" -> 2 ">>" -> 1 -> 0` across `1/24 #127 <-> 24/24 #153`, **not yet re-checked by eye**) |
| Plain one-shot `button1.aud` plays when left-clicking anywhere inside the open popup window (`0..360 x 0..200`), when moving button highlight with Left/Right arrows, and when pressing Enter (on `Portal`, `HelpNormal`, and `HelpBook`); modal popups block background bar/map clicks | User check in the original (2026-11-20) & user ear confirmation (2026-11-21); `world-change.md` (`Q151`); `MASK/GAME/SOUND/button1.aud` | `Daventry` `sound.log` (`start button1.aud` on `help:right` and `help:enter`) + user ear check |
| `Panel::Popup` shows and hides immediately (`away` snapped `0.0` / `1.0`, no `0.2 s` slide timer) so text, buttons, checkbox, and window vanish on the same frame | User check in the original (2026-11-20) | unit test `panels::tests::popup_shows_and_hides_immediately`; `Daventry` `gui.log` (`popup hidden immediately`, zero empty redraws; **not yet re-checked by eye**) |

## B1-F2 2026-11-21: `KQHazer` shared GPU haze buffer, quiet ramp logs, and wireframe-with-rings `H`-key marker — core `active.rs`; viewer `object_light.rs`, `object_light.wesl`, `ground_haze.rs`, `ground_haze.wesl`, `building_light.rs`, `building_light.wesl`, `main.rs`, `world_build.rs`, `door_view.rs`, `load.rs`, `terrain_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| Live `haze_start`, `haze_end`, `ground_start`, `visible_distance`, `haze_colour`, and `F8` toggle live in one persistent 32-byte `ShaderBuffer` (`HazeBuffer`) shared across `ObjectMaterial`, `GroundMaterial`, and `BuildingMaterial` (updated in O(1) on `F8`, `setHaze`, `KQHazer`, and world change) | `object-lighting.md` rules 19, 23, 27–29, 33a; engine optimization matching `SceneLights` | unit test `object_light::tests::haze_buffer_packs_distances_colour_and_flag`; `DeadCity` `litehaze` + `Daventry -> DeadCity` `goworld` runs (`RESULT: PASS`, `average_frame_ms=7.54`); **checked live by the user 2026-11-21** |
| `Request::SetHaze { quiet: true }` keeps continuous `KQHazer` ramp updates out of terminal summary output while preserving zone entry, exit, `deleteObject` restore, and script `setHaze` summary logs | `object-lighting.md` rules 23b–23d, 33a; user check (2026-11-20) | unit test `hazer_zone_ramps_haze_on_move_and_restores_on_exit_and_delete`; `DeadCity` stdout vs `viewer.log` / `traps.log`; confirmed smooth in play (2026-11-21) |
| `MarkerKind::HazeZone` draws `KQHazer` bounds wireframe-only (`[-radius, +radius]` cube + 3 orthogonal `radius` sphere rings in teal-mint; `innerRadius.min(radius)` ring + 4 vertical cylinder guides in yellow-lime; viewer helper) | `object-lighting.md` rules 23a–23c; user visual feedback (2026-11-20) | unit test `wireframe_hazer_rings_and_guides_match_line_list_contract`; `DeadCity` `viewer.log` (`1 haze zone`, `radius 15000, innerRadius 3000`); **checked by eye 2026-11-21** |

## B3-1 2026-11-21: Swamp script commands (`KQConner::pay`, `KQConner::setRelDamage`, `KQMonster::preSpecial`, `setFirstPerson`, `KQConner::addImmunity`) — core `connor_inventory.rs`, `active.rs`, `journey.rs`; viewer `load.rs`, `connor_view.rs`, `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQConner::pay <itemType> <quantity>` (exact 3 words, `INVITEM_` fallback, partial decrement vs exact-count `unapply` + hand/ready-weapon clear without world drop, `$Console::Return` `"1"`/`"0"`) | `inventory.md` rules 27–30 (Batch 3) | unit test `pay_handles_partial_exact_unapply_and_failures`; Swamp `items.log` & `doors.log` (`KQConner::pay SilverCoins 2`: `x100 -> x98 -> 1`, not yet played by hand) |
| `KQConner::setRelDamage <fraction>` (`CurRelDamage = trunc(-level * 10 * frac)`, `CurRelHealth = -CurRelDamage`); `KQMonster::preSpecial <name>` (interruptable current movement + `"Stop"` wanted movement on Connor or named character); `Swamp/boiloil.cs:4` `beep` left unimplemented as confirmed typo | `script-commands.md` rules 5, 12a–12b (Batch 3) | unit test `batch3_set_first_person_set_rel_damage_pre_special_and_add_immunity`; Swamp `doors.log` (`level 15 -> CurRelDamage=-75, CurRelHealth=75`; `Connor preSpecial`) |
| `setFirstPerson` global `SetFirstPerson::WasThirdPerson` toggle & F2 lock mirror of `setThirdPerson` (tracking state while first-person view awaits its own step) | `camera.md` rule 8 (Batch 3) | unit test `batch3_set_first_person_set_rel_damage_pre_special_and_add_immunity`; Swamp `doors.log` (`WasThirdPerson 1 -> 0`); live check 2026-11-22: `Rtswdoor` hand-slot click plays in third person (first-person camera not yet built; user to verify original) |
| `Connor.kq` `immunities=` key (default `0`), `KQConner::addImmunity <mask>`, `Journey::immunities` carry-over across worlds, and bitwise immunity masking in `Active::connor_liquid` (`256` / `0x100` Swamp vs `0x40` Poison) | `water.md` "Liquid damage" rule 5 (Batch 3) | unit tests `taken_and_given_back_unchanged` and `batch3_set_first_person_set_rel_damage_pre_special_and_add_immunity`; Swamp `doors.log` (`connor immunities now 0x100`, not yet tested by wading in Swamp) |

## B3-2 2026-11-22 (+ `B3-F1` 2026-11-22): `KQSwampTile` & multi-pair/static `ChangeBmp` — core `swamp_tile.rs`, `active.rs`, `fight.rs`; viewer `load.rs`, `anim_view.rs`, `terrain_view.rs`, `door_view.rs`, `main.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `KQSwampTile` keys (`heightNum` `0` default, `damFactor` `0.48` default), truthy `"true"`/`"false"` handling for `Swamp::Tile::PowerOn`/`SwitchOn`, safe (`fiensafe.cs` via startup `KQObject::setScript` on placed `collision` entries) vs unsafe (`script == None`) classification, and click/`ThrowRock` handling | `tile-puzzle.md` rules 36–37 (Batch 3); `MASK/Swamp/kq/{worldvar.cs,newworld.cs}` (`B3-F1`) | `swamp_tile::tests`; `B3-F1` Swamp `items.log` & `doors.log`; live check 2026-11-22: safe tiles and rock throws onto unsafe tiles confirmed working by eye |
| Clarity `Reveal` highlights only safe `KQSwampTile`s via 2 texture pairs (`gastrap.bmp`->`I_gastrp.bmp`, `wtower5.bmp`->`I_wtowr7.bmp` for `1..=3`; `trapring.bmp`->`I_trprng.bmp`, `trunk.bmp`->`I_trunk.bmp` for `12..=13`) using per-entity cloned `StaticTexture` materials (path prefix stripped by `texture_basename`, `B3-F2`); `Reveal False` restores originals | `tile-puzzle.md` rule 38 (Batch 3) | unit tests `swamp_tile_reveal_gas_puffs_and_touch_damage_follow_rules_37_to_40`, `static_texture_basename_strips_vol_prefix`; `B3-F2` Swamp `viewer.log`: `static changeBmp: 24 part(s) swapped, 0 restored`; **live check 2026-11-22: Clarity highlights safe tiles as expected** |
| 0-damage projectile hit & Connor touch (`> 2.0 s` resettable cooldown) on armed unsafe tiles spawn auto-deleted `swmpgas.kqs` at `+750`/`+1,000`/`+1,250`, plus direct `trunc(-level * 10 * damFactor)` health loss and 1 Fire hit (`HitKind::Fire`); `F3` god mode logs `damage skipped (god mode on)` | `tile-puzzle.md` rules 39–40 (Batch 3) | unit tests; `B3-F1` Swamp `doors.log` & `fight.log`; live check 2026-11-22: tile flame damage on Connor confirmed working by eye (`F3` off) |

## B3-3 2026-11-22: Connor `Tile2` state & built-in `Tile2` rock throw — core `tile2.rs`, `active.rs`; viewer `connor_tile.rs`, `connor_tile2.rs`, `connor_view.rs`, `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `Tile2` (`16,384`) 10 × 7 grid (`tile00..tile56` in cols `2..=7`, row-3 door spots `(0,3)/(1,3)` & `(8,3)/(9,3)`), four forbidden empty-corner edge blocks, `where_is` directions (`East 200`/`West 100`/`South 300`/`North 400`/`Spot`), plain `jump` off West/East (`100`/`200`), lit stone `unlit_step`, `tileExit 0` lock overridden by `Die`, `0/0/37/23` weapon-swap refusal | `tile-puzzle.md` rules 29–35 (Batch 3) | `tile2::tests` (3 unit tests), `set_state_connor_tile2_emits_request`; Swamp `viewer.log`; live check 2026-11-22: `Tile2` puzzle itself confirmed working almost flawlessly (**open bug 2e**: 6 side flaming-arrow towers not working) |
| Row-3 door landings: cells `30/31` send `Cue 1` to `fiendoor` (`WestDoor`); cells `38/39` send `ConnorAction "do"` to `Rtswdoor` (`EastDoor`); built-in `Tile2` rock throw (`handsOff 1` -> `away` -> `Prrock.kq` `throw` -> `handsOff 0`, staying in `Tile2`) | `tile-puzzle.md` rules 34–35, `ranged-weapons.md` 8e (Batch 3); `MASK/Swamp/kq/{fiendoor.cs,swdoor.cs}` | Swamp `viewer.log` & `doors.log`; live check 2026-11-22: `Rtswdoor` (`hmhand`), cutscene start, and `Tile2` rock throws confirmed working |

## B3-4 2026-11-22: Mandragore (`KQMandragorTree` & `KQMandragorRoot`) — data `characters.rs`; core `mandragore.rs`, `character.rs`, `character_ai.rs`, `fight.rs`, `active.rs`; viewer `load.rs`, `anim_view.rs`, `terrain_view.rs`, `item_view.rs`, `connor_view.rs`, `door_view.rs`

| Fact | Source | Checked by |
| --- | --- | --- |
| `Class::MandragorTree` / `Class::MandragorRoot`, `0x4042` (`ProjImmune`) default `colliderType`, tree `rootDownCount`/`maxRoots` (ignoring `tooClose`/`repelDist`), root `offsetX/Y/Z`/`rebirthDelay`/`tooClose`/`repelDist`, and `.kq` `alarmRadius` (`15,000`) overriding `Monstats.mtl` (`4,000`/`3,000`) | `fighting.md` 7b (Batch 3); `MASK/Swamp/kq/{mandrg.kq,mandrtl.kq,mandrtr.kq}` | `kq8-data::characters::tests::mandragore_keys_and_defaults`; Swamp `characters.log` & `fight.log` (all 11 trees & 22 roots loaded, `alarmRadius 15000`, `0x4042`); live check 2026-11-22 (checked by eye) |
| 0-damage hits send tree (or root's live parent tree without hitting tree) to `Special laugh`; tree alarms immediately (`0.00 s` delay), alerts `mand<N>` roots without sight check (`now > 0`), refreshes team `last_seen`, awards `20,000` XP, and spawns solid `mt_dead.kqs` corpse (`BoxCollider` registered) across all `HitKind`s | `fighting.md` 7b (Batch 3); `MASK/GAME/patch/BOUNDS.KQ` | `mandragore::tests`, `mandragores_link_snap_repel_laugh_sink_rebirth_and_tree_death`; Swamp Run A/B; live check 2026-11-22: projectile laugh, tree kill, and dead trunk transformation confirmed by eye |
| Roots start visible (`unborn=false`, `"reactback"` added to `CONNOR_MOVEMENTS`), gravity off, link `'l'`/`'r'`->`'t'`, snap to `treePos + treeRot * offset`, repel Connor `< tooClose` by `repelDist` keeping Z + facing tree + `Special reactback` (`c_rctbk.kqs`/`conrowbk.aud`), sink (`rootDownCount`, tree `noTurn=true`) -> `rebirthDelay` -> `Birth` (`0x4042`/`0x201f`, `noTurn=false`), or die permanently (`0` XP) on tree death | `fighting.md` 7b (Batch 3); `MASK/GAME/Kq/Conner.anm` | unit tests; Swamp Run A/B; live check 2026-11-22: shoulder roots, killing roots, and `reactback` knockback animation confirmed by eye |

## B3-F3 2026-11-22: `KQCueEvent 500` object projectile launchers (`Q200`)

| Behaviour implemented | Source | Verification |
|---|---|---|
| Parse `mustReact != 0.0` in `ProjectileKeys::from_section` (`crates/kq8-data/src/characters.rs`) and pass `b.must_react` on `Touched::Connor` | `ranged-weapons.md` §9 (rules 34–35); `MASK/GAME/Swamp/PRfire2.kq` | `cargo test --workspace` (`796` passed); Swamp headless log |
| Load `ObjectLauncher` for placed non-unused, non-`Class::of` objects with `projectileFile` (`crates/kq8-viewer/src/load.rs`, `door_view.rs`); `KQCueEvent 500` launches projectile from `base + rot(shootOffset)` toward unrotated `start + relAimPoint` (`n2` in `1..=99_999` else `objProjDamage` = `20`, `bolt_kind` `0`/`2`/`3`/other, no launcher sound) and falls through to script `Cue 500` | `ranged-weapons.md` §9 (rules 34–37); `MASK/GAME/Swamp/totem.cs` | unit tests `cue_500_launches_object_projectile_falls_through_to_script_and_runs_die_script` & `a_door_gets_end_loop_after_open_and_get_end_loop_relays`; Swamp headless run (`totem.01`–`06` firing `PRfire2.kq` on 8 s loop in `doors.log`/`fight.log`); **live check 2026-11-22: totems fire across puzzle, hit and damage Connor as expected** |
| `WorldState::queue_bolt_die` + `Active::drain_bolt_die_calls` call projectile script `Die Normal` (non-`Weep`) on contact/maxDistance/floor-end with `place_of` / `KQObject::getLoc` returning `dead_bolt_pos` before deletion | `ranged-weapons.md` §9 (rule 36); `MASK/GAME/Swamp/swmpfrbl.cs` | unit test `cue_500_launches_object_projectile_falls_through_to_script_and_runs_die_script`; Swamp headless run (`swmpfrbl.cs` spawning auto-deleted `endfrbl.kqs` copies `NoName63`–`NoName67` at bolt end locations); **live check 2026-11-22: `endfrbl.kqs` bursts on impact (wall-impact sound fixed in `B3-F4`)** |

## B1-F1c 2026-11-22: Popup sound group 1 and normal open `button1.aud` (`Q199` b) — core `sound.rs`; viewer `gui_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| Added `GROUP_MENU = 1` + `Start::menu_one_shot` in `crates/kq8-core/src/sound.rs` (`setSfxVol`, unaffected by speech ducking); switched `queue_help_sound` (`button1.aud` for normal/starting-help open, `intrface.aud` only for F1 Help Book) and `queue_popup_click` in `crates/kq8-viewer/src/gui_view.rs` to `menu_one_shot` | `interface.md` rules 55–56 (`Q199` b) | unit test `sound::tests::menu_one_shot_plays_in_group_1_follows_sfx_vol_and_ignores_speech_ducking`; **live check 2026-11-22: popup open/click sounds and group 1 confirmed by ear** |

## B3-F4 2026-11-22: Projectile impact & `KQObject` `ouchSound` rules (`Q201`) — data `characters.rs`; core `active.rs`, `fight.rs`; viewer `load.rs`, `door_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| Parse `ouchSound=` on `ProjectileKeys` (`crates/kq8-data/src/characters.rs`) and load placed `.kq` `ouchSound=` overrides into `Objects`/`WorldState::object_ouch_sounds` (`crates/kq8-viewer/src/load.rs`, `door_view.rs`) | `ranged-weapons.md` rules 30 & 36, `sound.md` rule 41 (`Q201`) | unit test `shoot_offsets_and_weep_bolt_keys_are_read`; Swamp `viewer.log` (`132 placed object(s) override ouchSound (others default to sword.aud per Q201)`); **live check 2026-11-22 (confirmed by ear)** |
| `WorldState::play_object_ouch` + unified `bolt_hits_non_character` & `connor_hits_object`: buildings/doors/objects play `ouchSound` (default `sword.aud`) as group-`0` placed one-shot (`5,000 / 25,000`, `1.0 s` cooldown) at target origin unless `weaponType == 4` or scripted `0`-damage/`0x4000` (`ProjImmune`); bare terrain/range expiry silent | `ranged-weapons.md` rule 30, `sound.md` rule 41 (`Q201`) | unit tests `projectile_hit_on_building_plays_default_sword_with_cooldown_and_floor_stays_silent`, `projectile_hit_on_object_plays_ouch_sound_and_skips_scripted_immune`; Swamp `fight.log` & `sound.log` (`fiengate1` starts `GAME/SOUND/SWORD.AUD` at `(65536, 98304, 16320)`); **live check 2026-11-22: totem fireball impact on `fiengate1` confirmed by ear** |
| `RockFlight.ouch_sound` (`prrock.kq` = `rock.aud`): played as group-`0` placed one-shot (`5,000 / 25,000`) at landing position on removal (`rock_step`), skipping non-character target `ouchSound` | `ranged-weapons.md` rule 30, `sound.md` rule 41 (`Q201`) | unit test `thrown_rock_plays_own_ouch_sound_on_landing_and_skips_target_ouch`; **live check 2026-11-22 (confirmed by ear)** |

## B4-1 2026-11-22: `conClickItem` and `cls` — core `active.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `conClickItem <objectName>` (exact 2 words -> `"0"` on usage/missing; picks up world `KQInventoryItem` bypassing reach/turn/`handsOff`/queue and synchronously dispatches empty-hand `ConnorAction do do` via `calls_out`, returning `"1"`); `cls` safe no-op returning `"0"` | `script-commands.md` rules 12c–12d (Batch 4); `MASK/Gnome/kq/{armorgnm,brnzplat,chnmlst,myarmor,platmail,weapngnm,emptyloc}.cs` | unit test `cls_and_con_click_item_follow_script_commands_rules_12c_and_12d`; headless Gnome `doors.log`/`items.log` (`conClickItem NoName55` picking up `INVITEM_BronzePlate` and running `brnzplat.cs` synchronously; `cls` -> `0`; **not yet checked live in play**) |

## B4-2 2026-11-22: `useAlternateLightmap` — core `active.rs`; viewer `load.rs`, `wall_lights.rs`, `terrain_view.rs`, `world_build.rs`, `door_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `useAlternateLightmap <interiorName> <0|1>` (exact 3 words -> `"0"`, queues `Request::UseAlternateLightmap` + stores `WorldState::alt_lightmaps` for `KQInterior`, prints `useAlternateLightmap: Could not find object <name>` on missing, silently ignores non-interiors) | `script-commands.md` rule 12e, `object-lighting.md` rule 12 (Batch 4); `MASK/Gnome/kq/{fusebox.cs,gnome13.kq,gnome14.kq}` | unit test `use_alternate_lightmap_switches_interior_ignores_non_interior_and_returns_0`; Gnome `doors.log` (`fusebox.cs`, `gnome13`/`gnome14`, `missing`, `anvil`) |
| `KQInterior` `althwLightmapName` & `usingAltLighting` parsing, alternate `.dil` layout compatibility validation, `WallLights::set_alternate` immediate GPU `Image` swap, and `KeptWorld`/`Arrival` persistence | `object-lighting.md` rule 12 (Batch 4); `MASK/{Gnome,temple4}/resource/*.dil` & `MASK/temple4/kq/temple.kq` | unit test `wall_lights::tests::set_alternate_swaps_atlas_and_writes_image`; Gnome `viewer.log` (`gnome13l.dil` `512x175` & `gnome14l.dil` `512x215` compatible, `normal`/`alternate` swaps; **not yet checked live by eye**) |

## B4-3 2026-11-22: `KQBoulder` rolling and animation rules — core `active.rs`; viewer `load.rs`, `anim_view.rs`, `door_view.rs`, `connor_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `BoulderState` (`prev_pos`, `forced_active`, `frozen`), `Active::start()` `forced_active` + 1-update `pushston.kqs` advance + `+0.5 s` `KQCueEvent 50`, `Active::deliver()` `Cue 50` freeze while delegating other cues, `Active::boulder_pushed` (`Mode::Push`, `Δ ≥ 1e-6` -> 1-update unfreeze/advance/re-freeze + `+0.1 s` `KQMoveEvent` `yaw + π`), `move_command` `prev_pos` sync | `pushing.md` rules 12a–12d (Batch 4); `MASK/Gnome/kq/{boulder,bouldtrp}.kq/.cs` | unit tests `kq_boulder_startup_freezes_on_cue_50_and_delegates_other_cues`, `kq_boulder_pushed_rolls_when_connor_pushes_and_rotates_on_move_event`; Gnome `doors.log` (`update 0` -> `update 8 Cue 50` -> push steps `0.1333`..`93335.4`) |
| `load.rs` routes `KQBoulder` into animated objects; `anim_step` keeps `boulder` collider synced to `base`; `door_step` preserves `BoulderState::frozen` across `FrozenNow`; `push_apply` calls `doors.active.boulder_pushed` | `pushing.md` rules 12a–12d (Batch 4) | Gnome `900`-frame `viewer.log`, `connor.log`, `anim.log` (`[boulder]` collider synced at `90980..93189`; **visual roll axis in play not yet checked by eye**) |

## B4-F1 2026-11-22: Pushed objects visit traps (`Bouldtrp`/`Bouldtp2`) & `move boulder` collider sync — core `active.rs`; viewer `connor_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `Active::object_moved` + `connor_view::push_apply` send each pushed object (`BoxCollider` `BallStack::from_box` `Visitor` with `collision_type`, e.g. `boulder` `257`) through `traps_check`; `move_object` updates live `clickables` and emits `Request::Move` when moving a `KQBoulder` | `traps.md` rules 4–6; `MASK/Gnome/kq/{boulder,bouldtrp,bouldtp2}.{kq,cs}` | unit tests `pushed_object_triggers_mask_1_trap`, `move_boulder_emits_request_for_collider` (headless Gnome check pending after `B4-F3`) |

## B4-F2 2026-11-22: Deduplicate `changeBmp` materials across shared `.kqs` movements — viewer `anim_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| Deduplicated `Playing::texture_names` by `(shape_index, material_id)` via `dedup_texture_names` and guarded `p.bmp_saved` against saving the same `Handle<ObjectMaterial>` twice so `changeBmp ... normal` restores shared `.kqs` animations (e.g. `dragworm.anm`'s `dw_stop.KQS` across `run`/`stop`/`walk`) | `characters.md` rule 4a; `MASK/Gnome/kq/{dragworm.anm,fusebox.cs}` | unit test `anim_view::tests::dedup_texture_names_keeps_unique_shape_material_pairs` (headless Gnome check pending after `B4-F3`) |

## B4-F3 2026-11-22: `KQObject::collisionType` character support (`Dragon` melee targeting) — core `active.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `WorldState::collision_type` updates `Character::collision_type` when `name` resolves via `character_index` so `KQObject::collisionType Dragon +64` toggles `0x4002 <-> 0x4042` for `SWING_PICK` melee targeting | `doors.md` rule 12 / `script-commands.md`; `MASK/Gnome/kq/{dragworm.kq,fusebox.cs}` | unit test `collision_type_updates_character_for_swing_pick` (`0x4002 -> 0x4042`, picked by `SWING_PICK`) |

## B4-F1b 2026-11-22: `place_of` resolves live traps for `checkDistance` & quiet boulder turns — core `active.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `WorldState::place_of` resolves live `self.traps` (`Placement::at(t.settings.position)`) so `bouldtrp.cs`/`bouldtp2.cs` `checkDistance $1 boulder` succeeds; `move_command` skips `Request::Move` when a `KQBoulder` only rotates (`new.position == old`) | `anim-scripts.md` rule 31; `MASK/Gnome/kq/{bouldtrp,bouldtp2}.cs` | unit tests `pushed_object_triggers_mask_1_trap` & `move_boulder_emits_request_for_collider`; live user play check 2026-11-22 (`B4-F1`–`B4-F3` confirmed working) |

## B4-F4 2026-11-22: Numeric `ID #` lookup in `ItemTypes::find` & world items in `place_of` — data `items.rs`; core `active.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `ItemTypes::find` resolves numeric `ID #` strings (`1060`, `1150`, `2050`, ...) passed by `KQMonster::checkInventory` (`$ConInv::*`); `WorldState::place_of` resolves world `KQInventoryItem`s (`Place::World`) for `getLoc` and `checkDistance` | `inventory.md` rule 1 & `anim-scripts.md` rule 31; `MASK/GAME/Kq/Invinfo.itl`, `MASK/Gnome/kq/{weapngnm,armorgnm,emptyloc,warhammr,myweapon,myarmor}.cs` | unit tests `find_accepts_numeric_id_as_well_as_names`, `check_inventory_accepts_numeric_id_for_connor_and_world`, `get_loc_and_check_distance_resolve_world_items` |

## B4-F5 2026-11-22: `KQMonster::dropItem` equipment/rotation/`NewInvItem` & swap angles — core `active.rs`, `connor_inventory.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `KQMonster::dropItem <who> <type> x y z [rx ry rz]` parses optional rotation angles, sets `$NewInvItem`, resolves `z == -1 / -2` via `resolve_world_position`, un-applies and moves single non-stacking equipment (`WarHammer`, armor) as the same object preserving `script`, and clears hand/readied weapon/`take_ranged_item(NO_RANGED)` | `items.md` rules 9 & 11, `inventory.md` rule 17, `ranged-weapons.md` 8f; `MASK/Gnome/kq/{armorgnm,brnzplat,platmail,chnmlst,emptyloc,weapngnm}.cs`, `MASK/barren/kq/tw_weird.cs` | unit tests `drop_item_moves_equipped_armor_with_rotation_unapplies_and_restores_placeholder`, `drop_item_keeps_script_on_short_weapon_and_restores_fists`, `drop_item_takes_one_from_the_stack_and_lays_a_new_one` |
| Dropping Connor's only `ShortWeapon`, `LongWeapon`, or outer `Armor` (with no `FullArmor`/`GodArmor` carried) restores the matching slot placeholder (`INVITEM_Fists`, `INVITEM_NoRanged`, `INVITEM_NoArmor`) | **Stand-in** matching `inventory.md` rule 23 (so re-entering `Wpngnome`/`Armrgnome` keeps `1280`/`1310`/`2120` in `getConInventory`) | unit tests above (`INVITEM_NoArmor`, `INVITEM_Fists`) |
| `connor_inventory::receive` preserves `Place::World` item `angles` when equipment swaps an old piece into the picked-up item's world spot | `inventory.md` rule 8 + observed `MASK/Gnome/kq/{worldvar,worldinv}.cs` (`$Gnome::Armory::rot = -1.1 0 2.4`) | unit test `swapping_with_world_equipment_preserves_item_angles` |

## B4-F6 2026-11-22: Preload available equipment models for console `give` & shop swaps — viewer `load.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `Placer::item_models` (`extra_equipment_types`, `Placer::load_item_shape`) preloads all non-placeholder equipment `ItemType`s (`connor_inventory::places`) whose `.kqs` exists in the world or `GAME/Common` (`MASK/GAME/Common/Leathr.kqs` for `INVITEM_LeatherArmor`) before reading `body` transforms, matching the existing `LongWeapon` preload pattern | `items.md` rule 1 & rule 5 (`GAME/Kq/Invinfo.itl`, `GAME/Common/*.kqs`), `inventory.md` rules 3 & 8; observed `give leatherarmor` swap onto Gnome Armor Shop shelf | unit test `extra_equipment_types_filters_placeholders_non_equipment_and_loaded`; headless Gnome (`GAME/Common/leathr.kqs` loaded, `NoName74` swapped to `(147535, 43484, 12595)`, item entity spawned); live user play check 2026-11-22 (`B4-F6` confirmed working) |

## B4-F7 2026-11-22: Sweep moving animated objects against Connor (`ConnorCollision`) — core `collide.rs`; viewer `anim_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `CylinderCollider::from_stack` + `moving_object_hits_connor` sweep moving ball-stack colliders (`old_pos -> position`) against Connor's cylinder (`CONNOR_TYPE = 0x4` in `CREATURE_BITS`, so overlap starts don't re-trigger) and `anim_view::anim_step` dispatches `Active::connor_touch` when `con_collision` & `colliderMask` match | `connor-collision.md` rules 1–3, 5–10, 18–19, 21; `MASK/Gnome/kq/{rrock1,rrock2,rrock3}.kq`, `MASK/Gnome/kq/rollside.cs` | unit test `rolling_rock_hits_standing_connor_once_on_entry_and_not_while_overlapping`; headless Gnome (`--at 57500,106500,0 --lift 17500`: `rrock1` hits standing Connor on all 3 passes at updates 9, 36, 60 -> `reactback` + `KQAttackEvent`, health `220 -> 211 -> 201 -> 186`) |

## M4 4f-1 2026-11-22: Main menu core state & rules and game-pause sound rules — core `main_menu.rs`, `sound.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| Seven menu items (`New`, `Load`, `Save`, `Options`, `Replay Intro`, `Quit`, `Back to Game`), control IDs (`9, 10, 11, 14, 500, 12, 502`), `500.MSG` cases (`13, 10, 24, 19, 22, 20, 2`), 640×480 row rectangles (`64..624`, height `40` at `125, 165, 205, 245, 285, 345, 405`), mask cursor rect (`16..56`, centred on item), title lines 1–2 from `starttxt.txt` | `main-menu.md` rules 1, 2, 4, 5, "Data used"; `MASK/GAME/Kq/Mainmenu.kq`, `MASK/GAME/Resource/Mainmenu.gui`, `MASK/starttxt.txt` | unit tests `title_and_popup_text_splitting`, `startup_list_wrapping_skips_disabled_items` |
| Item availability (`game_running = has_started_or_loaded && !connor_died`; `Load` enabled iff `has_saves && !corrupt_save_pending`; `Save` & `BackToGame` enabled iff `game_running`), save-folder scan (`Pvol::parse` + `savedGameName` read-only check), corrupt-save Yes/No popup (`500.MSG` 157, click on Yes works like Enter) | `main-menu.md` rules 3, 6, 8, 23, 25, "Do not reproduce" 1 & 3 | unit test `corrupt_save_scan_and_death_state` |
| 0.2 s real-time input block on menu show and after `Replay Intro`; Up/Down wrap around skipping disabled items (`click.aud`); Enter on key-down (`intrface.aud`); click picks enabled item without hover moving cursor; Up/Down blocked while popup open | `main-menu.md` rules 8, 12, 17–21, "Do not reproduce" 5 | unit tests `startup_list_wrapping_skips_disabled_items`, `new_game_difficulty_popup_and_gameplay_resume` |
| Popups (`NewConfirm` 158, `CorruptSave` 157, `QuitConfirm` 159, `Difficulty` 120; disk check omitted per "Do not reproduce" 2): >38-char 2-line split at middle space; 2 buttons (`x 188, 361`) / 3 buttons (`x 158, 274, 392`) at `y 250`, `88×36`; Left/Right non-wrapping with `click.aud` even at edge; Enter press shows button pressed without moving the mouse pointer (user check in original 2026-11-23, `4f-F1`) and release picks; Escape closes `Difficulty` to menu or resumes game from Yes/No when running; combat levels `0.66 / 1.0 / 1.5` (`1 / 2 / 3`) | `main-menu.md` rules 10, 13–16, 24, "Edge cases", "Do not reproduce" 2; user check in original 2026-11-23 (`Q203`) | unit test `new_game_difficulty_popup_and_gameplay_resume` |
| `SoundSystem::set_game_paused` pauses/resumes playing sounds in groups 0, 2, 3, and 5 (keeping their place and waiting fades) while leaving groups 1 (menu SFX) and 4 (`Start::menu_theme`, `MENU_THEME_FADE_RATE = 1.0`) playing and stepping fades | `sound.md` rules 50 & 54; `main-menu.md` rules 7.2, 8, 9 | unit test `sound::tests::game_pause_pauses_groups_0_2_3_5_and_keeps_1_and_4_playing_and_fading` |

## M4 4f-2 2026-11-22: Main menu drawing, dithered items, popups, and theme music — core `gui_draw.rs`; viewer `menu_view.rs`, `sound_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `Piece::TextDithered` / `Canvas::text_dithered` draws disabled menu items on a 1-in-4 diagonal grid (`(x + 2*y) % 4 == 0`), matching the Invisibility potion's dither on Connor | `main-menu.md` rule 3 (checked by user in original 2026-10-21), `potions.md` rule 29, `objects.md` "Dithered objects" rule 3 | unit test `gui_draw::tests::dithered_text_keeps_one_in_four_pixels_on_diagonal_grid` |
| `MenuView` loads `Mainmenu.gui`, `Menus.ppl`, `500.MSG`, `starttxt.txt`, `main18.pbm`, `45-line.pbm`, `Mskcur.pba` (animated at 10 fps real time), `gm_win2.pbm`, `g_ybtn.pba`, `g_yhibtn.pba`, `Cursgui.pba`, and fonts `27.pft`, `45.pft`, `consoleg.pft`, drawing the 640×480 menu in the centred 4:3 screen frame and logging to `menu.log` | `main-menu.md` rules 1–5, 14–16, "Data used"; `interface.md` rule 26a (`Q108` d, `Q171` c) | headless viewer runs in `4f-3` (`menu.log`, `viewer.log`) |
| `SoundSim` (`set_game_paused`, `play_menu_sound`, `start_menu_theme`, `fade_menu_theme`) plays group-1 menu SFX and group-4 `theme.aud` (with 1.0/s fade-out) even while gameplay sounds are paused or no 3D camera is active yet | `main-menu.md` rules 7.2, 8, 9; `sound.md` rules 50 & 54 | headless viewer runs in `4f-3` (`sound.log`, `menu.log`) |

## M4 4f-3 2026-11-23: Main menu launch options (`--skip-menu`), gameplay pause, New game, and death — core `connor_inventory.rs`; viewer `main.rs`, `world_build.rs`, `gui_view.rs`, `terrain_view.rs`, `menu_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| Startup Main Menu by default (`cargo play`), skipped when `--skip-menu` / `--no-menu` (or `--world` / gameplay test options without `--menu`) is passed | `main-menu.md` rule 6; **our launch option** (user request 2026-11-22 & agent-memory "Main menu postponed") | headless runs: startup menu (`menu.log` `startup shown (New selected)`) vs `--skip-menu` (`startup skipped (--skip-menu / direct world launch)`); live user check by eye and ear 2026-11-23 |
| Opening from gameplay via Escape, top-bar mask button (`IDCTL_INVBAR_MASK`, `BarResult::Menu`), or Escape on the Portal Yes/No popup (outside `handsOff`): pauses game clock (`playing`/`running`), pauses sound groups `0, 2, 3, 5`, hides gameplay panels, records 160×120 thumbnail in memory (`SavedThumbnail`, never writing `current.bin` to `MASK/`), starts `theme.aud` in group 4, and selects `Back to Game` | `main-menu.md` rules 7, 8, 22, "Do not reproduce" 3; `interface.md` rule 41; `world-change.md` rule 26; `sound.md` rule 54 | headless run (`menu.log`, `sound.log`: `2 sound(s) in groups 0, 2, 3, 5 paused`, `160x120 thumbnail saved in memory`, `theme.aud` started, `chosen=BackToGame`, `NewConfirm` -> Escape resumes gameplay and fades `theme.aud` `1.0 -> 0.0` over `1.0 s`); live user check by eye and ear 2026-11-23 |
| Starting a New game from the Difficulty popup (`Easy`/`Normal`/`Hard` -> combat level `1`/`2`/`3` = `0.66`/`1.0`/`1.5`): clears kept worlds (`cur`), fades `theme.aud` over 1 s, skips `conStart.cs` (`$Testing` false), applies combat level before characters enter, and runs `world.cs Daventry Begin` (`BeginNow("Daventry")`) | `main-menu.md` rules 10, 24; `world-change.md` rules 9, 10, 21; `fighting.md` rule 3 | headless run (`menu.log`, `viewer.log`, `sound.log`: `Difficulty Normal -> combat level 2 (1)`, `== new game: Daventry ==`, `update 0: world.cs Daventry Begin: run`, `stngrl.cs` opening cutscene plays); live user check by eye and ear 2026-11-23 (`4f-F1` loading screen & in-game New game confirmed) |
| Connor death (`Mode::Dead`) waits 2.0 s in `handsOff` then opens the Main Menu with `Save` and `Back to Game` disabled and `New` selected | `main-menu.md` rule 25 | headless `--lift 5000` run (`connor.log` `update 23: animation mode dying -> dead` at `t = 1.675 s`, `menu.log` opens at `t = 3.669 s` with `Save & Back to Game off, New chosen`); live user check by eye 2026-11-23 |
| `trap_step` syncs `connor_dying = true` and increments `connor_stats_events` when Connor enters `Mode::Dying` or `Mode::Dead` (including fatal fall landings), and `gui_step` passes `dead` to `gui.meters.send(&stats, dead, red)` so the health bar drops to `0 %` (`0 of 107 px`) as soon as his death animation starts (`4f-F2`) | `interface.md` rule 21 (*"0 while Connor is dead (dying states)"*), `fighting.md` rule 29 | headless `--lift 5000` run (`connor.log` & `gui.log` `update 12: stats event #1: ... sent ... health 0 % (bar 0 of 107 px)`); live user check by eye 2026-11-23 |
| On-screen debug text overlay (`DebugHudText`: `HLabel`, `FloorLabel`, `LightLabel`, `HazeLabel`, `AiLabel`, `GodLabel`, `CameraLabel`) starts hidden (`DebugHudShown(false)`) and toggles show/hide with **F12** (`toggle_debug_hud`, `--key f12,N`) | **Our viewer helper** (user request 2026-11-23) | headless run (`--key f12,10 --key f12,20`: `viewer.log` shows debug text toggled on at update 10 and hidden at update 20) |

## M4 4g-1 2026-11-24: Submenu screen switching, Options (`Options.gui`), Credits (`credits.gui`), and `Options.cs` persistence — core `submenus.rs`, `sound.rs`; viewer `menu_view.rs`, `sound_view.rs`, `talk_view.rs`, `load.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `MenuScreen` (`0, 2, 4, 5, 6, 7, 8`), `Options.gui` 9 rows with two `Mskicng.pba` spinning cursors, `0..100` Music/SFX/Speech sliders (step 5, non-zero `< 10` clamped to `10%`, live updates to groups `4..=5`, `0..=2`, `3`), `Subtitles` (`Options::Captions`), `Starting Help` (`notStartingHelp`, disabled when `== 2`), and 10-command `Options.cs` persistence under `kq8-artifacts/user/Options.cs` (`MASK/` read-only per `AGENTS.md` rule 4) | `submenus.md` rules 1, 12–16, `main-menu.md` rule 11; observed `MASK/GAME/Resource/Options.gui`, `MASK/GAME/ENGLISH/500.MSG` (`#19, #33, #54, #12, #25, #26, #27, #152, #168, #3, #29, #14`), `MASK/GAME/8Gui/slider.pba` (2 `10×10` frames) (`Q206` b, c) | unit tests `slider_volume_floor_and_options_cs_round_trip`, `options_navigation_skips_locked_help_and_steps_sliders_by_5`, `sound::tests::set_sliders_updates_playing_menu_theme_and_sfx_live`; headless run (`menu.log`, `sound.log`, `kq8-artifacts/user/Options.cs`) |
| `CreditsState` (`credits.gui`) reads `GAME/ENGLISH/400.MSG` cases `1..=25`, formats `%d` on final page `25` with Connor's experience, renders `'*'`-prefixed lines in `20sl.pft` (17 px) and others in `27sl.pft` (30 px) vertically centered, shows `kqteam08.pbm` at `(120, 10)-(520, 310)` on final page (`2000 + 500*n` ms / `14000 + 500*n` ms), and fades/restarts `theme.aud` on Escape or completion back to Main Menu | `submenus.md` rules 23–25, "Edge cases"; observed `MASK/GAME/ENGLISH/400.MSG` (25 cases, case 25 has 8 leading `" "` lines so normal centering places text below `y=310` without `+145` px; `Q206` a), `MASK/GAME/Resource/credits.gui` | unit test `credits_timing_score_formatting_and_vertical_centering`; headless run (`menu.log`: `25 pages from 400.MSG`, Escape restarts `theme.aud`) |

## M4 4g-2 2026-11-24: Graphics (`Graphics.gui`) and Remap Move Keys (`movekey.gui`, `movekeys.kq`) — core `submenus.rs`; viewer `menu_view.rs`, `light_view.rs`, `shadow_view.rs`, `anim_view.rs`, `gui_view.rs`, `connor_view.rs`, `game_camera.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `GraphicsState` (`Graphics.gui` 12 rows, two `Mskicng.pba` cursors): active resolution (`640x480` / `800x600`, `> ... <`) and live `Dynamic Lights` (`set_dynamic_lighting` shared with `F9`), `setShadowMask` bits (`0x01` Connor, `0x04` Monster, `0x02` Character, `0x08` Object; applied live in `shadow_view.rs` & `anim_view.rs`), and Brightness slider (`0..100` step `5`, `0.3 + (val / 100.0) * 1.4` range `0.3..1.7`, default `50 -> 1.0`), keeping legacy DirectDraw/Direct3D/3Dfx mode switch disabled per "Do not reproduce" | `submenus.md` rules 17–19, "Do not reproduce"; observed `MASK/GAME/Resource/Graphics.gui`, `MASK/GAME/ENGLISH/500.MSG` (`#32, #9, #23, #7, #8, #1, #116, #117, #31, #6, #11, #5, #15, #4, #34`) (`Q206` c) | unit test `graphics_shadow_bits_and_brightness_formula`; headless run (`menu.log`: `Dynamic Lights -> Off`, `shadow bit 0x4 -> false (mask=11)`, `Brightness -> 55 (1.07)`, persisted in `kq8-artifacts/user/Options.cs`) |
| `MoveKeysState` / `MoveKeysFile` (`movekey.gui` 14 rows, `movekeys.kq` `KeyChar0..12` / `KeyName0..12` / `KeyWord0..12`): key-capture (`500.MSG` `#112`), reserved-key rejection (`F2, F11, F12, Escape, Enter, PageUp, PageDown, 1..8, Numpad 1..9` -> `#67`), duplicate-key displacement (unbinds `otherRow` to `99` / `#112` and jumps capture to `otherRow`, locking exit until resolved), `kq8-artifacts/user/movekeys.kq` persistence (`MASK/` read-only), and live reload into `Gui`, `Controls`, and `GameCam` | `submenus.md` rules 20–22, "Edge cases"; observed `MASK/GAME/Resource/movekey.gui` (`x = 10..40` & `600..630`), `MASK/movekeys.kq`, `MASK/GAME/ENGLISH/500.MSG` (`#54..#61, #68, #69, #64, #65, #113, #66, #34, #112, #67`) (`Q206` c) | unit test `movekeys_reserved_rejection_duplicate_displacement_and_round_trip`; headless run (`menu.log`: `F2` rejected `#67`, `'W'` bound & displaced row `0` into capture, rebound to `'S'`, `kq8-artifacts/user/movekeys.kq` saved) |

## M4 4g-3 2026-11-24: Load Game (`Loadgame.gui`), Save Game (`Savegame.gui`), and `Pvol::build` writer — data `pvol.rs`; core `submenus.rs`; viewer `menu_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `Pvol::build` packs `(name, bytes)` entries into uncompressed `"PVOL"` / `"VBLK"` (`size \| 0x80000000`, 4-byte padded) / `"vols"` / `"voli"` archives matching `Pvol::parse` | `submenus.md` rule 10; `crates/kq8-data/src/pvol.rs` | unit test `pvol::tests::pvol_build_round_trips_through_parse` |
| `SaveLoadState`, `SaveLoadPopup`, `scan_save_entries`, `decode_thumbnail_bin`, `can_append_save_char`: descending `save<N>.vol` scan across `kq8-artifacts/user/save/` and read-only `MASK/save/`, 10-row list & 190-px scrollbar (`thumb_y = 205 + top*190/(n-10)`, 10-row page clicks, drag), `Left`/`Right` button highlight (`Load`/`Save`, `Delete`, `Cancel`), 1.0 s Load thumbnail debounce, live Save thumbnail & 300 ms blinking `"\|"` caret (30 chars, `<= 2599` weighted width tenths, 500/200 ms Backspace repeat), Delete/Overwrite (`Cancel` default)/Max-50 (`#166`) popups without 4/110 MB disk checks | `submenus.md` rules 3–10, "Edge cases", "Do not reproduce"; `MASK/GAME/Resource/{Loadgame,Savegame}.gui` (`gui-verify.log`), `MASK/GAME/Kq/{Loadgame,Savegame}.kq`, `MASK/GAME/ENGLISH/500.MSG` (`#36, #37, #10, #24, #86, #35, #119, #125, #166, #148, #29`) | unit tests `save_list_sorts_descending_and_scrolls_with_10_row_window`, `save_name_typing_enforces_30_char_and_2599_width_limits`, `save_load_popups_delete_overwrite_and_max_50_limit`, `load_thumbnail_debounce_fires_after_1000_ms`; headless runs (`menu.log`: `save23.vol` thumbnail at `1.005 s` debounce, `save25.vol` write/scan/delete round-trip) |

## M4 4g-4 2026-11-24: Full `save<N>.vol` world save and restore round-trip — core `save_game.rs`; viewer `world_build.rs`, `menu_view.rs`

| Behaviour implemented | Source | Verification |
|---|---|---|
| `SavedWorldSnapshot`, `KeptWorldSnapshot` (`KQ8R_SAVE_V1`), `format_world_var_cs`, `parse_world_var_cs`, `parse_game_state_kq`: saves `savedGameName`, `savedGameVersion` (`1.000000`), `worldName`, `WorldVar.cs`, `current.bin`, `GameState.kq` (`nextNoName`, `difficultyFactor`), `worldVolumes`, `maps`, and `kq8r_state.txt` (`Journey` + Connor position/heading + `KeptWorld` states/variables/gone/came_in/rivers/maps) to `kq8-artifacts/user/save/save<N>.vol` | `submenus.md` rules 10–11; `MASK/save/save1.vol` inspected (`WorldVar.cs`, `GameState.kq` trailing `u32` + `f32`) | unit tests `kq8r_snapshot_round_trips_connor_journey_and_kept_worlds`, `parses_original_worldvar_cs_and_binary_gamestate_kq` |
| `WorldChange::restore_save` / `pending_restore` / `PerformLoad`: restores KQ8R saves (all `Journey`, `KeptWorld`, and Connor position/heading) and original `MASK/save/*.vol` saves (`worldName`, `WorldVar.cs`, `GameState.kq` `difficultyFactor` & `nextNoName`) through the world loading screen without running `BeginNow` (`world.cs <World> Begin`) | `submenus.md` rule 11; `world-change.md` rules 9, 13, 17–19 | headless Test 1 (`save25.vol` round-trip in `Daventry`, Connor stayed at `(193000, 193000)` heading `3.1416`, `5` items carried); headless Test 2 (original `MASK/save/save24.vol` loaded `Swamp`, `53` `WorldVar.cs` vars, `difficultyFactor=0.66`) |








