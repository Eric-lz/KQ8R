# STATUS

Hand-over in case the chat is lost. Brought up to date 2026-10-17 at the
end of A's lighting study session (Q70–Q74 and the last open lighting
points; previous: B's haze and draw distance session, M1 steps 6d–6h). Read with
`AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`, `MODLOG.md` (newest
entries at the top). The short public summary is `README.md`; the detailed
"what works" list is `docs/FEATURES.md`.

## Where we are (2026-10-17)

- **M2 (walk Connor around Daventry) is done**, including water (wading),
  the camera keeping out of walls, and leaning into turns while running.
- **M3 (Daventry comes alive) is well under way.** Done and checked by eye
  by the user: doors, falling leaves and drips, IceWorld's snow, flowing
  water, animated objects, traps (walk/run zones), items lying around and
  picking them up, chests, small chests, the jar and the mushroom
  baskets, the fish, no clicks through walls or the ground.
- **Lighting is done (M1 steps 4–5), all checked by eye by the user:**
  the ground by its light/shadow maps (F7 ×1 / ×2 / off); models
  (objects, Connor, items, leaves) by the world's ambient light and sun,
  worked out on the graphics card (`object_light.rs` + `.wesl`);
  buildings by their `.dil` light maps (`building_light.rs` + `.wesl`).
  `kq8 lighting-verify` checks the light files and all 235 `.dil` files.
- **Haze and draw distance built as the original does them** (M1 steps
  6d–6h, 2026-10-16, from A's answers to Q68/Q69, `object-lighting.md`
  rules 23, 27–40; F8 / `--haze` switch it all). Objects fade and drop out
  by distance from the camera (unchanged, checked by eye 2026-10-15).
  New: the haze colour is snapped to the palette and the background is
  palette colour `newSky n` (Daventry: both entry 17); the ground has its
  own material (`ground_haze.rs`), fades per corner to 0.99 at
  `visibleDistance` and ends there; building walls fade per surface and
  drop out at 0.99 unless `b_hazeSolid` (Daventry's tomb); particles fade
  per emitter; objects are near / far / off by distance from **Connor**
  (`draw_distance.rs`; Dead City's flame goes off at 12,100 in the log).
  Checked in the logs and **by eye by the user (2026-10-16): "everything
  looks correct just as the original"**.
- **`--renderer auto|vulkan|dx12`** (B, 2026-10-16): choose the graphics
  API at launch; `viewer.log` says which one ran. Tested in the log on
  both APIs and checked by the user (2026-10-16). Not tested on Linux or
  another graphics card. OpenGL is not built in.
- **Clicking studied, Q60–Q64 answered** (A, 2026-10-15,
  `docs/behaviour/clicking.md`, plus fixes in `items.md`, `chests.md`,
  `doors.md`): buildings take clicks by default (so walls block), the
  ground never blocks; accept lists; `Init`/`EndLoop`/`getEndLoop`;
  new items are named `NoName<n>`; the pickup sound plays on every add
  to Connor's inventory. **Next for B:** build these (adjust the Q63
  stand-in, add the `EndLoop`/`getEndLoop` calls). Not checked in the
  original yet.
- **Lighting leftovers studied, Q70–Q74 answered** (A, 2026-10-17,
  `object-lighting.md` rules 10, 14, 16, 23–26b, 28, 35, 37, 41–43;
  `items.md` 5b; `objects.md`; `terrain-lighting.md` 15): the draw
  distance ball is always a `BOUNDS.KQ` box ball, centre not turned
  (Q70); items: far mask 0, so far items are not drawn or clickable
  (Q71); point lights fully traced for models, walls and the ground,
  with `tsAttenuationEnd` an end distance on models but a fade length
  on walls and ground (Q72); haze colour entries 10–244 (the `pcmt`
  chunk is never read), measured per world; `0x1000` walls keep their
  light map; the "unlit" switch is never on; `torchlight.kq` never
  loaded (Q73); collider switch, `makeActive`, `setActiveCamera`, bit
  0x400 = lights group (Q74). New find: some models carry their own
  lights (`.kqs` light nodes, rule 26b). Follow-up the same day: light
  maps bilinear, no colour key on walls or ground, the ground's far-edge
  cut point, model lights placed by the object only. Lighting's open
  list is down to Direct3D and the flames' 11,000 vs 12,100 (both need
  the running game).
  **Next for B:** point lights (Dead City), starting with lights on
  models: `KQPointLight`s and the lights models carry (`.kqs` light nodes,
  rule 26b; e.g. the arrival portal's moving green light). **The user
  checked the original (2026-10-17):** the portal's light moves and
  changes colour on walls and ground, a smaller baked patch stays with
  dynamic lighting off, and Connor is lit green either way; our viewer
  doesn't do this yet. Then the collider switch, the two script commands,
  items' far mask.
- **Later for A:** Dead City's flames: the user's ~11,000 vs the traced 12,100 from Connor (`object-lighting.md` open question 9; small, kept for a later session); Q53 (camera follow-ups); pushable objects
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

- `crates/kq8-data` (88 unit tests): file formats. VOL archives, bitmaps,
  palettes, materials, `.kqs` shapes, terrain (surface and floor grids),
  `.dig` buildings, `.kq` text, world scripts (start-up, incl. item
  lines), `.sae` key maps, object placement rules, the `BOUNDS.KQ` box
  table, `.anm` transition lines, `envInfo.kq` ground flags, the ground's
  light and shadow maps (`lightmap.rs`), object lights (`lights.rs`),
  building light maps (`dil.rs`), emitter /
  particle catcher / ground animator settings, the item type list
  (`Invinfo.itl`), the haze rules (`haze.rs`: objects, ground, palette
  colour, `newSky`), object near / far / off rules (`objects.rs`).
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
  default); `--model NAME` shows one model; `--renderer auto|vulkan|dx12` the
  graphics API (`renderer.rs`); `--light x1|x2|off` the
  light at start; `--haze on|off` the haze at start. The full option list with descriptions is in
  `docs/FEATURES.md` ("Launch options"). Test options: `--frames N`, `--lift N`, `--at X,Y,H`,
  `--hold F,T,R[,N]`, `--jump N`, `--turn N[,1|-1]`, `--click NAME,N`
  (repeatable; skips the line-of-sight test), `--aim NAME,N` (a click
  with the line-of-sight test), `--item TYPE`. Haze and drawing: the
  ground's material `ground_haze.rs`, walls `building_light.rs`, models
  `object_light.rs` (each with its `.wesl` shader), near / far / off
  `draw_distance.rs`.
- Logs go to `kq8-artifacts/logs/`: `viewer.log`, `connor.log` and
  `camera.log` (one line per update), `doors.log` (clicks, scripts,
  events), `traps.log`, `items.log`, `effects.log`, `ground_anim.log`,
  `anim.log`, `lightmap-verify.log`, `lighting-verify.log`. `viewer.log`
  also logs the ground light under Connor and the camera, and every
  second how many objects and wall surfaces the haze leaves out and how
  many objects are near / far / off (plus every change of state).

## What works (tested)

- M0: VOL archives, images and palettes (palette entry 0 drawn black, as
  measured in the original).
- M1 (checked by eye in Daventry): ground with textures; 30 buildings; the
  objects the world scripts load; see-through objects. All 12 worlds load.
  Lighting of the ground, models and buildings, and the haze/fog (see
  "Where we are"). F7 (light) and F8 (haze) switch them; start options
  tested, the key presses not tested by a test option.
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
- Tests: 275 unit tests pass (88 + 181 + 6), 2026-10-16.

## What doesn't work yet / next

- **Next step:** B builds A's clicking answers (Q60–Q64), the Q54
  change, and the Q70–Q74 answers (point lights, collider switch,
  `makeActive` / `setActiveCamera`, items' far mask). Lighting and haze
  still lack: point lights (Dead City) and their effect on walls'
  haze (× 0.04), the brightness setting, the per-object "unlit" switch.
  The draw distance doesn't yet switch colliders (bit 0x8), and
  `makeActive` / `setActiveCamera` are not done.
- **How 6d–6h were checked by eye (user, 2026-10-16, all correct):** `cargo run -p kq8-viewer --release --
  --world Daventry`: the sky behind the far ground is the same dark blue
  as the fog; far ground fades almost fully, then ends; distant houses
  drop out wall by wall. `--world DeadCity`: walk away from the torches
  (camera close behind Connor): the flames vanish when Connor is about
  12,100 from them, and swinging the camera round without moving him
  doesn't bring them back. F8 shows everything for comparison.
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

- Open points inside `object-lighting.md` (not in QUESTIONS.md): only
  the flames' 12,100 (traced and built) vs the user's ~11,000 (open
  question 9, kept for a later session) and the Direct3D renderer (6,
  not studied; Glide only). The rest was answered 2026-10-17.

- Q60–Q64: answered 2026-10-15 (`clicking.md`); B's code still uses the
  older stand-ins (Q63: the ground and the 5 `colliderType=136` buildings
  should not block clicks).
- Q53: camera obstacle details (how the view line meets the ground and a
  line starting under water; the BSP line test; whether the new direction
  is 3D; updating only while Connor moves). Stand-ins in place.
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
- All others are answered or closed (Q70–Q74 answered 2026-10-17, not
  built yet; Q68–Q69 answered and built 2026-10-16; Q65–Q67 answered and built, the
  lighting session 2026-10-14/15; Q55 answered in
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
