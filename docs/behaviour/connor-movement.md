# Connor's ground movement (walk, run, back, turn)

Study session A, 2026-10-11. Sources: game data files (observed) and
`Mask.exe` (studied, described here in our own words). Jumping, falling,
slopes, steps and walls are in `connor-physics.md`; the camera is in
`camera.md`. Water, ropes and pushing are **not** covered yet.

## Purpose

How the player's key presses become Connor walking, running, stepping back
and turning on flat ground, and which animation plays for each.

## Units and axes

- Distances are in game units. Connor's collision box is 1,300 units tall
  (`GAME/patch/BOUNDS.KQ`, entry `Connor`; corrected 2026-10-12, Q33; the
  1,600 in `[ConnerCollider]` of `GAME/Kq/Connor.kq` is not used for it)
  and walks about 1,127 units per second, so 1 unit is
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
   a given direction (confirmed 2026-10-17: `KQConner::walkTo`,
   `runTo` and `pointAndCue` use it, `anim-scripts.md` rules 24–25). He then turns (apparently
   the shorter way round) at **double rate, 3.5 rad/s**, and stops exactly
   on the target heading once the remaining angle is smaller than one update's turn.
   **What cancels it (corrected 2026-10-18, Q104 a, traced; matches the
   user's check in the original):** only a turn key **press** (the
   moment the key goes down), not a key that is being held.
   - The turn keys reach Connor as events, not as a state read every
     update: one event when a key goes down (turn +1 or −1) and one
     when it comes up. Only the "down" event cancels the automatic
     turn. A key that was already held when the turn started sends
     nothing more, so it can't cancel it; the player has to let go and
     press again. Releasing a key doesn't cancel the turn either (it
     sets the turn direction to 0, and the automatic turn sets it again
     in the next update).
   - One exception: releasing one turn key while the **other** one is
     still held makes the game send the held one again as a fresh
     "down" ("Do not reproduce", "Release of one key cancelling the
     other"), and that does cancel the automatic turn.
   - **While the automatic turn runs, a held turn key does nothing.**
     Every update, the automatic turn overwrites the turn direction with
     its own (+1 or −1, towards the target), and the speed is the
     automatic turn's **3.5 rad/s** whatever key is held; the key does
     not add to it or slow it down.
   - **When the automatic turn ends, the turn direction is set to 0.**
     A turn key that is still held from before does **not** take over
     again: Connor stops turning until that key is let go and pressed
     again (nothing re-sends held keys at that moment). Traced, and
     checked by the user in the original (2026-10-18).
   - Forward/back is a separate piece of intent and is not touched: if
     forward is held he keeps walking or running while the automatic
     turn swings him round (user's check), and because the turn
     direction is non-zero during the turn, a run also **leans** into
     it (rule 3) exactly as with a held turn key.
   - The same rules hold for every automatic turn, whatever started
     it: the turn after a click (`clicking.md` "Connor turns to face
     what he clicked"), `KQMonster::speak` (`talking.md` rule 7), a
     chest remark (`chests.md` rules 8–9), `pointAndCue` and `walkTo` /
     `runTo` (`anim-scripts.md` rules 24–25). But `walkTo` / `runTo`
     switch `handsOff` on, and the scripts that use `pointAndCue` do so
     too, and while `handsOff` (or the engine's own input lock) is on,
     turn key presses don't reach Connor at all
     (`anim-scripts.md` rule 21), so those turns can't be cancelled.
   - A cancelled turn ends **without** its "turn finished" signal: a
     `pointAndCue` cue is then never sent, and a `walkTo` would never
     start walking (it can't happen under `handsOff`, see above).
   - A new turn request while one is running simply replaces the
     target; there is only ever one automatic turn.
   - Pressing a turn key also cuts Connor's current line short unless
     the line is marked `#` (`talking.md` "Cutting a line short"). A
     line that is still waiting out the 0.5 s after a turn
     (`talking.md` rule 7) is not in the queue yet, so the press does
     not stop it: he still says it (user's check).
3. **Banking while running** (details traced 2026-10-13, Q56): when
   running forward *and* turning, Connor leans into the turn (a rotation
   about his forward axis, through his feet). Walking never banks.
   - **The "lean allowed" switch.** Every update in which Connor stands
     (Wait) or moves (Move), the game picks his movement from the intent
     (table in "Which movement plays") and sets the switch **on exactly when it picks `run`
     or `waterrun`** (forward +1 and the run switch on), off otherwise.
     So the start `stoprun` leans (the pick is already `run`); the stops
     (`runstopleft`, `runstopright`, `stoprunstop`) and the run's wait for
     its stop point don't (forward is let go, so the pick isn't `run`
     any more); a pivot (`left`/`right`) and `back` don't. The start of a
     jump makes the same pick (to choose `jump`, `runJump` or
     `backJump`), so a running jump starts with the switch on. In the
     other states (jumping, falling, `Special` movements like
     `reactback`, death …) the code doesn't recalculate it.
     **But the user saw in the original that Connor is upright during a
     jump even with the turn key held (Q59), and what causes this was
     not found** (see "Open questions" 4). The rule: treat jumps,
     falls and landings as "switch off" (he straightens at the normal
     0.03 per update), which is B's stand-in, checked by the user against the original; other states as the code says. Four of Connor's special
     states (not identified by name; the ones that handle the jump key
     their own way; **not** the rope, corrected 2026-10-21: on the rope
     turning itself is off, `ropeandhook.md` rule 22) switch leaning off while they are
     active and back on when they end. It is also forced
     off in the first-person view, and when Connor's `allowLeaning` flag
     is 0 (it is 1 from the start; nothing in the data changes it; code
     that might was not found).
   - **Building.** Each update in which the turn intent is ±1, the switch
     is on and the forward intent is +1, the lean moves **0.03 rad**
     toward the turn's side, but only if, **before** the step, it is
     still short of 0.15 rad on that side. From upright that gives 0.03,
     0.06, 0.09, 0.12, 0.15 (a hair under, in 32-bit floats), **0.18**:
     the lean ends at **≈ 0.18 rad (10.3°)** after 6 updates, not 0.15
     as this rule said before.
   - **Returning.** Each update in which the turn intent is 0 or the switch
     is off: if the lean is within 0.029 rad of upright it becomes exactly
     upright; otherwise it moves 0.03 rad toward upright.
   - **Neither.** Turning, switch on, but forward not +1 (only possible in
     the states that don't recalculate the switch, e.g. letting go of
     forward during a running jump while still turning): the lean stays
     as it is.
   - **Turning the other way** while leaning: the same 0.03 steps go
     straight through upright to the other side; the snap near upright
     only happens when returning. Because of float rounding, after
     swinging over from +0.18 the other side stops at −0.15 (the step
     that lands on −0.15000001 is not "short of" −0.15); from upright
     either side reaches 0.18.
   - **Size of the turn:** the turn intent is always −1, 0 or +1 (a half
     value from a key release becomes 0, "Movement intent" rule 2), so
     there is no half lean.
   - **Drawing only:** the lean is part of his orientation, so his model
     tilts. His collision balls don't turn with it (they always sit
     straight above his position), and his heading (which way he faces,
     used by the movement and the camera) isn't changed by it.

### Starting and stopping (data: `Conner.anm`; rules traced 2026-10-12, Q25)

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
- No other changes Connor makes by himself have transitions: walk ↔ run,
  anything ↔ `back`, `left`, `right` switch directly (see rule 4 below).
  (Water ones exist, e.g. `walk` → `waterwalk` via `walkwaterwalk`; when
  "in water" holds and how the switch happens: `water.md`.)

"Phase" everywhere here is the fraction of the animation's play time,
0 at the start, 1 at the end (the `<start>` value on the `loop=` line is
also a phase).

**The two flags on each `loop=` line** (`loop=<shape> <anim> <start phase>
<speed> <loops> <can be cut short>`):

- 5th field, **loops**: 1 = plays round and round (`stop`, `walk`, `run`,
  `back`, `left`, `right`); 0 = plays once and holds its last frame.
- 6th field, **can be cut short**: 1 = a new movement may replace it at
  once; 0 = it always plays to its end first. `stop`, `walk`, `run`,
  `back`, `left`, `right` are `1 1`. **Every transition animation
  (`stopwalk`, `stoprun`, the stop animations) is `0 0`.** Gestures are
  `0 1`.

**Rules (traced):**

1. **Asking for a movement.** Connor's movement code asks for a new
   movement only when its choice changes (the table in "Which movement
   plays"). The request is remembered as the **wanted** movement (a new
   request replaces an older one).
2. **Choosing a transition.** Among the current movement's transitions
   whose `<to movement>` is the wanted one, the game takes the one whose
   `<from phase>` comes **next going forward** from the current phase:
   - from ≥ current phase: distance = from − current;
   - from < current phase: distance = from − current + 1 if the current
     animation loops (it is reached after the wrap), otherwise it is
     never reached and not chosen;
   - a `<from phase>` of **−1 means "at once"**: on a looping animation
     it always wins (its distance is ≤ 0);
   - on a tie the one listed first wins.
   (For an animation playing backwards the same is done the other way
   round; Connor's `back` has no transitions, so it does not matter
   here.)
3. **Waiting for the "from" point.** If a transition was chosen, the
   current animation keeps playing (and keeps moving him) until its phase
   reaches the chosen `<from phase>` within an update. That update's
   phase is cut off exactly at the from point, and the transition starts.
   With −1 the transition starts at the next animation update.
   **Order within one update (Q40 a):** Connor's state step runs first
   (that is where a movement is asked for), then the animation step
   (advance the phase, fire cues, check the from point, apply the
   frame). In both cases (a from point reached, or −1) the transition
   is set up at the end of that update's animation step, and it is
   **first advanced in the next update**. So pressing forward while
   standing gives one update of `stop` (the update the key is first
   seen; the whole update, nothing cut), then 7 of `stopwalk`, then
   `walk` from 0.85. Letting go during `stoprun` gives one update of
   `run` cut at 0.867, then `stoprunstop`. The rest of a cut update is
   not carried over.
4. **No transition listed.** If none of the current movement's
   transitions leads to the wanted one: if the current animation can be
   cut short (6th field 1) or has already reached its end, the wanted
   movement starts **at once, from phase 0**. Otherwise (a `0 0`
   animation still playing) nothing happens yet; the wanted movement is
   kept and handled when that animation ends (rule 6).
   **Every request also replaces a waiting transition (Q40 b):** the
   choice of rule 2 is made again for the new wanted movement, and if it
   finds nothing, the waiting one is dropped. "Starts at once from phase
   0" holds **even when the wanted movement is the one already playing**:
   the animation is set up afresh at its start. Example: walking, let go
   at phase 0.5 (`stopwalkstop` waits for 0.90), press forward again at
   0.7: `walk` has no transition to `walk`, so the waiting stop is
   dropped and `walk` restarts from phase 0 (a visible jump in the step
   cycle). See "Do not reproduce".
5. **Playing a transition.** The transition movement plays from phase 0
   at speed 1, once, to its end. It moves Connor by its own per-frame
   steps, like any movement (see "Steps in the start animations" below).
   The game remembers the transition's `<to movement>` and `<to phase>`.
6. **When the transition ends:**
   - wanted movement = its `<to movement>`: the `<to movement>` starts at
     its `<to phase>` (e.g. `walk` at 0.85);
   - wanted movement changed meanwhile (for example the key was let go
     during `stoprun`, so the wanted one is now `stop`): the
     `<to movement>` **still starts at its `<to phase>`**, and at once the
     choice of rule 2 is made again, from there to the newly wanted
     movement. If that finds a transition, rules 3–6 repeat. If not, the
     newly wanted movement starts at once from phase 0 (the `<to
     movement>` can be cut short).
   - A one-shot animation ending with no transition in progress: the
     wanted movement, if any, starts from phase 0.

**What this gives for starting (worked out from the rules and data):**

- Forward pressed while standing (`stop` loops, `stop → walk` has
  from −1): `stopwalk` starts at the next update, plays 0.467 s (7
  ticks), then `walk` at 0.85. Running: `stoprun`, 0.667 s (10 ticks),
  then `run` at 0.867.
- Let go during `stopwalk`: it still plays to its end (it is `0 0`),
  then `walk` starts at 0.85, and the walk → stop choice from 0.85 is
  `stopwalkstop` (from 0.90, the next point ahead), reached about one
  update later. Running: `run` starts at 0.867, and `stoprunstop` has
  from 0.867 (distance 0), so it starts at the next update. This is the
  measured "one update of `run`, then `stoprunstop`".
- Run switched on during `stopwalk`: `stopwalk` plays to its end, `walk`
  starts at 0.85, there is no walk → run transition, so `run` starts at
  once from phase 0 (the `walk` is not seen).
- Run switched on or off while walking or running: direct switch, the
  new loop from phase 0, no transition.
- Back, or a turn key alone, from standing: no transition listed, `stop`
  can be cut short, so `back` / `left` / `right` start at once from 0.
- Turning during a start: turning does not depend on the animation
  (see "Turning"), so he turns while `stopwalk` / `stoprun` play.
- Walking into a wall during `stopwalk`: head-on handling as for `walk`
  (`Stop`, forward blocked; see `connor-physics.md`). During `stoprun`:
  the `crash`, as for `run`.

**Steps in the start animations (data, `GAME/Common`).** These two do
not move him evenly and are not straight ahead. Frame k's offset is the
move from frame k−1 to frame k, over one frame's time (1/15 s; the play
time is (frames − 1) / 15 s). The model faces its −y axis; its +x side
is Connor's **left** (this matches the drift to his left that I measured
in the original). Units as everywhere.

| Shape | Frames | Play time | Steps (left, forward) per frame | Total |
| --- | --- | --- | --- | --- |
| `C_stpwk.kqs` (`stopwalk`) | 8 | 0.4667 s | frames 1–7: each (7.32, 11.22) | 51.2 left, 78.5 forward: 93.8 at 33° left of facing |
| `C_stprn.kqs` (`stoprun`) | 11 | 0.6667 s | 1: (0.18, 0.19) 2: (1.32, 1.28) 3: (3.85, 3.15) 4: (7.81, 5.73) 5: (8.79, 12.77) 6: (6.11, 86.57) 7: (6.04, 129.17) 8: (4.43, 136.26) 9: (3.59, 107.78) 10: (1.77, 43.80) | 43.9 left, 526.7 forward (corrected from 536.7, Q40 c) |

So `stoprun` stays almost on the spot for its first third (and goes
about 45° to the left), then surges forward. Spreading its distance
evenly over its play time, as is done now for `walk` and `run`, would
look wrong: the steps have to be applied frame by frame, both axes. (The
stop animations, `stoprunstop` especially, also step sideways; same
rule.)

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
  at its normal frame rate. Keep the end value **0.18 rad** on both sides
  (rule 3); the −0.15 end after swinging over from the other side is a
  float-rounding accident and need not be copied.
- **Release of one key cancelling the other.** Recommended: track which keys
  are held and derive the intent from them, so releasing Left while Right
  is still held keeps turning right. This is a deliberate improvement; keep
  a setting to get the original behaviour if exact matching is wanted.
- **Turning disabled → silently dropping input.** Fine to keep, but log it.
- **Walk or run restarting from phase 0 when asked for again while it
  plays** (rule 4, Q40 b). Pressing forward again while a stop is still
  waiting snaps the step cycle back to its start. **Confirmed in the
  original by the user (2026-10-12):** letting go of forward and quickly
  pressing it again resets the walking animation. **Decision (user,
  2026-10-12): not reproduced.** When the wanted movement is the one
  already playing, the waiting transition is dropped and the animation
  carries on (B's implementation). A deliberate difference from the
  original.

## How to check

- Log Connor's position every update while walking straight for 10 s:
  expect ≈ 11,270 units travelled (≈ 25,180 when running).
- Log heading while holding Left for 2 s: expect ≈ 3.5 rad change.
- Log the chosen movement name for each input combination in the table.
- In the original game, walk Connor between two landmarks with known
  positions (from the `.kq` files) and time it; compare with ours.
- **In the original (Turning rule 2, Q104 a, not checked yet):** stand
  facing away from the alchemist's sign, hold the **left** turn key so
  Connor spins, and click the sign while still holding it. Expected by
  the code: he turns to face the sign at the fast automatic speed (the
  held key changes nothing), and when he faces it he **stops turning**
  although left is still held; only letting go and pressing left again
  starts him turning again. If instead he goes on spinning left at once
  after facing the sign, the "held key is not re-sent" rule is wrong.
  **Checked by the user in the original (2026-10-18): as described.**
- **Our logs:** per turn key event: down/up, and "auto-turn cancelled"
  only on a down; per update during an automatic turn: heading, turn
  direction, forward intent.

## Measured in the running original (2026-10-12)

Read from the game's memory on flat ground (game clock time). Update
counts below are my log lines (one per drawn frame, about 78 ms, one or
two 1/15 s ticks each); go by the seconds.

- Walk: 1,083–1,131 units/s over 2.4–3.1 s stretches (average about
  1,100, about 2% under the data's 1,127). Run: 2,492–2,561 (average
  about 2,520; data 2,518). Stepping back: 1,079–1,085.
- Turning on the spot: 98–102° per second (data: 1.75 rad/s = 100°/s),
  6.7° per 1/15 s tick, the same left and right.

## Open questions

1. Are the per-frame steps applied in one go when each frame is reached, or
   blended between frames? Affects smoothness, not average speed.
   **Measured (2026-10-12),** on flat ground: walking, the distance per
   1/15 s tick changes smoothly between 70 and 81 (average 75), rising
   and falling with the step cycle, with no jumps or empty ticks. That
   fits steps **spread within each frame** (each frame's own step over
   that frame's time), not one even speed for the whole loop and not
   whole steps at frame changes. Not yet compared frame by frame with the
   steps in `C_walk.kqs`.
2. ~~Which transition is used when several have "from" phases, and what
   happens if the stop is requested between them?~~ **Traced
   2026-10-12:** the next "from" point ahead, waiting for it; −1 = at
   once (rules in "Starting and stopping"). The sideways drift below is
   the start animations' own sideways steps (table there). **Still not
   checked:** the measured start distances (about 80 and 385–490) are
   under the data's totals (93.8 and 526.7); probably my log's first
   and last updates cut parts off, not confirmed. The running stops'
   15% phase offset below is not explained by the rules either; most
   likely my phase estimate, not confirmed.
   **Measured (2026-10-12), before the trace:**
   - A stop never cuts the cycle at once. After the key is let go, the
     walk or run keeps playing for 0–4 updates (state already Wait),
     then a stop animation from the list plays.
   - Walking (5 stops): let go at 32% of the cycle → `walkstopleft`
     started at about 50%. Let go at 76%, 89% and 95% → `stopwalkstop`
     started at about 95–100%. `walkstopright` never played. That fits
     "wait for the next listed point, play that point's animation", if
     my phase estimate runs about 5% late (it is worked out from the
     clock, ±1 update, not read from the game).
   - Running (8 stops): `runstopleft` (12–13 updates, about 1 s) and
     `stoprunstop` (6–8 updates, about 0.6 s) both played;
     `runstopright` never did. The phases fit the same rule only with an
     unexplained 15% offset, so this part is **not settled**.
   - Starting: `stopwalk` 5–6 updates (about 0.45 s, about 80 units),
     `stoprun` 6–9 updates (0.6–0.7 s, 385–490 units). Letting go during
     `stoprun`: one update of `run`, then `stoprunstop`.
   - Seen, not explained: during `stopwalk` and `stoprun` he moves about
     35–50° to the left of his facing, and during `stoprunstop` almost
     straight to his right (short distances). Probably sideways steps in
     those animations' data; check the `.kqs` steps.
   To settle it, the reader needs to log the animation phase itself
   (the place in memory is not found yet).
3. Jumping, falling, slopes and walls: answered in `connor-physics.md`
   (walkable limit is 36.9°, not 35°). Water (`waterwalk`), ropes and
   pushing are not studied yet.
4. **Why Connor is upright during a jump (Q59, closed 2026-10-13: the user compared the original with our build and accepted the "switch off in jumps, falls and landings" rule as correct; the cause in the code stays unknown and is not being pursued).** The user
   saw in the original (2026-10-13) that he doesn't stay leaned through
   a jump, even holding the turn key. Reading the code says he should:
   the jump start leaves the lean switch on, the turn key keeps working
   in the air, the lean is updated in every move including the jump's,
   the jump animations carry no rotation of their own, and only the lean
   rule itself ever changes the lean value. Checked and ruled out: the
   jump start, the jump key, the state-start code shared by all states,
   the animation steps, every place that writes the lean, the switch,
   the turn direction, the forward intent or the turn on/off flag, and
   the `allowLeaning` flag (only four special states, not the jump,
   clear it). Not ruled out: something in how the drawn orientation is
   taken from the mover during a jump, or the user's test conditions
   (whether forward was held and whether he turned in the air; a 10°
   lean may also be hard to see in the jump pose). How to settle it:
   (a) the user repeats the test, holding forward and turn the whole
   time, and says whether Connor turns in the air and whether he
   straightens at once or over about half a second; or (b) a debugger
   run (as for Q30) logging the lean value every update through a
   running jump.
