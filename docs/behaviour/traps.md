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
4. **Who sets a trap off** (corrected 2026-10-13, Q57, Q58). A trap's own
   collision type is 0x10; its mask is `colliderMask`. **Only the trap's
   own mask counts:** a trap is told about a touch when its mask covers
   the visitor's `colliderType` (`connor-collision.md` rule 2; the
   two-sided rule only decides which pairs are looked at). Connor's own
   mask has bit 0x10, but that only means Connor is told too, and he
   ignores traps (rule 5). Any object can be a visitor, not only Connor.
   The masks in the data:

   | Mask | Set off by | Traps |
   | --- | --- | --- |
   | 4 | Connor (type 4) | 285 traps, all 58 in Daventry |
   | 516 (4 + 0x200) | Connor, inventory items (type 0x220) | 17 in Temple1 (`trap01.kq`: ";Collide with Conner=4, Inventory=512"), IceWorld `pondtrap`, Swamp `platrap` |
   | 517 (4 + 0x200 + 1) | the above and anything with type bit 1 | Gnome `triptrp1–4` |
   | 519 (4 + 0x200 + 2 + 1) | the above and anything with type bit 2 | DeadCity `triptrp2–5`, `trptrp12–15` |
   | 5 (4 + 1) | Connor and type bit 1 | Gnome `agtrap`, `aptrap`, `aptrap2`, `wgtrap`, `wgtrap2` |
   | 1 | type bit 1 only, **not Connor** | Gnome `bouldtrp`, `bouldtp2`; DeadCity `girltrap`; Temple2 `booktrap` |
   | 2 | type bit 2 only | DeadCity `fightrp3` |
   | 6 (4 + 2), 7 (4 + 2 + 1) | Connor and type bit 2 (and 1) | Temple4 `vortrap`; DeadCity `coaltrap` |
   | 0 | **nobody**, until a script adds bits | Barren `gongtp*`, `tiletpn`; DeadCity `portrap2–5`, `portrp2b–5b` |

   Type bit 1 is on most solid objects and many monsters (0x21, 0x101
   `boulder`, 0x121 `pushbook`, 0x141 push blocks, 0xC021 monsters, the
   rolling rocks' 0x1). Type bit 2 is on creatures (monsters 0x42,
   0x4042, 0xC022, 0x8022 gnomes, 0x4002 …). The scripts fit: `bouldtrp`
   checks for the `boulder`, `booktrap` for `pushbook`, `girltrap` for
   the pushed box `girlbx.02`, `coaltrap` sets whatever entered on fire.
   Mask 0 traps get bits from scripts (`KQObject::collision portrap2 +4`,
   `KQObject::collision gongtpX 4`).
5. **A trap is never solid** (traced 2026-10-13). Connor's reaction to a
   touch with type 0x10 is nothing, and a plain object's is nothing too
   (`connor-collision.md` rule 3), so nobody is stopped or pushed by a
   trap.

   **Only moving things set traps off.** A touch is only found while the
   visitor moves (`connor-collision.md` rule 1). An object lying still in
   a trap's box doesn't count until it moves; an item counts while it is
   tossed or falls (its throw moves it through its mover), a pushed box
   while it is pushed. Whether being placed by `move` or `loadKQ` counts
   was not traced.

### Entering and leaving

6. **Entering.** When a visitor's collision shape touches the trap's box,
   it is added to the trap's "inside" list (once; touching again while
   inside does nothing). If it is the **first** one inside, the script
   runs with `$1` = trap name, `$2` = `On`, `$3` = the visitor's object
   name. If someone else was already inside, it runs with `$2` = `Cue`,
   `$3` = `301`, `$4` = the newcomer's name. In Daventry only Connor sets
   traps off, so there it is always `On` with `$3` = `Connor`.
   **"Touches" (traced 2026-10-13):** found by the same swept test as
   walls (`connor-physics.md` "Collision shape"), using the visitor's own
   shape: for Connor, items and other ball stacks, each ball's path is
   tested against the trap's box widened by the ball's radius; for a box
   object, its box. It is tested **along the whole move**, so a fast
   move that passes right through a thin trap in one update still
   enters it (and the next move's stay test, rule 7, then finds him
   outside: `Off`). A visitor whose shape **starts inside** the box also
   counts as touching, at its next move.
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
    `KQObject::collision <trap> n` adds the bits of n when n is above 0
    (with or without a `+`), removes them when n is below 0, and refuses
    0 with a message; it works on the trap's live mask, the same for any
    bits (traced 2026-10-13, Q57 d).
    **An object already inside stays inside** (traced): removing a bit
    sends nothing at that moment and doesn't empty the list. The stay /
    leave test (rules 7–8) doesn't look at the mask, so when that object
    later leaves, the trap still sends its `Off` (or `Cue 302`). Only new
    entries need the bit. (`clifTrap.cs` has no `Off` branch, so for it
    this changes nothing.)

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| trap's own collision type | 0x10 | traced |
| `colliderMask` in trap data | 4 in 285 traps (all of Daventry); 0, 1, 2, 5, 6, 7, 516, 517, 519 elsewhere (rule 4) | data |
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

1. ~~Whether removing a trap's mask bit (rule 16) sends `Off` for whoever
   is inside.~~ Answered 2026-10-13: no; they stay inside and get the
   normal `Off` when they leave (rule 16).
2. ~~The exact contact test used for "touching" (rule 6).~~ Answered
   2026-10-13: the swept shape test, along the move (rule 6).
3. Whether an object placed inside a trap by `move` or `loadKQ`, without
   moving there, sets it off (rule 5).
