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

### M5 — Fight

**You will see:** monsters that move, attack and die; Connor's health and
stats.

- A: monster AI per type, combat rules and numbers, stats, death.
- B: implement.

### M6 — The whole game

Save/load, moving between worlds, all 12 worlds, cutscene videos, music,
options. A and B as needed per topic.

Already known for later worlds (kept out of M3): Gnome's rolling rocks
knocking Connor back and hurting him (`connor-collision.md`, Q55;
damage needs M5), Gnome's pushable `boulder` (`KQBoulder`, not
studied), the other pushable classes (`pushing.md` open question 1),
damaging water (poison, lava, swamp).

### M7 — Match the original, then improve

Play the original next to ours, list differences, fix them. After that,
the improvements this project was started for: controls, graphics,
gameplay changes.

Improvements already noted (original behaviour we copy until then):

- Pushed barrels go through walls (`pushing.md` rule 4.1, the user's
  check 2026-10-18): make walls stop them, after M7.

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
