# Conversations (`KQConner::conversation`, `KQConvInfo`, `TalkExit`, `InvConv`)

Study session A, 2026-10-19. Traced in `Mask.exe` unless marked "data"
(read from the game's files) or "not traced".

## Purpose

How Connor holds a conversation with a character (the Wizard, the
Unicorn-beast, later the Sage, the Witch Doctor, …): what starts it,
how Connor gets to his talking spot, who says which line, the
conversation cameras, what the player can do meanwhile, how walking
away ends it with a "goodbye" and resumes it later with a "welcome
back", and which script messages a conversation sends.

Builds on `talking.md` (lines, message files, queues, voices, captions,
the mouth, `TalkComplete`, `clearSpeak`): a conversation is a second
kind of queue that uses the same line playing. Also uses `clicking.md`
(`ConnorAction`), `anim-scripts.md` rule 25 (the automatic turn),
`objects.md` Q20 (d) (`move` flags), `camera.md` (`follow`, `move
KQCamera`), `interface.md` (the bottom bar).

Words used here:

- **Partner**: the character Connor talks to (the first word after the
  command).
- **Talking spot**: the place Connor stands during the conversation
  (`convLocX/Y/Z` of the partner's info object).
- **Conversation queue**: the partner's second line queue, used only by
  conversations (separate from the queue that `speak` uses).
- **Start area**, **stay area**, **special areas**: the boxes of the
  info object (`box0`, `box1`, `box2`…; rule 3).

## Rules

### The command

1. **`KQConner::conversation <partner> <file> <noun> <verb> <case>
   <first> <last> <enemy>`** (9 words or more; fewer: the usage text is
   printed and nothing happens). `<partner>` must be a `KQMonster`
   (otherwise "'…' is not KQMonster", nothing happens). `<first>` and
   `<last>` are sequence numbers, `<last>` = 0 meaning "up to the last
   sequence that exists" as for `speak` (`talking.md` rule 6).
   `<enemy>`: 0 means Connor puts his weapon away first (rule 7); any
   other number means he keeps it out. Data: 169 calls in the scripts,
   **all with `<enemy>` = 0**; 19 partners have the goodbye keys
   (rule 20).
2. What the command does at once:
   - the partner's own `speak` queue is emptied (no `TalkComplete`);
   - the partner's conversation queue is made the first time (an
     object called `<name>Conv`), with the partner's **goodbye** and
     **welcome-back** lines (rule 20);
   - the lines `<first>`…`<last>` are prepared but **not yet queued**:
     they are queued when Connor is ready (rules 8–10);
   - the partner is asked for the `Stop` movement and goes into its
     **talk state** (rule 17);
   - Connor is sent a "start conversation" message (handled in the same
     update, after the script): Connor's own `speak` queue is emptied
     (no `TalkComplete`), he remembers the partner and the partner's
     position at this moment, is asked for `Stop`, and goes into his
     **talk state** (rules 5–16). If he goes into it and `handsOff` is
     off, the bottom bar's panels that were showing are hidden
     (`interface.md`; the meters, the left and right panels and the
     health bar) and remembered for the end (rule 15).

### The info object (data, traced)

3. Each partner has a `KQConvInfo` object named **`<partner name>Info`**
   (`[WizardInfo]` in `wizinfo.kq`, `[UglybeastInfo]` in `uglyinfo.kq`;
   loaded by the world's `permwrld.cs`, Daventry lines 374–376). Keys:
   - `convLocX`, `convLocY`, `convLocZ`: the talking spot (game units).
   - `nBox`, `box0`…: areas on the ground, each 4 corners `x y` (8
     numbers), ignoring height. A point is **inside** an area when it
     lies within the four-cornered shape (the game adds up the angles
     the corners make seen from the point and calls it inside when the
     sum is between 6.1 and 6.4 radians, i.e. 2π; a point exactly on a
     corner counts as outside). The order of the corners doesn't matter
     for convex shapes (all in the data are). `box0` = **start area**,
     `box1` = **stay area** (leaving it ends the conversation),
     `box2`+ = **special areas** (only the Wizard has one).
   - `nCam`, `cam0`…: conversation cameras, `x y z rx ry rz` each (the
     same six numbers as `move KQCamera 0 x y z rx ry rz`).
   - Data: 16 info objects: Daventry Wizard (3 boxes, 7 cameras) and
     Uglybeast (2, 7); Dead City Ferryman (2, 8), Sylph (2, 7); Swamp
     Protree, Wisp1–4 (2, 4 each); Gnome Sage, Barren Witchdr, Ice World
     Dragon, Gryphking, Nymphqueen, Snow exit Archon (2, 7 each).
   - The `EnvInfo` objects in every world are something else (same
     file name ending, not `KQConvInfo`).
4. **A partner without an info object:** the game does not check; it
   would crash (Do not reproduce). Every partner in the data has one.

### Clicking a partner (who has an info object)

5. A click on a character that has an info object does not use the
   usual reach test (`clicking.md`). Instead, Connor's position (x, y)
   is tested against the info boxes, in this order:
   1. inside any **special area** (`box2` or later): the script gets
      **`ConnorAction Special Special`**;
   2. else inside the **start area** (`box0`): the click counts as
      within reach, whatever the distance, and goes on as a normal
      click: **`ConnorAction do do`**, or the held item as usual
      (`clicking.md`, `inventory.md`; an item the partner refuses gives
      "This will not work.");
   3. else: **`ConnorAction TooFar TooFar`** (the partner's
      `tooFarAwayAction` is not looked at).
   In all three, the click's turn afterwards is as for any click
   (`clicking.md` rules 25–33). Data: `wizard.cs` answers `TooFar` with
   "Hello, there, sir wizard!" / "Come closer, lad!" (1000/4/14/32) and
   `Special` (behind his back, `box2`) with "Come around, lad...to
   where I can see you." (1000/4/0/64); `ugly.cs` answers `TooFar`
   with "Hello, beast!" / "Over here! Come closer!" (1000/59/0/32).

### Connor's talk state: getting to the talking spot

6. **On entering** the talk state (a new state each time):
   - the camera's "right button held" lock is released (`camera.md`);
   - if Connor is in **first person**, the view switches to third
     person; switching views by key is blocked until the end (rule 16);
   - the partner's gesture movements are made ready (rule 18);
   - the **case** is worked out from where Connor stands at this moment:
     - inside the **stay area** (`box1`) and less than **125** units
       (3D distance) from the talking spot: case **"there"**;
     - inside the stay area, 125 or more from the spot: case
       **"close"**;
     - else inside the **start area** (`box0`): case **"walk"**;
     - else: no case (rule 11, edge case).
7. **Weapon away first:** if `<enemy>` was 0 and Connor holds his sword
   or bow drawn, he plays the movement **`away`**; when it ends he is
   put in bare-hand mode and plays **`out`**; when that ends he goes on
   with rule 8. Otherwise rule 8 follows at once. (With `<enemy>` ≠ 0
   the weapon stays drawn; not used in the data.)
8. **Case "there":** the partner is told to start (rule 10) at once;
   Connor stays where he is.
9. **Case "close":** the partner is told to start at once, and Connor is
   **put on the talking spot** at once (position = `convLocX/Y/Z`, his
   heading unchanged; the same as `move Connor 2 x y z 0 0 0`, flags 2
   of `objects.md` Q20 (d)). No walking.
10. **Case "walk":** Connor turns toward the talking spot with the
    automatic turn and plays the movement **`stopwalk`** (a walk start).
    When it ends, he is **put 440 units short of the spot**, on the line
    from where he stood at the start to the spot, at the spot's height,
    **facing the spot**, and plays **`walkstopright`** (a walk that
    stops, whose own movement carries him the rest of the way); the
    partner is told to start at the same moment. When `walkstopright`
    ends, Connor turns to face the partner (its position from rule 2)
    with the automatic turn. (The original does not walk the whole way:
    it places him, then lets one movement finish the approach.)
    - "The partner is told to start" (rules 8–10): the partner's `speak`
      queue is emptied, and the conversation's lines (rule 2) are added
      to its conversation queue (rule 21) and the first one starts.
11. **No case** (Connor in none of the two boxes): nothing more
    happens. The partner is never told to start, and the stay-area test
    (rule 13) is not running, so Connor stays in the talk state with
    nothing playing (traced; Do not reproduce). In the data this
    doesn't happen in normal play: a click from outside the start area
    gives `TooFar` (rule 5), and the scripts that start conversations
    from cutscenes first move Connor into the stay area (checked for
    Daventry: the Wizard's map and ring scenes put Connor 1,083 and 344
    units from the spot, inside `box1`: case "close").

### During the conversation

12. **Lines:** the lines are played exactly like `speak` lines
    (`talking.md` rules 10–11): the speaker is each line's talker
    (Connor's lines by Connor, the Wizard's by the Wizard; a "generic"
    talker by the partner), voice, captions, ducking (never `nofade`),
    mouth, one after the other. Differences:
    - the **issuer is the partner**, for all lines (Connor's too): after
      every line the **partner's script** gets `TalkComplete` with the
      line's tuple, and the partner's notify group gets it too
      (`talking.md` rule 13);
    - after a line that is not the last, Connor's mouth goes back to
      rest;
    - a line's **camera number** `[n]` (`talking.md` rule 3) is used
      (rule 14);
    - the player can't cut a line short: movement keys and Jump don't
      (`talking.md` rule 16 excludes the talk state), and clicks only do
      what rule 19 says; `#` marks don't matter here;
    - `speak` for Connor or for the partner is ignored while either is
      in its talk state (`talking.md` rule 8).
13. **The player can still move Connor** (the movement keys are not
    blocked by the talk state, only by `handsOff`). While the stay-area
    test runs (from rule 8, 9 or the end of `walkstopright` in rule 10):
    - when Connor moves, his walking movement plays; when he stops
      again, `Stop`;
    - **every update**, his position (x, y) is tested against the stay
      area (`box1`). When he is **outside** it, the conversation ends
      the "walked away" way: the test stops, the partner is told
      (rule 22: the goodbye), and Connor leaves the talk state (rule
      15) at once.
14. **Conversation cameras:** when a line that has a camera number `[n]`
    starts while Connor is in his talk state:
    - if Connor has reached his spot (case "there", "close", or
      `walkstopright` has ended), he turns to face the partner (its
      position from rule 2) with the automatic turn;
    - the camera stops following and **cuts** to `cam<n>` of the info
      object (position and angles, as `move KQCamera 0 …`); it stays
      there until the next camera number or the end (rule 15).
    A line without a camera number leaves the camera as it is (the
    normal following camera until the first camera line). No check
    that `n` < `nCam` (data: the largest `[n]` is within `nCam` for
    every partner: Daventry 6 of 7, Dead City 7 of 8, Swamp 3 of 4, …).
    Camera numbers in a `speak` line (not a conversation) are ignored.
    Data: Daventry's file has 75 lines with a camera number (Wizard and
    Uglybeast); the Wizard's first meeting (1000/4/14/27/1…14) uses
    cameras 2, 3, 3, 5, 2, 1, 1, 1, 1, 3, 5, 2, 3, 6.
    **Moving the camera stops the cuts** (traced 2026-10-19 after the
    user's check in the original): the first update the player holds
    the **right mouse button** (the camera-move button, `camera.md`)
    while Connor's stay-area test runs, the camera is placed **once** at
    `cam0`'s position, **aimed at the partner** (its position from rule
    2), and stops following anyone; from then on, for the rest of this
    conversation, **camera numbers are ignored** (no cuts, and Connor no
    longer turns to the partner on them). The player swings and zooms
    the camera with the mouse as usual (`camera.md` "Swinging and
    zooming"), around the partner. At the end the camera follows Connor
    again (rule 15). **Checked by the user in the original
    (2026-10-19):** moving the camera cancels the cuts and lets it
    rotate freely around the two talking.

### The end

15. **Connor leaves the talk state** (only from the talk state):
    - when the conversation's **last line** has finished (before the
      partner's `TalkComplete` for it is run), or
    - when he **walks out of the stay area** (rule 13), or
    - on a click with an item on the partner (rule 19).
    He goes to **Move** if he is moving, else plays `Stop` and goes to
    **Wait**. The talk state can only be left this way (to Wait, Move,
    Special, or dying); other state requests wait until then. On
    leaving:
    - the camera is first put at `cam0`'s **position looking at the
      partner**, then **follows Connor again** (`KQCamera::follow
      Connor`, `camera.md`), so it swings in from there;
    - first person is allowed again, and if he was in first person
      before the conversation he goes back to it;
    - **`TalkExit`**: the partner's script runs with `$1` = partner's
      name, `$2` = `TalkExit` (no more words), at once, **unless** the
      goodbye was sent (rule 13; then `TalkExit` comes later, rule 22);
    - the partner goes to **Wait**;
    - the bottom bar's panels hidden at the start come back, if they
      were showing then, none of them is showing now, and `handsOff` is
      off.
    Order at a normal end: last line ends → `TalkExit` → `TalkComplete`
    of the last line. Data: `ugly.cs` relies on it: `TalkExit` sends
    itself cue 2 0.5 s later to turn the beast back to its pond
    (`Turned` = 1), then `TalkComplete` 1000/59/0/6/11 gives the
    experience and `handsOff 0`.
16. The partner, at the end of the last line, goes to Wait (or Move if
    it is moving), as after any last `speak` line (`talking.md`).

### The partner's talk state

17. While the partner is in its talk state, it turns toward Connor's
    **current** position with the automatic turn (`anim-scripts.md` rule
    25) **every fourth update**; nothing else. It can go to any of
    Talk, Wait, Move, Special, dying and one other state not used here.

### Gestures (the lip-sync codes above 7)

18. A lip-sync file's mouth codes 0–7 are mouth shapes (`talking.md`
    rule 17). **Codes 8–15 play the movements `gesturea`…`gestureh`**,
    code **16 plays `stop`**, and the mouth is not changed by any of
    them; codes above 16 do nothing. For conversations and for `speak`
    lines alike:
    - a character other than Connor plays it only if it stands still
      (no walking movement), is in a talk state (as far as traced: the
      flag it needs is set when a conversation starts) and has that
      movement in its animation list;
    - Connor plays it only if he stands still, has no weapon drawn and
      is not in first person.
    Data: 1,110 lip-sync files of the game have codes 8–16 (4,080
    codes; Daventry 93 files, 393 codes; e.g. `S0RS0400.081`, the
    Wizard's welcome-back line 1000/4/0/8/1, codes 8, 10, 12). Movement
    lists with
    `gesturea`…: `wizard.anm` (Movement14–21), `girl.anm`,
    `knight.anm` and others; Connor's `Conner.anm`, `CONNER2.anm`,
    `Connerfp.anm`.

### The player's clicks during a conversation

19. While Connor is in his talk state a click in the world does
    nothing, **except a click on the partner with an item held**:
    - the partner's script runs with `$2` = **`InvConv`**, `$3` = the
      held item's name (the same `INVITEM_…` word as in `ConnorAction`
      `$4`);
    - Connor leaves the talk state at once (rule 15, with `TalkExit`);
    - then the click goes on as a normal click on the partner (rule 5:
      usually `ConnorAction <item> INVITEM_…`).
    The conversation queue is not emptied by this; scripts do it with
    `KQMonster::clearSpeak <partner>` (Data: `wizard.cs` on the ring,
    candle and ashes; `ugly.cs` on the horn, which also answers
    `InvConv`).

### Goodbye and welcome back (`GB…`, `WB…` keys)

20. A character's `.kq` section may name two lines: **goodbye**
    `GBMsg`, `GBNoun`, `GBVerb`, `GBCase`, `GBSeq` and **welcome back**
    `WBMsg`, `WBNoun`, `WBVerb`, `WBCase`, `WBSeq` (data: 19 `.kq`
    files; Wizard 1000/4/0/7/1 "Well, then, off with you! Return when
    you wish to converse further." and 1000/4/0/8/1 "Ah, I see you have
    seen fit to return and finish our previous discussion. As I was
    saying..."; Uglybeast 1000/59/0/7/1 and 1000/59/0/8/1). A key
    missing means no such line (a file number and noun both 0 = none).
21. **Adding lines to the conversation queue** (when the partner is told
    to start, rule 10): as `speak`'s queue (`talking.md` rule 9), with
    the duplicate check, except:
    - if the conversation was **paused by a goodbye** (rule 22) and
      lines are still waiting in the queue: the **welcome-back line is
      put in front**, then the new lines are added after the waiting
      ones (not added if the range's last line is already waiting:
      rule 9's check), and playing starts: welcome back, then the
      interrupted line **from its beginning**, then the rest;
    - if a goodbye was asked for but has not played yet, it is dropped
      first.
22. **Walking away** (rule 13), on the partner's side:
    - if **more than one line** is left in the conversation queue (the
      one playing and at least one waiting) and the partner has a
      goodbye line: the goodbye is put in front and **starts at once**,
      cutting off the line that was playing (that line goes back to
      the head of the waiting lines). When the goodbye ends: the
      captions go, the sounds come back up, the queue stops (the
      waiting lines stay for later, rule 21), and the partner's script
      gets **`TalkExit`**. No `TalkComplete` for the goodbye.
    - otherwise (on the last line, or no goodbye line): nothing more;
      the last line plays to its end with its usual `TalkComplete`, and
      **no `TalkExit` is sent** (traced; see Do not reproduce).
    - The welcome-back line gives no `TalkComplete` either (and no
      `TalkExit`).

### `clearSpeak` on a partner

23. `KQMonster::clearSpeak <partner>` (completes `talking.md` rule 15):
    the conversation queue is emptied at once (line stopped, captions
    off, sounds up; no `TalkComplete` for those lines), the partner
    goes to **Wait**, and Connor is sent an empty "line finished"
    message (not the last): his mouth goes to rest and **his notify
    group gets `TalkComplete 0 0 0 0 0`**. It does **not** end Connor's
    talk state. Data: `ugly.cs` tests `$7 == 0` in `TalkComplete`.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| "there" distance (talking spot, 3D) | under 125 units | traced |
| walk-in placing | 440 units short of the spot | traced |
| partner turns to Connor | every 4th update | traced |
| "inside" test | angle sum 6.1–6.4 rad | traced |
| info objects | 16 (Daventry: Wizard, Uglybeast) | data |
| cameras per partner | 4–8 | data |
| conversation calls | 169, all `<enemy>` 0 | data |
| goodbye / welcome-back keys | 19 characters | data |
| gesture codes | 8–15 → `gesturea`–`gestureh`, 16 → `stop` | traced |

## Data used

- Scripts: `KQConner::conversation`, `KQMonster::clearSpeak`; messages
  `ConnorAction` (`do`, `TooFar`, `Special`, items), `InvConv`,
  `TalkComplete`, `TalkExit`.
- `<world>/kq/*info.kq` (`KQConvInfo`): `convLocX/Y/Z`, `nBox`,
  `box<n>`, `nCam`, `cam<n>`; loaded by `permwrld.cs`.
- Character `.kq`: `GBMsg/GBNoun/GBVerb/GBCase/GBSeq`,
  `WBMsg/WBNoun/WBVerb/WBCase/WBSeq`.
- Message files: `[n]` camera numbers (`talking.md` rule 3).
- Lip-sync files: codes 8–16.
- Animation lists (`.anm`): `away`, `out`, `stopwalk`, `walkstopright`,
  `Stop`/`stop`, `gesturea`…`gestureh`.

## Edge cases

- No case on entry (rule 11): stuck; see Do not reproduce.
- Case "walk" when Connor starts less than 440 units from the spot
  (outside the stay area): he is put 440 from the spot, i.e. possibly
  further back than he was (not seen in the data).
- A second `KQConner::conversation` while Connor is still in his talk
  state: the partner's side happens (rule 2), but Connor's state can't
  be entered again, so no new case is worked out and the partner is
  not told to start (not traced further; the Daventry scripts start
  new conversations only after the last line or after an item click,
  when Connor has left the talk state).
- The interrupted line (rule 22): whether its sound's own "finished"
  message still arrives when the goodbye cuts it off was not traced.
  Treat it as cut: no `TalkComplete` for it.
- A conversation with a line missing from the message file: dropped as
  in `talking.md` rule 10.

## Do not reproduce

- **A missing info object** (rule 4): crash in the original. Do
  instead: log it and treat the conversation as case "there" with no
  cameras.
- **No case** (rule 11): Connor stuck in the talk state. Do instead:
  log it and treat it as case "there" (start the lines at once where he
  stands).
- **No `TalkExit` when Connor walks away on the last line** (rule 22):
  the partner's script never hears that the conversation ended (for
  the Uglybeast, it would not turn back to its pond). Do instead: send
  `TalkExit` once when Connor leaves the talk state by walking away,
  if no goodbye will play. (Our choice; log it.)
- The camera-number check (rule 14): check `n` < `nCam`; log and ignore
  a bad number.
- Keep the rest, including the teleport-then-`walkstopright` approach
  (rule 10), the stay-area test every update, and the order `TalkExit`
  before the last `TalkComplete`.

## How to check

- **Our logs (B):** per conversation: partner, tuple range, Connor's
  position, which boxes he is in, the case, the 3D distance to the spot;
  `away`/`out`, `stopwalk`, the placing (position, heading) and
  `walkstopright` with times; "partner told to start"; per line as in
  `talking.md`, plus the camera number and the camera cut (position,
  angles); every stay-area test result change; the goodbye (inserted or
  not, why); the end (why, state after, `TalkExit` sent or not, panels
  restored); `InvConv`; gesture codes and the movement played or why
  not.
- **Daventry checks (B, logs):**
  - Wizard (`alias wizard`: Connor at 193000, 198000, heading −1.7):
    inside `box0` and `box1`, 834 from the spot → click the Wizard:
    `ConnorAction do do` → `conversation Wizard 1000 4 14 27 1 13 0` →
    case "close": Connor placed at (192171, 198091, 6700), heading
    kept; line 1 "Hello?! You're still alive! …" (camera 2), …
  - From (191000, 197000) (inside `box0` and `box2`; the special area
    wins): `ConnorAction Special Special`, the Wizard says "Come
    around, lad...".
  - From (198000, 198000) (in no box): `TooFar TooFar`, "Hello, there,
    sir wizard!".
  - During the first meeting, walk Connor out of `box1`: the Wizard's
    line stops, "Well, then, off with you! …" plays, `TalkExit` after
    it. Click him again: "Ah, I see you have seen fit to return …",
    then the cut line again from its start, then the rest; the
    conversation is not added twice.
  - Uglybeast (`ugly.cs` moves Connor to the spot itself): case
    "there"; at the end of 1000/59/0/6/11: `TalkExit` before
    `TalkComplete`.
- **In the original (user, by eye and ear):**
  1. Talk to the Wizard: does the camera cut between fixed views as the
     lines go on? Does the bottom bar hide during the talk and come
     back after?
  2. Stand just inside his house door and click him: does Connor walk
     (or jump) to the spot in front of the Wizard before the talk
     starts?
  3. In the middle of the first talk, walk away: does the Wizard say
     "Well, then, off with you!"? Then click him again: "Ah, I see you
     have seen fit to return…" and the talk carries on where it was?
  4. Does the Wizard move his arms (gestures) while he speaks?

## Open questions

1. ~~Which Connor setting makes the talk state place the camera at
   `cam0` and then ignore camera numbers.~~ **Answered 2026-10-19**:
   the right mouse button held (rule 14), confirmed by the user.
2. Whether the gestures' "talk state" condition for characters other
   than Connor is set by anything else than a conversation start
   (rule 18), i.e. whether `speak` lines alone ever make a character
   gesture.
3. Connor's states from which the talk state can't be entered (the
   conversation then waits), e.g. in the middle of a jump: not traced.
