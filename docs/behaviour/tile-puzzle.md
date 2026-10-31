# The death-hall stone puzzle (Connor's `Tile` state, `tileExit`, `deadHallTeleport`)

Studied from `Mask.exe` (GOG 1.03) and the Dead City data on 2026-10-28
(Q174). Described here in our own words. "Traced" = read in the program;
"data" = read in the game files; "not traced" = not settled.

## Purpose

In Dead City's hall of death Connor must cross an 8 × 8 floor of stones
by jumping from stone to stone, stepping only on the letters D, E, A, T,
H in that order. The scripts only lay out the stones and hand Connor to
the program; the program's `Tile` state does the rest: the jumping, the
lit stone, the checking, the "teleports" and the end.

## Rules

### Setting up (data)

1. **`dhalltrp.cs`** (the hall's trap, on `On`, once per visit because it
   sets `DeadCity::DeadHall::TrapOn` false):
   - the first time (`DeadCity::DeadHall::Comment`): Connor's comment
     (2000.msg, noun 0, verb 0, condition 56), `loadKQ tileshow.kq` and
     the 64 stones `tile00` … `tile77` (rule 3);
   - camera flight, the hall doors (`ltddoor2`, `rtddoor2`) close, Cue 1
     after 5 s.
   - Cue 1: **`deadHallTeleport DeadHall DeadHal2`** (rule 22),
     `KQMap::stopLoc 1`, camera follows Connor, the inner doors
     (`ltddoor4`, `rtddoor4`) open, Connor walks to (238,100, 121,500),
     haze 20,000.
   - Cue 2 and 3: camera shots, doors close, `deleteObject DeadHal2` and
     the copied doors, `move Connor 0 238340 122000 3100 0 0 3.14`.
   - Cue 4: `tileExit 0`, `setState Connor Wait`, **`setState Connor
     Tile`**, `DeadCity::DeadHall::ReturnOk` true. Cue 5: `handsOff 0`.
2. **Every stone must exist by name** before Connor enters `Tile`: when
   the state starts it looks up all 64 names `tile<row><column>` (rows
   and columns 0–7) and copies their positions (traced). There is no
   check for a missing stone in the original (it would crash): **do not
   reproduce**; skip a missing stone and log it.

### The floor (traced + data)

3. The stones (data, `dhalltrp.cs`): `tile<r><c>` stands at
   x = 230,333 + 2,000·c, y = 124,297 + 2,000·r,
   z = 2,871 + 500·r. Row 0 is nearest the start, row 7 nearest the far
   door; the floor climbs 500 per row.
   The letters that matter: **D = `tile07`, E = `tile63`,
   A = `tile21`, T = `tile56`, H = `tile10`** (files `tile_d.kq`,
   `tile_e.kq`, `tile_a.kq`, `tile_t.kq`, `tile_h.kq`). The other
   letters (`tile_a2`, `tile_e2`, `tile_t2`, `tile_g`, …) and the blank
   stones (`notile.kq`, `notile2.kq`) are all "wrong".
4. **The right order is fixed in the program**, by grid position, not by
   letter or file name (traced): D (row 0, column 7) → E (row 6,
   column 3) → A (row 2, column 1) → T (row 5, column 6) → H (row 1,
   column 0). A second D/E/A/T/H elsewhere (`tile_a2`, …) is wrong.
5. **Spots off the stones.** Besides the 64 stones the state knows
   three more rows of spots in every column, with the same x and the
   z of the nearest stone row: two before row 0 (y − 4,000 and
   y − 2,000, called *row −2* and *row −1* below) and one after row 7
   (y + 2,000). Row −1 (y = 122,297) is the **start strip** where
   Connor stands at the beginning (he is put at x 238,340, column 4).
6. **Which spot Connor is on** (traced): the area is x 229,333–245,333
   and y 119,297–139,297 (the spots' outer edge ± 1,000). Inside it, the
   column is ⌊(x − 229,333) / 2,000⌋ and the row index (0 = row −2 …
   9 = row 7) is ⌊(y − 119,297) / 2,000⌋ (rounding toward zero).
   Outside it he is "south" (y too small), "north" (y too big), "west"
   or "east".

### The `Tile` state (traced)

7. **Entering** (`KQMonster::setState Connor Tile`): the stones are
   read (rule 2), the expected stone is set to **D**, the camera is
   told to **hold** (rule 7a), leaving the state is **forbidden** (the
   exit switch is set off, rule 20), and:
   - **with a weapon drawn** (sword or bow mode) he plays `away`, then
     the state asks for the weapon change to empty hands the ordinary
     way. That request meets the state's own refusal (rule 18), so he
     **says "A weapon will not help me here."** (file 0, noun 0, verb
     37, condition 23), and the state then sets his weapon mode to empty
     hands directly (no swap state, only the `away` it already plays).
     (Traced 2026-10-30, Q181; this is where the line the user heard on
     entering comes from.)
   - **with nothing drawn** he plays `stop` and says nothing.
   The stone he faces is lit (rule 10).
7a. **The camera's hold switch** (traced 2026-10-30, Q178): while it is
   on, the camera following Connor **keeps its present height** (the
   height rules don't move it) and its tilt is only the player's tilt
   setting, **not aimed up or down at Connor**; turning and distance
   work as usual. It is on from entering until the end of a
   `teleportdown`, off during the flight that follows (rule 13), and on
   again when that flight ends. (Answers open question 1.)
8. **The keys** (the movement wanted from the controls, as for walking):
   - **turn left / right**: he plays `left` / `right` and turns on the
     spot as when standing; the lit stone follows his facing;
   - **forward (walk or run)**: a **forward jump**;
   - **back**: a **back jump**;
   - nothing: `stop`.
   A jump starts when the wanted movement **changes** (a key press);
   after each landing the remembered movement is cleared, so **holding
   forward keeps him jumping**, one stone per jump. While a jump, a
   landing or a teleport is under way, the keys are ignored.
   Water walking / running words are ignored.
   **The Jump key** (Insert) also starts a jump: the same choice as a
   forward jump (or a back jump if the back key started the last one
   and he hasn't landed since; in practice forward), traced 2026-10-30.
   **Do not reproduce:** the original doesn't check that a jump, landing
   or teleport is under way before acting on the Jump key; ignore it
   then, like the other keys.
   When Connor is south of the area (rule 6) the state lets him walk and
   run normally (the leaning switch is on there; elsewhere it is off,
   `connor-movement.md`).
9. **Choosing the stone** for a jump: from Connor's spot, look at the
   8 neighbours (columns clamped to 0–7; rows include the extra spots of
   rule 5). Take the one whose direction from Connor is **closest to
   the way he faces** (forward jump) or **closest to straight behind
   him** (back jump). Not allowed: his own spot; and from the start
   strip (row −1), the two diagonal spots behind him (row −2). So jumps
   are always to a neighbouring spot, 8 directions.
10. **The lit stone.** The stone a forward jump would land on is drawn
    **at full brightness**: the state turns on that stone's per-object
    "unlit" switch (`object-lighting.md` rule 10, which is corrected by
    this) and turns it off on the previous one. Only real stones (rows
    0–7) are lit. For a back jump the landing stone is lit when the
    jump starts.
11. **The jump.** Connor first **turns on the spot to face a point**
    (the same turn-to-point as `KQConner::walkTo`, `character-movement.md`):
    for a forward jump the target spot; for a back jump the target's
    mirror image through Connor (so his back is to the target). When the
    turn ends he plays the jump movement, whose own steps carry him
    (data, `GAME/Common`):

    | Jump | Movement | Frames, length | Travel (sum of steps) |
    | --- | --- | --- | --- |
    | forward, straight | `tilejump1` (`c_tljmp1`) | 31, 2.0 s | 2,000 forward, 648 down |
    | forward, diagonal | `tilejump2` (`c_tljmp2`) | 36, 2.33 s | 2,840 forward, 674 down |
    | back, straight | `tilebackjump` (`c_tlbjmp`) | 43, 2.8 s | 1,999 back, 698 down |
    | back, diagonal | `tilebackjump2` (`c_tlbjp2`) | 43, 2.8 s | 2,836 back, 698 down |

    "Straight" = same row or same column; else diagonal.
    **His physics during a tile jump** (traced 2026-10-30, Q177;
    corrects "gravity puts him on the stone"):
    - when the jump movement **starts**, **gravity is switched off** and
      so is his **keep-on-ground** switch (no snapping to the terrain);
    - **step up and step down are not changed**: they stay at his usual
      **325** (unlike the ordinary jump, `connor-physics.md` "Jumping"
      rule 3, which sets them to 0);
    - when the jump movement **ends**, gravity and keep-on-ground are
      switched back on, before the landing check (rule 12). The teleport
      movements don't change any of them.
    So the arc is the movement's own steps, moved through the ordinary
    mover with step up / down 325. Each update's move lifts him by the
    step-up first, then goes across, then lowers him by the step-down
    (`connor-physics.md`), so an edge up to 325 above his feet is
    climbed. **That is how he gets onto a stone 500 higher**: on a north
    jump he rises about 510 on the movement's own steps, which leaves the
    stone's top edge only a few units above his feet when he reaches it;
    the step-up clears it, the step-down puts him on top, and the
    movement's downward steps then stop on the stone's top instead of
    taking him below it. (B's build, which sets step up to 0, comes about
    25 short.) Hitting the side of a stone in this state is never a
    crash (crashes happen only in walking and the ordinary Jump); he just
    slides along it.
12. **Landing.** When the jump movement ends he plays `stop`, the lit
    stone is updated, and his spot (rule 6) is checked:
    - **the expected stone**: right (rule 13);
    - **row −2** (behind the start strip): the game sends the inner door
      `ltddoor4` a click (`ConnorAction`, as if the player clicked it):
      `ddoor4.cs` takes him out (rule 19);
    - **the start strip (row −1)**: nothing, play goes on;
    - **anything else** (any other stone, the row after row 7, or
      outside the area): wrong (rule 14).
13. **A right stone**: forced third person (first person blocked), the
    expected stone moves on, a camera point is set, he plays
    `teleportdown` (1.4 s, sound `poofdown.aud`), and when it ends he is
    **moved to a stone next to the next letter** and the camera flies to
    the point, looking at him; then `teleportup` (1.4 s, `poofup.aud`):

    | Landed on | Next expected | Connor is put on | Camera point |
    | --- | --- | --- | --- |
    | D `tile07` | E `tile63` | `tile73` | above `tile76` |
    | E `tile63` | A `tile21` | `tile22` | above `tile34` |
    | A `tile21` | T `tile56` | `tile46` | above `tile43` |
    | T `tile56` | H `tile10` | `tile11` | above `tile24` |
    | H `tile10` | — (solved) | (237,333, 140,000, 6,900) | (237,333, 135,000, 8,500) |

    "Put on" a stone = that stone's position + 1,000 in z, by a move that
    keeps his heading (he drops onto it); "above" = stone + 2,000 in z.
    **The camera flight** (traced 2026-10-30, Q178; answers open
    question 4): the camera point is made the camera's only spot (as
    `addCamLoc … 1`) when he lands, but the flight starts **when
    `teleportdown` ends**, in the same moment he is moved and
    `teleportup` starts. It is exactly a `flyToCamLoc` with no time
    given (`camera.md` "Camera flights and cuts" rule 4), aimed at
    Connor's new position: it takes **distance from the camera to the
    spot ÷ 5,000 units per second** (N = whole part of time × 15, + 1
    updates), along a **straight line at constant speed**, rolling to 0,
    looking at a point 1,400 above his new position. Just before it the
    hold switch (rule 7a) goes off. So the flight is usually much longer
    than 0.5 s: a camera 15,000 from the spot flies for 3 s
    (example only; the camera's real starting place is wherever
    following left it). When the flight **ends** the camera follows
    Connor again and the hold switch goes back on (not after H). The
    keys work again as soon as `teleportup` ends, even if the camera is
    still flying.
    **First person**: the "forced third person" of a teleport is lifted
    only once **both** `teleportup` and the flight have finished
    (whichever ends last); after H it stays (the state ends).
    So the player never jumps between letters; each letter sends him
    next to the next one. **H** also allows leaving the state, sets the
    variable `DeadCity::Azriel::TrapOn` to `true` (it arms
    `azrieltp.cs`, the meeting with Death beyond the far door), and if he
    was in first person sets `<world>::SetThirdPerson::WasFirstPerson`
    to 1; after `teleportup` he is set to `Wait` (state over).
14. **A wrong stone**: the lit stone goes dark, the expected stone goes
    back to **D**, a camera point is set at (237,333, 118,500, 4,500),
    forced third person, `teleportdown`; then he is put back at the
    **start** (238,340, 122,300, 3,100), the camera flies there, his
    held keys are cleared, `teleportup`. He keeps his heading. Nothing
    else: **no damage, no fall**.
15. **What he says** (after `teleportup` ends, 2000.msg noun 2,
    verb 0): the first teleport of the visit, right or wrong:
    condition 3. Later, the first time he is sent back to the start
    after having had at least one right stone: condition 133. Each
    once per visit (they live in the state, which starts fresh each
    time).
16. **The stones themselves** (data): each plays its one movement
    (`stop`) over and over (5th field 1 = repeating, 2.0 s, 31 frames).
    The letter stones (`tile<x>.kqs`) **dip** about 115 units and come
    back; the blank ones (`notile.kqs`, `notile2.kqs`) **rise** about
    115 and come back, so the floor gently heaves. The cue at the end of
    each cycle moves the stone by the small z number (a relative move):
    `KQMoveEvent 3 0 0 -2.3575 0 0 0 0` (letters) / `… 2.374 …` (blank).
    So the number is a **z distance in world units**, not a turn.
    **It cancels the cycle's leftover** (checked 2026-10-30, Q176, from
    the data and `objects.md` "Loops that move the whole object" rule
    6a): the steps a cycle adds are frames 1–30 (letters −1.297, blank
    +1.297) **plus the last frame's step once more at the wrap**
    (letters +3.665, blank −3.665), so a letter stone ends a cycle
    **+2.368** higher and a blank one **−2.368** lower; the cues take
    −2.3575 / +2.374 off. What is left is **+0.011** per cycle for the
    letters and **+0.006** for the blank ones (a cycle is 31 updates,
    2.07 s): after 10 cycles a stone stands about 0.1 (letters) or 0.06
    (blank) above where it started, at the same point of its bob; after
    an hour about 18 and 10. Leaving out the wrap step (our loader's
    sum) turns the signs round and makes the stones drift 3.7 per cycle.
    Stepping on a stone does not change it.
17. **The Reveal potion** (data): `tileshow.kq` (a hidden clicker object
    in the reveal group, `groupsAlways=2054`) gets `Reveal True` /
    `False` (`potions.md` rule 31); `tileshow.cs` then swaps the
    textures of the five right stones to their `…cl.bmp` versions and
    back (`KQObject::changeBmp`), so the right letters stand out while
    the potion lasts.
18. **Weapon changes** in this state are refused: Connor speaks his
    common line (file 0, noun 0, verb 37, condition 23) and the change
    is cancelled. This happens to **every request that goes through
    Connor's ordinary weapon change** (traced 2026-10-30, Q181), each
    time:
    - the draw key (Delete): with empty hands it asks for the sword (or
      the bow, if he has one and last had it out), so the line is said;
    - the away key (Alt): only acts with a weapon drawn, so in this
      state (empty hands after rule 7) it does nothing, silently;
    - clicking a weapon in the bottom bar (the hand or the ranged slot);
    - scripts' `KQConner::setDoMode 0 | 1 | 4` (it first asks for Wait,
      which is refused silently, then for the weapon change: line) and
      `KQConner::setRequiredWeaponMode` (line);
    - a request for the bow with no bow carried is dropped before that,
      silently.
    All of these except scripts are ignored while the controls are off
    (`handsOff`). Number keys and the mouse wheel were not traced
    separately; if they work through the bottom bar's slots they act
    as the slot clicks.

### Leaving (traced + data)

19. **Back out through the start**: a back jump from the start strip
    lands in row −2, which "clicks" `ltddoor4`; with
    `DeadCity::DeadHall::ReturnOk` true, `ddoor4.cs` runs `tileExit 1`
    (rule 21) and walks Connor back out (camera shots, `move Connor 0
    237333 121000 3500 0 0 0`, a temporary copy of the hall
    `deadhal2.kq` and its doors, `deadHallTeleport DeadHal2 DeadHall`,
    `KQMap::stopLoc 0`, walk to (182,200, 69,800), the trap armed
    again). Clicking that door with the mouse does the same.
20. **The state cannot be left** while the exit switch is off: any
    other state asked of Connor (Wait, Fall, scripts' `setState`,
    weapon swap) is refused. The switch is turned on by `tileExit 1`
    and by finishing on H.
21. **`tileExit <0|1>`** (traced): if Connor is in one of the three
    tile states (`Tile`, `Tile2`, `Tile3`), set that state's exit switch
    (0 = he cannot leave, 1 = he can); with 1 he is also put into
    `Wait` at once. In any other state the command does nothing. With
    no argument (or more than one) it prints
    "usage:  tileExit <1 = can exit | 0 = cannot exit>".
22. **`deadHallTeleport <from> <to>`** (traced): two copies of the hall
    model stand in the world (`DeadHall` and `DeadHal2`). The command
    moves **Connor and the camera** by the same offset, `to`'s
    position − `from`'s position (positions only; heading and camera
    aim are kept), so the player sees no jump. Then the near/far
    (active) lists of all objects are worked out again at once for the
    new place, and 0.5 s later Connor gets an internal notice (not
    traced). Unknown names print
    "DeadHallTeleport:  cannot find object <name>" and do nothing;
    wrong argument count prints
    "usage:  DeadHallTeleport fromHallName toHallName".

### The scales (Connor's `Scales` state, traced 2026-10-28)

Dead City's Scales of Justice: a second puzzle state built like `Tile`
(same refusal of other states, same weapon-swap line as rule 18; it
is not left with `tileExit`, only by stepping away, below).

23. **Entering**: `portrap3.cs` puts Connor at (217,450, 49,140, 2,900),
    heading 1.57, then sets `Scales`. A drawn weapon is put away first.
    His movement list gains `conscale.anm`: `onscale1` (25 frames),
    `onscale2` (46), `offscale1` (12), `offscale2` (12) (data).
24. **Keys**: turn left / right plays `left` / `right` and turns on the
    spot; back held with a turn key also turns; forward (walk or run)
    takes one step (rules 25–26); nothing held plays `stop`.
25. **Forward on the floor** (Connor's x < 219,000): of three headings,
    the one nearest his own is chosen:
    - **−1.57 (step away):** he is put on the start spot facing −1.57,
      the camera goes to (216,360, 52,250, 7,260) and he walks to
      (209,500, 49,065). This is the **only way out** of the state.
    - **2.207 or 0.933 (one of the two dishes):** he is put on the start
      spot at that heading and plays `onscale1` then `onscale2`; the
      camera during this is at (216,115, 53,496, 5,560) for 2.207 and
      (216,115, 44,790, 5,560) for 0.933. Afterwards he is turned 180°
      and raised 200, and the camera is at (216,500, 49,140, 4,500),
      looking along −1.57.
26. **Forward on a dish** (x ≥ 219,000) steps off: on the dish with
    y > 50,000 he turns to −0.933, with y < 50,000 to −2.207 (exactly
    50,000: nothing happens); then `offscale1`, `offscale2`, the camera
    goes to (214,421, 49,340, 4,190) and follows him again.
27. **The puzzle itself is script** (data): the dish colliders
    (`scalelt.cs` / `scalert.cs` on `ConnorCollision`), the feather and
    the `DeadCity::Scales::*` variables; `scalcmpl.cs` runs the win,
    and `scale.cs` Cue 1 drains the blood with five `matchHeights`
    (`terrain.md`) and `terAnimSwitch Blood1`–`Blood5`.
    The state does not set `DeadCity::Scales::Connor` itself.

## Numbers and units

- Stone spacing 2,000 (x and y), rise 500 per row; spot margin 1,000.
- Jump travel 2,000 / ≈ 2,840 (straight / diagonal); movements 2.0,
  2.33, 2.8, 2.8 s; `teleportdown` 21 frames, `teleportup` 22 frames,
  1.4 s each (`Conner.anm` movements 51–56).
- Put-on height + 1,000 z; camera points + 2,000 z above a stone.
- Start (238,340, 122,300, 3,100); finish (237,333, 140,000, 6,900).
- `deadHallTeleport` notice delay 0.5 s.

## Data used

- `dhalltrp.cs`, `ddoor4.cs`, `tileshow.cs`, `tileshow.kq`,
  `tile_<x>.kq`, `notile.kq`, `notile2.kq`, `azrieltp.cs`,
  `worldvar.cs` (`DeadCity::DeadHall::*`, `DeadCity::Azriel::TrapOn`).
- `GAME/Kq/Conner.anm` movements `tilejump1`, `tilejump2`,
  `tilebackjump`, `tilebackjump2`, `teleportdown`, `teleportup`,
  `left`, `right`, `stop`, `away`; their `.kqs` in `GAME/Common`.
- `deadcity/resource/tile*.kqs`, `notile*.kqs` (stone bobbing).
- Dead City `2000.msg` noun 2 (conditions 3 and 133).

## Edge cases

- A save made in the state rebuilds the stone table on load (the state
  keeps whether it was built; positions are read again).
- The table copies each stone's position **when the state starts**,
  so a stone's bob (± 115 in z) at that moment is in it; harmless
  (gravity puts Connor down).
- **29 of the 64 stones have no collision box** (data + the box rule,
  `connor-physics.md` "Where every object's box comes from", checked
  2026-10-30, Q177): the box is looked up by the stone's model name,
  then by its own name (`tile15` …), and neither is in `BOUNDS.KQ` for
  all ten `notile2` stones and the letters b, c, f, k, m, n, o, p, q, r,
  s, u, y and the copies a2, e2, t2. This is the same in the original.
  It never matters for the puzzle: all of them are wrong stones, and the
  five letters and the four stones he is put on all have a box. Jumping
  onto one, Connor goes through it during the jump (gravity is off,
  rule 11), the landing check sends him back (rule 14), and during
  `teleportdown` (gravity on again, Fall refused by rule 20) he sinks
  toward the hall floor until he is moved to the start. Keep it so
  (log it); don't add boxes.
- The hall floor between the start strip and the far door has no
  building under the stones: only the start strip (`platfrml`,
  `platfrmc.dig`) and the far landing (`platfrm3`, `platfrmd.dig`) are
  buildings (data, our log).
- From the edge columns the neighbour search does not wrap.
- Jumping north off row 7 (row 8 spot) counts as wrong.
- Because of rule 20 the state also refuses the Fall state: Connor
  cannot fall while in it.
- Rule 15's "once per visit" assumes the state is made afresh each
  time Connor enters it (as the other Connor states are; not checked
  for this one).

## Do not reproduce

- The missing-stone crash (rule 2).
- Not a bug, but note: the right order is in the program, not the data.
  Keep it as data in our code (a list of grid cells with their
  "put on" stones and camera points) so it can be logged and checked.

## How to check

- Log on entering `Tile`: the 64 stones found, Connor's spot (rule 6).
- Log each jump: spot before, chosen spot, straight/diagonal/back, the
  facing angle; each landing: spot, expected cell, result
  (right / wrong / start strip / door).
- North jump from `tile46` to `tile56`: log Connor's z each update; he
  should end on the stone's top (about 5,621 + his own offset), with
  step up / down 325 throughout the jump.
- Each teleport: log the camera's distance to the spot when
  `teleportdown` ends, the flight's update count N (distance ÷ 5,000 ×
  15, whole part, + 1) and the update it ends on.
- A stone's z at the same point of its bob, cycle after cycle: it
  should rise by about 0.011 (letters) / 0.006 (blank) per cycle.
- In the original: from the start (column 4, row −1) press turn right
  until the stone ahead-right is lit, jump; jump along the strip to
  column 7 and forward onto D; Connor vanishes and reappears on
  `tile73` next to E. Step on any blank stone: back to the start, no
  health lost.

## Open questions

1. ~~The camera "hold" switch~~: answered 2026-10-30, rule 7a.
2. The internal notice 0.5 s after `deadHallTeleport` (not traced).
3. Scales: what happens to Connor's movement while `onscale2` /
   `offscale2` play (the program sets some mover switches first; maybe a
   held last frame; not traced).
4. ~~The camera flight speed after a teleport~~: answered 2026-10-30,
   rule 13 ("The camera flight").
5. **Tile2 (Swamp) and Tile3 (Barren) are separate states**, not the
   same one with other numbers (traced, briefly): they share the stone
   jump movements (`tilejump1/2`, `tilebackjump/2`), `stop` and `away`,
   and also use `jump`, `out` (and Tile2 `throw`), but have no
   teleports and their own puzzle rules (Swamp: `KQSwampTile`,
   `Swamp::Tile::SwitchOn` / `PowerOn`; Barren: `KQLavaTile`,
   `lvtile*.bmp`, `Barren::Tile::Complete`, `tiletpn2.cs` /
   `tiletps2.cs`). The jumping and lit-stone rules (8–11) are probably
   shared; to be studied with the Swamp and Barren batches.
