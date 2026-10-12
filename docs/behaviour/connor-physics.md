# Gravity, steps, slopes, walls, jumping and falling

Study session A, 2026-10-11. Sources: `Mask.exe` (studied, described here in
our own words) and the game data. "Traced" = read in the exe; "measured" =
computed from the data with a throwaway script. Follows on from
`connor-movement.md` (walking, running, turning), which says how Connor
*wants* to move. This note says what happens to that movement: how it is
held on the ground, stopped by walls, and how jumps and falls work.

## Purpose

Enough for M2: Connor walks over Daventry's ground and buildings, climbs
small steps, does not walk up cliffs, slides off steep slopes, stops at
walls or slides along them, jumps, and falls (and can die from a fall).

## Units

- Game units (1 unit ≈ 1 mm, see `connor-movement.md`). Z is up.
- **"Per update"** means once per mover update. Several values below are
  per update, not per second; see "Do not reproduce".
- **How often updates happen (traced, Q30 c):**
  - The game's update tick is a **fixed timer of 1/15 s of game time**,
    set when the game starts. Each time it fires, it sends the per-update
    event to every object, then sets itself again 1/15 s later.
  - **Game time follows real time.** Each frame the game measures the
    real time since the last frame (millisecond clock) and moves game
    time on by that much, then runs every timer that has come due. A slow
    frame therefore runs several ticks at once (catch-up). One frame
    moves game time by at most 1 s (a few scripted moments lower this
    limit for a while).
  - **`KQGame::FPSLimit 15`** (set by `console.cs`) only limits how often
    a frame is drawn and game time is moved on: at most once per 1/15 s
    of real time. It doesn't change the tick. Without the line there is
    no limit, and the tick still runs at 15 per second.
  - So the update tick does **not** run faster on a fast PC.
  - **Not confirmed:** that Connor's three-pass move is run from this
    tick. Which code hands the mover Connor's animation step each update
    was not found (open question 8).
  - **Seen in the running original (2026-10-12):** Connor's position
    changed once per drawn frame, and frames came every 78 ms (about 12.8
    a second), sometimes 156 ms while he stood still. A few times the fall
    timer moved on by two steps (0.12) in one frame. That fits the rules
    above (a 1/15 s tick, with catch-up when a frame is late), but it was
    not checked further.

## Where the values live

Every moving object has a **mover**. Connor's is a `KQAnimMover`, which uses
the base `KQMover` rules below unchanged (traced: it only adds turning and
banking, already in `connor-movement.md`). Values it uses:

| Value | Connor | Source |
| --- | --- | --- |
| `keepOnGround` | on | mover default (`objects.md`); `Connor.kq` doesn't set it |
| `useGravity` | see "Is gravity on at the start?" | mover default off |
| step up | 325 | monster/Connor default (traced); `.kq` key `stepUpAmount` |
| step down | 325 | same; `.kq` key `stepDownAmount` |
| walkable | contact normal z > 0.8 (slope under 36.9°) | fixed in the code (traced) |
| gravity | drop per update `10000 × (t² − t_prev²)` (see "Gravity") | fixed; measured in the running original (the code reading gave 4 times less) |
| collision type | 4 | Connor's constructor (traced); bit 4 lets the Fall state start and makes steep ground switch gravity on |
| collision mask | 0x281B | Connor's constructor (traced); bit 0x800 makes the terrain count in the up and down passes |
| collision box | x −200…200, y −150…150, z 0…1,600 | `GAME/Kq/Connor.kq` `[ConnerCollider]` |

### Collision shape (traced; answers open question 2 in part)

- **Connor collides as a box**: the `boxfMin*`/`boxfMax*` values of
  `[ConnerCollider]`, in his own space (feet at z 0, so the box sits on
  the ground). Each pass moves the box from where it was to where it is
  going and tests it against what lies along the way.
- **The sphere is never used.** The game's collider reader reads only
  `boxfMinX/Y/Z`, `boxfMaxX/Y/Z`, `deactivateWidth` and `deactivateSpace`
  from a collider section, plus the display keys every object section has
  (`active3DFlag`, `isFar`, `tooFar`, `translucency`, `hazeMultiplier`,
  `hazeSolid`, `hidden`). The words `sphereRadius`, `sphereX/Y/Z` don't
  appear anywhere in `Mask.exe`.
- The other keys in `[ConnerCollider]` (`collisionDetail=4`, `mask=65535`,
  `type=4`) are **not read from that section either**. Connor's real type
  and mask come from his constructor (table above). An object's type and
  mask can be set in its main `.kq` section with `colliderType` and
  `colliderMask` (`Connor.kq` has neither), and by the script commands
  `KQObject::collisionType` and `KQObject::collision` (a negative number
  clears bits).
- **Which pairs collide:** two objects are tested against each other when
  the first one's mask shares a bit with the second one's type, **or** the
  second one's mask shares a bit with the first one's type. Either side is
  enough. So an object with mask 0 can still be hit by Connor if Connor's
  mask covers its type.
- Defaults: an object that sets nothing has type 0x41 and mask 0. Object
  `.kq` files in the game use `colliderMask` values 0, 1, 2, 4 (316 files),
  11, 13, 15, 31 and 2055.
- **Not traced:** how the box is tested against a building's surfaces
  (`.dig`) and against another object's box; what each mask/type bit stands
  for, other than 0x800 (terrain, in Connor's mask) and 4 (Connor's type).

The mover also has `MaxSlopeDot` (default 0.819 = cos 35°), and the
`KQMonster::walkSlope` command sets it. **Connor's movement code doesn't use
it**: the 0.8 test above is fixed in the code. Where `MaxSlopeDot` is read
was not found; probably monster path-finding.

## Rules

### One update of the mover (traced)

The animation step and turning give a **wanted position** (x, y, z). The
mover then works in three passes, each stopped by collisions:

1. **Start.** If Connor was on the ground last update, or gravity is off:
   clear "on ground", point the pull straight down (0, 0, −1), set the air
   counter to 0. Otherwise (in the air, gravity on): add 1 to the air
   counter. See "Falling off an edge" for what happens at 5.
2. **Up pass.** Move straight up by **step up (325) + the wanted change in
   height** (the animation's own up/down step; 0 when walking). A ceiling
   stops it. This pass runs every update, also in the air (Q23).
3. **Across pass.** At that height, move to the wanted x, y. Walls stop or
   deflect it (see "Walls"). **The terrain is not tested in this pass**
   (Q26): he can move into a hillside here, and the down pass sorts it out.
4. **Down pass.** Add this update's gravity drop (if gravity is on), then
   move down by **step down (325)**. The terrain counts as something he
   can hit in this pass (see "The terrain in the passes" below). If this
   pass hits something, that is what he stands on:
   - contact normal z > 0.8 → **on ground**: he is put exactly on the
     contact point, and the fall timer is reset to 0;
   - contact normal z ≤ 0.8 (steep) → not on ground; the pull is turned to
     point down the slope, and gravity is switched on (Connor's own
     collision type has bit 4, so this always happens for him). He is
     **pushed off the surface** as for a glancing wall (see "Walls"): the
     rest of the move is projected onto the surface, which moves him
     sideways down the slope.
5. **Ground check** (`keepOnGround`): find the terrain height and normal
   under the new x, y. If he is **at or below** the terrain, put him on it,
   with the same 0.8 test for "on ground" or "steep". If he is above it,
   nothing changes (see `buildings.md`, "Where a building stands"). It only
   changes his height, never x or y. After step 4 this rarely does
   anything; it catches what the passes missed.

What this gives, on flat ground: up 325, across, down 325, land on the same
floor. **Steps up to 325 units high are climbed without doing anything**,
and going down steps up to 325 per update follows the floor. Anything
higher is a wall; a bigger drop leaves him in the air.

### The terrain in the passes (traced, Q26)

The terrain is not a collision shape. Instead, at the end of the **up** and
**down** passes (not the across pass) the game compares the terrain height
under where the pass ended with the height Connor ended at:

- If the terrain there is **higher** than his end height, the pass counts
  as a hit. The contact point is (end x, end y, terrain height), and the
  contact normal is the terrain's normal there.
- If the same pass also hit a building or object, whichever contact is
  **nearer to where the pass started** wins.
- If the terrain is at or below his end height, nothing happens. The
  terrain does **not** pull him down to it.
- The push-off after a steep or glancing hit is itself carried out as an
  across move, so it ignores the terrain too (walls and objects still stop
  it). It is tried at most 3 times in one update; after that he stays at
  the last contact point.

This only happens for movers whose collision mask has the "terrain" bit.
Connor's does: his constructor sets it, and `Connor.kq` doesn't change it.

Consequences:

- **Steep hill (over 36.9°), running into it.** Each update the across pass
  takes him 168 units into the hill. The down pass ends below the hillside,
  so it hits it, steeply. He is pushed back down the slope onto its surface
  and gravity turns on. On a slope of angle θ with this update's gravity
  drop d, his forward gain is about 168·cos²θ − d·sinθ·cosθ (computed from
  the rules). Gravity keeps growing, because the fall timer isn't reset on
  steep ground, so the push-back grows each update. He is never "on
  ground" there, so after 5 updates (about 0.4 s) the Fall state starts
  and he slides back down in it. Example at 45° (computed): 80, 71, 62,
  53, 44 units forward, about 310 in all. The formula holds both in the
  first update, where the pull is still straight down, and in later ones,
  where the drop goes along the slope as projected (see "Gravity"):
  the projected drop's backward part and the smaller push-off add up to
  the same thing.
  **Checked by the user in the original (Q29), and not explained by the
  traced rules:** walking straight into a steep slope, the walk animation
  keeps playing. He is pushed into and out of the slope over and over,
  **barely gaining height**, and never falls. Running, the same with a
  little more height and a more chaotic back-and-forth, but much less
  height than the rules give (B's code: 60, 60, 45, 31, 17 units up, then
  a fall). He only stumbles into Fall when he gets high enough to be off
  the ground for 5 updates, which head-on rarely happens. It does happen
  when he runs slightly downhill along the slope and then turns into it.
  Part of this is the exact count (rule 1 in "Falling off an edge"): a
  push-off that lands him back on the flat ground in front of the slope
  within 5 updates resets the counter. **Why he gains so little height
  is open** (open question 7).
- **Going downhill on walkable ground.** The down pass only lands him if
  the ground under his new spot is above where it ended, so a slope that
  falls faster than the gravity drop leaves him in the air for an update
  or more. The drop is 9 in the first update after leaving the ground,
  then 27, 45, ...; 9·k² in all after k updates. On most slopes he lands
  again within a few updates, which the user sees in the original as a
  slight camera "bounce" going down very steep walkable slopes. Copying
  the bounce exactly is not required (user's call). The B session's log
  (`--hold 1,0,1`, in the air every other update on a slope with normal z
  0.996) is what the original does too.
  **Checked by the user in the original (walking and running, the
  steepest walkable hills in Daventry): he never falls on walkable
  ground, however steep.**
  The traced rules disagree: by them, if the ground drops more than about
  45 units per update for 5 updates (running down slopes over about 15°),
  the air counter should start Fall. So something keeps him on walkable
  ground that this study did not find (open question 6). Searched without
  finding it: the gravity step (instruction by instruction), every reset
  of the fall timer, the ground check's comparison, the terrain lookup,
  and every place that changes Connor's step-up/step-down values.
  **Recommended rule (our own design, not traced; it reproduces what the
  user saw):** in the down pass, if he was on walkable ground last update
  and there is walkable ground no more than **step down (325) + this
  update's drop** below where the pass ends, put him on it (on ground,
  fall timer 0). So he follows any walkable slope at walking or running
  speed. A drop bigger than that (a ledge) or steep ground still leaves
  him "not on ground", and Fall starts after 5 updates as the user saw.
  This loses the slight camera bounce, which the user said need not be
  copied.
- **Going down unwalkable ground (over 36.9°).** Every down pass lands on
  steep ground, so he is never on the ground. After 5 updates Fall starts,
  and he slides down the slope in it. **Checked by the user in the
  original,** while holding the walk key: he steps onto the slope, drops
  a little, then the falling animation starts and he falls rapidly down
  the slope.

### Gravity (traced; strength measured in the running original)

- A fall timer `t` (seconds) belongs to the mover. Each update with gravity
  on, `t` grows by **0.06**, and the drop for that update is
  **`10000 × (t² − t_prev²)`**: 36, 108, 180, 252, 324, 396, … (72 more
  each update). One update's drop is capped at 5,000 (traced, not seen in
  the measurement).
  - **Measured 2026-10-12** (open question 8): read from the original's
    memory while it ran, every update in the air of a ledge walk-off and
    a standing jump, 18 updates with t from 0.06 to 0.90. The drop was
    **exactly 4.000 times** `5000 × (t² − t_prev²) / 2`, the formula
    this note first gave from reading the code, at every one of them.
    The timer values (+0.06 per update) were as traced. Where the factor
    of 4 comes from in the code was not found; the measured rule is what
    to implement.
  - In everyday terms: a constant acceleration of 20,000 units/s² if the
    0.06 per update is taken as seconds.
- The drop is applied along the pull direction: straight down in the air,
  down the slope after touching a steep surface. Exactly (traced, Q28 a):
  the vertical drop d is **projected onto the pull**. He moves along the
  pull by d·sinθ, not by d (θ is the slope's angle). That is d·sinθ·cosθ
  back horizontally and d·sin²θ down. On a slope of angle θ that is a
  slide accelerating at 20,000 × sin θ. (Measured: on two slopes the
  height lost per update was the drop × sin²θ to within 0.3 units.) The pull is reset to straight
  down at the start of every update that begins on the ground. After any
  down-pass contact it is set to the surface's downhill direction.
- If the pull points less than 30° below horizontal (for example after
  touching flat ground), the timer is reset to 0 instead of advancing, so
  there is no drop that update.
- The timer is reset to 0 when the **down pass** lands him on walkable
  ground (step 4). The ground check (step 5) does not reset it.

### Slopes (traced)

- Walkable: the surface's normal has z > 0.8, i.e. slopes up to about
  **36.9°**. Same rule on terrain and on buildings.
- Steeper: he is not on the ground, gravity turns on (see the flag note in
  step 4), and he slides down the slope. Sliding for 5 updates in a row
  starts a fall (next section), so a long steep slope ends in the fall
  animation and, if the drop is big enough, death. Checked by the user in
  the original.
- Walking up a slope steeper than 36.9° fails because each down pass lands
  on a steep surface and pushes him back (see "The terrain in the passes").

### Falling off an edge (traced)

1. **The air counter, exactly (traced, Q29):** at the **start** of each
   update, before any pass:
   - if the previous update ended **on the ground**, or gravity is off,
     the counter is set to 0 (and the pull is reset to straight down);
   - otherwise (it ended not on the ground, gravity on) the counter goes
     up by 1, and if it is now **5 or more**, Fall is asked for (unless
     he is jumping or his collision type lacks bit 4).

   So Fall is asked for at the start of the update that follows **5
   updates in a row that all ended not on the ground**. It is asked for
   before that update's own passes, so even if he lands in that update,
   the fall has already started. Four air updates followed by a landing
   in the fifth start **no** fall.
   **"On the ground"** means only this: in that update the down pass or
   the ground check put him on a surface with normal z **above 0.8**
   (walkable). Touching steep ground, being pushed off it, or standing on
   it never counts. There is **no height or speed condition** anywhere in
   the count or the Fall request (checked instruction by instruction).
   The start height is his height at that moment. **Connor's current
   state decides whether to accept** (traced):
   - **Move** (walking, running, stepping back, turning on the spot) and
     **Wait** (standing) both **accept** Fall in normal play. Both refuse
     it while the player's controls are taken away (a flag set by puzzles
     such as the tile puzzle, and by a few scripted moments).
     (Corrected: an earlier version of this note said Move refuses Fall.
     That was a misreading of the rule.)
   - A refused request is kept as "pending" (the game's own name for it
     is `nextState`) and tried first at the next state change. When
     Wait's current animation finishes, it also retries the pending state.
     Each update in the air (from 5 on) asks for Fall again anyway.
   - **Checked by the user in the original,** holding the walk key: off a
     high ledge he steps fully off, drops a little, then the falling
     animation starts and he falls **straight down**. His forward
     movement stops, which fits the walk steps ending when Fall replaces
     the walk animation with `jumphold` (which has no steps of its own). On an unwalkable slope the same happens,
     but he slides rapidly down the slope.
2. **What entering Fall does depends on the state he comes from**
   (corrected 2026-10-12; an earlier version said the timer was always
   set):
   - **From Move or Wait** (walking off a ledge or onto a steep slope):
     gravity is **not touched at all**: the timer keeps running from the
     5 updates in the air (0.30 s). The start height for the death rule
     is his height when Fall was requested. `jumphold` plays, repeating.
   - **After a jump** (`jump`, `runjump` or `backjump`): the fall timer
     is set to **0.36 s** (previous 0.30 s), the start height is lowered
     by a fixed amount (not read), and the matching hold animation plays
     (`jumphold`, `runJumpHold` or `backJumpHold`).

   Neither case touches the pull direction (Q30 a). If the last contact
   was steep, each drop stays projected down that slope until he lands.
   So walking off a ledge, the drops go 36, 108, 180, 252, 324 in the 5
   air updates (900 in all), then 396, 468, 540, … in Fall (72 more each
   update). (Measured in the running original 2026-10-12, including the
   timer carrying on from 0.30 after a walk-off and being set to 0.36,
   previous 0.30, after a standing jump. An earlier version of this note
   gave 4 times smaller drops; see "Gravity".) The hold animations have no movement of their own (`jumphold`:
   3 frames, all zero). Nothing else in Fall changes gravity or moves
   him. Its per-update check only does rule 3 and picks the landing or
   death animation.
3. Each update in Fall: if he dropped **less than 50** units since the last
   update, a "stuck" counter goes up; otherwise it goes back to 0. After
   more than 5 "stuck" updates he counts as on the ground.
4. The fall ends when he is on the ground, the stuck count passes 5, or he
   moved **up** this update.
5. **Death:** if the start height minus the landing height is more than
   **3,840**, he dies. The die animation (`jumpdie`) plays with
   `jumpdie.aud`, and when it finishes he goes to the Die state. Otherwise
   `jumpend` plays, then he goes back to walking (if a move key is held) or
   standing.

### Jumping (traced; arcs measured)

1. **When the Jump key works** (action `Jump`, key Insert):
   - he is on the ground, **or** within 250 units above the terrain under
     him (then the terrain's normal is used);
   - and the ground is walkable (normal z > 0.8), **or** it is steep but he
     is not facing uphill (the slope's normal doesn't point toward his
     back).

   Some puzzle states (tile and scale puzzles) handle the key their own way,
   which is not covered here.
2. **Which jump:** `runJump` if his current movement is `Run`, `backJump`
   if it is `Back`, otherwise `jump` (standing jump; it also covers walking).
3. **During the jump animation, gravity is switched off and step up/down
   are 0.** The arc comes entirely from the animation's per-frame steps
   (the same mechanism as walking). Walls and ceilings still stop him.
   Measured from `GAME/Common`:

   | Movement | File | Time | Forward | Highest | Height at the end |
   | --- | --- | --- | --- | --- | --- |
   | `jump` | `C_jump.kqs` | 1.333 s | 1,423 | 1,697 | +873 |
   | `runJump` | `C_rnjmp.kqs` | 1.333 s | 6,356 | 1,859 | +623 |
   | `backJump` | `C_bkjmp.kqs` | 1.2 s | −2,566 (backward) | 2,156 | +788 |

   ("Forward" is along −Y of the model, as for walking.) The standing jump
   stands still for its first 10 frames, then moves.
4. **When the animation ends:** gravity on, step up/down back to 325, then
   the Fall state starts with the jump's kind. The death limit is measured
   from **1,280 below** the height at that moment, so after a jump he dies
   if he lands more than **5,120** below where the animation ended. The
   hold, end and die animations follow the kind:

   | Kind | While falling | Landing | Death |
   | --- | --- | --- | --- |
   | `jump` (and walking off an edge) | `jumphold` | `jumpend` | `jumpdie` |
   | `runJump` | `runJumpHold` | none: straight back to running or standing | `runJumpDie` |
   | `backJump` | `backJumpHold` | `backJumpEnd` | `backJumpDie` |

   `runJumpHold` has transitions to `jumprun` / `jumpstop` in `Conner.anm`;
   when those play is not traced.
5. **Hitting something steep during a jump** ends the jump: gravity on,
   steps back to 325.
   - Landing on it (down pass) during `jump` or `runJump`: **crash** (state
     Special, movement `crash`, `c_crash.KQS`, `crash.aud`).
   - Running into it (up or across pass): `runJump` crashes; `jump` plays
     `Stop` and he stands (as for walls below).
   - `backJump` never does either.

### Walls (traced)

When the across pass hits something:

- **Head-on** means the angle between his movement and the wall's face is
  under **0.35 rad (about 20°)**, and the movement isn't straight up or
  down. During a jump, any surface with normal z < 0.8 also counts.
  This only applies while he is moving or jumping (states Move and Jump).
  - Movement `Run`, `stoprun`, `WaterRun`, `runwaterrun`, `runJump` or
    `PreBirth`: **crash**, as above.
  - Any other movement (walking, standing jump): he plays `Stop` and stands
    still (state Wait).
  - Neither happens against some objects (the hit object's collision info
    says so; meaning not traced), during `backJump`, or while a game flag
    blocks it.
- **Glancing:** remove the part of the remaining movement that goes into the
  wall, i.e. **slide along it**, and try again. This happens at most 3 times
  in one update; after that he stops at the contact point.

This matches the manual: straight into an object stops him; at an angle he
glances off and keeps going.

**Measured in the running original (2026-10-12),** against a straight
building wall, reading his mover from memory:

- **Which angle counts as head-on:** measure it between his facing and
  the wall's normal (straight into the wall). Head-on at 1.6° and 11.8°;
  glancing at 21.6° and 35°. That fits a 20° (0.35 rad) limit measured
  from the normal. (The wording "angle to the wall's face" above should be
  read this way.)
- **Walking head-on:** he stops at the contact point at once. `Walk`
  switches straight to `Stop` (standing), with **no** stop animation.
- **Running head-on:** `crash` plays for 5 updates (about 0.5 s). During
  it he is pushed **backwards** (straight back from his facing) by about
  440–600 units in all (about 63 per update), then stands. If run is
  still held, he starts running again (`stoprun`) and crashes again
  (seen twice in a row).
- **Glancing (walking or running):** he slides along the wall with the
  walk or run animation still playing. Each update's movement along the
  wall is the normal step × sin(angle from the normal): at 21.6°, 27–53
  per update instead of 72–143 (× 0.37); at 35°, 119–238 instead of
  208–416 (× 0.57). Nothing is lost beyond the part going into the wall.
  When the wall ends, he carries on at full step in his facing
  direction.

### Is gravity on at the start?

**Not traced.** The mover starts with gravity off, and neither `Connor.kq`
nor Connor's constructor turns it on. It is switched on by the end of every
jump, by landing on a steep surface, and by scripts (`setUseGravity
Connor 1` after cutscenes in IceWorld). The scripts turning it back to 1
suggest that 1 is the normal value.

- With gravity **off**, walking off a high edge leaves him **hovering** at
  the same height: the up pass lifts him 325 and the down pass lowers him
  325, and the ground check only ever raises him. No Fall state starts,
  because the air counter only counts with gravity on. (Corrected for Q23:
  this note used to say he drops 325 per update. That was wrong: the up
  pass is not skipped in the air.)
- With gravity **on**, the fall rules above apply.

**Recommendation:** start with gravity on and log it. If the original is
later found to start with it off, it is a one-line change.

## Data used

- `GAME/Kq/Connor.kq`: collider box (the sphere and the section's
  `mask`/`type`/`collisionDetail` lines are not read).
- `GAME/Kq/Conner.anm`: movement names and sounds (`jump`, `runJump`,
  `backJump`, `*Hold`, `*End`, `*Die`, `crash`, `jumprun`, `jumpstop`).
- `GAME/Common/C_jump.kqs`, `C_rnjmp.kqs`, `C_bkjmp.kqs` and the hold/end
  files: per-frame steps (the jump arcs).
- `.kq` keys a world object may set: `stepUpAmount`, `stepDownAmount`,
  `keepOnGround`, `useGravity`, `MaxSlopeDot`, `colliderType`.

## Edge cases

- Under a low ceiling the up pass is stopped, so he can't climb a step he
  could climb in the open.
- A drop of 325 or less per update is followed as "stairs down", without
  any fall.
- Jumping while within 250 units above the terrain (just after stepping off
  a small edge) still works.
- Falls that end against a slope (no progress for 6 updates) end like a
  landing.

## Do not reproduce

- **Per-update physics.** The fall timer grows by a fixed 0.06 s per
  update, and steps are 325 per update. At 15 updates per second, 0.06 per
  update makes gravity run at 0.9 × real time. Recommended: run the physics
  at a fixed 15 updates per second (simplest, matches the original), or
  scale both values by the real time step and accept small differences.
  Say which in DESIGN.md.
- **`walkSlope` bug:** `KQMonster::walkSlope` takes the cosine of the
  number in degrees *without* converting to radians. Only matters for
  monsters (M5); note it then.

## How to check

Log per update: position, on-ground flag, gravity on/off, fall timer, air
counter, state (Wait/Move/Jump/Fall/crash), movement name, contact normal.

- Stand on flat ground: z constant, on ground every update, fall timer 0.
- Standing jump on flat ground: 1.333 s of `jump`; highest point about
  1,697 above the start; at the end about 873 above and 1,423 forward;
  then Fall with `jumphold`, landing about 0.33 s later (about 5 updates at
  15 per second) with `jumpend`.
- Running jump: about 6,356 forward; back to running with no landing
  animation.
- Walk into a wall head-on: `Stop`, standing. Run into it head-on: `crash`.
  At 45°: he slides along the wall.
- Walk up a ramp under 36°: he climbs. Over 37°: he slides back.
- Run straight at a steep hillside (Q26): about 0.4 s and about 300 units
  of climb, then `jumphold` and a slide back. Log the contact normal of
  the down pass. It should be steep, and the push-back should grow each
  update.
- Walk or run down the steepest walkable slope you can find: **never** a
  Fall (checked in the original by the user, both walking and running).
  Log the air counter: it must never reach 5 on walkable ground.
- Hold walk and step off a high ledge, or onto an unwalkable slope: about
  5 updates of dropping, then `jumphold`. Forward movement stops; on the
  slope he slides down it (both checked in the original by the user).
- Walk off a ledge more than 3,840 high (with gravity on): `jumpdie`, then
  Die.

## Open questions

1. Is Connor's gravity on when a world starts? (See above.)
2. ~~Box or sphere?~~ The box (see "Collision shape"). Still open: how
   the box is tested against building surfaces and other objects' boxes,
   and what the mask/type bits mean.
3. When `jumprun` / `jumpstop` play after a running jump. **Measured
   (2026-10-12), one jump each:** run still held at the landing →
   `jumprun` (3 updates, about 830 units) and back to `Run`; run let go →
   `jumpstop` (13 updates, 1.0 s, about 1,860 units forward) and then
   standing. Both jumps covered about **6,480** units from the start of
   `runJump` to the landing (`runJump` 17 updates, 1.25 s; the table
   above says 6,356 for the animation alone).
4. What the "no crash" object flag means, and the game flag that blocks
   crashes.
5. Water (`submerged`, `WaterWalk`), ropes and pushing: not studied.
6. ~~What starts the fall while a move key is held?~~ The air counter
   does: Move accepts Fall (see "Falling off an edge"), and the user's
   checks match. **Still open:** what keeps him on walkable ground going
   downhill. The user saw no fall even running down the steepest walkable
   hills, but the traced rules predict one on slopes over about 15°. Not
   found after three searches (see "Going downhill on walkable ground").
   Until it is found, use the recommended rule there.
   **Measured in the running original (2026-10-12),** reading his mover
   from memory on a hill whose slope changes from 19° to 37° (the
   walkable limit):
   - Walking down 34–37°: every other update he ends **off the ground**,
     having dropped exactly 36 (the first gravity step, t = 0.06), and
     lands in the next update (25–85 down). The air counter goes 0, 1,
     0, 1, … and never passes 1. On 19–30° he stays on the ground every
     update.
   - Running down 29–37°: off the ground for 1 to 3 updates at a time
     (once 4), dropping 36, 108, 180, 252 as gravity builds, then lands
     and the count starts again. It **never reached 5**, so no Fall.
   - Nothing pulls him onto the slope beyond gravity: the drops are
     exactly the gravity rule (now 4 times the earlier figure, see
     "Gravity").
   **Likely explanation (not proven):** with the measured gravity he gets
   back to the slope within 1 to 4 updates, so the count of 5 is never
   reached. The earlier figure (4 times weaker) kept him in the air
   longer, which would explain why our version falls here. Check: run
   the same walk with the corrected gravity; if the air count stays
   under 5, the recommended rule in "Going downhill on walkable ground"
   is not needed.
7. Why Connor gains so little height walking or running into a steep
   slope in the original (user's check, Q29). Traced and found as the
   note says: the across pass ignores the terrain; the down pass hits the
   hillside at (end x, end y, terrain height) with the terrain's normal;
   the push-off projects the rest of the move onto the surface; it is
   carried out as an across move, so the terrain is ignored there too;
   the pull is set downhill and only reset at the start of an update that
   began on the ground. Nothing undoes the across move, and nothing adds a
   speed or height condition. Not looked at yet: who hands the mover its
   wanted position each update and what height it uses, and whether the
   end of the update moves him again. This may be the same unfound
   mechanism as open question 6.
   **Measured in the running original (2026-10-12):** running straight
   into a 45–54° hillside from flat ground, he got at most **80 units**
   up (usually 5–40). Each try lasts 2 to 4 updates off the ground
   (gravity drops 36, 108, 180, 252), and then the next update puts him
   back on the flat ground at the foot. He never reached 5 updates in
   the air, so no Fall and no falling animation, for about 12 seconds of
   holding run. **Likely explanation (not proven):** the same as open
   question 6, the measured gravity being 4 times stronger. Check: run
   at a steep hillside with the corrected gravity and compare with these
   numbers.
   **Also seen, not understood:** standing on flat ground at the foot
   with a horizontal pull, the fall timer kept growing (0.06 up to 0.42)
   for a few updates while he counted as on the ground, before going
   back to 0. Being on flat ground, he didn't move down. "Gravity" says
   the timer is reset when the pull is less than 30° below horizontal;
   this doesn't fit that. It may be a reading taken part-way through an
   update. Not checked.
8. **Falls are much faster in the original than these rules give** (user,
   Q30). He follows a steep slope down "sucked into the surface", and in
   Fall he gains falling speed much faster. Traced and matching the rules:
   the tick (15 per second of real time, whatever the frame rate), the Fall
   start (rule 2; pull unchanged), gravity 5,000 with +0.06 per
   update. Not found: which code hands Connor's animation step to the mover
   each update. A first guess (the move routine runs more often than the
   tick) was ruled out by the user's recordings below.
   **User's recording (running onto a slope too steep to walk, two
   places, same result):** once about the middle of his body reaches the
   start of the drop, he moves down quickly, following the slope; the
   falling animation starts **18 frames of a 60 fps video (0.3 s, 4
   changed game frames)** after the first frame that moves down, and he
   keeps sliding down the slope fast. The game felt choppy, about 15
   frames per second. By these rules the falling animation shows 5
   updates after the first update in the air: 1/3 s, 20 video frames.
   18 is within one update of that (each update spans about 4 video
   frames), so **the start of the fall matches the 15-per-second rules**
   and does not support the guess above. (A first count of "4 frames"
   was changed game frames, not video frames; it briefly looked like
   support for the guess.) What is still unexplained is the speed: how
   fast he goes down the slope and falls, not when Fall starts.
   **User's ledge fall (straight down, flat ground below, a drop of a bit
   under 2.5 Connor heights):** 23 frames of a 60 fps video from the
   first falling-pose frame to the first landing-pose frame, about 6
   updates. Connor is 1,600 units tall (collision box), so the drop was
   about 3,500 to 3,900 units. (A first comparison used 720 for his
   height, which was wrong.) He survived, which fits: the death limit,
   3,840 counted from the Fall start, is 2.4 Connor heights. By these
   rules (rule 2: walking off, 225 units before Fall, then 99, 117, …)
   the part after the Fall start, about 3,300 to 3,700 units, takes 15 to
   16 updates (about 60 to 64 frames). In 6 updates the rules give only
   864. So **the original falls about 4 times as far as these rules in
   the same time.** The start of the fall is on time (slope recording),
   so the air count is not faster; only the distance per update is.
   **Checked and ruled out (2026-10-12):** the fall animation moving him
   (`jumphold` has no movement); an extra drop in Fall (its per-update
   check only watches the drop and picks the end animation); a second
   call of the gravity step (one per mover update, from one place); the
   object move handler splitting a move into several (it runs the move
   once, plus a lowered second move only for an object standing in
   water). "The mover runs more often" is also out: the air count runs
   in the same routine, so Fall would start early, and it doesn't.
   **Not checked:** what else feeds the wanted height into the mover
   (the up pass adds the wanted change in height, so a downward wanted
   height from somewhere else would add to the fall); the drop routine
   adds a stored base vector to each drop; what it holds and what sets
   it is not traced. Stopped after these attempts.
   **Answered by measurement (2026-10-12):** Connor's mover was read from
   the original's memory while it ran (from outside the game, no
   debugger). Every update in the air, the drop was exactly 4 times the
   traced formula, at the traced timer values, and the stored base
   vector stayed 0. The new rule is in "Gravity". It accounts for the
   user's ledge fall (about 4 times as far per update). **Still open:**
   where the factor of 4 is in the code; nothing else in the fall
   differed from the rules.
   **Also from the user:** he never leaves the ground by running fast over
   a crest or down a walkable slope; the game seems to always put him on
   the ground below him (see open question 6). Only with no ground below
   (a real ledge) does he lose his forward speed and fall straight down.
   The only case where he keeps forward speed in the air is a jump.
