# Clicking things: what the click line hits, accepting items, scripted objects

Study session A, 2026-10-15. Sources: `Mask.exe` (studied, described here
in our own words) and the game data of all 12 worlds. "Traced" = read in
the exe; "data" = read from the files; "inferred" = follows from the two
but was not seen in the exe. Answers `QUESTIONS.md` Q62 (a), Q63 and Q64.
Updated 2026-10-16 (Q75: reach to a picked building, rule 9).
Builds on `doors.md` (rules 10–16: picking, reach, the `ConnorAction`
call, "too far"), `items.md` and `chests.md`.

## Purpose

A left click with the play cursor sends a line from the camera through
the mouse point into the world. The first "clickable" thing on that line
is the one Connor acts on. This note says what counts as clickable (and so
what can stand in the way), how an object decides whether it takes the
item Connor holds, and which script calls a clickable animated object
gets.

## Rules

### The click line (traced)

1. The line starts at the camera's eye and runs through the mouse point
   to a fixed length set on the camera's view (most likely its drawing
   distance; which setting it is was not identified).
2. The line is tested against **every collision shape whose collision
   type has bit 0x20** (`colliderType`, "clickable"), and against nothing
   else. The **nearest** hit along the line wins. Each shape is tested by
   its kind: boxes (objects, items, doors, chests) by their `BOUNDS.KQ`
   box, as it stands now (turned and moved with the object); buildings by
   their `.dig` BSP solid space (the same line test the camera uses,
   `camera.md` "Obstacles").
3. **Buildings are clickable by default.** A building (`KQInterior`) gets
   collision type **0x28** unless its file says otherwise, and 0x28
   contains the clickable bit 0x20. So a wall, floor or ceiling between
   the camera and an object **is hit first and takes the click**. This is
   why the user can't click through a wall or a ceiling (Q63).
4. Some buildings switch it off with `colliderType=136` (0x88: solid for
   the camera's view, not clickable). They are **not** in the way of a
   click. Daventry: `cave`, `church`, `wizhouse` (all three loaded by
   `permwrld.cs`). `zedrock1` and `zedrock2` also say 136, but no script
   loads them (corrected 2026-10-15). The other 28 Daventry building files
   are clickable. Counts of building files in all worlds (data; not every
   file is loaded): Barren 21 clickable / 4 not, CastleD 0 / 5, DeadCity 32 / 4,
   Gnome 22 / 0, IceWorld 8 / 2, SnowExit 1 / 1, Swamp 11 / 1, Temple1
   4 / 10, Temple2 14 / 0, Temple3 7 / 0, Temple4 2 / 0.
5. **Other things in the way** work the same: any object with bit 0x20
   in front (a door, a chest, an item, or one of Daventry's 102 object
   files with `colliderType=33`, 0x21) is picked instead of what is
   behind it. Objects without bit 0x20 (the default type 0x41: trees,
   plain props) never stop the line.
6. **The ground is never tested.** Neither the drawn ground nor the floor
   grid under water stops a click. By the code, a clickable object behind
   a hill can be clicked if the line reaches it without passing through a
   clickable building or object (not checked in the original; see "How to
   check").
7. **Deleted objects** (`deleteObject`, rule 22) are skipped. **Hidden
   objects are not skipped**: hiding changes only the drawing
   (`objects.md` "Hidden objects", `connor-physics.md`), so a hidden
   object that has bit 0x20 can still be clicked where it stands (traced;
   not checked in the original).
8. **An object with no box can't be clicked.** The box comes from
   `GAME/patch/BOUNDS.KQ` when the object enters the world (shape name
   first, then the object's own name, `connor-physics.md`). With neither
   in the table it gets no collision shape at all: not clickable, not
   solid.

### What happens to the picked thing (traced)

9. Reach and everything after it are unchanged: `doors.md` rules 11–16.
   The distance is from Connor's feet to the picked thing's box, against
   2 × Connor's `attackDist` (4,700).
   **For a building** (traced 2026-10-16, Q75) the box is the **`.dig`
   file's bounding box** (the 6 floats in its header, `buildings.md`),
   the same box the game gives the building's collider. Connor's position
   is turned into the building's own axes and the distance is from that
   point to the nearest point of the box (0 when Connor stands inside the
   box). It is **not** the point where the click line met the wall. B's
   stand-in (distance to the building's bounding box, placed with the
   building) is right.
10. **A picked building with no script does nothing at all.** In reach
    the click becomes a `ConnorAction` call; out of reach a building's
    default `tooFarAwayAction` is **1** (buildings are the only class that
    sets it), so it also becomes a call (`TooFar`) instead of Connor's
    "too far" remark. With no script both calls are dropped: **no remark,
    and Connor doesn't turn** (he turns only after a script call). So a
    click on a wall is silently swallowed. Three buildings have scripts
    (data): DeadCity `doorway2`, `doorway3`, SnowExit `stnhedge`; they get
    the calls like a door does.
11. The same holds for any picked object without a script: in reach, no
    item in hand: nothing happens and Connor doesn't turn. (Doors,
    chests and items have their own handling; `doors.md`, `chests.md`,
    `items.md`.)
12. With an item in hand, the "accepts?" question (rules 13–18) is asked
    of whatever was picked, buildings included; a building with no
    accept list says no, so Connor makes his stock "that doesn't work"
    remark.

### Does an object accept the item in Connor's hand? (traced; answers Q62 a)

13. Every object has a list of accepted item types and an **accept
    everything** switch. The question is answered **yes** when the
    switch is on, or when the held item's type is in the list. The type
    is compared, not the name: any item of a listed type counts. Doors,
    chests, items, animated objects, characters and buildings all use
    this one rule (chests and items have no list, so they say no).
14. `acceptsInventory=<value>` fills the list when the object **enters
    the world**. The value is split **at commas only**. Then:
    - if the **first** word is `all` or `INVITEM_All` (capitals ignored),
      the switch is turned on and the rest is ignored;
    - otherwise each word is an item type name, looked up like every
      type name (`INVITEM_` put in front if missing, capitals ignored,
      `items.md` rule 2) and added to the list, until a word `none`
      (capitals ignored), which ends the list.
15. `acceptAllItems=<n>`: a non-zero number turns on the same switch.
    It is the same thing as `INVITEM_All`.
16. **`KQObject::setAccepts <object> <value>`** reads a value the same
    way, at any time, and **adds** to what the object already has. It
    never empties the list and never turns the switch off (`none` only
    ends the reading). Used by Gnome's armory scripts (and two
    commented-out lines in Barren) to make a shop item accept
    `INVITEM_SilverCoins`.
17. Data: 73 `acceptsInventory` lines give a single type, 19 give comma
    lists (none with spaces), 36 give `INVITEM_All`. Every listed name is a real
    type in `Invinfo.itl` (checked, all 12 worlds). Three files write
    `acceptsInventory = …` with spaces round the `=` (Swamp `kettle`,
    `lfswdoor`, `rtswdoor`, and one `INVITEM_All`); that the value is
    read without the leading space was not checked.
18. If accepted, the click becomes a `ConnorAction` call with `$3` = the
    item object's name and `$4` = its type name (`doors.md` rule 13).

### Clickable animated objects with a script (traced; answers Q64)

19. **Clicking** a `KQAnimObject` (jar, baskets, small chests, signs, the
    fish, `coffnlid`, `Bricks`, `MillTree`, `wizbook`, …) works exactly as
    for a door: `doors.md` rules 11–16 and rules 13–18 above. These
    objects add nothing of their own. Whether their movement is playing
    doesn't matter: they are clickable whenever their collision type has
    bit 0x20, and the line meets their box as it stands at that moment.
    So the swinging signs, `fishies`, `deadbird`, `starball` and
    `girltree` are clickable while they move.
20. **`KQAnimPlayEvent <movement>`** starts that movement by name, the
    same for every animated object (`doors.md` rule 6 and Q45 d).
21. **Script calls they get:**
    - **`Init`** (`$1` = name, `$2` = `Init`) once, when the object
      enters the world. Every object with a script gets it (not only
      animated ones). It runs after the object's name is registered and
      its accept list is read, but **before** its collision shape is set
      up.
    - **`EndLoop <movement>`**: when a movement that **doesn't repeat**
      (`loop=` repeat flag 0) reaches its end (or its start, when playing
      backwards), the object sends itself a cue, and its script then gets
      `$2` = `EndLoop`, `$3` = the movement's name. This is the call the
      23 Daventry scripts test. It goes through the event queue, so it
      arrives after anything already waiting "now" (`doors.md` Q45 d).
      Repeating movements never send it. Only objects with a script send
      it.
    - **Doors get `EndLoop` too.** When `open` or `close` ends, a door
      first runs its script with `Open`/`Close` at once (`doors.md` rule
      5) and then, through the queue, also gets `EndLoop open` /
      `EndLoop close`. No Daventry door script reacts to `EndLoop` (data),
      so for Daventry this changes nothing. Chests: `chests.md` rule 6
      (already right).
    - **`Cue <n1> <n2> [<n3>]`** for a `KQCueEvent` (`doors.md` rule 23),
      with two reserved first numbers: **3333** gives `EndLoop` (above)
      instead of `Cue`, and **500** launches the object's projectile
      instead of calling the script (combat; not studied). No script in
      any world sends either number by hand (data).
    - `ConnorAction`, `TooFar`, `Die`, `ProjImmune`, `ConnorCollision`,
      `WorldCollision` as described in `doors.md` and
      `connor-collision.md`.
    - **Hearing about another object's movements: `getEndLoop <from> <to>
      <1|0>`.** With 1, object `<to>` is told whenever one of `<from>`'s
      non-repeating movements ends; 0 stops it. `<to>`'s script then gets
      `$2` = `Cue`, `$3` = `300`, `$4` = `0`, `$5` = **the movement's
      name** (traced: what is sent and when, the same moment as rule 21's
      `EndLoop`; the command's own bookkeeping is inferred from its usage
      text: one listener per object, a new one replaces the old). Daventry
      uses it with Connor as `<from>`: `bricks.cs` does `getEndLoop Connor
      $1 1`, plays Connor's `push` through `KQMonster::setState Connor
      special …`, and reacts to `Cue 300 … push`; also `ax.cs`, `girl.cs`,
      `pyramid.cs`, `shadow.cs`, `sprgnbow.cs` (data). Connor's movements
      end the same way as any animated object's.

### `deleteObject` (traced)

22. `deleteObject <name>` marks the named object **deleted at once**:
    from that moment the click line and all collision tests skip it, and
    it is queued to be removed from the world. The removal, and the
    freeing of its name, happen **when the current event has finished**
    (the script run in progress and what it calls; traced 2026-10-17,
    Q90 d). Until then a `loadKQ` with the same name is fatal in the
    original (`objects.md` "Where a world's objects come from" rule 6). It is not drawn any more. This is the same for placed
    objects, items, everything. `$Console::Return` = 0 when something was
    deleted; an unknown name prints `deleteObject: Object "<name>"
    doesn't exist` and leaves 1.
23. Whether a deleted placed object stays deleted after Connor leaves
    the world and comes back, or after loading a saved game, was not
    traced (M6).

### `KQStatsChangeRequestEvent` (partly traced)

24. The event carries **five whole numbers in the same order as an
    object's `stats=` line**: first health, fourth armour (`doors.md`
    rules 18–19). So `fishies.cs`'s `… KQStatsChangeRequestEvent 2 0 0 0
    0` concerns health only. **They are added** (traced 2026-10-17,
    `anim-scripts.md` rule 19, which also gives the caps and Connor's
    levels; the fifth number is experience). The user saw his health go
    **up** after eating the fish, which fits. The other three numbers'
    meanings are for the stats study (M5).

### Connor turns to face what he clicked (traced 2026-10-18, Q104 b)

Answers `doors.md` open question 3. Fits the user's check in the
original: every Daventry tombstone turns Connor when clicked, also those
whose script says `KQMonster::speak Connor Connor …`.

25. **The turn is part of the `ConnorAction` script call itself**, not
    of the script. Every object kind that gets that call shares it:
    doors, animated objects (signs, jars, `coffnlid`, …), plain scripted
    objects such as the tombstones (`classType=KQObject` with a
    `script=`), characters, buildings with a script, and inventory items
    lying in the world that have a script. Whenever the call is made,
    whatever its kind (`do`, an accepted item, `TooFar` or `Special`),
    Connor is turned to face the object afterwards. What the script does
    doesn't matter: it may say nothing, `speak Connor Connor …` or
    `speak Connor $1 …`.
26. **No call, no turn.** An object without a script gets no call and
    Connor doesn't turn (rules 10–11). Connor's own stock remarks are
    not calls either and turn nobody: "too far" (object with
    `tooFarAwayAction` 0) and "that doesn't work" (item not accepted).
    These two remarks are queued at once, with no 0.5 s wait. **A "too
    far" click turns him only when the object has a non-zero
    `tooFarAwayAction` and a script** (then the `TooFar` call is made,
    from any distance). In Daventry five scripted objects set
    `tooFarAwayAction=1`: `girl`, `landslid`, `ropenhok`, `ugly`,
    `wizard` (data); a click on one of them from any distance turns
    Connor towards it. Signs, tombstones and doors don't set it: from too
    far, the remark and no turn.
27. **Order:** (1) the object's script runs to its end, including
    anything it does at once (`speak` queues or starts its turn-and-wait,
    `sendEvent 0` only queues events for later); (2) Connor's state is
    looked at **after** the script, so a script that changes his state
    decides the answer; (3) the turn is started (unless rules 28–30
    stop it). The turn then runs over the following updates.
28. **States in which he turns:** Wait, Move, the conversation (talk)
    state, and three states not reached in Daventry's normal play: drawing
    or putting away a weapon, throwing a rock, and the tile-puzzle
    jumping state. Not in Push, Special (scripted animations), Jump,
    Fall, a crash, or any other state. Compare `talking.md` rule 7: a
    `speak` turns Connor in Wait, Move, weapon draw/put-away and talk
    (those last two are the "two states not used by remarks"); the click
    adds throwing and the tile puzzle. In Daventry, in practice, only
    Wait and Move matter. (Clicks are not taken at all in some other
    states; that is decided before the call and is not part of this
    rule.)
29. **`doNotPointAt=1`** on the clicked object: no turn; the script still
    runs. Default 0 for every kind **except inventory items, which
    default to 1**, so picking up an item with a script doesn't turn him
    unless its file says `doNotPointAt=0` (no file does; the data only
    ever writes `doNotPointAt=1`, 37 files; in Daventry `ax` and `urn`).
    The setting is looked up by its **whole name**: Daventry's `wizbook`
    and `wizquill` write `doNotPointAtMe=1`, which the game doesn't know,
    so **they do turn Connor** (traced; not checked in the game).
30. One more switch (on Connor, not identified, the same one that can
    stop a `pointAndCue` turn, `anim-scripts.md` rule 25) blocks every
    automatic turn of his. It was never seen set; ignore it.
31. **It is the same automatic turn** as for `speak`, `walkTo` and
    `pointAndCue` (`connor-movement.md` "Turning" rule 2): 3.5 rad/s,
    the shorter way round, re-aimed every update, the same `Stop` →
    `Left`/`Right` movements when he stands (`anim-scripts.md` rule 25),
    the same rules for held and pressed turn keys. The target is the
    object's **position** (its placement point, not its box centre and
    not the point where the click line hit it).
32. **Two turns from one click.** There is only one automatic turn; a
    new request replaces the target of the running one.
    - `speak Connor $1 …` (the clicked object): the speak starts a turn
      to the object, then the click asks for a turn to the same point.
      Result: **one** turn. The line still starts 0.5 s after the
      `speak` (that wait does not depend on the turn).
    - `speak Connor Connor …` (tombstones `tmbstn1`, `tmbstn10`–`19`,
      `tomb7`): no speak turn, the line is queued **at once** and starts
      straight away, and the click's turn runs while he talks. So these
      tombstones start talking about half a second **earlier** than
      `tmbstn2`–`9`, and he turns while speaking.
    - `speak Connor <another object> …`: the speak turn is replaced by
      the click's, so he ends up facing the clicked object; the line
      still comes 0.5 s after the `speak`.
33. **Chests are different** (`chests.md` rules 6–10): a chest takes the
    click itself without a script call; Connor turns only before a
    `Take`/`Empty` remark, and that turn does **not** look at his state
    (it happens in any state; `doNotPointAt` still stops it).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| clickable bit | collision type 0x20 | traced |
| building default collision type | 0x28 (clickable) | traced |
| building `tooFarAwayAction` default | 1 (other objects 0) | traced |
| not-clickable building type in the data | 0x88 (`colliderType=136`) | data |
| reach | 2 × `attackDist` = 4,700 | traced (`doors.md`) |
| reach to a building | Connor to its `.dig` header box, in the building's axes | traced |
| reserved cue numbers | 3333 = `EndLoop`, 500 = projectile | traced |

## Data used

- Every object's `colliderType`, `script`, `acceptsInventory`,
  `acceptAllItems`, `tooFarAwayAction`, `doNotPointAt`.
- Buildings' `.kq` files (`classType=KQInterior`) and `.dig` BSP.
- `GAME/patch/BOUNDS.KQ`, `GAME/Kq/Invinfo.itl`.
- Scripts: `KQObject::setAccepts`, `deleteObject`, `KQAnimPlayEvent`,
  `KQCueEvent`, `KQStatsChangeRequestEvent`.

## Edge cases

- A click that hits a building wall first, with an item in hand: Connor
  says "that doesn't work" (rule 12), even if the object behind the wall
  would have taken the item.
- An item lying **inside** a clickable building's solid space (if data
  ever put one there) could never be clicked.
- A hidden clickable object (e.g. Daventry's `wizchest1`, while hidden
  and once its script has made it clickable) can be clicked where it
  stands, by the code (rule 7). In Daventry its script makes it
  clickable and shows it at the same moment (data), so this does not
  show there.

## Do not reproduce

- **An unknown item type name in an accept list crashes the original**
  (the lookup's "not found" is not checked). No data has one. Skip the
  unknown word and log it.
- **`all` / `INVITEM_All` only works as the first word.** Later in a list
  it would be looked up as a type and crash. No data does this. Treat it
  as "accept everything" anywhere in the list.
- **`deleteObject a b c`** deletes only the **first** of the names that
  exists (it stops after one; missing names before it print an error).
  No data passes more than one name. Delete every listed object.

## How to check

- **In the original (rule 6, the ground):** find a clickable object you
  can see over the top of a low rise, then move the camera so the rise
  just covers it, and click where it is. By the code the click still
  works (if it is in reach). If it doesn't, something tests the ground
  after all.
- **In the original (rules 3–4):** click an item or door through the
  church wall or the wizard's house wall (`colliderType=136`) while it is
  in reach: by the code the click goes through those two buildings, but
  not through the pub's or Connor's hut's walls.
- **In the original (rule 10):** click a house wall in reach: nothing at
  all should happen (no remark, no turn). Click it from far away: also
  nothing (no "too far" remark).
- **In the original (rule 9, buildings, Q75):** with an item in hand,
  click a house wall from far off (more than 4,700 from the whole house):
  silence. Walk closer until Connor is within 4,700 of any part of the
  house's bounding box (which can be a different side from the wall you
  click): "that doesn't work". Our log: for a picked building, the distance to its
  `.dig` box and "in reach" / "too far".
- **In the original (rules 25–32, Q104 b):**
  1. Click `tmbstn1` and then `tmbstn2` (both in reach, Connor facing
     away from them). Expected: on `tmbstn1` he starts talking at once
     and turns while talking; on `tmbstn2` he turns first and starts
     talking about half a second later.
  2. Click the wizard's book (`wizbook`, `doNotPointAtMe=1`): by the
     code he **does** turn to it. Click the urn (`urn`,
     `doNotPointAt=1`): he does **not** turn.
  3. Click a sign from too far away: "'Tis beyond my reach.", **no**
     turn.
- **In our logs:** per click: the call made (`do`/item/`TooFar`/
  `Special`/none), Connor's state after the script, `turn to <object>`
  or the reason there is none (`doNotPointAt`, state, no call).
- **In our logs:** for every click: what the line hit first (name, kind,
  distance along the line), and for buildings "swallowed, no script".
  For every animated object with a script: `Init` at load, `EndLoop
  <movement>` when a play-once movement ends (e.g. a mushroom basket's
  `open`), with the update number.

## Open questions

1. Which view setting gives the click line's length (rule 1).
2. When a deleted object is finally removed, and whether deletions last
   across leaving the world and saving (rules 22–23).
3. ~~How Connor applies `KQStatsChangeRequestEvent`~~ Answered
   2026-10-17: added; the 5th number is experience (`anim-scripts.md`
   rule 19). The meaning of numbers 2 and 3 stays for M5.
4. `getEndLoop`'s own bookkeeping (rule 21, last point) was not traced.
