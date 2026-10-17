# Touches between Connor and objects (`ConnorCollision`, rolling rocks)

Study session A, 2026-10-13 (Q55, Q58). Sources: the objects that use
`conCollisionAction` and their scripts (`MASK/*/kq/`), Gnome's rolling
rocks (`rrock1–3.kq`, `rollside.cs`, `theroll.kqs`), `GAME/Kq/Conner.anm`,
`GAME/Common/C_rctbk.kqs`, and `Mask.exe` (studied, described here in our
own words).

## Purpose

When two collision shapes touch, each side may be **told** about it. An
object that is told about a touch with Connor can run its script with
`$2` = `ConnorCollision`; this is how the rolling rocks knock Connor back
and hurt him, how Daventry's waterfall makes Connor speak, how DeadCity's
scales notice him standing on them, and how two portals in Gnome and the
Swamp move him. This note also covers what the rocks' script then does
to Connor (the `Special` movement list, `setRelDamage`, `KQAttackEvent`)
and what happens when a ball-stack object meets another ball-stack
object, which `connor-physics.md` left open.

## Rules

### Who is told about a touch

1. Touches are only found **while something moves**: the moving shape is
   swept along its path (`connor-physics.md` "Collision shape"). Nothing is
   found between two shapes that both stand still.
2. A rough check lets a pair through when either side's mask covers the
   other's type. But **being told is one-sided**: of the two, an object is
   told about the touch only if **its own mask** covers the **other's type**.
   So the mover may be told, the other may be told, both, or neither.
3. An object that is told reacts by the other's type bits, looked at in
   this order; the first bit that is set decides:

   | Other's type has bit | A plain object (`KQObject`, `KQAnimObject`, doors …) |
   | --- | --- |
   | 0x100 | an obstacle for its own move |
   | 0x8 | an obstacle for its own move |
   | 0x1 | a **world touch** (rule 12) |
   | 0x2 | an obstacle for its own move |
   | 0x4 (Connor's type) | a **Connor touch** (rules 5–11) |
   | 0x2000 | a world touch (rule 12) |
   | none of these (e.g. a trap, 0x10) | nothing |

   "Obstacle" means it is handled as a wall for the object's own move
   (stop / slide, as in `connor-physics.md`). Connor's own type is just 4,
   so for any object he is a "Connor touch", never an obstacle.
4. **Connor** is told when his mask (0x281B) covers the other's type; for
   type-1 things and the others in his mask he reacts as a wall
   (`connor-physics.md`). **Except bit 0x100 (pushable)**, which comes
   first: for Connor it goes through the push check of `pushing.md`
   (rules 1–3) and is only a wall when that check fails. He ignores type 0x10 (traps; see `traps.md`
   rule 4).

### What happens when a moving object runs into Connor

5. **The object is not stopped and Connor is not pushed.** A moving
   object only ever turns a touch with Connor into a Connor touch (rule
   3), never into an obstacle, so it carries on along its path. Connor is
   told too (for a type-1 object), and the touch is noted as a wall for
   him, but that note is thrown away at the start of his own next move;
   it changes nothing. The object simply passes into and through him.
   Any effect on Connor comes from the object's script.
6. **Overlapping afterwards (ball stacks, rules 17–19):** while the object
   overlaps him, its further moves find no new touch with Connor (a ball
   that starts inside Connor's shape is ignored). Connor's own moves while
   he overlaps a type-1 ball-stack object (like a rolling rock) count as
   hitting it at his starting point, so **he cannot move until it has
   moved off him** (follows from the code; not checked in the game).

### The `ConnorCollision` call

7. **Needs all three:** the object has a `script`, `conCollisionAction` is
   not 0, and the object's own `colliderMask` includes 4 (so that it is
   told, rule 2). The touch can come from either side moving: Connor
   walking, running or falling into it, or the object moving into him.
   `bricks.kq` (Daventry) has `conCollisionAction=1` but its
   `colliderMask=4` is commented out, so it is never told; its script has
   no `ConnorCollision` branch anyway. (`urn.kq` has both commented out.)
8. **Arguments:** `$1` = the object's name, `$2` = `ConnorCollision`,
   `$3` = a **direction number** 1–6 (rule 9). The script runs at the
   moment of the touch, in the middle of the move that found it.
9. **The direction number** is worked out from the touch's surface
   direction (made unit length):
   - pointing upward at all (any z above 0): **1**;
   - pointing downward at all: **6**;
   - exactly level (z = 0, the usual case for a side touch): by its x and
     y: x above 0 gives **2**, or **3** if |x| < |y|; x below 0 gives
     **4**, or **5** if |x| < |y|; x exactly 0 gives 3 → 4 for y above 0
     and 5 → 6 for y below 0 (see "Do not reproduce").

   When **Connor moves onto or into a still object**, the direction points
   from the object toward Connor, so **1 means he came down on its top**.
   The data agrees: the DeadCity scales (`scalelt.cs`, `scalert.cs`),
   `heart.cs` and the Swamp tiles (`fienfire.cs`: "if landed on top")
   test `$3 == 1`. When the **object moves into Connor**, the direction
   points from Connor toward the object, the other way round (from the
   code; not checked). Whether a turned object's numbers use the world's
   axes or the object's own: not settled. No script tests any number
   other than 1.
10. **Repeat limit, 0.2 s per direction.** Each object keeps, for each of
    the six numbers, the game time of its last Connor touch with that
    number (all start at 0). A touch runs the script only if **more than
    0.2 s** have passed since the last touch with the same number; every
    touch, whether it ran the script or not, sets that time to now. So
    while the contact goes on (a touch found every update, e.g. Connor
    standing on a scale or pushing against a wall-like object), the
    script runs **once, at the start**, and again only after the touches
    in that direction have stopped for more than 0.2 s. The six numbers
    are independent: landing on top (1) and then walking into the side
    (2–5) can run it twice.
11. Being told doesn't change how Connor himself reacts: the object stays
    solid for him if his mask covers its type (the rocks, type 1: yes).

### `WorldCollision` (for completeness)

12. The same for touches with type-1 or type-0x2000 things, using
    `worldCollisionAction` instead: script call with `$2` =
    `WorldCollision`, `$3` = direction number, same 0.2 s limit (its own
    six clocks). With `worldCollisionAction` on, such touches are **no
    longer obstacles** for the object's own move (it passes through
    walls). Without it, they are obstacles as usual. No loose `.kq` file
    sets `worldCollisionAction`.

### What `rollside.cs` does to Connor

`rollside.cs` on `ConnorCollision`: `KQMonster::setState Connor Special
'reactback'`, `KQConner::setRelDamage 0.2`, `sendEvent 0 Connor
KQAttackEvent $1 0 $CurRelHealth 1`.

13. **`KQMonster::setState Connor Special '<list>'`**: Connor's `Special`
    state takes a comma-separated list of movement names and plays them
    **one after the other, each once**. When the last one ends he goes
    back to standing (Wait); the movement keys then work again as usual.
    A few special words can stand in the list: `KillConnor` (he dies on
    the spot), `Inv`, `AnimList=`, `AnimListOn`, `AnimListOff` (not
    studied; not used by these scripts). Whether the request is accepted
    depends on Connor's current state (the state-change rules,
    `connor-physics.md`; for `Special` from each state: not traced).
14. **`reactback`** is `Conner.anm` movement 34: `c_rctbk.kqs`, 30 frames,
    **1.933 s**, plays `conrowbk.aud` at its start, not looping. Its
    whole-body steps (data) are all along the model's +Y axis, i.e.
    **backwards** (the model faces −Y, `connor-movement.md`), **903 units
    in total**, 463 of them in frames 4–7 (about 0.27 s). He moves
    through his mover as usual, so walls (and a rock he overlaps, rule 6)
    stop him.
15. **`KQConner::setRelDamage r`** does not hurt Connor. It only sets two
    console variables from Connor's **level** (his experience level,
    starting at 1; his full health is 10 × level points):
    `CurRelDamage` = −(r × 10 × level) and `CurRelHealth` =
    +(r × 10 × level), each cut to a whole number toward zero. With 0.2:
    level 1 → −2 and 2; level 5 → −10 and 10. (The usage text calls `r`
    "0 to 100% of Connor's current maximum health".) `rollside.cs`'s
    `echo $CurRelDamage` only prints it.
16. **`KQAttackEvent`**, text `<attacker name> <kind> <damage> <flag>`:
    sent to Connor, it hurts him (rule 17). Kind is the weapon type
    (0 normal, 1 projectile, 2 fire, 3 ice, the same numbers as an
    object's `Die` reasons); it is remembered as the last thing that hit
    him. The flag is not used by the hurt rule traced here. The rocks
    send kind 0, damage `CurRelHealth` (2 × level), flag 1.
17. **Connor being hurt** (the shared hurt rule; full fighting is M5):
    - ignored while he is dead or dying, and in one other state (not
      identified);
    - damage 0 is ignored for health;
    - the hit is damage × a random factor between 0.25 and 1.0, rounded
      to the nearest whole number;
    - his armour (the 4th number of `stats=`, 0 in `Connor.kq`; armour
      items and spells change it, M5) is multiplied by a random factor
      between 0.75 and 1.25, rounded, and capped at 100; armour exactly
      100 before this means no loss at all;
    - the loss is hit × (100 − armour) / 100, cut to a whole number, but
      **at least 1**; it comes off his health;
    - health **below 0** → he dies;
    - his hurt sound may play (not traced in detail);
    - a few one-time help messages can appear (M5, not traced).

    Example, level 1 (10 health points), no armour, a rock (damage 2):
    the hit is 2 × (0.25 … 1.0) = 0.5 … 2.0, rounded: he loses **1 point
    two times out of three, 2 points one time out of three** (10% or 20%
    of his health per hit).

### Ball stack against ball stack (closes the gap in `connor-physics.md`)

Ball stacks are Connor, monsters and any object with `collisionShape=1`
(the rolling rocks: box ±900 × ±900 × 0–1,750 → one ball, radius 900,
centre 900 above the object's position).

18. When a ball-stack object moves and the other is also a ball stack,
    **each of the mover's balls** (its centre's path from old to new
    position) is tested against an **upright cylinder** around the other
    object, standing on the other's position. The cylinder is widened by
    the moving ball's radius sideways and at both ends (its exact radius
    and height come from the other object's own shape; the parts were not
    pinned down: likely its ball radius and its box height). The ball
    stacks don't turn with the objects. The nearest hit along the path
    counts.
    - Hit on the side: surface direction level, pointing from the
      cylinder's axis out through the hit point.
    - Hit on the top or bottom: straight up or straight down.
    - A move shorter than about 0.000001 in every axis is not tested.
19. **A ball that starts inside the cylinder:** if the other object's type
    has bit 2 or bit 4 (creatures and Connor), that is **no hit**, so the
    mover can move out (or on through). Otherwise (e.g. a rock, type 1)
    it is a hit right at the start point, with the surface direction
    along the movement; for Connor this ends as "stay where you are"
    (`connor-physics.md` slide rules: nothing left to slide). Projectiles
    always count a start inside as a hit.

### Other script calls used by the rocks

20. **`makeActive <name> 0|1`** sets an "active" switch on the named object
    (the same switch a saved game stores as `active3DFlag`); unknown
    names print "Could not find object". What the switch changes was
    **not traced**. Note: the `isActive3D=1` key in some `.kq` files
    (e.g. `waterfal.kq`) is not a key the game reads at all.

### Order inside one update

21. Not traced. Since a touch is found during whichever object's move
    reaches the other, running objects' moves after Connor's (as B does)
    only shifts a touch by at most one update.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| repeat limit per direction number | more than 0.2 s of game time | traced |
| direction numbers | 1 up … 6 down (rule 9) | traced |
| `reactback` | `c_rctbk.kqs`, 30 frames, 1.933 s, 903 units backwards | data |
| `setRelDamage` result | r × 10 × level, toward zero | traced |
| Connor's full health | 10 × level points (level starts at 1) | traced |
| hit random factor | 0.25 … 1.0, rounded | traced |
| armour random factor | 0.75 … 1.25, rounded, cap 100 | traced |
| minimum loss | 1 point | traced |
| rolling rock shape | one ball, radius 900, centre at z 900 | data + `connor-physics.md` ball rule |

## Data used

- `.kq` keys: `conCollisionAction`, `worldCollisionAction`, `colliderType`,
  `colliderMask`, `collisionShape`, `script`.
- Objects with `conCollisionAction=1`: Daventry `bricks` (never told,
  rule 7), `waterfal`; DeadCity `eplrdie`, `heart`, `scalelt`, `scalert`;
  Gnome `poolport`, `rrock1–3`; Swamp `swpgate5`.
- Scripts: `rollside.cs`, `waterfal.cs`, `scalelt.cs`, `scalert.cs`,
  `heart.cs`, `eplrdie.cs`, `poolport.cs`, `swpgate5.cs`, `fienfire.cs`
  (and `fienfir2/3.cs`).
- `GAME/Kq/Conner.anm` (movement 34), `GAME/Common/C_rctbk.kqs`.
- `GAME/Kq/Connor.kq` `stats=` (armour is the 4th number).

## Edge cases

- Daventry's waterfall (`waterfal`, type 33 = 0x21, mask 4): walking into
  it makes Connor say his line once; pressing on against it doesn't
  repeat it until he has been out of touch (in that direction) for more
  than 0.2 s. It stays solid.
- A rock that rolls back and forth over a spot: it runs the script each
  time it **enters** Connor's shape from outside, once per pass.
- Connor inside a rock (it rolled over him and he is stuck, rule 6): each
  of his blocked moves is a touch for the rock with a direction from his
  movement, which may be a different number from the rock's own entry
  touch; so a second hit soon after the first is possible (from the
  code; not checked).
- In the first 0.2 s of game time no `ConnorCollision` can run (the
  clocks start at 0). Harmless.
- The `ConnorCollision` script runs before the move that found the touch
  has finished; a script that moves Connor (`poolport.cs`, `swpgate5.cs`:
  `move Connor …`) does so mid-move. What the rest of that move then does:
  not traced.

## Do not reproduce

- **The x = 0 case of the direction number** (3/5 bumped to 4/6, so a
  purely sideways touch along y can come out as 6, "bottom"), and the
  fact that the sign of y is otherwise ignored, are quirks. No script
  depends on them (only `$3 == 1` is tested); keep 1 = upward and
  6 = downward exactly, the side numbers may be computed the same way
  for compatibility or more sensibly.
- **Getting stuck inside a type-1 ball-stack object** (rule 6, 19) is
  probably unintended. Recommended: reproduce it first (so the rocks feel
  the same), log it, and offer a fix later as an option.

## How to check

- **In our logs:** every Connor touch: object, direction number, game time,
  whether the 0.2 s limit let it through, and who moved. For the rocks:
  one `ConnorCollision` per pass over Connor, then `reactback` for 29
  updates at 15 per second (1.933 s), health before/after.
- **In the original (user):** stand Connor in a rolling rock's path at
  level 1 with no armour; each hit should take 1 point (sometimes 2) of
  his 10, i.e. 10% (sometimes 20%) of the health bar, and push him back
  about 900 units (roughly 70% of his own height of 1,300). After the
  stumble he should be able to walk again. Also: does he get stuck if a rock rolls over him (rule 6)?
- **Waterfall (Daventry, about 158,442, 302,482):** walk into it: one
  remark; keep pushing: no repeat; step away and back: remark again.

## Open questions

1. What `makeActive`'s switch does (rule 20).
2. The exact cylinder size in rule 18.
3. Which axes the direction number uses for a turned object, and the
   direction when the object is the mover (rule 9), are from the code
   only.
4. Whether `Special` can be entered from every state (rule 13).
5. Order of objects' and Connor's moves within an update (rule 21).
