# Characters: drawing and animating people and monsters (not fighting)

Study session A, 2026-10-19. Traced in `Mask.exe` unless marked "data"
(read from the game's files), "inferred" or "not traced". Updated the same
day for B's questions Q116–Q124: rules 8, 12, 14, 15, 18, 20, 27, 36 and a
new rule 37 (how shadows are drawn). **Moving on their own** (walking,
wandering, idling, noticing Connor, the flying mover, touches in their
own moves) is in **`character-movement.md`** (2026-10-19), which corrects
rules 8 and 29–31 here (marked below; Q128, Q129). Rule 36 gained
Connor's own shadow test (Q130).

## Purpose

How the game shows the other characters (everything of class `KQMonster`
and its relative `KQGeometryTalker`; Connor is `KQConner` and is covered by
`connor-movement.md`): where they come from, which model and movement they
show at load and while nothing happens, how scripts make them move, turn,
speak with their mouth and hands, and when they are drawn. Enough to draw
Daventry's characters standing and animating, and to build conversations
(`conversation.md`) on them. Chasing, attacking, wandering and dying are
M5; this note only says where they start, so that B can switch them off
cleanly.

Builds on: `objects.md` (placing, `loadKQ`, heights −1/−2, hidden objects,
`move`), `anim-scripts.md` (movements, `KQAnimPlayEvent`, `EndLoop`,
`getEndLoop`, the Special state), `connor-movement.md` (the `loop=` line,
transitions, the automatic turn), `talking.md` and `conversation.md`
(lines, mouth, gestures), `object-lighting.md` rules 34–43 (near / far /
off), `buildings.md` and `water.md` (`keepOnGround`), `connor-physics.md`
(gravity, steps, boxes from `BOUNDS.KQ`).

Words used here:

- **Character**: an object of class `KQMonster` or `KQGeometryTalker`.
- **State**: what the character is doing, as for Connor: **Birth** (not yet
  risen), **Wait** (standing), **Move**, **Special** (a script's movement
  list), **Talk** (in a conversation), **Die**, and a plain **no state**.
- **Active**: a character that has noticed Connor once (rule 29). Not the
  same as "near" (rule 33).
- **Movement**, **waiting movement**, **can be cut short**, **has ended**:
  as in `anim-scripts.md`.

## Rules

### Which objects are characters (question 1)

1. Two classes in the data are characters (data, all worlds):
   - **`KQMonster`** (155 sections): every person and creature, friend or
     foe: the Wizard, the Girl, the Knight, the Lady of the Lake, the
     ravens, the chickens, the pig, the henchman, goblins, spriggans,
     zombies, bats, the shadow monster, … There is no separate class for
     friendly people; the difference is only in their keys and scripts.
   - **`KQGeometryTalker`** (7): a `KQMonster` in every way except **how its
     mouth moves** (rule 27): instead of swapping a face texture it plays
     whole-model mouth movements. Daventry: the **Uglybeast** (`ugly.kq`)
     and the **Unicorn** it turns into (`unicorn.kq`).
   - Nothing else adds to the drawing. `KQMonsterManager` (`monsters.kq`)
     is not a character; it creates characters (rule 3). `KQMonsterType`
     lines (`GAME/Kq/Monstats.mtl`, `KQMonsterTypeList`) are the types it
     uses. `KQLips` sections are mouth texture lists (rule 25).
     `KQConvInfo` is the conversation info (`conversation.md`).
2. A character is an animated object with more on top: everything
   `objects.md` and `anim-scripts.md` say about animated objects holds for
   it (movement list, `loop=` line, `currAnimName`, `nextAnimName`,
   `animPos`, `KQAnimPlayEvent`, cues, `EndLoop`, `getEndLoop`,
   `translucency`, `hidden`, groups), except where this note says
   otherwise.

### Where characters come from

3. **Two ways in** (data and traced):
   - **Their own `.kq` file**, loaded by a script with `loadKQ`, like any
     object (Daventry: `newworld.cs` loads `girl.kq`, `wizard.kq`,
     `ravenwiz.kq`, `shadow.kq`, `Ugly.kq`; `bell.cs` loads `lady.kq`;
     `condoor.cs` loads `ravencnr.kq`; `rvngrvtp.cs` `ravengrv.kq`;
     `stngrl.cs` `ravensrh.kq`; `ugly.cs` `Unicorn.KQ`). Position and angles
     come from that file (`locX/Y/Z`, `dirZ`) or the `loadKQ` line.
   - **A line of the world's monster list** (`monsters.kq`,
     `[Monsters]`, `KQMonsterManager`, `numMonsters`, `monster0`…; Daventry
     has 52 lines). Each line is the same as the script command
     `KQMonster::add` (20 uses elsewhere in the data). Fields, separated by
     tabs, in this order: name, type, team, extra health, extra damage,
     extra protection, extra experience, x, y, z, heading, dependents,
     movement, items held, items dropped, chase mode, alarm radius, birth
     delay, attack distance, attack delay. `@` means "use the type's
     value", `none` means none.
4. **A monster-list line, for drawing** (traced):
   - The **type** names a line of `Monstats.mtl` (`KQMonsterType`), whose
     `KQFile` column names the character's own `.kq` file (`goblin2` →
     `goblin2.kq`, `knighttype` → `knight.kq`, `sprgnbowtype` →
     `sprgnbow.kq`). That file is read as if loaded by `loadKQ`, then the
     line's values are put on top: the object is renamed to the line's
     name, and the position and heading **replace** those of the file. An
     unknown type prints "KQMonster::add - Monster type undefined" and
     makes nothing.
   - **z = −1 in the line means "on the ground"**: the ground height at
     (x, y) is used (the same lookup as `objects.md` "Ground heights" rule
     1 for −1). The heading is the line's.
   - **A skin swap in the type field**: `type(a.bmp,b.bmp)` (Daventry:
     `bossprig` is `"sprigan(test2.bmp,test3.bmp)"`) uses the type before
     the bracket and sets the object's texture swap: wherever its model
     uses texture `a`, draw `b` instead. It is the same pair the `.kq` keys
     `originalBmp` / `alternateBmp` give (and the script command
     `KQObject::changeBmp`; not traced further). Quotes around the field
     are dropped.
   - **Movement field**: `none` → the character never wanders
     (`wanders` 0); anything else → it may wander (`wanders` 1, only once
     active, rule 29): `loop n x y z …` gives n points in order, `patrol n
     …` gives them and then the same points back again, `random` gives
     none; a z of −1 in a point is the ground. Wandering itself is M5.
   - The other columns are fighting values (M5).
   - Then it enters the world like any `loadKQ` object.
5. **Daventry's two script-placed actors start out of sight** (data):
   the monster list puts the **Knight** and the **Henchman** at (0, 0, on
   the ground), and their scripts move them later (`knight.cs` cue 10,
   `ax.cs`). The **Girl** stands at (100, 100, on the ground) from her
   `.kq` file; `girl.cs` `Init` moves her to her hut only if Connor's
   cross-world flag `MetGirlInDC` is set (`getConnorFlag`, a flag only Dead
   City's scripts set; in a new game she stays at (100, 100)).
   `KQMonster::setHome` only sets a character's home point (z −1 = ground);
   it **never moves** it (traced).

### From `.kq` to the picture (question 2)

6. **The keys a character reads for drawing** (traced; capitals ignored as
   for every key):
   - `animFile=<file>.anm`: its movement list (a `KQAnimLoopList`, the same
     format as `Conner.anm`). Every movement's `loop=` line names the model
     (`.kqs`) and sequence; the model switches with each movement, as for
     objects. There is no separate "shape" key for characters.
   - `idleAnimFile=<file>.anm`: a second list of fidgets, used only by
     active characters (rule 30).
   - `lips=<section>` and `baseLip=<texture>`: the mouth (rule 25).
   - `currAnimName`, `nextAnimName`, `animPos`, `flags`: as for objects,
     with the differences of rules 11–14.
   - `translucency` (rule 35), `hidden` (rule 34), `groupsActive3DFar` and
     the other group masks (rule 33), `shadowMask` (rule 36).
   - `monsterType=<name>`: the name used to find its box (rule 9); the
     monster list sets it to the line's type.
   - **No size, scale or offset key** exists for characters (none is read;
     `objects.md` Orientation rule 5). The model is drawn at the object's
     position and heading, as built.
   - `flyingObject=1` gives the character a "flying" mover (Daventry: the
     Knight and the bats). For standing still it changes nothing visible;
     how it moves is M5 (not traced).
   - Read but not used: `isActive3D` (`object-lighting.md` rule 38).
7. **The model's origin is at its feet** (data): every character's
   `BOUNDS.KQ` box starts at z 0 or very near it (the bat's starts at 1,170:
   its model hangs above its origin), and the models' lowest points are at
   about z 0 (`wizstop.kqs` lowest corner z 5.5, data via `kq8
   shape-verify`). So the object's position is the point on the ground
   between its feet.
8. **Standing on the ground** (traced): a character has the same mover as
   Connor, so `connor-physics.md` applies to it unchanged:
   - **Gravity is on** unless `monUseGravity=0` (default 1; Daventry sets 0
     for `ravenwiz` and `sprgnbow`). With gravity on, the character falls
     onto the ground or a building floor as Connor does. With it off, it
     stays at its z (the Wizard's raven on its perch at 7,245, the bow
     spriggan on the tomb roof at 8,370).
   - **`keepOnGround`** (`buildings.md`, `water.md`): default on (−1 =
     floor), `keepOnGround=1` = onto the water surface (the Uglybeast, the
     Unicorn, the Lady, the Knight, the bats, the ravens, the Henchman),
     `keepOnGround=0` = no raise (`ravenwiz`).
   - **Step up and step down 325**, as Connor (`stepUpAmount=0` /
     `stepDownAmount=0` in `sprgnbow.kq`).
   - `KQObject::setUseGravity <name> 0|1` and `KQMonster::setMonUseGravity
     <name> 0|1` switch gravity (Daventry: `knight.cs` switches the Knight's
     off before placing him on the tomb).
   - **What the mover collides with** (traced 2026-10-19, Q122): a
     character's own collision **type is 0x42 and mask 0x281F** unless its
     file says otherwise (`colliderType`, `colliderMask`; a file that sets
     only the type keeps mask 0x281F). These are the character's own
     defaults, not the plain-object ones (0x41 / 0). The mask has the
     terrain bit 0x800, so the terrain counts in the up and down passes
     exactly as for Connor (`connor-physics.md` "The terrain in the
     passes"). Buildings, other characters, Connor and plain objects take
     part by the ordinary pair rule (`connor-physics.md`): 0x281F shares a
     bit with buildings (0x28), plain objects (0x41), characters (0x42)
     and Connor (4), so with the defaults a falling character lands on a
     building floor, an object, another character or Connor's head.
     **Refined 2026-10-19 (Q128):** what the character then *does* is
     its own table, not the plain-object one of `connor-collision.md`
     rule 3: plain objects, buildings, characters and Connor (in its own
     move only) are all obstacles, in every pass
     (`character-movement.md` rules 33–35).
   - **A file with `colliderMask=0`** (Daventry: the Wizard and the Knight,
     type 49185 = 0xC021) has **no terrain bit**: the terrain is not
     tested in its passes, and its mover hits only things whose own mask
     covers its type (Connor's 0x281B does, through bit 1; buildings' mask
     is 0, so **it falls through building floors**). Only the ground check
     (`keepOnGround`, step 5 of the mover) stops it, at the terrain.
     Data: the Wizard stands at the terrain height of his house anyway; the
     Knight's script turns his gravity off before placing him.
   - **No Fall state, no fall death, no steep-slope gravity** (traced): the
     mover starts the Fall state after 5 updates in the air, and switches
     gravity on when landing on steep ground, only for an object whose
     collision **type has bit 4** (Connor's). Characters' type 0x42 (and
     every `colliderType` in Daventry's character files) lacks it. Their
     state list has no Fall state either. A character that drops from any
     height just falls with gravity and lands; nothing else happens.
   - **Gravity runs only while the character is updated** (rule 11: near
     or far, not hidden). A character that starts in the air stays there
     until it first comes in range, then falls from there in its updates,
     with the same gravity as Connor (`connor-physics.md` "Gravity").
   - **`flyingObject=1`** (Daventry: the Knight and the bats): a different
     mover. Each update it sets the character's step up and step down to
     0 and its gravity off, does the ordinary move, then moves it **down
     by 150** and, if a ground or building surface lies above its feet
     within its height, **raises it by up to 300**. So a flyer settles
     onto the surface below at up to 150 units per update (2,250 per
     second) and rides over bumps. Daventry's bats have their model 1,170
     above their position (rule 7), so this keeps them hanging in the air
     above the ground. **Corrected 2026-10-19 (Q129):** the 150 drop is
     all or nothing (cancelled if the down pass hits anything), and the
     raise tests the **drawn ground (the water surface)** and buildings,
     so a flyer over water ends each update **on the surface**, not 150
     under it. Details and how they fly about: `character-movement.md`
     rules 31–32.
9. **The box** (traced): a character's collision box comes from `BOUNDS.KQ`
   looked up by its `monsterType` (from the monster list: the line's type,
   e.g. `goblin2`, `sprgnbowtype`, `knighttype`) and, if that finds
   nothing, by its object name (`Wizard`, `Girl`, `Lakelady`,
   `Uglybeast`, `Ravenw`, …). The lookup is **exact, capitals included**
   (`connor-physics.md`; `BOUNDS.KQ` has both `henchman` 3,000 high and
   `Henchman` 2,000 high, and the Henchman's type is `henchman`). The box
   is also the ball for the near / far / off test (`object-lighting.md`
   rule 35). Characters collide as a stack of spheres (as in
   `connor-collision.md`); not needed to draw them.
10. **Textures**: as for any model (`models-and-textures.md`), from the
    current world's folders. The skin swap of rule 4 and the mouth of rule
    28 replace single textures by name.

### What plays at load (question 3)

11. **A character does nothing until it is first in range** (traced). Its
    update (animation, state, mover) runs only while it is **near or far**
    (`object-lighting.md` rules 34–37), not while it is off or hidden. So
    its first movement starts in the first update in which it is near or
    far, not when it is loaded. (Ordinary animated objects behave the same
    way, `anim-scripts.md` rule 3.)
12. **The start-up** (traced; **corrected 2026-10-19**, Q116 and Q121;
    refines `anim-scripts.md` rule 3 for characters):
    - **When:** at the first of these moments: the character's first
      update (rule 11), the moment it first becomes near or far, or the
      **first movement request** it gets (a `KQAnimPlayEvent`, a state
      asking for a movement, a `Special` list), **whatever its range or
      `hidden` flag**. A request to a character that has never started a
      movement runs the start-up first, then is handled as usual.
    - **The names start as `Stop`** (corrected 2026-10-19): every animated
      object, characters included, begins with current movement **`Stop`**
      and waiting movement **`Stop`**. `currAnimName` and `nextAnimName`
      replace them when the file has them; a missing key leaves `Stop`.
      So a character without `currAnimName` starts on its `stop` movement
      (capitals ignored), or on movement 0 if its list has no `stop`
      (every Daventry character list has one, data). The old step "empty
      current name: the waiting movement starts and the character stays in
      no state" never happens: no file in the game sets an empty
      `currAnimName` (data).
    - **What it does:**
      1. The current movement starts afresh, with `animPos` and the bit-8
         random start of `flags` as for objects.
      2. **Only if the character has no state yet:** if its list has a
         movement called **`Birth`** (capitals ignored) it goes into the
         **Birth** state (rule 16); otherwise it is put into **Wait**
         `birthDelay` seconds later (through the event queue, in game
         time; rule 13). A character that already has a state (set by a
         script before the start-up, e.g. `sprdump`'s `Special`) keeps it,
         and no Wait is queued.
      3. The mouth is set to shape 0 (rule 25).
    - **`birthDelay`** comes from the monster-list line (the 18th field,
      rule 3; `@` = the type's value) or the `.kq` key, default 0.
      **Correction (Q116, data):** it is not 0 for every Daventry
      character: `goblin18` and `goblin20` have **1**, the zombies `RoadZ1`
      **3** and `RoadZ5` **3.5** (zombies have `Birth`, so it does not
      apply to them). The two goblins start on `stop` (frame 0, speed 1;
      their `flags` has no bit 8), and one second later Wait asks for
      `Stop` again, which restarts it at a random frame and speed
      (rule 13).
    - **Example, `sprdump`** (Q121): `outdoor.cs` shows it and gives it
      `setState … special dump` before it was ever updated. Entering
      Special asks for `dump`; that request runs the start-up at once:
      `stop` (its `currAnimName`) starts, nothing is queued (it already
      has the Special state), then `dump` replaces `stop` at once (`stop`
      can be cut short). Its later first update finds a movement playing
      and does nothing extra. So `dump` plays from its start, and
      `EndLoop dump` comes at its real end. B's stand-in gives the same
      result.
13. **Wait, when it starts** (traced): asks for the movement **`Stop`**
    (capitals ignored, so `stop` matches) the ordinary way
    (`anim-scripts.md` rule 4: at once if the running movement can be cut
    short or has ended, otherwise when it ends). If the character's
    `flags` has bit **0x10** (the **default for characters**; a `flags=`
    key replaces it, e.g. `flags=32` on the Wizard, the Girl, the Knight,
    the Uglybeast and the Unicorn), `Stop` starts at a **random frame** and
    plays at a **random speed from 0.9 to 1.1** (the same random part as
    `anim-scripts.md` rule 3, bit 8). So a field of chickens or goblins
    does not breathe in step.
14. **Every Daventry character starts on `stop`** (corrected 2026-10-19,
    from rule 12's `Stop` default; data): movement 0 (`attack` in most
    lists, the Lady's `down`) is **not** shown at the start. Characters
    without `currAnimName` (the Wizard, the Girl, the Lady, the ravens, the
    chickens, the pig, the goblins, the spriggans, the zombies, the bats,
    the Shadow) start on `stop` by default; the others set
    `currAnimName=stop`. Movement 0 only plays when something asks for it
    (fighting, M5) or for a list without `stop` (none in Daventry).
15. **No state** (corrected 2026-10-19): a character is in "no state" only
    between its start-up and the Wait that the start-up queues (for 0 s,
    or `birthDelay`), or when a script asks it for a state it can't have
    (Do not reproduce). **The Lady and the Shadow are in Wait like the
    others** (the earlier version said they stay in no state; that came
    from the empty-name step of rule 12, which never happens). The Lady,
    worked through: her `Init` sends `KQAnimPlayEvent up`; the request
    runs the start-up (`stop` starts, Wait is queued), then `up` replaces
    `stop` at once; Wait then asks for `Stop`, which waits for `up` to end
    (`up` can't be cut short). At the end of `up`, `Stop` starts, `EndLoop
    up` reaches `lady.cs` (which sends `stop` again, harmless) and Wait
    asks for `Stop` once more (rule 18). In "no state" a play-once
    movement that ends holds its last frame, as for an object.
16. **Birth** (traced): only for characters whose list has `Birth`
    (Daventry: **every zombie**, `zombie.anm` `birth`):
    - **Not drawn and not solid**: their group bits are replaced by
      `groupsAlways` plus bit 1 only, so none of the draw bits (0x10, 0x40,
      0x80, 0x100) nor the collider bit (0x8) is set. Gravity is off.
      (If the list has a `PreBirth` movement it loops visibly instead;
      none in Daventry.)
    - They rise only when they notice Connor within their alarm radius
      (rule 29); then Wait plays `Birth` (roughly: once Connor is not
      standing on the spot; M5) and gives them back their groups and
      gravity. For Daventry's
      graveyard zombies the alarm radius is 0 until `zomtrap.cs` sets it;
      the road zombies have 4,000–18,000. **Before M5, draw no zombie.**
17. **Speed** (traced, as for objects): a movement plays at its `loop=`
    line's speed (Daventry: 1.0 for nearly all; the Lady's `down` 0.8 and
    `up` 0.6), times the random 0.9–1.1 of rule 13 where it applies. The
    frame rate comes from the model (`connor-movement.md`). Nothing about
    playback speed differs from Connor's.

### Movements following one another (question 3, continued)

18. **In Wait, a finished movement goes back to `Stop`** (traced): when a
    play-once movement ends while the character is in Wait:
    - first the ordinary end (`anim-scripts.md` rule 8: the waiting
      movement starts if there is one; then `EndLoop` to its own script and
      `Cue 300` to its `getEndLoop` listener);
    - then, **unless the movement that ended was `Stop` or a gesture**
      (`gesture…`), Wait asks for `Stop` again, at a **random frame and
      speed 0.9–1.1** (for every character, whatever its `flags`).
    So `sendEvent 0 Wizard KQAnimPlayEvent wizmap1` plays the map movement
    once and the Wizard then stands again; `wizard.cs` also sends
    `KQAnimPlayEvent stop` itself after `wizmap3`, which restarts `Stop`
    (harmless).
    - **All of it happens before the next picture** (traced 2026-10-19,
      Q119): the end of the movement, the waiting movement, Wait's `Stop`,
      the script's `EndLoop` run, and whatever that script sends with
      delay 0 are all handled within the **same update**, before the frame
      is drawn (`anim-scripts.md` rule 8, "same update"). Order: (1) the
      waiting movement starts, if any; (2) Wait's `Stop` starts (or waits,
      if the waiting movement can't be cut short); (3) the script gets
      `EndLoop` (and the listener `Cue 300`); (4) the script's own
      requests with delay 0 arrive. So for the Uglybeast: `pivot` ends,
      `Stop` starts at the old heading, `ugly.cs` runs `move Uglybeast 3 …
      -1.57` and sends `stand`, and `stand` replaces `Stop` at once (`Stop`
      can be cut short), all before anything is drawn. **The first picture
      after the pivot shows `stand` at the new heading**; `Stop` is never
      seen. This is why the original shows no snap. (Answers (a): the
      script runs through the queue, but still before the frame is drawn;
      (b): Wait's `Stop` comes first and the script's `stand` replaces it.)
    - **Movements whose model has a loop duration of 0 never end**
      (traced 2026-10-19, Q120; `anim-scripts.md` rule 10a). Their phase
      never moves, so they hold their pose and never come here. This covers
      the `KQGeometryTalker` mouth poses (rule 27): no return to `Stop`
      between mouth codes.
19. **In Talk, nothing goes back to `Stop` by itself** (traced): the Talk
    state only does the ordinary end, so a gesture that ends holds its
    last frame until the next gesture or code 16 (`stop`, rule 28) asks
    for something else. The data relies on this: 91 of Daventry's 93
    lip-sync files with gesture codes end with a 16 (data).
20. **Transitions** work as for Connor (`anim-scripts.md` rule 4.2,
    `connor-movement.md`): Daventry's only character transition is the
    Girl's `cower` → `stop` through `cowerstop` from phase 0.88 (data).
    **Where they meet the character rules** (traced 2026-10-19, Q124):
    - **The end of the transition movement is an ordinary end**: when
      `cowerstop` ends, its target (`stop`) starts as for Connor
      (`connor-movement.md` rule 6), then `EndLoop cowerstop` goes to the
      character's script and `Cue 300 … cowerstop` to its listener (the
      name is the transition movement's), then, in Wait, rule 18 applies:
      `cowerstop` is not `Stop` or a gesture, so Wait asks for `Stop`
      again, and as the `stop` that just started can be cut short, it
      **restarts at a random frame and speed** in the same update. The
      picture shows `stop` at a random frame.
    - **The `endLoop` cue** (`cueN=<phase> endLoop`) makes a character do
      its whole "movement ended" handling at that phase: the waiting
      movement starts at once if there is one (even if the running one
      can't be cut short), `EndLoop` and `Cue 300` are sent, and in Wait
      rule 18 asks for `Stop`. **The movement itself is not stopped**: if
      nothing replaces it (no waiting movement, and Wait's `Stop` has to
      wait because it can't be cut short), it plays on to its real end,
      which then runs the end handling a second time (a second `EndLoop`).
    - **Correction of the data (Q124 c):** the cue is not on `birthx`.
      Daventry's only one is on the Knight's **`attack`** (movement 0,
      `herostop`, `0 1`, phase 0.84); `birthx` has only a sound cue. The
      other two are on the archon's `attack` (Barren, Snow exit). `attack`
      can be cut short, so Wait's `Stop` replaces it at the cue. Nothing
      before M5 plays `attack`, so the cue is **not needed before M5**.
21. **Compared with Connor, the differences are**: characters have no
    walk / run / turn movements chosen from keys; their Wait doesn't react
    to a turn (rule 23); they return to `Stop` after any one-shot (rule 18)
    where Connor's Wait asks for `Stop` only when he stops; and their
    `Stop` gets the random frame and speed. Everything else (`loop=`
    fields, cut short, transitions, cues, speed sign) is the same code.

### Script-driven animation on characters (question 4)

22. **The commands** (traced; data counts are Daventry's):
    - `sendEvent <t> <name> KQAnimPlayEvent <movement>` (66): as for
      objects (`anim-scripts.md` rules 4–5); in Wait, rule 18 then brings
      `Stop` back.
    - `getEndLoop <name> <listener> 1|0` (38) and the `EndLoop` message:
      as for objects. Daventry uses them on the Wizard (map scene:
      `getEndLoop $1 $1 1` and `Cue 300 … wizmap1`), the Lady, the Knight
      (`birthx`), the ravens (`fromconnor`), `sprdump` (`dump`).
    - **`KQMonster::setState <name> <state> [words…]`**: states by name,
      capitals ignored: `Wait`, `Move`, `Unborn` (= Birth), `Die`,
      `Special`, and Connor-only ones (`SwapWeapons`, `ThrowRock`, `Jump`,
      `Push`, `Tile`, `Tile2`, `Tile3`, `Scales`, `Choice`, `Portal`). A
      character has only Birth, Wait, Move, Die, Special and Talk; asking
      it for another leaves it in "no state" (Do not reproduce). An
      unknown name prints "cannot find state". A state is only entered if
      the current one allows it (Talk allows Talk, Wait, Move, Birth, Die,
      Special); otherwise the request waits and is tried again when Wait's
      movements end.
    - **`Special <list>`** on a character (Daventry: `Ravens/Ravenc/Raveng`
      `caw`, `fromconnor`, `fromgrave`, `fromsarah`; Shadow `'SM_P1S4,SM_P1S5'`
      and `'SM_P1S7,…,SM_P1S11'`; Henchman `laugh`, `cdhm5`; `sprdump`
      `dump`): **the same list handling as Connor's** (`anim-scripts.md`
      rule 16: one movement at a time, each waits for the previous to end,
      `AnimList=` etc. allowed), and when the list is used up the
      character goes to **Wait** (so `Stop`). The words after `Special`
      are joined with spaces into the list.
    - **`move <name> <flags> x y z rx ry rz`** (`objects.md` Q20 d): moves
      a character like any object. `ugly.cs` turns the Uglybeast toward
      Connor with `KQAnimPlayEvent pivot` (a movement that turns the beast
      in place, `ug_pivot.kqs`, played once), and at its `EndLoop` does
      `move Uglybeast 3 0 750 0 0 0 -1.57` (relative: 750 sideways, a
      quarter turn) so that the model's new rest position matches the end
      of the pivot, then `KQAnimPlayEvent stand`. `wizard.cs` moves the
      Wizard into Connor's spot for the ring scene (`move Wizard 0 192420
      197853 6630 …`), back afterwards, and away to 183807 (out of the
      house) at `EndLoop ringspl3`.
    - **`makeTranslucent <name> <0–256>`** (2): sets the see-through value
      **at once**, no fade (traced); 256 = solid, drawn as `objects.md`
      "See-through objects" rule 2. `wizard.cs`: 150 during the ring
      spell, 256 after.
    - **`KQObject::setHidden <name> 0|1`**: `objects.md` rule Q18. A hidden
      character is not drawn, not solid, and **not updated at all** (rule
      11): its animation is frozen where it was. Daventry: `sprdump`
      (`hidden=1` in its file) is shown by `outdoor.cs` and given `Special
      dump` at the same moment.
    - **`deleteObject <name>`**: as for objects (`clicking.md` rule 22).
      Daventry deletes `Ravenc`, `Ravens`, the Knight, the Uglybeast (then
      loads the Unicorn at the same place), the Lady.
    - `pauseAnim sprdump` (`anim-scripts.md` rule 14) freezes the movement
      while the mover runs.
    - **`KQMonster::changeAnmFile <name> <file>`** (1 use in Daventry): the
      new list is taken at the next end of a movement (traced only that
      far).
    - **`KQMonster::setCanTurn <name> 0|1`** (none in Daventry):
      switches `noTurn` (rule 23).

### Turning (question 5)

23. **Turning a character** (traced):
    - A character turns only when something asks: `speak` with a listener
      (`talking.md` rule 7: monsters always), the Talk state (rule 24), and
      the AI (M5). **`noTurn=1` blocks all of these**: the request is
      ignored (Daventry: the Wizard, the Knight, the Shadow, the Uglybeast,
      the Unicorn; data). That is why the Wizard does not turn toward
      Connor in his conversations; his scripts and his talking spot are
      laid out so that he faces Connor anyway.
    - The turn is the **same automatic turn as Connor's**: toward the
      target point at **3.5 rad/s** (the mover's `rotateSpeed` 1.75 × 2;
      no data file sets `rotateSpeed`), the target replaced by a newer
      request, ending when the remaining angle is less than one update's
      turn.
    - **No turning movement plays**: unlike Connor, a character's Wait and
      Talk do not switch to `Left` / `Right`; it turns on the spot while
      its current movement (usually `Stop`) goes on.
24. **In Talk** (`conversation.md` rule 17): every 4th update it asks to
    turn toward Connor's current position (unless `noTurn`).

### The face and the hands (question 6)

25. **The mouth texture** (traced; refines `talking.md` rule 18):
    - `lips=<section>` names a `KQLips` section (`lip0=`…`lip7=`, eight
      texture names; Daventry: `[WizardLips]` `WF0`–`WF7`, `[GirlLips]`
      `Girl_0`–`Girl_7`, `[LadyLips]` `DAMF0`–`DAMF7`, data).
    - The **face texture** is `baseLip` if given (only Connor's and one
      Temple 3 skeleton's files have it, data), else `lip0`. The models
      carry that texture (`wizstop.kqs` has `WF0.BMP`, `G_stop.kqs`
      `girl_0.bmp`, `Ll_stp.kqs` `Damf0.bmp`; data).
    - Mouth shape n (0–7) draws `lip<n>` wherever the model uses the face
      texture; n of 8 or more draws `lip0`. Names are matched ignoring
      capitals (inferred: the lists and the models differ in capitals,
      e.g. `Girl_0` / `girl_0`).
    - The shape is **kept across movements**: whenever a new movement (and
      so maybe a new model) starts, the current shape is applied to it
      again. At the first update the shape is 0.
    - A character without `lips=` has no mouth change. At the end of every
      line the shape goes back to 0 (`talking.md`).
26. **Daventry's talking characters** (data): with a texture mouth: the
    Wizard, the Girl, the Lady. Without `lips=`: the Knight, the ravens,
    the Henchman, the Shadow, the bow spriggans (their lines play with no
    mouth movement). Geometry mouth (rule 27): the Uglybeast, the Unicorn.
27. **`KQGeometryTalker`'s mouth** (traced): each lip-sync code **starts a
    movement at once** (not a request: it replaces whatever plays, even a
    movement that can't be cut short): code 0 and code 16 → `rest`, 1 →
    `f`, 2 → `m`, 3 → `r`, 4 → `w`, 5 → `ih`, 6 → `ah`, 7 → `e`. These are
    whole-model mouth poses in `ugly.anm` / `unicorn.anm` (each `0 1`). It
    **never gestures**.
    - **The poses hold** (traced 2026-10-19, Q120): their models
      (`ug_rest.kqs`, `ug_f.kqs`, …) have a **loop duration of 0.0**
      (data), so the pose's phase never moves and it **never ends**
      (`anim-scripts.md` rule 10a). No `EndLoop`, and no return to `Stop`
      from rule 18, even though the Uglybeast is in Wait while it speaks.
      Each pose stays until the next code replaces it. So during a line
      the body stands still in one pose and only the mouth changes; there
      is no flicker. (Answers (a): the poses are not exempt by name; they
      just never end.)
    - **At the end of a line** nothing returns it to `stop` (the "shape 0"
      of rule 25 is a texture change that does nothing for it): the last
      code of its lip-sync file decides (usually 0, `rest`), and that pose
      holds.
    - **At the end of the last line of a queue**: the "speaker goes to
      Wait" of `talking.md` rule 13a does nothing here, because the
      Uglybeast is **already in Wait** (rule 13a only acts when the speaker
      is not). It stays on the held pose until a script asks for
      something else. **Its scripts do:** the "too far" line (`ugly.cs`,
      `speak Uglybeast Connor 1000 59 0 32 1 0`, lines 1–2) ends with
      `TalkComplete … 0 32 2` → `sendEvent 0 Uglybeast KQAnimPlayEvent
      stop`, which brings back `stop` (answers (b)).
      **The user's check (2026-10-19)** fits this exactly: clicked from far
      away, it says the "too far" line, its animation changes from the
      idle one to a still pose where only the mouth moves, then goes back
      to idle.
    - In a conversation (Talk state) the same holds: the poses never end.
      There the end of the queue's last line does act (it is not in Wait):
      `Stop` is asked for and it goes to Wait (`talking.md` rule 13a).
    Codes 8–15 and above 16: see Do not reproduce.
28. **Gestures** (traced; **corrects `conversation.md` rule 18**): codes
    8–15 ask for `gesturea`…`gestureh`, 16 for `stop`, the ordinary way
    (`anim-scripts.md` rule 4), for a `KQMonster` (not a
    `KQGeometryTalker`) when:
    - it is not walking (its mover reports no walking movement), **and**
    - its **current movement can be cut short or has ended**, **and**
    - the movement exists in its list (else nothing).
    There is **no talk-state condition**. A conversation start only marks
    the partner's current movement as "can be cut short" (that is what
    `conversation.md` rule 2 means by "the anim flag"); a character
    standing in `Stop` (`1 1` in every Daventry list) qualifies anyway. So
    **`speak` lines gesture too** (answers `conversation.md` open question
    2): e.g. the Knight's lines (6 files with gesture codes) are `speak`
    lines, not a conversation.
    Data, Daventry: gesture codes are in lines of the Wizard (39 files),
    Connor (39), the Girl (8), the Knight (6); none in the Uglybeast's own
    lines (its 15 files with codes 8–16 are Connor's lines).

### What a character does on its own (question 7)

29. **Noticing Connor** (traced; enough to switch it off): while near or
    far, every 15th update (1 s at 15 updates per second) a character
    checks whether it **can see Connor**: a straight line from its
    position 1,000 above its feet (`lineOfSightOffset`) to Connor's
    position 1,250 above his, not blocked by the ground or by anything
    solid other than itself; **no distance limit** (`canAlwaysSeeConner=1`
    skips the test; the Knight has it). The first time it can see him in
    Wait, it becomes **active** (at most `KQWorld::MaxMonsters` active at
    once). **Corrected 2026-10-19 (`character-movement.md` rules 20–21):**
    the eye is at the **top of its `BOUNDS.KQ` box** (the 1,000 and the
    key are replaced when it enters the world); the line is blocked only
    by the drawn ground, plain objects and buildings, **not by other
    characters or Connor**; `MaxMonsters` is 6 in Daventry; and a
    character **stops being active when it goes off** (out of range).
    Only an active character:
    - **wanders** (if `wanders` is on: walks from point to point; M5),
    - **idles** (rule 30),
    - is **alarmed** when it can see Connor and is closer than
      `alarmRadius`: then chasing and fighting start (M5).
30. **Idling** (traced, not followed in detail): an active character with
    an `idleAnimFile` (Daventry: `goblin2`, `sprigan`, `sprigan2`) now and
    then plays a fidget from that list in Wait (about every 2 s plus a
    random part), then returns to `Stop` (rule 18). M5 with the rest.
    **Made exact 2026-10-19** (`character-movement.md` rules 15–19): the
    first fidget comes as soon as it is active; the next is due 2 s plus
    a random 0–10 whole seconds after the last one ends; it is picked at
    random from the list; a wanderer stops walking for it.
31. **Which keys keep a character still** (data and traced): `wanders=0`
    (or movement `none` in the monster list), `alarmRadius=0`,
    `homeRadius=0`, `chaseRadius=0`, `noTurn=1`. Daventry's people all have
    `wanders=0`: the Wizard, the Girl, the Lady, the Uglybeast, the
    Unicorn, the ravens, the Shadow, the Knight (also `homeRadius=0`,
    `chaseRadius=0`). The chicken `chicken1` (`loop 4 …`), `chicken3`
    (`patrol 4 …`), the bats (`loop 4 …`) and the goblins with points
    wander once active; `chicken2` and the pig (`piggy1`, `none`) don't.
    Script commands that change this: `KQMonster::setAlarmRadius`,
    `setChaseRadius`, `setHomeRadius`, `goHome` (M5).
    **Refined 2026-10-19** (`character-movement.md` rules 9, 23, 27, 31):
    for characters from the monster list, the list's (or the type's, for
    `@`) movement, alarm radius and chase mode **replace** the `.kq`
    keys (`goblin2.kq`'s `alarmRadius=1500` is never used); the bats too
    have alarm radius 0 (they wander but are never alarmed); `guard`
    means chase radius 0: such a character turns to follow Connor but
    never moves after him.
32. **What B can show before M5** (our recommendation, not a game rule):
    every character in Wait, **standing in `Stop`**, never active. That is
    exactly what the original shows **until a character first sees
    Connor**; after that, wanderers walk about and enemies attack, which
    is M5. Zombies stay unborn (not drawn, rule 16). Turning toward Connor
    happens only through `speak`, conversations and scripts, which are
    built. (M5's first part, everything up to the first swing, is now
    described in `character-movement.md`.)

### Visibility (question 8)

33. **Near / far / off** (`object-lighting.md` rules 34–43) applies to
    characters unchanged, with their box ball (rule 9). Daventry's file
    keys: the Wizard has `groupsActive3DFar=0`: **not drawn when far**
    (beyond 0.83 × 27,000 = 22,410 from Connor, minus his ball). Others use
    the default far mask (drawn, not solid). Off and hidden characters are
    not updated (rule 11), so their animation does not move on.
34. **Hidden** (`objects.md` Q18): as for objects; Daventry: `sprdump`.
35. **See-through** (`objects.md` Q14): `translucency` as for objects
    (Daventry: the Knight 165, the Shadow 180; the Wizard 150 during the
    ring spell). No fading in or out exists for characters other than
    `makeTranslucent` steps set by scripts and the haze fade of
    see-through objects.
36. **Shadows** (traced, the drawing itself not traced): an object
    **casts a shadow** when (the world's shadow mask) AND (its own
    `shadowMask`) AND (the "allow shadows" switch) is not 0, **and** it is
    solid (`translucency` 256). Values:
    - the world's mask: `setShadowMask <n>`; the program's own value is 1,
      and `MASK/Options.cs` sets **15** at start (data);
    - `shadowMask` of an object: default **8** for plain objects, **2** for
      characters (both cast with 15); `shadowMask=0` turns it off (1,149
      sections in all worlds say 0, e.g. Daventry's chickens and pig; the Girl says 4, data);
    - `allowShadows 0|1`: all on (default) or all off. **Corrected
      2026-10-19 (Q123, data):** it is used far more widely than the first
      version said: every world's `world.cs` sets it when the world loads
      (1 in Daventry, Barren, Dead City, Ice World, Snow exit, Swamp; 0 in
      Castle of the Dead, Gnome and the four Temples), and some scripts
      switch it while Connor is somewhere special: Dead City's rope traps
      (`ropetp1a/2a/2c.cs` 1, `ropetp1b/2b/2d.cs` 0), Ice World's ropes and
      elevator, Barren's elevator and `shdwon/shdwoff.cs`, Snow exit's
      `portalt3.cs`, and `GAME/kq/gnomtele.cs` (1). The switch is also kept
      in saved games (with Connor's data).
    So in Daventry **the Wizard, the Girl, the ravens, the goblins,
    spriggans, zombies, bats, the Henchman, the Lady and the Uglybeast cast
    shadows**; the chickens, the pig, the Knight and the Shadow don't.
    - **Connor** (traced 2026-10-19, Q130): the **same test** as every
      character, with **his own `shadowMask` 1**, set by his constructor
      (`Connor.kq` has none, and no script changes it, data). So (a) 15
      AND 1 → he casts in every world whose mask has bit 1 (all of them,
      with `Options.cs`'s 15); (b) **`allowShadows 0` switches his off
      too** (Castle of the Dead, Gnome, the Temples, Dead City's rope
      traps …); (c) he casts only while he is **solid** (`translucency`
      256: a see-through Connor has no shadow) and while his figure is
      drawn (the shadow is made as part of drawing it, so a hidden Connor
      has none). Whether first person skips that drawing step was **not
      traced**.
    **The user's check in the original (2026-10-19):** the shadow is a
    **dark copy of the figure** on the ground, and where parts of it
    overlap (e.g. an arm over the body) the ground gets **darker again**:
    each part darkens the ground on its own, with no merging. Copy that
    look until M7; drawing the shadow as one even shape is noted as an M7
    improvement (`docs/DESIGN.md`).
37. **How a shadow is drawn** (traced 2026-10-19, Q123; the pixel blending
    itself not traced):
    - **Direction: along the sun.** The direction is the aim of the first
      sun-type light in the scene (the world's sun, `tsLightAim` in
      `light/SunlgtHW.kq` on hardware; Daventry (0, 1, −0.66);
      `object-lighting.md` rules 1–5), made length 1. With no sun in the
      scene, (1, 1, −1). So in Daventry shadows fall toward +y, about 1.5
      units sideways for every unit of height. Not straight down.
    - **The copy:** each triangle of the character's current model frame
      (as drawn, with its position and heading) that faces one way with
      respect to the sun (the test keeps one side of the model, probably
      the side facing the sun) is projected **along the sun direction
      onto the plane of each receiving surface triangle**, cut to that
      triangle, and drawn there. Each projected triangle is drawn on its
      own, so where two parts of the model cover the same spot (an arm
      over the body), the spot is darkened twice: the look the user saw.
      There is no flattening onto one plane at the feet: the shadow
      follows every terrain triangle and building surface it falls on.
    - **Lifted 20 units**: each receiving plane is moved 20 units toward
      its front before projecting, so the shadow sits just above the
      surface (against flicker).
    - **What it lies on:** (1) the **terrain**, but only when the
      character is within **1,500** units (up or down) of the ground
      height under its position; the terrain triangles searched are those
      under its box moved along the sun by its height above the ground.
      (2) **Buildings** (detailed 2026-10-20, Q135, traced): every
      building that is near or far (active) and close enough, and of it
      the surfaces that reach into a search box. **Not on objects or
      other characters.** In detail:
      - **The character's size.** Every model file (`.kqs`) stores a
        ball after its version number: a radius and a centre, in the
        model's own space (origin at the character's position, turned
        with its heading). The game uses the ball of the model currently
        shown (each movement file has its own; e.g. Connor's
        `C_stop.kqs` radius 879, centre 827 up; `C_run.kqs` 1,134 /
        799; the goblin's `gobnew33.kqs` 942 / 741). From it the
        shadow code takes a **reach size s = 0.287 × that radius**
        (Connor standing: 252). Not the `BOUNDS.KQ` box. (That the
        game's model holds exactly these stored values is matched by
        their size and place, not traced in the file reader.)
      - **(a) Reach.** The building's ball is the one of
        `object-lighting.md` rule 40 with its centre on the **turned
        middle of the `.dig` box** (the box in the `.dig` header, moved
        and turned by the building's placement; radius half its
        diagonal). The character's point is its **model ball's centre**,
        moved and turned with the character into the world. The building
        is searched when the **distance between the two centres is at
        most building radius + s + 500**.
      - **(b) Height band.** In the character's own space: from **s
        above the model ball's centre** down to **6 × s below it** (so
        7 × s tall; Connor standing: 1,079 above his feet down to 687
        below them). It is measured from the ball's centre, not from the
        top or the feet. If it is taller than 1,800 it is cut to 1,800
        around its own middle (any model ball radius over about 896,
        e.g. Connor running: about 915 below his feet to 885 above).
      - **(c) The area.** Also in the character's own space, so it
        **turns with the character's heading** (its sides are along the
        character's front and side, not the world's x and y). Take a
        square 2 × s across centred on the model ball's centre, at the
        band's top height; move a copy of it along the sun down to the
        character's foot level (height 0 of its own space). The area is
        the smallest rectangle around both squares. Each side longer
        than **1,800** is cut to 1,800 around its own middle; shorter
        sides stay as they are. (Connor standing in Daventry, facing
        along the sun: the copy moves about 1,635, so about 505 across by
        2,139, cut to 1,800 along the sun.) With the band it makes a box;
        no sun-type light pointing level (no downward part) means no
        building search at all.
      - **Which surfaces count:** the game walks the building's space
        partition (the `.dig` BSP) to the regions the box reaches and
        takes their surfaces whole when they reach the box. Its own test
        per surface is looser than intended (see "Do not reproduce"), so
        some surfaces just outside the box are also taken. Do: a surface
        counts when any part of it is inside the box. A chosen surface
        receives the whole shadow (cut only to that surface's own
        triangles, as above). **The box only picks surfaces; it does not
        cut the shadow at its edge.** Surfaces outside it get none, so a
        long shadow that runs past the box onto another surface is
        missing there (probably the cut the user saw; not reproduced,
        the user's decision).
      - **(d) Which surfaces never receive:** only surfaces with
        material number 255 (Castle D's 96, `object-lighting.md`) and a
        surface type that no `.dig` uses (all 172,783 surfaces have type
        0, data). Surfaces whose material is of kind 0 (nothing drawn,
        `models-and-textures.md`) **are not left out** by the original,
        so a shadow could float on an invisible surface there. Do
        instead: skip them (B's choice; Daventry has none).
    - **The 1,500 limit is a cut, not a fade** (user's check,
      2026-10-19): at the top of Connor's standing jump (about 1,697 above
      the ground, `connor-physics.md` "Jumping") his shadow disappears,
      and it comes back as he comes down. Nothing fades with distance.
    - **Over water the water surface receives it** (user's check): the
      drawn ground (the surface grid, `water.md`), not the floor under the
      water. When Connor wades, only the part of him above the water
      shows in the shadow (the parts below the surface do not reach it;
      how the game drops them was not traced: do the same, e.g. by
      projecting only points above the receiving plane).
    - **How dark:** a strength of **0.5** is passed to the renderer for
      every character's and object's shadow (the drawing code's own
      default is 0.75; a model-less path uses 0.8). That most likely means
      each layer halves the brightness of what is under it (two layers: a
      quarter), but the blending itself was **not traced**. No fading with
      distance or haze was found in the shadow code (not traced further).
    - **When:** only while the character is drawn (near or far, not
      hidden), and only when the cast test of rule 36 passes in that frame.
    - **Connor** goes through the same character code (his class is
      built on the characters' one, Q135), so all of the above applies to
      him, buildings included.
    - **Plain objects** (Q123 d): **the same**, through their own drawing
      code (Q135: its building search is a copy of the characters' one
      with the same numbers, 0.287, 500, 6, 1,800, using the plain
      object's own model ball): same direction (the sun's aim, there not made length 1, which
      only changes how far the terrain search reaches), the same 20-unit
      lift and 0.5 strength. Default `shadowMask` 8 and the world mask 15,
      so every plain object without `shadowMask=0` casts one in Daventry
      (plain objects also need a model; not traced further).
    - **`shadowvisible=1`** (Q123 e; Daventry trees, stones, the
      fountain), and the script command `KQObject::showShadow <name> 0|1`
      which sets the same flag (no script uses it): it marks the objects
      whose shadows the game's **built-in relighting tool** bakes into the
      terrain light map (`terrain-lighting.md` rule 13, console
      `lightTerrain`). The shipped maps already contain them. **In play it
      does nothing**: it has nothing to do with the live shadows above.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| update only while | near or far (not off, not hidden) | traced |
| `Stop` random start | random frame, speed 0.9–1.1 | traced |
| default `flags` for characters | 0x10 (random `Stop` at Wait start) | traced |
| `birthDelay` default | 0 s (Daventry: goblin18, goblin20 1 s; RoadZ1 3, RoadZ5 3.5) | traced, data |
| starting movement names | current `Stop`, waiting `Stop` (until a key replaces them) | traced |
| collision type / mask default | 0x42 / 0x281F | traced |
| flying mover | steps 0, no gravity; down 150 (all or nothing), up up to 300 per update | traced (Q129) |
| shadow direction | the sun's aim, length 1; (1, 1, −1) with no sun | traced |
| shadow lift / strength | 20 units / 0.5 | traced |
| shadow on terrain only within | 1,500 of the ground under the position | traced |
| shadow reach size s | 0.287 × the shown model's stored ball radius (Connor standing 252) | traced, data |
| building searched when | centre distance ≤ building radius + s + 500 | traced |
| shadow height band on buildings | s above to 6 × s below the model ball's centre; at most 1,800 tall | traced |
| shadow area on buildings | rectangle around a 2 × s square at the band top and its copy moved along the sun to the feet; each side at most 1,800 (cut around its middle); turns with the character | traced |
| surfaces left out | material 255 only (kind-0 materials not left out; we skip them) | traced, data |
| automatic turn rate | 3.5 rad/s (`rotateSpeed` 1.75 × 2) | traced |
| turn toward Connor in Talk | every 4th update | traced |
| sight test | every 15th update; eye at its box top (corrected), target 1,250 above Connor's | traced |
| sight test distance limit | none | traced |
| `wanders` default (`.kq` without the key) | 1 | traced |
| gravity default (`monUseGravity`) | on | traced |
| step up / down | 325 | traced |
| `shadowMask` default | 2 (characters), 8 (other objects), 1 (Connor) | traced |
| world shadow mask | 1 built in, 15 from `Options.cs` | traced, data |
| mouth shapes | 0–7 (`lip0`–`lip7`), 8+ → `lip0` | traced |
| geometry mouth | 0/16 `rest`, 1 `f`, 2 `m`, 3 `r`, 4 `w`, 5 `ih`, 6 `ah`, 7 `e` | traced |
| Daventry characters | 22 `KQMonster` + 2 `KQGeometryTalker` files, 52 monster-list lines | data |
| Daventry gesture lip files ending in 16 | 91 of 93 | data |

## Data used

- Character `.kq`: `classType`, `animFile`, `idleAnimFile`, `currAnimName`,
  `nextAnimName`, `animPos`, `flags`, `lips`, `baseLip`, `monUseGravity`,
  `keepOnGround`, `stepUpAmount`, `stepDownAmount`, `flyingObject`,
  `noTurn`, `wanders`, `alarmRadius`, `homeRadius`, `chaseRadius`,
  `canAlwaysSeeConner`, `lineOfSightOffset`, `birthDelay`, `monsterType`,
  `translucency`, `hidden`, `shadowMask`, `groupsActive3DFar`,
  `originalBmp`, `alternateBmp`, `script`.
- `[<name>Lips]` `KQLips`: `lip0`–`lip7`.
- `<world>/kq/monsters.kq` (`KQMonsterManager`), `GAME/Kq/Monstats.mtl`
  (`KQMonsterTypeList`: type → `.kq` file).
- `.anm` lists: `stop`, `Birth`, `PreBirth`, `gesturea`–`gestureh`,
  `rest`, `f`, `m`, `r`, `w`, `ih`, `ah`, `e`, transitions, `endLoop`
  cues.
- `GAME/patch/BOUNDS.KQ` (boxes by type or name).
- Lip-sync files (`S*.*`), `MASK/Options.cs` (`setShadowMask 15`).

## Edge cases

- A character first in range during a cutscene starts its movements then,
  not at load (rule 11). Two characters loaded together but entering range
  at different times are not in step (and `Stop` is random anyway).
- `KQAnimPlayEvent` (or any movement request) sent before the first
  update runs the start-up at that moment (rule 12), even for a character
  that is off or hidden; the request is then handled as usual (it starts
  at once if the starting movement can be cut short).
- A character in Talk that a script makes play a one-shot (none in
  Daventry) holds its last frame until the conversation ends.
- A character moved by `move` while gravity is on drops to the ground from
  there in its next updates (as Connor, `objects.md` "(d)").
- The monster list's z of exactly −1 uses the ground; any other negative z
  is used as written and then raised by `keepOnGround`.
- A `KQGeometryTalker` that a script puts in Special or plays a movement on
  during a line: the next lip-sync code replaces that movement at once.

## Do not reproduce

- **Unknown states for a character** (`setState <monster> Jump`, `Push`,
  …): the original replaces its state by an empty one and the character
  then never returns to Wait by itself. Do instead: log and ignore the
  request. (No Daventry script does this.)
- **`KQGeometryTalker` with lip-sync codes 8–15 or above 16**: the original
  starts a movement whose name is left over from memory (undefined). Do
  instead: ignore those codes for geometry talkers. (Daventry: never
  happens, rule 28 data.)
- ~~"No state" after a waiting first movement~~ (withdrawn 2026-10-19: it
  never happens, rule 15).
- **The `endLoop` cue's double end** (rule 20): if nothing replaces the
  movement at the cue, its real end runs the end handling again (a second
  `EndLoop`). Only the Knight's and archon's `attack` have the cue, and it
  can be cut short, so Wait's `Stop` always replaces it at the cue. Keep
  it; log a second `EndLoop` if it ever happens.
- **The loose per-surface test for shadows on buildings** (rule 37 (2),
  Q135): the original means to test a ball around each surface against
  the search box, but a whole-number division makes the ball's centre
  the building's own origin (its radius is the surface's farthest corner
  from there). Surfaces inside the box always pass; some outside it pass
  too. Do instead: any part of the surface inside the box.
- **Shadows on invisible surfaces** (rule 37 (2) (d)): the original does
  not leave out kind-0 materials. Do instead: skip them.
- Keep the rest, including the random `Stop`, the return to `Stop` in Wait
  only, the `noTurn` block, and no turning movement.

## How to check

- **Our logs (B):** per character, when it enters the world: name, class,
  file, position and heading (after the ground rule and `keepOnGround`),
  animation list, box found (by type or by name, and which key). At its
  start-up: the update number, what triggered it (first update, coming in
  range, or a request and which), the first movement, its state after
  (Wait queued / Birth / kept), the `Stop` start frame and speed. Every later movement start with the reason (script
  event, Wait's return to `Stop`, gesture code, mouth code, Special list).
  Every mouth change: shape, texture replaced and texture drawn. Per
  second: how many characters are near / far / off, drawn, with shadow.
- **Numbers to expect (data):** the Wizard's `Stop` is `wizstop.kqs`, 31
  frames; the chickens' `chk_stop.kqs` 30 frames; the Uglybeast's
  `UG_STOP.KQS` 40 frames; the ravens' `Ravnstp.kqs` 46 frames (its
  `stop` sends itself `Cue 10` at phase 0 of every loop, which may make it
  `caw`, `ravencnr.cs`). Every Daventry character's first movement is
  `stop` (rule 14; the earlier "Wizard starts on `attack`" was wrong); the
  Wizard's face texture is `WF0.BMP` until he speaks. The Uglybeast's
  mouth poses never end (no `Stop` between codes, rule 27); after the
  pivot, the first frame logged as drawn shows `stand` (rule 18). The Girl stays at (100, 100) in a new game.
  No zombie is drawn. The Knight and the Henchman start at (0, 0).
- **In the original (user, by eye):**
  1. The Wizard idling in his house: he breathes in place (`Stop`), facing
     the way his file says (heading −1.57); he does not turn to follow
     Connor around the room, even while talking.
  2. The Uglybeast at its pond: it stands facing the pond; click it from
     its start area: it pivots round to face Connor (`pivot`), then
     stands; its mouth moves as a whole-model change while it talks.
  3. The chickens behind the farm: standing still pecking at first? Do
     they start walking about only after Connor has had a clear view of
     them (rule 29), or from the start? Are the three of them out of step?
  4. The graveyard: no zombie visible until they rise.
  5. Shadows: do the Wizard and the goblins have a shadow on the ground,
     and the chickens none? What does it look like (dark copy of the
     figure, round blob)?

  **The user's check (2026-10-19):** (1) the Wizard does not turn to face
  Connor, as rule 23 says. (2) The Uglybeast turns to face Connor, but it
  looks like a model movement: its heading stays the same while it turns,
  which fits rule 22 (`pivot` is a movement; the heading itself is only
  changed by `ugly.cs`'s `move … 3 … -1.57` at the pivot's end, together
  with the switch to `stand`, so it shows no jump). (3) Not visible:
  the chickens come into view only when they are already drawn, so
  whether they start walking only after being seen can't be told by eye
  (rule 29 stays traced only). (4) Zombies stay hidden in the ground and
  appear only after rising once Connor is close enough, as rule 16 says.
  (5) Shadows: a dark copy of the figure, overlaps darker (rule 36).
  **Later check (2026-10-19, Q120):** clicked from far away, the Uglybeast
  says the "too far" line; its animation changes from idle to a still pose
  with only the mouth moving, then back to idle (rule 27).

- **Still to check in the original (user, 2026-10-19 additions):**
  6. Shadow direction in Daventry: at a sunny spot, does a goblin's or
     the Wizard's shadow fall to one side (toward +y in world axes: the
     same side as the dark sides of hills, `terrain-lighting.md`) rather
     than straight under them (rule 37)?
  7. Shadow darkness: is one layer of shadow about half as bright as the
     ground beside it, and the overlaps about a quarter?
  8. Shadows on buildings: does a character standing on a building floor
     (e.g. a spriggan in the castle ruins) cast its shadow on the floor?
     Do plain objects (a barrel, a cart) cast shadows like characters?
  9. The two goblins `goblin18` / `goblin20` (rule 12): nothing to see
     (a one-second restart of `stop`), skip unless convenient.
  10. The bats (`flyingObject`, rule 8): do they hang still in the air at
      their place before Connor comes near, or drift down?

  **The user's check (2026-10-19, later):** (6) every character's shadow
  falls to one side, the same side as the hills' shadows: confirms the
  sun direction of rule 37. (7) About half as bright, about a quarter
  where parts overlap: fits strength 0.5 with each layer darkening again
  (rule 37; keep this look until the M7 improvement in `docs/DESIGN.md`).
  (8) Characters and barrels cast shadows on building floors **and
  walls**: confirms buildings as receivers (any of their surfaces in
  range, rule 37) and plain objects casting the same way. (10) Can't be
  told by eye: bats are only visible once Connor is close, which is
  already in range, so the flying mover's settling stays traced only.
  **And:** shadows don't fade with distance; Connor's shadow vanishes at
  the top of a jump and returns on the way down (the 1,500 cut); shadows
  lie on the water surface, and a wading Connor's shadow shows only his
  part above the water (rule 37).

## Open questions

1. What `flags` bit 0x20 (`flags=32` on the Wizard, the Girl, the Knight,
   the Uglybeast, the Unicorn) does: it drops the object's link to the
   terrain when it enters the world; the effect was not traced.
2. Shadows (rule 37): the exact blend of the strength 0.5 (the user's
   check: about half, a quarter on overlaps), which side of the model the
   facing test keeps, and how parts below a water surface are left out:
   not traced. (No fading, and the water surface receiving them: settled
   by the user's checks.)
3. ~~Wandering, idling, alarm, chasing, walking speed and the flying
   mover's movement: M5. The flying mover's settling (rule 8) is traced
   but not fully sure.~~ **Answered 2026-10-19** in
   `character-movement.md` (up to the first swing; the fight itself is
   the next study). Its findings differ from rules 8 and 29–31 here in
   the points marked "corrected" / "refined" there.
4. `KQObject::changeBmp` and `changeAnmFile` in detail.
