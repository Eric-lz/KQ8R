# STATUS

Hand-over in case the chat is lost. Brought up to date 2026-10-14 after
commit `6586d0d` (terrain lighting done, object lighting studied), plus
the uncommitted viewer change (ground view is the default, `--model`). Read with
`AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`, `MODLOG.md` (newest
entries at the top). The short public summary is `README.md`; the detailed
"what works" list is `docs/FEATURES.md`.

## Where we are (2026-10-14)

- **M2 (walk Connor around Daventry) is done**, including water (wading),
  the camera keeping out of walls, and leaning into turns while running.
- **M3 (Daventry comes alive) is well under way.** Done and checked by eye
  by the user: doors, falling leaves and drips, IceWorld's snow, flowing
  water, animated objects, traps (walk/run zones), items lying around and
  picking them up, chests, small chests, the jar and the mushroom
  baskets, the fish, no clicks through walls or the ground.
- **M1 step 4, ground lighting, is done** (B, 2026-10-14): the ground is
  drawn with the game's light/shadow maps at ×1 (no doubling), which the
  user checked by eye against the original (dark and muddy, as the
  original); A confirmed ×1 in the exe (Q66). F7 cycles ×1 / ×2 / off.
- **Object and building lighting is studied** (A, 2026-10-14,
  `docs/behaviour/object-lighting.md`): models lit by the hardware light
  files (Daventry ambient 0.3 + sun 0.1, so 30–42% of the texture),
  buildings by their `.dil` light maps, haze, brightness setting.
  The user checked in the original: objects as dark as the grass with the
  sunny side slightly lighter; haze fades objects with distance from the
  camera; Connor's hut inside darker but reasonably lit.
- **Object and building lighting done** (B, 2026-10-15, M1 step 5):
  light files and `.dil` light maps read and checked (`kq8
  lighting-verify`, PASS; layout corrected by Q67); models (objects,
  Connor, items, leaves) lit on the graphics card (`object_light.rs`,
  `.wesl`), checked by eye by the user; buildings lit by their light
  maps (`building_light.rs`, `.wesl`), checked by eye by the user.
  **Haze done** (B, 2026-10-15, M1 step 6, F8 on/off): objects by the
  note's rules; ground, walls and background fogged as a stand-in after
  the user's check ("dense, dark fog hides everything"; Q68). Checked
  by eye by the user.
- **Clicking studied, Q60–Q64 answered** (A, 2026-10-15,
  `docs/behaviour/clicking.md`, plus fixes in `items.md`, `chests.md`,
  `doors.md`): buildings take clicks by default (so walls block), the
  ground never blocks; accept lists; `Init`/`EndLoop`/`getEndLoop`;
  new items are named `NoName<n>`; the pickup sound plays on every add
  to Connor's inventory. For B after lighting: adjust the Q63 stand-in
  and add the `EndLoop`/`getEndLoop` calls. Not checked in the original
  yet.
- **Later for A:** Q53 (camera follow-ups), Q54; pushable objects
  (`KQBoulder`); Connor against monsters; Q27.

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* that reads the player's
own install at runtime. Roadmap M0–M7 in `docs/DESIGN.md`. Current
milestone: **M3, Daventry comes alive**.

## Setup

- Game: GOG.com release, "Version 1.3" (`MASK/Version.txt`; `MASK/readme.txt`
  says 1.0.0.3), in `./MASK/` (read only, never committed). KQ8FixWin11
  installed there (Windows 11 fix, no-CD).
- Rust stable (MSVC), Bevy 0.20 for the viewer (`crates/kq8-viewer`).
- Two-session workflow (`docs/SESSIONS.md`): **A** studies `Mask.exe` in
  Ghidra and writes `docs/behaviour/*.md` (no code); **B** writes code from
  those notes and the data, never sees the exe.
- Ghidra 12.1.4 at `C:\Tools\Ghidra`, project `re/ghidra/KQ8`, program
  `re/Mask.exe`; GhidraMCP 2.0 via `.mcp.json` (session A only).
  `re/` (Ghidra project, scratch notes, throwaway scripts) is never
  committed.
- Measuring the running original: a read-only memory reader
  (`re/dbg/mem_fall.py`; start the game normally, run
  `python -I re\dbg\mem_fall.py <log>`, play, stop it **by its own
  process only**). It gave the fall speed, slopes, movement, turning,
  jumps, walls and floor heights. Running the game under the x32dbg
  debugger with stops did not work (the game crashed or froze when
  resumed from a stop); that route is not in use.
- Builds: close the viewer before building, so no side build folder
  (`target/side`) is needed. `cargo clean --profile dev` now and then
  keeps `target/` small.

## Code layout

- `crates/kq8-data` (84 unit tests): file formats. VOL archives, bitmaps,
  palettes, materials, `.kqs` shapes, terrain (surface and floor grids),
  `.dig` buildings, `.kq` text, world scripts (start-up, incl. item
  lines), `.sae` key maps, object placement rules, the `BOUNDS.KQ` box
  table, `.anm` transition lines, `envInfo.kq` ground flags, the ground's
  light and shadow maps (`lightmap.rs`), object lights (`lights.rs`),
  building light maps (`dil.rs`), emitter /
  particle catcher / ground animator settings, the item type list
  (`Invinfo.itl`).
- `crates/kq8-core` (181 unit tests; named `kq8-game` until 2026-10-13):
  game rules without drawing. Connor: `connor_mover.rs` (steps, gravity,
  landing, Fall, fall death, building and object hits, Jump),
  `connor.rs` (keys → movement, turning, banking, steps from the
  animation, start/stop transitions, crash, wall blocks, jumps, wading),
  `connor_update.rs` (one update of Connor), `collide.rs` (ball stack,
  building BSP sweep, object box sweep, type/mask rule, line tests),
  `water.rs` (the wading test). World: `camera.rs` (the following camera
  with the obstacle check), `script.rs` (the `.cs` script runner),
  `events.rs` (`sendEvent` queue), `door.rs`, `chest.rs`, `item.rs`
  (item objects and inventories), `trap.rs`, `click.rs` (picking and
  reach), `named_anim.rs` (an object's named animations, e.g. `open`),
  `active.rs` (doors, chests, items, scripted objects, events and scripts
  together), `emitter.rs` (particles and catchers), `ground_anim.rs`
  (flowing ground textures), `anim.rs` (looping animations of objects).
  Runs at a fixed **15 updates per second**; the ground animators run on
  continuous game time.
- `crates/kq8-cli`: `kq8` checks (`vol-verify`, `bitmap-verify`,
  `bitmap-png`, `shape-verify`, `terrain-verify`, `building-verify`,
  `item-verify`, `lightmap-verify`, `lighting-verify`) and `kq8 text <file>` to print a game text file
  (`vol:<World>/<volume>/<NAME>` for a file inside an archive).
- `crates/kq8-viewer`: Bevy viewer. `[--world <W>]` shows a world (the
  default); `--model NAME` shows one model; `--light x1|x2|off` the
  ground light at start. The full option list with descriptions is in
  `docs/FEATURES.md` ("Launch options"). Test options: `--frames N`, `--lift N`, `--at X,Y,H`,
  `--hold F,T,R[,N]`, `--jump N`, `--turn N[,1|-1]`, `--click NAME,N`
  (repeatable; skips the line-of-sight test), `--aim NAME,N` (a click
  with the line-of-sight test), `--item TYPE`.
- Logs go to `kq8-artifacts/logs/`: `viewer.log`, `connor.log` and
  `camera.log` (one line per update), `doors.log` (clicks, scripts,
  events), `traps.log`, `items.log`, `effects.log`, `ground_anim.log`,
  `anim.log`, `lightmap-verify.log`, `lighting-verify.log`. `viewer.log` also logs the ground
  light under Connor and the camera every second.

## What works (tested)

- M0: VOL archives, images and palettes (palette entry 0 drawn black, as
  measured in the original).
- M1 (checked by eye in Daventry): ground with textures; 30 buildings; the
  objects the world scripts load; see-through objects. All 12 worlds load.
  Ground lighting: hill shadows from the light/shadow maps at ×1, checked
  by eye against the original; worlds without maps (Barren, Gnome,
  temples) at full brightness; F7 cycles ×1 / ×2 / off (start option
  tested, key press not tested by a test option).
- Viewer helpers: H shows invisible, unused and hidden objects; U shows
  the floor grid under the water (key presses not tested by a test
  option); camera position shown bottom left; F switches to a free flying
  camera.
- M2 (all checked by eye unless marked): Connor at his start point; walk,
  run, back, turn (arrows, End = run on/off); slopes and falls with the
  measured gravity; buildings and objects are solid (stop, slide, steps,
  floors, crash when running head-on, the wall blocks); start and stop
  animations; standing, running and backward jumps; a held turn key keeps
  turning him at a wall (key repeat; checked in the logs only); water:
  Connor and objects stand on the floor grid under the water, he wades
  (`waterwalk`/`waterrun`) where the water is deep enough, and can do a
  standing jump while wading (that jump: checked in the log only).
  Banking: he leans into turns while running (up to 0.18 rad, as traced;
  checked by eye at 0.15, the 0.18 end value in the logs only), also while
  wading, and straightens up in a jump (Q59, the user's check).
- Camera (checked by eye): follows Connor on a leash (1,500–5,000),
  keeps at least 1,400 above the ground, right mouse swings and zooms
  (Shift: tilt), Page Up/Down tilt, Home resets, F11 fixed distance.
  It keeps out of walls: buildings, closed doors and hills block its
  view, trees and plain objects don't; when blocked it swings round to
  look along the way Connor came and comes closer if it has to; dragging
  it into a wall only creeps it closer.
- M3 (checked by eye unless marked):
  - Doors: left click with the mouse opens them within reach (4,700);
    from too far Connor's "too far" remark is only logged; the open door
    is out of the doorway but still solid; the castle keep's doors open
    only with the key (test option `--item`).
  - Clicks only reach what the camera can see: walls, closed doors and
    the ground block them (Q63 stand-in; matches the user's check).
  - Emitters: Daventry's falling leaves and stream drips; leaves vanish
    inside the cottage (particle catcher); IceWorld's snow follows Connor
    and stays out of buildings.
  - Flowing ground textures (river, stream, Swamp, DeadCity blood) with
    each square keeping its own orientation, timed 0.3 s from each change.
  - Animated objects play their starting movement (fountain, windmill,
    mill wheel, signs, torches, fish). Gnome's three rolling rocks roll
    out and come back each cycle, and their collision boxes move with
    them. Gnome's pushable `boulder` stands still, as in the original.
  - Traps: Daventry's 52 trap boxes switch on and off and run their
    scripts; walk and run zones change Connor's pace and the camera's
    leash (1,000–2,500 inside). Music, ropes, teleporters and cutscenes
    are only logged.
  - Items: the 22 items Daventry puts out at the start are drawn and can
    be picked up (counted in `items.log`); item scripts run (farm coins).
  - Chests: the pub's big chest and the keep's chest of coins (open,
    take, "empty"); the wizard's hidden chest stays hidden.
  - Scripted objects: the small chests, the jar and the mushroom baskets;
    clicking a swimming fish makes it disappear (sound and health only
    logged); other scripted things (signs, stone villagers) run their
    scripts, their remarks logged.
- Tests: 269 unit tests pass (84 + 181 + 4), 2026-10-15.

## What doesn't work yet / next

- **Next step:** the user checks the haze by eye; Q68 for A (ground,
  buildings, particles, the misspelled `hazeMulitplier`); then A's
  clicking answers (Q60–Q64).
- Clicking: only doors, chests, items and scripted objects react. No
  sound, no remarks (Connor says nothing; remarks are logged), Connor
  doesn't turn to face what he clicks. No inventory screen (M4).
  `deleteObject` on placed objects that are not clickable with a script
  is a stub (Q64).
- Camera, still missing: the original's own automatic swing back behind
  Connor when he turns (not built; while walking the leash already makes
  the camera trail behind him), first person (F2), shake and cutscene
  cameras. Our camera updates every tick; the original only while Connor
  moves or the camera is used (Q53 d).
- Animated objects play only the movement they start with (scripts can't
  switch them yet, `KQAnimPlayEvent` to them is only logged) and make no
  sound. A rolling rock that runs into Connor does nothing to him (in
  the original he stumbles and takes damage; rule in `connor-collision.md`,
  Q55, not built yet).
- Water: no damage yet (poison, lava, swamp).
- Collision, not done: Connor against monsters (none drawn), pushing
  objects, ropes; objects with no box in the game's table stay
  walk-through. Hidden objects are solid, as traced (not checked in the
  original).
- Loops (`walk`, `run`) still spread their step evenly over the loop
  (Q24); one-shot animations use their frame steps.
- Not drawn yet: moving lights (Dead City's point lights; Daventry has none), people and monsters (`.anm` characters; orange cubes with H), the
  teleport pad's look (Q14b), arrow sparks and the dragon fire (they come
  with fighting).
- Nothing yet of: sound, music, talking, inventory screen, fighting,
  stats, story, saving, travel between worlds.

## Open questions (`docs/behaviour/QUESTIONS.md`)

- Q65, Q66 (ground light scale, ×1 or ×2): answered 2026-10-14 (×1).
- Q67 (`.dil` layout): answered 2026-10-15 in `object-lighting.md`. B's
  readings are right; the five 12-byte files are never loaded by the game;
  Castle D's 96 no-light-map surfaces have material 255 (no material):
  don't draw them.
- Open points inside `object-lighting.md` (not in QUESTIONS.md): wall
  drawing not traced (walls assumed to use only their light map), the
  per-object "unlit" switch, point-light range, `0x1000` building
  materials (full bright or not).

- Q60–Q64: answered 2026-10-15 (`clicking.md`); B's code still uses the
  older stand-ins (Q63: the ground and the 5 `colliderType=136` buildings
  should not block clicks).
- Q53: camera obstacle details (how the view line meets the ground and a
  line starting under water; the BSP line test; whether the new direction
  is 3D; updating only while Connor moves). Stand-ins in place.
- Q54: what a false text `test` and `set`/`echo` leave in
  `$Console::Return`. Stand-ins in place.
- Q48: partly answered (emitter-vs-catcher order in a tick not settled;
  no visible difference).
- Q41: jumps: (a) jump during the wait before a stop, (c) the 250 limit,
  (d) a running jump's landing with only a turn key held. Stand-ins.
- Q27: camera details: mouse scale, what the tilt turns, the automatic
  swing back behind Connor, the start place and Daventry's `follow none`,
  when `walk.cs`/`run.cs` run.
- Q24: partly answered (steps spread within each frame; not compared
  frame by frame).
- Q29 (e): why Connor climbs so little into a steep hillside (the rest
  answered).
- Q30 (b): what keeps Connor on a slope going downhill; "likely answered"
  by the stronger gravity, but the re-check A asked for was not recorded.
- Q41 is listed above; Q4 and Q14 (b) below.
- Q42: entry 0 in textures' own colour tables (stand-in black; no visible
  difference).
- Q14b: the teleport pad's see-through look.
- Q4: missing texture names.
- All others are answered or closed (Q55 answered in
  `connor-collision.md`, not built yet; Q56 answered by A's trace and
  built 2026-10-13; Q59 closed by the user's check).

## Files that matter

- `docs/behaviour/*.md`: the rules (session A).
- `docs/behaviour/QUESTIONS.md`: B's questions and A's answers.
- `docs/provenance/README.md`: which note each piece of code follows, and
  how it was checked.
- `docs/DESIGN.md`: architecture, roadmap, step plans.
- `docs/FEATURES.md`: what works, in detail, and how each was checked.
- `MODLOG.md`: every change with its test.

## Prompt for a fresh session

See  [docs/SESSIONS.md](docs/SESSIONS.md)
