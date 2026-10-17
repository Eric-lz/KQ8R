# Buildings (`KQInterior`, `.dig` / `.dil`)

Study session A, 2026-10-11. Answers `QUESTIONS.md` Q11 and the open part
of Q10 (buildings at `locZ` −1). Sources: `Mask.exe` (studied, described in
our own words) and the data files. "Traced" = read in the exe; "measured" =
checked on the data with a throwaway script.

## What the files are

- A building is a `KQInterior` object. It uses three files with the same
  base name as `shapeName`: `<name>.dig` (shape), `<name>.dil` (lighting) and
  the material list named by `material=` (in Daventry always
  `interior.dml`, 52 materials, shared by all buildings). The game's error
  text names exactly these three: "(.dig,.dil,.dml)". (Traced.)
- These are the Darkstar engine's interior files (class names
  `ITRGeometry`, `ITRLighting`, the same family as Starsiege/Tribes). The
  game's palette for them is `KQWorld::interiorPalette` in `world.cs`
  (Daventry: `daventry.ppl`, same as the objects).

## `.dig` layout (`ITRGeometry`), traced and measured

PERS block like the other files: `"PERS"`, u32 block size, u16 name length,
name `ITRGeometry` padded to even length. **The block ends at block size + 8**;
5 of 153 files have unused bytes after that, ignore them. Then, all
little-endian:

| Field | Size | Notes |
| --- | --- | --- |
| version | i32 | 4 in almost all files, 2 in a few |
| build id | u32 | ignore |
| texture scale | f32 | 0.125 in every Daventry file = 1 texel per 8 units |
| bounding box | 6 × f32 | min x,y,z then max x,y,z, in the building's own space |
| 8 counts | 8 × i32 | for the 8 tables below, in this order |
| 8 tables | | in the same order, record sizes 20, 8, 48, 1, 4, 12, 8, 16 bytes |
| extra value | u32 | only if version > 3 (meaning not traced, 0 in data) |
| extra value | u32 | only if version = 4 (meaning not traced, 0 or 1) |

Tables (meaning from the data; every index stays inside its table, checked on
all 153 files):

1. **Surfaces**, 20 bytes each (one flat polygon each):
   - byte 0: flags. **Bit 0x80 set = the surface faces the same way as
     its plane's normal; clear = the opposite way** (planes are shared by
     both sides of a wall). Corrected 2026-10-16 (study session A, Q78):
     the first version said the reverse, probably measured before the
     corner-order fix of Q15 below. Traced: the game's wall drawing shows
     a bit-set surface only when the camera is on its plane normal's side,
     and its point-light test uses the plane normal for set, the reversed
     normal for clear. Data: the lowest horizontal surfaces of Daventry's
     buildings (their bottoms, which face down and outward) all have a
     downward plane normal and the bit set. B's earlier count (bit and
     corner order agree on 19,958 of 19,958 Daventry surfaces) still
     holds; only the direction was misread. B draws from the corner order
     (clockwise seen from the front, step 2 below), which stays right.
   - byte 1: material index into the material list (0–51 in Daventry).
   - bytes 2, 3: texture size in texels minus 1 (width, height).
   - bytes 4, 5: texture offset in texels (x, y).
   - u16 at 6: plane index.
   - u32 at 8: first corner in the corner table.
   - u32 at 12: first entry in some other table, with byte 17 as its count
     (**not** the `.dil` light map: that is found by surface number,
     `object-lighting.md`; meaning not traced).
   - byte 16: number of corners (3–9, mostly 4).
   - bytes 18, 19: always 0.
2. Tables 2–4: **BSP nodes** (8 bytes), **BSP leaves** (48 bytes) and
   **visibility bits** (1 byte). The game uses these to decide which parts
   are visible from where (inside/outside). Not needed to draw the
   building; not decoded.
3. Table 5, **corners**, 4 bytes: u16 point index, u16 texture-coordinate index.
4. Table 6, **points**, 12 bytes: x, y, z (f32), building space.
5. Table 7, **texture coordinates**, 8 bytes: u, v (f32), always 0–1.
6. Table 8, **planes**, 16 bytes: normal x, y, z and d (f32), with
   `normal · point + d = 0` for the surface's points (within 0.5 units).

## Drawing a building (enough for its outside)

1. For each surface, take its corners in order and make a polygon (a fan of
   triangles from the first corner is fine: the polygons are flat and
   convex-looking; not checked for convexity).
2. **Front side: corners run clockwise seen from the visible side, the
   same as `.kqs` models.** (Corrected 2026-10-11, Q15. The first version of
   this note said anticlockwise. That was wrong: it read the building's
   underside and roof as a floor and a ceiling seen from inside. Session B
   measured on all 153 files that 99.7% of surfaces on the bounding box's
   sides face outward with clockwise corners, and the user confirmed the
   outside looks right drawn that way. Not traced in the exe.)
   Hide back faces as for models.
   **The inside is made of separate surfaces**, not the back of the outside
   ones: walls have thickness, so the inner face lies on its own plane.
   Measured: only 536 of 15,662 planes in all `.dig` files carry surfaces
   facing both ways. So drawing every surface with back faces hidden is
   right from both outside and inside.
3. Texture coordinates, measured: texel position =
   `(offset_x + u × (size_x + 1), offset_y + v × (size_y + 1))`. In 93% of
   Daventry surfaces this equals a world-axis position ÷ 8, modulo 256,
   i.e. the texture is laid straight on the building at 8 units per texel and
   repeats. **Likely** (not traced): divide the texel position by the
   material bitmap's width/height and let the texture repeat.
4. Lighting: `.dil` holds the building's pre-made light maps, one per
   surface. Decoded 2026-10-14: see `object-lighting.md` ("Rules:
   buildings", `.dil` layout and rules 12–16).

## Where a building stands (Q10 follow-up)

1. Drawing transform: points × rotation + (locX, locY, locZ), same rule as
   objects (`objects.md`). The game internally moves the object's reference
   point to the centre of the bounding box but takes that back out again for
   drawing, so the net effect is just the rule above. (Traced.)
2. **Every object's mover has `keepOnGround` switched on unless the `.kq`
   says otherwise** (the default value in the mover is "true"). (Traced.)
3. When a mover with `keepOnGround` moves, the game asks for the ground
   height under the object's position; **if the object is below it, it is
   raised to exactly that height; if it is above, nothing happens**.
   (Traced in the move code.) Not traced: that this also runs once when a
   building is added. The data says it does: 20 of the 21 Daventry building
   sections with `locZ` −1 or −2 are over ground 7,650–39,525 units high
   (`zedrock2` sits at x, y = 0, 0), while every building with a real `locZ`
   is within about 35 units of the ground (`Dock3` 10 below, `look` far
   above on purpose). Ground here = rough value from one height-map corner,
   not the exact triangle height.
4. So: **if `locZ` is below the ground under (locX, locY), use the ground
   height there instead.** The buildings' lowest points are about 0 in their
   own space, so they then sit on the ground.
5. This explains Q10's `keepOnGround=1` objects floating up to 6,600 above
   the ground: the rule only raises, it never lowers.
6. **Which ground (added 2026-10-12, Q43):** the mover's default "on" value
   is −1, and −1 means the **floor** grid (`terrh2.dib`, the ground under
   water, where a world has one). A `.kq` with `keepOnGround=1` uses the
   **surface** grid (`terrh.dib`) instead; `keepOnGround=0` (some DeadCity
   and Barren buildings) means no raise at all. A building without the line
   stands on the floor: Swamp's `shack2` (`locZ=-1`) at 5,355, not 6,120.
   Details in `water.md`.
