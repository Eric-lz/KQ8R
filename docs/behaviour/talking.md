# Remarks and speech (`KQMonster::speak`, stock remarks, captions, `TalkComplete`)

Study session A, 2026-10-18. Traced in `Mask.exe` unless marked "data"
(read from the game's files), "community" (the libkq8fpc format notes and
the kqmoefix source listed in `docs/RESEARCH.md`, checked against the
data) or "not traced".

## Purpose

How a character says a line: which text and sound file a line is, how
lines queue up, what the player sees and hears (captions, the voice,
other sounds turned down, the mouth moving), what can cut a line short,
and how scripts hear that a line has finished (`TalkComplete`). This
covers Connor's **remarks** (clicking a sign, a chest, "too far") and any
`KQMonster::speak` line. **Conversations** (`KQConner::conversation`,
the talk state, choosing what to say, conversation cameras) are a
separate system and are not covered here (see "Open questions").

Builds on: `sound.md` rules 53 and 58 (speech sounds, ducking),
`clicking.md` rules 10–12 (when a click gives a stock remark),
`chests.md` rules 1, 8–9 (`Take`/`Empty` remarks), `anim-scripts.md`
rule 25 (the automatic turn), `doors.md` "The script language".

Words used here:

- **Line**: one entry of a message file: one sentence or a few, said by
  one character, with its own sound file.
- **Tuple**: the five numbers that name a line: **file**, **noun**,
  **verb**, **case**, **sequence**. A remark of several sentences is
  several lines with the same file, noun, verb and case and sequences
  1, 2, 3, …
- **Talker**: the character a line belongs to, stored in the message
  file with the line (a number, named in `TALKERS.SH`).
- **Issuer**: the character named first in `KQMonster::speak` (the one
  whose queue the line goes into). Usually the same as the talker, not
  always.

## Rules

### The files (data, community; file names and lookup traced)

1. **Message files** `<file>.MSG` hold the lines. Daventry's are in
   `MASK/daventry/English/1000.MSG` (254 lines; `castled/English` has an
   identical copy, and `temple2` an identical copy of `temple1`'s file);
   Connor's general lines are in
   `MASK/GAME/ENGLISH/0.MSG` (140 lines). Other worlds: 2000 Dead City,
   3000 Swamp, 4000 Gnome, 5000 Barren, 6000 Ice World, 7000 the
   temples; `400.MSG` and `500.MSG` in `GAME/ENGLISH` have no sound
   files. The game opens a message file by its number the first time a
   line from it is needed and keeps it. Which folders it searches is not
   traced; the data only fits "the current world's `English` folder,
   then `GAME/ENGLISH`".
2. **Format** (community note `kq8msg.txt`, checked on all 12 files):
   after a 2-byte header (type 0x0F or 0x8F, header size 0), a 4-byte
   version (5010 in all files), a 2-byte data size, a 2-byte "last id"
   and a 2-byte count; then `count` records of 11 bytes (noun, verb,
   case, sequence, talker: 1 byte each; text offset: 2 bytes, counted
   from the start of the version field; four more bytes, always 0, not
   used); then the texts, each ending in a zero byte. The editor's
   comments that follow are not used. The file number is the file
   name.
3. **Stage directions** in the text: everything inside `( … )` is taken
   out before the text is shown. Inside the brackets, `#` sets the
   line's **"no stop" mark** (rule 15) and a number in square brackets
   `[n]` is a conversation camera number (conversations only). Example:
   `([#]IN A HOPEFUL TONE) The alchemist's shop. …` shows as "The
   alchemist's shop. …" with one space in front (the space after the
   bracket is kept) and "no stop" set.
   Data: 476 of the 2,080 lines in the 12 files have `#`; in Daventry's
   file 100 of 254, in `0.MSG` 21 of 140 (the stock remarks have it, the
   item descriptions don't).
4. **Sound file** of a line: `A` + file (3 characters) + noun (2) + verb
   (2) + `.` + case (2) + sequence (1), each number written in **base
   36** (digits 0–9 then A–Z) and padded with zeros on the left.
   Example: Daventry 1000/55/0/0/1 → `A0RS1J00.001`. Its **lip-sync
   file** is the same name with `S` instead of `A`. Both are in the same
   folder as the message file. Data, lines without a sound file next to
   their message file: Daventry, Dead City, Gnome, Swamp none; Barren
   50 of 282, Ice World 2, Temple 1 19, `0.MSG` 6, `400.MSG` all 25,
   `500.MSG` 168 of 169; `castled` and `temple2` have none at all in
   their own folders (where their voices come from is not traced). The
   sound files are the speech files of `sound.md` rule 58.
5. **Talkers:** `MASK/GAME/ENGLISH/TALKERS.SH` lists `(define NAME n)`.
   The game turns each name into lower case with a capital first letter
   (`CONNOR` → `Connor`, `LAKELADY` → `Lakelady`, `GEN_GNOME1` →
   `Gen_gnome1`) and uses it as an **object name**: the line is spoken
   by the object with that name. Ten "generic" talkers are spoken by the
   issuer instead: `Me`, `Iceorc`, `Gen_gryph`, `Firedwarf`,
   `Ghenchman`, `Gen_hillmn`, `Gen_gnome1`, `Gen_gnome2`, `Gen_gnome3`,
   `Gen_nymph`. Daventry's lines (data): Connor 172, Wizard 40,
   Uglybeast 17, Girl 8, Knight 6, Unicorn 3, Henchman 2, Lucreto 2,
   Lakelady 1, Ravenc, Ravens, Raveng 1 each. Connor's talker number is
   4.

### `KQMonster::speak <issuer> <to> <file> <noun> <verb> <case> <first> <last> [nofade]`

6. 855 lines in the scripts (data), 81 Daventry scripts make Connor
   speak. `<issuer>` must be a `KQMonster` (Connor counts); otherwise the
   console says "'…' is not KQMonster" and nothing happens. `<to>` is an
   object name (or an object id number). `<first>` and `<last>` are
   sequence numbers; **`<last>` = 0 means "up to the last sequence that
   exists"** (the highest of the unbroken run first, first + 1, … in the
   file). `nofade` is only recognised as the 10th word exactly.
7. **Turning to face the listener:** if `<to>` names another object than
   the issuer (names compared ignoring capitals), the issuer turns to
   face it with the automatic turn (for Connor the 3.5 rad/s turn of
   `anim-scripts.md` rule 25, with the same `Stop` → `Left`/`Right`
   movements) and the lines are queued **0.5 s later**. Connor only
   turns while he is in Wait or Move (or two states not used by
   remarks); in any other state (Push, Special, a jump, …) he doesn't
   turn and the lines are queued at once. `speak Connor Connor …`, or a
   `<to>` that doesn't exist: no turn, queued at once.
8. A `speak` for an issuer that is in a **conversation** (the talk
   state) is ignored entirely.
9. **The queue:** each character has one queue of lines, made the first
   time it speaks (an object called `<name>Talk`). `speak` adds lines
   `<first>`…`<last>` to the end. A line already in the queue (same
   file, noun, verb, case and **first** sequence) is **not added
   again**: clicking a sign five times while its remark plays gives the
   remark once; after it has finished, a click gives it again. If the
   queue was empty the first line starts at once; otherwise it waits.
   There is no limit on the queue's length.
10. **When a line starts:**
    - its message and its two files are looked up; a line missing from
      the message file is dropped and the next one is tried;
    - the **speaker** is the line's talker object (rule 5);
    - the **voice**: Connor's lines play as a plain sound (no position);
      anyone else's line is placed at the speaker's position at that
      moment, min distance **2,000**, max **30,000** (`sound.md` rule 58);
    - **captions**: if the console variable `Options::Captions` is true
      (`MASK/Options.cs` sets it true), the line's text (rule 3) is shown
      in the caption panel of the GUI;
    - **other sounds are turned down** to **0.2** (`sound.md` rule 53),
      **unless** the `speak` had `nofade`; the answer to `sound.md`'s open
      question "what makes a speech line skip ducking" is this word;
    - the mouth movements of the lip-sync file are scheduled (rule 17);
    - the line lasts exactly as long as its sound file plays.
11. **When a line's sound ends:** the speaker's mouth goes back to rest;
    the issuer is told "line finished" (rule 13; the last line of the
    `speak` is marked as the last); the line leaves the queue and the
    next one starts at once. **When the queue is empty** the caption
    panel is hidden and the other sounds come back to full volume
    (ducking stays on between the lines of one queue).
12. A `speak` changes nothing about the issuer's state or movement:
    Connor can walk, jump and turn while he talks.

### Hearing that a line has finished: `TalkComplete`

13. After **every** line (not only the last), a script runs with `$1` =
    its object's name, `$2` = `TalkComplete`, and `$3`…`$7` = the
    finished line's **file, noun, verb, case, sequence** (as decimal
    numbers). Which scripts:
    - the **issuer's own script**, at once, **except when the issuer is
      Connor** (Connor has no script and his queue never runs one);
    - every object registered with **`KQMonster::stateChangeNotify
      <recipient> <issuer>`** (the recipient's script, a moment later:
      the message is queued, not run at once). Traps can be recipients
      too (`traps.md` rule 15). `KQMonster::clearStateChangeNotify
      <recipient> <issuer>` removes it. That the group passes the message
      to each member is inferred from the data (`wizbook.cs` registers
      for Connor and reacts to his line 1000/63/0/0/1), not traced in the
      group itself.
    - Example (data): the alchemist's sign `alcmsign.cs` makes Connor say
      1000/55/0/0/1…; nobody hears its end. `wizbook.cs` does
      `stateChangeNotify wizbook Connor`, then `speak Connor Connor 1000
      63 0 0 1 1`, and on `TalkComplete` 1000 63 0 0 1 starts the
      ripping animation.
14. **`KQMonster::testMessage f1 n1 v1 c1 [s1] f2 n2 v2 c2 [s2]`** (9 or
    11 words) compares two tuples (with or without the sequence) and
    gives **0 when they are equal** (so `if eval KQMonster::testMessage
    $3 $4 $5 $6 $7 1000 63 0 0 1` runs its branch on a match), 1
    otherwise. The file number is compared as a 16-bit number, the
    others as single bytes. Another word count: both usage texts are
    printed and the answer is 1.

### Cutting a line short

15. **`KQMonster::clearSpeak <name>`** empties that character's queue at
    once: the line playing stops, the waiting lines are dropped, the
    caption goes and the sounds come back up. **No `TalkComplete`** is
    sent for any of them. The mouth goes back to rest. (It also ends a
    conversation line; not covered here.)
16. **The player cuts Connor's line short** (same effect as
    `clearSpeak Connor`) by **pressing** forward/back, a turn key or Jump,
    or by **clicking in the world**, but only when:
    - the line playing has **no "no stop" mark** (`#`, rule 3), and
    - Connor is not in a conversation.
    Releasing keys doesn't count, and `handsOff` swallows the presses
    anyway (`anim-scripts.md` rule 21). In practice: the stock remarks
    and most of Connor's Daventry remarks have `#` and always play to
    the end; the item descriptions of `0.MSG` and the conversation lines
    can be cut short.

### The mouth (lip sync)

17. The lip-sync file (community note `kq8lip.txt`) is a list of pairs
    (time, mouth shape): time in **1/60 s** from the start of the line,
    shape 0–7 (0 = rest). At each time the speaker's mouth changes,
    as long as that line's sound is still the one playing. Shapes above
    7 are handled differently (not traced). A line without a lip-sync
    file simply has no mouth movement.
18. **Connor's mouth** is a texture swap: his model's face texture
    `baseLip=cf0.bmp` is replaced by `lip<n>` of `[ConnerLips]` in
    `GAME/Kq/Connor.kq` (`CF0.bmp`…`CF7.bmp`); a shape of 8 or more uses
    `lip0`. Other characters do the same with their own `lips=` list
    (e.g. `[WizardLips]` `WF0`–`WF7`). With a helmet or armour his face
    texture comes from `lipsList=ConnerLipsList` (`cf`, `chf`, …, not
    traced further; M5). In first person his mouth isn't changed.

### Connor's built-in remarks (traced; text from the data)

19. These are said by the game itself, through the same queue, without
    turning (except the chest):

    | When | Tuple (file noun verb case seq) | Text |
    | --- | --- | --- |
    | Clicking something out of reach whose `tooFarAwayAction` is 0 (`clicking.md` rule 10; `items.md`, `chests.md`) | 0 0 0 1 1 | 'Tis beyond my reach. |
    | Using an item on something that doesn't take it (`clicking.md` rule 12) | 0 0 0 4 1 | This will not work. |
    | A chest's `Take…`/`Empty…` remark (`chests.md` rules 8–9; Connor turns to the chest first unless it has `doNotPointAt`, then 0.5 s) | the chest's `Msg Noun Verb Case Seq End` | e.g. `smlchst3`: 0 109 14 33 1 "'Tis empty." |
    | Daventry only, the first time a monster other than a chicken dies (once per game; not in the demo) | 1000 0 0 62 1 | Whence came these foul creatures? This is madness! |
    | Drawing a weapon in some special states (ropes, puzzles; M5) | 0 0 37 23 1 | A weapon will not help me here. |
    | Inventory screen: certain items and the rope (M4) | file 0: 0/92/14/0/1 (at most every 3 s), 0/0/0/0/3, 0/0/58/37/1, 0/0/58/36/1 | (see `0.MSG`) |

    Monsters also have stored remarks of their own (alarm lines and
    others; M5).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| delay between turning to the listener and the first line | 0.5 s | traced |
| other sounds while a line plays | × 0.2 (none with `nofade`) | traced |
| other characters' voices | placed, min 2,000, max 30,000 | traced |
| lip-sync time unit | 1/60 s | traced, community |
| mouth shapes | 0–7 (0 rest) | traced, community |
| line duration | its sound file's length | traced |
| Daventry lines | 254 (file 1000), all with sound | data |
| stock lines | `0.MSG`, 140 lines, 134 with sound | data |
| lines with the "no stop" mark | 476 of 2,080 (12 files) | data |
| message file version | 5010 | data |

## Data used

- `MASK/<world>/English/<n>.MSG`, `MASK/GAME/ENGLISH/0.MSG`: lines.
- `MASK/<world>/English/A*.*`, `S*.*`: voice and lip-sync files (rule 4).
- `MASK/GAME/ENGLISH/TALKERS.SH`: talker numbers (rule 5).
  (`VERBS.SH` names the verbs, e.g. 14 `DO_V`, 29 `LOOK_V`; only for
  reading.)
- `MASK/Options.cs`: `Options::Captions`.
- `.kq`: `lips=`, `baseLip=`, `lipsList=`, `[…Lips]` `lip0`–`lip7`;
  chests' `Take…`/`Empty…` keys.
- Scripts: `KQMonster::speak`, `clearSpeak`, `testMessage`,
  `stateChangeNotify`, `clearStateChangeNotify`, `TalkComplete`.

## Edge cases

- `<last>` smaller than `<first>`: nothing is queued (not traced
  further; no data does it).
- A `speak` while the same issuer's turn-and-wait (rule 7) is still
  pending: the issuer has only one waiting slot, so the later `speak`
  replaces the earlier one's tuple and both starts play the later
  tuple, once (the second is a duplicate, rule 9). Not seen in the
  data.
- A line whose talker object doesn't exist in the world: not traced
  (the original seems not to guard against it). No Daventry line has a
  missing talker in the scripts that run (not checked one by one).
- A line with no sound file (rule 4: none in Daventry's own file; some
  in Barren, Ice World, Temple 1 and `0.MSG`): what ends it is not
  traced (see "Do not reproduce").
- Connor speaking during `handsOff`: allowed (cutscenes do it); the
  player can't cut it short (presses are swallowed).

## Do not reproduce

- **The "TalkComplete" freeze** (community: kqmoefix's source describes
  it; it matches the trace). A speaker remembers the sound object of a
  line it passed on to its notify group, to avoid passing it back to
  itself, and ignores later "line finished" messages whose sound object
  is the same one. The original compares memory addresses, and newer
  Windows versions often give a new sound the address of the one just
  freed, so a later line's `TalkComplete` is sometimes swallowed and a
  cutscene waiting for it freezes. Do instead: give every line that
  starts its own number, never reuse it, and skip only a message that
  really comes back from the notify group.
- **A line that can't be played** (no sound file, or the talker missing):
  don't leave the queue stuck. End the line after a time worked out
  from its text (for example 0.06 s per character, at least 1 s) and
  log it.
- Keep the rest, including the duplicate check (rule 9) and the 0.5 s
  wait after turning.

## How to check

- **Our logs (B):** per `speak`: issuer, to, tuple range, turned or
  not, queued at once or after 0.5 s, "duplicate: not queued"; per line:
  start time, speaker, sound file name, its length, caption text,
  ducked or not, each mouth change (time, shape); per end: "line
  finished", the `TalkComplete` scripts run (own script / notify
  group), queue empty → captions off, sounds back up; per cut-short:
  what cut it.
- **Daventry checks (B, logs):**
  - Click the alchemist's sign (`alcmsign`, at 245734, 255974): Connor
    turns to it, 0.5 s after the `speak` the line `A0RS1J00.001` starts
    (78,122 bytes), caption "The alchemist's shop. There could be
    something useful inside." (with the leading space), sounds ducked to 0.2 until its end; no
    `TalkComplete` anywhere. Click it 3 more times during the line:
    nothing is queued.
  - Click the same sign from too far (out of reach): "'Tis beyond my
    reach." from `GAME/ENGLISH/A0000000.011` (if `tooFarAwayAction` is
    0 for it), no turn.
  - Open `smlchst3` twice, then click it: Connor turns to it and says
    "'Tis empty." (`A000310E.0X1`).
- **In the original (user, by eye and ear):**
  1. Click the alchemist's sign: does Connor turn first and start
     speaking about half a second later? Does the background sound
     drop while he speaks? Do captions show (they are on in
     `Options.cs`)?
  2. While he says it, press forward: he should **not** stop talking
     (the line has `#`).
  3. Watch his face up close during a line: the mouth texture should
     change.

## Open questions

1. **Conversations** (`KQConner::conversation`, 173 lines in the data):
   the talk state, choosing lines, the `[n]` camera numbers and the
   `<name>Info` objects (`KQConvInfo`), `TalkExit`, how the queue is
   used there. Not studied; needed for the Daventry characters (Wizard,
   Girl, Knight, Unicorn, Uglybeast, Lady of the Lake), M4/M5.
2. Mouth shapes above 7 (rule 17).
3. Which folders are searched for message and sound files (rule 1).
4. Where and how the caption panel is drawn (font, place, wrapping):
   GUI, M4.
5. What ends a line that has no sound file (Edge cases).
