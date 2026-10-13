# Doors (`KQDoor`), the scripts they run, and the minimum of the `.cs` language

Study session A, 2026-10-12. Sources: `Mask.exe` (studied, described here
in our own words) and the game data (`daventry/kq/*.kq`, `*.cs`,
`daventry/resource/*.kqs`, `GAME/patch/BOUNDS.KQ`, `GAME/Kq/Connor.kq`,
the manual). "Traced" = read in the exe; "data" = read from the files;
"inferred" = follows from the two but was not seen in the exe. Answers
`QUESTIONS.md` Q44 at the end. Builds on `objects.md` (movement lists,
`loop=` lines, `loadKQ`, script variables) and `connor-movement.md`
(animation phase, cues, "can be cut short").

## Purpose

A door is an animated object with four movements (`closed`, `opened`,
`open`, `close`) and a small built-in state: it knows whether it is shut,
open, opening or closing. The player clicks it; that runs the door's own
script (`script=<file>.cs`), and the script tells the door to open or
close by sending it a `KQDoorEvent`. When an opening or closing animation
ends, the door runs its script again so puzzles can react. Switches and
levers that only move between two positions (the arrow-trap switches
`arroswt1–3`) are doors too.

## Daventry's doors (data)

| Object (`.kq`) | Loaded by | Shape | Script | Notes |
| --- | --- | --- | --- | --- |
| `Saradoor`, `Wizdoor`, `Winddoor`, `Pubdoor`, `Girldoor`, `WatMdoor`, `Farmdoor` | `newWorld.cs` | `door1`/`door2` | `toggleme.cs` | plain house doors |
| `Condoor` (Connor's house) | `newWorld.cs` | `door1` | `condoor.cs` | first entry line, loads the raven |
| `Alchdoor` | `newWorld.cs` | `door2` | `Alchdoor.cs` | reacts to `ProjImmune` while boarded up |
| `outdoor` (outhouse) | `newWorld.cs` | `outdor` | `outdoor.cs` | starts a cutscene, then can't be clicked |
| `keepdrl`, `keepdrr` (castle keep, two halves) | `newWorld.cs` | `keepdrl`/`keepdrr` | `KeepDoor.cs` | locked; opened with the key item |
| `Tombdor` | `newWorld.cs` | `door3` | `tombdor.cs` | opened by `urn1.cs`, not by clicking |
| `Tombdor2` (hero's tomb) | `newWorld.cs` | `tombdoor` | `tombdor2.cs` | breakable (`stats=1 …`), opens when hit |
| `arroswt1–3` (arrow-trap switches) | `newWorld.cs` | `chainbtn` | `arroswt1–3.cs` | two-position switch |

`tombdoor.kq` (an older file using `door.dts`, one movement) is **not
loaded** by any script; ignore it.

All of them: `classType=KQDoor`, `currAnimName=closed`, `keepOnGround=0`
(they keep their `locZ`), movement list with `Movement0=closed`,
`Movement1=opened`, `Movement2=open`, `Movement3=close`. The `loop=` lines
(meaning in `objects.md` Q12 rule 2; the last two numbers are "loops" and
"can be cut short", `connor-movement.md`):

| Movement | Typical line | What it shows |
| --- | --- | --- |
| `closed` | `door1.kqs door1 0.0 0.0 0 1` | first frame, still |
| `opened` | `door1.kqs door1 1.0 0.0 0 1` | last frame, still |
| `open` | `door1.kqs door1 0.0 1.0 0 1` + `cue0=0.0 playsound door4.aud` | plays forward once, sound at the start |
| `close` | `door1.kqs door1 0.93 -1.0 0 1` | plays backward once from 93 % (the keep doors and switches from 1.0) |

## Rules

### The door's state (traced)

1. A door has four states: **closed**, **opening**, **opened**, **closing**.
   **Every door starts closed**, whatever its `currAnimName` says (the
   movement shown at load still follows `currAnimName`, `objects.md` Q12;
   all Daventry doors say `closed`, so the two agree).
2. **`KQDoorEvent` with value 1 = open.** Only if the door is closed: it
   starts the `open` movement and becomes *opening*. In any other state
   the event is ignored.
3. **Value 2 = close.** Only if the door is opened: it starts `close` and
   becomes *closing*. Otherwise ignored.
4. **Any other value (0, or no number at all) = toggle.** Closed → as
   rule 2; opened → as rule 3; opening or closing → ignored. A
   `KQDoorEvent` with no number counts as "other", so it toggles.
5. **When `open` finishes**, the door becomes *opened*, switches to the
   `opened` movement, and runs its script with `$1` = the door's name,
   `$2` = `Open`. **When `close` finishes**: *closed*, `closed`
   movement, script with `$2` = `Close`. (These two are the only script
   calls with just two arguments.) "Finishes" is the usual end of a
   play-once animation (`objects.md`, `connor-movement.md`): forward to the
   end, or backward to phase 0.
6. Rule 5 looks only at the **name of the movement that finished**
   (`open` or `close`, capitals ignored), not at the state. So a script
   that starts `open` directly with `KQAnimPlayEvent open`
   (`tombdor2.cs` does) still ends with the door *opened* and an `Open`
   call. While that `open` plays, the state is still *closed*, so a
   `KQDoorEvent 1` arriving then would start `open` again from the start.
7. Movements are started the normal way (by name, through the movement
   rules of `objects.md`/`connor-movement.md`). Doors have no transition
   lines, so the change is immediate.

### Timing (data)

8. `door1`, `door2`, `door3`, `tombdoor`, `keepdrl`, `chainbtn` (and by
   the same layout `keepdrr`, `outdor`): **16 frames, play time 1.0 s**.
   So `open` takes **1.0 s**; `close` from 0.93 takes **0.93 s** (from 1.0:
   1.0 s). At the game's 15 updates per second that is 15 and about 14
   updates.
9. Cue points fire when the phase passes them during an update: a cue
   at phase *c* fires when *c* lies in [lower, higher) of the phase before
   and after that update, in either direction (traced). So `cue0=0.0`
   on `open` fires in its first update.

### Clicking a door (traced; controls from the manual)

10. Doors are used with the **play cursor** (golden arrow; with a weapon
    out, hold **Ctrl** to get it). A left click picks the nearest object
    under the cursor whose collision type has **bit 0x20** ("clickable").
    All Daventry doors have it (`colliderType` 20513 = 0x5021, keep doors
    4129 = 0x1021, `Tombdor` 16481 = 0x4061, switches 33 = 0x21).
11. **Reach.** The door is in reach when the distance from Connor's
    position (his feet point) to the door's box is less than **2 ×
    Connor's `attackDist`**. Connor's `.kq` sets `attackDist=2350` (the
    built-in default for characters is 1,250), so the reach is **4,700
    units**. The distance is measured in the door's own axes: the point
    is turned into the door's frame, and on each axis only the part
    outside the box counts (0 inside). The box is the door's
    `BOUNDS.KQ` box (`connor-physics.md`), e.g. `door1.kqs`
    x −630…600, y −190…−60, z 0…1,730.
12. **In reach, no item in hand:** the door's script runs with
    `$1` = door name, `$2` = `ConnorAction`, `$3` = `do`, `$4` = `do`.
    (Scripts test either `$3` or `$4`; both work.)
13. **In reach, an inventory item in hand:** the door is asked whether it
    accepts the item. If not, Connor says a stock "that doesn't work"
    line (which message: not traced) and the script is not run. If yes,
    the script runs with `$3` = **the item object's name** and `$4` =
    **the item's type name** (e.g. `INVITEM_KeepKey`): `KeepDoor.cs`
    tests `$4` and deletes `$3`. How "accepts" is decided was not traced
    in detail; the doors use `acceptsInventory=<item type>` (likely also
    `acceptAllItems=1`).
14. **Out of reach:** if the door's `tooFarAwayAction` is 0 (the default;
    no Daventry door sets it), Connor says a stock "too far" line (which
    message: not traced) and nothing is sent to the door. If it is
    non-zero, the script runs with `$3` = `$4` = `TooFar`. (Seen by the
    user in the original: clicking a closed door from far away, Connor
    says it is too far. The exact distance was not measured in the game.)
15. A third kind, `$3` = `$4` = `Special`, exists for characters with a
    conversation-area object (`<name>Info`, class `KQConvInfo`). No door
    has one; doors never get it.
16. After the script call, Connor **turns to face the door**, unless the
    door has `doNotPointAt` set, and only in some of his states (standing,
    walking and a few others; the exact list was not matched to state
    names).
17. The script runs when the click is handled, before anything else
    happens; it is the script that sends `KQDoorEvent`. **The door does
    nothing by itself when clicked.**

### Hits (traced)

18. A door has hit points: the **first number of `stats=`** (five whole
    numbers). **Default for a door: 10,000** (other objects differ).
    `Tombdor2` and `Tombdor` set `stats=1 0 0 0 0`.
19. When a weapon hit with damage reaches a door, the damage (random,
    at least 1; combat details are for the combat study) is taken off its
    hit points. If the 4th `stats` number is 100 the object takes no
    damage (values above 100 are cut to 100).
20. When the hit points are **below 0** after a hit, the script runs with
    `$2` = `Die` and `$3` = the kind of the last damage: `Normal`,
    `Projectile`, `Fire` or `Ice`. The door is not removed and nothing
    else happens: it is up to the script (`tombdor2.cs` opens the door and
    clears its clickable bit). Every further damaging hit runs `Die`
    again, because the hit points stay below 0.
21. A hit that carries **no damage** runs the script with `$2` =
    `ProjImmune` instead (`Alchdoor.cs` uses it for a remark while the
    boards are up).

### Other script calls a door can get (traced)

22. `$2` = `Init` when the object enters the world (only `urn1.cs`
    uses it; door scripts ignore it).
23. `$2` = `Cue`, `$3`, `$4` (and `$5` if non-zero) = the numbers of a
    `KQCueEvent` sent to the object (`outdoor.cs` sends one to itself
    0.07 s later and reacts to it).

### Collision while open (inferred from data, confirmed in the game; code not traced)

24. Doors keep collision type bit 0x1, which Connor's mask covers, so a
    door blocks him (`connor-physics.md` "Which pairs collide"). Opening
    does not change the collision type (only the click bit is cleared by
    some scripts). The door's box never changes shape, but a closed door's
    box fills the doorway (`door1.kqs` is 1,230 wide), and Connor walks
    into houses in the original, so **the box must swing with the door**:
    the door object itself turns by the frame steps of its animation
    (about 11–13° per frame about a hinge near one edge, ~165° in all;
    `models-and-textures.md` "Frame transform"), and its box turns with
    it. Where the game applies these steps to a door was **not found**.
    **Confirmed by the user in the original (2026-10-12):** with the door
    open the doorway is free, and walking into the open door leaf blocks
    Connor: the door stays solid, just out of the way.

## The script language: what doors need (traced unless marked)

Scripts are the engine's console language, run **line by line, to the
end, at the moment they are called** (nothing waits). A line is at most
about 500 characters.

#### Words and arguments

- A line is split into words at spaces/tabs. `#` starts a comment (data;
  every script uses it).
- `$1` … `$n` are the arguments of the current script call; `$1`–`$10`
  that weren't given are empty.
- `$Some::Name` is replaced by the variable's text, which is then split
  into words again (`objects.md` Q20 rule 2). **An unset or empty variable
  vanishes from the line**, so `test $X == 0` with `X` unset becomes
  `test == 0`. In double quotes (`"$X"`) it stays one word, possibly
  empty. `condoor.cs` quotes `$Daventry::Raven::ConnorHouse` for this
  reason.
- Command names are matched exactly (capitals count). A first word that
  is not a command but names a script (`toggleme.cs`, or a name without
  dot, which gets `.cs` added) runs that script with the rest of the line
  as its arguments (`objects.md` "Where a world's objects come from").
  An unknown command prints an error and counts as failed.
- **Every command leaves a result in `$Console::Return`.**

#### `if`, `else`, `endif`

- `if <command…>` runs the rest of the line as a command (in practice
  always `test …`), then **takes the branch when `$Console::Return` is 0
  or empty**. `test` follows shell habits: **0 means true**.
- `else` switches to the other branch; `endif` closes the `if`.
  Everything between is skipped or run accordingly; nested `if`s in a
  skipped part are counted, not run, so their `else`/`endif` pair up
  correctly.
- **`else if` is not a keyword.** It is an `else` followed by a new
  nested `if` on the same line, which needs its **own** `endif`. That is
  why the scripts end with two or three `endif` lines in a row
  (`condoor.cs`: `if … else if test $2 == Die … endif endif`). Implement
  it exactly that way and the scripts balance.

#### `test <a> <op> <b>`

- Exactly three words after `test`; anything else prints a usage line
  and counts as **false**. (So `test $X == 0` with `X` unset is false.)
- Operators: `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`.
- If **both** sides look like numbers (start with a digit, or with `-`
  or `.` followed by a digit), they are compared as numbers
  (`1.0 == 1` is true). Otherwise as text, **capitals count**
  (`true == True` is false). `&&`/`||` work on whole numbers only
  (non-zero = true) and print an error for text.

#### Commands door scripts use

| Command | What it does | Status |
| --- | --- | --- |
| `set <Name> <value…>` | sets a variable (names like `Daventry::KeepDoor::Locked`) | objects.md; capitals in names not traced, keep as written |
| `echo …` | prints the line to the console (for us: the log) | traced as a command; output format not studied |
| `sendEvent <delay> <object> <EventType> [data…]` | sends an event to the named object after `<delay>` seconds of game time; `0` = as soon as the queue is next handled, after events already waiting for "now" (same time: first sent, first handled). Unknown object or event type: error message, nothing sent. Fewer than 4 words: usage message. | traced |
| `sendEvent … KQDoorEvent <n>` | rules 2–4 | traced |
| `sendEvent … KQAnimPlayEvent <movement>` | starts that movement by name; the door's state is not changed (rule 6) | traced |
| `sendEvent … KQCueEvent <n1> [<n2>]` | the target's script gets `Cue n1 n2` (rule 23) | traced |
| `sendEvent … KQStatsChangeRequestEvent 0 0 0 0 <n>` | rewards Connor (stats; combat/stats study) | not studied |
| `KQObject::collisionType <name> <±bits>` | positive: adds the bits to the object's collision type; negative: removes them; 0 refused. `-32` removes "clickable" (the door can't be clicked any more); `-288` also removes 0x100 ("push", per `urn1.cs`'s comment) | traced |
| `KQMonster::speak …`, `doPopup`, `addMapIcon`, `earnTeleport`, `teleportersEarned`, `KQMonster::checkInventory`, `deleteObject`, `handsOff`, `KQMonster::setState`, `pauseAnim`, `preloadShape`, `KQObject::setHidden`, `loadKQ` | dialogue, GUI, inventory, cutscene control | not studied here (M4 and later); stand-ins: log them, `checkInventory` → `$Console::Return` 0 |

## Numbers and units

| Thing | Value | Source |
| --- | --- | --- |
| door open / close time | 1.0 s / 0.93 s (16 frames) | data |
| reach | < 2 × `attackDist` = 4,700 from Connor's feet to the door's box | traced (factor 2, default 1,250) + data (2,350) |
| door hit points default | 10,000 | traced |
| clickable bit | collision type 0x20 | traced |
| `KQDoorEvent` default value | 3 (= toggle) | traced |

## Data used

- Door `.kq`: `classType=KQDoor`, `currAnimName`, `animLoops` list,
  `loop=`, `nCue`/`cueN`, `script`, `colliderType`, `acceptsInventory`,
  `stats`, `tooFarAwayAction`, `doNotPointAt`, `keepOnGround`.
- Door scripts (`daventry/kq/*.cs`), variables from `worldvar.cs`.
- `GAME/Kq/Connor.kq`: `attackDist`.
- `BOUNDS.KQ` door boxes; `.kqs` frame counts and play time.

## Edge cases

- **Plain doors only open.** `toggleme.cs` and `condoor.cs` send
  `KQDoorEvent 1`, which only opens (rule 2). Once open, clicking them
  again does nothing to the door (the script still runs; `condoor.cs`
  only says its line the first time). **Confirmed in the original by the
  user (2026-10-12):** Connor's front door opens on a click and stays open
  on the next click.
- Clicks while a door is moving are ignored by the door, but the script
  still runs. The arrow switches flip their variable
  (`Daventry::arrotrp1::SwitchOn`) on every click, so a click during the
  switch's 1-second movement puts the variable and the switch out of
  step. That is the original's behaviour (the scripts are data); keep it.
- `Saradoor`'s `close` has `cue0=0.95`, but `close` plays from 0.93 down
  to 0, so that sound **never plays** (rule 9). Keep it silent.
- `Pubdoor`, `WatMdoor`, `Winddoor`: `opened` asks for sequence `door1`
  inside `door2.kqs`, which doesn't have it; by `objects.md` Q13 rule 1
  the shape's first sequence plays instead, the same thing. Harmless.
- A door hit after `Die` keeps running `Die` (rule 20).
- `KeepDoor.cs`'s "do" branch for an unlocked door can't be reached:
  unlocking also makes both halves unclickable.

## Do not reproduce

- Nothing found in the door code itself. Note for testing: in Daventry
  the toggle (value 0) is sent only by `KeepDoor.cs` (to both halves when
  the key is used, which opens them from closed), and `close`/`Close` is
  reached only by the arrow switches (value 2).

## How to check

- **In the original:** click an open house door (e.g. Connor's): it
  should stay open (edge case 1). Click a door from far away: Connor makes
  a remark and the door stays shut; walk closer until it opens and note
  the distance (expected: within 4,700 of the door's box, i.e. quite far).
- **In the original:** open and close the arrow switch quickly; count
  clicks until the trap state and the switch disagree.
- **In the original (rule 24):** look at an open door from inside and
  outside: it should stand turned ~165° from closed, out of the doorway,
  and Connor should pass freely. Walk into the open door leaf itself: it
  should block him where it now stands.
- **In our game (log):** for each click: door name, distance to box,
  reach, the four script arguments, the `KQDoorEvent` value, state before
  and after; for each finished movement: name and the `Open`/`Close`
  call. Expected for `Condoor`: click in reach → `condoor ConnorAction do
  do` → `KQDoorEvent 1` → state opening for 15 updates → opened →
  `condoor Open`.

## Open questions

1. Where the game applies an animation's frame steps to a door (rule 24),
   and whether the jump to 0.93 at the start of `close` adds or removes
   the skipped step. If steps were simply added up, every open/close cycle
   would leave the door one step (~12°) further open; Daventry's plain
   doors never close, so this hardly matters there. Recommendation: work
   out the door's turn from the current phase (sum of the steps up to the
   shown frame), which never drifts.
2. How "accepts this item" is decided (rule 13), and which stock lines
   Connor says for "too far" and "doesn't work" (M4).
3. Connor's states in which he turns to face the clicked door (rule 16).
4. Which update a `sendEvent 0` event is handled in (the same one or the
   next).

## Q44 (answered, traced)

`loadKQ` with a position first replaces a z of −1/−2 by the floor/surface
height (`water.md`), **then** hands the position to the object's mover;
the `keepOnGround` raise is part of the mover placing the object, so it
runs **after** the replacement and on the replaced z. So B's order is
right: −2 → surface (0 at the DeadCity portal) → raised to the floor
(2,784) for a mover with the default `keepOnGround` (−1). Matches the
user's check.
