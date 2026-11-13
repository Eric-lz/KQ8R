# Agent memory (copy)

A copy of the notes Claude Code keeps in its private memory folder
(`C:\Users\ericp\.claude\projects\c--Games-KQ8R\memory\`), taken on
2026-11-13. That folder is loaded into every session automatically but
lives only on this machine. This file is a snapshot and does not update
itself.

## Rules from the user (feedback)

### Commit each step
Commit each finished step separately: code, MODLOG entry, and
DESIGN/provenance updates in one commit (message like
"M3 step 3x-N: ..."), so a single step can be reverted.
- **Why:** The user answered "Yes, commit each step" on 2026-10-17 when
  steps were piling up uncommitted. AGENTS.md otherwise forbids
  committing unless asked, so this is the standing request.
- **How to apply:** After a step's tests pass and its MODLOG entry is
  written, stage only that step's files and commit. Don't mix two steps
  in one working tree before committing.

### No side build folders
If a build fails because the viewer window holds `kq8-viewer.exe`, ask
the user to close the viewer rather than building into a second target
folder. If a side folder can't be avoided, delete it right after testing.
- **Why:** A leftover `target/side/` from 2026-10-11 used 12 GB. On
  2026-10-13 the user agreed to this cleanup.
- **How to apply:** Before building the viewer, check that `kq8-viewer`
  isn't running. Never leave a `CARGO_TARGET_DIR` copy in place. Keep
  `debug = false` for dependencies in `Cargo.toml`.

### Keep the artifact logs
Never run `rm kq8-artifacts/logs/*.log` (or anything like it) to get
clean logs. The folder holds named reference logs from earlier steps
(`camera-3m4-*.log`, `map-verify.log`, `shape-verify.log`, ...).
- **Why:** On 2026-10-22 (step 6e-1) about 100 useful logs were wiped
  by mistake.
- **How to apply:** Each viewer run truncates its own logs anyway. Copy
  fresh logs out to the scratchpad by name. Look at a folder before
  deleting anything from it.

### play.cmd stays local
`play.cmd` in the repo root (starts `target\release\kq8-viewer.exe`
without cargo) stays on the user's machine only: not whitelisted in
`.gitignore`, never committed or pushed. Its earlier copy in commit
769e161's history is fine (user, 2026-10-23); no history rewrite.
- **Why:** The user doesn't want the script on GitHub.
- **How to apply:** Don't stage it, don't add a whitelist line, don't
  mention it in README (DESIGN/STATUS call it a local script). Editing
  it locally is fine if asked.

### Daventry test spots
Daventry's church (trap `chrchtrp`, around x 202,500–216,300,
y 180,900–186,400) is walled in: the only way out is climbing with the
rope. Don't use it for runs where Connor walks in and out. Use the
wizard's music zone `mwiztrap` instead:
`--at 193000,193000,3.1416 --hold 1,0,1,90` runs north through it
(On at update ~23, Off at ~60).
- **Why:** Many runs were wasted probing the church's walls until the
  user stepped in (2026-10-17).
- **How to apply:** Before picking a test spot, check that Connor can
  reach and leave it. When unsure, ask the user. Heading 3.1416 = +y,
  1.5708 = +x in `--at`.
- **Pushing spots:**
  - Bricks puzzle: `--at 156625,303748,1.57 --lift 6000 --hold 1,0,0,60`
    (needs the lift because `--at` drops Connor to the lowest ground under
    the cave ledge).
  - Hay bale in the windmill: `--at 146500,248928,1.5708 --hold 1,0,0,300`.
  - Barrel `barrel.04`: `--at 205364,267646,1.5708`.

### Check a constant's width (study sessions only)
When a study answer depends on a number read from Mask.exe, read it at
the width the instruction uses (8-byte double or 4-byte float). Treat a
surprising 0 or 1 as a red flag.
- **Why:** On 2026-10-22 (Q126) an 8-byte 0.5 was read as a 4-byte 0.
  The note then gave the wrong conversation camera centre, which cost
  the user a check in the original game and a correction commit.
- **How to apply:** Session A only. Session B never reads Mask.exe.

## Project decisions

### M3 is Daventry only
M3 ("Daventry comes alive") covers Daventry only and was closed as done
on 2026-10-19 after the user's camera check. Gnome's rolling rocks,
Gnome's pushable `boulder` (`KQBoulder`), other pushable classes and
damaging water belong to M6 (or M5 for damage).
- **Why:** Sessions kept treating later-world items as open M3 work, so
  M3 could never be finished.
- **How to apply:** When listing what's left of M3, count only
  Daventry's objects and puzzles.

### Main menu postponed
The main menu (Escape / mask button, New game, opening cutscenes) is
postponed (user, 2026-10-20). Don't suggest it as the next step.
- **Why:** Starting straight in Daventry lets the user test new features
  at once.
- **How to apply:** When the menu is eventually built, keep a launch
  option that skips it and starts in the world directly.

### M7 idea: default weapon damage
In M7 (improvements), give every weapon a default base damage, so a
weapon carried into a world where its data has no damage values still
works. Example: Azriel's Hammer (`prhammer.kq`) has no
modifier/min/maxDamage in 9 worlds, so it does 0 there. The user wants
this for cheat codes that hand out late-game weapons early (stated
2026-10-20).
- **Why:** In normal play the hammer is traded away in the Swamp, so
  only cheats would expose the 0 damage.
- **How to apply:** Don't change damage rules before M7. See
  `docs/behaviour/azriel-hammer.md`, "Do not reproduce".

### Script coverage workflow
Since 2026-10-27, missing script features are handled in batches from
`docs/SCRIPT-COVERAGE.md` (generated by
`cargo run --release -- script-coverage --out docs/SCRIPT-COVERAGE.md`),
not one cutscene at a time. Batch order is in `docs/DESIGN.md`, "script
coverage checklist" (SC-2). Ready-made A/B prompts are in
`docs/SESSIONS.md`.
- **Why:** The user found piece-by-piece fixing slow and asked for a
  systematic way.
- **How to apply:** In a B session, build the batch session A studied,
  rerun the tool, commit the new list and report what's left. The user
  still checks each batch in play.
