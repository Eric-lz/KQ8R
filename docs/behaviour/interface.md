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
     for the in-game screen);
   - `cursor=IDBMA_CURS04`: the normal play pointer is `curs04.pba`.
2. The menus (`Mainmenu.kq`, `Options.kq`, `Graphics.kq`, `movekeys.kq`)
   use the volume `game / $KQGame::BitDepth GUI` (`GAME/8Gui`),
   `palette=Menus.ppl`, `useWorldPal=0`, cursor `IDBMA_CURSGUI`; the
   loading screen uses `loading.ppl` [data].
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
    screen. (Our build quits on Escape as a stand-in.) Whether the world
    clock is stopped behind the menu was not traced.
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

20. Each bar is a picture cut to a width [traced]:
    - shown width = the bar's full width × **percent** / 100, cut down to
      whole pixels; a percent that isn't zero shows at least 1 pixel;
    - the rest of the bar's width is drawn with the bar's "empty"
      picture if it has one;
    - the percent is kept between 0 and 100.
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
    changed [traced].
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
    hidden; when it plays was not traced (likely on a level-up).

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
31. Equipment adds to stats 2–4 while worn (`inventory.md` rules 13–15);
    so the numbers on the slots are Connor's own stats, not the item's
    numbers.
32. **The red "alternative" number.** Each of the three slot numbers
    has a second text in `14red.pft` (`IDCTL_INVBAR_*_ALTTXT`) that is
    turned on by a flag sent with the number [traced]. The flags come
    from three switches on Connor that the potions of shield and
    strength set while they last [inferred: the stats code changes the
    armour and hand-weapon numbers when those switches are on; the
    manual: "protection increases to 50% or by 50%", "hand weapon stat
    to 150%"]. M5.
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

## The bottom-right panel

45. `IDCTL_INVBAR_ROPE` (rope and hook, `2rope0` empty / `2rope1`
    carried), `IDCTL_INVBAR_ROCKS` (`2rocks0` … `3`) with the rock count,
    `IDCTL_INVBAR_MINIMAP` (`2mapblnk.pbm`, the map icon; it changes when
    the magic map is carried) [traced which item goes where; the rocks
    picture levels were not traced].

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
    picture belongs to the box; how the picture fills it, tiled or
    stretched, was not traced]. **Checked by the user in the original
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

- With no world palette loaded nothing on this screen has its colours;
  the menus never use the world palette.
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
2026-10-18; all as described.** Check 7 is not done yet.

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

## Open questions

1. How layout rectangles are scaled at 800 × 600 (rule 47); the wrap
   width 400 is inferred.
2. Whether the world clock stops behind the main menu (rule 14; the
   bars don't interrupt play, the user's check).
3. When the level-up flame plays (rule 26); what the status icons'
   hook-up / hook-down mean exactly (rope places; M5/M6).
4. The rocks picture steps, and the minimap icon (rule 45).
5. Font flag bit 6 (0x41) and the `sl`, `black`, `shdw`, `gshdw`, `red`
   font variants beyond their use in the layouts (Q106): not traced.
   From the layouts: `14shdw` for normal slot numbers, `14red` for the
   boosted numbers, `14` for the health-slot counts, `sl`/`black` only
   in menus and credits.
6. How the caption picture fills the box (tiled or stretched) and what
   a picture's modifier 0x8000 and attribute 0x8 / 0x9 mean (Q107 d).
