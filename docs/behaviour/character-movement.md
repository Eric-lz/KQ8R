# Characters moving on their own: walking, wandering, idling, noticing Connor

Study session A, 2026-10-19. Traced in `Mask.exe` unless marked "data"
(read from the game's files), "inferred" or "not sure". Stops where an
attack starts (the fight is the next study). Answers Q128 and Q129
(`QUESTIONS.md`); corrects `characters.md` rules 8, 29–31 and open
question 3 (said there too). **Updated 2026-10-20 (A)**: answers Q132,
Q133 and Q134 (rules 7, 18, 21, 25, 27, 28, 30 corrected, open
questions 1–3 settled) and Q131 (in `connor-collision.md` rule 18).

## Purpose

How a character (`KQMonster`, `characters.md` rule 1) moves without a
script telling it to: how it walks, wanders between points, fidgets,
notices Connor, chases him, gives up and goes home, and how bats fly.
Enough for B to make Daventry's chickens, goblins, spriggans, zombies,
bats and the pig behave as in the original up to the first swing.

Builds on: `characters.md` (states, start-up, Wait's return to `Stop`,
the turn toward a point, near / far / off), `connor-movement.md` (speed
from the animation, the automatic turn), `connor-physics.md` and
`connor-collision.md` (the mover, passes, touches), `anim-scripts.md`
(movement requests), `objects.md` (ground heights −1 / −2).

Words used here:

- **Update**: one game tick, 15 per second.
- **Active**, **alarmed**: see rules 20 and 23. An active character is
  one on the game's short list of characters that "know" about Connor.
- **Target**: the one a character is after once alarmed. Always Connor
  in this note.
- **Home**: the character's home point (`setHome`, rule 13).
- **Steering**: the character asking the automatic turn
  (`characters.md` rule 23) to face a point.

## Rules

### 1. Walking

1. **The same mover and animation path as Connor** (traced). A
   character moves only by the per-frame steps of the movement it is
   playing, exactly as Connor does (`connor-movement.md` "Forward speed
   comes from the animation"). The AI never pushes it along; it only
   picks a movement (`walk` or `run`) and steers. So **its speed comes
   from its `walk` or `run` model's frame steps and loop length**, times
   the `loop=` speed (1.0 for all of Daventry's). There is no speed key
   per character or per type. **Corrected 2026-10-24 (Q164, traced):**
   not "distance per loop ÷ loop length" but (sum of steps + the last
   frame's step) × (N − 1)/(N × loop length), N = frames
   (`objects.md` "Loops that move the whole object" rules 6 and 6a);
   within about 1 % of the old figure for most, up to 7 % for the
   spriggan's walk.
2. **Daventry's speeds** (data, from the models' frame steps by the
   corrected rule 1, which gives Connor's walk 1,124 and run 2,558;
   the old "sum ÷ length" figure in brackets where it differs):

   | Character (list) | `walk` | `run` |
   | --- | --- | --- |
   | chicken (`chicken.anm`) | 119 units/s (`Chk_wlk1`) | 947 (`chk_run`) |
   | bone goblin (`goblin2.anm`) | 926 [917] (`g2_walk`) | 3,106 [3,068] (`g2_run`) |
   | goblin (`goblin.anm`, `gobcon1`) | 3,106 [3,068] (`g-run`!) | 3,040 [3,109] (`g-walk4`!) |
   | spriggan (`sprigan.anm`) | 1,708 [1,589] (`spwalk`) | 3,365 (`Sprun`) |
   | zombie (`zombie.anm`) | 503 [511] (`z-walk`) | no `run` |
   | bat (`bat.anm`) | 1,159 (`batfly`) | no `run` |
   | pig (`piggy.anm`) | 437 (`pigwalk`) | 437 (same model) |
   | Knight, Henchman, bow spriggan | 0 (their `walk` is a standing model) | none |

   `goblin.anm` names `g-walk4` under `run` and `g-run` under `walk`
   (looks swapped in the file); both are fast. Keep the data as it is.
3. **Which movement** (traced): wandering always asks for **`walk`**.
   When a character is alarmed (rule 23) it tosses a coin: **half the
   time `run`, half the time `walk`**, and it tosses again every time it
   starts moving after Connor again from standing. After it has been hit
   twice it always runs; one hit is enough while it can see Connor
   (`fighting.md` rules 26–27). Going home after leaving its
   chase area, and the "stuck" escape (rule 7), use `run`. Taking up the
   chase again while moving home (rule 28) tosses no coin: it keeps the
   movement it has (usually `run`). A list without `run`
   uses `walk`; a list without `walk` but with `run` uses `run`.
4. **Starting and ending a walk** (traced): entering the moving state
   first asks for `Stop` at a random frame and speed (as Wait does,
   `characters.md` rule 13), then at once for the walk or run movement.
   Both are ordinary requests (`anim-scripts.md` rule 4), so the walk
   replaces `Stop` straight away (`stop` can be cut short in every
   Daventry list). When the walk or run movement ends it is simply asked
   for again (they loop anyway). There are **no start, stop or turn
   movements** for characters: no transitions are used here, and the
   `walk` / `run` pair is all there is.
5. **Turning** (traced): steering uses the automatic turn of
   `characters.md` rule 23: **3.5 rad/s, while the walk goes on**, so a
   walking character turns in an arc (at 926 units/s and 3.5 rad/s a
   walking bone goblin turns on a circle of about 260 radius; inferred). The turn
   aims at the point anew in every update until it faces it, then the
   character walks straight. No banking (Connor's lean needs his keys).
   `noTurn=1` blocks it. Standing characters turn on the spot, with no
   turning movement.
6. **Steps, slopes, walls, water, edges** (traced; the mover is
   Connor's, `connor-physics.md`):
   - steps up and down of 325; the walkable-slope limit of Connor's
     mover (changeable with `KQMonster::walkSlope`, Do not reproduce);
   - **walls**: a character's own move uses the ordinary slide of an
     animated object: in the up pass it stops at the contact; in a down
     pass it lands on a surface that faces up enough (more than 0.8 of
     straight up); in the across pass it **slides along** the wall
     (a surface tilted less than 0.7 toward up counts as an upright
     wall), re-tries the rest of the move, and after 3 slides in a row it
     stays where it touched. Connor's extras (the head-on crash, the wall
     blocks, the push-off from a monster's top) are **not** used;
   - **no Fall state, no fall death** (`characters.md` rule 8): a walk off
     an edge just falls with gravity and lands;
   - **water**: `keepOnGround` decides (`characters.md` rule 8): the
     default walks along the floor under the water; `keepOnGround=1`
     stays on the surface. No wading movements: they keep playing `walk`
     / `run` (not traced further);
   - **no path finding, no edge or water avoidance**: the AI only steers
     straight at a point (rules 9, 24, 27) and uses the stuck escape
     (rule 7). Nothing tests ahead for drops, water or buildings.
7. **The stuck escape** (traced; answers Q132 (a), (b)): the check is
   switched on only once its `walk` or `run` is **actually playing**. If
   the request has to wait (behind a movement that can't be cut short,
   such as a zombie's `birth`), the check starts when the walk starts.
   One second after that, and then every half second, the character
   compares where it is with where it was at the last check. If it has
   moved **less than 200 units** (in 3D) since then:
   - it stops steering for **3 seconds**;
   - it tests the four **world** diagonals, not ones relative to its
     heading, in this order: **north-east** (+x, +y), **north-west**
     (−x, +y), **south-west** (−x, −y), **south-east** (+x, −y), i.e.
     anticlockwise seen from above. Each test is a **level line from its
     feet** (its position; both ends at that height) to a point 1,250
     out in x and 1,250 in y (so about 1,770 long). The first one tried
     is north-east the first time it is stuck after it went into its
     moving state; each further time it is stuck while it stays moving,
     the first one tried is one further on. Every new start of moving
     (from standing) starts again at north-east;
   - a diagonal is blocked by the **drawn ground** (see Do not
     reproduce for the exact test at foot height) or by the box of
     anything solid it runs into: plain objects, **other characters**
     and buildings (types with bit 0x1, 0x2 or 0x8). **Connor never
     blocks it.** Its own box never counts, nor does any box the line
     starts inside;
   - it **turns at once** to face along the first free diagonal, outward
     (no turning time: a snap). If all four are blocked it snaps to face
     **north (+y)**, a fixed world direction, whatever way it faced
     before (answers open question 1);
   - a wanderer moves on to its **next wander point** (rule 11);
   - it switches to **`run`**.
   (Logged by the original as "`<name> is stuck`" to the debugger, which
   confirms the reading.) **Consequence for the chickens** (inferred from
   the numbers): their walk is 119 units/s, about 60 per half second, so
   they are "stuck" at the first check every time they start walking and
   from then on run (947 units/s) round their points.
8. **Where it heads for**: only its current wander point (rule 11), Connor
   (or where it last saw him, rule 26), or its home (rule 28). Points are
   aimed at in x and y only; heights don't matter.

### 2. Wandering

9. **Who wanders** (traced and data): a character wanders only if its
   `wanders` is on **and it is active** (rule 20). `wanders` comes from
   the monster list's movement field when the character comes from
   `monsters.kq` (it replaces the `.kq` key): `none` → off; `@` → the
   type's field in `Monstats.mtl` (every Daventry type says `none`);
   anything else → on. A `.kq`-loaded character without the key has it
   on (`characters.md` rule 31: Daventry's people all say `wanders=0`).
10. **Where the points come from** (traced):
    - **`loop n x y z …`**: the n points in order; after the last it goes
      back to the first.
    - **`patrol n x y z …`**: the n points, then the same points back
      from the second-last to the first (`A B C D` → `A B C D C B A`),
      then round again (so it passes `A` twice in a row: the second time
      it is reached at once).
    - Only the first n triples are read; any more numbers in the field
      are ignored (data: `goblin20` says `patrol 2` and lists three
      points: it patrols between the first two).
    - z of exactly −1 in a point means the floor height; other values
      (Daventry's −2) are kept as written. They don't matter: points are
      used in x and y only.
    - **`random`**, or no points at all (and no `Birth` movement): when
      it enters the world the character gets four points: **its own
      position, then three random points** around it, each at a random
      distance between **2 and 3 times its alarm radius** and a random
      direction (each new direction at least about 0.79 rad on from the
      previous one), on the ground. No Daventry character uses this (its
      wanderers all list their points; data).
    - Daventry's lists all start with the character's own position, so
      it "reaches" point 1 at once and turns toward point 2 (data).
11. **Following the points** (traced): the character starts walking in
    whatever direction it faces. In every update it measures its distance
    to the current point (x and y only):
    - within **250**: the point counts as reached; it takes the next one
      and **turns toward it** (rule 5);
    - otherwise it remembers the closest it has been; it **re-aims** only
      if it has drifted well away: when the square of its distance is
      more than 12,500² above the square of its closest distance so far
      (so, in practice, only after being pushed far off course). Normally
      it turns once per point and walks straight.
    - `wanderAndStop=1` (no Daventry file has it): at the end of the list
      it stops wandering for good instead of starting over, and sends a
      cue (to whom: not traced).
12. **How long it waits** (traced): a wanderer **without** an idle list
    (rule 15) never stops: chickens, bats and `gobcon1` walk (or run)
    their points for as long as they are active. A wanderer **with** an
    idle list stops whenever its idle time comes (rule 17), plays one
    fidget standing, and walks on at once; the next fidget is due 2–12 s
    after that one ended. So it walks for 2–12 s, fidgets, walks again.
13. **How far from home** (traced): wandering has **no distance limit**:
    the points are the limit. The home point (`setHome`, the monster
    list's position, a `.kq` file's `startLoc`) is used only for the
    chase leash (rule 27), the return (rule 28) and the reset (rule 21).
    `KQMonster::setHome <name> x y z` sets it (z −1 = floor) and never
    moves the character (`characters.md` rule 5).
14. **When it stops wandering** (traced): when it is alarmed (rule 23),
    when its idle time comes (rule 12), when `goHome` is on (rule 29),
    when it stops being active (rule 21: it is no longer updated anyway),
    or when `wanderAndStop` ends the list.

### 3. Idling

15. **`idleAnimFile`** (data and traced): a second movement list of
    fidgets (Daventry: `goblin2.kq` → `gob2idle.anm`, `sprigan.kq` and
    `sprigan2.kq` → `sprgidle.anm`; both lists hold `attack` and `react`,
    so the fidget is a swing at the air or a flinch). The chickens' files
    have the key commented out (`#idleanim=`): they never fidget.
16. **When** (traced): only an **active** character in **Wait**, not
    alarmed, and only when its idle time has come. The idle time starts
    at 0, so the **first fidget comes as soon as it is active**. A
    wanderer that is walking goes back to Wait when its idle time comes,
    and fidgets there.
17. **How often** (traced): when a fidget ends, the next is due **2 s
    plus a random whole number of seconds from 0 to 10** later (0 and 10
    half as likely as the others: a random number from 0 to 10, rounded).
    Any movement that ends while a fidget is under way counts as its end.
18. **Which one, and the list swap** (traced; answers open question 3):
    the idle list file is read (the game keeps it once loaded) and one
    of its movements is picked at random, all equally likely. For that
    fidget the idle list **takes the place of the character's own list**
    as its current movement list, and the movement is asked for from it
    as an ordinary request. The character's **own list is put back as
    soon as any movement ends** (the fidget's end, or the end of
    whatever cut it short), and also whenever its state asks for a new
    movement in the usual way (e.g. `walk` when it starts moving). When
    the fidget ends, Wait asks for `Stop` (from its own list, back by
    then) at a random frame and speed (`characters.md` rule 18).
    - Side effect (traced): when a movement **named `attack`** (or
      `attack2`) ends, its next attack becomes due `attackDelay` later
      and its chase timer (rule 23) is set to 5 s; when one **named
      `react`** ends, its chase timer is set to 5 s (and the 2 s pause
      before it can flinch again, `fighting.md` rule 23). This happens for Daventry's fidgets too, whose
      names are `attack` and `react`. Harmless before the fight: the
      chase timer only matters with a target.
19. Characters without an idle list never fidget; their `Stop` loops.

### 4. Noticing Connor

20. **Becoming active** (traced; refines `characters.md` rule 29):
    - Every 15th update (once a second) a character that is updated
      (near or far, not hidden) checks whether it **can see Connor**: a
      straight line from its eye to a point 1,250 above Connor's
      position. **The eye is at the top of its `BOUNDS.KQ` box** (the
      box found by `characters.md` rule 9), not 1,000 above its feet:
      the 1,000 default (and the `lineOfSightOffset` key) is replaced by
      the box top when the character enters the world. Daventry: chicken
      400, pig 490, bone goblin and spriggan 1,300, zombie 1,750, bat
      2,030 (data). **Correction of `characters.md` rule 29.**
    - The line is blocked by the **drawn ground** (the water surface
      included), **plain objects** and **buildings**, but **not by other
      characters or Connor** (the test looks for types 1 and 8; characters
      are 0x42, Connor 4). The character itself never blocks it. **No
      distance limit.** (Correction 2026-10-27, from the data, Q170:
      characters whose file sets `colliderType` with bit 0x1, such as
      `basilisk`, `sprigan2`, `shadow` and the 0xC021 talking
      characters, **do** block other characters' lines.) `canAlwaysSeeConner=1` skips the test (always
      sees him).
    - In Wait, a character that is not active and can see Connor joins
      the **active list**, if there is room: at most
      `KQWorld::MaxMonsters` (Daventry's `world.cs` sets **6**; Ice World
      10). A character with a `team` (Daventry: `windguys`, `ruinguards`)
      may take the place of one without a team. The rest stay inactive
      and try again each update.
    - Chickens and the pig (alarm radius 0) become active too and use up
      places (data and traced).
    - Between checks the character keeps the last result. It counts the
      updates since it last saw Connor (used in rule 26).
21. **Losing active** (traced, new): when a character goes **off**
    (neither near nor far, `object-lighting.md` rules 34–37), or is
    pushed off the list by a team member, it is taken off the active
    list and forgets its target. Then, if it was on its way home, or if
    Connor is more than **twice its alarm radius** away (the straight 3D
    distance between Connor's current position and its own; traced,
    answers open question 2), it is **put back on its home point at
    once** (z −1 = floor, −2 = surface), and a character
    whose list has `hide` (the zombies) goes back underground (Birth). A
    moving character switches to Wait. **With an alarm radius of 0 this
    always happens**: chickens and bats are back at their home point
    whenever Connor comes back after they went out of range.
22. **No hearing** (traced): nothing in the noticing code uses sounds,
    Connor's speed or his weapon. A character notices Connor by sight
    and distance (rule 23), by being told (rule 24) or by being hit (the
    fight study).
23. **Alarm** (traced): an active character that is not idling, in Wait
    or walking its points, is **alarmed** when it **can see Connor and is
    closer than its alarm radius** (straight distance between their
    positions) and `goHome` is off. Then:
    - its target becomes Connor; it remembers where he is;
    - it **waits a random 0 to 1 second** before it reacts (in that time
      it carries on as before);
    - its **chase timer** is set to 5 seconds (10 when it was told to
      run, i.e. after being hit);
    - it says its alarm line once (`AlarmMsg`, `AlarmNoun`, `AlarmVerb`,
      `AlarmCase`, `AlarmSeq`, `AlarmEnd` in its file; no Daventry file
      has them, data), else some types outside Daventry (skeletons, orcs,
      dwarves, henchmen) may shout a random battle cry;
    - its walk / run coin is tossed (rule 3);
    - the alarm spreads (rule 24).
    **Alarm radius**: the monster list's field, or the type's for `@`
    (it replaces the `.kq` key: `goblin2.kq`'s 1,500 is never used for
    the listed goblins, data). Scripts change it with
    `KQMonster::setAlarmRadius` (Daventry: the graveyard trap, rule 30).
    A radius of 0 means never alarmed.
24. **Telling the others** (traced): when one character is alarmed,
    every **active** character with an alarm radius above 0 that **can
    see Connor** (at any distance) or whose team has a member that can
    see him is alarmed too; and the alarmed one's team members (made
    active if needed) with an alarm radius above 0 are alarmed. Being
    told counts as an alarm (rule 23) without the distance test.
25. **What it does then** (traced; the fight starts at the end; answers
    Q133 (b)):
    - **Standing (Wait)**: it turns to face Connor (or, while it can't
      see him, the spot where it last saw him, if that is more than 500
      away), on the spot. If Connor is out of sight or farther than its
      attack distance (`doMode=1`: 1.25 × its attack distance), and its
      chase radius is above 0, it starts moving after him (rule 3 for
      walk or run). While he is in view and within that distance it
      stays, turning to him, and **attacks once its attack time has
      come, whatever Connor is doing** (**next study**). A character with
      a chase radius of 0 (`guard`, `none`) **never moves**: it only
      turns to follow Connor and fights if he comes within reach.
    - **Moving**: in every update it steers at Connor (or the last seen
      spot, as above). When it can see him and he is within its attack
      distance (`attackDist`, default 1,250; Daventry's list mostly `@`):
      - once its attack time has come, the attack starts, whatever
        Connor is doing (**next study**);
      - before that, a melee character **stops** (Wait, then turns to
        face him) only when **Connor has no forward or backward movement
        input** at that moment: his forward / back key is not held (he
        stands, or only turns on the spot). A walk or run forward, or
        walking backward, counts as moving. A scripted walk of Connor
        sends no key, so it counts as not moving. While he is moving it
        keeps steering at him and walking or running, so it **follows a
        walking Connor at reach** instead of stopping and starting;
      - `doMode=1` (ranged) characters stop when within 0.75 × their
        attack distance, whatever Connor does; they may also back away
        (fight study).
    - When its next attack is due: `fighting.md` rules 6–11 (the first
      one is due at once).
26. **Losing sight** (traced): once it has not seen Connor for **45
    updates (3 seconds)**, it gives up (rule 28), unless it has a team and
    a team member can see him (then the count starts again). While
    standing the exception is slightly different: it has a team and a
    chase radius above 0 (no team member needs to see him).
27. **The leash** (traced): once the chase timer (rule 23) has run out,
    in every update:
    - with no team, if Connor is more than **twice its alarm radius**
      away, it gives up;
    - if its chase radius is above 0 and it is farther than that from its
      **home** (3D), it switches to `run` and gives up.
    Standing (Wait), the first test is slightly wider: it gives up beyond
    twice its alarm radius when it has no team **or** a chase radius of
    0 (traced). Each time it takes up the chase again (rule 28) the chase
    timer starts anew at 5 s, so the leash is tested again only 5 s
    later.
    `chase a b` in the monster list sets **home radius a** and **chase
    radius b** (traced); `guard` or `none` set both to 0; `@` takes the
    type's. Scripts: `KQMonster::setChaseRadius`, `setHomeRadius`.
28. **Giving up and going home** (traced; answers Q133 (a)):
    - it forgets its target and marks itself "on the way home";
    - **a character whose list has `hide`** (the zombies) goes back into
      the ground instead (rule 30);
    - otherwise it turns toward its home point. **Moving**, it steers
      toward home in every update and keeps moving. **Standing**, it
      starts moving home at a `run` at its next update; a character with
      a chase radius of 0 instead stays where it is and is no longer "on
      its way home". When it is within **100 in x and in y** of home, it
      stops (Wait) and its hit count is cleared;
    - **taking up the chase again** (only while moving, and only while
      `goHome` is off), in either of two cases:
      1. it **can see Connor and he is within its attack distance**.
         **There is no other condition**: no minimum time on the way
         home, no chase timer, no direction test;
      2. it is within its **home radius** of home (3D), can see Connor
         and he is within its alarm radius.
      This is an alarm (rule 23), with its 0–1 s reaction delay (in that
      time it keeps going the way it was: toward home) and a new 5 s
      chase timer, but **no walk / run coin**: it keeps its `run`.
    - **Consequence**: while Connor stays in its view within its attack
      distance, a character can't be shaken off by its leash. Every 5 s
      (the chase timer) it is past the leash, gives up for one update,
      heads home for up to a second, and turns on Connor again. In the
      original this is the time when it is **fighting him** (it attacks
      whenever its attack time comes, rule 25), so in practice the chase
      ends only once Connor is beyond its attack distance (or out of
      sight) at a moment the leash is tested; then it goes home and
      comes back only by case 2 (near home) or if he comes within its
      attack distance again. The loop B found is the original's
      behaviour; what makes it look wrong is the stand-in for the attack
      (see rule 25: follow a moving Connor, don't stop and start).
    - (A character with no collision shape, none in Daventry, behaves
      differently in case 1: fight study.)
29. **`KQMonster::goHome <name>`** (traced): it gives up at once and
    walks home, and stays unalarmable until `KQMonster::goHome <name> 0`.
    Daventry's `home1.cs` and `home6.cs` traps use it for the ruin
    spriggans and some goblins and spriggans while Connor is in certain
    places (data).
30. **The zombies** (traced and data; answers Q134): a zombie in Birth
    is not drawn (`characters.md` rule 16). It still does the sight
    check.
    - **Rising**: when it is **not active, can see Connor and he is
      within its alarm radius**, it becomes active (if there is room) and
      is alarmed; **`birthDelay` seconds later** it goes to Wait, which
      waits until Connor is not standing on it (farther than its ball
      and his added together), then plays `birth`, makes it drawn and
      solid, switches its gravity on and turns it toward Connor.
    - **The chase during `birth`**: at the next update it is in the
      standing chase of rule 25 and normally goes into its moving state
      at once. But `birth` can't be cut short, so its `walk` waits until
      `birth` has ended: it stays on the spot, steered toward Connor
      (turning on the spot) while it rises. Its stuck check is switched
      on only when the walk really starts (rule 7), and its first check
      comes 1 s after that. So, as seen, **the chase waits for `birth`
      to end**; B's stand-in (staying in Wait while `birth` plays) looks
      the same, except that the original can turn to follow Connor while
      it rises. If Connor comes within its attack distance during
      `birth`, the attack waits for `birth` to end too (fight study).
      After `birth` it chases as in rules 25–27 (zombies have no `run`:
      they walk at 511).
    - **Sinking back** (rule 28): it goes into Birth and plays `hide`
      where it stands, still drawn, still solid and **still active**.
      When `hide` ends it is **taken off the active list** (freeing a
      place), made not drawn and not solid, gravity off, and **put back
      on its home point** (z −1 = floor, −2 = surface). It is then no
      longer on its way home.
    - **While `hide` plays it can't be alarmed again.** If another
      character tells it (rule 24) during `hide`, it is taken off the
      active list at once and goes underground at its home point that
      moment (the rest of `hide` is not shown).
    - **Rising again**: only by the "Rising" case above, from its home
      point: once it is off the active list, can see Connor (from its
      eye at home) and he is within its alarm radius, it rises
      `birthDelay` s later. **Rule 28's re-take never applies
      underground**: it applies only to a character that is moving. So
      B's current version (rule 28's re-take from underground, then
      `birthDelay`, not before `hide` has ended) is **not** how the
      original does it; the "not before `hide` has ended" part is right,
      as a consequence. With Daventry's numbers this doesn't loop: a
      zombie that passed its leash is more than its chase radius from
      home, so Connor near it is usually outside its alarm radius of
      home.
    - **Graveyard** (data, `zomtrap.cs`): the 14 `zograv` zombies start
      100,000 south of the graveyard with alarm radius 0. Entering the
      trap moves them into the graveyard, makes that their home, and gives
      them alarm radii **4,500–8,500**, chase radius 10,000 and home
      radius 6,000. Leaving it sets their alarm radius to 0 and their
      chase radius to **100**: any alarmed zombie is then over its leash,
      gives up and sinks back (`hide`); 10 seconds later, if Connor has
      not come back, they are moved away south again. With an alarm
      radius of 0 they can't rise; entering the trap again gives them
      their radii back, and they rise again from their graveyard homes
      (fits what the user saw in the original).
    - **Road** (data): `RoadZ1` alarm 4,000, rises 3 s after noticing;
      `RoadZ5` 4,000, 3.5 s; `RoadZ2`–`RoadZ4` 18,000, at once. Chase 8,000
      / 12,000 or 4,000 / 8,000 (home / chase).
    - `zograv1` and `zograv2` (behind the farm): the type's values (alarm
      5,500; `zograv2`'s line: 8,000, chase 4,000 / 9,000).

### 5. Flying characters (`flyingObject=1`)

31. **They wander and chase like the others** (traced): the AI is the
    same; only the mover differs. Daventry's bats (`bat1`–`bat5`) loop
    four points at **1,159 units/s** (`batfly`), never stop (no idle
    list), are never alarmed (alarm radius 0, data) and reset to their
    first point when they come back into range (rule 21). The Knight is
    also a flyer but his `walk` doesn't move him.
32. **The flying mover, each update** (traced; **corrects
    `characters.md` rule 8** and answers Q129):
    1. step up and step down set to 0, gravity off;
    2. the ordinary move (`connor-physics.md`), which ends with the
       `keepOnGround` raise (bats: `keepOnGround=1`, onto the water
       surface where there is water);
    3. **down 150** as a down pass. **If that pass hits anything, the
       drop is cancelled completely** (it stays where step 2 left it); it
       is not stopped at the contact;
    4. then a test straight down from about the top of its collision
       shape to its feet against the **drawn ground** (the water surface,
       not the floor under it) and **buildings**; if that finds a surface,
       it is **raised onto it, by at most 300**, as an up pass (cancelled
       if that pass hits something).
    So over land the drop always meets the ground and is cancelled: the
    bat stays on the ground height. Over water the drop goes through the
    surface (the down pass tests the floor), and step 4 finds the
    surface and lifts it straight back: **it ends every update on the
    water surface**, not 150 under it. Over a building floor or roof
    within its height (step 4), it ends on that surface. Its model hangs
    1,170–2,030 above its position (its box, data), which is why bats
    look like they fly. A flyer placed higher (`bat2` at 6,790, `bat3` at
    14,000) sinks by 150 an update until it is on the surface below.
    (Not sure of the exact start height of step 4's test; any start
    above 150 gives the same result.)

### 6. What others do to a moving character (answers Q128)

33. **A character's own touches** (traced). A character is told about a
    touch when its own mask covers the other's type (`connor-collision.md`
    rule 2). It then reacts by the other's type bits, first set bit first,
    with **its own table** (the plain-object table of
    `connor-collision.md` rule 3 does not apply to it):

    | Other's type has bit | In the character's own move |
    | --- | --- |
    | 0x100 (pushable) | obstacle |
    | 0x8 (buildings, 0x28) | obstacle |
    | 0x1 (plain objects 0x41, type-1 things) | **obstacle** |
    | 0x2 (other characters, 0x42) | obstacle |
    | 0x4 (Connor) | **obstacle, but only in the character's own move** |
    | 0x2000 | obstacle |
    | none of these (traps, 0x10) | nothing |

    "Obstacle" means the slide of rule 6. So with the default mask
    0x281F a walking or falling character **stops at, slides along or
    lands on** buildings, plain objects, other characters and **Connor**
    (a character falling onto him lands on his head). This holds for
    **every pass**: up (stop), across (slide) and down (land): answers
    Q128 (a), (b) and (c).
    - **The shape it meets at Connor** (traced; answers Q132 (c)): the
      same ball-against-cylinder test as Connor meets at a character
      (`connor-collision.md` rule 18, now exact), with the roles
      swapped. The character's balls are swept against an upright
      cylinder around Connor: radius 200 (his ball radius), from his
      feet to 1,300 above them (the top of his top ball), widened by the
      moving ball's radius. A ball that starts inside it is no hit
      (Connor's type has bit 0x4), so a character overlapping him can
      walk out. Other characters are met the same way (their own
      cylinders).
34. **When something else moves into a standing character** (traced):
    Connor's touch is ignored by it (the "own move" condition); the
    other kinds are noted but play no part until its own next move
    (inferred: no effect seen). **The character is never pushed**: the
    mover is the one that stops or slides (for Connor, `connor-physics.md`
    "Exact hit handling": a character is a wall for him, never head-on).
35. **A character with `colliderMask=0`** (Daventry: the Wizard, the
    Knight) is told about nothing: in its own moves it passes through
    everything, and the terrain is not tested (`characters.md` rule 8).
    Connor still stops at it (his mask covers its type).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| speed | (sum of the model's frame steps + its last step) × (N − 1)/(N × loop length) (rule 1, corrected 2026-10-24) | traced |
| walk or run when alarmed | 50 / 50, tossed at each start of a chase move | traced |
| turn rate while walking | 3.5 rad/s (automatic turn) | traced |
| stuck check | 1 s after its walk / run really starts, then every 0.5 s; stuck if moved < 200 | traced |
| stuck escape | world diagonals NE, NW, SW, SE, level from its feet, ±1,250 in x and y; snap turn; all blocked → face north (+y); no steering 3 s; `run` | traced |
| way-home re-take | sees Connor within attack distance (nothing else), or within home radius of home and Connor within alarm radius | traced |
| stop within reach (melee, before its attack is due) | only when Connor has no forward / back input | traced |
| ranged stop / start distances | stop within 0.75 ×, start moving beyond 1.25 × attack distance | traced |
| wander point reached | within 250 (x, y) | traced |
| re-aim at a wander point | distance² > closest² + 12,500² | traced |
| random wander points | own position + 3 at 2–3 × alarm radius, angles ≥ 0.79 rad apart | traced |
| first fidget | at once when active | traced |
| next fidget | 2 + round(random × 10) s after the last ends | traced |
| sight check | every 15 updates (1 s) | traced |
| eye height | top of the `BOUNDS.KQ` box | traced |
| target point on Connor | 1,250 above his position | traced |
| sight blocked by | drawn ground, plain objects, buildings (not characters, not Connor) | traced |
| active list size | `KQWorld::MaxMonsters`: 6 (Daventry), 10 (Ice World) | data |
| reaction delay after alarm | random 0–1 s | traced |
| chase timer | 5 s (10 s after being hit) | traced |
| turn to the last seen spot only if | more than 500 away | traced |
| give up after not seeing Connor | 45 updates (3 s) | traced |
| leash | beyond 2 × alarm radius (no team) or chase radius from home | traced |
| home reached | within 100 in x and y | traced |
| reset to home on leaving range | when Connor > 2 × alarm radius away (3D), or on the way home | traced |
| zombie sinking back | `hide` where it gave up; at its end off the list, hidden, put on its home point | traced |
| attack distance default | 1,250 | traced |
| flying mover | down 150 (all or nothing), up ≤ 300 onto drawn ground / buildings | traced |
| walkable slope default | cos 35° (Connor's) | traced |

## Data used

- `<world>/kq/monsters.kq`: movement field (`none`, `@`, `random`, `loop
  n …`, `patrol n …`), chase mode (`guard`, `none`, `chase a b`, `@`),
  alarm radius, birth delay, attack distance. `@` → `GAME/Kq/Monstats.mtl`.
- Character `.kq`: `animFile`, `idleAnimFile`, `wanders`, `wanderAndStop`,
  `wanderPts`, `curWanderPt`, `startLoc`, `alarmRadius`, `homeRadius`,
  `chaseRadius`, `team`, `canAlwaysSeeConner`, `flyingObject`,
  `keepOnGround`, `noTurn`, `doMode`, `colliderType`, `colliderMask`,
  `Alarm*`. (`lineOfSightOffset` is read but replaced, rule 20.)
- `.anm`: `walk`, `run`, `stop`, `hide`, `birth`; idle lists.
- `.kqs` frame steps of the `walk` / `run` models (speed).
- `GAME/patch/BOUNDS.KQ` (eye height, ball size).
- `<world>/kq/world.cs` (`KQWorld::MaxMonsters`), `zomtrap.cs`,
  `home1.cs`, `home6.cs`, `sprdump.cs` (radius and home commands).

## Edge cases

- A character that is not active but in Move (e.g. put there by a script)
  just keeps walking straight: the wander and alarm parts need it active.
- Being active doesn't need the alarm radius: a chicken becomes active the
  first time it can see Connor and starts wandering then.
- A wanderer reaching its last point with `patrol` "reaches" the first
  point twice in a row (rule 10); harmless.
- A character that drifted well away from its line (slid along a long
  wall) re-aims only after more than 12,500 (rule 11) or through the stuck
  escape.
- `KQMonster::setChaseRadius … 100` (the graveyard trap's "off") makes an
  alarmed zombie give up at its next leash check (rule 30).
- During a chase the walk / run coin is tossed again at each new start, so
  a goblin can switch between running and walking after Connor.
- The stuck escape's round (rule 7) and the "stuck check on" switch belong
  to the moving state and start afresh each time it starts moving: a
  character that stops and starts often always tries north-east first.
- A zombie told by a team mate while its `hide` plays goes underground at
  its home at once (rule 30).
- A guard (chase radius 0) that loses sight of Connor stays where it is
  (rule 28).

## Do not reproduce

- **`KQMonster::walkSlope <name> <degrees>`** takes the cosine of the
  number as if it were radians, so `90` (the only value used, outside
  Daventry) gives −0.45: every surface counts as walkable. That happens
  to be what "90 = walk on everything" means; do instead: convert degrees
  properly and treat 90 as "everything".
- **The chickens' constant "stuck" escape** (rule 7) is a side effect of
  their slow walk. Keep it until M7 (it is how the original looks; to be
  checked, How to check 1); then consider scaling the stuck distance with
  the walking speed.
- **`goblin.anm`'s swapped `walk` / `run`** (rule 2): keep the data.
- **The stuck escape's ground test at foot height** (rule 7): the line
  starts exactly at the character's feet, and the ground test decides by
  which side of the surface the start point is on. A character standing
  exactly on the ground then counts ground that falls away along the line
  as "blocked" and ground that rises as free; one a hair above it counts
  rising ground as blocked. Which case the original is in depends on the
  last digits of the numbers (not sure). Do instead: a diagonal is blocked
  when the ground anywhere along it is higher than its feet (B's 1 unit
  above the feet is fine).
- **The stuck escape's snap faces 3.14, not π, from the free diagonal**
  (off by 0.0016 rad, invisible). Do instead: face exactly along it.
- Keep the rest, including the random reaction delay, the snap turn of the
  stuck escape and the reset to home.

## How to check

**Our logs (B):** per character and second: state, active or not, target,
position, heading, distance to Connor, can see (and what blocked the
line), current wander point and its distance, closest distance, movement
playing (walk / run / stop / fidget), idle time; every alarm (who told
it), every give-up (why: lost sight, leash, distance), every stuck event
(the free diagonal chosen), every reset to home, every touch in its own
move (what it hit, stop / slide / land).

**In the original (user, by eye), Daventry:**

1. **Chickens behind the farm** (`chicken1` near (268767, 209932),
   `chicken3` near (244688, 216198); `chicken2` near (230048, 216411)
   never moves): once Connor has a clear view of them, do `chicken1` and
   `chicken3` start moving round their four points and keep going without
   stopping? Do they **run** rather than walk after the first second, and
   **snap round** now and then (rule 7)?
2. **The bone goblin near Sara's house** (`goblin22`, from (289284,
   192527) to (278230, 188608), about 11,700 apart): it walks between the
   two points at a bit below Connor's walking speed (about 13 s a leg),
   and stops now and then for a swing at the air or a flinch, then walks
   on after a few seconds.
3. **A goblin's chase**: let `goblin22` see Connor from closer than
   4,000. After up to a second it turns to him and comes after him; some
   chases at a run faster than Connor's run, some at a walk. Hide behind
   a building: about 3 s later it runs back toward its points.
4. **The leash**: run away from a chasing bone goblin until it is more
   than its attack distance behind you (it then walks: the coin; its run
   is faster than yours); after 5 s and once it is beyond its chase
   radius from home (6,000–9,000), it turns and runs home; on its way
   back it comes for Connor again only if he is within its attack
   distance and in view, or once it is near home (4,000 of it) and he is
   within its alarm radius. If instead Connor lets it stay within reach,
   it keeps fighting him and the leash never takes it home (rule 28).
   Also: does a goblin following a walking Connor at reach keep walking
   with him, and stop as soon as he stands (rule 25)?
   **Checked by the user in the original (2026-10-20):** running away
   far enough sends the goblin running home. A goblin following a
   walking Connor keeps up with him and, even while he keeps walking,
   eventually closes in to within reach and **stops to attack**: the
   attack is played standing still (it can't attack and move forward at
   the same time). When Connor stands still it catches up and attacks
   too. This fits rule 25: within reach, once its attack is due, it
   attacks whatever Connor does. A hint for the fight study: the attack
   seems due soon after it gets within reach.
5. **Spriggans by the windmill** (`sprigwnd2`, `sprigwnd3`, team
   `windguys`): when one notices Connor, does the other come too even if
   farther away (rule 24)?
6. **Graveyard zombies**: from how far do they rise once Connor is in the
   graveyard (about 4,500–8,500, each its own)? When Connor leaves the
   graveyard trap, do the risen ones sink back into the ground?
7. **Road zombies** (near (192916, 316732)): `RoadZ1` rises about 3 s
   after Connor comes within 4,000 in view; `RoadZ2`–`RoadZ4` from very
   far (18,000).
8. **Reset**: watch a chicken or bat walk away from its start, take Connor
   far enough that it is no longer drawn, come back: is it back at its
   start point?
9. **Bats**: do they circle their four points without stopping, at about
   Connor's walking speed? Over the lake (`bat2`, near (161976,
   201141)), at the same height above the water as above land?
10. **Sight**: does a goblin notice Connor through another goblin or a
    chicken in between (it should), but not through a cart or a barrel
    (it shouldn't)?
11. **Walls**: does a walking goblin that meets a fence slide along it,
    and, if it gets stuck, snap to a diagonal and run off?

## Open questions

Answered 2026-10-20: the "all four blocked" heading (face north, rule
7), the reset distance (3D to Connor's current position, rule 21), the
idle list swap (rule 18).

1. Water for walking characters: the water test that drives Connor's
   wading was not followed for characters (rule 6).
2. ~~The fight~~: answered 2026-10-20 in `fighting.md` (attack timing,
   hits, damage, `react`, dying, ranged characters and fleeing;
   characters without a collision shape on their way home: its rule 15).
3. The exact side the stuck escape's ground test takes at foot height
   (Do not reproduce): not sure, and not worth copying.
