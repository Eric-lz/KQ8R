# King's Quest: Mask of Eternity reimplementation in Rust

A fan rewrite of *King's Quest: Mask of Eternity* (Sierra, 1998) in Rust,
to make it run well on modern computers and, later, to improve its
controls, graphics and gameplay.

> [!IMPORTANT]
> This project requires an original copy of *King's Quest: Mask of Eternity* to run.
> It will not start without one. It will not modify any of the original game's files,
> only read them. **This repository contains no game files.**

Unofficial fan project, not affiliated with or endorsed by the game's owners.

The code is written with an AI coding agent (Claude Code);
see [CREDITS.md](CREDITS.md).

## Current state

A rough picture of how far the rewrite of *King's Quest: Mask of Eternity*
has come. The detailed plans are in [docs/DESIGN.md](docs/DESIGN.md), and
the full list of what works is in [docs/FEATURES.md](docs/FEATURES.md).

So far everything is built and tested in the first world, **Daventry**.

| Milestone | What it means | Status |
| --- | --- | --- |
| M0 | Read the game's files | ✅ Done |
| M1 | See Daventry | ✅ Done |
| M2 | Walk Connor around Daventry | ✅ Done |
| M3 | Daventry comes alive | 🟡 Mostly done |
| M4 | Talk, listen, use the menus | 🟡 Under way |
| M5 | Fight | ⬜ Not started |
| M6 | The whole game | ⬜ Not started |
| M7 | Match the original, then improve | ⬜ Not started |

### Done

- **M0 – Reading the game's files.** The game's archives, pictures and
  colour palettes are read straight from your own copy of the game.
- **M1 – Seeing Daventry.** The ground, buildings, trees and objects are
  drawn where the game puts them, with the original's lighting
  and distance fog. All 12 worlds load.
- **M2 – Walking around.** Connor walks, runs, steps back, turns, jumps
  and wades through water. He falls, slides off steep slopes and bumps
  into walls the way he does in the original. The camera follows him and
  keeps out of walls.
- **M3 – Daventry comes alive (mostly).** Doors, chests, items to pick
  up, traps, animated objects (windmill, mill wheel, fountain), falling
  leaves, flowing water, sound effects and music. The game's own puzzle
  scripts run, scripted cutscenes play, and the first puzzle (pushing
  the bricks to block the waterfall) works.
- **M4 – So far.** Connor's remarks and other spoken lines play with
  their captions and lip sync. The on-screen interface works: the bottom
  bar (health, experience, weapons, potions), the top bar (quest items,
  gold) and holding an item on the mouse pointer to use it.

### Next

- **Conversations** with the wizard and the other people in Daventry.
- **The main menu:** pausing the game with Escape key.
- **What's left of M3:** a few cutscene and camera details, damage from
  rolling rocks, and pushable boulders.

### Later

- **M5 – Fighting.** Monsters that move, attack and die; Connor's weapons,
  armour, health and experience in battle.
- **M6 – The whole game.** Saving and loading, travelling between worlds,
  all 12 worlds playable, the cutscene videos, the options screen.
- **M7 – Match the original, then improve.** Play the original side by
  side with ours and fix every difference. After that, the improvements
  this project was started for: better controls, better graphics and
  gameplay fixes. (A first one is already in: the bottom bar's corner
  panels can sit at the window's edges on wide screens.)

## What you need

- Windows (only tested on Windows 11).
- [Rust](https://www.rust-lang.org/tools/install), stable, with the MSVC
  toolchain (the default on Windows).
- *King's Quest: Mask of Eternity*, installed in a `MASK` folder inside this repository. The game is
  only read, never changed. If it is somewhere else, add
  `--game <folder>` to the commands below, or set the `KQ8_GAME_DIR`
  environment variable. *Tested only with GOG.com copy (version 1.3, also called
  1.03).*

To run the *original* game for comparison on Windows 11 we use the
[KQ8FixWin11](https://github.com/SegMash/KQ8FixWin11) compatibility
fix (also used as a no-CD patch).

## Try it

Open a terminal in this folder and run:

```sh
cargo play
```

This opens Daventry's ground with its objects and lets you walk Connor arund.
To open a different world, use the `--world` flag.
For example, to spawn Connor in the Swamp, you can use

```sh
cargo play --world swamp
```

*`cargo play` is a short name for `cargo run --release -p kq8-viewer --`
(set in `.cargo/config.toml`); either form works.*

Other worlds: Swamp, DeadCity, Gnome, Barren, IceWorld, Snowexit,
temple1–temple4, CastleD. (not case-sensitive)

To spawn Connor at a specific coordinate, use `--at x,y,heading`, where `heading` is
the direction he's facing.

To change graphics API, use `--renderer vulkan|dx12|auto`.
Default is `auto`.

There are other launch options, but most are used for debugging only.
You can check them out at [docs/FEATURES.md](docs/FEATURES.md#aunch-options-(viewer))

The first build takes several minutes.

### Key bindings

| Key | What it does |
| --- | --- |
| Arrow keys | Walk, step back, turn |
| End | Running on or off |
| Insert | Jump |
| Space | Put the bottom bar away or bring it back |
| 1–8 | Use the health or potion slot 1–8 |
| Mouse at the top edge | Show the top bar (quest items, gold) |
| Right mouse on an item in a bar | Connor describes it |
| Left mouse on a quest item, gold, rope or rocks | Hold it on the pointer (again: put it back); then left-click something in the world to use it |
| Left mouse | Use the door or chest, or pick up the item, under the mouse |
| Right mouse + move | Swing (left/right) and zoom (up/down) the camera |
| Shift + right mouse | Tilt the camera |
| Page Up / Page Down, Home | Tilt the camera, reset the tilt |
| F11 | Camera keeps a fixed distance |
| F6 | Bottom bar's corner panels at the window's edges, or back on the 4:3 screen (not in the original) |
| F | *DEBUG:* Free flying camera (W/A/S/D, Q/E, right mouse to look) and back |
| H | *DEBUG:* Show hidden and unused objects (traps, sound sources...) |
| U | *DEBUG:* Show the ground under the water |
| F7 | *DEBUG:* Light of the ground and objects: x1 (default, as in the original), x2, off |
| F8 | *DEBUG:* Haze (distance fog) on / off |
| F9 | *DEBUG:* Dynamic lights on / off |
| G / B | *DEBUG:* Test: lift Connor high / a little and let him fall |
| Esc | Quit |

Logs are written to log files in `kq8-artifacts/logs/` (for
example `connor.log`, `doors.log`, `items.log`). If something looks
wrong, those logs are the most useful thing to report.

### Checking the game files

These commands read the whole game and report any file they can't
understand (each also writes a log to `kq8-artifacts/logs/`):

```sh
cargo run --release -- vol-verify
cargo run --release -- bitmap-verify
cargo run --release -- shape-verify
cargo run --release -- terrain-verify
cargo run --release -- lightmap-verify
cargo run --release -- lighting-verify
cargo run --release -- building-verify
cargo run --release -- item-verify
cargo run --release -- aud-verify
cargo run --release -- msg-verify
```

Other debugging tools: `cargo run --release -- text <file>` prints one of the
game's text files; `cargo play --model smlchest.KQS` shows one model on its own.

## How the repository is organised

The code is a Cargo *workspace*: one repository holding several Rust
packages, called *crates*.

```text
crates/
  kq8-data/     reads the game's files (archives, images, models, ground,
                object lists, scripts) into plain data
  kq8-core/     the game's rules, without drawing (Connor's movement,
                camera, scripts, doors, chests, traps, items, sound rules,
                speech queues)
  kq8-audio/    plays sounds: our mixer and the sound card (cpal)
  kq8-viewer/   the window: draws a world and runs the rules; also our
                test tool (H and U keys, free camera, test options)
  kq8-cli/      the `kq8` command-line tool for the file checks above
docs/
  DESIGN.md     how the project works, and the roadmap
  FEATURES.md   what works and what doesn't, in detail
  milestones/   step-by-step plans of finished milestones
  behaviour/    notes on how the original game behaves, and open questions
  provenance/   where each piece of knowledge came from
  RESEARCH.md   existing tools and documentation for this game
MODLOG.md       every change, with how it was tested
```

Not in the repository (kept only on your machine): `MASK/` (the game),
`target/` (build output) and `kq8-artifacts/` (logs).

## How the work is done

The original game's behaviour is studied from its data files and its
program. To keep the rewrite our own, this is done in two separate
sessions: one studies the original and writes plain-language notes in
[docs/behaviour/](docs/behaviour/); the other writes the Rust code from
those notes only, without seeing the original program. Every rule
records where it came from in
[docs/provenance/README.md](docs/provenance/README.md). More in
[docs/DESIGN.md](docs/DESIGN.md#working-method-for-studying-the-original).

## More

- [STATUS.md](STATUS.md): current state of the project, used for session hand-over
- [MODLOG.md](MODLOG.md): change log
- [CREDITS.md](CREDITS.md): sources and licences
- [docs/DESIGN.md](docs/DESIGN.md): how the project works and the plan
- [docs/milestones/](docs/milestones/): step-by-step plans of finished milestones
- [docs/FEATURES.md](docs/FEATURES.md): what works, in detail
- [docs/SESSIONS.md](docs/SESSIONS.md): two session workflow, in detail
- [docs/RESEARCH.md](docs/RESEARCH.md): existing tools and documentation
- [docs/behaviour/QUESTIONS.md](docs/behaviour/QUESTIONS.md): open
  questions about the original's behaviour
- [docs/provenance/README.md](docs/provenance/README.md): where each rule
  comes from and how it was checked
