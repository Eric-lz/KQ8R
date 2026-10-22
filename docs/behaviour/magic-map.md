# Magic map

Study session A, 2026-10-21. Sources: `Mask.exe` (traced), the map files
`GAME/Kq/*map.kq` and teleport files `GAME/Kq/*tele.kq` / `*tele.cs`, the
world scripts (`newworld.cs`, `wizard.cs`, `keepdoor.cs`, `earntele.cs`,
`telepad.cs`, `teletrap.cs`, the `addMapIcon` callers), the layout
`Gplayscr.gui` (read by `kq8 gui-verify`), `GAME/Kq/Invinfo.itl`,
`GAME/ENGLISH/500.MSG`, the pictures in `GAME/Resource` and each world's
`8gui`, and the user's saved games in `MASK/save` (read only, to see how
the map is stored). Marks: **[traced]** = read in the program, **[data]**
= in the game files, **[inferred]** = concluded, not seen directly.

Builds on `interface.md` (rules 8–11, 15, 19, 45, the map window's
place and slide) and `inventory.md` (the `MagicMap` item).

## Purpose

A map of the current world in a window on the left of the screen.
Places Connor has been near are drawn in; the rest stays blank ("fog").
An arrow shows where Connor is and which way he faces. Scripts put small
pictures (icons) on it to mark things that happened. The map can show
the maps of other worlds Connor has visited, and when he stands on a
teleporter pad it sends him to another world's pad. A small live copy
of the map, centred on Connor, sits in the bottom-right panel.

## Words used here

- **Map picture**: a 460 × 460 picture of the whole world (8-bit
  colour). Each world has two: the **full** picture (`fullMap`, the
  drawn map) and the **blank** one (`blankMap`, the empty parchment).
- **Map pixel**: one pixel of that 460 × 460 picture.
- **Fog**: which map pixels are still blank. One yes/no per map pixel.
- **Map object**: the `KQMap` object a world loads from its
  `<x>map.kq` file. Only one is the "current" map at a time.
- **Map index**: a number 0–15 that names each of the sixteen possible
  maps (rule 4). The game uses it to keep per-map flags on Connor.

## Rules

### Getting the map and switching it on

1. **The item** [data, traced]. `MagicMap` (`Invinfo.itl`, category
   `MagicMap`, picture `IDBMP_TODO` = `todo.pbm`, sounds `magic.aud` /
   `beam.aud`) goes into the bottom-right map slot, not into the eight
   slots (`inventory.md`). The Wizard gives it during his map scene in
   Daventry (`wizard.cs`: `give MagicMap` when his `wizmap4` part ends,
   with 15 experience), then pops up help text 153 ("MAP: When you
   possess the magic map you can open and close it by pressing [^8].
   Left-click on the open map to position it, right-click to zoom
   in/out.").
2. **"Active"** (`isActive`) [traced]. The current map only works while
   it is active: only then does it follow Connor (rules 9–12), does the
   map key or a click on the bottom-right slot open the window
   (rule 30), and does the slot show the live map (rule 27).
   - The map becomes active when the `MagicMap` item is put in the
     bottom-right slot, and inactive when the slot is emptied [traced].
     The slot blinks 5 times when the item arrives, like the other slots
     (`interface.md` rule 44) [traced].
   - Scripts: `KQMap::activate` / `KQMap::deactivate` [traced]. Every
     world's `newworld.cs` except Daventry's and Castle Daventry's
     activates its map as soon as it is loaded; Castle Daventry's
     `world.cs` activates it only if Connor carries `MagicMap` [data].
   - All map files ship with `isActive=0`; the value is saved with the
     map (a saved Daventry map has `isActive=1`) [data].
   - Activating starts listening to Connor's movement (rule 9) and works
     out the fog circle (rule 13) [traced].
3. **One map per world** [traced, data]. A world loads one map file
   (`loadKQ <x>map.kq` in its `newworld.cs`) and the game keeps a single
   current map; commands like `KQMap::activate` act on it (they print
   "KQMap not loaded" if there is none). Worlds and their files:

   | World | Map file | Map index | Name (500.MSG) | Pictures | `fogRadius` |
   | --- | --- | --- | --- | --- | --- |
   | Daventry | `davmap.kq` | 0 | 38 "Kingdom of Daventry" | `davmap(0).pbm` | 10,000 |
   | Dead City | `dedMap.kq` | 1 | 39 "Dimension of Death" | `dedmap(0)` | 8,000 |
   | Swamp | `swmpMap.kq` | 2 | 40 "The Swamp" | `swmpmap(0)` | 12,000 |
   | Gnome | `gnommap.kq` | 3 | 41 "Underground Realm / of the Gnomes" | `gnomap(0)` | 5,000 |
   | Barren | `barrmap.kq` | 4 | 42 "The Barren Region" | `barmap(0)` | 12,000 |
   | (none) | `dwrfMap.kq` | 5 | 43 "The Dwarf Underground" | `dwfmap(0)` | 5,000 |
   | Ice world | `frosmap.kq` | 6 | 44 "The Frozen Reaches" | `icemap(0)` | 12,000 |
   | Temple 1–3 | `tmp1map`–`tmp3map.kq` | 7–9 | 45–47 "Realm of the Sun / Level 1–3" | `tmpNmap(0)` | 3,000 |
   | Temple 4 | `tmp4map.kq` | 10 | 48 "Realm of the Sun / Altar Room" | `tmp4map(0)` | 2,000 |
   | (none) | `ice1map`–`ice3map.kq` | 11–13 | 49–51 "Icelord's Castle / Level 1–3" | `icelmap(0)`; 2 and 3 use `dedmap(0)` | 2,500 / none |
   | Castle Daventry | `castmap.kq` | 14 | 52 "Castle Daventry" | `cstlmap(0)` | 5,000 |
   | Snow exit | `exitmap.kq` | 15 | 53 "Paradise Lost" | `exitmap(0)` | none |

   The map index is the `mapName` number minus 38 (`#38` → 0) [traced].
   `dwrfMap.kq` and `ice1map`–`ice3map.kq` are **never loaded** by any
   script, so those four maps never appear [data]. The several-map
   worlds the prompt mentions are really separate worlds: each temple
   level is its own world with its own map.

### The picture and the window

4. **World to map pixel** [traced]. With the map file's `west`, `east`,
   `north`, `south` (world units):
   - across: u = (x − west) ÷ (east − west); map pixel x =
     u × 460, the fraction dropped (towards zero);
   - down: v = 1 − (y − south) ÷ |north − south|; map pixel y =
     v × 460, the fraction dropped.
   So **north (+Y) is at the top, east (+X) at the right**. There is no
   clamp: a position outside the box gives a pixel outside 0–459, which
   simply reveals nothing (rule 12). (460 is the full picture's width and
   height; all 28 map pictures are 460 × 460 [data].) If west = east or
   north = south the map does nothing at all [traced].
   Scale: Daventry 409.6 world units per map pixel both ways, Gnome 355,
   Swamp 495, Barren 581, Dead City 512, Ice world 616 / 606, Castle 120,
   temples 63–184, Snow exit 137 / 143 [data, computed].
5. **What the window holds** [data: layout `Gplayscr.gui`]. The map
   window `IDCTL_MAPWIN` is 320 × 352 at (0, 64) of the 640 × 480
   layout. Inside it (its own coordinates):

   | Part | Rectangle | Picture |
   | --- | --- | --- |
   | top border | (0, 0)–(320, 12) | `97mapt.pbm` |
   | left border | (0, 12)–(12, 308) | `97mapl.pbm` |
   | right border | (308, 12)–(320, 308) | `97mapr.pbm` |
   | bottom border | (0, 308)–(320, 320) | `97mapb.pbm` |
   | map area | (12, 12)–(308, 308), 296 × 296 | `4black.pbm` behind the map |
   | "previous map" arrow | (0, 320)–(28, 352) | `maplft.pba` (2 frames) |
   | "next map" arrow | (28, 320)–(56, 352) | `maprit.pba` (2 frames) |
   | name plate | (56, 320)–(264, 352) | `mapname.pbm` (208 × 32) |
   | "Go" button, off | (264, 320)–(320, 352) | `gooff.pba` |
   | "Go" button, on | (264, 672)–(320, 704), i.e. off screen | `goon.pba` |
   | "Loading" text | (78, 900)–(238, 917), off screen | `#10` "Load" |

   The borders, arrows, plate and buttons come from the current world's
   `8gui` folder, so they are in the world's own palette [data].
6. **What is drawn in the map area** [traced]:
   - the map's own picture in memory (rule 7), scaled to the current zoom
     and placed by the current pan (rules 22–25), clipped to the map
     area;
   - on top, Connor's arrow (rule 17), only when the map shown is the
     current world's.
7. **The map's own picture in memory** [traced]: when the map is
   loaded it starts as a copy of the **blank** picture; every map pixel
   that is out of the fog is copied from the **full** picture; then the
   icons are drawn on (rule 19). Revealing later (rule 12) copies more
   full-picture pixels in. Pictures are 8-bit and drawn with the world's
   palette; in its own world a map is shown with its colours unchanged
   [traced; that the map pictures were painted for their world's palette
   is inferred].
8. **The name plate** [traced]. `mapName` `#n` is text n of `500.MSG`
   (noun 0, verb 0, case n). If the text has a line break, its two lines
   go into the two-line text fields (`IDCTL_MAPWIN_NAME_TEXT2` at
   (56, 306)–(264, 352) and `TEXT3` at (56, 320)–(264, 366), font
   `console.pft`, centred), the one-line field empty; otherwise the text
   goes into `IDCTL_MAPWIN_NAME_TEXT1` (56, 320)–(264, 352), centred, and
   the other two are emptied. Changed whenever the shown map changes.

### Following Connor and the fog

9. **When the map updates** [traced]. An active map is told every time
   Connor's position is changed (each movement step, a script `move`, a
   placement). It does **not** run on a timer; while Connor stands still
   nothing is revealed. Each time:
   - unless "stop location" is on (rule 15), his (x, y) is turned into a
     map pixel (rule 4);
   - only if that pixel differs from the last one: the map snaps back to
     the current world if another world's map was being shown (rule 34),
     the current map's index is marked "visited" on Connor (rule 33), and
     the fog is cleared around the new pixel (rule 12); then the icons
     are drawn again (rule 19);
   - always: the arrow frame is updated from his heading (rule 17).
10. Height (z) plays no part [traced].
11. The "last pixel" starts unset, so the first update after loading
    always reveals [inferred: it starts at a value no real pixel has;
    initial value not traced].
12. **Revealing** [traced]: a disc of radius r map pixels (rule 13)
    around Connor's pixel (cx, cy) is copied from the full picture into
    the map's picture, and those pixels are marked out of the fog:
    - rows cy − r + 1 to cy + r (2r rows);
    - in row cy + d the half-width is w = √(r² − e²) with the fraction
      dropped, where e = 1 − d for rows on or above Connor's row
      (d ≤ 0) and e = d below it; so the top row (d = 1 − r) and the
      bottom row (d = r) have w = 0, the widest rows (w = r − 1, since
      e is never 0) are Connor's row and the one below it;
    - in each row, columns cx − w to cx + w − 1 are copied;
    - rows and columns outside 0–459 are skipped (cut at the edges).
    Once out of the fog, a pixel never goes back [traced].
13. **The fog radius in map pixels** [traced]: r = 460 × `fogRadius` ÷
    (east − west), whole-number division (fraction dropped). It uses the
    east–west size for both directions. If `fogRadius` is missing or
    0 or less, the world's `hazeDistance` (from its terrain) is used
    instead, or 20,000 if the world has no terrain. Worked out when the
    map is set up and again by `KQMap::setFogRadius n`. Values [data,
    computed]: Daventry 24, Dead City 15, Swamp 24, Gnome 14, Barren 20,
    Ice world 19, Castle 41, temples 1–3 16 / 18 / 17, temple 4 31, Snow
    exit 131 (its `fogRadius` is commented out, so `hazeDistance` 18,000
    is used). In world units the disc is `fogRadius` across-ways
    (Daventry 10,000 = 24 pixels × 409.6, within one pixel).
14. **`useFog`** [traced]. All shipped maps have `useFog=1`.
    `KQMap::useFog 0` (only used by the debug alias `teleport`, rule 41)
    copies the whole full picture in at once, without marking the pixels
    out of the fog. `KQMap::useFog 1` only sets the flag back. Revealing
    (rule 12) happens either way.
15. **"Stop location"** (`holdMapLoc`, `KQMap::stopLoc 0|1`) [traced,
    data]: while on, Connor's position is ignored: no revealing, no
    arrow move, no "visited" mark (the arrow frame still turns). Dead
    City turns it on when Connor is put into the second copy of the
    death hall (`dhalltrp.cs`, `deadHallTeleport DeadHall DeadHal2`,
    `stopLoc 1`) and off again when he comes back (`ddoor4.cs`,
    `death.cs`, `stopLoc 0`) [data], so the arrow stays where the real
    hall is.
16. **Saving the fog** [traced, data]. The fog is kept per map, saved
    with the map object in the saved game and in the per-world state
    the game keeps when Connor leaves a world. Each world therefore has
    its own fog; it is never shared. Stored as run lengths, `rle0`,
    `rle1`, …:
    - the 460 rows are read top to bottom, each as **464** places (the
      460 pixels and 4 unused places at the end of each row);
    - `rle0` counts places still in the fog, `rle1` the next run out of
      the fog, `rle2` in the fog again, and so on, alternating; runs run
      on from one row into the next;
    - after the last run comes one more key with the value **−1**, which
      ends the list (a map with nothing revealed is just `rle0=-1`, as
      every shipped file has).
    Example (the user's save 10, Daventry): `rle0=18768` (40 rows of 464
    plus 208), `rle1=15`, `rle2=446`, … `rle704=42496`, `rle705=-1`
    [data]. The four unused places per row are written as "in the fog"
    and are ignored on reading [traced; inferred that they are always in
    the fog].
    **Corrected 2026-10-22 (Q150 b, traced):** revealing copies pixels up
    to but not including the circle's right edge, but marks the edge
    column as revealed too; the right end is clipped at the map width
    (460), so when a circle reaches the right border **column 460 is
    marked** (columns 461-463 never are). That is the Castle save's
    column 460, the same bug as "Do not reproduce" 2. Reading still
    ignores it. All icons are drawn again after every reveal.

### Connor's arrow

17. **Arrow frame** [traced, data]. `maparrow.pba` has **16 frames**,
    10 × 10 pixels. Frame 0 points **up** (north), frame 4 right (east),
    frame 8 down, frame 12 left: each frame is 22.5° **clockwise** from
    the one before [data: the 16 frames dumped and looked at]. The
    frame is the nearest one to the way Connor faces:
    frame = (π − heading) ÷ 2π × 16, rounded to the nearest whole
    number, then wrapped into 0–15. "Heading" is Connor's turn about Z
    (0 = facing −Y, south; positive = turned anticlockwise, as everywhere
    in these notes; `connor-movement.md`). Check: heading π (facing
    north) → frame 0 (up); heading π/2 (facing east) → frame 4 (right).
18. **Arrow place** [traced]: its top-left corner is at Connor's map
    pixel scaled by the zoom (map pixel × shown width ÷ 460) plus the
    picture's on-screen top-left, minus half the arrow's size (5, 5), so
    the arrow's centre is on Connor. It is drawn only when the shown map
    is the current world's.

### Map icons

19. **What they are** [traced, data]. Small Windows `.bmp` pictures
    (8-bit, e.g. `dv_alchy.bmp` 11 × 14, in `GAME/Resource`) that
    scripts stamp onto a map picture to mark a place or a change (the
    alchemist's door, a felled tree, the teleporter, a freed prisoner,
    opened gates, poisoned water…). Each icon is drawn with its top-left
    corner at map pixel (x, y); its pixel values are used directly as
    palette indices; **value 0 is see-through** [traced]. Icons are drawn
    **over the fog**: an icon shows even where the map is still blank
    [traced].
20. **`addMapIcon file.bmp x y [picture.pbm]`** [traced]: adds an icon
    to the **current world's** map object, for the given map picture
    (default: the picture now shown). The same file at the same x and y
    on the same picture is not added twice. If the picture is the one
    shown now, it is drawn at once; otherwise it is only kept, and drawn
    when that picture is shown (rule 34). Scripts use the extra argument
    to mark other worlds' maps: the Swamp's `kettle.cs` puts `dv_poisn`
    on `davmap.pbm` and `davmap0.pbm`; the Sage Gnome puts `gn_telep` on
    `barmap.pbm` and `barmap0.pbm` [data]. Such an icon lives in the
    world that added it (rule 34) [traced].
21. **`removeMapIcon x y`** [traced]: removes the icon at (x, y) on the
    picture shown now, and paints that rectangle back: blank picture
    first, then the full picture wherever the fog is cleared. No shipped
    script uses it [data].
    `cMapIcons` and `mapIcon<n>`, `srcIcon<n>`, `xIcon<n>`, `yIcon<n>`
    save the icon list with the map (picture, icon file, x, y) [traced,
    data: a saved Daventry map has 6]; all shipped files have
    `cMapIcons=0`.
    **Added 2026-10-22 (Q150 a, traced):** shipped scripts **do** use it:
    Dead City, 30+ times (`aswitch1`-`4`, `cswitch1`-`4`, `sylftest`,
    `triptrp2`-`5`; `kswitch`'s use is commented out). It removes only
    the **first** icon with exactly that x, y whose picture is **the one
    shown at that moment** (if the player has paged the map window to
    another world's map, nothing matches and the icon stays, inferred).
    The rectangle is repainted: the blank picture first, then the full
    picture wherever the fog is cleared; the fog is tested in groups of
    8 columns, so cleared pixels up to 7 columns either side of the
    rectangle are repainted from the full picture too. **Other icons
    under or next to it are not redrawn**, so an overlapping icon is cut
    (Dead City's jail icons are 5-6 pixels apart). The icon's slot in
    the list is emptied, not compacted.

### Mouse on the map

22. **The view** [traced]. The map picture is drawn at a size and
    offset kept as a rectangle measured from the **centre of the map
    area**. At the start of a new game it is (−152.83, −152.11) to
    (145.44, 146.16): the whole 460 × 460 picture shrunk to about
    298 × 298 (× 0.648), so it just fills the 296 × 296 map area, very
    nearly centred [traced]. This rectangle is kept on Connor, so the
    zoom and pan carry on into the next world and are kept in saved
    games [traced that it is stored on Connor; that it is saved with
    him is inferred].
23. **Left button: drag to move ("position") the map** [traced; which
    button from the help text and the handler order]. While the left
    button is held over the map area, every mouse movement moves the
    picture by the same number of screen pixels. The move is limited so
    that the **centre of the map area always stays on the picture**: the
    picture's left edge can't go right of the centre, its right edge
    can't go left of it, and the same for top and bottom.
24. **Right button: drag sideways to zoom** [traced]. While the right
    button is held, each sideways mouse movement of dx screen pixels
    multiplies the picture's size by 2^(dx ÷ 64), about the centre of the
    map area: 64 pixels to the right doubles it (zoom in), 64 to the left
    halves it. Up/down movement does nothing. Limits: the shown width
    stays between 460 ÷ 4 = **115** and 460 × 8 = **3,680** screen pixels
    (× 0.25 to × 8 of the picture's own size); past a limit it is set to
    the limit. A right click without moving does nothing.
25. **Following Connor** [traced]: each time Connor's map pixel changes
    (rule 9), the big map is marked to follow him. While marked, at each
    draw, if the arrow's top-left corner is outside x 20–280 or y 20–340
    the picture is moved towards bringing it back: by (20 − x) ÷ 10 + 1
    pixels when x < 20, (x − 280) ÷ 10 + 1 when x > 280, the same for y
    with 20 and 340 (whole-number divisions), repeated up to 6 times in
    one draw. When the arrow is inside, the mark is cleared. So after
    the player drags the map away, it stays put until Connor moves to
    another map pixel. Which coordinates these limits are in was not
    traced. **The user's check in the original (2026-10-21):** once
    Connor moves, the map scrolls to keep the arrow in view **at all four
    edges** (so the top limit does work); the exact margin could not be
    measured by eye. Build it as: keep the arrow's top-left corner at
    least 20 pixels inside the visible map area on every side, with the
    step above (open question 2).
26. The pointer over the map is the normal play pointer [inferred:
    nothing in the map changes it].

### The bottom-right slot (answers `interface.md` open question 4)

27. **A live mini-map** [traced]. `IDCTL_INVBAR_MINIMAP` (56 × 56 at
    (132, 4) of the bottom-right panel, framed by `2minimapT/B/L/R.pbm`)
    shows `2mapblnk.pbm` while the map is not active. Once the map is
    active the slot draws **the current map itself**, placed so that
    Connor's map pixel is at the middle of the slot, with his arrow
    there, at the same size (zoom) as the big map's view rectangle
    [traced]. The item's own picture (`todo.pbm`) is not used there
    [traced: the slot draws the map instead]. **The slot keeps its own
    zoom**: it starts from the view stored on Connor when the map is set
    up and does **not** change when the big map is zoomed (the user's
    check, 2026-10-21; traced that each view keeps its own copy). At the
    start view (× 0.648) the 56 × 56 slot shows only the part of the map
    around Connor, not the whole world (the user saw this too).
    **Added 2026-10-22 (Q150 d, traced): the mini-map slot never
    blinks.** Its own drawing replaces the normal slot drawing, which is
    where the blink lives (the item still sets a blink count of 5,
    unused). When the map arrives it is switched on at once and the slot
    goes straight from the empty picture to the live map at its next
    draw. While the map is not active the live-map drawing draws nothing
    and the layout's `2mapblnk.pbm` stays.
28. With the map not active, the slot's `2mapblnk.pbm` is 52 × 52 in
    the 56 × 56 box; how it fills the box is `QUESTIONS.md` Q108 (b),
    not traced here.

### Opening and closing (adds to `interface.md` rules 9, 11, 15)

29. Show/hide and the sounds `map.aud` / `mapbgone.aud`: `interface.md`
    rule 9.
30. **Ways to open or close it** [traced]:
    - the map key (default Tab): toggles the window if the current map
      is **active** (rule 2). Nothing else is checked: **not handsOff,
      not a conversation, not the rope or a fight** (only Escape checks
      handsOff in that key handler);
    - a left click on the bottom-right map slot: toggles it, also only if
      the map is active;
    - `setMapGui 0|1` (no shipped script uses it [data]);
    - `handsOff 1` hides it and `handsOff 0` doesn't bring it back
      (`interface.md` rule 15).
31. Whether it is open is part of the saved panel set (`interface.md`
    rule 8) [traced there].

### Other worlds' maps (the arrows under the map)

32. **Paging** [traced]. The "previous" / "next" arrows step through
    the sixteen map indexes (rule 3) backwards / forwards, wrapping
    0–15, and stop at the first map that is marked visited (rule 33);
    the current world's own map counts. If none is found after 16 steps
    nothing changes.
33. **"Visited"** [traced]: a map is marked visited on Connor when
    Connor's map pixel changes while it is the current, active map
    (rule 9), so a world appears in the list once he has walked around
    in it with the map. The Sage Gnome's `earnTeleport barrMap.kq` also
    marks Barren visited (rule 37), so Barren can be paged to before
    Connor has been there [traced].
34. **Showing another world's map** [traced]:
    - that world's map is read from the state the game kept when Connor
      left that world (so with its own fog and icons) [traced that it is
      looked up by file name through the game's resource search, which
      looks in the `cur` folder first, where each left world's map is
      kept as `<map>.kq`: `world-change.md` rule 16, traced 2026-10-21];
    - drawn as that map's own picture with its fog if its `useFog` is on,
      else its full picture;
    - its colours are matched to the **current** world's palette: each
      of its 256 palette entries is replaced by the current palette's
      entry with the smallest squared RGB difference (all 256 entries
      can be chosen). The palettes are `GAME/Resource/<name>.ppl` (map
      index 0 `daventry`, 1 `newdead`, 2 `swamp`, 3 `gnome`, 4 `barren`,
      5 `dwarf`, 6 `ice`, 7–10 `temple01`–`04`, 11–13 `icelord`, 14
      `castle`, 15 `exit`; the 256 RGBX entries start at byte 28)
      [traced, data];
    - the current world's icons kept for that picture (rule 20) are drawn
      on it;
    - its name goes on the plate; no arrow is drawn;
    - the "Go" button is updated (rule 38).
    Paging back to the current world's map shows it exactly as it was.
    If Connor's map pixel changes while another world's map is shown, the
    window **snaps back** to the current world's map (rule 9).
    **Order (Q150 c, traced 2026-10-22):** the icons are drawn twice,
    before the colour match and again after it with their own palette
    indexes (index 0 see-through; the icon `.bmp` is never remapped).
    The second drawing covers the same pixels, so the result is **icons
    drawn after the match, indexes as they are**, not colour-matched.

### Teleporting

35. **Where you can teleport** [data, traced]. Between the teleporter
    pads of six worlds: Daventry (inside the keep), Dead City, Swamp,
    Gnome, Barren and the Ice world. Each has a pad trap
    (`teletrap.cs` / Ice world `telepad.cs`) whose On runs
    `onTeleport` and Off `offTeleport`, and a `TeleportScript` /
    `TeleportPortalKQ` in its map file. Temples, Castle Daventry and the
    Snow exit have none.
36. **On the pad** [traced]: `onTeleport` sets "Connor is on the pad"
    on the current map and refreshes the window (rule 38); `offTeleport`
    clears it. This flag is not saved.
37. **Earning a teleporter** [traced, data]. `earnTeleport` marks the
    **current** world's map as `earnedTeleport=1` (saved with it) and, for
    map indexes 0–7, sets that world's "teleporter earned" mark on
    Connor. Where it is called: Daventry when the keep is unlocked with
    the key (`keepdoor.cs`, also adds the `dv_telep` icon at 250, 98);
    Dead City, Swamp and Gnome by their `earntele.cs` (once each); the
    Ice world when Connor leaves the elevator (`eleport.cs`). With an
    argument (only the Sage Gnome's `earnTeleport barrMap.kq`) it instead
    sets Barren's marks on Connor: the `SageGnomeEarnedTeleport` flag,
    Barren "visited" and Barren "teleporter earned"; the argument's text
    itself is not looked at [traced]. `teleportersEarned` puts the number
    of worlds with the "teleporter earned" mark (0–8) in
    `$Console::Return`; the pads' own click scripts use it for Connor's
    remark (more than 1: "I've seen this before"; else "What is that?")
    [data].
38. **The "Go" button** [traced]. Lit (`goon.pba` moved over the
    `gooff.pba` button) only when, after a page:
    - another world's map is shown, and
    - Connor is on the pad (rule 36), and
    - that world's teleporter is earned: its map's `earnedTeleport` is
      set, or (Barren only) Connor has `SageGnomeEarnedTeleport`.
    Otherwise it shows unlit. Clicking the **unlit** button opens help
    popup 151 ("TELEPORTING: To use the teleporter, stand on the magical
    pad and open your map. Then select a world where you have already
    found another teleporter and then click the teleport button.") with
    an OK button.
39. **Clicking the lit "Go" button** [traced, data] runs the **current**
    world's `TeleportScript` with `$1` = the **shown** world's
    `TeleportPortalKQ` (e.g. in Daventry, Gnome shown: `Davtele.cs` with
    `$1` = `Gnomtele.kq`). The shipped scripts all do the same: delete an
    old portal object if one is remembered, `loadKQ $1 @ <this world's
    pad position>` (a `KQPortal` whose `world` is the destination),
    remember its name, and `KQMonster::setState Connor Portal <name>`.
40. **The Portal state** (shared with walking exits between worlds)
    [traced]:
    1. a Yes/No popup: text 114 "You are about to load a new region.
       Are you sure you want to do that?", buttons 29 Yes, 14 No;
    2. **No**: the portal's script gets `Decline`; if the portal has a
       `noLoc` other than (−1, −1, −1) Connor is moved there (the
       teleport portals have −1); Connor goes back to waiting, still on
       the pad, map still open;
    3. **Yes**: the portal's script gets `Accept`; `handsOff 1` (so the
       bottom bar slides away and **the map is hidden**, `interface.md`
       rule 15); first person is switched off and blocked; Connor's
       target is cleared; the longest time step per frame is set to two
       frame periods (as a `handsOff` with a number does);
    4. if the portal has an `enterAnim` (`teleportdown` in all but the
       Swamp's), Connor plays it (from the portal's `enterAnmFile` if it
       names one, else from his current list; **corrected 2026-10-21**:
       the `conner.anm` → `conner2.anm` swap for the full armour applies
       to the arrival's `exitAnmFile`, see `world-change.md` rule 5);
       when it ends, or at once if there is none, the world change
       starts: Connor's arrival position and
       direction (`destConnorLoc` / `Dir`) and the camera's (`destCamLoc`
       / `Dir`) are handed to the next world, which is the portal's
       `world`;
    5. in the new world Connor is placed there, the camera too, and he
       plays the `exitAnim` (`teleportup`; the Swamp's portal has `out`)
       [traced 2026-10-21 in `world-change.md` rules 6, 20: the places go
       over as `KQPortal::ConLoc` / `CamLoc` and are used by the new
       world's first load of Connor and of the camera; Connor arrives in
       the Portal state with his next movement set to `exitAnim`;
       `KQPortal::ConAnim` is read but not used]. When it ends: first person allowed again, the portal's
       `exitScript` gets `Exit`, the time step limit goes back to 1 s,
       `handsOff 0`, Connor waits.
    The map stays closed on arrival (hidden by `handsOff 1`, not brought
    back by `handsOff 0`) [traced through `interface.md` rule 15].
41. **Debug teleport inside a world** [traced]: the console alias
    `teleport` ("`KQMap::activate ; KQMap::useFog 0 ; KQMap::teleport
    1`") shows the whole map, and then a left click on the map moves
    Connor to that spot: x = west + (east − west) × (click's fraction
    across), y the same from the south edge up, z = the ground height
    there + 250. Not a game feature (Do not reproduce 5).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| Map pictures | 460 × 460, 8-bit | data |
| Map area in the window | 296 × 296 at (12, 12) | data |
| Window | 320 × 352 at (0, 64) | data |
| Start view | (−152.83, −152.11)–(145.44, 146.16) from the area's centre (× 0.648) | traced |
| Zoom per 64 px sideways (right button) | × 2 | traced |
| Zoom limits (shown width) | 115 – 3,680 px (× 0.25 – × 8) | traced |
| Follow margins (arrow top-left) | x 20–280, y 20–340 | traced (space unsure) |
| Follow step | (distance past the margin) ÷ 10 + 1 px, up to 6 per draw | traced |
| Arrow | 16 frames, 10 × 10, frame 0 up, clockwise, 22.5° each | data, traced |
| Fog radius | 460 × `fogRadius` ÷ (east − west), whole number | traced |
| Fog radius missing | world `hazeDistance`, else 20,000 | traced |
| Fog rows saved | 464 places per row, 460 rows, hidden run first, ends with −1 | traced, data |
| Icon see-through value | 0 | traced |
| Map name | `500.MSG` 0 0 (38 + map index) | traced, data |
| Teleport question | `500.MSG` 0 0 114, Yes 29, No 14 | traced, data |
| Unlit "Go" help | `500.MSG` 0 0 151 | traced, data |
| Debug teleport height | ground + 250 | traced |
| Slot blink on getting the map | 5 times (`interface.md` rule 44) | traced |

## Data used

- `GAME/Kq/<x>map.kq` (`KQMap`): `north`, `south`, `east`, `west`,
  `useFog`, `fogRadius`, `holdMapLoc`, `isActive`, `blankMap`,
  `fullMap`, `arrowPBA`, `mapName`, `earnedTeleport`, `TeleportScript`,
  `TeleportPortalKQ`, `cMapIcons` (+ `mapIcon/srcIcon/xIcon/yIcon<n>`),
  `rle<n>` [traced which are read]. The saved form also has the usual
  object keys (`active3DFlag`, `hidden`, …) [data].
- `GAME/Kq/<x>tele.kq` (`KQPortal`): `world`, `enterAnim`, `exitAnim`,
  `destConnorLocX/Y/Z`, `destConnorDirX/Y/Z`, `destCamLocX/Y/Z`,
  `destCamDirX/Y/Z`, `noLocX/Y/Z`, plus `enterAnmFile`, `exitAnmFile`,
  `exitScript` (not set in the teleport files) [traced which names exist].
- `GAME/Kq/<x>tele.cs` (Daventry `Davtele.cs`, Dead City `Deadtele.cs`,
  `swamtele.cs`, `gnomtele.cs`, `bartele.cs`, `icetele.cs`) [data].
- `GAME/Resource`: the map pictures (table in rule 3), the icon `.bmp`
  files, the `.ppl` palettes, `kqGuiTag.TTAG`, `Gplayscr.gui` [data].
- Each world's `8gui`: `97mapt/b/l/r.pbm`, `maparrow.pba`, `maplft.pba`,
  `maprit.pba`, `mapname.pbm`, `gooff.pba`, `goon.pba`, `2mapblnk.pbm`,
  `2minimapT/B/L/R.pbm`, `4black.pbm`, `todo.pbm`, `console.pft` [data].
- `GAME/ENGLISH/500.MSG` texts 38–53, 114, 151, 153, 146 (the
  "TELEPORTER: A swirling pad is a teleporter…" popup that `keepdoor.cs`
  shows with 151) [data].
- `GAME/Kq/Invinfo.itl` item 53 `INVITEM_MagicMap` [data].

## Edge cases

- **Map key during a conversation, a cutscene, on the rope, in a
  fight**: it opens and closes the map whenever the map is active; the
  game checks nothing else (rule 30) [traced]. In a cutscene the map is
  hidden when `handsOff 1` starts, but Tab can open it again during the
  cutscene [traced; **the user's check, 2026-10-21:** the map opens
  during conversations and scripted scenes].
- **Dying with the map open**: Connor's dying state itself doesn't touch
  the map [traced]; the `handsOff` of the death path (`main-menu.md`
  open question 5) would hide it [inferred].
- **Map open across a world change**: every world change goes through
  the Portal state, whose `handsOff 1` hides the map, so it arrives
  closed (rule 40) [traced].
- **Loading a saved game**: the open/closed state comes back with the
  panel set (`interface.md` rule 19); each map's fog, icons and
  `earnedTeleport` come back with it (rule 16) [traced].
- **Connor outside the map's box**: the pixel is outside the picture;
  nothing is revealed and the arrow is drawn off the picture (clipped
  to the map area) [traced: no clamp].
- **Paging while Connor walks**: the window jumps back to the current
  world's map as soon as he crosses into another map pixel (about 410
  world units in Daventry) [traced].
- **No map carried but the world activates it** (`newworld.cs` in every
  world but Daventry and the castle): the map would work without the
  item. In the normal game Connor always has it by then [inferred, not
  checked].
- **Teleporting to a world whose map isn't visited**: impossible; it
  can't be paged to (rule 32). Barren is the exception made by the Sage
  (rule 37).
- **`earnTeleport` in a world with map index 8 or more** (temples,
  castle, exit): only the map's own `earnedTeleport` is set; no mark on
  Connor [traced]; no shipped script does this.

## Do not reproduce

1. **Snap-back leaves the "Go" button lit** [traced]: when Connor's
   pixel changes while another world's map is shown, the window returns
   to the current world's map but the "Go" button is not updated. If
   Connor shuffles across a map-pixel boundary on the pad, the button
   stays lit over the current world's map, and clicking it would still
   teleport to the world that was shown. Instead: update the button
   whenever the shown map changes.
2. **Revealed edge pixel** [traced]: revealing copies columns cx − w to
   cx + w − 1 but marks cx + w as out of the fog too, so after loading a
   save one more pixel per row appears. Use the same columns for both.
3. **`useFog 0` is lost on reload** [traced]: the full picture is copied
   in without marking the fog, so a reloaded game shows only the fog
   again. Only the debug alias uses it; if we keep it, mark the pixels.
4. **Follow margins** (rule 25): their coordinates were not traced; the
   user saw them work on all four edges. Keep the arrow at least 20
   pixels inside the visible map area on all four sides.
5. **The debug teleport** (rule 41) and `KQMap::teleport`: leave it out
   of normal play (a developer key at most).
6. **The unused maps** (`dwrfMap`, `ice1map`–`ice3map`; `ice2`/`ice3`
   even point at Dead City's pictures): don't load them; keep their
   indexes free so the numbering of the others stays.
7. **The 4 unused places per fog row** are an artefact of byte-wide
   storage. We may store the fog our own way, but if we ever read or
   write the original's saved maps, keep the 464-place rows.

## How to check

In the original (GOG 1.03). Start a new game, talk to the Wizard until
he gives the map. **The user ran 1–11 on 2026-10-21: all as described.**
Details from that check: 2, the whole of Daventry fits the big map at
first, but not the mini-map (expected, rule 27); 6, the mini-map does not
change size when zooming; 7, the view follows the arrow at every edge
once Connor moves, margin not measurable by eye; 8, the map opens during
conversations and scripted scenes.

1. **Getting it**: when the scene ends, the bottom-right slot blinks and
   then shows a tiny live map with the arrow in the middle; the help
   popup about the map appears. Before that, Tab does nothing and the
   slot shows the empty picture.
2. **Opening**: Tab opens the map from the left (sound), Tab closes it.
   A left click on the bottom-right slot does the same. Note whether the
   map first shows the whole of Daventry fitted into the window (rule
   22).
3. **North is up**: walk north (the direction you'd call "up" on the
   map); the arrow moves up. Turn Connor on the spot slowly: the arrow
   turns in 16 steps, clockwise when Connor turns right.
4. **Fog**: with the map open, walk in a straight line: a band about 24
   map pixels to each side opens up (a bit under a tenth of the map's
   width). Standing still, nothing more opens.
5. **Pan**: hold the left button on the map and drag: the map follows.
   Try to drag it far off: it stops when the window's centre reaches the
   map's edge.
6. **Zoom**: hold the right button and move the mouse right: it zooms
   in (about ×2 per 64 pixels); left: out. Moving up/down does nothing.
   Note how far in and out it goes (×8 and ×¼ of the picture's size).
   Does the bottom-right mini-map change size too, at once?
7. **Follow (the uncertain part)**: drag the map so the arrow is near
   the **top** edge, then walk north. Does the view move to keep the
   arrow in, and how close to the top edge does it get before it moves?
   Do the same at the bottom, left and right edges. Report which edges
   keep a margin and roughly how many pixels (rule 25, open question 2).
8. **Tab in a cutscene / conversation**: during a conversation with a
   villager, and during a script scene (e.g. the Wizard's), press Tab:
   does the map open? (Traced: yes.)
9. **Pages**: after visiting a second world with the map, the arrows
   under the map step between the worlds' maps; the other world's map
   shows its own explored part, its name on the plate, and no arrow.
   Walk a few steps: the window jumps back to the current world.
10. **Teleport**: once two teleporters are earned, stand on a pad, open
    the map, page to the other world: the "Go" button lights. Click it:
    the question "You are about to load a new region…" with Yes/No. No:
    nothing happens. Yes: the bottom bar and map slide away, Connor sinks
    into the pad, the new world loads, he rises on its pad; the map is
    closed afterwards. Click the unlit "Go" button elsewhere: the
    "TELEPORTING: …" help appears.
11. **Saved fog**: explore part of Daventry, save, quit, load: the same
    part is open.

In our build: log every map update (Connor x, y → map pixel, pixels
revealed, arrow frame), every zoom/pan (rectangle), every page (index,
name, "Go" lit or not) and every teleport step to a `map.log`, and
compare with the above. `kq8` can also check rule 16 against the
user's saved games: decode each saved map's `rle` list and count its
revealed pixels.

## Open questions

1. The last-pixel start value (rule 11): not traced; assume "unset".
2. ~~Whether the follow margins work on every edge~~: they do (the
   user, 2026-10-21; rule 25). Their exact size in the original is still
   not known; 20 pixels inside the visible area is the build rule.
3. ~~Whether the mini-map follows the big map's zoom~~: it doesn't (the
   user, 2026-10-21; rule 27). Still open, small: whether it picks up the
   big map's zoom after a world change (each view reloads the stored
   view when the map is set up [traced]; not checked).
4. ~~The arrival side of a world change~~: answered 2026-10-21 in
   `world-change.md` rules 6 and 20.
5. ~~How the game finds another world's kept map~~: answered 2026-10-21
   in `world-change.md` rule 16: each left world's map is saved as
   `cur/<map>.kq` and the file search looks in `cur` first.
6. How 800 × 600 scales the window, the map area and the margins
   (`interface.md` open question 1).
