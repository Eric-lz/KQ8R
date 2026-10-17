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
