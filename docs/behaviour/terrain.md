# Terrain (outdoor ground)

Study session A, 2026-10-11. Sources: game data files (observed, with
throwaway measurements) and `Mask.exe` (studied, described in our own
words). Covers the ground grid, heights and height lookup. Materials,
textures, lighting maps and the "floor" image are only partly covered.
Buildings and interiors (`.dig`/`.dil`) are a separate system, not covered.

## Purpose

The outdoor ground of a world: a height grid that is drawn as the
landscape and that the game asks "how high is the ground here, which way
does it face, and what is it made of?" (used by Connor, monsters, the
camera and object placement).

## Where it is defined (data)

Each world has `<world>/kq/Terrain/TERRAIN.KQ` (inside the world's `kq`
archive, also loose under `data/<World>/KQ/Terrain/`). It is plain text,
class `KQTerrain`. Daventry's:

| Setting | Daventry | Meaning (status) |
| --- | --- | --- |
| `groundScale` | 12 | Grid square size is 2^groundScale: **4,096 units** for 12, **2,048** for 11 (Dead City, Snow exit). Confirmed by measurement for 12 and 11 |
| `heightBMP` | `terrh.dib` | Height image (in `resource`) (confirmed) |
| `heightBMPScale` | 255 (Barren: 138) | Height per image value (measured, see below) |
| `floorBMPName` | `terrh2.dib` | The **floor** grid: the ground under water, which Connor stands on. See `water.md` (traced) |
| `materialBMP` | `terrm.dib` | Ground material per grid square (see "Ground textures") |
| `materialListFile` | `davennew.dml` | Ground materials (a material list, same format as in models) |
| `materialDirection` | `terrain/daventry.bin` | Texture name and orientation per material (see "Ground textures") |
| `visibleDistance` | 27,000 | How far the ground is drawn (meaning by name; not traced) |
| `hazeDistance` | 18,000 | Where haze starts (by name; not traced) |
| `perspectiveDistance` | 18,500 | Not traced (likely where texture perspective correction stops) |
| `smoothing` | 0 | Not traced |
| `lightScale` | 2 | Not traced |
| `detailTableNmin/max` | ranges | Level-of-detail distance bands; not traced |
| `lightmapName`, `shadowmapName`, `SWlightmapName`, `SWshadowmapName` | `.bin` files | Pre-computed light and shadow maps (hardware / software renderer); not studied |
| `envinfoName` | `envInfo.kq` | Environment settings; not studied |

Defaults when a setting is missing: `groundScale` 12, `heightBMPScale` 60,
`materialListFile` `terrain.dml`, `materialBMP` `terrm.dib`, `heightBMP`
`terrh.dib`.

The file may also list "changed heights" regions (`nChangedHeights`,
`h_startxN`, `h_startyN`, `h_endxN`, `h_endyN`) and deactivated lights
(`nDeactivatedLights`, `fTypeN`, `fRedN`...). Not used in Daventry's file;
what they do is open.

## Rules

### The grid

1. The height image is a 129 × 129 8-bit Windows BMP (`terrh.dib`). Each
   pixel is one **grid point**; 129 points make **128 × 128 squares**.
2. Each square is 2^`groundScale` units wide: **4,096** for 12, so the
   ground covers **524,288 × 524,288 units** (≈ 524 m if 1 unit = 1 mm),
   from (0, 0) to (524,288, 524,288); **2,048** for 11 (half that size).
3. Orientation: pixel column `i` is at x = i × 4,096. Pixel row `j`,
   **counting from the bottom row of the image** (the first row stored in
   a normal bottom-up BMP), is at y = j × 4,096. No mirroring.
4. The pixel value (palette index; the palette is a plain grey ramp, so
   index = grey level) gives the height:
   **height = value × `heightBMPScale`** (no offset). Measured by matching
   object positions against the image, for objects that stand on the
   ground: Dead City 48.0 × value (setting 48), Daventry 256.2 × value
   (setting 255), Barren 141.7 × value (setting 138); correlations 0.9998 to
   1.0000. The small excess in Daventry and Barren is within the scatter of
   objects standing slightly above the ground. **Now traced (2026-10-12):**
   the grid is built as value × `heightBMPScale`, stored as a whole number,
   no offset; the floor image (`water.md`) the same way.

### Height between grid points

1. Each square is split into **two triangles**. The split direction
   alternates like a chessboard: for square (i, j), if `(i + j)` is even
   the diagonal runs from the square's (0, 0) corner to its (size, size)
   corner; if odd, from (size, 0) to (0, size). (Confirmed.)
2. The ground inside each triangle is the **flat plane through its three
   corner heights** (no smoothing between triangles). (Confirmed.)
3. The surface normal (which way the ground faces) is the normal of that
   triangle, length 1. It is what slopes are judged by (see
   `connor-movement.md`; slope rules still to be studied).

### Ground textures (answers `QUESTIONS.md` Q8)

Measured from Daventry's data (observed; not traced in the exe):

1. `terrm.dib` is a 128 × 128 8-bit BMP, one pixel per **square**, same
   orientation as the height image (column = X, bottom stored row = Y 0).
2. The pixel value is a **direct index** into both the terrain material list
   (`materialListFile`, e.g. `davennew.dml`) and the direction table
   (`materialDirection`, e.g. `kq/Terrain/DAVENTRY.BIN`). Both give the same
   texture name for every value used.
3. Value 159 (`stone1.bmp`) fills 15,224 of 16,384 squares. That is the
   rock around the playable valley: the other textures sit in one block
   (columns 29–72, rows 40–127) and 225 of Daventry's 245 placed objects
   stand inside it.
4. The direction table is: a 32-bit count (255), then one 28-byte record per
   index: 32-bit index (equal to its position), 16-byte zero-terminated
   texture name, a 32-bit value that is always 0 (meaning unknown), and a
   32-bit **orientation code 0–7**.
5. **One texture covers exactly one square.** It is laid on the square
   according to the orientation code. Write the square's local position as
   `u` from its west edge (0) to its east edge (1) and `v` from its north
   edge (0) to its south edge (1); texture coordinates use the image's own
   layout (column from the left, row from the top). The texel shown at
   (u, v) is taken from the texture at:

   | Code | Texture position used | In words (seen from above, north up) |
   | --- | --- | --- |
   | 0 | (u, v) | as stored: image top at the north edge |
   | 1 | (1−u, v) | mirrored east–west |
   | 2 | (u, 1−v) | mirrored north–south |
   | 3 | (1−u, 1−v) | half turn |
   | 4 | (v, 1−u) | quarter turn clockwise |
   | 5 | (1−v, 1−u) | mirrored across the north-east–south-west diagonal |
   | 6 | (v, u) | mirrored across the north-west–south-east diagonal |
   | 7 | (1−v, u) | quarter turn anticlockwise |

   How this was found: for every possible meaning of the eight codes, a
   throwaway script added up how much neighbouring squares' textures
   differ along their shared edges (2,494 neighbour edges, colours from
   `daventry.ppl`). The table above gives a mismatch of 748; ignoring the
   codes gives 2,652. Twelve random starting guesses all ended at this
   table. For codes 0–5 the next-best meaning is clearly worse; codes 6 and
   7 are rare (160 and 24 edges), so for them the table also relies on the
   eight codes being the eight different turns/mirrors (swapping 6 and 7
   gives 820, clearly worse).
6. The terrain textures are the world's `8bit.vol` images (chunked bitmaps
   with mipmaps, 128 × 128), coloured with the world's
   `KQWorld::terrainPalette`.

### Holes and edges

1. A grid square can be marked as having **no ground** (a hole); then the
   height query fails for points in it. Which data marks a square as a hole
   is open question 3.
2. Outside the grid, or in a hole, the game answers "no ground": height
   −2,147,483,648 and a straight-up normal. Treat it as "nothing here".

### What a height query returns

For a point (x, y): the ground point (x, y, height), the surface normal,
and the ground **material number** stored for that square (most likely
filled from the material map `terrm.dib`; not traced). What uses it is not
traced yet; terrain sounds and ground damage (`KQWorld::PoisonTime`,
`SwampTime` in `world.cs`) are likely users.

## Data used

- `<world>/kq/Terrain/TERRAIN.KQ`
- `<world>/resource/terrh.dib`, `terrh2.dib`, `terrm.dib`, the terrain
  material list, `<world>/kq/Terrain/<name>.bin`, light/shadow `.bin` files.

## Edge cases

- Points exactly on a grid line: either neighbouring square gives the same
  height (the triangles share edges), so no special case is needed.
- Some worlds comment out `floorBMPName` (Gnome, temples) or have no such
  line (Barren, CastleD, SnowExit); then there is no floor grid and every
  "floor" height question gets the surface (`water.md`).

## Do not reproduce

Nothing identified yet.

## How to check

- Log the height our code returns at the positions of Daventry objects that
  stand on the ground: they should match their `locZ` within a few units.
  (The measurement above did exactly that with a throwaway script.)
- Draw the ground with the checkerboard split; compare silhouettes of hills
  with the original from the same camera spot (by eye, if wanted).

## Open questions

1. (Answered: value × `heightBMPScale`, measured and now traced.)
2. (Answered in `water.md`: `floorBMPName` is the floor under the water.)
3. What marks a square as a hole. (Texture choice and orientation: answered
   in "Ground textures".)
4. What the terrain material flag bits 0x1000 and 0xF000 mean
   (`QUESTIONS.md` Q5).
5. (Answered: one texture per square, oriented by the code table.)
6. Visible/haze/perspective distances, detail bands, lighting maps.
