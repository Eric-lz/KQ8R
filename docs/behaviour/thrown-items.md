# Thrown items (the flight of a tossed or dropped item)

Study session A, 2026-10-20. Source: `Mask.exe` (studied, described here
in our own words) and Daventry's data. Answers Q137. Extends `items.md`
rule 12 (the launch) and `fighting.md` rule 31 (drops).

## Purpose

When a script runs `KQObject::tossItems` (a smashed barrel) or a character
dies with a drop list, each new item is **thrown**: it flies in a steep arc
out of the thrower, tumbling, and comes to rest where it first touches the
world. This note covers that flight, from the launch to the moment the
item lies still and is an ordinary item again.

## Rules

### Who flies

1. Every item made by `tossItems` and every item a dying character drops
   gets its own **thrower** (a special mover) for the flight. Both use the
   same thrower, with the same numbers. Items placed by
   `KQMonster::inventory world`, `dropItem` or a chest do not fly
   (`items.md` rule 5). Nor does a dying character's weapon whose name is
   a part of its die model: it is laid where the death leaves it
   (`fighting.md` rule 31a; thrown only if that spot fails a line test).
2. The item starts at the throw point (`tossItems`: the object's position
   raised by `offsetZ`; drops: `fighting.md` rule 31) with **no turn at
   all** (all three angles 0). This placement is an instant move, with no
   collision test.

### The launch (as `items.md` rule 12, now complete)

3. Each item draws its own random values, each uniform:
   - **heading** 0 to 6.28 radians (all round);
   - **elevation** 75° to 85° above the horizontal;
   - **launch speed** 5,000 to 6,500 units per second, split into a
     horizontal speed (speed × cos elevation, 435 to 1,682) and an upward
     speed (speed × sin elevation, 4,830 to 6,475);
   - **tumble** −0.5 to +0.5 radians **per update** (rule 9).
   The launch point is remembered, and a flight clock starts at 0.

### The flight, step by step

4. **When it moves:** once per game update (15 per second; the update's
   time step is 1/15 s of game time). Nothing finer.
5. **No speed is kept from update to update.** Each update the clock goes
   up by 1/15 s, and the wanted position is worked out **directly from the
   launch point** with the exact formula of a throw:
   - horizontal distance from the launch point = horizontal speed × t,
     along the heading;
   - height above the launch point = upward speed × t − ½ × g × t²,
     with **g = 8,085 units per second²** and t in seconds.
   So the item is always exactly on its arc (there is no "move first, then
   slow down" error to copy). The highest point is 1,440 to 2,590 units
   above the launch point (upward speed² / 2g), reached after 0.60 to
   0.80 s; back at launch height after 1.19 to 1.60 s, about 540 to 2,610
   units away (worked out from the formula).
6. **First step out of the thrower:** if the wanted position of an update
   is less than **25 units** from the launch point, the clock is moved on
   by another 1/15 s (repeatedly) until it is not. At the game's speeds
   the first step is already 300+ units, so this never happens in
   practice.
7. **The move itself is an ordinary move with collisions.** The item goes
   from where it is to the wanted position through the same three passes
   as every mover (`connor-physics.md` "One update of the mover"): straight
   up or down to the wanted height (the item's step up and step down are
   0), then across to the wanted x, y, then down. Gravity of the ordinary
   kind is **off** during the flight (the arc has its own). What can stop
   it is in rules 11–14.
8. **Going down:** from the first update in which the wanted height is
   lower than the height the item had at the start of that update, the
   item counts as **coming down**. This matters for rules 10 and 16.

### The tumble

9. Each update, **including after the first touch** (rule 12) until the
   flight ends (rule 16), the item is turned by its tumble angle about
   **its own x axis** (the model's left-right axis). Only that one axis;
   the angle is the same every update and is **not** scaled by time (it
   is per update, not per second). At 15 updates per second that is up to
   ±7.5 radians (±430°) per second; on average about 3.75 rad/s (215°/s).
   (B's stand-in turns all three axes at 90–180°/s; replace it.)
10. When the flight ends (rule 16), the item is set back to **no turn at
    all** (angles 0, 0, 0), wherever it is. This is the "normal resting
    pose" the user saw. It is not the turn it had before the throw (that
    was also 0 for every thrown item, rule 2).

### What the flight tests on the way

11. The item's collision shape is its `BOUNDS.KQ` ball stack (`items.md`
    rule 4), swept along each pass. With its mask (0x809) it meets:
    - **the floor grid** (`terrh2`, or the drawn ground in a world with no
      floor grid; `terrain.md`), in the up/down passes only, as for Connor
      ("The terrain in the passes"): it hits when it ends a pass below the
      floor grid at that x, y;
    - **buildings** (`KQInterior`, type bit 0x8): every wall, floor and
      roof of the building's collision;
    - **objects whose type has bit 0x1 or 0x100**: the default object type
      (0x41) and the barrels (0x141, `colliderType=321`). For an item every
      such touch is a hard obstacle (items have no script, so there is no
      `WorldCollision` call, `connor-collision.md` rule 12).
    It does **not** meet characters (type 0x42), Connor (type 4), other
    items, or objects with `colliderType=0` (e.g. the barrel's burst
    `brlupxcr.kq`): it flies straight through them.
    **Not tested at all:** an item type whose shape has no `BOUNDS.KQ` box
    has no collision shape, so it passes through everything (see "Edge
    cases").
12. **At a touch** (anything in rule 11), in this order:
    1. The ordinary collision answer moves the item for the rest of this
       update, as for any mover (`connor-physics.md` "Walls", the glancing
       case): it is put at the contact point; if the contact came in the
       up/down part of the move on a surface facing up (normal z > 0.8) it
       stays there; otherwise the rest of this update's movement is
       projected onto the surface and tried again (up to 3 times), so it
       **slides along** a wall for the rest of that 1/15 s. There is no
       head-on stop or crash for items.
    2. The item type's **land sound** plays at the item's place (full to
       5,000, silent at 25,000), **only the first time** the item touches
       anything in its whole flight (also on a wall, a barrel, a roof).
       This is the only place the land sound plays during a flight
       (`items.md` rule 7 covers an item picked up before any touch).
    3. Then it depends on **what** was touched:
       - an object that has a **script**, or any **animated object**
         (`KQAnimObject` and its kinds), whatever else it is: the item is
         **thrown again** from where it now is: new random heading,
         elevation, speed and tumble as in rule 3, clock back to 0, not
         "coming down", landed flag cleared. So it bounces off such
         objects, always steeply upward (75°–85°) in a random direction,
         with a fresh speed; nothing of the old speed is kept. Daventry's
         barrels (all have scripts) are like this.
       - anything else: **the floor grid, a building without a script**
         (the pub, `PUB.dig`, has none; Daventry's buildings have none),
         a plain object without a script: the arc **ends here**. The item
         is marked as landed and coming down, and ordinary gravity is
         switched on (rule 15).
13. **Over or inside a building (Q137 c2):** coming down onto a building's
    floor or roof is a touch of that building (rule 12): the item stops on
    the building's collision surface. It never falls through a building
    floor to the terrain under it. The terrain's floor grid is only met
    where no building or object is in the way.
14. **The thrower's own landing check:** each update, after working out
    the wanted position, the thrower looks up the **floor grid** height at
    that x, y. If the item is coming down and the wanted height is below
    it, the arc ends (as rule 12's "anything else"), even if no touch was
    reported. (No sound is made by this check itself; in practice the
    down pass then touches the floor grid and rule 12 plays it.)

### After the arc ends: settling

15. From then on the wanted position is **where the item already is**
    (the arc is no longer used; the flight clock still runs). The item
    only moves by the **ordinary gravity** of the movers (the same as
    Connor's falls, `connor-physics.md` "Gravity": the 0.06 timer and the
    measured drops 36, 108, 180, … per update), straight down, with the
    ordinary collisions, until something holds it. The tumble goes on
    meanwhile (rule 9). So an item that hit a building's wall **slides
    down from the hit point and drops straight down beside the wall**; an
    item that hit the floor stays where it touched.
    At the end of every move, the item is also lifted onto the floor grid
    if it ended below it (raise only, never lowered; the floor grid, not
    the drawn ground). So it can never come to rest below the floor grid.
16. **The flight ends** at the start of an update when either:
    - the item is coming down (rule 8) and its position has **not changed
      since the update before** (compared in whole units), i.e. it was
      held by something for a whole update; or
    - the flight clock has passed **10 seconds** (counted from the last
      throw or re-throw).
    Then: ordinary gravity off, the turn set to 0, 0, 0 (rule 10), and the
    item gets its ordinary mover back; it is an ordinary item lying in the
    world from now on (no more updates, no more falling). A thrown item
    whose flight has ended no longer counts as "thrown" for `items.md`
    rule 7.

### No bounce, slide or roll on the ground (Q137 d)

17. After touching the ground the item does not bounce, slide or roll. On
    walkable ground it stays at the contact point. On a steep slope
    (normal z ≤ 0.8) the ordinary mover rules apply: gravity pulls it down
    the slope as it would Connor (`connor-physics.md` "Slopes") until it is
    held for one update; then the flight ends.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| update rate of the flight | 15 per second, time step 1/15 s | traced |
| heading | uniform 0–6.28 rad | traced |
| elevation | uniform 75°–85° | traced |
| launch speed | uniform 5,000–6,500 units/s | traced |
| gravity of the arc | 8,085 units/s², height = vz·t − ½·g·t² from the launch point | traced |
| tumble | uniform −0.5 to +0.5 rad **per update**, about the item's own x axis | traced |
| first-step minimum | 25 units from the launch point | traced |
| longest flight | 10 s since the last (re)throw | traced |
| end test | coming down and unchanged position (whole units) for one update | traced |
| resting turn | 0, 0, 0 | traced |
| land sound | first touch only; full to 5,000, silent at 25,000 | traced |
| item mask / what it meets | 0x809: floor grid, buildings, objects with type bit 0x1 or 0x100 | traced |
| step up / step down of an item | 0 / 0 | traced |
| gravity after the arc | ordinary mover gravity (`connor-physics.md`) | traced; strength measured for Connor only |

## Data used

- `GAME/Kq/Invinfo.itl`: land sound, shape (for the `BOUNDS.KQ` box).
- `GAME/patch/BOUNDS.KQ`: the item's collision box (ball stack).
- Each world's floor grid (`terrh2.dib`) and buildings' `.dig` collision.
- `.kq` files of the objects it may touch: `colliderType`, `script`,
  `classType` (`KQAnimObject`, `KQAnimOnce`, ... count as animated).

## Edge cases

- **An item type with no `BOUNDS.KQ` box** (`INVITEM_TruthKey`,
  `INVITEM_MarbleTablet2`, `INVITEM_Orcbow`; `items.md` edge cases) has no
  collision shape: its flight tests nothing on the way (no walls, no
  buildings, and not even the floor grid in the passes). Only the
  thrower's own check (rule 14) ends the arc when it comes down below the
  floor grid, and the lift onto the floor grid (rule 15) puts it on the
  terrain, inside a building if it came down there. None of these three
  is thrown by Daventry's data.
- **The 10-second limit** ends the flight wherever the item is, even in
  the air (then it would stay hanging, with no gravity). Only possible for
  a fall of more than about 10 s after the last (re)throw, e.g. off a very
  high place; not seen in Daventry.
- **A re-throw inside a building** (an animated object indoors) throws it
  steeply upward again; the ceiling then stops it (a building touch, rule
  12), and it settles.
- Thrown from inside an object that blocks items (not the case for the
  barrels: the burst `brlupxcr.kq` / `brlupxms.kq` has `colliderType=0`):
  touches are only found while moving into a shape, so starting inside one
  is not a touch (`connor-collision.md` rule 1 and the ball-stack rules).
  Not traced further.
- When a re-throw happens in the up/down part of a move, the rest of that
  same update's move (the across part to the old wanted x, y) still runs;
  the new arc starts in the next update. Invisible at 1/15 s.
- Picking the item up while it flies: allowed (`fighting.md` rule 31); the
  thrower is dropped with it.

## Do not reproduce

- Nothing that changes what the player sees. The 10-second limit leaving
  an item in the air (edge cases) is a theoretical bug: if it is ever hit,
  switch ordinary gravity on instead.

## How to check

- **In our logs**, per thrown item and update: clock t, wanted and actual
  position, "coming down", every touch (what was touched: floor grid,
  building name, object name and whether it re-threw), when the land
  sound plays, and the end (reason: held / 10 s) with the final position.
  Expected: positions exactly on vz·t − ½·8,085·t²; landing 1.2–1.6 s after
  a throw on flat ground; the tumble angle growing by the same amount each
  update about one axis.
- **In the original:** break the crystal barrel south-west of the pub and
  watch:
  - each crystal turns end over end about one axis only, at a steady rate
    that differs between crystals (some almost still, some fast), and
    lies flat in its normal pose the moment it stops;
  - a crystal that flies against the pub's wall should slide down and
    drop at the foot of the wall, no farther out; one that hits a barrel
    is flung up again in a new direction;
  - a crystal coming down on a roof or floor stays up there.
  (Confirmed by the user 2026-10-20: at the pub's wall the items hit it
  and fall down.)

## Open questions

1. ~~The user's report of an item bouncing off the pub's wall.~~
   **Settled 2026-10-20 by the user's second look in the original:** the
   items hit the wall and then fall down, as rule 12 says (the arc ends
   at a building without a script; no bounce).
2. The user saw a "slow" tumble. The traced rate is up to 430°/s (average
   215°/s) about one axis. That may look slow because it is about one
   axis only and some items get a small angle; not sure.
3. Gravity after the arc ends (rule 15) was measured 4 times stronger than
   the code reads for Connor (`connor-physics.md` "Gravity"); assumed the
   same for items, not measured.
4. The bow spriggans' bolt (`fighting.md` open question 2) is moved by a
   different kind of object (a projectile), not by this thrower; not
   studied here.
