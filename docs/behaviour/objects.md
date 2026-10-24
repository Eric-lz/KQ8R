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
     Used exactly as written. Frame shown = **floor(a × frameCount +
     0.001)** (cut down, not rounded; corrected 2026-10-22, Q153: the
     same rule as during play, rule 6). For a sequence that does not
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
     (Later traced for Connor: it is "can be cut short", see
     `connor-movement.md` "The two flags on each `loop=` line".)
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
   The random speed replaces the `loop=` speed. This only happens when
   the **current** movement is started; an object with no `currAnimName`
   whose waiting movement starts first never gets it (`anim-scripts.md`
   rule 3, Q85).
   `nextAnimName` is read too: it is the **waiting movement** (traced
   2026-10-17, `anim-scripts.md` rules 2–3).
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

### Loops that move the whole object (Q51, 2026-10-13)

1. **The steps are added up.** Every update, after an animated object's
   movement advances, the frame steps of that advance are put onto the
   object's **current** placement and the object is moved there (through
   its mover, so its collision box goes with it). This is the same path
   for every animated object, Connor included: it is how his walk loop
   carries him forward. Nothing recomputes the placement from the phase.
   (Traced: the composition onto the current placement and the move.
   Which frames' steps one update includes, wrap included: rule 6a.)
2. So Gnome's rolling rocks (`rrock1–3`, loop `theroll`, 54 frames,
   3.53 s, repeating) **really travel** along the path their steps
   describe (up to 9,691 away) and would end each cycle about 57 units
   from where it began, drifting further every cycle.
3. **The data puts them back.** Each rock's moving movement has a cue at
   phase 1.0: `cue1=1.0 sendEvent @ KQMoveEvent 0 57500 111710 18265 0 0
   0 0` (`rrock1.kq`; the others with their own start). `@` is the
   object itself; `KQMoveEvent` with first number 0 sets its position
   and turn **absolutely** (x y z, then three angles). So at every wrap
   the rock jumps back exactly to its start, and the 57-unit leftover
   never builds up. (A commented-out line next to it does the same with a
   relative move by the leftover, `KQMoveEvent 3 0.109 -56.617 1.670 …`:
   the designers saw the drift.) The first number's meaning beyond 0 is
   read from the same event's handler on the camera: flag value 1 = the
   position is added to the current one, 2 = the turn is added, 3 = both;
   the last number non-zero = don't refresh the object at once. That is
   the camera's handler; for objects only the 0 case (absolute) is used
   by this data. Treat the other values as "not checked" for objects.
4. **A cue at exactly 1.0 fires on a repeating loop.** At a wrap the cue
   range checked is from the old phase up to just above 1 (1.0001), then
   from 0 to the new phase. So `cue1=1.0` fires once per cycle, when the
   loop wraps. (For a play-once movement that stops at exactly 1.0, the
   range ends at 1.0 and excludes it, `doors.md` rule 9.)
   **Played backwards** (speed below 0, repeating; traced 2026-10-17,
   Q81 c): when the phase wraps from near 0 back to near 1, the ranges
   are from 0 up to the old phase, then from the new phase up to 1.0001.
   Each range includes its lower end and excludes its upper end, as
   always. So going backwards a cue at exactly 0 fires at the wrap, and
   a cue at 1.0 fires too (it lies in the second range). Without a wrap,
   backwards or forwards, the range is from the smaller phase to the
   larger one.
   **A cue at exactly 0.0 on a forward repeating loop** (traced
   2026-10-22, Q158 c): a forward wrap sets the phase to exactly 0
   (rule 6), so that update's second range is from 0 to 0 and holds
   nothing. The cue at 0.0 fires in the **next** update (its range starts
   at 0): once per pass, one update (1/15 s) after the wrap. A movement
   that starts at phase 0 fires its 0.0 cue in its first update. A
   movement with speed 0 never fires a cue at 0.0 (its range is empty).
   **One difference (forward wraps only):** if a transition is waiting
   for a phase that lies inside either part of the wrap, **no cues fire
   in that update**; the transition is taken instead. A backward wrap
   has no such check.
5. So for B: apply the steps by adding them up (as for Connor), run the
   cue's `sendEvent @ KQMoveEvent …` (moving the object and its box), and
   the rocks roll out and come back each cycle. The **collision box
   follows the object** (it is the object's box at its current
   placement); Q33 a's "boxes are fixed" means the box's size and shape
   never change, not its place.
6. **The frame shown for a phase p** (traced 2026-10-22, Q153; replaces
   the earlier "keep B's rule", which was wrong): for a sequence of N
   frames, **frame = floor(p × N + 0.001)**: cut down, with a tiny
   allowance of a thousandth of a frame, never rounded to the nearest.
   The same for repeating and play-once loops, forwards or backwards,
   at the start and during play. No blending between frames: the frame
   drawn is the loop's first sequence + that whole number.
   - **At or past the end:** repeating: when the frame would reach N it
     becomes 0 and the phase is set to **exactly 0**; what went past 1
     is thrown away, not carried into the next cycle. Play-once: frame
     N − 1, phase held at 1.
   - **Below 0** (backwards): repeating: last frame, phase (N − 1)/N;
     play-once: frame 0, phase 0.
   - **Speed:** play-once moves 1/duration per second; repeating moves
     (N − 1)/(N × duration) per second, so one cycle takes
     N × duration/(N − 1). Duration 0: the phase never moves.
   - **Example, the King's portrait** (Castle, `kingart.cs` `stop2` =
     `kingart .99 0.0 1 1` on the 46-frame `kingart.kqs`): floor(45.54 +
     0.001) = **frame 45, the last (pushed) frame**, held there (speed
     0). Matches the user's check.
   - `checkedRoundingBug=1` in `.anm` files is never read by the game.
   - This frame player is used for **every `.kqs` animation**: plain
     animated objects, doors, chests, characters and Connor alike
     (traced 2026-10-24, Q164). Only shapes of another kind (`.dts`)
     use a different player, not studied.
6a. **Which frames' steps one update adds** (traced 2026-10-24, Q164).
   Every frame of a `.kqs` sequence carries a step (its frame
   transform). An update that moves the shown frame from a to b adds the
   steps of **every frame it enters**, in order:
   - **Forwards:** frames a + 1, a + 2, … up to b. Several frames in one
     update all count. Across a wrap: a + 1 … N − 1, then 0 … b.
   - **Entering frame 0 forwards** adds the step of the sequence's
     **last frame (N − 1)** instead of frame 0's own step, but only when
     the loop's first frame is the **first frame of the whole file**
     (the loop starts at frame 0 of the `.kqs`; true for every walk and
     run model checked: Connor's and all Daventry characters'). In a
     file with several loops, a later loop's frame 0 adds its own step.
     Frame 0's own step is zero in these files, so without this the
     wrap would add nothing.
   - **Backwards** (speed below 0): the steps of frames a − 1 down to b
     are **taken off** (each step undone); across the wrap: a − 1 … 0,
     then N − 1 … b. There is **no** last-frame swap backwards, so frame
     0's own (zero) step is used.
   - The phase is **not** recomputed from distance: what goes past the
     end at a forward wrap is dropped (rule 6), so an update that wraps
     enters frame 0 and adds one step, however far past the end it went.
   - **So one forward cycle of a repeating walk or run** covers the sum
     of frames 1 … N − 1's steps **plus the last frame's step once
     more**, and takes N × duration/(N − 1) seconds (rule 6). With the
     game's 1/15 s updates and Connor's and Daventry's models (a frame
     time of exactly one or half an update), no time is lost at the
     wrap. **Speed = (sum of steps + last step) × (N − 1)/(N ×
     duration)**, times the `loop=` speed. This is within 1–2 % of the
     older "sum ÷ duration" for walks, but not the same: Connor's run
     2,558 (not 2,518), walk 1,124 (not 1,127); **back** (`c_walk` at
     −1) 1,070 (sum × (N − 1)/(N × duration), no extra step). Table
     per character: `character-movement.md` rule 2.
   - The user's measurement in the original (`connor-movement.md`
     "Measured in the running original") fits this: run 2,492–2,561 (top
     of the range = 2,558; lower values from updates longer than 1/15 s,
     whose overshoot past a wrap is dropped), walk 1,083–1,131 (1,124),
     back 1,079–1,085 (1,070, about 1 % under) [inferred match].
   - Rule 2's rolling rocks: their 57-unit leftover per cycle was the
     plain sum; with this rule it differs, but the cue at 1.0 puts them
     back absolutely anyway, so nothing changes in play.
7. **User's check (2026-10-13):** our viewer already shows the three
   rolling rocks correctly. B's way (steps chained from the phase, jump
   back at the wrap) and the game's way (steps added up, reset to the
   start by the cue at 1.0) give the **same picture**; B's can stay. Only
   switch if something else ever moves the rock mid-roll.
8. **Not the same thing: Gnome's `boulder`** (class `KQBoulder`, the
   stone Connor pushes in the big room). Its only movement `stop` is a
   repeating rolling loop (`pushston`, speed 1) by the data, and our
   viewer plays it, so it rolls on the spot. The original shows it still.
   So `KQBoulder` must be controlling its own animation (likely playing
   it only while pushed). Not studied; until pushable objects are
   studied, **don't play a `KQBoulder`'s movement** (show frame 0).

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
4. **(b) Teleport pad: drawn solid** (traced 2026-10-22). In the Glide
   model drawing a part can be see-through or cut out in only four ways:
   the object's translucency (rule 1, `makeTranslucent`); material flag
   `0x1000` (black texels not drawn); material flag `0x2000` (alpha per
   palette index from the world palette's `tpal` chunk, only on `0xF000`
   materials, which draw nothing); and the per-object flag set by
   `dither` ("Dithered objects" below). The pad uses none of them:
   `telepad1.kq` has no `translucency`, both materials of `telepad.kqs`
   (`telepad.BMP`, `Teleport.BMP`) are plain `0x403`, the scripts
   (`telepad.cs`, `teletrap.cs`'s `onTeleport` / `offTeleport`) do
   nothing visual. **Draw it solid**, with the world palette's colours.
   What the user remembers may be `Teleport.BMP`'s ring with a **pure
   black centre** (it looks like a hole or a portal) and the pad's
   swirling 16-frame movement (1 s, repeating); also `tooFar=0.4` with
   `groupsActive3DFar=0` makes the pad vanish beyond 0.4 ×
   `visibleDistance` from Connor (it pops out, it doesn't fade)
   [inferred]. **Confirmed by the user's screenshot of the original
   (2026-10-22):** the pad is solid; the stone ring with signs and the
   red-orange swirl with a black centre, nothing behind shows through.

### Dithered objects (traced 2026-10-22, Q144)

1. **`KQObject::dither <name> <mode>`**: the game's own usage text gives
   `0` none, `1` dithered, `2` "justTrans"; `3` is also stored but
   changes no materials (the Invisible potion puts Connor in mode 3; his
   look then is drawn by other code). A missing or extra word writes the
   usage line; an unknown name writes "KQObject::dither cannot find
   '<name>'".
2. It changes **every material of the object's model** whose kind is not
   "nothing" (`models-and-textures.md` "Material flags").
3. **`1`, dithered:** the material's colour key is switched on (flag
   0x1000: pure-black texels are not drawn, lighting forced to none, so
   full brightness), and **the texture itself is rewritten** so that
   about **3 of every 4 texels** become colour 0 (black, so see-through):
   each texture row keeps every 4th texel and the next row's kept texels
   are shifted by 2 (when the width is a multiple of 4; the rewrite also
   blanks 2 extra texels per row). A screen-door pattern about 25 %
   solid, laid on the **texture** (it scales with the texture on screen,
   not with screen pixels).
4. **`2`, justTrans:** colour key only, texture untouched: only texels
   already pure black become see-through.
5. **`0`:** the colour key is removed from all those materials and the
   texture is loaded again from its file. Side effect: materials whose
   file already had the colour key (0x1403) lose it too; harmless with
   the shipped data (those textures have no black).
6. The rewrite is made in the loaded texture itself, so other objects
   using the same texture very likely show the holes too until `dither
   0` reloads it [inferred].
7. **Clicks and collisions don't change**: only the drawing does.
8. **`dithered=<mode>` in an object file** is read into the same stored
   value, for **every class** (corrected 2026-10-24, Q165, traced):
   - **Every object applies it when it is added to the world** (at load
     or by `loadKQ`), if it is not 0: plain objects (`KQObject`), doors,
     chests, animated objects, `KQAnimOnce`, characters (`KQMonster`).
   - **Objects that play movements apply it again every time a movement
     starts** (before each new movement the old model is un-dithered,
     unless the mode is 3, then the new model gets the mode). This is the
     shared movement start of all animated objects, so it covers chests,
     `KQAnimObject`, `KQAnimOnce` and characters [traced]; doors use the
     same movement start [inferred: a door is an animated object; its
     own player was not followed].
   - So `wizchest1` is drawn screen-doored once shown, and Ice World's
     `snowmane` (a character, mode 1) is screen-doored in every
     movement.
9. Data (corrected 2026-10-24): **mode 1** only on Daventry `wizchest`
   (`KQChest`) and Ice World `snowmane` (`KQMonster`). **Mode 2**
   (colour key only, rule 4) on Daventry `arroswt1`-`3` (doors), Dead
   City `altar_a1` (animated object), Gnome `hector`, `lucreto` (characters), `irnlck`
   (plain object), `irnlckx` (`KQAnimOnce`), Ice World `chainpc`,
   `cryslock` (doors), Swamp `pulchain` (door), and the temples' animated
   `chandle1`. (The earlier list said 1 for all of them; only the two
   above are 1.) Script uses: Daventry `keepwall` (a plain object;
   Clarity, `potions.md` rule 31) and Dead City `fakedr1a`-`d`, always
   `1` then `0`.
10. B's stand-in (opacity 0.5 for every part) is the wrong look: use the
    texel pattern with the colour key.
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
6. **`loadKQ` while playing** (traced 2026-10-17, Q90): the same
   command and the same steps as at the world's start; nothing checks
   whether the world is already running. The file is read, the name rule
   and the position rules (the −1/−2 rule; x y z and angles replacing
   the file's) are applied, the object is added to the world, and the
   `keepOnGround` raise happens at its first move step, as at the start.
   On being added:
   - the near/far/off test runs at once against Connor's **current**
     position, so it starts active or not straight away;
   - `hidden=1` keeps only its `groupsAlways`, as at the start;
   - its script gets **`$1=<name> $2=Init`** at once, during the
     `loadKQ` line, before the next script line runs (the same call as
     at the start; for animated objects just before their movement
     starts);
   - an animated object (including `KQAnimOnce`) starts its movement as
     at the start (`anim-scripts.md` rule 3);
   - a sound object starts playing at once if it is active
     (`sound.md` rule 46).
   - `$KQObject::loc` holds six numbers, so `loadKQ <file> @
     $KQObject::loc` gives position **and** angles (9 words).
   - **Same name:** if a live object already has that name, the game
     stops with a fatal error (name already used): no replacing, no
     duplicate. A deleted object's name is only free once the event that
     deleted it has finished (`clicking.md` rule 22), so deleting and
     reloading the same name in one script run is also fatal. 31
     delete-then-reload pairs in the data were checked in part; all are
     in different events (data). For us: log an error and refuse the
     load, don't quit.

### Ground heights −1 and −2 (Q17)

1. **In a `loadKQ` line, a z of exactly −1 or exactly −2 means "on the
   ground".** Before placing the object, the game replaces that z with the
   ground height at (x, y). This happens **whatever `keepOnGround` says**.
   (Traced.) The same rule is used by at least one other object-creating
   command. (Corrected 2026-10-11: the first version said `move` uses it
   too; that is **not traced**. Scripts write `move Connor 0 x y -1 …`,
   which suggests the same meaning.)
2. **Corrected 2026-10-12 (Q43):** −1 and −2 are different. **−1 gives
   the floor height** (`terrh2.dib`, the ground under water) and **−2 the
   surface height** (`terrh.dib`). The first version of this rule said the
   ground ignores the difference; that was wrong (it was misread). In a
   world without a floor image both give the surface. See `water.md`.
3. Other values are used as written; then `keepOnGround` (default on) only
   raises objects that are below the ground (default: onto the floor;
   `keepOnGround=1`: onto the surface; `water.md`).
   **The raise also runs after rule 1 (Q44, traced 2026-10-12):** the
   −1/−2 replacement happens first, then the position goes to the
   object's mover, whose placing includes the raise. So a −2 (surface)
   can still be lifted to a higher floor (DeadCity portal: 0 → 2,784).
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
3. Which bit means what (**traced 2026-10-16**, see
   `object-lighting.md` rules 34–40 for near / far / off): bits 0x10,
   0x40, 0x80 and 0x100 each put the object in a draw list (any one means
   "drawn"); 0x8 switches its collider on; 0x2 enters it in the near / far
   / off distance test; **0x400** puts it in the lights group, so its own
   lights shine (traced 2026-10-17, `object-lighting.md` rule 37; the
   earlier guess "running animations" was wrong). With the default `groupsAlways` = 6, **a
   hidden object is not drawn and cannot be touched** until a script calls
   `setHidden <name> 0`.
4. So: don't draw `wizchest1` at the start. An object whose
   `groupsAlways` includes 0x40 (e.g. `groupsAlways=1097` = 0x449, one
   object in the game) would still be drawn when hidden.

## Script variables, `getLoc` and `move` (Q20)

1. **(a) `KQObject::getLoc <name>`** (scripts also write just `getLoc`,
   which does the same; how the short name is found is not traced) reads the
   object's **current** position and turn and sets two variables (traced):
   - `$KQObject::loc` = `"x y z dirX dirY dirZ"` (six numbers; the three
     angles are worked back from the object's rotation, the reverse of
     the `dirX/Y/Z` rule);
   - `$KQObject::locOnly` = `"x y z"`.
   "Current" means after `loadKQ` placed it, including the −1/−2 rule.
   Whether a `keepOnGround` lift has happened yet at that moment is not
   traced. For the Weirdling it doesn't matter: its z 1,744 is above the
   ground there (1,656).
2. **(b) A `$variable` that is not the first word of a line is replaced by
   its text, and that text is split into words again** (traced in the
   console reader). So `loadKQ anvil.kq same $KQObject::loc` becomes
   `loadKQ anvil.kq same x y z dirX dirY dirZ`: nine words, position **and**
   angles, and all `loadKQ` rules apply as if the numbers were written out.
   (Inside double quotes it stays one word.) The anvil, coals and grinder
   therefore get the Weirdling's position (144476, 131239, 1744) and turn
   (dirZ ≈ −1.43). Their own `.kq` files also use variables:
   `locX=$Barren::Seller::xPos` etc., set in Barren's `worldvar.cs` to the
   same numbers. **Values in `.kq` files can be `$variables` too.**
3. **(c)** Variables seen in `loadKQ` lines: `$KQObject::loc` (346 times),
   `$1` (6, a script's first argument), `$IceWorld::ChestPos` (1). The
   latter is set in IceWorld's `newWorld.cs` just before:
   `set IceWorld::ChestPos $ChestX $ChestY $ChestZ 0 0 $ChestRotZ` with
   `ChestX 279587`, `ChestY 498027`, `ChestZ 31641`, `ChestRotZ -2.9`.
   So it reads as `279587 498027 31641 0 0 -2.9`. (From the scripts; that
   `set` joins several words with spaces is likely, not traced.)
4. **(d) `move <who> <flags> x y z [rx ry rz]`** (traced: up to six
   numbers, missing ones are 0; flag meanings from the game's own help
   text): 0 = position and turn absolute; 1 = position relative, turn
   absolute; 2 = position absolute, turn relative; 3 = both relative. So
   `move flamelh.1a 3 0 0 2000` raises the flame 2,000 above where it was
   loaded, keeping its turn.
   **Relative position uses the world axes (Q21, traced):** the game adds
   x, y, z straight onto the object's current position. The object's turn
   is not applied to the offset. So the DeadCity flames `flamedt.32a…38c`
   and pots `pot.prs1a…2c` move by exactly the written numbers along
   world x, y, z, however their torch or shelf is turned.
   **Relative turn** (flags 2, 3): the game builds a turn from rx, ry, rz
   and combines it with the object's current turn. For turns only about
   Z (all 9 cases in the game) that is the same as adding the angles. The
   order of combination for other turns is not traced.
   The `move` command sends the object a move event and the object sets its
   new position directly, without the mover's collision passes (traced).
   **Refined 2026-10-17 (Q92):** there are no collision tests and no
   floor search; it is an instant jump. The vertical bookkeeping of a
   move step still runs: up by the step-up, down by the step-down, plus
   **one gravity step** if gravity is on, and the terrain-only
   `keepOnGround` raise applies. So `move Connor 0 x y 15000 …` from a
   timed script leaves him about one gravity step (about 44) below
   15,000, "in the air", and he falls from there. When the `move` comes
   from a script that runs inside his own update (a `Push`), the rest of
   that update follows it (`pushing.md` rule 11).
   **`setHidden`** only sets the hidden flag and reruns the near/far
   test; position and height don't change. Showing the object again
   rebuilds its collider at its current place (traced, Q92 c).
5. **(e) 4 or 5 numbers in a `loadKQ` line:** the angles are used only when
   all three are given. With x y z plus one or two more numbers, the extra
   numbers are **ignored** and the angles are 0 0 0. (Traced: angles need
   at least 9 words.) So Swamp's `grass.kq swmp.01 166759 68128 -2 -1` is
   at (166759, 68128, ground), not turned.
