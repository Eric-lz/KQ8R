# MODLOG

What changed and what was tested. Newest entries at the top.
Format: date and title, then Changed / Why / Tested how / Result / Still broken / Next.

## 2026-10-12 Study session: fall speed measured in the running original (open question 8)

**Changed:** `docs/behaviour/connor-physics.md` (Gravity, values table, Falling rule 2, Units, open question 8), `docs/behaviour/QUESTIONS.md` (Q30). In `re/` (never committed): `re/dbg/mem_fall.py`, a reader that reads Connor's mover from the game's memory from outside (Windows ReadProcessMemory: read only, no debugger, no pauses, nothing written to the game).
**Why:** The debugger approach froze or crashed the game (entry below), so we switched to reading memory from outside (option 6 in `STATUS.md`).
**Tested how:** The user started the game normally and stood at a ledge, ran off it, landed, stood, then did one standing jump. The reader found Connor by itself (id 0x80B with his mover) and logged a line each time a value changed: 1,091 lines, about one update every 78 ms. I compared every update in the air with the traced rule.
**Result:**

- Every drop in the air (18 updates, timer 0.06 to 0.90) is **exactly 4.000 times** the traced `5000 × (t² − t_prev²) / 2`. New rule: `10000 × (t² − t_prev²)`, so 36, 108, 180, 252, 324, then 396, 468, … (+72 each update). This is the "about 4 times faster" from the user's video.
- As traced: timer +0.06 per update; Fall asked for after 5 updates in the air; after a walk-off the timer carries on from 0.30; after a jump Fall sets it to 0.36 (previous 0.30); the stored base vector is always 0; on a slope the height lost is the drop × sin²θ (to within 0.3 units).
- Standing jump: highest point 1,681 above the start (the note says about 1,697).
- First version of the reader crashed at its first sample (it read too little of Connor's object). Fixed, then it worked.

**Still broken / not tested:** Where the factor of 4 is in the code was not found. Only one ledge fall and one jump were measured. Q30 (b) (what holds him to a walkable slope going downhill) is still open. B's code still uses the old numbers.
**Next:** Session B: change the gravity drop to `10000 × (t² − t_prev²)` (connor-physics.md "Gravity") and update the tests' expected numbers. Session A: the same reader can measure open questions 6 and 7 (walking down a walkable slope, running into a steep one).

## 2026-10-12 Study session: why the original crashes or freezes after a debugger stop

**Changed:** `STATUS.md` "Current problem" only. No code, no notes in `docs/behaviour/`.
**Why:** Both earlier live attempts crashed the game on resume (0xC000041D), so no fall data was logged. Before trying the full script again, find the cause with the smallest possible test.
**Tested how:** With the user at the game, session A drove x32dbg through the plugin: read the debugger's exception settings; ran the game under the debugger with no stops of ours; then set one stop at Connor's update, checked the CPU flags while paused, deleted the stop, cleared the single-step flag, resumed with plain `run` (twice). When the game froze, read its thread states through Windows (`Get-Process`).
**Result:**

- Exception settings are the defaults; the debugger keeps its own signals. Ruled out.
- No stops of ours: the game plays normally under the debugger.
- At the stop, the single-step flag was already on (flags 0x344). After deleting the stop, clearing it and resuming: no crash, but a full freeze both times. 21 of 22 threads suspended, CPU zero, the debugger couldn't pause it any more. In the first of the two runs I told the user the game had "survived" because it didn't crash and the debugger reported "running"; the user's report showed it had frozen. Corrected.
- Likely cause (not proven): changing the stop the game is paused on breaks the debugger's "step past the stop" routine. With the flag left on, the step reaches the game (crash). With the flag cleared, the debugger waits for a step that never comes, with the other threads held (freeze).

**Still broken / not tested:** No fall data logged yet. The explanation is untested. The script's `var` fix is untested.
**Next:** One run to test the explanation: at the stop, single-step once first, then delete the stop, then `run`. If that fails, switch to reading the game's memory from outside with no stops (`STATUS.md` ideas).

## 2026-10-12 Study session: debugger set-up to watch falls in the original

**Changed:** Nothing in the repository except this entry. Outside it: x64dbg (which contains x32dbg) unzipped to `C:\Tools\x64dbg` (path chosen by the user). In `re/` (never committed): a debugger script that logs Connor's movement while the original runs, a decoder for its log, and scratch notes.
**Why:** Open question 8 (falls about 4 times faster in the original) wasn't solved by reading the exe. Watching the running game shows the real numbers each update, and which code changes Connor's height.
**Tested how:** The download's SHA-256 checksum matches the one GitHub publishes for the release. The decoder was run on a made-up log.
**Result:** Ready to run. While preparing it, reading the exe showed something new: each move ends by scheduling Connor's next update event at "now + the time step the move was given", and the move's result is applied only when that event arrives. So Connor's update rate may follow the time step he's given, not only the 1/15 s tick. Not confirmed; the log will show it.
**Still broken / not tested:** The script hasn't been run yet, and x32dbg itself hasn't been started. Not tested: whether the game runs normally under the debugger.
**Next:** The user runs the game under x32dbg with the script, walks off a ledge and does a standing jump; session A reads the log.
**Added later (same day):** The user added the x64dbg-mcp-server plugin so session A can drive x32dbg directly. The first set-up failed with "401 Unauthorized": the header's name had literal quote marks in it (`"Authorization"`). Fixed by adding an `x32dbg` entry to `.mcp.json` (git-ignored) that reads the token from the Windows user environment variable `X32DBG_MCP_TOKEN`, so the token is in no file (AGENTS.md rule 6). The plugin listens on 127.0.0.1:9095 only (checked). Not tested yet: the connection with the new entry (needs the variable and a restart of VS Code).
**First run through the plugin (failed):** The connection works. The game started under the debugger and reached Connor's first update, and the logging stops were set while the game was paused at that point. On resuming, the stop at the paused spot went off again as an unrecognised breakpoint error. Resuming once more passed it to the game as a fatal error inside a Windows callback (0xC000041D), so that game session is lost. No Connor data was logged. Probable cause (not proven): the stop at the spot where the game was paused had been changed while paused on it, so the debugger didn't step around it on resume. All stops were cleared afterwards. Plan for attempt 2: the first stop deletes itself when hit; step one instruction before setting any stop at that spot; resume with the debugger's own `erun` command.
**Attempt 2 (failed, same crash):** I followed that plan, and the game crashed on the first resume with the same fatal callback error, this time with no stop at that spot. The debugger log shows the real cause: `erun` passes the debugger's own last exception (here, my single step) to the game ("Exception will be thrown in the program"), and inside a Windows callback that's fatal. My explanation for attempt 1 was wrong. Also found: `$name = value` doesn't create a debugger variable ("Not enough arguments"), so the Connor-only conditions would have been wrong; the command is `var name, value`. Both fixed in the scratch script, not tested. Stopped after two attempts as AGENTS.md asks. Write-up in `STATUS.md` "Current problem".

## 2026-10-11 Study session: user's recording of a steep-slope fall

**Changed:** `docs/behaviour/connor-physics.md`, open question 8 (the user's recording and what it implies). No code, no new tracing.
**Why:** The user recorded the original running onto a slope too steep to walk on, to time the fall.
**Tested how:** The user watched the recording frame by frame in two places. I compared it with the traced rules.
**Result:** The falling animation starts 18 frames of the 60 fps video (0.3 s, 4 changed game frames; the game ran at about 15 fps) after the first frame that moves down. The traced rules give 5 updates, 1/3 s, 20 video frames. That's within one update, so the start of the fall matches the 15-per-second rules. **Correction:** my first reading took the "4 frames" as video frames and said this backed the guess that the mover runs more often than 15 times a second. It doesn't; the note now says so. The user also saw that he never leaves the ground running over a crest or down a walkable slope (always put on the ground below him), and loses his forward speed only off a real ledge.
**Still broken / not tested:** Why he slides and falls so fast (speed, not timing) is still open (open question 8). The open question 8 guess is now weaker.
**Ledge fall (added after):** the user timed a straight fall from a bit under 2.5 Connor heights: 23 frames at 60 fps from the falling pose to the landing pose, about 6 updates. **Correction:** I first compared it using 720 units for Connor's height; he is 1,600 (collision box), so the drop was about 3,500 to 3,900 units. The rules need 15 to 16 updates after the Fall start for that and give only 864 units in 6. So the original falls about 4 times as far per update. The start of the fall is on time, so "the mover runs more often" is ruled out (the air count would also run faster).
**Follow-up tracing (2026-10-12):** Changed `connor-physics.md` ("Falling off an edge" rule 2, open question 8) and the Q30 answer in `QUESTIONS.md`.

- **Correction of my Q30 answer:** entering Fall sets the timer to 0.36 (previous 0.30) **only after a jump**. From Move or Wait (walking off), gravity is left alone; the timer runs on from 0.30, so the first Fall drop is 99, not 117. B's code follows the old text.
- Ruled out as the cause of the fast fall: the fall animation (`jumphold` has no movement); an extra drop in Fall's per-update check; a second gravity step per update (one caller); the object move handler running the move several times (once, plus a second lowered move only in water).
- Not checked: whether something else lowers the wanted height handed to the mover; a stored base vector the drop routine adds to every drop.

**Still broken / not tested:** Why the original falls about 4 times as far per update is not found. Stopped after these attempts.
**Next:** User check that would separate "stronger gravity" from "something else": time two ledge falls of clearly different heights (about 1 and about 2 Connor heights), counting from the first frame he leaves the ledge to the landing. If the times grow like a falling object (twice the height, about 1.4 times the time), it is a stronger gravity, and its strength can be worked out from the two times.

## 2026-10-11 Study session: Q30 (update rate, Fall start, fast falls)

**Changed:** `docs/behaviour/connor-physics.md` (Units "How often updates happen", "Falling off an edge" rule 2, new open question 8), `docs/behaviour/QUESTIONS.md` (Q30). No code.
**Why:** In the original, falls and slides down steep slopes are much faster than in B's code. B asked whether the update rate, the Fall start or the pull explains it.
**Tested how:** Traced in `Mask.exe`: the `FPSLimit` command, the main loop and the per-frame routine, the game clock, the 1/15 s update timer, the Fall state's start and per-update routines (instruction by instruction), Connor's per-update handler, and the code that calls the mover's move routine.
**Result:** The update tick is a fixed 1/15 s timer in game time, and game time follows real time (millisecond clock, at most 1 s per frame, catch-up when frames are slow). `FPSLimit 15` only limits how often frames are drawn and game time is moved on; it doesn't change the tick. So updates don't run faster on a modern PC. Entering Fall sets the timer to 0.36 (previous 0.30) and doesn't touch the pull. Nothing else in Fall changes gravity. B's numbers (117, 135, 153… per update; drops projected onto the slope) are what the traced rules give.
**Failed attempts:** Looked for what hands Connor's animation step to the mover each update, to confirm it runs on the tick. Found the object move-event handler and two other callers (other classes), but not Connor's sender. Also checked the frame-rate display, the sound timer (`timeSetEvent`, sound only) and the extra thread (graphics). Several attempts, so stopped.
**Still broken / not tested:** Why the original falls so much faster is not explained (open question 8). A guess, not shown: the move routine also runs more often than the tick, which would also explain open questions 6 and 7. What keeps him on a steep slope going down (Q30 b) is open.
**Next:** User: record the original while Connor walks off a high ledge and count frames from the falling pose to the landing (rules: about 0.7 s for 2,000 units). That decides whether to look further at how often the mover runs.

## 2026-10-11 Study session: Q29 (what counts as on the ground) and Q28 (a)

**Changed:** `docs/behaviour/connor-physics.md` ("Falling off an edge" rule 1, "Gravity", the steep-hill consequence, new open question 7), `docs/behaviour/QUESTIONS.md` (Q28, Q29). No code.
**Why:** B's slope code falls on a slow walk into a steep slope; the user's checks in the original show it doesn't, and that he gains little height.
**Tested how:** Read in `Mask.exe`, at instruction level: the air-counter update and the Fall request. Searched the exe for every write to the mover's on-ground flag and air counter. Re-read the gravity step for how the drop follows the pull.
**Result:** Only a walkable contact (normal z > 0.8) counts as on the ground; steep contact never does. There's no speed or height condition. The count is exact: Fall is asked for at the start of the update after 5 updates in a row that ended not on the ground. So 4 air updates and a landing in the 5th start no fall, which fits the user's walking check; B's code seems to fall one update early. Q28 (a): the drop is the vertical drop projected onto the pull, so the note's 45° example stands.
**Still broken / not tested:** Why he gains so little height running into a steep slope is not explained (open question 7). Not looked at yet: where the mover's wanted position comes from each update, and the end-of-update step. Q28 (b), (c) and whether `jumpend` plays in full after a one-update fall are not traced.
**Next:** B fixes the count and re-checks the walking case; then open question 7 (which may also explain open question 6) if the user wants it.

## 2026-10-11 Camera tests follow the hand-tuned values

**Changed:** `crates/kq8-game/src/camera.rs`, tests only (4 of them). They now use `AIM_HEIGHT`, `MIN_HEIGHT`, `HEIGHT_RATE` and `FIXED_ZOOM_PER_UPDATE` instead of the note's numbers written in. The camera code and the user's values (aim 800, height rate 0.5, commit `10cf08b`) are unchanged.
**Why:** The user asked. The 4 tests assumed the aim point is at the camera's minimum height (both 1,400 in the note). With aim 800 there is a 600 vertical gap, so the leash, zoom and fixed distances are now measured in 3D, and the height rule then moves the camera up or down.
**Tested how:** `cargo test -p kq8-game` with the user's values, and once with the note's values (1,400 and 0.1) swapped in temporarily, then restored.
**Result:** All `kq8-game` tests pass with the user's values: 17 at this commit (run on a copy of the committed files), 24 with the slopes work that came after. The 7 camera tests also pass with the note's values.
**Still broken / not tested:** With the aim below the minimum height, the fixed-distance mode (F11) doesn't hold its distance exactly. Each update the height rule moves the camera after the distance is set, so the distance drifts by up to the 600 gap. The test now allows for this. Not checked in the viewer.
**Next:** None for the tests.

## 2026-10-11 Study session: when the Fall state starts (with the user's checks)

**Changed:** `docs/behaviour/connor-physics.md` ("The terrain in the passes", "Gravity", "Slopes", "Falling off an edge" rule 1, How to check, open question 6), `docs/behaviour/QUESTIONS.md` (Q26 row). No code.
**Why:** The user asked for falling to be finished before session B starts, so B can do slopes and falling together.
**Tested how:** Traced in `Mask.exe` Connor's states and which state changes each one accepts, the pending-state (`nextState`) handling, the gravity step instruction by instruction, every reset of the fall timer, and the ground check's comparison. The user checked the original game: holding walk off a high ledge, and onto an unwalkable slope.
**Result:** Move and Wait both accept Fall in normal play. So after 5 updates in the air Fall starts, also while walking, which matches the user's checks: he drops a little, then falls straight down (ledge) or slides down (slope). Gravity follows the "pull" direction. The pull is reset to straight down each update that starts on the ground. Gravity is skipped when the pull is within 30° of horizontal. The timer is reset only by a down-pass landing on walkable ground.
**Failed attempts:** My first reading of the Move state's accept rule was backwards. It went into the previous entry and commit `0215e83` ("Move state refuses Fall"), which this entry replaces. The user's checks showed it couldn't be right, and re-reading the rule fixed it. Two searches for some other fall trigger found nothing, because none is needed.
**User's running test:** Running down the steepest walkable hills in Daventry, he never falls. The traced rules predict a fall on slopes over about 15°, so they are missing something. Searched without finding it: the gravity step instruction by instruction, the terrain lookup's extra flag (unused), every reset of the fall timer, and every write to the step-up/step-down values (the others belong to other movers). Three attempts, so stopped.
**Still broken / not tested:** What keeps Connor on walkable ground going downhill is not found. The note gives a recommended rule for B instead, marked as our own design: snap to walkable ground within step down + this update's drop below, if he was on walkable ground last update. It reproduces everything the user saw, but loses the slight camera bounce.
**Next:** Commit; B implements slopes and falling with the recommended rule.

## 2026-10-11 Study session: correction, no Fall while walking or running downhill

**Changed:** `docs/behaviour/connor-physics.md` ("The terrain in the passes", "Slopes", "Falling off an edge" rule 1, How to check, new open question 6), `docs/behaviour/QUESTIONS.md` (Q26 prediction withdrawn). No code.
**Why:** The previous entry predicted that running down slopes over about 15° would start the Fall state. The user checked the original: no falling animation on any walkable slope. It only appears on unwalkable slopes, and the camera bounces slightly going down very steep walkable ones.
**Tested how:** Traced in `Mask.exe` how the Fall request is handled. Connor's current state decides whether to accept a new state, and the Move state's rule refuses Fall unless a "controls taken away" flag is set (puzzles, a few scripted moments).
**Result:** While walking or running, being in the air never starts Fall, which matches the user's test. The short hops downhill still happen (the camera bounce). The steep-hill example changes: at 45° he gains about 390 units in 0.6 s, then slides back (computed).
**Still broken / not tested:** What starts the fall on unwalkable ground or off a ledge while a move key is held is not traced (open question 6). Whether Wait and other states accept Fall is not traced. The previous prediction was wrong because it assumed every state accepts Fall.
**Next:** Open question 6 if B needs it for step 2e; otherwise Q27.

## 2026-10-11 Study session: Q26 (terrain in the passes) and Q23 (gravity off)

**Changed:** `docs/behaviour/connor-physics.md` (steps 2–5, new section "The terrain in the passes", gravity-off correction, How to check), `docs/behaviour/QUESTIONS.md` (Q23, Q26 answered). No code.
**Why:** M2 step 2c (slopes) was waiting on Q26. Q23 is about the same update.
**Tested how:** Traced in `Mask.exe`: the mover's three-pass update, the sweep each pass calls, the terrain test inside it, Connor's collision response, where gravity is applied, and Connor's constructor (collision type and mask).
**Result:** The terrain counts as a hit at the end of the up and down passes, not the across pass, when it is above where the pass ended. Connor's collision mask turns this on. A steep hit pushes him back onto the slope, and gravity grows until Fall starts (about 0.4 s). Downhill, the terrain never pulls him down, so B's "in the air every other update" log matches the original. With gravity off he hovers over an edge; the old "drops 325 per update" was wrong.
**Still broken / not tested:** Nothing checked in the running game. The prediction that running down slopes over about 15° starts the Fall state is derived from the rules only. Whether the gravity drop goes straight down or along the slope's pull, while sliding, is not re-checked.
**Next:** The collision-shape question (`connor-physics.md` open question 2), then Q27.
**Added (same session):** New section "Collision shape" in `connor-physics.md`. Connor collides as the `boxf` box; the sphere keys are never read (the strings aren't in `Mask.exe`), and neither are the collider section's `mask`/`type`/`collisionDetail`. His type (4) and mask (0x281B) come from his constructor. Two objects are tested when either one's mask shares a bit with the other's type. Not traced: the box-vs-building test and the meaning of most bits.

## 2026-10-11 Stop publishing local working notes

**Changed:** `.gitignore`. Untracked (kept on disk) `STATUS.md`, `STATUS-handoff.md`, `MODLOG-template.md` and `docs/SESSIONS.md`. No code.
**Why:** These are working notes for hand-offs between chats, not something readers of the repo need. Past history is left as is.
**Tested how:** `git check-ignore -v` reports all four as ignored; `git ls-files` no longer lists them; the files are still on disk.
**Result:** The removal is staged, not committed.
**Changed (links):** Also fixed the links to the local-only files in `README.md`, `docs/DESIGN.md` and the header of this file.
**Still broken / not tested:** `AGENTS.md` and `CREDITS.md` still name the local-only files in plain text. Links checked with grep only.
**Next:** None.

## 2026-10-11 Study session: Q21 (relative move axes)

**Changed:** `docs/behaviour/objects.md` (Q20 (d)), `docs/behaviour/QUESTIONS.md`. No code.
**Why:** B asked whether a relative `move` on a turned object uses world axes or the object's own axes.
**Tested how:** Traced the `move` command, the move event it sends, and the object's handler in `Mask.exe`.
**Result:** World axes: x, y, z are added straight onto the current position, and the turn is not applied. Relative turns are combined with the current turn; for turns about Z only, that is the same as adding the angles. `move` sets the position directly, without collision passes.
**Still broken / not tested:** The combination order for non-Z turns is not traced. Whether the ground rule runs after `move` is not traced.
**Next:** M2 questions from B.

## 2026-10-11 Study session: M2 physics (gravity, steps, slopes, walls, jump, fall)

**Changed:** New `docs/behaviour/connor-physics.md`; pointer in `docs/behaviour/connor-movement.md`. No code.
**Why:** M2 needs Connor held on the ground, stopped by walls, and able to jump and fall.
**Tested how:** Traced in `Mask.exe`: the mover's three-pass move, gravity, the ground check, Connor's collision response, the Jump and Fall states, the jump permission check, and the monster/Connor constructors. Measured the jump arcs from `C_jump`, `C_rnjmp` and `C_bkjmp.kqs`.
**Result:** Step up and step down are 325 per update. Gravity is 5,000 units/s², with the timer advancing 0.06 s per update. Ground is walkable when the normal's z is above 0.8 (36.9°); steeper ground makes Connor slide. A jump's arc comes from its animation, with gravity off. After a jump comes the Fall state. A fall kills Connor if he drops more than 3,840 (5,120 after a jump). Walls hit within about 20° of head-on make him stop when walking and crash when running; at other angles he slides along them.
**Still broken / not tested:** Whether Connor's gravity is on at the start of a world is not traced; the recommendation is on. The collision shape used against buildings is not traced, and `MaxSlopeDot` is unused for Connor. Nothing has been checked in the running game.
**Next:** Answer session B's M2 questions; water, ropes and pushing later.

## 2026-10-11 Study session: Q20 (script variables, getLoc, move)

**Changed:** `docs/behaviour/objects.md`, `docs/behaviour/QUESTIONS.md`. No code.
**Why:** Objects placed through `$KQObject::loc` were missing.
**Tested how:** Traced `KQObject::getLoc`, the console's variable substitution and the `move` command in `Mask.exe`; read Barren, IceWorld and Swamp scripts.
**Result:** `$KQObject::loc` holds six numbers (position and angles). Variables are split into separate words, so `loadKQ` treats them as written numbers. `move` has flags 0–3 (absolute or relative). With 4–5 numbers, the extras are ignored. Values in `.kq` files can be variables.
**Corrections:** Withdrew the Q17 claim that `move` uses the −1 ground rule (not traced).
**Still broken / not tested:** Whether a relative move uses world or object axes, and how the short `getLoc` name is found, are not traced.
**Next:** M2 study.

## 2026-10-11 Study session: Q5 and Q3 answered

**Changed:** `docs/behaviour/models-and-textures.md`, `docs/behaviour/QUESTIONS.md`. No code.
**Why:** The last open texture questions.
**Tested how:** Traced the Glide renderer's function table: the colour-key switch and the blend modes. Traced the palette set-up that copies the Windows system palette. Measured palette entries and texture colour use.
**Result:** 0x1000 = colour key (pure black is see-through) plus full brightness; 0x2000 = alpha blending (only in materials with nothing to draw). Palettes already contain the system colours; the game replaces 10 entries at each end only on a 256-colour screen.
**Still broken / not tested:** The Direct3D path is not checked. The teleport pad's see-through look (Q14b) is still unexplained. What happens to missing texture names is still open.
**Next:** M2 study (slopes, collisions, jumping).

## 2026-10-11 Study session: Q18 (hidden), Q4 (lookup order)

**Changed:** `docs/behaviour/objects.md` (Hidden objects), `docs/behaviour/models-and-textures.md` (Q4), `docs/behaviour/QUESTIONS.md` (Q4, Q18, stale Q5 row). No code.
**Why:** Open questions from session B.
**Tested how:** Traced `setHidden`, the object group switching, `loadVolume`, the search list and `moveVolToFront` in `Mask.exe`. Hashed every duplicate file name across the loaded folders of all 12 worlds.
**Result:** A hidden object keeps only its `groupsAlways` groups, so by default it isn't drawn. When names clash, the folder loaded last wins. The game reads loose folders, including patch folders that the `.vol` archives don't contain.
**Still broken / not tested:** Group bit meanings are inferred. The order inside `GAME/` and what happens to missing texture names are not traced. Q3 (system colours), Q5 (which colour is see-through) and Q14b (teleport pad) are still open.
**Next:** M2 study.

## 2026-10-11 Study session: Q19 (missing locZ)

**Changed:** `docs/behaviour/objects.md`, `docs/behaviour/QUESTIONS.md`. No code.
**Why:** Barren's two lookout buildings have no `locZ` and were skipped.
**Tested how:** Traced the `.kq` position reader and the mover's starting values in `Mask.exe`; computed the ground height at both lookouts from Barren's height map.
**Result:** A missing position value stays at 0, then the ground rule lifts the object onto the ground (11,592 for both lookouts).
**Still broken / not tested:** Not checked by eye.
**Next:** M2 study.

## 2026-10-11 STATUS.md brought up to date

**Changed:** `STATUS.md` (after M2 step 2f: what is done, next steps 2c/2d/2e, open questions Q22–Q27; corrected the game version to what `MASK/Version.txt` and `readme.txt` say, the test count, the F key, and the step order).
**Why:** The user asked for the hand-over to show the current state.
**Tested how:** Read against `git log`, `cargo test` (kq8-data 41, kq8-game 17) and `QUESTIONS.md`.
**Result:** —
**Still broken / not tested:** —
**Next:** —

## 2026-10-11 README.md simplified

**Changed:** `README.md` (rewritten in plain words: what works, what doesn't, how to try it with a key table; the numbers and details are in `docs/provenance/README.md` and this log).
**Why:** The user found it out of date ("no buildings or objects") and too technical.
**Tested how:** Read against `STATUS.md` and the latest entries here; the commands listed exist.
**Result:** —
**Still broken / not tested:** —
**Next:** —

## 2026-10-11 M2 step 2f: the camera follows Connor

**Changed:** new `crates/kq8-game/src/camera.rs` (rules, 7 tests), new `crates/kq8-viewer/src/game_camera.rs` (keys, 15-per-second update after Connor, blended drawing, `camera.log`), `crates/kq8-viewer/src/load.rs` (Camera.sae, walk/run ranges, camera start), `crates/kq8-viewer/src/connor_view.rs` (keys `prior`/`next`/F11; the old C and F-follow keys removed), `crates/kq8-viewer/src/terrain_view.rs` (free camera only when chosen; Esc always quits), `crates/kq8-viewer/src/main.rs`, `docs/DESIGN.md` (2f plan, moved before 2c), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q27), `README.md`, `STATUS.md`.
**Why:** The user asked to do the camera before slopes. Rules from `camera.md`.
**Tested how:** `cargo test` (kq8-game 17, kq8-data 41). Viewer: `--hold 1,0,1` (running, 1,500 frames), read `camera.log`; Swamp, Barren, CastleD open with Connor and the camera.
**Result:** Daventry: the camera starts at the script's `KQCamera` spot (3,147 from the aim point), stays put until Connor is 5,000 away, then trails at 4,999.0; never below 1,400 above the terrain; rises smoothly when he climbs. Keys found: F11, Page Up/Down, Home, Shift, right mouse.
**Checked by eye** by the user 2026-10-11: follow distance and height above the ground right; Page Up/Down tilt right. Mouse swing and zoom were the wrong way round (mouse left swung right, mouse down zoomed out): both flipped in `game_camera.rs`, then the mouse tilt with Shift was backwards and was flipped in `camera.rs` (swing confirmed after the first fix). Then, after the user compared with the original: one range (1,500–5,000) for walking and running (no longer tied to `walk.cs`/`run.cs`), and the free camera looks with the right mouse button, as the original's camera does.
**Still broken / not tested:** Stand-ins until Q27: mouse pixels ÷ 600, tilt pitches the view by tilt − 0.2, range tied to the run switch, follow on although Daventry's script says `follow none`. Not done: swing back behind Connor (Q27 c), obstacle check (2d), first person.
**Next:** User's eye check.

## 2026-10-11 STATUS.md updated

**Changed:** `STATUS.md` (rewritten for the state after M2 step 2b: code layout, what works, next steps, open questions Q14b and Q22–Q26).
**Why:** The user asked to keep the hand-over current.
**Tested how:** Read against `MODLOG.md`, `docs/DESIGN.md` and `docs/behaviour/QUESTIONS.md`.
**Result:** —
**Still broken / not tested:** —
**Next:** Q26, then M2 step 2c.

## 2026-10-11 M2 step 2b: Connor walks, runs, steps back and turns

**Changed:** new `crates/kq8-data/src/action_map.rs` (`.sae` reader, test), new `crates/kq8-game/src/connor.rs` (intent → movement, turning, step from the animation; tests), `crates/kq8-viewer/src/load.rs` (`load_animation`: every frame of a loop; Connor's six movements and keys), `crates/kq8-viewer/src/connor_view.rs` (keys held, walking, frames drawn in turn, F follow), `crates/kq8-viewer/src/main.rs` (`--hold F,T,R` test option), `docs/DESIGN.md` (plan 2b), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q24, Q25, Q26), `README.md`.
**Why:** M2 step 2b, as asked; rules from `connor-movement.md`. Intent from keys held, as the note recommends; no setting for the original behaviour (user's choice).
**Tested how:** `cargo test` (kq8-game 10, kq8-data 41). Viewer: standing, and `--hold` for walk (1,0,0), run (1,0,1), turn left (0,1,0), back (−1,0,0), each about 12 s; read `connor.log`.
**Result:** Speeds measured from the animations: walk 1,126.6, run 2,518.1, back −1,126.6 units/s (the note's ≈ 1,127 and ≈ 2,518). Walk: 11,191 in 149 updates, exactly that speed. Turn: 3.383 rad in 29 updates (1.75 rad/s). Back: same distance, other way.
**Checked by eye** by the user 2026-10-11: animations work; walking and running pace very close to the original. Seen: running at a steep slope he slides a long way up it, then slowly back down; in the original he is stopped much sooner (Q26 / step 2c).
**Still broken / not tested:** Running downhill he is 1.4 units in the air every other update, and he half-climbs ground steeper than 36.9° (after about 90 updates of running north from the start): both because the terrain only acts in the ground check, as the note reads; asked as Q26 (sliding is step 2c). No start/stop animations (Q25); steps spread evenly (Q24); no banking.
**Next:** User's eye check.

## 2026-10-11 M2 step 2a: Connor stands in Daventry, gravity on

**Changed:** new crate `crates/kq8-game` (`mover.rs`: Connor's mover, unit tests), `Cargo.toml` (workspace), `crates/kq8-viewer/Cargo.toml`, `crates/kq8-viewer/src/connor_view.rs` (new: draw Connor, run the mover at 15 updates per second, test keys), `crates/kq8-viewer/src/load.rs` (`place_connor`), `crates/kq8-viewer/src/terrain_view.rs`, `crates/kq8-viewer/src/main.rs` (`--lift N`, fixed 15 Hz schedule), `docs/DESIGN.md` (M2 plan, physics rate), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q23), `README.md`.
**Why:** Start of M2, as asked: Connor's gravity on, logged. Rules from `connor-physics.md`.
**Tested how:** `cargo test` (5 new mover tests pass, 45 in all). Viewer runs: standing (300 frames), `--lift 2000` and `--lift 5000` (700 frames); read `kq8-artifacts/logs/connor.log`.
**Result:** Connor loaded from `GAME/Kq/Connor.kq` at (277744, 199295, 13633) with `c_stop.kqs`. Standing: z stays 13,633.5, on ground every update. Lift 2,000: Fall starts at update 5, safe landing at update 14 (fell 1,856). Lift 5,000: dies at update 22 (fell 4,856 > 3,840).
**Checked by eye** by the user 2026-10-11: Connor looks right (textures, standing pose); B lifts him a bit, G more; he falls naturally.
**Still broken / not tested:** He cannot walk; no fall animations; steep ground only logged. The update rule hovers with gravity off, which contradicts the note: asked as Q23.
**Next:** User's eye check; then 2b (walking).

## 2026-10-11 Palettes used as stored (Q3)

**Changed:** `crates/kq8-data/src/palette.rs` (`engine_colors` with its forced system colours removed; `shown_colors` returns the stored colours; tests), `crates/kq8-cli/src/bitmap_png.rs`, `crates/kq8-viewer/src/load.rs` (use it; texture log counts pixels on entries 0–9/246–255 and on entry 0), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q22).
**Why:** `models-and-textures.md` "System colours" (Q3): the game swaps in system colours only on a 256-colour screen; on a normal screen it uses the palette as stored.
**Tested how:** `cargo test` (40 pass). Viewer on all 12 worlds; counted textures (coloured by the world palette) that use the affected entries.
**Result:** None in 11 worlds. Swamp: `STN1.BMP` (35 pixels on edge entries) and `WIRLPOOL.BMP` (16,512 pixels on entry 0); `Teleport.BMP` (3,961 pixels on entry 0). Entry 0 is stored pure blue, so those pixels are now blue instead of black. The Q5 answer says the engine sets entry 0 to black, which conflicts: asked as Q22.
**Still broken / not tested:** Not checked by eye. Entry 0's colour depends on Q22.
**Next:** User's eye check (Swamp whirlpool), Q22.

## 2026-10-11 Q21 answered: relative moves use the world axes

**Changed:** `crates/kq8-viewer/src/load.rs` (log wording only), `docs/DESIGN.md`, `docs/provenance/README.md`.
**Why:** `objects.md` Q20 (d), answer to Q21: a relative `move` is along the world axes, which is what the code already did.
**Tested how:** Built; no behaviour change.
**Result:** The 17 DeadCity cases are now logged as following Q21 instead of "not traced".
**Still broken / not tested:** Nothing new.
**Next:** —

## 2026-10-11 M1 step 3f: hidden objects (Q18)

**Changed:** `crates/kq8-data/src/objects.rs` (`hidden`, `groupsAlways`, `drawn()`; unit test), `crates/kq8-data/src/world_script.rs` (`KQObject::setHidden`; test), `crates/kq8-viewer/src/load.rs` (applies `setHidden`, logs), `crates/kq8-viewer/src/terrain_view.rs` (hidden objects shown only with H), `docs/DESIGN.md` (plan 3f), `docs/provenance/README.md`, `README.md`.
**Why:** Rule from `objects.md` "Hidden objects" (Q18): `wizchest1` should not be drawn at the start.
**Tested how:** `cargo test -p kq8-data` (40 pass). Ran the viewer on all 12 worlds.
**Result:** Placed counts unchanged; objects hidden at the start, now drawn only with H: Daventry 1 (`wizchest1`), Gnome 20 (bridges and gem lights), Barren 50 (docks, rails, lava tiles), Snowexit 1 (`portal4`). Other worlds 0. No start-up script un-hides anything.
**Still broken / not tested:** Not checked by eye. That bit 0x40 of `groupsAlways` means "drawn" is inferred, not traced (no object we place uses it). Daventry's `KQObject::setHidden $NewInvItem 1` is not run (the variable comes from an inventory command we do not run; the item has no position).
**Next:** User's eye check.

## 2026-10-11 M1 step 3e: script variables, getLoc and move (Q20)

**Changed:** `crates/kq8-data/src/world_script.rs` (variables, `set`, `$1`, quotes, `getLoc`, `move`, 4–5-number `loadKQ`; the runner now calls the viewer as it goes, through a small "host" interface; tests), `crates/kq8-data/src/objects.rs` (tests), `crates/kq8-viewer/src/load.rs` (objects placed while the scripts run, by a new `Placer`; `.kq` values that are variables; `move`; logs), `docs/DESIGN.md` (plan 3e), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q21), `README.md`.
**Why:** Barren's anvil, coal box and grinding wheel (seen missing by the user), DeadCity's torch flames and others are placed through script variables. Rules from `objects.md` "Script variables, getLoc and move" (Q20).
**Tested how:** `cargo test -p kq8-data` (39 pass). Ran the viewer on all 12 worlds and compared placed counts with the previous run.
**Result:** 0 unreadable script lines in every world (DeadCity had 184). Placed: Barren 287 → 290 (anvil, coals, grinder at the Weirdling's spot 144476, 131239, 1744, turned −1.43), DeadCity 953 → 1137 (184 `move` lines run, e.g. flame `flamelh.1a` lifted 2,000 above its torch), Swamp 645 → 647 (grass with 4 numbers), IceWorld 99 → 100 (the chest-room henchman figure `Icehench`). The other 8 worlds unchanged. Commands we do not run are no longer reported as unreadable.
**Checked by eye** by the user 2026-10-11: Barren's anvil, coal box and grinding wheel are in the shed; DeadCity's torches have flames.
**Still broken / not tested:** 17 DeadCity relative moves of turned objects use world axes (not traced, asked as Q21). IceWorld's `move Henchman` comes before Henchman is loaded, so it does nothing. `$KQGame::runOptimal` is never set (passed to `permwrld.cs`, unused there as far as we load). Q18 (`hidden`) is answered but not applied yet: next small step.
**Next:** Q18 (`hidden`), then commit when asked.

## 2026-10-11 Barren's shed objects: question Q20; `kq8 text`

**Changed:** `crates/kq8-cli/src/main.rs` (new command `kq8 text <file>`: prints a game text file with line numbers; nothing is saved), `docs/behaviour/QUESTIONS.md` (Q20).
**Why:** The user saw an anvil, a coal container and a grinding wheel missing from a shed in Barren at about (145447, 132566, 2139).
**Tested how:** `cargo run -- text vol:Barren/kq/NEWWORLD.CS` and the same for DeadCity's `PERMWRLD.CS` and Barren's `TW_WEIRD.KQ`.
**Result:** They are `anvil.kq`, `coals.kq`, `grinder.kq`, loaded with the position `$KQObject::loc`, which the line before sets with `getLoc Weirdling` (the weapon seller stands at 144476, 131239, 1744, next to the user's spot). DeadCity uses the same pattern 65 times (torch flames), followed by `move <flame> 3 0 0 <n>`. The notes do not cover `getLoc`, script variables or `move`, so not guessed: asked as Q20.
**Still broken / not tested:** These objects stay missing until Q20 is answered (Barren 3, DeadCity 184 lines, IceWorld 1 with `$IceWorld::ChestPos`). Swamp's 2 unreadable lines are a different case (4 numbers on a `loadKQ` line), added to Q20 as (e).
**Next:** Wait for Q20.

## 2026-10-11 M1: missing locX/locY/locZ are 0 (Q19); Barren's lookouts drawn

**Changed:** `crates/kq8-data/src/objects.rs` (each position key read on its own, missing = 0; `from_section`/`from_load` always give a placement; unit test), `crates/kq8-viewer/src/load.rs` (no more "no position" skip; logs which keys were missing), `docs/provenance/README.md`, `README.md`.
**Why:** Barren's `lookout` and `lookout2` (the user's missing hilltop building and bridge) have no `locZ`. Rule from `objects.md` "Missing locX / locY / locZ" (Q19).
**Tested how:** `cargo test -p kq8-data` (38 pass). Ran the viewer for all 12 worlds (`--terrain --world <W> --frames 3`) and compared the placed counts with the previous run.
**Result:** Barren: 284 → 287 placed. `lookout` (78812, 124815) and `lookout2` (71571, 108848) raised from 0 to 11,592, as the note says. `sparks16` has no `locX`, so it lands at x 0, about 20,000 under the ground (its file turns the ground lift off); that is what the rule gives. Every other world places the same objects as before. Objects with no position keys at all (e.g. music, inventory, lava animators) have no model; they now show as green cubes at the map corner with H.
**Still broken / not tested:** Not checked by eye yet.
**Next:** User's eye check in Barren.

## 2026-10-11 Viewer: camera position on screen

**Changed:** `crates/kq8-viewer/src/terrain_view.rs` (bottom-left text, updated every frame; the log's camera lines use the same conversion), `crates/kq8-viewer/src/main.rs` (registers it).
**Why:** The user asked to see the camera's X, Y, Z in the window.
**Tested how:** Built without warnings; `kq8-viewer --terrain --frames 10` runs and closes normally. The text itself is **not tested** (needs eyes).
**Result:** "Camera X … Y … Z …   ground …" in game units (Z up, same numbers as the `.kq` files and the log), plus the ground height under the camera.
**Still broken / not tested:** Ground view only.
**Next:** User's eye check.

## 2026-10-11 Barren's missing hilltop building: question Q19

**Changed:** `crates/kq8-viewer/src/load.rs` (a skipped "no position" object now logs its sections and keys), `docs/behaviour/QUESTIONS.md` (Q19).
**Why:** The user saw a big building missing on a hill in Barren, near where they closed the viewer: game (74236, 101580).
**Tested how:** Read the viewer log for Barren; ran all 12 worlds and listed objects skipped for having no position.
**Result:** It is `lookout2` (`lookout4.dig`) at (71571, 108848), and nearby `lookout` (`lookout.dig`) at (78812, 124815). Both files have `locX` and `locY` but no `locZ`, so we skip them. They are the only two such objects in the game. The notes do not say what a missing `locZ` means, so not guessed: asked as Q19. Later the user gave the exact spot of the missing building and bridge (74394, 93402, z 12137). It lies inside `lookout4.dig` placed at lookout2's x, y and turned 180° (the model reaches about 22,000 units south of its position), which confirms lookout2. Added to Q19.
**Still broken / not tested:** Both lookouts are still missing until Q19 is answered.
**Next:** Wait for Q19.

## 2026-10-11 Viewer: wider field of view in the ground view

**Changed:** `crates/kq8-viewer/src/terrain_view.rs` (camera field of view 70 degrees vertical, was Bevy's default 45).
**Why:** The user asked for a wider view.
**Tested how:** Built; `kq8-viewer --terrain --frames 10` runs and closes normally. The look is **not tested** (needs eyes).
**Result:** About 100 degrees horizontally on a 16:9 window.
**Still broken / not tested:** Only the ground view; the single-model view keeps its old field of view.
**Next:** User's eye check.

## 2026-10-11 Viewer: H shows invisible objects

**Changed:** `Cargo.toml` (Bevy UI features, for the text), `crates/kq8-viewer/src/load.rs` (collects objects with a position but nothing to draw; logs `hidden=` values), `crates/kq8-viewer/src/terrain_view.rs` (cubes + H key), `crates/kq8-viewer/src/main.rs` (registers the key; fixes the help text: it is the left mouse button), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q18), `README.md`.
**Why:** The user asked to see the invisible objects out of curiosity.
**Tested how:** Built; `kq8-viewer --terrain --frames 5`; log says 72 no-model markers and 5 `.anm` characters. The H key itself is **not tested** (needs a person at the keyboard).
**Result:** 77 cubes, 1 m wide, sitting at each object's position (same ground rules as drawn objects), hidden at start. Green = no model (traps, sounds, walking paths, leaf/stream emitters, camera); orange = `Girl`, `Wizard`, `Ravenw`, `Shadow`, `Uglybeast`. Found `wizchest1` has `hidden=1`; meaning unknown, asked as Q18, still drawn.
**Still broken / not tested:** No name labels (names and positions are in the log as `marker [...]` lines). Trap and path sizes are not shown, only their anchor point.
**Added after the user's question:** H also shows **unused objects**: `.kq` files in the world's archives that no start-up script loads (they were drawn before step 3c placed only script-loaded objects). 16 placed; 29 templates at (0, 0, 0) and 54 without a model or position are left out. Among them are the pictures the user remembered: `tap01`, `tap02` (near the church), `tap04`, flat picture models no Daventry script names. The log line `unused object ...` says which other scripts, if any, name the file: e.g. `potion`, `Pyramid`, `bell`, `waterfall2` are named by event scripts (`Starball.cs`, `Unicorn.cs`, `BELSTND.CS`, `BRICKS.CS`), so they probably appear later in the game; `tap01/02/04`, `easel2`, `pubart1`, `fenc2`, `zedrock1/2` are named by none. Checked by comparing the object lists of the step 3b build and this one (3b had 170, all of which are now either loaded or among these).
**Added after that:** a small white text in the top-left corner, "Hidden objects: hidden (H)" / "Hidden objects: SHOWN (H)". Needed Bevy's UI features (`bevy_ui`, `bevy_ui_render` in `Cargo.toml`). Built without warnings and the viewer runs; the text itself is **not tested** (needs eyes). All 12 worlds load with the H key code (counts in the session notes); DeadCity has 184 `loadKQ` lines with a variable position (`$KQObject::loc`) that we cannot read yet.
**Next:** User's eye check, then commit.

## 2026-10-11 M1 step 3d: script objects at z −1/−2 stand on the ground

**Changed:** `crates/kq8-data/src/objects.rs` (new rule + unit test), `crates/kq8-viewer/src/load.rs` (uses it, logs it), `docs/DESIGN.md`, `docs/provenance/README.md`, `README.md`.
**Why:** 79 Daventry objects (graveyard tombs, 44 trees) were ~8,000 units under the ground.
**Tested how:** `cargo test -p kq8-data` (37 pass). `kq8-viewer --terrain --frames 30`, then read `kq8-artifacts/logs/viewer.log`.
**Result:** 79 objects put on the ground by the −1/−2 rule. Objects more than 500 units below the ground: 83 → 4 (two docks, the lookout, the mill wheel, which reach into water or hillside on purpose).
**Still broken / not tested:** Not checked by eye yet. Teleport pad look (Q14b), haze, building lighting, the river's flowing textures (`wateran`) still missing.
**Next:** User's eye check, then commit.

## 2026-10-11 Study session: Q17 (objects at −1 under the ground)

**Changed:** `docs/behaviour/objects.md`, `docs/behaviour/QUESTIONS.md`. No code.
**Why:** 79 script-placed Daventry objects (tombs, trees) ended up under the ground.
**Tested how:** Traced the `loadKQ` command and the height lookup it calls in `Mask.exe`; read the `.kq` files of the position-less objects.
**Result:** In `loadKQ` (and `move`), a z of exactly −1 or −2 is replaced by the ground height, regardless of `keepOnGround`. `Cottage2–4` and `wateran1–9` have no model.
**Still broken / not tested:** The difference between −1 and −2 makes no difference on the ground; any other floor that answers the lookup is not traced.
**Next:** M2 study.

## 2026-10-11 Study session: Q13–Q16, two corrections

**Changed:** `docs/behaviour/objects.md` (Q13, Q14, Q16), `docs/behaviour/buildings.md` (rule 2 corrected), `docs/behaviour/models-and-textures.md` (flag 0x10000 corrected), `docs/behaviour/QUESTIONS.md`. No code.
**Why:** Session B's questions from M1 steps 3a/3b.
**Tested how:** Traced `loadKQ`, the save-game writer, the sequence lookup, the `translucency` reader and the model drawing code in `Mask.exe`. Read Daventry's `world.cs`, `newworld.cs` and `permwrld.cs`, and counted which `.kq` files they load. Counted plane sides in all `.dig` files.
**Result:** Only script-loaded objects exist. `loadKQ`'s values are name, x y z, then dirX dirY dirZ. A sequence name that isn't found falls back to sequence 0. `translucency` = opacity n/255.
**Corrections:** Buildings' front faces are clockwise, like models; my earlier rule was wrong and session B caught it by eye and by measurement. Flag 0x10000 halves the fog, not the opacity.
**Still broken / not tested:** The teleport pad's see-through look is unexplained. Whether skipped frames' steps apply on a jump is not traced.
**Next:** M2 study (slopes, collisions, jumping).

## 2026-10-11 Study session: material flags (Q5, partly)

**Changed:** `docs/behaviour/models-and-textures.md` (new section), `docs/behaviour/QUESTIONS.md`. No code.
**Why:** The implementation session asked what the unknown material bits do.
**Tested how:** Traced the model drawing code in `Mask.exe`. Counted colour indices in every texture named by the worlds' material lists.
**Result:** The flag word splits into material kind, lighting mode (none/flat/smooth), 0x1000 (forces no lighting), 0x2000, and 0x10000 (50% see-through).
**Still broken / not tested:** Which renderer options 0x1000 and 0x2000 switch on is not identified, and which colour (if any) is see-through is unknown. No texture uses colour index 0.
**Next:** M2 study (slopes, collisions, jumping), the rest of Q3/Q4.

## 2026-10-11 Study session: animated objects at load (Q12)

**Changed:** `docs/behaviour/objects.md` (new section), `docs/behaviour/QUESTIONS.md`. No code.
**Why:** The implementation session needs the right starting frame for doors and other animated objects.
**Tested how:** Traced the movement list reader, the `loop=` parser and the start-up code in `Mask.exe`; counted the `loop=` patterns in Daventry's `.kq` files.
**Result:** In `loop=`, a is the start position (0–1), b the speed (1, −1 or 0) and c whether it repeats. With no `currAnimName` the game uses movement 0. `[movementK]` is matched by number. `curAnimName` is not read.
**Still broken / not tested:** The 4th number (d) and `nextAnimName` are not traced.
**Next:** Q5 (material flags).

## 2026-10-11 Study session: buildings (Q11) and ground snapping (Q10)

**Changed:** `docs/behaviour/buildings.md` (new), `docs/behaviour/objects.md`
(Q10 follow-up), `docs/behaviour/QUESTIONS.md` (statuses). No code.
**Why:** The implementation session could not read the building files.
**Tested how:** `.dig` reader and `KQInterior` placement traced in
`Mask.exe`. Throwaway scripts in `re/` parsed every `.dig` in the install
(153 files) and checked the layout, indices, corner order and texture
mapping on Daventry's 30 buildings.
**Result:** `.dig` layout known well enough to draw buildings. Buildings at
`locZ` −1 are lifted onto the ground by `keepOnGround`, which is on by
default and only ever raises an object.
**Still broken / not tested:** That the ground lift runs when a building is
first added was not traced (the data agrees). `.dil` lighting, the BSP
tables and the two extra header values are not decoded. Texture mapping is
measured, not traced.
**Next:** Q12 (animation start frame), Q5 (material flags).

## 2026-10-11 Study session: answers to Q1, Q3, Q4; camera additions

**Changed:** `docs/behaviour/models-and-textures.md` (new),
`docs/behaviour/QUESTIONS.md` (statuses), `docs/behaviour/camera.md`
(swing back behind Connor; tilt reset after 10 units). No code.
**Why:** The implementation session asked these while drawing models (M1).
**Tested how:** Matrix convention traced in `Mask.exe`. Frame transforms
measured in the data: Connor's walk (steady part offsets, per-frame steps)
and doors (about 12–13° per frame, about 165° in total). Palettes and
archive order read from `world.cs` and `mask.cs`.
**Result:** Q1 answered (row vectors; frame transforms move the whole
object, not used for drawing; flags are hints). Q3 and Q4 partly answered.
**Still broken / not tested:** The code that applies frame steps was not
found in the exe; the conclusion rests on the data. Q2, Q5 and parts of
Q3/Q4 are still open.
**Next:** Q2, Q5, then terrain height, slopes and collisions.

## 2026-10-11 Study session: behaviour notes for Connor's movement and the camera

**Changed:** `docs/behaviour/connor-movement.md`, `docs/behaviour/camera.md`,
`docs/behaviour/QUESTIONS.md` (new). Outside git: Ghidra labels, a Ghidra
script that turns table-only code into functions
(`re/ghidra_scripts/CreateTableFunctions.java`: 531 candidates, 520
created, 11 failed), throwaway analysis scripts and scratch notes in `re/`.
No code in the repo.
**Why:** M2 (walk Connor around Daventry) needs both behaviours.
**Tested how:** Traced the behaviour in `Mask.exe` with Ghidra; measured
animation steps from `GAME/Common/C_walk.kqs` and `C_run.kqs`; checked the
camera controls against the game's manual (`MASK/Manual.pdf`).
**Result:** Walk ≈ 1,127 units/s and run ≈ 2,518 units/s, taken from the
animations' per-frame steps; turning 1.75 rad/s; banking into turns when
running, up to 0.15 rad. The camera trails Connor inside a min/max distance
range set by the walk/run scripts, stays at least 1,400 above the ground,
and has the mouse/Shift/F11/F2 controls. The manual agrees with what the
code does.
**Mistakes caught:** First drafts claimed things not yet traced (who uses
"turn to face target", when turning is disabled, a zoom formula). These
are corrected or marked as open questions in the notes. `Camera.sae` is
the real camera key map, not only a debug map as I first assumed.
**Still broken / not tested:** Nothing checked against the running game.
Open questions are listed at the end of each note.
**Next:** Terrain height, slopes and collisions.

## 2026-10-11 M1 step 3c: only script-loaded objects; translucency

**Changed:** `kq8-data`: new `world_script.rs` (follows `newWorld.cs` and
the scripts it runs, collects `loadKQ` lines), `objects.rs`
(`Placement::from_load`, `translucency`, `opacity`), 3 new tests.
`kq8-viewer`: objects now come from the world scripts only; objects with
`translucency` < 256 drawn at n/255; material flag 0x10000 no longer
see-through; unused `World::entries_ending` removed. Docs: `DESIGN.md`
(step 3c plan), provenance, `README.md`, question Q17. Session B, from
`objects.md` (Q13, Q14, Q16) and the corrections to `buildings.md` rule 2
and flag 0x10000.
**Why:** You passed on the study session's answers and corrections and
asked to place only what the world scripts load.
**Tested how:** `cargo test` (36 pass), `cargo clippy` (0 warnings),
`kq8-viewer --terrain --frames 120`, then read `viewer.log`.
**Result:** Scripts run: newWorld.cs → WorldVar.cs, WorldInv.cs,
permwrld.cs; 446 `loadKQ` lines, none unreadable, none inside if-blocks.
347 objects placed (was 170) with 189 models; 15 raised onto the ground;
9 see-through (waterfalls, water, well water, torches). Placement
self-check 0.035 units. Skipped: 72 without a model (traps, sounds,
triggers), 5 monsters (`.anm`), 19 without any position (e.g.
`Cottage2–4`, `wateran1–9`), 3 files not in Daventry's archives
(`terrain.kq`, drawn separately; `connor.kq`; `davmap.kq`).
**Problem found:** 79 objects (graveyard tombs, 44 trees) sit ~8,000 units
under the ground. Their `loadKQ` line says z = −1 and their template says
`keeponground=0`, so the current rules do not raise them. Not guessed:
question Q17.
**Checked by eye:** All furniture and objects now seem to be there. The
waterfall and the fountain water are see-through. Nothing out of place or
doubled at a quick glance. The trees and graveyard tombs are all
underground at about the same depth, as the log predicted (Q17).
**Still broken / not tested:** The 79 sunken objects (Q17); the teleport
pad (Q14 b); distance fade for see-through objects (needs haze); flag 8
"random start frame" on 10 objects is not applied.
**Next:** Q17.

## 2026-10-11 M1 step 3b: buildings

**Changed:** `kq8-data`: new `building.rs` (`.dig` reader, triangles,
corner-order normal), `pers.rs` (accepts a missing padding byte at the
end of the data; `u16`), 3 new tests. `kq8-cli`: new `building_verify.rs`
(`kq8 building-verify`). `kq8-viewer/load.rs`: `load_building`;
`KQInterior` objects now drawn as buildings, with the interior palette.
Docs: `docs/DESIGN.md`, `docs/provenance/README.md`, `README.md`. Session
B, from behaviour note `docs/behaviour/buildings.md` (Q11, Q10 follow-up).
**Why:** M1 step 3b: Daventry's buildings.
**Tested how:** `cargo test` (33 pass), `cargo clippy` (0 warnings),
`kq8 building-verify`, `kq8 shape-verify` (unchanged),
`kq8-viewer --terrain --frames 120`, then read `viewer.log`.
**Result:** All 153 `.dig` files read, 0 failures. 172,780 of 172,783
surfaces have the expected corner order. Viewer: 30 of 33 Daventry
buildings drawn (30 different models, 0 missing textures), 19 raised
onto the ground. 170 objects in total. Average frame 6.1 ms.
**First run failed:** 61 of 153 files would not read. My PERS reader
expected a padding byte that these files do not have at the very end.
The study note says so ("ends at block size + 8"); fixed.
**Mistake caught:** A test line I wrote was garbled again (second time).
Rewritten before running.
**Found by eye, then fixed:** You saw the buildings in the right places
but every wall inside-out. I had followed the note's rule 2
(anticlockwise = front). Measured with `building-verify`: on the outer
sides of all 153 buildings' boxes, 99.7% of ~23,000 surfaces are
clockwise seen from outside (roofs 6,069 vs 0). So buildings now use
clockwise fronts, like models. The note's floor/ceiling numbers are the
same finding read as inside views. Correction sent to the study session
as Q15.
**Checked by eye (after the fix):** Walls and roofs look right from
outside, insides look right, and textures are placed correctly.
**Still broken / not tested:** `ruins1/3/4.dig` do not exist in the game
files (skipped). No building lighting, so buildings are evenly bright.
Waterfalls/streams still solid (Q14). No monsters. **Objects missing
from inside houses** (you noticed): the 106 objects at (0, 0, 0) that I
skipped as "templates" are placed by Daventry's world script `permwrld.cs`
(`loadKQ <file> <name> x y z a b c`, 180 such lines), which we do not
read. How `loadKQ` works is behaviour: question Q16.
**Next:** M1's goal is reached (Daventry's ground, objects and
buildings). Remaining M1 polish waits on Q13/Q14; then M2.

## 2026-10-11 M1 step 3a fixes: ground rule, start frames, see-through

**Changed:** `kq8-data/objects.rs`: `keepOnGround` and the ground rule,
`current_loop()` (all six `loop=` values, movement 0 when there is no
usable `currAnimName`), replacing `current_loop_shape`. `kq8-data/shape.rs`:
`start_sequence()` (start frame). 3 new tests, 1 updated. `kq8-viewer`:
objects use their start frame and the ground rule; materials with flag
0x10000 drawn at 50% opacity; more log lines. Docs:
`docs/provenance/README.md`, `README.md`, new question Q13. Session B,
from behaviour notes `objects.md` (Q12), `buildings.md` (Q10 follow-up),
`models-and-textures.md` (Q5 partly).
**Why:** The study session answered Q12 and part of Q5, and refined Q10.
**Tested how:** `cargo test` (30 pass), `cargo clippy` (0 warnings),
`kq8-viewer --terrain --frames 120`, then read `viewer.log`.
**Result:** 140 objects placed (was 104), 120 models. 35 animated objects
now start in movement 0 (the sign, bell, baskets...). Raised onto the
ground: 0 (the mill wheel and the keg say `keepOnGround=0`). Only 2 start
past frame 0: `Pyramid` (frame 63/70) and `telepad1` (frame 5/16).
`Pyramid`'s earlier frames move it; those steps are not applied (Q13 a).
20 `loop=` lines name a sequence the model does not have; the first loop
is used (Q13 b). 2 materials drawn see-through. Placement self-check
still 0.035 units.
**Problem met:** Your viewer window was open, so Windows would not let
the build replace `kq8-viewer.exe`. I did not close it. I built a second
copy in `target/side/` (git ignores it) and tested with that.
**Checked by eye:** The alchemist's sign is right, including its tilt in
the starting frame. The bell and the baskets look right. Doors are closed.
**Found by eye:** The waterfalls and the teleport pad should be
see-through but are solid. The cause is in the data: their materials do
not have flag 0x10000. The waterfalls' object files say
`translucency=200` (22 Daventry objects have such a key, values 50–256
across the game). The teleport pad has no such key. Meaning not known, so
not guessed: question Q14.
**Still broken / not tested:** Waterfalls, streams, teleport pad etc.
drawn solid (Q14). Buildings still missing (step 3b).
**Next:** Step 3b (buildings).

## 2026-10-11 M1 step 3a: place Daventry's objects

**Changed:** `kq8-data`: new `objects.rs` (placements, rotation, the
current animation's model), 4 new tests. `kq8-viewer`: `--terrain` now
also draws objects (`load.rs`: `load_objects`, `World::entries_ending`,
palette lookup for any `KQWorld::...Palette`; `terrain_view.rs`:
`setup_objects`, `bevy_transform`; `log.rs`: quiet mode; `main.rs`).
Also corrected the fly-speed comment after your speed change. Docs:
`docs/DESIGN.md`, `docs/provenance/README.md`, `README.md`, new
question Q12. Session B, from behaviour note `docs/behaviour/objects.md`
(Q9, Q10) and the data.
**Why:** M1 step 3a: see Daventry's objects in place.
**Tested how:** `cargo test` (27 pass), `cargo clippy` (0 warnings),
`kq8-viewer --terrain --frames 120`, then read `viewer.log`.
**Result:** 355 sections with a position: 104 placed (74 `KQObject`, 17
doors, 8 animated objects, 3 chests, 2 portals) using 90 models. Bevy's
placement equals the game's rule to 0.035 units. 21 rest on the ground
(±50), 81 are higher, 2 lower (mill wheel 601 below, keg 422 below).
Skipped: 91 at (0, 0, 0), 58 with no model, 51 animated without
`currAnimName` (Q12), 33 buildings (step 3b), 15 monsters (`.anm`), 2 whose
first frame is empty (`map1.kqs`, `Ug_Cut5b.kqs`), 1 with `shapeName=stop`.
Loading 224 ms; first frame after ~2.4 s; average frame 6.0 ms.
**Not in the plan, added:** The plan only covered `shapeName`, but only
`KQObject` sections have one. Doors, chests and so on name their model
through their animation list. That is plain file layout, so I added it.
Which frame to show is not in the data, so they show their first frame
(Q12).
**Checked by eye:** You flew around: many objects in place, including
doors, paintings and the mill wheel. The keg stands where the pub should
be. Inside an expected house, items float where a table should be: the
table is probably part of the building (step 3b). Most doors look closed,
but you are not sure. So "closed" is not confirmed (Q12).
**Still broken / not tested:** No buildings, so Daventry still looks
mostly empty. Also missing: monsters, emitters, the 51 animated objects
without `currAnimName`.
**Next:** Step 3b (buildings) needs Q11; door frames need Q12.

## 2026-10-11 M1 step 2b: draw Daventry's ground with textures

**Changed:** `kq8-data/src/terrain.rs`: `ground_triangles()` and
`texture_coord()` (orientation codes 0–7), 4 new tests. `kq8-viewer`:
`--terrain` mode (new `terrain_view.rs`: ground scene, flying camera,
camera log; `load.rs`: `load_ground`, terrain palette from `world.cs`;
`main.rs`: shared mesh/material helpers, mode switch). Docs:
`docs/DESIGN.md`, `docs/provenance/README.md`, `README.md`. Session B,
from behaviour notes `terrain.md` (Q6–Q8) and `models-and-textures.md`
(Q2, Q3).
**Why:** M1 step 2b: see Daventry's ground.
**Tested how:** `cargo test` (23 pass), `cargo clippy` (0 warnings),
`kq8-viewer --terrain --frames 120`, then read `viewer.log`. Also reran
`kq8-viewer smlchest.KQS` to check the model view did not change.
**Result:** All 16,384 squares (32,768 triangles) drawn with 30 textures,
all found in Daventry's archives (128×128, mipmapped). Palette
`DAVENTRY.PPL`, as named by `world.cs`. Loading 27 ms, average frame 6.0 ms.
Camera starts at game (262144, 262144, 16750), 4,000 units above the
ground. Chest unchanged (same bounds, 30 of 32 faces outward).
**Mistake caught:** A shell step dropped a backslash in `load.rs` (build
error), and one test line I wrote was garbled. Both fixed before testing.
**Checked by eye:** You flew over it: "everything looks perfect". The land
shape and textures are right, and textures join up between squares.
**Still broken / not tested:** No lighting, haze, shadow maps, holes or
`terrh2.dib`. No buildings or objects. Mipmaps not used. Only Daventry
looked at; other worlds not tried in the viewer.
**Next:** Step 3, objects and buildings.

## 2026-10-11 Fix: hide the back sides of model triangles

**Changed:** `crates/kq8-viewer/src/main.rs`: each triangle's corners
are given to Bevy in reverse order, and both-sided drawing is switched
off (Bevy's default hides back faces). The viewer logs how many triangles
face away from the model's centre. `docs/provenance/README.md`,
`docs/DESIGN.md`, `README.md`. Session B, from behaviour note
`docs/behaviour/models-and-textures.md` (Q2).
**Why:** The original hides back faces, and its front faces have clockwise
corners. Bevy's front faces are counter-clockwise, hence the reversal.
**Tested how:** `kq8-viewer` with `smlchest.KQS`, `broom.KQS`,
`alcmsign.KQS` (`--frames 30` each), read the log. `cargo clippy`.
**Result:** Chest: 30 of 32 triangles face outward (the old order would
give 2 of 32). Broom 12 of 22, sign 49 of 66. Those two are not box-shaped,
so this test says little about them.
**Checked by eye:** You turned the chest and the sign all the way around:
both look good (nothing missing, no insides showing).
**Still broken / not tested:** Only these two models checked by eye.
**Next:** Step 2b.

## 2026-10-11 M1 step 2a: read the terrain, height lookup

**Changed:** `kq8-data`: new `kq_text.rs` (reader for `[section]` /
`key=value` files), `terrain.rs` (settings, height/material grids,
direction table, height lookup). `kq8-cli`: new `terrain_verify.rs`
(`kq8 terrain-verify`). Docs: `docs/DESIGN.md` (2a done, 2b plan),
`docs/provenance/README.md`, `README.md`. Session B, from behaviour note
`docs/behaviour/terrain.md` (Q6, Q7) and the data files.
**Why:** M1 step 2a: the ground has to load before it can be drawn.
**Tested how:** `cargo test` (19 pass, 5 new: row flip, corners, both
diagonals, slope direction, text files), `cargo clippy` (0 warnings),
`kq8 terrain-verify` on all 12 worlds.
**Result:** 17 terrain files read, 0 failures, 0.1 s. Objects exactly on
our ground (±0.5 units): Daventry 11 (heights 6,630–30,600), Barren 11
(92 within 5), Ice World 11, Dead City 7, Snow exit 2. Daventry's height
step is exactly 255. Many objects stand exactly +5 or +25 above the ground.
That explains the note's 256.2.
**First run failed:** 4 "failures" were the temple worlds' `terrain2.kq`
naming `ice.dml`, which those worlds lack. A fact about the data, so it is
now a note, not a failure.
**Still broken / not tested:** Gnome: 0 objects on the ground (median
17,450 units off; probably caves, not studied). Temple grounds are flat at 0.
Holes, `terrh2.dib` (differs from `terrh.dib` at 120 Daventry points, all
1–5 steps lower) and texture mapping (Q8) open. Nothing drawn yet.
**Next:** Step 2b, draw the ground (untextured until Q8), after your go-ahead.

## 2026-10-11 M1 step 2 planned; terrain size and layout blocked on questions

**Changed:** Docs only: `docs/DESIGN.md` (M1 step 2 plan; roadmap
corrected: `.dig`/`.dil` are buildings, not terrain),
`docs/behaviour/QUESTIONS.md` (Q6–Q8). No code.
**Why:** Start of M1 step 2. You asked me not to guess terrain scale,
height lookup or texture mapping.
**Tested how:** Read `TERRAIN.KQ` of every world and the Daventry terrain
files; measured them with throwaway scripts kept outside the repo.
**Result:** File layouts are clear: heights 129×129 (0–254), materials
128×128, `daventry.bin` = 4 + 255 × 28 bytes exactly, names match
`davennew.dml` 255 of 255. Units and orientation are **not** clear.
**Dead end:** Two attempts to find the scale from object positions.
(1) Straight-line fit of object height against the height image, 4,096-unit
squares, 4 image orientations: best R² = 0.24 (weak). (2) Scan of 12 square
sizes (1,024–8,192), height steps 80–200 per grey value, 4 orientations,
counting objects within 150 units of the surface: best 23 of 247, with
several quite different settings tied, which is noise. Most objects
are not on open ground (inside buildings, on tables, flying). Stopped
there and asked the study session instead (Q6, Q7).
**Odd finding:** The most used material number (159, on 93% of squares)
is `stone1.bmp`. That suggests the material image is not a plain index,
or I misread it (Q8).
**Still broken / not tested:** Nothing built yet. Step 2a (reading the
files) does not depend on the answers; step 2b (drawing) does.
**Next:** Your choice: build step 2a now, or wait for Q6–Q8.

## 2026-10-11 Fix: frame transforms are not used to draw model parts

**Changed:** `kq8-data/src/shape.rs`: `Shape::geometry` no longer applies
the sequence's frame transform, only each part's mesh node transform; doc
comments and the unit test updated (the test now gives frame 0 a 50-unit
step and checks it is ignored). `kq8-cli/src/shape_verify.rs`: the matrix
test no longer transposes frame transforms (they are not drawn).
`docs/provenance/README.md`. Session B, from behaviour note
`docs/behaviour/models-and-textures.md` (Q1 b).
**Why:** The study session found frame transforms are the step the whole
object makes during an animation (Connor's walk steps ~70–81 units per
frame; doors turn ~11–13° per frame). Drawing them would make Connor
lurch and doors never open past 13°.
**Tested how:** `cargo test` (14 pass), `cargo clippy` (0 warnings),
`kq8 shape-verify` before and after, compared line by line;
`kq8-viewer smlchest.KQS --frames 60`.
**Result:** No difference in any number: all 4,573 rest poses identical,
because frame 0 always has a zero step. The chest looks the same
(same bounds in the viewer log).
**Still broken / not tested:** Nothing visible changes until animation is
added (M2/M3); that is when the frame steps will be used, to move objects.
**Next:** M1 step 2 (terrain), after your go-ahead.

## 2026-10-11 M1 step 1b checked by eye

**Changed:** Docs only (`README.md`, `docs/provenance/README.md`). No code.
**Why:** You looked at the viewer.
**Tested how:** You ran `kq8-viewer smlchest.KQS` and `alcmsign.KQS`. I
then measured the sign's tilt in each animation frame with a throwaway
program kept outside the repo (scratchpad).
**Result:** Chest correct: lid closed, wood-and-metal texture. The sign
looks right but tilted. Measured: its animation is 31 frames over 2.0 s.
The tilt goes 10° (frame 0) → 0° (6) → 10° (15) → 0° (23), a swing. The
viewer shows frame 0, so the tilt is the start of the swing, as you
remembered. This also supports the row-vector matrix choice (Q1).
**Still broken / not tested:** Whether models are mirrored is not ruled
out (Q2). Neither model was confirmed to have readable lettering. No
animation yet. Q1–Q5 still open.
**Next:** M1 step 2, terrain.

## 2026-10-11 M1 step 1b: show one textured model in a Bevy window

**Changed:** New crate `crates/kq8-viewer` (`main.rs` Bevy window and
orbit camera, `load.rs` model/texture/palette lookup, `log.rs`). Added
Bevy 0.20.0 (only its 3D features) to the workspace. `Cargo.toml`:
`default-members`, so plain `cargo run -- <command>` still runs `kq8`.
Docs: `README.md`, `CREDITS.md`, `docs/DESIGN.md` (layout),
`docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md`.
**Why:** Second part of M1 step 1: see a game model on screen.
**Tested how:** `cargo run --release -p kq8-viewer -- smlchest.KQS --frames 60`
(also `--frames 120`, and `broom.KQS`, `alchy.KQS` with `--frames 30`),
then read `kq8-artifacts/logs/viewer.log`. `cargo test`, `cargo clippy`.
**Result:** Window opens and draws on the RTX 4060 Ti (Vulkan). Chest:
32 triangles, 1 texture (`ITEMS22.BMP` 128×128), first frame after
~0.5–1.1 s, average 6.4 ms per frame. For all 4 textures checked, the
image's own colour table equals `DAVENTRY.PPL` for every colour used.
**Mistake caught:** Adding a second program to the workspace broke
`cargo run -- vol-verify` (cargo could not tell which program to run).
Fixed with `default-members` before handing over.
**Note:** While I worked, another session replaced `QUESTIONS.md` with a
table template, which removed my Q1–Q5. I re-added them as table rows and
did not change anything else in that file or in `connor-movement.md`.
**Still broken / not tested:** **Nobody has looked at the window yet.**
Not known: upside-down textures, a mirror image, parts in wrong places.
No lighting, no animation, no transparency, both sides of triangles drawn.
**Next:** You look at the chest (procedure in the hand-over message). Then
M1 step 2, terrain.

## 2026-10-11 M1 step 1a: read 3D models (.kqs) and material lists (.dml)

**Changed:** `kq8-data`: new `pers.rs` (the PERS wrapper + a byte reader),
`material.rs` (`TS::MaterialList`), `shape.rs` (`KQ8::Shape`,
`TS::CelAnimMesh`, building the triangles of one animation frame).
`kq8-cli`: new `shape_verify.rs` (`kq8 shape-verify`). Docs:
`docs/DESIGN.md` (M1 step 1 plan), `docs/provenance/README.md`, new
`docs/behaviour/QUESTIONS.md` (Q1–Q5). Session B; no behaviour note exists
for this, so it is built from the data files and the libkq8fpc notes.
**Why:** First part of M1 step 1: models must load before they can be shown.
**Tested how:** `cargo test` (14 tests, 4 new). `kq8 shape-verify` on the
whole game.
**Result:** 4,573 models and 25 material lists parse, 0 failures, 0 unread
bytes, 3.4 s. 23,153 meshes, 2,397,317 faces, all indices in range. Z is
up (tallest axis in 2,950 models; 3,177 stand on z ≈ 0). Matrix test
favours row-vector (245 models clearly better vs 7).
**Dead end / correction:** The community note says material flags are 1
byte. They are 4 bytes. File sizes only add up that way.
**Still broken / not tested:** 4 `TS::Shape` tomb files skipped.
Matrix convention, front faces, texture palette, texture search order and
3 flag bits are open questions (QUESTIONS.md). 29 texture names are
missing from the game files. Nothing drawn yet.
**Next:** Step 1b, show one model in a Bevy window.

## 2026-10-11 Ghidra connected, two-session workflow, roadmap

**Changed:** `docs/SESSIONS.md` (new: rules and prompts for the study and
implementation sessions, behaviour-note format), `docs/DESIGN.md` (links
to it; the four-step plan replaced by a milestone roadmap M0–M7),
`.gitignore` (`/re/`). Outside git: `re/` holds a copy of `Mask.exe`, the
Ghidra project and study scratch notes; `.mcp.json` configures the
GhidraMCP bridge (pinned to release 2.0). No code changed.
**Why:** You decided to study `Mask.exe` and to keep the two-session method
from ai-game-modding-guides guide 13. Behaviour of object kinds, movement,
combat and camera lives in the exe, not the data.
**Tested how:** Called the Ghidra tools from this chat: program info,
class list, string search, cross-references.
**Result:** Connected to `re/Mask.exe` (sha256 matches the game's exe),
6,413 functions. Ghidra recovered only 2 classes although ~399 C++ class
names are in the file; its class detection apparently does not understand
this 1998 compiler's layout. The names are still usable by following
references by hand.
**Still broken / not tested:** Class structure not recovered yet. `/mcp`
toggle for the implementation session not tested.
**Next:** Your choice of graphics library, then M1 (see Daventry).

## 2026-10-11 Fix: wrong palette for the Daventry image

**Changed:** Docs only (`README.md`, `docs/provenance/README.md`). No code.
**Why:** You looked at `daventry.png` made with `daventry.ppl`: very dark,
speckled, the paper background wrong. It should be a pencil drawing on old
paper.
**Tested how:** Searched the game scripts (`mask.cs`, `world.cs`) for
palette names, then converted `daventry.pbm` with every `.ppl` in `GAME/`
and `data/Daventry/` and compared how much colour changes between
neighbouring pixels.
**Result:** All 16 loading palettes `GAME/8Gui/l*.ppl` have identical
colours and score 18.4, against 44.5 for `daventry.ppl` and 107.5 for
`Menus.ppl`. The image's most common colour (index 11, 44,125 pixels) is
near-black (8,0,0) in `daventry.ppl` and brown-grey (128,112,96) in the
loading palette, which explains the dark picture.
**Mistake:** In the previous entry I called `daventry.ppl` "the right one".
My measurement only showed it was better than `Menus.ppl`, not that it was
correct. I should have tried every palette before concluding. My first file
search also stopped after 40 entries and missed the `l*.ppl` files.
**Still broken / not tested:** You checked the new PNG: colours correct.
Which of the 16 identical loading palettes the engine loads is unknown
(it does not matter for colours).
**Next:** Step 3 (`.msg`).

## 2026-10-11 Step 2: read images and palettes, export PNG

**Changed:** `kq8-data`: new `chunks.rs`, `bitmap.rs`, `palette.rs`.
`kq8-cli`: new `source.rs` (load a loose file or `vol:` archive entry),
`bitmap_png.rs` (`kq8 bitmap-png`), `bitmap_verify.rs` (`kq8 bitmap-verify`);
`main.rs` takes the new commands. Added the `png` crate. Updated
`docs/provenance/README.md`, `docs/DESIGN.md`, `CREDITS.md`, `README.md`.
**Why:** Step 2 of the plan: first real asset format on top of the archives.
**Tested how:** `cargo test` (9 library tests: chunked 8-bit, mipmaps,
24-bit colour order, Windows BMP, both palette kinds). `kq8 bitmap-verify`
on the whole game. `kq8 bitmap-png` on `GAME/8Gui/daventry.pbm`.
**Result:** bitmap-verify: 6,495 images and 50 palettes, 0 failures,
1.7 s. Variants found: 1,909 `.pbm` + 1,173 mipmapped archive `.bmp`
(chunked), 3,382 Windows-BMP `.bmp` (8 of them 24-bit), 31 `.dib`, 44 `.ppl`,
6 RIFF `.pal`. bitmap-png: 640×480, 154 colours used.
**Dead end recorded:** I first paired `daventry.pbm` with `Menus.ppl` (the
menu palette locked in `mask.cs`). A measurement showed it was wrong:
average neighbouring-pixel colour change was 107.5, against 44.5 with
`Daventry/resource/daventry.ppl`, and 14–68 for images that carry their own
palette. The world's own palette is the right one. (Wrong: see "Fix: wrong palette" above.)
**Still broken / not tested:** Nobody has looked at the PNG yet (you need
to). Transparency, palette lighting/haze tables, `.pba` image arrays and
`.pal` with 237 colours are not handled. The forced system colours at both
ends of the palette come from the GPL notes and are not verified against
`Mask.exe`. No real 24-bit chunked image exists to test that path.
**Next:** Step 3, dialogue text (`.msg`).

## 2026-10-11 Read the modding guides, adopt their templates and working method

**Changed:** Cloned `ai-game-modding-guides` (commit `0c14be4`) and
`GameDecompLibrary` (commit `0e1357d`) into `reference/`, which is never
committed (added to `.gitignore`). Copied `MODLOG-template.md` and
`STATUS-handoff.md` to the project root (AGENTS.md refers to them).
Added `docs/provenance/README.md`, a "working method" section in
`docs/DESIGN.md`, new findings in `docs/RESEARCH.md` (versions, ScummVM,
GameDecompLibrary, EULA), and an attribution-template layout in `CREDITS.md`.
README now says it's an unofficial fan project and mentions AI use.
Reformatted this log to the template.
**Why:** You asked me to follow the guides. They recommend recording
provenance, splitting disassembly work across two sessions, and the
templates AGENTS.md expects.
**Tested how:** `git status --short -uall` to check only our own files show
up, and `git check-ignore` on `reference/`.
**Result:** Only project files are listed; `reference/`, `MASK/`, `target/`
and `kq8-artifacts/` are ignored. No code changed, so `vol-verify` was not
rerun.
**Still broken / not tested:** Nothing new to test. One mistake caught:
I first wrote an unverified claim in the provenance file (that all VOL
streams start with `00 04/05/06`). Corrected it to what was actually observed.
**Next:** Step 2 of the plan, a `.pbm` bitmap + `.ppl` palette dumped to PNG.

## 2026-10-11 Step 1: read VOL archives, prove the data loads

**Changed:** Whitelist `.gitignore`; Cargo workspace with `kq8-data`
(`VolArchive`, `GameDir`) and `kq8-cli` (`kq8 vol-verify`); `docs/DESIGN.md`,
`docs/RESEARCH.md`, `CREDITS.md`.
**Why:** The smallest proof that we can read the original data. Every other
format lives inside these archives.
**Tested how:** `cargo test` (3 unit tests on a hand-built archive), then
`cargo run --release -- vol-verify` on the GOG 1.03 install.
**Result:** 48 archives, 14,444 entries, 382,267,214 bytes unpacked, all
14,444 byte-identical to the installer's unpacked copies, 0 errors, 0 size
slack, ~8.5 s. `RESULT: PASS`.
**Still broken / not tested:** The reference copies were unpacked by
`kqmoevol`, not by `Mask.exe`, so this proves agreement with kqmoevol, not a
direct comparison with the original game. Not tested on Linux/macOS.
**Next:** Bitmaps and palettes.
