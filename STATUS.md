# STATUS

Hand-over in case the chat is lost. Brought up to date 2026-10-16 at the
end of B's session that built A's answers to Q54, Q60–Q64 and Q70–Q74
(M3 step 3g, M1 steps 7a–7c, 7f). Read with `AGENTS.md`,
`docs/DESIGN.md`, `docs/SESSIONS.md`, `MODLOG.md` (newest entries at the
top). The short public summary is `README.md`; the detailed "what works"
list is `docs/FEATURES.md`.

## Where we are (2026-10-16)

- **M2 (walk Connor around Daventry) is done.** **M3 (Daventry comes
  alive) is well under way** (doors, emitters, flowing water, animated
  objects, traps, items, chests, scripted objects; checked by eye).
- **Lighting, haze and draw distance** (M1 4–6h) are built as the
  original does them and were checked by eye by the user (2026-10-16).
- **New this session (B, 2026-10-16), from A's answers; in the logs and
  unit tests, and checked by eye by the user (2026-10-16: "looks good",
  the portal lights Connor):**
  - **Clicking (M3 3g, `clicking.md`):** the click line hits buildings
    (walls, floors, ceilings take and silently swallow the click; the
    church, cave and wizard's house don't, `colliderType=136`) and no
    longer the ground; accept lists (`a,b,c`, `INVITEM_All`, `none`,
    `KQObject::setAccepts`); `EndLoop` arrives one update later through
    the event queue, doors get it too, `getEndLoop` relays; `deleteObject`
    works on every object; new items are named `NoName<n>`; the pickup
    sound is logged on every add to Connor's inventory; a false text
    `test ==` leaves −1 (Q54).
  - **Draw distance (M1 7a, 7b):** balls from `BOUNDS.KQ` (not the
    model); items not drawn when far; the collider switch (far, off and
    hidden objects are not solid, not clickable, the camera passes);
    `setActiveCamera` / `makeActive`. **Behaviour change:** hidden
    objects no longer block Connor (they did before, by our older
    reading).
  - **Point lights on models (M1 7c):** Dead City's seven `KQPointLight`s
    now load, and the lights models carry (`.kqs` light nodes: the portal,
    torches, coals; Daventry's lamps, candles, chandeliers, fireplaces)
    light nearby models, Connor included, per model part on the graphics
    card (`scene_lights.rs`, `object_light.wesl`). By Dead City's arrival
    portal the log shows the circling green light reaching Connor and
    changing colour every second.
  - **Q73 leftovers (M1 7f):** checked; only the ground's light map is
    not bilinear yet.
- **Later for A:** Q75–Q77 (new, small); Dead City's flames (11,000 vs
  12,100, open question 9); Q53 (camera follow-ups); pushable objects
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

- `crates/kq8-data` (89 unit tests): file formats. VOL archives, bitmaps,
  palettes, materials, `.kqs` shapes, terrain (surface and floor grids),
  `.dig` buildings, `.kq` text, world scripts (start-up, incl. item
  lines), `.sae` key maps, object placement rules, the `BOUNDS.KQ` box
  table, `.anm` transition lines, `envInfo.kq` ground flags, the ground's
  light and shadow maps (`lightmap.rs`), object lights (`lights.rs`),
  building light maps (`dil.rs`), emitter /
  particle catcher / ground animator settings, the item type list
  (`Invinfo.itl`), the haze rules (`haze.rs`: objects, ground, palette
  colour, `newSky`), object near / far / off rules (`objects.rs`), the
  lights a `.kqs` carries per frame (`shape.rs` `sequence_lights`).
- `crates/kq8-core` (188 unit tests; named `kq8-game` until 2026-10-13):
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
  `active.rs` (doors, chests, items, scripted objects, buildings as click
  targets, accept lists, `EndLoop`/`getEndLoop`, `deleteObject`, the
  collider switch, events and scripts together), `emitter.rs` (particles and catchers), `ground_anim.rs`
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
  `object_light.rs` (each with its `.wesl` shader), near / far / off and
  the collider switch `draw_distance.rs`, point lights on models
  `scene_lights.rs`.
- Logs go to `kq8-artifacts/logs/`: `viewer.log`, `connor.log` and
  `camera.log` (one line per update), `doors.log` (clicks, scripts,
  events), `traps.log`, `items.log`, `effects.log`, `ground_anim.log`,
  `anim.log`, `lightmap-verify.log`, `lighting-verify.log`. `viewer.log`
  also logs the ground light under Connor and the camera, and every
  second how many objects and wall surfaces the haze leaves out and how
  many objects are near / far / off (plus every change of state), every
  collider switch, and the scene's lights (how many, what the nearest
  gives Connor).

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
  - Clicks: walls, floors and ceilings in front take the click (and
    swallow it), the ground doesn't (A's Q63 answer, 2026-10-16; checked
    in the logs only, the older stand-in was checked by eye).
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
- New 2026-10-16, checked in the logs and unit tests only (see "Where we
  are"): accept lists, `EndLoop` through the queue, `getEndLoop`,
  `deleteObject` for everything, `NoName<n>`, `BOUNDS.KQ` balls, items'
  far mask, the collider switch, `setActiveCamera` / `makeActive`, point
  lights and carried lights on models.
- Tests: 285 unit tests pass (89 + 188 + 8), 2026-10-16.

## What doesn't work yet / next

- **Done after the check (2026-10-16):** M1 7e-0, the ground's light map
  is one bilinear picture on the card (checked by eye by the user).
- **Next step (B):** 7e (point light patches painted into the ground's
  picture, `object-lighting.md` rule 26). 7d (walls, rule 16) waits for
  A's answer to Q78. Planned in `docs/DESIGN.md` M1 step 7.
- **Checked by the user (2026-10-16), all good:** `cargo run -p kq8-viewer
  --release -- --world DeadCity --at 182213,42600,3.1416`: Connor stands
  by the arrival portal; his side facing the portal should get a green
  light that circles and changes colour, as in the original (walls and
  ground don't get it yet). `--world Daventry`: click a house wall in
  reach (nothing happens, nothing in the way), and a door or item behind
  the church or wizard's house wall (the click goes through those two).
- Clicking: only doors, chests, items, scripted objects and buildings
  react. No sound, no remarks (Connor says nothing; remarks are logged),
  Connor doesn't turn to face what he clicks. No inventory screen (M4).
  `getEndLoop Connor …` (`bricks.cs`) is recorded but Connor's scripted
  movements don't exist yet. `checkInventory` by id number (Q62 b) not
  built.
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
  walk-through. Hidden, far and off objects are not solid (rule 41; not
  checked in the original). A deleted or far building's walls stay
  solid.
- Loops (`walk`, `run`) still spread their step evenly over the loop
  (Q24); one-shot animations use their frame steps.
- Lighting still lacks: point lights on walls and the ground (7d, 7e),
  the ground's light map bilinear (7e-0), the brightness setting.
- Not drawn yet: people and monsters (`.anm` characters; orange cubes with H), the
  teleport pad's look (Q14b), arrow sparks and the dragon fire (they come
  with fighting).
- Nothing yet of: sound, music, talking, inventory screen, fighting,
  stats, story, saving, travel between worlds.

## Open questions (`docs/behaviour/QUESTIONS.md`)

- Open points inside `object-lighting.md` (not in QUESTIONS.md): only
  the flames' 12,100 (traced and built) vs the user's ~11,000 (open
  question 9, kept for a later session) and the Direct3D renderer (6,
  not studied; Glide only). The rest was answered 2026-10-17.

- New from B (2026-10-16), small: Q75 (reach to a clicked building),
  Q76 (the draw-distance ball of a building), Q77 (which light flag bit
  means "point"; a `KQPointLight`'s place in the near / far / off test;
  the 442 vs 174 count of shapes with lights).
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
- All others are answered or closed (Q54, Q60–Q64, Q70, Q71, Q74 built
  2026-10-16; Q72 built for models, walls and ground next; Q73 checked;
  Q68–Q69 answered and built 2026-10-16; Q65–Q67 answered and built, the
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
