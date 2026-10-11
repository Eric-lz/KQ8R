# King's Quest: Mask of Eternity reimplementation in Rust

Unofficial fan project, not affiliated with or endorsed by the game's
owners. Contains no game files: you need your own copy of the game.
The code is written with an AI coding agent (Claude Code); see
[CREDITS.md](CREDITS.md).

- Game version: 1.03 (GOG)
- Game files in ./MASK/ folder (read only, never committed)
- Using NO-CD patch ([KQ8FixWin11](https://github.com/SegMash/KQ8FixWin11))

## What works

- Reading the game's packed world archives (`data/<World>/*.vol`):
  all 48 archives, 14,444 files, decompress and match the installed copies
  byte-for-byte. Tested on Windows 11 with the GOG 1.03 install.
- Reading images (`.pbm`, `.bmp`, `.dib`) and palettes (`.ppl`, `.pal`):
  all 6,495 images and 50 palettes in the game load without errors, and
  any of them can be saved as a PNG to check by eye.
  Not done yet: transparency, the palettes' lighting/haze tables, and image
  arrays (`.pba`).

- Reading 3D models (`.kqs`) and material lists (`.dml`): all 4,573 models
  and 25 material lists load without errors (4 old-format tomb models are
  skipped).
- Reading every world's ground (heights, materials, settings) and asking
  "how high is the ground here?". Checked against object positions: in
  Daventry 11 objects stand exactly on the computed ground.
- Showing a world's ground with its textures (`kq8-viewer --terrain`), with
  a camera you can fly around. Checked by eye (2026-10-11): Daventry's
  ground looks right (shape, textures, joins between squares).
- Reading every building file (`.dig`, 153 in the game) and drawing
  Daventry's 30 buildings in the ground view. Checked by eye: placement,
  walls, roofs, insides and textures look right (no building lighting yet).
- Placing the objects Daventry's start-up scripts load (347 of them:
  furniture, doors, chests, signs, trees...), turned the way the game turns
  them, animated ones at their starting frame, see-through ones (waterfalls,
  torches) see-through. Checked by eye. The 79 that the scripts place "on
  the ground" (z −1/−2: tombs, many trees) now stand on it (not checked by
  eye yet). Press H to show the 77 objects that have nothing to draw as
  small cubes: green = invisible in the game (traps, sounds, paths,
  leaf emitters), orange = characters we cannot draw yet (`.anm` files).
  H also shows 16 unused objects: in the game's files but not loaded at
  the start (pictures `tap01/02/04`, `pubart1`, the potion, the pyramid...).
  A small text in the top-left corner says whether they are shown.
  H also shows objects the game hides at the start (`hidden=1`, e.g. the
  wizard's chest in Daventry, Barren's docks and lava tiles).
- Start-up scripts' variables, `getLoc` and `move` are followed (Barren's
  forge objects, DeadCity's torch flames). Checked by eye.
- M2 started: Connor stands at his start point by the cottage in his
  standing pose, with gravity on (15 updates per second). Test keys: G
  / B lift him 5,000 / 2,000 to watch him
  fall (5,000 kills him; the log says so). Checked by eye.
- Connor walks (Up), steps back (Down), turns (Left/Right) and runs (End
  switches run on/off; it starts on), with his animations and the speeds
  of the original (walk 1,127, run 2,518 units/s, measured). No start/stop animations yet; running into a steep
  hillside looks wrong until slopes are done (step 2c): he slides far up
  it, then slowly back. Checked by eye.
- The game camera follows Connor as in the original: it trails him on a
  leash (same distance walking and running), stays 1,400 above the ground, right mouse
  swings and zooms, Page Up/Down tilt, Home resets, F11 keeps a fixed
  distance. F switches to the free flying camera and back. Not yet: it
  does not swing back behind him by itself, nor avoid walls. Checked by
  eye (mouse directions corrected after the check).
- The other 11 worlds open the same way (`--world <name>`) and their
  objects load by the same rules; only Barren's lookouts were looked at
  by eye so far (not yet after the fix that places them).
  Not checked by eye yet. Checked by eye (before the last fixes, 104 objects):
  objects are where expected (the pub's keg, the mill wheel, doors,
  paintings; then the sign, bell, baskets and closed doors). Buildings and
  monsters are missing, so the village still looks mostly empty, and
  waterfalls/streams are drawn solid instead of see-through.
- Showing one model in a window with its textures (`kq8-viewer`), with a
  camera you can turn with the mouse. Tested: the window opens and draws
  (about 6 ms per frame). Checked by eye (2026-10-11) on two models: the
  small chest (lid closed, wood-and-metal texture) and the alchemist's
  sign look right. Only the first frame of each model's animation is
  shown, so the swinging sign appears tilted.

## What doesn't work yet

Everything else. There is nothing to play yet: no buildings or objects on
the ground, no lighting,
no animation, no sound, no game logic. Models are drawn without lighting,
because those rules are not known yet.

## Try it

```sh
cargo run --release -- vol-verify
cargo run --release -- bitmap-verify
cargo run --release -- bitmap-png GAME/8Gui/daventry.pbm --palette GAME/8Gui/ldaventr.ppl
cargo run --release -- shape-verify
cargo run --release -- terrain-verify
cargo run --release -- building-verify
cargo run --release -p kq8-viewer -- smlchest.KQS --world Daventry
cargo run --release -p kq8-viewer -- --terrain --world Daventry
```

The last one writes `kq8-artifacts/png/daventry.png`. Files inside an
archive are named `vol:<World>/<archive>/<NAME>`.

Use `--game <folder>` or the `KQ8_GAME_DIR` environment variable if the game
is not in `./MASK/`. Logs are written to `kq8-artifacts/logs/`.

## More

- [docs/DESIGN.md](docs/DESIGN.md): how the project works and the plan
- [docs/RESEARCH.md](docs/RESEARCH.md): existing tools and documentation
- [CREDITS.md](CREDITS.md): sources and licenses
- [MODLOG.md](MODLOG.md): change log
