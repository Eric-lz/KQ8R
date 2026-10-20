# STATUS

Hand-over in case the chat is lost. Brought up to date 2026-10-20 after
B built the fight (M5 5c, `fighting.md`). Read with `AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`,
`MODLOG.md` (newest entries at the top). The short public summary is
`README.md`; the detailed "what works" list is `docs/FEATURES.md`.

## Where we are (2026-10-20)

- **2026-10-20 (A): Q137 answered**, `docs/behaviour/thrown-items.md`
  (`items.md` rule 12 points there): a thrown item follows the exact arc
  (8,085 units/s²) from its launch point, 15 updates a second; it is
  stopped by the floor grid, buildings and most objects (characters and
  Connor are flown through), slides off walls and drops at their foot,
  is flung up again by barrels (scripted or animated objects), tumbles
  about one axis at a fixed angle per update, and rests with no turn.
  For B: replace the 5c-8 / fix 7 stand-ins.

- **2026-10-20 (B, after the user's fight check): fixes 1–7 and 5d.**
  Readying the fists plays `draw` and switches Connor to the weapon's
  movement set, sword pointer (Ctrl: the play pointer); no readying in a
  jump, its landing or a stumble; `conversation.md` rule 7 (weapon away
  first); attack clicks find creatures and scripted objects (stand-in,
  Q139); drops drawn where they land and tumbling (stand-in rates,
  Q137 e); **5d**: objects loaded by scripts while playing (templates,
  `copies_view.rs`), `KQAnimOnce` ends, `tossItems`: the barrels burst,
  leave their wreck and throw their items. Q138, Q139 open (partly
  answered by the user). One commit each; see MODLOG.

- **2026-10-20 (B, M5 5c-1 to 5c-10): the fight built** from
  `fighting.md` (plan and steps in `docs/DESIGN.md` "5c"; one commit
  each; every attack, hit, miss, roll, flinch, death, drop and shot in
  `kq8-artifacts/logs/fight.log`). Characters attack at once on
  arriving, then every swing + `attackDelay` (no swallowed swing: the
  user's decision); hits at the `Do` cue (reach, × 1.25 with his back
  key, front half); rolls and protection; Connor flinches, says ouch,
  dies (he stays dead: game over is another study). Connor hits back
  with his fists: click the hand-weapon slot to ready them, then left
  click a creature (Shift: the big swing; Ctrl: the plain click) — the
  **input is a stand-in** from the manual. Hit characters flinch or say
  ouch, run after him, die with their die movement, throw their drops,
  leave a corpse and give experience when the death ends. The bow
  spriggans shoot bolts (**stand-in flight**). Combat level
  (`--combat-level 1|2|3`, stand-in for the menu). All checked in the
  logs only; **not looked at by eye, not compared with the original.**

- **2026-10-20 (B, M4 4d-7b-3): shadows fall on buildings** (walls and
  floors of the near and far buildings around the caster: characters,
  Connor, plain objects). **2026-10-21 (B, 4d-7b-5): A's answer to Q135
  built** (`characters.md` rule 37 (2): s = 0.287 × the shown model's
  stored ball radius, the band from the ball's centre, the area turned
  with the caster); checked in the logs (Connor standing: box 505 ×
  1,800, 687 below to 1,079 above his feet, as A's numbers). After the
  user's look (2026-10-21): darker (sRGB 0.5), and the area reaches the
  whole shadow so it climbs walls at once (4d-7b-5c, -5d); **checked by
  the user: "looks a lot better"**. **Checked by the user against the original
  (2026-10-20): works as expected**, shadows also show inside a
  building when Connor stands outside, as in the original. The
  original cuts some shadows off; by the user's decision we don't.

- **2026-10-20 (A): the fight studied**, in new
  **`docs/behaviour/fighting.md`** (M5's next part; not yet checked in
  the original). In short: a character's first attack is due at once,
  then every swing length + `attackDelay`; it starts only when its
  running movement can be cut short, and it stands still while swinging.
  The hit is decided at the attack's `Do` cue: within reach (× 1.25 if
  Connor walks backward) and Connor in its front half. Damage is rolled
  (25–100 % on Connor, 75–125 % on characters), then protection; dead
  below 0. A hit makes it flinch (`react`) if the roll is high and its
  last flinch ended over 2 s ago, and alarms everyone who sees Connor
  "with run". Dying: items drop at its feet at once, the die movement by
  the hit's kind, then experience to Connor, a corpse, `Die <kind>` to its
  script. Bow spriggans are ranged (they hold a crossbow), never move.
  Daventry's fighters' numbers are tabled in its rule 5.
- **Done 2026-10-20: characters moving on their own (M5 5a, 5b),
  checked by the user against the original** (the chase, the leash, the
  follow, the zombies sinking and rising again: "both match exactly").
- **2026-10-20 (B, M5 5b-1 to 5b-5): A's answers Q131–Q134 built**
  (one commit each; plan in `docs/DESIGN.md` "5b"): the exact character cylinder (goblin 0–1,600) and the one-ball
  vertical test; landing on a character's top pushes Connor 250 off; the
  stuck escape's world diagonals NE → NW → SW → SE, north when all are
  blocked, the check on only once the walk plays; **a goblin now follows
  a walking Connor at reach** and stops only when he has no forward /
  back key (the user's check in the original; no more stop-and-go); the
  way-home re-take keeps `run` (the leash can't take it home while he
  stays within reach: the original's, A); zombies sink with `hide`, go
  underground on their home point and rise again only from there.
  Earlier the same day (B, after the user's check of 5a): `getLoc`
  while playing (the graveyard zombies got no home).
- **2026-10-20 (A): Q131–Q134 answered** (`character-movement.md`,
  `connor-collision.md` rule 18, `connor-physics.md`), and the user
  checked the leash and the follow in the original.

- **2026-10-20 (B, M5 5a-1 to 5a-9): characters move on their own**,
  from `character-movement.md` (one commit per step; plan in
  `docs/DESIGN.md` "5a"; **checked in the logs only, not by eye, not
  against the original**). New core module `character_ai.rs`. Walking by
  the walk / run steps (goblin22 917 / 3,068), steering, sliding, the
  stuck escape (chicken1 stuck at once and running at 947, as the note
  predicts); its own moves meet objects, characters and Connor (Q128
  built); sight every second and the active list (6); wandering
  (`loop`, `patrol`); fidgets (goblin22 `react` / `attack`); alarm,
  telling the team (the windmill spriggans), chase up to the attack
  distance (**stand-in for the fight: it stops there**), losing sight,
  the leash, the way home; `goHome`, `setHome`, the radius commands; the
  road zombies rise (RoadZ2–4 at once, RoadZ1 after 3 s) and sink back;
  bats on the lake's surface (Q129 built). New questions **Q132** (the
  stuck escape's diagonals and line height; Connor's shape for a
  character), **Q133** (the leash loops while Connor is within reach),
  **Q134** (does the chase wait for `birth`?).

- **2026-10-19 (A): characters moving on their own studied**, in
  new **`docs/behaviour/character-movement.md`** (M5's first part, up to
  the first swing; not yet checked in the original). In short: a
  character moves only by its `walk` / `run` model's steps (bone goblin
  917 / 3,068 units/s, spriggan 1,588 / 3,366, zombie 511, bat 1,159,
  chicken 119 / 947) and steers with the 3.5 rad/s turn; no path finding,
  only a "stuck" escape (moved < 200 in 0.5 s: snap to a free diagonal,
  run; the chickens trigger it all the time). Wanderers follow their
  `loop` / `patrol` points once **active** (seen Connor once; at most 6
  at a time; lost again when out of range, and then reset to their home
  point if Connor is far); goblins and spriggans stop for a random
  fidget (`attack` / `react`) every 2–12 s. Sight: every second, from the
  top of the character's box, blocked by ground, objects and buildings
  but not by characters. Alarm when seen inside the alarm radius:
  0–1 s reaction, then turn and chase (walk or run, 50/50); gives up
  after 3 s out of sight or beyond its leash (`chase a b`: home radius a,
  chase radius b) and goes home; zombies sink back (`hide`). Corrects
  `characters.md` rules 8 and 29–31. **Q127–Q130 answered** (doors and
  chests start on `Stop`/`Stop` but their own `open` replaces the waiting
  `Stop`; characters' own touch table, Connor an obstacle in their own
  move; a flyer over water ends each update on the surface; Connor's
  `shadowMask` is 1 and `allowShadows 0` hides his shadow too).
- **New 2026-10-20 (B, M4 4d-12-1 to -3): characters are solid for
  Connor** (`connor-collision.md` rules 2–4, 18–19; one commit per step;
  plan in `docs/DESIGN.md` "4d-12"; **checked by the user against the
  original 2026-10-20**: same stopping distance at a goblin, slides
  round it; walking away, the Wizard and his conversation right). Each
  near character is an upright cylinder for him; a character hit is
  never head-on, so he slides round it (straight at its middle he stays,
  still in his walk). Checked in the logs at the goblin, the Wizard (and
  the conversation still puts him on his spot), the Uglybeast, and
  turning away. New question **Q131** (the cylinder's size; vertical
  moves; landing on a monster's top: until answered an ordinary
  landing).
- **New 2026-10-19 (B): A's character answers built** (one commit per
  step; plan in `docs/DESIGN.md` "A's answers Q116, Q117, Q121–Q124";
  **none looked at by eye yet**): the movement names start as `Stop`
  (4d-8: every Daventry character starts on `stop`, the torches and
  fish get their random start); the start-up at the first update or
  first request, Wait after `birthDelay` through the queue (4d-9);
  the end of a `speak` (4d-10: nothing when already in Wait);
  transitions on characters (4d-11; the Girl's `cowerstop`, unit tests
  only: she is never in range); gravity (4d-6b: characters fall onto
  the terrain and building floors, mask 0 only the ground check, no
  Fall state; the flying mover for the bats); shadows on the terrain
  (4d-7b-1, -2: a dark copy of the figure along the sun, 0.5 multiply,
  cut beyond 1,500; Connor's too, 4d-7b-2b). **Shadows checked by the
  user (2026-10-20):** they match the original (wading, the jump's cut),
  except that the original's flicker and gaps are, by the user's
  decision, not reproduced. Shadows on buildings (4d-7b-3, Q135
  built in 4d-7b-5) and plain objects casting (4d-7b-4) are built. New questions for A: **Q127** (doors, chests and the
  waiting `Stop`), **Q128** (what stops a character's fall: objects,
  characters, Connor), **Q129** (the flying drop over water).
- **New 2026-10-19 (B, after the user's check of 4e): 4e-8 to 4e-10.**
  The user found: offering an item, the camera cuts and their stop by the
  right button, and walking away and back all work; but the right-button
  camera turns between the Wizard and Connor in the original (now so,
  Q126 updated), and the Uglybeast still snapped and flickered (now
  fixed from A's answers: Q120 a 0 s loop never ends, Q119 delay-0 events
  before drawing). **All three checked by the user in our viewer
  (2026-10-19): fixed.** Other characters
  standing still is expected until M5 (wandering, `characters.md`).
  Q119's change makes every `EndLoop` and delay-0 event one update
  earlier, also in M3's scenes.
- **New 2026-10-19 (B, M4 4e-1 to 4e-7): conversations are built** from
  `conversation.md` (one commit per step; plan in `docs/DESIGN.md` "4e";
  **not looked at by eye yet**): the info objects (`kq8 conv-verify`: 17
  in the data, 169 `KQConner::conversation` calls all with `<enemy>` 0,
  19 characters with goodbye/welcome-back keys); the partner's second
  line queue with goodbye and welcome back; the command, the partner's
  Talk state and Connor's talk state (cases "there", "close", "walk" with
  `stopwalk` / `walkstopright`, the stay test, the end with `TalkExit`
  before the last `TalkComplete`); clicks by area (`Special`, `do`,
  `TooFar`) and `InvConv`; the camera cuts (the Wizard's first meeting
  2 3 3 5 2 1 1 1 1 3 5 2 3 6, as the note) and the right button (new
  test option `--right N`); the bottom bar away and back; Connor's
  gestures. Checked from the logs against every Daventry check in the
  note (Wizard: the three click spots, the first meeting, walking away
  and coming back; Uglybeast: case "there", `TalkExit` then
  `TalkComplete 6 11`). New log `conversation.log`. Q125 (keys before
  the stay test, Jump, push) and Q126 (the camera's aim) for A.
- **New 2026-10-19 (B, M4 4d-1 to 4d-5): characters are drawn and
  animated** from `characters.md` (one commit per step; plan in
  `docs/DESIGN.md` "4d"; checked by the user after 4d-2): the monster
  list and the types (`kq8 character-verify`: Daventry 22 + 2 files, 52
  lines, 0 problems); every character drawn from its `.anm` list (the
  orange cubes are gone; zombies hidden until born; the boss spriggan's
  skin swap; clickable by their box, not solid); the states (updated
  only while near or far, the first update, Wait and `Stop` at a random
  frame and speed, back to `Stop` after a one-shot, no state, `setState`
  and Special lists); turning toward a `speak` listener at 3.5 rad/s
  (`noTurn` blocks it); mouths (lip textures, the Uglybeast's mouth
  movements) and gestures; Wait after the last line. New test option
  `--command "WORDS",N`; new log `characters.log`. Left: A's answers to
  Q116–Q124 (below), 4d-6 gravity, 4d-7 shadows.

- **New 2026-10-19 (A): Q116, Q117, Q119–Q124 answered** (`characters.md`
  rules 8, 12, 14, 15, 18, 20, 27, 36, new 37; `talking.md` 13a;
  `anim-scripts.md` 3, 8, 10a; `connor-physics.md`). For the user's two
  sightings: **the snap (Q119)**: the original handles everything sent
  with delay 0 before drawing the frame, so `stand` replaces `Stop`
  unseen; **the flicker (Q120)**: the Uglybeast's mouth poses have a loop
  duration of 0 and never end, so no `Stop` comes back between codes
  (the "too far" line's return to idle is `ugly.cs` sending `stop`).
  Found on the way: every animated object starts with the movement
  names `Stop`/`Stop`, so all Daventry characters start on `stop`, the
  Lady and the Shadow are in Wait, and `flags=8` torches and fish get the
  random start (corrects `anim-scripts.md` rule 3). Gravity (Q122):
  characters are type 0x42 / mask 0x281F (terrain and buildings count),
  no Fall state. Shadows (Q123): along the sun, onto terrain and
  buildings, each triangle on its own, strength 0.5; `shadowvisible` is
  only for the relighting tool. **User's check:** shadows fall to one
  side like the hills', about half as bright (a quarter on overlaps), and
  land on building floors and walls, from characters and barrels alike. No fading; cut beyond 1,500
  above the ground (gone at the top of a jump); on water they lie on the
  surface (a wading Connor shows only his part above it).
- **2026-10-19 (A, B): Q118** (the world palette is `KQWorld::objectPalette`): built,
  the bars and captions show in all 12 worlds.
- **2026-10-19 (A): characters and conversations studied**
  (`characters.md`, `conversation.md`), both checked by the user in the
  original.
- **2026-10-19 (B): M3 is done** (the camera checked by the user against
  the original). **2026-10-18 (B, M4 4c):** the on-screen interface and
  Connor's inventory, checked by the user (`docs/FEATURES.md`).
- **New 2026-10-18 (B): the back flip at a wall.** Backed against a
  wall with back still held, Jump is the back flip (the user's check in
  the original); stand-in rule: `stop` with the back wall block on
  (Q112). Checked by the user.
- **Before that (A, 2026-10-18):** `interface.md` and `inventory.md`
  written and checked by the user in the original.
- **M2 and M3 (Daventry comes alive) are done**: doors, emitters,
  flowing water, animated objects, traps, items, chests, scripted
  objects, clicking, sound effects and world music, scripted animations,
  pushing and the bricks puzzle, cutscenes and the camera (checked by
  eye; see `docs/FEATURES.md`).
- **Before that (B, M4 4a, 4b): remarks and speech** (`talking.md`),
  checked by the user against the original.
- **Later for A:** Dead City's flames (11,000 vs 12,100, open
  question 9); Connor against monsters.

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* that reads the player's
own install at runtime. Roadmap M0–M7 in `docs/DESIGN.md`. Current
milestone: **M4** (remarks and speech, the interface and the inventory,
characters and conversations done; the menus postponed) and **M5**
(characters moving on their own done; the fight next). M0–M3 are done.

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
  stacking, equipment, applying, slot clicks, the hand). Viewer
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
- The interface and inventory (M4 4c): checked by the user against the
  original, 2026-10-18 (the corner panels option: looked at by the user,
  no complaint). The back flip at a wall: checked by the user.
- Conversations (M4 4e): checked in the logs only (see "Where we are").
- Tests: 548 unit tests pass (126 + 406 + 5 + 11), 2026-10-20.

## What doesn't work yet / next

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
  Unicorn), `sprdump`'s scene, a real turn in the viewer.
- **Stand-ins of 4a:** the caption panel's look. Two details of `eval` (`anim-scripts.md` rule 18) are not
  built, as no data uses them: `eval` alone gives 1 (the note: 0), and
  its words are not read again as a new line.
- **Stand-ins of 4c** (Q108–Q111): the 800 × 600 screen and how the
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
  Q111 (Space's sounds, the mask's place and size, black behind the
  bars) built from the user's checks; the exact rules are not traced.
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
- **Stand-ins of 5c (the fight):** Connor's input side (open question
  1): a second click on the ready weapon's slot puts it away; a click on
  no creature aims at the level of his feet; turning toward the click
  first only when it is behind him; readying switches his whole
  movement set to the weapon's list (`confists.anm`: fists-up stance)
  with its `draw` / `undraw`, and the pointer to the sword
  `IDBMA_CURS03` (Q138, after the user's check); his swing and flinch
  play as one-item Special lists. The bolt (open question 2):
  straight, no gravity, from 3/4 of the shooter's height at 3/4 of his,
  drawn without pitch. The thrown drops' flight (Q137 now answered in
  `thrown-items.md`, not built yet). Game over
  (Connor stays dead). The corpse has no draw-distance test and no skin
  swap (bossprig). The flinch pause is fixed at 2 s (Q136,
  `reactPause`). Not built: fleeing / keeping distance (rule 37; none
  in Daventry), the tips' texts, surprise lines (none in Daventry),
  "immune to projectiles", ranged weapons and Delete for Connor. Not
  run in the viewer: the real mouse swing, fire / ice / projectile
  deaths, picking up an item in the air, the other two bow spriggans.
  Seen in the logs: rule 26's alarm with "run" also reaches far
  characters that are active and see Connor (as written; not checked
  in the original).
- **Stand-ins of 5a / 5b (characters moving):** the idle list kept in the
  same player as `idle:<name>` (the same result as the original's swap,
  rule 18); the stuck line 1 above the feet (the note's recommendation).
  Not run in the game yet: losing sight, leaving / re-entering the
  graveyard trap, `wanderAndStop`, random wander points, landing on a
  goblin's top (unit test only), ranged characters (`doMode=1`).
- **Next step, suggested:** for the user: play the fight by eye
  (`fighting.md` "How to check" 1–8) in ours and the original. For A:
  Connor's controls (the input side of the fight, his weapon lists,
  game over), the bolt's flight, Q136. For B: the thrown items' flight
  (`thrown-items.md`). **The main
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
- Collision, not done: Connor against monsters (none drawn), ropes;
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
- For A (asked 2026-10-20 by B, M5 5c, not blocking): **Q136**
  (`reactPause`: is it the flinch's 2 s pause?), **Q138** (what readying a hand
  weapon does: the movement set, `draw` / `undraw`, the pointer). `fighting.md` open questions 1–3
  (Connor's input side, the bolt's flight, which menu sets the combat
  level) are built as stand-ins.
- For A (asked 2026-10-19 by B, M4 4e): **Q125** (Connor's keys before
  the stay test; Jump and push while talking), **Q126** (where the
  conversation camera aims and what the right-button swing centres on).
  Not a question, for A's notes: the data has **17** `KQConvInfo`
  objects, not 16 (Barren's `AlthillmanInfo` and `FrghillmanInfo` besides
  the Witch Doctor's).
- `characters.md` open questions 1, 2 and 4 (`flags` bit 0x20, how the
  renderer blends shadows, `changeBmp` / `changeAnmFile`): not blocking;
  question 3 (the AI) answered 2026-10-19 in `character-movement.md`,
  whose own open questions are now: 1 water for walking characters
  (small), 3 the stuck line's exact ground test at foot height (not
  worth copying); 2 (the fight) answered 2026-10-20 in `fighting.md`.
- `fighting.md` open questions (not blocking the Daventry fight): the
  input side of Connor's attack, the bolt's flight and aim height, which
  menu sets the combat level, whether drops spread, Connor's death.
- Q131–Q134: answered by A and built 2026-10-20 (M5 5b).
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
  while the coins are held), **Q111** (answered by the user's checks
  and built; the exact rules are worth tracing), **Q112** (what the
  back flip at a wall really tests).
- `interface.md` open questions 1–6 and `inventory.md` open questions
  1–4 (menu pause, level-up flame, rocks picture, held item on world
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
- Q41 is listed above; Q14 (b) below.
- Q42: entry 0 in textures' own colour tables (stand-in black; no visible
  difference).
- Q14b: the teleport pad's see-through look.
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
