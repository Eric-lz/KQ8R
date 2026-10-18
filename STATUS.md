# STATUS

Hand-over in case the chat is lost. Brought up to date 2026-10-17 at the
end of B's sound-effects session (M3 3h). Read with `AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`,
`MODLOG.md` (newest entries at the top). The short public summary is
`README.md`; the detailed "what works" list is `docs/FEATURES.md`.

## Where we are (2026-10-17)

- **M2 (walk Connor around Daventry) is done.** **M3 (Daventry comes
  alive) is well under way** (doors, emitters, flowing water, animated
  objects, traps, items, chests, scripted objects, clicking; checked by
  eye).
- **Lighting, haze and draw distance** (M1 4–7) are built as the original
  does them and checked by eye by the user, including point lights on
  building surfaces, F9 (dynamic lighting on/off) and `--dynamic on|off`.
  Q80 answered 2026-10-17 (A counted differently; B's counts are right).
- **New 2026-10-17 (B, M3 3h): sound effects play** (`sound.md`):
  - `kq8 aud-verify` decodes all 1,750 `.aud` and 2,849 speech files;
    all 22 checks against the note match. `--wav <file>` writes one to
    `kq8-artifacts/wav/` to listen to.
  - Audio library chosen by the user: **cpal** (the sound card) plus our
    own mixer (new crate `kq8-audio`), so the note's rules apply exactly.
  - Playing: door, chest and scripted-object `playsound` cues (placed,
    5,000 / 25,000), item pickups and `KQSound::play` (plain), and
    Daventry's looping sound objects (water wheel, mill, waterfall,
    windmill, fountain): loudness from Connor's distance with the
    original curve, left/right from the camera, sliders from
    `Options.cs` (effects 0.14), mix limit 8, loops paused beyond max
    and switched off beyond 27,100. Every start and stop is in
    `kq8-artifacts/logs/sound.log`. Viewer option `--sound off` mutes.
  - **Checked by ear by the user (2026-10-17):** chest, pickup, door
    (louder on the left as expected), doors and items in normal play.
  - **Footsteps (3h-4):** Connor's animation cues now fire (doors.md
    rule 9, the wrap rule of `objects.md`); a `footstep` cue plays
    dirt / grass / rock / water by the ground under him (terrain
    flags, or a building's `footsoundType`), a `playsound` cue a placed
    sound at him. Stand-ins asked as **Q81** (house floors without
    `footsoundType` = dirt; objects = the terrain under him; the
    backward loop's wrap; in the air = the last ground). Checked by ear
    by the user (2026-10-17): like the original (stone on the bridge,
    water in the river, house floors like the ground outside).
- **New 2026-10-17 (A): world music studied** (Q83 answered;
  `sound.md` section "World music", rules 57–57l): the list of tunes,
  push / pop / play with 1.4 s crossfades, the 0.2 game-clock delay of
  script pushes and pops, the world tune at full loudness on start,
  pause, leaving a world. Footsteps (Q81 a, b, d) answered
  in rules 39–39a.
- **New 2026-10-17 (B, M3 3h-5): world music plays** (`kq8-core`
  `music.rs`, fades in `sound.rs`): `forest.aud` from the first frame at
  0.14; the music zones push and pop their tunes 0.2 s after Connor
  enters or leaves, with 1.4 s cross-fades, and `forest.aud` restarts
  from its beginning. Checked in the logs with the wizard's zone
  (`mwiztrap`) and by ear by the user (2026-10-17). Not built: pause, videos,
  world changes (rules 57h–57j).
- **New 2026-10-17 (A, cleanup): Q80, Q81 (c), Q82 answered; Q4 tried
  again.** Q82: A had misread the crash list in `connor-physics.md`
  ("Walls", "Jumping" rule 5, now corrected): `jump` crashes like
  `runJump` (not `PreBirth`), and during a jump any surface with normal z
  under 0.8 is head-on. B's stand-in is nearly right; small differences
  listed in QUESTIONS.md Q82. Q81 (c): B's stand-in is right
  (`objects.md` rule 4). Q80: see `object-lighting.md` "How to check".
  Q4 closed: none of the 29 missing texture names is ever drawn in play.
- **New 2026-10-17 (A): animated objects controlled by scripts studied**
  (`docs/behaviour/anim-scripts.md`): `KQAnimPlayEvent` (waiting movement,
  starts at once if the current one can be cut short or has ended, else
  at its end; starts from its own `loop=` line; unknown name = movement 0),
  `nextAnimName` = the waiting movement, the order at a movement's end
  (waiting movement first, then `EndLoop` / `Cue 300`), `KQAnimOnce`
  (gets `Die` at its end; Daventry's barrels and the alchemist's boards),
  `setDeleteWhenDone`, `pauseAnim` (toggles), Connor's
  `setState … special` word list, single quotes / `;` in scripts, and
  `KQStatsChangeRequestEvent` adds (5th number = experience). Ready for B.
  Daventry's bricks puzzle still needs its `Push` trigger (pushables study).
- **New 2026-10-17 (A): pushing studied** (`docs/behaviour/pushing.md`):
  type bit 0x100 = pushable; Connor pushes when moving forward on the
  ground with empty hands, facing it within 60°, on the across pass.
  Scripted pushables (also 0x10000: Daventry's bricks, coffin lid, urn)
  get a `Push` script call each pushing update; physical ones (barrels,
  the hay bale) move with his step, only while their height stays the
  same, and Connor follows at 0.95. The bricks puzzle is now fully
  described. Ready for B. The user's checks in the original (hay bale,
  barrels on flat ground only, no push with a weapon or backwards) all
  match.
- **New 2026-10-17 (A): Q84–Q93 answered** (B's questions from 3i–3k):
  - Q84 (`connor-physics.md`): a steep jump hit that isn't head-on does
    not end the jump; Connor **does turn** (1.75 rad/s) while `crash`
    plays.
  - Q85/Q86 (`anim-scripts.md` rules 3, 19–20): the six `flags=8`
    objects with no `currAnimName` never get the random start (in step);
    a start doesn't use up the update; levels: strictly more than, once
    per event, cap then full heal; **second stat +5, third +2** (the note
    had them swapped); `Die` always has `$3`.
  - Q87/Q88/Q91 (`pushing.md`, `connor-physics.md`): box-shaped objects
    (barrels) are never stopped by walls or objects and are only raised
    by the ground (they float downhill); the hay bale slides along walls;
    every object touched is told, a tie keeps the later-tested one.
  - Q89/Q93 (`anim-scripts.md` rules 21–27, `camera.md`): `handsOff`
    in full (re-sends held keys at 0); `setThirdPerson` is a paired
    first-person-key toggle; `move KQCamera` angles (rx tilt, rz heading
    from +Y); the watermill cutscene waits for `KQConner::walkTo` and
    `pointAndCue`, which we don't have yet.
  - Q90/Q92 (`objects.md`, `pushing.md` rule 11): `loadKQ` while playing
    = as at the start plus an `Init` call; same name = fatal. The pop-up:
    the `Push` script runs inside Connor's update, so the original drops
    him onto the floor in the same update; the visible table and bricks
    are part of his model then.
  - Ready for B. Still to check in the original: barrels pushed downhill
    or into a wall, the hay bale at an angle against a wall.
- **Later for A:** Dead City's flames (11,000 vs 12,100, open
  question 9); Q53 (camera follow-ups); pushable objects (`KQBoulder`);
  Connor against monsters; Q27.

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

- `crates/kq8-data` (105 unit tests): file formats. Sound files and
  their lookup (`aud.rs`). VOL archives, bitmaps,
  palettes, materials, `.kqs` shapes, terrain (surface and floor grids),
  `.dig` buildings, `.kq` text, world scripts (start-up, incl. item
  lines), `.sae` key maps, object placement rules, the `BOUNDS.KQ` box
  table, `.anm` transition lines, `envInfo.kq` ground flags, the ground's
  light and shadow maps (`lightmap.rs`), object lights (`lights.rs`),
  building light maps (`dil.rs`), emitter /
  particle catcher / ground animator settings, the item type list
  (`Invinfo.itl`), the haze rules (`haze.rs`: objects, ground, palette
  colour, `newSky`), object near / far / off rules (`objects.rs`), the
  lights a `.kqs` carries per frame (`shape.rs` `sequence_lights`), point
  lights on building surfaces (`wall_light.rs`).
- `crates/kq8-core` (206 unit tests; named `kq8-game` until 2026-10-13):
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
  (flowing ground textures), `anim.rs` (looping animations of objects),
  `sound.rs` (sound rules: groups, distance, balance, mix limit, sound
  objects; no sound card).
- `crates/kq8-audio` (5 unit tests): our mixer and the sound card (cpal).
  Runs at a fixed **15 updates per second**; the ground animators run on
  continuous game time.
- `crates/kq8-cli`: `kq8` checks (`vol-verify`, `bitmap-verify`,
  `bitmap-png`, `shape-verify`, `terrain-verify`, `building-verify`,
  `item-verify`, `lightmap-verify`, `lighting-verify`, `aud-verify
  [--wav <file>]`) and `kq8 text <file>` to print a game text file
  (`vol:<World>/<volume>/<NAME>` for a file inside an archive).
- `crates/kq8-viewer`: Bevy viewer. `[--world <W>]` shows a world (the
  default); `--model NAME` shows one model; `--renderer auto|vulkan|dx12` the
  graphics API (`renderer.rs`); `--light x1|x2|off` the
  light at start; `--haze on|off` the haze at start; `--dynamic on|off`
  overrides `Options.cs`'s `useDynamicLighting`; `--sound on|off` the
  sound card (`sound_view.rs`). The full option list with descriptions is in
  `docs/FEATURES.md` ("Launch options"). Test options: `--frames N`, `--lift N`, `--at X,Y,H`,
  `--hold F,T,R[,N]`, `--jump N`, `--turn N[,1|-1]`, `--click NAME,N`
  (repeatable; skips the line-of-sight test), `--aim NAME,N` (a click
  with the line-of-sight test), `--item TYPE`. Haze and drawing: the
  ground's material `ground_haze.rs`, walls `building_light.rs`, models
  `object_light.rs` (each with its `.wesl` shader), near / far / off and
  the collider switch `draw_distance.rs`, point lights on models
  `scene_lights.rs`, on building surfaces `wall_lights.rs`.
- Logs go to `kq8-artifacts/logs/`: `viewer.log`, `connor.log` and
  `camera.log` (one line per update), `doors.log` (clicks, scripts,
  events), `traps.log`, `items.log`, `effects.log`, `ground_anim.log`,
  `anim.log`, `sound.log`, `lightmap-verify.log`, `lighting-verify.log`,
  `aud-verify.log`. `viewer.log`
  also logs the ground light under Connor and the camera, and every
  second how many objects and wall surfaces the haze leaves out and how
  many objects are near / far / off (plus every change of state), every
  collider switch, and the scene's lights (how many, what the nearest
  gives Connor), and per point light what it does to building surfaces
  (tested, skipped, not in front, out of reach, blocked, lit).

## What works (tested)

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
- Checked by eye by the user (2026-10-16): point lights on building
  surfaces, `KQPointLight` always shining, reach to a building in its
  own axes. F9 (dynamic lighting) checked by the user 2026-10-17.
- Sound effects (M3 3h, 2026-10-17): see "Where we are". Checked by
  ear by the user. World music (3h-5) too.
- Tests: 337 unit tests pass (105 + 216 + 5 + 11), 2026-10-17.

## What doesn't work yet / next

- **Next step, suggested:** for B: cues of scenery animations (`anim.rs` doesn't run cues), and M3/M4 (Connor's
  scripted movements, remarks, inventory screen). The brightness setting
  (rule 17) is still not built.
- Sound, not done: the item land sound (nothing is thrown),
  hurt sounds (fighting), ducking under speech, pause, speech,
  `loadKQ` while playing (so `Waterfall2Sound` never appears), and
  `setActiveCamera` / `makeActive` for sound objects.
- Clicking: only doors, chests, items, scripted objects and buildings
  react. No remarks (Connor says nothing; remarks are logged),
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
  switch them yet, `KQAnimPlayEvent` to them is only logged) and their
  cues (sounds) don't fire. A rolling rock that runs into Connor does nothing to him (in
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
- Lighting still lacks: the brightness setting (rule 17). Point lights
  on building surfaces light every surface of a drawn building, not only
  those in view (same picture, more work).
- Not drawn yet: people and monsters (`.anm` characters; orange cubes with H), the
  teleport pad's look (Q14b), arrow sparks and the dragon fire (they come
  with fighting).
- Nothing yet of: talking, inventory screen, fighting,
  stats, story, saving, travel between worlds.

## Open questions (`docs/behaviour/QUESTIONS.md`)

- Open points inside `object-lighting.md` (not in QUESTIONS.md): only
  the flames' 12,100 (traced and built) vs the user's ~11,000 (open
  question 9, kept for a later session) and the Direct3D renderer (6,
  not studied; Glide only). The rest was answered 2026-10-17.

- For B (answered 2026-10-17 by A, not built): Q84–Q93 (see "Where we
  are").
- For B (answered 2026-10-17, not built): Q81 (c) (stand-in already
  right; only the forward-wrap transition detail is new).
- `sound.md` open questions 5–6 (music after a video; whether a world
  change cuts the old tune): not traced, checks by ear suggested.
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
- Q41 is listed above; Q14 (b) below.
- Q42: entry 0 in textures' own colour tables (stand-in black; no visible
  difference).
- Q14b: the teleport pad's see-through look.
- All others are answered or closed (Q80 answered 2026-10-17; Q79 answered by A 2026-10-17 and checked
  by B in 7k; Q75–Q78 answered by A and built
  2026-10-16; Q54, Q60–Q64, Q70–Q74 built 2026-10-16; Q73 checked;
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
