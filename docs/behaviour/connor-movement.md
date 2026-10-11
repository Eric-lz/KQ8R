# Connor's ground movement (walk, run, back, turn)

Study session A, 2026-10-11. Sources: game data files (observed) and
`Mask.exe` (studied, described here in our own words). Jumping, falling,
slopes, steps and walls are in `connor-physics.md`; the camera is in
`camera.md`. Water, ropes and pushing are **not** covered yet.

## Purpose

How the player's key presses become Connor walking, running, stepping back
and turning on flat ground, and which animation plays for each.

## Units and axes

- Distances are in game units. Connor is 1,600 units tall (collision box in
  `GAME/Kq/Connor.kq`) and walks about 1,127 units per second, so 1 unit is
  very likely 1 mm. Treat this as the working assumption.
- Z is up. Turning is rotation about Z. Positive turn = counter-clockwise
  seen from above = turning left.
- Angles are radians.

## Controls (data: `MASK/Connor.sae`)

`Connor.sae` is a text "action map": each key press (`make`) or release
(`break`) sends an action name with a number. The default bindings:

| Key | Press | Release |
| --- | --- | --- |
| Up | `MoveY 1` | `MoveY 0.5` |
| Down | `MoveY -1` | `MoveY -0.5` |
| Left | `RotateZ 1` | `RotateZ 0.5` |
| Right | `RotateZ -1` | `RotateZ -0.5` |
| End | `Run` (toggle) | — |
| Insert | `Jump` | — |
| Delete | `Draw` (weapon) | — |
| Right mouse | `MoveCamera 1` | `MoveCamera 0` |

The bindings must be read from the file, not hard-coded: the game's key
setup screen (`GAME/Kq/movekeys.kq`) lets the player change them.

## Rules

### Movement intent

1. Connor keeps three pieces of intent: **forward/back** (−1, 0 or +1),
   **turn** (−1, 0 or +1) and a **run** on/off switch.
2. `MoveY` sets forward/back and `RotateZ` sets turn. The number is cut to
   its whole-number part, toward zero: 1 → 1, −1 → −1, 0.5 → 0, −0.5 → 0.
   That is why releasing a key (sending ±0.5) stops the movement.
3. `Run` flips the run switch each time it arrives (End key = toggle, not
   hold).
4. Scripts can also set the run switch directly with `KQConner::setRun 0|1`
   (used by `GAME/Kq/walk.cs` and `run.cs`). A freshly created Connor starts
   with the run switch **on**; the world scripts normally set it.
5. The game can disable turning for Connor (where and why is not studied
   yet). While disabled, turning input is ignored; disabling it also clears
   the current turn direction.
6. Other axes exist in the action list (`MoveX`, `MoveZ`, `RotateX`,
   `RotateY`) and are stored the same way, but Connor's ground movement does
   not use them. There is no sidestep.

### Which movement plays

Chosen from the intent every update:

| Forward/back | Turn | Run switch | Movement |
| --- | --- | --- | --- |
| +1 | any | on | `run` (in water: `waterrun`) |
| +1 | any | off | `walk` (in water: `waterwalk`) |
| −1 | any | any | `back` (no running backwards) |
| 0 | +1 | any | `left` (pivot on the spot) |
| 0 | −1 | any | `right` (pivot on the spot) |
| 0 | 0 | any | none: stand (`stop`) |

The movement names are the `MovementN=` names in `GAME/Kq/Conner.anm`.

### Forward speed comes from the animation

1. Each movement in `Conner.anm` names an animation inside a shape file
   (`loop=c_walk.kqs c_walk <start> <playback speed> ...`).
2. In the shape file, every frame of the animation carries a small
   whole-body offset. For walking and running this offset is a step along
   the model's −Y axis (the direction the model faces); the first frame's
   offset is zero.
3. Connor's travel per animation cycle is the sum of those per-frame steps;
   his speed is that distance divided by the animation's loop duration
   (also stored in the shape file).
4. Measured from the GOG 1.03 files (observed):

   | Movement | Shape | Frames | Loop length | Distance per loop | Speed |
   | --- | --- | --- | --- | --- | --- |
   | walk | `GAME/Common/C_walk.kqs` | 20 | 1.2667 s | 1,427.1 | ≈ 1,127 units/s |
   | run | `GAME/Common/C_run.kqs` | 15 | 0.9333 s | 2,350.2 | ≈ 2,518 units/s |

5. `back` plays the walk animation with playback speed −1.0 (see
   `Conner.anm`), so stepping back moves backwards at walking speed.
6. The per-frame steps are uneven (a walk has a slight surge each footfall).
   Spreading the cycle's distance evenly is close, but see open question 1.

### Turning

1. While the turn intent is non-zero, Connor's heading changes by
   `turn × 1.75 rad/s × elapsed time` (about 100° per second), whether he is
   standing, walking or running.
2. **Turn to face a target**: the game can ask Connor to turn until he faces
   a given direction (what asks for this is not studied yet; probably
   scripted moves such as `KQConner::walkTo`). He then turns (apparently
   the shorter way round) at **double rate, 3.5 rad/s**, and stops exactly
   on the target heading once the remaining angle is smaller than one update's turn.
   Any player turn input (non-zero) cancels this automatic turn.
3. **Banking while running**: when running forward *and* turning, Connor
   leans into the turn (rotation about his forward axis), building up to at
   most **0.15 rad (≈ 8.6°)**. When he stops turning or stops running, the
   lean returns to upright at the same rate; once it is within 0.029 rad of
   upright it snaps to exactly upright. Walking never banks. Some of
   Connor's own states also switch banking off; which ones is not identified
   yet.

### Starting and stopping (data: `Conner.anm`)

Transitions between movements are data, listed per movement as
`transitionN=<from phase> <to movement> <to phase> <transition movement>`:

- Stand → walk plays `stopwalk`, then enters `walk` 85% into its cycle.
- Stand → run plays `stoprun`, then enters `run` 86.7% into its cycle.
- Walk → stand picks the stop animation by how far through the step
  cycle Connor is: from 45% `walkstopleft`, from 90% `stopwalkstop`,
  from 95% `walkstopright`.
- Run → stand: from 6.7% `runstopright`, from 60% `runstopleft`, from
  86.7% `stoprunstop`.
- Footstep sounds fire at the `cueN=<phase> footstep` points (walk: 10% and
  60% of the cycle; run: 0% and 53.3%).

How the game chooses between several transitions with different "from"
phases (nearest, next one reached, or first match) is open question 2.

## Data used

- `MASK/Connor.sae`: key → action bindings.
- `GAME/Kq/Connor.kq`: Connor's object (class `KQConner`), collision box,
  animation list file names.
- `GAME/Kq/Conner.anm`: movement names, shape animation, playback speed,
  cue points, transitions.
- `GAME/Common/C_walk.kqs`, `C_run.kqs` (and the other `c_*.kqs`): per-frame
  steps and loop durations. The shape format is in the community notes
  (`libkq8fpc/doc/kq8kqs.txt`, "sequence" = one frame, `frameTransform`).
- `GAME/Kq/walk.cs`, `run.cs`: scripts switching walk/run.

## Edge cases

- Forward and turn at the same time: walk/run wins; Connor turns while
  moving (no pivot animation).
- Releasing one arrow while still holding the opposite one: the release
  sends ±0.5 → 0, so movement stops even though the other key is still down.
  This is how the original behaves with its own bindings. See "Do not
  reproduce".
- Run switch toggled while standing: no visible effect until he moves.

## Do not reproduce

- **Frame-rate-dependent banking.** The original changes the lean by a
  fixed 0.03 rad per update, not per second, so the lean speed depended on
  the frame rate (the game is capped at 15 updates per second by
  `KQGame::FPSLimit 15` in `console.cs`, which gives 0.45 rad/s). Implement
  it as 0.45 rad/s times elapsed time instead, which matches the original
  at its normal frame rate.
- **Release of one key cancelling the other.** Recommended: track which keys
  are held and derive the intent from them, so releasing Left while Right
  is still held keeps turning right. This is a deliberate improvement; keep
  a setting to get the original behaviour if exact matching is wanted.
- **Turning disabled → silently dropping input.** Fine to keep, but log it.

## How to check

- Log Connor's position every update while walking straight for 10 s:
  expect ≈ 11,270 units travelled (≈ 25,180 when running).
- Log heading while holding Left for 2 s: expect ≈ 3.5 rad change.
- Log the chosen movement name for each input combination in the table.
- In the original game, walk Connor between two landmarks with known
  positions (from the `.kq` files) and time it; compare with ours.

## Open questions

1. Are the per-frame steps applied in one go when each frame is reached, or
   blended between frames? Affects smoothness, not average speed.
2. Which transition is used when several have "from" phases, and what
   happens if the stop is requested between them?
3. Jumping, falling, slopes and walls: answered in `connor-physics.md`
   (walkable limit is 36.9°, not 35°). Water (`waterwalk`), ropes and
   pushing are not studied yet.
