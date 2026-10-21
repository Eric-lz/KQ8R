# Rope and hook

Study session A, 2026-10-21. Sources: `Mask.exe` (studied, described in
our own words), `GAME/Kq/Conrope.anm`, `GAME/Kq/Invinfo.itl`,
`GAME/ENGLISH/0.MSG`, and every world's rope scripts (`daventry/kq/ropetrp*.cs`,
`churchr*.cs`, `useropeh.cs`, `ropenhok.kq/.cs`; `barren/kq/7rope*`,
`basrope*`, `btrope*`; `deadcity/kq/ropetp*`, `ropetrap.cs`, `ropetrp2.cs`;
`gnome/kq/ropetp*`; `iceworld/kq/ropeice*`, `ropetrp*`, `towrope*`;
`swamp/kq/fienrop*`, `telerop*`). Builds on `inventory.md` rules 9–12
(clicking a slot, the item in hand), `interface.md` rules 15 and 45 (status
icons, the rope button), `traps.md` (trap boxes and their `On`/`Off`
scripts) and `camera.md` (follow, `move`). Marks: **[traced]** (read in
the code), **[data]** (read in the game files), **[inferred]**,
**[not traced]**.

## Purpose

Connor climbs up and down cliffs and walls at fixed **rope places**. With
the Rope and Hook in his inventory he takes it from the inventory bar,
clicks in the world, throws the hook up (from below) or hooks it over the
edge (from above), climbs while the player holds forward or back, gets
off at the other end, and goes back to his weapon. The whole climb is one
special state of Connor's; while it runs he can't be hurt and monsters
lose him.

## Rules

### Rope places

1. A rope place is a pair of **trap boxes** (`KQTrap`, `colliderMask=4`,
   `traps.md`), one at the foot of the climb and one at the top [data].
   Their `On` script tells Connor where the rope is with
   `KQConner::setRope Bottom|Top x y z rx ry rz`, and their `Off` script
   clears it with `KQConner::setRope None` [data]. The data has **27 rope
   places** (27 `Bottom` and 27 `Top` scripts, list in "Data used").
2. `KQConner::setRope` [traced]:
   - `None`, `Top` or `Bottom` (not case-sensitive) sets Connor's **rope
     status**: none, "at the top of a rope" or "at the foot of a rope".
     Anything else prints the usage line and changes nothing.
   - It accepts 1, 4 or 7 words after the command: just the status
     (place and direction then 0, 0, 0), status + place, or status +
     place + direction (rx ry rz, radians, the same three turns as an
     object's `dirX/dirY/dirZ` [inferred: same convention]). Every
     script in the data uses the 7-word form, always with rx = ry = 0
     [data]. Other counts print the usage line.
   - The **place and direction are only stored if the status change was
     accepted** (rule 4). The two `setRopeCam` spots (rule 3) are cleared
     on every `setRope`, accepted or not.
3. The camera settings for the climb [traced]:
   - `KQConner::setRopeCam x1 y1 z1 x2 y2 z2`: two candidate camera spots
     for the climb (rules 18, 20). A spot counts as **set only if all
     three of its coordinates are non-zero**; otherwise the game makes its
     own two spots (rules 18, 20).
   - `KQConner::setRopeCam2 x y z`: where the camera goes when the rope is
     taken from the inventory at the **top** (rule 9).
   - `KQConner::setRopeCamTopLimit z` / `setRopeCamBottomLimit z`: the
     highest / lowest height the camera may reach while Connor climbs
     (rules 19, 21). A limit of 0 means "no limit". `setRope None` clears
     both limits.
   - `KQConner::setRopeCamAngle angle range`: a heading and a tolerance
     for where the camera should be looking when the player clicks
     (rule 11). In practice it has no effect (see "Do not reproduce").
   - `setRopeCam2`, `setRopeCamAngle` and the limits are **not** cleared by
     `setRope Top/Bottom`: they keep their last value until a script sets
     them again (see "Edge cases").
4. **The status only changes if Connor carries the rope** [traced]. When
   `setRope` runs and the interface is up, the game first checks that
   Connor's inventory holds an item of category `Rope`
   (`INVITEM_RopeAndHook`, id 4050, `Invinfo.itl`). If not, it hides both
   hook icons and **leaves the status as it was** (also for `None`), and
   the place and direction are not stored. So walking into a rope trap
   without the rope does nothing, and picking the rope up while standing
   inside a trap box does not make that place usable until Connor walks
   out and in again [inferred from the rule].
5. Each accepted change also notes the game time of the change [traced;
   nothing was found that reads it].
6. `setRope None` additionally [traced]: if Connor has the rope on the
   pointer (rule 8), it is put back (the pointer returns to normal); both
   camera limits are cleared; both hook icons are hidden.

### The hook icons (answers `interface.md` open question 3)

7. Two status icons (`interface.md` rule 15, `IDCTL_TIMER` places) tell
   the player a rope can be used here [traced]:
   - **`HookUp`** is shown while the status is **Bottom** (the rope can be
     thrown **up**);
   - **`HookDown`** is shown while the status is **Top** (the rope can be
     let **down**);
   - nothing for None. Changing status hides the old icon first.
   - Neither is shown while Connor is **on the rope** (from the moment he
     grabs it, rules 18/21, until his climb animation ends, rule 23); at
     the end of the climb the icon for the current status comes back
     (normally the other end's trap has set it by then).
   - Without the rope in the inventory, no icon (rule 4).

### Taking the rope from the inventory (answers Q110 b)

8. A left click on the **rope button** (bottom right, `interface.md`
   rule 45) [traced]:
   - while Connor is in the rope state (rules 13–26): ignored;
   - if the rope is already on the pointer: it is put back (and the
     first-person lock of rule 9 is released);
   - status **None**: Connor says `0.MSG` file 0, noun 0, verb 58, case 37,
     sequence 1: **"I am not near anything to climb."** The rope is **not**
     put on the pointer (only the remark). (As for every slot, the click
     is ignored during `handsOff` and while Connor is dead, `inventory.md`
     rule 9.)
   - status **Bottom**: the rope becomes **the item in hand**: the pointer
     turns into its cursor (`IDBMA_CURS06`, `Invinfo.itl`).
   - status **Top**: first the camera is moved (rule 9), then the rope
     becomes the item in hand as for Bottom.
9. **Top only, at the moment the rope is taken** [traced]: the game
   forces the third-person view (if Connor was in first person it switches
   to third person and remembers that) and blocks the first-person key;
   then it sends the camera straight to the `setRopeCam2` spot, aimed at a
   point **1,000 above Connor's position** (the camera turns to look at
   it; no roll). The camera also gets a flag that keeps it there
   [inferred; what the flag does exactly was not traced]. Putting the rope
   back (rule 8, second case) releases the first-person lock and returns
   to first person if it was forced; **the camera stays where it is**
   (the user's check in the original, 2026-10-21).
### Starting the climb: the click in the world

10. With the rope on the pointer, **a left click anywhere in the world**
    (whatever is under the pointer) starts the climb, unless Connor is in
    a state where world clicks are always ignored (`handsOff`, Connor
    dead, and two other special states) [traced]. The rope leaves the
    pointer in every case.
11. Before starting, the game compares where the camera looks with the
    `setRopeCamAngle` heading and tolerance. **Because of a mistake in that
    test it always passes** (unless the camera's up/down angle is exactly
    0.4 rad, see "Do not reproduce") [traced]. If it ever failed, Connor
    would say `0.MSG` 0/0/58/36/1 **"I don't see anything here to
    climb."** and nothing else would happen.
12. Starting: the first-person lock of rule 9 is released (so a first-person
    player at a top place is briefly back in first person), then Connor
    is asked to enter the **rope state** [traced]. Connor's state change
    goes through the usual "may I leave this state" check of his current
    state (`connor-physics.md`); from standing or walking it is accepted.

### The rope state: the order of events

13. **On entering** [traced]:
    - the camera stops following Connor and stays where it is;
    - Connor turns on the spot to face the rope's place (the normal
      turn-to-a-point turn of `walkTo`, `camera.md`/`connor-movement.md`);
    - the camera spots and limits of rule 3 are copied for this climb;
    - if Connor is in **first person**, the view switches to third person
      and the game remembers to switch back at the end (rule 26);
    - the first-person key is blocked; **`handsOff` is switched on with the
      pointer kept** (the player can't move him) until he hangs on the
      rope (rules 19, 21);
    - Connor's current **hand mode** (`inventory.md`, `fighting.md`: hands,
      sword, bow …) is remembered;
    - the rope animation list `GAME/Kq/Conrope.anm` is loaded and its
      animations for this end are made ready;
    - **only at the foot and only if he was in first person**: the camera
      is first put 1,000 from the rope's place (the direction of that
      1,000 was not read reliably), turned like the rope, then moved as in
      rule 18;
    - Connor plays **`away`** (putting his weapon away) from his normal
      animation list.
14. **`away` ends**: Connor's hand mode switches to **empty hands** (hand
    mode 4, `pushing.md`) unless it already is. Then:
    at the foot he plays **`draw`**, at the top **`drawdown`** (both from
    `Conrope.anm`: taking the rope out) [traced].
15. **From the foot** [traced]:
    - `draw` ends: Connor's **gravity is switched off**; he plays
      **`btoss`** (throwing the hook up).
    - `btoss` ends: Connor's step-up and step-down heights
      (`connor-physics.md`) are saved and set to 0; Connor is **placed
      exactly at the rope's place and direction** (rule 1's x y z and
      rx ry rz); a separate **rope model** is created at Connor's place,
      playing `Conrope.anm`'s **`rope`** animation (`cr_rope.kqs`,
      21 frames, sounds `cr_rope.aud` and `cr_rope2.aud`), with gravity
      off, deleted when its animation ends; the camera is moved close to
      Connor (exact placement partly traced, open question 2);
      Connor is told to stand still (`Stop`).
    - The rope model's animation ends → rule 18 (Connor grabs the rope).
16. **From the top** [traced]:
    - `drawdown` ends: Connor is **placed exactly at the rope's place and
      direction**; the camera is moved (rule 20); gravity is switched off;
      he plays **`ttoss`** (hooking the rope over the edge, 80 frames;
      there is no separate rope model).
    - `ttoss` ends: he plays **`ton`** (getting onto the rope from above).
    - `ton` ends → rule 21.
17. Every animation of the rope state plays from `Conrope.anm` except
    `away` and `out` (Connor's normal list for his hand mode) [traced].
    In order, from the foot: `away`, `draw`, `btoss`, the rope model's
    `rope`, `bon`, `climb` (upward), `toff`, `taway`, `out`. From the top:
    `away`, `drawdown`, `ttoss`, `ton`, `climb` (downward), `boff`,
    `baway`, `out` (either climb can end at the other end, rule 23).

### Grabbing the rope from below

18. **When the rope model's animation ends** [traced]:
    - Connor's current position is noted as the **camera's aim point**
      for the climb;
    - the camera is moved to one of two spots, aimed at that point: if
      the place's `setRopeCam` spot 1 is set (rule 3), the **nearer** to
      where the camera was (at the end of `btoss`) of spot 1 and spot 2
      (spot 2 if equally near); otherwise the nearer of two spots the game
      makes: the rope's place plus **(2,500, 2,500, 4,800)** or
      **(−2,500, 2,500, 4,800)**, both turned by the rope's direction
      (x, y along the rope's own axes, z up);
    - Connor's gravity is off; the hook icons are hidden (rule 7);
    - **Connor is lifted 5,000 straight up** (his position moves; his
      heading doesn't change) — the climb animations are drawn hanging
      below his position [inferred from the data: every top place is
      about 5,000 above its foot, see "Numbers"];
    - he plays **`bon`** (getting onto the rope from below, 32 frames).
19. **`bon` ends** [traced]: the camera's height range for the climb is
    set: lowest = the camera's height now; highest = 5,000 above that,
    or, if something solid is in the way straight above the camera
    (terrain or a world object that isn't Connor), that height; then
    250 is taken off the highest; and if the place has a top limit
    (rule 3) and the highest is above it, the highest becomes the limit.
    Then the **climb starts at its first frame** (Connor at the bottom of
    the rope), `handsOff` goes off, his animation is **frozen** until a
    key is held, and **turning is switched off** (rule 22).

### Getting onto the rope from above

20. The camera at the top, when `drawdown` ends (rule 16) [traced]: if spot 1 is
    set, the nearer of spot 1 and spot 2 to where the camera was;
    otherwise the nearer of the rope's place plus **(5,000, −5,000,
    −2,000)** or **(−5,000, −5,000, −2,000)**, turned by the rope's
    direction. The camera is aimed at the rope's place. (These spots are
    below the edge, looking up at it.)
21. **`ton` ends** [traced]: the camera's height range is set the other
    way: highest = the camera's height now; lowest = 5,000 below that, or
    the height of anything solid straight below the camera; then 250 is
    added to the lowest; and if the place has a bottom limit and the
    lowest is below it, the lowest becomes the limit. Gravity off; the
    hook icons are hidden; **Connor turns half round (π) on the spot**;
    his position is noted as the camera's aim point; the **climb starts at
    its last frame** (Connor at the top of the rope); `handsOff` goes off,
    the animation is frozen, turning is switched off.

### Climbing

22. While Connor hangs on the rope, each update (15 per second,
    `connor-physics.md`) [traced]:
    - **no movement key held**: the climb animation stays frozen (he
      hangs still);
    - **back held** (the movement the game calls `Back`): the climb
      animation plays **backwards** (down the rope); the camera's height
      goes **down 50** (not below the lowest of rules 19/21) and the aim
      point goes down 50 (no limit);
    - **forward held**: the animation plays **forwards** (up the rope);
      camera height **up 50** (not above the highest) and the aim point
      up 50;
    - **any other movement key** (left, right, sidesteps …) **does
      nothing**: he hangs still as with no key (the user's check in the
      original, 2026-10-21; the code reads as if any movement other than
      `Back` climbs up, but the original doesn't do that: follow the
      check);
    - the camera keeps its x and y and is re-aimed at the aim point.
    - Turning is switched off for the whole climb (left/right do nothing;
      this also means no leaning, `connor-movement.md` "Turning"); the jump
      key does nothing (rule 28).
23. Connor's **position does not change while he climbs**: the climb is
    the animation `climb` (`cr_climb.kqs`, 78 frames; sounds `getup.aud`
    at the start, `ropefal1/2/3.aud` at 22 %, 50 % and 72 %, `gettoff.aud`
    at 86 % of the animation, `Conrope.anm`) scrubbed forwards or
    backwards. **The climb ends when the animation reaches an end in the
    direction it is playing** [traced]:
    - the hook icons may show again (rule 7); turning is switched back
      on; **`handsOff` goes on again** (pointer kept) for the getting-off;
    - reached the **top end** (playing forwards) → **`toff`** (getting off
      at the top, 21 frames);
    - reached the **bottom end** (playing backwards) → Connor is **moved to
      1,000 toward his back (his model's +y side) and 4,900 down** from his
      position (in his own frame), gravity goes back on, and he plays
      **`boff`** (getting off at the foot, 33 frames).
    - So a climb started from either end can end at either end: from the
      foot, holding back takes him down to the foot again, and the same
      the other way.

### Getting off

24. At the top: `toff` ends → the camera **follows Connor again**
    (`camera.md` follow) and he plays **`taway`** (putting the rope away,
    26 frames). At the foot: `boff` ends → **`baway`** (41 frames); the
    camera follows Connor again when `baway` ends [traced].
25. `taway` or `baway` ends [traced]: gravity on; the step heights saved
    in rule 15 are put back (only at the foot side: the top side never
    saved them, see "Edge cases"); the camera follows Connor. Then his
    **hand mode goes back** to the one remembered in rule 13:
    - if he was **holding a weapon** (hand or ranged) when the climb
      started, he plays **`out`** (taking that weapon out) from its list;
    - if his weapon was **put away** (empty hands), nothing is drawn: `out`
      is skipped and rule 26 follows at once.
    This holds for a climb up and a climb down (the user's check in the
    original, 2026-10-21). (Reading of the code had suggested a weapon
    "last put away" might be drawn too; the original doesn't do that.)
26. `out` ends (the end of the rope state) [traced]: `handsOff` off; the
    first-person key is allowed again; the rope animation list is let go;
    Connor goes to **standing** (`Wait`); if rule 13 switched him out of
    first person, the view goes back to first person.

### While Connor is in the rope state (from `away` to `out`)

27. **He can't be hurt**: hits on him are ignored completely (no damage,
    no hurt sound or reaction) for the whole rope state [traced].
28. **His state can't be changed** until rule 26: requests for any other
    state (jumping, a `Special` reaction, even death or a script's state
    change) are refused [traced]; the rope state only lets him go, at the
    end, to standing, walking, a `Special` movement or death. A refused
    request stays waiting and is replaced by the rope state's own request
    for standing at the end [inferred from `connor-physics.md`'s state
    change rule], so in effect it is dropped.
29. **Monsters lose him**: every character is told Connor is 100,000
    below his real place while he is in the rope state (`potions.md`
    rule 26) [traced earlier].
30. Keys and clicks [traced]:
    - the **first-person key** does nothing (blocked, rule 13);
    - the **weapon keys** do nothing (with empty hands, readying a weapon
      is asked of his current state, and the rope state ignores it; no
      remark: "A weapon will not help me here." is **not** said on the
      rope, rule 38);
    - the **rope button** is ignored (rule 8);
    - during the throws, grabs and getting-off (`handsOff` on) movement
      keys and world clicks are ignored as for any `handsOff`
      (`clicking.md`); while he hangs on the rope they are not, but
      movement keys only climb (rule 22);
    - a **click on an item** in the world while he hangs **picks the item
      up** as usual (the user's check in the original, 2026-10-21);
    - a **rock throw** while he is in the rope state is **ignored** (see
      "Do not reproduce": the original does something odd).
31. **Gravity is off** from the end of `draw`/`drawdown` until he gets off
    (rule 23 bottom end, or rule 25) [traced].
32. The rope state is kept in **saved games** (with Connor's data): the
    rope status, place, direction, camera spots, `setRopeCam2` spot, angle,
    tolerance and limits; and, if a game is saved mid-climb, the state's
    own progress (which step, whether first person must come back, the
    hand mode before, the camera's old place, the aim point, the camera's
    height range, the saved step heights) [traced: the keys are listed in
    "Data used"]. Loading a game saved mid-climb works normally, with
    no ill effects (the user's check in the original, 2026-10-21).

### The rope and hook item

33. `INVITEM_RopeAndHook` (id 4050, category `Rope`, picture
    `IDBMP_2ROPE1`, cursor `IDBMA_CURS06`, model `rophook.kqs`, sounds
    `hook.aud`, `Invinfo.itl`) [data]. It is a whole-game item (not used up;
    nothing in the data removes it). Picking it up gives **+10 experience**
    and help pop-up 0x80 4.5 s later (`potions.md` rule 40) [traced
    earlier].
34. Daventry: the rope lies on the mill (`ropenhok.kq`, a `KQObject` with
    `tooFarAwayAction=1`). Clicking it gives it only while
    `Daventry::Mill::OnTrap2` is true (Connor stands in the mill's trap);
    otherwise Connor comments (`1000 21 14 19`, sequence 1 the first time
    and sets `Daventry::RopenHook::progress` 2, then sequence 2, or the
    generic `0 0 0 1` once the mill tree is down) [data].
35. Daventry's three bottom rope traps run `useropeh.cs`: the **first
    time** Connor enters one **with the rope**, help pop-up **145** is shown
    (then `Daventry::Dialog::UseRope` is 0) [data].
36. Most top-place scripts also run `KQConner::setRun 0` (Connor walks,
    doesn't run, near the edge); Dead City's switch `allowShadows` at their
    places (`characters.md` rule 36); Daventry's `ropetrp2.cs` also sets
    `KQCamera::setRange 1500 5000` and gives 25 experience the first time
    (`Wizard::foundHouse`) [data].

### Corrections to other notes (from this study)

37. `inventory.md` rule 9 ("at a rope place he throws the rope at once"):
    **wrong**. At a rope place the rope becomes the item in hand; the climb
    starts with the next click in the world (rules 8–12). Away from a rope
    place only the remark is said; the rope is **not** held (Q110 b).
38. `inventory.md` rule 9 (rocks refused in "rope climbing and two puzzle
    states") and `talking.md`'s table ("Drawing a weapon in some special
    states (ropes, puzzles)"), `connor-movement.md` "Turning" ("four …
    special states … such as ropes"): the rope state is **not** one of
    those four states. On the rope a rock can be put on the pointer as
    anywhere (throwing it is ignored, rule 30), weapon keys are silent
    (rule 30), and leaning can't happen because turning is off (rule 22).

## Numbers and units

| What | Value |
| --- | --- |
| Rope status values | None, Top, Bottom |
| Height of every climb | about **5,000** (top place z − foot place z: 4,840–5,032 in the data, 5,000 at 11 of the 27 places; x and y of the two ends within 225) [data] |
| Lift when grabbing from below | 5,000 straight up [traced] |
| Drop after climbing down to the foot | 1,000 toward his back, 4,900 down, in his own frame [traced] |
| Turn when getting on from above | π (half a turn) [traced] |
| Camera aim when taking the rope at the top | 1,000 above Connor's position [traced] |
| Camera spots made by the game, foot | rope place + (±2,500, 2,500, 4,800) in the rope's frame [traced] |
| Camera spots made by the game, top | rope place + (±5,000, −5,000, −2,000) in the rope's frame [traced] |
| Foot, first-person start | camera 1,000 from the rope's place (direction of the 1,000 not read reliably) [traced] |
| Camera height range while climbing | 5,000 (up from the start at the foot, down from the start at the top), shortened by an obstacle, then 250 kept clear, then the limit [traced] |
| Camera and aim point speed while climbing | 50 per update (750 per second at 15 updates/s) [traced] |
| Animations (frames, `Conrope.anm`) | `baway` 41, `boff` 33, `bon` 32, `btoss` 29, `climb` 78, `draw` 18, `drawdown` 18, `rope` 21, `taway` 26, `toff` 21, `ton` 38, `ttoss` 80 [data] |
| Climb animation direction | forward = up, backward = down [traced] |
| Aim test (ineffective) | up/down angle 0.4 rad; heading tolerance from `setRopeCamAngle` (1.9 everywhere) [traced/data] |
| Remarks | `0.MSG` 0/0/58/37/1 "I am not near anything to climb."; 0/0/58/36/1 "I don't see anything here to climb." [data] |
| Experience | +10 for picking the rope up; +25 first visit to Daventry's wizard-house top (script) [traced/data] |

## Data used

- `GAME/Kq/Conrope.anm` (`CONROPEAnimLoops`): the 12 rope animations, their
  `.kqs` files (`GAME/Common/Cr_*.kqs`) and sound cues; `footstep` and
  `playsound` cues as in `anim-scripts.md`.
- `GAME/Kq/Invinfo.itl` item 54 (`INVITEM_RopeAndHook`); `GAME/Kq/Bounds.kq`
  `rophook.kqs`.
- `GAME/ENGLISH/0.MSG`: the two remarks above; `A rope and a hook could prove
  useful for climbing.` is the item's description.
- Rope trap scripts (each a pair of `KQTrap` boxes, `On` → `setRope`, `Off` →
  `setRope None`): Daventry `ropetrp1/2` (wizard's house), `ropetrp3/4`
  (cave behind the waterfall; they also switch the waterfall's collision),
  `churchr1/2`, `churchr3/4`; Barren `7rope1/2`, `basrope1/2`, `btrope1/2`,
  `btrope3/4`; Dead City `ropetp1a–d`, `ropetp2a–d`, `ropetp3a–f`,
  `ropetrap`/`ropetrp2`; Gnome `ropetp1a/b`, `2a/b`, `3a/b`; Ice World
  `ropeice1/2`, `ropetrp1/2`, `towrope1–4`; Swamp `fienrop1–4`,
  `telerop1/2`; Temple 1 `ropetp1a/b` (108 `KQConner::setRope` lines in
  all, `None` included).
- Saved-game keys (Connor): `ropeStatus`, `ropeLoc`, `ropeDir`,
  `ropeCamLoc1`, `ropeCamLoc2`, `ropeCam2Loc`, `ropeCamAng`,
  `ropeCamRange`, `ropeCamTopLimit`, `ropeCamBottomLimit`; the rope state:
  `doneRoping`, `reachedTop`, `doModeBefore`, `oldCamPos`, `conPos`,
  `onRope`, `hideIcon`, `firstToThird`, `bottomCamLimit`, `topCamLimit`,
  `conCamBottomLimit`, `conCamTopLimit`, `ropeCamLoc1/2`, `origStepUp`,
  `origStepDown` [traced; names as written in save files].

## Edge cases

- **No rope in the inventory**: traps do nothing, no icon (rule 4).
- **A camera spot with any coordinate 0** counts as "not set" (rule 3);
  no data spot has a 0 coordinate.
- **`setRopeCam2`, the angle and the limits are sticky**: they keep the
  last value set by any script. Daventry's `ropetrp2.cs` (wizard's house,
  top) sets `setRopeCam2`, the angle and `setRun 0` **only inside its
  first-visit block** (the `foundHouse` test); on later visits the top
  camera goes to whatever `setRopeCam2` spot was set last (the same spot,
  unless another top place, e.g. the cave's `ropetrp4.cs`, was used in
  between, in which case it goes to that place's spot) [data + traced].
  Keep this, it is the data's behaviour; log it.
- The **top limit is only used when starting from the foot**, the
  **bottom limit only when starting from the top** (rules 19, 21). Top
  scripts that set both (`7rope2`, `basrope2`, `btrope2`, `btrope4`,
  `ropeice2`, `towrope2/4`, `fienrop4`, `telerop2`) only use the bottom
  one; `btrope3` (a foot) only the top one [data + traced]. A bottom
  limit only raises the lowest camera height (it never lowers it).
- **Step heights**: only saved (and set to 0) on the foot side (`btoss`),
  but put back at both `taway` and `baway`; from the top they are not
  changed at all; a saved value of 0 is not put back [traced].
- Leaving a trap box **while the rope is on the pointer** puts it back
  (rule 6). Leaving it during the climb (Connor's real position is at the
  top while he climbs, rule 23) just changes the status and icons; the
  climb goes on [traced].
- From the foot, `bon` starts after the rope model's animation, not after
  `btoss`: if the rope model failed to load the climb would stall
  [inferred].

## Do not reproduce

- **The camera aim test (rule 11)**: it compares the camera's up/down
  angle with 0.4 rad twice ("above 0.4 or below 0.4"), so it passes for
  every angle but exactly 0.4, and the heading test after it is never
  reached. Keep the observable behaviour: **any world click with the rope
  in hand starts the climb**; don't build the angle test; keep the
  "don't see anything" remark only if a real test is ever wanted (it was
  probably meant to need the camera to look toward the rope).
- **Per-update steps**: the camera moves 50 per update, not per second.
  Implement **750 per second** (the original at 15 updates per second).
- The briefly restored first person on the click at a top place (rule 12,
  forced off again a moment later by rule 13): a flicker in the original;
  better not to switch views twice.
- **Throwing a rock while on the rope**: the original does something
  odd with it (the user's check, 2026-10-21; probably a bug). Ignore rock
  throws for the whole rope state instead.
- Unused: the game time noted on each status change (rule 5); the
  `RopeMode` text in the program (nothing uses it).

## How to check

- **Data**: `grep setRope` over the worlds' `kq` folders: 108 lines, 27
  `Bottom` and 27 `Top`; each `Top` place is 4,840–5,032 above its
  `Bottom` one, x and y within 225.
- **In the original** (the user): in Daventry, take the rope (mill), walk
  to the wizard's house rope foot (`ropetrp1`, near 147,646 207,586): the
  `HookUp` icon appears beside the spinning-mask place. Click the rope
  button: the pointer becomes the rope. Click anywhere (even the sky):
  Connor puts his weapon away, takes the rope out, throws, the rope
  model swings up, he climbs on. Hold forward: he climbs, the camera rises;
  let go: he hangs still; hold back: he climbs down. At the top he gets
  off and draws his weapon again if he was holding one (checked by the
  user in the original, 2026-10-21: all of this matches). Away from a rope place, clicking the
  rope button only gives "I am not near anything to climb." and the
  pointer stays normal.
- **Ours (to log)**: status changes with their trap name, the icon shown,
  each animation start/end of the sequence, the camera spot chosen (and
  which of the two), Connor's position at grab and at the end, the camera
  height range, the climb direction per update and the frame, hits
  ignored while roping.

## Open questions

1. ~~What the camera does if the rope is put back at the top~~: it stays
   where it is (the user's check, rule 9). What the camera flag set then
   does inside the game was not traced; nothing visible depends on it.
2. The camera placement at the end of `btoss` (rule 15): it is built from
   Connor's frame with the numbers 2,000, (0, 250, 350), (0, −500, 2,000),
   a tilt of 1.27 rad and a half turn added to the rope's heading, but the
   way they are combined was not read reliably. Stand-in until answered:
   leave the camera where it is until rule 18 moves it. Not checked in the
   original.
3. ~~Clicks and rocks while hanging~~: answered by the user's check
   (rule 30).
4. ~~Which weapon is drawn at the end~~: answered by the user's check
   (rule 25).
5. ~~Loading a game saved mid-climb~~: works normally (the user's check,
   rule 32).
6. ~~Which movement keys climb~~: only forward (up) and back (down); the
   others do nothing (the user's check, rule 22).
