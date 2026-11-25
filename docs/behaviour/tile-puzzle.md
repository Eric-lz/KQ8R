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

### The scales (Connor's `Scales` state, traced 2026-10-28; Q198 traced 2026-11-20)

Dead City's Scales of Justice: a second puzzle state built like `Tile`
(**state number 131,072** in `StateChange` messages; same refusal of
other states; it is not left with `tileExit`, only by stepping away,
below).

23. **Entering and weapon refusal**: `portrap3.cs` puts Connor at
    (217,450, 49,140, 2,900), heading 1.57, then sets `Scales`. If a
    weapon is drawn when entering, `Scales` plays `away` and switches his
    mode directly to empty hands without going through the weapon-swap
    state, so he does **not** speak the refusal line on entry. While in
    `Scales`, any weapon-swap request is refused with the same line as
    `Tile` (file 0, noun 0, verb 37, condition 23, sequence 1 — rule 18)
    and the pending state request is cleared. His movement list gains
    `conscale.anm`: `onscale1` (25 frames), `onscale2` (46),
    `offscale1` (12), `offscale2` (12) (data).
24. **Keys**: turn left / right plays `left` / `right` and turns on the
    spot; back held with a turn key also turns (and blocks forward steps
    while held); nothing held plays `stop`. The state compares the
    movement Connor's keys currently ask for against the last wanted
    movement it acted on (ignoring case), and triggers a step only when
    that wanted movement **changes** to forward (`Walk` or `Run`).
    However, when `onscale2` or `offscale2` finishes, the state clears
    both its busy flag and the remembered wanted movement: so pressing
    forward once takes one step, and **keeping forward held** across the
    end of `onscale2` / `offscale2` immediately triggers the next step on
    the very next update.
25. **Forward on the floor** (Connor's x < 219,000): of three target
    headings (−1.57, 2.207, 0.933), the one nearest his own is chosen
    with strict `<`. At the entry heading `1.57` (32-bit float), `2.207`
    (the north dish) is fractionally closer than `0.933` (`≈ 0.6370000`
    vs `0.6370000` rounded a step higher), so pressing forward without
    turning steps onto the **north dish** (`2.207`). (On an exact
    mathematical tie, strict `<` starts no animation while leaving the
    state stuck busy — see "Do not reproduce".)
    All camera views in `Scales` are **instant cuts** to a fixed world
    position and fixed angles `(rx tilt, ry roll, rz heading)` — no
    flight and no look-at target:
    - **−1.57 (step away):** Connor is snapped to (217,450, 49,140,
      2,900), heading −1.57; camera following and auto-places /
      auto-turns are switched off; the camera cuts to **(216,360,
      52,250, 7,260)**, angles **(−0.716, 0, 2.644)**; leaving the state
      is enabled; then the state immediately starts the same auto-walk
      as `KQConner::walkTo` (without a cue target) to **(209,500,
      49,034, 2,909)** (earlier doc: 49,065). That walk **leaves `Scales`
      for `Wait` at once**, switches **`handsOff 1`** on, and **walks**
      (not runs) away. Neither `Scales` nor `walkTo` turns `handsOff`
      back off or restores camera following: in Dead City the walk
      carries Connor straight into the room's exit portal trap
      (`portrap7`, `portrap7.cs`), which runs `walkExit` (ending the
      walk) and plays the return portal sequence (`Cue 5` restores
      `KQCamera::follow Connor`, `handsOff 0`, and `setThirdPerson`).
      This is the **only way out** of `Scales`.
    - **2.207 (north dish) or 0.933 (south dish):** Connor is snapped to
      (217,450, 49,140, 2,900) at that heading; camera following and
      auto-places / auto-turns are switched off; the camera cuts to
      **(216,115, 53,490, 5,560)**, angles **(−0.38, 0, −2.449)** for
      2.207 (earlier doc: 53,496), or **(216,115, 44,790, 5,560)**,
      angles **(−0.38, 0, −0.692)** for 0.933; then he plays `onscale1`
      followed by `onscale2`. When `onscale2` ends, Connor gets a
      relative move of **(0, 0, +200)** and a relative turn of **+3.14**
      (about 180°), and the camera (still not following) cuts to
      **(216,500, 49,140, 4,500)**, angles **(−0.1, 0, −1.57)**.
26. **Forward on a dish** (x ≥ 219,000) steps off:
    - on the **north dish** (`y > 50,000`): Connor's heading **snaps at
      once** to **−0.933** (not a gradual turn; position kept), and the
      camera (still not following) cuts back to **(216,115, 53,490,
      5,560)**, angles **(−0.38, 0, −2.449)**;
    - on the **south dish** (`y < 50,000`): Connor's heading **snaps at
      once** to **−2.207** (position kept), and the camera (still not
      following) cuts back to **(216,115, 44,790, 5,560)**, angles
      **(−0.38, 0, −0.692)**;
    - (at `y == 50,000` exactly, nothing happens);
    - he plays `offscale1` then `offscale2`. When `offscale2` ends, the
      camera cuts to **(214,420, 49,340, 4,190)** (earlier doc:
      214,421), angles **(−0.2, 0, −1.64)**, and resumes **`follow
      Connor`** with auto-places / auto-turns back on.
26a. **Gravity during the steps** (traced 2026-11-01, Q189): just before
    `onscale1` (onto a dish) and before `offscale1` (off it) the state
    switches **gravity and keep-on-ground off**; the movement's own steps
    carry him up or down. When `onscale1` / `offscale1` ends it switches
    both **back on** (keep-on-ground in its usual floor mode) and plays
    `onscale2` / `offscale2` with ordinary gravity, so he settles on
    the dish box or the floor under him. Step up / down are not touched.
    The positions in rules 25–26 are put on him with the start-spot move
    (heading set, height 2,900) before `onscale1`, and with a relative
    move (turn 180°, 200 up) after `onscale2`.
27. **The puzzle itself is script** (data): the dish colliders
    (`scalelt.cs` / `scalert.cs` on `ConnorCollision`), the feather and
    the `DeadCity::Scales::*` variables; `scalcmpl.cs` runs the win,
    and `scale.cs` Cue 1 drains the blood with five `matchHeights`
    (`terrain.md`) and `terAnimSwitch Blood1`–`Blood5`.
    The state does not set `DeadCity::Scales::Connor` itself.

### The Swamp fiend-tile puzzle (Connor's `Tile2` state and `KQSwampTile`, traced 2026-11-20, Batch 3)

Swamp's fiend room is a 6 × 7 field of stones and tree stumps
(`KQSwampTile`) flanked by two doors (`fiendoor` on the west,
`Lfswdoor` / `Rtswdoor` on the east) and six fire-shooting totems. Like
Dead City's `Tile`, the scripts load the objects and switch Connor into
`Tile2`; the `Tile2` state handles grid movement, throwing rocks from the
puzzle, and the door exits, while `KQSwampTile` itself handles gas puffs,
damage, and Reveal highlights.

28. **Setting up (data):** `Swamp/newworld.cs` loads 42 `KQSwampTile`
    objects named `tile<col><row>` (columns 0–5 at x = 50,000 + 2,000·col,
    rows 0–6 at y = 92,600 + 2,000·row, on the drawn surface z = −2),
    built from `gastrap1–3.kq` and `stump1–2.kq`. Twelve tiles
    (`tile02`, `tile11`, `tile16`, `tile20`, `tile23`, `tile26`, `tile31`,
    `tile32`, `tile34`, `tile35`, `tile45`, `tile54`) are assigned the
    comment-only script `fiensafe.cs` (`KQObject::setScript`), which
    marks them as safe; the remaining 30 have **no script** and act as
    unsafe gas traps. Six totems (`totem.01`–`totem.06`, `totem.kq`) stand
    along the west side (x = 44,727) and repeat `Cue 500` every 8 s
    (see `ranged-weapons.md` section 9) while
    both `Swamp::Tile::SwitchOn` (toggled by `traplvr.cs`, starts `true`)
    and `Swamp::Tile::PowerOn` are true. Entry into `Tile2` comes from
    `swdoor.cs` (east double doors unlocked with `INVITEM_hmhand`),
    `fiendoor.cs` (west gate via `fienlevr.cs`), or the room's floor
    traps `tiletrp1` / `tiletrp2` (`tiletrap.cs`).
29. **The 10 × 7 grid (`Tile2`, state number 16,384, traced):** when
    `Tile2` starts (and when rebuilt after loading a save), it looks up all
    42 `tile<col><row>` objects (columns 0–5, rows 0–6) and writes their
    positions into **columns 2–7** of a 10-column by 7-row grid. The outer
    columns (0, 1, 8, 9) stay at (0, 0, 0) **except on row 3** (the middle
    door row, y = 98,600):
    - **west door approach (`fiendoor`)**: `(col 1, row 3)` =
      `tile03.x − 2,000` (48,000) and `(col 0, row 3)` =
      `tile03.x − 4,000` (46,000), keeping `tile03`'s y and z;
    - **east door approach (`Rtswdoor`)**: `(col 8, row 3)` =
      `tile53.x + 2,000` (62,000) and `(col 9, row 3)` =
      `tile53.x + 4,000` (64,000), keeping `tile53`'s y and z.
    Just like `Tile` (rule 2), the original does not check for missing
    tiles (**do not reproduce**: skip a missing tile and log it).
30. **Which spot Connor is on (traced):** the bounds use margin 1,000:
    `minX = col0_row3.x − 1,000` (45,000),
    `maxX = col9_row3.x + 1,000` (65,000),
    `minY = tile00.y − 1,000` (91,600),
    `maxY = tile56.y + 1,000` (105,600).
    Inside (`minX < x ≤ maxX` and `minY < y ≤ maxY`):
    `col = trunc((x − minX) / ((maxX − minX) × 0.1))` (0–9),
    `row = trunc((y − minY) / ((maxY − minY) × 1/7))` (0–6), returning
    `col + row × 10` (0–69). Outside: y < `minY` → 300 (south),
    y > `maxY` → 400 (north), x < `minX` → 100 (west), x > `maxX` → 200
    (east).
31. **Entering and leaving `Tile2` (traced):** entering builds the grid
    and sets the exit switch off (`tileExit 0`, rule 20). If a hand or
    ranged weapon is drawn, Connor plays `away`, asks for empty hands
    through the normal weapon-swap path (hitting `Tile2`'s refusal so he
    speaks `"A weapon will not help me here."` — file 0, noun 0, verb 37,
    condition 23 — as in `Tile` rule 7), and forces empty hands directly;
    otherwise he plays `stop` and lights the tile ahead. While in `Tile2`,
    leaving is allowed only when the exit switch is on (`tileExit 1`,
    rule 21) **or when entering `Die` (state 8)** — unlike `Tile`, traps
    and gas can kill Connor inside `Tile2`. Any weapon-swap request is
    refused with the same 0/0/37/23 line. Unlike `Tile`, `Tile2` has
    **no camera hold switch and no teleports**.
32. **Keys, target choice, and forbidden edge moves (traced):** while
    neither a jump nor a rock throw is under way, turn keys (`left` /
    `right`) turn on the spot and update the lit tile; `walk` / `run`,
    `back`, and the Jump key (`Insert`, forwarding forward/back like `Tile`
    rule 8) pick a target spot. Leaning stays off. If Connor stands
    outside the grid to the **west (100)** or **east (200)**, he plays
    his ordinary `jump` movement without turning first. Inside the grid,
    from `(col, row)`, the 8 neighbours (clamped to `0–9`, `0–6`) are
    scored by angular distance to his facing (forward) or opposite facing
    (back), with **four edge blocks** so he never jumps into the empty
    corners off columns 0, 1, 8, or 9:
    - from column 2 (`tile0*`), stepping west into column 1 is allowed
      **only onto row 3** (the `fiendoor` spot);
    - from column 7 (`tile5*`), stepping east into column 8 is allowed
      **only onto row 3** (the `Rtswdoor` spot);
    - from column 1 (west door spot), leaving row 3 is allowed **only
      eastward** (into column 2, e.g. `tile02` or `tile04`);
    - from column 8 (east door spot), leaving row 3 is allowed **only
      westward** (into column 7, e.g. `tile52` or `tile54`).
33. **Lit stone and jump physics (traced):** the forward target tile (or
    the back-jump target when a back jump starts) is drawn at full
    brightness (`tile<col−2><row>`'s unlit switch) whenever its column is
    in **2–7**, turning off the previous one. When the turn-to-target ends,
    **gravity and keep-on-ground go off** (step up/down stay at 325) and he
    plays `tilejump1` (straight forward), `tilejump2` (diagonal forward),
    `tilebackjump` (straight back), or `tilebackjump2` (diagonal back).
    When the jump movement finishes, gravity and keep-on-ground go back on,
    he plays `stop`, the highlight refreshes, and his landing cell is
    checked.
34. **Door exits on landing (traced + data):** after every jump lands:
    - **cells 30 or 31** (column 0 or 1, row 3 — west side): queues
      **`Cue 1`** (extra 0) to **`fiendoor`** (`fiendoor.cs` Cue 1 runs
      `tileExit 1`, clears `PowerOn` and `SwitchOn`, and walks him out
      through the west gate);
    - **cells 38 or 39** (column 8 or 9, row 3 — east side): queues an
      empty-handed click (`ConnorAction`, `"do"`) to **`Rtswdoor`**
      (`swdoor.cs` opens both east doors, runs `tileExit 1`, and walks him
      back outside).
    Landing anywhere else leaves him standing in `Tile2`.
35. **Throwing a rock inside `Tile2` (traced):** Connor can toss rocks
    onto tiles to test for gas without leaving `Tile2`. Started from any of
    three paths while in `Tile2`: clicking a `KQSwampTile` with
    `INVITEM_Rock` in hand, clicking elsewhere with a rock in hand
    (`ranged-weapons.md` 8e), or a script calling `KQMonster::setState
    Connor ThrowRock` (which detects `Tile2` and calls this sequence
    directly without changing state):
    1. switches `handsOff 1` on (cursor hidden as normal, same as console
       `handsOff 1`) and marks a throw pending and the state busy;
    2. plays `away`;
    3. remembers Connor's current projectile file name and temporarily sets
       it to `INVITEM_Rock`'s (`Prrock.kq`);
    4. when `away` finishes while `handsOff` and throw-pending are on,
       clears throw-pending and plays `throw` (`c_rktoss.kqs`, whose `do`
       cue launches the rock as usual);
    5. when `throw` finishes, clears busy, turns `handsOff 0` back off, and
       restores Connor's remembered projectile file name.
    (`Prrock.kq` runs `rockHit.cs`, dropping a rock item where it lands.)
36. **`KQSwampTile` keys and defaults (traced + data):** an object
    subclass with two extra `.kq` keys: `heightNum` (whole number, default
    `0`; data: `gastrap2` = 1, `gastrap1` = 2, `gastrap3` = 3, `stump1` =
    12, `stump2` = 13) and `damFactor` (decimal, default **0.48**; no
    shipped `.kq` file sets it). All five template files set
    `colliderType=16417` (which includes bit `0x4000`, immune to
    projectiles), `colliderMask=5`, `acceptsInventory=INVITEM_Rock`, and
    `groupsAlways=2054` (for Reveal).
37. **Clicking a `KQSwampTile` (traced):** turns Connor toward the tile; if
    clicked with `INVITEM_Rock` in hand, triggers `Tile2`'s built-in rock
    throw (rule 35) when in `Tile2`, or switches to `ThrowRock`
    (`ranged-weapons.md` section 8) otherwise.
38. **Reveal potion on `KQSwampTile` (traced):** when Reveal turns on
    (`potions.md` rule 31), **only tiles that have a script** (the 12 safe
    tiles with `fiensafe.cs`) swap textures:
    - `heightNum` 1, 2, or 3 (gas traps): `gastrap.bmp` → `I_gastrp.bmp`
      and `wtower5.bmp` → `I_wtowr7.bmp`;
    - `heightNum` 12 or 13 (stumps): `trapring.bmp` → `I_trprng.bmp` and
      `trunk.bmp` → `I_trunk.bmp`.
    When Reveal turns off, those original textures are restored. Unsafe
    tiles do not glow.
39. **Rock hit (0-damage projectile) on an unsafe tile (traced):** because
    every `KQSwampTile` has collision bit `0x4000` (projectile-immune), a
    thrown rock delivers a 0-damage hit to the tile. When damage is 0, both
    `Swamp::Tile::PowerOn` and `Swamp::Tile::SwitchOn` are non-zero,
    **and the tile has no script** (unsafe), it spawns a `KQAnimOnce`
    playing **`swmpgas.kqs`** (auto-deleted when done, speed 1,
    non-repeating) at the tile's position raised by a `heightNum` offset:
    - `heightNum` 1 (`gastrap2`): **+750**;
    - `heightNum` 2 (`gastrap1`) or 12 (`stump1`): **+1,000**;
    - `heightNum` 3 (`gastrap3`) or 13 (`stump2`): **+1,250**.
    Safe tiles (with `fiensafe.cs`) puff nothing.
40. **Connor colliding with an unsafe tile (traced):** when Connor touches
    a `KQSwampTile`, if **more than 2.0 s** have passed since the tile's
    last collision time, both `Swamp::Tile::PowerOn` and
    `Swamp::Tile::SwitchOn` are non-zero, **and the tile has no script**:
    1. spawns `swmpgas.kqs` at the same `heightNum` z offset (rule 39);
    2. subtracts **`trunc(−Connor.level × 10 × damFactor)`** from Connor's
       health directly via a stats event (with default `damFactor = 0.48`,
       that is −4 at level 1, −9 at level 2, −24 at level 5 — about 48% of
       max health, bypassing protection);
    3. also sends Connor a **1-damage Fire (kind 2)** attack event
       (causing his fire reaction/sound, or `firedie` if that kills him).
    On **every** collision with the tile (whether gas fired or not), the
    tile updates its last-collision time to now — so staying on an unsafe
    tile resets the 2.0 s cooldown each frame and prevents repeated blasts
    until Connor leaves the tile for more than 2 s and returns.
41. **Unused legacy scripts (`fienfire.cs`, `fienfir2.cs`, `fienfir3.cs`,
    data):** these three scripts sit in `MASK/swamp/kq/` (using
    `loadAnim swmpgas.kqs` and `KQConner::setRelDamage .2`), but no
    shipped `.kq` or `.cs` assigns them. `KQSwampTile` replaced them in
    code (rules 39–40, raising the health loss from 0.2 to `damFactor`
    0.48).

### Barren Region's lava tile puzzle (Connor's `Tile3` state and `KQLavaTile`, traced 2026-11-25, Batch 5)

Barren Region's lava bridge is a 5 × 7 grid of rising and sinking lava
tiles (`tile00`–`tile46`, all `KQLavaTile`) over a lava river, entered
from either the south bridge (`tiletps.cs`) or the north bridge
(`tiletpn.cs`). Connor must jump across the tiles in **prime-number
order** (`3 → 5 → 7 → 11 → 13 → 17 → 19`) to form a permanent bridge
along column 3. (`Tile3` is **state number 65,536** in `StateChange`
messages.)

42. **The `KQLavaTile` class (`Barren/lavatile.kq`, traced):** a
    subclass of `KQAnimObject` with 8 animations in two sets: normal
    bobbing (`stop`, `down`, `up`, `stopup` using `lvtiledn.kqs` /
    `lvtileup.kqs`) and locked/alternate (`stop1`, `down1`, `up1`,
    `stopup1` using `lvtiled2.kqs` / `lvtileu2.kqs`). It starts
    `hidden=1` at `z = -700`.
    - **Four `KQCueEvent` cues (`2400`–`2403`)**:
      - **`Cue 2400 [<bmp>]`** (lock up and optionally swap texture):
        sets the tile's mode to `1` (locked). If a bitmap name is
        supplied (e.g. `lvtile03.bmp` or `lvtile01.bmp`), switches to the
        `*1` animation set: `stop` → `up1`; `stopup` → `stopup1`; `up` →
        `up1` at the same phase plus a relative move of `(0, 0, +50)`;
        `down` → `down1` at the same phase with negated playback speed
        and a `reversing` flag so it rises back up. Then swaps texture
        `lvtile01.bmp` to `<bmp>` and sets `useAltAnim = 1`.
      - **`Cue 2401`** (sink permanently): sets mode to `2` (sunk). If
        playing `stopup`/`stopup1`, starts `down`/`down1`; if playing
        `up`/`up1`, negates playback speed and sets `reversing = 1`.
      - **`Cue 2402`** (decrement active count / sink when 0):
        decrements `activeCount`; when it reaches `0`, sets
        `forceDown = 1` and sinks (`stopup` → `down`, `stopup1` →
        `down1`, `up`/`up1` → negate speed and `reversing = 1`).
      - **`Cue 2403`** (increment active count / rise): increments
        `activeCount`; when `activeCount == 1` **or** the tile is named
        `tile43`, unhides the tile (`setHidden 0`), plays `up` (or `up1`
        if `useAltAnim`), and clears `forceDown = 0`. (`tiletps.cs` sends
        `Cue 2403` to 32 tiles with staggered delays `0.7..11.4 s` when
        Connor first approaches the unsolved puzzle — omitting `tile42`,
        `tile43`, and `tile44`.)
    - **End-of-loop transitions (`onEndLoop`):**
      - `down` ends → plays `stop`; if mode ≠ `2` and `forceDown == 0`,
        schedules an internal delayed play of `up` in **`0.1 s`**;
        otherwise hides (`setHidden 1`).
      - `up` ends → if `reversing == 0`, plays `stopup` and schedules an
        internal delayed play of `down` in **`7.0 s`**. If `reversing !=
        0`, clears `reversing`, plays `stop`, restores positive speed,
        applies relative move `(0, 0, -50)`, and hides (`setHidden 1`).
      - `up1` ends → if `reversing == 0`, plays `stopup1` (**stays up
        permanently**, no 7 s timer). If `reversing != 0`, clears
        `reversing`, plays `stop1`, restores positive speed, applies
        relative move `(0, 0, -50)`, and hides (`setHidden 1`).
      - `down1` ends → if `reversing != 0`, clears `reversing`, restores
        positive speed, applies relative move `(0, 0, +50)`, and plays
        `stopup1`. Otherwise plays `stop1`, and if mode == `2` or
        `forceDown != 0`, hides (`setHidden 1`).
    - The delayed `0.1 s` / `7.0 s` internal play event only fires if
      mode == `0` and `forceDown == 0`.
43. **The 5 × 7 grid and bridge spots (`Tile3`, traced + data):**
    `newworld.cs` places `tile<r><c>` (rows `0..4` south-to-north,
    columns `0..6` west-to-east) at `x = 86,640 + 2,000·c`,
    `y = 151,943 + 2,000·r`, `z = -700`.
    - Cell index = `row * 10 + col` inside the area `[tile00.x − 1,000,
      tile46.x + 1,000] × [tile00.y − 1,000, tile46.y + 1,000]`. Outside:
      `300` = south (south bridge), `400` = north (north bridge), `100` =
      west, `200` = east.
    - Two bridge landing spots are computed from `tile03` and `tile43`:
      south bridge = `tile03 − (0, 2,000, 0)`, north bridge =
      `tile43 + (0, 2,000, 0)`.
44. **Entering, leaving, and music (`Tile3`, traced):**
    - On entry: reads all 35 tile positions (same missing-tile crash
      warning as rule 2), sets `tileExit 0`, sheaths any drawn weapon
      (plays `away` and requests empty hands — same refusal line
      `0/0/37/23` as `Tile` rule 7), and schedules pushing **`mlava.aud`**
      after **`0.2 s`**.
    - While `tileExit == 0`, refuses every state change **except `Die`
      (state 8)**; weapon-swap requests (`0x20`) speak the standard
      refusal line (`0/0/37/23`, rule 18).
    - On leaving the state: schedules popping **`mlava.aud`** after
      **`0.2 s`**, and if Connor has **not** yet solved the puzzle
      (`solvedTile3 == 0`), sends **`Cue 2402`** to all 35 tiles
      (`tile00`–`tile46`) so they sink back into the lava.
45. **Controls, jumping, and blocked cells (`Tile3`, traced):**
    - Same turn-in-place (`left`/`right`), forward/back jump animations
      (`tilejump1`, `tilejump2`, `tilebackjump`, `tilebackjump2`), lit
      forward-target tile (`unlit` flag on tiles `0..46`), and jump
      physics (gravity and `keepOnGround` off during the jump, restored
      on landing, step up/down unchanged at 325) as `Tile` (rules 8–11).
    - **Blocked cells** (penalized by `+π` in the angle search so Connor
      cannot jump onto them):
      - While unsolved (`solvedTile3 == 0`): row `4`, columns `2, 3, 4`
        (`tile42`, `tile43`, `tile44`) are blocked.
      - Once solved (`solvedTile3 != 0`): all columns other than column
        `3` are blocked, so Connor only jumps along the column-3 bridge.
    - **Bridge entry and exit jumps:**
      - From the **south bridge** (`300`): chooses among `tile02`,
        `tile03`, `tile04` (straight for `tile03`, diagonal for `02`/`04`)
        and the south exit point `(92,640, 144,000, 1,000)` (straight,
        marks `exiting = 1`).
      - From the **north bridge** (`400`): chooses between `tile43`
        (straight) and the north exit point `(92,640, 162,000, 1,000)`
        (straight, marks `exiting = 1`).
      - From **row 0, columns `2, 3, 4`**: facing south can jump onto the
        south bridge spot (straight from col 3, diagonal from cols 2/4;
        marks `exiting = 1`).
      - From **row 4** (only when `solvedTile3 != 0`): facing north can
        jump onto the north bridge spot (straight; marks `exiting = 1`).
      - When an `exiting = 1` jump finishes: calls `tileExit 1`, and if
        Connor is alive (`health > 0`), runs **`tiletps2.cs StartExit`**
        (if `cell == 300`, south) or **`tiletpn2.cs StartExit`**
        (otherwise, north), which runs Connor off the bridge and restores
        controls.
46. **The prime-number puzzle (`Tile3` landing check, traced):** each
    time Connor finishes a non-exiting jump, his grid cell is tested:
    - If `solvedTile3 != 0` and `pendingDrop24 != 0` and he is no longer
      on `tile24` (cell `24`): clears `pendingDrop24` and sends **`Cue
      2401`** to `tile24` so the last prime tile sinks behind him.
    - Otherwise, if he landed on one of the **7 prime tiles**, the state
      tracks `lastPrime` and `inOrder` (stepping on any non-prime tile
      leaves them untouched):

      | Order | Tile (cell) | Prime | Required `lastPrime` | Sound (plain one-shot, if in order) | Locked texture (`Cue 2400`, once per visit) |
      | ---: | --- | ---: | ---: | --- | --- |
      | 1 | `tile12` (`12`) | 3 | *(any — resets `inOrder = 1`)* | `T3.aud` | `lvtile03.bmp` |
      | 2 | `tile41` (`41`) | 5 | `3` | `T5.aud` | `lvtile05.bmp` |
      | 3 | `tile36` (`36`) | 7 | `5` | `T7.aud` | `lvtile07.bmp` |
      | 4 | `tile05` (`5`) | 11 | `7` | `T11.aud` | `lvtile11.bmp` |
      | 5 | `tile20` (`20`) | 13 | `11` | `T13.aud` | `lvtile13.bmp` |
      | 6 | `tile45` (`45`) | 17 | `13` | `T17.aud` | `lvtile17.bmp` |
      | 7 | `tile24` (`24`) | 19 | `17` | `T19.aud` | `lvtile19.bmp` |

    - Stepping on prime 5–19 out of order (or with `inOrder == 0`) sets
      `inOrder = 0` and plays no sound, but still locks that tile up with
      its prime bitmap (once) and updates `lastPrime`. Stepping back on
      `tile12` (prime 3) restarts the sequence (`inOrder = 1`,
      `lastPrime = 3`).
    - **Solving the puzzle** (landing on `tile24` with `lastPrime == 17`
      and `inOrder != 0`):
      1. Plays `T19.aud`.
      2. Sends **`Cue 2403`** to `tile43` (raising it) and **`Cue 2400
         lvtile01.bmp`** to all 5 column-3 tiles (`tile03`, `tile13`,
         `tile23`, `tile33`, `tile43`), locking the centre bridge up.
      3. Sends **`Cue 2401`** to all other tiles in the 5 × 7 grid
         **except** column 3 and `tile24` (where Connor stands), sinking
         them permanently; sets `pendingDrop24 = 1` so `tile24` sinks as
         soon as Connor jumps off it.
      4. Sets `Connor.solvedTile3 = 1` (persisted in `Connor.kq`),
         awards **50,000 XP** (`KQStatsChangeRequestEvent 0 0 0 0 50000`),
         sets console variable `Barren::Tile::Complete = true`, and adds
         map icon **`ba_Order.bmp`** at `(155, 177)`.
47. **Barren script-coverage note (`Altarboy.cs`, data):** line 16 of
    `Barren/Altarboy.cs` (`if test $2 == INVITEM_StoneOrder`, nested
    inside `if test $2 == ConnorAction`) is a typo in the original script
    for `$4` (line 28 correctly checks `else if test $4 ==
    INVITEM_StoneOrder`). It is **not** a script call sent by the engine
    and can never be true.

## Numbers and units

- Stone spacing 2,000 (x and y), rise 500 per row; spot margin 1,000.
- Swamp tiles (`Tile2`): 6 × 7 tiles in columns 2–7 of a 10 × 7 grid
  (spacing 2,000, z −2 on surface); outer approach cells on row 3 (`y =
  98,600`) at x = 46,000 / 48,000 (west) and 62,000 / 64,000 (east).
- `KQSwampTile`: `damFactor` default 0.48; collision cooldown 2.0 s; gas
  offsets +750 (`heightNum` 1), +1,000 (2, 12), +1,250 (3, 13); extra
  Fire hit 1 damage (kind 2).
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
- `Swamp`: `newworld.cs` (`tile00`–`tile56`, `fiensafe.cs` assignments),
  `gastrap1–3.kq`, `stump1–2.kq`, `fiendoor.{kq,cs}`, `fienlevr.{kq,cs}`,
  `lfswdoor.kq`, `rtswdoor.kq`, `swdoor.cs`, `tiletrp1–2.kq`,
  `tiletrap.cs`, `totem.{kq,cs}`, `traplvr.{kq,cs}`, `Prrock.kq`,
  `rockHit.cs`, `swmpgas.kqs`, unused legacy `fienfir*.cs`.
- `GAME/Kq/Conner.anm` movements `tilejump1`, `tilejump2`,
  `tilebackjump`, `tilebackjump2`, `teleportdown`, `teleportup`,
  `left`, `right`, `stop`, `away`, `throw` (`c_rktoss.kqs`), `jump`;
  their `.kqs` in `GAME/Common`.
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
- Because of rule 20 (`Tile`) the state also refuses the Fall state:
  Connor cannot fall while in it. `Tile2` refuses every state except `Die`
  when the exit switch is off (rule 31).
- Rule 15's "once per visit" assumes the state is made afresh each
  time Connor enters it (as the other Connor states are; not checked
  for this one).

## Do not reproduce

- The missing-stone crash (rule 2, rule 29).
- **`Scales` exact-tie soft lock** (rule 25): comparing distance to the
  three floor headings uses strict `<` from a default index of `0`
  (none), so if Connor's heading is mathematically equidistant between two
  targets, no animation starts while `busy` stays on forever. Break ties
  deterministically instead (for example in favour of `2.207`, north
  dish).
- Not a bug, but note: the right order is in the program, not the data.
  Keep it as data in our code (a list of grid cells with their
  "put on" stones and camera points) so it can be logged and checked.

## How to check

- Log on entering `Tile`: the 64 stones found, Connor's spot (rule 6).
- **In `Tile2` / `KQSwampTile` (Swamp, B):**
  - Open `Rtswdoor` with `INVITEM_hmhand` or walk onto `tiletrp1`; log
    the 10 × 7 grid, cells `30/31` and `38/39` (x = 46k/48k and 62k/64k),
    and the refusal to step into empty outer corner cells (rule 32).
  - Drink a Reveal potion: verify only the 12 `fiensafe.cs` tiles swap to
    `I_gastrp.bmp`/`I_wtowr7.bmp`/`I_trprng.bmp`/`I_trunk.bmp`.
  - Throw `INVITEM_Rock` while standing inside `Tile2`: confirm `handsOff`
    toggles, `away` -> `throw` plays with `Prrock.kq` without leaving
    `Tile2`, safe tiles do not puff, and unsafe tiles spawn `swmpgas.kqs` at
    +750 / +1,000 / +1,250.
  - Step onto an unsafe tile with `PowerOn` and `SwitchOn` true: confirm
    health drops by `trunc(level × 10 × 0.48)` plus 1 Fire attack event,
    `swmpgas.kqs` spawns, and staying on the tile does not repeat damage.
  - Jump onto column 0/1 row 3 (`fiendoor` Cue 1) and column 8/9 row 3
    (`Rtswdoor` `ConnorAction` `"do"`): confirm `tileExit 1` runs and
    Connor exits cleanly.
- **In `Scales`:** log each of the 6 camera cuts (position, `(rx, ry,
  rz)`, follow on/off); verify pressing forward right after `portrap3.cs`
  (heading `1.57`) chooses the north dish (`2.207`), holding forward
  across `onscale2`'s end steps back off cleanly into `follow Connor` at
  `(214,420, 49,340, 4,190)`, and turning toward `−1.57` and pressing
  forward leaves `Scales` → `Wait` with `handsOff 1`, walking into
  `portrap7` (`walkExit` + return portal).
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
3. ~~Scales: Connor's movement while `onscale2` / `offscale2` play~~:
   answered 2026-11-01 (Q189), rule 26a.
4. ~~The camera flight speed after a teleport~~: answered 2026-10-30,
   rule 13 ("The camera flight").
5. ~~**Tile2 (Swamp)** and **Tile3 (Barren)**~~: both answered (`Tile2`
   2026-11-20, Batch 3, rules 28–41; `Tile3` & `KQLavaTile` 2026-11-25,
   Batch 5, rules 42–47).
