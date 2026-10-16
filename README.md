# King's Quest: Mask of Eternity reimplementation in Rust

A fan rewrite of *King's Quest: Mask of Eternity* (Sierra, 1998) in Rust,
to make it run well on modern computers and, later, to improve its
controls, graphics and gameplay.

Unofficial fan project, not affiliated with or endorsed by the game's
owners. **This repository contains no game files: you need your own copy
of the game.** The code is written with an AI coding agent (Claude Code);
see [CREDITS.md](CREDITS.md).

## Current state

**Not playable yet.** Today you can load any of the game's 12 worlds in a
viewer, walk Connor around, open doors and chests and pick up
items. The goal is the whole game; see the roadmap in
[docs/DESIGN.md](docs/DESIGN.md#roadmap). We are at milestone **M3,
"Daventry comes alive"**.

### What works

- **Reading the game's files**: archives, images, 3D models, buildings,
  ground, world scripts, item types.
- **Seeing a world**: ground, buildings and objects placed as in the
  original. All 12 worlds load; Daventry is the best checked.
- **Connor moves like the original**: walk, run, turn, jump, lean into
  turns, slide on steep slopes, fall, wade through water. Walls, floors
  and objects are solid.
- **Camera** that follows Connor and stays out of walls, as in the
  original.
- **Doors** open with a mouse click when Connor is close enough.
- **Chests, baskets and items**: open them, pick things up (counted in a
  log; no inventory screen yet).
- **Ground lighting**: hill shadows from the game's light maps, as dark
  as in the original.
- **Object and building lighting**: objects and Connor lit by the world's
  light and sun, buildings by their own light maps.
- **Choice of graphics API**: Vulkan or DirectX 12 (`--renderer vulkan|dx12`).
- **Distance fog**: a dark fog hides everything far away, and far objects switch off as in the original (F8 on/off).
- **Point lights on objects** (Dead City's lights, lamps, candles), as in the original.
- **Living world**: falling leaves, snow, flowing rivers, windmills,
  torches.
- **Traps**: invisible zones that, for example, make Connor walk indoors
  and run outdoors.

Everything above was checked against the original, mostly by eye. Full
details and how each was checked: [docs/FEATURES.md](docs/FEATURES.md).

### What doesn't work yet

- No story, talking, inventory screen, fighting or saving.
- No sound or music.
- People and monsters are not shown.
- Clicking works only on doors, chests, items, buildings and a few scripted
  objects; Connor doesn't speak (his remarks are only logged).
- Point lights don't light walls yet.
- Objects only play their starting animation.

Full list: [docs/FEATURES.md](docs/FEATURES.md#what-doesnt-work-yet).

## What you need

- Windows (only tested on Windows 11).
- [Rust](https://www.rust-lang.org/tools/install), stable, with the MSVC
  toolchain (the default on Windows).
- *King's Quest: Mask of Eternity*, installed in a `MASK` folder inside this repository. The game is
  only read, never changed. If it is somewhere else, add
  `--game <folder>` to the commands below, or set the `KQ8_GAME_DIR`
  environment variable. *Tested only with GOG.com copy (version 1.3, also called
  1.03).*
- To run the *original* game for comparison on Windows 11 we use the
  [KQ8FixWin11](https://github.com/SegMash/KQ8FixWin11) compatibility
  fix (also used as a no-CD patch).

## Try it

Open a terminal in this folder and run:

```sh
cargo run --release -p kq8-viewer
```

This opens Daventry's ground with its objects and lets you walk Connor arund.
To open a different world, use the `--world` flag.
For example, to spawn Connor in the Swamp, you can use

```sh
cargo run --release -p kq8-viewer -- --world swamp
```

*The first `--` separates Cargo's options from the viewer's options.*

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
| Left mouse | Use the door or chest, or pick up the item, under the mouse |
| Right mouse + move | Swing (left/right) and zoom (up/down) the camera |
| Shift + right mouse | Tilt the camera |
| Page Up / Page Down, Home | Tilt the camera, reset the tilt |
| F11 | Camera keeps a fixed distance |
| F | *DEBUG:* Free flying camera (W/A/S/D, Q/E, right mouse to look) and back |
| H | *DEBUG:* Show hidden and unused objects (traps, sound sources...) |
| U | *DEBUG:* Show the ground under the water |
| F7 | *DEBUG:* Light of the ground and objects: x1 (default, as in the original), x2, off |
| F8 | *DEBUG:* Haze (distance fog) on / off |
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
```

Other debugging tools: `cargo run --release -- text <file>` prints one of the
game's text files; `cargo run --release -p kq8-viewer -- --model smlchest.KQS` shows one model on its own.

## How the repository is organised

The code is a Cargo *workspace*: one repository holding several Rust
packages, called *crates*.

```text
crates/
  kq8-data/     reads the game's files (archives, images, models, ground,
                object lists, scripts) into plain data
  kq8-core/     the game's rules, without drawing (Connor's movement,
                camera, scripts, doors, chests, traps, items)
  kq8-viewer/   the window: draws a world and runs the rules; also our
                test tool (H and U keys, free camera, test options)
  kq8-cli/      the `kq8` command-line tool for the file checks above
docs/
  DESIGN.md     how the project works, and the roadmap
  FEATURES.md   what works and what doesn't, in detail
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
- [docs/FEATURES.md](docs/FEATURES.md): what works, in detail
- [docs/SESSIONS.md](docs/SESSIONS.md): two session workflow, in detail
- [docs/RESEARCH.md](docs/RESEARCH.md): existing tools and documentation
- [docs/behaviour/QUESTIONS.md](docs/behaviour/QUESTIONS.md): open
  questions about the original's behaviour
- [docs/provenance/README.md](docs/provenance/README.md): where each rule
  comes from and how it was checked
