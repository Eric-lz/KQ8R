# King's Quest: Mask of Eternity reimplementation in Rust

Unofficial fan project, not affiliated with or endorsed by the game's
owners. Contains no game files: you need your own copy of the game.
The code is written with an AI coding agent (Claude Code); see
[CREDITS.md](CREDITS.md).

- Game version: 1.3 (GOG.com)
- Game files in ./MASK/ folder (read only, never committed)
- Using NO-CD patch ([KQ8FixWin11](https://github.com/SegMash/KQ8FixWin11))

## What works

- **Reading the game's files:** archives, images, palettes, 3D models,
  buildings, the ground of every world, and the scripts that set up each
  world.
- **Seeing a world:** the ground, the buildings and the objects the game
  places at the start, each where the original puts them, with animated
  objects at their starting pose and see-through things (waterfalls,
  fountains) see-through. Checked by eye in Daventry; the other 11 worlds
  load but have only been looked at in a few places.
- **Connor:** he stands at the world's start point, walks, runs, steps back
  and turns with the original's keys and speeds, with his animations.
  Starting to walk or run plays the original's short start animation
  first (with its small step to his left), then the walk or run (checked
  by eye). Letting go, he finishes his step and plays the matching stop
  animation (checked by eye). He
  follows hills, runs down steep walkable slopes without falling, can't
  climb slopes steeper than about 37° (he slides back), and falls off high
  places with the falling, landing and death animations. Falls and slides
  are as fast as in the original (gravity measured in the running game).
  Buildings are solid: he stops when he walks straight into a wall,
  slides along it at an angle, climbs low steps, and stands on building
  floors. Checked by eye: he goes inside houses, crosses bridges and
  slides along walls. After walking into a wall he stands idle until
  forward is released and pressed again, or a turn key is pressed, as in
  the original (the turn key part checked in the logs, not yet by eye). He
  does a standing jump (Insert) from standing or walking, with the
  original's arc (checked by eye), and running and backward jumps
  (Insert while running or stepping back), all checked by eye. Jumping
  into a wall or steep slope: a standing jump stops him, a running jump
  crashes, a back flip slides along and plays on, as in the original. Running
  straight into a wall knocks him back (the crash animation), as far as
  in the original (checked by eye). Objects are solid too (trees,
  signposts, doors, crates, stone villagers…), with each object's box
  from the game's table (checked by eye).
- **The camera** follows Connor like the original's: it trails behind him,
  keeps above the ground, and can be swung, zoomed and tilted.
- **Curiosity key H:** shows things the game hides or never uses (traps,
  sound sources, unused pictures and objects that appear later in the
  story).

## What doesn't work yet

There is nothing to play yet. Some objects have no
box in the game's table and stay walk-through (as in the original, if we
read it right). The camera doesn't swing
back behind him by itself or avoid walls. People and monsters are not shown,
and there is no lighting, distance haze, sound, talking, inventory, fighting
or story. The current plan is in [docs/DESIGN.md](docs/DESIGN.md) and the
latest changes in [MODLOG.md](MODLOG.md).

## Try it

See a world and walk Connor around:

```sh
cargo run --release -p kq8-viewer -- --terrain --world Daventry
```

| Key | What it does |
| --- | --- |
| Arrow keys | Connor walks / steps back / turns |
| End | Running on or off |
| Right mouse + move | Swing (left/right) and zoom (up/down) the camera |
| Shift + right mouse | Tilt the camera |
| Page Up / Page Down, Home | Tilt the camera, reset the tilt |
| F11 | Camera keeps a fixed distance |
| F | Switch to a free flying camera (W/A/S/D, Q/E, right mouse to look) and back |
| H | Show or hide hidden and unused objects |
| G / B | Test: lift Connor high / a little and let him fall |
| Esc | Quit |

Other worlds: Swamp, DeadCity, Gnome, Barren, IceWorld, Snowexit,
temple1–4, CastleD. One model on its own:
`cargo run --release -p kq8-viewer -- smlchest.KQS --world Daventry`.

Checks of the game files (each writes a log to `kq8-artifacts/logs/`):

```sh
cargo run --release -- vol-verify
cargo run --release -- bitmap-verify
cargo run --release -- shape-verify
cargo run --release -- terrain-verify
cargo run --release -- building-verify
```

Use `--game <folder>` or the `KQ8_GAME_DIR` environment variable if the game
is not in `./MASK/`.

## More

- [docs/DESIGN.md](docs/DESIGN.md): how the project works and the plan
- [docs/provenance/README.md](docs/provenance/README.md): where each rule
  comes from and how it was checked
- [docs/RESEARCH.md](docs/RESEARCH.md): existing tools and documentation
- [CREDITS.md](CREDITS.md): sources and licenses
- [MODLOG.md](MODLOG.md): change log
