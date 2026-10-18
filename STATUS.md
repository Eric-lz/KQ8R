# STATUS

Hand-over in case the chat is lost. Brought up to date 2026-10-18 at the
end of B's M4 step 4c (the on-screen interface and Connor's inventory,
4c-1 to 4c-5; Q108–Q110 asked). Read with `AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`,
`MODLOG.md` (newest entries at the top). The short public summary is
`README.md`; the detailed "what works" list is `docs/FEATURES.md`.

## Where we are (2026-10-18)

- **New 2026-10-18 (B, M4 4c): the on-screen interface and Connor's
  inventory are built** from `interface.md` and `inventory.md`, one
  commit per step, **not seen by eye yet** (the user's check is next):
  - 4c-1: the caption's stone strip covers the whole caption box (400
    wide on the 800 × 600 screen of `set800mode 1`), rows broken at its
    width, the box a little left of the middle with the map closed.
  - 4c-2: the bottom bar from `Gplayscr.gui` in the world's pictures
    and fonts (armour / hand / ranged slots and numbers, eight slots,
    health and experience bars with the level, rope / rocks / map
    icon); Space slides it away and back in 0.2 s; handsOff slides it
    away (spinning mask) and back only if it was shown; the meters pop
    up alone for 1.5 s on a change; changed numbers flash; the caption
    sits on the bar. Connor's start inventory from `Connor.kq`.
    Bricks run: level 2, health 20/20, experience 40 (200 %), hand 15,
    ranged 2, armour 1, as `interface.md` check 2 predicts.
  - 4c-3: the top bar (mask button, quest strip with arrows, gold)
    follows the pointer and pops down for a new quest item; the eight
    slots with counts, keys 1–8, right-click descriptions (voice only);
    stacking, equipment pushing out the old piece (it drops where the
    new one lay), applying items and potion timers.
  - 4c-4: quest items, coins, rope and rocks held on the game's own
    pointer pictures; a world click offers the held item (accepted: the
    hand empties; refused: "This will not work." and it stays).
  - 4c-5: Q105 built: the click turn looks at Connor's state at the
    click (or the one the script asked for); buildings read
    `doNotPointAt`.
- **Before that (A, 2026-10-18):** `interface.md` and `inventory.md`
  written and checked by the user in the original; (B, M4 4a-7 to
  4a-9) turn keys as events, the turn after a click, the chest remark
  in any state, all checked by the user.
- **M2 (walk Connor around Daventry) is done.** **M3 (Daventry comes
  alive) is well under way**: doors, emitters, flowing water, animated
  objects, traps, items, chests, scripted objects, clicking, sound
  effects and world music, scripted animations, pushing and the bricks
  puzzle (most checked by eye; see `docs/FEATURES.md`).
- **New 2026-10-18 (B, M4 4a-7 to 4a-9): A's answers to Q104 and Q103
  are built**, one commit per step, **checked by the user in the viewer
  (all as in the original)**:
  - 4a-7: turn keys are events (`kq8-core/src/turn_keys.rs`): only a
    turn key **press** cancels an automatic turn; a held key neither
    cancels nor speeds it, and doesn't take over when it ends; forward
    stays the player's during the turn.
  - 4a-8: Connor turns to face what he clicked after every
    `ConnorAction` call (doors, scripted objects, buildings and items
    with a script), in Wait or Move, unless `doNotPointAt`; one
    automatic turn (a second request only changes the target).
    `tmbstn1` reads at once and turns while reading, `tmbstn2` turns
    and reads 0.5 s later; `wizbook` turns him, `urn1` doesn't.
    **Q105** asked (small points; stand-ins).
  - 4a-9: a chest's remark turns him in any state; only
    `doNotPointAt` gives no turn and no wait. Q103 (a)–(c) recorded
    in the provenance (stand-ins confirmed).
- **Before that (B, M4 4a, 4b): remarks and speech** (`talking.md`):
  message, talker and lip-sync files; speech queues and the `speak`
  family of commands; voices and ducking; captions (stand-in panel);
  Connor's own remarks; cutting a line short; his mouth texture.
  Checked by the user against the original. Conversations are not
  studied yet.
- **Before that (B, M3 3m and 3n):** camera flights and cuts, scripted
  walks, the windmill tumble, and when the follow camera updates
  (Q94–Q99). Details in `MODLOG.md`.
- **Later for A:** conversations (`KQConner::conversation`); Dead City's flames (11,000 vs 12,100, open
  question 9); Q53 (camera follow-ups); pushable objects (`KQBoulder`);
  Connor against monsters; Q27.

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* that reads the player's
own install at runtime. Roadmap M0–M7 in `docs/DESIGN.md`. Current
milestones: **M3, Daventry comes alive**, and **M4** (remarks and speech
done; the interface and inventory built in 4c).

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

- `crates/kq8-data` (114 unit tests): file formats, also fonts and
  `.pba` arrays (`font.rs`), screen layouts and the tag file (`gui.rs`). Sound files and
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
- `crates/kq8-core` (320 unit tests; named `kq8-game` until 2026-10-13):
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
  objects, ducking; no sound card), `talk.rs` (speech queues, line
  timing, mouth schedule; M4 4a). `kq8-data/src/msg.rs`: message,
  talker and lip-sync files. Viewer `talk_view.rs`: voices, captions,
  Connor's mouth. M4 4c: `caption.rs` (wrap, the box's place),
  `panels.rs` (shown/hidden, sliding, pop-ups), `meters.rs` (bar
  percents and widths), `gui_draw.rs` (a panel drawn from its layout),
  `inventory.rs` (slot pictures), `connor_inventory.rs` (adding,
  stacking, equipment, applying, slot clicks, the hand). Viewer
  `gui_view.rs`: the panels, the pointer, clicks and keys on the bars.
- `crates/kq8-audio` (5 unit tests): our mixer and the sound card (cpal).
  Runs at a fixed **15 updates per second**; the ground animators run on
  continuous game time.
- `crates/kq8-cli`: `kq8` checks (`vol-verify`, `bitmap-verify`,
  `bitmap-png`, `shape-verify`, `terrain-verify`, `building-verify`,
  `item-verify`, `lightmap-verify`, `lighting-verify`, `aud-verify
  [--wav <file>]`, `msg-verify`) and `kq8 text <file>` to print a game text file
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
  with the line-of-sight test), `--item TYPE` (a real held `TestItem`),
  `--key NAME,N` (`space`, `1`–`8`, `left:`/`right:` + a layout control
  name). Haze and drawing: the
  ground's material `ground_haze.rs`, walls `building_light.rs`, models
  `object_light.rs` (each with its `.wesl` shader), near / far / off and
  the collider switch `draw_distance.rs`, point lights on models
  `scene_lights.rs`, on building surfaces `wall_lights.rs`.
- Logs go to `kq8-artifacts/logs/`: `viewer.log`, `connor.log` and
  `camera.log` (one line per update), `doors.log` (clicks, scripts,
  events), `talk.log` (speech), `gui.log` (panels, stats sent, the bars'
  contents, the hand), `traps.log`, `items.log`, `effects.log`, `ground_anim.log`,
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
- Remarks and speech (M4 4a) and the click turn and turn keys (4a-7
  to 4a-9): checked by the user against the original, 2026-10-18.
- The interface and inventory (M4 4c): built and checked in the logs
  (values, timings, pictures chosen); **not seen by eye**.
- Tests: 450 unit tests pass (114 + 320 + 5 + 11), 2026-10-18.

## What doesn't work yet / next

- **Left from 3m and 3n (checked by the user 2026-10-18):** at the end
  of the watermill walk Connor turns left and goes on a bit further
  than in the original; not explained by Q98 (`anim-scripts.md` open
  question 6). A script's `move Connor` no longer sweeps the traps
  (stand-in, **Q102**).
- **Stand-ins of 3m** (not traced; none reached in Daventry): a
  state request refused during an unfinished jump is tried again every
  update; objects later in the collision list after a push are not
  tested; `follow Connor` during a flight ends it.
- **Stand-ins of 3n:** one camera update on the first tick (Q27 d); the
  heading after `move KQCamera` while following kept until the next
  `follow`, flight or cut (Q100 a); a deep lift whose line meets no
  ground (Q100 b); one `follow` finding both conditions turns the
  "turn back" switch on (Q100 c).
- **Stand-ins of 4a:** the caption panel's look; another character's
  turn to its listener is only logged (Q103 d; monsters don't move
  yet). Two details of `eval` (`anim-scripts.md` rule 18) are not
  built, as no data uses them: `eval` alone gives 1 (the note: 0), and
  its words are not read again as a new line.
- **Stand-ins of 4c** (Q108–Q110): the 800 × 600 screen and how the
  layout's pictures scale with it (× 1.25, nearest pixel, tiled when
  smaller); the panels' animations at 15 frames per second; counts of 0
  not written; the rocks picture; the blink's order; potions on game
  time; a `give`'s pushed-out piece at Connor's feet; the rope held and
  its remark; the pointer's click point at its top-left. Not built: the
  map (M6), the help popup and F1, the red boosted numbers and the
  potion timer icons (M5), the level-up flame (not traced), weapon
  readying (M5). Escape still quits: the original's main menu pauses
  the game (a menu step must stop the updates and the panels' timers).
  When standing, an automatic turn plays `Stop` for its first update
  even if he was already turning on the spot (as rules 24–25 say).
- **Next step, suggested:** the user's checks by eye of 4c (`docs/FEATURES.md`
  "The on-screen interface"; the commands are in `MODLOG.md`). For A:
  Q108–Q110, then conversations (`KQConner::conversation`), needed for
  the wizard and the other Daventry characters (the panels' side of
  conversations is built, unit-tested only). For B after that: the main
  menu (Escape, the mask button) with the game paused.
- Sound, not done: the item land sound (nothing is thrown),
  hurt sounds (fighting), pause, and `setActiveCamera` / `makeActive`
  for sound objects.
- Speech, not done: conversations (not studied), the Daventry monster-death
  remark (no monster dies), monsters' mouths (not drawn), helmets'
  and armour's faces (M5).
- Clicking: only doors, chests, items, scripted objects and buildings
  react. `Special` calls (conversation areas) are not made yet.
  `checkInventory` by id number (Q62 b) not built.
- Camera, still missing: the original's own automatic swing back behind
  Connor when he turns (not built; while walking the leash already makes
  the camera trail behind him), first person (F2), shake; the roll of
  `move KQCamera` and of flights is not drawn (logged).
- Cutscenes: a request from a cue reaches Connor one update after the
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
- Nothing yet of: conversations, menus, fighting, story, saving, travel
  between worlds.

## Open questions (`docs/behaviour/QUESTIONS.md`)

- Open points inside `object-lighting.md` (not in QUESTIONS.md): only
  the flames' 12,100 (traced and built) vs the user's ~11,000 (open
  question 9, kept for a later session) and the Direct3D renderer (6,
  not studied; Glide only). The rest was answered 2026-10-17.

- Q97–Q99: built 2026-10-18 (3n). The user's watermill walk difference
  is still not explained (`anim-scripts.md` open question 6).
- `talking.md` open questions 1–3 and 5 (conversations, shapes above
  7, folders, a line with no voice) are still open; 4 (the caption
  panel) answered in `interface.md`.
- Q106 (partly) and Q107 (mostly) answered 2026-10-18 in
  `interface.md` and built (4c-1); still open: index 0 in fonts, font
  flag bit 6, the picture attribute bits.
- For A (asked 2026-10-18 by B, M4 4c): **Q108** (the 800 × 600 screen:
  how rectangles, pictures and fonts scale; how smaller pictures fill
  their box; the panels' animation speed), **Q109** (counts of 0, the
  rocks picture, the blink's order, the potions' clock, a `give`'s
  pushed-out piece, the top bar in a window), **Q110** (the remark
  numbers in `inventory.md` rule 9: our reading names the right lines;
  the rope held or not; the pointer's click point; the gold button
  while the coins are held).
- `interface.md` open questions 1–6 and `inventory.md` open questions
  1–4 (menu pause, level-up flame, rocks picture, held item on world
  change, applied quest items).
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
- All others are answered or closed (Q105 answered and built
  2026-10-18 (4c-5); Q103–Q104 answered, built and
  checked by the user 2026-10-18; Q94–Q96 answered and built
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
