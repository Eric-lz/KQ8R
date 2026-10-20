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
| Action-only commands of Daventry's trap scripts are stubs (one log line, result 0); question commands (`getloc`, `checkDistance`, `exist`, `checkSave`, `getConInventory`, `getEndLoop`, `KQConner::getDoMode`) stay unknown (result 1) | **Our choice** until they are studied | Log |
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
