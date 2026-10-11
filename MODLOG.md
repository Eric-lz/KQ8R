# MODLOG

What changed and what was tested. Newest entries at the top.
Format: [MODLOG-template.md](MODLOG-template.md).

## 2026-10-11 Study session: material flags (Q5, partly)

**Changed:** `docs/behaviour/models-and-textures.md` (new section), `docs/behaviour/QUESTIONS.md`. No code.
**Why:** The implementation session asked what the unknown material bits do.
**Tested how:** Traced the model drawing code in `Mask.exe`. Counted colour indices in every texture named by the worlds' material lists.
**Result:** The flag word splits into material kind, lighting mode (none/flat/smooth), 0x1000 (forces no lighting), 0x2000, and 0x10000 (50% see-through).
**Still broken / not tested:** Which renderer options 0x1000 and 0x2000 switch on is not identified, and which colour (if any) is see-through is unknown. No texture uses colour index 0.
**Next:** M2 study (slopes, collisions, jumping), the rest of Q3/Q4.

## 2026-10-11 Study session: animated objects at load (Q12)

**Changed:** `docs/behaviour/objects.md` (new section), `docs/behaviour/QUESTIONS.md`. No code.
**Why:** The implementation session needs the right starting frame for doors and other animated objects.
**Tested how:** Traced the movement list reader, the `loop=` parser and the start-up code in `Mask.exe`; counted the `loop=` patterns in Daventry's `.kq` files.
**Result:** In `loop=`, a is the start position (0–1), b the speed (1, −1 or 0) and c whether it repeats. With no `currAnimName` the game uses movement 0. `[movementK]` is matched by number. `curAnimName` is not read.
**Still broken / not tested:** The 4th number (d) and `nextAnimName` are not traced.
**Next:** Q5 (material flags).

## 2026-10-11 Study session: buildings (Q11) and ground snapping (Q10)

**Changed:** `docs/behaviour/buildings.md` (new), `docs/behaviour/objects.md`
(Q10 follow-up), `docs/behaviour/QUESTIONS.md` (statuses). No code.
**Why:** The implementation session could not read the building files.
**Tested how:** `.dig` reader and `KQInterior` placement traced in
`Mask.exe`. Throwaway scripts in `re/` parsed every `.dig` in the install
(153 files) and checked the layout, indices, corner order and texture
mapping on Daventry's 30 buildings.
**Result:** `.dig` layout known well enough to draw buildings. Buildings at
`locZ` −1 are lifted onto the ground by `keepOnGround`, which is on by
default and only ever raises an object.
**Still broken / not tested:** That the ground lift runs when a building is
first added was not traced (the data agrees). `.dil` lighting, the BSP
tables and the two extra header values are not decoded. Texture mapping is
measured, not traced.
**Next:** Q12 (animation start frame), Q5 (material flags).

## 2026-10-11 Study session: answers to Q1, Q3, Q4; camera additions

**Changed:** `docs/behaviour/models-and-textures.md` (new),
`docs/behaviour/QUESTIONS.md` (statuses), `docs/behaviour/camera.md`
(swing back behind Connor; tilt reset after 10 units). No code.
**Why:** The implementation session asked these while drawing models (M1).
**Tested how:** Matrix convention traced in `Mask.exe`. Frame transforms
measured in the data: Connor's walk (steady part offsets, per-frame steps)
and doors (about 12–13° per frame, about 165° in total). Palettes and
archive order read from `world.cs` and `mask.cs`.
**Result:** Q1 answered (row vectors; frame transforms move the whole
object, not used for drawing; flags are hints). Q3 and Q4 partly answered.
**Still broken / not tested:** The code that applies frame steps was not
found in the exe; the conclusion rests on the data. Q2, Q5 and parts of
Q3/Q4 are still open.
**Next:** Q2, Q5, then terrain height, slopes and collisions.

## 2026-10-11 Study session: behaviour notes for Connor's movement and the camera

**Changed:** `docs/behaviour/connor-movement.md`, `docs/behaviour/camera.md`,
`docs/behaviour/QUESTIONS.md` (new). Outside git: Ghidra labels, a Ghidra
script that turns table-only code into functions
(`re/ghidra_scripts/CreateTableFunctions.java`: 531 candidates, 520
created, 11 failed), throwaway analysis scripts and scratch notes in `re/`.
No code in the repo.
**Why:** M2 (walk Connor around Daventry) needs both behaviours.
**Tested how:** Traced the behaviour in `Mask.exe` with Ghidra; measured
animation steps from `GAME/Common/C_walk.kqs` and `C_run.kqs`; checked the
camera controls against the game's manual (`MASK/Manual.pdf`).
**Result:** Walk ≈ 1,127 units/s and run ≈ 2,518 units/s, taken from the
animations' per-frame steps; turning 1.75 rad/s; banking into turns when
running, up to 0.15 rad. The camera trails Connor inside a min/max distance
range set by the walk/run scripts, stays at least 1,400 above the ground,
and has the mouse/Shift/F11/F2 controls. The manual agrees with what the
code does.
**Mistakes caught:** First drafts claimed things not yet traced (who uses
"turn to face target", when turning is disabled, a zoom formula). These
are corrected or marked as open questions in the notes. `Camera.sae` is
the real camera key map, not only a debug map as I first assumed.
**Still broken / not tested:** Nothing checked against the running game.
Open questions are listed at the end of each note.
**Next:** Terrain height, slopes and collisions.

## 2026-10-11 M1 step 3a fixes: ground rule, start frames, see-through

**Changed:** `kq8-data/objects.rs`: `keepOnGround` and the ground rule,
`current_loop()` (all six `loop=` values, movement 0 when there is no
usable `currAnimName`), replacing `current_loop_shape`. `kq8-data/shape.rs`:
`start_sequence()` (start frame). 3 new tests, 1 updated. `kq8-viewer`:
objects use their start frame and the ground rule; materials with flag
0x10000 drawn at 50% opacity; more log lines. Docs:
`docs/provenance/README.md`, `README.md`, new question Q13. Session B,
from behaviour notes `objects.md` (Q12), `buildings.md` (Q10 follow-up),
`models-and-textures.md` (Q5 partly).
**Why:** The study session answered Q12 and part of Q5, and refined Q10.
**Tested how:** `cargo test` (30 pass), `cargo clippy` (0 warnings),
`kq8-viewer --terrain --frames 120`, then read `viewer.log`.
**Result:** 140 objects placed (was 104), 120 models. 35 animated objects
now start in movement 0 (the sign, bell, baskets...). Raised onto the
ground: 0 (the mill wheel and the keg say `keepOnGround=0`). Only 2 start
past frame 0: `Pyramid` (frame 63/70) and `telepad1` (frame 5/16).
`Pyramid`'s earlier frames move it; those steps are not applied (Q13 a).
20 `loop=` lines name a sequence the model does not have; the first loop
is used (Q13 b). 2 materials drawn see-through. Placement self-check
still 0.035 units.
**Problem met:** Your viewer window was open, so Windows would not let
the build replace `kq8-viewer.exe`. I did not close it. I built a second
copy in `target/side/` (git ignores it) and tested with that.
**Checked by eye:** The alchemist's sign is right, including its tilt in
the starting frame. The bell and the baskets look right. Doors are closed.
**Found by eye:** The waterfalls and the teleport pad should be
see-through but are solid. The cause is in the data: their materials do
not have flag 0x10000. The waterfalls' object files say
`translucency=200` (22 Daventry objects have such a key, values 50–256
across the game). The teleport pad has no such key. Meaning not known, so
not guessed: question Q14.
**Still broken / not tested:** Waterfalls, streams, teleport pad etc.
drawn solid (Q14). Buildings still missing (step 3b).
**Next:** Step 3b (buildings).

## 2026-10-11 M1 step 3a: place Daventry's objects

**Changed:** `kq8-data`: new `objects.rs` (placements, rotation, the
current animation's model), 4 new tests. `kq8-viewer`: `--terrain` now
also draws objects (`load.rs`: `load_objects`, `World::entries_ending`,
palette lookup for any `KQWorld::...Palette`; `terrain_view.rs`:
`setup_objects`, `bevy_transform`; `log.rs`: quiet mode; `main.rs`).
Also corrected the fly-speed comment after your speed change. Docs:
`docs/DESIGN.md`, `docs/provenance/README.md`, `README.md`, new
question Q12. Session B, from behaviour note `docs/behaviour/objects.md`
(Q9, Q10) and the data.
**Why:** M1 step 3a: see Daventry's objects in place.
**Tested how:** `cargo test` (27 pass), `cargo clippy` (0 warnings),
`kq8-viewer --terrain --frames 120`, then read `viewer.log`.
**Result:** 355 sections with a position: 104 placed (74 `KQObject`, 17
doors, 8 animated objects, 3 chests, 2 portals) using 90 models. Bevy's
placement equals the game's rule to 0.035 units. 21 rest on the ground
(±50), 81 are higher, 2 lower (mill wheel 601 below, keg 422 below).
Skipped: 91 at (0, 0, 0), 58 with no model, 51 animated without
`currAnimName` (Q12), 33 buildings (step 3b), 15 monsters (`.anm`), 2 whose
first frame is empty (`map1.kqs`, `Ug_Cut5b.kqs`), 1 with `shapeName=stop`.
Loading 224 ms; first frame after ~2.4 s; average frame 6.0 ms.
**Not in the plan, added:** The plan only covered `shapeName`, but only
`KQObject` sections have one. Doors, chests and so on name their model
through their animation list. That is plain file layout, so I added it.
Which frame to show is not in the data, so they show their first frame
(Q12).
**Checked by eye:** You flew around: many objects in place, including
doors, paintings and the mill wheel. The keg stands where the pub should
be. Inside an expected house, items float where a table should be: the
table is probably part of the building (step 3b). Most doors look closed,
but you are not sure. So "closed" is not confirmed (Q12).
**Still broken / not tested:** No buildings, so Daventry still looks
mostly empty. Also missing: monsters, emitters, the 51 animated objects
without `currAnimName`.
**Next:** Step 3b (buildings) needs Q11; door frames need Q12.

## 2026-10-11 M1 step 2b: draw Daventry's ground with textures

**Changed:** `kq8-data/src/terrain.rs`: `ground_triangles()` and
`texture_coord()` (orientation codes 0–7), 4 new tests. `kq8-viewer`:
`--terrain` mode (new `terrain_view.rs`: ground scene, flying camera,
camera log; `load.rs`: `load_ground`, terrain palette from `world.cs`;
`main.rs`: shared mesh/material helpers, mode switch). Docs:
`docs/DESIGN.md`, `docs/provenance/README.md`, `README.md`. Session B,
from behaviour notes `terrain.md` (Q6–Q8) and `models-and-textures.md`
(Q2, Q3).
**Why:** M1 step 2b: see Daventry's ground.
**Tested how:** `cargo test` (23 pass), `cargo clippy` (0 warnings),
`kq8-viewer --terrain --frames 120`, then read `viewer.log`. Also reran
`kq8-viewer smlchest.KQS` to check the model view did not change.
**Result:** All 16,384 squares (32,768 triangles) drawn with 30 textures,
all found in Daventry's archives (128×128, mipmapped). Palette
`DAVENTRY.PPL`, as named by `world.cs`. Loading 27 ms, average frame 6.0 ms.
Camera starts at game (262144, 262144, 16750), 4,000 units above the
ground. Chest unchanged (same bounds, 30 of 32 faces outward).
**Mistake caught:** A shell step dropped a backslash in `load.rs` (build
error), and one test line I wrote was garbled. Both fixed before testing.
**Checked by eye:** You flew over it: "everything looks perfect". The land
shape and textures are right, and textures join up between squares.
**Still broken / not tested:** No lighting, haze, shadow maps, holes or
`terrh2.dib`. No buildings or objects. Mipmaps not used. Only Daventry
looked at; other worlds not tried in the viewer.
**Next:** Step 3, objects and buildings.

## 2026-10-11 Fix: hide the back sides of model triangles

**Changed:** `crates/kq8-viewer/src/main.rs`: each triangle's corners
are given to Bevy in reverse order, and both-sided drawing is switched
off (Bevy's default hides back faces). The viewer logs how many triangles
face away from the model's centre. `docs/provenance/README.md`,
`docs/DESIGN.md`, `README.md`. Session B, from behaviour note
`docs/behaviour/models-and-textures.md` (Q2).
**Why:** The original hides back faces, and its front faces have clockwise
corners. Bevy's front faces are counter-clockwise, hence the reversal.
**Tested how:** `kq8-viewer` with `smlchest.KQS`, `broom.KQS`,
`alcmsign.KQS` (`--frames 30` each), read the log. `cargo clippy`.
**Result:** Chest: 30 of 32 triangles face outward (the old order would
give 2 of 32). Broom 12 of 22, sign 49 of 66. Those two are not box-shaped,
so this test says little about them.
**Checked by eye:** You turned the chest and the sign all the way around:
both look good (nothing missing, no insides showing).
**Still broken / not tested:** Only these two models checked by eye.
**Next:** Step 2b.

## 2026-10-11 M1 step 2a: read the terrain, height lookup

**Changed:** `kq8-data`: new `kq_text.rs` (reader for `[section]` /
`key=value` files), `terrain.rs` (settings, height/material grids,
direction table, height lookup). `kq8-cli`: new `terrain_verify.rs`
(`kq8 terrain-verify`). Docs: `docs/DESIGN.md` (2a done, 2b plan),
`docs/provenance/README.md`, `README.md`. Session B, from behaviour note
`docs/behaviour/terrain.md` (Q6, Q7) and the data files.
**Why:** M1 step 2a: the ground has to load before it can be drawn.
**Tested how:** `cargo test` (19 pass, 5 new: row flip, corners, both
diagonals, slope direction, text files), `cargo clippy` (0 warnings),
`kq8 terrain-verify` on all 12 worlds.
**Result:** 17 terrain files read, 0 failures, 0.1 s. Objects exactly on
our ground (±0.5 units): Daventry 11 (heights 6,630–30,600), Barren 11
(92 within 5), Ice World 11, Dead City 7, Snow exit 2. Daventry's height
step is exactly 255. Many objects stand exactly +5 or +25 above the ground.
That explains the note's 256.2.
**First run failed:** 4 "failures" were the temple worlds' `terrain2.kq`
naming `ice.dml`, which those worlds lack. A fact about the data, so it is
now a note, not a failure.
**Still broken / not tested:** Gnome: 0 objects on the ground (median
17,450 units off; probably caves, not studied). Temple grounds are flat at 0.
Holes, `terrh2.dib` (differs from `terrh.dib` at 120 Daventry points, all
1–5 steps lower) and texture mapping (Q8) open. Nothing drawn yet.
**Next:** Step 2b, draw the ground (untextured until Q8), after your go-ahead.

## 2026-10-11 M1 step 2 planned; terrain size and layout blocked on questions

**Changed:** Docs only: `docs/DESIGN.md` (M1 step 2 plan; roadmap
corrected: `.dig`/`.dil` are buildings, not terrain),
`docs/behaviour/QUESTIONS.md` (Q6–Q8). No code.
**Why:** Start of M1 step 2. You asked me not to guess terrain scale,
height lookup or texture mapping.
**Tested how:** Read `TERRAIN.KQ` of every world and the Daventry terrain
files; measured them with throwaway scripts kept outside the repo.
**Result:** File layouts are clear: heights 129×129 (0–254), materials
128×128, `daventry.bin` = 4 + 255 × 28 bytes exactly, names match
`davennew.dml` 255 of 255. Units and orientation are **not** clear.
**Dead end:** Two attempts to find the scale from object positions.
(1) Straight-line fit of object height against the height image, 4,096-unit
squares, 4 image orientations: best R² = 0.24 (weak). (2) Scan of 12 square
sizes (1,024–8,192), height steps 80–200 per grey value, 4 orientations,
counting objects within 150 units of the surface: best 23 of 247, with
several quite different settings tied, which is noise. Most objects
are not on open ground (inside buildings, on tables, flying). Stopped
there and asked the study session instead (Q6, Q7).
**Odd finding:** The most used material number (159, on 93% of squares)
is `stone1.bmp`. That suggests the material image is not a plain index,
or I misread it (Q8).
**Still broken / not tested:** Nothing built yet. Step 2a (reading the
files) does not depend on the answers; step 2b (drawing) does.
**Next:** Your choice: build step 2a now, or wait for Q6–Q8.

## 2026-10-11 Fix: frame transforms are not used to draw model parts

**Changed:** `kq8-data/src/shape.rs`: `Shape::geometry` no longer applies
the sequence's frame transform, only each part's mesh node transform; doc
comments and the unit test updated (the test now gives frame 0 a 50-unit
step and checks it is ignored). `kq8-cli/src/shape_verify.rs`: the matrix
test no longer transposes frame transforms (they are not drawn).
`docs/provenance/README.md`. Session B, from behaviour note
`docs/behaviour/models-and-textures.md` (Q1 b).
**Why:** The study session found frame transforms are the step the whole
object makes during an animation (Connor's walk steps ~70–81 units per
frame; doors turn ~11–13° per frame). Drawing them would make Connor
lurch and doors never open past 13°.
**Tested how:** `cargo test` (14 pass), `cargo clippy` (0 warnings),
`kq8 shape-verify` before and after, compared line by line;
`kq8-viewer smlchest.KQS --frames 60`.
**Result:** No difference in any number: all 4,573 rest poses identical,
because frame 0 always has a zero step. The chest looks the same
(same bounds in the viewer log).
**Still broken / not tested:** Nothing visible changes until animation is
added (M2/M3); that is when the frame steps will be used, to move objects.
**Next:** M1 step 2 (terrain), after your go-ahead.

## 2026-10-11 M1 step 1b checked by eye

**Changed:** Docs only (`README.md`, `docs/provenance/README.md`). No code.
**Why:** You looked at the viewer.
**Tested how:** You ran `kq8-viewer smlchest.KQS` and `alcmsign.KQS`. I
then measured the sign's tilt in each animation frame with a throwaway
program kept outside the repo (scratchpad).
**Result:** Chest correct: lid closed, wood-and-metal texture. The sign
looks right but tilted. Measured: its animation is 31 frames over 2.0 s.
The tilt goes 10° (frame 0) → 0° (6) → 10° (15) → 0° (23), a swing. The
viewer shows frame 0, so the tilt is the start of the swing, as you
remembered. This also supports the row-vector matrix choice (Q1).
**Still broken / not tested:** Whether models are mirrored is not ruled
out (Q2). Neither model was confirmed to have readable lettering. No
animation yet. Q1–Q5 still open.
**Next:** M1 step 2, terrain.

## 2026-10-11 M1 step 1b: show one textured model in a Bevy window

**Changed:** New crate `crates/kq8-viewer` (`main.rs` Bevy window and
orbit camera, `load.rs` model/texture/palette lookup, `log.rs`). Added
Bevy 0.20.0 (only its 3D features) to the workspace. `Cargo.toml`:
`default-members`, so plain `cargo run -- <command>` still runs `kq8`.
Docs: `README.md`, `CREDITS.md`, `docs/DESIGN.md` (layout),
`docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md`.
**Why:** Second part of M1 step 1: see a game model on screen.
**Tested how:** `cargo run --release -p kq8-viewer -- smlchest.KQS --frames 60`
(also `--frames 120`, and `broom.KQS`, `alchy.KQS` with `--frames 30`),
then read `kq8-artifacts/logs/viewer.log`. `cargo test`, `cargo clippy`.
**Result:** Window opens and draws on the RTX 4060 Ti (Vulkan). Chest:
32 triangles, 1 texture (`ITEMS22.BMP` 128×128), first frame after
~0.5–1.1 s, average 6.4 ms per frame. For all 4 textures checked, the
image's own colour table equals `DAVENTRY.PPL` for every colour used.
**Mistake caught:** Adding a second program to the workspace broke
`cargo run -- vol-verify` (cargo could not tell which program to run).
Fixed with `default-members` before handing over.
**Note:** While I worked, another session replaced `QUESTIONS.md` with a
table template, which removed my Q1–Q5. I re-added them as table rows and
did not change anything else in that file or in `connor-movement.md`.
**Still broken / not tested:** **Nobody has looked at the window yet.**
Not known: upside-down textures, a mirror image, parts in wrong places.
No lighting, no animation, no transparency, both sides of triangles drawn.
**Next:** You look at the chest (procedure in the hand-over message). Then
M1 step 2, terrain.

## 2026-10-11 M1 step 1a: read 3D models (.kqs) and material lists (.dml)

**Changed:** `kq8-data`: new `pers.rs` (the PERS wrapper + a byte reader),
`material.rs` (`TS::MaterialList`), `shape.rs` (`KQ8::Shape`,
`TS::CelAnimMesh`, building the triangles of one animation frame).
`kq8-cli`: new `shape_verify.rs` (`kq8 shape-verify`). Docs:
`docs/DESIGN.md` (M1 step 1 plan), `docs/provenance/README.md`, new
`docs/behaviour/QUESTIONS.md` (Q1–Q5). Session B; no behaviour note exists
for this, so it is built from the data files and the libkq8fpc notes.
**Why:** First part of M1 step 1: models must load before they can be shown.
**Tested how:** `cargo test` (14 tests, 4 new). `kq8 shape-verify` on the
whole game.
**Result:** 4,573 models and 25 material lists parse, 0 failures, 0 unread
bytes, 3.4 s. 23,153 meshes, 2,397,317 faces, all indices in range. Z is
up (tallest axis in 2,950 models; 3,177 stand on z ≈ 0). Matrix test
favours row-vector (245 models clearly better vs 7).
**Dead end / correction:** The community note says material flags are 1
byte. They are 4 bytes. File sizes only add up that way.
**Still broken / not tested:** 4 `TS::Shape` tomb files skipped.
Matrix convention, front faces, texture palette, texture search order and
3 flag bits are open questions (QUESTIONS.md). 29 texture names are
missing from the game files. Nothing drawn yet.
**Next:** Step 1b, show one model in a Bevy window.

## 2026-10-11 Ghidra connected, two-session workflow, roadmap

**Changed:** `docs/SESSIONS.md` (new: rules and prompts for the study and
implementation sessions, behaviour-note format), `docs/DESIGN.md` (links
to it; the four-step plan replaced by a milestone roadmap M0–M7),
`.gitignore` (`/re/`). Outside git: `re/` holds a copy of `Mask.exe`, the
Ghidra project and study scratch notes; `.mcp.json` configures the
GhidraMCP bridge (pinned to release 2.0). No code changed.
**Why:** You decided to study `Mask.exe` and to keep the two-session method
from ai-game-modding-guides guide 13. Behaviour of object kinds, movement,
combat and camera lives in the exe, not the data.
**Tested how:** Called the Ghidra tools from this chat: program info,
class list, string search, cross-references.
**Result:** Connected to `re/Mask.exe` (sha256 matches the game's exe),
6,413 functions. Ghidra recovered only 2 classes although ~399 C++ class
names are in the file; its class detection apparently does not understand
this 1998 compiler's layout. The names are still usable by following
references by hand.
**Still broken / not tested:** Class structure not recovered yet. `/mcp`
toggle for the implementation session not tested.
**Next:** Your choice of graphics library, then M1 (see Daventry).

## 2026-10-11 Fix: wrong palette for the Daventry image

**Changed:** Docs only (`README.md`, `docs/provenance/README.md`). No code.
**Why:** You looked at `daventry.png` made with `daventry.ppl`: very dark,
speckled, the paper background wrong. It should be a pencil drawing on old
paper.
**Tested how:** Searched the game scripts (`mask.cs`, `world.cs`) for
palette names, then converted `daventry.pbm` with every `.ppl` in `GAME/`
and `data/Daventry/` and compared how much colour changes between
neighbouring pixels.
**Result:** All 16 loading palettes `GAME/8Gui/l*.ppl` have identical
colours and score 18.4, against 44.5 for `daventry.ppl` and 107.5 for
`Menus.ppl`. The image's most common colour (index 11, 44,125 pixels) is
near-black (8,0,0) in `daventry.ppl` and brown-grey (128,112,96) in the
loading palette, which explains the dark picture.
**Mistake:** In the previous entry I called `daventry.ppl` "the right one".
My measurement only showed it was better than `Menus.ppl`, not that it was
correct. I should have tried every palette before concluding. My first file
search also stopped after 40 entries and missed the `l*.ppl` files.
**Still broken / not tested:** You checked the new PNG: colours correct.
Which of the 16 identical loading palettes the engine loads is unknown
(it does not matter for colours).
**Next:** Step 3 (`.msg`).

## 2026-10-11 Step 2: read images and palettes, export PNG

**Changed:** `kq8-data`: new `chunks.rs`, `bitmap.rs`, `palette.rs`.
`kq8-cli`: new `source.rs` (load a loose file or `vol:` archive entry),
`bitmap_png.rs` (`kq8 bitmap-png`), `bitmap_verify.rs` (`kq8 bitmap-verify`);
`main.rs` takes the new commands. Added the `png` crate. Updated
`docs/provenance/README.md`, `docs/DESIGN.md`, `CREDITS.md`, `README.md`.
**Why:** Step 2 of the plan: first real asset format on top of the archives.
**Tested how:** `cargo test` (9 library tests: chunked 8-bit, mipmaps,
24-bit colour order, Windows BMP, both palette kinds). `kq8 bitmap-verify`
on the whole game. `kq8 bitmap-png` on `GAME/8Gui/daventry.pbm`.
**Result:** bitmap-verify: 6,495 images and 50 palettes, 0 failures,
1.7 s. Variants found: 1,909 `.pbm` + 1,173 mipmapped archive `.bmp`
(chunked), 3,382 Windows-BMP `.bmp` (8 of them 24-bit), 31 `.dib`, 44 `.ppl`,
6 RIFF `.pal`. bitmap-png: 640×480, 154 colours used.
**Dead end recorded:** I first paired `daventry.pbm` with `Menus.ppl` (the
menu palette locked in `mask.cs`). A measurement showed it was wrong:
average neighbouring-pixel colour change was 107.5, against 44.5 with
`Daventry/resource/daventry.ppl`, and 14–68 for images that carry their own
palette. The world's own palette is the right one. (Wrong: see "Fix: wrong palette" above.)
**Still broken / not tested:** Nobody has looked at the PNG yet (you need
to). Transparency, palette lighting/haze tables, `.pba` image arrays and
`.pal` with 237 colours are not handled. The forced system colours at both
ends of the palette come from the GPL notes and are not verified against
`Mask.exe`. No real 24-bit chunked image exists to test that path.
**Next:** Step 3, dialogue text (`.msg`).

## 2026-10-11 Read the modding guides, adopt their templates and working method

**Changed:** Cloned `ai-game-modding-guides` (commit `0c14be4`) and
`GameDecompLibrary` (commit `0e1357d`) into `reference/`, which is never
committed (added to `.gitignore`). Copied `MODLOG-template.md` and
`STATUS-handoff.md` to the project root (AGENTS.md refers to them).
Added `docs/provenance/README.md`, a "working method" section in
`docs/DESIGN.md`, new findings in `docs/RESEARCH.md` (versions, ScummVM,
GameDecompLibrary, EULA), and an attribution-template layout in `CREDITS.md`.
README now says it's an unofficial fan project and mentions AI use.
Reformatted this log to the template.
**Why:** You asked me to follow the guides. They recommend recording
provenance, splitting disassembly work across two sessions, and the
templates AGENTS.md expects.
**Tested how:** `git status --short -uall` to check only our own files show
up, and `git check-ignore` on `reference/`.
**Result:** Only project files are listed; `reference/`, `MASK/`, `target/`
and `kq8-artifacts/` are ignored. No code changed, so `vol-verify` was not
rerun.
**Still broken / not tested:** Nothing new to test. One mistake caught:
I first wrote an unverified claim in the provenance file (that all VOL
streams start with `00 04/05/06`). Corrected it to what was actually observed.
**Next:** Step 2 of the plan, a `.pbm` bitmap + `.ppl` palette dumped to PNG.

## 2026-10-11 Step 1: read VOL archives, prove the data loads

**Changed:** Whitelist `.gitignore`; Cargo workspace with `kq8-data`
(`VolArchive`, `GameDir`) and `kq8-cli` (`kq8 vol-verify`); `docs/DESIGN.md`,
`docs/RESEARCH.md`, `CREDITS.md`.
**Why:** The smallest proof that we can read the original data. Every other
format lives inside these archives.
**Tested how:** `cargo test` (3 unit tests on a hand-built archive), then
`cargo run --release -- vol-verify` on the GOG 1.03 install.
**Result:** 48 archives, 14,444 entries, 382,267,214 bytes unpacked, all
14,444 byte-identical to the installer's unpacked copies, 0 errors, 0 size
slack, ~8.5 s. `RESULT: PASS`.
**Still broken / not tested:** The reference copies were unpacked by
`kqmoevol`, not by `Mask.exe`, so this proves agreement with kqmoevol, not a
direct comparison with the original game. Not tested on Linux/macOS.
**Next:** Bitmaps and palettes.
