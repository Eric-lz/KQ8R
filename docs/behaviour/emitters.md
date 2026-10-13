# Emitters, particle catchers and ground animators

Study session A, 2026-10-12. Sources: `Mask.exe` (studied, described here
in our own words) and the game data (`daventry/kq/*.kq`, `permwrld.cs`,
`newWorld.cs`, `daventry/resource/terrm.dib`, `envinfo.kq`, the `.kq` files
of all 12 worlds for the key lists). "Traced" = read in the exe; "data" =
read or computed from the files with a throwaway script (not in the repo).
Covers three object kinds: `KQEmitter` (small particles: falling leaves,
water drips, sparks), `KQParticleCatcher` (stops particles inside a
building) and `KQTerrainAnimator` (the flowing-water ground textures).
Builds on `objects.md` (loading, see-through objects) and `terrain.md`
(ground squares and materials).

## Purpose

- **Emitter:** a point that keeps a fixed number of particles alive. Each
  particle is a small model (`.kqs`) that flies off with its own speed,
  may spin, sway and fall, and starts again at the emitter when its life
  runs out. Leaves drifting from trees, drips from the stream.
- **Particle catcher:** makes one emitter's particles disappear when they
  enter a building's box, so leaves don't fall through the roof into the
  room.
- **Ground animator:** every few tenths of a second changes the ground
  texture of a set of grid squares to the next texture of a list, which
  makes water look like it flows.

## What Daventry uses (data)

| Objects | Kind | Loaded by | What |
| --- | --- | --- | --- |
| `leaf1`–`leaf10` (`leaf.kq`, `leaf2–10.kq`) | emitter | `permwrld.cs` (`leaf8–10` later in the file) | falling leaves under trees |
| `stream1`–`stream4` (`strm1–4.kq`) | emitter | `permwrld.cs` | four drip jets at the same point (208643, 246565, 9724), one per direction |
| `Cottage2/3/4` | particle catcher | `permwrld.cs` | catch `leaf1`, `leaf2`, `leaf3` in the building `Cottage` (`house1.dig`) |
| `AnimatedRiver1`–`9` (`wateran1–9.kq`), `uglywat2`, `FullStream` (`fullstrm.kq`) | ground animator | `newWorld.cs` | river, stream and the "ugly water" squares |

`firebits.kq` and `icebits.kq` are emitters too, but they are attached to
fire and ice arrows by the projectile files (`emitter=firebits.kq`); that
belongs to the combat study. No script in any world uses the `KQEmitter::…`
or ground-animator commands (data), so they are only listed at the end.

## Emitters (traced)

### Emitter keys

All in the emitter's own `.kq` section. Vectors are written as three keys
`<name>X`, `<name>Y`, `<name>Z`; a missing key is 0.

| Key | Meaning | Default |
| --- | --- | --- |
| `shape` | the particle model (`.kqs`) | none |
| `loc` | the emitter's position (world units) | 0 0 0 |
| `radius` | particles start anywhere in a **box** of ± this around `loc` | 0 0 0 |
| `forceMin`, `forceMax` | each particle's velocity, per axis, units **per tick** | 0 |
| `gravity` | added acceleration, units per tick² (rule 7) | 0 |
| `rotMin`, `rotMax` | each particle's spin, radians **per tick**, about x, y, z | 0 |
| `maxSinPeriod` | swaying, per axis (rule 6) | 0 |
| `maxLife` | particle life in ticks | **100** |
| `maxParticles` | number of particles | **200** |
| `delayStart` | non-zero: particles appear one by one (rule 4); only "zero or not" counts | 0 |
| `follow`, `followMesh`, `followOffset` | attach the emitter to an object (rule 11) | none |
| `translucency` | see-through like other objects (`objects.md` Q14) | 256 (solid) |
| `randomZRot` | **no effect** (see "Do not reproduce") | 1 |
| `dying`, `deadCount` | saved-game state (rule 10) | 0 |

A **tick** is one update of the game's fixed 15-per-second clock (the same
clock as Connor's updates). The data's own comment in `firebits.kq` says
so: "How many cycles (1/15th of sec) does a particle live".

### Emitter rules

1. **Start.** When the emitter enters the world it makes `maxParticles`
   particles. For each one, chosen at random (each number uniform between
   its limits, independently):
   - its **start offset**: x in [−radiusX, radiusX], y in [−radiusY,
     radiusY], z in [−radiusZ, radiusZ];
   - its **velocity**: x in [forceMinX, forceMaxX], y and z likewise
     (world axes);
   - its **spin** (if any `rotMin`/`rotMax` value is non-zero): x in
     [rotMinX, rotMaxX], y and z likewise;
   - its **sway speed** per axis (rule 6);
   - its **starting age**: a whole number from 0 to `maxLife` (fraction ×
     `maxLife`, rounded down).
   It starts unturned at `loc` + offset, and is then moved on by
   age × velocity (so the particles are already spread along their paths
   when the world appears; no gravity or swaying in this jump).
2. **These choices are made once.** Velocity, offset, spin and sway speed
   stay the same for the particle's whole existence, through all its
   restarts. So **each particle repeats exactly the same path every
   life**; only the mix of ages makes the stream look random.
3. **Every tick, for each particle:**
   1. age + 1;
   2. sway angles + sway speeds (rule 6);
   3. position + velocity (or the swaying version, rule 6);
   4. if it spins: it turns further by its spin (the same small turn every
      tick, so it keeps turning; the angles use the same convention as an
      object's `dirX/Y/Z`, assumed, not checked);
   5. if gravity is set: position + **age × gravity** (rule 7);
   6. if age > `maxLife`: it **restarts**: age 0, position = `loc` + its
      own start offset. Its turn and sway angle are **not** reset.
4. **Shown or not.** A particle is drawn only if it is "visible" and its
   age is not 0. Particles start visible, **except** when `delayStart` is
   non-zero: then all start hidden and each becomes visible at its first
   restart, so they come out one after another over the first life. A
   restart always makes a particle visible again (also after a catcher
   hid it). So a particle is on screen for `maxLife` ticks and off for 1
   tick per cycle of `maxLife` + 1 ticks.
5. **Each emitter is updated by itself**, particles in list order. The
   update has no collision test: particles fly through the ground and
   walls (only a catcher hides them).
6. **Swaying (`maxSinPeriod`).** For an axis with a non-zero value *P*,
   each particle gets a sway speed = random fraction × *P* × 6.28 ÷
   `maxLife` radians per tick (6.28 as stored in the game, not exactly
   2π). Its sway angle for that axis starts at 0 and grows by that speed
   every tick. While the angle is not 0, the step on that axis is
   **velocity × sin(angle)** instead of velocity. So the particle drifts
   back and forth along that axis, up to *P* swings per life. Axes with
   *P* = 0 move normally.
7. **Gravity.** Each tick the position also moves by age × gravity (age
   after step 1). After *n* ticks of a life that adds up to gravity ×
   *n*(*n*+1)/2: an evenly growing fall (or rise, for positive values like
   `firebits`' `gravityZ=3`).
8. **Units.** Positions in world units (Z up). `stream1`: velocity
   (−3…3, 20, 50…60) per tick and gravity −5 → the drip rises about 50
   per tick at first and falls back down; life 40 ticks (2.7 s).
   `leaf1`: velocity (0…200, 10…20, −30), swaying on x with up to 3
   swings per life of 150 ticks (10 s); 6 leaves.
9. **Drawing.** Each particle is the `shape` model at the particle's
   position and turn, coloured like any other model; the emitter's
   `translucency` applies (read for emitters like every object; that it is
   applied to each particle is likely, not traced). Which frame of the
   shape is shown: the particle models are single still models (data);
   draw frame 0.
10. **Dying (not used in Daventry).** An emitter can be told to die (the
    `dying` state, used by things like arrows that end). From then on a
    particle whose life ends is not restarted but removed; when all are
    gone, the emitter removes itself. If it is already dying when it
    enters the world (a saved game), it is removed at once.
11. **Following (not used in Daventry).** `follow=<object name>` (or the
    word `KQCamera` for the camera) puts the emitter at that object's
    position plus `followOffset` and moves it along when the object moves.
    The offset is in the object's own axes, except when its x and y are
    both 0, then it is added in world axes. With `follow`, the velocity
    limits are turned with the object, so particles fly out in the
    object's own directions. With `followMesh` also set, each restarting
    particle takes the object's current turn and gets a new random
    velocity (the arrow trails).
12. **Active.** An emitter receives the ticks only while the game counts
    it as active; what switches that (probably closeness to the player or
    camera, as for other objects) was not traced. Daventry's emitters set
    nothing about it. Recommendation: update them always.

## Particle catchers (traced)

1. Keys: `emitter=<emitter name>`, `interior=<building name>`.
2. The names are looked up once, right after loading (when the object's
   start-up event is handled, after the loading script has finished), so
   the emitter may be loaded after the catcher, as in `permwrld.cs`. If
   the emitter isn't there, the catcher does nothing.
3. From the building it takes its **bounding box** (the box the building
   keeps for collision; computed from the building model, `buildings.md`)
   and its placement. "Top" = the box's top z + the building's z.
4. **Every tick:** for each particle of its emitter whose z is at or below
   the top, the particle's position is turned into the building's own
   axes; if it lies inside the box (min ≤ value < max on all three axes)
   the particle is made **invisible**. It keeps moving and becomes
   visible again at its next restart (emitter rule 4).
5. So under a roof the leaves vanish where they would enter the house.
   Elsewhere nothing stops them: they can pass below the ground before
   their life ends (whether that is visible in Daventry was not checked).

## Ground animators (traced; square check against the data)

### Ground animator keys

| Key | Meaning |
| --- | --- |
| `numOrigTextures`, `origTexture0…` | which ground squares to animate: every square whose material number (`terrm.dib` value, `terrain.md`) is one of these |
| `numExclude`, `excludeX0`/`excludeY0`… | squares (column, row) to leave out |
| `numGridBlocks`, `gridBlockX0`/`gridBlockY0`… | instead of the two above: an explicit list of squares. **If `numGridBlocks` is not 0, `origTexture` and `exclude` keys are not read at all.** Only the first `numGridBlocks` entries are read. |
| `numMaterialIndex`, `materialIndex0…` | the cycle: material numbers shown one after another |
| `delay` | seconds between changes |
| `repeatflag` | 1 = loop the cycle; 0 = play it once and keep the last one |
| `startNow` | 1 = start when loaded |
| `frameCount` | saved-game state: where in the cycle it is (0 = start) |
| `numMaterialIndexII`, `materialIndexII0…` | a second cycle, used only by the unused command `terAnimSwitch` |

Square coordinates are the same as in `terrain.md`: column = x index,
row = y index (bottom row of the image = row 0). Checked: all excluded
and listed squares of Daventry's animators have material 15 in
`terrm.dib`, the material of the stream column at x = 45.

### Ground animator rules

1. **Choosing the squares** (once, when it enters the world): with
   `numGridBlocks`, the listed squares. Otherwise every square of the
   world (any order) that is not in the exclude list and whose **current**
   material is one of the `origTexture`s.
2. **Each change** sets **all** chosen squares to the same material: the
   next number of the cycle. The squares' own variety is lost while
   animating (e.g. `AnimatedRiver1` turns both its 56 and 57 squares into
   57, 89, 90, 89, …).
3. **Timing.** When started, the first change happens `delay` seconds
   later and shows the **first** cycle entry, then one change every
   `delay` seconds: entry 1, 2, …, last, then (with `repeatflag=1`) entry 0
   again. Before the first change the squares show their original
   materials. Daventry: `delay=0.3` everywhere, so a 4-step cycle repeats
   every 1.2 s.
4. With `repeatflag=0` the animator stops after the last entry and the
   squares keep it. (None in Daventry.)
5. A non-zero `frameCount` (saved game) applies that entry at once when
   loaded, before the timer starts.
6. The ground's texture comes from the square's material number, so the
   drawn texture changes (with the material's orientation code from the
   direction table, `terrain.md`). Whether the **envInfo flags** (water,
   footsteps, `water.md`) follow the changed number was not traced; in
   Daventry every cycle uses materials with the same flags as the squares
   it animates (data: e.g. 56, 57, 89, 90 all only `0x10`; 111–116 all
   `0x210`), so it makes no difference there.

### Daventry's animators (data)

| Animator | Squares chosen | Cycle (material numbers) |
| --- | --- | --- |
| `AnimatedRiver1` | 86 (materials 56, 57) | 57, 89, 90, 89 |
| `AnimatedRiver2` | 0 (materials 6, 7 not on the map) | 6, 87, 88, 87 |
| `AnimatedRiver3` | 62 (13–16) − 5 excluded = 57 | 13, 81, 82, 81 |
| `AnimatedRiver4` | 19 (17–20) | 17, 83, 84, 83 |
| `AnimatedRiver5` | 0 (21–24) | 21, 85, 86, 85 |
| `AnimatedRiver6` | 17 (91–94) | 91, 95, 96, 95 |
| `AnimatedRiver7` | 32 (182–185) | 182, 97, 98, 97 |
| `AnimatedRiver8` | 2 (99–102) | 99, 103, 104, 103 |
| `AnimatedRiver9` | 13 (105–108) | 105, 109, 110, 109 |
| `uglywat2` | 4 (111–114) | 111, 115, 116, 115 |
| `FullStream` | 4 listed: (45, 53)–(45, 56) | 15, 81, 82, 81, 82 |

(The counts are all squares with those materials; only
`AnimatedRiver3` has excluded squares among them, so it animates 57.)

## Numbers and units

| Thing | Value | Source |
| --- | --- | --- |
| tick | 1/15 s | traced (game clock) |
| default `maxParticles` / `maxLife` | 200 / 100 ticks | traced |
| sway factor | 6.28 per `maxLife` | traced |
| "non-zero" for rot, sway, gravity | absolute value ≥ 0.000001 | traced |
| ground animator `delay` | 0.3 s in all Daventry files | data |

## Data used

- Emitter, catcher and animator `.kq` files; `permwrld.cs`, `newWorld.cs`.
- Particle models `leafpart.kqs`, `leaf2.kqs`, `drip.kqs` (Daventry
  resources).
- `terrm.dib` (square materials), `envinfo.kq` (flags), the terrain
  material list and direction table (`terrain.md`).
- The `Cottage` building (`cottage.kq`, `house1.dig`) for the catcher box.

## Edge cases

- Square (45, 57): excluded from `AnimatedRiver3` (`excludeX4/Y4`) and
  listed as `FullStream`'s fifth square, but `FullStream` says
  `numGridBlocks=4`, so the fifth entry is never read: that square **never
  animates**. Keep it (it is the original's data).
- `AnimatedRiver2` and `5` find no squares and do nothing.
- `leaf2`: `radiusZ=100`, so its leaves start up to 100 above or below
  `loc`.
- A `0` in `forceMin`/`forceMax` for an axis with `maxSinPeriod` keeps
  that axis still (sin × 0).
- An emitter with `maxParticles=0` does nothing.

## Do not reproduce

- `randomZRot` (default on) only draws a random number and throws it away;
  it has no visible effect. Leave it out (it only shifts the random
  sequence, which we don't copy anyway).
- The random numbers come from the C library's generator; no need to
  match it. Uniform random fractions are enough.

## How to check

- **In our game (log):** for `stream1` and `leaf1`, for particle 0: its
  offset, velocity, spin, sway speed, starting age, then per tick its age,
  position and visibility. Expected: the same path every life, a restart
  after `maxLife` + 1 ticks, one hidden tick per cycle, `stream1`'s drop
  peaking and falling back within 40 ticks.
- **In our game (log):** for each ground animator, the number of squares
  chosen (expect the table above) and each change with its time and
  material (one change per 0.3 s).
- **In the original:** watch the leaves near the cottage (`leaf1–3`):
  they should vanish when they reach the roof box, not pass into the
  room. Watch one leaf: it should come back along the same path each time.
- **In the original:** look at square (45, 57) at the stream: it should
  stay still while the four squares above it flow.

## Open questions

1. What makes an emitter "active" (emitter rule 12).
2. Whether particles get the emitter's `translucency` (rule 9).
3. Whether changed ground materials change the envInfo flags of the
   square (no effect in Daventry; matters for other worlds, e.g. Swamp's
   26 animators).
4. Script commands, unused by the data: `KQEmitter::setLoc`, `setRadius`,
   `setForceMin`, `setForceMax`, `setGravity`, `setRotMin`, `setRotMax`,
   `setMaxSinPeriod`, `setMaxParticles`, `setMaxLife`, `setFollowObject`,
   `setFollowOffset`, `reinit`, and `terAnimSwitch <animator> [n]`
   (swaps the two cycles). Not studied further.
