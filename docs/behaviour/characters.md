# Characters: drawing and animating people and monsters (not fighting)

Study session A, 2026-10-19. Traced in `Mask.exe` unless marked "data"
(read from the game's files), "inferred" or "not traced".

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
12. **The first update** (traced; refines `anim-scripts.md` rule 3 for
    characters):
    1. If `currAnimName` is empty and a waiting movement is set
       (`nextAnimName`, or a `KQAnimPlayEvent` that arrived before), that
       movement starts, and **nothing else happens**: the character stays
       in **no state** (rule 15).
    2. Otherwise the current movement starts (`currAnimName`, or
       **movement 0** if it is empty or unknown), with `animPos` and the
       bit-8 random start of `flags` as for objects. Then:
       - if the movement list has a movement called **`Birth`** (capitals
         ignored): the character goes into the **Birth** state (rule 16);
       - otherwise it is put into **Wait** after `birthDelay` seconds (a
         monster-list column; 0 for every Daventry character, so at once,
         through the event queue).
    3. The mouth is set to shape 0 (rule 25).
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
14. **Movement 0 is usually a pose, not `stop`** (data): in most Daventry
    lists movement 0 is `attack` (the Wizard's is `wizstop.kqs` at speed 0,
    a still frame; the Lady's is `down`, her sinking). It shows at most
    until Wait asks for `Stop`: at once when movement 0 can be cut short
    (`0 1`), at its end when it can't (`0 0`, e.g. the ravens' `attack`,
    `Ravnstp` played once). Characters with `currAnimName=stop` (the
    Knight, the Henchman, the Uglybeast, the Unicorn, the bow spriggans,
    `sprdump`) start on `stop` directly.
15. **No state** (traced): a character whose first movement came from step
    12.1 (Daventry: the **Shadow**, `nextAnimName=stop`; the **Lady**, whose
    `Init` sends `KQAnimPlayEvent up` before her first update) never gets
    the Wait state by itself. It just plays what it is told: a play-once
    movement that ends **holds its last frame** (no return to `Stop`), and
    the scripts ask for the next one (`lady.cs`: `EndLoop up` →
    `KQAnimPlayEvent stop`). It gets a real state only when something sets
    one (`KQMonster::setState`, a conversation, the end of a `speak`
    queue: `talking.md` rule 13's "the speaker goes to Wait").
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
19. **In Talk, nothing goes back to `Stop` by itself** (traced): the Talk
    state only does the ordinary end, so a gesture that ends holds its
    last frame until the next gesture or code 16 (`stop`, rule 28) asks
    for something else. The data relies on this: 91 of Daventry's 93
    lip-sync files with gesture codes end with a 16 (data).
20. **Transitions** work as for Connor (`anim-scripts.md` rule 4.2,
    `connor-movement.md`): Daventry's only character transition is the
    Girl's `cower` → `stop` through `cowerstop` from phase 0.88 (data).
    The `endLoop` cue (the Knight's `birthx` at 0.84, data) makes the game
    treat the movement as ended there (`anim-scripts.md` rule 9).
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
    whole-model mouth poses in `ugly.anm` / `unicorn.anm` (each `0 1`, about
    one frame). It **never gestures**. At the end of a line nothing
    returns it to `stop` (the "shape 0" of rule 25 is a texture change
    that does nothing for it): the last code of its lip-sync file decides
    (usually 0, `rest`); after the last line of a queue, the "speaker goes
    to Wait" of `talking.md` rule 13 asks for `Stop`, which starts at once
    (the mouth movements can be cut short). Codes 8–15 and above 16: see
    Do not reproduce.
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
    once). Only an active character:
    - **wanders** (if `wanders` is on: walks from point to point; M5),
    - **idles** (rule 30),
    - is **alarmed** when it can see Connor and is closer than
      `alarmRadius`: then chasing and fighting start (M5).
30. **Idling** (traced, not followed in detail): an active character with
    an `idleAnimFile` (Daventry: `goblin2`, `sprigan`, `sprigan2`) now and
    then plays a fidget from that list in Wait (about every 2 s plus a
    random part), then returns to `Stop` (rule 18). M5 with the rest.
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
32. **What B can show before M5** (our recommendation, not a game rule):
    every character in Wait, **standing in `Stop`**, never active. That is
    exactly what the original shows **until a character first sees
    Connor**; after that, wanderers walk about and enemies attack, which
    is M5. Zombies stay unborn (not drawn, rule 16). Turning toward Connor
    happens only through `speak`, conversations and scripts, which are
    built.

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
    - `allowShadows 0|1`: all on (default) or all off; only Barren and
      Castle of the Dead scripts use it (data).
    So in Daventry **the Wizard, the Girl, the ravens, the goblins,
    spriggans, zombies, bats, the Henchman, the Lady and the Uglybeast cast
    shadows**; the chickens, the pig, the Knight and the Shadow don't.
    `shadowvisible=1` (Daventry trees, stones, the fountain) is read but
    what it does was not traced. How the shadow is drawn was **not
    traced** (open question 2). **The user's check in the original
    (2026-10-19):** the shadow is a **dark copy of the figure** on the
    ground, and where parts of it overlap (e.g. an arm over the body) the
    ground gets **darker again**: each part darkens the ground on its own,
    with no merging. Copy that look until M7; drawing the shadow as one
    even shape is noted as an M7 improvement (`docs/DESIGN.md`). The
    direction of the shadow (from the sun's `tsLightAim` or straight down)
    was not traced or checked.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| update only while | near or far (not off, not hidden) | traced |
| `Stop` random start | random frame, speed 0.9–1.1 | traced |
| default `flags` for characters | 0x10 (random `Stop` at Wait start) | traced |
| `birthDelay` default | 0 s | traced |
| automatic turn rate | 3.5 rad/s (`rotateSpeed` 1.75 × 2) | traced |
| turn toward Connor in Talk | every 4th update | traced |
| sight test | every 15th update; eye 1,000 above feet, target 1,250 above Connor's | traced |
| sight test distance limit | none | traced |
| `wanders` default (`.kq` without the key) | 1 | traced |
| gravity default (`monUseGravity`) | on | traced |
| step up / down | 325 | traced |
| `shadowMask` default | 2 (characters), 8 (other objects) | traced |
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
- `KQAnimPlayEvent` sent before the first update to a character without
  `currAnimName` keeps it in "no state" for good (rule 15); one sent to a
  character with `currAnimName` waits as for objects.
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
- **"No state" after a waiting first movement** (rule 15): keep it (the
  Lady's and the Shadow's scripts rely on holding the last frame), but log
  it once per character.
- Keep the rest, including the random `Stop`, the return to `Stop` in Wait
  only, the `noTurn` block, and no turning movement.

## How to check

- **Our logs (B):** per character, when it enters the world: name, class,
  file, position and heading (after the ground rule and `keepOnGround`),
  animation list, box found (by type or by name, and which key). At its
  first update: the update number, the first movement, how it was chosen
  (12.1 / 12.2), its state (Wait / Birth / no state), the `Stop` start
  frame and speed. Every later movement start with the reason (script
  event, Wait's return to `Stop`, gesture code, mouth code, Special list).
  Every mouth change: shape, texture replaced and texture drawn. Per
  second: how many characters are near / far / off, drawn, with shadow.
- **Numbers to expect (data):** the Wizard's `Stop` is `wizstop.kqs`, 31
  frames; the chickens' `chk_stop.kqs` 30 frames; the Uglybeast's
  `UG_STOP.KQS` 40 frames; the ravens' `Ravnstp.kqs` 46 frames (its
  `stop` sends itself `Cue 10` at phase 0 of every loop, which may make it
  `caw`, `ravencnr.cs`). The Wizard's first movement is `attack` (a still
  `wizstop` frame) for at most one update before `Stop`; his face texture
  is `WF0.BMP` until he speaks. The Girl stays at (100, 100) in a new game.
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

## Open questions

1. What `flags` bit 0x20 (`flags=32` on the Wizard, the Girl, the Knight,
   the Uglybeast, the Unicorn) does: it drops the object's link to the
   terrain when it enters the world; the effect was not traced.
2. How shadows are drawn (rule 36): the look is known from the user's
   check (a dark copy of the figure, overlaps darker); their direction
   and what `shadowvisible` does are not traced.
3. Wandering, idling, alarm, chasing, walking speed and the flying mover:
   M5.
4. `KQObject::changeBmp` and `changeAnmFile` in detail.
