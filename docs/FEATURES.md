# What works, in detail

The full list behind the short summary in [README.md](../README.md): what
each feature does, how close it is to the original, and how it was
checked ("by eye" = the user looked at it in the viewer; "in the logs" =
checked from the numbers in `kq8-artifacts/logs/`). Newest details are in
[MODLOG.md](../MODLOG.md); open questions in
[behaviour/QUESTIONS.md](behaviour/QUESTIONS.md).

## Reading the game's files

Archives, images, palettes, 3D models, buildings, the ground of every
world, the scripts that set up each world, and the list of all 135 item
types (coins, crystals, weapons...; checked with `item-verify` against
every item name in the game's data).

## Seeing a world

The ground, the buildings and the objects the game places at the start,
each where the original puts them, with animated objects at their
starting pose and see-through things (waterfalls, fountains) see-through.
Checked by eye in Daventry; the other 11 worlds load but have only been
looked at in a few places. The ground is lit by the game's light maps
(hill shadows), compared with the original by eye in Daventry
(2026-10-14): as stored (x1) matches. Since 2026-10-16 the map is blended
bilinearly between its points, as the original's card does it (checked
by eye by the user). Objects (and Connor) are lit by the
world's ambient light and sun (2026-10-15): in Daventry 30-42% of their
texture's brightness, the sunny side a little lighter. Checked in the
logs (Connor and the three objects nearest his start stay within
0.30-0.42, as the behaviour note says) and by eye against the original
(user, 2026-10-15: much closer to the original).
Buildings are lit by their own pre-made light maps (`.dil`, 2026-10-15):
every building in 10 worlds, Connor's hut's numbers match the behaviour
note; checked by eye by the user (2026-10-15).
Haze (2026-10-15): objects (and Connor, items, leaves) fade into the
world's haze colour with distance from the camera and are left out past
the haze end (Daventry 18,000 to 22,950 units, as the behaviour note
says), unless their file sets `b_hazeSolid`. Checked in the logs and by
eye by the user (2026-10-15).
Haze as the original does it (2026-10-16, behaviour note answers to Q68
and Q69): the haze colour is the world palette's colour nearest to the
script's, the background is the palette colour `newSky` names (Daventry:
the same dark blue); the ground fades per corner up to 99% at
`visibleDistance` and ends there; building walls fade per surface and
drop out at 99% (except buildings marked `b_hazeSolid`, like Daventry's
tomb); falling leaves fade together per emitter; objects switch off when
they are `visibleDistance` from **Connor** (Dead City's flames at about
12,100, checked in the log). Checked by eye by the user (2026-10-16):
matches the original.
Draw distance details (2026-10-16, answers to Q70, Q71, Q74; checked in
the logs only): how far an object reaches is measured from its box in
the game's box table (not its model); items far from Connor are not
drawn; objects that are far or switched off (and hidden ones) are not
solid and can't be clicked, and come back when Connor gets near.
Point lights on models (2026-10-16, answer to Q72; checked in the logs
and by eye by the user: the portal lights Connor): Dead City's seven point lights and the lights
some models carry (the arrival portal's circling green light, torches,
coals; in Daventry lamps, candles, chandeliers, fireplaces) light
nearby objects and Connor, as in the original. Since 2026-10-16 they also
paint soft patches of light on the ground (Dead City's arrival portal
and torches; checked by eye by the user: like the original). Walls don't get them yet.

## Connor

- **Walking:** he stands at the world's start point, walks, runs, steps
  back and turns with the original's keys and speeds, with his
  animations.
- **Start and stop:** starting to walk or run plays the original's short
  start animation first (with its small step to his left), then the walk
  or run. Letting go, he finishes his step and plays the matching stop
  animation. Both checked by eye.
- **Leaning into turns:** running and turning at the same time, he leans
  into the turn, up to about 10° (0.18 rad, as traced), from the running
  start on, and straightens up when he stops turning or running or jumps,
  also when running through water. Checked by eye against the original at
  0.15 rad and for jumps; the 0.18 end value only in the logs so far.
- **Hills and falls:** he follows hills, runs down steep walkable slopes
  without falling, can't climb slopes steeper than about 37° (he slides
  back), and falls off high places with the falling, landing and death
  animations. Falls and slides are as fast as in the original (gravity
  measured in the running game).
- **Buildings are solid:** he stops when he walks straight into a wall,
  slides along it at an angle, climbs low steps, and stands on building
  floors. Checked by eye: he goes inside houses, crosses bridges and
  slides along walls. After walking into a wall he stands idle until
  forward is released and pressed again, or a turn key is pressed, as in
  the original (the turn key part checked in the logs, not yet by eye).
  Holding a turn key there keeps him turning until he slides off the
  wall, as in the original (checked in the logs, not yet by eye).
- **Jumps:** a standing jump (Insert) from standing or walking, with the
  original's arc, and running and backward jumps (Insert while running or
  stepping back), all checked by eye. Jumping into a wall or steep slope:
  a standing jump stops him, a running jump crashes, a back flip slides
  along and plays on, as in the original.
- **Crashing:** running straight into a wall knocks him back (the crash
  animation), as far as in the original (checked by eye).
- **Objects are solid too** (trees, signposts, doors, crates, stone
  villagers…), with each object's box from the game's table (checked by
  eye).
- **Water:** he walks into water on the bottom, with the water up to his
  chest, and switches to the wading walk and run while it is deep.
  Buildings and objects stand on the ground under the water where the
  original puts them (e.g. the Swamp shack). Checked by eye (2026-10-12).

## The camera

Follows Connor like the original's: it trails behind him, keeps above the
ground, and can be swung, zoomed and tilted. It doesn't go through walls,
closed doors or hills: when a building gets between it and Connor it
swings round to look along the way he came, and comes closer if it has
to; dragging it into a wall only creeps it closer. Trees and plain
objects don't stop it, as in the original. Checked in the logs
(`camera.log`) and by eye.

## Daventry comes alive (milestone M3)

- **Doors:** a left click on a door uses it, as with the game's play
  cursor. In reach (the original's 4,700 units from the door) the door's
  own script runs, which opens it: it swings open on its hinge in the
  original's time, and its box swings with it, so the doorway is free and
  the open door still blocks Connor where it stands. Plain house doors
  stay open on a second click; from too far nothing happens (Connor's
  remark is only written to the log); the castle keep's doors open only
  with the key (test option `--item`; there is no inventory screen yet).
  Checked in the logs (`doors.log`) and by eye with real mouse clicks.
- **Clicks and walls:** a wall, floor or ceiling between the camera and
  an object takes the click and nothing happens, as the original does it
  (behaviour note answer to Q63, 2026-10-16). The ground no longer stops
  a click, and the church, cave and wizard's house don't either (their
  files say so). Checked in the logs (the pub's ceiling takes the click
  aimed at the chest upstairs); the earlier version (walls and ground
  block) was checked by eye.
- **Items an object accepts:** lists of several items and "accepts
  anything" work, and scripts can add to them (2026-10-16; the keep's
  doors with the key checked in the logs).
- **Script details (2026-10-16, logs and unit tests):** an object's
  script hears when its play-once movement ends one update later (as the
  original's event queue does), doors too; `getEndLoop`; scripts can
  delete any object; new items are named `NoName0`, `NoName1`, ... as in
  the original.
- **Falling leaves and stream drips:** the small particles the game keeps
  flying around Daventry's trees and over the stream, with the original's
  speeds, lifetimes, swaying and gravity. Leaves that drift into the
  cottage vanish at its box, as in the original. In IceWorld it snows
  around Connor wherever he goes, but not inside buildings. Checked in
  the logs (`effects.log`) and by eye.
- **Flowing water:** the rivers and the stream flow: their ground
  textures change every 0.3 s (timed from each change, as the original
  does). Also Swamp's water and DeadCity's blood. Checked in the logs
  (`ground_anim.log`) and by eye.
- **Animated objects:** the windmill sails, mill wheel and gears,
  swinging signs, shutters, torches, lamps, waterfalls, fish and the
  fountain's water move as the game's files say, torches and other rows
  of things out of step as in the original. Checked in the logs
  (`anim.log`) and by eye. Gnome's three rolling rocks roll out and come
  back each cycle, and their collision boxes move with them (checked by
  eye and by walking to a rock's start spot while it is away). Gnome's
  pushable `boulder` stands still, as in the original (checked by eye);
  it can't be pushed yet.
- **Traps:** Daventry's 52 invisible trap boxes are loaded and switch on
  and off as Connor walks in and out of them (including the music zones'
  smaller "get in" box), running their scripts. Walk and run zones work:
  inside Connor's house (and the other houses with such zones) he drops
  to a walk and the camera follows closer (1,000–2,500); outside he runs
  again (1,500–5,000). Checked in the logs (`traps.log`) and by eye, the
  same as in the original. Music, ropes, teleporters and cutscenes
  started by traps are later milestones (their commands are only logged).
- **Items lying around:** the coins, mushrooms, crystals, weapons and
  other things the original puts in Daventry at the start (22, from its
  start-up scripts), each with its own model. Clicking one near Connor
  picks it up: it disappears into his inventory (which can't be seen yet;
  `items.log` counts it), and items with their own script run it (the
  farmhouse coins). Checked in the logs and by eye (the crystal).
- **Chests:** the big chest upstairs in the pub opens on the first click
  and three crystals appear in it; the second click takes them, the third
  only logs Connor's "it's empty" remark. The same for the chest of coins
  in the keep. The wizard's hidden chest stays hidden (it needs a Reveal
  potion). Checked in the logs and by eye (the pub chest, also that it
  can't be clicked through the ceiling or walls).
- **Small chests, the jar and the mushroom baskets:** the small chest by
  the pub's keg and the one at the farm drop their coins beside them and
  open, then only "it's empty"; the baskets give Connor their mushrooms
  and turn into empty baskets. Checked in the logs and by eye.
- **Fish and other scripted things:** clicking a swimming fish near
  Connor makes it disappear, as in the original (checked by eye; its
  eating sound and the health it gives are only logged: no sound or
  health yet). Other clickable things with a script (signs, the stone
  villagers...) run it too; most only make Connor speak, which is logged.

## Viewer helpers (not in the original)

- **H:** shows things the game hides or never uses (traps, sound sources,
  unused pictures, objects that appear later in the story; people and
  monsters as orange cubes).
- **U:** shows the hidden ground under the water (the "floor"): first
  press both (the visible ground see-through where the floor differs, the
  floor tinted blue where lower, red where higher), second press the
  floor only, third press back to normal.
- **F:** a free flying camera.
- **G / B:** lift Connor high / a little and let him fall.
- **F7:** the light of the ground and the objects as in the game (x1,
  default), doubled (x2), or off (full bright); the mode is shown on
  screen. x1 is the default because it matches the original by eye
  (user, 2026-10-14, ground).
- **F8:** haze (distance fog) on or off; shown on screen.
- **Graphics API** (`--renderer`): draw with Vulkan or DirectX 12 instead
  of letting the graphics library choose; `viewer.log` says which one
  ran. Checked by the user 2026-10-16 on Windows with one NVIDIA card;
  not tested on Linux or other graphics cards. OpenGL is not built in.

## Launch options (viewer)

Run them after `--` in the command, for example
`cargo run --release -p kq8-viewer -- --world Daventry --frames 120`.
With no model name the world's ground opens; `--model` opens one model.
"Test" options drive Connor from the command line so a run can be
repeated and checked in the logs. They do not change the game's rules.

### Showing things

- `--world <name>`: which world to load (default `Daventry`). The names
  are listed in [README.md](../README.md).
- `--model <file>`: show one model on its own, e.g. `--model smlchest.KQS`
  (default: no model, so the ground opens). Add `--palette <file>` to
  colour it with another palette; that option works only with `--model`.
- `--light x1|x2|off`: how the ground and the objects are lit at the
  start (default `x1`). F7 changes it while the window is open.
- `--haze on|off`: haze at the start (default `on`). F8 changes it while
  the window is open.
- `--renderer auto|vulkan|dx12`: which graphics API draws the picture
  (default `auto`: the graphics library chooses, Vulkan on our test PC,
  and the `WGPU_BACKEND` environment variable still works). `vulkan` and
  `dx12` force that one. OpenGL is not built in. `viewer.log` says which
  one was used (`renderer: asked ... got ...`).
- `--game <folder>`: where the game's files are (default `MASK`). The
  environment variable `KQ8_GAME_DIR` sets the same folder.
- `--frames <number>`: close the window by itself after that many frames,
  and write the result line in the log. Useful for tests.

### Starting Connor somewhere (ground view only)

- `--at X,Y,H`: start him on the ground at X, Y, facing heading H in
  radians (instead of where the world's script puts him).
- `--lift <number>`: lift him that high once at the start, then let him
  fall. Same as the G and B keys (5,000 and 2,000).

### Moving Connor (ground view only)

- `--hold F,T,R[,N]`: hold forward (1 forward, -1 back), turn (1 left,
  -1 right) and run (1 on, 0 off) instead of the keys. With the fourth
  number N, forward and turn are let go after N updates.
- `--turn N[,1|-1]`: press a turn key before update N + 1 and keep it
  pressed (1 left by default, -1 right).
- `--jump N`: press Jump once, before update N + 1.

### Clicking and items (ground view only)

- `--click NAME,N`: click the object called NAME before update N + 1. Can
  be given more than once. It skips the line-of-sight test (see below).
- `--aim NAME,N`: like `--click`, but the click only works if the camera
  can see the object, as in the game.
- `--item TYPE`: put an item of that type in Connor's hand, for example
  the key that opens the castle keep's doors.

An "update" is one step of the game's rules, the same step each time you
run the test. Unknown options and bad values stop the viewer with a
message that shows the usage.

## What doesn't work yet

- **Not playable:** there is no story, talking, inventory screen,
  fighting, saving, or travel between worlds.
- **Not drawn:** people and monsters, point lights on walls (they light
  models and the ground so far), fire and ice arrow sparks and
  the dragon fire (they come with fighting), the teleport pad's look.
- **No sound or music** at all.
- **Clicking:** doors, chests, items, scripted objects and buildings
  react; nothing else does. Connor says nothing (his remarks, e.g. "too far" or
  "locked", are only logged) and doesn't turn to face what he clicks.
  Picked-up items are only counted in the log.
- **Animated objects** play only the movement they start with (scripts
  don't switch them yet) and make no sound. A rolling rock that runs
  into Connor does nothing to him yet (in the original he stumbles and
  takes damage). Pushable objects can't be pushed.
- **Camera:** it doesn't swing back behind Connor by itself; no first
  person view (F2), camera shake or cutscene cameras.
- **Water:** no damage yet (poison, lava, swamp).
- **Collision:** some objects have no box in the game's table and stay
  walk-through (as in the original, if we read it right); no ropes.
