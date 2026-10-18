# Pushing objects

Study session A, 2026-10-17. Traced in `Mask.exe` unless marked "data"
(read from the game's files) or "not traced".

## Purpose

How Connor pushes things: when walking into an object turns into
pushing, what the push state does, how a pushed object moves (barrels,
the hay bale), and how a "scripted" pushable hands control to its script
(`Push` call: Daventry's bricks, the hero's coffin lid, the church urn).

Builds on: `connor-collision.md` rules 2–4 (who is told about a touch,
the order of type bits), `connor-physics.md` "Walls" and "Exact hit
handling" (what a recorded hit does to Connor's move), `connor-movement.md`
(movements and the per-frame steps that move him), `anim-scripts.md`
rule 16 (Connor's Special state, which the push scripts use next).

Words used here:

- **Type bits**: an object's `colliderType` number, read as bits (hex).
- **Pushable**: type bit **0x100**. **Scripted pushable**: bits 0x100
  **and 0x10000**. **Physical pushable**: 0x100 without 0x10000.
- **Pass**: one of the three parts of Connor's move in an update (up,
  across, down; `connor-physics.md`).
- **Hand mode**: which weapon Connor holds: **sword = 0, bow = 1, empty
  hands = 4** (the game's own help text for `KQConner::getDoMode`). He
  starts with empty hands; scripts set it with `KQConner::setDoMode`.

## Rules

### When a touch becomes a push

1. In the table of `connor-collision.md` rule 3, bit **0x100 comes
   first**. For **Connor**, a touch with an object whose type has 0x100
   is not simply a wall: it goes through the push check (rules 2–4).
   Plain objects keep treating 0x100 as an obstacle.
2. The push check only acts when **all** of these hold; otherwise the
   touch is an ordinary wall hit for Connor (recorded, then stop / slide
   / head-on as in `connor-physics.md`):
   1. Connor is the one **moving** (not the object moving into him).
   2. He is **on the ground or already pushing**: his height is less than
      1 unit above the height the standing state last recorded for him
      (in practice: not jumping or falling), or his state is already Push.
   3. **Empty hands** (hand mode 4). With a sword or bow drawn, pushables
      are just walls.
   4. The touch comes from the **across** pass (not up or down).
   5. **He faces it**, to within 60° (cos ≥ 0.5):
      - scripted pushable: his facing against the touched surface
        (pointing into it);
      - physical pushable: his facing against the direction from him to
        the object's centre (horizontal only). "Centre" is the object's
        **position** (its placement point), not the middle of its box;
        "him" is his position at the start of this update, before this
        update's move (Q87 e).
      The turn keys keep working in the Push state at the normal rate
      (nothing there checks the state; only `handsOff` and
      `KQMonster::setCanTurn 0` block turning). Turning with forward
      held keeps pushing; turning alone fails item 6 below (he stands);
      turning more than 60° away fails this item (back to Move) (Q87 d,
      inferred: no block was found).
      If this fails while he is already pushing, he goes back to the
      Move state; the touch is an ordinary wall hit.
   6. His chosen movement is **`walk`, `run` or `push`**. Anything else
      (backing up, turning on the spot, wading) makes it an ordinary wall
      hit **and puts him in the standing state** (Wait).
3. If all hold, Connor asks for the **Push state** (rules 6–8). If his
   current state refuses it, the touch is an ordinary wall hit. Once in
   Push, every such touch resets the push counter (rule 7) and then:
   - **scripted pushable** (0x10000 set): the **object's script runs with
     `$1` = its name, `$2` = `Push`**, nothing else (no `$3`). It runs at
     the moment of the touch, in the middle of Connor's move, and again
     at **every** later pushing touch (each update), until the script
     stops it (rule 10);
   - **physical pushable**: the object is moved (rules 4–5).
   In both cases **the touch is not recorded as a hit for Connor**, so it
   does not stop or slide him.

### Physical pushables (barrels, the hay bale, boxes, ice blocks)

4. **Moving the object**, at each pushing touch, if Connor's move in this
   pass is not zero (more than 0.000001 on some axis):
   1. The object is asked to move by **Connor's horizontal step of this
      pass** (x and y; its height and turn stay), through its own
      collision sweep over one update (1/15 s). During that sweep it
      **ignores Connor** (his type bit 4 is taken out of its mask for the
      moment). What can stop it depends on its shape (corrected
      2026-10-17, Q87 a–c, traced, not checked in the game):
      - **Box-shaped objects** (no `collisionShape`: the default shape
        is a box, turned with the object; Daventry's barrels) are
        **never stopped by anything**: not walls, buildings or other
        objects. Their touches are only overlap tests (against a
        building, its bounding box) and carry no contact point, and
        only a touch with a contact point is recorded as a hit. The
        barrels' mask is also 0, so they aren't even told about touches.
      - **Ball-stack objects** (`collisionShape=1`; the hay bale, mask
        11) get real sweeps and are stopped by what their mask covers.
        A hit is handled like Connor's slide: the rest of the move is
        turned along the surface and swept again, at most 3 retries per
        pass, then it stays at the contact (pulled back 25 units). There
        is **no head-on stop** for objects: pushed straight in it stops,
        pushed at an angle it slides along. In the across pass a surface
        whose normal z is between 0 and 0.7 counts as a vertical wall.
      - **Ground afterwards: raise only.** Objects have no gravity by
        default (`useGravity` off) and step up / step down 0. With
        `keepOnGround` on, after the move: if the ground at the new spot
        is at or above the object's height, the object is put on it; if
        the ground is lower, nothing happens. With `keepOnGround=0` (the
        hay bale, the bricks) there is no ground step at all. This is the
        same rule as at load (`objects.md` Q10), applied after every
        move.
   2. **If its height changed at all** (any difference from exactly
      0.0), it is put back where it was and counts as not moved. With
      the raise-only ground rule above, this means: pushed toward
      **higher** ground it doesn't budge; pushed toward **lower** ground
      its height doesn't change, so it **moves and floats** at its old
      height (traced 2026-10-17, Q87 c; not checked in the game). On flat
      ground it moves normally.
   3. **Connor** is then put at the **start of his pass plus 0.95 × the
      object's actual movement** (all three axes, including any slide
      along a wall). So he follows it,
      falling slightly behind; next update his step reaches it again. If
      it didn't move, he stays at the start of the pass.
      **This does not end his pass** (traced 2026-10-18, Q95; B's
      stand-in, which ends the pass there and ignores the wall, is
      wrong). In order:
      - Everything his balls can touch is tested in **one list, in the
        order things entered the world** (`connor-physics.md` "Against
        a building" step 7). The push happens when the pushed object's
        turn in that list comes. At that moment his position **and his
        sweep** both become the 0.95 position (call it **P**): for the
        rest of the list his move has length 0. Buildings later in the
        list then test nothing (a move under 0.1 is never tested);
        objects later in the list with a zero-length move: not traced.
      - Things **earlier** in the list were tested on his full move (pass
        start to pass end). A wall contact found there is kept as usual
        (the nearest, pulled back 25), and the push does not remove it.
      - After the sweep, the kept hit (if any) goes to the hit handling
        of `connor-physics.md` "Walls", with two differences: the
        **wanted end is P**, and his **movement counts as zero** (start
        and end of the sweep are both P), so the hit is **never
        head-on**: no `Stop`, no wall block, no `crash`. It is handled
        as glancing: target = P projected onto the wall's plane through
        the kept contact (target = P − n × ((P − contact) · n)); he is
        put at the contact and swept to the target (the usual slide,
        with its retries). Its result **replaces P**.
      - So with a wall in the sweep he ends **on the wall's contact
        plane** (his centre 225 from the face), keeping only the part of
        P that runs along the wall. The pushed object goes on: a
        box-shaped barrel is never stopped (rule 4.1) and goes into the
        wall.
      - The slide's sweep is a new sweep. If it touches the pushed
        object again, the push rules apply inside it too (the object
        moves by the slide's step; his end becomes the slide's start +
        0.95 × that). Pushing straight into a wall the slide is about 0
        long and nothing more happens.
      - **Daventry:** every building (`permwrld.cs`, run from
        `newworld.cs` line 64; the pub is `pub.kq`) enters the world
        before every barrel (`newworld.cs` lines 283–286), so a pub wall
        is always found on his full move. (If a building came later in
        the list than the barrel, he would follow the barrel through
        the wall; this doesn't happen in Daventry's data.)
      - **Afterwards (Q95 c):** only an update in which his sweep really
        reaches the object is a pushing touch (rule 3). Held at the
        wall, he no longer advances while the barrel moves on by his
        full step at each touch, so within one or two updates his step
        no longer reaches it. While he is in Push, wall hits are never
        head-on (head-on only acts in Move and Jump), so he just stays
        pressed against the wall. After **4 updates without a pushing
        touch** Push ends (rule 7): key held → Move (walking), and the
        next walk into the wall is head-on → `Stop` with the forward
        block (`connor-physics.md` "Walk held against a wall"). This
        matches the user's check in the original (2026-10-18): the
        barrel goes through the pub's wall, the wall stops Connor, and
        he stops pushing once the barrel is too far in.
5. Physical pushables do **not** run a `Push` script (their scripts, e.g.
   `barelup.cs`, only answer `Die`).

### The Push state

6. **Entering**: Connor's `push` movement starts (`Conner.anm` movement
   `push`: `c_push.kqs`, 30 frames, 1.933 s, repeating; sounds
   `push1.aud` at 0, `Push3.aud` at 0.5, `Push4.aud` at 0.733; footsteps
   at 0 and 0.5, data). Its per-frame steps move him **forward 36.6 to
   40.6 per frame** (about 590 per second, half his walking speed, data),
   which is what keeps him pressing into the object.
7. **Every update**, a counter (set back to 0 by every pushing touch,
   rule 3) is checked, together with whether a movement key still asks
   for a movement:
   - counter below 4 and a key held: `push` is asked for again and the
     counter goes up by 1;
   - counter below 4 and no key: **standing** (Wait);
   - counter 4 or more (no pushing touch for 4 updates): key held →
     **Move** (back to walking or running); no key → Wait.
8. From Push he can only go to Wait, Move, Special (scripts), Die, and
   one more state (0x8000, not identified).

### Scripted pushables (Daventry: bricks, coffin lid, urn)

9. **Data:** `colliderType=65825` (0x10121 = scripted pushable +
   clickable 0x20 + 1) on `bricks.kq`, `coffnlid.kq`, `urn.kq`
   (Daventry), `kingart.kq` (Castle D); the Barren symbols (`KQDoor`)
   and Dead City's `paddle2` (`KQPaddleObject`) have it too; Temple 3's
   `order01–09` (`KQPushTile`, 0x10101) have their own class (not
   traced).
10. **What the Daventry scripts do with `Push`** (data): all three take
    control at once on the first call, so the repeated calls of rule 3
    don't matter:
    - they remove the push and click bits (`KQObject::collisionType $1
      -288`, -0x120; the scripts' own comment: "remove both push and
      click"), so the next touch is an ordinary wall hit;
    - `handsOff 1`, camera changes, `move Connor …` to a set spot;
    - Connor plays a scripted animation through the Special state
      (`anim-scripts.md` rule 16): `bricks.anm` `push`, `urn.anm` `away`
      then `push`, `c_pshoff.anm` `pushoff`, which ends the Push state;
    - `bricks.cs` only acts while `$Daventry::bricks` is `up`
      (`worldvar.cs` sets it at the start) and sets it to `down`;
      `urn1.cs` uses `$Daventry::Church::urn` the same way; `coffnlid.cs`
      has no such guard (the bit removal is enough).
11. **The bricks puzzle, start to end** (data, all its steps now
    described): Connor walks into `Bricks` with empty hands → Push state,
    `bricks.cs Push` → camera and Connor placed, `getEndLoop Connor
    Bricks 1`, bricks and `Table` hidden, Connor plays `push` from
    `bricks.anm`, push/click bits removed, experience +50, map icon →
    when that movement ends: `Cue 300 0 push` → bricks shown and moved,
    `KQAnimPlayEvent down`, waterfall replaced (`deleteObject`,
    `loadKQ waterfa2.kq`, `watrfsn2.kq`), `water` → `blocked`, `KQCueEvent
    5` → camera, `KQCueEvent 10` 4 s later → controls back, Connor
    remarks.
    **Timing and heights** (traced 2026-10-17, Q92):
    - The `Push` script runs **inside** Connor's movement update, at the
      moment his across pass touches the bricks (rule 3). So after its
      `move Connor 0 156652 303600 15000 …` the **rest of that same
      update still runs**: a gravity step and the down pass with its
      325 step-down, which tests the floor. The floor there (the cave
      building's top, 14,812) is within reach, so he lands on it
      (14,837, 25 above the floor) **in the same update**; the original
      never draws him at 15,000.
    - During `push`, the real `Bricks` and `Table` are hidden; what the
      player sees are copies of the table and bricks **inside Connor's
      own model** `c_brks1.kqs`, whose origin is the bricks' place (same
      x, y and turn). So anything that moves Connor up or down moves
      "the bricks and the table" with him on screen.
    - The real `Bricks` and `Table` keep their placed height (14,862;
      `keeponground=0`, no gravity, step heights 0). Nothing changes it
      until `move Bricks 3 0 -200 -300` (exactly 300 down, 200 along
      −y). `brktabl2.kqs` also has steps of 0. `setHidden` does not
      change an object's position or height (`objects.md`).
    - The cue-10 `move Connor … 15000` comes from a timed cue, outside
      his update, so there he should be drawn about 160 up for one update
      before he drops (inferred, not checked in the original).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| pushable type bit | 0x100 | traced |
| scripted-push type bit | 0x10000 | traced |
| facing needed | cos ≥ 0.5 (within 60°) | traced |
| hand mode for pushing | 4 (empty hands); sword 0, bow 1 | traced |
| "on the ground" margin | 1 unit above the recorded standing height | traced |
| updates without a pushing touch before leaving Push | 4 | traced |
| pushed object's sweep time | 1/15 s (0.0666) | traced |
| share of the object's movement Connor follows | 0.95 | traced |
| smallest move that pushes | 0.000001 | traced |
| `push` forward step | 36.6–40.6 per frame, 30 frames in 1.933 s | data |

## Data used

- `colliderType` of every object (bits 0x100, 0x10000, 0x20).
- `GAME/Kq/Conner.anm` movement `push` and `c_push.kqs` frame steps.
- Scripts: `bricks.cs`, `coffnlid.cs`, `urn1.cs` (Daventry), the Special
  lists `bricks.anm`, `urn.anm`, `c_pshoff.anm`; `worldvar.cs`.
- Daventry placements (data, `newworld.cs`): hay bale `haybail.01` in
  the windmill (147703, 248928, 15195; `keeponground=0`); barrels
  `barrel.04` (206564, 267646, 8345), `barrel.07` (199536, 262152, 8185),
  `barrel.12` (188861, 269213, 8185), `barrel.13` (188287, 266690, 8185).

## Edge cases

- **Sword or bow drawn**: nothing can be pushed (rule 2.3).
- **Backing into a pushable** (`back`) or turning against it: no push,
  and he is set to standing (rule 2.6).
- **Jumping into a pushable**: not on the ground, so an ordinary hit
  (head-on and crash rules of `connor-physics.md` apply).
- **A physical pushable toward higher ground**: it never moves (rule
  4.2); Connor plays `push` against it and stays put, because each update
  he is put back to the start of his pass. Toward lower ground it moves
  and floats at its old height (rule 4.1, raise only).
- **A barrel pushed into a wall or another object**: it goes into or
  through it (box-shaped objects are never stopped, rule 4.1).
- **Two touches at the same distance** (the bricks and their table):
  both objects are told; see `connor-physics.md` "Against a building"
  step 7.
- **A scripted pushable whose script does nothing** (or has no script):
  Connor stays in Push and walks **into** it, since the touches aren't
  recorded as hits for him (rule 3). No Daventry object does this (data).
- The object moving into Connor never pushes it (rule 2.1); see
  `connor-collision.md` rule 5.

## Do not reproduce

- **Walking into a scripted pushable that doesn't react** (last-but-one
  edge case): no data does it; if it ever happens, treat the touch as a
  wall hit as well, and log it.
- Keep the rest, including "uphill: no push", "downhill: floats" and
  "barrels pass through walls" (the user saw that in the original,
  2026-10-18).

## How to check

- **Our logs (B):** per pushing touch: object, which conditions of rule
  2 passed or failed, facing cos, state change; for physical pushes the
  asked step, the object's movement, "height changed, put back", and
  Connor's new position; for scripted pushes the `Push` call.
- **Daventry, bricks:** the game's own debugging shortcut `alias bricks`
  (`newworld.cs`) puts Connor at (156625, 303748, 15000), turn 1.57, next
  to `Bricks` (156652, 303600). Walking forward into it with empty hands
  should start the puzzle (rule 11).
- **In the original (user), checked 2026-10-17, all as expected:**
  1. The hay bale in the windmill slides along the floor at about half
     walking speed and stops at walls (rules 4, 6).
  2. Barrels move, but only where the ground is perfectly flat (rule
     4.2). Pushing downhill was not tried; by the trace (Q87 c) a barrel
     pushed downhill moves and floats.
- **Checked by the user in the original (2026-10-18), matching the
  2026-10-17 trace:**
  1. A barrel pushed uphill stops (rule 4.2).
  2. A barrel pushed against the pub's wall goes through it (rule 4.1,
     box-shaped objects are never stopped).
  3. The hay bale pushed into a wall at an angle slides along it (rule
     4.1, ball-stack objects slide).
  4. Downhill can't be tried: there is no slope near Daventry's barrels.
     So "floats downhill" (rule 4.2) never shows in Daventry.
  3. With a weapon drawn, or walking backwards, nothing is pushed
     (rules 2.3, 2.6).
- **Barrel into the pub's wall (Q95, our logs, B):** push `barrel.04` or
  `barrel.07` straight into the pub. Expected per update in
  `connor.log`: while the wall is not in his sweep, Connor at the pass
  start plus 0.95 × the barrel's movement, as now; in the first update with a wall
  contact, "push, then kept wall hit: wanted end = P, movement 0, not
  head-on, slide to the wall plane", and his distance to the wall's face
  225 from then on (never less); the barrel's x/y keeps changing by his
  full step at each pushing touch; then 1–2 more pushing touches, 4
  updates without one, Push → Move, a walking head-on → `Stop` with the
  forward block. The user saw this in the original (2026-10-18).

## Open questions

1. The other pushable classes (`KQBoulder` in Gnome, `KQPushTile` in
   Temple 3, `KQPaddleObject`, the Barren symbols as `KQDoor`) may react
   in their own way; not traced (not in Daventry).
2. State 0x8000 that Push can switch to: not identified.
3. The exact moment the standing state records Connor's height (rule
   2.2): not traced; "on the ground, not jumping" is the reading.
