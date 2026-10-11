# STATUS

Written 2026-10-11 by the study session (A) as a hand-over in case the chat
is lost. Read with `AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`,
`MODLOG.md`.

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* that reads the player's
own install at runtime. Roadmap M0–M7 in `docs/DESIGN.md`.

## Setup

- Game: GOG 1.03 ("Version 1.3") in `./MASK/` (read only, never committed).
- Rust stable (MSVC), Bevy 0.20 for the viewer (`crates/kq8-viewer`).
- Two-session workflow (`docs/SESSIONS.md`): **A** studies `Mask.exe` in
  Ghidra and writes `docs/behaviour/*.md` (no code); **B** writes code from
  those notes and the data, never sees the exe.
- Ghidra 12.1.4 at `C:\Tools\Ghidra`, project `re/ghidra/KQ8`, program
  `re/Mask.exe`; GhidraMCP 2.0 via `.mcp.json` (session A only).
  `re/` (Ghidra project, scratch notes `re/notes.md` with addresses,
  throwaway scripts) is never committed.

## What works (tested)

- M0: VOL archives (`kq8 vol-verify`), images/palettes (`kq8 bitmap-verify`,
  `bitmap-png`, confirmed by eye).
- M1 step 1: models and material lists, one textured model in the viewer
  (checked by eye).
- M1 step 2: terrain read, ground height lookup, ground drawn with textures
  in the viewer (session B; see its MODLOG entries for test results).

## Behaviour notes ready (session A)

- `connor-movement.md`: controls, walk ≈ 1,127 / run ≈ 2,518 units/s (from
  animation steps), turning 1.75 rad/s, banking, start/stop transitions.
- `camera.md`: follow camera, min/max range, height, mouse/Shift/F11/F2.
- `models-and-textures.md`: row-vector transforms; frame transforms move the
  object (not drawing); right-handed, no mirror; clockwise front faces;
  back faces hidden; per-world palettes; archive order.
- `terrain.md`: square = 2^groundScale, height = value × heightBMPScale,
  chessboard triangle split, ground texture choice and orientation table.

## What doesn't work yet / next

- M1 step 3: objects placed (3a done by B); buildings next, from
  `docs/behaviour/buildings.md` (Q11: `.dig` layout; buildings at locZ −1
  are lifted onto the ground). Animated objects' start frame: Q12 in
  `objects.md`.
- Still open for A: which colour (if any) is see-through and what material
  bits 0x1000/0x2000 switch on (Q5), system colours for textures (Q3),
  which archive wins for duplicate names / missing textures (Q4), terrain
  holes, `floorBMPName`, `.dil` lighting, the 4th `loop=` number.
- M2 study (A): slopes (walkable limit ~35° suspected), collisions
  (manual: stop when straight on, glance off at an angle), jumping.

## Files that matter

- `docs/behaviour/QUESTIONS.md`: B's questions and A's answers (status column).
- `docs/provenance/README.md`: where each fact came from.
- `re/notes.md` (local only): addresses for everything A found.

## Prompt for a fresh session

Study session: use the "Starting session A" prompt in `docs/SESSIONS.md`,
topic "Q5 and the M2 topics in STATUS.md". Implementation session: the
"Starting session B" prompt, task "M1 step 3".
