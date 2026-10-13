# Traps (trigger boxes)

Study session A, 2026-10-13. Sources: Daventry's trap `.kq` files and
scripts (`MASK/daventry/kq/`), all 12 worlds' trap settings, and
`Mask.exe` (studied, described here in our own words).

## Purpose

A `KQTrap` is an invisible box in the world. When Connor walks into it, or
out of it, the trap runs its script. Daventry uses 50 traps for: switching
between walking and running near houses and the camera range that goes
with it (`walk_la.cs` / `run_la.cs`), music zones (church, wizard), rope
climbing spots, teleporter pads, the cliff-edge cutscene, telling monsters
to go home, and save-game prompts. Nothing in a trap is drawn, and a trap
never blocks anything.

## Rules

### What a trap is made of (data)

1. Keys a trap reads (all optional except the box): `loc` (`locX/Y/Z`),
   `BBMinX/Y/Z`, `BBMaxX/Y/Z` (the box, in the trap's own space, i.e.
   added to `loc`, never turned), `script`, `colliderMask`, `trapType`,
   `innerPercent`. Unknown to traps and ignored: `keepOnGround` (22
   Daventry traps have it; a trap is never moved onto the ground by it).
2. `locZ = -1` means "on the floor grid" (the ground under water,
   `water.md`): the z is replaced by the floor height at the trap's x, y
   when the trap enters the world. Only −1; a −2 is not special for traps.
3. Traps without `locX/Y` (e.g. `conwalk`, `conrun`) are templates; world
   scripts place copies with `loadKQ` and a position (`objects.md`).
4. **Who sets a trap off.** A trap's own collision type is 0x10; its mask
   is `colliderMask`. The usual pair rule applies (`connor-physics.md`
   "Which pairs collide": either side's mask covers the other's type).
   Every trap in the data has `colliderMask=4`, Connor's type, so only
   Connor sets them off. Monsters (types like 0xC021) do not.
5. **A trap is never solid.** It only reports contacts; nothing is pushed
   or stopped by it (from play; the "no response" part is not traced).

### Entering and leaving

6. **Entering.** When Connor's collision shape touches the trap's box,
   he is added to the trap's "inside" list (once; touching again while
   inside does nothing). If he is the **first** one inside, the script
   runs with `$1` = trap name, `$2` = `On`, `$3` = Connor's object name.
   If someone else was already inside, it runs with `$2` = `Cue`, `$3` =
   `301`, `$4` = the newcomer's name. (Only Connor sets traps off in the
   data, so in practice it is always `On`.)
7. **Staying.** Every time an object inside moves, the trap checks
   whether it is still inside: first, is the **centre of the object's
   box** (its position plus the middle of its `BOUNDS.KQ` box) inside the
   trap's box (both corners included)? If yes, still inside. If not, is
   the object's collision shape still touching the trap's box? If yes,
   still inside. Only when both fail has it **left**.
8. **Leaving.** The object is taken off the list. If the list is now
   empty, the script runs with `$2` = `Off`, `$3` = the object's name.
   Otherwise `$2` = `Cue`, `$3` = `302`, `$4` = the name. While the world
   is being shut down, no `Off` is sent.
9. So `On` comes when he first **touches** the box, and `Off` only once
   he is **out of touch and his middle is outside** it.

### `trapType` (bit flags)

10. **Bit 2 (`trapType=2`), the "sticky" trap.** When the trap enters the
    world, its box is shrunk once to `innerPercent` of its size (every
    one of the six `BB` numbers multiplied, so it shrinks towards the
    trap's own `loc`, not towards the box's middle). When Connor enters,
    the box grows back to full size (all six divided by `innerPercent`);
    when he leaves, it shrinks again. So he has to get into the **inner**
    box to start the trap, and out of the **full** box to stop it. This
    stops music zones from flickering on and off at the edge. All 14
    such traps in the game are music zones (`chrchtrp` 0.9, `mugltrap`,
    `mwiztrap`, `mwzhtrap` 0.8 in Daventry).
11. **Bit 1 (`trapType=1`): one-shot.** After the `On` call the trap
    deletes itself. Not used by any trap in the data.
12. No `trapType` (0, the usual): the box is used as written.

### Other script calls

13. **`Init`**: when the trap enters the world, the script runs with
    `$2` = `Init` (no other arguments).
14. **`Cue`**: `sendEvent … <trap> KQCueEvent n1 n2` runs the script with
    `$2` = `Cue`, `$3` = n1, `$4` = n2 (and a fifth word when the event
    carries one), as for doors (`doors.md` rule 23). `clifTrap.cs` uses
    this: it sends itself `Cue 10` to switch running back on.
15. Traps also pass on `TalkComplete` (when a conversation they started
    ends) and `StateChange` (a monster's state change they asked to hear
    about) to their script. Those belong to M4/M5.
16. Scripts commonly turn a trap off for good with
    `KQObject::collision <trap> -4` (removes bit 4 from its mask, so
    Connor no longer counts; `clifTrap.cs` does it on its first `On`).
    What happens to an object already inside at that moment was not
    traced: our stand-in is that no `Off` is sent.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| trap's own collision type | 0x10 | traced |
| `colliderMask` in all trap data | 4 (Connor) | data |
| cue numbers for a second object entering / leaving | 301 / 302 | traced |
| `innerPercent` values in the data | 0.8, 0.85, 0.9 | data |
| `trapType` values in the data | only 2 (14 traps) | data |

## Data used

- Trap `.kq` files: `classType=KQTrap`, `loc`, `BBMin/BBMax`, `script`,
  `colliderMask`, `trapType`, `innerPercent`.
- Their scripts (`*.cs`), and the shared ones in `GAME/Kq/`
  (`walk_la.cs`, `run_la.cs`, ...).
- `GAME/patch/BOUNDS.KQ` for the box of the object inside (Connor's).

## Edge cases

- Template traps placed several times (`walktp1`, `runtp1` ...): each
  copy is its own trap with its own "inside" list.
- Two traps overlapping (e.g. a walk zone inside a music zone) are fully
  independent; both get their `On`.
- A trap whose box is only touched at its very edge: `On` is sent
  (touching counts), and `Off` comes on the next move that takes him out
  of touch, because his centre is already outside.
- Saved games store each trap's "inside" names and whether its box is
  currently shrunk (M6).

## Do not reproduce

- Nothing found. Keep the asymmetric enter/leave test (touch to enter,
  centre-outside-and-not-touching to leave); it is what makes a trap feel
  forgiving at its edge.

## How to check

- **In our logs:** for every trap event: trap name, `On`/`Off`/`Cue`,
  Connor's position, the box used (shrunk or full). Walk Connor into
  `chrchtrp` from outside: `On` once his shape reaches the inner (90%)
  box; walk back out: `Off` only after his centre leaves the full box.
- **In the original, by ear:** near the wizard's house (`mwzhtrap`, a
  sticky trap, about (143974, 206673)), walk slowly in until the wizard
  music starts, then step back a little: the music should keep playing
  until you are clearly further out (the full box).
- **Checked by the user (2026-10-13):** entering the areas around
  Connor's house and near the wizard starts their music, and leaving
  switches back to the world music. (Connor's house area is a plain
  walk zone, `walk_la.cs`, which also pushes `minside.aud`; not
  sticky.) The church music plays inside the church; the user can't
  walk in and out there. `chrchtrp`'s box (x 201,730–217,023, y
  180,518–186,681) covers the church and some ground around it; the
  stickiness itself (step back a little, music stays) is not checked
  yet.

## Open questions

1. Whether removing a trap's mask bit (rule 16) sends `Off` for whoever
   is inside.
2. The exact contact test used for "touching" (rule 6): the same
   collision pair test as for walls, assumed; it may be a box-against-box
   overlap only.
