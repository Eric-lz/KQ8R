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

### M3 — Daventry comes alive

**You will see:** doors opening, traps firing, chests, animated objects,
the first puzzles working.

- A: the script language (`.cs` commands like `set`, `if test`,
  `sendEvent`), the event system, and each object kind used in Daventry
  (door, trap, chest, emitter, animation lists...).
- B: script interpreter, event system, the object kinds.

Notes: `docs/behaviour/doors.md` (doors, their scripts, the minimum of the
`.cs` language), `docs/behaviour/emitters.md` (emitters, particle
catchers, ground animators).

- **3a — Daventry's doors (planned 2026-10-12; 3a-1, 3a-2, 3a-3 done
  2026-10-12, MODLOG; checked in the logs, and by eye with the mouse by
  the user).** Rules from
  `doors.md`.
  Three sub-steps, each with its own tests and MODLOG entry.
  - **Words used here.** A *script* is one of the game's `.cs` text files
    (puzzle logic). An *event* is a message sent to a named object
    (`sendEvent`), handled when its time comes. A *movement* is one of an
    object's named animations (`closed`, `open`, ...); its *phase* runs
    from 0 (first frame) to 1 (last frame). A *cue* is an action written
    in the movement (`cue0=0.0 playsound door4.aud`) that fires when the
    phase passes it.
  - **3a-1, the script runner** (`kq8-core` `script.rs`, no Bevy). Only
    what `doors.md` "The script language" lists:
    - a line is split into words (spaces/tabs), `#` starts a comment, a
      double-quoted part is one word;
    - `$1`…`$n` the call's arguments (missing ones empty), `$Name` a
      variable, replaced and split into words again; an unset or empty
      one **vanishes**, unless quoted (then one word, maybe empty);
    - `if <command…>` runs the rest of the line and takes the branch when
      `$Console::Return` is 0 or empty; `else` flips; `endif` closes;
      skipped `if`s are counted so their `else`/`endif` pair up; `else
      if …` = `else` + a nested `if` that needs its own `endif`;
    - `test a op b`: exactly 3 words (else a usage line, false); numbers
      when both look like numbers, else text with capitals counting;
      `&&`/`||` whole numbers only (error for text);
    - `set`, `echo` (to the log), a first word that names a script (with
      `.cs` added if it has no dot) runs it with the rest as arguments,
      an unknown command logs an error and fails;
    - `sendEvent <delay> <object> <type> [data…]` and
      `KQObject::collisionType <name> <±bits>` are handed to the caller;
      the later-milestone commands (`KQMonster::speak`, `doPopup`,
      `addMapIcon`, `earnTeleport`, `teleportersEarned`,
      `KQMonster::checkInventory`, `deleteObject`, `handsOff`,
      `KQMonster::setState`, `pauseAnim`, `preloadShape`,
      `KQObject::setHidden`, `loadKQ`, `setThirdPerson`, ...) are
      **stubs**: one log line, nothing else (`checkInventory` → 0).
    - Stand-ins (asked as Q45): `$Console::Return` is `0` for true or
      success and `1` for false or failure (the note gives only "0 =
      true"); a word "looks like a number" as the note says and its value
      is its leading number part. **Confirmed 2026-10-13** (`doors.md`
      "Q45" e, f; see 3r).
    - The start-up follower (`kq8-data` `world_script.rs`) is **not**
      changed in this step (it still skips a line with an unknown
      variable and ignores `if`). Moving it onto the new runner is a later
      step of its own.
    - Unit tests on made-up scripts (never copies of the game's files):
      vanishing and quoted variables, nesting, `else if` with its extra
      `endif`, `test` with numbers/text/wrong word count, `&&`/`||`,
      `$Console::Return`, a script calling another, `sendEvent` parsing.
      Viewer log: every Daventry door script read through the block
      counter (balanced `if`/`endif`).
  - **3a-2, the door and the event queue** (`kq8-core` `door.rs`,
    `events.rs`, no Bevy).
    - Door state (rule 1): closed / opening / opened / closing, closed at
      the start. `KQDoorEvent 1` opens only when closed, `2` closes only
      when opened, anything else (also no number) toggles; ignored while
      moving (rules 2–4). `KQAnimPlayEvent <movement>` starts it by name,
      state unchanged (rule 6).
    - Playing a movement: its `loop=` start phase and speed; phase moves
      by speed × (1/15 s) / play time per update; a play-once movement
      finishes at 1 (forward) or 0 (backward). When the finished
      movement is called `open`/`close` (any capitals): state opened /
      closed, the `opened` / `closed` movement, and the script with `$1`
      = door name, `$2` = `Open` / `Close` (rule 5).
    - Cues: a cue at phase c fires when c is in [lower, higher) of the
      phases before and after the update (rule 9). `playsound` is logged
      (no audio yet); anything else is logged as not done.
    - **The door's turn** (rule 24, open question 1): worked out from the
      phase every update, never added up: the shown frame is
      round(phase × (frames − 1)), as for Connor's one-shots, and the turn
      is the frame steps 1…k chained (each step in the door's own,
      already-turned axes). Measured from `door1.kqs` with a throwaway
      program: every chained step turns about the same point, (−634,
      −63) in the door's own space (its hinge edge); frames 13, 14, 15 are
      165°, 167.75°, 165°. So `close` from 0.93 (frame 14) starts 2.75°
      past `opened` (frame 15), a tiny jump, and a full cycle comes back
      to exactly 0°. The box turns with it: the door's collider is moved
      every update.
    - Events (`events.rs`): game time = updates / 15. `sendEvent` with
      delay d is due at now + d; due events are handled in time order,
      same time in send order. Unknown object or event type: logged,
      nothing sent. `KQCueEvent n1 [n2]` → the target's script with `Cue
      n1 n2` (rule 23). `KQStatsChangeRequestEvent` → logged (stats are
      M5).
    - **Order in one update** (stand-ins, Q45; confirmed 2026-10-13,
      `doors.md` "Q45" d): (1) clicks; (2) events
      due by now, including those just sent with delay 0 (open question
      4); (3) each moving door advances, except one started in this
      update, which first advances in the next (as Connor's transitions,
      Q40 a), cues fire, finishes call the script (events sent there are
      handled in the next update); (4) the door colliders are moved; then
      Connor's update as before.
    - Unit tests: the state table (1/2/0/none in every state), `open` 15
      updates and `close` from 0.93 14 updates (door1/door2; each door
      uses its own file's frames and time, Q46), `Open`/`Close` calls,
      `KQAnimPlayEvent open` from closed ends opened with `Open`, cue at
      0.0 fires in the first update, `Saradoor`-like cue at 0.95 on
      `close` never fires, turn after 10 open/close cycles is exactly 0,
      event order and delays.
  - **3a-3, clicking and the viewer** (`kq8-core` `click.rs`;
    `kq8-viewer` `door_view.rs`, `load.rs`, `main.rs`).
    - Load every `KQDoor`: its four movements (`loop=`, cues), all frames
      of its shape with their frame transforms, `script=`,
      `colliderType`, `acceptsInventory`, `acceptAllItems`,
      `tooFarAwayAction`, `doNotPointAt`, its `BOUNDS.KQ` box. Doors are
      drawn by `door_view.rs` (turned as above, the shown frame's mesh),
      no longer as static copies.
    - The play cursor: a **left click** sends a ray from the camera
      through the mouse; the nearest box hit among objects whose
      collision type has bit 0x20 is picked (rule 10). Stand-in (Q45):
      only those boxes are tested; walls, terrain and other objects do
      not hide a clickable one (confirmed, `doors.md` Q45 b). No weapons yet, so the cursor is always
      the play cursor.
    - Reach (rule 11): Connor's feet turned into the door's own axes
      (its current, turned box), only the part outside the box counts on
      each axis; in reach when that distance is < 2 × `attackDist`
      (`GAME/Kq/Connor.kq`, 2,350 → 4,700). The box as it stands, turned
      (confirmed, Q45 c).
    - In reach, nothing in hand: script `<door> ConnorAction do do`.
      Item in hand (test option only, no inventory yet): accepted when
      the door's `acceptsInventory` names its type, or `acceptAllItems=1`
      (stand-in, open question 2), then `<door> ConnorAction <item
      object> <item type>`; refused: logged "that doesn't work" remark,
      no script. Out of reach: `tooFarAwayAction` 0 → logged "too far"
      remark, nothing sent; else `ConnorAction TooFar TooFar` (rules
      12–14). A picked object that is not a door: logged, its script not
      run (only doors in this step).
    - `KQObject::collisionType` changes the runtime collision type: −32
      makes a door unclickable. Removing bit 0x1 would stop Connor's
      collision; no Daventry script does, so it is logged, not applied.
    - Not done (logged): Connor turning to face the door (rule 16, which
      states not traced), the remark lines themselves (M4), hits,
      `Die`, `ProjImmune` (no combat yet, M5), `tooFar=` (a display
      setting, not reach: Q45 a).
    - Test options: `--click NAME,N` clicks object NAME before update
      N + 1 (no mouse needed); `--item TYPE` puts an item of that type in
      Connor's hand (object name = the type).
    - Log `kq8-artifacts/logs/doors.log`: for every click the picked
      object, distance to its box, reach yes/no, the script arguments;
      every script line that does something (echo, `set`, stubs, errors);
      every `KQDoorEvent` with its value and the state before and after;
      every finished movement with its `Open`/`Close` call; cues; one
      line per update for each moving door (phase, frame, turn angle).
    - Expected (`doors.md` "How to check"): `Condoor` clicked in reach →
      `Condoor ConnorAction do do` → `KQDoorEvent 1` → opening for 15
      updates → opened → `Condoor Open`; clicked again: stays open; from
      far away: "too far", nothing sent; open door turned ~165° out of
      the doorway, still solid where it now stands.

- **3b — emitters, particle catchers, ground animators (planned
  2026-10-13; 3b-1, 3b-2, 3b-3 done 2026-10-13, MODLOG; checked in the
  logs and by eye by the user).** Rules from `docs/behaviour/emitters.md`. Three
  sub-steps, each with its own tests and MODLOG entry.
  - **Words used here.** An *emitter* is an invisible point that keeps a
    fixed number of *particles* (small models: leaves, drips) flying; each
    particle repeats the same path every *life*. A *particle catcher* hides
    one emitter's particles inside a building's box. A *ground animator*
    swaps the ground texture of some grid squares every few tenths of a
    second (flowing water). A *tick* is one game update (1/15 s).
  - **Where the code goes.** Reading the keys: `kq8-data` `effects.rs`
    (no rules, just the numbers with their defaults). The rules:
    `kq8-core` `emitter.rs` (particles and catchers) and `ground_anim.rs`
    (animators), no Bevy, unit-tested. Drawing and the tick:
    `kq8-viewer` `effects_view.rs` (emitters, catchers) and
    `ground_anim_view.rs` (animators), run in the fixed 15-per-second
    update after the doors and Connor. Logs:
    `kq8-artifacts/logs/effects.log` and `ground_anim.log`.
  - **Random numbers:** a small seeded generator of our own (uniform
    fractions, the note says the original's sequence needn't be copied).
    Same seed every run, so two runs give the same log.
  - **3b-1, emitters.** Keys and defaults from the note's table
    (`maxParticles` 200, `maxLife` 100, `translucency` 256). Position: the
    object's place after the usual placement rules (`loc`, `loadKQ`
    position); whether the ground rules apply to emitters is not in the
    note: the log lists any emitter below the ground. At the start every
    particle gets its offset, velocity, spin, sway speeds and starting age
    (rule 1) and is moved on by age × velocity. Every tick: rule 3 steps
    1–6 in that order, sway (rule 6), gravity as age × g (rule 7),
    restart after `maxLife` + 1 ticks, `delayStart` (rule 4). "Non-zero" =
    absolute value ≥ 0.000001. Spin: the turn angles add up and are used
    like an object's `dirX/Y/Z` (the note's assumption). Drawn: frame 0 of
    `shape` at the particle's position and turn, only when visible and age
    ≠ 0, with the emitter's `translucency` (likely, open question 2), blended
    between ticks except right after a restart. Always active (rule 12).
    Not done (logged): `follow` / `followMesh` (rule 11; no Daventry
    emitter uses it), dying (rule 10; no file uses it), the script
    commands (open question 4). `randomZRot` ignored (Do not reproduce).
    Tests: same path every life, restart after `maxLife` + 1 ticks with
    one hidden tick, gravity sum n(n+1)/2, sway = velocity × sin, delayStart.
    Log: per emitter its settings and particle count; for `stream1` and
    `leaf1`, particle 0's choices and then per tick its age, position and
    visibility (note "How to check").
  - **3b-2, particle catchers.** After loading, each catcher looks up its
    emitter and building by name (any case; nothing found → does nothing,
    logged; the original counts capitals, but every catcher's names match
    exactly in all worlds, 3r). Box = the building's `.dig` bounding box; top = box top +
    building z. Every tick, after the emitters: each visible particle of
    its emitter with z ≤ top is turned into the building's own axes and
    hidden if min ≤ value < max on all three axes. Catchers run after the
    emitters in the same tick (the original's order is not settled; ours
    is fine, `emitters.md` catcher rule 5). Log: each catcher's
    box and a line whenever it hides a particle.
  - **3b-3, ground animators.** Squares chosen once at load (explicit
    list if `numGridBlocks` ≠ 0, else every square whose material is an
    `origTexture`, minus the exclude list). Started when `startNow=1`
    (else logged and left alone). Changes: the first after `delay`
    seconds, entry 0; then every `delay` seconds the next entry; with
    `repeatflag=0` stop at the last. ~~Timer on the game clock (ticks /
    15), next due `delay` after the previous due time~~ (stand-in;
    **replaced in 3r-1**, Q49: continuous game time, checked every drawn
    frame, next due `delay` after the time of the change). Ground flags
    (envInfo) not changed (open question 3). Drawn: animated squares are
    taken out of the normal ground mesh's surface (their floor stays in
    it) and drawn one entity each; on a change its texture follows the new
    material, while its orientation stays that of its own loaded material
    (changed after the user's check; confirmed by Q50). The floor view
    (U) treats them as surface squares.
    Log: squares chosen per animator (expect the note's table: 86, 0, 57,
    19, 0, 17, 32, 2, 13, 4, 4), square (45, 57) never animated, and each
    change with its game time and material.

- **3b-4 — emitters that follow something (planned 2026-10-13; done
  2026-10-13, MODLOG; checked in the logs and by eye by the user).**
  `emitters.md` rule 11. In the data only IceWorld's `snow` follows
  something when a world starts (Connor); the dragon fires are made by
  the dragon heads (`fireFile=`, monsters, M5) and Temple4's `lucfire` by
  its altar script, so those two kinds are not started yet.
  - `follow=<name>`: Connor, `KQCamera` (the game camera) or a placed
    object (any case). Every update, before the particles move, the
    emitter's `loc` = the target's position + `followOffset`, the offset
    turned with the target unless its x and y are both 0 (then world
    axes). Connor's turn = his heading about z; the camera: position only,
    no turn (the original turns the offset by the camera's turn, Q52 c;
    no emitter in any world follows the camera). Target not found: the
    emitter is not started (logged; the original would crash, so keep
    this, Q52 b).
  - Velocity limits turned with the target: a particle's velocity is
    chosen in the limits, then turned by the target's turn at that
    moment (rule 1: at the start; rule 2: kept after that).
  - `followMesh`: at each restart the particle takes the target's current
    turn and a new random velocity (turned). Its value in the data is a
    mesh-part name (`fire`, `Box01`). **Answered (Q52 a), not built yet:**
    the emitter sits at that part of the target's (animated) model,
    `followOffset` unused; part missing → the emitter dies (rule 10). To
    do with the dragon heads (M5) and Temple4's `lucfire`, the only users.
  - Particles already flying don't move with the target (their positions
    are world positions); restarts happen at the moved `loc`.
  - Tests: offset in own vs world axes, turned velocity, restart at the
    moved spot, `followMesh` new velocity and turn. Log: `snow`'s `loc`
    vs Connor every second, particle 0 as before.

- **3c — animated objects play (planned 2026-10-13; done 2026-10-13,
  MODLOG; checked in the logs and by eye by the user).** Rules from
  `objects.md` "Animated objects at load" (rules 2–4, Q13), the phase
  rule the doors use (`doors.md` rules 5, 8) and the loop length Connor's
  loops use (the loop's stored length). Until now every animated object
  other than a door was drawn frozen in its start frame (the user saw the
  fountain's water, `wellwatr`, as drips stuck in the air).
  - **Which objects:** every placed object with `animLoops` that is not a
    door, whose current movement's `loop=` speed is not 0 (Daventry: the
    windmill sails, mill wheel and gears, signs, shutters, torches, lamps,
    burners, waterfalls, fish, the fountain water, …). Speed 0 stays a
    still model, as now.
  - **Playing** (`kq8-core` `anim.rs`, no Bevy): phase starts at `a`
    (`loop=` start); each update it moves by speed × (1/15 s) ÷ the loop's
    stored length; repeating loops wrap round 0..1, one-shots stop at the
    end (and stay there: `nextAnimName` is not traced). Frame shown:
    repeating round(phase × frames) mod frames (rule 2, as the start
    frame), one-shot round(phase × (frames − 1)) (as doors). `flags` bit 8:
    random start phase and speed × a random 0.9–1.1 (rule 4).
  - **Whole-object steps** (frame transforms, models-and-textures.md Q1 b):
    first measured and logged per object. Applied, if any running object
    has them, like the doors: the pose of the shown frame = its steps
    1…k chained, never added up across cycles. (Q51: the original adds
    them up and the data's cue puts the object back each cycle; the same
    picture for Gnome's rocks, the only ones, so kept, by the user's
    check. `KQBoulder` is not played: 3r-2.)
  - **Not done (logged):** cues (sounds), `nextAnimName`, `animPos`,
    scripts changing an object's movement (`KQAnimPlayEvent` to non-door
    objects). Collision boxes stay fixed. (Q51 rule 5: in the original a
    box moves with its object; "fixed" in Q33 a means its size. Only the
    rolling rocks move, and their colliders don't follow yet; not done.)
  - **Drawing** (`kq8-viewer` `anim_view.rs`): all frames of the loop
    built once per (model, loop), one entity per frame, only the shown one
    visible (as doors); hidden and unused objects keep their H-key
    visibility; see-through objects keep their opacity.
  - **Log** `kq8-artifacts/logs/anim.log`: per object its loop, frames,
    length, start phase and speed (with the random parts), its steps;
    per update for a few objects (`wellwatr`, `windprop`, `millwheel`,
    a torch) the phase and frame. (A loop's stored length is
    (frames − 1) / 15 s, so a repeating loop that runs through all its
    frames per cycle skips one frame per cycle at 15 updates per second;
    that follows from the rules, not a bug.)
  - Tests: phase and wrap, frame rule (start frame equals the still model's
    start frame, e.g. `telepad1` 0.3 × 16 → 5), one-shot stops at the end,
    random range, speed −1 runs backwards.

- **3r — review after the answers to Q45–Q52 (planned 2026-10-13; 3r-1 to
  3r-4 done 2026-10-13, MODLOG; checked in the logs and by eye by the
  user).** The
  study session answered Q45–Q52 (`doors.md` "Q45", rules 8 and 24;
  `emitters.md` emitter rules 2 and 11, catcher rule 5, ground animator
  rules 2 and 3; `objects.md` "Loops that move the whole object"). Each
  answer checked against the code and, where it could matter, against the
  data of all 12 worlds (throwaway scanner in B's scratch folder, nothing
  kept):

  | Q | Answer | Our code | Change |
  | --- | --- | --- | --- |
  | Q45 a–f | all six stand-ins right; `$Console::Return` details | matches (script call keeps the last command's result; `-.5` is text; unknown command 1) | comments only. Not copied: a false text `==` can leave −1 instead of 1 (which sign for which order is not given; both count as false; asked as Q54) |
  | Q46 | each door plays its own `.kqs` | already | none |
  | Q47 | sway and turn carry on; paths drift | already | none |
  | Q48 | catcher order not settled (ours fine); names compared with capitals counting | we ignore capitals | none: every catcher's `emitter`/`interior` name in all worlds matches exactly (Daventry 3, IceWorld 10), so both give the same result |
  | Q49 | next change = clock time **of this change** + `delay`, on continuous game time, not the 15/s tick | previous due + `delay`, checked once per tick | **3r-1** |
  | Q50 | the square's own orientation | already (the fix) | none |
  | Q51 | steps added up + the data's reset cue; user: our chained way gives the same picture, keep it | chained per frame | none for the rocks; **3r-2** for `KQBoulder` (rule 8: don't play it) |
  | Q52 a | `followMesh=<part>`: sit at that model part, `followOffset` unused, part missing → the emitter dies | name ignored | none now: only the dragon fires (IceWorld, made by monsters, M5) and `lucfire` (Temple4's altar script) use it, and neither is started. Do it when those arrive |
  | Q52 b, d | missing target: keep ours; order right | already | none |
  | Q52 c | offset turned by the target's whole turn, the camera's too | Connor heading, objects all three angles, camera no turn | none: no emitter in any world follows `KQCamera` (only `snow` → Connor, offset z only) |

  - **3r-1, ground animator clock (Q49).** `kq8-core` `ground_anim.rs`:
    after a change at clock time t the next is due at t + `delay`
    (`update` takes the clock time; the change records it). `kq8-viewer`
    `ground_anim_view.rs`: the step moves from the 15-per-second update to
    every drawn frame, with game time = the fixed clock's time + its
    overstep (the time already run since the last update), so it is
    continuous and stays in step with the other systems' game time. The
    log gives each change's game time and the gap since the last change
    in seconds; the 10-second summary gives the gaps' smallest, largest
    and average. Expected: gaps 0.300 s plus at most one drawn frame (at
    60 frames per second, 0.300–0.317 s), never the 0.267/0.333 s of the
    old tick-based timer. Unit test: changes checked at irregular frame
    times are each `delay` after the previous **change**, not after the
    previous due time.
  - **3r-2, `KQBoulder` still (Q51 rule 8).** `kq8-viewer` `load.rs`: an
    object whose `classType` is `KQBoulder` is not played (drawn still,
    frame 0: its `stop` loop starts at 0.0), logged. Only one in the game
    (Gnome `boulder`, `pushston`). Until pushable objects are studied.
  - **3r-4, a moving object's collider moves with it (Q51 rule 5).**
    Found while checking 3c against Q51: Gnome's three rolling rocks have
    `BOUNDS.KQ` boxes (`theroll.kqs`, ±900 × ±900 × 0–1,750, type 0x1,
    which Connor collides with), but our boxes stay at the rocks' start
    while the rocks roll up to 9,691 away. `kq8-viewer` `anim_view.rs`:
    every update, an object whose loop has whole-object steps puts its
    collider (found once by the object's name in the world's object
    colliders; none or several → logged, left alone) at the pose it is
    drawn with (the chained steps of the shown frame, as 3c). Runs in
    `anim_step`, which comes after Connor's update, so Connor is tested
    against the box of the previous update (one update late; the
    original's order between objects was not traced). Not done: what
    happens when a moving box runs into a standing Connor (the mover only
    sweeps Connor's own moves; a box that ends up around him counts as
    "started inside", which was not traced, `collide.rs`); the rocks
    knocking him down (not studied). Log `anim.log`: each such collider
    found, then its position every second for the first 10 s.
  - **3r-3, docs.** Comments in `script.rs`, `ground_anim.rs`,
    `ground_anim_view.rs`, `emitter.rs`/`effects_view.rs` and the plans
    above (3a, 3b, 3b-4, 3c) now say "confirmed" where they said
    "stand-in"; provenance updated; Q54 asked.
  - **Next after 3r:** traps (`traps.md`), chests (`chests.md`), items
    and picking up (`items.md`), each planned here before it is built.

- **3d — Daventry's traps (planned 2026-10-13; 3d-1 to 3d-3 done
  2026-10-13, MODLOG; checked in the logs, and by eye against the original
  by the user).** Rules from `traps.md`
  (and `connor-movement.md` rule 4, `camera.md` "Distance" rule 2 for the
  two commands). Three sub-steps, each with its own tests and MODLOG
  entry.
  - **Words.** A *trap* is an invisible box; when Connor walks into it or
    out of it, its script runs (`On` / `Off`). Its *inside list* is who is
    in it now. A *sticky* trap (`trapType=2`) has a smaller *inner* box to
    get in and the full box to get out, so music zones don't flicker at
    the edge.
  - **Data (Daventry, counted 2026-10-13):** 58 trap files, all
    `colliderMask=4` (Connor only; other worlds differ, Q57); 19 are
    templates that `newworld.cs`-style scripts place 17 times with
    `loadKQ` (positive z; their angles are ignored, the box never turns);
    8 files have `locZ=-1`; 4 sticky traps (`chrchtrp` 0.9, `mugltrap`,
    `mwiztrap`, `mwzhtrap` 0.8).
  - **3d-1, the trap rules** (`kq8-core` new `trap.rs`, no Bevy). A `Trap`:
    name, position, box (`BBMin/BBMax` added to the position, never
    turned), script, mask (`colliderMask`, default 0, stand-in: not
    stated in the note), `trapType`, `innerPercent`, inside list, shrunk
    yes/no. On entering the world: sticky → box × `innerPercent` (all six
    numbers, so it shrinks towards the position, rule 10). `check(object)`
    for one object that moved (name, type, position, its `BOUNDS.KQ` box,
    its ball stack), returning what to call:
    - who counts: the trap's mask has a bit of the object's type
      (stand-in, Q58: rule 4's two-sided pair rule would let every trap
      see Connor, whose own mask 0x281B has the trap type 0x10, and rule
      16 could not switch a trap off); otherwise nothing;
    - **touching** = one of the object's balls reaches the box (the ball's
      centre within its radius of the box; stand-in for open question 2:
      the same balls as for walls, tested where he ends the update, not
      along the way);
    - not inside and touching → added; first one in → `On <name>`, else
      `Cue 301 <name>` (rule 6); sticky → box back to full (÷
      `innerPercent`);
    - inside: still in if the centre of his box (position + middle of his
      `BOUNDS.KQ` box) is in the box (edges count) or he is touching;
      else taken off, list empty → `Off <name>`, else `Cue 302 <name>`
      (rules 7, 8); sticky and empty → shrunk again;
    - one-shot (`trapType` bit 1): gone after its `On` (rule 11; unused in
      the data);
    - mask change (`KQObject::collision`, rule 16): applied at once; no
      `Off` is sent for whoever is inside (stand-in, open question 1),
      and since the pair rule now fails, later moves neither add nor
      remove him (stand-in; logged).
    Unit tests: enter on touch, stay while the centre is in, leave only
    when both fail; sticky inner/full box and the shrink/grow numbers; a
    second object gives `Cue 301/302`; the pair rule (mask 0 never fires);
    one-shot.
  - **3d-2, traps in Daventry** (`kq8-viewer` `load.rs`, `door_view.rs`
    or a new `trap_view.rs`; `kq8-core` `active.rs`). Loading: every
    placed `KQTrap` becomes a trap: position as written (`keepOnGround`
    ignored, rule 1), z of −1 → the floor grid height there (rule 2), the
    template copies at their `loadKQ` place. The traps join the doors'
    world (`Active`): the same variables, script runner and event queue.
    At the start each trap's script gets `Init` (rule 13). Each update,
    **after Connor's update** (so the trap sees where he is now), if his
    position changed, every trap checks him; `On`/`Off`/`Cue` run the
    trap's script with `$1` = trap name (rules 6, 8). `sendEvent … <trap>
    KQCueEvent n1 n2` → `Cue n1 n2` (rule 14, as doors). `KQObject::collision
    <name> <±bits>` changes a trap's (or door's) mask (rule 16).
    `TalkComplete` and `StateChange` are M4/M5. The shared scripts
    (`walk_la.cs` ...) are in `GAME/Kq`: scripts are looked up in the
    world first, then `GAME/patch`, then `GAME/Kq`
    (`models-and-textures.md` "Which one wins"; no Daventry script has a
    game-wide twin). Log
    `kq8-artifacts/logs/traps.log`: every trap at load (name, file, box,
    shrunk box, mask, script, `Init`), and every `On`/`Off`/`Cue` with
    Connor's position, his box centre, the box used, and the script's
    lines. The main log gets a count at load and each `On`/`Off`.
  - **3d-3, what the walk/run zones do** (`kq8-core` `active.rs`, viewer).
    `KQConner::setRun 0|1` sets Connor's run switch (connor-movement.md
    rule 4); `KQCamera::setRange <min> <max>` sets the camera's range,
    refused unless min < max (camera.md). Both are handed to the viewer
    as requests and applied before the camera's update in the same tick.
    The other commands the Daventry trap scripts use and that only act
    (music, monsters, ropes, cutscene camera, `move`, `handsOff`, ...) are
    added to the stubs (one log line); commands that answer a question
    (`getloc`, `checkDistance`, `exist`, `checkSave`, ...) stay unknown
    (error line, result 1) until they are studied. `--hold F,T,R` then
    sets the run switch only at the start, so the scripts can change it.
    What you'll see: by Connor's house and the other houses he drops to a
    walk and the camera comes closer; away from them he runs again.
  - **3d-4, corrections after Q57/Q58 (planned and done 2026-10-13).**
    The study answers (`traps.md` rules 4, 6, 16) change two stand-ins:
    - **Touching is swept** (rule 6): each of the visitor's balls, from
      where its move started to where it ended, against the trap's box
      widened by the ball's radius (a slab test); a ball that starts inside
      counts. The move that enters is not also tested for leaving. For
      staying (rule 7, "still touching") the same swept test is used
      (stand-in: the note doesn't say; at most one update later `Off`).
    - **Mask changes only gate coming in** (rule 16): those inside stay
      inside and get their `Off` when they leave.
    - Q58's one-sided rule was already the stand-in: unchanged.
    Tests: a move straight through a thin trap gives `On`, the next `Off`;
    a path beside it nothing; mask removed while inside → `Off` on
    leaving, no new entry.

- **3e — chests and items (planned 2026-10-13; done 2026-10-13: 3e-1
  to 3e-5 and 3e-4b, MODLOG).** Rules from
  `chests.md` and `items.md` (and `doors.md` rules 10, 11, 13, 14 for
  clicking, `objects.md` for heights and hiding). Five sub-steps, each
  with its own tests and MODLOG entry.
  - **Words.** An *item type* is one line of `GAME/Kq/Invinfo.itl`
    (`INVITEM_Crystal`: its model, sounds, size). An *item object* is one
    thing of that type, in the world or carried, holding a *quantity*
    (one object can be 200 coins). An *inventory* is the list of item
    objects a character carries (in M3 just a list; the screen is M4).
  - **Data (Daventry, counted 2026-10-13):** 3 `KQChest`s placed
    (`smlchst3` 3 crystals, pub upstairs; `smlchst4` 200 silver coins,
    keep; `wizchest1` 1 lady bell, hidden, type 1 until a Reveal potion);
    the `jar` chest template is not loaded. 3 *scripted* chests, plain
    animated objects with a script, type 33 (`smlchst1`, `smlchst2`,
    `jar1`). Items at the start: 21 `KQMonster::inventory world …` lines
    in `WorldInv.cs` and 1 in `newWorld.cs` (`BirthdaySuit`, hidden); 23
    Daventry scripts make or give items.
  - **3e-1, item types** (`kq8-data` new `items.rs`, `kq8` CLI). Read
    `Invinfo.itl`: `[InventoryItemTypeList]`, `itemN=` lines, fields
    tab-separated in the order of the `;itemX=` header (rule 1). Keep
    the name, id, category, land and pickup sounds, shape, radius,
    translucency. Name lookup (rule 2): put `INVITEM_` in front unless
    the name starts with exactly that, then compare ignoring capitals.
    New CLI command `kq8 item-verify` (log
    `kq8-artifacts/logs/item-verify.log`): number of types (expect 135),
    each type's shape found or not (world archives, then `GAME`), its
    `BOUNDS.KQ` box (shape name lower-cased, as for objects), and every
    item type named by any world's scripts and chests that the lookup
    does not find. Unit tests: field split, the prefix rule
    (`silverCoins`, `INVITEM_Crystal`, `ladybell`, `invitem_X` gets a
    second prefix, so it is not found).
    **Done:** 135 types, all models found, 3 without a box; of 758 type
    names in the data only `INVITEM_All` (an `acceptsInventory` value) and
    one id number don't resolve (Q62). `acceptsInventory` turned out to
    hold comma-separated lists too (Q62); it matters for using items on
    doors (today: exact comparison), not for this step.
  - **3e-2, item objects and inventories** (`kq8-core` new `item.rs`,
    `active.rs`; no Bevy). An item object: name, type, quantity, third
    number −1 (rule 3), where it is (in the world at a position and turn,
    or carried by someone), hidden, script, `doNotPointAt` (default on,
    rule 4). Commands in the running world's scripts:
    - `KQMonster::inventory <who> <type> [qty] [x y z [rx ry rz]]` and
      `give <type> [qty]` (rule 9): a new object; `world` → placed (z −1
      / −2 → floor / surface height, which the caller supplies), any
      other name → carried by it; sets `NewInvItem`; unknown type →
      error line, result 1.
    - `KQMonster::checkInventory <who> <type> [var]` (rule 10): result 1
      if found, 0 if not or on error (was a stub answering 0).
    - `KQObject::setScript <item> <script|none>` and `setDontLook <item>
      0|1` on item objects; `KQObject::setHidden` and `deleteObject` on
      item objects. On other objects: as now (stub or unknown).
    - Stubs, logged: `KQObject::tossItems` (smashed barrels need weapons,
      M5; its flight is not fully traced, items.md open question 3),
      `KQMonster::dropItem` (only `rockHit.cs`, M5), `KQObject::changeBmp`,
      `KQObject::setGroupsAlways`.
    - **Name of a new item object:** not in the note. Stand-in:
      `<type without INVITEM_>.<n>`, n counting from 1 per world
      (`Crystal.1`); asked as Q60.
    - **Clicking an item** (rules 6–8): picked like any clickable object
      by its box (the type's shape in `BOUNDS.KQ`, lower-cased; our
      reading of `chests.md`'s crystal check); type 0x220 (clickable).
      Out of reach → "too far" (logged, nothing); item in hand →
      "that doesn't work"; in reach → taken out of the world, added to
      Connor's inventory, pickup sound logged (no audio yet), hidden from
      the world; then its script (if any) runs with `<item> ConnorAction
      do do`. Items never collide with Connor (type 0x220 and mask 0x809
      share no bit with his type 4 and mask 0x281B, pair rule).
    Unit tests: make in the world and carried, `NewInvItem`, unknown
    type, `give`, `checkInventory` 1/0 and its variable, pickup in and
    out of reach, script after pickup with `setScript … none` removing it
    (`Tavcoin.cs`), hidden items not clickable.
    **Done.** Also: a world command can now set script variables (the
    runner asks the world for them after each command), and
    `setHidden`/`deleteObject` on objects other than items stay stubs.
    Our choices: a world item needs x y z (else error); `setScript` and
    `setDontLook` on other objects are "not tracked" (result 1).
  - **3e-3, items at the start and in the viewer** (`kq8-data`
    `world_script.rs`; `kq8-viewer` `load.rs`, new `item_view.rs`).
    The start-up runner runs `KQMonster::inventory`, `give`,
    `KQObject::setScript`, `setDontLook` and `setHidden` on items through
    its host and sets `NewInvItem`, so `WorldInv.cs`'s lines work
    (`KQObject::setScript $NewInvItem dagger.cs`). The viewer: every
    world item gets a model (its type's shape, loaded once per type; also
    the types chests and world scripts can make later), its
    translucency, z −1/−2 → floor / surface height, its turn; hidden ones
    are not drawn and can't be clicked. Clicks go through the same
    picking as doors (nearest box). On pickup the model disappears; when
    a script makes a new world item while playing, it appears. Log
    `kq8-artifacts/logs/items.log`: every item made (type, quantity,
    name, where, `NewInvItem`), every pickup (distance, Connor's count of
    that type before and after), the start count in the main log.
    **Done.** The `KQMonster::inventory` / `give` line reader is shared
    (`item::read_make`) by the start-up runner's host (`load.rs`) and the
    running game. Daventry: 22 items (21 in `WorldInv.cs`, the hidden
    `BirthdaySuit`), models for 25 types. Item models are looked up like
    objects' (this world, then `GAME/Common`), so a type whose model only
    another world has is not drawn (Gnome's `schroomC.kqs`, not needed at
    the start there).
  - **3e-4, chests** (`kq8-core` new `chest.rs`, `active.rs`;
    `kq8-viewer` `load.rs`, `door_view.rs`). A `KQChest`: its keys (rule
    1), default type 0x8021 unless `colliderType` (rule 2; today chests
    load with the plain default 0x41, which is not clickable), movements
    `closed`/`open` played as doors' are (the phase step and cues shared
    with `door.rs`; no `opened`: it stays on the last frame, rule 3),
    states closed / open / empty. Clicks (rules 4–10): reach, "too far"
    and "doesn't work" as doors; 1st click plays `open` and makes the
    item object at `invLoc`/`invDir` (or goes straight to empty if no
    type); 2nd click moves the remembered item into Connor's inventory
    (from the world, or already carried: only the state changes, the
    note's stand-in), then the `Take` remark if any; later clicks the
    `Empty` remark (logged, lines are M4). The chest's script is not run
    on clicks; it gets `Init`, `EndLoop <movement>` when a movement
    finishes, and `Cue`. Chests are drawn like doors (frame of the
    current movement). `KQObject::collisionType` works on chests
    (`wizchest.cs`). Unit tests: the three clicks, no item type, item
    already picked up, item in hand refused, out of reach, `EndLoop
    open` after 1 s.
    **Done.** In the viewer a chest is loaded and drawn as a door (the
    loader's door list with `DoorAssets::chest` set; same box lookup and
    per-frame models); the running game keeps chests apart
    (`ActiveChest`). Daventry: `smlchst3`, `smlchst4`, `wizchest1`
    (hidden, type 0x1).
  - **3e-4b (replaced by 3g-2 after Q63), clicks need the camera's line of sight (planned and done
    2026-10-13, MODLOG).** The user's check in the original (2026-10-13): an
    object can't be clicked through a wall or the ground, but can when
    Connor is behind a wall and the camera sees the object. This
    contradicts `doors.md` Q45 b ("walls, terrain and other objects do not
    hide a clickable one"); asked as Q63. Applies to every click (doors,
    chests, items). Stand-in until Q63 is answered: the mouse ray (it
    already starts at the camera) is tested against the camera's view
    obstacles (`camera.md` "Obstacles": the drawn ground; buildings;
    objects with collision type bit 0x8 or 0x1000, including doors as
    they stand); if one is hit nearer than the picked box, the click picks
    nothing. Doors and chests block only when they are not the one picked.
    Reach (2 × `attackDist` from Connor's feet) is unchanged. Code:
    `kq8-viewer` `door_click_input` gets the ground and view blockers it
    needs (the same ones `game_camera.rs` uses); `kq8-core` `active.rs`
    `pick` takes an "is the line blocked before t?" test. Unit test: a box
    behind a wall is not picked, in front of it is; the ground blocks.
    Log: a blocked click names what blocked it and at what distance.
    Viewer check: from the pub's ground floor the upstairs chest can't be
    clicked; standing upstairs it can.
    **Done** as planned (`Active::pick_seen`, `game_camera::view_blockers`
    shared with the camera), plus a test option `--aim NAME,N` (the
    mouse's ray, aimed by the program at the middle of NAME's box).
  - **3e-5, scripted chests and baskets** (`active.rs`, viewer). A
    clickable object with a script that is not a door, chest or item:
    clicking it in reach runs `<name> ConnorAction do do` (`chests.md`
    rule 11 says these chests work with "the script runner, `items.md`
    and the commands above"; the call is the same as a door's, rule 12);
    `KQAnimPlayEvent <movement>` to it plays that movement (as doors,
    rule 6), so `smlchst1`, `smlchst2` and `jar1` open and drop their
    coins, and the mushroom baskets (`give mushroom 5`) give. These
    objects are drawn like doors. `KQMonster::speak` is logged as
    Connor's remark (M4).
    **Done.** Which objects: placed `KQAnimObject`s with a script and the
    click bit whose movement is still at load (Daventry 14); those whose
    movement plays at load (signs, fish) stay with `anim_view` and are not
    clickable yet. The movement-playing part of chests moved to
    `kq8-core` `named_anim.rs` (`NamedAnimPlayer`; first called
    `movement.rs`, renamed for clarity), used by chests and these
    objects; doors keep their own. Our stand-ins for what the notes
    describe only for doors and chests (`Init`, `EndLoop`, the click
    call, `KQAnimPlayEvent`) are asked as Q64.
  - **What you'll see after 3e:** coins, mushrooms, crystals and the
    other start items lying where `WorldInv.cs` puts them; click one near
    Connor and it vanishes into his (invisible) inventory, logged. In the
    pub upstairs the big chest opens on the first click and a crystal
    appears in it; the second click takes it; the third logs "empty".

- **3f — clicking objects that move at load, and `deleteObject`
  (planned and done 2026-10-13, MODLOG).** The user's check in the original
  (2026-10-13): clicking a swimming fish makes it disappear, Connor makes
  an eating sound and gains health. `fishies.cs` (data) does it:
  `deleteObject $1`, `KQSound::play eat.aud`, `KQStatsChangeRequestEvent
  2 0 0 0 0` to Connor, once (`Daventry::fish::full`, shared by the three
  fish). Asked as part of Q64 (d).
  - **Which objects.** Every clickable placed object with a script that
    isn't a door, chest, item or 3e-5 scripted object: the ones that move
    at load (Daventry 10: signs, fish, dead birds, `starball`,
    `girltree`) and still ones without movements. They stay drawn as now
    (`anim_view` or the still models). None of Daventry's has
    whole-object steps, so the click box stays at the placement (an object
    with steps logs that its box doesn't follow; not needed yet).
  - **Clicks** (stand-in, Q64 a): as doors and 3e-5 objects (the shared
    `script_args`): `ConnorAction do do` in reach, item in hand by
    `acceptsInventory`, out of reach by `tooFarAwayAction`. `Init` at the
    start (Q64 c). `KQAnimPlayEvent` to them: logged, not done (their
    animation belongs to `anim_view`). `KQCueEvent` → `Cue`.
  - **`deleteObject <name>`** on such an object or a 3e-5 object
    (stand-in, Q64 d): gone for good: not drawn, not clickable, no
    collision, not a view blocker; events to it are dropped (logged). On
    other placed objects it stays a stub. The viewer gets a request:
    hides the object's entities (a name tag on every placed object's
    entities), turns its collider off (moved far below the world, size 0)
    and drops its view box.
  - **Stubs:** `KQSound::play` (no audio yet); the stats event stays
    "stats are M5".
  - **Code:** `kq8-core` `active.rs` (`Clickable` gets the script and the
    click keys, `gone`; `Request::Delete`); `kq8-viewer` `load.rs` (the
    keys for `Clickable`), `terrain_view.rs`/`anim_view.rs` (name tags),
    `door_view.rs` (applies deletions). Unit tests: a scripted clickable
    runs `ConnorAction`; a fish-like script deletes itself (gone, not
    picked, request sent, a later event dropped); one without a script is
    logged as before. Viewer check: `--click fishies.01,N`: the fish is
    gone, `eat.aud` and the stats event logged; clicking it again does
    nothing; Connor can walk through where it was.
    **Done** as planned. Every placed entity now carries the object's
    name (`terrain_view::ObjectName`) so a deletion can hide it; deletions
    are applied in `trap_step` (with the other script requests).

- **3g — the clicking and script answers (Q54, Q60–Q64) (planned and done
  2026-10-16).** A answered these 2026-10-15/16 (`clicking.md`,
  `doors.md` "Q45" e, `items.md` rules 7 and 9, `chests.md` rule 8); our
  stand-ins from 3e/3f/3r are replaced one at a time. Each sub-step has
  its own unit tests and MODLOG entry.
  - **3g-1 (done 2026-10-16), `test` leaves −1** (Q54, `doors.md` Q45 e): a false text `==`
    leaves `-1` when the left word comes first in byte order, `1`
    otherwise; every other false text test leaves `1`. Unit test
    `test abc == abd` → −1, `abd == abc` → 1, `abc < abb` → 1.
  - **3g-2 (done 2026-10-16), what the click line hits** (Q63, `clicking.md` rules 1–12).
    Replaces 3e-4b's stand-in. The click line is tested against every
    clickable shape (bit 0x20) only, nearest wins: the boxes we already
    test, **plus buildings** (type 0x28 by default, so clickable, unless
    `colliderType` says otherwise: Daventry's cave, church and wizhouse
    have 136 and are not). Buildings are tested with the BSP line test the
    camera already uses. **The ground no longer blocks**, and neither do
    objects without bit 0x20. A picked building with no script: nothing
    happens, logged "swallowed, no script" (rule 10; in reach or out:
    `tooFarAwayAction` 1). With an item in hand: "that doesn't work"
    (rule 12). Buildings with a script (none in Daventry) get the calls as
    doors do (logged only for now: no Daventry building has one).
    Hidden objects stay unclickable: with default masks hiding drops
    their collider (`objects.md` "Hidden objects" 3, `object-lighting.md`
    rule 41), which is what the click line tests. Code: `kq8-core`
    `active.rs` (`pick_seen` takes "the nearest clickable building hit"
    instead of the view blockers), viewer `door_view.rs` / `game_camera.rs`
    (a list of clickable buildings). Unit tests: a box behind a clickable
    wall is not picked and the wall is; behind a 136 building it is
    picked; the ground doesn't block. Log: what the line hit first (name,
    kind, distance).
  - **3g-3 (done 2026-10-16), accept lists** (Q62, `clicking.md` rules 13–18): split at
    commas; first word `all` / `INVITEM_All` (any capitals; anywhere in the
    list, our choice from "Do not reproduce") = accept everything; other
    words are type names looked up with `items.md` rule 2 until `none`;
    unknown names skipped and logged; `acceptAllItems` non-zero = the same
    switch; compare the held item's **type**. `KQObject::setAccepts`
    adds. Unit tests: list, `INVITEM_All`, `none`, unknown name, prefix
    and capitals, `setAccepts` adds.
  - **3g-4 (done 2026-10-16), `EndLoop` through the queue, doors too, `getEndLoop`** (Q64 c,
    `clicking.md` rule 21): a finished non-repeating movement sends the
    object a cue 3333 through the event queue (arrives the next update,
    after what is already waiting); a cue 3333 becomes `EndLoop
    <movement>`, 500 is "projectile, combat" (logged). Doors get
    `EndLoop open/close` after their `Open`/`Close`. `getEndLoop <from>
    <to> 1|0`: one listener per object; when `<from>`'s non-repeating
    movement ends, `<to>` gets `Cue 300 0 <movement>`. Connor as `<from>`:
    his movements (`push`) are not played by scripts yet, so it is
    recorded and logged only. Unit tests: `EndLoop` one update later,
    door `Open` then `EndLoop open`, `getEndLoop` relays, 0 stops it.
  - **3g-5 (done 2026-10-16), `deleteObject` for everything, `NoName<n>`, chest pickup
    sound** (`clicking.md` rule 22, `items.md` rules 7 and 9, Q60, Q61):
    `deleteObject` on doors, chests and items too (deleted at once: not
    clickable, not solid, not drawn), every listed name, unknown name →
    error line and `1`; new item objects are named `NoName<n>`, one
    counter from 0 for the whole run; a chest's 2nd click logs the type's
    pickup sound. Unit tests for each.
- **3h — sound effects (planned 2026-10-17).** Rules from
  `docs/behaviour/sound.md` only (numbers there are "rule N"). Three
  steps, each with its own test and MODLOG entry.
  - **Words used here.** A *sample* is one number of the sound wave; a
    sound file holds 22,050 of them per second (per channel). *Decoding*
    turns the game's packed bytes into those numbers. A *mixer* adds
    the sounds playing at the same moment into what the speakers get.
    *Loudness* is a factor from 0 (silent) to 1 (as recorded); *balance*
    (pan) from −1 (left) to +1 (right).
  - **3h-1 (done 2026-10-17), reading `.aud` files** (no sound card needed).
    `kq8-data` `aud.rs`: accepts a file as SOL or WAV exactly as rule 12
    says, else refuses it with a reason. SOL: data always from byte 14
    (rule 10), one byte per sample per channel, stereo bytes alternate
    left/right, each byte adds or (bit 0x80) subtracts a step from a
    128-entry table, each channel starting at 0, 16-bit wrap-round
    (rules 7–9). The table's numbers come from the community note
    `kq8aud.txt` (rule 7 says the game's table is the same); our code
    builds it from its pattern instead of copying the list. Uncompressed
    SOL (no game file) reads signed 16-bit (rule 9). WAV: only the
    fields needed (format chunk: tag 1, channels, rate, bits 8 or 16, the
    `data` chunk; "Do not reproduce" 6); 8-bit is unsigned, silence
    0x80. Output: rate, channels, samples as signed 16-bit, and counts
    of wrap-rounds. New command **`kq8 aud-verify [--wav <file>]`**:
    decodes every `.aud` under `MASK/` and every SOL file in the
    `English` folders (speech), logs one line per file and a summary
    compared with the note's numbers (1,750 `.aud`, 1,749 SOL + 1 WAV,
    rates, stereo count, `AXEDOOR.AUD` 11,847 samples, `FOREST.AUD`
    stereo 178.5 s, `NONE.AUD` 8 samples, `AMBDEAD.AUD` 187.6 s,
    header + data = file length, 0 wrap-rounds). `--wav <file>` also writes that
    one file, decoded, to `kq8-artifacts/wav/<name>.wav` to listen to.
    Never into the repository. **Done** as planned; all 16 checks match
    (the note's counts include the copies under `MASK/data/`; its 2,498
    decoded files are the loose SOL ones, 1,006 `.aud` + 1,492 speech).
  - **3h-2, playing the sounds the viewer already logs** (planned
    2026-10-17; **done** 2026-10-17, see the end of this item). **Library, chosen by the user (2026-10-17): `cpal`**
    (Apache-2.0, version 0.18.2), which only opens the sound card and asks
    for numbers, **plus our own small mixer**, so the note's loudness and
    balance rules are applied exactly and can be unit-tested without a
    sound card. Three parts:
    - `kq8-data` `aud.rs` `find`: rule 13's lookup. The name as given,
      then with its extension replaced by `.AUD` (added if it has none),
      then `.WAV`; each tried in the search list of
      `models-and-textures.md` ("Where texture names are looked up"):
      the world's loose `patch`, `8gui`, `resource`, `English`, `8bit`,
      `kq`, `sound` folders, then `GAME/patch` and the other `GAME`
      folders (their order inside `GAME` is not traced; only the `sound`
      folders hold `.aud` files, so it makes no difference). A name with
      a folder in it is looked up from the game folder.
    - `kq8-core` `sound.rs` (the rules, no sound card): the six groups,
      their slider values read from `MASK/Options.cs` (`setSfxVol`,
      `setSpeechVol`, `setMusicVol`; 1 before it runs; rules 50–52); a
      sound's loudness = own × group, at most 1 (rule 21); placed sounds:
      distance from Connor in 3D (rule 28), the original curve (rule 29,
      kept by the user's decision), balance from the camera's own axes
      (rule 31; 0 when the sound is straight above or below, "Do not
      reproduce" 4), the pan law (rule 23), all redone every frame (rule
      32); beyond max a one-shot is stopped (rule 30); the mix limit 8
      (rule 26: when more than 8 are playing, the oldest non-looping one of
      the lowest group stops first); finished one-shots leave the playing
      list on the 200 ms housekeeping step (rule 20); a missing or broken
      file is skipped and logged once per name ("Do not reproduce" 2). It
      gives the mixer plain orders (start, change loudness, pause, resume,
      stop) and writes every start and stop to the log.
    - New crate **`kq8-audio`** (the only one that uses `cpal`): the mixer
      (adds the playing sounds, each at its two channel loudnesses,
      converting 22,050 or 44,100 Hz to the card's rate by drawing a
      straight line between neighbouring samples, our method; rule 18 says
      any rate is fine), and the sound card output. If no sound card can
      be opened, the viewer goes on without sound and logs why.
    - Sources in 3h-2: door, chest and scripted-object `playsound` cues,
      placed one-shots 5,000 / 25,000 at the object's position (rules 35,
      49); item pickup, a plain one-shot (rule 42); `KQSound::play`, a
      plain one-shot (rule 40). **Not in 3h-2:** footsteps (Connor's
      animation cues are not run by our code yet; their own step later),
      the item land sound (nothing is thrown yet), scenery animations'
      cues (`anim.rs` does not run cues yet).
    - Viewer: `--sound on|off` (default on; `off` runs every rule and
      log line but opens no sound card). Log `kq8-artifacts/logs/sound.log`:
      groups at the start, every start (name, file found, plain/placed,
      position, Connor's distance, loudness, balance), every stop (why:
      finished, beyond max, mix limit), and for playing placed sounds a
      line whenever loudness or balance changed by 0.05 or more.
    - **Test:** unit tests for the lookup (`raven.snd` → `RAVEN.AUD`,
      `poffup.aud` not found), the curve (1.0 at 4,999, 0.75 at 5,000, 0.5
      at 10,000, 0 at 20,000, stopped beyond 25,000), the balance (beside
      the camera ±0.82, ahead 0), the pan law, the mix limit (12 one-shots:
      the first three stop as the 10th, 11th, 12th start), group loudness
      (a door next to Connor = 0.14); the mixer (pan law on samples,
      rate conversion length). Viewer: open a door with `--click`, read
      `sound.log`; then the user listens.
    - **Done** as planned. The lookup checks went into `kq8 aud-verify`
      (all match). Listener and camera axes are taken from the drawn
      camera (the game camera or, after H, the free one); with no Connor
      in the world, the camera is the listener. Checked by ear by the
      user (2026-10-17).
  - **3h-3, Daventry's looping sound objects** (`KQAudibleObject`,
    rules 43–48; **done** 2026-10-17): the six objects of the table in rule 44, looping placed
    sounds in group 0; on only while near/far (switched off beyond 27,100
    by the draw-distance test, restarted from the beginning when they come
    back), paused beyond max (or playing silently when started out of
    range, rule 30), stopped by `deleteObject` (rule 48).
    Code: `kq8-core` `sound.rs` (`Audible`), the loader (`load.rs`
    `place_audible`: the `.kq` keys, position as written, `deltaX/Y/Z`
    ignored), the viewer (`sound_view.rs`: each frame, before the placed
    sounds are redone; `deleteObject` seen through the script world's
    deleted list). The near/far/off test is measured from Connor only;
    `setActiveCamera` and `makeActive` (`object-lighting.md` rule 42) are
    not applied to sound objects yet (no Daventry script uses them on
    one). Only 5 of the 6 are loaded at the start: `Waterfall2Sound`
    comes from `bricks.cs` (`loadKQ watrfsn2.kq` when the waterfall is
    blocked), and `loadKQ` while playing is still a stub.
  - **3h-4, Connor's animation cues and footsteps (planned 2026-10-17;
    done 2026-10-17, as planned).**
    Asked for by the user after 3h-3. Two parts, one MODLOG entry.
    - **Connor's cues.** His movements now keep their `cueN=` lines
      (`Conner.anm`). After each of his updates, the cues between the
      phase before and after the update fire, by `doors.md` rule 9:
      [lower, higher). At the wrap of a forward loop: [old, 1.0001) then
      [0, new) (`objects.md` "Loops" rule 4); a backward loop's wrap
      (his `back`) mirrors it, [0, old) then [new, 1.0001) (stand-in,
      Q81 c). A transition cut short at its from point fires only up to
      there; the new movement starts at 0 the next update, so its cue at
      0.0 fires then. Action names ignore capitals (rule 34);
      `PlaySound` = a placed one-shot at Connor (rule 35), `FootStep` =
      below; `EndLoop`, `Do`, `SendEvent`, `Interruptable` are logged
      only (not built for Connor).
    - **Footsteps** (rules 36–39): a plain one-shot, group 0. The ground
      he stands on (this update's down-pass contact, else the last one,
      stand-in Q81 d): the terrain gives the square's `envInfo.kq` flags;
      a building gives 2ⁿ for `footsoundType=n` (n 0–8, other values
      0x1); a building without the key 0x1 (stand-in, Q81 a); an object
      the terrain flags under him (stand-in, Q81 b). **Updated
      2026-10-17** to A's answer (rule 39a): a building is remembered only
      from a building hit, any other hit clears it, and in the air it is
      kept 2 updates, then the terrain under him; (a) and (b) were right.
      The first match of
      0x1 dirt, 0x2 grass, 0x4 rock, 0x8 / 0x100 / 0x40 water, 0x20
      weirdb picks the set, none = no step (so `cav2.kq`'s 4 is silent,
      "Do not reproduce" 5, kept). File `<set>1/2/3` at 25 / 50 / 25 %,
      from a small random number generator of ours (fixed start, so test
      runs repeat). Found by rule 13 (`dirt1` → `DIRT1.AUD`).
    - Logs: `connor.log` gets a line per cue fired; `sound.log` the start
      with what he stood on, the flags, the set and the roll.
    - **Test:** unit tests for the cue range (forward, wrap, cue at 0.0
      and 1.0, backward, backward wrap), the set order, the 25/50/25
      split, `footsoundType`. Viewer: walk on grass, on the bridge
      (`bigbrig2`, rock) and in a house; count steps in `sound.log`
      (walk: 2 per cycle); the user listens.
  - **3h-5, world music** (`KQMusic`, rules 33, 57–57l; planned
    2026-10-17, after A answered Q83; **done** 2026-10-17, as planned,
    tested with the wizard's trap `mwiztrap`: the church's trap can't be
    left yet, its only way out is the rope). One MODLOG entry.
    - **Fades** in `kq8-core` `sound.rs`: each playing sound gets a
      *level* of its own (1 unless faded), multiplied into its loudness
      (rule 21). A fade moves the level towards a target by 0.75 per
      second in the 200 ms housekeeping steps (0.15 a step, so 0 → 1 in
      7 steps), stops exactly at the target, waits while the sound is
      paused (rule 57g), and a fade that reaches 0 stops the sound
      (rule 33). A sound can start at level 0.
    - **The tune list**, new `kq8-core` `music.rs` (`Music`): the tunes
      from `music.kq` (`music0` at the bottom, rule 57); each tune a
      looping plain sound in group 5 (rule 57a); names compared ignoring
      capitals. World start: the top starts at full loudness, no fade
      (57b). `push`: nothing if the name is anywhere on the list (57c 1);
      else the old top fades out (and stays on the list, stopped at 0),
      the new one starts at once at 0 and fades in, or at full loudness
      if the list was empty (57c). `pop`: not on the list, nothing; the
      top fades out and leaves the list, the one below fades in from its
      beginning, or, if it is still fading out, from where it is (57d 2);
      the only one: silence (57d 3); a lower one leaves silently, its fade
      cut off (57d 4). `play`: the top fades out and leaves, the new one
      fades in, no name check (57e).
    - **Scripts** (`active.rs`): `KQMusic::push` / `pop` go into the event
      queue, done **0.2 s of game time later** (3 updates), in the order
      sent; `play` at once (57f). A missing file name logs `Usage:
      KQMusic::push music_file`; `KQMusic::create` / `delete` log "is
      obsolete, use music.kq instead". Handed to the viewer like sounds.
    - **Viewer**: `load.rs` reads the `KQMusic` object's `nMusic` and
      `music<n>` keys; `sound_view.rs` starts the top on the first frame
      and applies the requests. No music object: requests are ignored
      (logged in `sound.log`; the original says nothing).
    - **Not in 3h-5:** pause (57h; there is no pause yet), videos (57i;
      none played yet), leaving a world and coming back (57j; one world
      per run), Barren's lava-tile pushes from the game itself (57k; that
      puzzle isn't built).
    - **Test:** unit tests: a fade's steps (0.15 per step, 7 steps, stop
      at 0), push / pop / play cases of 57c–57e (already on the list,
      crossfade, pop of the top, of a lower one, of the only one, pop
      while the one below is still fading out), the 0.2 s delay and
      order. Viewer: `sound.log` shows `forest.aud` at the start, at
      0.140; walk into the church trap and out (`chrchtrp`): `mchurch.aud`
      fades in over about 1.4 s, then out, and `forest.aud` starts again
      from its beginning. The user listens.
  - Not in 3h: speech (rule 58), ducking (rule 53), pause (rules 54,
    57h), hurt sounds (rule 41, need fighting).
- **Fix: a jump that hits something, from A's Q82 answer** (planned
  2026-10-17; replaces the Q82 stand-in; **done** 2026-10-17, as
  planned). Source: `connor-physics.md`
  "Jumping" rule 5 and "Walls" (corrected 2026-10-17). One step, one
  MODLOG entry.
  - **The rule:** during `jump` or `runJump`, a steep hit (surface
    normal z under 0.8) ends the jump (gravity on, steps 325). Then:
    - a **down-pass** hit → `crash` (rule 5, first bullet);
    - an **up, across or slide** hit → `crash` **only if the head-on
      test said head-on** for that hit. During a jump every steep
      surface is head-on (so both `jump` and `runJump` crash, they are
      in the crash list), except against a `KQMonster` or while the
      not-traced Connor flag is set;
    - `backJump`: unchanged (no head-on, slides, plays on).
  - **What changes in the code** (`kq8-core/src/connor_update.rs`): the
    stand-in "every non-back jump hit crashes" becomes the decision
    above, made from the hit's own record (its pass and its head-on
    result) instead of "any steep hit". The "stand-in Q82" wording goes;
    the log line says *why* it crashed ("down pass" or "head-on").
  - **Not reachable today, logged if it happens:** a steep up/across hit
    that is *not* head-on (a `KQMonster` or the flag: neither exists in
    our code yet). Rule 5 says the jump ends; what follows is not said,
    so it is asked as **Q84 (a)**; until then it is handled like a
    `backJump` hit (gravity on, the animation plays on, then Fall) and
    logged as a stand-in.
  - **Not changed, asked as Q84 (b):** rule 5 says that at a crash
    "turning stays allowed, and a turn key still held is passed on
    again"; our `crash` ignores the keys (as "Walls" says for the run
    crash). Which is right is not clear to B.
  - **Test:** unit tests: standing jump into a wall → crash with
    "head-on" in the note; running jump → crash; steep hill on the
    down pass → crash; a glancing (non-head-on) steep hit in a jump →
    no crash (built with a test world, since nothing in Daventry gives
    one). Viewer: the tree at Daventry's start (`--jump 10`),
    `connor.log`.
- **3i — animated objects controlled by scripts, pushing, the bricks
  puzzle (planned 2026-10-17).** Rules from `anim-scripts.md` and
  `pushing.md`. Goal: Daventry's bricks puzzle works from start to end
  (`pushing.md` rule 11). Each sub-step has its own unit tests, viewer
  check and MODLOG entry, in this order.
  - **Words used here.** *Waiting movement*: a movement a script asked
    for that starts as soon as the rules allow (`anim-scripts.md` rule
    4). *Can be cut short*: the 6th field of a `loop=` line, 1 = a new
    movement may replace it at once. *Has ended*: a play-once movement
    reached its end. *Pushable*: an object whose type has bit 0x100.
  - **Where things stand (2026-10-17).** Animated objects live in three
    places: doors (`door.rs`), chests and the clickable scripted objects
    such as `Bricks` (`named_anim.rs`, run by `active.rs`), and every
    other animated object (`anim.rs`, run and drawn by the viewer's
    `anim_view.rs`: `water`, `waterfall`, torches, the windmill...).
    Scripts reach only the first two kinds. Connor has modes Free, Fall,
    Land, Jump, Crash, Dying, Dead; no Push or Special. `loadKQ`, `move`,
    `handsOff`, `setThirdPerson`, `KQCamera::follow`,
    `KQMonster::setState`, `pauseAnim` are stubs while playing.
  - **3i-1, the waiting movement and `KQAnimPlayEvent`** (rules 1–10),
    `kq8-core` (**done** 2026-10-17, as planned; `Door` now holds a
    `NamedAnimPlayer`). One movement player for doors, chests and scripted
    objects (the shared part of `door.rs` and `named_anim.rs`), with:
    current movement name (as asked, even when unknown), the waiting
    slot, "can be cut short" from the `loop=` line's 6th field (new
    field on `DoorMovement`), "has ended" (phase ≥ 0.9999 forwards, ≤ 0
    backwards). `KQAnimPlayEvent`: waiting slot filled, then start at
    once if the running animation can be cut short or has ended, else
    wait (rule 4); unknown name → movement 0, logged (rule 5); a
    request to a stopped `0 0` movement waits forever, logged once
    (rule 6); at the end of a play-once movement the waiting one starts
    first, then `EndLoop`/`Cue 300` name the ended one (rule 8).
    Transitions (rule 4.2): objects have none (data), so not built;
    logged if a `transitionN=` line is ever found on an object. The door
    rules on top (`open`/`close` → state, `opened`/`closed`, `Open`/
    `Close`) stay. Objects run by `anim_view` (`water`) are **not**
    reached in this step: logged "not run by the script rules yet"
    (3i-4).
    - **Log** (`doors.log`): per request: object, asked name, current
      movement, "at once" / "waiting" / "waiting forever (rule 6)",
      start phase and speed; per end: ended movement, the waiting one
      started (if any), then `EndLoop` / `Cue 300`.
    - **Test:** unit tests: cut-short starts at once; not cut-short waits
      for the end then starts before `EndLoop`; two requests keep the
      second; `0 0` waits forever; unknown name → movement 0 with the
      name kept; a request for the playing movement restarts it.
      Viewer: the bricks' end (`Bricks` `stop` is `1 1` so `down` starts
      at once) can only be reached in 3k; until then the chests
      (`--click smlchst3,30`) must behave exactly as before (compare
      `doors.log`).
  - **3i-2, `KQAnimOnce` and `setDeleteWhenDone`** (rules 11–13), core:
    a flag on the object; at its end its script gets `Die`, its listener
    `Cue 300`, then deletion if asked; no `EndLoop`, no waiting
    movement. (**Done** 2026-10-17 in core: `NamedAnimPlayer::once`,
    `ActiveObject::delete_when_done`, `setDeleteWhenDone`. Which objects
    get the flag is set where `KQAnimOnce` objects are made: 3k.) `boardx.kq` loading itself: stop after one round, logged
    ("Do not reproduce"). Unit tests only: in Daventry `KQAnimOnce`
    objects are only made by `loadKQ` from a fight (M5), and `loadKQ`
    while playing comes in 3k.
  - **3i-3, `pauseAnim` and `setAnimPos`** (rules 14–15), core: toggle;
    while paused no phase change, no cues, no end, no waiting start;
    prints `<name>, <phase>`; unknown name prints the note's line. Unit
    tests. (Daventry's only use is the monster `sprdump`, M5.) (**Done**
    2026-10-17: doors, chests and scripted objects; `pauseAnim` is no
    longer a stub.)
  - **3i-4, scripts reach every animated object** (rules 1–10 for the
    objects `anim_view` plays now). **Architecture change, asked first
    (AGENTS.md "Ask before large refactors").** B's proposal: the rules
    and the state of every animated object live in `kq8-core` (`Active`
    gets them all, with all their movements, the random start of
    `flags` bit 8, and the repeating frame rule of 3c); `anim_view`
    only draws what core says (movement, frame, pose), building frames
    for every movement an object has. Needed for `water` →
    `blocked` in the bricks puzzle. Test: `anim.log` for the tracked
    objects must stay the same as before (windmill, well water, torch).
    (**Approved by the user and done** 2026-10-17: `Active` gets `anims`
    (`AnimObject`), `anim_view` draws what core says; objects with more
    than one movement join even when still. The repeating frame rule of
    3c and the cue ranges at a wrap (`objects.md` "Loops" rule 4) moved
    into `DoorMovement`. Rules 3 and `flags` bit 8 meet on six objects:
    asked as Q85, stand-in: the random start is kept, and the
    first-update start uses up that update.)
  - **3i-5, the script reader** (rules 17–18), core `script.rs`:
    single quotes switch `$` replacement off until the next one (marks
    dropped, words still split at spaces); `;` ends a command; `( … )`
    one word without the brackets; more than 40 words → `Console: Too
    many arguments`, the command fails. Unit tests. (**Done** 2026-10-17
    for scripts run while playing; the start-up follower in `kq8-data`
    `world_script.rs` is unchanged: it only follows `loadKQ`, `set` and
    `move`, none of which uses these.)
  - **3i-6, `KQStatsChangeRequestEvent` adds** (rule 19): five stats per
    object (start 0; where objects' starting stats come from is not in
    the note, so 0 and logged), added, the fourth capped at 100, health
    below 0 → script gets `Die`. Connor: added, then his level rules
    (fifth = experience). Starting values of Connor's stats are not in
    any note: asked as a question, logged as a stand-in. Unit tests;
    viewer: clicking a fish (`--click fishies.01,N`) logs Connor's
    health going up by 2. (**Done** 2026-10-17, new `kq8-core`
    `stats.rs`. Connor's start is in `Connor.kq` after all: `stats=10 10
    0 0 0`, `level=1`. Open details asked as Q86 with stand-ins.)
  - **3i-7, Connor's Special state** (rule 16): new mode `Special` in
    `connor.rs`: the item list one at a time (`AnimList=`, `AnimListOn`,
    `AnimListOff`, `KillConnor`, `Inv` logged only, any other word a
    movement that must end before the next item), then back to standing.
    The extra `.anm` lists (`bricks.anm`, `c_pshoff.anm`, ...) are read
    from the world's `kq.vol` like `Conner.anm`, and their models loaded
    by `connor_view` when first asked (logged). A movement ending tells
    the `getEndLoop` listener `Cue 300 0 <movement>` (3g-4 recorded it
    for Connor but nothing played). Test: unit tests on the list;
    viewer in 3k. (**Done** 2026-10-17. The movements are found in the
    world's scripts and loaded at the start (Daventry: 87, from 13 lists
    and his own `Conner.anm`; loading 0.5 s longer). An item that names a
    movement not loaded is skipped and logged (stand-in; the note doesn't
    say). `KillConnor`: our Dead mode (stand-in). New test option
    `--special ITEMS,N` (items separated by `;`), as if a script asked at
    update N.)
  - **3j, pushing** (`pushing.md`). Details worked out 2026-10-17 after
    reading the mover:
    - **Colliders learn their kind.** `BoxCollider` gets the object's
      `colliderType`, `colliderMask` and whether it is a ball stack
      (`collisionShape=1`); doors, chests and scripted objects refresh
      theirs every update from `Active` (as their placement now), so a
      script's `KQObject::collisionType` reaches Connor's collision.
    - **Where the check happens.** In the mover's across pass (and the
      slides of that pass, not the down pass's slide): a hit on a
      collider with bit 0x100 goes through rule 2. What the mover can't
      know is passed in the walker's wish: the walker accepts Push
      (mode Free or Push), already pushing, his facing, his chosen
      movement, empty hands (hand mode 4 always: `setDoMode` is a stub).
      "On the ground" (rule 2.2): the mover is in its normal state (not
      jumping or falling).
    - **The Push state** in `connor.rs`: mode `Push`; the movement
      chosen is `push` instead of the keys' choice (asked as any
      movement, so `pushright` / `pushleft` lead back to `stop`); the
      counter of rule 7; turning as when walking (Q87 d stand-in).
      `push`, `pushleft`, `pushright` are added to the movements loaded
      from `Conner.anm`.
    - **Physical push** (rules 4–5): the mover asks the scene to move
      the object by his across step; the scene sweeps the object's own
      ball stack against buildings and the other colliders its mask
      makes obstacles (bits 0x100, 0x8, 0x1, 0x2, 0x2000 of the other's
      type; Connor's bit 4 taken out), stops at the first contact (Q87 b
      stand-in), applies the ground rule (Q87 c stand-in) and puts it
      back if its height changed; Connor goes to the pass start + 0.95 ×
      the movement. The viewer then moves the object's collider and its
      drawn entities. Box-shaped pushables (the barrels) are not moved
      (Q87 a), logged.
    - **Scripted push** (rules 3, 9–10): the touch is not a hit; the
      object is passed through for the rest of that pass, and its script
      gets `Push` **right after Connor's move in the same update** (our
      order: the scripts run from `Active`, which Connor's move can't
      reach; the note says mid-move, and what follows a mid-move
      `move Connor` is not traced anyway). If the object is still a
      scripted pushable after its script ran, its later touches are wall
      hits, logged ("Do not reproduce").
    - **3j-1, the push check and the Push state** (rules 1–3, 6–8;
      **done** 2026-10-17. Until 3j-2/3j-3 an accepted touch acts as on
      an object that doesn't move: rule 4.3, he stays at the start of
      the pass. Rule 2.6's "standing state" changes only the state, not
      the animation, so `pushright`/`pushleft` play when he lets go):
      Connor's touch with a 0x100 object goes through the six conditions
      of rule 2 (hand mode: empty hands, 4, at the start; `setDoMode`
      stays a stub), then mode `Push`, `push` movement, the counter of
      rule 7. Log per touch: object, each condition, facing cos, state
      change.
    - **3j-2, physical pushables** (rules 4–5; **done** 2026-10-17 for
      ball-stack objects, the hay bale; box-shaped ones wait for Q87 a): the object is swept by
      Connor's across step over 1/15 s ignoring Connor, put back if its
      height changed at all, Connor at start + 0.95 × its movement.
      The pushed object's collider and its drawn model move. Test: the
      hay bale in the windmill (`haybail.01`, `pushing.md` "Data used"):
      `--at` next to it, `--hold 1,0,0`; the log shows it sliding at
      about half walking speed and stopping at a wall; a barrel on a
      slope doesn't move.
    - **3j-3, scripted pushables** (rules 3, 9–10; **done** 2026-10-17.
      At the bricks Connor touches `Table` and `Bricks` at exactly the
      same distance: asked as Q88; stand-in: a pushable wins a tie, and
      after it is passed through the table still stops him): the touch calls the
      object's script with `Push` (`$1` its name) at once, mid-move, and
      is not a hit; "a scripted pushable whose script does nothing":
      treated as a wall hit and logged ("Do not reproduce").
  - **3k, the bricks puzzle end to end** (`pushing.md` rule 11). Needs,
    besides 3i and 3j, the commands `bricks.cs` uses while playing:
    `move` (Connor, `KQCamera`, an object; `objects.md` (d)),
    `KQCamera::follow` (`camera.md` rule 1), `handsOff`,
    `setThirdPerson`, `KQObject::setHidden` on `Bricks` and `Table`,
    `loadKQ waterfa2.kq` / `watrfsn2.kq` while playing,
    `deleteObject WaterfallSound` (`sound.md` rule 48). Not in any note:
    `handsOff`, `setThirdPerson`, the camera's view from `move KQCamera`
    angles, what Connor's move does after a script moved him mid-move,
    `loadKQ` while playing: asked as questions before 3k is built.
    Asked 2026-10-17 as Q89 (`handsOff`, `setThirdPerson`, `move
    KQCamera` angles) and Q90 (`loadKQ` while playing), with stand-ins.
    Sub-steps, each with its own test and MODLOG entry:
    - **3k-1, `move` while playing** (**done** 2026-10-17; a plain
      object's drawn turn is not changed yet, logged) (`objects.md` "(d)", Q21): flags
      0–3 (absolute / relative position and turn; relative position along
      the world axes); set directly, no collision passes. Connor
      (position and heading; the mover's state is kept), doors, chests,
      scripted and animated objects (their placement), plain placed
      objects (their colliders and drawn parts). `KQCamera` in 3k-3.
    - **3k-2, `KQObject::setHidden` on plain placed objects** (**done**
      2026-10-17 through the draw-distance test; an object hidden in its
      file and shown by a script stays invisible: not done, logged) (`objects.md`
      "Hidden objects" rule 3: not drawn, can't be touched): the `Table`.
    - **3k-3, the camera and controls** (**done** 2026-10-17; `move
      KQCamera` keeps looking at its last aim point): `KQCamera::follow none|Connor`
      (`camera.md` rule 1: following none, the camera stays where it
      is); `move KQCamera` sets its position only (Q89 c stand-in);
      `setThirdPerson` logged (Q89 b); `handsOff 1|0`: movement keys,
      Jump and clicks ignored while on (Q89 a stand-in).
    - **3k-4, `loadKQ` while playing** (**done** 2026-10-17; Daventry has
      17 prepared copies; a copy that comes in is not clickable yet: the
      H-key copies get no collider) (Q90 stand-in: as at the start, no
      call, a sound starts at once): files named in the scripts but not
      loaded at the start are already prepared, hidden ("unused" copies,
      M1); `loadKQ <file>` makes that copy live: drawn, known to the
      scripts (animated, clickable), its sound playing. Only a file with
      one prepared copy and no name or position on the line (the bricks'
      case); others logged as not done.
    Test: `--at 156625,303748,1.57 --hold 1,0,0` (the game's own
    `alias bricks`); the logs show the Push call, Connor's `push` from
    `bricks.anm`, `Cue 300 0 push`, `Bricks` → `down` at once with no
    `EndLoop` after, the waterfall swapped, `water` → `blocked`, the
    cues 5 and 10, controls back; then Connor can walk through where
    the waterfall stood.
  - **3l — the answers to Q84–Q93 (planned 2026-10-18; done
    2026-10-18, 3l-1 to 3l-7; Q94 asked in 3l-2).** Study session
    A answered B's questions from 3i–3k (2026-10-17) and the user checked
    some in the original (2026-10-18). Each sub-step fixes what a
    stand-in got wrong; each has its own log check, MODLOG entry and
    commit. Order: most visible first.
    - **3l-1, the pop-up at the bricks** (**done** 2026-10-18; Q92; `pushing.md` rule 11,
      `objects.md` "(d)" refined, `anim-scripts.md` rule 16). The `Push`
      script runs inside Connor's update, so after its `move Connor …
      15000` the rest of that update (a gravity step and the down pass
      with its 325 step-down) still runs and he lands on the floor
      (14,837) in the same update. Ours: the script runs right after his
      update (`trap_step`), so he was drawn at 15,000 and fell.
      **How:** the requests a `Push` call makes are marked "inside his
      update"; a `move Connor` among them is followed at once by the
      rest of his update: the move step's own vertical bookkeeping (one
      gravity step, the terrain raise) and then the mover's down pass and
      ground check (new `Mover::after_move`, `rest_of_update`), with the
      same scene his update used. A `move Connor` from anywhere else (a
      timed cue, outside his update) gets only the move step's
      bookkeeping (`objects.md` "(d)": one gravity step if gravity is on,
      the terrain raise), so he is drawn a little below the given height
      and falls from there (cue 10 of the bricks). Log: a `connor.log`
      line for each scripted move with the height before and after the
      rest of the update. Test: the bricks run; `connor.log` from the
      push to the end of `push`: z never 15,000.
    - **3l-2, the watermill cutscene** (**done** 2026-10-18; Q94 asked
      for the head-on case; Q93; `anim-scripts.md` rules
      24–27, `connor-movement.md` "Turning" rule 2). New in `kq8-core`
      `connor.rs`: an automatic turn (3.5 rad/s, the shorter way, stops
      exactly once the rest is under one update's turn; any player turn
      cancels it) and a scripted walk: `KQConner::walkTo x y <who> <c>
      <d>` / `runTo` (handsOff on, Wait, turn to face, then Move straight
      ahead walking or running; the flat distance is fixed at the start
      of the walk; stops when he has gone that far, back to Wait, cue);
      the cue also comes at a head-on hit; `pointAndCue x y z <who> <c>
      <d>` (turn, then cue; at once if already facing); `walkExit` ends a
      walk with its cue. `active.rs`: the commands become requests; the
      viewer sends `KQCueEvent c d` to `<who>` when Connor reports the
      end. Log (`connor.log`, `traps.log`): each start, the turn's end,
      the distance, the stop, the cue. Test: walk into the watermill room
      (`--at` at the trap `walktp1`): cues 1–6 of `wtrmwalk.cs` in order
      and `handsOff 0` at cue 6, about 4.2 s after the turn of cue 2.
    - **3l-3, cutscene control** (**done** 2026-10-18; Q89; `anim-scripts.md` rules 21–23,
      `camera.md` "Shake and cutscene cameras", "What the camera looks
      at" rule 1, "First person" rule 7). `handsOff 1`: also the run key
      and the mouse-look button; the camera's fixed mode (F11) goes back
      to normal; key presses swallowed, releases counted; `handsOff 0`:
      every movement key still held is sent again as a press (which also
      clears wall blocks as a press does). `handsOff 1 N`: N logged
      only. `move KQCamera`: the angles set the view (rx tilt, rz
      heading from +Y, positive anticlockwise; ry roll logged, not
      drawn); flags bit 2 adds them to the current view. `follow Connor`
      starts from where the camera is: pulled in or pushed out to the
      range at once; if its view heading is more than 0.4 rad off Connor
      it turns toward him by 1/3 of the rest per update, snapping under
      0.063 rad (one snap on both sides, "Do not reproduce"); tilt at
      once. `setThirdPerson`: the paired toggle's variable
      (`<World>::SetThirdPerson::WasFirstPerson`) is kept and the
      first-person key's lock logged (we have no first person). Test:
      the bricks run (`camera.log`: the view heading 0.9 during the
      shot, the turn back after cue 10) and a held key through
      `handsOff 0` (`connor.log`).
    - **3l-4, pushing** (**done** 2026-10-18; Q87, Q88, Q91; `pushing.md` rules 2.5,
      4.1–4.3, `connor-physics.md` "Against a building" step 7). In the
      mover's sweep, every object touched is told at once (the push check
      runs there, in the order the objects entered the world); a contact
      replaces the kept one when its distance is ≤ the kept one's (tie:
      the later-tested wins; the "pushable first" stand-in goes); an
      accepted push is not recorded. Box-shaped pushables (no
      `collisionShape`: the barrels) move by Connor's step and are never
      stopped; ball-stack ones (the hay bale) slide like Connor (up to 3
      retries, no head-on stop; normal z between 0 and 0.7 counts as a
      vertical wall); after the move the ground only raises (with
      `keepOnGround`); any height change puts it back. Test: a barrel
      against the pub wall goes through it; uphill it stays; the hay bale
      at an angle slides along the windmill wall; the bricks still start
      (`connor.log`).
    - **3l-5, turning during `crash`** (**done** 2026-10-18; Q84; `connor-physics.md`
      "Jumping" rule 5, "Walls"). Turn keys turn him at 1.75 rad/s while
      `crash` plays (run crash and both jump crashes); forward, back and
      run wait. Still no turning from a jump's start to its landing's
      end. Q84 a: a steep hit that isn't head-on (a `backJump` hit, or the
      cases our code can't reach yet) does not end the jump: gravity
      stays off and steps stay 0 until the animation ends, then Fall as
      usual (ours switched gravity on). Test: `--hold 0,0,1` run into a
      wall with `--turn` during the crash (`connor.log`: heading changes
      by 0.117 per update during `crash`).
    - **3l-6, animated objects and stats** (**done** 2026-10-18; Q85, Q86; `anim-scripts.md`
      rules 3, 19–20, `objects.md` "Animated objects at load" rule 4).
      First update of an animated object: (1) empty current name and a
      waiting movement → that one starts plainly (no random part); (2)
      else the current movement starts with `animPos` and, with `flags`
      bit 8, the random frame and a random speed that replaces the
      `loop=` speed; (3) the phase moves on in that same update. So the
      six torches/fish/birds start in step. Levels: strictly more than
      the threshold, once per event, health capped at the old level then
      a full heal on level up; second stat +5, third +2. `Die` always
      gets `$3` (the last weapon-hit kind, `Normal` if never hit). Test:
      unit tests; `anim.log` (the six objects at phase 0, speed 1);
      `doors.log` at the bricks (+50: level 2, 40 left, health 20).
    - **3l-7, `loadKQ` while playing** (**done** 2026-10-18; Q90; `objects.md` "Where a
      world's objects come from" rule 6, `clicking.md` rule 22): the new
      object's script gets `Init` at once, during the `loadKQ` line; a
      name still used by a live object (or deleted in the same event):
      error logged, the load refused (the original stops the game). Test:
      unit tests; the bricks run (`doors.log`: `Init` for `waterfall2`'s
      script if it has one).
    - Not in 3l: first person (none yet), `follow <other object>`,
      `KQCamera` cut/fly lists (`addCamLoc`, `flyToCamLoc`; logged), the
      mask icon and GUI panels (no GUI yet).
  - **3m — the answers to Q94–Q96 and the camera flights (planned
    2026-10-18; done 2026-10-18, 3m-1 to 3m-4; Q97 asked in 3m-4).** Study session A answered Q94–Q96 and traced the
    camera flights (2026-10-18). Four sub-steps, each with its own log
    check, MODLOG entry and commit. Order: what blocks play first.
    - **3m-1, the windmill tumble: `KQConner::endJump`, `JumpDir`**
      (**done** 2026-10-18; Q96; `anim-scripts.md` rules 28–30, `connor-physics.md`
      "Jumping" rule 4). The viewer tells the scripts, before they run,
      whether Connor is in the Jump state and with which jump
      (`WorldState::connor_jump`). `JumpDir`: 1 in the Jump state with
      `backJump`, 0 otherwise (outside a jump also a console line).
      `endJump`: outside the Jump state a console line and 0; in it a
      request that does what a crash does to the jump (gravity on,
      steps 325, the animation cut short, the jump marked finished) with
      no state change, and 1. A state request (`setState … special`, a
      scripted walk) while a jump is not finished: with `handsOff` on
      it finishes the jump the same way first; without it, it is
      refused and tried again at each later update (STAND-IN: when the
      original tries again was not traced; logged). Once Special
      replaces Jump, the jump's end no longer starts Fall. The trap's
      `On` script runs inside Connor's move (rule 30), so its `move
      Connor` gets the rest of his update, as a `Push` call's does
      (3l-1). Test: the user's windmill jump; `connor.log`: state
      `special` with gravity on from the trap's update; after cue 300
      (33 updates later) z 8050 → 7650; then `--jump` works again.
    - **3m-2, a push and a wall in the same sweep** (**done** 2026-10-18; Q95; `pushing.md`
      rule 4.3, `connor-physics.md` "Against a building" step 7). The
      push no longer ends the across pass: when the pushed object's
      turn in the list comes, Connor's position **and his sweep**
      become P (pass start + 0.95 × the object's movement); the objects
      later in the list see a zero-length move (STAND-IN: they test
      nothing, as buildings under 0.1; not traced; logged). A wall
      contact kept from earlier in the list is then handled with wanted
      end P and no movement: never head-on; P is projected onto the
      wall's plane through the contact and he slides there from the
      contact (the usual slide; a push inside it moves the object by
      the slide's step, his end = the slide's start + 0.95 × that).
      Test: push a barrel into the pub; `connor.log`: "push, then the
      kept wall hit", 225 from the face, then Push → Move → `Stop` with
      the forward block.
    - **3m-3, scripted walks** (**done** 2026-10-18; Q94; `anim-scripts.md` rules 24–25, rule
      24 corrected). During the automatic turn (counted per update):
      update 1 `Stop`; from update 2 (no wall block) the turn direction
      counts as a turn key for the animation (`Left`/`Right`; the
      walker's own key turning stays off, the automatic turn turns
      him); one update later `Walk` (`Run` for `runTo`) with the start
      and distance recorded; when the turn ends Move starts afresh and
      the distance is measured again. `pointAndCue`: `Left`/`Right`
      until the turn ends, then nothing asks (Wait). A wall block on at
      the start: `Stop` through the turn, the walk still starts (a
      scripted walk's choice ignores the blocks), the block stays on
      after. `walkTo x y <who>` without both cue numbers: a console
      line, the walk, no cue. Test: the watermill run; `connor.log`:
      `stop`, `left`, then `walk` during the turn; "measured again";
      arrival at (190800, 230100).
    - **3m-4, camera flights and cuts** (**done** 2026-10-18; Q97 asked; `camera.md` "Camera flights and
      cuts" rules 1–7, "Edge cases", "Do not reproduce"). New
      `kq8-core/src/camera_flight.rs`: the spot list (`addCamLoc x y z
      [newlist]`), the spot nearest the camera, the flight (N =
      whole(seconds × 15) + 1 updates, or distance ÷ 5,000 per second;
      equal steps on a straight line, roll in equal steps to `endRoll`),
      the aim (1,400 above the aim point; tilt −0.2 + the angle up to it
      each update; heading: locked within 0.02, else 1/3 of the rest per
      update with one snap angle 0.063 on both sides), the cut (at once,
      one follow update), and the pull-in on update N ("places" back
      on: the range around the aim point, along the line to it). The
      viewer's game camera gets the "places"/"aims" switches (`follow
      Connor` both on, `follow none` both off), runs a flight once per
      update from the update after the command (the note's cue timing),
      and stays still after it. An empty list: nothing, an
      error logged. STAND-IN (not settled): the height on update N: the
      height rule is not applied on that update (logged with the height
      it would want). Roll is logged, not drawn (as for `move
      KQCamera`). Test: the watermill run; `camera.log`: N = 31 from cue
      4, the same step every update, heading 2.153 and tilt +0.016 once
      locked, update 31 at about (188999.0, 229341.5, 10863), then still
      until cue 6; the mill wheel (`wheeltp1.cs`): N = 46.
  - **3n — the answers to Q97–Q99: when the camera updates (planned
    2026-10-18; done 2026-10-18, 3n-1 to 3n-5; Q100, Q101 asked).** Study session A answered Q97–Q99 (`camera.md` "Height
    above the ground" rule 2, "When the camera updates at all", "`move
    KQCamera` while following", "Camera flights and cuts" rules 3, 4, 7;
    `anim-scripts.md` rules 24–25). One commit per step. A stand-in
    added in 3n-1: one camera update on the first tick (Daventry's
    start-up script says `follow none`; Q27 d).
    - **3n-1, the camera updates only when it should** (**done** 2026-10-18). Before, the follow
      camera runs every tick (Q53 d). From now on `camera_step` still
      runs every tick, but it runs a camera update only for a cause:
      (1) **Connor moved**: his position changed by more than 0.1 on an
      axis since the last tick, or his facing changed (cos or sin of
      the heading by more than 0.0001); a script's `move Connor` is
      labelled as such in the log; (2) **`follow`**: one update right
      after the command, with the aim point counting as not moved;
      (3) **the "turn back" switch**: every tick while it is on; `follow`
      turns it on when the view is more than 0.4 rad off Connor, a later
      `follow` that finds the camera farther than the maximum turns it
      off, and so does the end of a flight (both logged); (4) **the
      player's camera controls** (mouse button held, tilt keys, Home,
      F11); (5) flights and cuts, as before. Within one tick all causes
      give **one** update (our method: the original handles one message
      per committed move, and in normal play that is one per update).
      "Queued, not immediate" needs no queue in our build: the camera's
      system runs after the traps' and cues' scripts in each tick, so
      the update for a trap's `move Connor` already comes after its
      `move KQCamera`. "Connor is moving" (obstacles) and the previous
      aim point now count **ticks** without a "moved" cause: after 3 the
      previous aim point is reset to where he stands. Logs: every update
      with its causes; ticks without an update are simply missing from
      the table.
    - **3n-2, the refined height rule** (**done** 2026-10-18; `camera.md` "Height above the
      ground" rule 2). The order of an update becomes the note's: leash
      (max) or fixed distance, then the height, then the push out to the
      minimum. Height: (1) lift at once to ground + 1,400 if under it by
      at most the range's maximum; deeper: to where the line from the
      update's start to the new spot meets the floor grid, wanted height
      taken again there (STAND-IN if the line meets nothing: lift to
      ground + 1,400); (2) snap within 100; (3) otherwise ease only if
      the aim point moved more than 20 since the previous record, by the
      **sum of sizes** |Δx| + |Δy| + |Δz| ("Do not reproduce"), from the
      height **at the start of the update**: + (1 − 0.9^(15 × seconds))
      × (wanted − height after 1–2), i.e. 10% per update at 15 per
      second; not moved: kept (snapped only if the mouse swings and it
      was lifted earlier). This replaces the hand-tuned 0.5 per update.
      Log: lifted / snapped / eased / kept, the sum, the wanted height.
    - **3n-3, `move KQCamera` while following** (**done** 2026-10-18; Q100 asked). The script's place and
      view stay until the next camera update (3n-1 gives this). In that
      update the tilt is set at once by the follow view (looks at the
      aim point) and the heading is **kept** as the script left it: the
      swing-behind rules (open question 7) are not decoded. STAND-IN:
      the heading stays kept in later updates too, until the next
      `follow`, flight or cut (no Daventry script needs more); each
      update logs how far off Connor it is.
    - **3n-4, Q97: the flight's last update and the cut** (**done** 2026-10-18). The height
      rule as "not moved": lifted at once if under the ground + 1,400,
      snapped within 100 of the wanted height, otherwise kept (the
      stand-in wording goes). The log gives the floor-grid ground under
      the end spot (watermill: is it above 9,463.4?).
    - **3n-5, Q98: logs only** (**done** 2026-10-18; Q101 asked). In the update a scripted walk ends:
      `walk`'s phase, the stop transition chosen and the "from" point it
      waits for, and that transition's steps. No rule change.
    - Tests: the mill wheel (`--at 186300,227500,0`): at most 2 updates
      after the trap's `On`, none until cue 1; the first shot about
      (185500, 226520, ~9,627); the second logged at exactly (185000,
      237850, 10850), tilt −0.25, heading −2.8, still until the flight.
      Walking: updates every tick while he walks, none from 1 tick after
      he stops. The watermill flight still ends at (188999.0, 229341.5,
      10863.4).

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
  - **4b-3, captions in the original font** (viewer). If the layout
    names the caption's place and font, draw the caption there in that
    font (glyph pictures scaled from 640×480 to the window). What the
    layout doesn't say (colour, line wrapping, a band behind the text)
    stays our stand-in until A's note; if it names no font, a sample
    line in each font for the user to pick by eye.
  - Tests: unit tests on hand-made files; `font-verify`, `gui-verify`
    on the install; viewer run with the alchemist's sign.

### M5 — Fight

**You will see:** monsters that move, attack and die; Connor's health and
stats.

- A: monster AI per type, combat rules and numbers, stats, death.
- B: implement.

### M6 — The whole game

Save/load, moving between worlds, all 12 worlds, cutscene videos, music,
options. A and B as needed per topic.

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
