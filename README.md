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

A rough picture of how far the project has come.
The detailed plans are in [docs/DESIGN.md](docs/DESIGN.md), and
the full list of what works is in [docs/FEATURES.md](docs/FEATURES.md).

So far everything is built and tested (mostly) in the first world, **Daventry**.

| Milestone | What it means | Status |
| --- | --- | --- |
| M0 | Read the game's files | ✅ Done |
| M1 | See Daventry | ✅ Done |
| M2 | Walk Connor around Daventry | ✅ Done |
| M3 | Daventry comes alive | ✅ Done |
| M4 | Talk, listen, use the menus | 🟡 Under way |
| M5 | Fight | 🟡 Under way |
| M6 | The whole game | 🟡 Under way |
| M7 | Match the original, then improve | ⬜ Not started |

### Done

- **M0 – Reading the game's files.** Terrain, buildings, pictures and
  colour palettes are read straight from your own copy of the game.
- **M1 – Seeing Daventry.** The ground, buildings, trees and objects are
  drawn with the original's lighting and distance fog. All 12 worlds load.
- **M2 – Walking around.** Connor walks, runs, steps back, turns, jumps
  and wades through water. He falls, slides off steep slopes and bumps
  into walls the way he does in the original.
- **M3 – Daventry comes alive.** Doors, chests, items to pick
  up, traps, animated objects (windmill, signs, fountain), falling
  leaves, flowing water, sound effects and music. The game's own puzzle
  scripts run, scripted cutscenes play, and the first puzzle (pushing
  the bricks to block the waterfall) works. The camera follows Connor
  as in the original.
- **M4 – So far.** Connor's remarks and other spoken lines play with
  their captions and lip sync. The on-screen interface works: the bottom
  bar (health, experience, weapons, potions), the top bar (quest items,
  gold) and holding an item on the mouse pointer to use it. Daventry's
  people and creatures are drawn and animated: they breathe, gesture
  and move their mouths while they speak, and can be clicked.
  Conversations with the wizard and the beast at the pond work: Connor
  walks to his spot, the camera cuts between fixed views, and walking
  away ends the talk with a goodbye. Characters stand on the ground
  and on building floors, and cast shadows on the ground and on buildings.
  Connor can't walk through characters: he slides round them.
- **M5 – So far.** Daventry's creatures move on their own as in the original: they wander, fidget, notice Connor, chase him up to striking distance and give up; zombies rise from the ground. They fight: they swing at Connor, he can punch back, they flinch, die, drop items and leave bodies; the bow spriggans shoot (first version, checked in the logs only). Health items and the four potions work, with their timer icons (checked in the logs only).
- **M6 – So far.** The Rope and Hook: Connor climbs up and down at rope places (checked by eye). Azriel's Hammer: thrown, hits one thing, flies back and is caught (checked in the logs only). The magic map: fog, arrow, zoom and pan, the mini-map, other worlds' maps and the teleporter button (checked in the logs only). Moving between worlds: walking exits and teleporters ask "load a new region?", show the loading screen and arrive with Connor's things; a world you come back to keeps its map, its story progress and what was taken from it (loading screen checked by eye, the rest in the logs only).

### Next

- **M5 – Fighting:** check the fight and the potions by eye against the original; Connor's controls and weapons (the click-to-swing is a stand-in for now).

### Later

- **The main menu** (postponed: the viewer starts straight in Daventry,
  which keeps testing quick).

- **M5 – Fighting.** Connor's other weapons and bows, game over.
- **M6 – The whole game.** Saving and loading (with worlds kept in full),
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

To spawn Connor in a different world, e.g. in the Swamp, run:

```sh
cargo play --world swamp
```

Other worlds:
`Swamp, DeadCity, Gnome, Barren, IceWorld, Snowexit,
temple1–temple4, CastleD`

There are other launch options, mostly for debugging.
You can find them in
[docs/FEATURES.md](docs/FEATURES.md#launch-options-viewer),
or by running:

```sh
cargo play --help
```

### Command console

While the game runs, you can go back to the terminal you started the game from
and use console commands. Type `help` to get a list of commands.

### Key bindings

Follows original game's default keybindings, plus extra debugging keys.

| Debug key | What it does |
| --- | --- |
| F6 | Switch UI between centered (4:3) or widescreen |
| F | Free flying camera (W/A/S/D, Q/E, right mouse to look) and back |
| H | Show hidden and unused objects (traps, sound sources...) |
| U | Show the ground under the water |
| F7 | Light of the ground and objects |
| F8 | Haze (distance fog) on / off |
| F9 | Dynamic lights on / off |
| F5 | Animated objects: new / old drawing (test, while the animation rewrite is under way) |
| G / B | Test: lift Connor high / a little and let him fall |
| F3 | Test: god mode on / off (Connor can't be hurt or die from a fall) |
| F4 | Test: creatures' AI off / on (they don't notice or attack Connor) |

Fighting (first version): click the hand-weapon slot in the bottom bar
(or press Delete) to ready Connor's fists, then left-click a creature to
swing (Shift: the big swing, Ctrl: a normal click). Delete swaps between
the hand and the ranged weapon, Alt puts the weapon away. With the
crossbow ready a left click shoots where you point; with Azriel's Hammer
ready it throws the hammer.

## How the work is done

To keep the rewrite our own, this project is developed in two separate
sessions:

- One session studies the original game's files and program,
 and writes its behaviour in plain-language notes in
[docs/behaviour/](docs/behaviour/);
- The other session writes Rust code from those notes only,
and has **no access to the original game's files**. Every rule
records where it came from in
[docs/provenance/README.md](docs/provenance/README.md). More in
[docs/DESIGN.md](docs/DESIGN.md#working-method-for-studying-the-original).

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
cargo run --release -- character-verify
cargo run --release -- map-verify
```

Other debugging tools: `cargo run --release -- text <file>` prints one of the
game's text files; `cargo play --model smlchest.KQS` shows one model on its own.

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
