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
| collider type | 4 | Connor's constructor (traced); bit 4 lets the Fall state start |
| collision box | x −200…200, y −150…150, z 0…1,600 | `GAME/Kq/Connor.kq` `[ConnerCollider]` |

`Connor.kq` also gives a sphere (radius 1,000 at z 700). Which shape is used
against buildings and objects is **not traced**; use the box.

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
   stops it.
3. **Across pass.** At that height, move to the wanted x, y. Walls stop or
   deflect it (see "Walls").
4. **Down pass.** Add this update's gravity drop (if gravity is on), then
   move down by **step down (325)**. If this pass hits something, that is
   what he stands on:
   - contact normal z > 0.8 → **on ground**, and the fall timer is reset
     to 0;
   - contact normal z ≤ 0.8 (steep) → not on ground; the pull is turned to
     point down the slope, and gravity is switched on when bit 4 is set in
     the collision result's flags (which flag that is, own or hit surface,
     is not fully traced; assume it is set for Connor).
5. **Ground check** (`keepOnGround`): find the terrain height and normal
   under the new x, y. If he is **at or below** the terrain, put him on it,
   with the same 0.8 test for "on ground" or "steep". If he is above it,
   nothing changes (see `buildings.md`, "Where a building stands").

What this gives, on flat ground: up 325, across, down 325, land on the same
floor. **Steps up to 325 units high are climbed without doing anything**,
and going down steps up to 325 per update follows the floor. Anything
higher is a wall; a bigger drop leaves him in the air.

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
  on a steep surface and pulls him back.

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

- With gravity **off**, walking off a high edge drops him 325 per update
  (the step-down pass, about 4,900 units/s) with no Fall state and no death.
- With gravity **on**, the fall rules above apply.

**Recommendation:** start with gravity on and log it. If the original is
later found to start with it off, it is a one-line change.

## Data used

- `GAME/Kq/Connor.kq`: collider box and sphere.
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
- Walk off a ledge more than 3,840 high (with gravity on): `jumpdie`, then
  Die.

## Open questions

1. Is Connor's gravity on when a world starts? (See above.)
2. Which collision shape (box or sphere) is used against buildings and
   objects, and how the building BSP is tested.
3. When `jumprun` / `jumpstop` play after a running jump.
4. What the "no crash" object flag means, and the game flag that blocks
   crashes.
5. Water (`submerged`, `WaterWalk`), ropes and pushing: not studied.
