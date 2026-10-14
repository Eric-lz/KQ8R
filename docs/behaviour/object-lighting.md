# Object and building lighting (models, buildings, brightness, haze)

Study session A, 2026-10-14. Sources: `Mask.exe` (studied, described here in
our own words), the light files in each world's `kq/Light/`, the scripts
`swtohwlt.cs` / `hwtoswlt.cs`, `TERRAIN.KQ`, the objects' `.kq` files, the
`.dil` / `.dig` files and `Options.cs`. Covers the **hardware renderers**
(the Glide path was traced; Direct3D was not checked). Answers Q66 and the
open part of Q65. The ground itself is in `terrain-lighting.md`.

"Traced" = read in the exe. "Measured" = checked on the data with a
throwaway script. "Not traced" = an assumption, marked as such.

## Purpose

Models (`.kqs`) are lit while the game runs: each corner (or each triangle)
gets a brightness from one ambient light plus a sun, and the texture is
multiplied by it. Buildings (`.dig`) are not lit while the game runs: each
wall carries a small pre-made light map in the building's `.dil` file,
multiplied over its texture like the ground's light map. In both cases the
light **darkens** the texture (0 = black, 1 = the texture as stored) and
nothing doubles it. This is why the original looks dark and muddy.

## Rules: which lights are used

1. Each world has two sets of light files in `kq/Light/`: the software set
   (`SUNLIGHT.KQ`, `AMBLIGHT.KQ`) and the hardware set (`SunlgtHW.kq`,
   `AmblgtHW.kq`). `newWorld.cs` loads neither (its lines are commented out).
   When the world starts (and after the renderer changes) the game runs a
   script of the world's: with a **hardware** renderer `swtohwlt.cs`, with
   the software renderer `hwtoswlt.cs` (traced). `swtohwlt.cs` deletes the
   software lights and loads `light/SunlgtHW.kq` and `light/AmblgtHW.kq`.
   **So on hardware the `*HW.kq` files are the lights.** "Hardware" means any
   renderer whose name is not "Software" (traced).
2. Daventry on hardware: **ambient 0.3** (grey), **sun 0.1** (grey), sun aim
   (0, 1, −0.66). The software set is much brighter (ambient 0.5, sun 0.5).
3. All lights in the world are gathered **once per frame** for the whole
   scene (traced): the ambient is the **sum** of all ambient lights' colours,
   and every sun or point light goes into one shared list. There is no
   per-object choice of lights, apart from point lights' distance (rule 13).

### Light values per world (data, hardware set / software set)

| World | Ambient HW / SW | Sun HW / SW | Sun aim |
| --- | --- | --- | --- |
| Daventry | 0.3 / 0.5 | 0.1 / 0.5 | 0, 1, −0.66 |
| Barren | 0.5 / 0.5 | 0.6 / 0.6 | 0, 1, −0.66 |
| Castle | 0.5 / 0.5 | 0.4 / 0.6 | 0, 1, −0.66 |
| Dead City | 0.37 / 0.55 | 0.15 / 0.2 | 1.57, 1.57, −1.57 |
| Gnome | 0.24 / 0.8 | 0.03 / 0.05 | 0, 0, −1.2 |
| Ice World | 0.4 / 0.4 | 0.2 / 0.2 | 1.57, 1.57, −2.25 |
| Snow exit | 0.3 / 0.6 | 0.6 / 0.6 | 0, 1, −0.66 |
| Swamp | 0.3 / 0.3 | 0.6 / 0.6 | 0, 1, −0.66 |
| Temples 1–3 | 0.28 / 0.8 | 0.01 (T3: 0) / same | 0, 0, −1.2 |
| Temple 4 | 0.3 / 0.8 | 0.325 / 0.13 | −1.57, −1.57, −1.57 |

All are grey (red = green = blue) except where a file says otherwise; read
all three channels anyway.

## Rules: models (`.kqs`)

4. **Light at a corner** = ambient + the sun's part (+ point lights' parts,
   rule 13), then **each colour channel capped to 0–1** (traced). Values
   below 0 become 0.
5. **The sun's part** = sun colour × max(0, −(aim · normal)), where `aim`
   is the `tsLightAim` vector from the file and `normal` the corner's normal
   direction, both in the same space (the game turns the aim into the
   model's own space; same result as turning the normal into world space).
   **The aim is not made unit length** (traced): Daventry's (0, 1, −0.66) is
   1.198 long, so a surface facing straight into the sun gets 1.198 × the
   sun colour. A surface facing straight up gets 0.66 × the sun colour.
   Surfaces facing away get no sun.
6. **No shadows on objects.** The sun reaches every corner that faces it;
   hills and buildings do not shade models. The ground's light/shadow map
   under an object does **not** darken it (not used anywhere in the model
   path that was traced). The engine has an unused hook for a "sun map" per
   light; nothing in the game fills it (searched, not found).
7. **The lighting kind of the material** (`0x0F00` of the material flags,
   `models-and-textures.md`):
   - `0x400` **smooth**: rule 4 at each of the triangle's three corners, with
     each corner's own normal; the colour is blended across the triangle.
     A corner shared by several triangles is lit once per frame.
   - `0x200` **flat**: the whole triangle gets **one grey value**: the light
     of its **first corner** (rule 4, with that corner's normal), turned into
     a single brightness (0.299 × red + 0.587 × green + 0.114 × blue, see
     "Do not reproduce"), capped at 1.
   - `0x100` **none**: full brightness (light 1.0).
   - `0x1000` (colour key) and `0x2000` (texture alpha) force **none**,
     whatever `0x0F00` says (traced; as in `models-and-textures.md`).
8. **Corner normals.** Each stored vertex carries a byte that picks one of
   256 fixed directions built into the program. We do not copy that table.
   Use the normal you compute from the geometry (the average of the
   triangles' normals at that vertex, as B does now). The difference is a
   small error in the angle; not measured.
9. **Texture × light, no doubling** (traced, Glide): the drawn colour is
   texture colour × light (per channel, 0–1). Nothing doubles it.
10. **A per-object "unlit" switch** exists: when it is on, smooth materials
    draw at full brightness. It is off by default and is **not** a key in any
    object file (the object reader doesn't read it). What turns it on was not
    found (open question 2). Treat it as always off.
11. **Haze on models**, see rules 19–23.

### What this gives in Daventry (hardware)

| Surface | Light | Texture shown at |
| --- | --- | --- |
| facing away from the sun | 0.3 | 30% |
| facing straight up | 0.3 + 0.1 × 0.66 = 0.366 | 37% |
| facing the sun squarely (normal = −aim direction) | 0.3 + 0.1 × 1.198 = 0.42 | 42% |
| material "none" (`0x100`, or `0x1000`) | 1.0 | 100% |

So in Daventry objects lit by the sun look about as dark as the ground
(ground: 47% sunlit, 27% shadow, `terrain-lighting.md`), and their sunny
and shady sides differ only a little (30–42%).

## Rules: buildings (`.dig` + `.dil`)

12. **Which `.dil`** (traced). A building (`KQInterior`) on hardware uses
    `<shapeName>.dil` (same base name as the `.dig`). Exception: if its
    `.kq` names `althwLightmapName` **and** the alternate lighting is on
    (`usingAltLighting=1` in the file, or the console command
    `useAlternateLightmap <name> 1`), it uses that file instead. The
    software renderer uses `swLightmapName` / `altswLightmapName` (the
    `*_s.dil` files); ignore those. Daventry names no alternates: every
    building uses its own `.dil`. (Alternates exist only in Gnome and
    Temple 4, switched by `fusebox.cs` and `altar.cs`.)

### `.dil` layout (`ITRLighting`), traced and measured on all 235 files

PERS block as usual (`"PERS"`, u32 size, u16 name length 11, `ITRLighting`
padded to 12 bytes). Then, little-endian:

| Field | Size | Notes |
| --- | --- | --- |
| version | u32 | 4 in every file |
| build id | u32 | **equals the `.dig`'s build id** (checked on all 153 `.dig`/`.dil` pairs) |
| light scale shift | u32 | 4 in every file: one light texel per 16 × 16 texture texels |
| 5 counts | 5 × u32 | tables A–E below, in this order |
| table A | 16 bytes each | animated lights; **count 0 in every file** |
| table B | 8 bytes each | light states; **0 in every file** |
| table C | 24 bytes each | state data; **0 in every file** |
| table D | 48 bytes each | **one record per `.dig` surface**, same order (count = the `.dig`'s surface count, all files) |
| table E | 1 byte each | the light-map texels, packed (byte offsets below point here) |
| (end) | u32 | 0 |

Table D record (only the fields we need; the rest describe the light map's
placement in floats and repeat what the texture fields give):

- u32 at 0: **byte offset** of this surface's light map in table E.
- u32 at 4: 0 in the data.
- byte 8, byte 9: light map **width** and **height** in texels.
- byte 10, byte 11: light map **offset** x, y in texture texels (0–15).
- The texels: width × height 16-bit values from that offset, row by row,
  each **4 bits brightness, red, green, blue** (bits 12–15, 8–11, 4–7,
  0–3), the same packing as the ground maps.

13. **Placing a wall's light map** (traced). Work in the surface's texture
    texels (`buildings.md` "Drawing a building", step 3: position =
    u × (size_x + 1), v × (size_y + 1), measured from the surface's own
    texture origin, before adding the texture offset). Light texel (i, j)
    sits at texture position (16 × i − offset_x, 16 × j − offset_y). Between
    those points the light is blended smoothly (bilinear, the Glide texture
    filter; not traced for this texture unit). Check: width =
    ⌈(offset_x + size_x + 1) ÷ 16⌉ + 1 and height likewise. That holds for
    every surface of 148 of 153 files. The 5 others (`temple4/level4`,
    `swamp/ramp`, `daventry/zedrock1`, `zedrock2`, `deadcity/platform`)
    miss on almost every surface; cause not found. For them, clamp the
    lookup to the map's edge (stand-in).
14. **Wall colour** = texture × light-map colour (channel ÷ 15), red by
    red, green by green, blue by blue; no doubling (traced: the light maps
    go to the card unchanged as 4-4-4-4 colour, and the card multiplies).
    The top 4 bits (brightness) are not used for colour.
15. **Inside and outside are the same rule.** Inner and outer walls are
    separate surfaces (`buildings.md`), each with its own light map.
    Measured on Daventry: many texels are 0 (black), e.g. 16,017 of 81,085
    in `cnnrhut.dil`. Those are probably faces that are never seen.
16. Whether the sun and ambient lights also tint building walls on hardware
    was **not traced** (the wall drawing code was not read). Recommended:
    walls use only their light map.

### Daventry building light values (measured, red channel)

| File | Most common values | Share of texels at 15 |
| --- | --- | --- |
| `cnnrhut.dil` (Connor's hut) | 0, 3, 4, 5, 2 | 3.6% |
| `house1.dil` | 2, 6, 7, 5, 4 | 0.6% |
| `pub.dil` | 0, 6, 3, 7, 5 | 2.1% |
| `castle.dil` | 1, 5, 6, 4, 7 | 4.8% |
| `tomb.dil` | 3, 5, 6, 4, 7 | 0.7% |

So Daventry's walls are mostly drawn at 13–47% of their texture's
brightness, about as dark as the ground. The bright and dark patches are
baked into the map: the designers' tool lit each wall from lights placed
in and around the building, so "lamps" inside a room are just brighter
texels, not lights the game computes. **User's check (2026-10-14):**
inside Connor's hut it is darker than outside, darker in the corners, but
overall reasonably lit. That fits a baked map with brighter areas; whether
our ×1 drawing of the hut matches that impression is to be compared once
B draws it (open question 1).

## Rules: screen brightness, gamma, world settings (Q66 b)

17. **`setBrightness`** (`Options.cs`: `setBrightness 50`) becomes a factor
    **0.3 + 1.4 × value ÷ 100**: 0 → 0.3, **50 → 1.0**, 100 → 1.7 (traced).
    On Glide this factor (a) multiplies every palette colour (each channel
    × factor; if one channel would pass 255, all three are scaled down
    together so the colour keeps its hue) and (b) is handed to the card's
    gamma setting once, when the screen opens. **At the shipped value 50 it
    is 1.0: no change and gamma 1.0 (linear).** Nothing else brightens the
    picture.
18. Nothing in `world.cs` or `envInfo.kq` changes lighting. `envInfo.kq`
    lists ground types (dirt, grass, water, ...) per texture. `world.cs`
    only loads the palette and sets the haze colour (`resetHazeColor`).

## Rules: haze (distance fog)

19. Per object, each frame: **haze = (distance − hazeDistance) ÷
    (end − hazeDistance)**, capped at 1. Distance is from the camera to the
    object's position (one value per object, not per corner). Closer than
    `hazeDistance` it is 0.01 (traced). `hazeDistance` and `visibleDistance`
    come from `TERRAIN.KQ`. **end = 0.85 × `visibleDistance`**, unless that
    is not beyond `hazeDistance`, then end = `visibleDistance`. With no
    terrain the game uses 20,000 and 25,500.
    Daventry: hazeDistance 18,000, visibleDistance 27,000 → end 22,950.
    Dead City: 9,000 and 12,000 → end 10,200.
20. Material flag **`0x10000` halves** the haze for that material (traced).
21. **An object whose haze reaches 1 is not drawn at all**, unless its
    `.kq` has `b_hazeSolid=1` (traced; 2 Daventry objects have it). The
    test is made per material, after the `0x10000` halving.
22. After that test the haze is multiplied by the object's `hazeMultiplier`
    (default 1.0; no Daventry object sets it).
23. Solid objects are then drawn **blended towards the haze colour** by
    the haze amount. Haze colour = `resetHazeColor R G B` in `world.cs`
    (Daventry 4 8 12, almost black; the 0–255 scale is assumed). How the
    card blends was not traced; a plain blend is assumed. **See-through
    objects** (`translucency` below 256) get no haze colour: their opacity
    is `translucency` ÷ 255, times a fade (1 while haze < 0.5, then
    1.2 − haze).

## Rules: point lights and flicker lights (partly traced)

24. `KQPointLight` (`tsLightType=2`, `tsLightPosition`,
    `tsAttenuationBegin`, `tsAttenuationEnd`) adds to models: colour ×
    falloff × facing. **Falloff**: 1 up to `tsAttenuationBegin`, falling in
    a straight line to 0 at `tsAttenuationEnd`, 0 beyond (traced).
    **Facing** uses one direction per object (from the light to the
    object), not per corner (traced). Exactly when a whole object counts as
    out of range was not traced.
25. Data: **Daventry has no point lights.** Only Dead City has any: 10
    `KQPointLight` files (`PLIGHT01`, `Plight02`, `facelt01`–`facelt08`).
    `torchlight.kq` (`KQFlickerLight`, a sun-type light whose colour
    wanders between `baseColor` and `baseColor + colorRange` in `numsteps`
    steps) is in every world's `Light` folder, but no script found loads it.
26. Moving lights painting light onto the ground (`terrain-lighting.md`
    rule 15) were not studied further.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| lights used on hardware | `SunlgtHW.kq`, `AmblgtHW.kq` (via `swtohwlt.cs`) | traced, data |
| Daventry HW ambient / sun | 0.3 / 0.1, aim (0, 1, −0.66), not normalised | data, traced |
| corner light | ambient + Σ colour × max(0, −aim·n), cap 0–1 per channel | traced |
| flat grey | 0.299 R + 0.587 G + 0.114 B, cap 1 | traced (see Do not reproduce) |
| texture × light | ×1, no doubling (models, walls, ground) | traced (Glide) |
| `.dil` light texel | 16 × 16 texture texels (= 128 × 128 units at 8 units per texel) | traced, data |
| brightness factor | 0.3 + 1.4 × setting ÷ 100; 50 → 1.0 | traced |
| haze | (d − hazeDistance) ÷ (0.85 × visibleDistance − hazeDistance), cap 1, min 0.01 | traced |
| point light falloff | 1 to begin, linear to 0 at end | traced |

## Data used

- `<world>/kq/Light/SunlgtHW.kq`, `AmblgtHW.kq` (`tsLightAim.*`,
  `tsLightIntensity.*`, `intensity.*`); `swtohwlt.cs`.
- Material flags in each `.kqs`'s material list (`0x0F00`, `0x1000`,
  `0x2000`, `0x10000`).
- `<world>/resource/<shape>.dil`, the building's `.kq` (`shapeName`,
  `althwLightmapName`, `usingAltLighting`).
- `TERRAIN.KQ`: `hazeDistance`, `visibleDistance`. `world.cs`:
  `resetHazeColor`. Objects' `.kq`: `translucency`, `hazeMultiplier`,
  `b_hazeSolid`.
- `Options.cs`: `setBrightness`.

## Edge cases

- More than one ambient light: their colours add (rule 3).
- Light above 1 in a channel: capped (rule 4). Daventry never gets there
  (max 0.42); Swamp and Barren do (0.3 + 0.6 × 1.198 = 1.02 → 1).
- A `.dil` whose build id differs from its `.dig`: none in the data.
  Recommended: log it and draw the building unlit.
- A building with no `.dil`: draw it unlit (full brightness) and log it.
  Not traced.
- The 5 `.dil` files whose map sizes don't fit rule 13: clamp (stand-in).

## Do not reproduce

- **The flat-shading grey mixes green and blue the wrong way round**: the
  game weights the second channel by 0.114 and the third by 0.587 (the
  usual weights are 0.587 for green and 0.114 for blue). Every light in the
  game is grey, so the result is the same either way. Use the usual
  weights.
- The 256-direction normal table: compute normals from the geometry
  (rule 8).
- The sun's aim not normalised (rule 5): **do reproduce** it. It is how
  the original looks, and the designers chose the light values with it.

## How to check

- **Our logs (B):** at world load, log the light files used and the sum
  of ambient and suns (Daventry: "ambient 0.3, sun 0.1 aim 0,1,−0.66").
  For a few objects near Connor's start (Connor's own model and two or
  three objects in view), log the minimum, mean and maximum corner light
  over their smooth corners: Daventry must stay within 0.30–0.42.
  Log how many materials are flat / smooth / none.
- **Buildings (B):** for each `.dil`, log surface count, map bytes and how
  many surfaces fail the size check of rule 13 (expected: 0 except the 5
  files named). For `cnnrhut.dil`: 1,129 surfaces, 162,170 bytes, red
  histogram starting 0 ×16,017, 3 ×7,775, 4 ×7,344.
- **By eye in the original (user):**
  1. **Objects vs ground. Confirmed by the user (2026-10-14):** objects
     look about as dark as the grass, the sunny side slightly lighter.
  2. **Building walls. Checked by the user (2026-10-14):** inside Connor's
     hut it is darker than outside and darker in the corners, but overall
     reasonably lit (the first version of this note said "mostly dark",
     which overstated it). Next: compare our viewer's hut with the
     original side by side once B draws it.
  3. ~~Full-bright materials.~~ Withdrawn: in Daventry the `0x1000`
     textures (plaster, straw roof) appear only in the buildings' material
     list (`interior.dml`), not in any model, and buildings are lit by
     their light maps, so there was nothing concrete to look at. Whether
     the flag means anything on a building is open question 7.
  4. **Haze. Checked by the user (2026-10-14):** objects fade and vanish
     with distance from the **camera** (not Connor), as rule 19 says. The
     distances can't be measured in the original; our viewer can log the
     camera distance at which a chosen object disappears (expected
     22,950 in Daventry) for a rough side-by-side comparison.
  5. **Brightness setting.** Changing the in-game brightness slider makes
     everything brighter or darker together; at the middle (50) it is the
     shipped look.

## Answers to questions

- **Q66 (a):** ×1 confirmed: on Glide the light map goes to the card
  unchanged as 4-bit-per-channel colour and is multiplied with the texture,
  no doubling anywhere (rules 9, 14). `terrain-lighting.md` rule 10 is
  corrected.
- **Q66 (b):** Nothing brightens the drawn picture at the shipped settings:
  `setBrightness 50` is a factor of exactly 1.0 (rule 17). `world.cs` and
  `envInfo.kq` don't touch lighting (rule 18).
- **Q65 (a), why Daventry's hardware maps are at about half the others'
  scale:** no engine rule scales them per world (traced). It's in the data.
  Daventry's own hardware **light files** are also much dimmer than its
  software ones (ambient 0.3 vs 0.5, sun 0.1 vs 0.5), while Swamp, Barren
  and Ice World use the same values for both. So Daventry was made darker
  on purpose for hardware. Its maps, its object lights and (measured) its
  building light maps all agree. Not everything lines up exactly, though.
  Snow exit has a dim hardware ambient (0.3 vs 0.6) but full-scale maps.
  Recomputing Daventry's maps from its hardware lights gives about 5–6,
  not the stored 7, so the maps were made with somewhat different values.
- **Q65 (b):** both corrections applied to `terrain-lighting.md`.

## Open questions

1. The wall drawing code was not read. It is assumed that walls use only
   their light map, not the sun or ambient (rule 16), and that the filtering
   between light texels is bilinear (rule 13).
2. What turns on the per-object "unlit" switch (rule 10)?
3. Why 5 `.dil` files don't fit the size rule (rule 13).
4. How a point light decides an object is out of range (rule 24), and
   the dynamic ground light (`terrain-lighting.md` rule 15).
5. Whether `torchlight.kq` is loaded by anything (rule 25). No script
   found.
6. Direct3D renderer not checked (Glide only).
7. Building materials with flag `0x1000` (in Daventry's `interior.dml`:
   `11gnd`, `Plstrwl1/2`, `Srwroof`): do they ignore the wall's light map
   (full brightness) or not? Not traced. Stand-in: use the light map.
