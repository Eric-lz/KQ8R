# Water: the floor image, wading, and which ground counts

Study session A, 2026-10-12. Sources: `Mask.exe` (studied, described here in
our own words) and the game data. "Traced" = read in the exe; "measured" =
computed from the data with a throwaway script (not in the repo). Answers
`QUESTIONS.md` Q43. Builds on `terrain.md` (the ground grid) and
`connor-physics.md` (the mover's passes).

## Purpose

Some worlds have water you can walk into. The game does this with **two
ground grids**: the one you see (the water surface where there is water)
and a second "floor" grid underneath (the bottom of the water). Connor walks
on the floor and wades when the surface is well above his feet.

## Rules

### Two ground grids (traced)

1. Every world has the **surface grid**, built from `heightBMP`
   (`terrh.dib`). It is the ground that is drawn.
2. A world whose `TERRAIN.KQ` has a `floorBMPName` line (not commented out)
   also gets a **floor grid** from that image (`terrh2.dib`). Worlds with
   one: **Daventry, Swamp, IceWorld, DeadCity**. Gnome and the temples have
   the line commented out; Barren, CastleD and SnowExit have no line (CastleD
   and SnowExit ship a `terrh2.dib`, but it is never read).
3. Both grids are built the same way: same size, same square size, same
   triangle split (`terrain.md`), and **height = pixel value ×
   `heightBMPScale`**, a whole number (now traced in the code, which settles
   `terrain.md` open question 1 exactly: no offset). Each grid square
   carries the same ground material from `terrm.dib`.
4. When a world has **no floor grid**, every "floor" question below is
   answered by the surface grid instead.
5. Whether the floor grid is drawn: **not traced, not sure.** The drawn
   ground is given a set-up step that the floor grid is not given, so most
   likely the floor is only used for standing and collision, never drawn.
   (The engine's own name for it is "GridFloor".)

### Which grid answers (traced)

Every height question to the terrain says which grid it wants: "floor" or
"surface".

| Who asks | Grid used |
| --- | --- |
| A mover's terrain test in the up and down passes (`connor-physics.md` "The terrain in the passes"): the hit **height** | **floor**, always |
| The same test: the **normal and material** returned with the hit | floor if the mover's `keepOnGround` is −1, otherwise surface |
| The `keepOnGround` raise ("if below the ground, lift onto it", `buildings.md` rule 3) | `keepOnGround` = **−1: floor**; = **1: surface**; = 0: no raise |
| A `loadKQ` line with z exactly **−1** | **floor** height at (x, y) |
| A `loadKQ` line with z exactly **−2** | **surface** height at (x, y) |
| The camera keeping above the ground (`camera.md`) | floor |
| The water test (below) | surface |

- **`keepOnGround` values.** The mover's built-in default is **−1** (that is
  the "on" value `buildings.md` and `objects.md` talk about). A `.kq` line
  `keepOnGround=1` sets 1, `=0` sets 0. Connor's `.kq` doesn't set it, so he
  has −1: **he stands on the floor grid**. Many monsters and people set
  `keepOnGround=1` (e.g. Daventry `henchman`, `knight`, `bat`; most of
  Barren's): they are lifted onto the **surface**, so they stay on top of
  water.
- A building or object whose `.kq` has `locZ=-1` and no `keepOnGround`
  line is raised by the default (−1) onto the **floor**. Swamp's `shack2`
  therefore stands at `terrh2` = **5,355**, not 6,120 (Q43).

### Water depth (measured)

Where the ground material is a liquid (next section), the surface is above
the floor; elsewhere they are almost always equal.

| World | Liquid squares | Surface − floor there | Other squares where they differ |
| --- | --- | --- | --- |
| Daventry | 201 water (4 of them also swamp) | 510–1,275, mostly **1,020** | 4 (surface 255 higher) |
| Swamp | 676 water, 153 swamp | up to 2,040, mostly **765**; 180 liquid squares have no depth at all | 347 surface higher (up to 6,885: under buildings such as the shack), 9 floor higher |
| IceWorld | 272 water (all also "ice") | 765–1,275 | 7 |
| DeadCity | 16 water, 120 poison, 423 lava | 288–864, mostly **864** | **1,086 where the floor is higher** (up to 2,784), 20 surface higher |

"Liquid squares with no depth" (Swamp): surface = floor, so nobody ever
counts as standing in it (see the water test).

### Ground flags from `envInfo.kq` (traced)

Each world's `kq/envInfo.kq` (`[EnvInfo]`, `nTextures`) has one line per
ground material, `textureN=`, a row from a spreadsheet: columns separated
by tabs, a column is "on" when it holds a letter or digit (the files use
`x`). The column names are in the comment line `;textureX=`:

| Column | Flag | Used for (status) |
| --- | --- | --- |
| `inUse` | none (skipped) | |
| `dirt` | 0x1 | footstep sound set (not studied) |
| `grass` (IceWorld: `snow`) | 0x2 | footstep sounds |
| `rock` | 0x4 | footstep sounds |
| `water` | 0x8 | **liquid** |
| `weird1`/`unused1` | 0x10 | footstep sounds |
| `weird2`/`unused2` | 0x20 | footstep sounds |
| `poison` | 0x40 | **liquid**; damage (`KQWorld::PoisonTime`) |
| `fastpoison` / `lava` | 0x80 | **liquid**; damage (`KQWorld::FastPoisonTime`) |
| `swamp` | 0x100 | **liquid**; damage (`KQWorld::SwampTime`) |
| `fire` | 0x200 | damage kind "fire" |
| `ice` | 0x400 | damage kind "ice" |
| `ontop` | 0x800 | hazards you sink into slowly (see Edge cases) |

How to read a line exactly as the game does: take the text after `=`,
strip spaces and tabs from both ends, skip its first character (the
`inUse` mark), then walk the rest: each tab moves to the next column, each
letter or digit turns the current column on. Only 16 columns fit.
(Stripping matters: rows whose last mark is in the `water` column end in
`x`, and that `x` counts.) Column names differ between worlds, but the
**position** decides the flag. **Liquid = water, poison, fastpoison/lava or
swamp** (0x1C8).

The flags of a position are the flags of the material of the grid square it
is in (`terrm.dib` value → `textureN`).

### The water test (traced)

After **every move** of an object (Connor included), the game looks at
where it ended:

- `S` = **surface** height at its (x, y);
- `z` = its height (Connor: his feet);
- liquid = the square's flags include a liquid (above).

Then:

1. **`submerged` = liquid and S ≥ z + 600.** This is stored on the mover
   (it is the same setting a `.kq` can give with `submerged=`, but the
   test overwrites it after every move, wherever the object is).
2. "Deep" = liquid and S ≥ z + **250**: only used for damage (Edge cases).

Nothing else happens in plain water: **no slowing, no floating, gravity
and ground as usual** (he stands on the floor grid). The water only
changes which animation plays.

### Wading animations (traced)

1. The movement choice (`connor-movement.md` "Which movement plays") uses
   `submerged`: forward with the run switch on → **`waterrun`** instead of
   `run`; forward walking → **`waterwalk`** instead of `walk`. `back`,
   `left`, `right` and standing are the same in and out of water.
2. While Connor is moving, the choice is **redone every update**, and a
   new movement is asked for only when the chosen name changes. So walking
   into deep water with forward held switches to `waterwalk` by itself, and
   back to `walk` when he comes out, with the usual transition rules. The
   transitions are in `GAME/Kq/Conner.anm` (e.g. `walk` → `waterwalk` via
   `walkwaterwalk` from phase 0.2; `run` → `waterrun` via `runwaterrun` at
   once; `waterwalk` → `walk` via `waterwalkwalk` from 0.958; `waterrun` →
   `run` via `waterrunrun` from 0.947).
3. The test runs after the move, so the choice in an update uses
   `submerged` from the end of the previous update's move.
4. Speed in water comes from the water animations' own steps, as for
   every movement (no separate speed setting found).
5. Running head-on into a wall in `waterrun` or `runwaterrun` crashes, as
   `run` does (`connor-physics.md` "Walls").

## Numbers

| Value | Meaning |
| --- | --- |
| 600 | Surface at least this far above the feet → `submerged` (wading animations) |
| 250 | Surface at least this far above the feet → "deep" (damage only) |
| 100 | `ontop` hazards: surface no more than this far *below* the feet |
| 15 | `ontop` hazards: sinking per move |
| Connor 1,300 tall | So in Daventry's 1,020-deep water he wades with the water at about his chest |

## Data used

- `<world>/kq/Terrain/TERRAIN.KQ`: `heightBMP`, `floorBMPName`,
  `heightBMPScale`.
- `<world>/resource/terrh.dib` (surface), `terrh2.dib` (floor),
  `terrm.dib` (materials).
- `<world>/kq/envInfo.kq`: ground flags per material.
- `.kq` `keepOnGround`, `submerged`; `GAME/Kq/Conner.anm` water movements.

## Edge cases

- **Floor above the surface** (Q43 d; DeadCity 1,086 squares, up to
  2,784; Swamp 9 + 17): Connor stands on the floor, so above the drawn
  ground there. Not checked in the original; these may be under buildings
  or platforms. Worth a look in our viewer (How to check).
- **Damage (not for this step, only outlined):** after each move, on a
  poison / fastpoison / swamp square, when that hazard's timer says so
  (`KQWorld::PoisonTime` etc.) and the object is "deep" (or has a built-in
  "always" flag), the game sends a damage event of kind poison, fire or
  ice. Not studied further.
- **`ontop` hazards (not for this step):** on an `ontop` square, on a
  damage tick, with the surface no more than 100 below Connor's feet: his
  `keepOnGround` and gravity are switched **off**; while the surface is less
  than 250 above his feet he is moved down **15** more (one extra move).
  So he sinks slowly into, e.g., Barren's lava (603 squares: lava + fire +
  ontop). What switches them back on is **not traced**.

## Do not reproduce

- A mover with `keepOnGround=1` gets its terrain **hit height** from the
  floor but its **normal and material** from the surface. Where the two
  differ (water) that mixes two grids. Harmless for Connor (−1). For
  monsters later, use one grid (the surface) for both.

## How to check

- **Swamp shack** (Q43): place `shack2` at the floor height (5,355); the
  stove, tables, chests and barricades should then sit 18–70 above its
  floors, and Connor dropped in should land about 765 lower than now.
- **Daventry river:** walk Connor into the river. Expected in `connor.log`:
  his feet go down to the floor grid (about 1,020 below the drawn water
  surface), `submerged` turns on when the surface is ≥ 600 above his feet,
  the next update asks for `waterwalk` (through `walkwaterwalk`), and on
  the way out back to `walk`. Compare by eye with the original: the water
  should come up to about his chest.
- **DeadCity:** stand Connor on some of the squares where the floor is
  higher than the surface and see what is drawn there.
- In the running original, the memory reader (`re/dbg/mem_fall.py`, study
  session only) can log Connor's height in the river to confirm he stands
  on the floor grid. Not done yet.

## Open questions

1. Is the floor grid ever drawn? (Most likely not; not traced.)
2. What switches `keepOnGround` and gravity back on after an `ontop`
   hazard? (Not traced; not needed for walking in water.)
3. Floor-above-surface squares in DeadCity: not looked at in the original.
4. Nothing here was checked in the running original; it is all traced or
   measured from data.
