# Models, animation frames and textures

Study session A, 2026-10-11. Answers to `QUESTIONS.md` Q1, Q2, Q3 and Q4
(partly). Sources: game data files (observed, with throwaway measurements)
and `Mask.exe` (studied, described in our own words). Q5 partly
answered ("Material flags").

## Transforms (Q1)

### (a) Row vectors

A transform is a 3×3 matrix plus a translation. The game moves a point as a
**row vector**: `point' = point × M + translation`, where `M` is stored row
by row (first three numbers = first row). Combining "first A, then B" gives
the matrix `A.M × B.M` and the translation `A.translation × B.M +
B.translation`. **Confirmed** in `Mask.exe`. The implementation session's
current choice is right.

### (b) Mesh node transforms vs frame transforms

- **Mesh node transform**: places one part of the model in the model's own
  space. Applied when drawing that part.
- **Frame transform** ("sequence" `frameTransform` in the community notes):
  **not part of drawing the parts.** It is the step that the *whole object*
  makes when the animation reaches that frame. It is added onto the object's
  own placement in the world, frame after frame:
  - Connor's walk: each frame steps about 70–81 units along −Y with no
    rotation (see `connor-movement.md`). The mesh node offsets in the same
    frames stay steady (−5 to −12 on Y); nothing cancels the step, so
    drawing with it would make Connor lurch forward and snap back each cycle.
  - Doors (`door1.kqs`, `door2.kqs`, `door3.kqs`, `keepdrl.kqs`,
    `keepdrr.kqs`, `outdor.kqs`, `tombdoor.kqs`): each frame turns about
    11–13° about Z with a matching offset (consistent with a hinge about 620
    units from the model origin). Over the 16 frames that adds up to about
    165°. Read as absolute poses the door would never open more than 13°.
  - Frame 0 of every animation has a zero step (start of the loop).
- Of 2,321 `KQ8::Shape` files in `GAME/` and `daventry/`: 1,095 have frames
  with a step (almost all character animations), 8 have frames that rotate
  (the doors above and `fp_react.kqs`), the rest have none.
- **Evidence**: measured from the data (above). The code that applies the
  step was **not** found in `Mask.exe` yet; the conclusion rests on the data.
  Confidence: high for "not used when drawing parts"; exact timing of the
  step (all at once on reaching a frame, or blended) is still open
  (`connor-movement.md` open question 1).

What this means for drawing a model: draw each part with its mesh node
transform, then the object's world placement. Ignore the frame transform
while drawing; use it to move the object when animations play (M2/M3).

### (c) Flag bits

The flags (1 = has rotation, 2 = has translation, 4 = has scale) are hints
only. Moving a point uses the full matrix and translation regardless of the
flags. The game uses them to take shortcuts (for example, to invert a
transform it simply flips rows and columns when the scale bit is clear,
which is only valid for pure rotations). Treat them as information; never
skip the matrix or translation because a bit is clear, unless you have
checked the data agrees.

## Front faces, hidden back faces, handedness (Q2)

1. **Handedness: the world is right-handed and shown without a mirror**:
   X east, Y north, Z up. Evidence: the game builds a turn about Z so that a
   positive angle moves +X towards +Y (anticlockwise seen from above), and
   the left-arrow key sends a positive turn (`Connor.sae`). Anticlockwise
   seen from above is a left turn for anyone standing upright, so the
   picture on screen must be unmirrored. Converting to a Y-up, right-handed
   renderer (Bevy) is a plain rotation: (x, y, z) → (x, z, −y). No mirror.
2. **Front side: corners run clockwise when seen from the front.** Measured
   across 2,320 models (`daventry/resource` and `GAME/Common`): comparing
   each triangle's corner order with the normals stored in its vertices,
   1,471,394 triangles are clockwise seen from the side the normals point
   to, 16,414 the other way (98.9% vs 1.1%); 2,317 of 2,320 models are
   mostly clockwise. Vertex order is the order the face lists its three
   corners.
3. **Back faces are hidden.** In the 3dfx Glide renderer the game switches
   back-face removal on once at start-up and never changes it, so every
   model is one-sided (no per-material "two-sided" setting was found).
   The Direct3D and software renderers were not checked.
   In Bevy terms: front face = clockwise after the rotation in point 1
   (or reverse each triangle's corner order and keep Bevy's default), with
   back-face culling on.

## Which palette colours textures (Q3)

From the world scripts (`<world>/kq/world.cs`, observed): each world names
its palettes:

| Setting | Daventry | Other worlds |
| --- | --- | --- |
| `KQWorld::objectPalette` | `daventry.ppl` | Swamp `swamp.ppl`, Dead City `newdead.ppl`, Gnome `gnome.ppl`, Barren `barren.ppl`, Ice World `ice.ppl`, Snow exit `exit.ppl`, temples `temple0N.ppl`, Castle `Castle.ppl` |
| `KQWorld::terrainPalette` | `daventry.ppl` | same as object palette |
| `KQWorld::connorPalette` | `daventry.ppl` | (other worlds not checked) |
| `KQWorld::interiorPalette` | `daventry.ppl` | (other worlds not checked) |

So: objects use the world's object palette, the ground uses the terrain
palette, Connor the Connor palette, interiors the interior palette. In the
Daventry data these are all the same file, but keep them separate
so a world can differ. This matches what the implementation session
measured (`daventry.ppl` gives the smoothest result).

The scripts also set `KQWorld::loadingPalette daventry.ppl`, which does not
match the loading-screen image's palette (`GAME/8Gui/l*.ppl`, confirmed by
eye). What `loadingPalette` is used for is open.

Whether the first/last 8 system colours are forced for textures is **not
answered yet** (still being studied).

### System colours (Q3, answered 2026-10-11)

- In the palette files, entries **0–9 and 246–255 already hold the
  standard Windows system colours** (e.g. `daventry.ppl`: 1 = dark red,
  7 = light grey, 8 = (192, 220, 192), 9 = (166, 202, 240), 246 = cream,
  255 = white). The one exception is **entry 0, stored as pure blue
  (0, 0, 255)** where Windows has black. (Measured.)
- The game replaces these **20 entries** (10 at each end, not 8) with the
  computer's real system colours **only when the display runs in
  256-colour mode**. (Traced: it asks Windows for the system palette and
  copies it in, only for an 8-bit screen.) In any other mode the palette is
  used as stored.
- So for a modern display: **use the palette as stored, except entry 0,
  which is black** (see the next point). The current "force 8 system
  colours at each end" rule (from libkq8fpc notes) is not what this game
  does on a normal screen. It only matters for textures that use those
  entries (e.g. `Teleport.BMP`).
- **Entry 0 is black on a normal screen too (measured 2026-10-12, Q22).**
  In the running original (GOG version, normal desktop, not 256-colour
  mode), the copy of `daventry.ppl` held in memory matches the file in
  all 256 entries **except entry 0, which is (0, 0, 0)** instead of the
  stored (0, 0, 255). No other entry is changed. So whatever the screen
  mode, use **black for entry 0**. This corrects the earlier "blue as
  stored" for entry 0 and agrees with the `0x1000` answer below. (Where
  the game does this was not traced; only Daventry's palette was
  checked, and the rule is assumed to hold for every world's palette.)
  Affected textures: Swamp's `WIRLPOOL.BMP` and `Teleport.BMP` (black,
  not blue, where they use entry 0).

## Where texture names are looked up (Q4, partly)

When a world is entered, `MASK/mask.cs` loads that world's archives in this
order (observed):

1. `<parent world>/Sound`
2. `<world>/Kq`
3. `<world>/8 Bit` (textures; `8bit.vol`)
4. `<parent world>/English` (text)
5. `<world>/Resource`
6. `<world>/8 Gui`
7. `<world>/Patch`

then moves a `cur` folder to the front of the search list
(`KQGame::moveVolToFront cur`). The game also has a default search path
`.` (`console.cs`).

This is why the shared models in `GAME/Common` name textures that exist only
in world archives: the same model is textured by the **current world's**
`8bit.vol`. A texture missing from the current world is expected to be
missing in the original too.

**Which one wins (traced, 2026-10-11):**

- `loadVolume` puts each new folder at the **front** of the search list,
  and a lookup walks the list from the front and takes the **first** match.
  So the folder loaded **last** wins. For a world the search order is
  therefore: `cur` (moved to the front last), `<world>/Patch`,
  `<world>/8 Gui`, `<world>/Resource`, `<parent>/English`, `<world>/8 Bit`,
  `<world>/Kq`, `<parent>/Sound`, and then the game-wide `GAME/...` folders,
  which the program loads before any world. The order inside `GAME/` (for
  example `GAME/patch` before `GAME/Kq`) is **not traced**; the patch
  folder first is the likely reading.
- **The game reads the loose folders** (`MASK/daventry/Kq`,
  `MASK/barren/Patch`, ...), not `MASK/data/<World>/*.vol`. `mask.cs` names
  folders, and the patch folders exist only as loose folders:
  `GAME/patch` (2 files), `barren/patch` (13), `deadcity/patch` (5),
  `iceworld/patch` (3), `temple4/patch` (1). They hold fixed `.kq`/`.cs`
  files (e.g. Barren's `world.cs`, `brokbrig.kq`, `elevator.cs`).
  **Reading only the `.vol` archives misses these fixes.** The `.vol`
  files match the loose copies for everything else. (Traced: the volume
  loader checks whether the path is a folder and, if so, reads the folder;
  in this install every such folder exists.)
- Measured, all 12 worlds: 180 names appear in more than one loaded
  folder; 104 of those differ in content. For art files the differences
  are few: `4green.pbm`/`4red.pbm` (world `8gui` over `GAME/8Gui`),
  `clicker.kqs`, `bp_badie.kqs` (Barren) and `mush1.kqs` (Swamp), where the
  world's copy wins over `GAME`'s.

Still open: what the game does when a texture name is found nowhere (29
names). Not traced; draw a plain colour.
Second try 2026-10-17, also without result: the material list's own
loading only reads the names from the file; the step that turns a name
into a picture is somewhere else, and no error message about a missing
texture was found to lead there.
**Settled from the data instead (2026-10-17): none of the 29 names is ever
drawn in normal play**, so the answer doesn't matter:

- 25 names (`gate02`, `grani13`–`24`, `marbl11`–`36`, `stone04`–`78`)
  are only in `deadcity/resource/zedcity.dml`, a material list that no
  file in the game names.
- `bro20.bmp` is only in `deadcity/resource/bro.kqs`. Dead City's brother
  is drawn with `bro_die`, `bro_lkup`, `bro_stp`, `bro_talk` and
  `bro_dead.kqs`, never `bro.kqs`. (His lip pictures are `bro0`–`bro8`.)
- `ghoul.bmp` is only in `deadcity/resource/lifesucm.kqs`, which no
  file names.
- `chf0.bmp` is only in `GAME/Common/Helmet.kqs`, the model of the
  Helmet item. No world places a helmet and no monster carries one; it
  appears only in Connor's own lists. (`chf` is the name of Connor's
  face pictures with the helmet in `Connor.kq`'s skin list, but no `chf`
  picture exists in the game at all; the plain face `cf0`, `cf1`, ...
  does.)
- `fndmask.bmp` is only in `swamp/resource/fndmask.kqs`, which no file
  names.

So a plain colour (our stand-in) is fine; nobody will see it. Not
checked in the original, since there is nothing to look at.

## Material flags (Q5, partly)

Traced in the model drawing code of `Mask.exe`; meanings of the names are
ours. The flags word is split into parts:

| Bits | Meaning | Confidence |
| --- | --- | --- |
| `0x000F` | Kind of material. 3 = textured (the drawing code only sets up a texture for 3). 0 = nothing to draw (all 31 `0xF000` and 192 `0x0` materials) | traced for 3; 0 = "nothing" is likely, not traced |
| `0x0F00` | Lighting: `0x100` = none (full brightness), `0x200` = flat (one light value per triangle, from its facing), `0x400` = smooth (light per corner) | traced |
| `0x1000` | **Colour key on**: in the Glide renderer, texels whose final colour is pure black (0, 0, 0) are not drawn. Also **forces lighting to "none"**, so black stays exactly black | traced (Glide `grChromakeyMode(on)`, `grChromakeyValue(0)`); D3D path not checked |
| `0x2000` | **Alpha blending from the texture's alpha** (Glide blend mode 3) and lighting "none" | traced; only appears inside `0xF000` materials, which have nothing to draw |
| `0x10000` | **Halves the haze** (distance fog) on that material, so it stays clearer far away. Not see-through. (Corrected 2026-10-11: the first version said "halves the opacity"; the value it halves is the haze amount, which grows with distance from the camera.) | traced |

What the data says about `0x1000` (`0x1403`, 20 materials): the textures
are `nightsky`, `11gnd`, `Plstrwl1/2` (plaster walls), `Srwroof` (straw
roof) and similar: things that should not be shaded, not see-through
things. **No texture in the game uses colour index 0** (checked in the
textures of all 1,398 world material-list entries whose image was found), so "index 0 is see-through"
is not how these files work. **Answer (2026-10-11):** the see-through colour for `0x1000` is
**pure black after palette lookup** (in `daventry.ppl` that is index 10,
plus index 0, which the engine sets to the system colour black). Measured:
the 20 `0x1403` textures in the worlds use neither, so in this data the
key changes nothing visible. Draw `0x1000` materials at full brightness and
treat black texels as see-through. Draw `0x10000` materials fully solid
(they only get less fog). See-through objects otherwise come from the
object's `translucency` key (`objects.md`, Q14).

## Do not reproduce

Nothing identified yet.

## How to check

- Draw `door1.kqs` frame 0 with frame transforms ignored: a closed door.
  Apply frames 1..N as accumulated steps: the door swings about 165° about
  its hinge edge.
- Draw Connor's walk model with frame transforms ignored: he walks on the
  spot without lurching.
