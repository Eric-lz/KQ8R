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
9. The `endLoop` cue word inside movement data (3 lines, all in monster
   `.anm` files: Daventry's `knight`, the `archon`, data) makes the game
   behave as if the movement ended at that phase. Monsters only; M5.
10. A repeating movement never ends, so it never sends `EndLoop`
    (`clicking.md` rule 21) and never lets a waiting movement start
    unless it can be cut short.

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
      needed (the counter of `items.md` rule 9; the moment of naming is
      not traced separately for `loadKQ`). That name is what their
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

### `KQStatsChangeRequestEvent` (answers `clicking.md` open question 3)

19. On an ordinary object the five numbers are **added** to its five
    stats; the fourth is then capped at 100; if health (the first) drops
    **below 0**, the object's script gets `Die`. Connor adds them the
    same way and then applies his levels (for M5, traced, not checked in
    the game): the **fifth number is experience** (his log line prints
    "hlth … expr … lvl"); health is capped at 10 × level; when
    experience passes 10 × 2^(level − 1) (150,000 from level 15 on), it
    drops by that amount, the level goes up by 1, health is set to the
    new maximum, the third stat +5, the fourth +1 (cap 100), the second
    +2.

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
- **In the original (user), once fighting works or with a weapon:** break
  the alchemist's boards. Expected: the boards fall (one animation), then
  lie on the ground; the falling animation plays once and doesn't repeat.
  Barrels likewise: burst, then the wreck stays.

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
