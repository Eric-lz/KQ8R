# Third-person and first-person camera

Study session A, 2026-10-11; "Obstacles" added 2026-10-13. Sources: game data files, the game's own
manual (`MASK/Manual.pdf`, "Controlling the Camera"), and `Mask.exe`
(studied, described here in our own words). Covers the gameplay camera that
follows Connor. Scripted cutscene cameras (`KQCamera::addCamLoc`,
`cutToCamLoc`, `flyToCamLoc`) and camera shake are only summarised.

## Purpose

Keep Connor in view while he moves, let the player swing, zoom and tilt the
camera, avoid putting the camera under the ground, and offer a first-person
view.

## Units

Game units (very likely millimetres; see `connor-movement.md`), radians.
"Per update" means per game update; the original runs at 15 updates per
second (`KQGame::FPSLimit 15` in `console.cs`).

## Controls

From `MASK/Connor.sae`, `MASK/Camera.sae` and the manual:

| Input | Action | Effect |
| --- | --- | --- |
| Hold right mouse + move mouse left/right | `MoveCamera` | Swing the camera around Connor |
| Hold right mouse + move mouse forward/back | `MoveCamera` | Zoom in/out (normal mode) or tilt (fixed mode) |
| Shift (held) | `ModifyDn`/`ModifyUp` | While held, swap the two modes' meaning of forward/back mouse |
| Page Up / Page Down | `RotateX ±0.05` | Tilt up/down (normal mode) or zoom (fixed mode) |
| Home | `ResetX` | Put the tilt back to default |
| F11 | `CameraChangeMode` | Switch normal ↔ fixed-distance mode |
| F2 | `CameraFirstPerson` | Switch third ↔ first person |

The manual's own words: the camera "will attempt to keep Connor in the
middle of the screen"; "continuing to hold the right mouse button down
while Connor moves will maintain the camera in the same relative position";
tilt "will automatically reset itself as soon as Connor moves".

`Camera.sae` also binds letter keys (`a z s f e c o p k l j m`) to direct
camera moves and turns (`MoveX/Y/Z`, `RotateX/Y/Z`, `...2`). These shift
the camera itself by fixed steps and look like development helpers. Not
needed for normal play; keep them behind a debug switch.

## Rules

### What the camera looks at

1. The camera follows one object, normally Connor
   (`KQCamera::follow Connor`; `KQCamera::follow none` stops following,
   used around cutscenes). With `follow none` the camera stays
   **exactly** where it is, place and view, until the next `move
   KQCamera`, `follow`, fly/cut or shake (traced 2026-10-17, Q89 c).
   `follow <object> 1` only turns the camera toward the object without
   moving it (game's usage text). For `follow Connor` after a cutscene,
   see "Shake and cutscene cameras".
2. It aims at a point **1,400 units above** the followed object's base
   (Connor's collision box is 1,300 tall, so a little above his head).
3. The view direction is from the camera to that point, so Connor stays in
   the middle of the screen.

### Distance (normal mode, the default)

1. The camera does **not** sit at a fixed spot behind Connor. It stays where
   it is until the distance to the aim point goes outside a range:
   - farther than the **maximum** → pulled in to 1 unit inside the maximum
     (so it trails behind Connor like on a leash);
   - closer than the **minimum** → pushed out to 1 unit beyond the minimum.
2. The range is set by scripts with `KQCamera::setRange <min> <max>`
   (min must be smaller than max, otherwise the command is refused):
   - walking: 1,000 – 2,500 (`GAME/Kq/walk.cs`)
   - running: 1,500 – 5,000 (`GAME/Kq/run.cs`)
   - before any script sets it: 1,500 – 5,000.
3. Distances are measured in 3D, from camera to aim point.

### Distance (fixed mode, F11)

1. The camera keeps the distance it had when the mode was switched on, at
   whatever angle Connor's movement leaves it.
2. That distance can be changed by zooming (Shift + right-drag, or Page
   Up/Down) and is kept between minimum + 1 and maximum − 1. Page Up/Down
   change it by 0.05 × 2,500 = 125 units per update while held.

### Height above the ground

1. The camera wants to be at least **1,400 units above the terrain under
   the camera**, and at least 1,400 units above the followed object's base.
   The higher of the two wins.
2. If the camera is within 100 units of that height it jumps straight to it;
   otherwise it moves **10% of the remaining gap per update** (smooth rise
   and fall).
3. The terrain height under the camera comes from the world's terrain (M1).
   It is the **floor grid** (the ground under water, `water.md`), not the
   drawn surface, so over Daventry's river the camera may come down to
   1,400 above the river bed (corrected 2026-10-13).

### Swinging and zooming with the mouse

1. While the right mouse button is held, sideways mouse movement swings the
   camera around the aim point (around the vertical axis). The swing angle is
   proportional to the mouse movement, with a factor of 3π/4 per unit of the
   game's mouse value; how mouse pixels map to that unit is open question 1.
2. Forward/back mouse movement in normal mode moves the camera along the
   line to the aim point (zoom) by 5,000 units per unit of mouse value,
   whatever the current distance. The result is still kept inside the
   min/max range. If a zoom would carry the camera past the aim point to
   the other side, it is instead placed at the minimum distance on its
   original side.
3. In fixed mode (or normal mode with Shift held) forward/back mouse
   movement tilts instead, by 0.1 × 7.5 × mouse movement.
4. The first update after pressing the button uses no movement (only the
   change since the previous update counts, and there is none yet).
5. Releasing the button clears the stored mouse movement.

### Tilt (pitch)

1. Default tilt is **0.2 rad** (about 11°) down. The original stores it
   as −0.2: in the camera's angles negative tilt looks **down** (traced
   2026-10-17 from `move KQCamera`, see "Shake and cutscene cameras";
   answers open question 2).
2. Page Up/Down tilt at 0.1 × 3 × 0.05 = **0.015 rad per update** while held
   (0.225 rad/s at 15 updates per second).
3. Tilt is limited to ±1.57 rad (straight up/down) in third person.
4. Home returns tilt to the default.
5. Tilt also returns to the default by itself when Connor moves: in normal
   (not fixed) mode, once he has moved more than 10 units horizontally
   (measured as |Δx| + |Δy|) while the tilt is not at default, it snaps back.
   This matches the manual ("it will automatically reset itself as soon as
   Connor moves").

### Swinging back behind Connor

1. When Connor turns so that the camera is no longer behind him, and the
   player is not using the mouse on the camera, the camera swings round
   towards his back by itself. It starts when the angle between "behind
   Connor" and the camera's direction exceeds about 0.25 rad, and moves a
   fraction of the remaining angle each update.
2. The exact states and rates (constants seen: 0.25, 0.5, 1.25, 1/3, 0.21,
   10/3) were not fully decoded: open question 7.

### Obstacles (studied 2026-10-13)

Words used here: the **aim point** is the point 1,400 above Connor's feet
that the camera looks at; the **offset** is the arrow from the aim point
to the camera; turning **around** Connor means turning about the vertical
line through the aim point (the camera's height is left alone).

**What blocks the view.** The test is a straight line from the aim point
to the camera. The nearest of these blocks it:

1. **The drawn ground** (the visible terrain, not the floor grid under
   water).
2. **Objects whose collision type has bit 0x8 or bit 0x1000.** In the
   data that means:
   - **buildings**: every building (`.dig` interior) gets type 0x28
     unless its `.kq` says otherwise, so all of them block. The `.kq`
     files that do set a type for a building use 0x88 or 0xA8 (Daventry:
     `CAVE`, `CHURCH`, `WIZHOUSE`, `zedrock1`, `zedrock2`), which block
     too;
   - **doors** and door-like objects with bit 0x1000 (Daventry: the 11
     house doors at 0x5021, the keep doors at 0x1021, `tombdoor` at
     0x5040; in other worlds also big push blocks, shelves, `deadhero`).
     A door keeps blocking after a script clears its click bit;
   - a few others with bit 0x8: Barren's `BARRTOMB` (0x29), Snowexit's
     `stnhedge` and DeadCity's `DOORWAY2`/`DOORWAY3` (0x28).

   Plain objects (default type 0x41: trees, signposts, fences, barrels),
   monsters and Connor himself do **not** block: the camera passes
   through them. Objects are tested by their box from `BOUNDS.KQ`
   (turned and moved with the object, so an open door blocks where its
   leaf now stands), buildings by their BSP. These are the same shapes
   Connor collides with.

**Each camera update**, in this order (steps 1–5 are the rules above):

1. Offset from the aim point to where the camera is now.
2. Mouse swing and zoom.
3. Leash distance (max), or the fixed distance.
4. Height above the ground.
5. Push out to the minimum distance.
6. **An avoidance swing already running** (see step 8) turns the offset
   around Connor by its step. If the player is moving the camera with the
   mouse this update, the swing is cancelled instead.
7. **The view test**, from the aim point to the camera's new spot. If
   it's clear, the update is done.
8. **If it's blocked:**
   1. If the player moved the camera with the mouse this update, the
      offset goes back to the previous update's offset, shortened by 5%.
      The 5% is skipped if it would bring the camera under the minimum
      distance. So dragging the camera into a wall does not work; the
      camera only creeps in a little for every update the player keeps
      trying.
   2. Then, only when **all three** of these hold: avoidance is switched
      on (it is by default; nothing found switches it off), Connor is
      moving (see below), and Connor's feet are not under the ground at
      his spot (floor grid):
      - Take Connor's aim point from **one update ago** (A) and the point
        where the view line was blocked (H). Take the point halfway
        between them (M). The new offset points from the aim point
        towards M, with the same length as before.
      - Start a **swing**. Its step per update is 0.66 × the angle around
        Connor between the old (blocked) direction and the new one. The
        swing goes on, update after update, turning the same way, until
        the camera has turned as far as the direction from the aim point
        to A. That is, it ends **looking along the path Connor just came
        by**, which is known to be clear. It stops on the update where
        that direction is reached or passed, so it may overshoot by up to
        one step.
      - Test the view line again, along the new direction. If it's still
        blocked, the camera is **pulled in** along that direction to 100
        units short of the new hit point, and the swing is cancelled.
   3. If any of the three conditions fails, the camera stays where step
      8.1 left it, view blocked or not.
9. The result is the camera's new spot. The view test is not repeated,
   and the height rule is not applied again.

**When the camera updates at all.** The follow camera recalculates only
when Connor moves (in every update in which he moves), and while the
player uses the mouse or tilt keys on it. If Connor stands still with the
view blocked, nothing changes until he moves or the player moves the
camera.

**"Connor is moving"** (step 8.2): the camera keeps Connor's position
from the update before. It counts him as moving from the update after he
starts moving until 3 updates have passed without movement. After 3 still
updates, the "one update ago" position is reset to where he stands. So
the first update of a new walk after standing still never starts a swing
(not checked in the game).

#### Obstacle numbers

| What | Value |
| --- | --- |
| pull-in margin before the hit point | 100 units |
| point used for the new direction | halfway between A and H (0.5) |
| swing step per update | 0.66 × the first turn (radians) |
| shortening when dragging into a wall | ×0.95 per update |
| "still" after | 3 updates without movement |
| what blocks | the drawn ground, collision type bit 0x8 or 0x1000 |

### First person (F2)

1. The view is from Connor's eyes, facing where he faces.
2. Tilt is limited to between −0.54 rad and +1.57 rad (the first-person
   limits differ from third person).
3. Mouse forward/back tilts (factor 0.1 × 10 per unit of mouse value).
4. If the player has tilted and Connor then moves more than 10 units (and
   the mouse is not tilting), the tilt snaps back to the default.
5. The game also tips the view to follow the slope of the ground Connor
   stands on (amount scaled by 2/π of the slope angle, within the tilt
   limits). Details are open question 5.
6. Connor's first-person animations and arm models come from the
   `...FpAnimList` and `ConnerFpSkinList` entries in `GAME/Kq/Connor.kq`.
7. **`setThirdPerson`** (traced 2026-10-17, Q89 b) is a toggle with a
   memory, kept in the script variable
   `<World>::SetThirdPerson::WasFirstPerson`. Cutscene scripts call it in
   pairs (before and after):
   - In first person: switches to third person, forbids the first-person
     key, sets the variable to 1.
   - In third person with the variable at 1 (the second call): sets it
     to 0, allows the first-person key again and **goes back to first
     person**.
   - In third person with the variable at 0: only **flips** whether the
     first-person key is allowed. Camera place, distance and tilt are
     not touched. (`setThirdPerson 1` in third person does the same and
     sets the variable to 0.)
   - So a call without its pair leaves the first-person key locked, or
     unlocked when it should be locked.

### Shake and cutscene cameras (summary only)

- `KQCamera::shake <seconds> <xMax> <xPeriod> <zMax> <zPeriod>` wobbles the
  camera with two sine waves for the given time (usage text in the game).
- `KQCamera::addCamLoc x y z [newlist]` builds a list of camera spots;
  `cutToCamLoc x y z [roll]` jumps to one and aims at the given point;
  `flyToCamLoc x y z [seconds endRoll]` flies there over time. Scripts use
  these in cutscenes. To be studied before M3/M4.
- **`move KQCamera <flags> x y z rx ry rz`** (traced 2026-10-17, Q89 c):
  - Flags: bit 1 = the position is **added** to the current one
    (otherwise it replaces it); bit 2 = the angles are added (otherwise
    they replace them). 0 = both absolute, 3 = both relative. Missing
    numbers count as 0.
  - Angles in radians: **rx = tilt** (positive looks up, negative looks
    down), **ry = roll**, **rz = heading**: 0 looks along world **+Y**,
    positive turns anticlockwise seen from above (π/2 looks along −X),
    the right-hand rule round Z as for objects. The view direction is
    (−sin rz × cos rx, cos rz × cos rx, sin rx). (The angle order is
    traced; that the camera looks along its own +Y is inferred, but the
    data fit it: the watermill shot's heading 2.632 points at Connor's
    `walkTo` target, 2.64; the bricks shot's 0.9 has Connor a little
    left of centre at 1.10; `miltrap1.cs`'s −0.86 is a high camera
    looking down.)
  - The camera takes that place and view **at once**, and the next frame
    is drawn straight away.
- **`follow Connor` after a cutscene** starts from where the camera is,
  with no jump to a preset spot (traced):
  - If the camera is farther from the aim point than the range's
    maximum, it is pulled in to max − 1 at once; if nearer than the
    minimum, pushed out to min + 1 at once; otherwise it stays.
  - Heading: if it faces more than **0.4 rad** away from Connor, it turns
    toward him by **1/3 of the remaining angle per update** and snaps the
    last part (the snap point is lopsided: below 0.63 rad on one side,
    about 0.063 rad on the other; looks like an original bug, see "Do not
    reproduce").
  - Tilt (inferred): set at once to the default tilt plus the angle that
    looks at the aim point.
  - Then the normal follow rules apply (the camera only updates while
    Connor moves, or while the turning flag that `follow Connor` sets is
    on; open question 7).
  - The scripts use this: just before `follow Connor` they `move
    KQCamera` to a spot close to the follow view (bricks cue 10: about
    0.19 rad off), so there is no visible jump.

## Data used

- `MASK/Connor.sae`, `MASK/Camera.sae`: bindings.
- `GAME/Kq/walk.cs`, `run.cs`: `KQCamera::setRange` per movement mode.
- World scripts (`daventry/kq/*.cs` and others): `KQCamera::follow`,
  cutscene commands.
- Terrain height (from M1 terrain work): the floor grid for the height
  rule, the drawn surface for the view test.
- Collision types: `colliderType` in each object's `.kq`. Buildings
  default to 0x28, other objects to 0x41 (`connor-physics.md`). Boxes come
  from `GAME/patch/BOUNDS.KQ`, building BSPs from the `.dig` files.

## Edge cases

- `setRange` with min ≥ max: refused, range unchanged (the game prints an
  error message).
- Switching walk → run widens the range; the camera does not jump, it just
  has more room before it gets pulled or pushed.
- Following `none`: camera stays where it is.
- View blocked very close to the aim point: the pull-in length is the
  distance to the hit minus 100. If the hit is less than 100 away, that
  is negative, and the original puts the camera on the far side of
  Connor (see "Do not reproduce").
- The pulled-in camera can be closer than the minimum distance. On the
  next update the minimum rule pushes it out again and the view test
  pulls it back in, for as long as the obstacle is there.
- If the first turn is zero (old and new direction the same), the swing
  has a step of 0 and stays "running" without moving until the mouse or
  a new block resets it. No visible effect.

## Do not reproduce

- **The lopsided snap** when `follow Connor` turns the camera back
  toward him (below 0.63 rad on one side, about 0.063 rad on the other):
  use one snap angle on both sides (0.063 rad, the smaller), and turn by
  1/3 of the remaining angle per 1/15 s, done per second.

- **Per-update rates.** The height smoothing (10% per update), tilt speed
  (0.015 rad per update) and fixed-mode zoom (125 units per update) depend on
  frame rate in the original. Implement them per second so they match the
  original at 15 updates per second:
  - height: close `1 − 0.9^(15 × elapsed seconds)` of the remaining gap;
  - tilt: 0.225 rad/s; fixed-mode zoom: 1,875 units/s.

- **Negative pull-in.** When the hit is under 100 units from the aim
  point, put the camera at a small minimum distance in front of the aim
  point (e.g. 1 unit), never behind it.
- **Per-update obstacle rates.** The swing step (a fixed angle per
  update, 0.66 × the first turn) and the ×0.95 drag shortening are per
  update. Per second at 15 updates per second: the swing turns 9.9 × the
  first turn per second; the shortening is 0.95^(15 × seconds).

## How to check

- Log camera distance to Connor while walking in a straight line: it
  should settle at just under 2,500 (walk) or 5,000 (run).
- Log camera height above terrain on a hill: never below 1,400 for long;
  rises smoothly.
- Hold Page Up for 1 s: tilt changes by about 0.225 rad.
- Right-drag sideways: camera circles Connor at constant distance.
- Obstacles, in the original (by eye) and in our logs (numbers):
  - Walk Connor past a tree or signpost with the camera trailing behind
    it: the camera goes **through** it (trees don't block).
  - Walk round a corner of Connor's house, close to the wall: the camera
    should never end up inside the house. It swings round to look along
    the way he came, and comes closer when the swing isn't enough. Log:
    blocked yes/no, the hit point, the swing step and what's left of it,
    the pulled-in distance (100 short of the hit).
  - Stand still with the house between the camera and Connor: nothing
    happens until he moves.
  - Hold the right mouse button and drag the camera into a wall: it
    doesn't go in. It creeps closer by 5% per update while you keep
    dragging, never closer than the minimum (1,000 walking).
  - A closed door between the camera and Connor blocks the view; an open
    one only where its leaf now stands.
  - **Checked in the original by the user (2026-10-13):** (1) the camera
    flies through trees; (2) walking round a house corner, it never goes
    through the wall, swings round, and comes closer when it has to; (3)
    dragging the camera into a wall is blocked: it comes closer to Connor
    instead of going inside, and "feels very stuck and resistant to
    movement". That fits the rule: every update the drag is refused, only
    the 5% creep gets through. Keep that feel (it's the original's); a
    smoother drag is an M7 improvement, not a fix. Not checked: standing
    still with a blocked view, doors, the exact numbers.
- In the original game, compare screenshots of the same spot with the same
  inputs (you can do this by eye; we only compare numbers we log).

## Open questions

1. How mouse movement (pixels) becomes the game's mouse value. Needed for
   swing/zoom speed.
2. (Answered 2026-10-17: yes, negative tilt looks down; see Tilt rule 1.)
3. (Answered: tilt resets after Connor moves more than 10 units, in both
   views; see Tilt rule 5.)
4. (Answered 2026-10-13, see "Obstacles".) Still open: what switches
   avoidance off (on by default; no script command found that changes
   it); whether Connor's "moved" signal comes in every update he moves or
   only when his position or facing really changes (assumed: whenever
   either changes); the turn direction of the swing is inferred from the
   signs of the two angles, not checked in the game.
5. First-person slope tilt details and the eye height.
6. Whether the camera updates at the same rate as Connor (assumed yes).
7. The automatic swing behind Connor: when exactly it starts and stops, and
   how fast it turns.
