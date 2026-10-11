# Placing objects from `.kq` files

Study session A, 2026-10-11. Answers `QUESTIONS.md` Q9 and Q10. Sources:
`Mask.exe` (studied, described in our own words) and the data files.

## Orientation (Q9)

1. An object's placement is read as: first a full matrix if a key
   `<prefix>RMat` exists (flags followed by nine numbers); otherwise the
   three angles `dirX`, `dirY`, `dirZ`; then the position `locX`, `locY`,
   `locZ`. No `.kq` file in `daventry/` or `GAME/` uses `RMat`, so in
   practice it is always the three angles. (Traced.)
2. Angles are **radians** (they feed the same angle-to-rotation function
   used everywhere; data values like 1.57 and 3.14 agree).
3. Positive angles follow the right-hand rule: looking from the positive end
   of the axis back towards the origin, a positive angle turns
   anticlockwise. So positive `dirZ` turns +X towards +Y (anticlockwise seen
   from above), positive `dirX` turns +Y towards +Z, positive `dirY` turns
   +Z towards +X. (Traced.)
4. Order, with row vectors (`point × matrix`, see `models-and-textures.md`):
   the rotation matrix is `RotY(dirY) × RotX(dirX) × RotZ(dirZ)`, i.e. a
   point is turned about **Y first, then X, then Z**. (Traced.)
   With column vectors this is `RotZ · RotX · RotY`.
5. **No scale or size field**: the placement transform has no scale, and no
   object file uses a scale key.
6. `rotMinX/Y/Z` / `rotMaxX/Y/Z` belong to particle emitters (`KQEmitter`,
   which also has a `KQEmitter::setRotMin` script command). They are not
   object orientation; ignore them for placement.

## Height (Q10)

1. **`isGroundRelative` is never read by the game**: the name does not occur
   anywhere in `Mask.exe` (case-insensitive search). Treat it as a leftover
   from the level editor. `locZ` is used as written.
2. Measured in Daventry (`KQInterior` sections with that flag): either
   `locZ` is −1/−2, or it is an absolute height 10–35 units above the ground
   under the object (barn, docks, pub; `look` is 1,607 above).
   **Answered in `buildings.md`**: `keepOnGround` is on by default and
   raises anything below the ground up onto it, so use
   `max(locZ, ground height at locX, locY)`.
3. **`keepOnGround`** is read together with `useGravity`, `submerged` and
   `MaxSlopeDot`. Default on. Rule (see `buildings.md`): if the object is
   below the ground it is raised to the ground, never lowered. An object
   whose `.kq` says `keepOnGround=0` keeps its `locZ`.
4. Related, for M2: `MaxSlopeDot` defaults to 0.819 = cos 35°: ground whose
   normal is more than about 35° from straight up is too steep to walk on.
   (Default traced; how it is used not traced yet.)

## Animated objects at load (Q12)

All traced in `Mask.exe` unless marked otherwise.

1. **Movement list.** `[<list>]` gives `nMovement` and names `Movement0..`;
   for movement *k* the game then reads the section `[movement<k>]` **by
   number** (answer to c). The `MovementK=` value is the movement's name,
   which `currAnimName` refers to.
2. **`loop=<model> <sequence> <a> <b> <c> <d>`** is read as two words, two
   decimal numbers and two whole numbers:
   - `a` = **start position**, a fraction of the sequence from 0 to 1.
     Frame shown = `round(a × frameCount)`. For a sequence that does not
     repeat, anything at or past the end shows the last frame (so 1.0 =
     last frame).
   - `b` = **playing speed**: 1 = forward at normal speed, −1 = backwards,
     0 = stopped (holds the start frame).
   - `c` = **repeat** (1 = loops, 0 = plays once and stops at the end). A
     repeating sequence runs through all `frameCount` frames per cycle; a
     non-repeating one goes from the first to the last frame
     (`frameCount − 1` steps).
   - `d` = an on/off switch kept with the running animation. **Its effect
     is not traced.** It is 1 in most lines, 0 in 30 Daventry lines.
   - So the doors: `closed` = first frame, stopped; `opened` = last frame,
     stopped; `open` = play forward once; `close` = play backward once from
     93%.
3. **Which movement shows first (answer to b):** the one named by
   `currAnimName`. If that key is missing, empty or names no movement in the
   list, the game uses **movement 0** (the first in the list), possibly
   after printing an error message.
4. **Also read from the object's section:** `animPos` (a position 0–1; if
   it is past the start position from the `loop=` line, the animation jumps
   there; not used in Daventry) and `flags`. If `flags` has the bit 8 set,
   the animation starts at a **random frame** and plays at a random speed
   between 0.9 and 1.1 (so rows of swaying things don't move in step).
   `nextAnimName` is read too (what to play after this one); not traced
   further.
5. **Key names are matched ignoring capitals but otherwise exactly**
   (`_stricmp`). So `milltree.kq`'s `curAnimName` (one r) is **not read**
   (answer to d): that object starts in movement 0.

### Follow-ups (Q13)

1. **(b) The sequence name is looked up.** When a movement starts, the
   game makes a new animation player, which begins on the model's
   **sequence 0** (the first in the file). It then searches the model for
   the `loop=` line's sequence name and switches to it if found. If the
   name is not in the model, nothing changes and sequence 0 plays.
   (Traced.) Your "use the first loop when the name is not found" matches.
2. **(a) Starting past frame 0** (`Pyramid`, start 0.9): the start position
   is set by jumping the player there, not by playing up to it. **Not
   traced** whether the object steps of the skipped frames are added on the
   jump. Likely not (it is a jump, the same call that `animPos` uses);
   draw it without them, as you do now.

## See-through objects (Q14)

1. **`translucency=<n>`** is read for every object (with `hazeMultiplier`,
   `hazeSolid` and `hidden`). Default 256 = solid. (Traced.)
2. When drawing an object with `n` below 256, **every material of every
   part** gets opacity `n / 255`. (Traced; the scale constant is 1/255.)
   So 200 ≈ 78%, 150 ≈ 59%. Far away it fades further: once the object's
   haze is past 0.5, opacity is multiplied by `1.2 − haze` (haze grows
   with distance, 0 near to 1 at the haze end). Translucent objects get
   this fade instead of being tinted by the fog.
3. Material flag `0x10000` is **not** transparency (corrected in
   `models-and-textures.md`: it halves the fog on that material).
4. **(b) Teleport pad: open.** `telepad1.kq` has no `translucency`, its
   materials are plain `0x403`, and `telepad.cs` only has Connor's lines.
   I did not find another source of see-through drawing. Draw it solid for
   now. Not checked: whether the 8-bit palettes' extra tables (`.ppl`)
   make some colours see-through.

## Where a world's objects come from (Q16)

All traced in `Mask.exe` unless marked "scripts" (read in the world's
`.cs` files).

1. **Only what the scripts load is in the world.** No code loads all
   `.kq` files of a world. (Scripts:) a new game runs the world's
   `world.cs` in three phases (`Init`, `Load`, `Begin`). `Load` runs
   `newWorld.cs`, which loads, in file order: `WorldVar.cs`,
   `terrain/terrain.kq`, Connor, the camera, …, `WorldInv.cs`,
   `permwrld.cs $KQGame::runOptimal`, then its remaining `loadKQ` lines.
   Each `.cs` line runs in order, and a script named on a line runs fully
   at that point. Of Daventry's 428 `.kq` files, 75 are never named by a
   `loadKQ` line at world start (orbs, chickens, goblins, `fenc2`, …).
   Leave them out. Monsters come from `monsters.kq` lists, a separate
   topic. Some objects are loaded later by event scripts (e.g. `ax.cs`).
2. **`loadKQ <file> [<name> [x y z [dirX dirY dirZ]]]`:**
   - `<name>`: `same` = keep the section name from the `.kq` file; `@` =
     no name; anything else = the object is renamed to it (so
     `chessset.01` is a second chess set). Missing = keep the section name.
   - With **at least x y z**, the given position **replaces** the `.kq`'s
     `locX/Y/Z`, and the rotation becomes the three angles given, or
     **0 0 0** if they are missing. The `.kq`'s own `dirX/Y/Z` are not used
     then.
   - The three angles are `dirX dirY dirZ`, same unit (radians) and same
     rule as in `.kq` files (it is the same angle-to-rotation function).
   - Your `13981 0 0` is a misread: in those lines 13981 is z (e.g.
     `stool.kq stool.02 260045 225746 13981 0 0 -0.81`).
   - With only a file name, the `.kq`'s own position and angles are used.
3. **(b)** Corrected by Q17 below: `loadKQ` has its own −1/−2 rule.
   The `keepOnGround` raise also applies, as for any object.
4. Connor and the camera have special cases (they can take a saved
   position); ignore that for now.
5. Saved games (for later): the game writes its own script that runs
   `WorldVar.cs`, then `permWrld.cs`, then one `loadKQ` per object,
   `loadKQ WorldState.kq`, and finally `newWorld.cs`.

### Ground heights −1 and −2 (Q17)

1. **In a `loadKQ` line, a z of exactly −1 or exactly −2 means "on the
   ground".** Before placing the object, the game replaces that z with the
   ground height at (x, y). This happens **whatever `keepOnGround` says**.
   (Traced.) The same rule is used by other commands that take a position,
   e.g. `move Connor 0 x y -1 …` in the scripts.
2. −1 and −2 pass a different on/off value to the height lookup, but the
   ground (`KQTerrain`) ignores it, so both give the same ground height.
   (Traced as far as the ground's lookup; if some other floor answers the
   lookup, the difference may matter. Not seen.)
3. Other values are used as written; then `keepOnGround` (default on) only
   raises objects that are below the ground.
4. A `.kq` file's own `locZ` does **not** go through rule 1. There a −1 is
   handled by the `keepOnGround` raise (e.g. the buildings, which are
   loaded by `loadKQ <file>` with no position).
5. **`Cottage2/3/4` and `wateran1–9` have no model and need no position.**
   (From the data.) `Cottage2–4` are `KQParticleCatcher`s tied to the
   cottage building (`interior=Cottage`, `emitter=leaf1`): probably where
   falling leaves collect. `wateran1–9` are `KQTerrainAnimator`s: they
   cycle ground-texture numbers (`materialIndexN`, every `delay` = 0.3 s,
   repeating) on the river squares, apart from the `exclude` squares. That
   is the flowing river. Not traced; skip them for M1.

### Missing `locX` / `locY` / `locZ` (Q19)

1. Each of `locX`, `locY`, `locZ` (and `dirX/Y/Z`) is read on its own. A
   **missing key leaves that value unchanged**. (Traced: the number reader
   only writes when the key is found.)
2. The value it keeps is the mover's starting position, which is
   **(0, 0, 0)**. (Traced in the mover's constructor.) So a missing `locZ`
   is 0, and a missing `locX`/`locY` is 0 too. Missing angles are 0.
3. Then the usual `keepOnGround` raise applies (default on): z = 0 is
   below the ground, so the object is lifted to the ground height at its
   (x, y). Use `max(0, ground height)`, the same as for −1.
4. Barren's `lookout` and `lookout2` both stand on a flat hilltop. The
   ground under both origins is 11,592 (our height lookup), so both sit at
   z = 11,592. Their models reach below that (lowest points −1,803 and
   −2,890 in model space), so foundations go into the hill. The user's
   point at about 12,137 is 545 above the base, which fits a deck or floor
   in the model (not checked).

## Hidden objects (Q18)

1. `hidden=<n>` is read as a whole number; any value other than 0 means
   hidden. Scripts switch it with `KQObject::setHidden <name> 0|1` (about
   280 such lines in the game, e.g. `bricks.cs`, `miltrap1.cs`, and
   `newworld.cs` hides a newly made inventory item). (Traced.)
2. What it does (traced): every object belongs to "groups", given as bit
   masks: `groupsAlways` (default 6), `groupsActive3D` (default 0x448,
   added when Connor is near) and `groupsActive3DFar` (default 0x440,
   added when far). A hidden object is treated as **not active**: it
   keeps only its `groupsAlways` bits.
3. Which bit means what is **inferred, not traced**: 0x40 is in both the
   near and the far defaults, so it is very likely "drawn"; 0x8 is
   near-only, likely "can be touched/collided with"; 0x400 is tied to
   running animations. With the default `groupsAlways` = 6, **a hidden
   object is not drawn and cannot be touched** until a script calls
   `setHidden <name> 0`.
4. So: don't draw `wizchest1` at the start. An object whose
   `groupsAlways` includes 0x40 (e.g. `groupsAlways=1097` = 0x449, one
   object in the game) would still be drawn when hidden.
