# The on-screen interface (panels, meters, caption)

Study session A, 2026-10-18. Sources: `Mask.exe` (studied, described here
in our own words), the game's data (`GAME/Kq/*.kq`, `Gplayscr.gui`,
`kqGuiTag.TTAG`, `movekeys.kq`, `Options.cs`, `Invinfo.itl`), and the
printed manual `MASK/Manual.pdf` (pages 15–27). The rules for the items
themselves (what goes where, stacking, swapping, using) are in
`inventory.md`.

Marks used below: **[traced]** read in the exe; **[data]** read in the
game files; **[manual]** stated in the manual; **[inferred]** worked out
but not seen directly; **[not traced]** not settled.

## Purpose

While Connor plays, the screen carries a set of panels around the 3D
view: a top bar (mask, quest items, gold), a bottom bar (armour and
weapons, health and experience meters, eight health/potion slots, rope,
rocks, map icon), a map window, a caption strip for spoken lines, a help
popup, and small spell-timer icons. This note says which files draw
them, what each shows, how the numbers become bars, and when each panel
is shown or hidden.

## Files

1. **The in-game screen** is described by `GAME/Kq/Gplayscr.kq`
   (`[GamePlayScreen]`, `classType=KQGamePlayScreen`) [data]:
   - layout `guiFile=GPlayScr.gui` (in `GAME/Resource`);
   - pictures and fonts from the volume `$KQWorld::Name / $KQGame::BitDepth
     GUI`, i.e. **each world's own `8gui` folder** (`MASK/daventry/8gui`,
     253–254 files per world: the bar pictures `1*.pbm` quest items,
     `2*.pbm` bottom/top slots, `3*.pbm` health slots, `4*.pbm` meters,
     `97map*.pbm` map borders, cursors `*.pba`, fonts `14*.pft` and
     `console.pft`, the caption strip `stontran.pbm`);
   - `palette=<world>`, `useWorldPal=1`: **the world's palette** colours
     everything on this screen, fonts and caption included (answers Q106
     for the in-game screen). Which file that is: **the world's
     `KQWorld::objectPalette`** from its `kq/world.cs`; see "The world
     palette" just below (Q118). The text `<world>` is never read as a
     file name;
   - `cursor=IDBMA_CURS04`: the normal play pointer is `curs04.pba`.
2. The menus (`Mainmenu.kq`, `Options.kq`, `Graphics.kq`, `movekeys.kq`)
   use the volume `game / $KQGame::BitDepth GUI` (`GAME/8Gui`),
   `palette=Menus.ppl`, `useWorldPal=0`, cursor `IDBMA_CURSGUI`; the
   loading screen uses `loading.ppl` [data]. Each time a world loads,
   the game copies `GAME/8Gui/L<first 7 letters of the world's
   folder>.ppl` (e.g. `ldaventr.ppl`, `ldeadcit.ppl`) over
   `GAME/8Gui/loading.ppl`, with the matching loading picture
   [traced]. Not part of the game-play screen.

3. `GAME/Kq/KqGui.kq` names the tag file `kqGuiTag.ttag` and the four
   screen files (main menu, game play, options, loading) [data].
4. **Layout format.** The `.gui` blocks (dialogs `UIDL`/`CCBD`, buttons
   `CCBB`, texts `UITX`, pictures `UIBM`), their ids, rectangles, fonts,
   texts and pictures, the tag file and the `.pft` / `.pba` formats are
   described by the community notes `kq8gui.txt`, `kq8tag.txt`,
   `kq8pfon.txt`, `kq8pbma.txt` (docs/RESEARCH.md); B already reads them
   (`kq8 gui-verify`, `kq8 font-verify`). What each block **does** is not
   in those notes; it is below.
5. **Rectangles** of children are relative to their dialog [inferred
   from the layout: every child fits inside its parent that way; the
   screen's own code places dialogs in screen fractions of 640 × 480].
   A text with a degenerate rectangle (right < left, e.g. the counters
   `IDCTL_HEALTHBAR_1TEXT` (29, 2)-(28, 1), `IDCTL_INVBAR_GOLDTEXT`
   (632, 5)-(631, 4)) is anchored at that point and aligned by its
   justification: with 0x12 the number's right edge sits at the left x,
   its top at the top y [inferred]. Blocks at y 900 (`_FX` flashes,
   `IDCTL_MAPWIN_LOADING`) are off screen until code moves them; the
   `_FX` animations are shown on top of their slot when they play
   [inferred].
6. **Justification values** (Q107 b) [inferred from how the layouts use
   them; the bit split 0–2 horizontal / 3–5 vertical is from
   `kq8pfon.txt`]: horizontal 1 = left, 2 = right, 4 = centred; vertical
   1 (0x8) = centred in the rectangle, 2 (0x10) = from the top. So 0x9
   left, 0xA right, 0xC centred, 0x11 left/top, 0x12 right/top. The
   caption code sets its own value 0x14 (centred, from the top of each
   line's slot) [traced].

### The world palette (answers Q118)

There is **one display palette** at a time for the whole window: the
3D view and every panel are coloured by the same 256 colours [traced].
A screen with `useWorldPal=0` (the menus, the loading screen) puts its
own `palette=` file into the window when it is shown. A screen with
`useWorldPal=1` (only the game-play screen) puts **nothing** there: it
uses whatever the window holds [traced].

1a. **What fills the window during a world load** [traced, order from
    `mask.cs` and `world.cs`]:
    1. the start of the load puts `black.ppl` in (screen black);
    2. the loading screen puts `loading.ppl` in;
    3. `world.cs` (`Load`): `blackOut` puts `black.ppl` in again,
       `KQGame::selectScreen 1` shows the game-play screen (no palette
       change), then `loadPalette $KQWorld::loadingPalette 0` puts that
       file in, background colour index 0;
    4. `mask.cs`: `loadPalettes 0` reads the four world settings
       `KQWorld::objectPalette`, `terrainPalette`, `interiorPalette`,
       `connorPalette` (an empty setting is skipped) and puts the
       **object palette** in, background colour index 0.
       `KQWorld::loadingPalette` is only a script variable; the program
       itself never reads it.
1b. **Every frame while the 3D view draws** [traced]: before drawing
    the terrain it switches the window to the terrain palette, before
    the interiors to the interior palette, before the objects to the
    object palette (each only if that palette loaded, and only if it
    differs from the one already in). The frame therefore always **ends
    on the object palette**, and the panels, captions and fonts drawn
    over the view use it [the panels being drawn after the view is
    inferred: they cover it]. `connorPalette` is loaded but not used
    for this switching.
1c. **So the file is `KQWorld::objectPalette`** of the current world.
    In every shipped world all five settings name the same file, and
    `loadingPalette` too [data], so in practice it is that one file:

    | World folder | File |
    | --- | --- |
    | Daventry | `daventry.ppl` |
    | Barren | `barren.ppl` |
    | Gnome | `gnome.ppl` |
    | Swamp | `swamp.ppl` |
    | DeadCity | `newdead.ppl` |
    | CastleD | `Castle.ppl` (file `castle.ppl`) |
    | IceWorld | `ice.ppl` |
    | SnowExit | `exit.ppl` |
    | Temple1–4 | `temple01.ppl` – `temple04.ppl` |

    The `defaultPaletteName=` line in `GAME/Kq/<World>.kq` is **not
    used** (the program never reads that key [traced]; CastleD's even
    names a file that doesn't exist, `CastleD.ppl`). A rebuild should
    read `objectPalette` from `world.cs` (both the `BitDepth 8` and the
    other branch name the same file) and may fall back to
    `loadingPalette`, then to the other three.
1d. **Changes while playing.** Nothing in the shipped data changes the
    palette after the load: `loadPalette`, `blackOut` and `loadPalettes`
    appear only in `world.cs` (`Load`) and `mask.cs` (new world /
    restore world) [data]. Entering or leaving an interior changes only
    the camera, not the palette [traced]; inside an interior the frame
    still ends on the object palette. A world change (portal, load)
    runs the whole of 1a again, so the bars take the new world's
    colours from the first frame of play. After the main menu or the
    options screen (which put `Menus.ppl` in), the first frame of the
    3D view puts the world palette back, because it differs [traced].
    If a script did call `loadPalette` during play, the panels would
    show that file only until the next frame's switch (1b) put the
    object palette back [traced logic; no shipped script does it].
1e. **Where the file is read from.** By name, through the open resource
    folders: both the world's `resource` folder and `GAME/Resource` are
    open at play time and both hold a byte-identical copy of every
    world palette [data]; which one wins was not traced and makes no
    difference. Name case doesn't matter (`Castle.ppl` finds
    `castle.ppl`) [inferred: Windows file names].
1f. **If it is missing.** `loadPalette` writes "loadPalette: Could not
    load palette" to the console and leaves the window as it was;
    `loadPalettes` leaves that slot empty, and the frame's switch skips
    an empty slot [traced]. So with no object palette the panels keep
    the last palette that did load (the interior or terrain one, else
    the `loadingPalette`, else `black.ppl`: all black). The rebuild
    should log the missing name and still draw the panels with any
    world palette it found.
1g. **Conversation captions** are drawn by the game-play screen itself
    (the caption box `IDDLG_CAPTION`, rules 46–52) in `console.pft`, so
    they, their stone strip and their fonts use the same world palette
    as the bars [traced: the screen has no palette of its own]. There
    is no separate conversation palette.
1h. **Saved-game pictures** (the 160 × 120 thumbnail in the load and
    save menus) are stored in the colours of the world they were taken
    in and recoloured to the current palette when shown; the program
    holds its own list of world palette names for this (`daventry.ppl`
    … `temple04.ppl`, `dwarf.ppl`, `icelord.ppl`, `castle.ppl`,
    `exit.ppl`) [traced; detail not needed for the bars].
1i. **Each world's pictures are made for its own palette** [data]. Every
    world's `8gui` folder has its own copy of the panel pictures, and
    the copies differ byte for byte: the artists converted each picture
    to the nearest colours of that world's palette. So the grey stone,
    the slots and the frames come out almost the same grey in every
    world, while the meters differ a little (main colour of `4red.pbm`:
    Daventry 152, 8, 0; Dead City 224, 24, 0; Swamp 184, 0, 0; of
    `4green.pbm`: 0, 104, 0; 16, 72, 0; 8, 72, 0). **Always pair a
    world's pictures with that same world's palette**: Daventry's
    pictures in the Dead City palette give purple and orange.

## The panels on the game-play screen

7. Ten top-level dialogs in `Gplayscr.gui` [data], with what each is
   for [traced unless marked]:

   | Layout id | Rectangle (640 × 480) | What it is |
   | --- | --- | --- |
   | `IDCTL_INVBART` | (0, 0)-(640, 64) | **Top bar**: mask button (left), quest-item strip `IDCTL_INVBAR_QUEST` (7 slots + 2 arrows), gold button and count (right) |
   | `IDCTL_INVBARL` | (0, 416)-(192, 480) | **Bottom left**: armour, hand (short) weapon, ranged (long) weapon, each with its number |
   | `IDCTL_HEALTHBAR` | (192, 416)-(448, 466) | **Eight health/potion slots** with counts (the name says "health bar" but it holds items) |
   | `IDCTL_METERS` | (192, 466)-(448, 480) | **Meters**: red health bar (left), level number (centre), green experience bar (right), level-up flame |
   | `IDCTL_INVBARR` | (448, 416)-(640, 480) | **Bottom right**: rope, rocks and count, small map icon |
   | `IDCTL_MAPWIN` | (0, 64)-(320, 416) | **Map window** (magic map; M6) |
   | `IDCTL_VIEWWIN` | (320, 64)-(640, 400) | The free view area to the right of the map; the screen keeps it between the top bar's bottom and the bottom bar's top [traced]; not drawn itself [inferred] |
   | `IDDLG_CAPTION` | (320, 400)-(640, 416) | **Caption strip** |
   | `IDCTL_POPUP` | (140, 120)-(510, 320) | **Help popup** (yes/no/cancel, "Turn off help"; Daventry help dialogs) |
   | `IDCTL_TIMER` | (304, 352)-(640, 400) | **Status icons** bottom right: spinning mask (handsOff), hook down / hook up (rope can be used here), and the four potion timers (`Clar`, `Inv`, `Str`, `vul`) |

8. **Panel groups.** The game moves the panels in groups, each a bit of
   one "shown" set [traced]:
   - **Top**: the top bar.
   - **Bottom**: bottom left, bottom right, the eight slots and the
     meters together. Asking for any one of the first three moves all
     three plus the meters.
   - **Meters alone**: the meters can be shown without the rest of the
     bottom bar (rule 17).
   - **Map**, **caption**, **popup**: each on its own.
   The set is saved in saved games (`activeScreenElements`) [traced].
9. **Sliding.** Panels don't pop; they slide [traced]:
   - Top bar: slides up off the top of the screen to hide, down to show.
     Sound `intrup.aud` when it starts to show, `intrdown.aud` when it
     starts to hide.
   - Bottom panels (left, right, slots, meters): slide down off the
     bottom edge. No sound of their own.
   - Map window: slides off to the left. Sound `map.aud` on show,
     `mapbgone.aud` on hide.
   - A slide takes **0.2 s** (the speed is set so the whole distance is
     covered in 200 ms), counted in real (wall-clock) milliseconds, not
     game time. A panel asked to show while it is already showing or
     sliding in is left alone; the same for hiding.
   - The hidden top bar sits 1/480 of the screen above its own height
     (just off screen); the map sits 50/640 of the screen beyond its own
     width to the left [traced values].

## When panels show and hide

10. **Bottom bar key.** The "Toggle Interface" key (default **Space**)
    puts the whole bottom bar away or brings it back: if it is hidden it
    slides in, otherwise it slides out [traced]. Defaults come from
    `movekeys.kq`, whose 13 key rows are, in order: up, down, right,
    left, end, insert, delete, home, **tab** (Open/Close Map), shift,
    control, alt (Put Away), **space** (Toggle Interface) [data, matched
    to the `Movekey.gui` rows; traced that row 8 is the map key and row
    12 the bottom-bar key]. The manual says the same: Space for the
    bottom bar, Tab for the map [manual].
11. **Map key** (default **Tab**): toggles the map window, **only if
    Connor carries the magic map** [traced]. Not needed in early Daventry.
    (Exactly: only if the current map is active; nothing else is
    checked, not even `handsOff`: `magic-map.md` rules 2, 30.)
12. **The top bar follows the mouse** [traced]:
    - It slides down when the pointer reaches the **very top row** of
      the screen (y = 0).
    - It slides up again when the pointer goes **below y 60** (in the
      640 × 480 layout), unless it is in its "popped" time (rule 13).
    - It does not come down while the right mouse button is held (camera
      drag), while it is already moving, in the demo version, or while
      the pointer is one of the attack, rope or similar cursors (four
      cursors: `IDBMA_CURS03`, `CURS05`, `CURS06` (rope), `CURS07`); only
      the play pointer and item pointers bring it down. The manual says
      the attack cursors don't open it [manual].
13. **Popping for news.** When a quest item or a mask piece goes into
    the inventory, the top bar slides down by itself and slides up after
    **1.5 s** [traced]. When health or experience changes while the
    meters are hidden, the meters slide up by themselves and away after
    1.5 s (rule 17).
14. **Escape** opens the main menu (also clicking the mask button),
    except during `handsOff` (then Escape does nothing). **F1** shows the
    help popup, or closes it if open; Enter closes it; left/right arrows
    page through it while it is open [traced]. Opening the menu first
    saves a 160 × 120 thumbnail of the screen for the save screen
    (`current.bin`) [traced]. **The user's check in the original
    (2026-10-18):** showing the top or bottom bar does not interrupt
    play; only Escape (the main menu) takes the player out of the game
    screen, and **the game stops running while the main menu is open**
    (the user's check, 2026-10-18; traced 2026-10-21, `main-menu.md`
    rules 7–9). The bars never pause.
    (Our build quits on Escape as a stand-in.)
15. **handsOff and cutscenes** (`anim-scripts.md` rule 21; STATUS.md
    "GUI panels to hide") [traced]:
    - `handsOff 1`: if the bottom bar is showing, that is remembered and
      the **bottom bar** slides away; the **map** is hidden too; the
      spinning-mask icon appears in the status icons. The **top bar is
      not touched**, but item clicks on it are ignored during handsOff
      (`inventory.md` rule 9).
    - `handsOff 0`: the bottom bar slides back **only if it was showing**
      before; the mask icon goes. The map is not brought back.
    - Scripts can also move panels themselves: `setLowerGui 0|1` (bottom
      bar down/up), `setUpperGui 0|1` (top bar), `setMapGui 0|1`;
      `getGuiStatus` prints the five groups [traced].
16. **Conversations** (`KQConner::conversation`): when one starts the
    bottom bar slides away (remembering it was shown); when it ends it
    comes back if it was shown and is now hidden. Neither happens during
    handsOff [traced].
17. **First person** (F2) does **not** hide or move any panel: nothing
    on that path touches the panels [traced, by the list of everything
    that moves them]. Captions show as usual.
18. **Captions** show the caption strip while a line plays and hide it
    when the queue empties (`talking.md` rule 10); with
    `Options::Captions` false the strip never shows [traced].
19. **Screen opens** (world loaded, back from a menu): the saved panel
    set is shown and the rest hidden [traced].

## The meters (health, experience, level)

20. Each bar is its picture **stretched** to a width [traced; corrected
    2026-10-24, Q108 b: not cut]:
    - shown width = the bar's full width × **percent** / 100, cut down to
      whole pixels; a percent that isn't zero shows at least 1 pixel;
    - the bar's picture (`4red.pbm`, `4green.pbm`) is stretched to
      shown width × the bar's height (rule 20a), from the bar's left
      edge;
    - the rest of the bar's width is drawn with **`4black.pbm`** (tag
      351; every bar uses it, it is not in the layout), stretched to the
      remaining width × the bar's height. If either picture is missing
      the bar draws nothing;
    - the percent is kept between 0 and 100.
20a. **How a picture fills its box** (traced 2026-10-24, Q108 b), for
    every picture control in the layouts:
    - with the picture's **modifier 0x8000** (120 of the 124 pictures in
      `Gplayscr.gui`, every gameplay panel among them): the whole
      picture is **stretched to the box** (its width and height
      separately), nearest pixel, no smoothing. So the 6 × 6 `4red.pbm`
      fills its 86 × 6 bar, the 52 × 52 `2mapblnk.pbm` fills its
      56 × 56 box (the caption's `stontran.pbm` is not drawn by this
      rule: it is tiled, last point below);
    - **without** 0x8000 (only the save, load and main-menu warning
      backgrounds `sl_win2.pbm`, `gm_win2.pbm`): drawn **once at its own
      size** from the box's top-left corner; not tiled, not stretched;
    - not tiled in either case (a picture control is never tiled; only
      the caption dialog's background is, last point);
    - the **modifier**'s two lowest bits mirror the picture (1 left to
      right, 2 top to bottom); no layout sets them (all modifiers are
      0x8000 or 0) [traced];
    - the **attribute**'s lowest three bits are the drawing kind:
      **1 = colour 0 not drawn** (see-through), 0 = every pixel drawn
      [traced the kinds; that the layout's attribute is the number used:
      inferred from the values]. Only the caption's `stontran.pbm` has
      attribute 0x9 (kind 1, its colour-0 pixels let the game show
      through); all others are 0x8 (kind 0, solid). What bit 0x8 means
      was not traced.
    - **The caption box's stone is the exception: it is tiled, not
      stretched** (traced 2026-10-24, Q168; this point first inferred
      "stretched like the others", which was wrong and looked broken and
      checkered when built). The stone is the caption dialog's own
      background picture, not a picture control, and a dialog draws its
      background in one of two ways chosen by the dialog itself: by the
      rule above, or **tiled**. Only the caption dialog asks for tiled
      (no other dialog in the program does). Tiled means: the 64 × 64
      `stontran.pbm` is drawn **at its own size**, again and again,
      **from the box's top-left corner**, left to right along a row in
      steps of its width, then the next row down in steps of its height,
      until the box's width and height are covered; the copies at the
      right and bottom edges are **cut** by the box (only the part
      inside is drawn) [traced]. The caption box is 320 wide at
      640 × 480, so five whole copies side by side; with up to three
      rows of text (rule 48: rows × 15 + 4 high) the box is less than
      64 high, so only the stone's top part shows, one row of copies
      [inferred from the box size; the box's exact drawn height was not
      re-traced]. The tiles always start at the box's top-left, also
      when the box grows upward (rule 48). How the 64-pixel tile is
      scaled at 800 × 600 follows open question 1 (not traced); the
      tiled build at × 1.25, nearest pixel, looked right to the user
      against the original. Whether colour 0 stays see-through in the
      tiled drawing (attribute kind 1): not traced (not sure).
    Bars: health `IDCTL_METERS_HEALTH` (14, 4)-(100, 10), 86 × 6 pixels,
    `4red.pbm`; experience `IDCTL_METERS_EXP` (156, 4)-(242, 10),
    `4green.pbm` [data]. Frame pieces `4top`, `4bottom`, `4left`,
    `4right`, `4center` around them [data].
21. **Health percent** = health × 100 / (10 × level), whole-number
    division; at least 1 while health isn't 0; **0 while Connor is
    dead** (dying states) [traced].
22. **Experience percent** = experience × 100 / the current level's
    threshold (10 × 2^(level − 1), 150,000 from level 15; rule 26),
    whole-number division. It is only sent when the experience number
    changed [traced], and always when Connor is added to a world
    (rule 26a).
23. **Level** is written as a whole number in `IDCTL_METERS_LEVEL`
    (font `14shdw.pft`, centred) [traced, data].
24. **When they update**: after every stats change on Connor (anything
    that sends him the stats event: a health item, a script's
    `KQStatsChangeRequestEvent`, damage, a level-up), all five values
    are sent at once: armour, hand-weapon and ranged numbers, health
    percent, experience percent (if changed), level [traced].
25. **Pop-up.** Setting the health or experience percent while the
    meters are hidden makes them slide up and **slide away after 1.5 s**
    (real time) [traced]. When the bottom bar is brought back the
    meters stay (the timer is cleared). The manual's "if the bottom bar
    is closed and you are in combat, the meters pop up until combat is
    complete" [manual] is this rule seen from outside: every hit changes
    health and renews the 1.5 s [inferred].
26. **Level-up flame** `IDCTL_METERS_LEVELUP` (`flame14.pba`) starts
    hidden. **It plays, with `LEVEL.AUD`, whenever the level number the
    meters show changes** (traced 2026-10-22, Q149): each time the five
    values are sent (rule 24) the panel compares the new level with the
    one it shows; if they differ, the flame is shown and started again
    from its first frame (timed on the real-time clock), and the panel's
    own sound plays (`LEVEL.AUD`, the user's check). The same number
    again does nothing. So it plays on a level-up (rule 30), and also
    after `KQConner::setLevel` at the **next** stats change (rule 30a).
    The values are also sent when Connor arrives in a world; whether the
    panel's shown level is then still the old one (no flame) or empty
    (flame and sound at every world start or load) was not traced
    (check in the original: is `LEVEL.AUD` heard when a game starts or
    a saved game loads?). How fast the flame plays, whether it repeats
    and when it is hidden again: not traced. If the meters are hidden,
    the level change makes them pop up as in rule 25 (traced).

26a. **The experience sound `STATS.AUD`** (traced 2026-10-22, Q159;
    the file confirmed by the user's check). It plays when the
    experience percent is set (rule 22) and the new percent (0–100) is
    **not 0** and **at least the last percent shown**. That "last
    percent" starts at 0 when the program starts and is never reset,
    not even by a world change or a load.
    - **When the percent is set:** after a stats change in which the
      experience number changed (any source: the stats event, a kill,
      an item's built-in experience), and **every time Connor is added
      to a world** (game start, arriving from another world, loading a
      saved game: all five values are sent then, without the "only if
      changed" test of rule 22).
    - So: a gain that raises the percent plays it; a gain too small to
      change the whole-number percent plays it too (equal counts), but
      not while it is still 0 % (e.g. 20 experience at level 14,
      threshold 81,920). A loss that lowers the percent is silent.
      Health and the other stats alone never play it.
    - **With a level-up** in the same change: the experience left over
      is measured against the new, doubled threshold, so the percent
      usually drops and only `LEVEL.AUD` plays. If the new percent is
      still at least the old one, both play, `STATS.AUD` first.
    - **Inferred, not checked in the original:** arriving in a world
      (or loading) with experience above 0 % plays `STATS.AUD` once.
    - **How:** the interface sound group (as `LEVEL.AUD` and the other
      interface sounds), no position, volume 1.0; each play is a new
      sound, so two can overlap [inferred]. Nothing checks `handsOff`, a
      conversation or whether the meters are hidden. The meters pop up
      (rule 25).

## Connor's stats as the panel shows them

27. Connor's five stats, in the order of `stats=` and of the stats event
    (corrects nothing in `anim-scripts.md` rule 19; adds the names)
    [traced, from the panel]:
    1. **health**;
    2. the **hand-weapon number** (shown on the short-weapon slot);
    3. the **ranged-weapon number** (shown on the long-weapon slot);
    4. the **armour number** (protection, capped at 100; 100 = no
       damage, `doors.md` rule 19);
    5. **experience**.
28. Start (`GAME/Kq/Connor.kq`): `stats=10 10 0 0 0`, `level=1`
    [data]: health 10 of 10, hand weapon 10, ranged 0, armour 0, no
    experience.
29. **Maximum health** = 10 × level (the manual: "each level is worth 10
    points of health") [traced, manual].
30. **Level-up** (`anim-scripts.md` rule 19, confirmed): after any stats
    change, once: if experience is **more than** the threshold (10 ×
    2^(level − 1); 150,000 from level 15 on), experience drops by the
    threshold, the level goes up 1, health is set to the new maximum,
    hand weapon +5, ranged +2, armour +1 (cap 100). Reaching level 2 also
    sends Connor a help event (the Daventry help popup about levels;
    not traced further) [traced]. The manual's "the experience points
    required are doubled until level 15, then fixed" matches [manual].
    At most **one** level-up per stats change, even if the experience
    is more than two thresholds (traced again 2026-10-22, Q149 e): the
    extra stays as experience and the next stats change levels him up
    again. Each item's built-in experience is its own stats change, so
    temple 1's start-up items (`HearingHorn` +10,000, then
    `DecipheringAmulet` +25,000) give two level-ups (1 → 2 → 3) with
    their +5 / +2 / +1 each, before `setLevel 43` (B's reading is right).
30a. **`KQConner::setLevel <n>`** (traced 2026-10-22, Q149 a) sets **only
    the level number**. Health is not changed (maximum health is not
    stored: it is always 10 × level), the other stats and the experience are not
    touched; nothing is sent to the meters, no flame, no sound, no help.
    The new level counts from the next stats change: health is capped at
    10 × the new level, the threshold is the new level's (150,000 from
    level 15 up), and the meters show the new number, which plays the
    flame and `LEVEL.AUD` (rule 26). In every `conStart.cs` the
    `KQStatsChangeRequestEvent` right after `setLevel` is that change.
    The level-2 help comes only from a real level-up to 2 (rule 30).
    With a wrong number of words it writes its usage line and does
    nothing. Whether `$Testing` is true when a game starts on another
    world: not traced.
31. Equipment adds to stats 2–4 while worn (`inventory.md` rules 13–15);
    so the numbers on the slots are Connor's own stats, not the item's
    numbers.
32. **The red "alternative" number.** Each of the three slot numbers
    has a second text in `14red.pft` (`IDCTL_INVBAR_*_ALTTXT`) that is
    turned on by a flag sent with the number [traced]. The flags come
    from three switches on Connor that the potions of shield and
    strength set while they last [traced in `potions.md` rules 17, 21,
    37–39: Strength reddens the hand number, the shield the armour
    number; the ranged one is never used].
33. **A changed slot number flashes**: when the armour, hand or ranged
    number differs from the last one shown, the slot's `fx.pba`
    animation plays once over it [traced].

## The bottom-left slots (armour, weapons)

34. `IDCTL_INVBAR_ARMOR`, `_SHORT`, `_LONG` show the picture of the
    item worn in that place (its type's `GUIBitmap` in `Invinfo.itl`,
    e.g. dagger `IDBMP_2SORD1` → `2sord1.pbm`), or the empty picture
    (`2armr0`, `2sord0`, `2rng0`) [traced, data]. At the start Connor
    carries the placeholder items `INVITEM_Fists`, `INVITEM_NoRanged`,
    `INVITEM_NoArmor`, whose pictures are those empty ones (`inventory.md`
    rule 3).
35. **Which armour picture**: only body armour (`Armor`), under-armour
    (`UnderArmor`), full suits (`FullArmor`) and `GodArmor` change the
    armour picture; gloves, boots, helmet and pants change only the
    number. When both a chain-mail suit (under-armour) and body armour
    are worn, the picture is the suit's if the body armour is leather
    or the chain shirt, and the body armour's if it is plate or bronze
    plate [traced].
36. Numbers: `IDCTL_INVBAR_ARMORTEXT` etc., font `14shdw.pft`,
    right/top at (56, 5), (120, 5), (184, 5) inside the dialog [data].
37. Clicking a slot: `inventory.md` rule 9 (clicking the hand weapon
    readies it; fighting is M5).

## The eight health/potion slots

38. Slots 1–8 (left to right, `IDCTL_HEALTHBAR_1` … `_8`, 32 × 50 each,
    empty picture `3empty.pbm`) hold one stack each [traced, data]:
    1 `INVITEM_Mushroom`, 2 `INVITEM_Crystal`, 3 `INVITEM_SacredWater`,
    4 `INVITEM_ElixerOfLife`, 5 `INVITEM_Invulnerable` (shield),
    6 `INVITEM_Strength`, 7 `INVITEM_Clarity` (reveal),
    8 `INVITEM_Invisible`. (Health items by strength 5, 50, 100, 1,000
    health; the manual's order "least to most powerful" [manual].)
39. Each slot shows its count with `14.pft` at its top right
    (`_1TEXT` … `_8TEXT`); a stack whose count reaches 0 empties the slot
    [traced, data].
40. Keys **1–8** (and keypad 1–8) use the matching slot, as a click on it
    would [traced; key-to-slot order inferred and matching the manual].

## The top bar

41. **Mask button** `IDCTL_INVBAR_MASK`: shows the picture of the mask
    piece type carried (`2msk1` … `2msk5`); left click opens the main
    menu; right click: Connor says how many pieces he has (the item's
    description, `talking.md` rule 20) [traced, manual].
42. **Quest strip**: 7 slots (`IDCTL_INVBAR_QUEST1` … `7`, 64 × 64,
    empty `1empty.pbm`) between a left and right arrow [data]. Rules in
    `inventory.md` rules 5–7.
43. **Gold** `IDCTL_INVBAR_GOLD`: the coins picture depends on how many
    silver coins Connor has: none `2gold0`, 1–24 `2gold1`, 25–49
    `2gold2`, 50 or more `2gold3` [traced]. The count is the text
    `IDCTL_INVBAR_GOLDTEXT` (right/top at (632, 5)).
44. A slot whose item is new, or whose count went **up**, **blinks**:
    it switches between the item picture and the empty picture every
    0.25 s (real time), 5 times [traced].
    **Except the mini-map slot** (Q150 d, traced 2026-10-22): it never
    blinks; it goes straight to the live map (`magic-map.md` rule 27).

## The bottom-right panel

45. `IDCTL_INVBAR_ROPE` (rope and hook, `2rope0` empty / `2rope1`
    carried), `IDCTL_INVBAR_ROCKS` (`2rocks0` … `3`) with the rock count,
    `IDCTL_INVBAR_MINIMAP` (`2mapblnk.pbm` while the map is not active;
    once it is, the slot shows a live mini-map centred on Connor with his
    arrow: `magic-map.md` rule 27) [traced which item goes where; the
    rocks picture levels were not traced].

## The caption strip (answers `talking.md` open question 4 and Q107 a)

46. **What**: the `IDDLG_CAPTION` dialog: font `console.pft` (the
    world's, 15 pixels high, spacing 0), background picture
    `stontran.pbm`, text 2 pixels down [data].
47. **Wrapping** [traced]: the line is cut into rows, greedily, from the
    start:
    - widths are summed character by character (each glyph's width
      times the font's horizontal scale, plus the font's spacing after
      each character that fits; spaces count like any character);
    - when adding a character would make the row **wider than the
      box's width**, the row ends at the **last space before it** (the
      space is dropped), and the next row starts after that space;
    - a row may end with spaces that still fit (they are kept).
    The width used is the caption box's own width in screen pixels.
    In the layout that is 320 (640 × 480). **The shipped `Options.cs`
    says `set800mode 1`**; if the layout is scaled with the screen at
    800 × 600, the box is **400** pixels wide, which fits the user's
    measured 365–418 and B's stand-in [inferred; how the box is scaled
    at 800 × 600 was not traced]. The font is not scaled.
48. **Height**: one row per 15-pixel font height; the strip's top is set
    to (its bottom) − rows × 15 − 4, so it **grows upwards** [traced].
49. **Each row** is drawn centred in the box's width, in its own 15-pixel
    slot, from the top down [traced].
50. **Width of the stone**: the strip covers the **whole box** (its
    picture is the dialog's background), not the text [traced that the
    picture belongs to the box; it is **tiled** over the box at its own
    size from the top-left, rule 20a (traced 2026-10-24, Q168; first
    inferred "stretched", wrong)]. **Checked by the user in the original
    (2026-10-18):** a short line ("'Tis empty.") has a strip as wide as a
    long line. So the strip is the box's width, not fitted to the text.
51. **Where** [traced]: the box keeps its own width and height and is
    moved every time panels move:
    - its **bottom edge** = the **top edge of the highest bottom panel**
      as it stands now (the bottom-left bar, the eight slots, or the
      meters); with the bottom bar away they sit at the screen's bottom,
      so the caption is at the very bottom (the user's observation);
      with the bar up, the slots' top (y 416);
    - its **centre** is half-way between the map window's right edge and
      the screen's right edge. With the map open (right edge 320/640)
      that is x 480 of 640; with the map away (slid left, right edge at
      −50/640) it is 0.461 of the screen width: about 295 of 640, or 369
      of 800 — a little left of the middle [traced rule; **checked by the
      user in the original (2026-10-18): with the map closed the strip
      is slightly left of the centre**].
52. Item descriptions in the inventory have no caption (`talking.md`
    rule 20).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| slide time | 0.2 s (real time) | traced |
| pop-up time (top bar on news, meters on change) | 1.5 s (real time) | traced |
| top bar appears / hides | pointer at y 0 / below y 60 (of 480) | traced |
| blink | 0.25 s per change, 5 changes | traced |
| meter width | width × percent / 100, ≥ 1 px if percent ≠ 0 | traced |
| health percent | health × 100 / (10 × level) | traced |
| experience percent | exp × 100 / (10 × 2^(level−1)), 150,000 from level 15 | traced |
| gold picture steps | 1, 25, 50 coins | traced |
| caption row | 15 px (font height) + 4 px strip padding | traced |
| caption wrap width | box width: 320 at 640 × 480 (400 at 800 × 600, inferred) | traced / inferred |
| hidden top bar | 1/480 screen above | traced |
| hidden map | 50/640 screen beyond the left edge | traced |

## Data used

- `GAME/Kq/Gplayscr.kq`, `KqGui.kq`, `Mainmenu.kq`, `Options.kq`,
  `Graphics.kq`, `Loading.kq`, `MASK/movekeys.kq`, `MASK/Options.cs`.
- `GAME/Resource/Gplayscr.gui`, `kqGuiTag.TTAG`.
- `<world>/8gui/*` (pictures, cursors, fonts).
- `GAME/Kq/Invinfo.itl` (`GUIBitmap`, `cursor`, `category`).
- `GAME/Kq/Connor.kq` (`stats=`, `level=`, `[ConnerInventory]`).

## Edge cases

- With no world palette loaded the screen keeps the last palette that
  did load (rule 1f; at worst `black.ppl`, all black); the menus never
  use the world palette.
- The map key does nothing without the magic map.
- Escape during handsOff does nothing; Space, Tab and 1–8 are not
  checked for handsOff by the screen itself [traced]; the item use they
  lead to is refused during handsOff (`inventory.md` rule 9). Whether
  Space can bring the bottom bar back during a cutscene in the original
  is not settled.
- A caption line with a single word wider than the box: no space to
  break at; what the original does then was not traced (no data line
  is known to do it). Do instead: break the word at the box edge.

## Do not reproduce

- Slides and pop-up timers count **real** milliseconds; ours may use
  real time too (they are interface, not game rules), but pause them
  when the game is paused.
- The caption's off-centre position with the map closed (rule 51) is a
  side effect of the layout; reproduce it only if the user's check says
  the original does it.

## How to check

**Checks 3–6 below were done by the user in the original on
2026-10-18; all as described.** Check 7 too (see there).

1. **In our logs (B):** every panel show/hide with the reason (key,
   pointer, pop-up, handsOff, conversation, script) and the time; per
   stats change the five values sent (armour, hand, ranged, health %,
   exp %, level) and the bar widths in pixels; per caption the rows,
   their widths, the box width, its top and bottom.
2. **Daventry, in our build:** run the bricks puzzle (`bricks.cs`, +50
   experience at level 1): the log should show level 2, health 20 of
   20, experience 40, experience bar 40 × 100 / 20 = 200 → 100 %
   (full), health 100 %, hand weapon 15, ranged 2, armour 1.
3. **In the original (user):** at the start, the bottom bar shows
   armour 0, hand weapon 10, ranged 0, level 1, full red bar. Press
   Space: the bottom bar slides away in about a fifth of a second; press
   again: it comes back.
4. **In the original:** with the bottom bar away, eat a mushroom
   (key 1) or take damage: the meters alone slide up and go away about
   1.5 s later.
5. **In the original:** move the pointer slowly to the top edge: the
   top bar comes down only at the very top row; move down past about an
   eighth of the screen: it goes up. Pick up a quest item: the top bar
   comes down by itself and goes up 1.5 s later, and the new slot
   blinks.
6. **In the original:** click `smlchst3` twice and then once more
   ("'Tis empty.", a short line): is the stone strip still as wide as
   for a long line (rule 50)? With the map closed, is the strip a little
   left of the middle (rule 51)?
7. **In the original:** start the wizard's cutscene or any `handsOff`
   scene with the bottom bar up: it slides away and the spinning mask
   shows; afterwards it comes back. With the bar put away first, it
   stays away.
   **User (2026-10-18), brief look:** the bar goes away when a cutscene
   starts. The spinning mask shows only in scenes that take control
   away (`handsOff`); some of the wizard's scenes let Connor walk away
   and show no mask. That fits rules 15–16: those are conversations,
   which hide the bottom bar but don't add the mask. **Also checked by
   the user:** a bar that was showing comes back when the cutscene
   ends; a bar that was put away first stays away.
8. **The world palette (Q118).** In our build: every world, including
   DeadCity, CastleD, IceWorld, SnowExit and Temple1–4, should log the
   palette name it took from `world.cs` (`objectPalette`, e.g.
   `newdead.ppl`) and draw the bars and the caption strip. **In the
   original (user):** compare the bars in Daventry and in the Dead City
   (or the Ice World): the same pictures are coloured by each world's
   own palette, so the stone, the gold letters and the meters may look
   a little different from world to world; our build should show the
   same difference. Open the main menu with Escape and go back: the
   bars are in the world's colours again straight away (rule 1d).
   **User (2026-10-19), Daventry, Dead City, Swamp:** only the health
   and experience meters differ, in a slightly different red and
   green; everything else looks the same. That matches the data
   (rule 1i). Escape to the main menu and back: nothing changes, the
   bars are in the world's colours at once (rule 1d).

## Open questions

1. How layout rectangles are scaled at 800 × 600 (rule 47); the wrap
   width 400 is inferred.
2. ~~Whether the world clock stops behind the main menu.~~ It does
   (the user's check; rule 14).
3. ~~When the level-up flame plays (rule 26)~~: answered 2026-10-22
   (whenever the shown level changes); still open: its speed, repeats,
   when it hides, and whether it plays at a world start or load. (~~What the status icons'
   hook-up / hook-down mean~~: answered in `ropeandhook.md` rule 7:
   `HookUp` at the foot of a rope, `HookDown` at the top, only with the
   rope carried and not while on the rope.)
4. The rocks picture steps (rule 45). (~~The minimap icon~~: a live
   mini-map, `magic-map.md` rule 27.)
5. Font flag bit 6 (0x41) and the `sl`, `black`, `shdw`, `gshdw`, `red`
   font variants beyond their use in the layouts (Q106): not traced.
   From the layouts: `14shdw` for normal slot numbers, `14red` for the
   boosted numbers, `14` for the health-slot counts, `sl`/`black` only
   in menus and credits.
6. ~~How the caption picture fills the box and what modifier 0x8000
   and attribute 0x8 / 0x9 mean~~: rule 20a (2026-10-24): 0x8000 =
   stretched to the box, attribute kind 1 = colour 0 see-through. The
   caption dialog's own picture is **not** drawn that way: it is tiled
   (rule 20a, traced 2026-10-24, Q168). Still open: bit 0x8 of the
   attribute, and whether colour 0 stays see-through in the tiled
   drawing.
