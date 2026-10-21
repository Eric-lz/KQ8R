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
every item name in the game's data). All sound files: every `.aud` and
speech file decodes, with the lengths and rates the study measured
(`aud-verify`; `--wav <file>` writes one as a `.wav` to listen to).

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
note; checked by eye by the user (2026-10-15). Where the light maps sit
on the walls was checked against the original's placement (2026-10-17,
Q79, `kq8 lighting-verify`): no wall corner falls outside its map, and
point lights paint at the same texel positions the maps are drawn at.
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
and torches; checked by eye by the user: like the original).
Point lights on building surfaces (2026-10-16, answer to Q78; checked in
the logs and by eye by the user: "looks great"): walls, floors and
ceilings near a light get brighter, rebuilt every frame (a moving light
leaves no trail), only on the side facing the light, not through the
building's own walls; a lit surface stays visible further into the fog.
Daventry, Connor's hut: the fireplace lights 24 surfaces. Only with
dynamic lighting on (`Options.cs`, `--dynamic on`, or **F9** while the
viewer runs, which also swaps the ground's shadow and light maps as the
original's option does; checked by eye by the user 2026-10-17). Dead City's point
lights now shine whatever Connor's distance (Q77). Clicking a building
measures reach to its box in the building's own axes (Q75).

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
  along and plays on, as in the original. Backed against a wall with
  the back key still held, Jump is a back flip, as in the original
  (checked by the user 2026-10-18; the exact rule is a stand-in, Q112).
- **Crashing:** running straight into a wall knocks him back (the crash
  animation), as far as in the original (checked by eye). A turn key
  turns him while he stumbles, as in the original (2026-10-18, M3 3l-5;
  checked by eye by the user).
- **Objects are solid too** (trees, signposts, doors, crates, stone
  villagers…), with each object's box from the game's table (checked by
  eye).
- **Characters are solid** (M4 4d-12, 2026-10-20): Connor can't walk
  through the goblins, the Wizard, the Uglybeast and the others while
  they are near. Each is a round pillar for him (its size is our
  stand-in, Q131): he never bumps into one "head-on" as into a wall, so
  he keeps his walk and slides round it; walking straight at its middle
  he stays put. He can always walk away. Checked in the logs
  (`characters.log`: every hit, at 1,000 from a goblin, 600 from the
  Wizard, 940 from the Uglybeast) and **checked by the user against the
  original** (2026-10-20): he stops about as far from a goblin and
  slides round it, as in the original; walking away and the Wizard's
  conversation work.
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
objects don't stop it, as in the original. It recalculates only when
Connor moves, after `follow`, in flights and while the player uses it,
so a script's camera spot holds while he stands (M3 3n). Its position
never swings behind him by itself; only the view turns: Connor may drift
about 14° off the middle before the view catches up and locks on him.
Holding the right mouse button keeps him at the same spot on screen and
the camera at the same place relative to him; letting go lets him drift
again. The view looks a little below the point 1,400 above his feet, so
it points at about his chest (M3 3o). Checked in the logs (`camera.log`)
and by eye: the user (2026-10-19) found it matches the original.

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
- **Scripts switch animations** (2026-10-17, M3 step 3i): a movement
  asked for waits until the running one can be cut short or has ended;
  `pauseAnim` pauses. Checked in the logs (`doors.log`): the star globe
  given the spell page (`--item INVITEM_SpellPage --click starball.01,N`)
  plays `open` for 1.93 s, then `lid`. Not checked by eye.
- **Pushing** (2026-10-17, M3 step 3j; as traced 2026-10-18, 3l-4):
  walking into the windmill's hay bale with empty hands, facing it,
  pushes it along the floor at about half walking speed; pushed at an
  angle it slides along a wall, pushed straight in it stops; letting go
  plays his step back. Barrels move too: nothing stops them (they go
  through walls, as in the original), but a barrel pushed uphill stays.
  Checked in the logs (`connor.log`); the hay bale checked by eye by the
  user (2026-10-18: like the original). A barrel pushed into the pub
  goes into the wall while the wall stops Connor; once his step no
  longer reaches it he stops pushing and stands at the wall (Q95, M3
  3m-2; checked by eye by the user, 2026-10-18: like the original). Walking into Daventry's bricks (by the waterfall) runs
  their puzzle to the end (M3 step 3k): Connor's push animation, the
  table and bricks hidden then shown, the bricks go down, the waterfall
  and its sound are replaced (so the way behind it is open), the water
  changes, the camera stops following and takes the script's three
  places and views, the controls are off until the end. Connor lands
  on the cave floor in the same update as the push, so nothing pops up
  (3l-1). Checked by eye by the user (2026-10-18): matches the original.
- **Cutscenes** (2026-10-18, M3 3l-2, 3l-3): `handsOff` takes the
  controls (keys, Jump, run, mouse look, clicks) and gives them back with
  any held key acting at once; `move KQCamera` places and aims the camera
  (tilt and heading); `follow Connor` starts from where the camera is;
  `walkTo` / `runTo` / `pointAndCue` walk or turn Connor and cue the
  script. The watermill room's cutscene now ends and gives control back
  4.2 s after Connor's turn, as A traced. Checked by eye by the user
  (2026-10-18), before the camera flight was built. During a scripted
  turn Connor plays `Stop`, then `Left`/`Right`, then walks while the
  turn finishes (Q94, 3m-3). The user's check (2026-10-18): at the end of
  the watermill walk he goes on a bit further, turning left, than in the
  original (Q98); checked again after 3n: walking into the room matches
  the original; the extra step only shows when the test starts him
  inside the trap (`--at`), where the walk ends at another phase.
- **Camera flights and cuts** (2026-10-18, M3 3m-4): `addCamLoc`,
  `flyToCamLoc` and `cutToCamLoc`: the camera flies in a straight line
  at an even speed to the spot nearest it, turning to look just above
  the aim point, and on the last update jumps onto the range around it
  (the watermill room: 31 updates, then about 1,227 closer; the mill
  wheel: 46). Checked in the logs (`camera.log`) against A's numbers;
  the watermill room's flight checked by eye by the user (2026-10-18):
  looks like the original. The mill wheel cutscene (`wheeltp1.cs`)
  checked by eye after 3n (2026-10-18): no difference from the original.
  Entering the wheel from the other side (`wheeltp2.cs`) also set off
  `wheeltp1` (fixed 2026-10-18: a script's `move Connor` no longer sweeps
  the traps along the jump; Q102); checked by eye by the user after
  the fix (2026-10-18): works as expected.
- **The windmill tumble** (2026-10-18, M3 3m-1): jumping into the
  windmill's sails plays the tumble without a weapon; the script's
  `KQConner::endJump` finishes the jump, so he stands on the floor
  afterwards and can jump again; `KQConner::JumpDir` picks the back-flip
  tumble after a back flip. Checked in the logs (`connor.log`); the
  both tumbles (forward and after a back flip) checked by eye by the user
  (2026-10-18): like the original.
- **Connor's scripted animations** (2026-10-17, M3 step 3i-7): a
  script's `KQMonster::setState Connor special` list plays its movements
  one after another (also from the extra lists such as `bricks.anm`),
  then he stands. Checked in the logs with the test option `--special`;
  no Daventry script can start one yet without talking, fighting or
  pushing. Not checked by eye.
- **Animated objects:** the windmill sails, mill wheel and gears,
  swinging signs, shutters, torches, lamps, waterfalls, fish and the
  fountain's water move as the game's files say, rows of things with
  their own random start out of step as in the original; Daventry's
  torches, dead birds and fish start in step, as A traced (3l-6). Checked in the logs
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
  same as in the original. Music zones play their tune (see "World
  music"); ropes, teleporters and cutscenes
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
  eating sound plays since 2026-10-17; the health it gives is only
  logged). Other clickable things with a script (signs, the stone
  villagers...) run it too; most only make Connor speak, which is logged.

- **Sound effects** (2026-10-17, `docs/behaviour/sound.md`): doors,
  chests and scripted objects play the sounds in their movements, placed
  where they are; picking up an item and `KQSound::play` in scripts play
  plain sounds; Daventry's water wheel, mill, waterfall, windmill and
  fountain loop. Placed sounds get quieter with Connor's distance (the
  original's curve: a sudden step down at the near distance, silent
  before the far one) and come from the side of the screen they are on.
  Loudness follows the sliders in `MASK/Options.cs` (effects 0.14). At
  most 8 (+1) one-shots at once. Looping sounds pause when Connor is too
  far and switch off with the draw distance (27,100 in Daventry).
  Every start and stop is in `kq8-artifacts/logs/sound.log`. Checked in
  the logs and by ear by the user (doors, chests, pickups).
- **Footsteps and Connor's own sounds** (2026-10-17): his animations'
  cues fire (two steps per walk cycle, others in starts, stops, jumps
  and landings; his crash and falling sounds too). The step sound
  follows the ground: dirt, grass, rock or water from the ground's
  material, rock on the big bridge, silence in the cave. House floors
  without their own setting use dirt, and on a crate or table the steps
  sound like the ground under it (`sound.md` rules 39, 39a).
  Checked in the logs (`connor.log`, `sound.log`) and by ear by the user
  against the original (2026-10-17).
- **World music** (2026-10-17, `sound.md` rules 57–57l): Daventry's
  `forest.aud` plays from the start at the music volume (0.14). The
  music zones (wizard, unicorn, church) cross-fade to their tune over
  about 1.4 s when Connor walks in, and back to `forest.aud`, from its
  beginning, when he walks out; `mbattle.aud` too when scripts ask for
  it. Checked in the logs (`sound.log`, `traps.log`; the wizard's zone)
  and by ear by the user (2026-10-17). Not built: pausing music (there is no pause),
  videos, changing worlds.

## Remarks and speech (milestone M4, steps 4a-1 to 4a-9)

From `docs/behaviour/talking.md` (2026-10-18). Checked in the logs
(`talk.log`, `sound.log`, `connor.log`) and by the user against the
original (2026-10-18): the sign's turn, line, caption, background drop
and mouth; a movement key stopping a tombstone's reading; "too far".
The two differences the user found then (a held turn key cancelled the
automatic turn; some tombstones didn't turn him) were fixed in steps
4a-7 and 4a-8 (Q104, below). Also checked by the user (2026-10-18): an empty
chest says "'Tis empty."; with an item in hand (`--item`) an object that
doesn't take it gives "This will not work."

- **Lines:** the message files (`.MSG`), voices and lip-sync files are
  read (`kq8 msg-verify` matches the note's counts: 2,080 lines, 476
  that can't be cut short).
- **Scripts:** `KQMonster::speak` queues a character's lines, once (a
  remark already waiting is not added again); Connor first turns to
  what he talks about (in Wait or Move), then speaks 0.5 s later.
  `clearSpeak`, `testMessage`, `stateChangeNotify` and `TalkComplete`
  (scripts hear when a line ends) work.
- **Hearing it:** Connor's voice plays plain, others' placed at the
  speaker; the speech slider (0.34); effects and world music drop to
  0.2 of their loudness while a line plays.
- **Captions:** the line's text at the bottom of the screen while he
  talks (`Options::Captions` is true in `Options.cs`), in the game's own
  gold font on its stone strip, centred, the strip growing upwards one
  line at a time (M4 4b-3). Checked by the user: colours right. Since
  4c-1 the stone covers the whole caption box (400 pixels of an
  800 × 600 screen), rows break as the original's rule says, and the
  box sits a little left of the middle (the map is closed).
- **Mouth:** Connor's face texture follows the lip-sync file (`CF0`–`CF7`).
- **Connor's own remarks:** "'Tis beyond my reach." (too far), "This will
  not work." (an item not taken), the chests' "'Tis empty." with a turn
  to the chest, in any state of his (also mid-jump; 4a-9).
- **Turning to what he clicks (4a-8):** after any click that runs an
  object's script (doors, signs, tombstones, buildings and items with a
  script), Connor turns to face the object, standing or moving, unless
  the object says `doNotPointAt` (items never turn him). Only one turn
  at a time: a remark's turn and the click's are one turn. Stock
  remarks ("too far", "will not work") don't turn him. Checked by the
  user (2026-10-18): `tmbstn1` reads at once and turns while reading,
  `tmbstn2` turns first and reads half a second later; the wizard's book
  turns him, the church urn doesn't.
- **Turn keys during an automatic turn (4a-7):** only pressing a turn
  key cancels the turn; a key already held does nothing, and when the
  turn ends he stops turning until the key is pressed again. Held
  forward keeps him walking through the turn. Checked by the user
  (2026-10-18).
- **Cutting short:** a movement key, turn key, Jump or a click stops
  Connor's line unless it is marked as not stoppable (most remarks are).
- Checked runs: the alchemist's sign (turn, 0.5 s, 3.542 s line, clicks
  during it ignored), `smlchst3` ("'Tis empty."), a tombstone cut short
  by a turn key, a tombstone clicked from too far.

## Conversations (milestone M4, step 4e)

From `docs/behaviour/conversation.md` (2026-10-19). Checked in the logs
(`conversation.log`, `talk.log`, `camera.log`, `connor.log`), **not yet
by eye**.

- **Starting one:** clicking the wizard or the beast at the pond. Where
  Connor stands decides, not how far he is: in the area in front of the
  partner the talk starts; behind the wizard's back he says "Come around,
  lad..."; farther away "Hello, there, sir wizard!" (`kq8 conv-verify`
  checks the areas and cameras of all 17 partners in the data).
- **Getting to his spot:** close by, Connor is put on his talking spot;
  from farther inside the room he turns, starts walking and is brought
  to the spot by his walk-stop movement, then turns to the partner.
- **During the talk:** the lines play one after the other with voices,
  captions and lip sync; the bottom bar slides away and comes back at
  the end; the camera cuts between the partner's fixed cameras (for the
  wizard's first meeting 2, 3, 3, 5, 2, 1, 1, 1, 1, 3, 5, 2, 3, 6, as the
  study found); Connor and the wizard gesture with their hands; clicks
  in the world and the movement keys don't cut lines short. Holding the
  right mouse button stops the cuts and lets the camera turn around the
  partner.
- **Walking away:** leaving the area ends the talk; with lines left the
  partner says goodbye ("Well, then, off with you!..."); clicking him
  again plays the welcome back and the talk carries on from the line
  that was cut.
- **An item on the partner during the talk** ends it and offers the
  item (`InvConv`).
- Checked by the user (2026-10-19): offering an item, the camera cuts
  and the right button (turning between the two, as in the original),
  walking away and coming back; the Uglybeast's turn and lines without
  snapping or flickering.
- Stand-ins until the study session answers: Q125 (keys during the walk
  to the spot, Jump and pushing while talking), Q126 (where the
  conversation camera aims).

## Characters moving on their own (milestone M5, steps 5a, 5b; done)

From `docs/behaviour/character-movement.md` (A, 2026-10-19, completed
2026-10-20). **Checked by the user against the original (2026-10-20):**
creatures move as in the original, the chickens and bats keep to the
same routes, goblins, spriggans and zombies follow Connor; a goblin
follows a walking Connor at reach, stops when he stands, and runs home
when he gets far enough away; zombies rise, follow, sink when he is far
and rise again from their graves when he comes back (the graveyard's
too). Details and numbers in `kq8-artifacts/logs/characters.log`.

- **Walking:** a character moves only by its `walk` / `run` model's steps
  (goblin22 917 / 3,068 units a second, chickens 119 / 947, bats 1,159),
  turns at 3.5 rad/s while walking, slides along walls, and gets "unstuck"
  by snapping to a free diagonal and running (the chickens do it all the
  time, as the note predicts). Its own moves meet objects, other
  characters and Connor.
- **Noticing Connor:** every second each character tests its line of
  sight from the top of its box (the ground, objects and buildings block
  it, characters don't); at most 6 in Daventry become "active".
- **Wandering and fidgeting:** active characters follow their points
  (`loop`, `patrol`), and goblins and spriggans stop now and then for a
  fidget from their idle list.
- **Alarm and chase:** seen within its alarm radius, a character reacts
  after up to a second, tells its team and walks or runs after Connor; it
  follows him at reach while he walks and stops when he stands (where the
  fight will start); it gives up when it loses sight of him for 3 s or
  goes beyond its leash, and goes home. Scripts can set its home
  (`getLoc`, `setHome`), its radii and `goHome`.
- **Zombies:** they rise when Connor comes within their alarm radius
  (after their `birthDelay`), sink back when they give up, and rise again
  from their home when he comes back.
- **Bats:** they fly their loop and sit on the water surface over the
  lake.
- Built from A's answers Q131–Q134 (M5 5b): the exact character
  cylinder, landing on a character's top pushes Connor off, the stuck
  escape's compass diagonals, following a walking Connor at reach (the
  user's check in the original), zombies sinking home and rising again
  from there.
- Where its attack is due it attacks (M5 5c, below).

## The fight (milestone M5, step 5c, 2026-10-20)

From `docs/behaviour/fighting.md` (A, 2026-10-20). Every attack, hit,
miss, roll, flinch, death, drop and shot is in
`kq8-artifacts/logs/fight.log`. **Checked in the logs only, not by eye.**

- **Numbers:** every Daventry fighter's health, damage, protection,
  experience, attack delay, drops and corpse from the monster lists and
  the item list (`kq8 character-verify` prints them; all match the
  note's table).
- **Creatures attack:** at once when they reach Connor, then every swing
  plus attack delay (goblin22: every 2 s); they stand still while swinging
  and keep turning to him. A chasing creature always swings in full on
  arriving (the original's "swallowed swing" is not reproduced: the
  user's decision).
- **Hits on Connor:** at the swing's hit moment, within reach (a step
  back gives 1.25 × reach) and in front of it; damage rolled 25–100 %,
  then his protection; he flinches now and then (`react`) or says ouch;
  below 0 health he dies and stays dead (game over is not built).
- **Readying weapons (M5 5f-3, 5f-4):** a weapon slot readies that
  weapon (again: puts it away); Delete swaps between the hand and the
  ranged weapon (with nothing ready: the one put away last), Alt puts it
  away; only while Connor stands or walks. The fists come up with
  `draw` and go down with `undraw`; every other change puts the old
  weapon away and takes the new one out, and his whole set of
  movements becomes the weapon's. The pointer: sword, crossbow arrow or
  the golden one (Ctrl held with a weapon ready: the golden one). The
  Small Crossbow (`sprgnbow`'s drop) can be readied.
  `KQConner::setDoMode` / `getDoMode` work.
- **Shooting (M5 5f-5):** with the crossbow ready a left click aims at
  what it points at (a creature, an object, a building, the ground, or
  far away into the sky; never steeper than about 33°); Connor turns to
  it (far behind him he shoots near the end of the turn), plays his
  shot and the bolt leaves from the crossbow's tip, flies straight to
  the spot and on. No ammunition. A click on the Wizard or the Knight
  talks instead. Walking or running he shoots at once without stopping,
  at most twice a second, and a run slows to a walk until a second after
  the last shot (M5 5f-6). Checked by the user against the original
  (2026-10-21). Test options `--aim NAME,N`,
  `--aim-point X,Y,Z,N`.
- **Connor hits back (M5 5f-7):** with his fists ready a left click
  swings (`attack` / `attackb`; Shift: `attack2`, × 0.75–4 damage; Ctrl:
  the normal click). A creature or object clicked within his reach
  becomes his target and stays it, so a click on the ground still swings
  at it; far behind him he turns first. The Wizard and the Knight get
  the normal click. Test option `--swing NAME,N[,big]`.
- **Hit creatures:** they flinch (`react`, never twice within 2 s of the
  last) or say ouch, run after him from then on, and alarm everyone who
  sees him; hit unseen, a lone one runs about beside him until it finds
  him.
- **Barrels (M5 5d):** a punch breaks Daventry's barrels: each bursts,
  leaves its wreck, and the crystal and mushroom barrels throw their
  items (objects loaded by scripts while playing, `copies_view.rs`).
- **Dying:** its die movement; its drops fling up and land with their
  sound; when the death ends Connor gets its experience, a body stays
  where it fell, and it is removed.
- **Thrown items (M5 5e):** drops and barrel items fly in the traced arc;
  walls, roofs, floors and objects stop them (at a wall they drop to its
  foot), barrels and other scripted or animated objects throw them up
  again; they tumble about one axis and end lying flat; the land sound
  plays at the first touch. Characters are flown
  through.
- **Bow spriggans:** they never move, and shoot a bolt every swing plus
  delay (`sprigwnd1`: 1.7 s), aimed ahead of a running Connor; the
  bolt flies straight and tilted toward its aim, no gravity, and stops
  at the first building, object, character or Connor, the ground or
  after its range (M5 5f-1).
- **Combat level:** `--combat-level 1|2|3` (0.66 / 1.0 / 1.5 × the
  creatures' health and damage; stand-in for the menu).
- Not built: Connor's bows and other weapons' full movement sets, game
  over, fleeing (no Daventry creature flees), the tips' texts.

## The on-screen interface (milestone M4, step 4c)

- **The bottom bar (4c-2):** the original's panels from `Gplayscr.gui`
  in the world's pictures and fonts: armour, hand-weapon and ranged
  slots with their numbers (0, 10, 0 at the start), the eight
  health/potion slots (empty for now), the red health and green
  experience bars with the level between them, and the rope, rocks and
  map icon on the right (empty). **Space** slides the whole bar away or
  back in a fifth of a second. A change of health or experience while
  the bar is away pops the meters up alone for 1.5 s. A changed number
  flashes its slot. Cutscenes (`handsOff`) slide the bar away and show
  the spinning mask; afterwards it comes back only if it was shown.
  The caption sits on top of the bar. Clicks on the bar don't reach the
  world. Logged in `gui.log`. Checked by the user 2026-10-18.
- **The inventory bars (4c-3):** the top bar (mask button, the quest
  strip with its arrows, gold and its count) comes down when the pointer
  reaches the top of the window and goes up when it moves away; it also
  drops down for 1.5 s when a quest item comes in. The eight slots show
  mushrooms, crystals, waters, elixirs and the four potions with their
  counts; keys **1–8** or a click use one (health added, capped; potions
  wear off after their time). Picked-up stacks join (one gold entry);
  a new weapon or armour piece pushes out the old one, which drops
  where the new one lay, and its numbers change the bar. New items and
  higher counts blink. A right click on an item: Connor describes it
  (voice only). Logged in `gui.log` and `items.log`. Checked by the
  user 2026-10-18.
- **The item in hand (4c-4):** a left click on a quest item, the gold,
  the rope or the rocks puts it on the mouse pointer (the game's own
  pointer pictures; the play pointer otherwise) and takes it out of the
  quest strip; clicking it again puts it back. Clicking something in the
  world with it offers it: if the object takes it (its accept list, as in
  the original) the hand empties, otherwise "This will not work." and it
  stays on the pointer. The scroll can't be taken: Connor reads it out.
  Escape still quits (the original opens its main menu, which pauses the
  game; there is no menu yet). Checked by the user 2026-10-18.
- **A 4:3 screen in a wide window (4c-6):** the panels and the caption
  stay in the original's 4:3 shape, centred in the window; the world
  fills the whole window around them.
- **Corner panels (4c-11, not in the original, off by default):** on
  a wide window, F6 (or `--corners on`) moves the bottom bar's left
  panel (armour, weapons) and right panel (rope, rocks, map icon) to
  the window's edges; the rest stays centred.
- **Corrections after the user's checks (4c-7, 4c-8):** Space plays the
  top bar's sounds the other way round as the bottom bar moves
  (cutscenes don't); the spinning mask sits whole in the screen's bottom
  right corner at its own size, standing on the bottom bar and rising
  and falling with it (4c-9); the empty part of the meters' bars is
  black; the meters fit the bar without a gap (4c-9).

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
- **F9:** dynamic lighting on or off, like the original's video option:
  point lights on the ground, walls and floors come and go, and the
  ground swaps between its shadow map and light map; shown on screen.
  Models keep their point lights either way. Checked by eye by the user
  (2026-10-17): the moving lights go, the baked ones stay, and all come
  back.
- **Graphics API** (`--renderer`): draw with Vulkan or DirectX 12 instead
  of letting the graphics library choose; `viewer.log` says which one
  ran. Checked by the user 2026-10-16 on Windows with one NVIDIA card;
  not tested on Linux or other graphics cards. OpenGL is not built in.

## Launch options (viewer)

Add them after `cargo play`, for example
`cargo play --world Daventry --frames 120` (the same as
`cargo run --release -p kq8-viewer -- --world Daventry --frames 120`).
With no model name the world's ground opens; `--model` opens one model.
`cargo play --help` prints the list of options.
"Test" options drive Connor from the command line so a run can be
repeated and checked in the logs. They do not change the game's rules.

### Showing things

- `--world <name>`: which world to load (default `Daventry`). The names
  are listed in [README.md](../README.md); `--world` with no name prints
  them.
- `--model <file>`: show one model on its own, e.g. `--model smlchest.KQS`
  (default: no model, so the ground opens). Add `--palette <file>` to
  colour it with another palette; that option works only with `--model`.
- `--light x1|x2|off`: how the ground and the objects are lit at the
  start (default `x1`). F7 changes it while the window is open.
- `--corners on|off`: the bottom bar's left and right panels at the
  window's edges on a wide window (default `off`, as the original). F6
  changes it while the window is open.
- `--haze on|off`: haze at the start (default `on`). F8 changes it while
  the window is open.
- `--dynamic on|off`: dynamic lighting (point lights painting the
  ground and building surfaces), instead of what `Options.cs` says. The
  game's own files are not changed.
- `--sound on|off`: play sound through the sound card (default `on`).
  `off` still runs the sound rules and writes `sound.log`.
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
  number N, forward and turn are let go after N updates. The turn key
  counts as pressed once at the start and let go at N.
- `--turn N[,1|-1]`: press a turn key before update N + 1 and keep it
  pressed (1 left by default, -1 right). It is one press: after an
  automatic turn it does not turn him again (as a held key in the
  original).
- `--jump N`: press Jump once, before update N + 1 (repeatable).

### Clicking and items (ground view only)

- `--click NAME,N`: click the object called NAME before update N + 1. Can
  be given more than once. It skips the line-of-sight test (see below).
- `--aim NAME,N`: like `--click`, but the click only works if the camera
  can see the object, as in the game.
- `--swing NAME,N[,big]`: at update N ready Connor's hand weapon (if none
  is ready) and swing at the creature NAME (`big`: the Shift swing).
- `--combat-level 1|2|3`: the combat level before the creatures enter
  the world (0.66 / 1.0 / 1.5).
- `--item TYPE`: give Connor an item of that type (called `TestItem`)
  and put it in his hand, for example the key that opens the castle
  keep's doors.
- `--key NAME,N`: press a panel key before update N + 1 (`space`: put the
  bottom bar away or bring it back; `1`–`8`: use a slot), or click a
  control of the bars (`left:IDCTL_INVBAR_GOLD`, `right:IDCTL_HEALTHBAR_1`,
  names as in `Gplayscr.gui`). Can be given more than once.
- `--command "WORDS",N`: run one script command before update N + 1,
  as if a script did (words split at spaces; no variables). For example
  `--command "KQMonster::speak Wizard Connor 1000 4 0 8 1 1",30` makes
  the Wizard say a line. Can be given more than once.
- `--right N`: the right (camera-move) button counts as held from update
  N on, for a conversation: the camera stops cutting between the
  conversation cameras and turns around the person Connor talks to.

An "update" is one step of the game's rules, the same step each time you
run the test. Unknown options and bad values stop the viewer with a
message that shows the usage.

## What doesn't work yet

- **Not playable:** there is no inventory screen, saving, game over, or
  travel between worlds; the story's scenes run only as far as the
  built parts allow.
- **Characters:** the fight is a first version (M5 5c): Connor's
  controls are partly stand-ins; not
  compared with the original by eye. Their shadows on buildings
  follow the traced rule (Q135, 4d-7b-5), without the original's cut
  shadows (the user's decision); checked by the user.
- **Not drawn:** fire and ice arrow sparks and
  the dragon fire (they come with fighting), the teleport pad's look.
- **Sound:** music doesn't pause (no pause yet); monsters have no
  footsteps (there are no monsters yet); scenery
  animations (signs, the mill wheel's own movement) make no sound.
- **Clicking:** doors, chests, items, scripted objects, buildings and the
  characters with a script react; nothing else does.
- **Animated objects:** scripts switch them (M3 step 3i), but most
  Daventry scripts that do it start from things not built yet (fights,
  talking, `loadKQ` with a name or place). In Gnome (a later world, not
  part of M3), a rolling rock that runs into Connor does nothing to him
  yet (in the original he stumbles and takes damage).
- **Jumping into things:** a standing jump into a tree or wall makes
  Connor stumble (the crash and its sound), as the user saw in the
  original; when he starts falling the stumble is cut short and the
  falling animation takes over (also the user's check). The rule behind
  it is now A's (Q82). A back flip into a wall slides along it with
  gravity off until the flip ends (Q84 a, 3l-5; unit test only).
- **Conversations:** not checked by eye yet; the horn on the beast,
  `clearSpeak` on a partner and a real mouse drag during a talk were
  not run.
- **Camera:** no first person view (F2) or camera shake; the camera's
  roll is not drawn; the mouse's recentring near the window's edges is
  not built.
- **Cutscenes:** the windmill tumble (`miltrap1.cs`) plays the
  no-weapon version (`KQConner::getDoMode` answers empty hands, 2026-10-18; checked by eye by the user;
  weapons come with M5).
- **Water:** no damage yet (poison, lava, swamp).
- **Collision:** some objects have no box in the game's table and stay
  walk-through (as in the original, if we read it right); no ropes.
