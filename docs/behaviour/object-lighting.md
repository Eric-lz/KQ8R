# Object and building lighting (models, buildings, brightness, haze)

Study session A, 2026-10-14. Sources: `Mask.exe` (studied, described here in
our own words), the light files in each world's `kq/Light/`, the scripts
`swtohwlt.cs` / `hwtoswlt.cs`, `TERRAIN.KQ`, the objects' `.kq` files, the
`.dil` / `.dig` files and `Options.cs`. Covers the **hardware renderers**
(the Glide path was traced; Direct3D was not checked). Answers Q66 and the
open part of Q65. The ground itself is in `terrain-lighting.md`.

Updated 2026-10-16 (study session A, Q68 and Q69): haze on the ground,
building walls, particles and the background (rules 27–33), what the card
does with the haze (rule 23), the draw distance that switches objects off
(rules 34–40), and the walls' lights (rule 16).

Updated 2026-10-16 (study session A, Q76–Q78): point lights on building
surfaces in full (rule 16), which flag bit makes a model's light a point
light (rule 26b), `KQPointLight` and the near / far / off test (rule 37),
and the draw-distance ball of a building (rule 40).

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
   per-object choice of lights, apart from point lights' distance (rule 24).

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
    object file (the object reader doesn't read it). **Nothing in the game
    turns it on** (2026-10-17): no file key, no script command (`dither`
    changes the drawing style, `lightOnFire` sets group bit 0x1000; neither
    touches it), and a scan of the program for code that stores 1 into it
    found only unrelated objects. Treat it as always off.
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
| version | u32 | 4 in every file; read by the shared block loader before the rest |
| build id | u32 | **equals the `.dig`'s build id** (checked on all 153 `.dig`/`.dil` pairs) |
| light scale shift | u32 | 4 in every file: one light texel per 16 × 16 texture texels |
| unknown | u32 | **0 in every file** (corrected 2026-10-15, Q67 a: this field was missing from the table). Read and kept by the game; no use was found. Ignore it. |
| 5 counts | 5 × u32 | tables A–E below, in this order |
| table A | 16 bytes each | animated lights; **count 0 in every file** |
| table B | 8 bytes each | light states; **0 in every file** |
| table C | 24 bytes each | state data; **0 in every file** |
| table D | 48 bytes each | **one record per `.dig` surface**, same order (count = the `.dig`'s surface count, all files). The game always reads 48 bytes per record (Q67 b). |
| table E | 1 byte each | the light-map texels, packed (byte offsets below point here) |
| byte list | u32 count + that many bytes | **count 0 in every file**, so in practice one closing u32 0 (Q67, traced: a counted list, not a fixed end marker) |

Bytes after the PERS block's end (4 files: DeadCity `platfrmc`,
`skelring`, `tower2`, Temple 4 `level4`) are never read; ignore them, as
for `.dig` files (Q67 d).

A table D byte offset of **0xFFFFFFFF** means "this surface has no light
map". Only Castle D's `castle.dil` has them (96 surfaces), and those are
exactly the 96 surfaces of `castle.dig` whose material number is **255**.
`Castle.dml` has 26 materials, so 255 is not a material: it marks a
surface with no material. No other building uses 255. **Don't draw these
surfaces** (inferred from the data: the drawing code was not traced; they
have neither a texture nor a light map). They don't affect collision,
which uses the BSP, not the surfaces (Q67 c).

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
    those points the light is blended smoothly: **bilinear** (traced
    2026-10-17: the card's second texture unit, which holds the light
    maps, is set to bilinear filtering once at start-up and never changed;
    the same for the ground's light maps).
    **How the drawing places it** (traced 2026-10-17, Q79). The light
    map's texels are copied row by row, unchanged, into the top-left
    corner of a power-of-two picture for the card (no flip, no shift),
    which is drawn **clamped** at its edges. At each corner of the
    surface the game sends the card the light-map position
    ((texture x + offset_x) ÷ 16, (texture y + offset_y) ÷ 16) **plus
    half a light texel**. The half texel is there because the card puts
    a texel's centre half a texel in from its edge (texel 0 spans 0 to
    1, centre 0.5), as modern cards do. Net effect: the **centre** of
    light texel (i, j) lands exactly at texture position (16 × i −
    offset_x, 16 × j − offset_y), the position given above. Not 4
    texture texels further on. (The card's texel-centre convention is
    hardware behaviour, not something read in `Mask.exe`. If the card
    put centres on whole numbers instead, the texels would sit 8 texture
    texels **back**, not 4 forward, so the stored P0 of rule 16 fits
    neither way.) The data agrees (measured on all 164,838 surfaces of
    the 148 files with 48-byte records): with this placement **no
    surface corner ever falls outside the map's first or last texel
    centre**, and 7,511 surfaces have a corner exactly on texel 0's
    centre. The map sizes are the tightest that cover the surface this
    way. With texel i at 16 × i − offset_x + 4 instead, 47,226 surfaces
    would have corners up to a quarter texel before the first texel.
    On the drawing path read here the light map and the texture are
    drawn as two passes, the second multiplying the first. Check: width =
    ⌈(offset_x + size_x + 1) ÷ 16⌉ + 1 and height likewise. That holds for
    every surface of 148 of 153 files. The 5 others (`temple4/level4`,
    `swamp/ramp`, `daventry/zedrock1`, `zedrock2`, `deadcity/platform`)
    store **12-byte** table D records (B, Q67 b: the block size only fits
    12; read that way, every surface of all 153 files fits this rule).
    **The game never uses them:** it always reads 48-byte records, which
    would run past the end of these files, and **no script in any world
    loads these five buildings** (checked: no `loadKQ` of a `.kq` naming
    these shapes). So nothing decides between 12 and 48 in the original.
    B's reading (pick 12 or 48 from the block size) is fine; skipping the
    five files would be just as right.
14. **Wall colour** = texture × light-map colour (channel ÷ 15), red by
    red, green by green, blue by blue; no doubling (traced: the light maps
    go to the card unchanged as 4-4-4-4 colour, and the card multiplies).
    The top 4 bits (brightness) are not used for colour. This holds for
    every material, including those flagged `0x1000` (traced 2026-10-17:
    the wall drawing never looks at the lighting flags, and it keeps the
    card's colour key off, so pure black on a wall is drawn as black).
15. **Inside and outside are the same rule.** Inner and outer walls are
    separate surfaces (`buildings.md`), each with its own light map.
    Measured on Daventry: many texels are 0 (black), e.g. 16,017 of 81,085
    in `cnnrhut.dil`. Those are probably faces that are never seen.
16. **The sun and ambient never touch building surfaces** (traced
    2026-10-16). A building surface (wall, floor, ceiling, roof: every
    surface of the `.dig` is handled the same way, so inside a house the
    floor counts too) gets its light map and, only when dynamic lighting
    is on, the point lights near it. Dynamic lighting is on in the shipped
    `Options.cs` (`useDynamicLighting 1`). "Point lights" means every
    point light in the frame's light list (rule 3): `KQPointLight`s (Dead
    City, rule 25) **and the lights carried by models** (rule 26b), for
    example Daventry's fireplaces (`fireset2.kqs`, Begin 700, End 850), in
    exactly the same way. Sun-type lights in the list are skipped. So
    Daventry's walls and floors do get dynamic light, from its models'
    lights. Completed 2026-10-16 (Q78), all traced:
    - **When**: every frame, for every building surface that is drawn this
      frame (only the surfaces in view). With dynamic lighting on, the
      game throws away all the wall light maps it made the frame before,
      then rebuilds each drawn surface's map **from the `.dil` texels as
      stored**, then adds the lights. So nothing piles up and a moving
      light (the portal's) leaves no trail. With dynamic lighting off the
      `.dil` map is used as it is.
    - **Where the work is done**: in the building's own axes. Each light's
      position is turned into the building's axes (the inverse of the
      building's placement) once per building per frame.
    - **The surface's light-map rectangle.** Each table D record holds, as
      floats, the place of the surface's **first light texel**, P0 (bytes
      12–23), and two edge vectors U (bytes 24–35) and V (bytes 36–47),
      in building space. Measured: U runs from the first to the last
      texel of a row and V from the first to the last row (|U| = (width −
      1) × 128 and |V| = (height − 1) × 128 in Daventry). The rectangle's
      four corners are P0, P0 + U, P0 + V, P0 + U + V; its centre is the
      average of the four. **P0 is not the drawn texel (0, 0)** (Q79,
      traced and measured 2026-10-17): it lies **a quarter light texel
      (4 texture texels, 32 units in Daventry) further along the
      texture's x and y** than the drawn texel (0, 0) of rule 13, on
      160,649 of 164,838 surfaces to within 0.05 texture texel (the rest
      within about half a texture texel). U and V have exactly the drawn
      spacing, so the whole rectangle is shifted by that quarter texel,
      not shrunk. The game reads P0, U and V as stored and changes
      nothing; it uses them **only** for these point-light tests and
      sums, never for drawing (drawing uses the byte offsets of rule 13).
      The game contains no code that makes `.dil` files, so why the
      files put P0 there is not known: it comes from the tool that built
      them. (A guess, not checked: that tool took its light samples a
      quarter texel in.)
    - **In front**: d = the direction from the light to **P0** (the first
      corner), made unit length. The light counts only if facing · d is
      **below −0.1** (so the light is in front of the surface, more than
      about 6° off its plane). "Facing" is the surface's front direction:
      the plane normal when the surface's byte-0 bit 0x80 is set, the
      reversed plane normal when it is clear (the side the surface is
      drawn from; see the correction in `buildings.md`). Same as "the
      side from which the corners run clockwise".
    - **Switched on**: lights carried by an object with
      `turnOffDynLighting=1` are off for surfaces and the ground (rule
      26b). `KQPointLight`s are always on.
    - **Reach**: of the rectangle's four corners and its centre, take the
      one nearest the light. Its distance must be less than
      **`tsAttenuationBegin` + `tsAttenuationEnd`** (keys matched
      2026-10-17). Only those five points are tried, not the nearest point
      of the surface, so on a large surface a light near the middle of a
      long edge can fail this test.
    - **Line test**: from the light to the point **64 units from that
      nearest spot, towards the light**. Only **this building's own
      collision BSP** is tested (its solid space, the same test as
      `camera.md` "Obstacles" and Connor's collision with buildings).
      Other buildings, the ground and objects never block it. If the line
      passes through this building's solid space, the light adds nothing
      to this surface.
    - **Adding**: then every light texel of the surface gets colour ×
      falloff added. Texel (i, j) (i along a row, j row number) is placed
      at **P0 + U × i ÷ width + V × j ÷ height** in the original. That is a
      slip: by the data the true texel spacing is U ÷ (width − 1), so the
      original measures each texel a little nearer P0 than it is, and
      from P0, a quarter texel off the drawn texel (0, 0) (see "Do not
      reproduce"). d = the texel's distance from the light;
      **falloff = 1 up to `tsAttenuationBegin`, then 1 − (d − Begin) ÷
      End, not below 0**. So on surfaces (and the ground, rule 26)
      **`tsAttenuationEnd` is a fade length**, added on top of Begin: the
      light reaches 0 at Begin + End. On models (rule 24) the same key is
      an end distance. Example `PLIGHT01` (Begin 1,000, End 4,000): walls
      lit out to 5,000, models out to 4,000. Reproduce the difference.
    - **Stored back in 4 bits, like the ground** (rule 26): each colour
      channel = stored value ÷ 15 + light colour × falloff, capped at 1,
      then stored as a whole number, **channel × 15.9 rounded down**. With
      two lights the second adds to the first's rounded result. (The top
      4 bits get a brightness worked out from the three channels, which
      rule 14 does not use.)
    - **Haze**: a surface that passes every test above (including the line
      test) gets its haze multiplied by **0.04**, once per such light (two
      lights: × 0.04 twice), whether or not any texel ends up brighter.
      The test "haze 0.99 or more: not drawn" (rule 29) comes **after**
      this, so a lit surface beyond the haze limit is still drawn. Lit
      walls glow through the fog.
    - **Far or off buildings** (Q78 e): a building that is drawn is lit,
      near or far (rule 37: far still draws with the default masks). An
      off building is not drawn, so there is nothing to light. There is no
      other distance limit for surfaces besides the reach test. What
      decides whether a **light** shines is the light's own object: a
      `KQPointLight` always (rule 37); a model's light only while its
      object has group bit 0x400 (near or far with the default masks).
      Example: Daventry's fireplace objects (`fireset1.kq`,
      `fireset2.kq`) have `tooFar=0.4` and `groupsActive3DFar=0`, so their
      lights shine only while Connor is within about 0.4 × 27,000 =
      10,800 of them (minus their ball's radius).
    - A surface with no light map (table D offset 0xFFFFFFFF, Castle D
      only) gets an all-white map and no lights.

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
    (default 1.0). **Only that exact spelling is read** (traced). No file
    in the game uses it; the 13 Gnome files that write `hazeMulitplier`
    (misspelled) are ignored by the original, so in practice the multiplier
    is **always 1.0**. Read only the correct spelling, as the original does.
23. Solid objects are then drawn **blended towards the haze colour** by
    the haze amount (traced 2026-10-16):
    - **The blend**: drawn colour = (1 − haze) × colour + haze × haze
      colour, done by the card on the final colour (after texture × light),
      in plain screen colour values (gamma 1.0, rule 17). The haze amount
      goes to the card as a number from 0 to 255 (haze × 255). The card's
      fog table is never used.
    - **The haze colour**: `resetHazeColor R G B` in `world.cs`, on a
      **0–255 scale**. The game does not use these values directly: it
      picks the **palette colour nearest** to them (smallest sum of squared
      differences, among palette entries **10–244**; on a tie the lower
      entry wins), and fogs with that palette entry's colour. The palette
      it searches is the one loaded just before (`loadPalette
      $KQWorld::loadingPalette` in `world.cs`).
      **Which entries count (answered 2026-10-17):** a palette carries a
      256-bit "usable" mask that starts as entries 10–245 and could be
      replaced by the `.ppl` file's `pcmt` chunk. In the shipped palettes
      the `head` chunk says 3 chunks follow (`data`, `pspl`, `hzpl`), so the
      loader stops before `pcmt` and **never reads it** (it is all zeros
      anyway). So every entry 10–244 counts. Measured result per world
      (haze colour → entry, and the background entry of rule 31):

      | World | `resetHazeColor` | Haze entry (colour) | Background entry (colour) |
      | --- | --- | --- | --- |
      | Daventry | 4 8 12 | 17 (4, 8, 12) | 17, same |
      | Dead City | 0 0 0 | 10 (0, 0, 0) | 10, same |
      | Barren | 30 0 0 | 14 (30, 0, 0) | 14, same |
      | Swamp | 40 48 24 | 83 (40, 48, 24) | 83, same |
      | Snow exit | 32 88 112 | 229 (32, 88, 112) | 229, same |
      | Gnome | 0 0 0 | 10 (0, 0, 0) | 10, same |
      | Ice World | 90 100 190 | 64 (72, 128, 160) | 65 (72, 136, 160) |
      | Temples 1–4 | 18 18 18 | 11 (16, 16, 16) | 10 (0, 0, 0) |
      | Castle | (commented out) | not set: keeps the entry number left by the previous world, looked up in Castle's palette | 11 (8, 8, 8) |

      So in most worlds fog and background are the same colour, and the
      ground fades seamlessly into the background. For Castle use the
      background entry 11 as the stand-in (what the original shows depends
      on where Connor came from; not worth reproducing).
    - **See-through objects** (`translucency` below 256) get no haze colour
      (the card's fog is switched off for them): their opacity is
      `translucency` ÷ 255, times a fade (1 while haze < 0.5, then
      1.2 − haze).

## Rules: point lights (traced 2026-10-17, Q72)

24. **Point lights on models.** `KQPointLight` (`tsLightType=2`,
    `tsLightPosition.x/y/z`, `tsLightIntensity.red/green/blue`,
    `tsAttenuationBegin`, `tsAttenuationEnd`) adds colour × falloff ×
    facing to each corner's light (rule 4, before the cap). It is worked
    out **per model part** (each mesh of the `.kqs`), every frame:
    - **Out of range**: the part gets nothing from this light when
      (distance from the light to the centre of the part's bounding ball)
      − the ball's radius is **more than `tsAttenuationEnd`**. The ball:
      radius = the radius stored with the mesh in the `.kqs` (the mesh
      `radius` field), centre = the middle of the part's box in the
      current frame (the decoder reads it from the frame's data; we did not
      check how it maps to the fields B already reads, so the middle of the
      part's vertices in that frame is the stand-in).
    - **Facing**: one direction per part, from the light to the ball's
      centre (made unit length). facing = max(0, −(direction · corner
      normal)); a corner facing away gets nothing.
    - **Falloff**, from the corner's own distance d to the light: 1 up to
      `tsAttenuationBegin`; 1 − (d − Begin) ÷ (End − Begin) up to
      `tsAttenuationEnd`; 0 beyond. Here End **is** an end distance (on
      walls and the ground it is a fade length, rules 16 and 26).
    - Flat materials (rule 7) take the result at their first corner as
      usual; "none" materials are not lit at all.
    - Lights are gathered once per frame for the whole scene (rule 3);
      every model part checks every point light.
25. **Data.** Daventry has no point lights. Dead City loads seven at the
    start (`newworld.cs`: `Plight01`, `facelt03`–`facelt08`) and two more
    in `azrieltp.cs` (`facelt01`, `facelt02`); `Plight02` is never loaded.
    Example: `PLIGHT01` at (182,213, 41,595, 2,250), colour 0.3/0.7/0.4,
    Begin 1,000, End 4,000. They load the same on hardware and software
    (`swtohwlt.cs` only swaps the sun and ambient).
    `torchlight.kq` (`KQFlickerLight`) is in every world's `Light` folder,
    but **nothing loads it** (2026-10-17: no script, no object file and no
    text in the program names it). Leave it out.
26. **Point lights on the ground** (dynamic lighting, `terrain-lighting.md`
    rule 15), only when `useDynamicLighting` is 1 (as shipped). Every
    frame:
    - first the patches painted last frame are put back as they were;
    - then for every point light that is switched on: h = the light's
      height above the ground straight below it (light z − ground height
      there). If h is more than `tsAttenuationEnd`, the light paints
      nothing. Otherwise it paints the square of ground-map points around
      the point below it, half-width √(End² − h²);
    - each map point in that square: d = √(dx² + dy² + h²) (dx, dy: the
      point's horizontal offset from the light; h as above, the same for
      every point of the patch). Falloff = 1 up to `tsAttenuationBegin`,
      then 1 − (d − Begin) ÷ End, not below 0 (End as a fade length, as on
      walls). Each colour channel: stored value ÷ 15 + light colour ×
      falloff, capped at 1, then stored back as a whole number
      (channel × 15.9, rounded down) in the same 4-bit format;
    - it paints into the map being shown (the shadow map, rule 2 of
      `terrain-lighting.md`); at most 256 patches and 65,536 points saved
      per frame (more are skipped).
    PLIGHT01 example: h = 2,250 − ground; a patch out to about
    √(4,000² − h²) around the light.
26b. **Lights carried by models** (2026-10-17; file block located the
    same day, after the user's check). An animated object's `.kqs` holds
    its own lights per animation frame. They are the **light nodes B's
    shape reader already reads** (`LightNode`: flags, colour bytes,
    position, attenuation end, attenuation begin, intensity level,
    intensity scale) and each sequence's **light sub-sequence**:
    - **Which lights shine:** those of the object's current frame. The
      frame's sequence names a light sub-sequence (first light node and a
      count); those nodes are the object's lights for that frame (at most
      5 are used). A sequence with no light sub-sequence (−1) has none.
      So the light moves and changes colour as the animation plays.
    - **Kind:** flag bit **0x1** (the lowest bit) set = point light; clear
      = sun-type (its position field is then the aim, rule 5). Confirmed
      2026-10-16 (Q77 a): this is the only bit of the node's flags the
      game ever reads, so **0x4 does nothing** (no other code looks at the
      flags). In the data every node has 0x1, so every model light is a
      point light.
    - **Colour** per channel = byte ÷ 255 × intensity scale × strength,
      strength = intensity level × 0.2, capped at 1. Files of version 1–3
      have no level or scale: the game uses **1.0 for both** (traced; B's
      reader does the same), so strength = 0.2 and each channel is at most
      0.2.
    - **Begin / End** as for `KQPointLight` (rules 16, 24, 26: an end
      distance on models, a fade length on walls and ground).
    - **Position** (traced 2026-10-17): a point light's position is the
      node's position put through the **object's placement only** (its
      place, turn and scale in the world); the frame's own transform is
      **not** applied. A sun-type node's aim is used **as stored, in world
      axes**: it does not turn with the object.
    - They shine while the object is in the lights group (group bit
      0x400, rule 37) and its model is shown.
    - An object whose `.kq` has **`turnOffDynLighting=1`** (39 files, e.g.
      `flame_nd.kq`) keeps its lights for **models** but switches them off
      for **walls and the ground**.
    - **Data:** 442 differently named shapes carry light nodes (measured
      again 2026-10-16; B's `kq8 lighting-verify` agrees). The earlier
      figure 174 counted only the 12 worlds' own `resource` folders;
      `GAME/Common` adds 271 more (3 names are in both). Examples: the
      portal `portlite.kqs`, the torch flame `torch01.kqs`, candles,
      lamps, coals, the tiki, `signpst1.kqs`. Example `portlite.kqs`
      (Dead City's arrival portal, `portal.kq` at 182,186 / 41,007, right
      below `PLIGHT01`): one point light per frame, 17 frames in a 1.07 s
      loop, circling about 400 units round the portal's centre, colour
      cycling from (1.0, 1.0, 0.07) to (0, 1.0, 0.56) as bytes ÷ 255,
      i.e. at strength 0.2 at most (0.2, 0.2, 0.11); Begin 1,500, End
      3,000.

## Rules: haze on the ground, walls, particles and the background (Q68)

All traced 2026-10-16 unless marked. "Distance" is always the straight-line
3D distance from the **camera**. Every kind uses the same card blend and
haze colour as objects (rule 23); only how the haze amount is worked out
differs.

27. **The ground is hazed per corner.** Each ground corner gets its own
    haze from its own distance d:
    - d below `hazeDistance`: 0.01;
    - d from `hazeDistance` up to `visibleDistance`:
      0.01 + (d − `hazeDistance`) ÷ (1.02 × (`visibleDistance` −
      `hazeDistance`)), which rises in a straight line from 0.01 to
      **0.99** at `visibleDistance`;
    - d at or beyond `visibleDistance`: 0.99.
    The card blends the haze smoothly across each triangle between its
    corners. **The ground never reaches full haze** (max 0.99), and it uses
    `visibleDistance` itself as the end, **not** the 0.85 × `visibleDistance`
    the objects use. If `visibleDistance` is not beyond `hazeDistance`, the
    ground uses 0.6 × `visibleDistance` as its haze start instead.
28. **The ground ends at `visibleDistance`.** Corners beyond it are marked
    "past the far edge" (a mark of the ground's own, separate from the
    screen edges and the camera's near and far planes); a triangle whose
    corners are all past it is not drawn. A triangle with some corners past
    it goes through the same cutter as the screen edges, which cuts each
    edge where the mark changes and keeps the inside part (traced
    2026-10-17). **Where the cut falls** (traced the same day): each corner
    keeps its distance d from the camera; along an edge from an inside
    corner (d₁) to an outside one (d₂) the new corner is placed at
    t = (`visibleDistance` − d₁) ÷ (d₂ − d₁) of the way, i.e. where the
    two corners' distances, blended in a straight line, reach
    `visibleDistance` (not the exact sphere; very close for short edges).
    Everything else of the new corner (position, texture place, haze) is
    blended with the same t, so its haze lies between the two corners'
    (at most 0.99). (If d₂ = d₁ the game divides by 0.1 instead; it can't
    happen for an edge that crosses.)
29. **Building walls are hazed per wall surface**, from the distance of the
    surface's **nearest corner** to the camera, with the **objects'**
    numbers: haze = (d − `hazeDistance`) ÷ (end − `hazeDistance`),
    end = 0.85 × `visibleDistance` (rule 19), 0.01 when closer than
    `hazeDistance`, and **0.99** when the formula goes past 1.
    - A wall surface whose haze is **0.99 or more is not drawn**, unless
      the building's `.kq` has `b_hazeSolid=1` (then it is drawn at haze
      0.99, never vanishing). This is how the Daventry tomb and the temple
      rooms named in Q68 stay solid.
    - The material flag `0x10000` and `hazeMultiplier` are **not** used
      for walls.
    - A wall lit by a point light gets haze × 0.04 (rule 16) before the
      "0.99 or more" test, so it can stay drawn past the limit.
    - Daventry: walls start to fade at 18,000 and drop out when their
      nearest corner is past 18,000 + 0.99 × 4,950 ≈ 22,900. Dead City:
      9,000 to about 10,190.
30. **Particles** (leaves, drips, snow, any `KQEmitter`): all particles of
    one emitter share **one** haze, computed like an object's (rules 19–23)
    from the **emitter's own location**, not from each particle. An emitter
    that follows the camera (snow) is at the camera, so its particles get
    no haze. The emitter's `translucency`, `b_hazeSolid` and the particle
    shape's material flags apply as for objects.
31. **The background** is not the haze colour. Each world's `world.cs`
    runs `newSky <n>` (`KQWorld::background`: Daventry 17, Dead City 10,
    Barren 14, Castle 11, Ice World 65, Snow exit 229, Swamp 83, Gnome and
    Temples 1–2 10). This fills the whole screen with **palette entry n**
    of the world's palette, flat, with the card's fog **off**, behind
    everything. Where the ground ends (rule 28) this colour shows. The
    stars and planet lines in `world.cs` are commented out. When in the
    frame the fill is drawn was not traced; drawing it first, behind
    everything, gives the same picture.
32. The brightness factor (rule 17) scales every palette colour, so it also
    scales the haze colour and the background colour.
33. **Summary of the three haze ends** (Daventry / Dead City):

    | Thing | Haze starts | Full haze / gone |
    | --- | --- | --- |
    | Objects (one value each) | 18,000 / 9,000 | 22,950 / 10,200, then not drawn (rule 21) |
    | Building walls (per surface, nearest corner) | 18,000 / 9,000 | ≈ 22,900 / ≈ 10,190, then not drawn |
    | Ground (per corner) | 18,000 / 9,000 | 0.99 at 27,000 / 12,000, never drawn beyond |

## Rules: draw distance, objects switched off far from Connor (Q69)

All traced 2026-10-16. This is separate from the haze: it decides whether
an object takes part in the game at all.

34. **Every frame each object is put in one of three states: near, far or
    off**, by its distance from a reference point. The reference point is
    **Connor** (his position). Scripts can switch it to the camera with
    `setActiveCamera 1` and back with `setActiveCamera 0` (Dead City's
    `death.cs`, `laser2.cs`, `portrap*.cs` do this during cut scenes).
35. **The distance** = distance from the reference point to the centre of
    the object's bounding ball, **minus the ball's radius**. The ball
    (corrected 2026-10-17, Q70): it always comes from a **`BOUNDS.KQ`
    box**, never from the model's own geometry. Centre = middle of the
    box, radius = half the box's diagonal.
    - An object that has a collider when it enters the test (Q70: a box
      collider from `BOUNDS.KQ`) uses **its collision box**, the same box
      and the same lookup as for collision (`connor-physics.md` "Where
      every object's box comes from": for an animated object the name of
      its current movement's shape, then its own name).
    - An object with no collider yet looks up `BOUNDS.KQ` by its
      **`shapeName`** (lower-cased), then by its own name.
    - No entry found, or no shape name (Dead City's `flame.kq`: no
      `shapeName`, and neither `torch01.kqs` nor `flame` is in
      `BOUNDS.KQ`): radius 100, centre at the object's position.
    - Emitters: radius 250 around the emitter's location.
    - **The centre does not turn with the object** (traced): the box's
      middle, in the object's own axes, is added to the object's position
      as if the object were not turned. See "Do not reproduce".
    - The ball is worked out **once**, when the object enters the test
      (normally when it enters the world); it does not follow animations
      or later box changes.
36. **The limit** L = the world's `visibleDistance` (`TERRAIN.KQ`; 30,000
    if there is no terrain). With the distance D from rule 35:
    - D ≥ L: **off**;
    - D < L and D > `tooFar` × L: **far** (`tooFar` from the object's `.kq`,
      default **0.83**; 164 files set it, mostly 0.3–0.6);
    - otherwise: **near**.
    An object switched on with the script command `makeActive <name> 1` is
    always **near**, whatever the distance (`makeActive <name> 0` gives it
    back to the distance test). A hidden object (`objects.md`, Q18) is
    always off.
37. **What the states do** (this makes `objects.md` "Hidden objects"
    rule 3 traced): the object's group bits become `groupsAlways` plus
    `groupsActive3D` when near, `groupsAlways` plus `groupsActive3DFar`
    when far, and only `groupsAlways` when off. The bits that put an object
    in the draw lists are **0x10, 0x40, 0x80 and 0x100** (any of them means
    "drawn"); bit **0x8** switches its collider on. With the default masks
    (`groupsAlways` 6, near 0x448, far 0x440) an object is drawn and solid
    when near, drawn but **not solid** when far, and **neither drawn nor
    solid** when off.
    More bits (traced 2026-10-17): **0x400** puts the object in the
    **lights group**, the list the frame's light gathering walks (rule 3):
    lights carried by its model (rule 26b) shine only while it has 0x400.
    **0x2** keeps it in the near / far / off test itself.
    **`KQPointLight` is different** (corrected 2026-10-16, Q77 b): it is
    not an ordinary object and has no group masks. When it enters the
    world it puts itself straight into the lights group and stays there
    until it is removed. It **never takes part in the near / far / off
    test** (it doesn't read `groupsAlways` or `tooFar`), so it has no
    position or ball for that test, and it **shines everywhere, every
    frame**, however far Connor is. Its only position is
    `tsLightPosition`. 0x1000 is only a flag ("on fire", set by
    `KQObject::lightOnFire`), not a list. The other bits put the object in
    lists not studied here.
38. **Dead City's flames** (`flame.kq`, `flame_nd.kq`): no shape name, so
    radius 100; `groupsActive3DFar = 1089` (0x441) keeps them drawn when
    far. So they disappear when their position is more than
    12,000 + 100 = **12,100 from Connor**. Their haze never removes them
    (rule 20 halves it to at most 0.5, and see-through objects only fade
    from haze 0.5 on, rule 23). The user's estimate was about 11,000; the
    traced value is 12,100. Whether the difference is the estimate or
    something not found is open (open question 9). Their `isActive3D=1`
    line does nothing: the game never reads that key (the key it reads is
    `active3DFlag`, and no file uses it).
39. In Daventry (`visibleDistance` 27,000) objects are switched off
    beyond 27,000 from Connor, but the haze already removes them at
    22,950 from the camera (rule 21), so the switch-off is rarely visible
    there, except for objects with `b_hazeSolid=1`.
40. The same test applies to buildings (`KQInterior`) and emitters: they
    are switched off beyond `visibleDistance` from Connor (minus their
    radius). Buildings are large, so this rarely happens.
    **A building's ball** (traced 2026-10-16, Q76): buildings have their
    own rule and **do not use `BOUNDS.KQ`** at all (even the 6 buildings
    listed there):
    - radius = **half the diagonal of the `.dig` file's bounding box**
      (the 6 floats in its header, `buildings.md`);
    - centre = **the building's own position** as it is now (the test
      reads the position every frame, so the ground raise of
      `buildings.md` is included), **not** the middle of that box: the
      box middle is worked out but not used. (If the building already had
      its collider when it entered the test, the radius comes from the
      collider instead; that is the same box, so the same radius. In
      practice the test is entered before the collider exists.)
    - No `.dig` box (cannot happen with the data): radius 10,001.
    - Measured on Daventry's `.dig` files: radii from about 900
      (`zedrock2`) to 19,000 (`nchurch`); the box middle lies 400–14,000
      units from the building's origin (`csbridge.dig`: 14,000, more than
      its radius of 4,400), so the ball can miss part of the building.
      See "Do not reproduce".
41. **The collider switch (bit 0x8), as B needs it** (traced 2026-10-17):
    when an object's groups gain 0x8, its collider (if it has one) is added
    to the collision world at the object's current place; when they lose
    0x8 it is taken out. While out, nothing collides with it, the camera
    and clicks pass through it, and it does not collide with anything. It
    is the same switch whatever the reason (far, off, hidden, a script
    changing the masks). The groups are recomputed every frame, but
    nothing happens unless the near / far / off state changed.
42. **`setActiveCamera 0|1` and `makeActive <name> 0|1`** (traced):
    - `setActiveCamera <n>` sets the reference point for **all** objects:
      0 = Connor, any other number = the camera. Takes effect at the next
      frame's test. Its usage line is printed if the word count is not 2.
    - `makeActive <name> <n>`: n ≠ 0 makes that object always **near**
      (never far, never off), n = 0 returns it to the distance test. Takes
      effect at the next frame's test. A hidden object stays off. It only
      matters for an object that is in the test (bit 0x2).
    - Data: `setActiveCamera` in 16 Dead City scripts (cut scenes and
      traps; always paired with a `setActiveCamera 0`), `makeActive` in 32
      Dead City switch scripts (`aswitch*`, `cswitch*`).
43. **Items lying in the world** (`KQInventoryItem`, Q71, traced
    2026-10-17). Items are not made from `.kq` files, so no file sets their
    masks or `tooFar`. Fixed values:
    - `tooFar` = **0.83** (the default for every object);
    - near mask **0x448** (drawn, solid, clickable, lights);
    - far mask **0** (!): an item farther from Connor than 0.83 ×
      `visibleDistance` is **not drawn, not clickable, not solid**;
    - `groupsAlways` = **0x2** while the item lies in the world (put there
      by the world's item list, which adds 0x2 so it joins the test), and
      **0** while it sits in an inventory (then it is taken out of the
      test, set to off and removed from the world).
    - Its ball: its collider box from `BOUNDS.KQ` by the item type's shape
      (items always get a collider when their shape is in the table), else
      radius 100 (rule 35). Its see-through value comes from
      `Invinfo.itl`'s `Translucency` column.
    - Daventry: the far band starts at 0.83 × 27,000 = 22,410 from Connor,
      about where the haze removes items anyway (22,950 from the camera).
      Dead City: 9,960 from Connor.

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
| point light falloff, models | 1 to `tsAttenuationBegin`, linear to 0 at `tsAttenuationEnd` | traced |
| point light falloff, walls and ground | 1 to Begin, then 1 − (d − Begin) ÷ End (0 at Begin + End) | traced |
| point light out of range, model part | (distance to part's ball centre − its radius) > End | traced |
| ground patch | only if light height above ground ≤ End; half-width √(End² − h²) | traced |
| haze colour entries searched | 10–244 (`pcmt` never read) | traced, data |
| ground haze per corner | 0.01 + (d − hazeDistance) ÷ (1.02 × (visibleDistance − hazeDistance)), 0.01 to 0.99 | traced |
| wall haze per surface | object formula from the nearest corner, past 1 → 0.99; ≥ 0.99 not drawn unless `b_hazeSolid` | traced |
| wall lit by a point light | haze × 0.04 | traced |
| wall light: in front | facing · unit(P0 − light) < −0.1 | traced |
| wall light: reach | nearest of 4 corners + centre < Begin + End | traced |
| wall light: line test | to 64 units in front of that point, own BSP only | traced |
| wall light map | rebuilt from `.dil` every frame (dynamic lighting on), channel × 15.9 rounded down | traced |
| model light kind | node flag bit 0x1 = point; 0x4 unused | traced |
| `KQPointLight` | always in the light list, not in the near / far / off test | traced |
| building ball | centre = building position, radius = half `.dig` box diagonal | traced |
| card blend | (1 − h) × colour + h × haze colour; h sent as h × 255 | traced (Glide) |
| haze colour | nearest palette entry (10–244) to `resetHazeColor` R G B (0–255) | traced |
| background | flat palette entry `newSky n`, no haze | traced |
| draw distance | off when (distance from Connor to ball centre − radius) ≥ visibleDistance | traced |
| far band | distance > `tooFar` × visibleDistance, `tooFar` default 0.83 | traced |
| ball radius without a `BOUNDS.KQ` box | 100 (emitters 250) | traced |
| items | `tooFar` 0.83, near 0x448, far 0, always 0x2 in the world | traced |

## Data used

- `<world>/kq/Light/SunlgtHW.kq`, `AmblgtHW.kq` (`tsLightAim.*`,
  `tsLightIntensity.*`, `intensity.*`); `swtohwlt.cs`.
- Material flags in each `.kqs`'s material list (`0x0F00`, `0x1000`,
  `0x2000`, `0x10000`).
- `<world>/resource/<shape>.dil`, the building's `.kq` (`shapeName`,
  `althwLightmapName`, `usingAltLighting`). For point lights on surfaces
  also table D's floats P0, U, V (bytes 12–47) and the `.dig` planes,
  surface flag 0x80 and collision BSP; for the building's ball the `.dig`
  header bounding box.
- Objects' `.kqs` light nodes (flag bit 0x1, colour, position, Begin,
  End) and their `.kq` `turnOffDynLighting`, `tooFar`,
  `groupsActive3DFar` (whether their lights shine).
- `TERRAIN.KQ`: `hazeDistance`, `visibleDistance` (`perspectiveDistance`
  is not used by any of the haze or draw-distance rules). `world.cs`:
  `resetHazeColor`, `KQWorld::background` / `newSky`. The world's palette
  (`*.ppl`) for the haze and background colours. Objects' `.kq`:
  `translucency`, `hazeMultiplier` (correct spelling only), `b_hazeSolid`,
  `tooFar`, `groupsAlways`, `groupsActive3D`, `groupsActive3DFar`,
  `shapeName` (for the bounding ball).
- `Options.cs`: `setBrightness`, `useDynamicLighting` (point lights on
  walls).
- Scripts: `setActiveCamera 0|1`, `makeActive <name> 0|1`.
- `GAME/patch/BOUNDS.KQ` (bounding balls). Dead City's
  `kq/Light/PLIGHT01.KQ`, `facelt*.KQ` and `newworld.cs` / `azrieltp.cs`
  (point lights). Objects' `.kq`: `turnOffDynLighting`.
- `GAME/Kq/Invinfo.itl` (items' shapes and translucency).

## Edge cases

- More than one ambient light: their colours add (rule 3).
- Light above 1 in a channel: capped (rule 4). Daventry never gets there
  (max 0.42); Swamp and Barren do (0.3 + 0.6 × 1.198 = 1.02 → 1).
- A `.dil` whose build id differs from its `.dig`: none in the data.
  Recommended: log it and draw the building unlit.
- A building with no `.dil`: draw it unlit (full brightness) and log it.
  Not traced.
- The 5 `.dil` files with 12-byte records (rule 13): never loaded by the game; read them B's way or skip them.
- A surface with light-map offset 0xFFFFFFFF (material 255, Castle D only): don't draw it (table D notes).

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
- The misspelled `hazeMulitplier` in 13 Gnome files: the original ignores
  it (rule 22). Ignore it too; it would make those flames fade twice as
  fast, which the original never showed.
- The bounding ball's centre (rule 35) is added to the object's position
  without turning it with the object. For a turned object the ball sits a
  little off. Turn the centre with the object; the difference is at most
  the object's own size.
- Point lights treat `tsAttenuationEnd` as an end distance on models but
  as a fade length on walls and the ground (rules 16, 24, 26): **do
  reproduce** it; the light values were tuned with it.
- Castle's haze colour depends on the previous world (rule 23): don't
  reproduce; use the background entry.
- **Wall light texels measured in the wrong place** (rule 16): the
  original steps U ÷ width and V ÷ height from P0, but the texels are
  U ÷ (width − 1) apart, so the lit pattern is pulled towards the first
  corner by up to one light texel (128 units in Daventry). On top of
  that it starts from P0, which is a quarter texel (32 units in
  Daventry) further along both texture axes than the drawn texel (0, 0)
  (rule 16, Q79). Use the true, drawn texel positions, the ones rule 13
  uses to lay the map on the surface: texel (i, j) at
  **P0 + U × (i − ¼) ÷ (width − 1) + V × (j − ¼) ÷ (height − 1)**,
  which is "P0 moved back a quarter step along U and V". For a map 1
  texel wide, U has no length, so the step can't come from U: take 4
  texture texels along the texture's x from the `.dig` corners, or leave
  that axis at P0 (at most 32 units in Daventry); the same for height 1
  and V. The other wall tests (P0 **as stored** for "in front", the
  five points for reach) are how the original decides which surfaces
  light up: **do reproduce** those.
- **A building's draw-distance ball is centred on its position, not on
  its box** (rule 40). For buildings whose geometry lies far from their
  origin the ball misses part of them, so they could be switched off
  while part of them is still within `visibleDistance`. Recommended: use
  the middle of the `.dig` box, turned with the building, as the centre
  (the same choice as for objects, rule 35); radius as in the original.
  The difference only shows for a building near the `visibleDistance`
  edge whose walls the haze hasn't already removed.
- `setActivePoint` (a second reference point used when an object is
  beyond the limit from Connor) exists, but the only two uses in the data
  are commented out. Leave it out.

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
- **Light-map placement (B, Q79):** for every surface corner, the light
  texel position ((texture x + offset_x) ÷ 16, (texture y + offset_y) ÷
  16), texture x = u × (size_x + 1), before the clamp. Over the 148 files
  with 48-byte records (164,838 surfaces) expect **0 corners below 0 or
  above width − 1 / height − 1**, and 7,511 surfaces with a corner at
  exactly 0. Also log P0 minus that drawn texel (0, 0), in texture
  texels along the texture's x and y: expect (4, 4) to within 0.05 on
  160,649 surfaces.
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
  6. **Ground fog (Q68).** In the original, the ground should never go
     completely to the haze colour: just before its far edge it is 99%
     hazed, and beyond the edge the background colour (`newSky`) shows.
     In Dead City both are black, so it looks like a dark wall.
  7. **Draw distance (Q69).** Stand near Dead City's torches and walk away
     with the camera close behind Connor: the flames should vanish when
     Connor (not the camera) is about 12,100 from them. Swinging the camera
     round without moving Connor should not bring them back.
- **Our logs (B):** at world load, log the haze colour as the chosen
  palette index and its RGB, and the background palette index and its RGB.
  Every second, log how many objects are near / far / off and how many
  wall surfaces were left out by the haze. Daventry from the start point:
  expect 0 objects off (everything within 27,000 of Connor is near or
  far).
- **More logs (B), 2026-10-17 rules:** at world load, the haze entry and
  background entry per the rule 23 table (Daventry 17 and 17; Ice World
  64 and 65; Temples 11 and 10). In Dead City, per point light: position,
  Begin, End, and how many wall surfaces it reached this frame and how
  many ground points it painted (PLIGHT01: ground patch only if its height
  above the ground is under 4,000). Items: count near / far / off.
- **By eye in the original (user), Dead City:**
  8. **Point lights on walls and ground.** Near `PLIGHT01` (around x
     182,000, y 41,600) the walls and the ground should show a soft
     greenish patch (colour 0.3/0.7/0.4), reaching farther on walls and
     ground (up to 5,000) than on Connor (up to 4,000). With dynamic
     lighting switched off in the video options the patches should go
     away, while Connor still gets the light.
     **Checked by the user (2026-10-17):** yes, a greenish patch on walls
     and ground at the arrival portal, **changing in colour and
     intensity** (the portal's own moving light, rule 26b). With dynamic
     lighting off the moving light goes, and **a smaller green patch
     stays** on floor and walls (the light baked into the light maps: the
     light map is shown instead of the shadow map, `terrain-lighting.md`
     rule 2, and the walls' `.dil` maps). **Connor is lit green** on the
     side facing the light **both with dynamic lighting on and off**, as
     rules 24 and 26b say (models ignore that setting). Our viewer does
     not do this yet.

- **Logs for walls and floors (B), 2026-10-16 rules:** per frame (or every
  second), per point light: how many drawn surfaces it was tested on, how
  many failed "in front", "reach" and the line test, how many were lit,
  and the largest change of one texel channel (in 0–15 steps). Expect
  0 lit surfaces with dynamic lighting switched off.
  For Daventry's pub or Connor's hut with the fire burning: the fireplace
  light (Begin 700, End 850) should light floor and wall surfaces within
  1,550 units of it, and **nothing** once Connor is more than about
  10,800 from the fireplace object (its far mask has no 0x400).
- **By eye in the original (user), 2026-10-16 rules:**
  9. **Fireplace light on floors.** Inside a Daventry house with a
     fireplace, the floor in front of the fire should be brighter than
     with dynamic lighting off, as well as the walls next to it (whether
     the light also changes with the fire's animation frames depends on
     `fireset2.kqs`'s light nodes per frame; not measured). Switching
     dynamic lighting off in the video options should remove that extra
     light from walls and floor; models (e.g. Connor) keep it.
     **Checked by the user (2026-10-16):** the flame casts light with
     dynamic lighting on; with it off that light disappears.
  10. **Lights don't go through a building's own walls.** A surface of
      the same building that faces the fire but has another of that
      building's walls between it and the fire (e.g. the far wall of the
      next room, which faces back towards the fire through the dividing
      wall) should stay dark, if it is within 1,550 units of the fire. The outside
      face of the wall behind the fire stays dark too, but for a simpler
      reason: it faces away from the light.

## Answers to questions

- **Q79** (2026-10-17, where a wall's light texels sit; rules 13, 16):
  (a) the drawing puts the **centre** of light texel (i, j) at texture
  position 16 i − offset_x, 16 j − offset_y, exactly as rule 13 says. The
  game adds half a light texel to the coordinates it sends the card, so
  that the card's texel centres land there. Not + 4. The map sizes in
  the data fit only this placement. (b) P0 is used as stored, only for
  point lights, never for drawing. Its quarter texel comes from the
  files (the tool that built them, not in the game); why is not known.
  (c) Paint at rule 13's positions (P0 moved back a quarter step along U
  and V; B's current choice); keep P0 as stored for "in front" and
  reach. **Nothing changes in how B draws the light maps**: B's light
  coordinates (`(position + map offset) ÷ 16 + half a texel`) already
  match the original. Check: "How to check", light-map placement.
- **Q78** (2026-10-16, point lights on building surfaces; rule 16): (a)
  "in front" is measured from the light to the surface's light-map first
  corner P0 (stored in the `.dil` record), against the surface's front
  direction; (b) the line goes from the light to 64 units in front of the
  nearest of the four corners and centre, and only this building's own
  collision BSP can block it (not other buildings, the ground or
  objects); (c) yes, put back: with dynamic lighting on every drawn
  surface's map is rebuilt from the `.dil` every frame before the lights
  are added; (d) stored back in 4 bits, channel × 15.9 rounded down, as
  on the ground; (e) a drawn building is lit whether near or far; off
  buildings aren't drawn. Lights carried by models (fireplaces) reach
  walls and floors exactly like `KQPointLight`s, unless their object has
  `turnOffDynLighting=1` or lacks group bit 0x400. Floors inside houses
  are building surfaces and are lit the same way. New: texel positions
  slip (Do not reproduce), lit surfaces stay drawn past the haze limit,
  `buildings.md`'s bit 0x80 line corrected.
- **Q77** (2026-10-16): (a) bit 0x1 = point light; 0x4 is never read
  (rule 26b). (b) a `KQPointLight` is not in the near / far / off test at
  all and always shines (rule 37); B's stand-in (light position, radius
  100) would switch Dead City's lights off when Connor is far, which the
  original never does. (c) 442 is right; 174 counted only the 12 world
  folders, without `GAME/Common` (rule 26b).
- **Q76** (2026-10-16): a building's ball comes from its `.dig` bounding
  box: radius = half the box diagonal, centre = the building's position
  (not the box middle); `BOUNDS.KQ` is not used for buildings (rule 40).

- **Q70** (2026-10-17): an object with a box collider uses **the ball
  round its collision box** (`BOUNDS.KQ`): centre = box middle, radius =
  half the box diagonal. Objects without a collider use the `BOUNDS.KQ`
  box found by their `shapeName` (then their own name), so in both cases
  it is a `BOUNDS.KQ` box, never the model's own geometry; no box: radius
  100. The centre is **not** turned with the object in the original (rule
  35); turning it is our choice (Do not reproduce).
- **Q71** (items): yes, items in the world take part in the near / far /
  off test with fixed values: `tooFar` 0.83, near 0x448, **far 0** (an
  item in the far band is not drawn, not clickable), `groupsAlways` 0x2
  while in the world and 0 in an inventory (rule 43).
- **Q72** (point lights): keys matched; models fade from Begin to End,
  walls and the ground from Begin to Begin + End; out of range per model
  part by its bounding ball against End; ground patches per frame; lit
  walls' haze × 0.04 per light (rules 16, 24–26). New: lights carried by
  models and `turnOffDynLighting` (rule 26b).
- **Q73** (remaining points): usable palette entries 10–244 (the `pcmt`
  chunk is never read), with a measured table per world (rule 23); ground
  cut partly traced (rule 28, open question 10); `0x1000` building
  materials still use their light map (rule 14); the "unlit" switch is
  never turned on (rule 10); `torchlight.kq` is never loaded (rule 25).
- **Follow-up (2026-10-17, after the commit):** the light-map filter is
  bilinear (rule 13), walls and ground never use the colour key (open
  question 7), the ground's far-edge cut point (rule 28), model lights
  are placed by the object's placement only, sun-type aims in world axes
  (rule 26b). Left open: Direct3D (6) and the flames' 11,000 vs 12,100
  (9), both only checkable in the running game.
- **Q74** (collider switch, `makeActive`, `setActiveCamera`): rules 41–42.

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
- **Q68 (a) buildings:** yes, hazed, **per wall surface** from its nearest
  corner, with the objects' numbers; a surface at haze 0.99 or more is left
  out unless the building has `b_hazeSolid=1` (rule 29).
- **Q68 (b) ground:** hazed **per corner**, linear from 0.01 at
  `hazeDistance` to 0.99 at `visibleDistance`, blended across each
  triangle; never fully hazed; not drawn beyond `visibleDistance`
  (rules 27–28).
- **Q68 (c) particles:** one haze per emitter, from the emitter's location
  (rule 30).
- **Q68 (d) `hazeMulitplier`:** not read; only `hazeMultiplier` is, and no
  file uses it (rule 22).
- **Q68 (e) blend and scale:** plain linear blend by the card on the final
  colour, gamma 1.0; `resetHazeColor` is 0–255, snapped to the nearest
  palette colour; the card's fog table is not used (rule 23).
- **Q68, what is behind the ground:** a flat fill with palette entry
  `newSky n`, not the haze colour, with no haze on it (rule 31).
- **Q69 (a)–(c):** objects are switched off when (distance from Connor to
  their bounding ball) − radius ≥ `visibleDistance`; between `tooFar` ×
  `visibleDistance` and `visibleDistance` they use `groupsActive3DFar`.
  Measured from **Connor**, not the camera (unless a script says
  `setActiveCamera 1`). `flame.kq`'s `groupsActive3DFar = 1089` keeps it
  drawn when far; it is switched off at about 12,100 from Connor
  (rules 34–40). There is no separate camera far-plane cut for objects in
  the code that was read.

## Open questions

1. (Answered: walls get their light map plus point lights, never the sun
   or ambient, rule 16; light maps are filtered bilinear, rule 13,
   2026-10-17.)
2. (Answered 2026-10-17: nothing turns it on; rule 10.)
3. (Answered 2026-10-15, Q67: 12-byte records, in files the game never
   loads; rule 13.)
4. (Answered 2026-10-17, Q72: per model part, by its bounding ball
   against `tsAttenuationEnd`, rule 24; the ground, rule 26.)
5. (Answered 2026-10-17: nothing loads `torchlight.kq`, rule 25.)
6. Direct3D renderer not checked (Glide only).
7. (Answered 2026-10-17, rule 14: building walls always use their light
   map; the wall drawing never looks at a material's lighting flags, so
   `0x1000` does not make a wall full bright.) **Also answered
   2026-10-17:** pure black is **not** see-through on walls: the wall
   drawing switches the card's colour key off for every surface, whatever
   the material (the ground does the same). Draw walls' black texels as
   black.
8. (Answered 2026-10-17: entries 10–244, the `pcmt` chunk is never read;
   rule 23.)
9. The user saw Dead City's flames vanish at about 11,000 from Connor; the
   traced rule gives 12,100 (rule 38). Probably the estimate; a log of
   the distance at which our flames switch off, compared side by side with
   the original, would settle it.
10. (Answered 2026-10-17: the cut point, rule 28.)
11. (Answered 2026-10-17: the `.kqs` light nodes and light sub-sequences;
    placed by the object's placement only, rule 26b.)
