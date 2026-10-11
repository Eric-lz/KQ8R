# STATUS

Hand-over in case the chat is lost. First written 2026-10-11 by the study
session (A); updated 2026-10-11 by the implementation session (B) after
M2 step 2b (commit `875b423`). Read with `AGENTS.md`, `docs/DESIGN.md`,
`docs/SESSIONS.md`, `MODLOG.md` (newest entries at the top).

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* that reads the player's
own install at runtime. Roadmap M0–M7 in `docs/DESIGN.md`. Current
milestone: **M2, walk Connor around Daventry**.

## Setup

- Game: GOG 1.03 ("Version 1.3") in `./MASK/` (read only, never committed).
- Rust stable (MSVC), Bevy 0.20 for the viewer (`crates/kq8-viewer`).
- Two-session workflow (`docs/SESSIONS.md`): **A** studies `Mask.exe` in
  Ghidra and writes `docs/behaviour/*.md` (no code); **B** writes code from
  those notes and the data, never sees the exe.
- Ghidra 12.1.4 at `C:\Tools\Ghidra`, project `re/ghidra/KQ8`, program
  `re/Mask.exe`; GhidraMCP 2.0 via `.mcp.json` (session A only).
  `re/` (Ghidra project, scratch notes, throwaway scripts) is never
  committed.

## Code layout

- `crates/kq8-data`: file formats (VOL, bitmaps, palettes, `.kqs` shapes,
  terrain, `.dig` buildings, `.kq` text, world scripts, `.sae` key maps,
  object placement rules). 41 unit tests.
- `crates/kq8-game` (new in M2): game rules without drawing. `mover.rs`
  (steps, gravity, landing, Fall state, fall death), `connor.rs` (intent →
  movement, turning, step from the animation). Runs at a fixed **15
  updates per second** (decision in `docs/DESIGN.md`, M2 "Physics rate").
  10 unit tests.
- `crates/kq8-cli`: `kq8` checks (`vol-verify`, `bitmap-verify`,
  `bitmap-png`, `shape-verify`, `terrain-verify`, `building-verify`) and
  `kq8 text <file>` to print a game text file.
- `crates/kq8-viewer`: Bevy viewer. `--terrain [--world <W>]` shows a
  world; `--frames N`, `--lift N`, `--hold F,T,R` are test options.
- Logs go to `kq8-artifacts/logs/` (`viewer.log`, `connor.log` with one
  line per Connor update).

## What works (tested)

- M0: VOL archives, images and palettes (palettes used exactly as stored,
  Q3).
- M1 (all checked by eye in Daventry): ground with textures; 30 buildings;
  the objects the world scripts load (script variables, `getLoc`, `move`,
  hidden objects, missing positions = 0, z −1/−2 = on the ground); animated
  objects at their start frame; see-through objects. All 12 worlds load
  with 0 unreadable script lines (only Daventry, parts of Barren and
  DeadCity looked at by eye).
- Viewer helpers: H shows invisible objects (cubes), unused objects and
  hidden objects, with an on-screen note; camera position shown bottom
  left; field of view 70° vertical.
- M2 step 2a: Connor at his start point (from `GAME/Kq/Connor.kq`, placed
  by Daventry's `newWorld.cs` line 21), gravity on, falls and fall deaths
  as the note says (checked by eye and in `connor.log`).
- M2 step 2b: walk, run, back, turn from `MASK/Connor.sae` (arrows, End =
  run on/off), intent from keys held (the note's recommendation), all
  animation frames played. Speeds measured: walk 1,126.6, run 2,518.1
  units/s; turning 1.75 rad/s (checked by eye: pace close to the
  original). F = camera follows Connor.

## What doesn't work yet / next

- M2 step 2f, the following camera, is built (not yet checked by eye):
  leash, height, mouse swing/zoom, tilt, F11; F switches to the free
  camera. Missing: swing back behind Connor (Q27 c), obstacle check (after
  2d), first person.
- **Slopes:** running at a steep hillside, Connor slides a long way up it,
  then slowly back; in the original he is stopped quickly. Cause: the
  terrain only acts in the mover's ground check, as the note reads.
  Waiting for **Q26**; then M2 step **2c** (sliding, climbing up to 36.9°).
- Next M2 steps (plan in `docs/DESIGN.md`): **2d** walls (buildings and
  objects stop him or let him slide; needs the collision shape, physics
  note open question 2), **2e** jumping/falling animations, crash, death
  animation, **2f** the following camera (`camera.md`).
- No start/stop transition animations yet (Q25); frame steps spread evenly
  (Q24); no banking.
- Not drawn yet: haze/distance fade, building lighting (`.dil`), the
  river's flowing textures (`wateran*`), `.anm` characters (orange cubes
  with H), the teleport pad's look (Q14b).

## Open questions (`docs/behaviour/QUESTIONS.md`)

- Q14b: the teleport pad's see-through look.
- Q22: palette entry 0 on a normal screen: stored blue (Q3) or black (Q5
  answer)? Affects Swamp's whirlpool and teleport textures.
- Q23: the mover's up/down passes with gravity off (hover vs drop).
- Q24: frame steps applied per frame or spread.
- Q25: which start/stop transition plays.
- Q26: does the terrain stop the down pass (slopes, downhill running)?
- Q27: camera details (mouse scale, what tilt turns, swing behind Connor,
  start and `follow none`, range tied to walk/run, zoom direction).

## Files that matter

- `docs/behaviour/*.md`: the rules (session A).
- `docs/behaviour/QUESTIONS.md`: B's questions and A's answers.
- `docs/provenance/README.md`: which note each piece of code follows, and
  how it was checked.
- `docs/DESIGN.md`: architecture, roadmap, step plans.
- `MODLOG.md`: every change with its test.

## Prompt for a fresh session

Study session: the "Starting session A" prompt in `docs/SESSIONS.md`,
topic "open questions Q22–Q26 in STATUS.md". Implementation session: the
"Starting session B" prompt, task "M2 step 2c once Q26 is answered (camera 2f
built; check Q27)".
