# Animated objects controlled by scripts

Study session A, 2026-10-17. Traced in `Mask.exe` unless marked "data"
(read from the game's files) or "not traced".

## Purpose

How scripts change what an animated object is doing while the game runs:
start another movement (`KQAnimPlayEvent`), hear when one ends
(`EndLoop`, `getEndLoop`), pause it (`pauseAnim`), make one-shot objects
that burst and remove themselves (`KQAnimOnce`, `setDeleteWhenDone`), and
play scripted animations on Connor (`KQMonster::setState Connor special
…`). Also three details of the script reader that these lines rely on.

Builds on: `objects.md` ("Animated objects at load", the `loop=` line),
`connor-movement.md` ("Starting and stopping", rules 1–6, the same
machinery for Connor), `clicking.md` rules 20–22 (`EndLoop`,
`getEndLoop`, `deleteObject`), `doors.md` (the script language),
`items.md` rule 9 (`NoName<n>` names).

Words used here:

- **Movement**: one of an object's named animations (`[movementN]`,
  listed in its `KQAnimLoopList`).
- **Phase**: how far through the animation, 0 at the start, 1 at the end.
- **Current movement**: the one playing now. **Waiting movement**: one
  that was asked for and will start as soon as the rules allow (at most
  one; a newer request replaces it).
- **Can be cut short**: the 6th field of the `loop=` line
  (`connor-movement.md`), 1 = a new movement may replace it at once.
- **Has ended**: a movement that doesn't repeat (5th field 0) has reached
  its end: phase 0.9999 or more playing forwards, 0 or less playing
  backwards.

## Rules

### Current and waiting movement

1. Every animated object keeps its current movement's name, an optional
   waiting movement's name, and the running animation (phase, speed,
   repeat, can be cut short).
2. **`.kq` keys.** `currAnimName` is the current movement and
   `nextAnimName` is the **waiting movement** (the same slot that
   `KQAnimPlayEvent` fills, rule 4). Saved games also store
   `preTransAnimName`, `postTransAnimName`, `nextAnimStartTime`,
   `postTransAnimStartTime` (a transition in progress) and `notifyObject`
   (the `getEndLoop` listener). Those four transition keys appear in no
   data file (data); they matter for saving (M6).
3. **At load** (refines `objects.md` "Animated objects at load" rule 3):
   - **Correction 2026-10-19 (traced, found while answering Q121; it
     replaces the bullets on a missing `currAnimName` below):** every
     animated object starts with current movement name **`Stop`** and
     waiting movement name **`Stop`**. `currAnimName` and `nextAnimName`
     replace them; a missing key leaves `Stop`. So:
     - **Without `currAnimName`** the object starts on its **`stop`**
       movement (capitals ignored) if its list has one, otherwise on
       movement 0 (with `Stop` as its current name). The current name is
       never empty (no file sets an empty `currAnimName`, data), so the
       "empty name, waiting movement starts" step below never happens.
     - **`flags=8` objects without `currAnimName`** (Daventry `davtorch`,
       `fishies`, `fishies2`, `fishies3`, `deadbird`, `deadbir2`, all with
       `nextAnimName=stop` and a `stop` movement) therefore **do get the
       random start frame and speed**: they start on `stop` through the
       ordinary start, which applies bit 8. They are **not** in step with
       each other (the consequence bullet below is withdrawn).
     - **Without `nextAnimName`** the waiting slot holds `Stop` until the
       first request that starts at once clears it. If an object's first
       movement is played once and ends before any request came, `Stop`
       (or movement 0 if there is none) then starts (rule 8.1). Data scan
       of all worlds: in Daventry only `tombdoor` (starts on a play-once
       `stop`, so it plays it twice); elsewhere 16 objects in Dead City,
       Gnome and Swamp (traps' `up`/`down`, `flower1`, …).
     - **Doors, chests and other starts (traced 2026-10-19, Q127):**
       (a) **doors (`KQDoor`) and chests (`KQChest`) are animated objects
       and also start with `Stop` / `Stop`** (`tombdoor` above is a
       door). (b) A door's own `open` / `close` and a chest's `open` are
       **the same ordinary request as `KQAnimPlayEvent`** (rule 4), and
       every such request **writes its own name into the waiting slot**
       first: the old `Stop` is gone whether or not the request starts at
       once. If it starts at once, the slot is then empty; if it has to
       wait, the requested movement is what waits. So at the end of a
       door's or chest's `open` nothing is waiting (unless another
       request came meanwhile): no `stop` or movement 0 appears, as the
       user's M3 checks saw. "Until the first request that starts at once
       clears it" above is therefore exact only for requests that start
       at once; any request replaces it. (c) A character's **Special
       list** items (and its idle fidgets, `character-movement.md` rule
       18) also go through that request (the Special state first marks
       the running movement "can be cut short", so they start at once and
       empty the slot). A `KQGeometryTalker`'s **mouth poses** do not:
       they start directly and **leave the waiting slot as it was**; this
       never shows, because the poses never end (rule 10a).
     - Objects whose list has `stop` but whose movement 0 is something
       else now start differently from the old rule: none in Daventry;
       9 special classes elsewhere (Dead City's `death` and tower
       skeleton, Ice World's dragon, Swamp's mandragore, Temple 4's
       `lucend`), all M5 or later.
   - With a `currAnimName` that names a movement: that movement starts
     (with `animPos`, as `objects.md` says).
   - Without `currAnimName` (or empty): the name lookup fails and
     **movement 0** starts, but the object's current-movement **name
     stays empty**. On the object's first update, an empty current name
     makes the **waiting movement start at once** (rule 5), replacing
     movement 0.
   - So an object with no `currAnimName` but with `nextAnimName=<m>`
     really starts with `<m>`. In the data this never shows: in all 309
     such objects in the 12 worlds, `nextAnimName` is either missing (15)
     or names movement 0 (293); one names no movement at all and stays on
     movement 0 (data). Keep the rule anyway: it is the same code as
     rule 4.
   - **Refined 2026-10-17 (Q85, traced):** in the shipped game nothing
     actually starts when the object enters the world (that start is
     behind a debug switch, `doPurging`, which is on from the start and
     no script changes). Everything starts in the object's **first
     update**, in this order: (1) if the current name is empty and a
     waiting movement is set, the waiting movement starts from its own
     `loop=` line (no random part); (2) otherwise, if still nothing is
     playing, the current movement starts (the `currAnimName` one, or
     movement 0 if the name finds nothing), with `animPos` and, if
     `flags` has bit 8, the random start frame and speed; (3) the phase
     then **moves on in that same update** (speed × elapsed time), so a
     start at load time does not use up the first update.
   - ~~**Consequence for `flags=8` without `currAnimName`**~~ (withdrawn
     2026-10-19, see the correction at the top of this rule) (Daventry:
     `davtorch`, `fishies`, `fishies2`, `fishies3`, `deadbird`,
     `deadbir2`, all with `nextAnimName=stop`): step (1) starts `stop`
     plainly, so step (2) never runs and the random part is **never
     applied**. These objects start at phase 0, speed exactly 1, all in
     step with each other. Objects with a `currAnimName` and bit 8 do get
     the random part.
   - **The bit-8 random part** (traced): start frame = round(random ×
     (frameCount − 1)), phase = that frame ÷ frameCount; speed = a random
     value from 0.9 to 1.1 that **replaces** the `loop=` line's speed
     (it does not multiply it), so it always plays forward even if the
     line says 0 or −1.

### `KQAnimPlayEvent` (starting a movement from a script)

4. `sendEvent <delay> <object> KQAnimPlayEvent <movement>` (exactly one
   word after the event name in all 465 uses, data). When the event
   arrives:
   1. `<movement>` becomes the **waiting movement** (replacing any older
      waiting one).
   2. **Transition check**, the same as Connor's (`connor-movement.md`
      rule 2): if the current movement lists a transition to it, the
      object waits for that transition's "from" point (rules 3, 5 and 6
      there). **No placed object has transitions** (no `transitionN=`
      line in any `.kq` file; they exist only in Connor's and monsters'
      `.anm` files, data), so for objects this step never applies.
   3. Otherwise, if the running animation **can be cut short** or **has
      ended**, the waiting movement **starts at once** (rule 5) and the
      waiting slot is cleared.
   4. Otherwise nothing happens now; it starts when the running
      animation ends (rule 8).
5. **Starting a movement:**
   - The name is looked up in the object's movement list, **ignoring
     capitals**; the first match wins. An **unknown name starts movement
     0** (the debug build prints `KQAnimObject::playAnim: '<object>'
     could not load anim '<name>'`; the shipped game is silent). The
     current-movement name becomes the name **as asked**, even when it
     was unknown.
   - The movement starts with **its own `loop=` line**: start phase,
     speed (1, −1 or 0), repeat, can be cut short. So `close` on a door
     starts at 0.93 playing backwards. (A start phase given by other code
     overrides the line's; `KQAnimPlayEvent` never gives one.)
   - It starts **afresh even if it is the movement already playing**
     (same as Connor's rule 4).
   - The drawn model switches to that movement's model at once.
6. **Stopped movements that can't be cut short wait forever.** A
   movement with speed 0 never moves its phase, so it never "ends". If
   its `loop=` line also says `0 0`, a request sent while it is current
   never starts. The data has 30 such lines (data), for example
   `bricks.kq` `down` (Daventry) and Gnome's bridges and fuse box. They
   are **final poses** that no script asks to change afterwards (checked
   for `bricks`).
7. **`Interruptable 0|1` cue** (`cueN=<phase> interruptable <n>`):
   when the phase passes it, the running animation's "can be cut short"
   becomes `<n>`. Used in 119 `.anm` files (Connor and monsters), never in
   a placed object's `.kq` (data). It decides rule 4.3 for those.

### When a movement ends

8. **Ordinary animated objects** (`KQAnimObject`, doors, chests and the
   rest; not `KQAnimOnce`, rule 11). When a play-once movement has ended:
   1. **First** the waiting movement, if any, starts (rule 5). (If a
      transition was in progress, `connor-movement.md` rule 6 applies
      instead; objects never have one.) With no waiting movement the
      object **holds its last frame**.
   2. **Then**, through the event queue, the object's own script gets
      `EndLoop <movement>` and its `getEndLoop` listener gets `Cue 300 0
      <movement>` (`clicking.md` rule 21). Both name the movement that
      **ended**, not the one that just started.
   So when a script reacts to `EndLoop open` by sending `KQAnimPlayEvent
   opened`, the object has already been on its waiting movement (if any)
   for the rest of that update; usually there is none and it holds the
   last frame of `open` until the event arrives.
   - **"Through the event queue" still means the same update, before
     the frame is drawn** (traced 2026-10-19, Q119). The game's event
     queue is sorted by time. Each frame, every event whose time has come
     is handled, in time order (first come, first served for equal
     times), and only then is the frame drawn. The 15-per-second update
     of every object is itself one of those events and updates all
     objects in one go. An event sent with delay 0 during an update (the
     end of a movement, `EndLoop`, `Cue 300`, a script's `sendEvent 0 …`,
     a cue's `sendEvent`) gets the current time, so it is handled right
     after that update finishes, before the next update and before
     drawing; what those handlers send with delay 0 is handled in the same
     pass. **Nothing sent with delay 0 waits for the next update.** For
     our build: handle the queue until it holds nothing for the current
     time, then draw.
   - The end of a movement is itself such an event: it is noticed in the
     object's update but handled just after the update of all objects.
   - The waiting movement of 8.1 starts **at once even if the movement
     that ended could not be cut short** (it has ended), and, as for
     rule 4, it starts afresh from its own `loop=` line.
9. The `endLoop` cue word inside movement data (3 lines, all in monster
   `.anm` files: Daventry's `knight` `attack`, the `archon`'s `attack`,
   data) makes the game behave as if the movement ended at that phase.
   Monsters only; M5. Details for characters: `characters.md` rule 20
   (the movement is not stopped by the cue; Q124).
10. A repeating movement never ends, so it never sends `EndLoop`
    (`clicking.md` rule 21) and never lets a waiting movement start
    unless it can be cut short.

**10a. A movement whose model has a loop duration of 0 never ends**
(traced 2026-10-19, Q120). The time per frame is the model loop's
duration divided by (frames − 1); when that is 0 (or less), the phase
does not move at all, whatever the `loop=` line's speed. So a
play-once movement on such a model holds its pose for good: it never
sends `EndLoop`, a waiting movement behind it starts only if it can be
cut short, and a character's Wait never brings `Stop` back after it.
Data: the Uglybeast's and the Unicorn's mouth poses (`ug_rest.kqs`,
`ug_f.kqs`, … duration 0.0, 2 frames). Check other models with
duration 0 the same way (B's model reader has the duration).

### `KQAnimOnce`: objects that play once and then report

11. Objects with `classType=KQAnimOnce` (64 files in 9 worlds; Daventry:
    `barelupx`, `brlupxcr`, `brlupxms`, `boardpx`, `boardx`, data) play
    like other animated objects, but **when their movement ends**:
    1. their script runs with `$2` = **`Die`**, the same call as an
       object killed in a fight (the scripts only test `$2`);
    2. their `getEndLoop` listener, if any, gets `Cue 300 0 <movement>`;
    3. if `setDeleteWhenDone <name> 1` was given, the object is
       **deleted** (as `deleteObject`, `clicking.md` rule 22).
    They do **not** get `EndLoop`, and a waiting movement does **not**
    start.
12. **Daventry's breakables (data, using rule 11):**
    - Barrel (`barelup.kq`, script `barelup.cs`), when it dies (hit in a
      fight, M5): the script reads its position (`KQObject::getLoc`),
      deletes it, plays `breakbox.aud` and runs `loadKQ barelupx.kq @
      $KQObject::loc`. The new `KQAnimOnce` plays `brl_xpl` (the barrel
      bursting) once from where the barrel stood. At its end its script
      (`barelupx.cs`) gets `Die`, deletes it and loads `barelded.kq`
      (the wreck) at the same place. The two other barrels
      (`brlupcr`/`brlupxcr`, `brlupms`/`brlupxms`) work the same way.
    - `brlupcr.cs` also throws four crystals out of the barrel
      (`items.md` rule 12).
    - The alchemist's boards (`boardup.kq`, script `boardup.cs`): when
      they die, unless `KQMonster::checkInventory Connor Dagger` or
      `… Connor Fists` gives 1 (`items.md` rule 10; then Connor only
      remarks on it; what Connor carries then is for the inventory and
      fighting studies), the script sets `Daventry::Alchemist::BoardsUp 0`, deletes
      them, plays `breakbox.aud`, loads `boardpx.kq` (`KQAnimOnce`,
      `drbordie`, the boards falling) at their place, adds a map icon and
      gives Connor experience (`KQStatsChangeRequestEvent 0 0 0 0 20`,
      rule 19). At its end `boardpx.cs` deletes it and loads
      `drborded.kq` (the boards on the ground).
    - The `@` in `loadKQ … @ …` means "no name" (`objects.md`), so these
      objects get an automatic `NoName<n>` name the first time one is
      needed (the counter of `items.md` rule 9); for `loadKQ` the name is
      given **at the `loadKQ` line itself** (traced 2026-10-17, Q90).
      `loadKQ` also sets the script variable `$ObjectName` to the new
      object's name (no script uses it, data). That name is what their
      script gets as `$1`, so `getLoc $1` and `deleteObject $1` work.
13. **`setDeleteWhenDone <name> 1|0`** (17 uses, none in Daventry,
    data) sets or clears rule 11.3. It only works on `KQAnimOnce`
    objects; for any other object (or an unknown name) it prints
    `setDeleteWhenDone: could not find object '<name>'` and does
    nothing. With the wrong number of words it prints its usage line.

### `pauseAnim` and `setAnimPos`

14. **`pauseAnim <name>` toggles**: the first call pauses the object's
    animation, the next call resumes it. While paused, the phase doesn't
    move, no cues fire, nothing ends and a waiting movement isn't
    started; the object's mover still runs. It prints `<name>, <phase>`.
    Unknown name: `pauseAnim:  could not find anim <name>`. Used twice,
    both on Daventry's `sprdump` (a monster in a short scene: the
    outhouse door's script and `sprdump.cs` each call it once, data).
15. **`setAnimPos <name> <phase 0–1>`** sets the phase directly and
    prints the same line. No script uses it (data).

### Connor's scripted animations: `KQMonster::setState Connor special <list>`

16. 195 calls in the scripts (data). `<list>` is one word of
    comma-separated items, for example
    `'AnimList=bricks.anm,AnimListOn,push'` or `kneel,kneelstop`. Connor
    enters the Special state and the items are handled **one at a time,
    left to right**:
    - `AnimList=<file>`: load that animation list (an `.anm` file like
      `Conner.anm`, e.g. `bricks.anm`, `elevator.anm`, `voice1.anm`).
    - `AnimListOn`: from now on, movement names are looked up in that
      list instead of Connor's own. `AnimListOff`: back to his own list.
    - `Inv` followed by an item name (the next item): Connor holds a
      model of that item. Not traced further.
    - `KillConnor`: Connor dies (his Die state).
    - **Any other word is a movement**: it starts at once, and handling
      **stops until that movement ends** (compared by name, ignoring
      capitals). Then the next item is handled.
    - When the list is used up, Connor goes back to standing (Wait).
    A script that needs to know when he has finished registers with
    `getEndLoop Connor <itself> 1` and reacts to `Cue 300 0 <movement>`
    (`bricks.cs` waits for `push`).
    **Heights in the Special state** (traced 2026-10-17, Q92 a): the
    Special state changes nothing about gravity, the step-up and
    step-down heights (325) or the ground and floor contact rules; they
    keep running every update as in any state. A movement moves him only
    by its own per-frame steps, added each update to where he is (as for
    walking); the first frame's step is 0, so nothing jumps at the start.
    `bricks.anm`'s `push` (`c_brks1.kqs`, 51 frames, 3.33 s, data) has a
    step of **0 in every frame**, so Connor keeps the height his physics
    gives him.

### The script reader (additions to `doors.md` "The script language")

17. **Single quotes switch `$variable` replacement off** until the next
    single quote. The quote marks themselves are dropped and words are
    still split at spaces. So `'AnimList=bricks.anm,AnimListOn,push'`
    reaches the command as `AnimList=bricks.anm,AnimListOn,push`. 234
    script lines use single quotes, nearly all for `setState … special`
    (data).
18. Also: **`;` ends a command** like the end of a line (used only in
    debugging `alias` lines, data); **`( … )`** is one word with its
    spaces, without the brackets (only inside `echo` text, data); **at
    most 40 words** per command, more gives `Console: Too many
    arguments` and the command fails.
    **`eval <words…>`** (traced 2026-10-18, Q103 a): joins its words
    (after the line's own `$` replacement) with single spaces into one
    new line and runs that line as a command, exactly as if it stood
    on its own line in the script. Because the line is read again,
    `$` words in it are replaced **a second time**, and single quotes
    and `;` count again. `$Console::Return` afterwards:
    - the inner line ran (a known command, or a script found by name):
      **the inner command's own result** is kept, so `if eval
      KQMonster::testMessage $3 $4 $5 $6 $7 1000 63 0 0 1` takes its
      branch exactly when the tuples match, the same as without
      `eval`;
    - the inner line failed (unknown command, syntax error, too many
      words): **1**, with the console's error message;
    - `eval` with nothing after it: **0** (counts as true in an `if`;
      no data does this; log it).
    It does nothing else. In the data (all worlds) it is used only as
    `if eval KQMonster::testMessage …` (337 lines) and `if eval exist
    <object>` (28 lines, none in Daventry; `exist` is a game command,
    "Console::Return is 1 if object (and Connor) exist" by its usage
    text, not studied). The words given to it in the data are plain
    numbers and names, so the second `$` replacement never changes
    anything there. B's stand-in (run the rest as a command, keep its
    result) is right.

### `KQStatsChangeRequestEvent` (answers `clicking.md` open question 3)

19. On an ordinary object the five numbers are **added** to its five
    stats, in text order (the event's first number to the first stat,
    and so on); the fourth is then capped at 100 (the event's own fourth
    number is also cut to at most 100 when it is read; negative values
    stay); if health (the first) drops **below 0**, the object's script
    gets `Die`. Connor adds them the same way and then applies his
    levels (for M5, traced, not checked in the game; refined 2026-10-17,
    Q86):
    - The **fifth number is experience** (his log line prints health,
      maximum health, experience, the next threshold and the level).
      The second is his hand-weapon number, the third his ranged-weapon
      number, the fourth his armour (shown on the bottom bar;
      `interface.md` rules 27–33, confirmed 2026-10-18).
    - The "below 0 → `Die`" check comes right after the adding, before
      any cap.
    - Then health is capped at 10 × the **old** level.
    - Then, **once** per event (no repeat): if experience is **strictly
      more than** the threshold 10 × 2^(old level − 1) (150,000 from
      level 15 on), experience drops by the threshold, the level goes up
      by 1, health is set to the new maximum (10 × new level, a full
      heal), the **second** stat +5, the **third** stat +2, the fourth +1
      (cap 100). (Corrected: this rule used to say third +5, second +2;
      the numbers are counted here in the order they are written in
      `stats=` and in the event.)
    - Example (`bricks.cs`, Connor at level 1 with 0 experience, +50):
      50 > 10, so level 2 with 40 experience, health 20 of 20. Exactly 10
      would not level him up. A leftover above the new threshold (40 >
      20) is only acted on at the **next** stats event of any kind, even
      one of all zeros.
    - Only Connor has the levels; monsters and other objects just add.
    - **Starting stats:** an object with no `stats=` line starts with all
      five at **0** (plain objects, `KQAnimObject`, `KQAnimOnce`,
      `KQMonster`); doors start with health 10,000 (`doors.md` rule 18).
      `snowmane` (a `KQMonster`) has `stats=10 10 0 0 0` (data).

20. **`Die` always carries a `$3`** (traced 2026-10-17, Q86 e): the kind
    of the **last weapon hit** the object took (`Normal`, `Projectile`,
    `Fire`, `Ice`), and `Normal` if it was never hit. So a death from a
    stats event gives `$3 = Normal` unless an earlier hit was of another
    kind. A `KQMonster` whose health drops below 0 does not call its
    `Die` script here; it switches to its dying state instead; its
    script gets `Die <kind>` only when the die movement has ended
    (`fighting.md` rules 30–32).

### Cutscene control: `handsOff`, `walkTo`, `pointAndCue` (traced 2026-10-17, Q89 a, Q93)

21. **`handsOff 1 [N]`** takes control away from the player:
    - Ignored while it is on: movement and turn key **presses**, Run,
      Jump, drawing a weapon or the special weapon, "Away", the
      first-person key, clicks in the world, choosing items in the
      inventory/GUI, and weapon or mode changes. Holding the mouse-look
      button doesn't grab the mouse (inferred). Cursor-change keys and the
      camera's own keys are not blocked. Whether the GUI's own inventory
      and menu keys are blocked is not traced.
    - At the moment it switches on, Connor is told "forward/back 0, turn
      0", as if every key were let go: walking and turning stop, Move
      becomes Wait, and Push ends at its next update (nothing asks for
      `push`, `pushing.md` rule 7). Mouse-look stops; fixed camera mode
      (F11) goes back to normal mode.
    - The game keeps track of which movement keys are down during
      `handsOff`: presses are swallowed, releases still count.
    - **Not touched:** the Special state and its list (scripted
      animations keep running), and jumps or falls already under way
      (inferred: they end normally unless a script changes his state;
      he can't start a new Fall from Wait or Move while it is on).
    - On screen: the bottom GUI panels are hidden (their shown/hidden
      state is remembered), a spinning-mask icon appears, and the mouse
      pointer changes (its exact look is not traced).
    - `N`, the "number of dropable frames", limits how much game time may
      pass in one drawn frame: normally up to **1 s** is caught up in one
      go; with `N` given, at most **(N + 1) / 15 s**. It only matters when
      the computer stalls (the game then runs slower instead of jumping
      ahead). Data: N = 0 once, 1 five times, 2 five times, 3 once; none
      in the bricks or watermill scripts. Log it only.
22. **`handsOff 0`** gives everything back **at once**: input on, the GUI
    panels come back if they were shown before, the mask icon goes, and
    the per-frame limit goes back to 1 s. Then **every movement key still
    held is sent to Connor again**, so a forward or turn key held through
    the cutscene takes effect at once, without pressing it again.
    `handsOff` is kept in save games.
23. (Engine side, no script command.) While Connor plays certain
    movements of his own the game sets a separate "input locked" switch
    with the same effect on movement and turn keys; when it clears (and
    `handsOff` is off) a held turn key is sent again. Which movements set
    it is not traced. (It is stored in save games as `animSetHandsOff`.)
24. **`KQConner::walkTo x y <who> <c> <d>`** (and **`runTo`**, the same
    but running) walks Connor to a spot and then cues a script:
    - It switches `handsOff` on by itself and puts Connor in Wait.
    - He first **turns to face** (x, y) with the automatic turn at
      **3.5 rad/s** (`connor-movement.md` "Turning" rule 2). The turn
      re-aims every update from where he is now.
    - **During the turn** (corrected 2026-10-18, Q94 c, traced): the
      automatic turn sets the same turn direction a held turn key sets,
      and his standing state reacts to it as to a key (`connor-physics.md`
      "Walk held against a wall", "What Wait does each update"):
      - first update of the turn: he stands, `Stop` (the movement Wait
        always starts with);
      - from the second update, if neither wall block is on: he goes to
        Move with the turn-on-the-spot movement **`Left`** (turning
        anticlockwise, heading growing) or **`Right`**;
      - one update later, still in Move, the walk takes over: `Walk`
        (`Run` for `runTo`) is asked for and the start point and flat
        distance are recorded (next point).
      So a turn of 1 update stays in `Stop`; in a longer one he plays
      `Left`/`Right` for one update and then walks the rest of the turn,
      curving. Example, the watermill (0.576 rad, 3 updates): `Stop`,
      `Left`, then `Walk` while the last 0.110 rad is turned.
    - **When the turn ends** he is put into Move again (a fresh start of
      the Move state, even if he is already walking): from then on he
      walks (or runs) **straight ahead**, with no more steering, and the
      start point and the **flat distance to (x, y) are recorded again**
      from where he is at that moment. He stops once he has gone that
      far from that point, even if something pushed him off the line.
      (Corrected 2026-10-18: the old text said he only switches to Move
      when the turn ends; true only for a turn of 1 update.)
    - On stopping he goes back to Wait (`Stop`) and `<who>` gets
      `KQCueEvent <c> <d>` (its script runs with `Cue`, `$3` = c).
    - **How the end is checked and how he stops** (traced 2026-10-18,
      Q98):
      - The distance is checked in Move's own step, which runs at the
        start of each update **before** that update's animation step
        moves him. So it compares the flat distance gone **so far**
        (after the previous update's move) with the recorded distance;
        he stops in the first update where it is reached or passed. He
        never stops short; he is past it by up to one update's step
        (exactly on it only by chance).
      - Stopping is the **same as letting go of the forward key**: he
        goes to Wait, and Wait asks for `Stop` the ordinary way, so
        `connor-movement.md` "Starting and stopping" rules 2–6 apply:
        `walk` keeps playing (and moving him) until the next listed
        "from" point, then the walk → stand transition of that point
        plays (`walkstopleft` from 0.45, `stopwalkstop` from 0.90,
        `walkstopright` from 0.95; for `runTo` the run's ones), with its
        own steps, then `stop`. There is no jump straight to `stop`.
        The head-on end of a scripted walk (above) goes through Wait
        the same way, so it also asks for `Stop` with the transitions.
      - **Same update (Q101, traced 2026-10-19):** when Move's step finds
        the end, it switches him to Wait **at once**, and Wait asks for
        `Stop` as it starts, so the request is made in that same update,
        before that update's animation step. The transition is chosen
        from `walk`'s phase at that moment (the phase the previous update
        left). This is what B built.
      - The cue is sent in the same update, through the event queue, so
        the cued script runs in that update after Connor's own update.
    - **Head-on during a scripted walk or run** (traced 2026-10-18,
      Q94 a): when the walk or run of `walkTo` / `runTo` is under way
      and he hits something head-on (the head-on test of
      `connor-physics.md` "Walls", walking or running alike, with the
      same exceptions), there is **no `crash` and no `Stop` with a wall
      block**. Instead: he is put at the contact (pulled back 25, as for
      a head-on stop), the walk is cancelled, the cue is sent (the same
      cue as on arrival), and he goes to Wait, which plays `Stop`. A
      glancing hit is handled as usual (he slides and walks on). The
      console command `walkExit` also ends the walk and sends the cue.
    - **A wall block already on** (Q94 b, traced): the walk still
      starts. The block only stops Wait from starting a movement by
      itself, so during the turn he stays in `Stop` (no `Left`/`Right`),
      and when the turn ends he goes to Move and walks anyway (Move's
      own updates don't look at the blocks). `walkTo` does not clear the
      block: it is still on after the walk, so when the script gives
      control back he stands until a key event clears it
      (`connor-physics.md` "What clears a block").
    - **`walkTo x y` with no object** (Q94 d, traced; 6 lines in the
      data, all in Dead City and the Swamp): he turns and walks as
      above, `handsOff` goes on, and **no cue** is sent at the end. With
      an object but without both cue numbers (4 or 5 words) the game
      prints "must have objName..." to the console; the walk still
      happens and no cue is sent.
    - `walkTo` never switches `handsOff` off; the script must.
25. **`pointAndCue x y z <who> <c> <d>`** turns Connor to face (x, y)
    (z is ignored) with the same automatic turn, and sends `<who>`
    `KQCueEvent <c> <d>` **the moment the turn ends** (at once if he
    already faces that way). Only his heading changes. What plays
    (traced 2026-10-18, as in rule 24): `Stop` in the first update,
    then (no wall block on) Move with `Left` or `Right` until the turn
    ends; in the next update nothing asks for a movement any more, so he
    goes back to Wait (`Stop`).
    **Right after a scripted walk** (traced 2026-10-18, Q98 b): if a stop
    transition (rule 24) is still playing when the turn starts, it is a
    `0 0` animation, so the request for `Left`/`Right` waits until it
    ends (`connor-movement.md` "Starting and stopping" rule 4). The
    turn itself does not wait: the automatic turn changes his heading
    every update whatever animation plays ("Turning during a start"),
    so the rest of the stop transition's steps are taken along the
    turning heading and his path **curves**. When the transition ends,
    `Left`/`Right` starts if the turn is still going, otherwise he goes
    back to Wait and `stop`. This is what B built for the watermill
    (about 440 units of curve during `walkstopright`, `connor-3m3-
    watermill.log`); the traced rules give the same thing, so the
    difference the user saw is not explained by Q98 (see Open
    questions).
26. **If a cue never comes, nothing rescues it**: there is no time-out.
    `handsOff` stays on and the camera stays where the script put it.
    Only the head-on bump or `walkExit` end a `walkTo` early.
27. **Daventry's watermill room** (`wtrmwalk.kq`, trap `walktp1`, script
    `wtrmwalk.cs`, data), first entry only (`$Daventry::Mill::FirstTime`
    0 → 1):
    - `On`: `setRun 0`, range 1000 2500, `setThirdPerson`, `handsOff 1`,
      `follow none`, camera and Connor moved, cue 1 after 1 s;
    - cue 1: `walkTo 190800 230100`, cue 2 when he arrives (about 2,740
      units from where he was put);
    - cue 2: `pointAndCue 188000 230000 9000`, cue 3 when he faces it;
    - cue 3: `follow Connor`, `addCamLoc`, cue 4 after 0.2 s;
    - cue 4: `flyToCamLoc` over 2 s, cue 5 after 2 s and cue 6 after 4 s;
    - cue 5: Connor speaks, `$Daventry::RopenHook::progress` = 2;
    - cue 6: camera and Connor moved, `follow Connor`, **`handsOff 0`**,
      `setThirdPerson`, `lookup.cs`.
    So the control comes back about 4.2 s after he finishes turning in
    cue 2. Only cues 2 and 3 wait for something other than a timer.
    (2026-10-18: with rule 24's correction he plays `Left` for one
    update in cue 1's turn and starts walking during its last update;
    the distance is measured again when the turn ends, so he still
    arrives at (190800, 230100).)

### Ending a jump from a script: `KQConner::endJump`, `KQConner::JumpDir` (traced 2026-10-18, Q96)

28. **`KQConner::endJump`** (no arguments; the game's usage text: "use
    when Connor is in the jump state to get him out of it"):
    - It only acts while Connor is in the **Jump state** (the `jump`,
      `runJump` or `backJump` animation still playing, before Fall).
      Otherwise it prints its usage text to the console, changes
      nothing, and `$Console::Return` is 0.
    - In the Jump state it does to the jump exactly what a head-on crash
      does (`connor-physics.md` "Jumping" rule 5, "At the moment of the
      crash"): **gravity back on**, **step up and step down back to
      325**, the jump animation marked as **cut short**, and the jump
      marked as **finished**, so that he may now leave the Jump state.
      `$Console::Return` is 1.
    - It does **not** change his state or his animation. He stays in
      Jump and the jump animation plays on until something sets another
      state. If nothing does, the jump ends as usual at the end of its
      animation (Fall starts, "Jumping" rule 4); meanwhile gravity is
      already pulling him down while the animation still adds its steps.
    - **Why scripts need it:** while a jump is not finished, Connor's
      Jump state refuses every other state, so a `KQMonster::setState
      Connor special …` sent during a jump is not carried out at once
      (it waits; when it is tried again was not traced). One exception:
      **while `handsOff` is on**, asking Connor for any state during a
      jump finishes the jump first by itself, the same way (gravity on,
      steps 325, cut short). `miltrap1.cs` does both (`handsOff 1`
      comes first), with the same result.
    - **No Fall afterwards:** once he has left Jump for another state
      (Special here), the jump animation's end no longer does anything
      (that check belongs to the Jump state), so no Fall follows from
      that jump. Gravity and the steps stay as `endJump` left them (on,
      325); the Special state doesn't touch them (rule 16).
29. **`KQConner::JumpDir`** puts in `$Console::Return`:
    - **1** if Connor is in the Jump state and his current movement is
      **`backJump`** (names compared ignoring capitals);
    - **0** in the Jump state with `jump` (standing or walking jump) or
      `runJump`;
    - **0** if he is not in the Jump state (also after a jump has crashed,
      or once it has ended and Fall has started), and the usage text
      "returns 0 if forward, 1 if backward" is printed to the console.
    Nothing else is looked at: not his facing, not the sails. It only
    tells which jump he made (`connor-physics.md` "Jumping" rule 2:
    `backJump` comes from jumping while backing up).
30. **Daventry's windmill tumble** (`miltrap1.kq` / `miltrap1.cs`,
    data; order of events traced). The trap box is x 187300–189850,
    y 228600–231400, **z 8630–10630**, about 1,000 above the ground
    there (7650), so only a jump into the sails reaches it. Like every
    trap touch, the `On` script runs **inside Connor's own move**
    (`traps.md`; the same as the `Push` call, `pushing.md` rule 11).
    First time only (`$Daventry::MillTree::down` 0):
    - `setThirdPerson`, `handsOff 1`, `follow none`;
    - `JumpDir` → `$Daventry::Mill::JumpDir`: 0 for a standing, walking
      or running jump, 1 for a back flip;
    - `milgear1` hidden;
    - `endJump`: gravity on, steps 325, the jump finished (rule 28);
    - `move Connor 0 188700 230000 7500 0 0 1.57`: below the ground
      there, so the ground raise puts him on it; because this runs inside
      his update, the rest of his update follows (`objects.md` "(d)");
    - `getDoMode` (4 with empty hands) picks the list: empty hands →
      `normal` (JumpDir 0) or `normbak1,normbak2` (JumpDir 1), with cue
      300 after **2.2 s** or **2.74 s**; the weapon branches the same
      way with other lists;
    - `KQMonster::setState Connor Special …`: allowed at once (the jump
      is finished), so Special replaces Jump: no Fall from the jump;
    - `move KQCamera 0 190227 231352 10230 -.55 0 2.58`;
    - cue 1 after 0.6 s (1.2 s for the back flip): Connor gets
      `KQAttackEvent` from `milstone` (damage, M5); the camera moves to
      (190380, 232400, 12500), angles (−0.86, 0, 2.58);
    - cue 300: `milgear1` shown, camera to (192780, 232150, 9255),
      angles (−0.2, 0, 2.585), Connor to **(191300, 229530, 8050)**
      heading −1.57, `follow Connor`, `handsOff 0`, `setThirdPerson`.
    - If the Special list ends before cue 300, he stands (Wait) where the
      list left him until cue 300 moves him (rule 16).
    - After cue 300's `move` he is in Wait with gravity on and steps
      325, 400 above the ground (7650): one gravity step at the move
      (`objects.md` "(d)"), then he drops by the normal rules
      (`connor-physics.md` "Gravity", "Falling off an edge") and can
      jump again once he stands.
    **B's build (Q96):** the mover staying in its jump state after the
    script is wrong. `endJump` (or the `handsOff` rule above) finishes
    the jump, Special replaces it, and gravity is on from then on.

### Distance tests: `checkDistance` (traced 2026-10-21, Q142)

31. **`checkDistance`** tells a script whether two things are within a
    given distance. Three forms, chosen by the number of words after the
    command (traced):
    - `checkDistance <obj1> <obj2> <maxDist>`: between two objects;
    - `checkDistance <obj1> <x> <y> <z> <maxDist>`: between an object and
      a point;
    - `checkDistance <x> <y> <z> <x2> <y2> <z2> <maxDist>`: between two
      points (no script in the data uses this form).
    - **What it measures:** the **straight 3D distance** (x, y and z
      together, not level distance) between the objects' **positions**:
      the point each object is placed at (for Connor, his feet,
      `connor-physics.md`; for a placed object, its `loc` as it stands
      now, moved or not). Not their boxes, balls or middles.
    - **The result** in `$Console::Return`: **1 if the distance is at
      most `maxDist`** (exactly equal counts as within), **0 if it is
      farther**. It also prints the distance to the console.
    - **Errors**, each prints a console line and leaves **0**: an object
      name that isn't found ("can't find obj1/obj2"), an object whose
      position can't be read, or a wrong number of words (the three usage
      lines). Objects are found by name the same way `move` finds them.
    - **Daventry's axe** (`ax.cs`, data + the rule above): clicking the
      axe runs `checkDistance Connor ax 1700`; 1 gives the Small Axe and
      the cutscene, 0 makes the Henchman speak (`1000 9 14 18`). The axe
      hangs at (145910, 249914, 17670). From the windmill floor (about
      15,045–15,195) his feet are at least 2,475 below it, so he can
      **never** reach it from the floor. On the hay bale (placed at
      z 15,195, top 990 higher by `BOUNDS.KQ`'s `haypush.kqs`) his feet
      are about 16,185, 1,485 below the axe, so it works when his feet
      are within **about 830** of the axe's spot sideways (x, y)
      [inferred from the numbers; matches the user's check: not from
      the floor, yes from the bale].
    - Other uses (data): 28 scripts in 9 worlds, all with the first two
      forms; Connor to an object or a point, and also object to object
      (`girltrap girlbx.02`, `booktrap pushbook`, the boulders, the Gnome
      armoury's stands). None needs anything more.

### A script's walk during a weapon swap (Q141, 2026-10-21)

32. `ax.cs` cue 5 runs `KQConner::setDoMode 0` (the weapon is put away
    and the axe drawn: `ranged-weapons.md` rules 6–9), then `move Connor
    0 …`, then `KQConner::walkTo …` in the same event:
    - **The `move`** places Connor at once at (142879, 250000, 15325),
      heading −1.57 (the door side), in the middle of the swap: a script
      `move` is a placing, not a state change, so the swap doesn't stop
      it and it doesn't stop the swap (`objects.md`, `traps.md` rule 5).
      The swap's animations go on at the new spot. Its use here: the
      walk then starts from a known spot and heading.
    - **The `walkTo`** (rule 24) asks for standing; the swap refuses it
      (`ranged-weapons.md` rule 7) and the request is kept as
      **pending** (`connor-physics.md` "Falling off an edge": a refused
      request is tried first at the next state change). The automatic
      turn toward the target starts at once on its own. When it ends,
      the request becomes "walk", still pending. When the swap's
      last movement ends, the pending request is applied: **he walks
      after the swap** [traced]. That matches the user's check in the
      original ("he stops, draws the axe, then walks outside") and
      B's build.
    - **It is the same for any swap**, whoever started it (a slot click,
      a key, a script): the swap state refuses every request until its
      end, and the pending one is applied then [traced: nothing in this
      path depends on who started the swap]. Whether the automatic turn
      actually turns him while the swap plays was not traced. Here it
      hardly matters: from the `move`'s spot and heading the target is
      only about 0.16 rad off.
    - If the walk's request is replaced by another request before the
      swap ends (only the latest pending request is kept), the walk is
      lost and its cue never comes (rule 26) [inferred].
33. **`KQObject::setScript <name> <script.cs | none>`** (traced
    2026-10-22, Q154). Works on **any object with a script slot**:
    plain objects, characters, doors, animated objects, items,
    clickables. The object is found by name (or by number id) like the
    other object commands.
    - The new script is called **at once**, inside the command, with
      `$1` = the object's name and `$2` = **`Init`**, the same call an
      object gets when it is added to the world.
    - The old script is told nothing. It already had its own `Init` at
      load, and nothing undoes what that did.
    - `none` removes the script, with no call.
    - An unknown name does nothing and writes nothing; a wrong number
      of words writes the usage line.
    - An object with no name first gets an automatic one
      (`NoName<n>`).
    - The script name is kept as the object's ordinary `script` key, so
      it is saved with the object wherever the object's state is saved
      (whether a left world keeps its objects is `world-change.md`'s
      question).
    - Daventry (`newWorld.cs`): `sprigan` and `bossprig` get their own
      scripts right after `loadKQ monsters.kq`. So each gets **two**
      `Init`s: first its type's script's (`sprgnbow.cs`), then the new
      one's. Ours must call the new script's `Init` too.
34. **`loadObject <name> <file> [x y z [rx ry rz]]`** (traced
    2026-10-22, Q157) makes a new object in the world.
    - **The words after the file:** up to six numbers: the position,
      then three turn angles; missing numbers count as 0. Fewer than two
      words (a name and a file) writes the usage line.
    - **`$KQObject::loc`** (from `KQObject::getLoc`) holds all six
      numbers: the position and the object's three turn angles.
      `$KQObject::locOnly` holds only x y z. So `loadObject deadsprg
      sp_cadth.kqs $KQObject::loc` puts the corpse where Connor stands,
      **turned the way he faced** at that moment (in `sprgnbow.cs` he is
      turned 1.57 rad on the next line, after the corpse is placed).
    - **Heights** (inferred, not checked): a z of exactly −1 or −2 is
      replaced by a ground height, as in other placings.
    - **What kind of object:** picked by the file's extension.
      `.kqs` or `.dts`: a **plain object** with no animation player.
      `.dig`: a building (not studied further). Anything else: the line
      "unknown object '<extension>'" and nothing is made.
    - **The frame drawn:** every shape that the shipped scripts load
      this way (`sp_cadth`, `sp_dgdth`, `ap_mtrps`, `c_kodeth`,
      `grateded`) has a single pose [data], so it is drawn still.
    - **The name:** used as given. A name starting with `@` gets an
      automatic name (`NoName<n>`). Nothing checks for a duplicate name.
    - **Collision:** type 0x41, but no collision shape is built (no
      collider setting) [inferred from the defaults], so it is **not
      solid and can't be clicked** in practice. The same as the corpses
      of `fighting.md` rule 32, which are made the same way.
    - **No script**, so no `Init`; no sound.
    - **Saving:** an ordinary world object, very likely saved like the
      others [not traced].

35. **`sendEvent` in a movement's cue, and `@`** (traced 2026-10-22,
    Q158 a). A cue's text is `sendEvent <target> <Event> <numbers…>`:
    **no delay word** (unlike the script command); the event is queued
    for the current time, so it is handled right after the update
    (rule 8). If the text contains `@`, everything before the `@` is
    dropped and the `@` is replaced by **the name of the object playing
    the movement**; the rest is read the same way. So `cue0=0.000000
    sendEvent @ KQCueEvent 10 0` in `raven.anm` sends `Cue 10 0` to the
    raven itself, and `sendEvent @ KQMoveEvent 3 …` moves the object
    playing it. The target is looked up by name, so with two objects of
    the same name the first one found gets it [inferred].
36. **`random [min] [max]`** (traced 2026-10-22, Q158 b) puts a whole
    number in `$Console::Return` (as other commands' results):
    - `random` alone: 0 to 100; `random <max>`: 0 to max; `random <min>
      <max>`: min to max. Four or more words write the usage lines (the
      result is then 0, inferred).
    - The numbers are read as whole numbers (anything after a decimal
      point is ignored).
    - Result = min + **round**(u × (max − min)), u even between 0 and 1.
      So both ends are possible but come up **half as often** as each
      value between: `random 0 5` gives 0 and 5 about 10 % each, 1 to 4
      about 20 % each.
37. **Daventry's ravens** (data with rules 35, 36 and `objects.md`
    "Loops" rule 4; traced 2026-10-22, Q158 c). `raven.anm` `stop`
    (`Ravnstp`, 46 frames over 3.0 s, repeating, can be cut short) has a
    cue at 0.0 that sends the raven `Cue 10 0`; `ravensrh.cs` /
    `ravencnr.cs` then run `random 0 $Daventry::Raven::cawInterval` (5)
    and, if the result is 1 (about 20 %), `KQMonster::setState $1
    Special caw`.
    - The cue fires once per pass, one update after the wrap (a pass:
      46 updates, about 3.07 s at speed 1).
    - Their `stop` usually does not start at 0: characters start
      `Stop` at a random frame with speed 0.9–1.1 (`characters.md`,
      character flags 0x10 by default). So the first roll comes at the
      first wrap (or at once if the random frame is 0). After `caw`
      ends, standing starts `Stop` at a random frame again.
    - On average a raven caws about every 15 s plus the caw's own length
      [inferred from these numbers].

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| "has ended", forwards | phase ≥ 0.9999 | traced (`objects.md`) |
| "has ended", backwards | phase ≤ 0 | traced |
| cue number to a `getEndLoop` listener | 300 | traced (`clicking.md`) |
| words per script command | at most 40 | traced |
| fourth stat cap | 100 | traced |
| Connor's maximum health | 10 × level | traced |
| experience for the next level | 10 × 2^(level − 1); 150,000 from level 15 | traced |
| `KQAnimOnce` files | 64 (Daventry 5) | data |
| stopped `0 0` movements | 30 `loop=` lines | data |
| `walkTo`: updates of `Stop` before `Left`/`Right` in the turn | 1 | traced |
| `endJump`: gravity / step up and down afterwards | on / 325 | traced |
| windmill tumble: cue 300 after | 2.2 s (`normal`), 2.74 s (back flip) | data |
| windmill trap box height | z 8630–10630 (ground 7650) | data |
| `checkDistance`: distance kind / result | straight 3D between positions; 1 if ≤ max, 0 if farther or on any error | traced |
| the axe (`ax.cs`): max / axe's place | 1,700 / (145910, 249914, 17670) | data |
| the axe from the bale: feet below the axe / sideways reach | about 1,485 / about 830 | inferred from data |

## Data used

- `.kq`: `classType` (`KQAnimObject`, `KQAnimOnce`), `currAnimName`,
  `nextAnimName`, `animLoops`, `[movementN]` `loop=` lines, `script`.
- `.anm`: `Conner.anm` and the special lists (`bricks.anm`, …):
  movements, transitions, `cueN=… interruptable n`, `cueN=… endLoop`.
- Scripts: `sendEvent … KQAnimPlayEvent`, `getEndLoop`, `pauseAnim`,
  `setDeleteWhenDone`, `KQMonster::setState Connor special`, `loadKQ … @
  $KQObject::loc`, `KQStatsChangeRequestEvent`.

## Edge cases

- A request for the movement already playing restarts it from its
  `loop=` start (rule 5), even a looping one: a visible jump if it was
  mid-cycle.
- Two requests before the first could start: only the second is kept
  (rule 4.1).
- A request to an object whose current movement is a stopped `0 0` pose
  is never carried out (rule 6).
- A `KQAnimOnce` given `KQAnimPlayEvent` while playing: 62 of its 64
  movements in the data are `0 0`, so the request waits for the end, and
  then rule 11 runs instead (the request is never played). With the two
  `0 1` ones it would start at once and rule 11 would follow its end. No
  script does this (data).
- `pauseAnim` sent twice resumes; sent to an object already paused by
  another script, it resumes it.
- An unknown movement name never fails visibly: movement 0 plays.
- `KQConner::endJump` or `JumpDir` outside a jump: a console message,
  nothing else (rules 28–29). A trap like the windmill's touched during
  the Fall that follows a jump (not in Daventry's trap: the sails are
  reached on the way up) would get JumpDir 0 and an `endJump` that does
  nothing; the Special state is then asked for from Fall (not traced
  further).
- A scripted walk whose turn is interrupted by a player turn key: only
  a key **press** would cancel it, never a held key (corrected
  2026-10-18, Q104 a, `connor-movement.md` "Turning" rule 2); with
  `handsOff` on, key presses don't reach him, so this can't happen in
  the scripts' cutscenes.
- **`pointAndCue` has no lock of its own** (traced 2026-10-18, Q104 a):
  unlike `walkTo` it does not switch `handsOff` on and doesn't look at
  Connor's state. If a script used it without `handsOff`, a turn key
  press during the turn would cancel the turn and the cue would **never
  be sent** (the cue goes out only when a turn ends normally). Every
  use in the data that the player could interrupt sits after a
  `handsOff 1` or inside a cutscene (data, not checked one by one).
  Recommended: log "pointAndCue turn cancelled, cue lost" if it ever
  happens. A click turn or `speak` turn that starts during a
  `pointAndCue` turn replaces its target; the cue is then sent when
  that turn ends.

## Do not reproduce

- **`boardx.kq` would load itself forever**: its script `boardx.cs`
  answers `Die` by loading `boardx.kq` again. No script loads `boardx.kq`
  (the real chain is `boardup` → `boardpx` → `drborded`, data), so this
  never happens in play. If it is ever loaded, stop after one round and
  log it.
- Keep the rest, including the silent fallback to movement 0, but log
  every unknown movement name and every request swallowed by rule 6, so
  data mistakes show up in our logs.

## How to check

- **Our logs (B):** for each `KQAnimPlayEvent`: object, asked name,
  current movement, "started at once" / "waiting" / "waiting forever
  (rule 6)", and the start phase and speed used. For each end: the
  movement that ended, the waiting movement that started (if any), then
  `EndLoop` / listener `Cue 300` / `KQAnimOnce` `Die` and deletion, with
  the update number.
- **Daventry, by script (no fighting needed):** `bricks.cs` `Cue 300 …
  push` sends `KQAnimPlayEvent down` to `Bricks` (current `stop`, a
  `1 1` loop, so it starts at once at phase 1.0, speed 0, and stays) and
  `KQAnimPlayEvent blocked` to `water`. The log should show both starting
  in the same update, with no `EndLoop` from `Bricks` afterwards (`down`
  never ends, rule 6).
- **Windmill tumble (our logs, B):** `cargo play --world Daventry --at
  191200,229700,-1.5708 --jump 20 …` (the user's standing jump into the
  sails). `traps.log`: `JumpDir` 0, `endJump` "jump finished: gravity
  on, steps 325", `setState … normal` carried out at once. `connor.log`:
  state `special` (not `jump`) from the trap's update, gravity true;
  after cue 300 (33 updates after the trap) z 8050 → 7650 within a few
  updates, then state Wait, and `--jump` works again.
- **Watermill walk (our logs, B):** cue 1's turn: one update `Stop`,
  one `Left`, then `Walk` while the turn finishes; "walk distance
  measured again" at the turn's end; arrival at (190800, 230100) as
  before (rule 24).
- **End of the watermill walk (our logs, B, Q98):** the update the walk
  ends: distance gone ≥ distance recorded (never less), state Wait,
  `Stop` asked for; `walk` plays on to the next from point, then the
  stop transition of that point with its steps; during cue 2's turn the
  path curves while the transition plays, and `Left` only starts after
  it. Compare with the original (user): where Connor stands when the
  camera starts its flight (cue 4), against a fixed object in the room.
- **In the original (user), once fighting works or with a weapon:** break
  the alchemist's boards. Expected: the boards fall (one animation), then
  lie on the ground; the falling animation plays once and doesn't repeat.
  Barrels likewise: burst, then the wreck stays.
- **The axe (rule 31), ours against the original:** click the axe from the
  windmill floor: the Henchman's remark, no axe; push the hay bale under
  it, stand on the bale, click: the axe and the cutscene. Our log should
  show each `checkDistance` with both positions, the distance and the
  result (on the bale near the axe: about 1,485–1,700; on the floor:
  2,475 or more).
- **The swap then walk (rule 32):** in the axe cutscene Connor stands
  where the `move` put him, puts his weapon away and draws the axe, then
  walks out; the walk's start should be logged after the swap's last
  movement ends (checked by the user in the original, 2026-10-21).

## Open questions

1. ~~What sends `Bricks` its `Push` call~~ Answered 2026-10-17 in
   `pushing.md` (rules 1–3, 9–11).
2. When a movement starts, the object's collision type bit 0x400 is
   switched on or off depending on something about the new movement;
   what it means was not traced.
3. `Inv <item>` in Connor's Special list (rule 16): which hand, which
   model, and how long he holds it.
4. `KQMoveEvent` with flags 1 and 3 on objects (Barren's tiles, one Dead
   City box; not Daventry): not traced for objects (`objects.md` gives
   the camera's meaning).
5. The `endLoop` cue on monsters (rule 9) and monsters' own movement
   requests: M5.
6. **The watermill walk's end (Q98, 2026-10-18):** the traced rules
   (rules 24–25) match what B built, yet the user sees Connor "turn left
   and walk forward a bit more than the original". Not explained. Things
   to compare before studying further: (a) the phase of `walk` when the
   walk ends in our build against the step cycle (the next from point
   decides which stop plays: `walkstopright` from 0.95 is short, but a
   phase just past 0.95 would wait a whole cycle for `walkstopleft` at
   0.45); (b) the stop transitions' own steps (`.kqs` frame steps, as
   for the start animations in `connor-movement.md`); (c) the end
   position in the original, measured against something fixed.
   **Closed 2026-10-19 (Q101):** the one open point (same update or the
   next) is traced: same update, as built. The user then found that
   walking into the room, as one must in the original, matches the
   original; the extra walk only shows when our test starts Connor
   inside the trap (`--at 192408,227039,0`), a start the original
   can't make, so his walk phase at the end differs. Not a difference
   in normal play, so nothing is left to study for M3.
