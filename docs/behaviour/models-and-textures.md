# Models, animation frames and textures

Study session A, 2026-10-11. Answers to `QUESTIONS.md` Q1, Q3 and Q4
(partly). Sources: game data files (observed, with throwaway measurements)
and `Mask.exe` (studied, described in our own words). Q2 and Q5 are still
being studied.

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

Still open: which archive wins when a name exists in several loaded ones
(front of the list first is the likely reading of "move to front"), and what
the game does when a name is not found anywhere (29 names).

## Do not reproduce

Nothing identified yet.

## How to check

- Draw `door1.kqs` frame 0 with frame transforms ignored: a closed door.
  Apply frames 1..N as accumulated steps: the door swings about 165° about
  its hinge edge.
- Draw Connor's walk model with frame transforms ignored: he walks on the
  spot without lurching.
