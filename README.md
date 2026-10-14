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
  buildings, the ground of every world, the scripts that set up each
  world, and the list of all 135 item types (coins, crystals, weapons...;
  checked with `item-verify` against every item name in the game's data).
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
  animation (checked by eye). Running and turning at the same time, he
  leans into the turn, up to about 10° (0.18 rad, as traced), from the
  running start on, and straightens up when he stops turning or running
  or jumps, also when running through water (checked by eye against the
  original at 0.15 rad and for jumps; the 0.18 end value only in the logs
  so far). He
  follows hills, runs down steep walkable slopes without falling, can't
  climb slopes steeper than about 37° (he slides back), and falls off high
  places with the falling, landing and death animations. Falls and slides
  are as fast as in the original (gravity measured in the running game).
  Buildings are solid: he stops when he walks straight into a wall,
  slides along it at an angle, climbs low steps, and stands on building
  floors. Checked by eye: he goes inside houses, crosses bridges and
  slides along walls. After walking into a wall he stands idle until
  forward is released and pressed again, or a turn key is pressed, as in
  the original (the turn key part checked in the logs, not yet by eye).
  Holding a turn key there keeps him turning until he slides off the
  wall, as in the original (checked in the logs, not yet by eye). He
  does a standing jump (Insert) from standing or walking, with the
  original's arc (checked by eye), and running and backward jumps
  (Insert while running or stepping back), all checked by eye. Jumping
  into a wall or steep slope: a standing jump stops him, a running jump
  crashes, a back flip slides along and plays on, as in the original. Running
  straight into a wall knocks him back (the crash animation), as far as
  in the original (checked by eye). Objects are solid too (trees,
  signposts, doors, crates, stone villagers…), with each object's box
  from the game's table (checked by eye). He walks into water on the
  bottom, with the water up to his chest, and switches to the wading
  walk and run while it is deep (checked in the logs, not yet by eye).
  Buildings and objects stand on the ground under the water where the
  original puts them (e.g. the Swamp shack; checked in the logs).
- **Doors** (M3, first step): a left click on a door uses it, as with
  the game's play cursor. In reach (the original's 4,700 units from the
  door) the door's own script runs, which opens it: it swings open on its
  hinge in the original's time, and its box swings with it, so the doorway
  is free and the open door still blocks Connor where it stands. Plain
  house doors stay open on a second click; from too far nothing happens
  (Connor's remark is only written to the log); the castle keep's doors
  open only with the key (a test option, there is no inventory yet).
  Checked in the logs, and by eye with real mouse clicks
  (2026-10-13): the doors work and the clicks show in the log. Everything is
  written to `kq8-artifacts/logs/doors.log`.
- **Falling leaves and stream drips** (M3, emitters): the small particles
  the game keeps flying around Daventry's trees and over the stream, with
  the original's speeds, lifetimes, swaying and gravity. Leaves that
  drift into the cottage vanish at its box, as in the original. The
  rivers and the stream flow: their ground textures change every 0.3 s
  (timed from each change, as the original does; 2026-10-13, checked in
  the log and by eye).
  In IceWorld it snows around Connor wherever he goes, but not inside
  buildings (checked in the logs and by eye, 2026-10-13).
  Checked in the logs (`kq8-artifacts/logs/effects.log`, `ground_anim.log`) and by eye
  (2026-10-13).
- **Animated objects play** (M3): the windmill sails, mill wheel and
  gears, swinging signs, shutters, torches, lamps, waterfalls, fish and
  the fountain's water move as the game's files say, torches and other
  rows of things out of step as in the original. Checked in the logs
  (`kq8-artifacts/logs/anim.log`) and by eye (2026-10-13). Gnome's three
  rolling rocks roll out and come back each cycle (checked by eye), and
  since 2026-10-13 their collision boxes move with them (checked in the
  log only). Gnome's pushable `boulder` stands still, as in the original
  (checked by eye); it can't be pushed yet.
- **Traps** (M3): Daventry's 52 invisible trap boxes are loaded and
  switch on and off as Connor walks in and out of them (including the
  music zones' smaller "get in" box), running their scripts. Walk and run
  zones work: inside Connor's house (and the other houses with such
  zones) he drops to a walk and the camera follows closer (1,000–2,500);
  outside he runs again (1,500–5,000). Checked in the logs
  (`kq8-artifacts/logs/traps.log`) and by eye, the same as in the original
  (user, 2026-10-13). Music, ropes,
  teleporters and cutscenes started by traps are later milestones (their
  commands are only logged).
- **The camera** follows Connor like the original's: it trails behind him,
  keeps above the ground, and can be swung, zoomed and tilted. It doesn't
  go through walls, closed doors or hills: when a building gets between
  it and Connor it swings round to look along the way he came, and comes
  closer if it has to; dragging it into a wall only creeps it closer.
  Trees and plain objects don't stop it, as in the original. Checked in
  the logs (`camera.log`) and checked by eye by the user.
- **Ground key U:** shows the hidden ground under the water (the
  "floor"): first press both (the visible ground see-through where the
  floor differs, the floor tinted blue where lower, red where higher),
  second press the floor only, third press back to normal.
- **Items lying around:** the coins, mushrooms, crystals, weapons and
  other things the original puts in Daventry at the start (22, from its
  start-up scripts), each with its own model. Clicking one near Connor
  picks it up: it disappears into his inventory (which can't be seen yet;
  the log `items.log` counts it), and items with their own script run it
  (the farmhouse coins). Checked in the logs and by eye (the crystal).
- **Chests:** the big chest upstairs in the pub opens on the first click
  and three crystals appear in it; the second click takes them, the third
  only logs Connor's "it's empty" remark. The same for the chest of
  coins in the keep. The wizard's hidden chest stays hidden (it needs a
  Reveal potion). Checked in the logs and by eye (the pub chest, also
  that it can't be clicked through the ceiling or walls).
- **Small chests, the jar and the mushroom baskets:** the small chest by
  the pub's keg and the one at the farm drop their coins beside them and
  open, then only "it's empty"; the baskets give Connor their mushrooms
  and turn into empty baskets. Checked in the logs; not yet checked by
  eye.
- **Curiosity key H:** shows things the game hides or never uses (traps,
  sound sources, unused pictures and objects that appear later in the
  story).

## What doesn't work yet

There is nothing to play yet. Doors make no sound, Connor says nothing
when clicking (too far, locked) and doesn't turn to face the door; only
doors, chests, items and scripted objects react to clicks (swinging
signs and other things that move when the world starts are logged only; there is no inventory screen, picked-up items are only
counted in the log). Some objects have no
box in the game's table and stay walk-through (as in the original, if we
read it right). The camera doesn't swing
back behind him by itself. In water there is no damage
yet (poison, lava, swamp). Fire and ice arrow
sparks and the dragon fire are not shown (they come with fighting).
Animated objects play only the movement they start with (scripts don't
switch them yet) and make no sound. A rolling rock that runs into Connor
does nothing to him yet (in the original he stumbles and takes damage).
People and monsters are not shown,
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
| Left mouse | Use the door or chest, or pick up the item, under the mouse (play cursor) |
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
cargo run --release -- item-verify
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
