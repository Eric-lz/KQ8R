# STATUS

Hand-over in case the chat is lost. Brought up to date 2026-10-18 at the
end of B's session building Q97–Q99 (M3 3n: when the camera updates). Read with `AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`,
`MODLOG.md` (newest entries at the top). The short public summary is
`README.md`; the detailed "what works" list is `docs/FEATURES.md`.

## Where we are (2026-10-18)

- **M2 (walk Connor around Daventry) is done.** **M3 (Daventry comes
  alive) is well under way**: doors, emitters, flowing water, animated
  objects, traps, items, chests, scripted objects, clicking, sound
  effects and world music, scripted animations, pushing and the bricks
  puzzle (most checked by eye; see `docs/FEATURES.md`).
- **New 2026-10-18 (B, M3 3n): A's answers to Q97–Q99 are built**, one
  commit per step, checked in the logs only (not by eye yet):
  - 3n-1: the follow camera updates only when Connor moves (> 0.1 or a
    turn), after `follow`, while the "turn back" switch is on, while the
    player uses the camera, and in flights; each update's causes in
    `camera.log`. Stand-in: one update on the first tick (Q27 d).
  - 3n-2: the refined height rule (lift at once, snap within 100, ease
    10% only when the aim moved > 20); replaces the hand-set 0.5.
  - 3n-3: `move KQCamera` while following keeps its heading (stand-in
    for the undecoded swing-behind rules; Q100).
  - 3n-4: a flight's last update and a cut keep the height (Q97); the
    watermill's end spot has ground 7,650, so no lift.
  - 3n-5: logs for the end of a scripted walk (Q98); the numbers match
    the rules; Q101 asked.
  - Mill wheel (`wheeltp1.cs`) in our logs: first shot (185500, 226520,
    9627.3), heading kept 0.259 rad off Connor; second shot exactly at
    the script's (185000, 237850, 10850) until the flight.
- **Before that (B, M3 3m): A's answers to Q94–Q96 and the camera
  flights are built**, one commit per step, checked in the logs; 3m-1,
  3m-2 and 3m-4 also by eye by the user (like the original):
  - 3m-1 (Q96): `KQConner::endJump` / `JumpDir`: the windmill tumble
    finishes the jump, Special replaces it (no Fall), gravity stays on;
    after cue 300 Connor drops to the floor and can jump again. A trap's
    script now counts as inside Connor's update.
  - 3m-2 (Q95): a barrel pushed into the pub goes into the wall while
    the wall stops Connor; then Push → Move → `Stop` with the block.
  - 3m-3 (Q94): scripted walks play `Stop`, `Left`/`Right`, then `Walk`
    during the turn; a wall block no longer stops them. The watermill
    walk now arrives 7 updates sooner (no slow start from standing).
  - 3m-4: camera flights and cuts (`addCamLoc`, `flyToCamLoc`,
    `cutToCamLoc`; new `kq8-core/src/camera_flight.rs`): the watermill's
    flight (31 updates, the jump closer at the end) and the mill wheel's
    (46). **Q97 asked** (the height on a flight's last update).
- Before that (3l, 2026-10-18): A's answers to Q84–Q93 (the bricks
  without a pop-up, the watermill cutscene, `handsOff`, pushing barrels
  and the hay bale, turning during `crash`, torches in step, levels,
  `loadKQ` while playing); see MODLOG.
- **New 2026-10-18 (A): remarks and speech studied** (`talking.md`,
  not built): message and voice files, the per-character queue,
  captions, ducking (`nofade`), the mouth, `TalkComplete` and
  `stateChangeNotify`, what cuts a line short, the stock remarks.
  Conversations are not studied yet. The turn, the 0.5 s wait, the
  ducking, talking while walking and the mouth were confirmed by the
  user in the original.
- **New 2026-10-18 (A): Q97–Q99 answered** (`camera.md`,
  `anim-scripts.md`, not built): the follow camera updates only on a
  "Connor moved" message (a committed move that changes his position by
  more than 0.1 or his facing), after `follow`, every update while the
  rarely-set "turn back" switch is on, and during flights. The height
  eases (10%) only when the aim point moved more than 20; otherwise it
  only snaps within 100 or is lifted off the ground. A flight's last
  update keeps the height (B's stand-in is right). In a trap, the
  camera's update for a script's `move Connor` comes after the script's
  `move KQCamera`. The end of a scripted walk plays the ordinary stop
  transition and the next turn curves it: B's build already matches;
  the user's difference is not explained (`anim-scripts.md` open
  question 6).
- **Later for A:** conversations (`KQConner::conversation`); Dead City's flames (11,000 vs 12,100, open
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
- `crates/kq8-core` (278 unit tests; named `kq8-game` until 2026-10-13):
  game rules without drawing. Connor: `connor_mover.rs` (steps, gravity,
  landing, Fall, fall death, building and object hits, Jump, pushing),
  `connor_auto.rs` (scripted turns and walks: `walkTo`, `pointAndCue`),
  `connor.rs` (keys → movement, turning, banking, steps from the
  animation, start/stop transitions, crash, wall blocks, jumps, wading),
  `connor_update.rs` (one update of Connor), `collide.rs` (ball stack,
  building BSP sweep, object box sweep, type/mask rule, line tests),
  `water.rs` (the wading test). World: `camera.rs` (the following camera
  with the obstacle check), `camera_flight.rs` (cutscene camera flights
  and cuts), `script.rs` (the `.cs` script runner),
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
  `--hold F,T,R[,N]`, `--jump N` (repeatable), `--turn N[,1|-1]`, `--click NAME,N`
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
- Tests: 399 unit tests pass (105 + 278 + 5 + 11), 2026-10-18.

## What doesn't work yet / next

- **The user's checks of 3n (2026-10-18):** the mill wheel from the
  south (`wheeltp1.cs`): no difference from the original; walking feels
  like the original (kept); the watermill walked into from outside:
  like the original (the extra step only with our `--at` start inside
  the trap; Q101 note). From the north (`wheeltp2.cs`) `wheeltp1` fired
  too: fixed (a script's `move Connor` no longer sweeps the traps along
  the jump; stand-in, **Q102** asked); **to check by eye again**.
- **The user's checks of 3m (2026-10-18):** the windmill tumble (forward
  and back flip), barrels pushed into a wall (the wall stops Connor) and
  the watermill room's camera flight match the original. The scripted
  walk differs: at the end of the watermill walk Connor turns left and
  goes on a bit further than in the original; likely the walk's stop
  transition (`walkstopright`, about 460 units) playing while
  `pointAndCue` turns him (**Q98**, asked). The mill wheel cutscene
  (`wheeltp1.cs`): its two shots before the flight were lower and closer
  than in the original (**Q99**; built in 3n, not checked by eye yet).
- **Stand-ins of 3m** (not traced; none reached in Daventry): a
  state request refused during an unfinished jump is tried again every
  update; objects later in the collision list after a push are not
  tested; `follow Connor` during a flight ends it.
- **Stand-ins of 3n:** one camera update on the first tick (Q27 d); the
  heading after `move KQCamera` while following kept until the next
  `follow`, flight or cut (Q100 a); a deep lift whose line meets no
  ground (Q100 b); one `follow` finding both conditions turns the
  "turn back" switch on (Q100 c).
- **Next step, suggested:** the user's checks by eye of 3n. For B: cues of scenery
  animations, `loadKQ` with a name or place while playing (barrels and
  boards breaking, M5), remarks and speech (`talking.md`, studied by A),
  inventory screen (M4). The brightness setting (rule 17) is still not
  built.
- Sound, not done: the item land sound (nothing is thrown),
  hurt sounds (fighting), ducking under speech, pause, speech, and
  `setActiveCamera` / `makeActive` for sound objects.
- Clicking: only doors, chests, items, scripted objects and buildings
  react. No remarks (Connor says nothing; remarks are logged),
  Connor doesn't turn to face what he clicks. No inventory screen (M4).
  `checkInventory` by id number (Q62 b) not built.
- Camera, still missing: the original's own automatic swing back behind
  Connor when he turns (not built; while walking the leash already makes
  the camera trail behind him), first person (F2), shake; the roll of
  `move KQCamera` and of flights is not drawn (logged).
- Cutscenes: no GUI panels to hide and no spinning-mask icon during
  `handsOff`. A request from a cue reaches Connor one update after the
  cue (our order).
- Animated objects: scenery animations' cues (sounds) don't fire.
  `loadKQ` with a name or position while playing is not done; a copy
  that comes in is not solid or clickable. A rolling rock that runs into Connor does nothing to him (in
  the original he stumbles and takes damage; rule in `connor-collision.md`,
  Q55, not built yet).
- Water: no damage yet (poison, lava, swamp).
- Collision, not done: Connor against monsters (none drawn), Gnome's
  `KQBoulder`, ropes; objects with no box in the game's table stay
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

- Q97–Q99: built 2026-10-18 (3n). The user's watermill walk difference
  is still not explained (`anim-scripts.md` open question 6).
- For A (asked 2026-10-18 by B): **Q100** (the heading after `move
  KQCamera` while following in later updates; the deep lift; the "turn
  back" switch when one `follow` finds both), **Q101** (the update Wait
  asks for `Stop` after a scripted walk's end; lower priority after the
  user's check), **Q102** (does a script's `move` test the traps).
- For B (answered 2026-10-17, not built): Q81 (c) (stand-in already
  right; only the forward-wrap transition detail is new).
- `sound.md` open questions 5–6 (music after a video; whether a world
  change cuts the old tune): not traced, checks by ear suggested.
- Q53: camera obstacle details (how the view line meets the ground and a
  line starting under water; the BSP line test; whether the new direction
  is 3D). Stand-ins in place. (d) answered by Q99.
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
- All others are answered or closed (Q94–Q96 answered and built
  2026-10-18; Q84–Q93 answered 2026-10-17 and built 2026-10-18; Q80 answered 2026-10-17; Q79 answered by A 2026-10-17 and checked
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
