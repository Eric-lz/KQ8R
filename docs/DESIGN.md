# Design

How this project works, in plain language. Updated when the architecture
changes, not on every commit.

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* (KQ8, Sierra 1998) that
reads every asset from the player's own installed copy at runtime. The repo
contains only our own source code; no game files, no decompiled code.

## Ground rules that shape the design

- **Game data is read, never written.** The game folder (`./MASK/` by
  default) is opened read-only. The original game unpacks the current world
  from `data/<World>/*.vol` onto the hard disk; we do not do that. We read
  the `.vol` archives directly into memory instead.
- **Everything we produce goes to `kq8-artifacts/`** (logs, debug dumps).
  That folder is not in the `.gitignore` whitelist, so it is never committed.
  (Same idea as IW4L's `iw4l-artifacts/`.)
- **Log, don't look.** Every tool writes numbers (counts, sizes, offsets,
  timings) to a log so results can be checked without looking at the screen.
- **Match the original.** When a format detail is unclear, check it against
  the real files (and, later, the original `Mask.exe` in a disassembler kept
  outside the repo) rather than guessing.

## Working method for studying the original

Adopted from ai-game-modding-guides, guides 3 and 13 (not legal advice).

1. **Prefer the data files.** Learn formats from the game's own files and
   from community documentation before looking at `Mask.exe`.
2. **Record where every fact came from** in `docs/provenance/`.
3. **If disassembly is needed, split the work across two sessions.**
   Session A studies `Mask.exe` (in Ghidra, with all output kept outside the
   repo) and writes a behaviour description in its own words into
   `docs/behaviour/`. Session B, a fresh chat with no
   access to the disassembly, writes the Rust code from that description
   only. Not a legal safe harbour, but much better than one session
   doing both. Full rules and the prompts for each session:
   `docs/SESSIONS.md` (kept locally, not published).
4. **Never transcribe.** Describe *what* the original does (sizes, units,
   rules, timings), never *how its code is laid out* (struct field order,
   function structure).
5. **Do not reproduce the original's bugs, dead code, or unused fields**,
   such as the "TalkComplete" cutscene freeze that kqmoefix works around.
   Where we deliberately differ, say so in a code comment and in
   `docs/provenance/`.

## Repository layout

Modelled on IW4L (<https://github.com/vladtrc/iw4L>): a Cargo *workspace*
(one repo holding several Rust packages, called *crates*) under `crates/`.

```text
Cargo.toml            workspace definition
crates/
  kq8-data/           library: finds and reads game files (VOL archives,
                      models, terrain, object lists, scripts, ...)
  kq8-core/           library: the game's rules, no drawing (Connor, camera,
                      scripts and events, doors, chests, traps, items, ...);
                      named kq8-game until 2026-10-13
  kq8-cli/            the `kq8` command-line tool for checks and dumps
  kq8-audio/          sound output: our mixer and the sound card (cpal);
                      told what to play by kq8-core's sound rules (M3 3h-2)
  kq8-viewer/         the window (uses Bevy): draws a world and runs the
                      rules; also the developer's test tool (H key, free
                      camera, `--at`, `--click`, `--aim`, detailed logs)
docs/                 DESIGN.md (this file), RESEARCH.md (what exists online),
                      FEATURES.md (what works, in detail)
docs/milestones/      step-by-step plans of finished milestones (M1.md, ...)
docs/provenance/      where each piece of knowledge came from
docs/behaviour/       behaviour notes (session A) and QUESTIONS.md
kq8-artifacts/        logs and dumps (never committed)
reference/            third-party repos cloned for reading (never committed)
MASK/                 the game install (never committed)
```

Data flows game files → `kq8-data` → `kq8-core` → `kq8-viewer`. Open
decision (user): whether the game you play will be a separate program
later (e.g. `kq8-play`, using `kq8-data` and `kq8-core`, with the viewer
staying a test tool); B's recommendation, not decided.

More crates may come later (rendering). Asset *parsing*
stays in `kq8-data` so the engine never touches raw bytes directly. The
viewer follows that rule: `kq8-data` turns a model into a plain list of
triangles (positions, texture coordinates, material), and only the
viewer's `main.rs` knows about Bevy.

## Game data layout (as found in the GOG 1.03 install)

| Where | What |
| --- | --- |
| `data/<World>/{kq,resource,8bit,8gui}.vol` | Packed world data, 12 worlds, 48 archives, 133 MB |
| `data/<World>/{English,KQ,Sound}/` | Loose files next to the archives (speech, messages, ...) |
| `GAME/` | Shared data used by all worlds |
| `<world>/{kq,resource,8bit,8gui}/` | Copies unpacked by the GOG/kqmoevol install script. Used by us **only** as a reference to check our VOL reader |

## VOL archive format

Learned from Nico Bendlin's MIT-licensed `kqmoevol` extractor (see
`CREDITS.md`) and confirmed against the real files:

- 8-byte signature: `RESLIST` followed by a zero byte.
- Then entries, one after the other. Each entry is:
  - 20-byte file name (ASCII, zero-terminated),
  - 4-byte little-endian offset of the *next* entry (from start of file),
  - the file data, compressed with PKWARE DCL "implode" (an old 1980s/90s
    compression method), running up to the next entry.
- The last entry's "next" offset equals the archive size.

Decompression uses the `explode` crate (MIT), a Rust port of Mark Adler's
`blast.c`.

## Roadmap

The goal is a fully playable game. That is a multi-month project; the
roadmap is built around **milestones you can see and test**, each made of
small, separately revertable steps. Steps are planned in detail only when
a milestone starts.

Who does what (see `docs/SESSIONS.md`, kept locally):

- **A** = study session: works out behaviour from `Mask.exe`, writes
  `docs/behaviour/*.md`. No code.
- **B** = implementation session: writes Rust from data files, community
  format notes and behaviour notes. Never looks at `Mask.exe`.

Where the game keeps things (found so far):

- **Data files** say *what* exists: models, textures, terrain, object lists
  (`.kq` files are plain-text object descriptions: class, model, position,
  settings).
- **Scripts** (`.cs`) hold puzzle logic, e.g. "this switch opens that door
  and marks the puzzle solved". We run them with our own interpreter.
- **`Mask.exe`** holds *how* each kind of object behaves (monsters, traps,
  doors, Connor's movement, combat, camera). That needs study session A.

### M0 — Read the game's files (done)

- Step 1: VOL archives. 48 archives, 14,444 files, all match. (done)
- Step 2: images and palettes. 6,495 images, 50 palettes, PNG export,
  confirmed by eye. (done)

### M1 — See Daventry

**You will see:** a window showing Daventry's terrain and objects, with a
free-flying camera.

- Graphics library: **Bevy** (decided 2026-10-11; IW4L uses it). Keep game
  logic in Bevy's ECS (the part that stores game objects and runs their
  logic) and rendering separate, so the drawing part can be
  swapped later if Bevy's renderer ever becomes too slow (ai-game-modding-guides,
  guide 3).
- B: 3D models (`.kqs`) and material lists (`.dml`) → textured model in a
  window. Planned in detail in `docs/milestones/M1.md` ("M1 step 1").
- B: terrain (`kq/Terrain`, `.dib` images) → Daventry ground. Planned in
  detail in `docs/milestones/M1.md` ("M1 step 2"). Correction: `.dig`/`.dil` are not terrain;
  they are named after buildings (`pub`, `castle`, `wndmill`, ...) and go
  with the 33 `KQInterior` objects, so they belong to step 3.
- B: place objects from the `.kq` files.
- A, only if the data does not explain it: terrain size/scale, how textures
  and palettes are applied, lighting (`.ppl` shade/haze tables).

**Done.** Step-by-step plans and results:
[docs/milestones/M1.md](milestones/M1.md). In short:

- Step 1: one textured model in a window.
- Step 2: Daventry's ground.
- Step 3 (3a–3f): Daventry's objects, placed by the world scripts
  (`loadKQ` heights, script variables, `getLoc` and `move`, hidden objects).
- Step 4: ground lighting (light and shadow maps).
- Step 5: object and building lighting.
- Step 6: haze (distance fog) and draw distance.
- Step 7: draw distance answers and point lights (on models, walls and
  the ground; F9).

### M2 — Walk Connor around Daventry

**You will see:** Connor walking, running and turning, the camera following,
not falling through the ground or walking through walls.

- B: animations (`.anm`), Connor's model.
- A: Connor's movement (speeds, turning, jumping, slopes), camera behaviour,
  collision rules, controls of the original.
- B: implement from those notes.

Notes: `docs/behaviour/connor-movement.md`, `connor-physics.md`,
`camera.md`.

**Physics rate (decided 2026-10-11).** The game logic runs at a **fixed
15 updates per second**, as the original does (`KQGame::FPSLimit 15`).
Every per-update value is then used exactly as traced: the fall timer
grows by 0.06 s per update, step up/down are 325 per update, the air
counter counts updates. This is the simplest way to match the original's
jumps, falls and steps (including its gravity running at 0.9 × real time;
see `connor-physics.md` "Do not reproduce"). Drawing runs at the screen's
rate: Connor is drawn **between his last two updates** (blended by the time
since the last update), so motion looks smooth. The blend is display only
and never changes the logic. If a higher rate is wanted later, it is a
separate, logged change.

**Where the code goes.** A new crate `crates/kq8-core` (no Bevy) holds the
game rules: Connor's mover first. It asks the world for the ground under a
point through a small interface, so it can be unit-tested without the
game files. The viewer runs it at 15 updates per second.

**Done.** Step-by-step plans and results:
[docs/milestones/M2.md](milestones/M2.md). In short:

- 2a: Connor stands in Daventry, gravity on.
- 2b: walk, run, back and turn.
- 2c: slopes and falls.
- 2d: walls, floors and objects are solid.
- 2e: jumping.
- 2f, 2f-2: the following camera, and the camera avoids walls.
- 2g: start and stop transitions.
- 2h: water (the floor grid and wading).
- 2i: banking while running.

### M3 — Daventry comes alive (done)

**You will see:** doors opening, traps firing, chests, animated objects,
the first puzzles working.

**Scope: Daventry only** (the user, 2026-10-19). Objects and puzzles
that first appear in later worlds are **not** part of M3, even when the
data or a note already covers them: they come with M6 (the whole game),
or M5 when they hurt Connor. Known ones: Gnome's rolling rocks hitting
Connor (`connor-collision.md`, Q55) and Gnome's pushable `boulder`
(`KQBoulder`). Things built for other worlds along the way (the rolling
rocks' movement and moving boxes, the boulder standing still) stay.

- A: the script language (`.cs` commands like `set`, `if test`,
  `sendEvent`), the event system, and each object kind used in Daventry
  (door, trap, chest, emitter, animation lists...).
- B: script interpreter, event system, the object kinds.

Notes: `docs/behaviour/doors.md` (doors, their scripts, the minimum of the
`.cs` language), `docs/behaviour/emitters.md` (emitters, particle
catchers, ground animators).

**Done** (2026-10-19, the camera checked by the user against the
original). Step-by-step plans and results:
[docs/milestones/M3.md](milestones/M3.md). In short:

- 3a: doors and the script runner.
- 3b: emitters, particle catchers, ground animators; 3b-4 emitters that
  follow something.
- 3c: animated objects play.
- 3r: review after the answers to Q45–Q52.
- 3d: traps, and what the walk/run zones do.
- 3e: chests and items.
- 3f: clicking objects that move at load, `deleteObject`.
- 3g: the clicking and script answers (Q54, Q60–Q64).
- 3h: sound effects, looping sounds, footsteps, world music.
- 3i: animated objects run by scripts; 3j pushing; 3k the bricks puzzle.
- 3l–3n: cutscenes: camera and controls, camera flights and cuts, when
  the camera updates.
- 3o: closing M3: the view heading (four states), the tilt and the aim
  height, the mouse, the exact ground test, a script's `move` and traps;
  3o-7 the right button after the user's check.

Left for A, not blocking (`docs/behaviour/QUESTIONS.md`): Q113 a–c (the
view heading's state around cutscene cameras; the user's check matched
our stand-in after the watermill scene).

### M4 — Talk, listen, use the menus

**You will see and hear:** dialogue text, speech with lip sync, the
inventory and menus.

- B: messages (`.msg`), audio (`.aud`), fonts (`.pft`), GUI files.
- A: conversations, inventory rules, menu behaviour.

- **4a — remarks and speech (planned 2026-10-18; done 2026-10-18,
  4a-1 to 4a-6; Q103 asked).** From
  `docs/behaviour/talking.md` (A, 2026-10-18). A **line** is one entry
  of a message file with its own voice file; a **queue** is the list of
  lines one character still has to say; a **remark** is a line Connor
  says about something he clicked. Conversations (choosing what to say)
  are not studied and not in 4a. One commit per step; every step logs to
  `kq8-artifacts/logs/talk.log` (new).
  - **4a-1, reading the files** (**done**; `kq8-data/src/msg.rs`, new; rules 1–5,
    17, the community notes `kq8msg.txt` / `kq8lip.txt`). Message files
    (`<n>.MSG`: lines with noun, verb, case, sequence, talker, text),
    the stage directions taken out of the text (the "no stop" mark `#`),
    the voice and lip-sync file names (base 36), `TALKERS.SH` (talker
    number → object name, the ten "generic" talkers), lip-sync files
    (time in 1/60 s, mouth shape). New check `kq8 msg-verify`
    (`msg-verify.log`): per file the number of lines, of `#` marks and
    of lines without a voice file, to compare with the note's numbers
    (Daventry 254 lines, 100 with `#`; `0.MSG` 140, 21 with `#`, 6
    without a voice; 476 `#` in 2,080 lines).
  - **4a-2, the queues and the script commands** (**done**; `kq8-core/src/talk.rs`,
    new; `active.rs`; rules 6–9, 11, 13–15, "Edge cases", "Do not
    reproduce"). `KQMonster::speak`, `clearSpeak`, `testMessage`,
    `stateChangeNotify`, `clearStateChangeNotify` stop being stubs.
    One queue per character; the duplicate check; `<last>` 0; turning to
    the listener (Connor only in Wait or Move, through the existing
    scripted turn, then the 0.5 s wait; one waiting slot per issuer). A
    line lasts as long as its voice file (counted in game time, 15
    updates per second); without a voice file 0.06 s per character, at
    least 1 s (the note's suggestion, logged). At each line's end:
    `TalkComplete` with the tuple to the issuer's own script at once
    (not Connor's), and through the event queue to every object
    registered with `stateChangeNotify` (a registration of the issuer
    for itself is skipped: that is the message "coming back"). Every
    line gets its own number, never reused (the TalkComplete freeze is
    not reproduced). Lip-sync changes are scheduled and logged here.
    `eval` (used as `if eval KQMonster::testMessage …`, not described
    in any note): STAND-IN, runs the rest of the line as a command and
    keeps its result; question Q103.
  - **4a-3, hearing it** (**done**; viewer `talk_view.rs`, new; `sound_view.rs`;
    rule 10, `sound.md` rules 53, 58). The voice plays in group 3:
    Connor's plain, others placed at the speaker (2,000 / 30,000).
    While a line without `nofade` plays, groups 0, 2 and 5 are turned
    down to 0.2. `clearSpeak` stops the voice at once.
  - **4a-4, captions** (**done**; viewer; rule 10). With `Options::Captions` true
    (from `Options.cs`) the line's text is shown at the bottom of the
    screen; hidden when the queue is empty. STAND-IN: where and how the
    original draws the panel is not studied (open question 4); plain
    white text on a dark band.
  - **4a-5, Connor's own remarks and cutting a line short** (**done**; `active.rs`,
    viewer; rules 16, 19). "Too far" (0 0 0 1 1), "This will not work"
    (0 0 0 4 1), a chest's `Take…`/`Empty…` remark with the turn to the
    chest unless `doNotPointAt`. Pressing forward/back/turn/Jump or
    clicking in the world stops Connor's line when it has no `#` mark
    (not during `handsOff`).
  - **4a-6, the mouth** (**done**; viewer; rule 18). Connor's face texture
    `cf0.bmp` is swapped for `lip<n>` of `[ConnerLips]` at each mouth
    change. Other characters are not drawn yet (cubes), so only logged.
  - **4a-7 to 4a-9, A's answers to Q104 and Q103 (e)** (planned and
    **done** 2026-10-18; checked by the user in the viewer, like the
    original; Q105 asked). One commit per step; each step logs to `connor.log`
    and `doors.log` / `talk.log`.
    - **4a-7, turn keys are events** (**done** 2026-10-18; `connor-movement.md` "Turning"
      rule 2, `anim-scripts.md` "Edge cases"). Today Connor's turn
      direction is read from the held keys every update, and any
      non-zero turn cancels the automatic turn. New: a small core
      module `turn_keys.rs` keeps the turn direction and changes it
      only on key events: a key going **down** sets it (+1 left, −1
      right) and is the only thing that cancels an automatic turn;
      a key coming **up** sets it to 0, or, if the other turn key is
      still held, sends that one again as a fresh down (which cancels
      too). While the automatic turn runs it overwrites the direction
      every update at 3.5 rad/s (a held key adds nothing); when it
      ends the direction is set to 0, so a held key does not take
      over. Forward/back and the run switch stay the player's during
      a turn without a cue (click, `speak`, chest), so he walks or
      runs through it. `handsOff`: presses swallowed as now; when it
      goes off, the held turn key is sent again as a down (rule 22).
      `--turn N,dir` becomes "pressed at N and held"; `--hold`'s turn
      is pressed at the start and let go at its end. Logs: every turn
      key event (down/up, "sent again"), "auto-turn cancelled" only on
      a down, and "cue lost" when a turn with a cue is cancelled; per
      turn update the heading, turn direction and forward.
    - **4a-8, the turn after a click** (**done** 2026-10-18; `clicking.md` rules 25–33,
      `doors.md` rule 16). After every `ConnorAction` call (doors,
      scripted objects, scripted clickables, buildings with a script,
      items with a script) the click asks for a turn to the object's
      position (its placement point), unless it has `doNotPointAt`
      (items: on by default). The request goes into the same list as
      the script's own requests, after them, so a `setState` in the
      script is applied first; the viewer then looks at Connor's
      state (Wait or Move: our only states of rule 28's list) and
      starts the usual automatic turn, or logs why not. One
      automatic turn: a new turn request while a turn without a walk
      is running replaces only its target (a `pointAndCue` keeps its
      cue). Stock remarks never turn him (no change). Another
      character's `speak` turn (Q103 d): only logged, monsters don't
      move yet. Logs: per click the call kind (`do`, item, `TooFar`,
      `Special`, none), Connor's state after the script, and `turn
      to <object>` or the reason there is none.
    - **4a-9, the chest remark turns him in any state** (**done** 2026-10-18; `talking.md`
      rule 19, `chests.md` rule 10): the "not in Wait or Move → at
      once" branch goes; only `doNotPointAt` gives no turn and no
      wait.
    - Q103 (a)–(c): our stand-ins are right; only the provenance
      notes change.
    - Tests: unit tests for the key events and the click turn; viewer
      runs with `--turn` and `--click` on `alcmsign`, `tmbstn1`,
      `tmbstn2`, `wizbook`, `urn` and `smlchst3`.
  - Not in 4a: conversations; item descriptions in the inventory (no
    inventory screen, rule 20); the Daventry monster-death remark (no
    monster dies yet); helmets' and armour's face lists (M5).
  - Tests: `cargo test --workspace`; `kq8 msg-verify`; viewer runs
    clicking the alchemist's sign (`alcmsign`), `smlchst3` and `wizbook`
    with `--click`, read in `talk.log`, `sound.log`, `connor.log`.

- **4b — the interface's files: fonts, screen layouts, captions in the
  original font** (planned 2026-10-18). From the game's data and the
  community notes `kq8pfon.txt`, `kq8pbma.txt`, `kq8gui.txt`,
  `kq8tag.txt` (libkq8fpc, GPL-3.0: facts about layouts only, code and
  wording our own). No behaviour note yet: A is studying the interface
  (`interface.md`, `inventory.md`). A **font** is a set of small
  letter pictures (glyphs) with a table saying which picture each
  character uses. A **layout** (`.gui`) is a tree of boxes on a
  640×480 screen, each with an id, a rectangle, and a picture or a
  font and text. A **tag** is a number that names a font, picture or
  text; `kqGuiTag.TTAG` lists them with their names.
  - **4b-1, fonts** (**done** 2026-10-18; `kq8-data/src/font.rs`, new, and a bitmap-array
    reader for `.pba`). Read every `.pft` in `GAME/8Gui` and
    `GAME/Resource`: the font info (flags, glyph count, height,
    baseline, colours, scale, spacing), the character map, the glyph
    table and the glyph pictures. New check `kq8 font-verify`
    (`font-verify.log`): per font the glyph count, height, characters
    covered, whether every glyph fits its picture, and the file read to
    its exact end. Optional PNG sheet of each font for the user's eye.
  - **4b-2, layouts and tags** (**done** 2026-10-18; `kq8-data/src/gui.rs`, new). Read the
    tag file and the nine `.gui` files in `GAME/Resource`. New check
    `kq8 gui-verify` (`gui-verify.log`): each layout as a tree, with
    ids and tags turned into their names, rectangles, fonts and
    pictures; every file read to its exact end. This shows what the
    caption box (`IDDLG_CAPTION`, in `Gplayscr.gui`) and the health
    panel are made of.
  - **4b-3, captions in the original font** (**done** 2026-10-18; checked by the user: colours right, breaks mostly right; the strip narrowed to the text after the check; `kq8-core/src/caption.rs`,
    new: wrapping and the picture; viewer `talk_view.rs`). From the
    layout (`IDDLG_CAPTION`): the font (`console.pft`, the world's),
    the strip picture (`stontran.pbm`), 16 pixels per line, text 2
    pixels down. From the user's check in the original (2026-10-18):
    the caption sits at the bottom of the screen, just above the status
    bar (at the very bottom when the bar is hidden; we draw no bar yet,
    so at the bottom); the text is centred; the stone strip grows
    upwards one line at a time until every line fits; the alchemist's
    sign breaks after "something". Measured: that first line is 365
    pixels wide in the font, with "useful" 419, so the wrap width is
    between 365 and 418, not the layout's 320. Colours: the world's
    palette for both (gold letters with a black edge, grey stone;
    measured from the palette values), index 0 see-through. STAND-INS
    until A's note (Q106, Q107): wrap width 400, greedy word wrap,
    the strip across the full width of the screen, tiled from its
    top-left; the 640×480 picture stretched to the window.
  - Tests: unit tests on hand-made files; `font-verify`, `gui-verify`
    on the install; viewer run with the alchemist's sign.

- **4c — the on-screen interface and Connor's inventory** (planned
  2026-10-18). From `docs/behaviour/interface.md` and `inventory.md`
  (A, 2026-10-18, checked by the user in the original), `clicking.md`
  rules 12–18 and 34–36. One step and one commit at a time; every step
  logs to `kq8-artifacts/logs/gui.log` (new) and the logs it touches.
  Words: a **panel** is one of the boxes of `Gplayscr.gui` (top bar,
  bottom left, the eight slots, the meters, bottom right, map, caption,
  status icons); **sliding** is a panel moving on or off the screen in
  0.2 s; the **hand** is the item Connor holds as the mouse pointer.
  - **The screen size (stand-in for all steps, question Q108).** The
    shipped `Options.cs` says `set800mode 1`, so the game's screen is
    800 × 600 and the 640 × 480 layout is scaled by 1.25 (rule 47:
    inferred; the user's measured caption width fits it). How the
    panels' pictures are scaled is not studied: we stretch them with
    the layout (nearest pixel) and tile a picture smaller than its box
    (the 6 × 6 meter bar, the 64 × 64 stone); fonts are drawn at their
    own size (rule 47). The 800 × 600 picture is stretched to the
    window, as the captions are now.
  - **Where the code goes.** `kq8-core`: `caption.rs` (wrapping, the
    box's place), new `panels.rs` (which panels are shown, sliding,
    pop-up timers, the keys and the reasons), new `meters.rs` (the
    percentages and bar widths, the five values sent), new
    `gui_draw.rs` (a panel drawn into a picture from its layout block:
    pictures, numbers with their justification, bars cut to a width),
    new `inventory.rs` (Connor's inventory: categories, adding,
    stacking, swapping equipment, the quest strip, the hand, applying
    items). Viewer: new `gui_view.rs` (loads the layout, pictures and
    fonts of the world's `8gui`, draws each panel as one picture,
    moves it, takes the keys and the mouse on the panels).
  - **4c-1, the caption strip** (**done** 2026-10-18; interface.md rules 46–52, Q107). The
    stone covers the **whole caption box** (400 wide at 800 × 600), not
    the text (the user's check). Wrapping as rule 47: widths summed
    letter by letter; a letter that would pass the box's width ends
    the row at the last space before it (the space dropped, spaces that
    fit at a row's end kept); a row with no space breaks at the box's
    edge ("Edge cases", our choice). Height rows × 15 + 4, growing
    upwards; each row centred in the box, 2 pixels down in its own
    15-pixel slot. Place: bottom edge = top of the highest bottom panel
    (no bottom bar until 4c-2: the screen's bottom); centre half-way
    between the map window's right edge and the screen's right edge;
    there is no map yet (M6), so it is always closed (right edge at
    −50/640 of the screen): centre x 368.75 of 800, a little left of
    the middle (rule 51, the user's check). The stand-ins `WRAP_WIDTH`
    and `STRIP_MARGIN` go. Log: rows, their widths, box width, left,
    top, bottom.
  - **4c-2, the always-visible panel** (**done** 2026-10-18; interface.md rules 7–36). The
    bottom bar: bottom left (armour, hand and ranged weapon slots with
    their numbers), the eight slots (empty pictures; their contents
    come in 4c-3), the meters (red health and green experience bars,
    the level number), bottom right (empty rope, rocks and map-icon
    pictures). Panel groups and sliding (rules 8–9: 0.2 s of real
    time, panels off the bottom edge). **Space** (from `movekeys.kq`
    row 12) puts the bottom bar away or brings it back (rule 10).
    `handsOff 1` puts it away if shown (remembered) and shows the
    spinning mask in the status icons; `handsOff 0` brings it back
    only if it was shown (rule 15). Conversations (rule 16): the panel
    side is built and unit-tested, but nothing calls it yet
    (conversations are not studied). Script commands `setLowerGui`,
    `setUpperGui`, `setMapGui`, `getGuiStatus` (rule 15; no Daventry
    script uses them). Meters: health % = health × 100 / (10 × level),
    at least 1 when not 0; experience % = experience × 100 /
    threshold; bar width = width × % / 100, at least 1 pixel (rules
    20–23). After every stats change on Connor all values are sent
    (rule 24); a health or experience % sent while the meters are
    hidden slides them up alone and away 1.5 s later (rule 25). Slot
    pictures from what Connor wears (rules 34–35); his start inventory
    from `Connor.kq` `[ConnerInventory]` (`inventory.md` rule 4: the
    first mask piece, the three placeholders, the mask-button piece).
    Numbers in `14shdw.pft`, right/top (rule 36); a changed number
    plays `fx.pba` once over its slot (rule 33; frame rate a stand-in,
    Q108). The caption box sits on the highest bottom panel (rule 51).
    Not built: the red "alternative" numbers (potions, M5), the
    level-up flame (not traced), the map (M6), the help popup.
  - **4c-3, the inventory bars** (**done** 2026-10-18; interface.md rules 12–13, 38–45;
    inventory.md rules 1–11). Adding an item (rule 8): stacking
    (`Comb` 1) into one entry; equipment pushes out what shares its
    place (placeholders deleted, other items put into the world where
    the new one lay) and is put on (stats added); the pickup sound (as
    now). The top bar: slides down when the pointer reaches the top
    row, up again below y 60 (rule 12; not while the right button is
    held); pops down for 1.5 s for a new quest item or mask piece and
    the slot blinks 5 times, 0.25 s each (rules 13, 44). The quest
    strip: 7 slots in the order items were added, the arrows scroll
    by one (rules 5–6); the gold picture by count (rule 43). The eight
    slots with their counts (rules 38–39); **keys 1–8** and clicks use
    a slot (rules 40, 9): the item is applied (stack −1, its five
    numbers added through the stats event; `Dur` −1 heals for good;
    `Dur` > 0 runs a timer and takes the numbers back at its end;
    rules 13–15) and its `useSnd` plays. Bottom right: rope and rocks
    (rule 45; the rocks picture levels are not traced: stand-in).
    Right click on a slot: Connor's description of the item
    (`talking.md` rule 20; rule 10). The mask button opens no menu
    yet (logged). A click on a shown panel never reaches the world.
    Clicks on the bars are ignored during handsOff (rule 9).
  - **4c-4, the item in hand** (**done** 2026-10-18; inventory.md rules 7, 9, 12; clicking.md
    rules 12–18). A left click on a quest item, the gold, the rope or
    the rocks makes it the hand: the pointer becomes the type's
    `cursor` picture (a picture following the mouse; the play pointer
    `curs04.pba` otherwise) and the item leaves the strip (the others
    close the gap, rule 7). A world click with the hand is the click
    built in 3g (accept lists, `$3` / `$4`): accepted → the hand is
    emptied (not for money while `goldCoinPersists 1`); refused or out
    of reach → it stays. Clicking the item's own slot puts it back.
    `INVITEM_Scroll` and the rope's remarks (rule 9). `--item TYPE`
    now gives Connor the item and puts it in his hand. Escape still
    quits (stand-in): the original opens the main menu and **the game
    stops running while it is open** (rule 14, the user's check); a
    later menu step must pause the updates and the panel timers.
  - **4c-5, A's answers to Q105** (**done** 2026-10-18; clicking.md rules 34–36). (a) the
    turn to an object deleted by its own script: no change, provenance
    only; (b) the click looks at the state the script **asked for**
    (or his state at the click), not the one after his next update;
    (c) `doNotPointAt` is read for buildings too.
  - **After the user's check (2026-10-18)**:
    - **4c-6, a 4:3 screen in the middle of the window** (**done** 2026-10-18). The original
      runs at 4:3 and its panels are laid out for it; stretching them to
      a 16:9 window pulls them sideways. The user chose: the panels and
      the caption sit on a "virtual" 4:3 screen, the largest one that
      fits, centred in the window; the 3D world still fills the whole
      window. `kq8_core::panels::screen_frame(window)` gives that
      rectangle; the viewer places every panel, the caption, the
      pointer's screen position, the click areas and the pointer's size
      from it. Later, maybe: the corner panels in the window's corners
      (the user's third idea).
    - **4c-7, the user's corrections** (**done** 2026-10-18). Space plays `intrup.aud` /
      `intrdown.aud` like the top bar (handsOff does not); the spinning
      mask in the bottom right corner of the screen, smaller. Both from
      the user's check in the original, against `interface.md` rules 9
      and 7 (asked as Q111).
    - **4c-8, the user's second check** (**done** 2026-10-18): Space's
      sounds the other way round; the spinning mask whole (its pictures
      are 57 × 57, its box 48 × 48); black behind the meters' bars.
    - **4c-9, the gap and the mask on the bar** (**done** 2026-10-18):
      the gap beside the meters was ours (their picture drawn 18 rows
      for a 17-row place, squeezed by the engine); every panel is now
      drawn with its edges rounded where they stand on the screen. The
      spinning mask stands on the bottom bar like the caption.
    - **4c-10** (**done** 2026-10-18): the rest of the gap was Bevy
      keeping each picture's proportions inside its box; the pictures
      now stretch to fill their box. The mask doesn't rise with the
      meters alone.
    - **4c-11, corner panels (an improvement, off by default; done
      2026-10-18).** The user's third idea: on a window wider than 4:3,
      the bottom bar's left panel (armour, weapons) goes to the
      window's left edge and its right panel (rope, rocks, map icon) to
      the right edge; the slots, the meters, the top bar and the caption
      stay on the centred 4:3 screen, and the spinning mask follows the
      right panel. `--corners on|off` at the start, **F6** while
      playing. Only the panels' horizontal place changes (clicks follow
      it); everything else is as in 4c-6.
  - Tests: unit tests for each core module; viewer runs with `--click`
    on `alcmsign` and `smlchst3`, Space pressed by a test option
    (`--key KEY,N`, new), the bricks puzzle (`interface.md` check 2),
    the farm coins and the mushroom baskets (`inventory.md` checks
    2–3), read in `gui.log`, `items.log`, `doors.log`, `talk.log`.

- **4d — characters: drawing and animating them, not fighting**
  (planned 2026-10-19). From `docs/behaviour/characters.md` (A,
  2026-10-19, checked by the user in the original) and the notes it
  builds on (`anim-scripts.md`, `objects.md`, `object-lighting.md`,
  `talking.md`, `conversation.md` rule 18). Until now every
  `KQMonster` is an orange cube. Words: a **character** is an object of
  class `KQMonster` or `KQGeometryTalker`; the **monster list** is a
  world's `monsters.kq` (one tab-separated line per character); a
  **type** is a line of `GAME/Kq/Monstats.mtl` naming the character's
  own `.kq` file; a **state** is what the character is doing (Birth,
  Wait, Move, Special, Talk, Die, or none). Fighting, wandering,
  noticing Connor and idling are M5: before M5 every character stays
  "not active" (rule 32). One step and one commit at a time; every step
  logs to `kq8-artifacts/logs/characters.log` (new) and the logs it
  touches.
  - **Where the code goes.** `kq8-data`: new `characters.rs` (the
    monster list, the type list, a character's keys, its `.anm` list).
    `kq8-core`: new `character.rs` (the states on top of the existing
    movement player `named_anim.rs`; a character is an `AnimObject`
    with a `character` part, so the existing script commands reach it).
    Viewer: `load.rs` places characters; `anim_view.rs` draws them
    (it already shows a different model per movement).
  - **4d-1, reading characters** (**done** 2026-10-19; Daventry 22 + 2
    files, 52 lines, 0 problems; frame counts as the note; Q116 asked)
    (rules 1, 3, 4, 6, 25; data). The
    monster list line (20 fields, `@` = the type's value, `none`,
    quotes dropped, the skin swap `type(a.bmp,b.bmp)`, the movement
    field `none` / `loop n …` / `patrol n …` / `random`, z −1 = on the
    ground); the type list (type → `.kq` file); a character's keys with
    their defaults (`flags` 0x10, `shadowMask` 2, gravity on, `noTurn`,
    `wanders` 1, `lips`, `baseLip`, `monsterType`, `idleAnimFile`,
    `originalBmp` / `alternateBmp`); the `KQLips` section; the `.anm`
    movement list (as `Conner.anm`). New check `kq8 character-verify`
    (`character-verify.log`): per world every character file, its
    movements (and whether it has `stop`, `Birth`, gestures, the
    geometry mouth movements), its lips, every monster-list line with
    its type, file and box (`BOUNDS.KQ` by type, else by name). Numbers
    to expect (note "How to check"): Daventry 22 `KQMonster` + 2
    `KQGeometryTalker` files, 52 lines; `wizstop` 31 frames,
    `chk_stop` 30, `UG_STOP` 40, `Ravnstp` 46.
  - **4d-2, characters drawn where they stand** (**done** 2026-10-19:
    57 drawn in Daventry, the 21 zombies hidden; also clickable by
    their rule-9 box, and a drawn animated object now follows a
    script's `move`) (rules 3–5, 7, 9, 10,
    16, 33–35). Characters from their own `.kq` (script `loadKQ`, also
    the copies loaded while playing: the Lady, the ravens, the
    Unicorn) and from the monster list (`loadKQ monsters.kq`): the
    type's file read, renamed, the line's position and heading; z −1 on
    the ground, `keepOnGround` as for objects; the skin swap. Every
    movement of the `.anm` drawn as for animated objects (one model per
    movement). Characters with a `Birth` movement (the zombies) are not
    drawn (rule 16). Their box ball from `BOUNDS.KQ` (type, else name)
    for near / far / off. Still not solid for Connor (as now). The
    orange cubes go. Movements play as for plain animated objects in
    this step (the states come in 4d-3). Logs: per character its name,
    class, file, position and heading after the ground rules, list,
    box and how it was found.
  - **4d-3, the states** (**done** 2026-10-19; Q119, Q121 asked; Q121 was first written as a second Q118)
    (rules 11–19, 22). A character updates only
    while near or far, not off or hidden (the viewer tells core which
    are near or far each update). The first update (rule 12): a waiting
    first movement → **no state** (rule 15), else the current movement,
    then Birth (if the list has `Birth`) or Wait after `birthDelay`
    seconds. Wait asks for `Stop` (random frame and speed 0.9–1.1 with
    `flags` bit 0x10); after any one-shot except `Stop` and gestures,
    Wait asks for `Stop` again (always random). `KQMonster::setState
    <name> <state> [words]` for characters: Wait, Unborn, Special (the
    same list handling as Connor's, then Wait), Die and Move logged
    (M5), others logged and ignored (Do not reproduce). `setHome`
    (never moves), `setCanTurn`, `setUseGravity`, `setMonUseGravity`
    kept for later steps; `changeAnmFile` logged (not traced far,
    open question 4). Logs: first update number and how the first
    movement was chosen, the state, every movement start with its
    reason, the `Stop` start frame and speed.
  - **4d-4, turning** (**done** 2026-10-19; the Talk state's turn every
    4th update waits for conversations) (rules 23–24). `speak` with a listener turns the
    character toward it at 3.5 rad/s on the spot, the movement going
    on; a newer request replaces the target; `noTurn` (and
    `setCanTurn 0`) blocks it. Logs: every request, blocked or not, and
    the heading per update while turning.
  - **4d-5, the face and the hands** (**done** 2026-10-19; Q120 asked;
    new test option `--command "WORDS",N`) (rules 25–28; Q115). The mouth
    texture (`lips`: the face texture `baseLip`, else `lip0`, swapped
    for `lip<n>`, kept across movements, 0 at the start and after each
    line); the Uglybeast's and the Unicorn's mouth movements (codes
    0/16 `rest`, 1–7 `f m r w ih ah e`, started at once; 8–15 ignored);
    gestures for other characters (codes 8–15 `gesturea`–`gestureh`, 16
    `stop`, only when not walking and the current movement can be cut
    short or has ended, and the list has it); after the last line of a
    queue the speaker goes to Wait (Q117). Logs: every mouth change
    (shape, texture replaced, drawn), every gesture and why or why not.
  - **4d-5b, `makeTranslucent`** (**done** 2026-10-19; rule 22,
    `objects.md` "See-through objects" rule 2): the see-through value
    of an animated object or character, at once, no fade; the object
    gets its own copies of its materials. Connor's (Dead City) not
    built.
  - **4d-6, gravity and steps** (rule 8). **4d-6a done** 2026-10-19: a
    "gravity check" line per character in `characters.log` (how far
    above the floor and surface grids it starts). The rest **waits for
    Q122** (a character's collision type and mask, buildings and the
    terrain in its passes, flying characters, falls). Characters with gravity on
    fall onto the ground or a building floor as Connor does, step up
    and down 325; `monUseGravity=0` and `setUseGravity 0` keep them at
    their height. Planned in detail when 4d-1 to 4d-5 are done (how
    much of Connor's mover can be shared).
  - **4d-7, shadows** (rule 36). **4d-7a done** 2026-10-19: who casts a
    shadow (`shadow_mask_setting`, `casts_shadow`; a line per character
    in `characters.log`). Drawing **waits for Q123** (direction, what
    it lies on, how dark, plain objects, `shadowvisible`). Who casts one (world mask 15 from
    `Options.cs`, `shadowMask`, solid only). Look from the user's
    check: a dark copy of the figure on the ground, each part
    darkening on its own. The direction is not traced (open question
    2): stand-in straight down, asked as a question. Planned in detail
    when it starts.
  - **A's answers Q116, Q117, Q121–Q124** (planned 2026-10-19, after 4e).
    One step and one commit each:
    - **4d-8, the names start as `Stop`** (**done** 2026-10-19; `anim-scripts.md` rule 3
      correction, `characters.md` rules 12, 14). Every animated object
      and character starts with current and waiting movement `Stop`;
      `currAnimName` / `nextAnimName` replace them. So characters
      without `currAnimName` start on `stop` (not movement 0), and the
      `flags=8` torches and fish get the random start. Doors and chests
      keep today's load names until Q127 is answered (their own `open`
      could otherwise end in a leftover waiting `Stop`).
    - **4d-9, the start-up** (**done** 2026-10-19; rule 12, Q116, Q121). It runs at the first
      update or the first movement request (`KQAnimPlayEvent`, a state,
      a mouth code or gesture, a conversation), whatever the range;
      only with no state does it queue Birth or Wait, and the Wait comes
      `birthDelay` seconds later in game time, through the queue (so
      with 0 it is handled before the first picture, Q119). Mouth shape
      0. The stand-ins of Q121 and the "empty name, no state" step go.
    - **4d-10, the end of a `speak`** (**done** 2026-10-19; `talking.md` rule 13a, Q117): on
      the line marked last of a `speak` (not when the queue runs dry),
      the queue's owner, if a character, gets `Stop` and Wait before
      its `TalkComplete`; nothing if it is already in Wait; from Special
      only once the list's last movement has started; never Connor.
    - **4d-11, transitions on characters** (**done** 2026-10-19; rule 20, Q124 a, b; rules 2–6
      of `connor-movement.md`): the movement player chooses a
      transition by its from point, waits for it, plays the transition
      movement, then starts its target at its phase. The transition's
      end is an ordinary end (`EndLoop`, `Cue 300`, Wait's `Stop` at a
      random frame). The `endLoop` cue waits for M5 (only on the
      Knight's `attack`).
    - **4d-6b, gravity** (rule 8, Q122; `connor-physics.md` "Other
      characters on the same mover"). Characters use Connor's mover
      (`connor_mover.rs`), with their own collision type and mask (0x42 /
      0x281F unless the file says), their `BOUNDS.KQ` box as the ball
      stack, steps 325 (or `stepUpAmount` / `stepDownAmount`), and
      `keepOnGround` (−1 floor, 1 water surface, 0 none). It runs in the
      character's update only (near or far, not hidden), standing still
      (no walking before M5); a script's `move` sets its position first.
      - **4d-6b-1, the mover's switches** (**done** 2026-10-19; core): no Fall state and no
        "steep ground switches gravity on" without type bit 4; the terrain
        in the up and down passes only with mask bit 0x800; the ground
        check onto the water surface for `keepOnGround=1`. Connor keeps
        all three as now. Unit tests.
      - **4d-6b-2, characters fall** (**done** 2026-10-19; viewer, a new fixed-update step
        right after the characters' update): buildings count when the
        character's mask has 0x8 (every building type in the data has
        it: 0x28, 0x88); `colliderMask=0` (the Wizard, the Knight): no
        terrain, no buildings, only the ground check. Objects, other
        characters and Connor **wait for Q128** (which of
        `connor-collision.md` rule 3's reactions a character's own move
        has). `monUseGravity=0`, `setUseGravity`, `setMonUseGravity`.
        Logs in `characters.log`: per character the first mover update
        (start z, ground under it, what it collides with), every landing
        (on what, z), and a line when it stops moving.
      - **4d-6b-3, the flying mover** (**done** 2026-10-19; `flyingObject=1`: the bats, the
        Knight): no gravity and no steps; each update down by up to 150
        (stopped as in the down pass) and up by up to 300 onto a surface
        above its feet within its height. The note is "not sure of every
        detail": built as written, logged.
    - **4d-7b, shadows** (rules 36, 37, Q123; the user's check: a dark
      copy of the figure, each part darkening on its own).
      - **4d-7b-1, the geometry** (**done** 2026-10-19; core, new `shadow.rs`): the direction
        (the first sun's aim made length 1, else (1, 1, −1)); each model
        triangle facing the sun, cut to the part above the receiving
        plane, projected along the sun onto that plane lifted 20 toward
        its front, cut to the receiving triangle; the terrain receivers
        (the drawn surface grid's triangles under the box moved along the
        sun, only within 1,500 of the ground). Unit tests.
      - **4d-7b-2, characters' shadows on the terrain** (**done** 2026-10-19; viewer): every
        frame, for each character that casts (rule 36) and is drawn, its
        shown model triangles go through 4d-7b-1 into one mesh, drawn
        unlit with multiply blending at 0.5 (each layer halves what is
        under it, two layers a quarter: the user's check), no haze. Logs:
        per character the triangles cast and drawn, once a second.
        Connor casts too (4d-7b-2b, Q130). **Checked by the user
        (2026-10-20)**: the look, wading and the jump's cut match the
        original. **Not reproduced (the user's decision):** the original's
        shadows flicker and show gaps, seemingly at random (cause not
        known); ours are drawn steadily every frame.
      - **4d-7b-3, buildings receive them** (**done** 2026-10-20, Q135 stand-ins; **checked by the user
        (2026-10-20)**: as expected, shadows show inside a building when
        Connor is outside, as in the original; the original cuts some
        shadows off, **not reproduced (the user's decision)**; rule 37
        (2): active buildings within reach, surfaces near the character's
        height, an area at most 1,800 across). Read against the data
        first (`kq8 building-verify`): half of all floor surfaces are wider
        than 1,800 (Daventry's: tiles up to about 2,040), building balls
        876–18,990. What the note leaves open is **Q135** (the reach's
        sizes, the band's numbers, the area's shape, centre and whether it
        cuts, invisible surfaces); built with these **stand-ins**, each
        logged:
        - reach: centre-to-centre distance ≤ building radius + caster's
          radius + 500; the building's ball as our draw distance has it
          (the `.dig` box middle, turned); the caster's ball = the box
          around its shown model triangles (middle, half diagonal);
        - only buildings that are near or far (the draw distance's state;
          off = not active);
        - band: a surface counts when any part of it lies between 2 × the
          caster's height below its feet and 100 above its top;
        - area: a square 1,800 on x and y around the caster's middle moved
          along the sun down to its feet; a surface counts when its x, y
          extent reaches into it, and receives whole (no cut);
        - only drawn surfaces (not kind-0 materials or surfaces skipped
          for having no light map): a shadow on an invisible wall would
          float.
        - **4d-7b-3a, the plan, the data and Q135** (**done**; docs, the
          `building-verify` lines).
        - **4d-7b-3b, the choice of surfaces** (**done**; core `shadow.rs`): a
          building receiver (its ball, its drawn surfaces as world
          triangles with their box), the reach / band / area test giving
          the receiving triangles and counts for the log. Unit tests.
        - **4d-7b-3c, drawn** (**done**; viewer): at load, each drawn building copy
          becomes a receiver (buildings don't move); the draw distance
          shares each object's state; characters, Connor and plain
          objects add the chosen building triangles to their terrain
          ones (a plain object is worked out again when the set of active
          buildings near it changes). Logs: `characters.log` per
          character once a second (buildings in reach, surfaces, pieces);
          `shadows.log` the receivers at load and the plain objects'
          counts.
      - **4d-7b-4, plain objects cast them** (**done** 2026-10-20; **checked by the user**: the same pub barrel casts as in the original; rule 37 "Plain objects":
        `shadowMask` 8, the same drawing).
      - **4d-7b-5, A's answer to Q135** (**done** 2026-10-21, logs only; `characters.md`
        rule 37 (2) as detailed by A). Replaces 4d-7b-3's stand-ins:
        - **4d-7b-5a, the search box** (**done**; core `shadow.rs`): a caster =
          its own space (position, axes turned with its heading) and its
          shown model's stored ball (radius, centre). s = 0.287 ×
          radius. Reach: building ball centre to the ball's centre in
          the world ≤ building radius + s + 500. Band (own space): s
          above to 6 × s below the ball's centre, cut to 1,800 around its
          middle. Area (own space): the rectangle around a 2 × s square
          at the band's top and its copy moved along the sun to height 0,
          each side cut to 1,800 around its middle. No downward sun: no
          search. A surface counts when any part of it is inside the box
          (whole surfaces, no cut, as the user decided). Kind-0
          surfaces stay skipped (A: "Do instead"). Unit tests with A's
          numbers (Connor standing: band 1,079 above to 687 below his
          feet; running: 885 / 915; the area 505 × 2,139 cut to 1,800).
        - **4d-7b-5c, the user's check** (2026-10-21): shadows too light
          (0.5 now of the shown brightness, sRGB) and late on walls (the
          area's 1,800 cut along the sun is not reproduced, the user's
          decision); the sun's angle is the data's (`tsLightAim`), kept.
          Then the head still went under the wall: the area now runs
          along the sun from the top of the model's ball (the user's
          decision, 4d-7b-5d). **Checked by the user (2026-10-21):
          "looks a lot better".**
        - **4d-7b-5b, the model's ball in the viewer** (**done**; `load.rs`: models
          and movements keep the `.kqs` ball; a `ModelBall` component on
          each shown frame in `anim_view.rs`, `connor_view.rs`,
          `door_view.rs`, and on still objects' parts in
          `terrain_view.rs`; `shadow_view.rs` / `anim_view.rs` build the
          caster from the frame's place). A caster without a ball (an
          item) falls back to a box around its points, logged. Logs: the
          caster's s, ball and box per character once a second.
    - **4d-12, characters are solid for Connor** (planned 2026-10-20;
      `connor-collision.md` rules 2–4, 18, 19; `characters.md` rules 1,
      8, 9, 16, 33; `connor-physics.md` "Collision shape", "Exact hit
      handling"). Connor stops at characters or slides along them; what
      a character does when bumped (M5), characters against each other
      or objects in their own moves (Q128) and walking (M5) are not part
      of it.
      - **4d-12-1, the cylinder test** (**done** 2026-10-20; core `collide.rs`): a new
        `CylinderCollider` (name, position, type, mask, radius, height).
        Each of the mover's balls (its centre's path) is swept against
        an upright cylinder standing on the other's position, widened by
        the ball's radius sideways and at both ends; the earliest hit
        counts. Side hit: level surface direction out from the axis; top
        or bottom: straight up or down. Moves under 0.000001 in every
        axis are not tested. A ball that starts inside: no hit when the
        other's type has bit 2 or 4 (creatures, Connor); otherwise a hit
        at the start, surface direction against the movement. The
        contact is pulled back 25 like every object contact. **Stand-in
        for the size** (rule 18 "not pinned down: likely its ball radius
        and its box height"): radius = the other's ball radius (half the
        larger of its box's x and y sizes), height = its box height,
        standing on its position; asked as a new question. Unit tests:
        side hit and its direction, top and bottom, nearest of several
        balls, start inside a creature (no hit) and a non-creature (hit
        at the start), tiny moves skipped.
      - **4d-12-2, characters in Connor's scene** (**done** 2026-10-20; core
        `connor_mover.rs`, `character.rs`, `active.rs`): the mover's
        scene gets a list of character cylinders tested after the
        objects (nearest contact over all, as now). A character hit is
        never head-on (`connor-physics.md` "Exact hit handling": not
        against a `KQMonster`; every character is one, `characters.md`
        rule 1), so it is always the glancing handling: straight in, the
        slide has nothing left and he stays (still in his walk); at an
        angle he slides along. A character is in the list when: not
        deleted, it has a box, not in Birth (rule 16), its collider is
        on (near, not far, not off or hidden: rule 33), and Connor's
        mask covers its type (rules 2, 4: he is told, so it is a wall
        for him; no character type has the push bit 0x100). The
        cylinder follows the character's position every update.
        `characters.log`: a line when a character becomes solid / not
        solid (and why), with its cylinder. Unit tests: walking straight
        into a character stays outside the cylinder (no `Stop`, no
        block), at an angle slides along, starting inside a creature
        walks out, the list's rules.
      - **4d-12-3, the viewer** (**done** 2026-10-20; **checked by the user
        against the original**: the goblin's stopping distance and the
        slide round it, walking away, the Wizard and his conversation; `draw_distance.rs`: which characters
        have their collider bit now; `door_view.rs` `character_gravity`
        builds the list after the characters' own update; `connor_view.rs`
        puts it in Connor's scene). `characters.log`: every Connor hit on
        a character (who, where, surface direction, slid or stayed).
        Runs: the goblin at the start, the Wizard (and a conversation
        with him: Connor must still reach his spot), the Uglybeast, with
        `--hold` into them.
      - **Not built from the note** (asked): landing on top of a
        character pushes Connor 250 sideways away from it
        (`connor-physics.md`, "traced only this far"); until answered,
        landing on top is an ordinary landing.
  - Not in 4d: noticing Connor, wandering, idling, alarm, chasing,
    dying, the flying mover (M5); conversations (4e, from
    `conversation.md`).
  - Tests: unit tests for each core rule (first update, Wait and
    `Stop`, the return to `Stop`, no state, Special lists, turning, the
    mouth codes, gestures); `kq8 character-verify`; viewer runs near
    the Wizard (`--at` in his house), at the Uglybeast's pond with
    `--click Uglybeast`, at the chickens, at the graveyard, read in
    `characters.log`, `anim.log`, `talk.log`.

- **4e — conversations** (planned 2026-10-19). From
  `docs/behaviour/conversation.md` (A, 2026-10-19, checked by the user in
  the original), with `talking.md`, `characters.md` rules 19, 24, 28,
  `clicking.md`, `camera.md`, `interface.md` rule 16. Words: the
  **partner** is the character Connor talks to; its **info object**
  (`<partner>Info`, class `KQConvInfo`) gives his **talking spot**
  (`convLocX/Y/Z`), the **start area** (`box0`), the **stay area**
  (`box1`), **special areas** (`box2`…) and the **conversation cameras**
  (`cam0`…); the **conversation queue** is the partner's second line
  queue (`<partner>Conv`), next to its `speak` queue; a **talk state**
  is the state Connor and the partner are in while they talk. One step
  and one commit at a time; every step logs to
  `kq8-artifacts/logs/talk.log` and a new `conversation.log` (Connor's
  side: case, placing, stay test, the end, cameras).
  - **Where the code goes.** `kq8-data`: new `conv_info.rs` (the info
    object, the "inside" test); `characters.rs` gets the goodbye and
    welcome-back keys. `kq8-core`: `talk.rs` gets the conversation
    queues; new `conversation.rs` (Connor's talk state: the case, the
    approach, the stay test, the end, what the camera and the panels are
    told); `connor.rs` gets a `Talk` mode (the approach movements, or the
    keys); `character.rs` the partner's Talk state; `active.rs` the
    command, the clicks, `TalkExit`, `InvConv`. Viewer: `load.rs` reads
    the info objects; a new system after `trap_step` runs Connor's talk
    state and applies it to Connor, the camera and the panels.
  - **4e-1, reading** (**done** 2026-10-19; 17 info objects, Barren has three) (rules 3, 20; data). `KQConvInfo` sections: spot,
    boxes (4 corners each), cameras (6 numbers); the inside test (angle
    sum 6.1–6.4 rad; a corner counts as outside); `GBMsg…`/`WBMsg…`
    keys. New check `kq8 conv-verify` (`conv-verify.log`): per world
    every info object with its boxes and cameras, the largest camera
    number of the world's message lines against the cameras, the
    `KQConner::conversation` calls and their `<enemy>` word, the
    characters with goodbye/welcome-back keys; and the note's three
    Daventry points. Numbers to expect: 16 info objects (Wizard 3 boxes
    7 cameras, Uglybeast 2 and 7), 169 calls all with `<enemy>` 0, 19
    characters with the keys; (193000, 198000) in `box0` and `box1`, 834
    from the spot; (191000, 197000) in `box0` and `box2`; (198000,
    198000) in none.
  - **4e-2, the conversation queue** (**done** 2026-10-19) (core `talk.rs`, rules 12, 21, 22,
    23; unit tests only). A second queue per partner, issuer the
    partner; lines play as `speak` lines; each line's camera number
    reported when it starts; `TalkComplete` from the partner; the queue
    reports when its last line ends. Goodbye (put in front, cuts the
    line playing, which goes back to the head of the waiting lines; at
    its end the queue pauses and the caller is told for `TalkExit`; no
    `TalkComplete`); welcome back (a paused queue with lines waiting:
    welcome back in front, the new lines after the waiting ones with the
    duplicate check, no `TalkComplete`); a goodbye asked for and not
    played is dropped; `clearSpeak` empties it.
  - **4e-3, the command and the two talk states** (**done** 2026-10-19; the goodbye already plays on walking away) (rules 1, 2, 6–13,
    15–17). `KQConner::conversation` (usage, not `KQMonster`, `<last>`
    0); the partner's `speak` queue emptied, its current movement made
    cuttable, `Stop` asked for, its Talk state (turns toward Connor every
    4th update unless `noTurn`; no return to `Stop`); Connor's `speak`
    queue emptied and his Talk state: the case from his place ("there"
    under 125 from the spot in the stay area, "close" in the stay area,
    "walk" in the start area; none: logged, treated as "there", Do not
    reproduce), weapon away (no weapons yet: logged, skipped), "close"
    puts him on the spot, "walk" turns him toward it and plays
    `stopwalk`, then puts him 440 short of the spot facing it and plays
    `walkstopright`, then turns him to the partner; the partner told to
    start (its lines queued). The stay test every update; leaving it ends
    the conversation (Do not reproduce: `TalkExit` sent once if no
    goodbye will play). The end: Move if moving, else `Stop` and Wait;
    `TalkExit` before the last line's `TalkComplete`; the partner to
    Wait. While talking: `speak` for either is ignored; keys don't cut
    lines short. A missing info object: logged, treated as "there" with
    no cameras (Do not reproduce). STAND-INS (Q125): during the approach
    movements the keys are ignored; Jump and Push are refused for the
    whole talk state.
  - **4e-4, clicks** (**done** 2026-10-19) (rules 5, 19). A click on a partner: special area →
    `ConnorAction Special Special`; start area → in reach whatever the
    distance (`do do` or the held item); else `TooFar TooFar`; the click
    turn as usual. During the talk state a world click does nothing,
    except one on the partner with an item held: `InvConv <item>`, Connor
    leaves (with `TalkExit`), then the click goes on as above.
  - **4e-5, cameras and panels** (**done** 2026-10-19; the panels came with 4e-3; new test option `--right N`) (rules 6, 14, 15; `interface.md` rule
    16). A line with a camera number cuts the camera to `cam<n>` (checked
    against `nCam`: Do not reproduce) and turns Connor to the partner once
    he has reached his spot; the right button held during the stay test
    puts the camera at `cam0` aimed at the partner, swinging and zooming
    around it, and camera numbers are ignored from then on; at the end
    the camera goes to `cam0` looking at the partner, then follows
    Connor. The bottom bar is put away at the start and comes back at the
    end (built in 4c-2, now called). First person is not built: logged.
    STAND-IN (Q126): the point the camera aims at is the partner's
    position plus 1,400 (as for Connor), not its feet.
  - **4e-6, goodbye and welcome back** (**done** 2026-10-19, run in the viewer; the code came with 4e-2 and 4e-3) (rules 20–22). Walking away with
    more than one line left and a goodbye line: the goodbye, then
    `TalkExit`; clicking again: welcome back, the cut line from its start,
    the rest.
  - **4e-7, Connor's gestures** (**done** 2026-10-19) (rule 18). Codes 8–15 play
    `gesturea`…`gestureh`, 16 `stop`, when he stands still (no weapon
    drawn, not in first person: always true for now).
  - **After the user's check (2026-10-19):** 4e-8, the right-button
    camera turns around the point half-way between the partner and
    Connor (Q126 updated). Then A's answers Q120 (4e-9: a movement whose
    model's loop lasts 0 never ends) and Q119 (4e-10: events sent with
    delay 0 are handled in the same update, before drawing), which fix the
    Uglybeast's flicker and snap.
  - Tests: unit tests for each core rule (inside test, case, placing,
    stay test, the queue's goodbye and welcome back, the click areas);
    `kq8 conv-verify`; viewer runs at the Wizard (`--at 193000,198000,-1.7`
    then `--click Wizard`; from (191000, 197000) and (198000, 198000);
    walking out of `box1` with `--hold`) and at the Uglybeast (`--click
    Uglybeast`, case "there"), read in `conversation.log`, `talk.log`,
    `doors.log`, `camera.log`, `gui.log`.

### M5 — Fight

**You will see:** monsters that move, attack and die; Connor's health and
stats.

- A: monster AI per type, combat rules and numbers, stats, death.
- B: implement.

- **5a — characters moving on their own, up to the first swing**
  (planned 2026-10-20; **done** 2026-10-20, steps 5a-1 to 5a-9, with 5b;
  **checked by the user against the original 2026-10-20**). From `docs/behaviour/character-movement.md` (A,
  2026-10-19, **not yet checked in the original**), with `characters.md`,
  `connor-physics.md`, `connor-collision.md`, `anim-scripts.md`. Words:
  **active** = on the world's short list of characters that "know"
  about Connor (rule 20); **alarmed** = after Connor (rule 23); **home**
  = its home point (rule 13). The fight itself (the swing, being hit,
  dying) is the next study: where the note says "the attack starts", we
  stop (Wait, facing Connor) and log "attack would start (fight study)".
  One step and one commit at a time; everything logs to
  `kq8-artifacts/logs/characters.log` (per character once a second:
  state, active, target, position, heading, distance to Connor, can
  see / what blocked, wander point and its distance, movement playing,
  idle time; and every alarm, give-up, stuck event, reset, touch).
  - **Where the code goes.** `kq8-data/characters.rs`: the missing
    fields. `kq8-core`: new `character_ai.rs` (the rules of the note
    that decide: sight timing, the active list, wandering, idling,
    alarm, chase, going home; no drawing, unit tests), `character.rs`
    (the Move state, the walk's step, steering, the stuck check),
    `connor_mover.rs` (the character's own touch table and slide),
    `active.rs` (the script commands, the world's list). Viewer:
    `door_view.rs` (the character's scene: terrain, buildings, objects,
    other characters, Connor), `load.rs` / `anim_view.rs` (idle lists).
  - **5a-1, reading what the AI needs** (**done** 2026-10-20; the
    numbers below as expected, 0 new problems) (rules 9, 10, 20, 23, 27; data).
    The monster list's `team`, `chaseMode` (`guard`, `none`, `chase a
    b`, `@`) and `attackDist` (first number; `@` = the type's, the
    type list has the same two columns); the character keys
    `wanderAndStop`, `team`, `canAlwaysSeeConner`, `homeRadius`,
    `chaseRadius`, `attackDist` (default 1,250), `doMode`, `startLoc`,
    `Alarm*`; `KQWorld::MaxMonsters` from `world.cs` (Daventry 6). The
    list's values replace the `.kq` keys (rule 23). `kq8
    character-verify` prints them per line. Numbers to expect (data):
    goblin22 alarm 4,000, chase from type goblin2 (3,000 / 6,000);
    sprigwnd2 team `windguys`; bats and chickens alarm 0.
  - **5a-2, walking** (**done** 2026-10-20; goblin22 walks 917, runs
    3,068, slides along Sara's cottage and escapes; Q132 asked) (rules 1–8). A Move state: entering it asks for
    `Stop` at a random frame and speed, then at once for `walk` or `run`
    (rule 3's fallbacks); a walk or run that ends is asked for again.
    **The step comes from the movement**: per update the loop's added-up
    frame steps × the phase moved (spread evenly, as Connor's loops),
    turned by the character's heading, given to its mover (gravity,
    steps 325, slopes as now). A repeating movement's steps then move
    the character, not the drawing (otherwise the model would run ahead
    and jump back at each loop). Steering: the automatic turn (3.5
    rad/s) aimed anew at the point each update, walking on (`noTurn`
    blocks it). The stuck check (rule 7): 1 s after the start, then
    every 0.5 s; moved < 200 (3D) → no steering for 3 s, the first free
    diagonal of four (1,250 out, at its height; blocked by the ground,
    objects, buildings, other characters: mask 0xB) from one further
    round each time, a snap turn; all four blocked: turned round
    (**stand-in**, the note's open question 1); then `run`.
    `KQMonster::setState <name> Move` puts a character in Move (it then
    walks straight: the note's edge case). Test: `--command
    "KQMonster::setState goblin22 Move",30`; speeds as the note's table
    (917 walking).
  - **5a-3, a character's own touches** (**done** 2026-10-20; goblin22
    stops at Connor and runs round him) (rules 6, 33–35, Q128). The
    character's mover sees what its mask covers: buildings, plain
    objects, other characters (their cylinders), Connor (**stand-in**
    shape: the same cylinder rule as Q131's, from his box); every one an
    obstacle in every pass: up stops, down lands (surface up more than
    0.8), across slides (a surface tilted less than 0.7 toward up counts
    as upright, 3 slides then stays); never head-on, no wall blocks, no
    push-off. Traps (0x10) nothing. Nobody is pushed by a character.
    `colliderMask=0`: nothing.
  - **5a-4, sight and the active list** (**done** 2026-10-20; new
    `character_ai.rs`) (rules 20–22). Every 15th
    update (counted per character from its first update) a line from
    the top of its box to 1,250 above Connor, blocked by the drawn
    ground (water surface included), plain objects (type bit 1) and
    buildings (bit 8), not characters or Connor; no distance limit;
    `canAlwaysSeeConner`. In Wait (or Birth, rule 30), seen and not
    active → joins the list if there is room (`MaxMonsters`); a team
    member may take an untimed one's place. Losing active (off, or
    pushed out): forgets its target; if on the way home or Connor more
    than 2 × its alarm radius away (**stand-in**: plain 3D distance, the
    note's open question 2): put on its home point at once (z −1 floor,
    −2 surface), `hide` lists back into Birth; Move → Wait.
  - **5a-5, wandering** (**done** 2026-10-20; chicken1 runs its loop as
    the note predicts) (rules 9–14). Active, `wanders` on, in Wait, not
    alarmed, idle not due → Move along its points (`loop`, `patrol`
    there and back, `random`: own position + 3 random points at 2–3 ×
    alarm radius); reached within 250 (x, y) → next point and turn to
    it; re-aim only when distance² > closest² + 12,500²; the stuck
    escape takes the next point; `wanderAndStop`. `setHome`.
  - **5a-6, idling** (**done** 2026-10-20; goblin22 fidgets and walks
    on) (rules 15–19). The idle list (`idleAnimFile`) loaded
    as a second movement list; only active, in Wait, not alarmed, when
    the idle time has come (0 at first): a random fidget from that list,
    then Wait's `Stop`; the next due 2 + round(random × 10) s after it
    ended. A walking wanderer goes back to Wait for it.
  - **5a-7, alarm, chase, giving up, going home** (**done** 2026-10-20;
    Q133 asked: the leash loops when Connor stays within reach) (rules 23–29). Seen
    within the alarm radius (straight distance) and `goHome` off →
    alarmed after a random 0–1 s, chase timer 5 s, the walk / run coin,
    the alarm spread (active and seeing, or a team member seeing; the
    team made active). Standing: turns to Connor (or the last seen spot
    if more than 500 away), moves after him if out of sight or beyond
    its attack distance and its chase radius > 0. Moving: steers at him;
    within attack distance and seen → Wait facing him (the swing is the
    fight study). Not seen for 45 updates → gives up (team exceptions).
    After the chase timer: beyond 2 × alarm radius (no team) or beyond
    its chase radius from home → gives up (`run`). Giving up: target
    forgotten, on the way home (`hide` → Birth), steers home, within 100
    in x and y → Wait; re-takes the chase as rule 28 says.
    `KQMonster::goHome`, `setAlarmRadius`, `setChaseRadius`,
    `setHomeRadius`.
  - **5a-8, the zombies rising** (**done** 2026-10-20; Q134 asked) (rule 30). In Birth the sight check
    runs; not active, seen and within the alarm radius → active and
    alarmed; `birthDelay` s later Wait, which waits until Connor is
    farther than the two balls added, then plays `birth`, drawn, solid,
    gravity on, turned toward Connor; then the chase. The graveyard trap
    (`zomtrap.cs`) through its script commands.
  - **5a-9, the flying mover corrected** (**done** 2026-10-20; bat2 on
    the lake's surface, 6,630) (rule 32, Q129): the down 150
    all or nothing; the raise from the top of its shape onto the drawn
    ground (the water surface) or a building, at most 300, as an up
    pass. Bats end each update on the water surface.
  - **5b — A's answers Q131–Q134 and the user's checks** (planned
    2026-10-20; **done** 2026-10-20, 5b-1 to 5b-5; `character-movement.md` rules 3, 7, 18, 25, 27, 28, 30,
    33 as corrected, `connor-collision.md` rule 18, `connor-physics.md`
    "landing on a `KQMonster`"; the user's check of the leash and the
    follow). One commit each:
    - **5b-1, the exact cylinder** (**done** 2026-10-20) (Q131 a, b): radius = ball radius;
      bottom = position + the box's lowest z; height = box height with
      more than one ball, else 2 × the radius (goblin 0–1,600); whole
      units; a purely vertical move tests one ball (top up, bottom down).
    - **5b-2, landing on a character's top** (**done** 2026-10-20) (Q131 c): Connor lands, then
      is pushed 250 level, away from its position (over it: toward his
      back), as an across-pass move (3 slides), in any state.
    - **5b-3, the stuck escape** (**done** 2026-10-20) (Q132): on only once `walk` / `run`
      really plays; the world's diagonals NE, NW, SW, SE, ±1,250 in x and
      y, the round starting at NE on each start of moving; level from the
      feet (1 above, as the note recommends); Connor never blocks; all
      blocked → face north.
    - **5b-4, the chase** (**done** 2026-10-20) (Q133, the user's check): a melee character
      within reach stops only while Connor has no forward / back key;
      otherwise it follows him at reach (the attack itself is the fight
      study); `doMode=1`: stop within 0.75 ×, start beyond 1.25 ×; the
      re-take while moving home keeps its movement (no coin) and heads
      home during its reaction delay; only while moving; standing:
      home at a `run` next update, chase radius 0 stays; the standing
      leash also with chase radius 0; home distance 3D; a fidget named
      `attack` / `react` ending sets the chase timer to 5 s.
    - **5b-5, the zombies** (**done** 2026-10-20) (Q134): sinking plays `hide` where it is,
      still drawn, solid and active; when `hide` ends: off the list,
      underground, gravity off, on its home point, no longer on its way
      home; no alarm during `hide` (told during it: underground at home
      at once); rising again only by rule 30 (B's re-take underground
      removed); the chase during `birth` goes into Move at once (its walk
      waits; the stuck check from the walk's start).
  - Tests: unit tests for each rule in core; viewer runs at goblin22
    (Sara's house), the chickens behind the farm, the windmill
    spriggans, the graveyard and the road zombies, the bats, read in
    `characters.log`; the note's "How to check" for the user.
- **5c — the fight** (planned 2026-10-20; **done** 2026-10-20, steps 5c-1
  to 5c-10, checked in the logs only, not yet by eye). From
  `docs/behaviour/fighting.md` (A, 2026-10-20; rules 6 and 31–32 checked
  by the user in the original), with `anim-scripts.md` rule 7 (the
  `interruptable` cue), `items.md` rule 12 (the throw),
  `inventory.md` rules 9 and 13 (readying a weapon, equipment stats).
  Replaces the attack stand-in of 5a / 5b (a character within reach
  only stopped or followed). Words: **attack time** = when its next
  attack is due; **roll** = the random part of the damage; **flinch** =
  the `react` movement; **kind** = `Normal`, `Projectile`, `Fire`,
  `Ice`. One step and one commit at a time. Everything about the fight
  goes to a new `kq8-artifacts/logs/fight.log` (and, as now, to
  `characters.log`): per attack start (who, distance, angle off its
  heading, Connor's back key, attack time before / after, the
  movement); per `Do` (hit or miss and why, *D*, *R*, *P'*, loss, health
  after); per hit taken (flinch or ouch and why not, hit count, chase
  timer, who got the alarm with "run"); per death (kind, die movement,
  drops with names and places, experience sent, corpse shape, the `Die`
  line to the script).
  - **Decisions taken (the user, 2026-10-20):** the swallowed swing
    (rule 11b) is **not** reproduced: while a moving character's
    `Attack` plays, the moving state's stop rule is not applied (it
    stays in Move until the swing ends). Drops are thrown as
    `tossItems`. Connor's input side and the bow's bolt flight are not
    traced: **stand-ins**, logged as such (5c-5, 5c-9).
  - **Where the code goes.** `kq8-data`: the missing columns and keys.
    `kq8-core`: new `fight.rs` (the rules that decide: stats, the attack
    time, the hit test, the rolls, the flinch, dying; no drawing, unit
    tests); `character_ai.rs` (the attack in the standing and moving
    states, replacing the stand-in); `named_anim.rs` (the cue);
    `connor.rs` / `connor_update.rs` (his swing, flinch and death);
    `item.rs` (a thrown item's flight). Viewer: `fight.log`, the click
    that swings, drawing a thrown item and a corpse where they are.
  - **5c-1, reading the fight's numbers** (**done** 2026-10-20; all of
    rule 5's table matches; Q136 asked, not blocking) (rules 1, 2, 4, 5; data).
    Monster list: `xtraHealth`, `xtraDamage`, `xtraProtect`, `xtraExp`,
    `invHold`, `invDrop`, `attackDelay`; type list: `Health`, `Damage`,
    `Protect`, `Experience`, `invHold`, `invDrop`, `attackDelay`; the
    attack distance's extra numbers (`tooClose farEnough`); item list:
    the last column (projectile file); character keys `deadDts`,
    `idedKqs`, `projdKqs`, `ouchSound`, `normalReactPercent`,
    `specialReactAnim`, `specialReactPercent`, `likelyToFlee`,
    `mustFlee` (`Connor.kq`'s `minDamageFactor`, `maxDamageFactor` and
    weapon lists moved to 5c-5, where they are used). A character's five stats
    (rule 2: type + extra, `Damage` fills hand and ranged, held items
    add, a `LongWeapon` makes it ranged). `kq8 character-verify` prints
    them per character. **Test:** the numbers of the note's rule 5 table
    (e.g. `goblin22` 15 / 2 / 2 / 0 / 3, reach 1,500, delay 1;
    `sprgnbow` 130 / −3 / 2 / 15 / 15, ranged).
  - **5c-2, the `interruptable` cue** (**done** 2026-10-20; unit test
    only, it plays from 5c-3 on) (`anim-scripts.md` rule 7; not
    built so far for any player). When a character's phase passes
    `interruptable <n>`, its "can be cut short" becomes n. **Test:** unit
    test; the log shows goblin22's `attack` turning uncuttable at 0.19.
  - **5c-3, characters attack** (**done** 2026-10-20; goblin22 swings at
    once on arrival, then every 2 s at a standing Connor) (rules 6–12 with the Do instead of 11b).
    Attack time 0 at the start (first attack at once), never while an
    attack plays, now + `attackDelay` when it ends (or, cut short, at
    the next state change or hit). Standing: seen, closer than reach
    (ranged 1.25 ×), due, movement can be cut short → `Attack`. Moving:
    seen, closer than reach, due → `Attack` from Move (whatever Connor
    does); not due → 5b-4's stop rule as before; **while `Attack`
    plays the stop rule is skipped** (Do instead). It keeps turning;
    the stuck check is off during the swing; the chase timer 5 s at its
    start and end. The idle list's `attack` stays a fidget (no hit).
    At the `Do` cue only a log line ("hit test: 5c-4"). **Test:**
    goblin22 swings as soon as it is within 1,500 and then every
    ~2 s (1 s swing + 1 s delay) at a standing Connor (How to check 1).
  - **5c-4, the swing lands on Connor** (**done** 2026-10-20; goblin22
    kills a standing Connor in 8 hits; Connor's keys moved here from
    5c-5: `ouchSound`, the damage factors, the weapon lists) (rules 13, 14, 16–19, 29). At
    `Do`: in reach (≤ reach, 1.25 × with his back key held) and in front
    (< 90°) → a hit of its hand damage (`Fire` with a Flame Sword); the
    roll 25–100 %, his protection, at least 1 lost; his health in the
    meter. He flinches (`react`) when *R* > 0.9 *D*, his movement can be
    cut short and his last flinch ended over 2 s ago; else his
    `ouchSound` (at most once a second). Below 0: his dying state
    (`die`, `firedie` / `icedie`), then he lies dead (**stand-in**: game
    over is another study; the tips are logged only). **Test:** How to
    check 2 and 3 in the log (misses "too far" / "behind").
  - **5c-5, Connor hits back** (**done** 2026-10-20; two punches take
    goblin22 15 → 4 → −4; test option `--swing NAME,N[,big]`) (rules 39–41; **stand-in input**, the
    manual's description, open question 1). Clicking the hand-weapon
    slot readies it (`INVITEM_Fists` at the start → `confists.anm` by
    `Connor.kq`'s `ShortAnimList`; **stand-in**: only his `attack`,
    `attackb`, `attack2` come from that list, the rest stays
    `Conner.anm`); clicking it again puts it away (**stand-in**). With
    it ready, a left click in the world (not Ctrl) turns him toward the
    click (the automatic turn) and swings: `Attack` / `AttackB` 50/50,
    Shift `Attack2`; only when his movement can be cut short; nothing
    while a swing plays; the clicked character becomes his target. His
    `Do`: the same test with reach 2,350; damage = his hand damage
    (Shift: × 0.75–4, cut); the roll 75–125 %, its protection, its
    health (the rest of the hit is 5c-6). A viewer option to swing at a
    set update for tests. **Test:** two punches kill `goblin22` in the
    log (rule 19's example; it dies in 5c-7).
  - **5c-6, characters being hit** (**done** 2026-10-20; goblin21 under
    Connor's fists) (rules 20–28). In order: the kind;
    active; chase timer 5 s; the hit count; must flee; the damage; the
    attack time put right; the flinch or the ouch sound (rules 23, 24);
    it learns where Connor is; the alarm with "run" (hit count 2, `run`,
    10 s). Hit while not alarmed: rule 28 (with the Do instead: the
    point 0.52 rad off the line to Connor). **Test:** How to check 4
    and 5 (flinch ~3 in 5, never twice in 2 s; runs after one hit).
  - **5c-7, dying** (**done** 2026-10-20; goblin22 dies on the second
    punch, its corpse stays) (rules 30, 32, 33). Health 0; the die movement by
    kind; off the active list, not solid, gravity off, no state change
    ever again; when it ends: experience to Connor (his levels), a
    corpse (`deadDts` / `projdKqs` / `idedKqs`, none for fire) where it
    stood, turned the same way, same skin, no shadow, not solid; `Die
    <kind>` to its script; removed. `setState <name> Die now`. The
    first-kill tip is logged only. **Test:** How to check 6 (body
    stays, experience only when the death ends).
  - **5c-8, drops thrown** (**done** 2026-10-20; goblin22's mushrooms
    land 1.5 s later, 1,000–1,600 away; Q137 asked, not blocking) (rule 31, `items.md` rule 12). At the start
    of the dying: `invDrop` (list then type), each a new item at its
    feet, thrown: random heading, 75°–85° up, 5,000–6,500 units/s,
    gravity 8,085 units/s² (**stand-in** for the untraced time step:
    plain per-update motion), lands on the floor-grid height with its
    land sound; clickable in the air. **Test:** goblin22's two
    mushrooms fly up and land, positions in `items.log`.
  - **5c-9, the bow spriggans** (**done** 2026-10-20; `sprigwnd1` shoots
    every 1.7 s; keeping distance, rule 37, **not built**: no Daventry
    character can flee) (rules 34–38; **stand-in bolt**, open
    question 2). Ranged = holds a `LongWeapon`; they shoot from 1.25 ×
    reach standing. At `Do` a bolt (`Prscbolt.kq`'s shape) leaves from
    its position + the default offset, aimed where Connor will be
    (his velocity, bolt speed 8,000) at three quarters of his height
    (**not sure**, the note); **stand-in**: it flies straight, no
    gravity, until it hits a character, Connor, the ground or a
    building, or has gone `maxDistance` 18,000; it carries the
    shooter's ranged damage, kind `Projectile`. Keeping distance (rule
    37) as written (no Daventry character uses it). **Test:** How to
    check 7 (`sprigwnd1` shoots about every 1.7 s within 7,500).
  - **5c-10, the combat level** (**done** 2026-10-20) (rule 3). `applyCombatLevel`,
    `setCombatLevel 1|2|3` (0.66 / 1.0 / 1.5): health, hand and ranged
    damage of each character entering the world × the factor, cut; the
    roll on characters 85–125 % below 1.0. Daventry stays at 1.0. The
    menu that sets it is not traced: **stand-in** launch option
    `--combat-level 1|2|3`. **Test:** `--combat-level 3`: goblin22 health
    22 in the log.
  - Not in 5c: Connor's full weapon lists, ranged weapons for Connor,
    Delete (switch weapons), game over, the tips' texts, smashing
    barrels (`tossItems` from scripts; the throw of 5c-8 makes it easy
    later).
  - **After 5c: the user's fight check, fixes 1–7** (2026-10-20, one
    commit each, see MODLOG): the ready stance and sword pointer, no
    readying in a jump / landing / stumble, the weapon away before a
    conversation, attack targets (creatures, barrels), drops drawn where
    they land, the tumble.

#### 5d — Objects loaded while playing (planned 2026-10-20)

Why: a smashed barrel's script (`barelup.cs` and the two others) deletes
the barrel, then `loadKQ barelupx.kq @ $KQObject::loc` (the bursting
barrel, a `KQAnimOnce`), then (two of them) `KQObject::tossItems
$ObjectName 0 Crystal,…`; at the burst's end its script loads
`barelded.kq` (the wreck) the same way. The alchemist's boards work the
same. Today: a name or position on a runtime `loadKQ` is refused; only
one copy per file, prepared hidden at the start, can come in; nothing
built at start-up is kept, so no new copy can be made; `KQAnimOnce`
never reports its end; `tossItems` from a script is not built.

Sources: `objects.md` "Where a world's objects come from" rules 2 and 6
(the name rule: `same`, `@` = no name, a new name; x y z and angles
replace the file's; `Init` at once; the near/far test at once),
`anim-scripts.md` rules 11–12 (`KQAnimOnce`: at its end `Die` to its
script, `getEndLoop`, `setDeleteWhenDone`; `@` gives `NoName<n>` at the
line itself; `$ObjectName`), `items.md` rules 9 and 12 (`NoName`
counter, `tossItems`).

How: **templates**. At load, every file a world script names on a
`loadKQ` line is read once and its model or movements are built (as for
corpse shapes today), whatever its own position; the built meshes and
materials are **kept in resources** after start-up. A runtime `loadKQ`
in core makes a new object from the template (core side: name, place,
script, collision box, stats) and asks the viewer to spawn its entities
from the kept handles. The existing "dormant copy" path stays for the
files that have a place of their own, until a step replaces it (not
in 5d).

Steps (one commit each; each logged in `viewer.log` / the script logs):

1. **5d-1, `KQAnimOnce` ends** (**done** 2026-10-20; unit test; nothing
   in Daventry is one at the start) (core, and the loader's flag): a play-once object's end
   sends `Die` to its script (and `getEndLoop` 300, `setDeleteWhenDone`),
   for objects in `world.anims` too; `once` set by the loader from
   `classType`. **Test:** unit test; viewer: none yet in Daventry at the
   start.
2. **5d-2, the name and place rules in core** (**done** 2026-10-20;
   unit test; the object is made in core at the line itself, so the
   next line (`tossItems $ObjectName`) finds it): `loadKQ <file> <name>
   [x y z [rx ry rz]]` while playing: `same` / `@` / a new name, `@`
   gives `NoName<n>` (the items' counter) at the line, `$ObjectName`
   set, position and angles replace the file's (0 0 0 if no angles),
   the −1 / −2 ground rule; a live name refused as today. Asks the
   viewer for a copy of the template. **Test:** unit tests.
3. **5d-3, templates for still models** (**done** 2026-10-20; the
   wreck copied by `--command`; new module `copies_view.rs`) (viewer): the shapes of the
   `loadKQ` files kept (as `CorpseDraw`); a copy spawned where core
   says, with its draw-distance test and, if `BOUNDS.KQ` has a box and
   its type collides, its collider and click entry. **Test:** viewer:
   `--command "loadKQ barelded.kq @ <x y z 0 0 0>",N`: the wreck in
   `viewer.log` at that place.
4. **5d-4, templates for animated objects** (**done** 2026-10-20;
   barrel.07: burst, then wreck) (viewer): the movements
   kept (anim_view's built frames in a resource); a copy spawned and
   played by core from its first update; its end (5d-1) reaches its
   script. **Test:** viewer: punch `barrel.07`: `barelupx` bursts,
   then `barelded` stays (`viewer.log`, script logs).
5. **5d-5, `KQObject::tossItems <object> <offsetZ> <types>`** (**done**
   2026-10-20; barrel.12's four crystals; the plan's test option became
   `--swing barrel.12,N`, the camera there sits behind the pub) from a
   script (`items.md` rule 12): one item per type thrown from the
   object's place raised by `offsetZ`, with 5c-8's throw and fix 7's
   tumble. **Test:** viewer: punch `barrel.12`: four crystals fly and
   land; they can be picked up.

Not in 5d: replacing the dormant copies, saving `NoName` (M6), a
template's sound objects and emitters (none in the barrels' files).

#### 5e — Thrown items as traced (planned 2026-10-20)

Why: 5c-8's flight and fix 7's tumble were stand-ins. Items fly through
walls and roofs and can land under the pub's floor (the user's check).
Source: `thrown-items.md` (Q137, answered by A 2026-10-20).

How: each thrown item gets its own **mover**, the same one the
characters use (the code that moves a shape and stops it at the floor
grid, buildings and objects). It has the item's `BOUNDS.KQ` box, type
0x220, mask 0x809, steps 0 and gravity off. Each update the thrower works
out where the arc says the item should be and asks the mover to go
there. The flights move with the characters' movers (the viewer's
`character_gravity`), so they can see the buildings and objects.

Steps (one commit each; logged in `items.log`):

1. **5e-1, the arc and what stops it** (**done** 2026-10-20; barrel.12:
   two crystals hit the pub and drop at its wall) (rules 4–8, 11, 12.1–12.2, 13–16,
   the "anything else" case of 12.3). The exact arc from the launch
   point (8,085 units/s², 15 updates/s, the 25-unit first step); the move
   through the mover against the floor grid, the buildings, and the
   objects with type bit 0x1 or 0x100; at the first touch the land sound,
   and the arc ends (every touch ends it in this step, barrels too); then
   ordinary gravity until it is held for one update (or 10 s); the
   thrower's own floor-grid check; the raise onto the floor grid.
   Tumble and resting pose as before. **Test:** viewer, `--swing
   barrel.12`: crystals that hit the pub drop at its wall; every flight
   logs its touches and its end.
2. **5e-2, thrown again** (**done** 2026-10-20; crystals landing on
   barrel.13 fly up again) (rule 12.3): a touch on an object with a script
   or an animated object throws it again (fresh launch, clock 0).
   **Test:** an item hitting a barrel flies up again (`items.log`).
3. **5e-3, the tumble** (**done** 2026-10-20; barrel.12's crystals turn
   0.15–0.44 rad per update) (rules 9–10): one axis (its own x), −0.5 to +0.5
   rad per update, kept for the flight; turn 0, 0, 0 at the end.
   **Test:** by eye against the original (`thrown-items.md` open
   question 2).

#### 5f — Ranged weapons (planned 2026-10-20; done 2026-10-20, steps 5f-1 to 5f-7, checked in the logs only)

Why: Connor can't use the Small Crossbow `sprgnbow` drops, and the bow
spriggans' bolt is a stand-in (5c-9). Source: `ranged-weapons.md` (A,
2026-10-20, checked by the user in the original), with `fighting.md`,
`connor-collision.md` rules 18–19, `inventory.md` rule 9. Replaces the
stand-ins of 5c-9 (the bolt), 5c fixes 4–5 (the attack click's target,
Q139) and 5d fixes 9–11 (readying and putting away, Q138) where the note
differs; where it confirms what is built, provenance says so. Not in
5f: rocks and the Azriel Hammer, how objects such as barrels take damage
(Q139 e: the stand-in of 5c fix 5 stays, a bolt uses it too), the
note's open questions 1–5 (a swap during a swing, the right-button
click point, `collisionDetail`, the pointer names, rocks).

Words: **bolt** = any arrow or crossbow bolt; **weapon mode** = hand,
ranged or none (what is ready); **swap** = the movements played when
the mode changes; **aim point** = where Connor's bolt is aimed.

Everything logs to `fight.log` (the bolts: start offset, start point,
aim point, heading and pitch, damage; every update its position and
distance; its end: what it touched, damage sent, or floor grid / range;
Connor's requests: key or slot, state, taken or dropped, the swap's
movements, the list, the pointer; the shot click: the picks, the aim
point before and after the steepness limit, the angle off his facing,
waiting for the turn, standing or moving, the delay test).

Steps (one commit each):

1. **5f-1, the bolt's flight** (**done** 2026-10-20; `sprigwnd1`'s
   bolts hit Connor after 5 updates, the windmill's wall when he runs;
   Q140 asked, not blocking) (rules 25–31; the spriggans already
   shoot). The projectile file's keys with their defaults (`velocity`
   2,000, `maxDistance` 8,000, `colliderMask` 0x1F, `weaponType`, type
   1). Aim at the target + 0.75 × the top of its collision box, led as
   now. The bolt turned to the aim point once (heading, pitch) and moved
   speed / 15 per update, no gravity. Its shape: its `BOUNDS.KQ` box as a
   ball stack (the movers' rule, `connor-collision.md` rule 18; Q140 a
   asks A to confirm). Its sweep meets the buildings (mask bit 0x8),
   objects and characters whose type shares a bit with its mask, and
   Connor (type 4); never its shooter; a start inside a cylinder is a
   hit. After the move: at or below the floor grid → removed; more than
   `maxDistance` travelled → removed. A hit on a character: type bit
   0x4000 → 0 damage, else its damage, kind `Projectile` / `Fire` / `Ice`
   (`weaponType`); on Connor `fighting.md` rule 29; on an object the
   barrel stand-in. Removed at the touch point; no stick, no sound. The
   start stays the 5c-9 stand-in (3/4 of the shooter's height) until
   5f-2. Drawn turned by heading and pitch, its model's front +y.
   **Test:** `sprigwnd1` shoots about every 1.7 s; `fight.log` shows each
   bolt's heading, pitch, 533 units per update, its end.
2. **5f-2, where the bolt starts** (**done** 2026-10-20; `sprigwnd1`:
   (−55, −1,367, 954) from `spbattck` frame 2, the frame at the cue's
   phase, Q140 c) (rule 24). The loader records, per
   frame of each movement, the place of each named model part. At a
   character's first shot the part `arrow` (the bolt file's
   `meshName`) in the frame playing gives its offset (kept); fallback
   its `shootOffsets` (none in Daventry: 0, 0, 0). Start = position +
   its turn × offset. **Test:** `sprigwnd1`'s offset ≈ (−55, −1,367,
   954) in `fight.log`, the start point in front of it at that height.
3. **5f-3, weapon modes and the swap** (**done** 2026-10-20; Delete,
   Alt, slot clicks and `setDoMode` logged as the note says; a request
   during a swing waits for its end, open question 1, ours; the pointer
   by mode, rule 11, moved here from 5f-4) (rules 1–9, 11, 12; Q138). Core keeps
   the mode (hand / ranged / none) and the last one put away. Requests:
   a slot click (`SwordMode` hand, `BowMode` ranged, `HandMode` none,
   `NoModeChange` nothing), Delete (`Draw`), Alt (`Away`),
   `KQConner::setDoMode 0|1|4`; ignored while `handsOff`; same mode →
   none; ranged needs a real ranged item (the Do instead, else hand).
   Taken only while he stands or moves (our Free mode), otherwise
   dropped. The swap as rule 6: Fists from / to none `draw` / `undraw`;
   Shift held `away2`; otherwise `away` in the current list, the switch,
   `out` from the new list; then standing. The switch asks the playing
   movement again by name (rule 8). Ctrl's plain click with any weapon
   ready (rule 12). **Test:** `--key delete,N`, `--key alt,N`:
   `fight.log` shows each request, taken or dropped, the movements.
4. **5f-4, the crossbow readied** (**done** 2026-10-20; slot, Delete,
   Alt with the Small Crossbow as the note's "How to check" 1–2)
   (rules 10, 11, 21). A `LongWeapon`
   picked up becomes the ranged slot item: its projectile file and
   shoot offset (`Connor.kq`'s `shootOffsets`, any capitals). The
   ranged mode's list from `LongAnimList` (`conscbow.anm`), the bow
   pointer `IDBMA_CURS05`. Picking up a new one while ranged is ready
   refreshes the list at once. **Test:** take the crossbow from
   `sprgnbow`'s drop, ready it by its slot and with Delete; the log shows
   `away` then `out` from `conscbow.anm` and the pointer.
5. **5f-5, the standing shot** (**done** 2026-10-20; goblin22 hit for 5
   from (−231, −832, 1,067); the test option became `--aim NAME,N` and
   the new `--aim-point X,Y,Z,N`) (rules 13–18, 20, 22–25). The click with
   the ranged mode ready: an item on the pointer wins; Ctrl → plain
   click; the talk pick (types 0x40 / 0x20 / 0x08, a character with bit
   0x8000 → plain click); the aim pick (types 0x40 / 0x08, else the
   drawn ground, else the line's end; a character → its x, y); the
   steepness limit (1 rad from the vertical). He turns to it (third
   person); more than 45° off → the shot waits until 3 updates of turn
   are left. `attack` only when his movement can be cut short; the bolt
   at its `do` cue from the `arrow` part of the frame playing (fallback
   the shoot offset), aimed at the aim point, his ranged damage. A test
   option `--shoot NAME|x,y,z,N`. **Test:** shoot `goblin22`: the bolt
   leaves ~832 ahead, 1,065 up, flies to it, damage 5 at level 1.
6. **5f-6, the moving shot** (**done** 2026-10-20; a running Connor's
   shots at updates 63 and 72, the one at 70 refused; run back at 87)
   (rule 20, "Moving"). In Move: no `attack`;
   if 0.5 s have passed since the last shot the bolt leaves at once from
   the frame playing, the item's use sound plays; a run becomes a walk,
   back to running 1 s after the last moving shot. The end of `attack`
   also sets the next moving shot 0.5 s later. **Test:** walk and shoot
   with `--key` and `--shoot`: at most two shots a second.
7. **5f-7, the swing click's target** (**done** 2026-10-20; a ground
   click after punching goblin22 still swings at it; barrel.12 still
   bursts) (rule 32; Q139). The click line
   meets types 0x40 / 0x20 (characters, plain objects, barrels,
   clickables, clickable buildings); nothing or a building → turn to the
   click point and swing, target kept; a character with bit 0x8000 →
   plain click; anything else → its target if within 2,350 (3D), turn
   to its position, more than 45° off → the swing waits until 8 updates
   of turn are left. Replaces 5c fixes 4–5's box test and "behind 90°".
   **Test:** `--swing goblin22` then a ground click: the second swing
   still has goblin22 as target.

#### 5g — Health items and potions (planned 2026-10-21; done 2026-10-21, steps 5g-1 to 5g-8, checked in the logs only)

Why: the eight bottom-bar slots, their counts and keys 1–8 are built
(M4 4c-3), but potions only add their (all-zero) numbers and wait out
their time: no shield, strength, invisibility or clarity, no timer
icons, no red numbers. Source: `potions.md` (A, 2026-10-20; "How to
check" 2–6 done by the user in the original), with `inventory.md`
rules 8–18 and `interface.md` rules 20–33. Not in 5g: the help pop-ups
(rule 41's 0x80, 0x83, 0x84: not studied), the Temple 4 armour and
helmet skins and faces of armour in general (only what a potion
changes), the rope (M6), saved games and world changes (we have
neither yet: the potions are kept in a form that a save can store
later).

Words: **potion** = one of the four timed items (Clarity, Invisible,
Invulnerable = shield, Strength); **health item** = mushroom, crystal,
sacred water, elixir; **dose** = one use from a stack; **timer icon** =
the potion's picture in the status icons (`IDCTL_TIMER`, bottom right);
**reported position** = where the characters are told Connor is.

Game time: our updates (15 a second). They keep running when the
window loses focus (Bevy keeps updating an unfocused window) and stop
only when the game stops; there is no main menu yet, so "only the main
menu stops it" is true by construction (the menu, when it comes, must
stop the updates).

Everything logs to `items.log` (and `gui.log` for the icons and red
numbers, `characters.log` for Invisible, `doors.log` for Reveal):
per use the slot, item, count before/after, refused (why), sound; per
potion start/end the game time, protection and hand before/after;
every tick the seconds left, icon place and percent.

Steps (one commit each):

1. **5g-1, using an item** (**done** 2026-10-21) (rules 1–13). Mostly built in 4c-3; adds:
   refused while Connor is dying or dead (rule 2; until now only
   handsOff); the log line per use gets the slot, count before/after,
   "refused (why)" and the sound; a running potion clicked again (the
   stack's object is already applied) → nothing, logged; the last dose
   leaves the item at count 0 until its time is up, its slot drawn
   empty and not clickable (already so; tested). **Test:** unit tests;
   `cargo play --world Daventry --key 1,30 ...` with `give`.
2. **5g-2, the timer** (**done** 2026-10-21; Strength from update 30 to 330) (rules 33–34). Each running potion counts whole
   seconds: `Dur` at the start, −1 every 1 s of game time, ends when it
   would drop below 1 (exactly `Dur` seconds). Its state (item, seconds
   left, next tick, the remembered raises) is one record Connor keeps,
   so a save can store it later. **Test:** unit test; `items.log` shows
   20, 19, ... 1 then the end 20 s (300 updates) after the start.
3. **5g-3, the timer icons and the red numbers** (**done** 2026-10-21; Strength in `_1`, the shield in `_2`, the mask in the next free place; Q143 asked, not blocking) (rules 35, 37–39). A
   potion's picture goes to the first free place of `IDCTL_TIMER_1`–`_7`
   (`_1` is the right-most in the layout) and is freed at the end. It
   is drawn in two parts split at the height of the percent left
   (seconds left × 100 / `Dur`): the lower part from the picture's
   full frame, the upper part from its empty frame (each `.pba` has
   two: frame 0 a black square with the gold outline, frame 1 the
   picture; matches the user's description); redrawn only when the
   percent changes. While Strength runs the hand number is written in
   `14red.pft` (`IDCTL_INVBAR_SHORT_ALTTXT`) instead of the white one;
   the shield the armour number (`..._ARMOR_ALTTXT`). The spinning mask
   (handsOff) takes a place the same way and animates at 10 frames a
   second (rule 35, last point; was 15, Q108 d). **Test:** `gui.log`:
   place, percent 100 → 95 → ... → gone; the red flag on/off.
4. **5g-4, the shield** (**done** 2026-10-21; protection 2 → 50, gloves meanwhile → 51, end → 3) (rules 17–20, Do not reproduce 1). Protection
   → p + ⌊p/2⌋, at least 50, at most 100; the amount added is
   remembered and taken off at the end (not the old value restored).
   `setConnorFlag InTemple4 1`: each hit uses old + ⌊(raised − old)/2⌋,
   at most 85, the shown number unchanged. The orbit: `orbit1.kq`
   (`invulnfx.kqs`, see-through 165/256, no shadow), centre his `body`
   part − 820 + 500, radius 500, turned 0.2 rad every update, removed
   at the end. **Test:** `fight.log`/`items.log`: protection 0 → 50 →
   0; the orbit's place logged every update.
5. **5g-5, strength** (**done** 2026-10-21; hand 10 → 15 → 10, `coniron.bmp` on 448 materials, faces `cironf`, `cclif`) (rules 21–25). Hand number + ⌊hand/2⌋ (ranged
   unchanged), the half remembered and taken off at the end. Body skin:
   `ConnerSkinList` bit 2048 (`coniron.bmp`; first person
   `armconfx.bmp`, not drawn: no first person yet). Face: Connor's face
   pictures come from `ConnerLipsList` by bits (helmet 1, temple
   armour 2, Clarity 4, Strength 8; a missing combination → the entry
   with most of the wanted bits, Do not reproduce 3). **Test:** hand 10
   → 15 → 10; `connor.log` names the skin and face set.
6. **5g-6, invisible** (**done** 2026-10-21; a chasing goblin gives up 3 s after, goes home; 445 materials with `I_` pictures, 585 meshed) (rules 26–29). The reported position: characters
   are told Connor is 100,000 below where he is while it runs (sight,
   alarm, chase and leash use it). `KQConner::checkInvisible`. His
   textures: an `I_` version if the world has one (first two letters
   replaced), else a copy with 3 of 4 pixels see-through (one kept
   every 4 along a row, shifted 2 on each next row); redone when a skin
   or face changes; restored at the end. The hurt line (rule 27) is not
   built (which characters use it is not traced). **Test:** a goblin
   chasing him gives up within ~3 s (`characters.log`).
   Rule 26's other cases (cutscenes, talk, rope): step 5g-9.
7. **5g-7, clarity** (**done** 2026-10-21; keepwall dithered 30–930, `wizchest1` shown, drawn and opened; `dither`: stand-in, Q144) (rules 30–32). At the start every object whose
   `groupsAlways` has bit 2048 gets its script with `Reveal True`, at
   the end `Reveal False` (Daventry: `keepwall`, `wizchest1`). Face bit
   4 (`cclf`). **Test:** `doors.log`: keepwall `dither 1` / `0`,
   wizchest1 `setHidden 0` / back.
8. **5g-8, experience at pickup** (**done** 2026-10-21; candle +25 five updates after, feather +1,000 at once) (rules 40–41). The candle +25 0.3 s
   after it is added; ring of the dead hero, ashes +25; lady bell +20;
   rope and hook +10 (0.3 s); feather +1,000, green mushroom piece
   +5,000, mold +6,000, hearing horn +10,000, deciphering amulet
   +25,000, ladle +50,000 at once; every time added. The help pop-ups
   are logged only. **Test:** `give Candle`: experience +25 at the
   fifth update after.
9. **5g-9, characters ignore Connor during cutscenes** (planned
   2026-10-21, the user's go-ahead; **done** 2026-10-21: goblin22 gives
   up 3 s into a cutscene, sees him again after it) (rule 26: the same lowering by
   100,000 "during cutscenes (handsOff with the pointer hidden)"). The
   viewer tells the rules each update whether handsOff is on (a script's
   `handsOff 1` or a `walkTo`); while it is, characters are told Connor
   is 100,000 below, as with Invisible. Every change logged in
   `characters.log`. Not in this step: "while Connor talks" (a remark,
   or only his conversation state? asked, Q146) and the rope (not built,
   M6). `checkInvisible` stays Invisible only. **Test:** a goblin chasing
   Connor, then `handsOff 1`: it gives up within ~3 s; `handsOff 0`: it
   can see him again.

### M6 — The whole game

Save/load, moving between worlds, all 12 worlds, cutscene videos, music,
options. A and B as needed per topic.

Already known for later worlds (kept out of M3): Gnome's rolling rocks
knocking Connor back and hurting him (`connor-collision.md`, Q55;
damage needs M5), Gnome's pushable `boulder` (`KQBoulder`, not
studied), the other pushable classes (`pushing.md` open question 1),
damaging water (poison, lava, swamp).

#### 6a — The Rope and Hook (planned 2026-10-21; done 2026-10-21, steps 6a-1 to 6a-6, checked in the logs only)

Why: the rope places' trap scripts only log `setRope` (stubs since M3
3d-3), and clicking the rope gives a remark and holds it anyway (Q110
stand-in). Source: `ropeandhook.md` (A, 2026-10-21; checked by the user
in the original, "all matches"); the user's checks there win over what
the note says the code suggests. Not in 6a: saved games (we have none
yet: the rope's settings and the climb are kept as plain records a save
can store later, noted in STATUS); first person (not built: every
"switch to third person / back" is only logged, the first-person lock
is the one `setThirdPerson` already keeps); rock throws (not built, so
"ignored on the rope" is true by construction, noted); open question 2
(the camera during the throw from below): the note's stand-in, the
camera stays where it is until Connor grabs the rope.

Words: **rope place** = a pair of rope trap boxes (foot and top);
**rope status** = none / at the foot (`Bottom`) / at the top (`Top`);
**hook icon** = `HookUp` / `HookDown` in the status icons; **rope state**
= Connor's special state from `away` to `out`; **hanging** = the part
of it where the keys climb; **aim point** = where the camera looks
during the climb.

Where things go:

- `kq8-core/src/rope.rs` (new): the status and the place's settings
  (one record of Connor's), the six commands, the hook icon wanted, and
  the rope state's own rules as plain functions: the camera spot chosen
  (rules 18, 20), the camera height range (rules 19, 21), one climb
  update of the camera (rule 22), what each animation's end leads to
  (rules 13–26). Unit tests for each.
- Connor's walker gets a **Rope** mode (like Special: keys don't choose
  the movement, no turning); while hanging, forward plays `climb`
  forwards, back backwards, nothing else moves it (rule 22, the user's
  check). Fall, landing, push and wall events don't change the mode.
- Viewer `rope_view.rs` (new): runs the sequence on Connor (gravity,
  step heights, placing, the 5,000 lift, the drop), the camera spots,
  and the separate rope model (`conrope.anm/rope`, drawn at Connor's
  place while it plays, its two sounds).
- Everything logs to a new `kq8-artifacts/logs/rope.log` (and
  `items.log` / `gui.log` / `camera.log` / `connor.log` as usual).

Stand-ins (asked as **Q147**, not blocking): (a) the hook icon takes the
first free status place, like the potions and the spinning mask (Q143);
the layout puts `hookup.pba` in `IDCTL_TIMER_5`, `hookdown.pba` in `_6`;
(b) the rope state's "handsOff with the pointer kept" does what a
script's `handsOff 1` does (bottom bar away, spinning mask, keys and
world clicks ignored) except the pointer stays; (c) taking the rope at
the top moves the camera to the `setRopeCam2` spot as a `move KQCamera`
would while it follows (it stays there until its next update; the
"flag that keeps it there" is not known), so putting the rope back
leaves it where it is (the user's check); (d) the rope animations'
own steps: `climb` never moves him (rule 23); the others move him by
their steps as any one-shot does (only `taway` has one, 405 forward);
(e) each placing by the rope state counts for the traps as a script's
`move` (stay test), then the enter test at the new place, so the foot
trap's `Off` comes before the top's `On`; (f) the camera after `btoss`
(open question 2): from the user's check (close at eye level,
looking up): 2,000 behind him (250 was too close), 1,400 up, tilted 1.27 rad, looking the
way he faces.

Q147 answered by A (2026-10-21) and built, one commit each: (a), (d)
as built; (b) the rope's handsOff is a plain `handsOff 1`; (c) the top
camera skips his first move, the lock goes then; (e) a placing runs
only the stay test; (f) the rope model 2,000 behind him, the throw
camera at the rope's place + (0, 2,250, 350), tilt 1.27, heading + 3.14;
the grab's tie takes the + spot. `--rope-cam` removed.

Fix after the user's look (2026-10-21): every movement the rope state
starts after his own update is drawn from its own start at once (it
was drawn at the old one's end for one update: the snaps).

Steps (one commit each):

1. **6a-1, the commands, the rope status and the hook icons** (**done**
   2026-10-21; `ropetrp1` on at update 31 with the rope, refused
   without it) (rules
   1–7). `KQConner::setRope None|Top|Bottom [x y z [rx ry rz]]` (any
   case; 1, 4 or 7 words; else the usage line), accepted only if Connor
   carries an item of category `Rope` (rule 4; otherwise nothing changes,
   both icons hidden); place and direction stored only when accepted;
   the two `setRopeCam` spots cleared every time. `setRopeCam` (a spot
   counts only with no 0 coordinate), `setRopeCam2`, `setRopeCamAngle`
   (kept, unused: "Do not reproduce"), `setRopeCamTopLimit` /
   `BottomLimit` (0 = none); `setRope None` clears both limits and puts
   a held rope back (rule 6). The hook icon: `HookUp` for Bottom,
   `HookDown` for Top, none for None or while on the rope (rule 7),
   drawn from `INVITEM_HookUp` / `HookDown`'s picture. **Test:** `give
   RopeAndHook`, walk into `ropetrp1`: `rope.log` "status none ->
   bottom", the icon place; walk out: "-> none", gone. Without the rope:
   "refused: no rope carried".
2. **6a-2, the rope button** (**done** 2026-10-21) (rule 8, 9). In the rope state: ignored.
   Rope already held: put back (first-person lock released). None: the
   remark `0.MSG` 0/0/58/37 only, **not held** (replaces the Q110
   stand-in). Bottom: held (`IDBMA_CURS06`). Top: third person forced
   (logged), the first-person key locked, the camera to the
   `setRopeCam2` spot looking at 1,000 above Connor, then held. Put back
   at the top: the camera stays. **Test:** `--key left:IDCTL_INVBAR_ROPE,N`
   at each status; `items.log`, `camera.log`.
3. **6a-3, the click and the sequence to hanging** (**done** 2026-10-21;
   foot: hanging at update 193, top: at 210; the traps at a placing:
   stand-in, Q147 e) (rules 10–21, 31).
   With the rope held, any world click (whatever it meets, the sky too)
   empties the hand and asks for the rope state (no angle test: "Do
   not reproduce"); refused in handsOff, while dead or dying. Entering:
   the camera stops following, he turns to the rope's place (the
   `walkTo` turn), handsOff on (pointer kept), the hand mode
   remembered, `away` from the list in use. Then empty hands; the foot:
   `draw`, gravity off, `btoss`, step heights saved and 0, placed at the
   rope's place and turn, the rope model, `Stop`; the model's end: the
   aim point, the camera spot (nearer of the two, rule 18), the 5,000
   lift, `bon`. The top: `drawdown`, placed, the camera spot (rule 20),
   gravity off, `ttoss`, `ton`. Both end hanging, frozen, with handsOff
   off: at the foot the camera's height range up (rule 19), the climb's
   first frame; at the top the range down, the half turn, the last
   frame (rule 21). **Test:** `--aim-point` (a sky click) at the foot
   and at the top; `rope.log`: each animation's start and end, the spot
   and why, his place at the grab, the range.
4. **6a-4, climbing** (**done** 2026-10-21; test option `--hold
   F,T,R,FROM,N`, repeatable) (rules 22–23). Forward: `climb` forwards, back:
   backwards, any other key or none: frozen. The camera's height and the
   aim point move 750 per second (50 per update) with the movement,
   the height kept in its range; x, y kept, re-aimed every update. The
   climb ends at whichever end it reaches in the direction it plays.
   **Test:** `--hold 1,0,0,N` / `--hold -1,...`; `rope.log` per update:
   key, direction, frame, camera height.
5. **6a-5, getting off** (**done** 2026-10-21) (rules 23–26). Top end: icons may show,
   turning on, handsOff on, `toff`, the camera follows again, `taway`.
   Foot end: moved 1,000 toward his back and 4,900 down, gravity on,
   `boff`, `baway`, the camera follows. Then gravity on, step heights back
   (only if saved and not 0), the weapon: `out` from its list only if he
   held one (the user's check), else none; then standing, handsOff off,
   the first-person key free. **Test:** a full climb up and a full climb
   down; his place at the end, the mode back to Wait, the weapon.
6. **6a-6, while on the rope** (**done** 2026-10-21; a chaser losing
   him not run: the report line and far bats only) (rules 27–30). Hits ignored (no damage,
   sound or flinch: `fight.log`); every state request refused and
   dropped (special, walks, dying); characters told he is 100,000
   below (`potions.md` rule 26's code, a third reason "the rope"); the
   weapon keys and slots do nothing, no remark; the first-person key
   locked; the rope button ignored; an item click while hanging picks
   it up. **Test:** a goblin chasing him gives up; `--command "sendEvent
   ..."` hit / `setState` refused in `rope.log`.

#### 6b — Azriel's Hammer (planned 2026-10-21)

Why: the user threw the hammer in Swamp (where `constart.cs` gives it)
and saw an arrow. Nothing of the hammer was built: `prhammer.kq` is
`classType=KQHammer`, our projectile reader takes only `KQProjectile`
(`characters.log`: "Prhammer.kq not found or not a KQProjectile"), so
Connor kept his start bolt `PRarrow.kq` and its offset and shot an arrow
at `conhamr.anm/attack`'s `do` cue. Source: `azriel-hammer.md` (A,
2026-10-20, checked by the user in the original), with
`ranged-weapons.md` (readying, the click, the bolt's move) and
`fighting.md` rules 16–33 (the victim). Readying, putting away and the
pointer are already built (5f-3, 5f-4) and work for it (rule 1).

Words: **way out** / **way back** = the hammer's two flights; **throw
state** = Connor's state from `attack` to the end of `catch`.

Where things go:

- `kq8-data/characters.rs`: `ProjectileKeys` also reads `KQHammer`
  files, with `modifier`, `minDamage`, `maxDamage` (default 0) and a
  "hammer" flag.
- `kq8-core/fight.rs`: a bolt gets an optional hammer part (way out or
  back, the way-back range, the start point); `bolts_step` turns it back
  instead of removing it, and on the way back sees only Connor; the
  hammer's damage and the `lever1` / `tmblrcyl` case; "came back" is
  reported once for the viewer. Unit tests.
- Viewer `connor_view.rs`: the throw state (a new walker mode like the
  rope's: `attack`, then `wait` repeating, then `catch` when told);
  `door_view.rs`: the click, handsOff on and off, the launch at the `do`
  cue, the catch.
- Everything logs to `fight.log` (the note's "How to check" list).

Steps (one commit each):

1. **6b-1, reading the hammer's file, and the right projectile** (**done**
   2026-10-21; the cause: Connor's fight was set up before the start-up
   scripts' items reached core) (rules 5, 6). `KQHammer` files read with their keys; `characters.log` shows
   Daventry's 24,000 / 15,000 / 30 / 0 / 300. Find and fix why Connor
   kept `PRarrow.kq` although the hammer was carried at the start (the
   `shootOffsets` line `INVITEM_AzrielHammer -41 -605 1226` must be his
   offset too). **Test:** `--world Swamp`: `fight.log` names
   `Prhammer.kq` and offset (−41, −605, 1,226).
2. **6b-2, the flight** (**done** 2026-10-21; Q148 asked, not blocking:
   where it turns back without a touch) (rules 6–9; core, unit tests). Way out: as a
   bolt (speed / 15 per update, its box as a ball stack, mask 0x221F,
   never Connor). Turns back at the first touch, past `maxDistance`, or
   at / below the floor grid: put back at its last position before the
   touch, direction reversed, distance 0, range = way out − speed / 15,
   mask Connor only. A touch on the way out: `lever1` or `tmblrcyl` →
   `lever1.cs` with `lever1 ConnorAction`, no damage; type bit 0x4000 →
   a hit of 0 (no `ProjImmune`); other objects and characters → min(
   `modifier` + ranged damage, `maxDamage`), then at least `minDamage`,
   kind `Projectile`; buildings nothing. Way back ends touching Connor,
   past its range or at the floor grid: removed, "came back".
3. **6b-3, the throw state** (**done** 2026-10-21; stand-ins Q148 c–e: the
   hammer back during `attack`, requests during the throw, the panels)
   (rules 2–4; viewer). A click with the
   hammer ready (after the click rules 13–18 already built): turn toward
   the aim point with no waiting; refused unless standing or moving (not
   kept); handsOff on; `attack`; at its `do` cue the hammer leaves from
   his position + his turn × (−41, −605, 1,226) (rule 5: the fallback,
   from the user's check), aimed at the aim point, no lead; `attack`
   ends → `wait` repeating; "came back" → `catch`; its end → handsOff
   off, standing. Hits on him still count. No moving throw.
4. **6b-4, runs** (**done with 6b-3**: Swamp at an object, Daventry at a
   building, a goblin and the ground; not run: a throw from a real walk, a
   long throw at nothing) in Daventry (`give AzrielHammer`) and Swamp: at a
   creature, at nothing, at the ground near him; the time from throw to
   catch (1.0 s of flight at Daventry's range 15,000 → 1.25 s).

Fix after the user's check (2026-10-21, Q148 c): the hammer back while
`attack` plays cuts it short with `catch` at once (against a wall he
catches it the moment it leaves his hand).

Not in 6b: first person (not built); Dead City (not loaded yet, so the
lever is unit-tested only); the M7 base damage idea. How the hammer model
is turned in flight is not in the note: drawn like a bolt (pointing along
its flight, so it turns round for the way back), asked as a question if
it looks wrong.

#### 6c — A weapon's movement set loaded when it is given (planned 2026-10-21)

Why: the user's check: the Lake Sword given by the console in Dead City has
no stance (no sword-in-hand `stop`, walk, run). A weapon's **set** (its
`.anm` list's own versions of Connor's movements) is loaded at start-up
only for weapons the world mentions (carried, named in a script or an
object file, the fists). Loading every set in every world was tried and
undone: +2.2 s on every load (Dead City 2.4 → 4.6 s, Daventry 3.4 →
5.6 s). The user chose: load a set when its weapon is given. Not from a
behaviour note (the original's loading is not studied); only how the
movements are read is the same as at start-up. In normal play, once
moving between worlds exists, carried weapons are loaded at start-up as
now.

Steps (one commit each; logged in `viewer.log` and `connor.log`):

1. **6c-1, one set's loading as its own piece** (**done** 2026-10-21; no
   behaviour change). `load.rs`: the part of `place_connor` that reads one
   set's movements and their animations becomes a function (`connor_set`)
   both start-up and the viewer while playing can call. **Test:** Daventry's and
   Swamp's `viewer.log` "connor: movement …" lines unchanged.
2. **6c-2, loaded at the give.** At start-up a resource keeps what it
   needs: the game folder, the world's name, Connor's palette, and the
   lists of the weapons whose sets were not loaded. Each update the viewer looks at the
   weapons Connor carries; one whose set isn't loaded gets it now: the
   world's archives opened again, the movements read and drawn (frames
   under his body, his materials added to the potion looks and the
   mouth, so Strength / Invisible apply to them too). A weapon already
   ready switches to its set's movements at once. Logged: which set,
   how many movements, how long it took. **Test:** Dead City, `give
   LakeSword`, ready it: `connor.log` draws `conSword.anm/stop`;
   `viewer.log` the load time (~0.1 s expected).

### M7 — Match the original, then improve

Play the original next to ours, list differences, fix them. After that,
the improvements this project was started for: controls, graphics,
gameplay changes.

Improvements already noted (original behaviour we copy until then):

- Pushed barrels go through walls (`pushing.md` rule 4.1, the user's
  check 2026-10-18): make walls stop them, after M7.
- Character shadows darken the ground twice where parts of the figure
  overlap (`characters.md` rule 36, the user's check 2026-10-19): draw
  each shadow as one even shape, after M7.

### Outside the milestones: choosing the graphics API, `--renderer` (planned 2026-10-16, done 2026-10-16)

Not from the original game (its Glide / Direct3D choice is not studied):
our own launch option, asked for by the user. The viewer draws through
Bevy, which draws through **wgpu**, a layer that turns one set of drawing
calls into Vulkan, DirectX 12, Metal or OpenGL. Until now wgpu chose by
itself (Vulkan on the user's PC), and only the `WGPU_BACKEND` environment
variable could change that.

One step:

1. `--renderer auto|vulkan|dx12` (default `auto`). `auto` keeps today's
   behaviour (wgpu chooses; `WGPU_BACKEND` still works). `vulkan` or
   `dx12` tells Bevy's `RenderPlugin` to use only that API (it wins over
   `WGPU_BACKEND`). `vk`, `d3d12` and `directx` are accepted as other
   spellings. OpenGL (`gl`) is refused with a message: Bevy is built
   without its OpenGL part here (feature `gles`), and on Windows it would
   also need the ANGLE library; a test run with `WGPU_BACKEND=gl` crashed
   with "Unable to find a GPU". Adding it would be its own step.
   `viewer.log` records the API asked for at start-up and, once the
   graphics card is set up, the API actually used, the card's name and
   driver (`renderer: asked ... got ...`).
   If the chosen API is not available on the computer, Bevy stops with
   "Unable to find a GPU" (its own message); the log line written
   before that says which API was asked for.
   **Test:** run with each value and `--frames 60`; `viewer.log` must say
   `got Vulkan` / `got Dx12`; a wrong value must be refused.
   **Done** as planned (`crates/kq8-viewer/src/renderer.rs`); all values
   tested from the log, Daventry runs on both APIs; checked by the user
   2026-10-16. Not tested on Linux or another graphics card.

### Outside the milestones: the command console (planned 2026-10-21, done 2026-10-21)

Not from the original game: a developer tool asked for by the user, so
commands like `give SmCrossbow` or `move Connor 0 x y z` can be typed
while playing. It reuses the script commands we already run (the same
ones `--command "WORDS",N` runs); it adds no new game rules.

One step:

1. **Typed in the terminal.** The viewer reads lines from the terminal
   that started it (`cargo play`) on a side thread, so the game never
   waits for typing. Each line runs at the start of the next update (15
   per second), before the doors, as if a script had run it. The
   terminal shows the line, its `$Console::Return` result and the log
   lines the command wrote; a `move` of Connor or the camera is applied
   later in the same update (the trap step), so its "applied" line is
   only in `doors.log`. Everything the game already prints keeps
   printing. Console-only words: `help` (the list) and `where`
   (Connor's position and turn, the numbers `move Connor 0` takes).
   Every console line also goes to `kq8-artifacts/logs/console.log`.
   No in-game text box (it would need keyboard focus rules that fight
   the game's own keys); can come later.
   **Test:** `cargo play --world Daventry`, click in the terminal, type
   `give SmCrossbow`, Enter: the terminal prints the result `0` and the
   item line; the crossbow is in the bottom bar. `where`, then
   `move Connor 3 0 0 2000`: Connor drops from 2,000 higher (flags 3 add
   to his place and turn; 1 alone would set his turn to the number
   given, 0).
   **Done** as planned (`crates/kq8-viewer/src/console.rs`); tested with
   commands piped in (not typed by hand), checked in the logs only.
2. **The typing line** (the user's request after trying step 1: the
   game's output moved the half-typed text up the screen). In a real
   terminal the last line is kept for typing, with a divider above it;
   the game's output scrolls only in the lines above (the terminal's
   "scrolling region"). The console reads single keys (the `crossterm`
   library) and draws the typed text itself: Enter runs it, Backspace,
   Esc clears, Up/Down go through earlier lines, Ctrl+C still stops the
   game. The terminal is given back when the game ends (or crashes).
   With piped input (our tests) it reads plain lines as in step 1.
   **Test:** `cargo play --world Daventry`; the bottom line shows `> `,
   the game's lines appear above the divider while you type.
3. **More console words** (the user's request: a fuller `help`, `list`,
   more useful or fun commands). All ours, none change the game's rules;
   where one acts on the game it goes through a rule already built:
   - `help` grouped by use; `list` (the kinds), `list items|weapons|armor|
     potions|quest [word]` (the item list, `Invinfo.itl`, by category),
     `list creatures|doors|chests|objects [word]` (this world, with the
     distance from Connor), `find <word>` (any named thing).
   - `inv` (what Connor carries), `stats` (his five stats, level, god
     mode, AI, weapon).
   - `heal`, `xp <n>`: a `KQStatsChangeRequestEvent` to Connor
     (`anim-scripts.md` rule 19), as a script would send.
   - `god`, `ai`: the F3 / F4 test switches.
   - `goto <name>`: `move Connor 0` to 1,000 units from it on his side,
     500 higher, turned toward it (he falls onto the ground; he can land
     in a wall or a roof: it is a jump, not a walk).
   - `spawn <item> [count]`: `KQMonster::inventory world` 600 units in
     front of him (`items.md` rule 9).
   - `kill <name>` / `kill near [distance]`: the creature dies as from a
     hit (`fighting.md` rule 30 death, its drop, its body), without the
     hit's alarm to the others. A small core function, `console_kill`.
   **Test:** piped commands, `console.log`, `fight.log`.
   **Done** 2026-10-21 (`console_words.rs`; `fight.rs` `console_kill`
   with a unit test); tested with piped commands and the logs only.
