# Third-person and first-person camera

Study session A, 2026-10-11; "Obstacles" added 2026-10-13; height rule
and "when the camera updates" refined 2026-10-18 (Q97, Q99). Sources: game data files, the game's own
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
   The higher of the two wins (the **wanted height**).
2. **Refined 2026-10-18 (traced, Q97, Q99 c).** In one camera update,
   after the leash (step 3 of "Each camera update"), the height is worked
   out in three steps:
   1. **Lift off the ground, at once.** Let *gap* = (terrain under the
      camera's new spot + 1,400) − the camera's new height. If *gap* is
      more than 0 and not more than the range's maximum, the camera is
      put **straight at** terrain + 1,400 (no easing). If *gap* is more
      than the maximum (the camera would be very deep under the
      ground), it is put where the straight line from its old spot to
      its new spot meets the ground, and the wanted height is taken
      again at that spot. Either way the game remembers "lifted"
      (used in step 3).
   2. **Close enough: snap.** If the camera is now within **100** of the
      wanted height, it goes straight to it.
   3. **Otherwise it only moves if Connor moved.** "Moved" here means
      that the aim point has changed since the camera's previous
      record of it by more than **20**, measured as the **signed sum**
      Δx + Δy + Δz (see "Do not reproduce"). Normally the previous
      record is from the update before; after 3 updates without a
      "Connor moved" message (see "When the camera updates at all") it
      is reset to where he stands, so the sum is 0.
      - **Moved:** the new height = the camera's height **at the start
        of this update** + 10% × (wanted height − the height after
        steps 1–2). Both up and down (smooth rise and fall). "Lifted"
        is forgotten. Note the starting point: any height change the
        leash or the lift made in this update is thrown away and
        replaced by this step (see "Edge cases").
      - **Not moved:** the height stays as steps 1–2 left it. Only if
        the player is swinging the camera with the mouse this update
        and the camera was "lifted" in an earlier update does it snap
        to the wanted height.
   So the camera never eases toward the wanted height while Connor
   stands still; it only snaps (within 100) or is lifted off the ground.
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

**When the camera updates at all** (rewritten 2026-10-18, traced, Q99).
The follow camera runs its update (all of steps 1–9 and the aiming) only
on these occasions:

1. **A "Connor moved" message.** Connor (any followed object) sends it
   when one of his moves is **committed** and his position changed by
   **more than 0.1** on any axis, or his facing changed (any entry of his
   turn matrix by more than 0.0001). A move that changes nothing sends
   nothing. This covers:
   - his ordinary moves, one per update while he walks, runs, falls,
     turns or is carried by an animation's steps;
   - a script's **`move Connor`** (a jump to the new spot, committed at
     once);
   - **not** an update in which he stands still (an animation with no
     steps, gravity holding him on the floor): no message, no camera
     update.
2. **`follow <object>`** itself: it sends one such message at once (so
   there is one camera update right after the command, even if Connor
   doesn't move; the aim point counts as not moved for the height rule).
3. **Every update while the "turn back" switch is on.** `follow
   <object>` switches it on when the camera faces more than 0.4 rad away
   from the object (see "`follow Connor` after a cutscene"). It is
   switched off only when a flight ends, or by a later `follow` that
   finds the camera farther than the range's maximum. Nothing else
   switches it off, so after such a `follow` the camera updates every
   update, standing still or not, and its heading stays locked exactly
   onto Connor (heading state "exact", "Camera flights and cuts" rule
   5). Daventry's scripts avoid it: the camera spots they set just
   before `follow Connor` are 0.04–0.22 rad off (`stngrl.cs`,
   `wtrmwalk.cs` cue 6, `miltrap1.cs` cue 300), so in normal Daventry
   play the switch is off.
4. **Every update of a flight** (until it ends), and the one update of a
   cut ("Camera flights and cuts").
5. **The player's camera controls** (mouse swing/zoom, tilt keys): not
   re-studied here; take the rules above ("Swinging and zooming", "Tilt")
   as updates while the control is used.

**When the message arrives.** Messages go through the game's event
queue: they are handled **after** whatever is running at the moment has
finished, but still within the same update, before the frame is drawn.
An ordinary move is committed at the start of the next update (so the
camera follows one update behind the move it reacts to; it was always so
in normal play). A script's `move Connor` is committed at once, and its
message is handled right after the script (and the event that ran it)
ends.

So when a trap's `On` does `move Connor …` and then `move KQCamera …`
(Daventry's mill wheel, `wheeltp1.cs` lines 8–9), the order is:

1. Connor jumps to the script's spot; the "moved" message is queued.
2. The camera is put at the script's spot and angles.
3. The script ends; the queued message arrives; the camera runs one
   update **from the script's spot**: leash/minimum, height rule, view
   test, aiming (see "`move KQCamera` while following" in "Shake and
   cutscene cameras").
4. Next update: Connor's own move of the update the trap fired in is
   committed (he lands from the script's spot); if he moved more than
   0.1 there is one more camera update.
5. Then nothing until Connor moves again.

If Connor stands still with the view blocked, nothing changes until he
moves or the player moves the camera.

**"Connor is moving"** (step 8.2 and the swing behind him): the camera
keeps the aim point from its previous "Connor moved" message. It counts
him as moving from the first message until 3 updates have passed without
one. After 3 still updates, the previous aim point is reset to where he
stands. So the first update of a new walk after standing still never
starts a swing (not checked in the game).

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

### Shake and cutscene cameras

- `KQCamera::shake <seconds> <xMax> <xPeriod> <zMax> <zPeriod>` wobbles the
  camera with two sine waves for the given time (usage text in the game).
- Camera flights and cuts (`addCamLoc`, `flyToCamLoc`, `cutToCamLoc`):
  see "Camera flights and cuts" below.
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
  - **`move KQCamera` while following** (traced 2026-10-18, Q99 d): the
    move itself does not run a camera update. The script's place and
    view stand until the next camera update ("When the camera updates
    at all"); with Connor standing still and the "turn back" switch off
    that may be never, until he moves. The next update starts from the
    moved spot with the normal rules:
    - position: leash/minimum, height rule (with its "moved" test),
      view test, as in any update;
    - **tilt: set at once** to the player's tilt (default −0.2) + the
      angle up to the aim point, so the script's tilt is lost in that
      update;
    - **heading: not snapped.** It goes through the swing-behind rules
      (open question 7): with the "turn back" switch off, it only starts
      turning when it is more than 0.25 rad off Connor (and he counts as
      moving) or more than 0.5 rad off; within 0.5 rad it is turned only
      by 1.25 × the growth of the error since the previous update, and
      beyond 0.5 rad it is turned back to 0.25 rad off at once. (With the
      switch on: a third of the rest per update, then locked.)
  - **Daventry's mill wheel (`wheeltp1.cs`), worked out from these
    rules (not tested):** the first shot (camera put at (185500, 226520,
    9750), heading −0.3, tilt 0) gets the update of step 3 in the order
    above, then possibly the one of step 4, then none while Connor plays
    `wheelover` (no steps). In that update: the distance to the aim point
    is about 1,940, inside the range, so no leash; the wanted height is
    about 8,520 (Connor's base + 1,400; the ground under the camera,
    6,924 in our data, + 1,400 is lower); the camera is 1,230 above it,
    so no snap; if Connor's jump to the script's spot (from where he
    entered the trap) is "moved" (signed sum over 20), the height eases
    once: 9,750 + 10% × (8,520 − 9,750) ≈ 9,627; the step-4 update is
    normally "not moved" (he only lands), so the height stays. The tilt
    becomes about −0.2 + (−0.63) ≈ −0.83 (looking down at him). The
    heading is about 0.26 rad off him, just over 0.25, so at most a
    small turn. The second shot (cue 4, 0.8 s later, camera put at
    (185000, 237850, 10850)) comes while he stands still: **no update at
    all**, so it stays exactly as the script put it (no leash, no height
    rule, script's angles) until the flight at cue 1.
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
  - Then the normal follow rules apply: one camera update right after
    the command, then updates only on the occasions in "When the camera
    updates at all" (the "turn back" switch is the one set here when
    the heading is more than 0.4 rad off; it is cleared by the leash
    case above, i.e. when the camera was farther than the maximum).
  - The scripts use this: just before `follow Connor` they `move
    KQCamera` to a spot close to the follow view (bricks cue 10: about
    0.19 rad off), so there is no visible jump.

### Camera flights and cuts: `addCamLoc`, `flyToCamLoc`, `cutToCamLoc` (traced 2026-10-18)

Words used here: a **spot** is a camera place in the camera's list; the
**aim point** is the x y z given to `flyToCamLoc` / `cutToCamLoc`
(the game's usage text: "xyz is of point to AIM at, uses camlocs from
addCamLoc command for camera positions"). Two switches of the camera
matter, both set by `follow` (rule 1): **"places"** (the follow rules
set the camera's position) and **"aims"** (the follow rules turn the
view toward what is followed). `follow <object>` switches both on
(`follow <object> 1`: only "aims"); `follow none` switches both off.

1. **The list.** `KQCamera::addCamLoc x y z [newlist]`: if `newlist` is
   given and not 0, the list is emptied first; then the spot is added at
   the end. Without it (or 0) the spot is added to the spots already
   there. The list stays until the next `newlist` (fly and cut don't
   empty it). Data: 69 of 70 lines give `1` (5 of them as
   `$KQCamera::newLoc 1`), so nearly every cutscene uses a fresh list
   with one spot; only Dead City's `laser2.cs` adds a spot without it.
2. **Which spot.** Fly and cut both use the spot **nearest the camera's
   present position** (3D distance; on a tie the one added first).
3. **`KQCamera::cutToCamLoc x y z [roll]`** (roll in radians, 0 if
   missing; a 5th word is needed for it):
   - the camera is put on the spot at once;
   - it **stops following** (as `follow none` does for the target: Connor's
     moves no longer update it) and obstacle avoidance is switched off;
     the two switches stay as they were;
   - the aim point becomes what the camera "follows" (for the rules
     below, as if it were an object standing at the aim point: the
     camera looks 1,400 above it, "What the camera looks at" rule 2);
   - the player's tilt and heading offsets go back to the defaults
     (tilt −0.2, heading offset 0); the roll becomes `roll`;
   - the heading is set at once to look from the spot toward (x, y);
   - then one camera update runs at once with the normal follow rules:
     if "places" is on, the camera is pulled onto the range around the
     aim point (farther than the maximum → max − 1, nearer than the
     minimum → min + 1, then the height rule, settled as in rule 4's last
     update: the aim point counts as not moved); if "aims" is on, the
     view is aimed as in rule 5. With both off (after `follow none`) it stays
     on the spot, heading toward (x, y), and keeps its tilt.
4. **`KQCamera::flyToCamLoc x y z [seconds [endRoll]]`:**
   - **Duration:** `seconds` (a negative value counts as 0). If it is 0
     or missing: the distance from the camera to the spot ÷ **5,000**
     units per second. The flight takes **N = whole part of (duration ×
     15) + 1 updates** (2 s → 31 updates, 2.067 s; 3 s → 46).
   - **Path and speed:** a **straight line** from where the camera is to
     the spot, the same step every update ((spot − start) ÷ N): constant
     speed, no easing in or out. It is exactly on the spot after update N.
   - **Roll:** from its present value to `endRoll` in N equal steps.
     `endRoll` is only read when the line has exactly x y z seconds
     endRoll; otherwise it is **0**, so a flight without it levels a
     rolled camera.
   - At the start: it **stops following** and avoidance is off (as for a
     cut); the aim point becomes what it "follows"; the tilt and heading
     offsets go back to the defaults; **"places" is switched off** for
     the flight ("aims" stays as it was).
   - **Every update of the flight:** the step is added to the position,
     the roll step to the roll, then the normal follow update runs: it
     does not place the camera ("places" off) but, if "aims" is on, aims
     it (rule 5).
   - **The last update (N):** "places" is switched **on** again before
     that update's follow update runs, so the camera is pulled onto the
     range around the aim point like after a cut: farther than the
     range's maximum → pulled in to max − 1 **along the line to the
     aim point + 1,400**, nearer than the minimum → pushed to min + 1,
     then the height rule ("Height above the ground" rule 2). **Settled
     2026-10-18 (traced, Q97):** a flight or cut sets both the camera's
     present and previous record of the aim point to the aim point, so
     the aim point **counts as not moved** and the height never eases
     on this update. Only: (a) if the camera is now below the ground
     under it + 1,400 it is lifted there at once; then (b) if it is
     within 100 of the wanted height (the higher of ground + 1,400 and
     aim point + 1,400) it snaps there; otherwise its height stays.
     The same holds for a cut's one update. Avoidance is off,
     so no obstacle pull-in or swing. **So the camera can jump at the
     end of a flight** (watermill: about 1,227 units, rule 7).
   - **After the flight** it stays where that last update put it: it
     follows nothing, so it isn't updated again until a `move
     KQCamera`, `follow`, cut, flight or shake (the mouse look is off
     during `handsOff`).
   - Connor is told when the flight ends (an internal message used by
     some of his special states; scripts don't see it). The end is not
     a cue: scripts time what follows with `sendEvent`.
5. **What the camera looks at during and after a flight or cut**, when
   "aims" is on: the point **1,400 above the aim point**, from the
   camera's position, every update:
   - **tilt** = default tilt (**−0.2**) + the angle up to that point, set
     exactly each update (so the camera looks slightly below the raised
     point; in effect at about the aim point's own height when the
     camera is a few thousand units away and a little below it);
   - **heading:** within **0.02 rad** it locks on and from then on
     follows the point exactly each update; otherwise, as in "`follow
     Connor` after a cutscene": more than 0.63 rad off on the
     anticlockwise side (0.063 rad on the clockwise side) → turns by
     **1/3 of the remaining angle per update**, less → snaps onto it
     (the same lopsided rule; see "Do not reproduce");
   - **roll** = the flight's roll (rule 4), not changed by aiming.
   With "aims" off (a `follow none` and no `follow` since) the view
   keeps its heading and tilt through the whole flight; only the
   position and roll change.
6. **Together with other commands** (traced; none of these happen in
   Daventry's data):
   - `move KQCamera` during a flight sets place and angles at once; the
     flight's remaining steps are then added from the new place, so it
     ends away from the spot; with "aims" on, the view is turned back to
     the aim point at the next update.
   - `follow Connor` during a flight: both switches on and Connor is
     followed again; from the next update the follow rules place the
     camera behind him (the flight's steps are still added each update
     until N, then overruled by the leash).
   - `follow none` during a flight: the view stops turning; the position
     keeps flying, and at update N it is pulled onto the range around
     the aim point (rule 4).
   - `follow Connor` after a flight or cut works as in "`follow Connor`
     after a cutscene".
7. **Daventry's numbers (data + rules above), to check in `camera.log`:**
   - **Watermill room, `wtrmwalk.cs`** (range 1000–2500 from its own
     `setRange` at `On`):
     - cue 3: `follow Connor` (both switches on, leash clamp at once),
       `addCamLoc 190000 230000 10600 1` (the list is this one spot);
     - cue 4 (0.2 s = 3 updates later): `flyToCamLoc 186960 228000 10000
       2`: **N = 31**; step = ((190000, 230000, 10600) − camera at cue
       4) ÷ 31; roll 0 throughout;
     - looked-at point: **(186960, 228000, 11400)**; at the spot the view
       is heading **2.153**, tilt **+0.016** (−0.2 + 0.216);
     - update 31: distance from the spot to the looked-at point
       **3,725.8** > 2,500, so the camera is pulled in to 2,499 along
       that line: **(188999.0, 229341.5, 10863.4)**. The height rule
       wants 11,400 (aim point + 1,400), 536.6 higher: more than 100,
       and the aim point counts as not moved, so **the height stays at
       10,863.4** (Q97). It would only change if the ground under that
       spot + 1,400 were above 10,863.4 (ground above 9,463.4; then it
       is lifted to ground + 1,400), which B's log can check. Same view
       direction, about 1,227 units closer; it stays there until cue 6;
     - cue 5 comes 30 updates after cue 4 (one update before the flight
       ends), cue 6 at 60 updates: `move KQCamera 0 192880 230965 9260
       -.2 0 2.095`, then `follow Connor`.
   - **Mill wheel, `wheeltp1.cs`** (the windmill's other trap, Connor
     knocked under the wheel; no `follow none` in it, so both switches
     are on from normal play): cue 1 `addCamLoc 185000 213300 8800 1`,
     `flyToCamLoc 185800 211300 5610 3`: **N = 46**; looked-at point
     (185800, 211300, 7010); at the spot heading **−2.761**, tilt
     **−0.893**; distance 2,800.7, so at update 46 the camera is pulled
     in to max − 1 if the range's maximum is under 2,800.7 (walking,
     2,500: to (185086.2, 213084.6, 8607.2)), and stays if running
     (5,000). Height on that update (Q97): the aim point + 1,400 is
       7,010, below the camera, so the height stays unless the ground
       under it + 1,400 is above the camera (then lifted) or within 100
       of it (then snapped). Cue 2 comes after 2.5 s (37–38 updates, before the flight
     ends); cue 3 (4 s after cue 2): `follow Connor`.
   - **Windmill tumble, `miltrap1.cs`:** **no flight or cut**. It uses
     `follow none` and three `move KQCamera` (`anim-scripts.md` rule
     30), then `follow Connor` at cue 300. Between them the camera
     stays exactly at each moved spot (rule 1 of "What the camera looks
     at").

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
- **The height easing starts from the old height** (Height rule 2.3):
  in an update where Connor moved, the leash's vertical change and the
  lift off the ground are replaced by "old height + 10% of the gap". On
  a slope where Connor's base + 1,400 is the wanted height and the
  ground under the camera + 1,400 is above the camera's old height, the
  camera can end that update still below ground + 1,400; the next
  update lifts it again. Traced, not seen in the game.
- `flyToCamLoc` / `cutToCamLoc` with an **empty spot list**: the
  original uses a spot that was never set (see "Do not reproduce").
- `flyToCamLoc` with seconds 0 and the camera already on the spot: N = 1,
  a flight of one update (the last-update rule still applies).
- If the first turn is zero (old and new direction the same), the swing
  has a step of 0 and stays "running" without moving until the mouse or
  a new block resets it. No visible effect.

## Do not reproduce

- **The lopsided snap** when `follow Connor` turns the camera back
  toward him (below 0.63 rad on one side, about 0.063 rad on the other):
  use one snap angle on both sides (0.063 rad, the smaller), and turn by
  1/3 of the remaining angle per 1/15 s, done per second.

- **Fly or cut with no spot in the list:** the original reads a spot it
  never set (whatever was in memory). Do nothing instead (keep the
  camera where it is) and log an error.
- **The jump at the end of a flight** (rule 4, last update) is the
  original's behaviour as traced, and the user saw it in the original
  (2026-10-18): build it, so the cutscenes frame the same.

- **Per-update rates.** The height smoothing (10% per update), tilt speed
  (0.015 rad per update) and fixed-mode zoom (125 units per update) depend on
  frame rate in the original. Implement them per second so they match the
  original at 15 updates per second:
  - height: close `1 − 0.9^(15 × elapsed seconds)` of the remaining gap;
  - tilt: 0.225 rad/s; fixed-mode zoom: 1,875 units/s.

- **The "moved" test of the height rule** adds the signed changes
  Δx + Δy + Δz of the aim point, so a move of, say, +300 in x and −300
  in y counts as "not moved" and the height freezes for that update.
  Use the sum of the sizes (|Δx| + |Δy| + |Δz| > 20) instead; it gives
  the same result for every move that is mostly along one axis.
- **The "turn back" switch that never clears** ("When the camera
  updates at all", item 3): after a `follow` that finds the camera more
  than 0.4 rad off, the original locks the heading onto Connor and
  updates every frame until the next flight ends. Build it as traced
  (Daventry's scripts never trigger it), but log it whenever it
  switches on, so we notice if another world's script does.
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
- **Camera flights (our logs, B):** in `camera.log`, per update of a
  flight: update k of N, position, step, roll, heading and tilt, the
  "places"/"aims" switches. Watermill (`cargo play --world Daventry --at
  192408,227039,0 --frames 3000 --sound off`): N = 31 from cue 4; the
  position changes by the same step every update; at update 31 the
  pull-in to (188999.0, 229341.5, ~10863) (rule 7); heading 2.153 and
  tilt +0.016 once locked on; then no change until cue 6.
- **Camera flights (original, user, by eye):** in the watermill room's
  first-visit cutscene, the camera should glide in a straight line at an
  even speed for about 2 s, turning to look into the room, and then
  **jump about a third of the way closer** along its view at the end.
  **Confirmed by the user in the original (2026-10-18):** the camera
  jumps toward the end of the flight. (The size of the jump was not
  measured.)
- **When the camera updates (our logs, B):** log each camera update
  with its cause (Connor moved / `follow` / turn-back switch / flight /
  player control) and the height step taken (lifted / snapped / eased /
  kept, with the signed sum). Standing still on a slope after walking:
  updates stop 1 update after he stops (the last committed move), and
  the height stops changing at once even if it is far from the wanted
  height. Mill wheel (`wheeltp1.cs`): at most 2 updates after the
  trap's `On`, none from then to cue 1; the second shot logged at
  exactly (185000, 237850, 10850), tilt −0.25, heading −2.8.
- **Mill wheel (original, user, by eye):** the second shot (0.8 s after
  Connor is knocked over) should hold perfectly still, high up (the
  script's spot), until the flight; the first shot looks down at Connor
  from about the script's height.
- In the original game, compare screenshots of the same spot with the same
  inputs (you can do this by eye; we only compare numbers we log).

## Open questions

1. How mouse movement (pixels) becomes the game's mouse value. Needed for
   swing/zoom speed.
2. (Answered 2026-10-17: yes, negative tilt looks down; see Tilt rule 1.)
3. (Answered: tilt resets after Connor moves more than 10 units, in both
   views; see Tilt rule 5.)
4. (Answered 2026-10-13, see "Obstacles".) What switches avoidance
   off: answered 2026-10-18: `follow none`, `flyToCamLoc` and
   `cutToCamLoc` switch it off, `follow <object>` switches it on again
   ("Camera flights and cuts"). Connor's "moved" signal: answered
   2026-10-18 (Q99 b): only when his position changes by more than 0.1
   or his facing changes ("When the camera updates at all"). Still
   open: the turn direction of the swing is inferred from the signs of
   the two angles, not checked in the game.
5. First-person slope tilt details and the eye height.
6. Whether the camera updates at the same rate as Connor: answered
   2026-10-18 (Q99): it updates when he sends a "moved" message (one
   per committed move that changes something), handled one update after
   his ordinary moves; see "When the camera updates at all". How the
   player's mouse and tilt controls trigger updates was not re-studied.
7. The automatic swing behind Connor: when exactly it starts and stops, and
   how fast it turns.
