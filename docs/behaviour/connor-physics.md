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
- **"Per update"** means once per mover update. The game runs at most 15
  updates per second (`KQGame::FPSLimit 15`). Several values below are per
  update, not per second; see "Do not reproduce".

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
| gravity | 5,000 units/s² | fixed (traced) |
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
  steep ground. After 5 such updates (about 0.4 s) the air counter starts
  the **Fall** state, and he slides back. So he gets only a few hundred
  units up before stopping. Example at 45°: 80, 71, 62, 53, 44 units
  forward, about 310 in all.
- **Running downhill.** The down pass only lands him if the ground under
  his new spot is above where it ended, so a slope that falls faster than
  the gravity drop leaves him in the air for an update or more. The drop
  is 9 in the first update after leaving the ground, then 27, 45, ...;
  9·k² in all after k updates. On a gentle slope he lands again the next
  update, so the air counter never reaches 5 and nothing shows.
  **Derived, not checked in the original:** if the ground falls more than
  about 45 units per update, he hasn't landed after 5 updates and the
  **Fall** state starts (`jumphold`), then `jumpend` when he lands. That
  is slopes over about 15° when running (168 per update) and over about
  31° when walking (75 per update). The B session's log (`--hold 1,0,1`,
  in the air every other update on a slope with normal z 0.996) is what
  the original does too.

### Gravity (traced)

- A fall timer `t` (seconds) belongs to the mover. Each update with gravity
  on, `t` grows by **0.06**, and the drop for that update is
  `5000 × (t² − t_prev²) / 2` (constant acceleration of 5,000 units/s²,
  starting from rest at `t = 0`). One update's drop is capped at 5,000.
- The drop is applied along the pull direction: straight down in the air,
  down the slope after touching a steep surface. On a slope of angle θ that
  is a slide accelerating at 5,000 × sin θ.
- The timer is reset to 0 whenever he lands on walkable ground (step 4 or
  5 above), so walking never builds up speed.

### Slopes (traced)

- Walkable: the surface's normal has z > 0.8, i.e. slopes up to about
  **36.9°**. Same rule on terrain and on buildings.
- Steeper: he is not on the ground, gravity turns on (see the flag note in
  step 4), and he slides down the slope. Sliding for 5 updates in a row starts a fall (next section),
  so a long steep slope ends in the fall animation and, if the drop is big
  enough, death.
- Walking up a slope steeper than 36.9° fails because each down pass lands
  on a steep surface and pushes him back (see "The terrain in the passes").

### Falling off an edge (traced)

1. With gravity on, every update that ends **not on the ground** adds 1 to
   the air counter. At **5**, and if Connor is not jumping, he goes into
   the **Fall** state. The start height is his height at that moment.
2. Entering Fall sets the fall timer to **0.36 s**, as if he had already
   been falling that long (speed 1,800 units/s), and plays `jumphold`
   (repeating).
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
- Run straight at a steep hillside (Q26): about 0.4 s and a few hundred
  units of climb, then `jumphold` and a slide back. Log the contact normal
  of the down pass. It should be steep, and the push-back should grow each
  update.
- Run down a slope of 20° or more in the original game and watch whether
  Connor switches to the falling pose. The rules predict he does (see
  "The terrain in the passes"); this has not been checked.
- Walk off a ledge more than 3,840 high (with gravity on): `jumpdie`, then
  Die.

## Open questions

1. Is Connor's gravity on when a world starts? (See above.)
2. ~~Box or sphere?~~ The box (see "Collision shape"). Still open: how
   the box is tested against building surfaces and other objects' boxes,
   and what the mask/type bits mean.
3. When `jumprun` / `jumpstop` play after a running jump.
4. What the "no crash" object flag means, and the game flag that blocks
   crashes.
5. Water (`submerged`, `WaterWalk`), ropes and pushing: not studied.
