# STATUS

Hand-over in case the chat is lost. Brought up to date 2026-10-20 after
B built the fight (M5 5c, `fighting.md`). Read with `AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`,
`MODLOG.md` (newest entries at the top). The short public summary is
`README.md`; the detailed "what works" list is `docs/FEATURES.md`.

## Where we are (2026-10-31)

- **2026-10-31 (B): Q176–Q181 applied** (I-1 to I-7, logs only, not
  looked at by eye): the north tile jump lands (steps 325); the teleport
  camera flies at 5,000 a second and follows again when the flight ends,
  with the hold switch; scripts' weapon requests on the stones say the
  line, the Jump key jumps; Death's mouth codes; the camera commands read
  numbers as C's `atof`. **Not done:** I-2 (the stones' bob drift of
  +0.011 per cycle: left out by the user's choice, the stones stay
  level) and Q175 (the
  tower skeleton, a new feature). Also H-1 (armour skins), H-2 (deep
  water run-out) done; **open for A:** Q184 (armour skins), Q185
  (`gobcon1` put back home over and over).

- **2026-10-30 (A): Q175 to Q183 answered** (traced; nothing built yet).
  No question is open any more. What B should change:
  - **North tile jump (Q177):** keep step up / down at **325** during a
    tile jump (only gravity and keep-on-ground go off, back on when the
    jump movement ends); the step-up climbs the stone's edge.
    `tile-puzzle.md` rule 11. The 29 box-less stones are right.
  - **Stones' bob (Q176):** count the wrap step (`objects.md` rule 6a);
    then the cues cancel the leftover and B's stand-in can go.
    `tile-puzzle.md` rule 16.
  - **Teleport camera (Q178):** a `flyToCamLoc` with no time (distance
    ÷ 5,000 per second), starting when `teleportdown` ends; the hold
    switch keeps the camera's height and tilt. `tile-puzzle.md` rules 7a,
    13.
  - **Weapon line (Q181):** said on entering only with a weapon out (as
    built); also for scripts' `setDoMode` / `setRequiredWeaponMode`
    (B's are silent). The Jump key jumps too. `tile-puzzle.md` rules 7,
    8, 18.
  - **Camera roll (Q182, Q180):** G-5's sign is right (negative =
    picture clockwise); no scaling; numbers read with `atof`.
    `camera.md` rules 4a, 4b.
  - **Death's mouth (Q179):** code 16 starts `rest` (not `stop`); the
    poses hold; end of line does nothing. `talking.md` rule 18a.
  - **Death's key (Q183):** the spin twice is right. `anim-scripts.md`
    rule 3.
  - **Tower skeleton (Q175):** its standing, moving and falling states
    are in `fighting.md` section 7a (the stagger's ground probe is
    behind it: rule 23 corrected). The fall's camera and move offsets
    are as read, not checked in play.

- **2026-10-30 (B): G-1 to G-4, fixes after the user's check of F-4 to
  D-6** (logs and unit tests, **not looked at by eye**): on the stones
  the weapon is put away for good and every swap is refused with "A
  weapon will not help me here." (also said on entering: the user's
  check, Q181); the camera's roll is drawn (sign from the user's check,
  Q182); copies loaded while playing move their collider, so Connor
  rides the bobbing stones; an object a script loads starts its
  movement afresh, so Death's key flies to Connor and is given.
  **Open for A:** Q177 to Q182. **Next for B:** D-8 (batch 2 list below).

- **2026-10-30 (B): F-4 to F-6 and D-6** (logs and unit tests, **not
  looked at by eye**): Death is a character (`KQDeath`): his whole scene
  runs, his lines with mouth movements, the key, Connor sent outside;
  `flyToCamLoc … -.6` read (Q180); the torches and flames of his room
  are made; `changeBmp`, so the Clarity potion shows the five letters
  (and `tileshow`, loaded while playing, now gets the potion's calls).
  **Open for A:** Q177, Q178, Q179 (Death's mouth details), Q180.
  **Left in batch 2 for B:** D-8 `matchHeights` / `terAnimSwitch`, D-9
  `Lookup` and spirit traps, D-10 `ConnorCollision` and `Scales`, D-11
  `ThrowRock`, `KQWeepBolt`, `RiverControl`, `KQAttackEvent` on objects,
  `KQMonster::add`; `KQTowerSkeleton` waits for Q175. `changeBmp` on still
  objects, doors and characters is logged, not drawn.

- **2026-10-30 (B): fixes after the user's check of the death hall**
  (DESIGN "Fixes after the user's check", F-1 to F-3; logs only, **not
  looked at by eye**): the halls' doors now stand where `newWorld.cs`
  moves them (they were all in the hall's middle, blocking Connor);
  Death's trap is armed at the landing on H, so his scene starts at
  once; gravity is off while a tile jump plays, so Connor rises. Still
  wrong: north jumps are about 25 short of the next row (Q177); the
  camera flight after a teleport is too fast (Q178). **Next for B:** F-4
  Death as a character (he is not loaded, so his talk never starts),
  F-5 `flyToCamLoc` with a roll, F-6 the torches `azrieltp.cs` loads,
  then D-6 (the Reveal potion on the letters).

- **2026-10-30 (B): script coverage batch 2 (Dead City), first part
  built** (DESIGN "script coverage batch 2", D-1 to D-5 and D-7; logs and
  unit tests only, **not played by hand**): the death-hall stone puzzle
  runs from start to finish in a scripted test (jumps, lit stone, right
  and wrong stones, teleports, lines, the way out by the door);
  `KQMoveEvent` on objects; copies a script loads are solid; the small
  commands (react percents, Connor's flinch 0.5, required weapon mode,
  `invClick`, `lightOnFire`, `setCamRelConner`, `walkSlope`). Dead City
  now misses 10 commands (40 uses), was 19 (70).
  **Open for A:** Q176 (the stones' bob correction has the same sign as
  their leftover; a stand-in keeps them level), Q177 (north jumps can't
  climb the 500 between rows: Connor hits the stone's side and drops to
  the hall floor; the puzzle still resolves because the landing is judged
  by x and y). **Left in batch 2 for B:** D-6 `changeBmp` (Reveal
  letters), D-8 `matchHeights` / `terAnimSwitch`, D-9 `Lookup` and
  spirit traps, D-10 `ConnorCollision` and `Scales`, D-11 `ThrowRock`,
  `KQWeepBolt`, `RiverControl`, `KQDeath`, `KQAttackEvent` on objects,
  `KQMonster::add`; `KQTowerSkeleton` waits for Q175.

- **2026-10-28 (A): script coverage batch 2 (Dead City) studied, Q174
  answered.** New note `docs/behaviour/tile-puzzle.md`: the death-hall
  stones (jump one stone per key press toward Connor's facing, the
  target stone lit; D-E-A-T-H fixed in the program; a right letter
  teleports him next to the next one, a wrong stone back to the start,
  no damage; `tileExit`, `deadHallTeleport`) and the scales. The other
  batch items went into their notes (see MODLOG 2026-10-28): rock
  throwing, the weeper's bolt, spirit traps, river boxes, Death's lips,
  the tower skeleton, `KQMoveEvent` flags on objects, `changeBmp`,
  `matchHeights`, `terAnimSwitch`, `setCamRelConner`, `Lookup`, and more.
  Corrections: Connor's flinch value is 0.5, not 0.9 (`fighting.md`
  rule 29); the "unlit" switch is used (`object-lighting.md` rule 10).
  For the user to decide: spirit traps may drain on every entry once a
  spirit has passed (`traps.md`, possible original bug); Dead City's
  rats, snakes and spirits probably don't patrol in the original
  (`characters.md`, a data quirk). Next: B builds batch 2.

- **2026-10-27 (B): script coverage checklist** (the user's
  request): `kq8 script-coverage` writes `docs/SCRIPT-COVERAGE.md`,
  every command, Connor state, event, script call and class the game's
  files use, per world, and whether we have it. Study batches for A in
  `docs/DESIGN.md` "script coverage checklist" (batch 1: everywhere;
  batch 2: Dead City, with Q174).

- **2026-10-27 (B): Dead City's Paddle puzzle** (the user's report):
  the Paddle shows on its pedestal and can be clicked back; pushing it
  (Q173 built, P-2) plays Connor's push from his side, the tiki sinks,
  the Paddle comes back; the fourth starts the laser (user: the first
  pedestal works). Fix: the 2nd–4th Paddle showed nothing (a reused
  name found the deleted first one). The user's check (2026-10-27):
  working. Seen on
  the way, not done: the tiki lights' cue to `Terrain`,
  `KQCamera::setCamRelConner` (laser scene).

- **2026-10-27 (B): fix: the Shadow fight on the first arrival in
  Dead City** (the user's report: Connor only played `portalx`). The
  gate's script cues the object `ShadBat`, which our loader had dropped
  (no collision, so no box); scripted objects without a box are now kept
  for their script (`clicking.md` rule 21). The whole scene runs in the
  logs and plays only once; the user's look: it works. Then
  `makeTranslucent Connor` (the Shadow is part of his model there): see-
  through in movements 3, 5, 7 (logs only). The last shot (too close,
  user): Q172 built (P-1): a cut no longer ends a running flight, so it
  is pushed out like the scene's other cuts. The user's check
  (2026-10-27): the fight works perfectly.

- **2026-10-27 (B): fix: the caption stone looked checkered** (the
  user's report). `stontran.pbm` is a one-pixel checkerboard of
  see-through pixels; stretched by uneven amounts (the 800 × 600 layout
  × 1.25, then the window) it broke into stripes and blocks. The caption
  picture is now made at window pixels with each stone pixel a whole
  number of window pixels. The user's look (2026-10-27): looks great, and
  it adapts as the window is resized.

- **2026-10-26 (B): fix: Daventry's small waterfall was missing after a
  return** (the user's report): a kept world now also keeps the files
  scripts loaded while playing and loads them again on return
  (`world-change.md` rules 12, 14.5). **Then (K-1 to K-3):** animated and scripted objects that
  changed keep their movement, phase, place and collision type (the
  bricks stay down, the stream stays blocked; values as in the user's
  `save10.vol`). The user checked it in play (2026-10-27): working.

- **2026-10-25 (B): smooth joins (with `--smooth`, ours)** (DESIGN
  "smooth joins", J-1 to J-4): a change of movement is drawn mixed over
  0.15 s (`smooth join <s>`, 0 = off) where both files share the part.
  The user's look: good; the length stays tunable (`smooth join`), the
  user will fine-tune it later. Still jumps: Connor's
  walk ↔ run (run's body is numbered differently: a tolerant
  renumbering would be a further step), and zombies, spriggans, the
  wizard's other files, skeletons, orcs.

- **2026-10-25 (B): smooth animation (`--smooth`, ours, off by default)**
  (DESIGN "smooth animation", S-1 to S-6): model parts drawn part-way
  between frames, creatures, doors, chests, thrown items and bolts glided
  between updates, Connor's rope climb too. The user's look: all good
  (one fix: Connor's put-away movement). The rope climb's camera now
  glides between updates (always on; user-checked).
  Not done: shadows keep whole frames.

- **2026-10-25 (B): session A's Q167 and Q169 answers built** (plan in
  `docs/DESIGN.md` "building session A's Q167 and Q169 answers", steps
  C-1 to C-3, one commit each; **checked in the logs only, not played by
  hand or looked at by eye**):
  - C-1: model parts with their full placement; items laid on a part
    (`body` on the part, place and turn); `kq8 shape-parts <file>`.
  - C-2 (Q169): a dying character's weapon lies where its death movement
    leaves it (Daventry `sprgnbow`'s crossbow, Dead City `skeleton4`'s
    sword); its line test's blocking types are a stand-in (Q170).
  - C-3 (Q167): `Inv <item>` in Connor's Special list lays a real item;
    the roof kill's crossbow stays to be picked up.
  - Q168 needed nothing (the caption stone was already tiled).
- **2026-10-24 (B): session A's 2026-10-24 answers built** (plan in
  `docs/DESIGN.md` "building session A's 2026-10-24 answers", steps B-1
  to B-8, one commit each; **checked by the user 2026-10-24: all
  good**):
  - B-1 `getConInventory` (Q166): the bell rings with the small axe and
    the third hit brings in the Lady of the Lake.
  - B-2 `exist` (Q162): the roof choice's forward key plays the
    spriggan kill.
  - B-3 the loop speed (Q164) for Connor and characters: walk 1,124,
    run 2,558, back 1,070 units/s (measured 1,119–1,122 / 2,562 /
    1,066); character speeds as `character-movement.md`'s table.
  - B-4 `setState … Wait` (Q160): Connor stands up after `portalx` in
    the Shadow scene.
  - B-5 `setHaze` (Q161): the thick fog in the Shadow's home.
  - B-6 `dithered=` on every class (Q165): Daventry's arrow doors,
    Gnome's lock and characters.
  - B-7 panel pictures (Q108 b): stretched to their box, the bars' empty
    part `4black.pbm` (the caption stone tiled again: Q168 says tiled).
  - B-8 Q163 checked: one gap fixed (a character first seen in a real
    state now reports `0 -> new`).
- **2026-10-24 (B): session A's 2026-10-22 answers built** (steps A-1 to
  A-12; **checked by the user 2026-10-24**: the Shadow scene with fists
  out, the roof Choice, the ravens, the experience sound, the dither, the
  popup on Yes and the spinning mask work; the conversation camera looks
  the same as the original).
- **2026-10-23 (B):** the frame for a phase is cut down (Q153; the
  King's portrait stays pushed, user-checked); objects a script loads
  while playing get their click box (the key behind the portrait, user:
  works); the Shadow scene at the Dead City portal plays (user: works).
- **2026-10-22 (B, M6 6e): moving between worlds built** (the loading
  screen checked by the user, the rest in the logs; the popup's look
  "good enough"; Castle voices and the mirror scene user-checked).
  Console words `reload`, `goworld <World> [x y turn]`, `goworld portal
  <file>`. A world takes about 3.4 GB, so left worlds are not kept alive.
- **Earlier (2026-10-18 to 2026-10-21):** M2 and M3 done; M4's speech,
  interface, inventory, characters and conversations; M5's fight, ranged
  weapons, potions; M6's rope, hammer and magic map. Details:
  `docs/FEATURES.md` and `MODLOG.md`.
- **Later for A:** Dead City's flames (11,000 vs 12,100, open
  question 9); Connor against monsters.

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* that reads the player's
own install at runtime. Roadmap M0–M7 in `docs/DESIGN.md`. Current
milestone: **M4** (remarks and speech, the interface and the inventory,
characters and conversations done; the menus postponed) and **M5**
(characters moving on their own done; the fight next). M0–M3 are done.
M6 has started with the Rope and Hook (6a).

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
- `crates/kq8-core` (406 unit tests; named `kq8-game` until 2026-10-13):
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
  timing, mouth schedule; M4 4a; the conversation queues, 4e),
  `character.rs` (characters' states, M4 4d; walking, M5 5a), `character_ai.rs` (sight, the active list, wandering, fidgets, alarm and chase, M5 5a), `conversation.rs` (the
  conversation command and Connor's talk state, M4 4e).
  `kq8-data/src/msg.rs`: message,
  talker and lip-sync files. Viewer `talk_view.rs`: voices, captions,
  Connor's mouth. M4 4c: `caption.rs` (wrap, the box's place),
  `panels.rs` (shown/hidden, sliding, pop-ups), `meters.rs` (bar
  percents and widths), `gui_draw.rs` (a panel drawn from its layout),
  `inventory.rs` (slot pictures), `connor_inventory.rs` (adding,
  stacking, equipment, applying, slot clicks, the hand). M5 5g:
  `potions.rs` (timers, icon places, the four effects, pickup
  experience); M6 6a: `rope.rs` (rope places, the hook icons, the rope
  state's sequence, camera spots and ranges) and viewer `rope_view.rs`
  (runs it on Connor, the camera, the rope model); viewer `potion_view.rs` (the orbit, Connor's skin, faces
  and Invisible look, `dither`). Viewer
  `gui_view.rs`: the panels, the pointer, clicks and keys on the bars.
- `crates/kq8-audio` (5 unit tests): our mixer and the sound card (cpal).
  Runs at a fixed **15 updates per second**; the ground animators run on
  continuous game time.
- `crates/kq8-cli`: `kq8` checks (`vol-verify`, `bitmap-verify`,
  `bitmap-png`, `shape-verify`, `terrain-verify`, `building-verify`,
  `item-verify`, `lightmap-verify`, `lighting-verify`, `aud-verify
  [--wav <file>]`, `msg-verify`, `font-verify`, `gui-verify`,
  `character-verify`, `conv-verify`) and `kq8 text <file>` to print a game text file
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
  name); `--corners on|off` (the corner panels, F6). Haze and drawing: the
  ground's material `ground_haze.rs`, walls `building_light.rs`, models
  `object_light.rs` (each with its `.wesl` shader), near / far / off and
  the collider switch `draw_distance.rs`, point lights on models
  `scene_lights.rs`, on building surfaces `wall_lights.rs`.
- Logs go to `kq8-artifacts/logs/`: `viewer.log`, `connor.log` and
  `camera.log` (one line per update), `doors.log` (clicks, scripts,
  events), `talk.log` (speech), `conversation.log` (conversations),
  `characters.log`, `gui.log` (panels, stats sent, the bars'
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
  `docs/FEATURES.md`). F7 (light) and F8 (haze) switch them; start options
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
  it into a wall only creeps it closer. The view lets Connor drift about
  14° off the middle before it catches up; the right button held keeps
  him at his spot on screen; it points at about his chest (M3 3o,
  checked by the user against the original 2026-10-19).
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
    leash (1,000–2,500 inside). Teleporters and cutscenes are only
    logged (ropes: M6 6a).
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
- Sound effects (M3 3h, 2026-10-17): see `docs/FEATURES.md`. Checked by
  ear by the user. World music (3h-5) too.
- Remarks and speech (M4 4a) and the click turn and turn keys (4a-7
  to 4a-9): checked by the user against the original, 2026-10-18.
- The interface and inventory (M4 4c): checked by the user against the
  original, 2026-10-18 (the corner panels option: looked at by the user,
  no complaint). The back flip at a wall: checked by the user.
- Conversations (M4 4e): built from the logs, then checked by the user
  (walking away and back, the cuts, the Uglybeast; `docs/FEATURES.md`).
- Health items and potions (M5 5g, 2026-10-21): checked in the logs only
  (`MODLOG.md`); not looked at by eye yet.
- The Rope and Hook (M6 6a, 2026-10-21): **checked by the user by eye
  at two rope places: "working perfectly"**.
- Tests: 618 unit tests pass (131 + 482 + 5), 2026-10-21.

## What doesn't work yet / next

- **Kept worlds after K-1 to K-3 (2026-10-26):** still fresh on a
  return: doors and chests opened without a variable (and what was taken
  from chests), characters' place and health, hidden switches, items on
  the ground, copies a `loadKQ` made with a name or place.

- **After C-1 to C-3 (2026-10-25), not checked by the user yet:** the
  roof kill's crossbow lying where the movement drops it (and picking it
  up); a dead bow spriggan's crossbow (e.g. console `kill sprgnbow` with
  Connor at the roof) and a Dead City skeleton's sword lying where the
  death leaves them, while coins fly. Not tested in the viewer: a burnt
  bow spriggan throws its crossbow (its fire death has no `SmCrossbow`
  part); the line test blocked by a wall (unit test only).
  Rule 6's speed and wrap now also for doors, chests and every animated
  object (2026-10-25, logs only).

- **Left over from B-1 to B-8 (the user's look: all good, 2026-10-24),
  not built:** our camera's projection far plane does not follow
  `setHaze` (the ground and objects stop at `visibleDistance` anyway);
  `setState Connor Wait` from his jump, push, rope, talk or throw states
  (not described, nothing done); the status icons and popup buttons keep
  their picture's own see-through flag; `KQObject::changeBmp` is a stub.

- **Allocator: `mimalloc`, purge delay 0 (chosen by the user
  2026-10-22).** Tour of all 10 worlds, then Daventry ↔ Castle twice
  (`--world Daventry --frames 30000 --sound off` with one
  `--console "goworld <World>",30` per world; ~4.5 minutes; summary in
  `kq8-artifacts/logs/bench-allocators.txt`). Daventry's read, fresh →
  3rd return: Windows 3.5 → 14.6 s; `mimalloc` 3.1 → 3.5 s; with purge
  delay 0, 3.2 → 3.7 s. Memory at the end (working set / private):
  3.3 / 5.4 GB, 3.5 / 5.4 GB, 3.0 / 5.1 GB. Purge delay 0 against
  1000 ms, 3 tours each (`bench-purge.txt`): same read times, ~350 MB
  less working set on average, ~550 MB less at the highest. To turn it
  off for one run: `$env:MIMALLOC_PURGE_DELAY = "1000"`. `memory.rs`
  is Windows-only (Linux needs `/proc/self/status`). The bigger saving
  is ours. Measured (`world.log` "memory by kind"): identical copies are
  already shared; Daventry's animation frames are 56,468 meshes, 955 MB
  of 1,063 MB of meshes (textures 233 MB), 95,441 entities. Options:
  animate by moving parts: **measured, not worth it** (`viewer.log`
  "parts count": 85-91 % of the triangles stay, over half the parts
  change form every frame). **Plan chosen (DESIGN "Animation memory"):
  step 1 (no normal-memory copy of animation meshes; **done 2026-10-22**,
  Daventry in play 2.64 → 1.51 GB, checked by the user). **2026-10-23:
  step 3 skipped, step 4 (frames on the graphics card) now, in sub-steps
  4a-4g (DESIGN). 4a done (the compact data checked in all worlds:
  Daventry 71 MB instead of 955 MB). 4b built (still animated objects
  drawn the new way; F5 compares) and 4c (doors, chests, scripted
  objects) and 4d (characters) checked by the user. 4e built (copy
  templates and script copies; items and effects are still models, left
  as they are) and 4f (Connor) checked by the user; **4g done: the old
  drawing removed. Daventry in play 787 MB (was 2.64 GB), 3,565 entities
  (was 133,200).** Drawing work is open again.**
- **Loading time (2026-10-23, DESIGN "loading time"):** Daventry reads in
  1.3 s (was 3.5 s), everything drawn about 2.8 s after the start;
  a local `play.cmd` (not in the repository) skips cargo's 0.7 s. The timeline is in `viewer.log`. Smooth
  animation after M7. Other drawing work waits until step 4 is done.**
- **Moving between worlds, what is missing (6e):** worlds kept in full
  (positions, chests without a variable, items dropped, bodies, terrain
  changes, the frozen world clock): with saved games. Reading slows down
  with each change: fixed with `mimalloc` (see "Allocator" above). Building after the read
  still stops the window 0.5–0.9 s. The full-armour arrival list (rule
  5.2); Escape on the popup doesn't open the main menu (not built); a
  Clarity potion running on arrival doesn't reveal the new world's
  objects. Not yet looked at by eye: the popup, the arrivals. Help
  popups (`doPopup`, the "Go" help 151) are logged only.
- **Suggested next:** the user plays a few world changes by eye
  (Daventry ↔ Castle through the cave, the keep's teleporter with
  `--kept-maps`); then saved games (the same kept-world record).
- **Stand-ins of 6d:** the follow margin (20 px inside every edge);
  layout pixels at 800 × 600 for the view and the mouse; Connor's place
  given to the map once per update; a world never left shown from its
  shipped map file; the mini-map's view taken from Connor's when the
  map is set up.

- **Stand-ins of 6b (Q148):** where the hammer turns back without a touch;
  the model drawn pointing along its flight; requests during the throw wait for the catch; the
  throw leaves the panels as they are. Not run: a throw from a real walk,
  Dead City's lever.

- **Stand-ins of 3m** (not traced; none reached in Daventry): a
  state request refused during an unfinished jump is tried again every
  update; objects later in the collision list after a push are not
  tested; `follow Connor` during a flight ends it.
- **Stand-ins of 3n and 3o:** one camera update on the first tick,
  looking straight at Connor (Q27 d); the view heading's state around
  cutscene cameras (Q113 a–c; the user's check matched ours after the
  watermill scene); the mouse's recentring near the window's edges is
  not built (not settled; the note says it may be left out).
- **Stand-ins of 4d (characters):** a character's facing taken as
  Connor's (−Y). Q116–Q124 are all built (Q123 on buildings with Q135,
  4d-7b-5). Q127 (doors and chests keep the old load names: fine as they are) and
  Q130 (Connor's shadow test) answered by A, not built yet; Q128 and Q129
  built in M5 5a. Our
  readings: the shadow's terrain search uses the model's points, not
  its box; multiply by 0.5 for "strength 0.5"; a transition's target
  first moves in the next update. Not built: `changeAnmFile`, the
  `endLoop` cue (the fight). Not tested: the
  characters scripts load while playing (the Lady, three ravens, the
  Unicorn), a real turn in the viewer. `sprdump`'s scene (the outhouse
  door) runs and the spriggan is drawn (fixed 2026-10-25; logs only).
- **temple4's `LucEnd` / `LucEndFar`** (class `KQLucreto`, the final
  Lucreto) are not loaded as characters: no model, never drawn (found
  2026-10-25).
- **Stand-ins of 4a:** the caption panel's look. Two details of `eval` (`anim-scripts.md` rule 18) are not
  built, as no data uses them: `eval` alone gives 1 (the note: 0), and
  its words are not read again as a new line.
- **Stand-ins of 4c** (Q108–Q111): the 800 × 600 screen and how the
  layout's rectangles scale with it (× 1.25; pictures stretched to them
  since B-7); the panels' animations at 15 frames per second; counts of 0
  not written; the rocks picture; the blink's order; a `give`'s
  pushed-out piece at Connor's feet; the
  pointer's click point at its top-left. Not built: the map (M6), the
  help popup and F1, the level-up flame (not traced). Escape still quits: the original's main menu pauses
  the game (a menu step must stop the updates and the panels' timers).
  When standing, an automatic turn plays `Stop` for its first update
  even if he was already turning on the spot (as rules 24–25 say).
  Q111 (Space's sounds, the mask's place and size, black behind the
  bars) built from the user's checks; the exact rules are not traced.
- **Stand-ins and gaps of 5g (potions):** the spinning mask takes the
  first free status place like a potion, and the potion icon's used
  part is its `.pba`'s frame 0 (Q143); `KQObject::dither` draws half
  see-through, `dithered=1` ignored (Q144); the orbit's angle 0 along +x
  and the shape turning with it (ours). Not built: the hurt line while
  invisible (rule 27), the same lowering of Connor's reported place
  while he talks (Q146 a) (cutscenes done in 5g-9: every handsOff, Q146
  b; the rope in M6 6a-6), first-person arms,
  armour skins (the temple armour's `con65`), the help pop-ups (logged).
  The helmet's face set `chf` exists in no world, so his face stays.
  Potions are not kept across saves or worlds (we have neither yet).
- **Stand-in of the back flip at a wall (Q112):** the game may test the
  back key or the movement asked for rather than the back wall block;
  it only differs in cases not seen yet.
- **Stand-ins of 4e (conversations):** the keys are ignored during the
  walk to the spot, no Jump or push while talking (Q125); the camera
  aims 1,400 above the partner and swings around it like the following
  camera (Q126); the talk state waits while a jump is unfinished
  (`conversation.md` open question 3); Connor's state changes one step
  after core decides (same update). Not built: weapons away (`away` /
  `out`, no weapons yet), first person, the right button's lock release
  on entering. Not run: the horn on the Uglybeast, `clearSpeak` on a
  partner, a real mouse drag during a talk.
- **Stand-ins of 5c (the fight):** his swing and flinch play as one-item
  Special lists (so a weapon request during a swing waits for its end:
  `ranged-weapons.md` open question 1); a swing click on nothing turns
  him toward where the line meets the level of his feet. Game over
  (Connor stays dead). The corpse has no draw-distance test and no skin
  swap (bossprig). The flinch pause is fixed at 2 s (Q136,
  `reactPause`). Not built: fleeing / keeping distance (rule 37; none
  in Daventry), the tips' texts, surprise lines (none in Daventry). Not
  run in the viewer: the real mouse swing, fire / ice / projectile
  deaths, picking up an item in the air, the other two bow spriggans.
  Seen in the logs: rule 26's alarm with "run" also reaches far
  characters that are active and see Connor (as written; not checked
  in the original).
- **Stand-ins and open points of 5f (ranged weapons):** the bolt's own
  shape is its box's ball stack (one ball, radius 235: Q140 a); a
  character aims at Connor + 975 (0.75 × his box top 1,300; the note
  says 1,200: Q140 b); the `arrow` part is taken from the frame at the
  `do` cue's own phase (Q140 c); a script's `walkTo` right after
  `setDoMode` cuts the swap (the axe cutscene, Q141); `setDoMode` doesn't
  end a jump; the right button held (open question 2) and first person
  not done; the run comes back the simpler way (1 s after the last
  moving shot); how objects take damage is still the barrel stand-in
  (Q139 e); fire / ice bolts' sparks (`emitter`) not drawn.
- **Stand-ins of 5a / 5b (characters moving):** the idle list kept in the
  same player as `idle:<name>` (the same result as the original's swap,
  rule 18); the stuck line 1 above the feet (the note's recommendation).
  Not run in the game yet: losing sight, leaving / re-entering the
  graveyard trap, `wanderAndStop`, random wander points, landing on a
  goblin's top (unit test only), ranged characters (`doMode=1`).
- **Left in 6a (the rope):** the top trap's `On` comes at his first
  move (in `taway`), not during the climb as in the original (cause not
  found: the note's open question 7); the grab tie's 0.5 tolerance is
  ours. Not built: first person (only logged), saved games (none
  yet), rock throws (none yet: "ignored on the rope" by construction).
  Not run: a ranged weapon held at the start, the cave rope
  (`ropetrp3/4`), other worlds' ropes, a chaser losing him on the rope
  (only the report line and far bats seen), `setRopeCam` spots and the
  camera limits in the viewer (unit tests only).
- **Next step, suggested:** for the user: the rope by eye in Daventry
  (`ropeandhook.md` "How to check"). Then: check ranged weapons by eye
  (`ranged-weapons.md` "How to check" 1–9) and the fight
  (`fighting.md` "How to check" 1–8) in ours and the original. For A:
  Q140, Q136, Q143, Q144, Q146, game over. For the user: potions by eye
  in Daventry (`potions.md` "How to check" 2–7). **The main
  menu is postponed (the user, 2026-10-20):** starting straight in
  Daventry keeps testing quick; when it comes, keep a launch option
  that skips it.
- Sound, not done: pause, and `setActiveCamera` / `makeActive` for
  sound objects.
- Speech, not done: the Daventry monster-death remark, helmets' and
  armour's faces (M5).
- Clicking: only doors, chests, items, scripted objects, buildings and
  characters with a script react. `checkInventory` by id number (Q62 b)
  not built.
- Camera, still missing: first person (F2), shake; the roll of `move
  KQCamera` and of flights is not drawn (logged).
- Cutscenes: a request from a cue reaches Connor one update after the
  cue (our order).
- Animated objects: scenery animations' cues (sounds) don't fire.
  `loadKQ` with a name or position while playing is not done; a copy
  that comes in is not solid or clickable.
- Collision, not done: Connor against monsters (none drawn);
  objects with no box in the game's table stay
  walk-through. Hidden, far and off objects are not solid (rule 41; not
  checked in the original). A deleted or far building's walls stay
  solid.
- Loops (`walk`, `run`) still spread their step evenly over the loop
  (Q24); one-shot animations use their frame steps.
- Lighting still lacks: the brightness setting (rule 17). Point lights
  on building surfaces light every surface of a drawn building, not only
  those in view (same picture, more work).
- Not drawn yet: the
  teleport pad's look (Q14b), arrow sparks and the dragon fire (they come
  with fighting).
- Nothing yet of: menus, fighting, saving, travel between worlds.
- **Not M3 (M3 is Daventry only, the user 2026-10-19):** things of later
  worlds wait for M6 (listed there in `docs/DESIGN.md`): Gnome's rolling
  rocks hurting Connor, Gnome's pushable boulder, damaging water.

## Open questions (`docs/behaviour/QUESTIONS.md`)

- **Built 2026-10-24 (B-1 to B-8):** Q160, Q161, Q162, Q163 (one gap
  fixed), Q164, Q165, Q166, Q108 b. Still open for A: Q108 a, c, d;
  whether a saved game keeps `setHaze`'s values (we have no saving yet).
- **Built 2026-10-25 (C-1 to C-3):** Q169, Q167; Q168 needed nothing.
  The user's look at B-1 to B-8 and the two fixes: all good (2026-10-24).
- **New for A (2026-10-25): Q170**: which collision types block the
  dropped weapon's line test (`fighting.md` rule 31a.2: "any solid
  object"); B uses every type as a stand-in.
- **New for A (2026-10-25): Q171**: the level-up flame and the slot
  flashes are see-through in the original although rule 20a says
  attribute 0x8 is solid; B uses their files' own transparent flag.
- **Built 2026-10-27 (P-1): Q172** (a cut during a running flight).
- **Built 2026-10-27 (P-2): Q173** (the Paddle's push).

- Inside `magic-map.md` (not in QUESTIONS.md): 2 and 3 answered by the
  user's check (2026-10-21); 4 and 5 answered in `world-change.md`
  (2026-10-21); 6 (800 × 600) built with a stand-in.
- Inside `world-change.md`: three small points (not blocking):
  left-world variables in memory, the rope / movement list after
  arrival, `worldState.kq`'s events.
- **Built 2026-10-24 (A-1 to A-12):** Q112, Q125, Q126, Q138 (e and the
  swap during a swing), Q140 c, Q143 c, Q144, Q145, Q146, Q148, Q149,
  Q150, Q151, Q152, Q153 (Connor's frames; the speed and wrap: Q164),
  Q154, Q155, Q156, Q157, Q158 a-b, Q159, Q42; Q14 b and Q136 needed
  nothing. Small parts left: the level flame's speed and length, whether
  `LEVEL.AUD` / `STATS.AUD` play at a world start (not traced); Q81 c's
  forward-wrap transition detail (cues skipped when a waiting
  transition's phase lies in the wrap) not built.

- Open points inside `object-lighting.md` (not in QUESTIONS.md): only
  the flames' 12,100 (traced and built) vs the user's ~11,000 (open
  question 9, kept for a later session) and the Direct3D renderer (6,
  not studied; Glide only). The rest was answered 2026-10-17.

- `talking.md` open questions 3 and 5 (folders, a line with no voice)
  are still open; 1 and 2 (conversations, shapes above 7) answered in
  `conversation.md`, 4 (the caption panel) in `interface.md`.
- `conversation.md` open question 3 (states that delay the talk
  state): small, not blocking (1 and 2 answered); B's stand-in: an
  unfinished jump.
- **Q147** answered by A and built by B (2026-10-21). Left open, small:
  `ropeandhook.md` open question 7 (what sets off the top trap
  mid-climb).
- `characters.md` open questions 1, 2 and 4 (`flags` bit 0x20, how the
  renderer blends shadows, `changeBmp` / `changeAnmFile`): not blocking;
  question 3 (the AI) answered 2026-10-19 in `character-movement.md`,
  whose own open questions are now: 1 water for walking characters
  (small), 3 the stuck line's exact ground test at foot height (not
  worth copying); 2 (the fight) answered 2026-10-20 in `fighting.md`.
- `fighting.md` open questions (not blocking the Daventry fight):
  Connor's death (the input side and the
  bolt's flight answered 2026-10-20 in `ranged-weapons.md`, built in M5
  5f).
- Q131–Q134: answered by A and built 2026-10-20 (M5 5b).
- Q106 (partly) and Q107 (mostly) answered 2026-10-18 in
  `interface.md` and built (4c-1); still open: index 0 in fonts, font
  flag bit 6, the picture attribute bits.
- For A (asked 2026-10-18 by B, M4 4c): **Q108** (the 800 × 600 screen:
  how rectangles and fonts scale; the panels' animation speed; (b) how
  pictures fill their box answered and built 2026-10-24), **Q109** (counts of 0 for
  gold and rocks, the rocks picture, the blink's order, a `give`'s
  pushed-out piece, the top bar in a window; (d) the potions' clock and
  (a) the slots' count 0 answered in `potions.md` and built in M5 5g), **Q110** (the scroll's and rock's remark
  numbers in `inventory.md` rule 9; the pointer's click point; the gold
  button while the coins are held; the rope's numbers and "held or not"
  answered 2026-10-21 in `ropeandhook.md`), **Q111** (answered by the user's checks
  and built; the exact rules are worth tracing).
- `interface.md` open questions 1–6 and `inventory.md` open questions
  1–4 (level-up flame, rocks picture, held item on world
  change, applied quest items).
- For A (asked 2026-10-19 by B, M3 3o; small, not blocking): **Q113**
  a–c (the view heading's state around cutscene cameras: rule 1 from
  idle only, the exact aim before unlocking, the mill wheel's turn past
  Connor); d and e were settled by the user's checks.
- For B (answered 2026-10-17, not built): Q81 (c) (stand-in already
  right; only the forward-wrap transition detail is new).
- `sound.md` open questions 5–6 (music after a video; whether a world
  change cuts the old tune): not traced, checks by ear suggested.
- Q41: jumps: (a) jump during the wait before a stop, (c) the 250 limit,
  (d) a running jump's landing with only a turn key held. Stand-ins.
- Q27 a, a detail: the step from the mouse's window fraction to the
  camera (signs, centring) was not found; the user's checked directions
  stand.
- Q24: partly answered (steps spread within each frame; not compared
  frame by frame).
- Q29 (e): why Connor climbs so little into a steep hillside (the rest
  answered).
- Q30 (b): what keeps Connor on a slope going downhill; "likely answered"
  by the stronger gravity, but the re-check A asked for was not recorded.
- All others are answered or closed (Q27, Q53, Q100, Q102 built
  2026-10-19 (3o), Q128 and Q129 built 2026-10-20 (M5 5a-3, 5a-9), Q101 and Q48 closed; Q105 answered and built
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
