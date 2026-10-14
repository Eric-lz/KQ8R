# Terrain lighting (light and shadow maps on the ground)

Study session A, 2026-10-13. Sources: each world's `kq/Terrain/TERRAIN.KQ`,
the map files in `<world>/resource/`, `Options.cs`, the height map
(`terrh.dib`), and `Mask.exe` (studied, described here in our own words).
Covers the ground only. Models, buildings, brightness and haze are in
`object-lighting.md` (2026-10-14); moving lights are only partly studied.

## Purpose

The outdoor ground is not lit while the game runs. Each world ships
pre-made **light maps**: a grid of brightness values laid over the ground,
with the hills' shadows already baked in. When a ground square is drawn,
its texture is multiplied by the light map over that square.

## Rules

### Which file is used

1. `TERRAIN.KQ` names four map files (all in the world's `resource`
   folder): `lightmapName`, `shadowmapName` (used with the hardware
   renderers, Glide and Direct3D), `SWlightmapName`, `SWshadowmapName`
   (used with the software renderer). With hardware, only the first two
   are loaded; with software, only the `SW` pair.
2. Of each pair, **one is shown at a time**, chosen by the console command
   `useDynamicLighting <0|1>` (the video options menu shows it as an
   on/off item):
   - `useDynamicLighting 1` → the **shadow map** (`shadowmapName`);
   - `useDynamicLighting 0` → the **light map** (`lightmapName`).

   This install's `Options.cs` has `useDynamicLighting 1`, so the GOG game
   shows `daveshdw.bin` in Daventry. The game starts with the value 0
   until `Options.cs` runs. A change takes effect at the next update.
3. In Daventry the two hardware files differ in only 761 of 263,169
   points. In 757 of them red is the same (only the brightness, green or
   blue channel differs); 2 differ by 1 in red and 2 by 3 (largest step in
   any channel: 3). The picture is the same either way. (Corrected
   2026-10-14 from B's measurement, Q65; the first version said "each by
   one step".)
   (Dynamic lighting also lets moving lights brighten the ground, rule 15.)
4. **No file named** (Barren, Gnome, Temples 1–4 have empty names) or a
   file that can't be read: the map stays at its starting value, **full
   brightness everywhere** (every channel 15).

### The file format (data, confirmed by the loader)

5. A file is: a 32-bit little-endian **count**, then **count** 16-bit
   little-endian values. The count must equal side × side (below);
   otherwise the file is ignored and the map keeps its previous contents.
6. **Side** = 128 × 2^`lightScale` + 1, where 128 is the number of ground
   squares along each side of the world. The map has one point at every
   square corner and `2^lightScale − 1` more points between corners:

   | World | `lightScale` | Points per side | Spacing (`groundScale` 12) | File size |
   | --- | --- | --- | --- | --- |
   | Daventry, Dead City | 2 | 513 | 1,024 units (Dead City: 512, `groundScale` 11) | 526,342 bytes |
   | Swamp | 1 | 257 | 2,048 | 132,102 |
   | Castle, Ice World, Snow exit | 0 | 129 | 4,096 (Snow exit: 2,048) | 33,286 |

   Spacing = (2^`groundScale`) / 2^`lightScale` world units.
7. **Order**: row by row, each row along **+x** (point `i` at
   x = i × spacing), rows going along **+y** from y = 0 (row `j` at
   y = j × spacing). This is the same way round as the height map
   (`terrain.md` rule 3). Value number `j × side + i` (0-based, after the
   count) is point (i, j). The last row and column (index side − 1) sit
   on the far edge of the world.
8. **Checked:** recomputing the shadows of Daventry's hills from its height
   map with its sun direction (`kq/Light/SUNLIGHT.KQ`, aim 0, 1, −0.66)
   matches the stored shadow points at 97.5% of a random sample, against
   87–89% if the map were flipped or turned. So the order above is right.

### What a value means

9. Each 16-bit value is four 4-bit numbers (0–15):
   - bits 12–15: an overall brightness (the average of the three colours,
     roughly);
   - bits 8–11: **red**; bits 4–7: **green**; bits 0–3: **blue**.

   Daventry's hardware maps are grey (red = green = blue): **7** in sunlit
   ground (236,889 points), **4** in hill shadow (20,127 in the shown
   shadow map; 20,123 in the light map), and a few 6s and 8s. The software maps use **15** (lit) and **9** (shadow) in the same
   places (they agree at 99.998% of points).
10. **How the value lights the ground (hardware):** the ground texture is
    multiplied by the light map's colour, red by red, green by green, blue
    by blue (colour channel ÷ 15). The top 4 bits are not used for this,
    as far as traced. **No doubling (×1), confirmed 2026-10-14 (Q66):** the
    map goes to the card unchanged as 4-bit-per-channel colour and the
    card multiplies it with the texture; nothing doubles it. Sunlit
    Daventry ground is drawn at 7/15 ≈ 47% of the texture's brightness,
    shadow at 4/15 ≈ 27%. The user's check by eye agrees (the original is
    dark and muddy). The screen brightness setting does not change this at
    the shipped value (`setBrightness 50` is a factor of exactly 1.0,
    `object-lighting.md` rule 17). Daventry's maps are darker than other
    worlds' by the designers' choice, not by an engine rule
    (`object-lighting.md`, "Answers", Q65).
11. **Between points** the value is spread smoothly. Each ground square
    gets its own small light texture of (2^`lightScale` + 1) points per
    side, corner to corner, drawn over the square together with its
    ground texture (the game turns that small texture to match the
    square's texture rotation, so it always lines up with the world).
    Recommended: look the map up in **world coordinates** and blend the
    four surrounding points (bilinear); that is the same thing without
    the per-square turning. (The exact filtering in the Glide renderer was
    not traced; bilinear is assumed.)

### Other things in this area

12. `lightScale` only sets the map's resolution (rule 6); it doesn't
    change the brightness.
13. The maps were made with a developer tool built into the game
    (console `lightTerrain <0|1> <file>`): for each map point, the
    ambient light plus each sun-type light that a ray from the ground
    point can reach without hitting the terrain, capped at 1, then cut to
    4 bits; shadows of buildings and objects are added by the same tool.
    **We don't need it:** use the shipped files.
14. `nDeactivatedLights` / `fType`, `fRed` … (lights switched off in the
    map) and `nChangedHeights` (height edits) are read from `TERRAIN.KQ`;
    no world uses them. Not studied further.
15. **Dynamic lighting** (`useDynamicLighting 1`): each frame, moving
    "point" lights (torches, fires) that are close enough above the
    ground paint a soft round patch of extra light into the shown map,
    and the previous frame's patches are put back first. Belongs to the
    moving-lights topic (not studied in detail).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| map side | 128 × 2^`lightScale` + 1 points | traced, data |
| point spacing | 2^`groundScale` / 2^`lightScale` units (Daventry 1,024) | traced |
| file | u32 count, then count × u16 | traced, data |
| value layout | brightness ¦ red ¦ green ¦ blue, 4 bits each | traced |
| Daventry hardware values | 7 lit, 4 shadow (grey) | data |
| Daventry software values | 15 lit, 9 shadow | data |
| no file | all 15 (full brightness) | traced |
| shown map | shadow map if `useDynamicLighting 1` (this install), else light map | traced, `Options.cs` |

## Data used

- `<world>/kq/Terrain/TERRAIN.KQ`: `lightScale`, `groundScale`,
  `lightmapName`, `shadowmapName`, `SWlightmapName`, `SWshadowmapName`.
- The `.bin` maps in `<world>/resource/` (Daventry: `davelght.bin`,
  `daveshdw.bin`, `dswlght.bin`, `dswshdw.bin`).
- `MASK/Options.cs`: `useDynamicLighting`.

## Edge cases

- A count that doesn't match side × side: the file is skipped (map stays
  full bright, or whatever it held).
- Dead City and Snow exit have `groundScale` 11: same point counts, half
  the spacing.
- Values above 7 exist in the hardware maps (8 in Daventry). With ×1
  (rule 10) no value can pass 100%.

## Do not reproduce

- Nothing found. The per-square turned light textures are a detail of the
  1998 texture cache; sampling in world coordinates gives the same result.

## How to check

- **In our logs:** at load, log each map file used, its count, side,
  spacing, and a histogram of the red channel (Daventry shadow map:
  4 ×20,127, 6 ×4,598, 7 ×236,889, 8 ×1,555). Log the light value under
  Connor every second as he walks.
- **By eye in the original (user):** in Daventry, find a hill with a
  shadow on the grass behind it (the shadows lie on the +y side of hills,
  away from the sun). The shadowed grass should look about half as bright
  as the sunlit grass next to it. Our viewer at "×1" should match the
  original (done by the user on 2026-10-14: ×1 matches, rule 10).
- **Data check:** the 97.5% shadow agreement in rule 8 can be repeated
  from the height map and `SUNLIGHT.KQ`.

## Open questions

1. ~~Doubling or not on hardware (rule 10).~~ Answered 2026-10-14: ×1
   (Q66). The first version suggested ×2; that was wrong.
2. The exact filtering between points (rule 11). The Glide start-up sets
   bilinear filtering; not traced for the light map's texture unit.
3. ~~What the screen brightness setting does.~~ Answered in
   `object-lighting.md` rule 17: at 50, nothing.
4. Dynamic lights on the ground (rule 15): belongs to moving lights.
