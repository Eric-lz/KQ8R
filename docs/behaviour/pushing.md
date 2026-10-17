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
        the object's centre (horizontal only).
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
      moment), so walls, other objects and the ground can stop it; its
      own `keepOnGround` placement applies as for any move.
   2. **If its height changed at all** (any difference from exactly
      0.0), it is put back where it was and counts as not moved. So an
      object can only be pushed where its height stays the same: flat
      floors and flat ground. On a slope it doesn't budge.
   3. **Connor** is then put at the **start of his pass plus 0.95 × the
      object's actual movement** (all three axes). So he follows it,
      falling slightly behind; next update his step reaches it again. If
      it didn't move, he stays at the start of the pass.
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
- **A physical pushable on uneven ground**: it never moves (rule 4.2);
  Connor plays `push` against it and stays put, because each update he
  is put back to the start of his pass.
- **A scripted pushable whose script does nothing** (or has no script):
  Connor stays in Push and walks **into** it, since the touches aren't
  recorded as hits for him (rule 3). No Daventry object does this (data).
- The object moving into Connor never pushes it (rule 2.1); see
  `connor-collision.md` rule 5.

## Do not reproduce

- **Walking into a scripted pushable that doesn't react** (last-but-one
  edge case): no data does it; if it ever happens, treat the touch as a
  wall hit as well, and log it.
- Keep the rest, including "uneven ground: no push"; the user can check
  it (below).

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
     4.2). Pushing downhill was not tried; by the rule any change of
     height, down as well as up, stops the push.
  3. With a weapon drawn, or walking backwards, nothing is pushed
     (rules 2.3, 2.6).

## Open questions

1. The other pushable classes (`KQBoulder` in Gnome, `KQPushTile` in
   Temple 3, `KQPaddleObject`, the Barren symbols as `KQDoor`) may react
   in their own way; not traced (not in Daventry).
2. State 0x8000 that Push can switch to: not identified.
3. The exact moment the standing state records Connor's height (rule
   2.2): not traced; "on the ground, not jumping" is the reading.
