# MODLOG

What changed and what was tested. Newest entries at the top.
Format: date and title, then Changed / Why / Tested how / Result / Still broken / Next.

## 2026-10-17 Study session A: Q84–Q93 answered (docs only)

**Changed:** `docs/behaviour/`: `connor-physics.md` (Jumping rule 5, Walls, "Against a building" step 7), `anim-scripts.md` (rules 3, 12, 16, 19, new 20–27: `Die`'s `$3`, `handsOff`, `walkTo`/`runTo`/`pointAndCue`, the watermill cutscene), `pushing.md` (rules 2.5, 4.1–4.3, 11, edge cases, checks), `camera.md` (`follow none`, `move KQCamera` angles, `follow Connor` after a cutscene, `setThirdPerson`, tilt sign), `objects.md` (`loadKQ` while playing, `move` heights, `setHidden`), `connor-movement.md`, `sound.md` rule 46, `clicking.md` rule 22, `doors.md`; `QUESTIONS.md` Q84–Q93 marked answered; STATUS. Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** The user asked A to answer the questions B raised in M3 steps 3i–3k.
**Tested how:** Read in Ghidra (`Mask.exe`, through the Ghidra plugin's HTTP interface) and checked against the data (`wtrmwalk.cs`, `bricks.cs`, `.kq` files, `c_brks1.kqs`). Nothing was run in the original game.
**Result:** All ten answered. Corrections to earlier notes: the level-up bonuses (second stat +5, third +2, not the other way); a non-head-on jump hit doesn't end the jump; turning works during `crash`; barrels are never stopped by walls or objects and only raised by the ground; the bit-8 random start is lost on objects without `currAnimName`. B's stand-ins judged wrong: Q85 a/b, Q86 e, Q87 a–c, Q89 c (view not set), Q90 b (no `Init`). Likely causes for the user's three problems: the pop-up (Q92: the `Push` script runs inside Connor's update in the original), the watermill (Q93: `walkTo`/`pointAndCue` missing), the barrels (Q91).
**Still broken / not tested:** Not checked in the original: barrels pushed downhill (should float) or into a wall (should go in), the hay bale at an angle (should slide), Connor turning during `crash`, the drop after the cue-10 `move`. Not settled: which of Connor's movements set the engine's input lock; whether GUI inventory/menu keys are blocked by `handsOff`; whether far objects update (Q85 in-step torches); the lean during a run crash.
**Next:** B builds the fixes: `Push` script inside Connor's update (or finish his down pass after a `move`), `walkTo`/`pointAndCue`, `handsOff` details, `move KQCamera` angles, box pushables, the Q84 turning.

## 2026-10-17 M3 step 3k: the user's checks by eye (docs only)

**Changed:** `docs/behaviour/QUESTIONS.md`: Q91–Q93 from the user's tests. README "what doesn't work" lines. No code.
**Why:** The user tested steps 3i–3k by eye and reported what matches the original and what doesn't.
**Tested how:** By the user, in the viewer: the three runs listed under 3k-4 and free play.
**Result:** Matches the original: the bricks puzzle runs end to end and the way behind the waterfall opens (the user walked through); the hay bale pushes and stops at walls; the star globe opens and shows the potion. The potion can't be picked up while `--item INVITEM_SpellPage` is held (other potions in that room can't either; without `--item` they can), so this looks like the missing inventory, not this step.
**Still broken:** (1) The camera's place in the bricks cutscene differs from the original (Q89). (2) Right after the push, Connor, the bricks and the table briefly rise a few units and fall back before the animations play (Q92; not logged). (3) The watermill room's cutscene takes the camera (`handsOff`) and never gives it back (Q93). (4) Barrels play the push animation but don't move; in the original they move (Q87, Q91).
**Next:** Study session A answers Q85–Q93; then fix (2)–(4).

## 2026-10-17 M3 step 3k-4: `loadKQ` while playing (the new waterfall)

**Changed:** `kq8-core/src/active.rs`: `loadKQ <file>` is a world command (no longer a stub): a copy prepared at the start comes into the world (`Request::Load`); its name becomes known to the scripts; a line with a name or position, or a file with no prepared copy, is logged as not done; `set_dormant`; 1 new unit test. `sound.rs`: `Audible::dormant` (silent until loaded). `kq8-viewer`: `load.rs` prepares a "dormant" copy of every file a world script names in `loadKQ <file>` that the start-up scripts don't load and that has a place of its own (the H-key copies; sound objects are now prepared too, silent); `anim_view.rs` `bring_in` hands such a copy's player to `Active`; `sound_view.rs` `wake`; `door_view.rs` new system `apply_loads` (after `trap_step`) shows the copy's parts, brings in its animation and switches its sound on; `main.rs` schedules it. README and FEATURES updated; provenance added.
**Why:** The user's task, step 4: `bricks.cs` replaces the waterfall (`loadKQ waterfa2.kq`, `loadKQ watrfsn2.kq`), which opens the way behind it.
**Tested how:** `cargo test --workspace` (370 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 7000 --sound off`, `viewer.log`, `doors.log`, `traps.log`, `sound.log`.
**Result:** `viewer.log`: 17 dormant copies in Daventry (including `waterfall2`, `Waterfall2Sound`, the potion, the unicorn, the lady of the lake). Update 63: "loadKQ waterfa2.kq: waterfall2 comes into the world", same for the sound; update 64: "waterfall2 came in: 1 drawn part(s) shown, animation played by the world's rules", "Waterfall2Sound came in: sound object on". `sound.log` frame 665: `watrfall.aud` stopped ("WaterfallSound deleted"), `watrfal2.aud` started (looping, placed, max 30,000). The old waterfall's solid box (type 0x21) was switched off at update 64; the new one (type 0x20) is not solid.
**Still broken / not tested:** Not checked by eye or by ear; walking through where the waterfall stood was not tried in a run (it needs the puzzle first, and there is no way yet to move the test Connor mid-run). A copy that comes in is not clickable (`waterfa2.cs` only makes Connor remark). `loadKQ` with a name or position while playing (barrels, boards, `KQAnimOnce`) is not done. Q90 open.
**Next:** The user's checks by eye (below). Then whatever the study session answers for Q85–Q90.

## 2026-10-17 M3 step 3k-3: the camera and controls in a cutscene

**Changed:** `kq8-core/src/active.rs`: `KQCamera::follow Connor|none` (`camera.md` rule 1) and `handsOff 1|0` are world commands (no longer stubs) that hand requests to the viewer; following another object is logged as not done. `kq8-viewer`: `game_camera.rs`: `following` (when off, the camera stays where it is: no update), `move_to` for `move KQCamera` (position set or added; the view keeps looking at its last aim point: Q89 c stand-in), `follow`; `connor_view.rs`: `hands_off` (movement keys and Jump count as let go; Q89 a stand-in); `door_view.rs`: applies the requests and ignores clicks while hands are off. `setThirdPerson` stays a logged stub (Q89 b stand-in: we are always in third person).
**Why:** The user's task, step 4: the bricks' cutscene.
**Tested how:** `cargo test --workspace` (369 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 7000 --sound off` (forward held for 200 updates, through the whole cutscene), `traps.log`, `connor.log`.
**Result:** Update 13: handsOff on, the camera stops following, moves to (158986, 302415, 15926); update 64: to (158986, 293915, 16236); update 124: handsOff off, to (159236, 302565, 16276), follows Connor again. Connor stands from 64 to 124 although forward is held; at 125 he walks again and at 137 stops at the table (the bricks are no longer pushable).
**Still broken / not tested:** Not checked by eye. The camera's view angles from `move KQCamera` are not used (Q89 c): it looks at Connor's last aim point, which is probably not what the original shows. `handsOff`'s exact reach is a stand-in (Q89 a). `KQCamera::follow <other object>` not done.
**Next:** 3k-4, `loadKQ` while playing (the new waterfall and its sound).

## 2026-10-17 M3 step 3k-2: `setHidden` on plain placed objects

**Changed:** `kq8-core/src/active.rs`: `KQObject::setHidden <name> 0|1` on any other placed object (not a scripted object, chest or item) is remembered (`hidden_now`: a script's value wins over the file's `hidden=`); unknown names give an error line. `kq8-viewer` `draw_distance.rs`: the draw-distance test asks `hidden_now`, so a hidden object is "off" (not drawn, its collider out of the collision world: `objects.md` "Hidden objects" rule 3, `object-lighting.md` rule 41), and shown again by `setHidden 0`. 1 new unit test, 1 updated (it expected the old stub line).
**Why:** The user's task, step 4: `bricks.cs` hides and shows `Table` while Connor pushes.
**Tested how:** `cargo test --workspace` (369 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,60 --frames 6000 --sound off`, `doors.log`, `viewer.log`.
**Result:** Update 13 "setHidden Table: placed object hidden true", and the draw-distance test: "[Table] Off … collider out (1 box(es) for Connor)"; update 63 hidden false, update 64 "[Table] Near … collider back".
**Still broken / not tested:** Not checked by eye. An object hidden in its own file (`hidden=1`) that a script shows would stay invisible (its drawn parts start hidden for the H key); none in the bricks path.
**Next:** 3k-3, the camera and controls (`KQCamera::follow`, `move KQCamera`, `setThirdPerson`, `handsOff`).

## 2026-10-17 M3 step 3k-1: `move` while playing

**Changed:** `kq8-core/src/active.rs`: `move <who> <flags> x y z [rx ry rz]` is a world command (no longer a stub): flags 0–3 as `objects.md` "(d)" (relative position along the world axes, Q21; relative turns added only about Z); doors, chests, scripted and animated objects are moved at once (their placement); Connor, `KQCamera` and plain placed objects become a `Request::Move`; unknown names and flags above 3 are logged and not moved; `moved_placement`; 1 new unit test. `kq8-data` `world_script.rs`: `parse_move` public. `kq8-viewer`: `connor_view.rs` `move_to` (position and heading set or added, no collision passes, no blending from the old place), `door_view.rs` applies the requests (plain objects: colliders and drawn parts; the camera waits for 3k-3).
**Why:** The user's task, step 4: the bricks script moves Connor (twice) and the bricks.
**Tested how:** `cargo test --workspace` (368 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,60 --frames 6000 --sound off`, `doors.log`, `connor.log`.
**Result:** Update 13 (the `Push` call): "moved Connor from (157007, 303799, 14837) heading 1.570 to (156652, 303600, 15000)"; he drops onto the cave floor (14837) and plays `bricks.anm/push` there. Update 63: "move Bricks 3: (156652, 303600, 14862) -> (156652, 303400, 14562)". Update 123/124 (cue 10): Connor moved to the same spot again. The three `move KQCamera` lines are handed over and logged "not done yet (M3 3k-3)".
**Still broken / not tested:** Not checked by eye. A plain placed object's drawn turn isn't changed by `move` (position only; logged; none in the bricks path). Whether the ground rule applies after a `move` is not traced (the note); Connor simply falls onto the floor by gravity.
**Next:** 3k-2, `setHidden` on plain placed objects (the bricks' `Table`).

## 2026-10-17 M3 step 3j-3: scripted pushables (the bricks' `Push`)

**Changed:** `kq8-core/src/connor_mover.rs`: an accepted touch on a scripted pushable is not a hit: the object is passed through for the rest of that update's move (`MoverWorld::pass_through`) and the touch is reported; objects whose script didn't take control become walls (`Scene::not_pushable`); on an exact tie between two contacts the pushable's is taken first (Q88 stand-in). `active.rs`: `push_call` runs the object's script with `Push` and says whether it is still a pushable afterwards; `collision_type_of`; 1 new unit test (and the scripted case of the mover test now checks he goes on through). `kq8-viewer`: `connor_view.rs` collects the scripted pushes, `door_view.rs` (`trap_step`) calls the scripts right after his move, before the scripts' requests are applied. Q88 asked; provenance added.
**Why:** The user's task, step 3 part 3: `pushing.md` rules 3, 9–10.
**Tested how:** `cargo test --workspace` (367 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,60 --frames 6000 --sound off` (the game's `alias bricks` spot; `--lift` puts him on the cave floor, z 14837), `connor.log`, `doors.log`.
**Result:** Update 13: "push touch Bricks (scripted): … facing cos 1.000 … -> Push", the table stops him head-on in the same move; `bricks.cs Bricks Push` runs at once: `Daventry::bricks` down, `getEndLoop Connor Bricks 1`, `Bricks` hidden, `setState Connor special 'AnimList=bricks.anm,AnimListOn,push'`, collision type 0x10121 → 0x10001, +50 experience (level 2). Connor plays `bricks.anm/push` at updates 14–63; at 63 `Cue 300 0 push`: `Bricks` shown, `down` starts at once (phase 1.0, speed 0), `waterfall` and `WaterfallSound` deleted, `water` → `blocked` at once, `Cue 5`; at 123 `Cue 10` (4 s later), getEndLoop off. No `EndLoop` from `Bricks` afterwards; only one push touch in the whole run.
**Still broken / not tested:** Not checked by eye. Still stubs in that script (step 3k): `setThirdPerson`, `handsOff`, `KQCamera::follow`, every `move` (Connor, camera, the bricks' own `move Bricks 3 0 -200 -300`), `KQObject::setHidden Table`, `loadKQ waterfa2.kq` / `watrfsn2.kq` (the new waterfall and its sound), `KQObject::collision Bricks -4` (masks of scripted objects not tracked), `addMapIcon`, `speak`. The script runs right after Connor's move, not in the middle of it (our order). Q88 open.
**Next:** 3k, the bricks puzzle end to end: the commands above, asked first where no note covers them.

## 2026-10-17 M3 step 3j-2: physical pushables (the hay bale)

**Changed:** `kq8-core/src/connor_mover.rs`: an accepted touch on a physical pushable moves it by Connor's horizontal step of the pass (`MoverWorld::push_object`): the scene sweeps the object's own ball stack against the buildings and the objects its mask makes obstacles (bits 0x100, 0x8, 0x1, 0x2, 0x2000), stops at the first contact (Q87 b stand-in), applies the ground rule (Q87 c stand-in) and puts it back if its height changed; Connor goes to the pass start + 0.95 × what it moved. Box-shaped pushables are not moved (Q87 a), logged. 2 new unit tests. `kq8-viewer` `connor_view.rs`: new system `push_apply` (after `connor_step`) moves the pushed object's colliders (Connor's and the camera's) and its drawn parts; `main.rs` schedules it. Provenance added; README and FEATURES updated.
**Why:** The user's task, step 3 part 2: `pushing.md` rules 4–5, tested on the windmill's hay bale as suggested.
**Tested how:** `cargo test --workspace` (366 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 146500,248928,1.5708 --hold 1,0,0,300 --frames 6000 --sound off` (hay bale); `cargo play --world Daventry --at 205364,267646,1.5708 --hold 1,0,0,120 --frames 3000 --sound off` (barrel `barrel.04`); `connor.log`.
**Result:** Hay bale: first push at update 14 (75.1, his walking step), then 38.7 per update (`push`, 580 units/s) for 15 moves, from x 147703 to 148319.7, where "stopped by Windmill; did not move" from then on; Connor ends at x 147578.6, behind it. Barrel: Push state at update 17, "box-shaped: its own sweep is not described (Q87 a); not moved".
**Still broken / not tested:** Not checked by eye. Barrels don't move (Q87 a). Pushed objects' new places are not saved anywhere else (e.g. `getLoc` would still give the load position; no script asks). A pushed door, chest or scripted object would snap back (their colliders follow `Active`; none is a physical pushable in Daventry).
**Next:** 3j-3, scripted pushables.

## 2026-10-17 M3 step 3j-1: the push check and the Push state

**Changed:** `kq8-core/src/collide.rs`: `BoxCollider` gets a `ColliderKind` (`colliderType`, `colliderMask`, ball stack, keepOnGround); doors, chests and scripted objects give their current values (refreshed every update, so a script's `collisionType` reaches Connor's collision); constants `PUSHABLE`, `SCRIPTED_PUSH`, `OBSTACLE_BITS`. `connor_mover.rs`: the push check of rule 2 in the across pass (and its slides, not the down pass's): on the ground or pushing, empty hands, across pass, facing within 60° (into the surface for scripted, towards the object for physical), `walk`/`run`/`push`, state takes Push; every touch logged with each condition; an accepted touch is not a hit and leaves him at the pass start (rule 4.3 with no movement, until 3j-2/3j-3). `connor.rs`: mode `Push` (`push` instead of the keys' choice, turning allowed: Q87 d stand-in), the 4-update counter of rule 7, back to Move when facing fails while pushing, the standing state when the movement can't push. `connor_update.rs` wires them. `kq8-viewer` `load.rs`: `push`, `pushleft`, `pushright` loaded from `Conner.anm`; `collisionShape` read. 5 new unit tests. Q87 asked; provenance added.
**Why:** The user's task, step 3 part 1: `pushing.md` rules 1–3, 6–8.
**Tested how:** `cargo test --workspace` (364 pass), `cargo clippy --workspace --all-targets` clean. Viewer, the windmill's hay bale: `cargo play --world Daventry --at 146500,248928,1.5708 --hold 1,0,0,120 --frames 3000 --sound off`, `connor.log`.
**Result:** Walks from update 1 (`stopwalk`, `walk`), touches `haybail.01` at update 14: "on ground yes, empty hands yes, across pass yes, facing cos 0.998, movement yes, state takes Push yes -> Push"; `push` from update 15 (580 units/s, `push1.aud`, `Push3.aud` and footstep cues), held at the bale's face (x 146992.8) by a touch every update. Key let go at 121: Wait, `push` plays on to its from point, `pushright` 132–146 (its own steps take him 494 units back), `stop` at 147.
**Still broken / not tested:** The bale doesn't move yet (3j-2); the bricks don't react yet (3j-3). Not checked by eye. Rule 2.2 ("less than 1 unit above the standing state's recorded height") is read as "began the update on the ground, not jumping or falling". Q87 open (box-shaped pushables, stop or slide, ground rule, turning while pushing, "centre").
**Next:** 3j-2, physical pushables.

## 2026-10-17 M3 step 3i-7: Connor's Special state (`setState Connor special`)

**Changed:** `kq8-core/src/connor.rs`: new mode `Special`; `special_steps` works out the item list left to right (`AnimList=`, `AnimListOn`/`Off`, `Inv <item>` logged only, `KillConnor`, any other word a movement); each movement plays to its end, then the next item; used up → standing; `Step::ended` reports a play-once movement's end; `intern` for movement names read from files; 2 unit tests. `active.rs`: `KQMonster::setState Connor special <list>` becomes a request (other `setState` calls stay logged stubs); `connor_movement_ended` tells Connor's `getEndLoop` listener `Cue 300 0 <movement>`; 1 unit test. `script.rs`: `KQMonster::setState` off the stub list. `kq8-viewer`: `load.rs` finds every movement the world's scripts can ask for and loads it (his own extra ones from `Conner.anm`, the others as `<list>/<movement>`); `connor_view.rs` starts the state and collects ends; `door_view.rs` applies the request and relays the ends; new test option `--special ITEMS,N` (`main.rs`). Provenance added.
**Why:** The user's task, step 2: `anim-scripts.md` rule 16 (the bricks' `push` from `bricks.anm`).
**Tested how:** `cargo test --workspace` (361 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 156625,303748,1.57 --special "AnimList=bricks.anm;AnimListOn;push;AnimListOff;kneel;kneelstop,20" --frames 9000 --sound off`, `connor.log`, `viewer.log`; also the same without `AnimListOff`.
**Result:** `viewer.log`: 52 `setState` lines in Daventry's scripts, 87 movements loaded, no problems; loading 1.42 s (0.88 s before). `connor.log`: update 21 `special bricks.anm/push` (51 frames, 3.33 s; `Bricks.aud` cue at update 25), update 71 `kneel`, update 91 `kneelstop`, which repeats, so he stays kneeling (as the data intends: `girl.cs` sends `getup` later). Without `AnimListOff`, `kneel` is looked up in `bricks.anm`, not found, skipped with a log line, and he stands at update 71.
**Still broken / not tested:** Not checked by eye. In this test Connor stands on the ground below the bricks' ledge (z 9180, the bricks are at 14862): `--at` puts him on the lowest ground. `Inv <item>` is only logged (not traced). `KillConnor` uses our fall-death pose (stand-in). An item naming a movement that isn't there is skipped (stand-in; the note doesn't say). No Daventry script reaches this without pushing (3j), talking or fighting.
**Next:** 3j-1, the push check and the Push state.

## 2026-10-17 M3 step 3i-6: `KQStatsChangeRequestEvent` adds

**Changed:** New `kq8-core/src/stats.rs` (`Stats`, `ConnorStats`, 2 unit tests): the five numbers are added, the fourth capped at 100; Connor's health is capped at 10 × level and his level goes up when experience passes 10 × 2^(level − 1) (150,000 from level 15). `active.rs`: the event is applied to Connor or to any object (`stats=` line; doors 10,000 health without one; others 0, Q86 stand-in); an object whose health drops below 0 gets `Die`; 1 new unit test. `kq8-viewer` `load.rs` reads Connor's `stats=` / `level=` from `GAME/Kq/Connor.kq` and every placed object's `stats=`; `door_view.rs` hands them to `Active`. Q86 asked; provenance added.
**Why:** The user's task, step 1 part 5: `anim-scripts.md` rule 19 (the bricks give Connor 50 experience).
**Tested how:** `cargo test -p kq8-core` (237 pass). Viewer: fish `cargo play --world Daventry --at 208502,209600,1.5708 --click fishies.01,10 --click fishies.01,40 --frames 1500 --sound off`; star globe as in 3i-4 (`--frames 1500`); `doors.log`, `viewer.log`.
**Result:** `viewer.log`: "stats: Connor stats=10 10 0 0 0 level=1; 7 objects with a stats= line". Fish: "Connor [10, 10, 0, 0, 0] level 1 -> [10, 10, 0, 0, 0]": the +2 health is capped (he is at full health). Star globe: +15 experience → "[20, 12, 5, 1, 5] level 2; experience passed 10".
**Still broken / not tested:** Nothing shows the stats (no screen yet); Connor can't die from it (his Die state is M5). Q86 open: "passes" (more than or at least), one level per event or several, the cap order, other objects' default stats, `Die`'s `$3`; the bricks' +50 gives level 2 with our stand-in and level 3 if levels repeat.
**Next:** 3i-7, Connor's Special state.

## 2026-10-17 M3 step 3i-5: the script reader (single quotes, `;`, `( … )`, 40 words)

**Changed:** `kq8-core/src/script.rs`: `split_words` replaced by `split_commands` (a `Word` keeps whether it was quoted and whether its `$` was inside single quotes): single quotes are dropped and switch `$` replacement off until the next one, words still split at spaces; `;` ends a command; `( … )` is one word without the brackets; a command of more than 40 words fails (`Console: Too many arguments`, `$Console::Return` 1). 1 new unit test. Provenance added.
**Why:** The user's task, step 1 part 4: `anim-scripts.md` rules 17–18 (the bricks script's `'AnimList=bricks.anm,AnimListOn,push'` needs rule 17).
**Tested how:** `cargo test -p kq8-core` (234 pass); viewer star globe run as in 3i-4, `doors.log` compared line by line with the run before.
**Result:** Unit test: `'AnimList=bricks.anm,AnimListOn,push'` reaches the command as one word; `set Q '$1' $1` gives `$1 arg`; `set R $1;set T done` runs both; `echo (keep $1 here)` prints it as written; 41 words fail, 40 are fine. Viewer: `doors.log` the same as before (apart from update numbers), 0 errors.
**Still broken / not tested:** The 40-word count is taken after variables are replaced (the note doesn't say; no script line in the 12 worlds has more than 29 words, so it can't matter). Whether `;` inside single quotes ends a command is not said; ours ends it (no data has it). The start-up follower (`kq8-data` `world_script.rs`) keeps its own simpler reader.
**Next:** 3i-6, `KQStatsChangeRequestEvent` adds.

## 2026-10-17 M3 step 3i-4: scripts reach every animated object

**Changed:** Architecture change, approved by the user. `kq8-core/src/active.rs`: new `AnimObject` list (`anims`) for the animated objects the viewer's `anim_view` draws (`water`, waterfall, torches, windmill, star globe, ...): they follow the same movement rules as doors and chests (`KQAnimPlayEvent`, waiting movement, `pauseAnim`, `EndLoop` / `getEndLoop`, cues with sounds, `deleteObject`). `door.rs`: one step for all (`step_with`): the repeating frame rule of 3c (round, wrap) and cue ranges at a wrap (`objects.md` "Loops" rule 4). `named_anim.rs`: `flags` bit 8 random start and speed (`with_random_start`, `speed_scale`). `kq8-viewer`: `load.rs` gives `anim_view` every movement of such an object, and objects with more than one movement join even when still; `anim_view.rs` no longer plays anything itself (only unused, H-key objects keep their own player), it draws the movement and frame core says; `terrain_view.rs` builds them before the doors so `Active` gets them. 3 new unit tests. Q85 asked; provenance added.
**Why:** The bricks puzzle needs `water` → `blocked`; scripts couldn't reach objects drawn by `anim_view` (DESIGN.md 3i-4).
**Tested how:** `cargo test --workspace` (354 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 270813,225311,-1.5708 --click Condoor,20 --click Condoor,60 --frames 400 --sound off`, `anim.log` compared with the run before the change (`anim-before-3i4.log`); `cargo play --world Daventry --at 144046,208300,1.5708 --lift 6300 --item INVITEM_SpellPage --click starball.01,40 --frames 2500 --sound off`, `doors.log`.
**Result:** `anim.log`: the 72 objects animated before start with the same phase, speed and frame (random ones included); 3 still objects with several movements joined (`Dummy`, `Pyramid`, `bell`): 75 objects, 71 reachable by scripts, 8 with more than one movement. Per-update lines: `millwheel` (has `currAnimName`) identical; `windprop`, `wellwatr`, `Davtorch.01` (only `nextAnimName`) are one update behind (rule 3's first-update start, Q85 stand-in), `Davtorch.01` keeps its random start (0.2819). Star globe: click with the spell page at update 40 → `open` starts at once (its `stop` can be cut short), `starball.aud` cue at 41, `open` ends at 69 (1.93 s), `EndLoop open` and `Cue 300 0 open` at 70, the script sends `lid`, which starts at once (`open` has ended) and ends at 71 (`starbal3` has no length).
**Still broken / not tested:** Not checked by eye (the star globe and the 3 new objects should look the same as before until a script switches them). The star globe's `loadKQ Potion.kq` is still a stub (3k). Q85 open: whether the torches' random start survives rule 3 in the original, and whether the first-update start uses up the update.
**Next:** 3i-5, the script reader.

## 2026-10-17 M3 step 3i-3: `pauseAnim` and `setAnimPos`

**Changed:** `kq8-core/src/named_anim.rs`: `paused` flag: while set, no phase change, no cues, no end, no waiting movement starts. `active.rs`: commands `pauseAnim <name>` (toggles) and `setAnimPos <name> <phase>`, for doors, chests and scripted objects; both log `<name>, <phase>`; an unknown name logs `pauseAnim:  could not find anim <name>`. `script.rs`: `pauseAnim` removed from the stub list. 2 new unit tests. Provenance added.
**Why:** The user's task, step 1 part 3: `anim-scripts.md` rules 14–15.
**Tested how:** `cargo test -p kq8-core` (230 pass): `a_paused_animation_doesnt_move_fire_cues_or_end`, `pause_anim_toggles_and_set_anim_pos_sets_the_phase`.
**Result:** Paused for 40 updates: phase stays 0, no cue; resumed: the cue at 0 fires on the next move. In the script test: "pauseAnim: basket1, 0: paused", the unknown name's line, then "resumed" and "setAnimPos: basket1, 0.5", and the phase moves on from 0.5.
**Still broken / not tested:** Not tested in the viewer: Daventry's only `pauseAnim` calls are for the monster `sprdump` (M5), which we don't have, so they log "could not find anim sprdump". Objects played by `anim_view` can't be paused yet (3i-4). Return values and the usage lines are our choice (the note gives none).
**Next:** 3i-4 needs your decision (see the plan); meanwhile 3i-5, the script reader.

## 2026-10-17 M3 step 3i-2: `KQAnimOnce` and `setDeleteWhenDone` (core only)

**Changed:** `kq8-core/src/named_anim.rs`: `once` flag (a `KQAnimOnce`): no waiting movement starts at its end. `active.rs`: `ActiveObject::delete_when_done`; when a `KQAnimOnce`'s movement ends, its script gets `Die`, its `getEndLoop` listener gets `Cue 300 0 <movement>`, then it is deleted if asked (as `deleteObject`), with no `EndLoop`; new command `setDeleteWhenDone <name> 1|0` (only `KQAnimOnce` objects, else `setDeleteWhenDone: could not find object '<name>'`; wrong word count: a usage line in our words). 2 new unit tests. Provenance added.
**Why:** The user's task, step 1 part 2: `anim-scripts.md` rules 11–13.
**Tested how:** `cargo test -p kq8-core` (228 pass), unit tests `a_kq_anim_once_reports_die_tells_its_listener_and_is_deleted` and `a_kq_anim_once_keeps_its_last_frame_and_never_starts_the_waiting_one`.
**Result:** In the test: `Die` reaches the script (it sets a variable), Connor's listener line "tells Connor (getEndLoop: Cue 300 0 open)" appears, no `EndLoop open`, the object is deleted; before it is a `KQAnimOnce`, `setDeleteWhenDone` says it could not find it.
**Still broken / not tested:** Not tested in the viewer: Daventry's `KQAnimOnce` objects (barrel bursts, the alchemist's boards falling) are only made by `loadKQ` while playing, after a fight (M5). Nothing marks an object as `KQAnimOnce` yet; that comes with `loadKQ` while playing (3k). The `boardx.kq` guard ("Do not reproduce") belongs there too.
**Next:** 3i-3, `pauseAnim` and `setAnimPos`.

## 2026-10-17 M3 step 3i-1: scripts start movements (the waiting movement)

**Changed:** `kq8-core/src/named_anim.rs`: the movement player now keeps the current movement's name as asked, one waiting movement and "can be cut short"; `request` (`KQAnimPlayEvent`), `start`, `has_ended`, `with_names_at_load`; 8 new unit tests. `door.rs`: `DoorMovement` gets `cut_short` (6th field of `loop=`), "has ended" at 0.9999; `Door` now holds that same player (its four states stay on top). `active.rs`: `KQAnimPlayEvent` to doors, chests and scripted objects goes through the new rules, with a log line per request and per waiting movement that starts. `kq8-viewer` `load.rs` / `door_view.rs`: pass `currAnimName`, `nextAnimName` and the 6th field. Plan for all of 3i/3j/3k in `docs/DESIGN.md`; provenance section added.
**Why:** The user's task, step 1 part 1: `anim-scripts.md` rules 1–10.
**Tested how:** `cargo test --workspace` (347 pass), `cargo clippy --workspace --all-targets` clean. Viewer: chest `cargo play --world Daventry --at 202336,262000,1.5708 --click smlchst3,10 --click smlchst3,40 --frames 600 --sound off`; door `cargo play --world Daventry --at 270813,225311,-1.5708 --click Condoor,20 --click Condoor,60 --frames 400 --sound off`; `kq8-artifacts/logs/doors.log`.
**Result:** Chest and door as before: `smlchst3` `open` starts at update 10 and ends at 25, item, then empty; `Condoor` opens 20 → 35, `Open`, `EndLoop open` at 36. New at update 0 (rule 3): `smlchst1`, `jar1` and the six baskets have no `currAnimName` and start their `nextAnimName` `stop`; `smlchst2`'s `nextAnimName=closed` names no movement, so movement 0 plays under that name (the note's one such object).
**Still broken / not tested:** Objects played by `anim_view` (`water`, `waterfall`, torches, ...) are still out of the scripts' reach (step 3i-4, an architecture change I will ask about first). No Daventry script sends `KQAnimPlayEvent` to an object in a way the viewer can reach yet (the bricks need pushing, 3j). Not checked by eye (nothing looks different).
**Next:** 3i-2, `KQAnimOnce` and `setDeleteWhenDone`.

## 2026-10-17 A: pushing objects (`pushing.md`)

**Changed:** New `docs/behaviour/pushing.md` (rules 1–11): when a touch becomes a push (type bit 0x100; moving, on the ground, empty hands, across pass, facing within 60°, walking or running); the Push state (`push` movement, a 4-update counter, where it can go next); physical pushables (the object moves by Connor's step, only if its height stays exactly the same, Connor follows at 0.95); scripted pushables (bit 0x10000: a `Push` script call, which Daventry's bricks, coffin lid and urn scripts use to take over); the bricks puzzle from start to end. Cross-references in `anim-scripts.md` (open question 1 answered) and `connor-collision.md` rule 4. `STATUS.md`. No code.
**Why:** The user's request: the bricks puzzle needed its `Push` trigger (`anim-scripts.md` open question 1).
**Tested how:** Traced in Ghidra. Data survey of every `colliderType` with bit 0x100 in the 12 worlds, the `push` animation's frame steps, and Daventry's push scripts.
**Result:** Rules ready for B. Daventry pushables: `Bricks`, `coffnlid`, `urn` (scripted), four barrels and the windmill's hay bale (physical). The game's own `alias bricks` gives a test spot next to the bricks.
**Still broken / not tested:** Checked in the original by the user (2026-10-17), all as the note says: the hay bale slides at about half walking speed and stops at walls; barrels move only on perfectly flat ground; a drawn weapon or walking backwards pushes nothing. Pushing downhill was not tried. Not traced: the other pushable classes (Gnome's boulder, Temple 3's tiles, Dead City's paddle), state 0x8000, the moment the standing height is recorded.
**Next:** B: build `anim-scripts.md`, then `pushing.md`, then the bricks puzzle. 

## 2026-10-17 A: animated objects controlled by scripts (`anim-scripts.md`)

**Changed:** New `docs/behaviour/anim-scripts.md` (rules 1–19): the waiting movement and `nextAnimName`; `KQAnimPlayEvent` (when the new movement starts, from its own `loop=` line, unknown names fall back to movement 0, stopped `0 0` poses never change); the order at a movement's end; `KQAnimOnce` and `setDeleteWhenDone` with Daventry's barrel and board chains; `pauseAnim` (a toggle) and `setAnimPos`; Connor's `setState … special` word list; single quotes, `;`, `( … )` and the 40-word limit in scripts; `KQStatsChangeRequestEvent` adds its numbers (fifth = experience, Connor's levels for M5). Cross-references in `clicking.md` (rule 24, open question 3 answered), `objects.md` (`nextAnimName`), `doors.md` (script language) and `QUESTIONS.md` Q64. `STATUS.md`. No code.
**Why:** Suggestion 2: scripts that switch, pause and remove animated objects are the main piece missing from M3 ("Objects only play their starting animation").
**Tested how:** Traced in Ghidra. Survey of all 1,081 scripts and all `.kq` / `.anm` files for how each command is used (counts in the note).
**Result:** Rules ready for B. Data checks: no placed object has transitions; `KQAnimPlayEvent` always has one word; `nextAnimName` never changes which movement shows first in the shipped data; 30 stopped `0 0` poses (all final poses); `boardx.kq` is never loaded (it would load itself forever).
**Still broken / not tested:** Nothing was checked in the original game. Open: what sends `Bricks` its `Push` (pushables study), the collision bit switched when a movement starts, Connor's `Inv` item, `KQMoveEvent` flags 1 and 3 on objects, monsters (M5).
**Next:** B: build `anim-scripts.md` (plan in DESIGN.md first). A: pushable objects (`KQStateConnerPush`, `KQBoulder`), which the bricks puzzle needs.

## 2026-10-17 Fix: a jump that hits something follows A's corrected rule (Q82)

**Changed:** `kq8-core/src/connor_update.rs`: the Q82 stand-in ("every non-back jump hit crashes") replaced by `jump_hit_outcome`: during `jump` or `runJump` a steep hit crashes if it is on the down pass, or if the head-on test said head-on for it (up, across or slide pass). A steep hit that is not head-on ends the jump without a crash and the animation plays on (stand-in, Q84 a). The log line now says why: "crash (head-on; …)" or "crash (landed on something steep)". Tests: `a_standing_jump_into_a_wall_crashes_stand_in_q82` renamed `…_crashes_head_on`, new `only_head_on_or_down_pass_jump_hits_crash`. `docs/DESIGN.md` (plan, done), provenance, `docs/behaviour/QUESTIONS.md` (Q82 built; new **Q84**), `docs/FEATURES.md`, `STATUS.md`.
**Why:** A's answer to Q82 (`connor-physics.md` "Jumping" rule 5 and "Walls", corrected 2026-10-17): `jump` is in the crash list, and during a jump every steep surface is head-on, except against a `KQMonster` or with a Connor flag.
**Tested how:** `cargo test --workspace` (339 pass), `cargo clippy --workspace --all-targets` clean. `cargo play --world Daventry --jump 10 --frames 900 --sound off`, `kq8-artifacts/logs/connor.log`.
**Result:** Same movement as the stand-in, as expected: update 23, across pass, head-on into `tree.06` (normal (0, 1, 0), angle 0.0°) → "jump hit tree.06 (steep, across pass): the jump ends, crash (head-on; jump and runJump are in the crash list)". `crash` plays at updates 24–27 (`crash.aud` at 24), Fall starts at 28 (`jumphold`), lands at 30 (978 below the death limit's start), `jumpend` at 31. A's other difference ("walkable surfaces that are not head-on don't crash") was already true of the stand-in: only steep hits were ever looked at.
**Still broken / not tested:** In our code today the new rule and the stand-in give the same result everywhere: no `KQMonster` collides with Connor and the Connor flag isn't built, so the "not head-on" case is only covered by a unit test. Q84 asked: (a) what follows that case; (b) rule 5 says turning stays allowed at a crash, our `crash` ignores the keys (not changed). Not checked by eye after this change (the movement didn't change). Noticed, not changed: `connor_mover.rs` still runs the head-on angle test on down-pass hits, which "Exact hit handling" (Q37 c) says never happens; it doesn't affect this rule (a down-pass hit crashes before the head-on result is looked at), but it may affect walking down steep slopes. Left as a separate step.
**Next:** A: Q84. B (if you want): the down-pass head-on point above, as its own step.

## 2026-10-17 A: cleanup of open questions (Q80, Q81 c, Q82 answered; Q4 closed)

**Changed:** `docs/behaviour/connor-physics.md`: "Jumping" rule 5 and "Walls" corrected. The crash list is `Run`, `stoprun`, `WaterRun`, `runwaterrun`, `runJump` and `jump`; it used to say `PreBirth`, a misreading by one place in the game's name table. Also new: in a jump, any surface with normal z under 0.8 counts as head-on and overrides the ball-stack exception, plus what happens at the moment of a crash. `objects.md` rule 4: cue ranges when a loop plays backwards past its wrap, and the forward-wrap transition detail. `object-lighting.md`: how A's two Q80 counts were made, and the corrected count (164,622). `models-and-textures.md`: Q4 second try recorded. `QUESTIONS.md` (Q4, Q80, Q81, Q82), `STATUS.md`. No code.
**Why:** Suggestion 2 (the user's choice): close the small open questions that hold stand-ins.
**Tested how:** Traced in Ghidra (Connor's hit handling, both per-update animation advances, the name table, the material list loader). Q80: re-ran A's measuring script on the loose building files and compared it with B's way of counting.
**Result:** Q4 (follow-up the same day, from the data): none of the 29 missing names is ever drawn (25 are in `zedcity.dml`, which nothing names; `bro20` in an unused brother model; `ghoul` and `fndmask` in unused models; `chf0` in the Helmet item, which no world places), so Q4 is closed. Q82: a standing jump into anything steep crashes, as the user saw. B's stand-in is right apart from small differences (QUESTIONS.md). Q81 (c): B's stand-in is right. Q80: all three of A's numbers reproduced exactly (164,838, 7,511, 160,649). The 160,649 came from a rougher projection; B's 164,640 is the right kind of count. The five files that looked like "corners outside" are the ones without 48-byte records, misread by the script. In the code, what the game does with a texture found nowhere was not found after two tries; that no longer matters (see Q4 above).
**Still broken / not tested:** Not traced in the code: why the stumble ends early when Connor starts falling; the user's check in the original is the rule. The tile-puzzle jumps (Barren, Dead City, Swamp) would stop instead of crash; the user says these puzzles are in wide open spaces, so it can't happen.
**Next:** B: replace the Q82 stand-in with the corrected rule (small). 

## 2026-10-17 Docs: finished milestones' step plans moved out of DESIGN.md

**Changed:** The step-by-step plans of M1 (steps 1–7) and M2 (2a–2i) moved word for word from `docs/DESIGN.md` into the new `docs/milestones/M1.md` and `docs/milestones/M2.md`. M1 headings are `##` instead of `####`; nothing else changed. In `DESIGN.md` each of those milestones now has a short list of its steps and a link. The M1 intro and the M2 decisions ("Physics rate", "Where the code goes") stay in `DESIGN.md`, and so does M3, which is still in progress. The repository layout now lists `docs/milestones/`. `README.md`: `docs/milestones/` added to the folder list and to "More". No code changed.
**Why:** The user's choice (suggestion 5). `DESIGN.md` was 3,006 lines and is read at the start of every session.
**Tested how:** A Python script rebuilt the original text from the two new files and compared it with `git show HEAD:docs/DESIGN.md`. Searched `DESIGN.md` for "above" or "below" pointing at moved text. `git status` checked that the whitelist `.gitignore` picks up `docs/milestones/`.
**Result:** Both moved blocks match the original line for line (blank lines aside). M3 to the end of the file is unchanged. `DESIGN.md` is now 1,217 lines; `M1.md` has 973 and `M2.md` 855. The only "below" pointers were the two lines in the M1 intro, now pointing at `M1.md`.
**Still broken / not tested:** Code comments that say "docs/DESIGN.md M1 step 5c" (and similar) were left alone. They still lead there, because `DESIGN.md` links to the milestone file. The link targets have not been checked in a Markdown viewer (not tested).
**Next:** When M3 is finished, move its steps to `docs/milestones/M3.md` the same way.

## 2026-10-17 Fix: footsteps in the air follow A's rule 39a (Q81 d)

**Changed:** `kq8-core/src/sound.rs`: new `StepGround` (which building the steps sound like; 1 unit test), `building_floor_flags` comment (no key = 0, traced). `kq8-viewer/src/connor_view.rs`: the footstep uses it instead of "the last thing he stood on". Provenance, `docs/FEATURES.md`, `docs/DESIGN.md` (3h-4 note), `STATUS.md`.
**Why:** A's answer to Q81 (`sound.md` rule 39a): a building is remembered only from a building hit (any other hit clears it), and in the air it is kept for 2 updates with gravity on (nothing counts with gravity off), then the terrain under him now is used. Ours kept the last ground for as long as he was in the air. (a) and (b) were already right.
**Tested how:** `cargo test --workspace` (338 pass), clippy clean. `cargo play --world Daventry --at 202336,262400,1.5708 --lift 3400 --hold 1,0,0,60 --frames 900 --sound off`, `connor.log`.
**Result:** Steps on the pub's upper floor: "footstep on Pub (building without footsoundType: 0, so 0x1)", dirt. Walks off the edge: in the air at updates 49, 50, 51; "steps forget building Pub: 3rd update in the air" at 51; that update's step and the landing's two steps use the terrain (dirt).
**Still broken / not tested:** Not listened to (it sounds the same here: dirt either way). The `special` state of rule 39a doesn't exist in our code yet. Q81 (c) still a stand-in.
**Next:** —

## 2026-10-17 M3 step 3h-5: world music

**Changed:** New `kq8-core/src/music.rs` (`Music`, `MusicRequest`: the tune list, push / pop / play; 8 unit tests). `kq8-core/src/sound.rs`: each sound has its own level and can fade (`start_at_level`, `fade`, `set_level`, `is_playing`, `level`; fades step in the 200 ms housekeeping, wait while paused, stop the sound at 0; every step logged; 1 unit test). `active.rs`: `KQMusic::push` / `pop` go through the event queue 0.2 s later, `play` at once, usage line, `create` / `delete` obsolete; requests handed over in `WorldState::music` (1 unit test; the walk-zone test updated). `script.rs`: push / pop no longer stubs. Viewer: `load.rs` `place_music` (`nMusic`, `music<n>`), `sound_view.rs` starts the tune on the first frame and applies requests, `main.rs`. `docs/DESIGN.md` (3h-5 plan, done), provenance, `README.md`, `docs/FEATURES.md`, `STATUS.md`.
**Why:** The user asked for world music (`sound.md` rules 33, 57–57l, A's answer to Q83).
**Tested how:** `cargo test --workspace` (337 pass), `cargo clippy --workspace --all-targets` clean. `cargo play --world Daventry --at 193000,193000,3.1416 --hold 1,0,1,90 --frames 1300 --sound off` (runs north through the wizard's music zone `mwiztrap`), `sound.log` and `traps.log` read.
**Result:** `forest.aud` starts on frame 1, group 5, loudness 0.140, no fade. `mwiztrap` On at update 23 → push due 1.733 s; at t 1.796 `mwizard.aud` starts at level 0, both fade in 0.15 steps; `forest.aud` stopped at 0 and `mwizard.aud` done at 1.00 at t 3.002 (1.21 s: 7 steps on the shared 200 ms timer, the first one comes early). Off at update 60 → pop due 4.2 s; at t 4.263 `mwizard.aud` fades out and a new `forest.aud` starts from its beginning at 0; both done at t 5.602 (1.34 s). Failed tries first: I tried the church's zone (`chrchtrp`) for the test, but the church is walled in (its only way out is the rope, not built); push into it worked, the pop could not be reached. At the very start, slow first frames make the timer catch up two steps in quick succession (frames 3 and 4); harmless.
**Still broken / not tested:** Checked by ear by the user (2026-10-17): "everything works great, just as expected". Not built: pause (57h), videos (57i), leaving and coming back to a world (57j), Barren's lava-tile pushes from the game (57k). `play` is tested by unit test only (no script uses it). `mbattle.aud` (henchman fight) not tried: no fighting yet.
**Next:** You listen (see the reply). B: footsteps in the air per A's Q81 (d) answer (`sound.md` rule 39a), a separate small step.

## 2026-10-17 A: world music (Q83) and footsteps (Q81 a, b, d)

**Changed:** `docs/behaviour/sound.md`: new section "World music (`KQMusic`)" (rules 57–57l replace the old rule 57), rule 39 (a building without `footsoundType` = dirt), new rule 39a (what counts as the ground under him), rule 55 wording, numbers table, data, edge cases, "Do not reproduce" 8–10, "How to check" (music), open questions 3–6. `docs/behaviour/QUESTIONS.md` (Q83 answered; Q81 a, b, d answered, c open). `STATUS.md`. Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** B needs Q83 before building world music (rule 57 said "not studied further"); Q81 had stand-ins waiting for answers.
**Tested how:** Traced in `Mask.exe` with Ghidra (the music object, its push / pop / play handlers, the script commands, fades, pause, the video player, the footstep ground lookup). Data: all 12 `music.kq` files and all 131 `KQMusic` lines in the scripts (`grep`).
**Result:** Push of a name already on the list does nothing; otherwise a 1.4 s crossfade, and the covered tune stops and later comes back from its beginning. Pop of a lower tune removes it silently; pop of an unknown name does nothing; the last tune can be popped (silence). Script push / pop act 0.2 game-clock units later; `play` at once and is never used. The world tune starts at full loudness, no fade. Footsteps: only buildings count as "standing on"; crates and tables give the terrain under him; in the air the building is kept 2 updates. B's Q81 stand-ins are right except that detail.
**Still broken / not tested:** Not tested in the original. Not settled: what brings music back after a video (open question 5); that a world change removes the old music object is inferred (6); Q81 (c) not studied; Q82 (a) not studied this session. Two attempts to find where the mover's "standing on" record is written went nowhere before the third (the down pass) found it.
**User's check in the original (2026-10-17):** at the wizard's trap the tune cross-fades in over about 1.5 s; on leaving, the forest tune starts again from the start. Matches rules 57c and 57d.
**Next:** B: world music from rules 57–57l. You (done, see above): in Daventry, walk into and out of the wizard's trap and listen whether the forest tune comes back from its start. A: Q82 (a), Q81 (c).

## 2026-10-17 B: review of the sound work, next steps; question Q83 (music)

**Changed:** `docs/behaviour/QUESTIONS.md` (new **Q83**: world music), `STATUS.md` (open questions). No code.
**Why:** The user asked B to review the sound work (M3 3h) and suggest next steps. World music is the most audible missing piece, and `sound.md` rule 57 says it was not studied further.
**Tested how:** Read `sound.md`, the 3h plan, the MODLOG and STATUS. Data checked: every world's `music.kq` has `nMusic=1`; Daventry's `KQMusic` lines (`chrchtrp.cs`, `mugltrap.cs`, `mwiztrap.cs` push/pop; `ax.cs` push and `henchman.cs` pop of `mbattle.aud`); `playsound` cues in Daventry's `.anm` files are almost all on characters and monsters (not drawn yet); none on the scenery that animates today (fountain, windmill, wheel, signs, torches, fish).
**Result:** Sound effects (3h-1 to 3h-4) are done and checked by ear. Q81 stays open with working stand-ins (not blocking). Music needs Q83 answered first.
**Still broken / not tested:** As in STATUS "Sound, not done".
**Next:** A: Q83 (music), then Q81, Q82 a. B: world music once Q83 is answered; meanwhile `loadKQ` while playing (brings in `Waterfall2Sound`) or scenery cues.

## 2026-10-17 Fix: a fall cuts the stumble short (Q82 b, user's check)

**Changed:** `kq8-core/src/connor_update.rs`: removed the "crash plays to its end through a fall or a landing" part of the Q82 stand-in; a fall starting during the crash switches to the fall's hold animation at once, as before. The crash on a standing jump stays. `docs/behaviour/QUESTIONS.md` (Q82), provenance, `docs/FEATURES.md`.
**Why:** The user's check in the original (2026-10-17): the stumble does not play to its end; when he starts falling, the falling animation takes over. My extra rule was a guess and wrong.
**Tested how:** `cargo test --workspace`, clippy clean. `cargo play --world Daventry --jump 10 --frames 900 --sound off`, `connor.log`.
**Result:** `crash` updates 24–27 (with `crash.aud`), fall starts at 28 → `jumphold`, lands at 30 → `jumpend` at 31.
**Still broken / not tested:** Why the original crashes at all (Q82 a). Checked by eye by the user after this fix (2026-10-17): "looks just like the original".
**Next:** A: Q81, Q82 a.

## 2026-10-17 Stand-in for Q82: a standing jump into something crashes

**Changed:** `kq8-core/src/connor_update.rs`: a standing `jump` that hits something steep in the up or across pass now crashes like `runJump` (the note says it stops); a crash plays to its end even when a fall starts or he lands during it, and if he is still falling then, the fall's hold animation takes over. Test renamed and changed: `a_standing_jump_into_a_wall_crashes_stand_in_q82`. `docs/behaviour/QUESTIONS.md` (Q82: stand-in in place), provenance, `docs/FEATURES.md`, `STATUS.md`.
**Why:** The user's check in the original (2026-10-17): at the start, a standing jump into the tree makes Connor stumble with the crash sound. Chosen by the user as a stand-in until A answers Q82; it contradicts `connor-physics.md` "Jumping" rule 5.
**Tested how:** `cargo test --workspace` (327 pass), clippy clean. `cargo play --world Daventry --jump 10 --frames 900 --sound off`, `connor.log`, `sound.log`.
**Result:** Update 23: across hit on `tree.06` → "crash (stand-in Q82: the note says stop)". `crash` plays updates 24–30 (all 7), `crash.aud` starts at update 24 (placed at Connor, 0.140), its two footstep cues fire; the fall starts at 28, he lands at 30 ("landed during the crash: it plays on"), then `stop` at 31. Before: `stop` at 23, `jumphold` 28, `jumpend` 31.
**Still broken / not tested:** Why the original crashes is not known (Q82 a). What a crash does when it ends high in the air is our guess (falls with `jumphold`, then `jumpend`); not seen in a run. Not checked by eye yet.
**Next:** You: jump into the tree at the start and watch. A: Q82.

## 2026-10-17 User's check of footsteps (3h-4); jump into a tree (Q82)

**Changed:** `docs/behaviour/QUESTIONS.md` (Q81 a: the user's check; new **Q82**), `STATUS.md`, `docs/FEATURES.md`. No code.
**Why:** The user listened, compared with the original, and reported a difference in jumping.
**Tested how:** By ear and by eye, by the user, in our viewer and the original. Then `cargo play --world Daventry --jump 10 --frames 900 --sound off`, `connor.log`.
**Result:** Footsteps "sound just like the original": stone on the bridge, splashing in the river; house and pub floors sound like the ground outside in the original, which our stand-in (dirt) gives. Jump: in the original a standing jump into the tree at the start makes Connor crash; ours stands (`stop`), then falls (`jumphold`, `jumpend`). Log: update 23, across pass, head-on into `tree.06` (normal (0, 1, 0)) → "the jump ends, stop (forward blocked)". That is what `connor-physics.md` "Jumping" rule 5 says, so the note and the original disagree: asked as Q82, not changed.
**Still broken / not tested:** The jump into the tree (Q82).
**Next:** A: Q82 (and Q81). Or, if the user wants, a stand-in from the user's check (see the reply).

## 2026-10-17 M3 step 3h-4: Connor's animation cues and footsteps

**Changed:** `kq8-core/src/connor.rs`: `MovementLoop::cues` and `cues_between` (the cue range with the wrap rule; 1 test). `kq8-core/src/sound.rs`: `footstep_set`, `building_floor_flags`, `footstep_file`, `Dice` (2 tests). Viewer: `load.rs` keeps Connor's `cueN` lines and each building's `footsoundType`; `connor_view.rs` fires his cues after each update (`footstep` → a plain step sound by the ground under him, `playsound` → a placed sound at him, the rest logged), `FootSounds` resource; `sound_view.rs` plays them; `main.rs`. New question **Q81** in `docs/behaviour/QUESTIONS.md`. `docs/DESIGN.md` (3h-4 plan, done), provenance, `README.md`, `docs/FEATURES.md`, `STATUS.md`.
**Why:** The user asked for footsteps (`sound.md` rules 34–39).
**Tested how:** `cargo test --workspace` (327 pass), clippy clean. Viewer runs with `--sound off`, `connor.log` and `sound.log` read: walk near the fountain `--at 208050,240000,-1.5708 --hold 1,0,0,300 --frames 3500`; across the bridge `--at 156500,266400,1.5708 --hold 1,0,0,220 --frames 2600`; the pub's upper floor `--at 202336,262400,1.5708 --lift 3400 --hold 1,0,0,60 --frames 900`.
**Result:** Walk: 33 steps in 20 s, cues at 0.1 and 0.6 each cycle (phases 0.0605 → 0.1132 and 0.5868 → 0.6395); dirt, and water on 6 wet squares; files 1 / 2 / 3 picked 10 / 16 / 7 times. Bridge: 8 dirt steps on the slope, **15 rock steps on `bigbrig2`** (`footsoundType=2`). Pub upper floor: 5 steps "building without footsoundType: 0x1 (stand-in, Q81 a)" → dirt; walking off the edge, the landing (`jumpend`) fired its two footstep cues on the terrain. Both `footsoundType` buildings found (`cav2` 4, `bigbrig2` 2). First pub run stood on the ground floor, which is terrain there (z 8,160); `--lift` was needed to reach the upper floor.
**Still broken / not tested:** **Not listened to.** House floors use dirt as a stand-in, objects the terrain under him (Q81 a, b); a backward loop's wrap is our mirror (Q81 c). Not tried: the cave (`cav2`, should be silent), stepping on an object. Monsters have no steps (none exist yet).
**Next:** You: listen (see the reply), and if you can, compare house floors in the original (Q81 a). A: Q81.

## 2026-10-17 User's check of sound effects (M3 3h)

**Changed:** `STATUS.md`, `docs/FEATURES.md`, `README.md`: sound marked as checked by ear. No code.
**Why:** The user listened.
**Tested how:** By ear, by the user: the three test commands of 3h, then normal play (doors, items).
**Result:** `aud-verify`: 22 MATCHES, RESULT: PASS. Chest opening, then the crystal pickup, heard. Door creak heard, a little louder in the left speaker. In normal play "all the sounds are as expected". `aud-verify` itself plays nothing (it only writes `kq8-artifacts/wav/AXEDOOR.wav` to play in a media player); my test text could have said that more clearly.
**Still broken / not tested:** Footsteps, scenery animation sounds, music, speech not built. The decoded `.wav` not listened to.
**Next:** Footsteps (needs Connor's animation cues first).

## 2026-10-17 M3 step 3h-3: Daventry's looping sound objects

**Changed:** `kq8-core/src/sound.rs`: `Audible` (a `KQAudibleObject`: on while near or far, restart from the beginning, stop on `deleteObject`; 2 unit tests); log of balance changes only while a sound can be heard. Viewer: `load.rs` `place_audible` (the `.kq` keys, `Objects::audibles`), `sound_view.rs` runs them each frame with the world's `visibleDistance`, `main.rs` passes them on. Docs at the end of the session: `docs/DESIGN.md` (3h done), provenance, `README.md`, `docs/FEATURES.md` (sound, `--sound`), `STATUS.md`, `QUESTIONS.md` (Q61: the pickup sound plays now).
**Why:** Step 3 of sound effects (`sound.md` rules 30, 43–48).
**Tested how:** `cargo test --workspace` (324 pass), clippy clean. Viewer `--world Daventry --at 208050,240000,1.5708 --frames 300 --sound off` and `--at 208050,240000,-1.5708 --hold 1,0,1 --frames 9000 --sound off` (Connor runs off for about a minute); `viewer.log`, `sound.log`.
**Result:** 5 sound objects loaded with the note's files and distances (`WaterwheelSound`, `WatermillSound`, `WaterfallSound`, `WindmillSound`, `FountainSound`). From the fountain: fountain on at 10,466 (loudness 0.061); water wheel (26,385) and mill (21,033) on but beyond max: silent (rule 30). Running away: fountain paused at 20,115, switched off at 27,169; windmill switched on at 27,046 (silent); water wheel heard from 19,245 in (0.033 at 14,705 = 0.14 × (1 − 14,705 ÷ 19,250)), paused at 20,049, switched off at 27,191; mill switched off at 27,196. 79 log lines in about 60 s.
**Still broken / not tested:** **Not listened to.** `Waterfall2Sound` (`watrfsn2.kq`) is not loaded: it comes from `bricks.cs` while playing, and `loadKQ` while playing is a stub; `deleteObject WaterfallSound` tested by a unit test only. `setActiveCamera`/`makeActive` don't apply to sound objects yet (no Daventry script uses them on one). No new questions for A: every rule used was clear.
**Next:** You listen (see the reply). B: footsteps need Connor's animation cues run first (own step); scenery animation cues.

## 2026-10-17 M3 step 3h-2: doors, chests, pickups and script sounds play

**Changed:** Audio library chosen by the user: **cpal 0.18.2** plus our own mixer. New crate `crates/kq8-audio` (mixer and sound card, 5 unit tests). New `crates/kq8-core/src/sound.rs` (the rules: groups, distance curve, balance, pan law, beyond max, mix limit, housekeeping, missing files; 12 unit tests). `kq8-data/src/aud.rs`: `find`/`name_variants` (rule 13 lookup, 1 test). `kq8-core/src/active.rs`: door, chest and scripted-object `playsound` cues, item pickups and `KQSound::play` now queue sounds (`WorldState::sounds`); `script.rs`: `KQSound::play` no longer a stub. Viewer: new `sound_view.rs`, option `--sound on|off`, log `kq8-artifacts/logs/sound.log`. `aud-verify` checks the lookup too. `Cargo.toml` (workspace member, `cpal`), `docs/DESIGN.md` (plan, layout), provenance, `README.md`.
**Why:** Step 2 of sound effects (`sound.md` rules 13, 18–32, 35, 40, 42, 49–52).
**Tested how:** `cargo test --workspace` (322 pass), `cargo clippy --workspace --all-targets` clean. `aud-verify`. Viewer runs, `sound.log` read: chest `--at 202336,262000,1.5708 --click smlchst3,10 --click smlchst3,40 --click smlchst3,45 --frames 1200`; door `--at 270813,225311,-1.5708 --click Condoor,20 --click Condoor,60 --frames 400 --sound off`; fish `--at 208502,209600,1.5708 --click fishies.01,10 --click fishies.01,40 --click fishies.02,45 --frames 900`.
**Result:** Groups from `Options.cs`: effects 0.14, speech 0.34, music 0.14. Sound card opened: "Speakers (Realtek(R) Audio), 48000 Hz, 2 channels, f32". Chest: `chstopen` found as `GAME/SOUND/CHSTOPEN.AUD`, placed, distance 3,298, loudness 0.140, balance +0.128; finished after 1.406 s (file 1.331 s, the 200 ms step). Crystals: `energy.aud` plain, 0.140. Door: `door4.aud` placed at the door, distance 2,200, loudness 0.140, balance −0.305. Fish: `eat.aud` twice, plain, 0.140, from `fishies.cs` / `fishies2.cs` line 6. With `--sound off` the same lines, no card opened. Lookup: `raven.snd`, `sprgouch.wav`, `chstopen`, `door4.aud`, `dirt1` found, `poffup.aud` not found (all match). The one failed run: a click on `keepdrl` gave no sound because that door is locked (it needs the keep key); not a sound fault.
**Still broken / not tested:** **Nobody has listened yet**: the numbers are checked, the sound itself is not (you: see below). Not in this step: footsteps (Connor's animation cues aren't run by our code yet), the item land sound (nothing is thrown), cues of scenery animations (`anim.rs` doesn't run cues), ducking, pause, music, speech. Mix limit and beyond-max behaviour are tested by unit tests only; nothing in Daventry starts that many sounds yet.
**Next:** You: listen (commands in the reply). Then 3h-3, the looping sound objects.

## 2026-10-17 M3 step 3h-1: reading `.aud` sound files, `kq8 aud-verify`

**Changed:** New `crates/kq8-data/src/aud.rs` (SOL and WAV decoder, 9 unit tests); new `crates/kq8-cli/src/aud_verify.rs` and the `aud-verify [--wav <file>]` command in `main.rs`. Plan for all of 3h in `docs/DESIGN.md`; `docs/provenance/README.md`; `README.md` (check command, one line in "What works").
**Why:** Step 1 of sound effects (`docs/behaviour/sound.md` rules 1–17). The 128 step numbers come from the community note `kq8aud.txt` (rule 7 says the game uses the same table); our code builds them from their pattern.
**Tested how:** `cargo test -p kq8-data -p kq8-cli` (104 pass), clippy clean. `cargo run --release -- aud-verify --wav daventry/sound/AXEDOOR.AUD`, read `kq8-artifacts/logs/aud-verify.log`. Also `--wav` with `daventry/sound/FOREST.AUD` and `GAME/SOUND/hm_iced.aud`; the three `.wav` files opened with Python's `wave` module.
**Result:** 16 of 16 checks against the note match: 1,750 `.aud` (1,749 SOL, 1 WAV, 0 refused), 1,745 / 4 at 22,050 / 44,100 Hz, format 0x0D 1,657 / 0x1D 92, 2,849 speech files (2,839 / 10), header + data = file length in all 4,598 SOL files, 0 wrap-rounds, peak 32,768, `AXEDOOR.AUD` 11,847 samples (0.537 s), `FOREST.AUD` stereo 178.5 s, `NONE.AUD` 8 samples, longest `AMBDEAD.AUD` 187.6 s. Runs in about 10 s. The `.wav` files read back with the right rate, channels and length. RESULT: PASS. First run said FAIL on one count: my tally counted the WAV file's rate with the SOL ones (1,746); fixed to count SOL only, as the note does.
**Still broken / not tested:** Nobody has listened to a decoded file yet (you: see below). No sound plays in the viewer yet. Note's counts include the copies under `MASK/data/`; its "2,498 decoded" are the loose SOL files (1,006 + 1,492), not stated in the note but the numbers add up.
**Next:** You: listen to `kq8-artifacts/wav/AXEDOOR.wav`. Choose the audio library for 3h-2.

## 2026-10-17 User's answers to `sound.md` open questions 1 and 2

**Changed:** `docs/behaviour/sound.md` (rule 31, "Do not reproduce" 1, "How to check", open questions 1–2), `STATUS.md`. No code.
**Why:** The user checked the original and made the call on the distance curve.
**Tested how:** By ear in the original, by the user: waterfall on the left of the screen.
**Result:** (1) The sound comes from the left speaker; the balance follows the camera. (2) Keep the original distance formula; a corrected curve may be added later as an option.
**Still broken / not tested:** —
**Next:** B implements sound from `sound.md`.

## 2026-10-17 Study A: sound effects (new note `sound.md`)

**Changed:** New `docs/behaviour/sound.md`. `STATUS.md`. No code. Scratch notes and a copy of the community `kq8aud.txt` in `re/` (not committed).
**Why:** The user asked for a study of how sound effects behave, so B can play Daventry's sounds as the original does.
**Tested how:** Traced the sound engine in `Mask.exe` (file opening and decoding, mixing, groups, mix limit, placed sounds, fades, the script commands, cues, footsteps, hurt sounds, the `KQAudibleObject` class, the option sliders). Measured with throwaway scripts (in the scratch folder): headers of all 4,598 SOL files, a full decode of the 2,498 compressed ones, every sound name in Daventry's and `GAME`'s data against the files.
**Result:** The community `.aud` note is right on header, step table and decoding; corrections: the game ignores header byte 1 (data always at byte 14), never uses the 16-bit/signed bits after opening, and retries the name as `.AUD` then `.WAV`. All files are compressed 16-bit, no file overflows. Placed sounds: loudness from Connor's distance, balance from the camera; all one-shots use 5,000 / 25,000; the curve forgets to subtract min (sudden drop at min, silent before max, reproduced in the original). Mix limit 8 (a 9th can start; the oldest lowest-group non-looping sound is stopped). Six groups (effects, menu, speech, menu music, world music); shipped sliders 0.34 / 0.14 / 0.14; speech ducks effects and music to 0.2. Daventry's six looping sound objects listed with their numbers; they switch off beyond 27,100 (the near/far/off test).
**Still broken / not tested:** Nothing heard in the original by me. Not settled: which sign of the camera's sideways axis is "right" (open question 1, the user can check by ear), whether to keep the original distance curve (open question 2, the user decides), what makes some speech lines skip ducking, which screens pause the game, eight other engine places that play placed one-shots (same numbers). Music and speech only summarised.
**Next:** B implements sound playback from `sound.md` (plan in `docs/DESIGN.md` first). The user: answer open questions 1 and 2 of `sound.md` when convenient.

## 2026-10-17 M1 step 7k: light-map placement check (A's answer to Q79)

**Changed:** `crates/kq8-cli/src/lighting_verify.rs` (`rect_check` and the summary): per surface corner, the light texel position before the clamp; P0 minus the drawn texel (0, 0) in texture texels; our painting's texel positions against the drawn ones. `docs/DESIGN.md` (7k), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q80). `wall_light.rs` unchanged.
**Why:** A's answer to Q79 (`object-lighting.md` rules 13, 16, "How to check"): the drawing already matches the original, so check it with numbers instead of changing code.
**Tested how:** `cargo run --release -p kq8-cli -- lighting-verify`, read `kq8-artifacts/logs/lighting-verify.log`. Clippy clean; `wall_light` unit tests pass (5).
**Result:** 164,838 surfaces, 656,181 corners, **0 corners outside** the map (matches). P0 − drawn texel (0, 0): **mean (4.000, 4.000)** texture texels (matches). Our painting puts texels (0, 0) and the last one within 0.05 texture texel of the drawn ones on 164,528 of 164,838 surfaces; the other 310 are surfaces whose stored P0 is itself off (up to 10 texels). No `.dil` has a map 1 texel wide or high, so that case never happens. Painting matches "Do not reproduce": no fix. RESULT: PASS.
**Still broken / not tested:** Two of the note's counts were not reproduced: surfaces with a corner at exactly 0 (note 7,511; we get 365 on x and y, 7,276 on x or y) and P0 within 0.05 of (4, 4) (note 160,649; we get 164,640). Tried 64-bit floats, a 0.001 tolerance, each axis alone, 3D distance instead of per axis: none gives the note's numbers. Logged as "DIFFERS (Q80 open)", not a failure. Asked A as Q80. Nothing in the viewer changed, so nothing to see in the game.
**Next:** A answers Q80 (how those two were counted).

## 2026-10-17 Study A: answer Q79 (where a wall's light texels sit)

**Changed:** `docs/behaviour/object-lighting.md`: rule 13 (how the drawing places the light map), rule 16 (P0 is not the drawn texel (0, 0)), "Do not reproduce" (texel positions written out), "How to check" (light-map placement), "Answers" (Q79). `docs/behaviour/QUESTIONS.md`: Q79 answered. `STATUS.md`. No code.
**Why:** B measured that the `.dil`'s stored P0 sits 4 texture texels past rule 13's first light texel and asked which one is the drawn position.
**Tested how:** Traced the wall drawing in `Mask.exe` (how the light-map coordinates sent to the card are made, how the map is copied to the card, how P0 is read). Measured on all `.dig`/`.dil` pairs with a throwaway script (in `re/`, not committed).
**Result:** Rule 13 is right: the game adds half a light texel to the coordinates so the card's texel centres land at 16 i − offset_x. On all 164,838 surfaces (148 files), no corner falls outside the map's first or last texel centre with that placement (7,511 touch texel 0 exactly); with "+ 4" 47,226 would. P0 is read as stored, used only for point lights, never for drawing; its quarter texel comes from the files (no `.dil` builder in the game), reason unknown. B's current choice stands; **no change to how light maps are drawn**.
**Still broken / not tested:** The card's texel-centre convention (half a texel in) is hardware behaviour, not read in the exe; either way the stored P0 doesn't match the drawn texel. Why the files shift P0 is not known. Only the Glide two-pass path was read.
**Next:** B (optional): log the placement check in "How to check" (expected 0 corners outside, 7,511 at exactly 0, P0 offset (4, 4)).

## 2026-10-17 Fix: Bevy's dithering warning at start

**Changed:** `kq8-viewer`: both cameras (`terrain_view.rs` world view, `main.rs` model view) get `DebandDither::Disabled` next to `Tonemapping::None`.
**Why:** The user saw Bevy's warning "`Tonemapping::None` with `DebandDither::Enabled`, so dithering has no effect". Dithering only works with tone mapping, which we keep off so colours stay as computed (M1 step 4b); it did nothing.
**Tested how:** `cargo clippy -p kq8-viewer` clean; `cargo play --world Daventry --frames 60` and `cargo play --model smlchest.KQS --frames 60`, terminal output searched for the warning.
**Result:** The warning no longer appears (0 times in both runs); RESULT: PASS. No change to the picture expected (the setting had no effect). Checked by the user (2026-10-17): the warning is gone.
**Still broken / not tested:** —
**Next:** —

## 2026-10-17 `cargo play`: a short command for the viewer

**Changed:** New `.cargo/config.toml` with the alias `play = "run --release -p kq8-viewer --"` (first named `viewer`; renamed at the user's request); `.gitignore` whitelists it; `README.md` ("Try it") and `docs/FEATURES.md` ("Launch options") use `cargo play`. The user's own README edits (feature list, F9 row) kept.
**Why:** The user asked for a way to run the viewer without `-p kq8-viewer`. Making it the workspace's default member would have broken `cargo run -- vol-verify` (the checks), so an alias instead.
**Tested how:** `cargo play --world DeadCity --dynamic on --frames 60` (and the same with the first name, `viewer`); `viewer.log`.
**Result:** Exit 0; "world: DeadCity", "Shadow map dedshado.bin (--dynamic on)", "RESULT: PASS". `cargo run -- <check>` unchanged.
**Still broken / not tested:** —
**Next:** —

## 2026-10-17 User's check of F9 (M1 7j)

**Changed:** `STATUS.md`, `docs/FEATURES.md`: F9 marked as checked by eye. No code.
**Why:** The user tried the key.
**Tested how:** By eye, by the user.
**Result:** "The animated lights go away when pressing F9, leaving only the baked lights. Everything comes back when pressing F9 again."
**Still broken / not tested:** Q79 open.
**Next:** Q79 once A answers; then M3/M4 work.

## 2026-10-16 M1 step 7j: F9 switches dynamic lighting

**Changed:** `kq8-viewer`: `load.rs` loads both ground maps (the one shown and the other of the pair, `Ground::other_light`); `light_view.rs` `GroundLight::toggle_dynamic` and the F9 system `toggle_dynamic` (swaps the maps, rewrites the ground picture and its lookup, forgets last frame's patches, flips the switch the ground's and walls' point lights read), the label shows "dynamic lighting on/off (F9)", one unit test; `main.rs` runs it in the lighting chain before the ground and walls are painted. Plan in `docs/DESIGN.md`; provenance.
**Why:** The user asked for a hotkey. In the original the setting also picks the ground map (`terrain-lighting.md` rule 2), so F9 switches that too.
**Tested how:** `cargo test -p kq8-viewer` (11), clippy clean. Viewer `--world Daventry --frames 150|200` (three runs), `--world DeadCity --frames 150`; `viewer.log`.
**Result:** Both maps load: Daventry "Light map davelght.bin (Options.cs useDynamicLighting 0)" and "Shadow map daveshdw.bin (kept for F9)", Dead City `dedlight.bin` / `dedshado.bin`, all 513 × 513. Frame times 7.31, 11.83 and 15.28 ms in Daventry: the same noise as in 7g (one run matches the earlier 7.3–7.4 ms).
**Still broken / not tested:** **The key press itself is not tested** (no test option presses keys); checked only through the unit test of the swap.
**Next:** You: press F9 in Dead City by the portal and in Connor's hut.

## 2026-10-16 User's check of M1 7d–7i

**Changed:** `STATUS.md`, `docs/FEATURES.md`, `README.md`: marked as checked by eye. No code.
**Why:** The user compared the viewer's results.
**Tested how:** By eye, by the user.
**Result:** "looks great".
**Still broken / not tested:** Q79 open.
**Next:** F9 (7j), the user's request.

## 2026-10-16 M1 step 7i: reach to a clicked building in its own axes

**Changed:** `kq8-core/src/active.rs`: `ActiveBuilding` carries its `.dig` box and placement instead of its world box; `click_building` measures with `distance_to_box` (the objects' function) and logs "distance to its .dig box (its own axes)"; one new unit test. `kq8-viewer/src/load.rs` fills the two fields from the collider. Provenance.
**Why:** Q75 answered by A (`clicking.md` rule 9): the distance from Connor to the `.dig` header box in the building's own axes. The task said our stand-in already matched; it was close but not the same: it measured to the axis-aligned box around the **turned** box, which is larger for a turned building, so Connor could be "in reach" up to a few hundred units too early.
**Tested how:** `cargo test --workspace` (189 + 95 + 10), clippy clean. Viewer `--world Daventry --at 202336,262000,1.5708 --aim smlchst3,20 --item INVITEM_Crystal --frames 400`; `doors.log`.
**Result:** Unit test: Connor at (650, 50) by a 1,000-unit box turned 45°: 424.3 outside it (the old world box said 0). Viewer: "click building Pub: … distance to its .dig box (its own axes) 22.6, reach 4700: IN REACH; holds INVITEM_Crystal …: Connor's "that doesn't work" remark".
**Still broken / not tested:** The out-of-reach case was run only in unit tests, not in the viewer. Not compared with the original.
**Next:** Session end: STATUS, FEATURES, README.

## 2026-10-16 M1 step 7h: a building's draw-distance ball (relabel and log)

**Changed:** `kq8-viewer/src/draw_distance.rs`: comments and the ball's source label ("building .dig box (rule 40)" instead of "Q76 stand-in"); `terrain_view.rs`: one log line per building with its radius and how far its centre is from its position. No behaviour change. Provenance.
**Why:** Q76 answered by A (`object-lighting.md` rule 40): radius = half the `.dig` header box's diagonal; the note recommends the box middle turned with the building as centre (the original uses the position). Our "stand-in" already used the `.dig` header box (a building's `Model` box), so it is the recommended ball.
**Tested how:** `cargo build`; viewer `--world Daventry --frames 100`; `viewer.log`.
**Result:** 26 buildings "building .dig box (rule 40)"; radii from 2,666 (`cav2`) to 18,990 (`Church`); `csbridge` radius 4,398, centre 13,992 from its position (note: 4,400 and 14,000).
**Still broken / not tested:** Not checked by eye (it only shows for a building near the `visibleDistance` edge whose walls the haze hasn't removed).
**Next:** 7i, reach to a clicked building (Q75).

## 2026-10-16 M1 step 7g: `KQPointLight` always shines

**Changed:** `kq8-viewer/src/terrain_view.rs`: `KQPointLight` entities get no `DistanceTest` any more (log line "KQPointLight: n loaded, always shining"); `scene_lights.rs`: `PointLightSource` carries its name, and `gather_lights` takes every one that isn't deleted. Provenance.
**Why:** Q77 (b) answered by A (`object-lighting.md` rule 37): a `KQPointLight` is never in the near / far / off test and always shines. Our stand-in switched Dead City's far lights off.
**Tested how:** `cargo build`; viewer `--world DeadCity --frames 300` (five runs) and `--dynamic on`; `viewer.log`. Frame times compared with the code before 7d (same command, two runs).
**Result:** "scene lights: 8 (7 KQPointLight, 1 carried by models …)" from the start (before: 2, one `KQPointLight`); the ground log now paints `Facelt03`–`Facelt08` too (25–42 points each). Average frame 6.87–6.94 ms (before 6.87–7.00); two runs at 10.8 and 17.7 ms were not repeatable (something else on the machine).
**Still broken / not tested:** Not checked by eye. No building is lit by the `Facelt` lights from Dead City's start (no drawn building there).
**Next:** 7h, the building's draw-distance ball (Q76).

## 2026-10-16 M1 step 7d: point lights on building surfaces

**Changed:** `kq8-data`: `dil.rs` keeps each surface's light-map rectangle (P0, U, V; `LightRect`); new `wall_light.rs` (facing, "in front", reach, line end, texel positions, painting; unit tests). `kq8-viewer`: new `wall_lights.rs` (`WallData`, `WallLights`, `light_walls`, run every frame after the ground); `building_light.rs` / `.wesl`: each wall corner carries its surface index, a per-surface haze factor list at binding 6, haze capped at 0.99 then × factor before the 0.99 test; `terrain_view.rs` gives each building copy its own light picture, factors and materials; `load.rs` keeps the `.dil` per building and reads `--dynamic`; `main.rs` new option `--dynamic on|off`. `kq8-cli` `lighting-verify`: rectangles against rule 13, facing against corner order. Q79 added. Plan in `docs/DESIGN.md`; provenance.
**Why:** M1 step 7d, A's answer to Q78 (`object-lighting.md` rule 16); the user's check in the original: a Daventry fire lights walls and floor with dynamic lighting on, not with it off.
**Tested how:** `cargo test --workspace` (188 + 95 + 10), clippy clean, `kq8 lighting-verify` (PASS). Viewer `--world Daventry --frames 200` (your `Options.cs` has `useDynamicLighting 0`), the same with `--dynamic on`, `--world DeadCity --at 182213,42600,3.1416 --dynamic on --frames 400`; `viewer.log`.
**Result:** Daventry with dynamic lighting on, from the start: "[fireset.01] … Begin 700 End 850: tested 3079, skipped 1277, not in front 1028, out of reach 730, blocked 20, lit 24, largest texel change 3 ({"Cottage": 24})"; `lamp01.13` lights 36, the chandeliers 0–2; "63 surface(s) lit, 62 put back". With dynamic lighting off: "(dynamic lighting off: none) … 0 surface(s) lit". Frame 7.40 ms (no slowdown). Facing by bit 0x80 agrees with the corner order on every Daventry surface (164,835 of 164,838 in all files). **Found:** the stored P0 lies 4 texture texels (¼ light texel, 32 units) past rule 13's first texel on 164,685 of 164,838 surfaces; texels are painted at rule 13's positions; asked as Q79.
**Still broken / not tested:** Not checked by eye. **Dead City's arrival portal has no building within 29,000** (nearest `DeadHall`), so this step can't light anything there: the things around the portal are models (`spike31`–`spike36`, the portal), lit per corner since 7c. The haze × 0.04 is not seen in any log (only in the shader). Q79 open.
**Next:** You: `cargo run --release -p kq8-viewer -- --world Daventry --dynamic on`, walk into Connor's hut: the floor and wall near the fire should be a little brighter (warm) than with `--dynamic off`. Then 7g.

## 2026-10-16 Study (A): Q75–Q78, point lights on building surfaces

**Changed:** Notes only, no code. `docs/behaviour/object-lighting.md` rule 16 rewritten (point lights on every building surface, floors included), rules 26b, 29, 37, 40, numbers, "Do not reproduce", "How to check", answers; `clicking.md` rule 9 (reach to a building); `buildings.md` byte-0 bit 0x80 line corrected (it was backwards); `QUESTIONS.md` Q75–Q78 marked answered; `STATUS.md`. Addresses kept in `re/notes.md`.
**Why:** B needs Q78 before M1 step 7d (walls); Q75–Q77 were small leftovers.
**Tested how:** Traced in `Mask.exe` (Ghidra). Measured with throwaway scripts on the game files: `.dil` record floats (`cnnrhut.dil`), `.dig` bounding boxes (Daventry), floor facing vs bit 0x80 (4 Daventry buildings), `.kqs` files with light nodes (all 4,577).
**Result:** With dynamic lighting on, the game rebuilds every drawn building surface's light map from the `.dil` each frame, then adds each point light that is in front (measured to the map's first corner), within Begin + End of the nearest of 4 corners and centre, and not blocked by that building's own BSP; stored back as channel × 15.9 rounded down. Model lights (fireplaces) count the same. Found on the way: the original places texels U ÷ width apart, but the data spaces them U ÷ (width − 1) (|U| = (w − 1) × 128 on all of `cnnrhut.dil`), so it slightly misplaces the light (listed in "Do not reproduce"). `KQPointLight` is never in the near / far / off test and always shines. Light-node flag: only 0x1 is read. 442 shapes carry lights (174 = the 12 world folders only; `GAME/Common` has 271, 3 shared). Building ball: half the `.dig` box diagonal, centred on the building's position (box middles lie 400–14,000 from Daventry buildings' origins). Building reach box: the `.dig` header box. First reading of bit 0x80 against `buildings.md` gave "lights only light the back of walls"; the game's own culling code and the data showed the note was reversed, not the light test.
**Still broken / not tested:** Checked by the user afterwards (2026-10-16): a Daventry fire lights walls and floor with dynamic lighting on, and that light goes with it off. The other rules were not checked in the original. Whether `fireset2.kqs`'s light changes between frames was not measured. Direct3D not studied (Glide only).
**Next:** B: M1 7d (walls and floors), `KQPointLight` out of the draw-distance test, buildings' ball. You: in a Daventry house with a fire, compare walls and floor near the fire with dynamic lighting on and off (`object-lighting.md` "How to check" 9–10).

## 2026-10-16 M1 step 7e: point lights on the ground

**Changed:** `kq8-data`: `lightmap.rs` `fade_falloff`, `add_light`, `LightMap::paint_light` / `restore`, limits, unit test; `objects.rs` `Placement::turn_off_dyn_lighting`. `kq8-viewer`: `load.rs` keeps `useDynamicLighting` (`Ground::dynamic`); `scene_lights.rs` also lists the lights that reach the ground (`GroundLights`; `CarriedLights::dyn_off`); `light_view.rs` keeps the stored map, paints a working copy every frame (`paint_ground`) and rewrites only the touched texels; callers pass `turnOffDynLighting`. Plan in `docs/DESIGN.md`; provenance.
**Why:** M1 step 7e (`object-lighting.md` rule 26, `terrain-lighting.md` rule 15); the user's check in the original: the portal's light moves over the ground.
**Tested how:** `cargo test --workspace` (188 + 90 + 9), clippy clean. Viewer `--world DeadCity [--at 182213,42600,3.1416]`, `--world Daventry`, `--frames 200`; console (no errors); `viewer.log`.
**Result:** Dead City: "[PLight01] at (182213, 41595, 2250) h 1386 (End 4000): half-width 3752, 225 point(s)"; "[portal] … h 328 (End 3000): half-width 2982, 132 point(s)", the next second h 186 at x 182,347 (it moves); 357 points painted and 357 put back each frame. Daventry: the lamps and chandeliers near the start are higher above the ground than their End (1,951 > 1,500; 3,434 > 500): nothing painted there (2 points from one). Frames 7.3–7.6 ms.
**Still broken / not tested:** Checked by eye by the user afterwards (2026-10-16): "the portal and torches in Dead City look great, they cast dynamic light just like the original game". Walls don't get point lights yet (7d, waits for Q78). The light under Connor in the per-second "ground light" line uses the working map, so it includes patches.
**Next:** You: Dead City by the arrival portal (`--at 182213,42600,3.1416`): a soft green patch on the ground that moves and changes colour with the portal's light, plus PLIGHT01's steady greenish patch. Then 7d after A answers Q78.

## 2026-10-16 M1 step 7e-0: the ground's light map is bilinear

**Changed:** `kq8-viewer/src/light_view.rs`: the shown light map becomes one picture (`light_image`, one texel per map point, channel × 17, bilinear, clamped); no more vertex colours; F7 changes a setting in every ground material; unit test. `ground_haze.rs` / `.wesl`: the picture at bindings 3–4, looked up per pixel from the world position (`ground_light`), then the mode. `terrain_view.rs`, `ground_anim_view.rs` pass the ground light to `ground_material`. `kq8-data` `lightmap.rs`: `LightMap::with_size`, `set` (also for 7e). Plan in `docs/DESIGN.md`; provenance.
**Why:** A traced that the ground's light maps are filtered bilinearly (`object-lighting.md` rule 13, `terrain-lighting.md` rule 11); ours were blended per triangle. 7e will paint point lights into this picture.
**Tested how:** `cargo test --workspace` (188 + 89 + 9), clippy clean. Viewer `--world Daventry|DeadCity|Swamp|Barren --frames 150`, console checked for shader errors (none), `viewer.log`.
**Result:** Maps of 513, 257 and 129 points as pictures; the "ground light under Connor" lines unchanged (Daventry 7, 7, 7 = 47%); average frame 7.2–8.1 ms (no change).
**Still broken / not tested:** Checked by eye by the user afterwards (2026-10-16): "looks good". The F7 key not pressed by a test option.
**Next:** You: Daventry, look at hill shadow edges on the grass (they should look smooth, no diagonal steps along the ground's triangles) and press F7 to cycle the modes. Then 7e (point light patches on the ground).

## 2026-10-16 User's check of M3 3g and M1 7a–7c

**Changed:** `STATUS.md`, `docs/FEATURES.md`, `README.md`: marked as checked by eye. No code.
**Why:** The user compared the viewer with the original.
**Tested how:** By eye, by the user (Dead City portal, Daventry clicks).
**Result:** "Everything looks good"; the portal casts its light onto Connor's model.
**Still broken / not tested:** Point lights on walls and the ground (7d, 7e).
**Next:** 7e-0 (ground light map bilinear), 7d once its open points are answered.

## 2026-10-16 M1 step 7f: Q73 leftovers checked against our code

**Changed:** `ground_haze.wesl` comment only (the far-edge cut is not a stand-in any more). `docs/DESIGN.md` 7f result and new step 7e-0. Provenance. No behaviour change.
**Why:** A's answers to Q73 (`object-lighting.md` rules 13, 14, 28).
**Tested how:** Read the code: samplers in `building_light.rs`, alpha modes and discards in `building_light.wesl` / `ground_haze.wesl`, the ground's light in `light_view.rs`.
**Result:** Walls: bilinear light maps, no colour key ✓. The ground's cut at `visibleDistance` (blended corner distances, cut per pixel) is geometrically the same as the traced edge cut ✓. Differs: the ground's light map is blended per triangle, not bilinear (the note now says bilinear for the ground too).
**Still broken / not tested:** The ground's bilinear light map (planned 7e-0). Nothing run.
**Next:** 7d (walls), 7e-0 and 7e (ground).

## 2026-10-16 M1 steps 7c-2 and 7c-3: point lights and carried lights on models

**Changed:** New `kq8-viewer/src/scene_lights.rs`: `CarriedLights` (a model's lights per movement and frame, on its root entity), `PointLightSource` (an entity per `KQPointLight` with its own near / far / off test), `gather_lights` (every frame: lights of owners in the lights group, 0x400, into one fixed-size storage buffer; log once a second with what the nearest light gives Connor), unit tests. `object_light.rs` / `.wesl`: the buffer at binding 3 of every object material, a new vertex attribute `ATTRIBUTE_PART_BALL` (the part's ball) read through `specialize`, the shader's own vertex input, rule 24 in `corner_light`. `main.rs` `model_mesh` writes the part ball; `kq8-data` `Triangle::part_ball` (middle of the mesh node's vertices in that frame, the mesh's radius). `draw_distance.rs` fills `LightsGroup`; `terrain_view.rs` (still models, point light entities), `anim_view.rs`, `door_view.rs` attach and update `CarriedLights`. Provenance.
**Why:** M1 step 7c (`object-lighting.md` rules 3, 24, 26b, 37); the user's check in the original: the portal's moving green light lights Connor.
**Tested how:** `cargo test --workspace` (188 + 89 + 8), clippy clean. Viewer: `--world DeadCity --frames 200`, `--world DeadCity --at 182213,42600,3.1416 --frames 300`, `--world Daventry --frames 200`; console checked for shader errors (none); `viewer.log`.
**Result:** Dead City start: 2 lights (`PLight01`, the portal's), the portal's 3,225 from Connor (End 3,000): nothing on him. By the portal: "[portal] 877 from Connor … falloff 1.000, adds up to (0.200, 0.200, 0.013)", the next seconds (0.110, 0.200, 0.030), (0.059, 0.200, 0.042) as the light circles and changes colour. Daventry: 5 lamp lights gathered at the start (Begin 1,000, End 1,500), average frame 7.38 ms (before 7.94 ms on Vulkan; no slowdown measured).
**Still broken / not tested:** **Not checked by eye** (the GPU result can't be read back; the log shows the CPU copy of the same rule). Walls and ground don't get these lights yet (7d, 7e). Point lights' place in the draw-distance test is a stand-in (Q77 b); the part ball's centre is the note's stand-in. Daventry's lamps, candles and fireplaces now light models within 1,500 or so, which is new there.
**Next:** You: Dead City, stand Connor by the arrival portal and compare with the original (a green light circling, changing colour, on his side facing the portal). Then 7d (walls).

## 2026-10-16 M1 step 7c-1: the lights models carry, and `KQPointLight` objects (data only)

**Changed:** `kq8-data/src/shape.rs`: `ModelLight`, `LightNode::model_light`, `Shape::sequence_lights` (rule 26b), unit test. `kq8-cli/src/lighting_verify.rs`: new section "Lights carried by models" (shapes with light nodes, flag values, `portlite.kqs` frame by frame). `kq8-viewer/src/load.rs`: `Model::lights` (shown frame) and `Animation::frame_lights` (per frame); `loadKQ light/…` finds loose files in `data/<World>/KQ/`; `KQPointLight` objects are kept by name (`Objects::point_lights`) with a marker. Q77 added. Provenance. Nothing is drawn differently yet.
**Why:** M1 step 7c, from A's answers to Q72 (`object-lighting.md` rules 24–26b).
**Tested how:** `cargo test -p kq8-data shape` (6 pass); `kq8 lighting-verify` (PASS); viewer `--world DeadCity|Daventry --frames 30`, `viewer.log`.
**Result:** `portlite.kqs`: 17 frames, one light each, colour (0.200, 0.200, 0.013) → (0.000, 0.200, 0.111), Begin 1,500, End 3,000, as A's example. First run read every node as sun-type: I had taken "flag bit 1" as 0x2, but the data has only flags 1 and 5; changed to 0x1 (Q77 a). Dead City: the 7 point lights now load (before: "light/Plight01.kq not found"); 8 still models and 6 animations carry lights (portal, torches, coals, teleport). Daventry: 13 still models, 11 animations (candles, lamps, chandeliers, fireplaces, signpost). 442 shape files carry light nodes (A: 174; Q77 c).
**Still broken / not tested:** Not drawn yet (7c-2, 7c-3). A point light's place in the near / far / off test is a stand-in (Q77 b).
**Next:** 7c-2, gathering the lights each frame.

## 2026-10-16 M1 step 7b: the collider switch, `setActiveCamera`, `makeActive`

**Changed:** `kq8-core/src/active.rs`: a set of objects whose collider is out (`switch_collider`, `is_switched_off`): not picked, not clicked; `is_hidden`, `is_deleted`; commands `setActiveCamera` and `makeActive` → `Request::ActiveCamera` / `MakeActive`; one new unit test. `kq8-viewer`: `draw_distance.rs` treats hidden objects as off and `makeActive` objects as near, measures from the camera after `setActiveCamera 1`, and records every gain or loss of bit 0x8; new fixed-update system `apply_collider_switches` (first in the update chain) takes colliders and view boxes out by name and puts them back; `door_view.rs` applies the two requests (`ActiveRules`) and no longer rewrites a switched-off door/chest/object collider; `anim_view.rs` leaves a switched-off moving collider off; `game_camera.rs` skips switched-off doors as view blockers; items' tests carry the item's own name. Provenance.
**Why:** Q74 answered by A (`object-lighting.md` rules 36, 41, 42).
**Tested how:** `cargo test --workspace` (188 + 88 + 6), clippy clean. Viewer `--world Daventry --frames 200`; `--at 285098,203500,0 --hold 1,0,0,150 --frames 600` (walking towards `spinweel`, `tooFar` 0.3); `viewer.log`.
**Result:** Daventry start: 335 colliders switched out in update 0 (322 off, 13 far, e.g. `Condoor`, three far items, `spinweel` at 9,314). Walking in: "update 15: collider switch: [spinweel] Near at ball distance 8768: collider back".
**Still broken / not tested:** `setActiveCamera` / `makeActive` only unit-tested (only Dead City's cut scenes and switches use them; not run in the viewer). **Behaviour change:** hidden objects no longer block Connor (before: solid, our earlier reading); this follows `objects.md` "Hidden objects" 3 and rule 41 but is not checked in the original. Buildings far away stay solid (not needed). Not checked by eye.
**Next:** 7c, lights on models (`KQPointLight` and the lights `.kqs` models carry).

## 2026-10-16 M1 step 7a: draw-distance balls from `BOUNDS.KQ`, items' far mask 0

**Changed:** `kq8-viewer/src/draw_distance.rs`: `ShapeBalls` now holds the `BOUNDS.KQ` table and finds each ball by collision shape (lower-cased), then own name, else radius 100; buildings keep their geometry ball (stand-in, Q76); items get their type's shape, `tooFar` 0.83 and masks 0x2 / 0x448 / 0; every test records where its ball came from, logged once a second. `load.rs` passes the table (`Objects::game_bounds`); `terrain_view.rs`, `anim_view.rs` (by the drawn shape), `door_view.rs` (by the movement's shape), `item_view.rs` (by the item type's shape) use it. Q76 added. Provenance.
**Why:** Q70 and Q71 answered by A (`object-lighting.md` rules 35, 43; `items.md` 5b): the ball comes from `BOUNDS.KQ`, never the model; items are not drawn when far.
**Tested how:** `cargo build`; viewer `--world Daventry --frames 200`; `--world DeadCity --at 130800,172122,-1.5708 --hold 1,0,0,90 --frames 500` (the 6h flame walk; angles 0, ±1.5708 and 3.1416 tried first to find the one that walks west); `viewer.log`.
**Result:** Daventry from the start: 368 objects, 32 near, 13 far, 323 off (6h: 32 / 12 / 324); balls: 297 by shape, 21 items by type shape, 26 buildings (stand-in), 24 radius 100; 3 far items now "Far but not drawn … far 0x0". Dead City: `flame.05` Off → Far at 12,097, Far → Near at 10,041 (6h: 12,098 / 10,054; frame sampling).
**Still broken / not tested:** Not checked by eye. Far items are not drawn but can still be picked by the click line (their collider stays until 7b). Buildings' ball is a stand-in (Q76).
**Next:** 7b, the collider switch, `setActiveCamera`, `makeActive` (Q74).

## 2026-10-16 M3 step 3g-5: `deleteObject` for every object, `NoName<n>`, pickup sound on every add

**Changed:** `kq8-core/src/active.rs`: `deleteObject` is its own command (`delete_objects` / `delete_one`) for every kind of object and every listed name, with the original's error line and result; doors and chests get a `gone` flag (no collider, not picked or clicked); one set of deleted names drops events. The pickup sound is logged on every add to Connor's inventory (`pickup_sound`). `item.rs`: new items are named `NoName<n>` from 0. `kq8-viewer/src/door_view.rs`: deleted doors are hidden. Tests updated for the new names and log lines; one new test. Provenance.
**Why:** A's answers: `clicking.md` rule 22 (Q64 d), `items.md` rules 7 and 9 (Q60, Q61). Before, `deleteObject` on doors, chests and other placed objects was a stub, items were named `Crystal.1` etc., and a chest's 2nd click logged "pickup sound not stated".
**Tested how:** `cargo test -p kq8-core` (187 pass). Viewer Daventry: `--at 202336,262000,1.5708 --click smlchst3,10 --click smlchst3,40 --click smlchst3,45 --frames 1200`; `--at 208502,209600,1.5708 --click fishies.01,10 --click fishies.01,40 --frames 900`; `items.log`, `doors.log`, `traps.log`.
**Result:** Start items `NoName0` … `NoName21` (23 made in that run with the chest's crystal `NoName22`). Chest: 2nd click "NoName22 moved into Connor's inventory (INVITEM_Crystal 0 -> 3); pickup sound energy.aud". Fish: deleted in update 10, its cue dropped in update 25, the click in update 40 does nothing, its collider switched off.
**Still broken / not tested:** No sound at all yet (only logged). Deleting a building leaves its walls solid (not needed in Daventry). The name counter isn't game-wide across worlds or saved (no travel or saving yet). Deleting a door was tested only in a unit test, not in the viewer.
**Next:** M1 step 7a, the draw-distance ball and items' far mask (Q70, Q71).

## 2026-10-16 M3 step 3g-4: `EndLoop` through the event queue, doors too, `getEndLoop`

**Changed:** `kq8-core/src/active.rs`: a finished play-once movement of a door, chest or scripted object now goes through `movement_ended` (cue 3333 to itself through the queue, if it has a script; `Cue 300 0 <movement>` to its `getEndLoop` listener); `handle` turns cue 3333 into `EndLoop <movement>` (`call_named`, any object kind with a script) and logs cue 500; new `getEndLoop` command; the five copies of the cue-argument code are one `cue_args`. Two tests now expect `EndLoop` one update later; one new test. Module notes, provenance.
**Why:** Q64 (c) answered by A (`clicking.md` rule 21): `EndLoop` goes through the event queue, doors get it too, and `getEndLoop` exists (used by `bricks.cs` and five more Daventry scripts; it was an unknown command before).
**Tested how:** `cargo test -p kq8-core` (186 pass). Viewer `--world Daventry --at 233848,203000,1.5708 --click basket1,10 --frames 500`, `doors.log`.
**Result:** `basket1`: click in update 10, `open` starts, "movement 'open' ended: cues itself 3333" in update 11, "EndLoop open" and the script call in update 12. Unit test: door gets `Open` at once, then `EndLoop open` and its own listener's `Cue 300 0 open` one update later; after `getEndLoop … 0` only `EndLoop close`.
**Still broken / not tested:** `getEndLoop Connor …` (`bricks.cs`, `ax.cs`, …) is only recorded: Connor's scripted movements (`push` via `KQMonster::setState`) don't exist yet, so nothing is relayed. Nothing checked by eye.
**Next:** 3g-5, `deleteObject` for every object, `NoName<n>`, the chest's pickup sound.

## 2026-10-16 M3 step 3g-3: accept lists and `KQObject::setAccepts`

**Changed:** `kq8-core/src/item.rs`: new `Accepts` (read a value, answer by type) and a unit test. `active.rs`: each object's accept list is kept by name, read at `start()` (logged) or when first asked; `KQObject::setAccepts` adds to it; a click with an item in hand asks it; `$4` is the type's own spelling; door test helper now has a type list; new unit test. Provenance.
**Why:** Q62 (a) answered by A (`clicking.md` rules 13–18). Before, the whole `acceptsInventory` value was compared with the held item's type as text, so a comma list or `INVITEM_All` never accepted.
**Tested how:** `cargo test -p kq8-core` (185 pass). Viewer `--world Daventry --frames 60` (accept lists in `doors.log`); keep door `--at 217415,279800,1.5708 --click keepdrl,20 --frames 300` with `--item INVITEM_KeepKey`, `keepkey`, `INVITEM_Crystal`.
**Result:** Daventry: 7 objects accept something (`keepdrl`/`keepdrr` KeepKey, the three bell stands LadyBell, `starball.01` SpellPage, `poorbox` SilverCoins); no unknown names. Key (both spellings): script runs, both keep doors get `KQDoorEvent 0`. Crystal: "not accepted (accepts INVITEM_KeepKey)". First run with `keepkey` was accepted but the script got `$4 = keepkey` and did nothing; fixed by passing the type's own name (rule 18).
**Still broken / not tested:** No Daventry object uses a comma list or `INVITEM_All` among the loaded clickable ones; those cases are covered by unit tests only. `setAccepts` is used only in Gnome and Barren (not run in the viewer).
**Next:** 3g-4, `EndLoop` through the queue and `getEndLoop`.

## 2026-10-16 M3 step 3g-2: the click line hits buildings, not the ground

**Changed:** `kq8-core/src/active.rs`: buildings are click targets (`ActiveBuilding`, `add_buildings`, `clickable_building`), `pick_line` replaces `pick_seen` (nearest clickable box or clickable building, via the new `ClickWalls` trait), `click_building` (door calls; no script → `ClickResult::Swallowed`), two new unit tests replacing the line-of-sight one. `kq8-viewer`: `load.rs` builds each building's click keys (`tooFarAwayAction` default 1 for buildings), `door_view.rs` tests the clickable buildings' BSP for the click line and no longer the ground or view blockers; start-up log line "clicks: …". Q75 added to `QUESTIONS.md`. Provenance.
**Why:** Q63 answered by A (`clicking.md` rules 1–12): buildings are type 0x28 by default, which has the click bit, so walls take the click; the ground is never tested; `colliderType=136` buildings are not clickable.
**Tested how:** `cargo test --workspace` (183 + 88 + 6 pass), clippy clean. Viewer `--world Daventry --frames 400` with the 3e-4b checks: `--at 202336,262000,1.5708 --aim smlchst3,20`, `--at 266011,221900,0 --aim Mushroom.14,20`, `--at 270813,225311,-1.5708 --aim Condoor,20`; read `doors.log`.
**Result:** "23 of 26 buildings take clicks (not: Church, Cave, WizHouse)". Pub ground floor: "hits building Pub first, at 5319 units; it hides smlchst3 at 5974" → Swallowed, "nothing happens, Connor doesn't turn". Mushroom.14: hit at 4418, picked up. Condoor: hit at 5520, script runs.
**Still broken / not tested:** Not checked by eye. No test clicks through the church or wizard's house, or over a hill (the ground case; `clicking.md` "How to check" in the original). Reach to a building is our stand-in (Q75). Note: `--frames 90` was too short for `--aim …,20` (only update 0 reached); 400 works.
**Next:** 3g-3, accept lists (Q62).

## 2026-10-16 M3 step 3g-1: `test` leaves −1 for a false text `==`

**Changed:** `kq8-core/src/script.rs`: new `test_return` (what `test` leaves in `$Console::Return`), used by the runner; module notes updated; new unit test; the older test `console_return_and_if_on_any_command` now expects `-1` for `test a == b`. Plan for 3g (Q54, Q60–Q64) and M1 step 7 (Q70–Q74) written in `docs/DESIGN.md`. Provenance.
**Why:** Q54 answered by A (`doors.md` "Q45" e): a false text `==` leaves `-1` when the left word comes first in byte order.
**Tested how:** `cargo test -p kq8-core` (182 pass).
**Result:** `test abc == abd` → −1, `abd == abc` → 1, `ab == abc` → −1, `abc != abc` / `abd < abc` / `1 == 2` → 1; `if` still takes −1 as false.
**Still broken / not tested:** No script in the data reads `$Console::Return` as a number after a text test (A's search), so nothing visible changes. Not run in the viewer.
**Next:** 3g-2, what the click line hits (Q63).

## 2026-10-16 Docs: `--renderer` in README and FEATURES

**Changed:** `README.md` ("Try it": how to pick Vulkan or DirectX 12), `docs/FEATURES.md` ("Viewer helpers": what it does and how it was checked). No code.
**Why:** The user asked for the option to be described in both.
**Tested how:** Read the text back; no code changed.
**Result:** Both files describe the option.
**Still broken / not tested:** Linux and other graphics cards still not tested.
**Next:** —

## 2026-10-16 Viewer option `--renderer auto|vulkan|dx12`

**Changed:** New `crates/kq8-viewer/src/renderer.rs` (the option, Bevy's `RenderPlugin` set to the chosen API, a `viewer.log` line with the API, card and driver actually used, unit test `parse_names`); `main.rs` reads the option and adds the plugin. Plan and result in `docs/DESIGN.md` ("choosing the graphics API"); `FEATURES.md`, `README.md`, provenance.
**Why:** The user asked to choose between Vulkan and DirectX 12 at launch. Before, only the `WGPU_BACKEND` environment variable could do that. Our own option; nothing from the original game.
**Tested how:** `cargo clippy` clean, `cargo test -p kq8-viewer` (6 pass). `--model smlchest.KQS --frames 60` with each value, and `--world Daventry --frames 150` on dx12 and vulkan; read `viewer.log`.
**Result:** auto → Vulkan, vulkan → Vulkan, dx12 / directx → Dx12, all "RESULT: PASS". `WGPU_BACKEND=dx12` with auto → Dx12; with `--renderer vulkan` → Vulkan (the option wins). Daventry: Dx12 average 8.81 ms per frame, Vulkan 7.94 ms; no shader errors on either. `gl` and unknown values are refused with a message (exit 1).
**Still broken / not tested:** Checked by the user afterwards (2026-10-16): "it works" on both. Not tested on Linux or on another graphics card. OpenGL not available (Bevy built without it; `WGPU_BACKEND=gl` crashed earlier with "Unable to find a GPU"). Only tested on one NVIDIA card. On a PC without the chosen API Bevy stops with its own "Unable to find a GPU" message (not tested).
**Next:** When possible: try another graphics card and Linux (Vulkan).

## 2026-10-17 Study (A): last open lighting points

**Changed:** `docs/behaviour/object-lighting.md`: rule 13 (light maps filtered bilinear), rule 14 and open question 7 (no colour key on walls), rule 26b (model lights placed by the object's placement only; sun-type aims in world axes), rule 28 (where the ground's far-edge cut falls), open questions 1, 7, 10, 11 closed, answers list. `QUESTIONS.md` Q73 row. Scratch notes in `re/notes.md` (not committed). No code.
**Why:** The user asked to study what is still open after the previous commit.
**Tested how:** Traced in `Mask.exe` with the Ghidra tools: the card's texture-filter and colour-key calls and where the wall and ground drawing set them; the cutter's helper (the case for the ground's own far-edge mark); how a model light's position is placed.
**Result:** Light maps bilinear (set once at start-up). Walls and the ground switch the colour key off for every surface. The ground's cut is at t = (visibleDistance − d₁) ÷ (d₂ − d₁) along the edge, everything blended with t. Model lights: object placement only, no frame transform; sun aims not turned. A first reading in this session had the renderer's function table shifted by one entry and concluded "colour key on for walls"; checking the table against a known entry showed the shift, and the conclusion was reversed before writing it down.
**Still broken / not tested:** Open and only checkable in the running game: Direct3D (open question 6), the flames' 11,000 vs 12,100 (9). Nothing checked by eye.
**Next:** Session B: as before (point and model lights first). The note's open list for lighting is now only 6 and 9.

## 2026-10-17 Study (A): lights carried by models located; the user's Dead City check

**Changed:** `docs/behaviour/object-lighting.md` rule 26b (the `.kqs` light nodes and light sub-sequences are the lights models carry; version 1–3 defaults; data), "How to check" 8 (the user's result), open question 11 narrowed. `QUESTIONS.md` Q72 row. No code.
**Why:** The user checked Dead City's portal: the green light moves and changes colour, a smaller patch stays with dynamic lighting off, and Connor is lit by it either way, which our viewer doesn't do. `PLIGHT01` has a fixed colour, so the changing light had to come from elsewhere.
**Tested how:** `portal.kq` sits at 182,186 / 41,007, under `PLIGHT01`, with the model `portlite.kqs`. Dumped its light nodes with a throwaway script in the session scratchpad (mirrors B's `shape.rs` header layout); counted shapes with lights over all worlds. Traced the game's version-3 light reader (level and scale set to 1.0).
**Result:** `portlite.kqs`: 17 point lights, one per frame (1.07 s loop), circling about 400 units round the portal, colour cycling yellow-green to blue-green, Begin 1,500, End 3,000, strength 0.2. 174 shapes carry lights. The user's three observations match the rules: moving light (model light), baked patch with dynamic lighting off (light maps), Connor lit either way (models ignore the setting).
**Still broken / not tested:** Whether a frame's own transform moves its light was not checked. The baked patch was not measured in the maps.
**Next:** Session B: model lights (rule 26b) and `KQPointLight`s on models first (that is what the user saw on Connor), then walls and ground.

## 2026-10-17 Study (A): lighting leftovers, Q70–Q74 answered

**Changed:** `docs/behaviour/object-lighting.md`: rules 10 (unlit switch never on), 14 (`0x1000` walls keep the light map), 16 (point lights on walls: keys matched, End is a fade length, × 0.04 per light), 23 (palette entries 10–244, table per world), 24–26 (point lights on models per part, data, the ground's dynamic light), new 26b (lights carried by models, `turnOffDynLighting`), 28 (ground cut), 35 (the ball is always a `BOUNDS.KQ` box ball, centre not turned), 37 (bits 0x400 lights group, 0x2, 0x1000), new 41–43 (collider switch, `setActiveCamera` / `makeActive`, items), numbers, data, do-not-reproduce, how to check, answers, open questions 10–11. `items.md` rule 5b, `objects.md` (bit 0x400 corrected), `terrain-lighting.md` rule 15. `QUESTIONS.md`: Q70 answered, Q71–Q74 added and answered. Scratch notes in `re/notes.md` (not committed). No code.
**Why:** The user's session prompt: the haze, draw distance and lighting work (M1 4–6h) left Q70, items' near/far values, point lights and five open points.
**Tested how:** Traced in `Mask.exe` with the Ghidra tools (bounding ball, item defaults and the world's item list, point light reader and the model / wall / ground light code, palette loader, clipper, the commands). Data checks with throwaway scripts in the session scratchpad: `.ppl` chunks and the nearest palette entry for every world's `resetHazeColor`; `BOUNDS.KQ` for the flames; grep for `torchlight`, `turnOffDynLighting` (39 files), `setActiveCamera`, `makeActive`; a byte scan of the program for stores into the "unlit" field.
**Result:** Q70: both cases use a `BOUNDS.KQ` box (collider box, else by `shapeName`), centre = box middle, not turned; radius 100 without a box. Q71: items have far mask 0 and `tooFar` 0.83. Q72: `tsAttenuationBegin` / `End` matched; models fade Begin→End, walls and ground Begin→Begin+End; out of range per model part. Q73: the `pcmt` chunk is never read (the `head` count stops the loader first), so entries 10–244; in most worlds the haze entry equals the background entry (Daventry 17/17, Dead City 10/10). Q74: as in rules 41–42.
**Still broken / not tested:** Nothing checked in the original. Not found: the exact cut point at the ground's far edge (stand-in given), where models' own lights are stored in the `.kqs`, the centre of a model part's light ball versus B's decoded fields, see-through black on walls.
**Next:** Session B: point lights in Dead City (models, walls with haze × 0.04, ground patches), the collider switch with near / far / off, `setActiveCamera` / `makeActive`, items' far mask 0. You: check 8 under "How to check" in `object-lighting.md` (Dead City's green patch near `PLIGHT01`).

## 2026-10-16 User's check of M1 6d–6h

**Changed:** `STATUS.md`, `FEATURES.md`, `DESIGN.md`: 6d–6h marked as checked by eye. No code.
**Why:** The user compared the viewer with the original.
**Tested how:** By eye, by the user (Daventry and Dead City).
**Result:** "Everything looks correct just as the original."
**Still broken / not tested:** Q70 and the stand-ins listed under 6h; point lights.
**Next:** Session A: Q70 and the open points in `object-lighting.md`.

## 2026-10-16 M1 step 6h: objects near / far / off by distance from Connor

**Changed:** `kq8-data/src/objects.rs`: `tooFar`, `groupsActive3D`, `groupsActive3DFar`, `shape_named` read; `Activity` (near / far / off), `shape_ball`, `DRAW_BITS`; `drawn()` now takes any of the four draw bits (was 0x40 only); unit test `near_far_off`. New `kq8-viewer/src/draw_distance.rs`: every frame each object (placed objects, buildings, doors, chests, scripted and animated objects, items) is near / far / off by its ball's distance from Connor minus the radius; an object whose group bits then have no draw bit goes on a drawing layer the camera doesn't show (Bevy's `RenderLayers`, passed to its children). Emitters the same in `effects_view` (radius 250). Logs every change of state, objects that are near/far but not drawn, and once a second the counts, the nearest object that is off and the farthest that isn't. The 6c stand-in (nothing beyond `visibleDistance` from the camera) is removed.
**Why:** Q69 answered (`object-lighting.md` rules 34–40).
**Tested how:** `cargo test` (core 181, data 88, viewer 5, all pass), clippy clean. Dead City: Connor placed with `--at X,172122,0` east of `flame.05` (118533, 172122, 4881); walked towards it with `--hold 1,0,0,90`. Daventry from the start. `--haze off`. `viewer.log`.
**Result:** `flame.05` (no shape name → radius 100) is off with Connor at 12,169 and drawn at about 12,070; walking in: "Off -> Far at 12,098", "Far -> Near at 10,054" (ball edge 9,954, under 0.83 × 12,000 = 9,960), as rules 36 and 38 say. Hidden meshes drop as objects come back (layer removal works). Daventry from Connor's start: 368 objects, 32 near, 12 far, 324 off; 8 interior props (`plates`, `easel`, ...) are far but not drawn because their files set `groupsActive3DFar=0`. `--haze off`: 0 not drawn. Not checked by eye.
**Still broken / not tested:** Stand-ins: the ball of objects with a box collider (asked as Q70), items use default `tooFar` and group masks. Not done: the collider switch (bit 0x8), `makeActive`, `setActiveCamera`. The user's ~11,000 for the flames vs 12,100 (open question 9) is for the user to compare.
**Next:** You: the checks below; then the clicking work (Q60–Q64) or the Q54 change.

## 2026-10-16 M1 step 6g: one haze per emitter

**Changed:** `kq8-viewer`: the object material has a "haze from here" point (`haze_from`); each emitter gets its own material copies (with its `b_hazeSolid` / `hazeMultiplier`, now read for emitters too) whose point is the emitter's location, updated whenever it moves (following emitters). Before, each particle was hazed at its own position (stand-in).
**Why:** Q68 (c) answered (`object-lighting.md` rule 30).
**Tested how:** `kq8-viewer --world Daventry|IceWorld --frames 90`; console (no shader errors), `viewer.log`.
**Result:** Runs: Daventry 14 emitters, Ice World 2 (the snow follows Connor in the data, so its haze is measured 4,000 above him; the note's "follows the camera" example doesn't apply to Ice World). No log line shows the haze per emitter. Not checked by eye.
**Still broken / not tested:** Nothing measures the result; checking needs the eye (leaves far away should fade together, not one by one).
**Next:** 6h, near / far / off by distance from Connor.

## 2026-10-16 M1 step 6f: building walls hazed per surface

**Changed:** `kq8-viewer`: building models keep each surface's corners (`Model::surfaces`); `building_light.rs` puts them in one list on the graphics card per building (`WallCorners`, a storage buffer) and gives every wall corner its surface's place in it (`ATTRIBUTE_WALL_SURFACE`, read through `BuildingMaterial::specialize`). `building_light.wesl`: one haze per surface from its nearest corner to the camera, the objects' numbers; at 0.99 or more left out unless the building's `b_hazeSolid` (`for_building` gives those buildings their own material copies). The 6b per-point fog on walls is gone. Log every second: wall surfaces left out / kept at 0.99. Unit test `wall_haze`.
**Why:** Q68 answered (`object-lighting.md` rule 29).
**Tested how:** `cargo test -p kq8-viewer` (5 pass); `kq8-viewer --world Daventry|DeadCity --frames 120`, console checked for shader errors (none), `viewer.log`.
**Result:** Daventry from the start: of 19,660 wall surfaces 16,824 left out, 1,533 kept at 0.99 (`b_hazeSolid`: the tomb). Dead City from Connor's start: all 10,999 left out (every building is past ≈ 10,190 from the camera there; not checked by eye). Not checked by eye.
**Still broken / not tested:** Point lights on walls and their haze × 0.04 (rule 16) not done. The GPU and the log use the same rule but the GPU's result can't be read back; only the eye can confirm it.
**Next:** 6g, particles.

## 2026-10-16 M1 step 6e: the ground's own haze, ends at `visibleDistance`

**Changed:** New `kq8-viewer/src/ground_haze.rs` + `ground_haze.wesl`: `GroundMaterial` (texture × the light map's vertex colours, as before) replaces Bevy's unlit standard material on the surface, the floor view (U) and the flowing squares. Per corner haze 0.01 → 0.99 at `visibleDistance` (`Haze::ground_amount`, unit test `ground`), plain screen-colour blend; nothing drawn past `visibleDistance` from the camera. Bevy's distance fog removed (`sync_fog` → `sync_background`). F8 switches the ground's haze and edge.
**Why:** Q68 answered (`object-lighting.md` rules 27, 28): the ground is hazed per corner up to `visibleDistance` (not the objects' 0.85 ×), never fully, and is not drawn beyond it.
**Tested how:** unit test `ground` (first run only in 6h: it failed on my own wrong expectation at exactly `visibleDistance`, 0.9904 before the 0.99 cap; test fixed, the code was right); `kq8-viewer --world Daventry --frames 120` (console checked for shader errors: none) and `--world DeadCity --frames 60`; `viewer.log`.
**Result:** Daventry: "starts at 18000, 0.99 at 27000; haze at 22500: 0.500"; 206 ground materials. Dead City: 9,000 → 12,000. Not checked by eye.
**Still broken / not tested:** The cut where a triangle crosses `visibleDistance` is ours (open question 10). Walls still use the 6b per-point stand-in (6f next).
**Next:** 6f, walls per surface.

## 2026-10-16 M1 step 6d: haze colour from the palette, background from `newSky`, plain blend

**Changed:** `kq8-data/src/haze.rs`: `resetHazeColor` kept on its 0–255 scale, `palette_haze_colour` (nearest of entries 10–244), `new_sky` (`newSky n` / `newSky $KQWorld::background`), `Haze::background`; 3 unit tests. `kq8-viewer`: `load_haze` snaps the haze colour to the `KQWorld::loadingPalette` palette and reads the background entry; the background (clear colour) is that entry, haze on or off; objects and walls blend towards the haze on screen colours (`haze_blend` in both shaders). Plan for 6d–6h in `docs/DESIGN.md`.
**Why:** Q68 answered by A (`object-lighting.md` rules 23, 31): the game fogs with the nearest palette colour, the background is palette entry `newSky n`, and the card blends plain screen colours.
**Tested how:** `cargo test -p kq8-data haze` (6 pass). `kq8-viewer --world <W> --frames 30` for Daventry, DeadCity, IceWorld, Snowexit; read `viewer.log`.
**Result:** Daventry: haze 4 8 12 → `daventry.ppl` entry 17 = (4, 8, 12); background entry 17, the same colour. Dead City: entry 10 = black, background 10 black. Ice World: 90 100 190 → entry 64 (72, 128, 160), background 65 (72, 136, 160). Snow exit: entry 229 both. Not checked by eye.
**Still broken / not tested:** The ground still uses Bevy's fog (6e next). Which palette entries count as usable is a stand-in (all of 10–244, open question 8).
**Next:** 6e, the ground's own haze.

## 2026-10-16 Study (A): `test` / `set` / `echo` results, Q54 answered

**Changed:** `docs/behaviour/doors.md` "Q45" (e): text order for a false `==`, the other text operators, an unknown operator, `set`/`echo`. `QUESTIONS.md`: Q54 answered. `STATUS.md`: Q54 removed. Scratch notes in `re/notes.md` (not committed). No code.
**Why:** The user asked for Q54.
**Tested how:** Traced the console's `test`, `set` and `echo` handlers in `Mask.exe` with the Ghidra tools. Data check by grep over every `.cs` file: the operators used after `test`.
**Result:** A false text `==` leaves `-1` when the left word comes first in byte order (`test abc == abd` → −1) and `1` otherwise; a false text `<`, `<=`, `>`, `>=`, `!=` always leaves `1`. `set` and `echo` leave `0`. An unknown operator prints an error and leaves `0` (true), but no script uses one (the odd hits are all in comments). Scripts use `==` 4,748 times, `!=` 141, `>` 18, `<` 9, `>=`, `<=`, `&&`, `||` once each.
**Still broken / not tested:** Nothing checked in the running game (only matters to a script that reads `$Console::Return` as a number after a text test; none found).
**Next:** Session B: the one change is a false text `==` leaving `-1` when the left word comes first; the rest of B's stand-ins already match.

## 2026-10-16 Study (A): haze on ground, walls, particles; draw distance; Q68, Q69 answered

**Changed:** `docs/behaviour/object-lighting.md`: rules 16 (walls get light map + point lights, never sun/ambient), 22 (`hazeMulitplier` not read), 23 (the card's blend, haze colour snapped to the palette), new rules 27–33 (ground, walls, particles, background) and 34–40 (near / far / off by distance from Connor), numbers, data, do-not-reproduce, how to check, answers, open questions 8–10. `objects.md` "Hidden objects" rule 3: group bits now traced. `QUESTIONS.md`: Q68, Q69 answered. Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** The user asked for the remaining object-lighting questions (Q68, Q69) after B's stand-ins in M1 6b/6c.
**Tested how:** Traced in `Mask.exe` with the Ghidra tools (frame render, wall surface drawing, ground corner setup, Glide fog calls, `resetHazeColor`, `newSky`, the per-frame object near/far manager). Data checks by grep on the install: `flame.kq`/`flame_nd.kq`, Dead City and Daventry `TERRAIN.KQ` and `world.cs`, counts of `hazeMulitplier` (13 files), `hazeMultiplier` (0), `tooFar` (164), `setActiveCamera`, `setActivePoint` (2, both commented out), `useDynamicLighting 1` in `Options.cs`.
**Result:** Ground: per corner, 0.01 at `hazeDistance` to 0.99 at `visibleDistance` (not 0.85 ×), never fully hazed, not drawn beyond. Walls: per surface from the nearest corner with the objects' numbers, left out at haze 0.99 unless `b_hazeSolid`. Particles: one haze per emitter. Blend: plain, on the final colour, no fog table; haze colour = nearest palette colour to `resetHazeColor` (0–255). Background: flat palette entry `newSky n` (Daventry 17, Dead City 10), not the haze colour. Q69: objects switch off when (distance from **Connor** to their bounding ball) − radius ≥ `visibleDistance`; Dead City flames at about 12,100.
**Still broken / not tested:** Nothing checked in the original. The user's ~11,000 for the flames vs the traced 12,100 is not explained (open question 9). Not traced: which palette entries count as usable for the haze colour, the exact cut of ground triangles at the far edge, which stored point-light number is which key on walls, Direct3D.
**Next:** Session B: replace the 6b/6c stand-ins (ground fog curve to `visibleDistance` capped 0.99; walls per surface; background = `newSky` palette colour; draw distance from Connor minus the bounding radius, with near/far/off groups). You: the checks 6 and 7 under "How to check" in `object-lighting.md`.

## 2026-10-15 Study (A): `.dil` layout, Q67 answered

**Changed:** `docs/behaviour/object-lighting.md`: `.dil` layout table (the missing u32 after the shift; the closing 0 is a counted byte list; bytes after the block are ignored; `0xFFFFFFFF` = no light map), rule 13 (the five 12-byte-record files), edge cases, open question 3 closed. `clicking.md` rule 4: `zedrock1`/`zedrock2` are never loaded, so Daventry's non-clickable buildings are only cave, church and wizhouse. `QUESTIONS.md`: Q67 answered. Scratch notes in `re/notes.md`. No code.
**Why:** The user asked for Q67 before session B continues with building lighting (M1 step 5c).
**Tested how:** Traced the `.dil` reader in `Mask.exe` (field order, record sizes, what follows the tables). Data checks with throwaway scripts in the session scratchpad: all scripts in all worlds for `loadKQ` of the five odd buildings; `castle.dig` surface materials against `castle.dil`'s no-light surfaces; material 255 in all 153 `.dig` files; `Castle.dml`'s material count.
**Result:** (a) The extra u32 is the third field the reader takes (build id, shift, extra); it is stored but never used as far as found; ignore it. (b) The game always reads 48-byte records, and none of the five 12-byte files is loaded by any script, so the original never meets them. (c) The 96 Castle surfaces with no light map are exactly its 96 surfaces with material 255, which doesn't exist (26 materials): don't draw them. (d) Bytes after the block are never read.
**Still broken / not tested:** (a) what the extra field is for: not found. (c) "don't draw" is inferred from the data; the code that draws building surfaces was not traced. Nothing checked in the original (Castle D is far into the game).
**Next:** Session B: 5c, buildings, using the updated table (skip material-255 surfaces).

## 2026-10-15 Study (A): clicking, accepted items, scripted objects; Q60–Q64 answered

**Changed:** New `docs/behaviour/clicking.md` (what the click line hits, how an object decides it accepts the held item, the script calls of clickable animated objects, `getEndLoop`, `deleteObject`, `KQStatsChangeRequestEvent`). `items.md`: rule 7 corrected (sounds), new item names (`NoName<n>`), id numbers in `checkInventory`, items with no box. `chests.md` rule 8 (Q61). `doors.md`: my Q45 (b) answer corrected, rule 5 (`EndLoop` for doors too), rule 13, command table. `connor-physics.md`: where shape names are lower-cased (closes the Q39 "not found"). `QUESTIONS.md`: Q60–Q64 answered. Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** STATUS.md named Q60–Q64 as the next study work. The session prompt didn't name a topic, so I took that one.
**Tested how:** Traced in `Mask.exe` with the Ghidra tools. Data checks with throwaway scripts in the session scratchpad: every `acceptsInventory` name in all 12 worlds against `Invinfo.itl`; buildings' `colliderType` per world; scripts using `deleteObject`, `getEndLoop`, `KQCueEvent 3333/500`; `mask.cs` for the name counter.
**Result:** Q63: buildings get collision type 0x28 by default, which includes the click bit 0x20, so walls, floors and ceilings take a click before anything behind them. The ground is never tested. My earlier Q45 (b) answer ("buildings never clickable") was wrong. Q60: new items are named `NoName<n>` from one game-wide counter, saved with the game. Q61: the pickup sound is played by Connor's inventory on every add, so a chest's item makes it too. The old rule 7 described the land sound by mistake. Q62: accept lists split at commas, `INVITEM_All` only as the first word; all 47 names in the data are real types. Q64: scripted animated objects click like doors; `Init` on entering the world, `EndLoop <movement>` when a non-repeating movement ends (doors get it too); `getEndLoop` explains `bricks.cs`'s `Cue 300 … push`.
**Still broken / not tested:** Nothing checked in the original yet ("How to check" in `clicking.md`: click over a low hill, through the church wall, on a house wall). Not traced: the click line's length, when a deleted object is finally removed and whether deletions survive leaving the world or saving, add-or-set for `KQStatsChangeRequestEvent`, `getEndLoop`'s own bookkeeping.
**Next:** Session B: adjust the Q63 stand-in (the ground and the five `colliderType=136` buildings should not block; buildings take the click silently), and the `EndLoop`/`getEndLoop` calls. The user: the checks in `clicking.md`.

## 2026-10-15 M1 6c: draw distance for objects (Dead City's flames)

**Changed:** `kq8-data/src/haze.rs`: `Haze::visible` and `beyond_view`. `object_light.rs` / `.wesl`: an object farther than the world's `visibleDistance` from the camera is not drawn (with the haze on); the start-up and per-second haze logs include it. Q69 in `docs/behaviour/QUESTIONS.md`. DESIGN (6c), provenance.
**Why:** The user's check: Dead City's torch flames were drawn far beyond the fog; in the original they vanish at about 11,000 units from Connor. `flame.kq` / `flame_nd.kq` are see-through and their material (`torch01.kqs`, flags `0x10103`) halves the haze, so by the note's rules (20–23) they never vanish. Stand-in: Dead City's `visibleDistance` is 12,000.
**Tested how:** Unit test `end_rule` (12,000 kept, 12,001 not drawn). Viewer: DeadCity and Daventry `--frames 150`. `cargo test --workspace`; `cargo clippy --workspace --all-targets` (no warnings).
**Result:** Both runs `RESULT: PASS`, no console errors. Dead City from Connor's start: 1,060 of 1,071 placed objects left out, the nearest at 10,802. Daventry unchanged in the log (nearest left out at 24,170 from the start view; its `visibleDistance` 27,000 is past its haze end 22,950).
**Still broken / not tested:** **Not checked by eye.** Measured from the camera, not Connor (the user allowed it). 12,000 vs the user's about 11,000: which rule the original uses is Q69. The per-second log doesn't know which materials halve the haze, so it can't show the flames on their own.
**Next:** You: look at the torches in Dead City from far away.
**Checked afterwards (you, 2026-10-15):** looks right.

## 2026-10-15 M1 6b: fog over the ground and buildings (stand-in, the user's check)

**Changed:** `object_light.rs` `sync_fog`: Bevy's distance fog on every 3D camera (linear from `hazeDistance` to the haze end, the haze colour), so the ground (surface, floor view, flowing squares) fades per point; the background (clear colour) is the haze colour while the haze is on; F8 switches both. `building_light.rs` / `.wesl`: walls fade per point by the same rule (haze settings in the building material, F8 via `follow_mode`). `object_light.wesl`: the objects' haze blend now uses the same blend as Bevy's fog (linear colours) so objects and ground match. DESIGN (6b), Q68 (the user's observation added), provenance, FEATURES, README, STATUS.
**Why:** The user's check: objects vanish but "there is no actual haze"; the original has a very dense, dark fog that hides everything. The note covers objects only, so this is a stand-in until Q68 is answered.
**Tested how:** Viewer: `--world Daventry --frames 150`; `--haze off --frames 60`; DeadCity, IceWorld `--frames 60`. `cargo test --workspace`; `cargo clippy --workspace --all-targets` (no warnings).
**Result:** All runs `RESULT: PASS`, no console errors. Daventry: "fog on camera: linear 18000 to 22950 game units, background = haze colour"; Dead City 9,000 to 10,200; Ice World 18,000 to 22,950. `--haze off`: no fog put on the camera. 84 + 181 + 4 tests pass.
**Still broken / not tested:** **Not checked by eye.** F8 not tested by a key press. How the original fogs the ground and walls (per point? which curve?) is not known (Q68); the blend is on Bevy's linear colours, which looks lighter half-way than a blend on screen colours would.
**Next:** You: look at the fog in Daventry (fly up with F) and compare with the original.
**Checked afterwards (you, 2026-10-15):** looks right; small details to follow.

## 2026-10-15 M1 6: haze (distance fog) on models, F8 on/off

**Changed:** New `crates/kq8-data/src/haze.rs` (the end rule, the haze amount, rules 20–23 for one material, the `resetHazeColor` line). `terrain.rs` reads `hazeDistance` / `visibleDistance`; `objects.rs` reads `b_hazeSolid` and `hazeMultiplier`. `kq8-viewer`: `object_light.wesl` works out each object's haze from the camera's distance to its position and leaves it out, blends it towards the haze colour or fades it (see-through); `object_light.rs` haze settings, F8 (`toggle_haze`), on-screen line "Haze: on/off (F8)", the once-a-second haze log (`report_haze`), `for_object` (own material copy for `b_hazeSolid` / `hazeMultiplier` / see-through); `load.rs` `load_haze` and a `Shading` record (lighting kind + `0x10000`); `main.rs` `--haze on|off`; placed objects, animated objects and doors use their own haze keys. Q68 in `docs/behaviour/QUESTIONS.md`. DESIGN (M1 step 6), provenance, README, FEATURES.
**Why:** The user asked for the haze (`docs/behaviour/object-lighting.md` rules 19–23), with a key to switch it on and off.
**Tested how:** New unit tests: `end_rule`, `amount`, `material_and_object_rules`, `haze_colour_line`; `reads_a_section` extended. Viewer: `--world Daventry --frames 200`; `--haze off --frames 60`; `--haze maybe` (refused); DeadCity, Gnome, temple3, IceWorld, Swamp, CastleD `--frames 60`. `cargo test --workspace`; `cargo clippy --workspace --all-targets` (no warnings).
**Result:** 84 + 181 + 4 tests pass; all runs `RESULT: PASS`, no console errors. Daventry: haze 18,000 to 22,950, colour (4, 8, 12); 6 materials halve it; from Connor's start 286 of 313 placed objects are past the end (left out), the nearest at 23,427, a few fading. Dead City 9,000 to 10,200 (the note's numbers), colour black. `--haze off`: "haze: off at start".
**Still broken / not tested:** **Not checked by eye.** F8 not tested by a key press. The ground and buildings are not hazed (the note covers objects only; Q68), so in worlds with short haze (Dead City, 10,200) buildings stay in sight where objects vanish. Particles are hazed each at its own position (stand-in, Q68). The misspelled `hazeMulitplier` in 13 Gnome files is not read (Q68).
**Next:** You: walk or fly (F) away from an object and watch it fade and vanish; F8 to compare. Q68 for a study session.

## 2026-10-15 M1 5c: buildings lit by their `.dil` light maps

**Changed:** New `crates/kq8-viewer/src/building_light.rs` and shader `building_light.wesl`: a building material that multiplies each wall's texture by its light map. Each building's light maps are packed into one light picture (white texel at 0,0 for surfaces without a map); each wall corner gets a second set of picture coordinates by rule 13. `load.rs`: `load_building` reads `<shape>.dil` (`building_light_maps`), builds the picture and the coordinates, leaves out Castle D's material-255 surfaces (Q67 c), logs per building; `Model::light_atlas`, `Group::light_uvs`. `main.rs` `building_mesh`, registration, F7 for buildings (`building_light::follow_mode`). `terrain_view.rs`: buildings drawn with the new material. `kq8-data/src/dil.rs`: the closing u32 read as a counted byte list (Q67). DESIGN (5c plan, done), provenance.
**Why:** Step 3 of object lighting (`docs/behaviour/object-lighting.md` rules 12–16), way 2 chosen by the user. Q67 answered by A on the same day.
**Tested how:** New unit tests `packing_keeps_maps_apart`, `corner_coordinates`. Viewer: `--world Daventry --frames 120`, `--light off --frames 30`; CastleD, Swamp, DeadCity, Gnome, Barren, IceWorld, Snowexit, temple1, temple4 `--frames 60`. `kq8 lighting-verify`. `cargo test --workspace`; `cargo clippy --workspace --all-targets` (no warnings).
**Result:** 80 + 181 + 4 tests pass; all runs `RESULT: PASS`, no console errors; `lighting-verify` PASS. Daventry: all 30 buildings lit by their own map, 0 size-rule misses; Connor's hut 1,129 of 1,129 surfaces, red 0 ×16,017, 3 ×7,775, 4 ×7,344 (as 5a), light picture 512 × 167. Castle D: 5,222 of 5,318 surfaces lit, 96 not drawn (material 255; before, drawn magenta). No building in the 10 worlds fell back to full bright. Average frame 8.5 ms (Daventry).
**Still broken / not tested:** **Not checked by eye.** Walls get no sun or ambient (rule 16, stand-in); `0x1000` materials use the map (open question 7). Haze, point lights, the brightness setting not done.
**Next:** You: Connor's hut inside and out (below). Then haze (if wanted) or A's clicking answers (Q60–Q64).
**Checked afterwards (you, 2026-10-15):** everything looks right.

## 2026-10-15 M1 5b: models lit by the world's ambient light and sun

**Changed:** New `crates/kq8-viewer/src/object_light.rs` and shader `object_light.wesl`: a custom material for every model (placed objects, animated objects, doors and chests, items, particles, Connor, the `--model` view) that works out each corner's light on the graphics card from its normal and the object's rotation, and multiplies the texture. `kq8-data`: `shape.rs` gives each corner a normal (average of the triangles at that vertex), `material.rs` the lighting kind (`Lighting`: smooth / flat / none). `kq8-viewer`: `load.rs` reads the world's lights (`load_object_lights`) and carries each material's kind; `main.rs` `model_mesh` (meshes with normals; flat triangles carry their first corner's normal); `terrain_view`, `anim_view`, `door_view`, `connor_view`, `item_view`, `effects_view` use the new material; F7 and `--light` switch the objects too (label now "Light, ground and objects"). Logs: the lights, material counts, the corner light of Connor and the three objects nearest his start. `.gitignore`: whitelists `crates/*/src/**/*.wesl` (shader sources). DESIGN (5b changed to the graphics card, user's choice; done), provenance, README, FEATURES.
**Why:** Step 2 of object lighting (`docs/behaviour/object-lighting.md` rules 4–10). The planned way (colours stored in the meshes) can't work: many objects share one mesh while facing different ways; the user chose the graphics-card way.
**Tested how:** New unit tests: `corner_normals_average_the_triangles_at_a_vertex`, `lighting_kind`, `suns_in_bevy_space`. Viewer: `--world Daventry --frames 120`; `--model smlchest.KQS --frames 30`; Swamp, DeadCity, Gnome, IceWorld, Barren, temple4, CastleD `--frames 60`; `--at 270813,225311,-1.5708 --hold 1,0,0,80 --turn 40,-1 --light x2 --frames 200`. `cargo test --workspace`; `cargo clippy --workspace --all-targets` (no warnings).
**Result:** 80 + 181 + 2 tests pass. All runs `RESULT: PASS`, no errors on the console (a broken shader would print them). Daventry: ambient 0.3, sun 0.1; Connor 0.300–0.419 (mean 0.338), `tree.06` 0.300–0.404, `stngrl` 0.300–0.412, `lamp01.13` 0.300–0.420, inside the note's 0.30–0.42; 721 smooth, 4 flat, 365 none materials (buildings included in "none"). Chest: 96 of 96 corner normals point outward. Other worlds follow their light tables (Swamp, Ice World, Barren, Temple 4 reach the 1.0 cap on the sunny side; Gnome 0.24–0.28). Average frame 7.3–8.4 ms in Daventry.
**Still broken / not tested:** **Not checked by eye** (I can't see the screen): whether objects really look 30–42% bright next to the ground, and which side is sunny. The log numbers are worked out on the CPU with the shader's rules, not read back from the screen. F7 by key press not tested (the start option `--light x2` was). Buildings full bright (5c). Point lights, haze, brightness setting not done.
**Next:** You: look at Daventry (below). Then 5c, the buildings.
**Checked afterwards (you, 2026-10-15):** objects look lit much more closely to the original.

## 2026-10-15 M1 5a: reading object lights and building light maps (`.dil`)

**Changed:** Plan for object and building lighting in `docs/DESIGN.md` ("M1 step 5", sub-steps 5a–5c; haze left out until the user decides). New `crates/kq8-data/src/lights.rs` (the `loadKQ` lines of `swtohwlt.cs`, ambient lights, suns, point lights, the capped corner light, the flat grey) and `dil.rs` (`.dil` reader, texel lookup, the size check of rule 13); `building.rs` keeps the `.dig`'s build id. New CLI command `kq8 lighting-verify` (`crates/kq8-cli/src/lighting_verify.rs`). Q67 in `docs/behaviour/QUESTIONS.md`. `docs/provenance/README.md`, `README.md` (command list). Nothing drawn yet.
**Why:** Step 1 of object lighting (`docs/behaviour/object-lighting.md`): read and check the data before drawing with it.
**Tested how:** 9 new unit tests (Daventry's three corner-light numbers, ambient sum and capping, point lights and unknown classes, the flat grey, the `loadKQ` lines; a hand-made `.dil` with 48- and 12-byte records, texel order, the size rule, bad files refused, the "no map" offset). `cargo run --release -- lighting-verify`, log `kq8-artifacts/logs/lighting-verify.log`. `cargo test --workspace`; `cargo clippy --workspace --all-targets` (no warnings).
**Result:** RESULT: PASS. 78 + 181 + 1 tests pass. Lights: every world's `swtohwlt.cs` loads `light/SunlgtHW.kq` and `light/AmblgtHW.kq`; Daventry ambient 0.3, sun 0.1, aim (0, 1, −0.66), length 1.198; a model corner gets 0.300 / 0.366 / 0.420 facing away / up / into the sun, exactly the note's table. `.dil`: all 235 read; 153 of 153 `.dig`/`.dil` pairs match in build id and surface count and fit rule 13 on every surface; `cnnrhut.dil` 1,129 surfaces, 162,170 bytes, red 0 ×16,017, 3 ×7,775, 4 ×7,344 (the note's numbers). **The note's layout table is not quite right** (Q67): (a) one more u32 (0) after the shift; (b) the 5 files the note says miss the size rule have 12-byte surface records, not 48, and fit the rule when read that way; (c) 96 surfaces of Castle D's `castle.dil` have byte offset `0xFFFFFFFF` (read as "no light map"). Materials: models 10,981 smooth, 49 flat, 515 none, none with `0x1000`/`0x2000`; building lists (`.dml`) 223 have no shading bits. Buildings without a `.dil`: Daventry `ruins1`, `ruins3`, `ruins4` and some in other worlds (logged).
**Still broken / not tested:** Nothing drawn: objects and buildings are still full bright. The first two attempts at the `.dil` reader followed the note's table and failed on every file; the layout above was then measured on the data. How a "no light map" surface should look is unknown (Q67 c). Point lights are read, not used.
**Next:** 5b, light the models (you approve first). Q67 for a study session (not blocking 5b).

## 2026-10-14 Docs: REA setup and the session split

**Changed:** `docs/RESEARCH.md`, REA section: new point that its setup command registers REA user-wide in Claude Code (`~/.claude.json` and a skill in `~/.claude/skills`). That would give session B a disassembler, so a trial should use this project's `.mcp.json` for session A only. Includes how to undo a setup install. Docs only.
**Why:** The user asked to record it after asking what REA's setup would change.
**Tested how:** Read REA's `docs/installation.md` ("Limit REA to one project or agent profile"), `docs/client-configuration.md` and `src/application/SupportedClients.ts` (the `claude_code` entry).
**Result:** Recorded. The paths come from REA's code and docs.
**Still broken / not tested:** REA setup was not run, so the paths it writes were not observed.
**Next:** None.

## 2026-10-14 Reference: REA reverse-engineering tool

**Changed:** Cloned <https://github.com/morluto/rea> (commit `877d404`) into `reference/rea` (ignored by git, local only). `docs/RESEARCH.md`: new section "Reverse-engineering tools" with what REA is and whether it fits. Docs only; no code.
**Why:** The user asked to add REA to the references and to judge whether it's worth using.
**Tested how:** Read its README, `docs/windows-ghidra-p0.md`, `docs/ghidra-provider.md` and `docs/native-investigation.md`. `git status --short` after cloning showed nothing new, so `reference/rea` is ignored.
**Result:** Not worth installing now. Windows support is experimental, and 32-bit PE files like `Mask.exe` are accepted only on its `main` branch, not in npm releases up to 5.0.0. It analyses a fresh copy each time and can't use or update our Ghidra project. The idea worth borrowing is its method of checking a rebuilt function against the original on many inputs.
**Still broken / not tested:** REA was not installed or run. Nothing about its output on `Mask.exe` is verified.
**Next:** None. Look again if a release supports 32-bit PE on Windows and can open an existing Ghidra project.

## 2026-10-14 Docs: launch options in FEATURES.md

**Changed:** `docs/FEATURES.md`: new section "Launch options (viewer)" before "What doesn't work yet". It lists every viewer option (`--world`, `--model`, `--palette`, `--light`, `--game`, `--frames`, `--at`, `--lift`, `--hold`, `--turn`, `--jump`, `--click`, `--aim`, `--item`) with a short plain-language description, and says which ones only work in the ground view. Docs only; no code.
**Why:** The user asked for a list of launch options with what each one does.
**Tested how:** Each option's meaning was read from its code in `crates/kq8-viewer/src/main.rs` and `connor_view.rs`. Viewer runs: `--world Daventry --at 270813,225311,-1.5708 --hold 1,0,0,80 --turn 40,-1 --jump 20 --click Condoor,2 --aim Condoor,5 --light x2 --frames 60` (all accepted, `RESULT: PASS`; log line "test --aim Condoor ... nothing clicked (Q63 stand-in)"); `--lift 3000 --frames 30` (log line "test --lift: lifted 3000 to z=16633"); `--hold 1,0` (error: "--hold needs three or four numbers").
**Result:** The options parse and do what the section says for `--aim` and `--lift`. The `--hold`, `--turn`, `--jump` and `--click` lines were not read back: the log file was overwritten by the later runs.
**Still broken / not tested:** `--palette` not run (no palette file named in this check). The `--hold` turn sign (1 left) follows `--turn`'s meaning; not checked in the logs. The section was not read by the user.
**Next:** Keep this list in step with `parse_args` when a new option is added.

## 2026-10-14 Viewer: the ground is the default, `--model` shows one model

**Changed:** `crates/kq8-viewer/src/main.rs`: no `--model` now opens the world's ground (what `--terrain` did); `--model NAME` opens one model (what a bare model name did). `--terrain` is still accepted and does nothing. A bare model name is now an error that tells you to use `--model`. Help text, module comment and usage line updated. `terrain_view.rs`: header comment. `README.md`: "Try it" command is now `-- --world Daventry`; the single-model example uses `--model`. `docs/DESIGN.md`: viewer usage lines and the three "`--terrain` view" mentions. `STATUS.md` (ignored by git, local only): viewer line.
**Why:** The user wanted the ground to be the normal start, with single models behind a named flag.
**Tested how:** `cargo build --release -p kq8-viewer`; `cargo clippy -p kq8-viewer --all-targets` (no warnings); `cargo test -p kq8-viewer` (1 pass). Viewer runs: `--world Daventry --frames 120` (ground; log line `world: Daventry`, `RESULT: PASS`); `--model smlchest.KQS --frames 60` (model; `spawned: vertices=96`, `RESULT: PASS`); `--terrain --world Daventry --frames 30` (old flag, same as the ground); `smlchest.KQS` with no flag (error, exit 1).
**Result:** All four behave as described above. The model-view log line "front faces pointing away from the model centre: 30 of 32 triangles" was printed as before; this change did not touch that code.
**Still broken / not tested:** Nothing looked at by eye (the window is for the user to check). The argument parser has no unit test (it reads the real command line), so the checks above are by running the program. Older MODLOG entries and `docs/behaviour/QUESTIONS.md` Q43 still show `--terrain` in their recorded commands: they are history, and the flag still works.
**Next:** Remove the `--terrain` alias once nobody needs old commands (one line in `parse_args`).

## 2026-10-14 Object lighting note: the user's checks in the original

**Changed:** `docs/behaviour/object-lighting.md`: "How to check" records the results, building text corrected, new open question 7. Docs only.
**Why:** The user checked the note's by-eye items in the original game.
**Tested how:** User's check by eye in the original; data check (which Daventry files name the `0x1000` textures).
**Result:** Objects as dark as the grass, sunny side slightly lighter: **confirmed**. Haze: objects fade and vanish with distance from the camera: **confirmed** (distance not measurable in the original). Connor's hut inside: darker than outside and in the corners, but reasonably lit; the note's "mostly dark" overstated it, so it now says the bright areas are baked into the light map. The "full-bright materials" check is **withdrawn**: in Daventry those textures appear only in `interior.dml` (buildings), not in models, so it pointed at nothing concrete.
**Still broken / not tested:** Whether `0x1000` building materials skip their light map (open question 7). Hut brightness to be compared with our viewer once buildings are lit.
**Next:** Session B: implement `object-lighting.md`.

## 2026-10-14 Study (A): object and building lighting, Q65 and Q66 answered

**Changed:** New `docs/behaviour/object-lighting.md` (model lighting, building `.dil` light maps, brightness setting, haze, point lights). `terrain-lighting.md`: rule 10 now says ×1 (confirmed), rule 3 and rule 9 corrected (Q65 b), open questions 1 and 3 closed. `buildings.md`: points to the `.dil` section; corrects the guess about the surface field at byte 12. `QUESTIONS.md`: Q65 and Q66 answered. Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** The user asked for a study session on object and building lighting, plus Q66 (×1 or ×2, and anything that brightens the picture).
**Tested how:** Traced in `Mask.exe` with the Ghidra tools (Glide renderer only). Throwaway scripts in the session scratchpad checked the data: all 235 `.dil` headers, the `.dil`/`.dig` match on 153 pairs (surface count and build id), the light-map size rule on every surface, Daventry building light histograms, light files of all 12 worlds, point-light files.
**Result:** ×1 confirmed: light maps go to the card unchanged and are multiplied, no doubling. `setBrightness 50` is a factor of exactly 1.0. On hardware the world's `*HW.kq` lights are used (Daventry: ambient 0.3, sun 0.1), so Daventry models come out at 30–42% of their texture's brightness. Building walls use a per-surface light map in `.dil`: 4-bit colour, one texel per 16 × 16 texture texels. The size rule fits every surface of 148 of 153 files. Objects get no shadow from the ground or from buildings. Haze in Daventry runs from 18,000 to 22,950 units, and objects beyond the end are not drawn.
**Still broken / not tested:** **Nothing checked by eye yet** (How to check in the note). Not traced: the wall drawing code (whether sun and ambient also tint walls; assumed not), Direct3D, how a point light decides an object is out of range, what turns on the per-object "unlit" switch, why 5 `.dil` files miss the size rule. Daventry has no point lights; `torchlight.kq` is loaded by no script found.
**Next:** Session B: light models and buildings from `object-lighting.md` (and haze if wanted). The user: the by-eye checks in that note.

## 2026-10-14 Clippy clean on the whole workspace

**Changed:** `kq8-viewer`: `load.rs` (a doc comment described two tabs with real tab characters; now in words), `game_camera.rs` `camera_input` and `terrain_view.rs` `fly_input` (allowed "too many arguments": Bevy systems take what they use as arguments), `main.rs` (allowed "large size difference" on the one-off `Scene` choice). `kq8-data/src/objects.rs`: the test `reads_a_section` allows `3.14`, which is the game file's own value (`dirZ=3.14`), not π. Each allow has a comment saying why. No behaviour change.
**Why:** The user asked to clean up the four older warnings. The `3.14` test made `cargo clippy --workspace --all-targets` stop with an error (noted in the M3 3e-2 entry).
**Tested how:** `cargo clippy --workspace --all-targets`; `cargo test --workspace`; `cargo build --release -p kq8-viewer`; viewer `--terrain --world Daventry --frames 120`.
**Result:** Clippy: no warnings, no errors. 181 + 69 + 1 tests pass. Viewer `RESULT: PASS`.
**Still broken / not tested:** Nothing new.
**Next:** A study session (A) on object and building lighting (and Q66).

## 2026-10-14 M1 4c: x1 is the default ground light (user's check)

**Changed:** `crates/kq8-viewer/src/light_view.rs` (default mode x1, F7 cycles x1 → x2 → off, one more unit test), `main.rs` (`--light` default and help). Q66 (correction request) and a note on Q65 in `docs/behaviour/QUESTIONS.md`. DESIGN, provenance, README, `docs/FEATURES.md`.
**Why:** The user compared the viewer with the original by eye: x1 matches better, the original is very dark and muddy. The note's suggested stand-in was x2; the user's check wins (AGENTS/session rules), and session A is asked to confirm (Q66).
**Tested how:** `cargo test --workspace`; clippy on `kq8-viewer`; viewer `--terrain --world Daventry --frames 120`.
**Result:** 181 + 69 + 1 tests pass; no new clippy warnings. Viewer: "ground light: mode x1 at start (872 lit ground meshes, 3201408 corners); F7 cycles x1 -> x2 -> off"; at Connor's start raw 7 → 47%; `RESULT: PASS`.
**Still broken / not tested:** Objects and buildings are drawn full bright (the user noticed; not studied yet). F7 by key press not tested by me.
**Next:** A study session for object and building lighting, or Q66.

## 2026-10-14 M1 4b + 4c: the ground drawn with its light map (x2 / x1 / off), no tone mapping

**Changed:** New `crates/kq8-viewer/src/light_view.rs` (light modes x2 / x1 / off, F7 key, on-screen line "Ground light: ... (F7)", light under Connor and the camera every second in `viewer.log`). `kq8-data/src/terrain.rs`: `Terrain::square_cells` (a square cut into the light map's cells, same shape) and `GroundTriangle::half`. `kq8-viewer`: `load.rs` loads the shown map with the ground (`load_light_map`) and cuts the animated squares into cells; `terrain_view.rs` builds the ground from cells with a colour per corner, camera without tone mapping; `ground_anim_view.rs` lights the river squares; `main.rs` `--light x2|x1|off`, wiring, the model view's camera without tone mapping too. DESIGN (4b/4c done), provenance, README (F7 key, what doesn't work), `docs/FEATURES.md`.
**Why:** Steps 2 and 3 of terrain lighting (`docs/behaviour/terrain-lighting.md`), done together at the user's request. Tone mapping (Bevy's default colour curve) is off so the screen shows the colours as computed, needed for an eye comparison of x1 and x2.
**Tested how:** New unit tests: `square_cells` (n = 1 is the two triangles; for n = 4 every corner and centre is on the height lookup, clockwise, inside the triangle its `half` names, texture coordinates follow the code) and the three light modes. Viewer runs: `--terrain --world Daventry --frames 300`; `--at 208896,284672,0 --frames 120` (inside a hill shadow); `--at 208896,272000,-1.5708 --hold 1,0,1 --frames 1500` (walking); `--light x1 --frames 120`; Swamp, DeadCity, IceWorld, Barren `--frames 60`. Whole workspace `cargo test`; clippy: no new warnings.
**Result:** 181 tests pass in `kq8-core`, 69 in `kq8-data`, 1 in `kq8-viewer`. Daventry: "Shadow map daveshdw.bin (Options.cs useDynamicLighting 1) ... red 4 x20127, 6 x4598, 7 x236889, 8 x1555"; 4 x 4 cells per square; 872 lit meshes, 3,201,408 corners; average frame 6.78 ms; loading 793 ms. Light under Connor: start (277,744, 199,295) raw 7 → 93% (x2), 47% (x1); in the hill shadow at (208,896, 284,672) raw 4 → 53%; walking it changes smoothly (8.00, 7.77, 7.00, 6.90, 6.55, 6.19). Swamp, Dead City, Ice World load their shadow maps; Barren has none (full brightness). All runs end `RESULT: PASS`.
**Still broken / not tested:** **Not checked by eye** (nobody has looked at the lit ground yet), and **x1 vs x2 is still open** (your check decides). F7 not tested by a key press (only the start option). Stand-ins: x2, the blend inside a cell (not exactly bilinear), the multiply in screen colours, brightness setting ignored. Objects, buildings, haze and moving lights are not lit. `--hold` with heading −1.5708 still moved Connor south; heading direction not looked into (not part of this step).
**Next:** You: compare x2 and x1 with the original (below). Then the result goes into Q65 / the note.

## 2026-10-14 M1 4a: reading the ground's light and shadow maps

**Changed:** Plan for ground lighting in `docs/DESIGN.md` ("M1 step 4", sub-steps 4a–4c). New `crates/kq8-data/src/lightmap.rs` (the map file, its channels, the bilinear lookup, which file is shown, the `useDynamicLighting` line of `Options.cs`); `terrain.rs`: `TerrainSettings` reads `lightScale` and the four map names. New CLI command `kq8 lightmap-verify` (`crates/kq8-cli/src/lightmap_verify.rs`; `terrain_verify.rs` shares its file finder). Q65 in `docs/behaviour/QUESTIONS.md`. `docs/provenance/README.md`, `README.md` (command list).
**Why:** Step 1 of terrain lighting (`docs/behaviour/terrain-lighting.md`): read and check the maps before drawing with them.
**Tested how:** 6 new unit tests (side and spacing, value order and channels, wrong count / short file refused, bilinear blend, which file, the `Options.cs` line). `cargo run --release -- lightmap-verify`, log `kq8-artifacts/logs/lightmap-verify.log`. Whole workspace `cargo test`; clippy clean on `kq8-data` and `kq8-cli`.
**Result:** RESULT: PASS, 24 files in 6 worlds (Castle, Daventry, Dead City, Ice World, Snow exit, Swamp; the user's "7" is 6 in the data), 0 failures. Every count = side² and every file size = the note's table. `Options.cs`: `useDynamicLighting 1` → shadow maps shown. Daventry's `daveshdw.bin` red: 4 ×20,127, 6 ×4,598, 7 ×236,889, 8 ×1,555, exactly the note's numbers; light vs shadow differ in 761 points (the note's number), but 4 of them by more than one step in red (2 by 1, 2 by 3). Found: other worlds' hardware maps use 15 for sunlit ground (Swamp, Ice World, Snow exit, Castle) and Dead City 9, while Daventry uses 7; so ×2 (capped) gives 93–100% sunlit ground everywhere, ×1 gives 47% in Daventry but 100% elsewhere. Asked as Q65, with two small note corrections.
**Still broken / not tested:** Nothing is drawn with the maps yet (4c). Bilinear blending and the ×2 factor are stand-ins (note open questions 1, 2).
**Next:** 4b (tone mapping off, so colours show as computed), then 4c (the ground drawn lit, ×2 / ×1 / off).

## 2026-10-14 STATUS.md brought up to date

**Changed:** `STATUS.md` (local only, not committed): where we are (M3 traps, items, chests, scripted objects, fish done), next steps (terrain lighting or Q60–Q64), the `kq8-core` name and its current files, `item-verify`, the `--aim` option, all log files, test counts (62 + 181), what works and what doesn't, open questions Q60–Q64 added (Q55 answered, Q59 closed), fresh-session prompts. Docs only, no code.
**Why:** The user: update STATUS.md and any outdated information in it.
**Tested how:** Compared with MODLOG (newest entries), `docs/behaviour/QUESTIONS.md` status column, the source file lists and the viewer's options and log names in the code.
**Result:** STATUS.md matches MODLOG up to commit `3f435b6`.
**Still broken / not tested:** Nothing new.
**Next:** Nothing.

## 2026-10-13 README shortened for new users; details moved to `docs/FEATURES.md`

**Changed:** `README.md` rewritten: short "What works" and "What doesn't work yet" lists, "What you need", the keys (Insert and U added), the folder structure and its crates, how the work is done. The detailed per-feature text (how each thing was checked) moved to the new `docs/FEATURES.md`. `docs/DESIGN.md` layout lists it. Docs only, no code.
**Why:** The user: the README was too technical for new users.
**Tested how:** Compared every README claim with MODLOG, STATUS.md and DESIGN; checked the key bindings, CLI commands and `KQ8_GAME_DIR` in the code.
**Result:** Corrected outdated notes found on the way: wading and the Swamp shack were checked by eye on 2026-10-12, the fish on 2026-10-13, the rolling rocks' boxes by walking there (the old README said logs only).
**Still broken / not tested:** Links not checked on GitHub. `STATUS.md` (local only) is out of date: it still lists traps, chests and items as next steps and uses the old `kq8-game` name.
**Next:** Nothing.

## 2026-10-13 Rename the rules crate: `kq8-game` → `kq8-core`

**Changed:** `crates/kq8-game/` → `crates/kq8-core/` (package name, workspace `Cargo.toml`, the viewer's dependency, every `kq8_game::` path in `kq8-viewer`, one comment in `kq8-data`). DESIGN "Repository layout" now lists `kq8-core` (it was missing) and describes the viewer as it is today, plus the open decision about a separate game program. DESIGN and provenance use the new name; MODLOG history, QUESTIONS and STATUS.md keep the old one.
**Why:** The user: "kq8-game" reads like "the program you play"; the crate holds only the rules. "core" chosen by the user.
**Tested how:** `cargo test --workspace`; `cargo build --release -p kq8-viewer`; clippy on `kq8-core`.
**Result:** 181 + 62 tests pass; the viewer builds; clippy clean. No behaviour change.
**Still broken / not tested:** Nothing new.
**Next:** Terrain lighting or a study session for Q60-Q64 (user's choice).

## 2026-10-13 Renames: `movement.rs` → `named_anim.rs`, `mover.rs` → `connor_mover.rs`, `player.rs` → `connor_update.rs`

**Changed:** `crates/kq8-game/src/movement.rs` renamed to `named_anim.rs`, its type `MovementPlayer` to `NamedAnimPlayer` (uses in `chest.rs`, `active.rs`, `door_view.rs`); the file's first lines now say what it is. Also, at the user's request, Connor's `mover.rs` → `connor_mover.rs` and `player.rs` → `connor_update.rs` (module paths in `camera.rs`, `connor.rs`, `connor_view.rs`, `door_view.rs`). DESIGN and provenance point to the new names; older MODLOG entries keep the old ones.
**Why:** The user: "movement" reads as Connor's movement; `mover` and `player` didn't say they are Connor's. The file plays an object's named animations (`open`, `closed`, ...; "movements" in the game's files) when asked, for chests and scripted objects.
**Tested how:** `cargo test --workspace`; `cargo build --release -p kq8-viewer`; clippy on `kq8-game`.
**Result:** 181 + 62 tests pass; the viewer builds; clippy clean. No behaviour change. (No side build folder exists to remove: `target/` holds `debug`, `release` and the editor's `flycheck0`.)
**Still broken / not tested:** Nothing new.
**Next:** Terrain lighting or a study session for Q60-Q64 (user's choice).

## 2026-10-13 M3 3f: clicking objects that move at load (the fish), `deleteObject`

**Changed:** `kq8-game/src/active.rs`: clickable objects with a script (`Clickable` gets the script and click keys, `Clickable::plain`) are clicked like doors, get `Init` and `Cue`; `deleteObject` on them and on 3e-5 objects makes them gone for good and sends `Request::Delete`; events to deleted objects are dropped. `script.rs`: `KQSound::play` stub. `kq8-viewer`: `load.rs` passes each clickable's script and keys; `terrain_view.rs` `ObjectName` tag on every placed object's entities (`anim_view.rs` too); `door_view.rs` applies deletions (collider switched off, view box dropped, `hide_deleted` hides the entities, also from H); `main.rs` wiring. DESIGN (plan 3f), provenance, README.
**Why:** The user's check in the original: clicking a swimming fish makes it disappear, Connor eats it and gains health (`fishies.cs`: `deleteObject $1`, `KQSound::play eat.aud`, `KQStatsChangeRequestEvent 2 0 0 0 0`). The fish plays its movement from the start, so it was left out of 3e-5.
**Tested how:** 3 new unit tests (a fish-like script deletes itself once: gone, not picked, request sent, its later cue dropped, the stats event logged; no script: logged as before; `deleteObject` on a 3e-5 object and on an unknown kind). Viewer: `--terrain --world Daventry --at 208502,209600,1.5708 --click fishies.01,10 --click fishies.01,40 --click fishies.02,45 --frames 900`; Swamp, Gnome, DeadCity, IceWorld, Barren `--frames 60`. Whole workspace `cargo test`; clippy: no new warnings.
**Result:** 181 tests pass in `kq8-game`, 62 in `kq8-data`. `fishies.01` (1,943 from Connor): `fishies.cs` runs, "deleteObject fishies.01: clickable object gone", `KQSound::play eat.aud` stub, the stats event to Connor "stats are M5"; the viewer: "1 collider(s) switched off, 0 view box(es) dropped, hidden"; the second click: "deleted or not clickable now"; the cue 1 s later "deleted; dropped". `fishies.02` (own script `fishies2.cs`) the same. 118 `Init` calls at Daventry's start, no errors; other worlds load; Gnome and IceWorld each log one unknown command from an `Init` branch (`KQMonster::clearSpeak`, `getEndLoop`), harmless.
**Still broken / not tested:** **Not checked by eye** (the fish vanishing; walking through where it was). Eating sound and health are logged only (audio later, stats M5). `KQAnimPlayEvent` to these objects is logged, not done (their animation is `anim_view`'s). `deleteObject` on placed objects that are not clickable with a script stays a stub. All of these stand-ins are in Q64.
**Next:** You: click one of the three fish near (208,500–209,900, 210,900) with `--at 208502,209600,1.5708`.
**Checked afterwards (you, 2026-10-13):** works as in the original (the fish). The pub sign's click runs `pubsign.cs`; its remark (`KQMonster::speak … 56 0 0 1 0`) is in `doors.log` only, by design.

## 2026-10-13 M3 3e-5: scripted chests, the jar and the mushroom baskets

**Changed:** New `crates/kq8-game/src/movement.rs` (`MovementPlayer`: an object's named movements, moved out of `chest.rs`, which now uses it). `active.rs`: `ActiveObject` (clickable animated objects with a script): clicked like doors (the doors' argument rules now shared, `script_args`), `KQAnimPlayEvent` plays a movement, `Init`/`EndLoop`/`Cue` to their script, picked by the ray, `collisionType`/`setHidden`. `kq8-viewer`: `load.rs` sends such objects (still at load) through the door loading (`Mover::Object`); `door_view.rs` builds, moves (box) and draws them. Q64 in `docs/behaviour/QUESTIONS.md`. README, DESIGN, provenance.
**Why:** Step 5 of chests and items (`chests.md` rule 11): the small chests, the jar and the baskets are plain animated objects whose script does the work.
**Tested how:** New unit tests (`movement.rs` play-once; `active.rs`: a basket gives once, opens, `EndLoop open`, too far / item in hand; a small chest puts coins beside itself, then the remark; an unknown movement). Viewer: `--at 189493,267000,1.5708 --click smlchst1,10 --click smlchst1,40 --click SilverCoins.23,45 --frames 1200`; `--at 233848,203200,1.5708 --aim basket1,10 --click basket1,40 --frames 1000`; Swamp, Gnome, DeadCity, IceWorld `--frames 60`. Whole workspace `cargo test`; clippy: no new warnings.
**Result:** 178 tests pass in `kq8-game`, 62 in `kq8-data`. Daventry: 14 scripted objects; 10 clickable scripted ones that move at load left to `anim_view`. `smlchst1`: 1st click → `Daventry::smlchst1::full` false, `SilverCoins.23` x35 at (189493, 267693, 9115) (300 beside it) with `Tavcoin.cs`, `open` plays (updates 11–17), `EndLoop open`; 2nd click → only the remark stub; clicking the coins → 0 → 35, `Tavcoin.cs` removes its script. `basket1`: 5 mushrooms to Connor; its `open` (`basket.kqs`, the empty basket) ends in the next update: the data suggests a one-frame model that just swaps the look (not checked in the original). `--aim basket1` from outside the farmhouse is blocked by `Frmhut`, as 3e-4b intends. Other worlds: 3, 12, 2, 5 scripted objects, all load.
**Still broken / not tested:** **Not checked by eye.** `Init`, `EndLoop`, the click call and `KQAnimPlayEvent` for these objects are our stand-ins from the doors' and chests' rules (Q64). Swinging signs and other scripted things that move at load can't be clicked yet (Q64 d). Many of these scripts use commands not done yet (speech, cutscenes): logged as stubs or unknown.
**Next:** You: try the small chest by the pub's keg, the jar, and a basket (below). That finishes M3 step 3e (chests and items).
**Checked afterwards (you, 2026-10-13):** everything works as expected. Also checked in the original: clicking the swimming fish (one of the objects left to `anim_view`) makes it disappear, Connor makes an eating sound and gains health. `fishies.cs` (data) does exactly that (`deleteObject $1`, `KQSound::play eat.aud`, `KQStatsChangeRequestEvent 2 0 0 0 0` to Connor). Added to Q64 (d); not done yet: clicking objects that move at load, `deleteObject` on placed objects, stats (M5).

## 2026-10-13 M3 3e-4b: no clicks through walls or the ground

**Changed:** `kq8-game/src/active.rs`: `pick_seen` (the nearest clickable box, but only if the line from the camera to it isn't blocked), `target_point` (middle of an object's box). `kq8-viewer`: `game_camera.rs` `view_blockers` (the camera's obstacle list, now shared); `door_view.rs` `door_click_input` uses it for every click (doors, chests, items) and logs what blocked a click; new test option `--aim NAME,N` (`main.rs`). DESIGN, provenance.
**Why:** The user's check in the original (2026-10-13): Connor can't click through walls or the ground, but can from behind a wall when the camera sees the object. That contradicts `doors.md` Q45 b, which we followed; asked as Q63.
**Tested how:** New unit test (a wall between camera and chest blocks; behind the camera, behind the chest or at the chest's face it doesn't; nothing on the ray). Viewer with `--aim`: pub chest from the ground floor; a mushroom outdoors; Connor's front door. Whole workspace `cargo test`; clippy: no new warnings.
**Result:** 176 tests pass in `kq8-game`, 62 in `kq8-data`. `--at 202336,262000,1.5708 --aim smlchst3,20`: "meets smlchst3 at 5974 units, but building Pub blocks the line at 5319 units: nothing clicked". `--at 266011,221900,0 --aim Mushroom.14,20`: picked, then picked up (507 from Connor). `--at 270813,225311,-1.5708 --aim Condoor,20`: picked and clicked, the door doesn't block itself.
**Still broken / not tested:** What blocks is our stand-in (the camera's obstacles: ground, buildings, doors and door-like objects); whether tables, trees or characters block too is Q63. Not tested: a click from upstairs in the pub (`--at` puts Connor on the ground floor); please try it by mouse. The test option `--click` still skips the ray (on purpose: it tests the click rules alone).
**Next:** You: in the pub, try the chest from downstairs (nothing should happen) and upstairs (opens). Then 3e-5.
**Checked afterwards (you, 2026-10-13):** works like the original game.

## 2026-10-13 M3 3e-4: chests

**Changed:** New `crates/kq8-game/src/chest.rs` (a chest's keys, states closed/open/empty, its movements). `door.rs`: the phase step and cues of a movement split out (`DoorMovement::step`) so doors and chests share it; doors behave as before. `active.rs`: chests in the world (picked by the mouse ray, clicked with the three steps of `chests.md`, `Init`/`EndLoop`/`Cue` to their script, `collisionType` and `setHidden` on chests); one helper makes world items for scripts and chests. `kq8-viewer`: `load.rs` loads `KQChest`s like doors with their chest keys (default type 0x8021); `door_view.rs` builds, moves (box) and draws them with the doors' per-frame models, hidden ones not drawn. README, DESIGN, provenance.
**Why:** Step 4 of chests and items (`chests.md`).
**Tested how:** 7 new unit tests (`chest.rs`: `open` plays once and stays on its last frame, remarks; `active.rs`: the three clicks with `EndLoop open` after 16 updates and the crystal inside the chest's box picking the chest, no item → empty, item already picked up, too far / item in hand / not clickable / hidden then revealed). Viewer: `--terrain --world Daventry --at 202336,262000,1.5708 --click smlchst3,10 --click smlchst3,40 --click smlchst3,45 --frames 1200`; Swamp, IceWorld, Gnome `--frames 60`. Whole workspace `cargo test`; clippy: no new warnings.
**Result:** 175 tests pass in `kq8-game`, 62 in `kq8-data`. Daventry: 3 chests (`smlchst3` 3 crystals, `smlchst4` 200 silver coins, `wizchest1` lady bell, hidden, type 0x1); `open` turns them 0°. Clicks on `smlchst3` (Connor 3,208 from its box): 1st → `open` plays (cue `chstopen` at update 11, finished at 25, `EndLoop open`), `Crystal.23` x3 made at (202336, 263155, 11550); 2nd → moved into Connor's inventory 0 → 3, state empty; 3rd → Empty remark [0, 109, 14, 33, 1, 1] logged. Swamp 4 chests, IceWorld 2, Gnome 0: all load.
**Still broken / not tested:** **Not checked by eye** (lid opening, the crystals inside). The remarks and the chest sound are logged only (M4). Connor doesn't turn to face the chest before a remark (as doors). The test above clicks from the pub's ground floor, through the ceiling, which `doors.md` Q45 b allows. **The user checked the original (2026-10-13): clicks are blocked by walls and the ground, unless the camera sees the object.** Asked as Q63; planned as 3e-4b (applies to doors, chests and items). No pickup sound on the 2nd click (Q61).
**Next:** You: open the pub chest in the viewer (below). Then 3e-5, scripted chests and baskets.

## 2026-10-13 M3 3e-3: items at the start, drawn and picked up in the viewer

**Changed:** `kq8-data/src/world_script.rs`: the start-up runner passes `KQMonster::inventory`, `give`, `KQObject::setScript` and `setDontLook` to its host and sets `$NewInvItem`. `kq8-game/src/item.rs`: one reader for `inventory`/`give` lines (`read_make`), used by `active.rs` too; `describe` for the logs. `kq8-viewer`: `load.rs` reads the item types and boxes, makes the start items (z −1/−2 → ground), loads the models of every type the world can make or hold; new `item_view.rs` draws world items (see-through by the type's translucency) and hides picked-up/hidden/deleted ones; `door_view.rs` hands the items and the ground to the running game and writes `kq8-artifacts/logs/items.log`; `terrain_view.rs`, `main.rs` wiring. README, DESIGN, provenance.
**Why:** Step 3 of chests and items: the items Daventry puts out at the start should be there and clickable.
**Tested how:** New unit test (start-up item lines, `$NewInvItem`, lines inside `if`). Whole workspace `cargo test`; clippy: no new warnings. Viewer runs: `--terrain --world Daventry --frames 200`; `--at 266011,221900,0 --click Mushroom.14,10 --click Crystal.20,12 --click Mushroom.14,14`; `--at 228081,202000,0 --click SilverCoins.10,10 --click SilverCoins.11,12`; Swamp, Gnome, DeadCity `--frames 60`.
**Result:** Daventry: 135 types (132 with a click box); 35 item lines, 22 items, all in the world, `BirthdaySuit.22` hidden; mushrooms at −1 put at 7,650 / 13,770; models for 25 types, none failed. Mushroom.14 picked up from 507 away (count 0 → 1), Crystal.20 "too far" (32,820), clicking Mushroom.14 again does nothing. Farm coins: 0 → 7 → 12, `Farmcoin.cs` runs (Connor's remark logged as a stub) and removes its own script. Swamp 63 items (12 carried by characters), Gnome 74 (13), DeadCity 30 (11); all load and run. Kept 62 + 169 tests passing.
**Still broken / not tested:** **Not checked by eye**: whether the items look right and sit right (on tables, on the ground). No inventory screen (M4), no sounds. Gnome names `INVITEM_GreenMushroomPiece`, whose model `schroomC.kqs` is only in Swamp's archives: not loaded (no Gnome start item is one). Item names are our stand-in (Q60). Chests are next (3e-4).
**Next:** You: run the viewer and look at the items (below), click a few. Then 3e-4, chests.
**Checked afterwards (you, 2026-10-13):** the crystal looks right and disappears when clicked. You saw each pickup line **twice** in the terminal (`items:` and `doors:`): every item line is also kept in `doors.log`, and both logs copied click lines to the terminal. Fixed in `door_view.rs`: the doors' copy no longer goes to the terminal; both log files keep their line. Re-run: `--at 291749,198237,0 --click Crystal.20,10` gives the line once in the terminal, once in each log; door clicks (`Condoor`) still show.

## 2026-10-13 M3 3e-2: item objects, inventories, picking up (game rules only)

**Changed:** New `crates/kq8-game/src/item.rs` (item objects: type, quantity, where they are, hidden, script; the list of all of them). `active.rs`: scripts can now make items (`KQMonster::inventory`, `give`), look for them (`KQMonster::checkInventory`), and change them (`KQObject::setScript`, `setDontLook`, `setHidden`, `deleteObject`); clicking a world item picks it up; item lines collected for an `items.log`. `script.rs`: a world command can set script variables (for `NewInvItem`); `checkInventory`, `deleteObject`, `setHidden` are no longer stubs, `tossItems`, `dropItem`, `changeBmp`, `setGroupsAlways` are new stubs. `docs/DESIGN.md`, `docs/provenance/README.md`. No viewer changes.
**Why:** Step 2 of chests and items (`items.md` rules 3-10): the rules every item, chest and basket needs, testable without the window.
**Tested how:** 8 new unit tests (`cargo test -p kq8-game`): making items in the world and carried, `NewInvItem`, z −1 → floor height, errors that make nothing, `checkInventory` 1/0 and its variable, picking up (too far, item in hand refused, in reach → carried, pickup sound logged, the item's script runs once and removes itself like `Tavcoin.cs`, the pick ray then misses it), hidden and deleted items can't be clicked. Whole workspace `cargo test`; clippy clean on `kq8-game`. Viewer run (`--terrain --world Daventry --frames 300`) to check nothing else changed.
**Result:** 169 tests pass in `kq8-game` (was 161), 61 in `kq8-data`. Viewer: starts and exits normally; 17 doors, 52 traps, as before; no errors.
**Still broken / not tested:** Nothing visible yet: the viewer doesn't load item types or items (3e-3), so in the running viewer every item command reports "no such item type". Item names are our stand-in (Q60). Not done: tossing (`tossItems`), dropping, the remarks and sounds themselves (M4), Connor turning or bending. Clippy on the whole workspace stops at an older test in `kq8-data/src/objects.rs` (`3.14` taken for π), not from this step.
**Next:** You: `cargo test -p kq8-game` (should end `169 passed`). Then 3e-3: items at the start (`WorldInv.cs`) drawn and clickable in the viewer.

## 2026-10-13 Smaller build folder (no debug info for dependencies)

**Changed:** `Cargo.toml`: `debug = false` for all dependencies in dev builds (our own crates keep their debugging information). Deleted `target/side/` (a spare build copy from 2026-10-11, see the M1 3a entry) and the old dev builds (`cargo clean --profile dev`). Nothing in the repo besides `Cargo.toml`.
**Why:** `target/` had grown to 28 GB: 12 GB in `target/side/`, 13 GB of dev builds (many outdated copies, and a ~1.2 GB `.pdb` per viewer build, mostly Bevy's debugging information).
**Tested how:** Rebuilt from scratch: `cargo test --workspace` (6 min including the rebuild) and `cargo build -p kq8-viewer`; measured with `du`.
**Result:** All tests pass (61 + 161). Debug viewer: `kq8_viewer.pdb` 105 MB (was ~1.2 GB), exe 111 MB. `target/debug` 3.4 GB with the viewer, 2.5 GB without; whole `target/` 28 GB → 6.7 GB. Release builds unchanged.
**Still broken / not tested:** A debugger can no longer step inside Bevy's code (nobody does here).
**Next:** Run `cargo clean --profile dev` now and then; close the viewer before builds so no side copy is needed.

## 2026-10-13 M3 3e-1: the item type list (`Invinfo.itl`)

**Changed:** Plan for chests and items in `docs/DESIGN.md` (M3 3e, five sub-steps). New `crates/kq8-data/src/items.rs` (reads `GAME/Kq/Invinfo.itl`, the type-name rule), new CLI command `kq8 item-verify` (`crates/kq8-cli/src/item_verify.rs`, `main.rs`). Questions Q60, Q61, Q62 in `docs/behaviour/QUESTIONS.md`. `README.md`, `docs/provenance/README.md`.
**Why:** Step 1 of chests and items: every item (coins in a chest, mushrooms in the world) needs its type's model, sounds and size, and scripts name types loosely (`give mushroom 5`, `invItemTypeName=silverCoins`).
**Tested how:** Unit tests (field positions, `none` sounds, a line without a shape, the `INVITEM_` prefix rule). `cargo run --release -- item-verify`, log `kq8-artifacts/logs/item-verify.log`. Whole workspace `cargo test` passes; clippy clean on the two crates.
**Result:** 135 types read (`numItems=135`), 0 problems, no name twice; 16 categories (QuestItem 69, LongWeapon 20, ShortWeapon 13, ...). All 135 models found (world archives or `GAME/Common`); 3 have no box in `BOUNDS.KQ` (`truthkey.kqs`, `tablet2.kqs`, `orcxbow.kqs`). 3,523 scripts and object files in all 12 worlds name item types 758 times (182 different spellings); all resolve with the prefix rule except `INVITEM_All` (35 `acceptsInventory` values) and `2050` (an id number, Gnome `ARMORGNM.CS` line 293). Found on the way: `acceptsInventory` can be a comma-separated list (19 files).
**Still broken / not tested:** Nothing in the game uses the list yet (3e-2 does). Asked: Q60 (the name of a new item object; stand-in planned), Q61 (does a chest's 2nd click play the pickup sound), Q62 (`acceptsInventory` lists and `INVITEM_All`, type by id number, the 3 items without a box). The door code still compares `acceptsInventory` exactly (only matters once Connor can hold items, M4).
**Next:** You: run `item-verify` and check the numbers. Then 3e-2 (item objects, inventories, the item script commands, picking up).

## 2026-10-13 Study: terrain lighting (light and shadow maps)

**Changed:** new `docs/behaviour/terrain-lighting.md`; `docs/behaviour/terrain.md` (the `lightScale` and map-name rows point to it). Docs only, no code.
**Why:** The user wants lighting implemented, terrain first; there was no study yet.
**Tested how:** Traced in `Mask.exe` (Ghidra; scratch notes in `re/`, not committed): the `TERRAIN.KQ` reader, the map loaders, the `useDynamicLighting` switch, the per-square light textures, the Glide texture setup, and the developer relighting tool. Checked against the data: file sizes and counts in all 7 worlds that have maps, value histograms of Daventry's four maps, and a recomputation of the sun's hill shadows from the height map.
**Result:** File = u32 count + (128 × 2^lightScale + 1)² 16-bit values (Daventry 513 × 513, one point every 1,024 units), row by row along +x, rows along +y, same way round as the height map (the recomputed shadows match 97.5% as stored, 87–89% if flipped or turned). Each value = 4-bit brightness, red, green, blue; Daventry hardware maps are grey 7 (sun) / 4 (shadow). With `useDynamicLighting 1` (this install's `Options.cs`) the shadow map is shown, else the light map; Daventry's two differ in 761 points only. No file → full brightness.
**Still broken / not tested:** **Not settled: whether the hardware light value is doubled** (traced Glide setup multiplies without doubling, giving sunlit ground at 47%; the software maps' 15/9 suggest doubling, 93%). Needs the user's eye. Filtering between points assumed bilinear. Brightness setting and moving lights not studied.
**Next:** B implements from `terrain-lighting.md` with the ×2 stand-in and a switch for ×1; the user compares with the original.

## 2026-10-13 Study: Q59 (lean during a jump), not settled

**Changed:** `docs/behaviour/connor-movement.md` ("Turning" rule 3: jumps, falls and landings now follow B's stand-in; new open question 4), `docs/behaviour/QUESTIONS.md` (Q59 status), `STATUS.md` (stuck item). Docs only.
**Why:** The user saw Connor upright during a running jump with the turn key held, against my Q56 answer.
**Tested how:** Traced again in `Mask.exe` (Ghidra) and scanned the program for every instruction that writes the lean, its switch, the turn direction, the forward intent, the turn on/off flag and `allowLeaning`.
**Result:** **Failed to find the cause.** The code still says the lean carries on through a jump: the jump start leaves the switch on, turning works in the air, the lean is updated in every move, the jump animations have no rotation, and nothing outside the lean rule changes the lean. Found on the way: four special states (not named; probably ropes and the like) switch leaning off while active. My earlier "jumps keep the lean" is therefore unconfirmed and contradicted by the user; the note now tells B to keep the stand-in.
**Still broken / not tested:** The cause is unknown. `Special` and death states: from the code only.
**Next:** ~~User repeats the jump test.~~ **Closed the same day:** the user compared the original with our build and accepted B's behaviour (jumps, falls and landings straighten him) as correct; Q59 closed, no debugger run, no code change.

## 2026-10-13 M2 2i-4: a jump straightens Connor up (user's check)

**Changed:** `crates/kq8-game/src/connor.rs` (the lean switch is off while jumping, falling and landing; the jump test now expects him to straighten up). `docs/behaviour/QUESTIONS.md` (Q59). Docs: DESIGN (2i-4), provenance, README.
**Why:** The user checked in the original: Connor does not stay leaned through a jump, even holding the turn key. 2i-3 followed the traced rule (the switch is kept in jumps), which disagrees; asked as Q59.
**Tested how:** `cargo test --workspace`; viewer `--terrain --hold 1,1,1 --jump 20 --frames 900`, reading `connor.log`.
**Result:** 219 unit tests pass (58 + 161). Viewer: lean 0.180 at the jump (update 21: `runJump`), then 0.150, 0.120 … upright by update 26, turn key held throughout.
**Checked by the user (2026-10-13):** by eye, it now looks correct (as in the original).
**Still broken / not tested:** Whether the original straightens him gradually or at once, and what the crash, death and `Special` states do, wait for Q59.
**Next:** User checks by eye; then chests and items.

## 2026-10-13 M2 2i-3: banking as traced (ends at 0.18, the start leans, jumps keep it)

**Changed:** `crates/kq8-game/src/connor.rs` (`lean_allowed` switch, the traced step rules, `BANK_LIMIT` replaces `BANK_MAX`; banking tests rewritten, 1 more). Docs: DESIGN (2i-3), provenance, README.
**Why:** Session A traced banking (Q56, `connor-movement.md` "Turning" rule 3): the lean ends at 0.18 rad, not 0.15; the running start leans and the stops don't; jumps and falls keep the switch; the snap is checked before the step.
**Tested how:** `cargo test --workspace`; viewer `--terrain --hold 1,1,1,45 --frames 600`, reading `connor.log`.
**Result:** 219 unit tests pass (58 + 161). Viewer: lean from update 1 during `stoprun` (before: from 12, after the start), 0.030 … 0.180 by update 6, held; keys let go at 45: 0.150 … 0.030, upright at update 51 during `runstopleft`.
**Still broken / not tested:** Not checked by eye after the change (the user checked 0.15 by eye; 0.18 is about 1.7° more). The jump rule (the lean carries on in a jump while the turn is held) only unit-tested. First-person view and `allowLeaning` not done.
**Next:** User checks by eye; then chests and items (`chests.md`, `items.md`), planned first.

## 2026-10-13 M3 3d-4: traps after the Q57/Q58 answers

**Changed:** `crates/kq8-game/src/trap.rs` (touching swept along the move; mask only gates coming in; 3 new unit tests), `active.rs` (log wording; test), `crates/kq8-viewer/src/door_view.rs` (Connor's move since the last check). Docs: DESIGN (3d-4), provenance.
**Why:** Session A's answers: `traps.md` rule 6 (the touch is the swept shape test) and rule 16 (those inside still get `Off` after a mask bit is removed). Q58 confirmed the one-sided stand-in.
**Tested how:** `cargo test --workspace`; viewer `--at 270813,225311,-1.5708 --hold 1,0,0,80 --click Condoor,2` and `--at 186500,198000,1.5708 --hold 1,0,1,90`, comparing `traps.log` with 3d-3.
**Result:** 218 unit tests pass (58 + 160). Every `On` at the same update as before (18, 26, 23); every `Off` one update later (`Conrun.tp` 25 instead of 24, `Conwalk.tp` 39 instead of 38, `mwiztrap` 78 instead of 77), from the swept stay test.
**Still broken / not tested:** The swept test for staying is our stand-in (rule 7 doesn't say). Not checked by eye (one update, 1/15 s, later `Off`; the walk/run switching happens on `On`, so nothing visible should change).
**Next:** 2i-3, the banking correction (Q56).

## 2026-10-13 Study: touches with Connor, rolling rocks, trap masks, banking (Q55–Q58)

**Changed:** new `docs/behaviour/connor-collision.md`; `traps.md` (rules 4–6, 16, numbers, open questions), `connor-movement.md` ("Turning" rule 3, "Do not reproduce"), `connor-physics.md` (two "not traced" items now point to the new note), `QUESTIONS.md` (Q55–Q58 marked answered). Docs only, no code.
**Why:** Study session A for Q55 (`conCollisionAction`, `ConnorCollision`, the rolling rocks' hit), Q56's open points (banking), Q57 (trap masks outside Daventry) and Q58 (one-sided trap mask), which belongs to the same topic.
**Tested how:** Traced in `Mask.exe` with Ghidra (scratch notes in `re/`, not committed); checked against the data: counted trap masks and object types in all 12 worlds' `kq/` folders, read the scripts that use them, summed `C_rctbk.kqs`'s steps, and simulated the lean steps in 32-bit floats.
**Result:** A moving object passes through Connor without stopping or pushing him; its script (`ConnorCollision`, 0.2 s repeat limit per direction, `$3` = 1 for "on top") does the rest. The rocks' hit costs 1 point (sometimes 2) of 10 at level 1 and plays `reactback` (1.933 s, 903 units back). Traps: only the trap's mask counts (B's Q58 stand-in is right); items, solid objects and creatures set off the non-Connor traps; removing a bit doesn't send `Off`. Banking: lean allowed exactly when the pick is `run`/`waterrun`; **correction: the lean ends at 0.18 rad, not 0.15**.
**Still broken / not tested:** Nothing here was checked in the running game. Not traced: what `makeActive` changes, the exact cylinder size for ball-stack against ball-stack, the order of objects' moves in an update, whether `Special` can start from every state. Connor getting stuck inside a rock is from the code only.
**Next:** B can use the answers (Q55 for Gnome, Q56 for the lean's end value). User checks suggested in `connor-collision.md` "How to check".

## 2026-10-13 M3 3d-3: walk and run zones change Connor's pace and the camera

**Changed:** `crates/kq8-game/src/active.rs` (`KQConner::setRun`, `KQCamera::setRange` handed to the caller as requests; unit test), `script.rs` (26 more stubs: the action-only commands of Daventry's trap scripts), `crates/kq8-viewer/src/door_view.rs` (`trap_step` applies the requests), `connor_view.rs` (`set_run`; `--hold` sets the run switch only at the start), `game_camera.rs` (`set_range`). Docs: DESIGN, provenance, README.
**Why:** M3 step 3d-3: what the walk/run zones do (`walk_la.cs`, `run_la.cs`).
**Tested how:** `cargo test --workspace`; viewer `--terrain --at 270813,225311,-1.5708 --hold 1,0,0,80 --click Condoor,2` (walk in through Connor's front door), reading `traps.log`, `connor.log`, `camera.log`.
**Result:** 215 unit tests pass (58 + 157). Update 18 `Conrun.tp` On: run switch on, `walk -> run` at 19, camera range (1,500, 5,000); update 24 Off; update 26 `Conwalk.tp` On: run switch off, `run -> walk` at 27, camera range (1,000, 2,500). Tried first: walking out of the house from (268280, 225448) east: he walked into the closed door; from (268050, 225448) with the door clicked: the door opens inwards and stopped him (head-on, then "started inside Condoor's box"); the way in from outside works.
**Checked by the user (2026-10-13):** by eye, walking in and out of Connor's house: the same behaviour as in the original.
**Still broken / not tested:** The walk zone covers the house inside, so walking out the door is only tested with the door already open from outside. Music, ropes, teleporters, monsters and cutscene commands are stubs (logged only); `checkSave`, `getloc` and other question commands log as unknown.
**Next:** User checks by eye; then chests and items (`chests.md`, `items.md`), planned first.

## 2026-10-13 M3 3d-2: Daventry's traps switch on and off

**Changed:** `crates/kq8-game/src/active.rs` (traps in the world objects: `Init`, `connor_moved`, `KQCueEvent` to traps, `KQObject::collision`, a separate trap log; 2 unit tests), `crates/kq8-viewer/src/load.rs` (traps loaded; game-wide scripts from `GAME/patch` and `GAME/Kq`), `door_view.rs` (traps created with the doors, new `trap_step` after Connor's update, `traps.log`), `main.rs` (order). Docs: DESIGN (3d-2), provenance, README.
**Why:** M3 step 3d-2 (`traps.md`).
**Tested how:** `cargo test --workspace`; viewer runs in Daventry reading `traps.log`: `--at 277744,199295,3.1416 --hold 1,0,1,40`, `--at 268280,223800,3.1416 --hold 1,0,1,30`, `--at 186500,198000,1.5708 --hold 1,0,1,90`, `--at 143974,206673,1.5708 --hold 1,0,1,40`.
**Result:** 214 unit tests pass (58 + 156). 52 traps loaded (4 sticky, start shrunk), every script gets `Init`; 174 world + 30 game-wide scripts. `jumptp`: On at update 11 when his balls reach its box (y 199,822, box from 200,000); `jumpa.cs` sets its variable, asks for popup 133 (stub) and removes bit 4 (mask 0x4 -> 0x0). Connor's house `Conwalk.tp`: On at update 15, Off at 21; `walk_la.cs` runs (found in `GAME/Kq` after adding game-wide scripts; first try: "not found"). Sticky `mwiztrap`: On at x 189,041 (inner box), grows to full, Off at x 198,106 (full box), shrinks again; the overlapping `wiztrap` switches on separately. Ropes: `ropetrap` On at the foot of the wizard's cliff; rope commands logged as unknown.
**Still broken / not tested:** Nothing visible yet: `KQConner::setRun`, `KQCamera::setRange` (3d-3), music, rope, monster and cutscene commands log as unknown. The note says Daventry has 50 traps, we place 52 (no duplicates, none inside an if-block); not settled. `mwzhtrap` (the wizard's house) is about 5,000 above the ground where we tested, on top of the cliff, so not reached. **Found, not caused by this step:** running east at y 204,700 from x 140,000, Connor gets stuck in Fall (`jumphold`) at about (141,850, 204,070) from update ~28, sinking a few units per update; the same on the previous commit. Not investigated.
**Next:** 3d-3: `setRun` and `setRange` from the walk/run zones; stubs for the action-only commands.

## 2026-10-13 M3 3d-1: the trap rules

**Changed:** `crates/kq8-game/src/trap.rs` (new: `Trap`, entering / staying / leaving, sticky boxes, one-shot; 6 unit tests), `lib.rs`. `docs/DESIGN.md` (plan 3d), `docs/behaviour/QUESTIONS.md` (Q58), `docs/provenance/README.md`.
**Why:** M3 step 3d, traps (`traps.md`), planned in `docs/DESIGN.md` first.
**Tested how:** `cargo test --workspace`.
**Result:** 212 unit tests pass (58 + 154). Found while testing: by `traps.md` rule 4's two-sided pair rule every trap would see Connor (his mask 0x281B has the trap type 0x10), which contradicts rule 16; asked as Q58, stand-in: only the trap's mask counts.
**Still broken / not tested:** Rules only; nothing in the viewer yet (3d-2). "Touching" is tested where Connor ends each update, not along his path (stand-in).
**Next:** 3d-2, traps loaded in Daventry and their scripts run.

## 2026-10-13 Questions: Q55 data findings, Q57 trap masks

**Changed:** `docs/behaviour/QUESTIONS.md` only (Q55: what the data shows; new Q57).
**Why:** Found while reviewing the new notes: the rolling rocks' own script handles the hit on Connor (`conCollisionAction=1`, `rollside.cs` `ConnorCollision`: `reactback`, `setRelDamage 0.2`, `KQAttackEvent`); and 59 of 344 trap files outside Daventry don't use mask 4, against `traps.md` rule 4.
**Tested how:** Read and counted the data files in `MASK/*/kq/` (grep for `classType=KQTrap`, `colliderMask`, `conCollisionAction`, `ConnorCollision`).
**Result:** 12 objects use `conCollisionAction=1`, 2 of them in Daventry (`bricks`, `waterfal`; `waterfal.cs` makes a remark on `ConnorCollision`). Trap masks: 285 × 4, the rest 0, 1, 2, 5, 6, 7, 516, 517, 519; Daventry's 58 are all 4.
**Still broken / not tested:** No code change. Both questions wait for the study session.
**Next:** M3 traps for Daventry (not blocked by these questions).

## 2026-10-13 M2 2i-2: Connor leans while running through water too

**Changed:** `crates/kq8-game/src/connor.rs` (`waterrun` leans like `run`; unit test), `docs/behaviour/QUESTIONS.md` (Q56 partly answered by the user), `docs/DESIGN.md` (2i), `docs/provenance/README.md`, `README.md`, `STATUS.md`.
**Why:** The user checked 2i by eye against the original: the size of the lean and the pivot at his feet are right, and he does lean while running through water (we didn't).
**Tested how:** `cargo test --workspace`; viewer `--terrain --at 141312,198656,0 --hold 1,0,1 --turn 60,1 --frames 2500`, reading `connor.log`.
**Result:** 206 unit tests pass (58 + 148). Submerged at update 35, `runwaterrun` 36–43, `waterrun` from 44; turn key from 60, `lean starts: 0.030 rad (left)` at 61, leaning for the remaining 168 updates while he circles in the water.
**Still broken / not tested:** Wading lean not checked by eye yet. Still stand-ins (Q56): no lean during the start/stop animations, half turns lean fully, drawing only.
**Next:** User looks at the wading lean; back to the M3 list (traps).

## 2026-10-13 M2 2i: Connor leans into turns while running

**Changed:** `crates/kq8-game/src/connor.rs` (`Walker::bank`, the lean rule, 4 unit tests), `crates/kq8-viewer/src/connor_view.rs` (draws the lean, `connor.log` new last column `bank`, event lines when the lean starts and ends), `docs/DESIGN.md` (plan 2i), `docs/behaviour/QUESTIONS.md` (Q56), `docs/provenance/README.md`, `README.md`, `STATUS.md`.
**Why:** The user asked for banking (`connor-movement.md` "Turning" rule 3).
**Tested how:** `cargo test --workspace`; viewer runs `--hold 1,1,1,45`, `--hold 1,-1,1,30` and `--hold 1,1,0,40` in Daventry, reading `connor.log`.
**Result:** 205 unit tests pass (58 + 147). Running left: no lean while `stoprun` plays (updates 1–11), then 0.030, 0.060 … 0.150 from update 16, held; keys let go at 45: 0.120 … 0.030, upright at update 50. Running right: −0.150. Walking and turning: lean 0 throughout.
**Still broken / not tested:** Not checked by eye. Details are stand-ins (Q56): only the `run` loop leans (not `waterrun`, not the start/stop animations), pivot at his feet, always 0.15.
**Next:** User checks it by eye; back to the M3 list (traps).

## 2026-10-13 M3 3r-3: review of Q45–Q52, docs

**Changed:** `docs/DESIGN.md` (new step 3r with a table: each answer, what our code does, what changes; plans 3a, 3b, 3b-4, 3c corrected where they said "stand-in"), `docs/provenance/README.md` (rows marked confirmed or replaced; new section "Review after Q45–Q52"), comments in `script.rs`, `door.rs`, `effects_view.rs`, `anim_view.rs`, `README.md`, `QUESTIONS.md` (Q54, Q55).
**Why:** Session A answered Q45–Q52; the plans and code comments still called those points stand-ins.
**Tested how:** Read every answer against the code. Where an answer could matter, checked the data of all 12 worlds with a throwaway scanner (scratchpad, not kept): which emitters follow something, which use `followMesh`, which objects are `KQBoulder`, and whether the particle catchers' names match with capitals counting.
**Result:** Already as answered: Q45 a–f, Q46, Q47, Q50, Q52 b and d. No visible change needed: Q48 (all 13 catchers' names match exactly), Q52 a (`followMesh`: only the dragon fires and `lucfire`, not started yet), Q52 c (no emitter follows the camera). Code changes needed: Q49 (3r-1), Q51 rule 8 (3r-2), Q51 rule 5 (3r-4).
**Still broken / not tested:** Two small new questions: Q54 (the −1 a false text test can leave; `set`/`echo` results) and Q55 (what a moving rock does to Connor).
**Next:** Plan traps (`traps.md`), then chests and items.

## 2026-10-13 M3 3r-4: a rolling rock's collision box moves with it

**Changed:** `crates/kq8-viewer/src/anim_view.rs` (`anim_step` moves the collider of an object whose loop moves it; log lines in `anim.log`).
**Why:** Found while checking Q51: Gnome's rolling rocks have collision boxes (type 0x1, Connor collides with it) that stayed at the rocks' start while the rocks rolled up to 9,691 units away. `objects.md` Q51 rule 5: the box goes with the object.
**Tested how:** `cargo test --workspace --release`, clippy; viewer in Gnome (`--frames 2000`, about 11 s), read `anim.log`.
**Result:** 201 tests pass. `rrock1–3` each find their collider. `rrock1`'s box: y 111,710 at the start, 104,693 at update 15, 103,823 at 30, back to 111,316 at 45, and so on each 3.53 s cycle (x stays 57,500).
**Still broken / not tested:** **Not checked by walking into a rock.** The box moves one update after Connor's move (order not traced). If the box rolls over Connor, nothing happens to him: his next move "starts inside" the box, which our mover logs and ignores (Q55).
**Next:** You, in Gnome: walk to where a rock starts while it is away: you should no longer bump into an invisible box there.

**Checked afterwards (you, 2026-10-13):** the three rolling rocks still roll correctly. Walking Connor to where a rock starts, while it is away: he collides with nothing there. In the original that spot can't be reached: the rocks roll back and forth all the time, and when one hits Connor he stumbles and takes damage, so he would be hurt badly or die before getting there. Recorded in Q55.

## 2026-10-13 M3 3r-2: Gnome's boulder stands still

**Changed:** `crates/kq8-viewer/src/load.rs` (an object of class `KQBoulder` doesn't play its movement; one log line).
**Why:** `objects.md` Q51 rule 8: the `boulder` in Gnome's big room shows a rolling loop in its data, but the original shows it still; `KQBoulder` controls its own animation (not studied yet).
**Tested how:** viewer in Gnome (`--frames 60`), read `viewer.log`; throwaway scanner: `boulder` is the only `KQBoulder` in the game.
**Result:** `class KQBoulder: movement 0 not played, drawn still at its start`; Gnome now plays 233 objects (was 234); the three rolling rocks still play.
**Still broken / not tested:** **Not checked by eye.** The boulder can't be pushed (not studied).
**Next:** You: look at the boulder in Gnome's big room: it should stand still.

**Checked afterwards (you, 2026-10-13):** the boulder is now still; pushing it comes later.

## 2026-10-13 M3 3r-1: flowing water timed from each change

**Changed:** `crates/kq8-game/src/ground_anim.rs` (`update`: at most one change per check, next due = time of this change + `delay`; new test), `crates/kq8-viewer/src/ground_anim_view.rs` (checked every drawn frame on continuous game time instead of the 15-per-second update; log gaps in seconds), `main.rs` (moved from the fixed update to the frame update).
**Why:** `emitters.md` ground animator rule 3 (Q49): the original's animator doesn't use the 15-per-second tick; the next change is due `delay` after the moment of the change. Ours counted from the previous due time on the tick, so changes came 4 and 5 ticks apart (0.267 s and 0.333 s).
**Tested how:** `cargo test --workspace --release`, clippy; viewer in Daventry (`--frames 1500`, 8.5 s of game time), read `ground_anim.log`.
**Result:** 201 tests pass (58 + 143). 297 gaps between changes, 0.300–0.306 s, average 0.304 s (frames about 5.7 ms each), e.g. `AnimatedRiver1` at 0.3018, 0.6073, 0.9128, 1.2184 s.
**Still broken / not tested:** **Not checked by eye** (the difference from before is hard to see: about 1/30 s per change).
**Next:** You: look at Daventry's river: it should flow as before, a little more evenly.

**Checked afterwards (you, 2026-10-13):** the river flows nicely.

## 2026-10-13 Study: traps, chests, picking items up; Q45–Q52 answered

**Changed:** New notes `docs/behaviour/traps.md`, `chests.md`, `items.md`. Answers and corrections in `doors.md` (rule 8, rule 24, "Do not reproduce", new "Q45" section), `emitters.md` (emitter rules 2 and 11, catcher rule 5, ground animator rules 2, 3, 6), `objects.md` (new "Loops that move the whole object", Q51). `QUESTIONS.md`: Q45–Q52 marked answered (Q48 partly). Scratch notes in `re/` (not committed). No code.
**Why:** B needs behaviour notes for the rest of M3 (traps, chests, items) and answers to its open questions.
**Tested how:** Traced in `Mask.exe` with Ghidra (trap enter/leave and `trapType`, the chest's click steps, the item object and pickup, `KQMonster::inventory`/`give`/`checkInventory`/`tossItems`, the click line test, the script runner's `test` and `$Console::Return`, the per-update animation step, cue ranges at a wrap, emitter restarts and `followMesh`, the ground animator's timer and the square layout). Data read from all worlds' trap, chest and item files, `Invinfo.itl`, Gnome's `rrock1.kq`. Nothing run in the game.
**Result:** Traps: `On` when Connor touches the box, `Off` when he no longer touches it and his middle is out; `trapType=2` = shrunk inner box to enter, full box to leave (music zones). Chests: 1st click opens and puts the item at `invLoc`, 2nd click gives it to Connor, later clicks the "empty" remark; the chest's script is not run on clicks. Items: clicking in reach moves the item into Connor's inventory with its pickup sound; `give` = `KQMonster::inventory Connor`; `tossItems` throws items steeply (75–85°, 5,000–6,500/s). Questions: B's stand-ins for Q45 are all right; Q46 B right (each door's own file); Q47 drifting sway is right; Q50 the square's own orientation is right; Q51 steps are added up and Gnome's rocks are reset by a `KQMoveEvent` cue at 1.0 in the data; Q49 the wait restarts from each change (0.3 s on continuous time); Q52 `followMesh` = sit at that model part.
**Still broken / not tested:** Nothing checked in the original. Not settled: emitter-vs-catcher order (Q48), what `set`/`echo` leave in `$Console::Return`, the exact flight of a tossed item, the backward `close` steps of doors, an item's third number, whether Connor bends to pick things up. Q53 (B's camera follow-ups) not looked at yet.
**Next:** You: in the original, (1) walk slowly into the church yard until the music starts, step back a little: it should keep playing until you're clearly out; (2) open `smlchst3` (crystals): 1st click opens and shows them, 2nd click takes them, 3rd click Connor says it's empty; (3) watch a Gnome rolling rock: it rolls out and comes back to the same start each time. Then B can implement traps, chests and pickups; A can take Q53.

**Checked afterwards (you, 2026-10-13):** (1) The church yard has no music, the church inside does (can't walk in and out there); around Connor's house and near the wizard, music starts on entering and the world music returns on leaving, as the trap rules say. My "church yard" test was badly chosen; `traps.md` now points at the wizard's house (`mwzhtrap`) for the "step back a little" check. (2) Pub chests: `smlchst3` (big chest, second floor) opens and shows a crystal, the crystal can be clicked into the inventory, the next chest click says it's empty; `smlchst1` (small chest by the keg, a scripted object, not a `KQChest`) opens with the coins next to it, the next click says it's empty. One mismatch with the trace (I expected a silent click before "empty" after taking the crystal directly); recorded in `chests.md` with the user's behaviour as B's stand-in and a re-check.

**Checked again (you, 2026-10-13):** clicking only the big chest: open with the crystal inside → next click Connor takes the crystal → next click "empty". Matches `chests.md` rules 7–9 exactly. The case "pick the crystal up directly, then click the chest once" is still not settled.

**Checked a third time (you, 2026-10-13):** clicking "the crystal" then the chest also ends in "empty". Your explanation is right and the data confirms it: the crystal (290 tall, at 301 above the chest's base) lies wholly inside the chest's click box (2,060 tall), and the nearest clickable box wins, so that click was the chest's 2nd click. The trace's rules 7–9 hold with no exception; the stand-in in `chests.md` is withdrawn. (3) The three Gnome rolling rocks already look right in our viewer: B's method gives the same picture as the game's (steps added up plus the reset cue), so no change needed. The stationary ball in the big room is a different object (`boulder`, class `KQBoulder`, pushable): its data loop is a rolling animation, but the game shows it still, so `KQBoulder` controls its own animation. Not studied; `objects.md` Q51 rule 8 says not to play it until pushable objects are studied.

## 2026-10-13 M2 2f-2: the camera avoids walls

**Changed:** `crates/kq8-game/src/camera.rs` (`update_with`: obstacle steps 6–9, swing, drag creep, pull-in, "Connor is moving"; `ViewBlockers`; `ground_line_hit`), `collide.rs` (`line_hit` for buildings and boxes). `crates/kq8-viewer`: `game_camera.rs` (the blockers as they stand each update: ground, buildings, boxes, doors; log columns: moving, blocked by, hit, swing, pull-in, camera inside a building), `load.rs` (buildings default to type 0x28; view-blocking boxes with bit 0x8/0x1000), `terrain_view.rs` (`ViewBlockerSet`). Docs: DESIGN (plan 2f-2), provenance, README, QUESTIONS (Q53).
**Why:** `camera.md` "Obstacles" (study session 2026-10-13): the camera went through walls.
**Tested how:** `cargo test --workspace --release`, clippy; viewer in Daventry: Connor clicking his door open and walking in (`--at 270813,225311,-1.5708 --hold 1,0,0 --click Condoor,2`), circling inside and near the house (`--at 266000,222500,0 --hold 1,1,0`, `--at 266000,222500,3.1416 --hold 1,-1,1`); Swamp, DeadCity, Barren, IceWorld, Gnome, SnowExit started; read `camera.log` and `viewer.log`.
**Result:** 201 tests pass (58 + 143). Walking into the house: the wall blocks the view at update 39; the camera turns 0.084 rad towards his path and swings 0.056 rad per update, ending 0.039 rad past it at update 54 (within one step, as the note says). Circling inside the house with the camera starting outside: 38 updates blocked, 35 pulled in (to 3–1,337 from the aim point), the minimum pushing it out again each update (the note's edge case). In all three runs the camera is never inside a building's solid. View blockers: Daventry all 26 buildings and the 17 doors; DeadCity 41 boxes, Gnome 9, IceWorld 8; Barren's `smlrock` (0x21) doesn't block.
**Still broken / not tested:** Checked by eye by the user (2026-10-13): "so far it looks great". Dragging with the mouse into a wall was not reported on separately. Stand-ins: the ground test method and a line starting under water, the zero-width BSP walk, the 3D new direction (Q53). Not done: the camera updating only while Connor moves or the camera is used (Q53 d).
**Next:** Answers to Q53 from a study session; then traps, chests and picking things up (notes now arriving from the study session).

## 2026-10-13 M3 3b-4: emitters that follow something (IceWorld's snow)

**Changed:** `crates/kq8-game/src/emitter.rs` (`Emitter::new_following`, `follow_to`, `follow_loc`; velocities turned with the target; `followMesh` restarts; `Rng::fork`). `crates/kq8-viewer`: `effects_view.rs` (targets: Connor, the camera, placed objects; moved every update; logged), `terrain_view.rs` (the target lookup), `game_camera.rs` (`GameCam::position`). Docs: DESIGN (plan 3b-4), provenance, README, QUESTIONS (Q52).
**Why:** `emitters.md` rule 11; the one following emitter that exists at a world's start is IceWorld's `snow` (follows Connor), and its 10 catchers did nothing without it.
**Tested how:** `cargo test --workspace --release`, clippy; viewer in IceWorld with Connor running (`--hold 1,0,1 --frames 2500`) and in Daventry (`--frames 300`), read `effects.log` and `viewer.log`.
**Result:** 193 tests pass (58 + 135). IceWorld: `snow` (300 flakes) starts 4,000 above Connor and stays there as he moves (logged every second); particle 0 restarts inside the 8,000 box around that point; all 10 catchers find it and hid 296 flakes (Connor starts inside the lift building `Lvator`). Daventry unchanged: 14 emitters, 210 particles.
**Still broken / not tested:** **Not checked by eye.** The dragon fires (made by the dragon heads, fighting) and Temple4's `lucfire` (made by its altar script) still aren't running. `followMesh`'s part name, a missing target, the camera's turn and the order in a tick are stand-ins (Q52).
**Next:** You: walk around IceWorld and look at the snow (also at a building). Then a study session for traps and chests and Q45–Q52.

**Checked afterwards (you, 2026-10-13):** everything looks right: the snow follows Connor and stays out of buildings.

## 2026-10-13 M3 3c: animated objects play their movement

**Changed:** `crates/kq8-game/src/anim.rs` (new: a loop's phase, wrap, frame rule, random start and speed, the whole-object pose). `crates/kq8-viewer`: `anim_view.rs` (new: every frame built once per loop, the shown one visible, `anim.log`), `load.rs` (objects whose movement plays get all their loop's frames; the object palette loaded once), `terrain_view.rs` (those objects are no longer drawn as still models), `main.rs`. Docs: DESIGN (plan 3c), provenance, README, QUESTIONS (Q51).
**Why:** You saw "stuck drips" on the fountain: its water `wellwatr` is a 66-frame loop drawn frozen. Every animated object except doors was frozen the same way.
**Tested how:** `cargo test --workspace --release`, clippy; viewer in Daventry (`--frames 1500`), read `anim.log` and `viewer.log`; Swamp, DeadCity, Barren, IceWorld, Gnome, CastleD started (`--frames 40`).
**Result:** 190 tests pass (58 + 132). Daventry: 72 objects play, 40 different loops, 10 with a random start and speed (torches, dead birds, fish); `wellwatr` and `millwheel` move about one frame per update, as their stored lengths say. Other worlds: Swamp 426, DeadCity 213, Barren 135, Gnome 234, IceWorld 23, CastleD 8. Whole-object steps: none anywhere except Gnome's rolling rocks `rrock1–3` (up to 9,691 units), applied like the doors' (stand-in, Q51).
**Still broken / not tested:** **Not checked by eye.** Only the starting movement plays: scripts can't switch it yet, `nextAnimName` isn't done, no sounds (cues). Collision boxes don't follow the animation (as traced). The frame rule while playing and the rolling rocks are stand-ins (Q51).
**Next:** You: look at the fountain, the windmill, the mill wheel, the signs and the torches. Then commit if it looks right.

**Checked afterwards (you, 2026-10-13):** everything looks right in Daventry. Gnome's rolling rock spins; how it should move is left for later (Q51).

## 2026-10-13 Fix: animated ground squares keep their own orientation

**Changed:** `crates/kq8-viewer/src/load.rs` (`prepare_ground_animation`: every material an animated square shows is drawn with the orientation code of the square's own loaded material; log line counting the squares where that matters), `ground_anim_view.rs` (comment, a type alias and a clippy allow). Docs: QUESTIONS (Q50), provenance, DESIGN.
**Why:** You checked 3b by eye: leaves, catchers and drips are right, but the flowing water was turned wrongly. Daventry's shore squares were all turned the same way; in DeadCity the blood edges were right in one frame and turned 90° in the others; in Swamp most squares were wrong and some flipped for a single frame. Reading the direction tables (throwaway script): a shore or corner picture is one texture stored as up to four materials with different turns (Daventry 13–16 = `newriv1.bmp`, codes 0, 4, 1, 5), while the animation frames are one material each, mostly code 0. Drawing each frame with the new material's turn (the note's rule 6, as I read it) throws the squares' turns away and gives exactly what you saw.
**Tested how:** `cargo test --workspace --release`, clippy; viewer start-up in Daventry, DeadCity, Swamp; read `viewer.log`.
**Result:** 185 tests pass. Squares whose turn differs from a cycle material's (the ones this changes): Daventry 116, DeadCity 42 (`Blood2` 4, `Blood3` 38), Swamp 376.
**Still broken / not tested:** **Not re-checked by eye.** A stand-in from your check and the data; the note says otherwise, asked as Q50. The "stuck drips" at the fountain are not emitter particles: they are `wellwatr`, the fountain's 66-frame looping water animation, drawn frozen at frame 0 because animated objects don't play yet (not changed here).
**Next:** You: look at the water again in the three worlds. Decide whether animated objects (like `wellwatr`) should play next.

**Checked afterwards (you, 2026-10-13):** everything looks correct now: leaves, the cottage catcher, the stream drips, and the flowing water in Daventry, DeadCity and Swamp. Next: animated objects play their animations.

## 2026-10-13 M3 3b-3: ground animators (flowing water)

**Changed:** `crates/kq8-data/src/effects.rs` (`AnimatorSettings`), `terrain.rs` (`Grid::from_rows`, `Grid::set`, `Terrain::square_triangles`, which `ground_triangles` now uses). `crates/kq8-game/src/ground_anim.rs` (new: square choice, timed changes). `crates/kq8-viewer`: `ground_anim_view.rs` (new: one entity per animated square, texture and orientation follow its material, `ground_anim.log`), `load.rs` (animators collected; `prepare_ground_animation`; `Ground` keeps materials, orientations, texture names, palette), `terrain_view.rs` (animated squares' surface left out of the ground mesh; the U view's mode is now a resource), `main.rs`. Docs: DESIGN, provenance, README, QUESTIONS (Q49).
**Why:** `emitters.md` "Ground animators": Daventry's river and stream textures flow.
**Tested how:** `cargo test --workspace --release`, clippy; viewer in Daventry `kq8-viewer --terrain --frames 3000` (three runs), read `viewer.log` and `ground_anim.log`; Swamp, IceWorld, DeadCity started (`--frames 60`).
**Result:** 185 tests pass (58 + 127). Daventry squares per animator: 86, 0, 57, 19, 0, 17, 32, 2, 13, 4, 4, exactly the note's table; square (45, 57) is animated by none, (45, 53)–(45, 56) only by `FullStream`. First change at update 5 (entry 0), then 4 and 5 updates apart (16 each in 150 updates, 0.3 s on average). Swamp 25 animators / 676 squares, DeadCity 8 / 543, IceWorld 7 (`pond` has `startNow=0`, not started) / 263.
**Still broken / not tested:** **Not checked by eye**, and the U key with animated squares is not tested (no test option presses it). How 0.3 s falls on the 1/15 s clock is our stand-in (Q49). Ground flags don't follow a changed material (open question 3; no difference in Daventry). Average frame time varied 7.8–15.3 ms between three identical runs, so whether this step costs anything is not known (not investigated). Found on the way: `AnimatedRiver1`'s 86 squares are all material 57 (the note says 56 and 57; added to Q49).
**Next:** You: see the chat for where to look. Then a study session for Q47–Q49.

## 2026-10-13 Study: how the camera avoids walls (camera.md open question 4)

**Changed:** `docs/behaviour/camera.md`: new "Obstacles" section (replaces the two-line outline), the height rule uses the floor grid (correction), Data used, Edge cases, Do not reproduce, How to check, open question 4 answered. Scratch notes in `re/` (not committed). No code.
**Why:** You want the camera's wall avoidance implemented next; the note only had an outline.
**Tested how:** Traced in `Mask.exe` with Ghidra (the follow camera's update, the line test it uses, the "Connor moved" event, the object and building default collision types). A throwaway script (scratchpad, not in the repo) listed every `colliderType` in all 12 worlds' `.kq` files. Nothing run in the game.
**Result:** The camera tests a line from its aim point to itself against the drawn ground, buildings (default type 0x28, all block) and objects with collision bit 0x8 or 0x1000 (doors, a few blocks). Trees, signposts, fences and monsters don't block. When blocked while Connor moves: it turns towards the point halfway between his previous aim point and the hit, then keeps swinging (0.66 × that first turn per update) until it looks back along his path; if the new direction is blocked too, it is pulled in to 100 short of the hit. Dragging the camera into a wall with the mouse is refused (back to the last spot, 5% closer). Standing still, nothing is fixed until he moves. Correction: the camera's height rule uses the floor grid.
**Still broken / not tested:** Nothing checked in the original. Not found: what switches avoidance off (on by default). The swing's turn direction is inferred from the signs, not seen. The first update of a walk after standing doesn't swing (from the trace, not seen).
**Next:** You: in the original, walk round a corner of Connor's house close to the wall, and walk past a tree with the camera behind it (expect: the camera goes through the tree). Then B can plan the obstacle check from `camera.md` "Obstacles".

**Checked afterwards (you, 2026-10-13):** In the original: (1) the camera flies through trees; (2) round a house corner it never goes through the wall, swings round and comes closer when it has to; (3) dragging the camera into a wall is blocked, it comes closer to Connor, and feels very stuck and resistant, as the rule predicts (each update's drag refused, only a 5% creep). Recorded in `camera.md` "How to check". Still unchecked: standing still with a blocked view, doors, the numbers.

## 2026-10-13 M3 3b-2: particle catchers (leaves stop at the cottage)

**Changed:** `crates/kq8-data/src/effects.rs` (`CatcherSettings`), `crates/kq8-game/src/emitter.rs` (`Catcher`: hides an emitter's particles inside a turned building box until their restart), `collide.rs` (`BuildingCollider::bounds`, the building's own box). `crates/kq8-viewer`: `load.rs` (catchers loaded), `effects_view.rs` (looked up after the scripts, run after the emitters each update, logged), `terrain_view.rs` (buildings handed to them). Docs: DESIGN, provenance, README, QUESTIONS (Q48).
**Why:** `emitters.md` "Particle catchers": leaves must not fall through the cottage roof into the room.
**Tested how:** `cargo test --workspace --release`, clippy; viewer in Daventry `kq8-viewer --terrain --frames 7000`, read `effects.log` and `viewer.log`.
**Result:** 178 tests pass (56 + 122). `Cottage2/3/4` find `leaf1/2/3` and `Cottage` (`house1.dig`, turned −1.57; world box x 280,181–293,308, top 18,915). In about 620 updates `Cottage2` hid `leaf1` particles 19 times, all at x ≈ 280,190–280,250, just inside the box's west side, as the leaves drift east; `Cottage4` hid `leaf3` 4 times; `Cottage3` none (`leaf2` is far west of the cottage).
**Still broken / not tested:** **Not checked by eye.** The box is the building's whole bounding box, so leaves vanish in the air above the sloped roof too (that is what the note describes). Order catcher/emitter in a tick and the name case are stand-ins (Q48).
**Next:** 3b-3 (ground animators).

## 2026-10-13 M3 3b-1: emitters (falling leaves, stream drips)

**Changed:** `crates/kq8-data/src/effects.rs` (new: emitter keys and defaults). `crates/kq8-game/src/emitter.rs` (new: particles' choices, the per-tick rule, restarts, sway, gravity, `delayStart`; our own seeded random numbers). `crates/kq8-viewer`: `effects_view.rs` (new: particles drawn and run at 15 updates per second, `effects.log`), `load.rs` (emitters loaded with their particle models), `terrain_view.rs` and `main.rs` (wiring). Docs: DESIGN (plan 3b), provenance, README, QUESTIONS (Q47).
**Why:** `emitters.md` "Emitters": Daventry's leaves under the trees and the drips over the stream.
**Tested how:** `cargo test --workspace --release`, clippy; viewer in Daventry `kq8-viewer --terrain --frames 7000` (about 40 s), read `effects.log` and `viewer.log`.
**Result:** 176 tests pass (55 + 121). 14 emitters, 210 particles (`stream1–4` 40 each, `leaf1–10` 3–6 each); all above the ground, none moved by the placement rules. `stream1` particle 0: restart every 41 updates (maxLife 40 + 1), 1 update not drawn per life, rises from 9,724 to 10,002 then falls to 7,834; every life the same to 0.00. `leaf1` particle 0: life 151 updates, 1 not drawn; y and z repeat exactly, x does **not** (2,787 off in its second life, 5,400 in its third), because rule 3.6 keeps the sway angle across restarts while rule 2 says the path repeats: asked as Q47. Frames: 5.7 ms average.
**Still broken / not tested:** **Not checked by eye.** No catcher yet (leaves pass through the cottage roof; step 3b-2). Emitters that follow an object (IceWorld dragon fire and snow, Temple4) and arrow sparks are not started (logged). Particles get the emitter's `translucency` (likely, not traced). Stream drips go about 580 below the ground at the end of each life (the note says particles don't collide; whether that is visible in the original is not checked).
**Next:** 3b-2 (particle catchers). You: look at the leaves and the stream (see the chat for where).

## 2026-10-12 M3 3a-3: clicking doors, doors in the viewer

**Changed:** `crates/kq8-game/src/click.rs` (new: ray against a turned box, distance to a box in its own axes), `active.rs` (new: doors + event queue + scripts together; clicks with reach, `ConnorAction do do` / item / too far; `collisionType`; `Init` calls). `crates/kq8-data/src/objects.rs`: `movement_loops` (every movement of an object) and cue lines. `crates/kq8-viewer`: `door_view.rs` (new: doors drawn turned and showing their frame, left-click picking, `door_step` before Connor's update moves the door colliders), `load.rs` (doors loaded with all movements and frame steps, Connor's `attackDist`, clickable objects, the world's scripts and start-up variables), `terrain_view.rs` (`bevy_transform_from`), `main.rs` (`--click NAME,N` (repeatable), `--item TYPE`, systems). Docs: DESIGN (3a), provenance, README, QUESTIONS (Q45, Q46).
**Why:** `doors.md` rules 10–17 and 24: the player clicks a door; the door's script opens it; the open door must be out of the doorway but still solid.
**Tested how:** `cargo test --workspace --release`; viewer runs in Daventry with test clicks (no mouse), reading `doors.log`, `connor.log`, `viewer.log`: `--at 269813,225311,3.1416 --click Condoor,5 --click Condoor,40`; too far from the start spot `--click Condoor,3`; walking west into the doorway `--at 270813,225311,-1.5708 --hold 1,0,0` with and without `--click Condoor,2`; into the open leaf from inside `--at 267200,224000,1.5708 --hold 1,0,0 --click Condoor,2`; keep doors `--at 217414,279300,0` with and without `--item INVITEM_KeepKey`. Swamp, DeadCity, Barren, IceWorld started (`--frames 30`).
**Result:** 165 tests pass (53 + 112). Daventry: 17 doors, 105 other clickable objects, reach 4,700. Click from 1,000: `Condoor ConnorAction do do` → `KQDoorEvent 1` (update 5) → opening, first step update 6, frames 1…15 (12.81°, …, 165°, 167.75°, 165°) → opened at update 20 → `condoor.cs Condoor Open`. Second click: script runs, `open: ignored while opened`. From the start spot: distance 26,913.5, "too far", nothing sent. Door closed: Connor stops head-on at x 269,038 (225 from the face). Door open: he walks through the doorway into the house (to its far wall, 2,977 further). Into the open leaf: head-on stop, contact normal (−0.966, 0.260) = the leaf turned 165°. Keep doors: no key → "locked" (stub); key → both toggle open (180°, −180°) and become unclickable (`0x1021 → 0x1001`). Other worlds: 8 / 64 / 12 / 19 doors, all door scripts balanced; IceWorld `control.cs` uses `getEndLoop` (logged as unknown).
**Still broken / not tested:** **Not checked by eye; the mouse click is not tested at all** (only the `--click` option). No sound, no remarks, Connor doesn't turn to the door (rule 16), no hits/`Die`/`ProjImmune` (no combat yet), clicks on other objects only logged, `tooFar=` ignored (Q45 a). Stand-ins: picking ignores walls (Q45 b), reach uses the turned box (Q45 c), events/movement timing (Q45 d). `door3.kqs` opens in 1.6 s and 180° per the data, unlike `doors.md` rule 8 (Q46).
**Next:** You: see "How to test" in the chat (click Connor's door, watch it swing, walk in). Then a study session for Q45/Q46.

**Checked afterwards (you, 2026-10-13):** The doors work correctly by eye, and clicking with the real mouse works too; the clicks show up correctly in `doors.log`. Recorded in README and DESIGN (3a). Still not done: everything else in "Still broken" above (sound, remarks, Q45/Q46 stand-ins).

## 2026-10-12 M3 3a-2: the door and the event queue

**Changed:** `crates/kq8-game/src/door.rs` (new: four states, `KQDoorEvent`, `KQAnimPlayEvent`, movements by phase, cues, `Open`/`Close` on finishing, the turn worked out from the phase), `events.rs` (new: `sendEvent` queue, time then send order). Docs: DESIGN (plan 3a).
**Why:** `doors.md` rules 1–9, 22–24.
**Tested how:** `cargo test -p kq8-game --release`; a throwaway program (scratch folder) chaining `door1.kqs` and `keepdrl.kqs` frame steps.
**Result:** Unit tests: state table (14 cases), `open` 15 moving updates, `close` from 0.93 14, cue at 0.0 fires in the first moving update, a close cue at 0.95 never fires, `KQAnimPlayEvent open` ends opened with `Open`, 10 open/close cycles return exactly to the closed pose, event order and delays (0.07 s = two updates). Data: `door1.kqs`'s chained steps all turn about one hinge (−634, −63).
**Still broken / not tested:** Which update a movement first moves in, and whether `sendEvent 0` lands in the same update, are our stand-ins (Q45 d).
**Next:** 3a-3.

## 2026-10-12 M3 3a-1: the script runner

**Changed:** `crates/kq8-game/src/script.rs` (new): words, `#`, quotes, `$` arguments and variables (unset ones vanish), `if`/`else`/`endif` with `else if` as else + nested if, `test`, `$Console::Return`, `set`, `echo`, scripts calling scripts, stubs for later-milestone commands, world commands handed to the caller. The start-up follower (`world_script.rs`) is unchanged.
**Why:** `doors.md` "The script language".
**Tested how:** `cargo test -p kq8-game --release` (7 tests on made-up scripts, not copies of the game's); a throwaway program counting `if`/`endif` pairing over every Daventry `.cs`.
**Result:** 7 tests pass. All 174 Daventry scripts balance under "`else if` needs its own `endif`".
**Still broken / not tested:** `$Console::Return` values other than "0 = true", and how a number word is read, are stand-ins (Q45 e, f). Commands outside doors.md's list fail as unknown (logged).
**Next:** 3a-2.

## 2026-10-12 Study: emitters, particle catchers, ground animators

**Changed:** New note `docs/behaviour/emitters.md` (study session A). Scratch notes in `re/` (not committed).
**Why:** M3 item 3: Daventry's falling leaves, stream drips, and the flowing-water ground textures.
**Tested how:** Traced in `Mask.exe` with Ghidra; keys collected from all 12 worlds' `.kq` files; Daventry's animator squares counted from `terrm.dib`, flags from `envinfo.kq` (throwaway script, not in the repo). Nothing run in the game.
**Result:** Emitter: per-tick particle rules (random offset in a box, velocity, spin, sway by sin, gravity as age × g, restart after `maxLife` + 1 ticks with the same path, hidden for one tick, `delayStart` = appear one by one; defaults 200 particles, 100 ticks). Catcher: hides an emitter's particles inside a building's box (the cottage's leaves). Ground animator: picks squares by material (or an explicit list), sets them all to the next material of the cycle every `delay` (0.3 s in Daventry); per-animator square counts for Daventry. `randomZRot` has no effect.
**Still broken / not tested:** Not checked in the game. Not traced: what makes an emitter active, whether particles get `translucency`, whether changed ground materials change the envInfo flags (no difference in Daventry).
**Next:** You: near the cottage, check that leaves vanish at the roof and that one leaf repeats its path; at the stream, square (45, 57) should stay still. Then B can implement from `emitters.md`.

## 2026-10-12 Study: doors, their scripts, minimum `.cs` language (Q44 answered)

**Changed:** New note `docs/behaviour/doors.md` (study session A). `objects.md`: rule 3 of "Ground heights −1 and −2" (Q44 order), pointer for the 6th `loop=` field. `QUESTIONS.md`: Q44 answered. Scratch notes in `re/` (not committed).
**Why:** M3 starts with Daventry's doors; B needs the door's behaviour, the script events and enough of the script language to run the door scripts.
**Tested how:** Traced in `Mask.exe` with Ghidra; door data read from `daventry/kq/*.kq`, `*.cs`, the door `.kqs` files, `BOUNDS.KQ`, `Connor.kq`, the manual. Nothing run in the game.
**Result:** Door state machine (closed / opening / opened / closing; `KQDoorEvent` 1 open, 2 close, anything else toggle; moving doors ignore it; `Open`/`Close` script calls when the animation ends). Click rules (play cursor, collision bit 0x20, reach 2 × `attackDist` = 4,700 to the door's box, `ConnorAction do do` / item / `TooFar`). Hits (`stats` hit points, default 10,000; `Die`, `ProjImmune`). Language: `if`/`else`/`endif` (`else if` = else + nested if, needs its own `endif`), `test` (0 = true, numbers vs text, wrong word count = false), `$Console::Return`, `sendEvent` timing, `collisionType ±bits`. Q44: the `keepOnGround` raise runs after the −1/−2 replacement (B's order is right).
**Still broken / not tested:** Not checked in the game. Not traced: where a door's animation turns the door object (so its box swings out of the doorway; inferred from the data), how "accepts this item" is decided, Connor's remark lines, which update a `sendEvent 0` lands in. Prediction to check: Daventry's plain doors open but never close again on a click.
**Next:** You: in the original, click Connor's open front door again (expect: stays open), and note roughly how far away a door still opens on a click. Then B can implement doors from `doors.md`.

**Checked afterwards (you, 2026-10-12):** (1) Connor's front door opens on a click and stays open on the next click, as predicted. (2) From far away Connor says it's too far (distance not measured). (3) With the door open the doorway is free and the open door leaf still blocks Connor. Recorded in `doors.md` (edge cases, rules 14 and 24). The reach number (4,700) is still from the trace only.

## 2026-10-12 DeadCity portal raised onto the floor (keepOnGround after −1/−2)

**Changed:** `crates/kq8-data/src/objects.rs`: after a `loadKQ` z of −1/−2 is replaced by its grid's height, the `keepOnGround` raise runs too (was skipped). Unit test. `crates/kq8-viewer/src/load.rs`: log line when that raise moves an object. Docs: QUESTIONS (new Q44, for A to confirm), provenance.
**Why:** User, in the original: the portal next to DeadCity's flame tower floats up on the raised floor like the tower; ours stood at 0. Its parts are placed with z −2 (surface, 0 there) and have the default `keepOnGround` (floor, 2,784 there). The notes give both rules but not their order.
**Tested how:** `cargo test --workspace --release`; every object's position before and after in all 12 worlds (viewer logs, `--frames 30`).
**Result:** Only `portsa7`, `portsb7`, `porttp7`, `portaa7` moved: 0 → 2,784 (their trap `portrap7` is at 2,909). The user also said the hall (cutscene place) and the tower are right as they are.
**Still broken / not tested:** Not checked by eye. The order of the two rules is from the user's check, not traced (Q44).
**Next:** You: look at the portal (`--world DeadCity --at 217513,37000,3.1416`, walk north; it is west of the tower at (209000, 49034)).

## 2026-10-12 Viewer: key U shows the floor grid

**Changed:** `crates/kq8-viewer/src/terrain_view.rs`: the ground is drawn in parts (surface where it equals the floor, surface over a different floor, floor where equal, floor where different); key U cycles normal → both (surface see-through where a different floor is under or over it; that floor tinted blue where lower, red where higher) → floor only (the whole floor, textured, same tints) → normal; on-screen label; log lines. `main.rs`: the system. Docs: README, DESIGN (2h).
**Why:** User's request, to see the hidden floor (e.g. DeadCity's hall area).
**Tested how:** `cargo build`, clippy; viewer start-up in Daventry, DeadCity, Barren (`--frames 30`), read `viewer.log`.
**Result:** Triangles where the floor differs: Daventry 383 (all lower: the river), DeadCity 3,320 (1,150 lower, 2,170 higher), Barren 0 (no floor grid). No new clippy warnings.
**Still broken / not tested:** Key presses not tested (no test option presses U); not checked by eye. In "both", see-through ground can look odd where objects stand on it.
**Next:** You: press U three times in DeadCity near the hall and in the Daventry river.

## 2026-10-12 Standing jump while wading

**Changed:** `crates/kq8-game/src/connor.rs`: `waterwalk` and `waterrun` added to the movements a jump can start from; both give the standing `jump`. Unit test. Docs: provenance, README.
**Why:** User's check in the original: walking or running in water, Jump stops him and does a standing jump. Ours refused it.
**Tested how:** `cargo test --workspace --release`; viewer in Daventry, `--at 141312,198656,0 --hold 1,0,1 --jump 60 --frames 2500`, read `connor.log`.
**Result:** 142 tests pass. Update 61: `jump from waterrun`; jump ends at update 81, lands at 83 (safe), `jumpend`, then `waterrun` again.
**Still broken / not tested:** Not checked by eye. Jumping during `walkwaterwalk`, `runwaterrun` and the ways out is refused, like other start/stop animations (not checked in the original). Walking-in-water jump only unit-tested.
**Next:** DeadCity hall height, when you have looked.

**Checked afterwards (you, 2026-10-12):** M2 2h-1 and 2h-2 look right; the Swamp shack is correctly placed now.

## 2026-10-12 M2 2h-2: wading (water test and animations)

**Changed:** `crates/kq8-data/src/envinfo.rs` (new: `envInfo.kq` ground flags, the note's column rule), `terrain.rs` (flags per square, `flags_at`). `crates/kq8-game/src/water.rs` (new: the water test), `mover.rs` (`water()` lookup: surface height and flags; `submerged`), `connor.rs` (`waterwalk`/`waterrun` chosen when submerged; `waterrun`/`runwaterrun` crash head-on), `player.rs` (the test after every move, its result used by the next update's choice; event line when it changes). `crates/kq8-viewer`: loads the flags and the 6 water movements; `connor.log` columns `submerged surface_z`. Docs: DESIGN (2h), provenance, README.
**Why:** `water.md` (Q43): Connor wades where the ground is a liquid and the surface is at least 600 above his feet.
**Tested how:** `cargo test --workspace --release`; viewer in Daventry, from a dry square into the river and out: `--at 141312,198656,0 --hold 1,0,0 --frames 5000` (walking) and `--hold 1,0,1 --frames 2500` (running); read `connor.log`.
**Result:** 142 tests pass (52 + 90). Viewer log: Daventry 201 water squares (4 also swamp), as A counted. Walking: submerged at update 67 (surface 6,630, feet 6,017); update 68 asks for `waterwalk`, `walkwaterwalk` from phase 0.2, `waterwalk` from update 77; feet down to 5,610 (1,020 under the surface); submerged off at update 192, `waterwalkwalk` at 215, `walk` at 220, out on the far bank. Running: `runwaterrun` the update after (36), `waterrun`, then `waterrunrun` and `run` on the way out.
**Still broken / not tested:** Not checked by eye. No damage from poison, lava or swamp; no sinking into `ontop` hazards (not in this step). No jumping while wading (the notes don't say; our choice). The water's look is unchanged.
**Next:** You: walk into the Daventry river and compare with the original (water at his chest, the wading walk).

## 2026-10-12 M2 2h-1: the floor grid (ground under water) and placement

**Changed:** `crates/kq8-data/src/terrain.rs` (optional floor grid, `floor_at` / `ground_at`, surface fallback), `objects.rs` (`keepOnGround` default floor / 1 surface / 0 off; `loadKQ` z −1 floor, −2 surface). `crates/kq8-game/src/mover.rs`: the terrain lookup (Connor and the camera) uses the floor. `crates/kq8-viewer`: loads `floorBMPName`, places objects by the new rules, `--at` puts Connor on the floor; log lines name the grid. Docs: DESIGN (plan 2h), provenance.
**Why:** `water.md` (Q43): `terrh2.dib` is the ground under water; Connor and default objects stand on it. Fixes the Swamp shack (objects half through its floor).
**Tested how:** `cargo test --workspace --release`; viewer runs: Swamp `--at 83500,43000,0 --lift 3000` and `--at 74000,47000,0 --lift 3000` (Connor dropped into the shack); every object's position before and after in Daventry, Swamp, IceWorld, DeadCity (old build vs new, viewer logs).
**Result:** `shack2` at 5,355 (was 6,120). Connor lands at 6,468 (east) and 7,332 (west), 765 lower, as predicted; the stove, chests, table and barricades now sit 18–70 above the floors. Daventry and IceWorld: no object moved. Swamp: also `fientowr` 6,120 → 5,994, `proftree` 6,885 → 6,871. DeadCity: `DeadHal2`, `platfrml`, the four hall doors, `porttow3.01` and its flame moved **up 2,784**: they stand in two large blocks where the floor image is exactly 2,784 above the drawn ground (the note's open question 3); `Pool1`–`Pool5` (invisible click targets, `locZ=0`) moved down from the surface to the floor.
**Still broken / not tested:** Not checked by eye. The DeadCity hall area is not checked in the original (it may float 2,784 above the drawn ground in our viewer; whether that is what the game shows is unknown). A `keepOnGround` value other than 0/1 counts as the surface (our reading; none in the data).
**Next:** 2h-2 (wading). You: the Swamp shack by eye; DeadCity's hall area if you know it.

## 2026-10-12 Study session: water and the second height map (Q43)

**Changed:** New `docs/behaviour/water.md`. Corrected/extended `docs/behaviour/objects.md` (Q17 rule 2), `buildings.md` (rule 6), `terrain.md` (`floorBMPName`, open questions 1–2), `connor-physics.md` (terrain passes use the floor), `connor-movement.md` (pointer). `QUESTIONS.md`: Q43 answered. No code.
**Why:** B's Q43: the Swamp shack sits 765 too high, and Connor walks on top of water instead of wading.
**Tested how:** Traced in `Mask.exe` (terrain loading, the height lookup, the mover's terrain test and ground raise, the per-move water test, the movement choice, the `envInfo.kq` reader). Then a throwaway script (scratchpad, not in the repo) compared `terrh.dib` / `terrh2.dib` with the liquid flags from `envInfo.kq` in all 12 worlds.
**Result:** `terrh2.dib` is a second "floor" grid, the ground under the water. Connor stands on it (his `keepOnGround` default −1 selects the floor; the mover's terrain hits always use it). After each move, if the square's material is liquid and the drawn surface is ≥ 600 above his feet, `walk`/`run` become `waterwalk`/`waterrun`. `loadKQ` z −1 = floor, −2 = surface (my earlier Q17 rule 2 said they were the same: wrong). Buildings without a `keepOnGround` line stand on the floor, so `shack2` is at 5,355, as B guessed. Data: every liquid square has the surface above the floor (Daventry mostly 1,020 deep), except 180 Swamp squares with no depth.
**Still broken / not tested:** Nothing checked in the running original. Not traced: whether the floor grid is drawn (probably not), what restores gravity after an `ontop` hazard (lava sinking), damage. DeadCity has 1,086 squares where the floor is above the drawn ground; not looked at.
**Next:** B: stand Connor and default-`keepOnGround` objects on `terrh2`, `locZ` −1/−2 as in `water.md`, then the water test and the wading animations. Check the shack and Daventry's river by eye.

## 2026-10-12 A held turn key keeps turning him at a wall (keyboard auto-repeat)

**Changed:** `crates/kq8-viewer/src/connor_view.rs`: the movement key pressed last, while still held, counts as pressed again before every update (only that key, as Windows does; letting it go ends it), and the log says when that clears a wall block. New test option `--turn N[,1|-1]` (press and hold left or right before update N + 1; `main.rs`). `crates/kq8-game/src/player.rs`: new unit test. Docs: provenance, Q38 in QUESTIONS, README.
**Why:** User: against an object, head-on, a turn key turned him a little and then he stood; in the original he keeps turning while the key is held. Cause: a key press clears the wall block once; walking again starts `stopwalk`, whose step goes 33° to his left. Turning left, that glances off (so left already worked); turning right brings it back under the 20° head-on limit after 3 updates, he stops and forward is blocked again. With the held key repeating, each update clears the block again.
**Tested how:** `cargo test --workspace --release`; viewer in Daventry at `signpst.05`: `--at 173802,269850,-1.5708 --hold 1,0,0 --turn 40,-1` before and after the change, `--turn 40` (left), and backing in `--at 171300,269800,-1.5708 --hold -1,0,0 --turn 40,-1`; read `connor.log`.
**Result:** 134 tests pass (47 + 87). Before: heading −1.571 → −1.921 (0.35 rad), head-on at update 43, then `stop`, blocked, unchanged to the end. After: he turns every update; 3 head-on bumps (updates 43, 45, 47), glances off at update 49 and walks round the post. Backing in: turns every update, glances off at update 44. Left turn: same as before (glances at once).
**Still broken / not tested:** Not checked by eye. Letting go of the turn key is not tested in the viewer (`--turn` can't let go); by the rules he then stands if the last bump was head-on, as you described. No 0.5 s delay before the repeat starts (Windows has one; whether the game sees it is not known, asked in Q38). While bumping, `stop` and `stopwalk` alternate for a few updates (may flicker; not seen).
**Next:** You: check by eye at an object, both turn keys, forward and back held.

## 2026-10-12 Swamp shack objects sink into the floor: question Q43 for A

**Changed:** `docs/behaviour/QUESTIONS.md` (new Q43). No code.
**Why:** The user saw the stove, small table, three chests and two barricades half through the floor of the Swamp shack (`shack2`, around (84654, 44049)).
**Tested how:** Read `viewer.log` (Swamp run) for the shack's and the objects' positions; dropped Connor into the shack 3 times (`kq8-viewer --terrain --world Swamp --at X,Y,0 --lift 3000 --frames 600`) and read his landing height in `connor.log`; compared `terrh.dib` and `terrh2.dib` in every world with a throwaway script (scratchpad, not in the repo).
**Result:** The house has `locZ=-1` and is lifted to the terrain, 6,120. Its floors: 7,208 (east part), 8,072 (west part). The objects stand 715–765 below them. `terrh2.dib` (the "floor" image, unused so far) is 5,355 under the house; with the house there, every object would sit 18–70 above its floor. Likely `terrh2` is the bottom under the water, but how the game uses it is not known.
**Still broken / not tested:** Nothing fixed. The guess is not traced; whether Connor should walk on the water's bottom is also open.
**Seen by the user afterwards:** in our viewer Connor walks on top of the water in every world with water; in the original he goes down into it. Added to Q43; it fits `terrh2` being the ground under the water. README's "doesn't work" list and STATUS updated.
**Next:** A: Q43. Then B: place by the answer.

## 2026-10-12 Palette entry 0 drawn black (Q22)

**Changed:** `crates/kq8-data/src/palette.rs` (new `as_shown`: colours as stored, entry 0 black; `shown_colors` uses it; tests updated), `crates/kq8-viewer/src/load.rs` (textures with their own colour table use `as_shown` too; the log's own-vs-world comparison compares drawn colours), `crates/kq8-cli/src/bitmap_png.rs` (same rule in `kq8 bitmap-png`). Docs: `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (new Q42), `STATUS.md`.
**Why:** Session A measured entry 0 = black in the running original (Q22). The user chose to apply it to textures' own colour tables too, for one consistent rule.
**Tested how:** `cargo test --workspace --release`; `kq8 bitmap-png` on both `Teleport.BMP`; `kq8-viewer --terrain --world Swamp --frames 30` and `--world Daventry --frames 30`, then read `viewer.log`. Before the change, the user compared exported images of both `Teleport.BMP` (entry 0 blue vs black).
**Result:** 133 tests pass (47 + 86). Exported entry 0 is now (0, 0, 0) for Swamp and Daventry `Teleport.BMP` (was (0, 0, 255)). Viewer log: both `Teleport.BMP` lines now say `used_colours_differing_from_world_palette=0` (was 1). Found on the way: `swamp.ppl` already stores entry 0 black, so `WIRLPOOL.BMP` was black all along and doesn't change (the 2026-10-11 entry saying it turned blue was wrong); `daventry.ppl`, `barren.ppl`, `gnome.ppl` store blue. User: the changed pixels are outside the round teleport pad and never seen in the game.
**Still broken / not tested:** Not checked by eye in the viewer (nothing visible should change). Whether the game also makes entry 0 black in a texture's own table is not measured (Q42, stand-in). Unrelated, seen in the log and left alone: 104 Daventry textures with their own colour table use one colour that differs from `daventry.ppl` (not entry 0; it was like that before this change).
**Next:** A: Q42 if convenient.

## 2026-10-12 Study session: Q22 answered by measurement (palette entry 0 is black)

**Changed:** `docs/behaviour/models-and-textures.md` ("System colours": entry 0 is black on any screen), `docs/behaviour/QUESTIONS.md` (Q22 answered). No code.
**Why:** Q3 ("palette as stored", entry 0 blue) and Q5 ("entry 0 set to black") disagreed.
**Tested how:** Traced the code that sends palettes to the 3D card (Glide): it passes entry 0 on as it is in memory (only brightness applied; entries 1–9 and 246–254 that are pure black become magenta). Then read the running original's memory (read only, `re/dbg/mem_pal.py`) with the game in Daventry on a normal desktop: found the in-memory copy of `daventry.ppl`.
**Result:** It equals the file in all 256 entries except entry 0: (0, 0, 0) instead of (0, 0, 255). So entry 0 is black on a normal screen; Q5 was right, the entry-0 part of Q3 was wrong.
**Still broken / not tested:** Where entry 0 is set to black was not traced. Only Daventry's palette was read. The magenta replacement of black system entries (1–9, 246–254) when sending to the card is noted in my scratch notes only; no texture uses those entries as far as known.
**Next:** B: draw entry 0 as black (`WIRLPOOL.BMP`, `Teleport.BMP`).

## 2026-10-12 Study session: Q39 settled by measurement (box names ignore case)

**Changed:** `docs/behaviour/connor-physics.md` (new "Case doesn't matter in practice"; the old trace marked wrong; "How to check" updated), `docs/behaviour/QUESTIONS.md` (Q39 answered). No code.
**Why:** My code reading said `St_smom` has no box; the user saw Connor stop at her. Two reading attempts failed, so I measured instead.
**Tested how:** User at the original game in Daventry (stone mother's cottage); my memory reader (`re/dbg/mem_fall.py`, read only) logged Connor's position, state and the surface he touched while the user walked into her twice, from two sides.
**Result:**

- Head-on stop 575.2 from her centre along her box's axis = 350 (table half-width of `st_smom.kqs`) + 200 (ball) + 25 (pull-back). The hit was logged against `St_smom` with a face direction matching her heading.
- Second approach (while turning): 557 on the same axis, consistent with the same box.
- So the lower-case table entry is used for `St_smom.KQS`; B's stand-in is right.

**Still broken / not tested:** Where the game ignores the case is not found. Object names with a case difference: not tested. Wagons not measured (the user's check says they collide).
**Next:** Q22 (palette entry 0), when the user wants.

## 2026-10-12 Study session: Q39 and Q40 answered

**Changed:** `docs/behaviour/connor-physics.md` (which names are looked up in the box table, and their case; a game check), `docs/behaviour/connor-movement.md` (rule 3: order within an update; rule 4: a request replaces a waiting transition and restarts a playing movement; "Do not reproduce": that restart; `stoprun` total corrected to 526.7), `docs/behaviour/QUESTIONS.md` (Q39, Q40 answered). No code.
**Why:** B's open questions from M2 steps 2d-5 and 2g.
**Tested how:** Traced in Ghidra. The `stoprun` sum re-added by hand (B was right: 526.7). Not tested in the game.
**Result:**

- Q39: **wrong, corrected the same day.** I traced no lower-casing anywhere, which would mean the stone villagers and wagons have no collision. The user checked in the game: Connor does not walk through them. Q39 is open again; B's stand-in (lower-cased shape names) stays. Also checked: the archive's copy of `st_smom.kq` says `St_smom.KQS` too, so it isn't a different file.
- Q40 (a): −1 transitions are one update late too (state step, then animation step; the transition advances from the next update). B's reading is right.
- Q40 (b): asking for the movement that is playing restarts it from phase 0 in the original. **Confirmed in the game by the user**; the user decided to keep B's version (no restart), noted in "Do not reproduce".
- Q40 (c): corrected.

**Still broken / not tested:** Q39 not settled: two code-reading attempts failed; next is a measurement in the running game. Q22 (palette entry 0) still open.
**Next:** Q39 by measurement (where Connor stops against `St_smom`, and the box the running game holds for it).

## 2026-10-12 M2 2e-3: hitting something during a jump

**Changed:** `kq8-game` `mover.rs`: head-on also in the Jump state, where any surface with normal z < 0.8 counts (also on up-pass hits, so ceilings); the report names the pass of the first steep hit during a jump; `end_jump_on_hit` (gravity on, steps 325). `player.rs`: a steep hit ends the jump: `backJump` falls at once in `backJumpHold`, a down-pass hit or any `runJump` hit crashes, an up or across hit during `jump` stands him with forward blocked; 4 new tests. `connor.rs`: `stop_after_jump`. Docs: DESIGN, provenance, README.
**Why:** Last part of jumping (`connor-physics.md` "Jumping" rule 5, "Walls"; your back flip check).
**Tested how:** `cargo test --workspace` (133 tests; 132 after the fix below). Viewer in Daventry against `tree.06`: `--at 277744,199295,0 --jump 3`; `--at 277744,201500,0 --hold 1,0,1 --jump 13`; `--at 277744,199295,3.14 --hold -1,0,0 --jump 2`.
**Result:** Standing jump: hits the tree (across pass, update 16), stands with forward blocked, in the air, Fall 5 updates later, `jumpend`. Running jump: hits it (update 21), `crash`, then falls and lands. Back flip: hits it (update 13), slides, `backJumpHold` at once, lands at update 17, `backJumpEnd`, then steps back into the tree again (normal head-on stop). Fixed on the way: the back flip's hit was also logged as "head-on into a wall … stop" although no stop happened.
**Still broken / not tested:** Not checked by eye. The back flip's Fall start values after a hit are our choice. A standing jump stopped high against a wall stands in mid-air for the 5 updates before Fall starts, as the traced rules give; not checked in the original.
**Next:** You check 2e-2 and 2e-3 by eye.
**Checked by eye by you (2026-10-12):** 2e-2 and 2e-3, after the fix below, all look correct.
**Fix after your check (same day):** in the original the back flip doesn't crash or stumble at a wall or steep slope: it keeps going, then the falling animation until the ground. Ours cut the flip off at the hit and fell at once (my misreading of your first description). Now at the hit he slides, gravity comes on (rule 5), `backJump` plays on to its end, then Fall as after any jump. Viewer (`--at 277744,199295,3.14 --hold -1,0,0 --jump 2`): hit at update 13, `backJump` to update 20, `backJumpHold`, `backJumpEnd`. Tests: 132 pass (old back flip test replaced).

## 2026-10-12 M2 2e-2: running and backward jumps

**Changed:** `kq8-viewer` `load.rs` loads `runJump`, `runJumpHold`, `runJumpDie`, `jumprun`, `jumpstop`, `backJump`, `backJumpHold`, `backJumpEnd`, `backJumpDie`. `kq8-game` `connor.rs`: `land()` (a running jump has no landing animation: the keys' control at once in `runJumpHold`, and whatever the keys choose is asked for from there); at that landing a turn key alone counts as standing. `player.rs` calls `land()`; 4 new tests. Docs: DESIGN, provenance, Q41 (d).
**Why:** Second part of jumping (your go-ahead).
**Tested how:** `cargo test --workspace` (129 tests). Viewer runs in Daventry: `--at 277744,199295,3.14 --hold 1,0,1 --jump 20` and `--hold 1,0,1,30 --jump 20`; `--at 277744,199295,0 --hold -1,0,0 --jump 10`.
**Result:** Running jump: `runJump` 20 updates, top +1,858.8 (note 1,859); lands after one Fall update; run held: `jumprun` 3 updates, then `run` (the note measured 3); forward let go: `jumpstop` 15 updates (1.0 s, measured 1.0 s), then `stop`. Back flip: 18 updates, 2,565.7 back, top +2,156.1, end +788.1 (note 2,566 / 2,156 / 788), `backJumpHold`, `backJumpEnd` 11 updates, then stepping back again. First try of the back flip jumped from `stop`: stepping back from that spot hits `tree.06`, which blocks him, so a standing jump was right; tried facing the other way.
**Still broken / not tested:** Not checked by eye. How the original decides what follows a running jump's landing is not traced; ours follows the 2g rules plus your check (Q41 d). Hits during jumps are still glancing only (2e-3, next).
**Next:** 2e-3, hitting something during a jump.

## 2026-10-12 M2 2e-1: the standing jump

**Changed:** `kq8-game`: new `player.rs` (one update of Connor: walker and mover together; the falls, landings and head-on code moved here from the viewer unchanged; jumps: when one is allowed, start, end into Fall); `mover.rs` (state Jump, `start_jump` / `end_jump`: gravity off and steps 0 while jumping, then gravity on, steps 325, Fall with the death limit 1,280 lower and the timer at 0.36 after 0.30; lifting mid-jump ends it; clearer fall-end message); `connor.rs` (jump kinds with their hold/landing/death animations, mode Jump, which movements allow a jump). `kq8-viewer`: Jump key from `Connor.sae`, loads `jump`, `--jump N` test option, new `connor.log` column `above_jump_start`, jump event lines. Plan in DESIGN (M2 2e), provenance, README, Q41.
**Why:** Your go-ahead for jumping, with your checks in the original (no jump during start/stop animations; no turning in a jump).
**Tested how:** `cargo test --workspace` (125 tests, 3 new in `player.rs`). Viewer in Daventry at `--at 277744,199295,3.14`: `--jump 5`, `--hold 1,0,0 --jump 20 --frames 2000`, `--hold 1,0,1 --jump 4` and `--jump 20`, read from `connor.log`.
**Result:** Standing jump: `jump` for 20 updates (6–25), still for the first 9, highest +1,697.0, at the end +872.8 and 1,423.4 forward (the note: 1,697, 873, 1,423); then Fall, first drop 396. From walking: the same jump (forward movement stops, as the note says walking uses the standing jump); Fall 1 update (the ground there was 846 below the jump's end: drops 396 + 468), `jumpend` 10 updates, then `stopwalk`, `walk` with forward held. Refused during `stoprun` ("stoprun is playing") and from running ("runJump is not loaded yet", 2e-2). Unit tests: the arc; death only past 5,120 below the jump's end; refusals (falling, during `stopwalk`, 300 above the ground, facing up a 45° slope).
**Still broken / not tested:** Not checked by eye. The note's "landing about 5 updates after the jump" was written before gravity was corrected to 4 times stronger; we land sooner, as the numbers say. Running and backward jumps (2e-2). Hits during a jump are only glancing for now: no crash or stop on a steep hit (2e-3). Q41: jumping while turning on the spot or while a walk plays on to its stop point (we allow both), `backJump` after a steep hit, the 250 measurement.
**Next:** You check by eye; then 2e-2.
**Checked afterwards (you, 2026-10-12):** the standing jump looks correct. In the original, jumping while turning on the spot stops the turn and it starts again after the landing animation (ours does the same). A back flip into a wall or steep hill slides down in its falling animation until the ground (answer to Q41 b, for 2e-3).

## 2026-10-12 M2 2g-2: stop transitions (walk and run → stop)

**Changed:** `kq8-viewer` `load.rs` loads the six stop animations (`walkstopleft`, `stopwalkstop`, `walkstopright`, `runstopright`, `runstopleft`, `stoprunstop`), so walk's and run's stop transitions are used; `kq8-game` `connor.rs`: "moving" (the Move state, which allows head-on stops and crashes) now follows the keys' choice instead of the movement playing; `main.rs` / `connor_view.rs`: `--hold F,T,R,N` lets go after N updates. Docs: DESIGN (2g-2), provenance, README.
**Why:** Second half of your task. The state change: the note measured that he is already in Wait while the walk or run plays on to its stop point, and head-on only acts in Move.
**Tested how:** `cargo test --workspace` (122 tests, 6 new). Viewer in Daventry facing open ground: `--at 277744,199295,3.14` with `--hold 1,0,0,4`, `1,0,1,4`, `1,0,0,20`, `1,0,1,20`, read from `connor.log`.
**Result:** Let go during `stopwalk`: it ends at update 8, walk from 0.85 is cut at 0.9 in update 9, `stopwalkstop` updates 10–17, then `stop`. Let go during `stoprun`: it ends at update 11, update 12 is `run` 0.8667 → 0.8667 with no step, `stoprunstop` updates 13–21 (35.2 to his right in all, as the note measured), then `stop`. Let go walking at 0.48: walk on to 0.9 (8 updates), then `stopwalkstop`. Let go running at 0.51: run to 0.6, then `runstopleft`. Unit tests: all six stops by release phase; forward pressed again while a stop waits → walks on (Q40 b, our reading); pressed during a stop animation → it finishes, then `stop`, `stopwalk`, `walk`; no head-on while stopping. 2g-1 checked by eye by you, including the original's quirk (turn key held, then forward: starts at once).
**Still broken / not tested:** Checked by eye by you afterwards (all looked right). Q40 (a) and (b) still open (our readings). Running into a wall while a stop plays now slides or stops him without `crash` (follows the note's "Wait" measurement, not checked in the original). `runstopright` and `walkstopright` only seen in unit tests (the measured original never showed them either).
**Next:** You check by eye. Then whatever you choose next in M2 (2e, jumping, is the remaining step).

## 2026-10-12 M2 2g-1: start transitions (stop → walk via stopwalk, stop → run via stoprun)

**Changed:** `kq8-data` `objects.rs` (reads `transitionN` lines); `kq8-game` `connor.rs` (the walker follows rules 1–6 of "Starting and stopping": wanted movement, transition chosen by the next from point, played to its end, then its `to` movement at its `to` phase; one-shot animations step frame by frame on both axes; `stoprun` crashes and `stopwalk` stops at a wall like `run` and `walk`); `kq8-viewer` `load.rs` (loads `stopwalk`, `stoprun`, every movement's transitions and "can be cut short" flag, logs frame steps), `connor_view.rs` (new `connor.log` columns `played played_from played_to wanted_movement step_left step_forward`, event lines for requests and transitions). Plan in `docs/DESIGN.md` (M2 2g), provenance, README, Q40 in `QUESTIONS.md`.
**Why:** Your task: Connor's start animations, from `connor-movement.md` (Q25 answered). The stops come next as 2g-2.
**Tested how:** `cargo test --workspace` (116 tests, 10 new for the walker, 1 for the reader). Viewer runs in Daventry: `--terrain --world Daventry --hold 1,0,0 --frames 400` and `--hold 1,0,1`, read from `connor.log`.
**Result:** Walk: update 1 `stop` (no step), updates 2–8 `stopwalk`, each 7.32 left / 11.22 forward, 51.2 / 78.5 in all (x grew 51.2 = his left at heading 0), then `walk` from phase 0.85 (update 9: 0.8500 → 0.9026). Run: updates 2–11 `stoprun` with the note's uneven steps (0.18/0.19 … 1.77/43.80), 43.9 left / **526.7** forward, then `run` from 0.8667. After the crash into `tree.06` with run held: one update of `stop`, then `stoprun` again, which crashed into the tree again (head-on during `stoprun` = crash, as the note says). Unit tests: let go during `stopwalk` → plays to its end, then `stop`; run switched on during `stopwalk` → `run` from 0 when it ends; turning during a start; the choice rule (next ahead, wrap, −1, ties, one-shots never wrap).
**Mistake found in the note (not in our code):** its `stoprun` total says 536.7 forward; its own per-frame steps and `C_stprn.kqs` add up to 526.7. Asked as Q40 (c).
**Still broken / not tested:** Not checked by eye. Not checked against the original. Our reading of when a transition starts (one update of `stop` with no step before `stopwalk`) is asked as Q40 (a). `jumpend` now takes its whole step (78 forward, 291 down, all in its frame 1) in the first landing update instead of spreading it over 10: same rule, not checked by eye. Stops are still immediate (2g-2). After `jumpend`, walking again now goes through `stopwalk` (our choice, not traced). Letting go during a start and switching run on during one are only unit-tested (`--hold` can't change keys).
**Next:** You check by eye; then 2g-2, the stop animations.

## 2026-10-12 Study session: start and stop transitions traced (Q25)

**Changed:** `docs/behaviour/connor-movement.md` ("Starting and stopping" rewritten with the traced rules, the `loop=` flags and the start animations' per-frame steps; open question 2 updated), `docs/behaviour/QUESTIONS.md` (Q25 answered). No code.
**Why:** The user wants the start animations (standing → walking or running) implemented; B needs the rules first.
**Tested how:** Traced in Ghidra (how transition lines are read, chosen, waited for, played and ended; the `loop=` fields). Steps read from `C_stpwk.kqs` and `C_stprn.kqs`. Compared with my earlier measurements (start lengths, "one update of `run`, then `stoprunstop`", the drift to his left). Not tested in the game.
**Result:**

- A transition is chosen by the next "from" point ahead of the current phase; the current animation plays on until it; −1 = at once.
- Transition animations always play to their end (`0 0` on their `loop=` line); a key change meanwhile is handled when they end.
- Starting: `stopwalk` 0.467 s then `walk` at 0.85; `stoprun` 0.667 s then `run` at 0.867. Both step to his left (`stopwalk` 33° left throughout; `stoprun` almost on the spot for its first third, then a surge forward).
- The measured lengths and the "one update of `run`, then `stoprunstop`" fit the rules.

**Still broken / not tested:** The measured start distances (about 80 and 385–490) are under the data's totals (93.8, 536.7); probably my log windows, not confirmed. Not traced: how the leftover time of the update in which a transition starts is used.
**Next:** B implements the transitions (start first). Optional check in the original afterwards with the memory reader.

## 2026-10-12 Study session: Q38 traced (held walk key against a wall)

**Changed:** `docs/behaviour/connor-physics.md` "Walk held against a wall" (committed as `305647d`); this entry. No code.
**Why:** The measured "stays in `Stop` while forward is held" had no traced rule, and B asked (Q38) whether a turn key should turn him on the spot.
**Tested how:** Traced in Ghidra (the hit handler, Connor's key handler, the Wait state's update). Not tested in the game.
**Result:**

- A walking head-on stop sets a forward block (back block if he was walking backwards). Wait goes back to Move only when the keys ask for a movement and both blocks are off.
- Clearing: forward released → forward block; back released → back block; forward pressed → back block; back pressed → forward block; a turn key pressed → both. So a turn press makes him walk again while turning, not turn on the spot.
- Nothing else found clears them (not falling, landing or a new world), except one untraced Connor flag during hits.

**Still broken / not tested:** Whether a held key repeats its "pressed" event (keyboard auto-repeat) is not known. Check in the original: walk into a wall, keep forward held, press and hold a turn key; he should walk again (or stutter walk/stop if auto-repeat clears the block repeatedly).
**Next:** The remaining study topics: Connor against monsters, which running stop animation plays (Q25), Q27, Q22.

## 2026-10-12 Docs brought up to date after M2 step 2d

**Changed:** `docs/DESIGN.md` (2d marked done with its stand-ins; the inside-out box and "starting inside a box" items resolved; 2e no longer lists the crash), `docs/behaviour/QUESTIONS.md` (Q38 marked answered, auto-repeat still open), `STATUS.md` (not tracked: latest state, code layout, test counts, what works, what's next, open questions, fresh-session prompts). No code.
**Why:** The user asked for all relevant documents to be up to date after 2d.
**Tested how:** Read through; `cargo test --workspace` unchanged (kq8-data 46, kq8-game 59).
**Result:** Docs match the code at `ef205e5`.
**Still broken / not tested:** `docs/behaviour/connor-physics.md` has session A's uncommitted Q38 trace; left for A.
**Next:** The user chooses 2e (jumping) or the camera's obstacle check.

## 2026-10-12 M2 step 2d-5: objects are solid

**Changed:**

- `crates/kq8-data/src/bounds.rs`: `BoundingBoxes::lookup`, the box the game uses for a name (Q34: keys below `nItems`, case matters, inside-out boxes fixed, `rat` = item 8, `razetta.kqs` = item 775).
- `crates/kq8-game/src/collide.rs`: Connor's type 4 and mask 0x281B, the default type 0x41 and mask 0, `pair_collides`; `BoxCollider::overlaps` (rough check with the box enlarged by r and turned into the world).
- `crates/kq8-game/src/mover.rs`: `Scene` has an object list next to the buildings (`Scene::new`); each sweep tests both and keeps the contact nearest the old position. A ball starting inside an object's box: no hit, noted in the report's events.
- Viewer:
  - `load.rs`: keeps each placed object's shape name, `colliderType` and `colliderMask`; after the scripts ran, builds the object colliders and logs counts per world.
  - `terrain_view.rs`: new resource `ObjectColliders`.
  - `connor_view.rs`: passes the objects to the mover.
- Records: `docs/DESIGN.md` (2d-5 detailed plan, done), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q39), `README.md`.

**Why:** M2 step 2d-5, from `connor-physics.md` "Collision shape" (box table, object test, which pairs collide, hidden objects) and "Exact hit handling".
**Tested how:**

- `cargo test --workspace`: kq8-data 46, kq8-game 59. New tests:
  - the lookup (case, inside-out, duplicates, past `nItems`);
  - the pair rule;
  - the rough check never skips a turned box's corner;
  - in the mover: head-on stop 225 from an object's face, glancing slide past it, standing on a crate 25 above it, starting inside a box (no hit, noted).
- All 12 worlds loaded, collision summary read from each `viewer.log`.
- Daventry `signpst.05`: `--at 173802,269850,-1.5708 --hold 1,0,0` (from the east) and `--at 172302,271350,0 --hold 1,0,0` (from the north).
- DeadCity `crate.01`: `--at 171392,82080,-1.5708 --hold 1,0,0`.
- Regressions: the `Tomb` walk and run, Connor's house, running from the start spot.

**Result:**

- Daventry: 295 object colliders (KQObject 185, KQAnimObject 88, KQDoor 17, KQChest 3, KQPortal 2). Left out: 24 without a box (e.g. the `Davtorch` torches, the gates `davgate1/2`, jugs, `tombhero`), 2 by type and mask (`keepwall`, `pondclck`, type 0x20). The signpost's box is −250…−10, 0…250, 0…2,420, as planned.
- Other worlds: DeadCity 864, Swamp 265, Gnome 178, Barren 166, IceWorld 81, Temple1 50, Temple3 43, Temple2 25, CastleD 18, Temple4 4, SnowExit 3. Many more "type 0 and mask 0" in Gnome and the temples (153–261), which never collide.
- Signpost: stops head-on with the contact 200 from the turned box's face, 225 after the pull-back (east: 223.1, the walk-step rule; north: 225), `blocked fwd`.
- Crate (turned 25.8°): glances off and walks on past it (more than the 20° head-on limit).
- Regressions: the `Tomb` numbers and the house floor (13,980) unchanged. **New:** running straight ahead from Connor's start spot now hits `tree.06` (about 550 ahead) and crashes; before, he ran through it.

**Failed attempts, corrected:**

- Two of my new mover tests failed at first, both test setups: the head-on test looked at the last hit, which is the landing on the ground; the glancing test's path passed beside the post without touching it.
- Names compared exactly left 25 Daventry objects (the stone villagers, wagons, landslide…) without a box, because their files write the shape with capitals and the table has it in lower case. Now lower-cased first, as a stand-in (Q39).
- `colliderType=97  #World, Click, Attack` (Barren, IceWorld) was first read as "not a number" (default used); now 97.

**Still broken / not tested:**

- Checked by eye by the user afterwards (2026-10-12): looks good.
- Q39 (lower-casing shape names) is our guess.
- Hidden objects collide, as traced, so some invisible things are solid: Daventry's hidden `wizchest1`, 50 in Barren, 20 in Gnome, 1 in SnowExit. Not checked in the original.
- Objects whose model we can't load, `.anm` characters and `KQMonster`s get no box.
- Doors stay solid at their loading box (no opening yet).
- Starting inside a box is our choice (no hit).

**Next:** 2e (jumping) or the camera's obstacle check, as the user prefers.

## 2026-10-12 M2 step 2d-3c: the wall block as session A traced it

**Changed:**

- `crates/kq8-game/src/connor.rs`: the single "blocked direction" of 2d-3b became the two traced blocks (`blocked_forward`, `blocked_back`) and `Walker::key_event` (key pressed or released). While either block is on he stands in `stop`: no walking, running, stepping back or turning on the spot. Mode changes no longer clear the blocks.
- `crates/kq8-viewer/src/connor_view.rs`: forward, back, left and right presses and releases go to the walker; `blocked` column now `none`/`fwd`/`back`/`both`; an event line for each block a key clears.
- Records: `docs/DESIGN.md` (2d-3c), `docs/provenance/README.md`, `README.md`.

**Why:** Session A traced why a held walk key doesn't restart walking (`connor-physics.md` "Walk held against a wall", Q38). It differs from 2d-3b: a turn key clears the blocks (he walks again, turning) instead of turning him on the spot; releasing forward clears its block; pressing the opposite key clears the other block; blocks also stop running; falling and landing don't clear them.
**Tested how:**

- `cargo test --workspace`: kq8-data 44, kq8-game 53. New tests: the block holds for 45 updates with the key held, and pressing the same key again doesn't clear it; releasing it does; the whole clearing table; no running or turning on the spot while blocked; a mode change leaves the block on; forward held plus a turn key pressed → walks again, turning.
- Viewer regressions at the `Tomb`: walk head-on (`--at 237907.4,187510.2,0.027 --hold 1,0,0`) and run head-on (`--at 237968.9,187493.2,-0.2054 --hold 1,0,1`).

**Result:** Walk: stops at (237909.4, 187435.1), `stop` and `blocked fwd` to the end. Run: unchanged (3 crashes in 300 frames, the first ending at (238049.8, 187881.3), no block).
**Still broken / not tested:**

- The key events are tested only in unit tests (`--hold` presses no keys); the user's check by eye is still to come.
- Keyboard auto-repeat (A's open point): our keys report a held key as pressed once.
- The Connor flag that clears both blocks on hits is not traced, so not done.

**Next:** 2d-5 (objects).

## 2026-10-12 M2 step 2d-4: crash when running head-on

**Changed:**

- `crates/kq8-game/src/connor.rs`:
  - New walker mode `Crash`: `crash` plays once with the keys ignored, then the keys take over (like `Land`).
  - `stop_at_wall` is now `head_on_at_wall`: running switches to `Crash`; walking and stepping back stop as before.
  - **Fix found on the way:** a play-once animation now lasts its stored loop length and moves its whole step sum. Before, it lasted (frames − 1) / frames of the length and moved that share of the steps (`crash` would have pushed him 386 in about 6 updates instead of 441.5 in 7). The data shows the stored length already runs from the first frame to the last; the measured crash agrees.
- `crates/kq8-game/src/mover.rs`: a new test only.
- `crates/kq8-viewer/src/load.rs`: `crash` added to Connor's movements. `connor_view.rs`: event lines for the crash.
- Records: `docs/DESIGN.md` (2d-4 plan and done, the one-shot correction), `docs/provenance/README.md`, `README.md`.

**Why:** M2 step 2d-4, from `connor-physics.md` "Walls" (running head-on: `crash` in full, traced Q31 e; measured push-back).
**Tested how:**

- `cargo test --workspace`: kq8-data 44, kq8-game 51. New tests:
  - run head-on → `crash` for 7 updates, 63.07 straight back each, 441.5 in all, frames 1–7, keys ignored, then `run`;
  - a crash pushing him into a wall behind: no head-on, stops at the wall, never inside.
- The landing test now uses `jumpend`'s real timing (11 frames in 10/15 s).
- Viewer at the `Tomb` (`--at 237968.9,187493.2,-0.2054 --hold 1,0,1 --frames 600`).
- Regression: walk head-on (`--at 237907.4,187510.2,0.027 --hold 1,0,0`), safe landing (`--lift 2000`).

**Result:**

- Run: contact (237959.7, 187449.2) at update 1; `crash` updates 2–8, ending at **(238049.8, 187881.3)**; measured in the original **(238050.1, 187881.3)**. Then runs again and crashes again every 10 updates, as A saw (twice in a row).
- `viewer.log`: `crash` = 8 frames, 0.4667 s, steps (0, 441.5, 0).
- Walk head-on: unchanged (stops at (237909.4, 187435.1), blocked).
- Landing: `jumpend` updates 9–18, back to `stop` at 19 (10 updates, as before).

**Still broken / not tested:**

- Checked by eye by the user afterwards (2026-10-12): looks good.
- No `crash.aud` sound (no sound yet). No `stoprun` before running again (no start animations, Q25).
- Only `run` crashes. The other run-like movements and the exceptions (`KQMonster`, `backJump`, the Connor flag, "no crash" objects) need things we don't have yet.
- The crash pushing him into a wall behind is tested only in a unit test, not at a real wall.

**Next:** 2d-3c (the wall block as session A traced it, Q38), then 2d-5 (objects).

## 2026-10-12 M2 step 2d-3b: walk held against a wall (no more frozen frame)

**Changed:**

- `crates/kq8-game/src/connor.rs`: the walker remembers which key direction (forward or back) a head-on stop blocked (`Walker::blocked`). While the keys still ask for it, he stands in `stop` and its animation plays on. A fresh press of that key ends the block (`Walker::key_pressed`), and so does any mode change. Running is never blocked.
- `crates/kq8-viewer/src/connor_view.rs`: passes fresh forward/back presses to the walker; new `connor.log` column `blocked` (`none`, `fwd`, `back`); event lines when the block starts and ends.
- Records: `docs/DESIGN.md` (plan 2d-3b, plus the user's choices for 2d-4 and 2d-5), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q38, for A's information), `README.md`.

**Why:** The user saw him frozen on the walk's first frame with forward held against a wall, and checked the original: he stops in the idle animation and doesn't try again until forward is pressed again.
**Tested how:**

- `cargo test --workspace`: kq8-data 44, kq8-game 49 (new: `a_wall_stop_holds_until_the_key_is_pressed_again`, for forward and back: 45 updates held → `stop` with its phase moving on; turn key → turns on the spot; the other key moves him; pressed again → moves again; run not blocked; a mode change clears it).
- Viewer, walk head-on at the `Tomb` (`--at 237907.4,187510.2,0.027 --hold 1,0,0 --frames 400`).
- Regression: run head-on (`--at 237968.9,187493.2,-0.2054 --hold 1,0,1`).

**Result:**

- Walk: head-on at update 2, stops at (237909.4, 187435.1) as before. Then `stop` for every update to the end (33 updates), phase 0.034, 0.069 … 0.966, 0.000, … (the idle loop plays), `blocked fwd`, position unchanged. No walk/stop alternation.
- Run: unchanged (contact (237959.7, 187449.2), still running, `blocked none`).

**Still broken / not tested:**

- Checked by eye by the user afterwards (2026-10-12): working.
- Turning on the spot with the blocked key held is our choice, not checked in the original (Q38).
- Running head-on still keeps running (2d-4); objects don't collide (2d-5).

**Next:** 2d-4 (crash).

## 2026-10-12 M2 step 2d-3 checked by eye (user)

**Changed:** `README.md`, this entry. No code.
**Tested how:** The user ran the viewer (`--at 237907.4,187510.2,0.027` and free play).
**Result:**

- Works: walls and floors; he goes inside houses and walks across bridges. At an angle he slides along walls, which is correct.

**Still broken:**

- No collision with objects (trees, the stump near the wizard, closed doors, DeadCity crates, and more): expected, step 2d-5.
- Running head-on: he stops but keeps the run animation, with no crash: expected, step 2d-4.
- Walking head-on: he stops, which is correct, but stands frozen on the walk's first frame while forward is held, instead of the idle (`stop`) animation. No flicker. Cause, likely: each update the walker restarts `walk` at frame 0, then the head-on stop switches it to `stop` at frame 0, so neither animation advances. Not fixed yet; it ties in with Q37 (d).

**Next:** Fix the frozen frame, then 2d-4 (crash) and 2d-5 (objects), after the user's go.

## 2026-10-12 Study session: Q34 and Q37 answered; notes and STATUS brought up to date

**Changed:** `docs/behaviour/connor-physics.md` (how the box table is read; head-on test only on up and across passes; slide count per pass; walk held against a wall; an old 1,600 height marked as superseded), `docs/behaviour/QUESTIONS.md` (Q34, Q37 answered, Q36 marked read). `STATUS.md` (not tracked) rewritten at the top, plus "What works", "What doesn't work", open questions and the fresh-session prompt. No code.
**Why:** The user asked for all documents and the status to be up to date; Q34 and Q37 were waiting for A.
**Tested how:** Traced in Ghidra. The box table was also read from the running original's memory (read only): 1,111 entries, sorted byte by byte, inside-out boxes already swapped, `rat` and `razetta.kqs` duplicates in the order given. Q37 (d) from my wall log of 2026-10-12.
**Result:** Box names: case matters; read by key `itemN`; inside-out boxes fixed on loading; duplicates: `rat` = item 8, `razetta.kqs` = item 775. Hit handling: head-on test only on up- and across-pass hits; slides counted per pass (pass hit plus 3 slide hits handled; a 4th slide hit stops him); walk held against a wall stays in `Stop` (measured about 2.8 s).
**Still broken / not tested:** Not traced: why the held walk key doesn't restart walking; Connor against monsters. The 225 wall stop is not checked directly in the game.
**Next:** B continues 2d. A: monsters when B gets to them.

## 2026-10-12 M2 step 2d-3: buildings are solid (stop, slide, steps, floors, ceilings)

**Changed:**

- `crates/kq8-game/src/mover.rs`:
  - The up, across and down passes, and every slide retry, sweep Connor's balls against the buildings. The note's "Exact hit handling" decides what happens: a head-on stop, put at the contact, or a slide along the surface; on the 3rd hit of an update he stays at the contact.
  - New `MoverWorld` (ground plus solids) and `Scene` (terrain plus buildings). A plain ground still works, so the old tests run unchanged.
  - New `Wish` (step and "in the Move state").
  - The report lists every hit, the across result, and whether a hit was head-on.
- `crates/kq8-game/src/collide.rs`: the rough check covers the balls' full reach (`BallStack::reach`, `overlaps`).
- `crates/kq8-game/src/connor.rs`: `Walker::moving`, and `Walker::stop_at_wall` (walk or back → `stop`).
- Viewer:
  - `load.rs` builds a collider for every building the scripts load (26 in Daventry, hidden ones included) and reads Connor's box from `BOUNDS.KQ`.
  - `terrain_view.rs` holds the colliders as a resource.
  - `connor_view.rs` passes them to the mover, stops the walk on a head-on hit, and logs more. New `connor.log` columns: `across hits down_what touch_gap`. Event lines for each building hit that isn't a floor landing, and one line when the surface he stands on changes.
- Records: `docs/DESIGN.md` (2d-3 done), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q37), `README.md`.

**Why:** M2 step 2d-3, from `connor-physics.md` "One update of the mover", "Walls" (with "Exact hit handling") and "Collision shape".
**Tested how:**

- `cargo test --workspace`: kq8-data 44, kq8-game 48. New tests:
  - head-on stop; backwards is head-on; not moving is never head-on;
  - glancing at 21.6° and 35° slides 0.37 and 0.57 of the step;
  - a corner; past the end of a wall;
  - a 300 step climbed and a 400 one blocked; a low ceiling;
  - standing on a building floor; the walker's wall stop.
- Viewer runs (`--frames 300–1500`, logs read), at session A's `Tomb` wall spots (`connor-physics.md` "Wall test positions"; heading = atan2(dx, −dy)):
  - `--at 237907.4,187510.2,0.027 --hold 1,0,0` (walk head-on);
  - `--at 237968.9,187493.2,-0.2054 --hold 1,0,1` (run head-on);
  - `--at 239837.4,187700.6,0.3777 --hold 1,0,0` (walk glancing).
- Inside Connor's house: `--at 263402.8,223443.2,0` standing, and with `--hold 1,0,0`.
- Regression: the slope and hill spots from 2c, `--lift 5000`, and running from the start spot.

**Result:**

- Run head-on: contact (237959.7, 187449.2), as measured in the original.
- Glancing: slides at y 187449.5–187449.6 from x 239937.0, as measured (27.6 per update at our walk speed).
- Walk head-on: stops at (237909.4, 187435.1), 210 from the wall's face. The original stopped at 187437.8; the note says anywhere between 200 and 225 depending on where the last free step ends, and our walk step is 75 per update against its 72.
- In the house: feet at 13,980 (as measured). Walking toward −y he stays at 13,980 for 43 updates, then stops head-on at the inner wall and never enters it.
- Regression: the slope and hill numbers are the same as before the change; the 5,000 lift still ends in one fatal fall; running from the start spot hits no building.

**Still broken / not tested:**

- Nothing looked at by eye.
- Running head-on stops but keeps the run animation; the crash is 2d-4.
- With the walk key held against a wall, the movement alternates walk/stop every update; whether the original does that is asked (Q37 d).
- Three more readings of the note are asked as Q37 (a)–(c).
- Objects still don't collide (2d-5).
- The ground check is terrain-only, as traced; not tried where the terrain rises above a building floor.

**Next:** The user checks by eye (commands in the summary). Then 2d-4 (crash), after the user's go.

## 2026-10-12 M2 step 2d-2: collision tests (ball stack, building BSP sweep, object boxes)

**Changed:** New `crates/kq8-game/src/collide.rs`: the ball stack built from a box, the building sweep through the BSP tree, the object box sweep, and the 25 pull-back. `crates/kq8-game/src/lib.rs` (module). `docs/DESIGN.md` (2d-2 done), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q36, the wall result for session A). Connor's movement doesn't use any of it yet.
**Why:** M2 step 2d-2, from `connor-physics.md` "Collision shape" (traced, Q31, Q33).
**Tested how:**

- `cargo test -p kq8-game`: 38 pass, 13 of them new:
  - balls: Connor's four balls, overlap, a single ball;
  - pull-back;
  - wall: from both sides, along it, moving away, too short a move, an exact touch, starting inside;
  - floor and ceiling; a low lintel and a high one; a turned building;
  - object box: square corners, a turned object, the vertical one-ball rule, start inside, an inside-out box.
- **Against the original's measurements** (throwaway program in the scratch folder, using `collide.rs` and the real `.dig` files):
  - `ConHouse` floor at session A's three points;
  - the `Tomb` wall from session A's start points (placement from `TOMB.KQ`: 233311, 183109, ground 8160, dirZ −1.57).
- Also measured, all 153 buildings: the BSP is open everywhere just outside each bounding box (2.3 million sample points, none solid), so the rough check only saves time.

**Result:**

- Floor: feet at 13,980.0 (measured 13,980).
- Wall, run head-on: contact (237959.73, 187449.17), measured (237959.7, 187449.2).
- Wall, walk head-on: stops at (237909.4, 187437.8), as measured. The step before is free; the stopping step touches 13 in, so he stays where he was.
- Wall, glancing: contact y 187449.56, measured 187449.4–187449.6. Wall face at y 187224.7 (session A's estimate 187225).

**Failed attempts, corrected:**

- Two unit tests failed at first, both my test setups: the lintel's last split was the wrong way round (solid below the beam), and a "too short" move was really 50 units long.
- I also found that in 32-bit numbers `200 + 0.00001` is exactly 200, so the traced touch margin vanished. That comparison is now done in 64-bit numbers.

**Still broken / not tested:** Not in the movement yet, so nothing changes in the viewer. Inside-out object boxes give no hit until Q34 (c) is answered (objects are 2d-5).
**Next:** 2d-3, buildings in the up, across and down passes, with head-on stops and glancing slides.

## 2026-10-12 Study session: 25 pull-back on floors confirmed (Q35)

**Changed:** `docs/behaviour/connor-physics.md` ("Collision shape" pull-back bullet, "Walls" floor height), `docs/behaviour/QUESTIONS.md` (Q33 b, Q35 marked read). No code.
**Why:** B found the floor of Connor's house at 13955 in our data (Q35).
**Tested how:** My measurement in the original (feet at 13980 inside `ConHouse`) compared with B's floor height at the same three points.
**Result:** Feet 25 above the floor: the traced 25-unit pull-back on landings is confirmed in the game.
**Still broken / not tested:** The pull-back against walls (225 stop) and ceilings is not checked directly.
**Next:** None needed for floors.

## 2026-10-12 Floor height in Connor's house: the 25 pull-back on landings confirmed

**Changed:** `docs/behaviour/QUESTIONS.md` (Q35, the answer for session A), `docs/DESIGN.md` (2d "Still to check"). No code.
**Why:** Session A measured Connor's feet at z 13,980 inside his house in the original. It asked for the floor's height in our data: 13,955 would confirm the traced 25 pull-back on landings, 13,980 would refute it.
**Tested how:** A throwaway program in the scratch folder (not in the repo), using `kq8-data`. It reads `cnnrhut.dig`, places it as our viewer does (262411, 228400, 13795, no rotation, from `viewer.log`), then at session A's three points: (1) asks the BSP tree whether each height is solid or open, every 0.25 units straight down; (2) lists the surfaces covering the point.
**Result:** At all three points the BSP is open down to own z 160 and solid from 160 to 0. The solid top is a plane at own z 160.00 carrying the floor surfaces, so the floor is at **13,955**: feet 25 above it, the pull-back confirmed.
**Failed attempt, corrected:** My first listing used the wrong facing rule. It took the plane direction with the flip bit as the visible side; it is the opposite (Q15). So I briefly reported that the drawn floor was 160 below the collision floor. Read the right way, the surfaces at own z 160 are the floor seen from above, and those at 0 its underside. The drawn floor and the collision floor agree.
**Still broken / not tested:** The wall prediction (centre 225 from the face) is checked after 2d-3.
**Next:** 2d-2, the collision tests.

## 2026-10-12 Study session: Connor's height on a building floor measured

**Changed:** `docs/behaviour/connor-physics.md` ("Walls", wall test positions: height on a building floor). No code.
**Why:** B asked for his height on a building floor to check the 25 pull-back in our viewer.
**Tested how:** User walked from the ground outside Connor's house into it and stood still twice; my scratch reader logged his position and the surface he touched from the running original (read only).
**Result:** Ground outside: feet at z 13770 (terrain). Inside `ConHouse`, e.g. at (263402.8, 223443.2): feet at z 13980, surface straight up. The kept touch point equals his feet position, as traced.
**Still broken / not tested:** The floor's own height was not read, so the 25 is still not confirmed: B's viewer should find 13955 there if it is right. While stopping my reader I used a command that may also have closed any other Python program running at that moment.
**Next:** B compares the floor height in the viewer.

## 2026-10-12 M2 step 2d-1: collision data read (box table, building BSP trees)

**Changed:**

- New `crates/kq8-data/src/bounds.rs`: reader for `GAME/patch/BOUNDS.KQ`. Keeps every entry in item order; lookup by name ignoring case returns all matches; reports duplicates.
- `crates/kq8-data/src/building.rs`: now reads the BSP nodes (table 2) and the leaves' first byte (table 3) instead of skipping them.
- `kq8 building-verify` (`crates/kq8-cli/src/building_verify.rs`): checks the BSP trees and the box table.
- `docs/provenance/README.md`, `docs/DESIGN.md` (2d-1 marked done), `docs/behaviour/QUESTIONS.md` (Q34).

Nothing in the game or viewer uses these yet.
**Why:** M2 step 2d-1, the data that the traced collision rules need (`connor-physics.md` "Collision shape", Q31, Q33).
**Tested how:** `cargo test --workspace` (kq8-data 44 tests, 3 new: box table, duplicates, BSP tables on made-up bytes; kq8-game 25). `cargo run -p kq8-cli -- building-verify` (log in `kq8-artifacts/logs/building-verify.log`).
**Result:**

- `building-verify`: PASS, 153 buildings.
  - 30,400 nodes and 37,132 leaves. Every index is in range, node 0 is nobody's child in every tree, every other node has exactly one parent, and all nodes are reachable from node 0.
  - Every plane normal has length 1.00000 (the sweep's distances rely on that).
  - Leaf first bytes: 0 (16,688), 1 = solid (13,865), 2 (6,579). All the 2s are leaves that no node leads to.
- `BOUNDS.KQ`: 1,111 entries, `nItems=1111`, numbered 0–1,110 each once, no unreadable line. `Connor` = −200 −200 0 200 200 1,300.
  - Names used twice: `henchman`/`Henchman`, `rat` and `razetta.kqs` with different boxes; `sprigan` and `trip2`–`trip5` with the same box.
  - Inside-out boxes: `lfswdoor.kqs` (known) and `iceshard.kqs` (new).
  - A `#item968` line beside a normal `item968`.

**Still broken / not tested:** Q34 (which duplicate wins, case, `#` lines, inside-out boxes) is open; it only matters for objects (2d-5). The viewer was not run (nothing changed there).
**Next:** 2d-2, the collision tests in `kq8-game` (not used by the mover yet), with unit tests.

## 2026-10-12 Study session: wall test positions written out for checking

**Changed:** `docs/behaviour/connor-physics.md` ("Walls": a table of the wall test positions from my memory log, and what they imply). No code. (Scratch reader in `re/` now also logs the last surface touched; not committed.)
**Why:** B can't read my logs in `re/` and wants to put Connor at the same spot in our viewer and compare.
**Tested how:** Re-read my memory log of 2026-10-12 (Daventry wall tests). Reader change tried for a few seconds on the running game: it logged a terrain contact at his feet, as expected.
**Result:** The run contact and the glancing slide sit at y 187449.2–187449.6; from the traced rules the wall's face is at about y 187225, facing +y. The walking head-on stop is 11.6 closer, which fits "contact within 25 of the start → stay at the start". So the log supports a pull-back of at least 12; it doesn't prove exactly 25.
**Still broken / not tested:** Wall face position is worked out, not read from the data. Height on a building floor: not measured (needs a new session in the game).
**Next:** B compares in the viewer. If wanted: a short session in the game to measure the height on a building floor.

## 2026-10-12 M2 step 2d plan rewritten from the traced collision rules

**Changed:** `docs/DESIGN.md`, M2 step 2d only. No code.
**Why:** Session A answered Q31 and Q33. The old plan had stand-ins of our own: a box sweep against surfaces, every surface solid, hidden objects not colliding, a 1,600-tall box. It now follows the traced rules: Connor's box from `GAME/patch/BOUNDS.KQ` (1,300 tall, four balls of radius 200 at 200, 600, 1,000, 1,100); the `.dig` BSP sweep with solid leaves; the 25-unit pull-back on building and object hits; the slide projection; head-on judged by movement; object boxes by name from `BOUNDS.KQ`; hidden objects colliding.
**Tested how:** Data check only: `MASK/GAME/patch/BOUNDS.KQ` read by eye. `nItems=1111`; `Connor -200 -200 0 200 200 1300` (two older lines with 1,600 are commented out with `;`); `signpst1.kqs`, `shut1r.kqs` and `chandlir.kqs` have entries; `Davtorch` has none (so it never collides). Searched the code and docs for other uses of 1,600: none.
**Result:** Plan ready for the user's OK.
**Still broken / not tested:** Two traced predictions aren't checked in the original: standing 25 above building floors, and stopping with his centre 225 from a wall. Session A's wall log is in `re/`, which session B must not read, so the stop position needs to be written into the note for comparison.
**Next:** The user's OK, then 2d-1.

## 2026-10-12 Study session: Q33 answered (object boxes, pull-back, slide, head-on)

**Changed:** `docs/behaviour/connor-physics.md` ("Where the values live", "Collision shape", "Walls"), `docs/behaviour/QUESTIONS.md` (Q33 answered, Q31 corrected), Connor's height in `connor-movement.md` and `camera.md`. No code.
**Why:** B's Q33, for M2 step 2d (walls).
**Tested how:** Traced in Ghidra. Connor's box and balls also read from the running original's memory (read only, outside the process): box −200…200, −200…200, 0…1300; balls at 200, 600, 1000, 1100, radius 200.
**Result:** Object boxes come from `GAME/patch/BOUNDS.KQ` by name; an object not listed never collides. **Correction:** Connor's box is that table's `Connor` entry, not `[ConnerCollider]`; his top ball is at 1100, not 1400. The 25 pull-back applies to every building and object hit in every pass, never to the terrain. The slide starts at the pulled-back contact. The head-on test uses his movement, not his facing. The BSP root is node 0.
**Still broken / not tested:** Not checked in the game: standing 25 above building floors, and stopping 225 from a wall. Not traced: Connor against monsters (ball stack vs ball stack), and what happens when a ball starts inside an object's box.
**Next:** B uses this for 2d. A check of the 225 stop distance would settle the pull-back.

## 2026-10-12 Q33 asked (walls follow-up)

**Changed:** `docs/behaviour/QUESTIONS.md`: Q33. No code.
**Why:** After Q31's answers, four points still decide exact behaviour for M2 step 2d: which box objects use, where the 25-unit pull-back applies, where a glancing slide starts, and whether head-on is judged by movement or facing. The user wants all answers in before coding.
**Tested how:** Data check only: `kq8 text vol:Daventry/kq/<file>` for `signpst1.kq`, `Davtorch.kq`, `shut1r.kq`, `chandlir.kq`.
**Result:** None of them has a collider section; `signpst1.kq` has `#colliderType=257` and `#colliderMask=31`, commented out.
**Still broken / not tested:** 2d not started. Whether the BSP root is node 0 is not checked yet (to do from the data, read only).
**Next:** Session A answers Q33; then B rewrites the 2d plan and shows it before coding.

## 2026-10-12 Downhill rule removed

**Changed:** `crates/kq8-game/src/mover.rs`: the downhill rule (our own design, from the note's recommendation) is gone, along with its `snap` result in the log's `down` column. The unit test `running_down_walkable_slopes_never_falls` now checks walking and running at 5°, 20°, 30° and 36°: no Fall and the air count never reaches 5. It no longer requires him to be on the ground every update. `docs/provenance/README.md`, `docs/DESIGN.md` (2c) marked.
**Why:** With the measured gravity (G1) he gets back to a walkable slope within 1–3 updates, as in the original. The rule kept him glued to the ground, which the original doesn't do (entry below). Asked for by the user.
**Tested how:** `cargo test -p kq8-game` (25 pass). Viewer, `--frames 1000`, logs summarised as in the entry below: `--at 293994,218295,-1.222` and `--at 286169,190211,0.291`, each with `--hold 1,0,0` and `1,0,1`; also `--lift 5000`.
**Result:** Same as the "rule off" runs below. Walking down 36°: off the ground 23 of 84 updates, air count at most 1. Running down: 16 of 85, at most 3. Hill: walking at most 1, running at most 3. No Fall in any of them. `--lift 5000`: unchanged, Fall at update 6.
**Still broken / not tested:** Not looked at by eye: the small bounce going downhill should now show in the camera, as in the original. Only one slope tried.
**Next:** M2 step 2d (walls), once the user OKs the plan.

## 2026-10-12 Slopes rechecked with the corrected gravity; M2 step 2d (walls) planned

**Changed:** No code (two temporary experiments, both reverted, see below). `docs/DESIGN.md`: plan for M2 step 2d (walls), in 5 sub-steps. `docs/behaviour/QUESTIONS.md`: Q31 (walls: box turning, how the box is tested, which surfaces and objects are solid, crash length) and Q32 (this check, for session A).
**Why:** `connor-physics.md` open questions 6 and 7 were measured in the original and say "check with B's corrected gravity". Then plan 2d, as asked.
**Tested how:** `cargo test -p kq8-game`, and viewer runs (`--frames 1000`, about 85 updates each) on the two Daventry spots from step 2c: walkable 36° slope `--at 293994,218295,-1.222`, steep 52° hill `--at 286169,190211,0.291`, each with `--hold 1,0,0` (walk) and `1,0,1` (run). Each run twice: the downhill rule on, then off (a temporary environment-variable switch in `mover.rs`, removed afterwards; tests pass again, 25). `connor.log` summarised with a throwaway script in the scratch folder.
**Result:**

| Run | Rule on | Rule off | Original (measured by A) |
| --- | --- | --- | --- |
| Walk down 36° | never off the ground | off every other update (drop 36), lands next (73 down); max air count 1 | off every other update after 36, lands next (25–85) |
| Run down 36° | never off the ground | off 3 updates (36, 108, 180), lands with 252; max air count 3 | off 1–3 updates (once 4), never 5 |
| Walk into 52° hill | at most 14 up, no Fall | same | barely gains height |
| Run into 52° hill | at most 72 up, 3 updates off, back on the flat, no Fall | same | at most 80 up (usually 5–40), 2–4 updates off, no Fall |

- **The downhill rule is no longer needed.** Without it, our runs match the original's measurements, including the small bounce going downhill that the rule removed. With it, he never leaves the ground. Not removed yet: it is a separate step for the user to approve.
- With the rule off, the unit test `running_down_walkable_slopes_never_falls` fails. It is the test's own check that he is on the ground every update; its "air count under 5, no Fall" checks still pass.
- Seen in the hill log: at the foot of the hill he is on the flat (the ground check) while his down pass hits the hillside, so the fall timer keeps growing (0.30, 0.36, 0.42), then resets. That is likely the "also seen, not understood" in open question 7; sent to A as Q32.
- **Data check for 2d:** `crash` (`c_crash`) moves him 441.5 units straight back over 7 steps (63 per update), the measured push-back. Read by loading `crash` and `stoprun` in a temporary build; reverted (`CONNOR_MOVEMENTS` back to 9 entries).

**Still broken / not tested:** Only one slope and one hill, and only straight down or straight in. Nothing looked at by eye.
**Next:** User's OK on the 2d plan (and on removing the downhill rule as its own step). Q31 for session A.

## 2026-10-12 M2 follow-up G2: walking off a ledge doesn't reset the fall timer

**Changed:** `crates/kq8-game/src/mover.rs`: when Fall starts, the fall timer is no longer set to 0.36; it runs on from the 5 updates in the air (0.30). `FALL_START_TIMER` (0.36) is kept for jumps (step 2e) and not used yet. New unit test `walking_off_a_ledge_keeps_the_fall_timer`. The log line now says "fall timer runs on from 0.30 s". `docs/provenance/README.md` and `README.md` updated.
**Why:** `connor-physics.md` "Falling off an edge" rule 2 (corrected 2026-10-12, measured in the original): the head start is only for Fall after a jump. From walking or standing, gravity is left alone.
**Tested how:** `cargo test -p kq8-game` (25 tests pass). Same viewer run as G1 (`--lift 5000 --frames 400`), then read `connor.log` and `viewer.log`.
**Result:** Fall timer 0.24, 0.30, then 0.36, 0.42, … (before: 0.30 → 0.42). Drops 252, 324, then 396, 468, 540, … as the note gives for a walk-off. Fall at update 6, lands at update 12 after 4,100 units, dies (more than 3,840).
**Still broken / not tested:** Not checked by eye. The "start height lowered by a fixed amount" after a jump isn't relevant until jumps exist (2e).
**Next:** The user checks a few falls by eye. Then M2 step 2d (walls) or 2e (jumps).

## 2026-10-12 M2 follow-up G1: gravity 4 times stronger (as measured)

**Changed:** `crates/kq8-game/src/mover.rs`: `GRAVITY` 5,000 → 20,000, so the drop per update is `10000 × (t² − t_prev²)` (`connor-physics.md` "Gravity", Q30). Two unit tests' expected numbers changed (below). Plan in `docs/DESIGN.md` (M2, "Follow-up after the fall-speed measurement"); `docs/provenance/README.md` rows updated.
**Why:** Session A measured the running original: every drop in the air is exactly 4 times the old traced rule. Our falls were about 4 times too slow.
**Tested how:** `cargo test -p kq8-game` (24 tests pass). Viewer run `cargo run -p kq8-viewer -- --terrain --lift 5000 --frames 400`, then read `kq8-artifacts/logs/connor.log`.
**Result:**

- Unit test drops: 36, 108, 180 (were 9, 27, 45).
- 45° hill unit test: forward gains 66, 30, −6, −42, exactly the note's rule 168·cos²θ − d·sinθ·cosθ. In update 5 the slide carries him back past the foot of the hill and he is on the flat again, so only 4 updates end in the air and **no Fall starts**. The old test expected a Fall, then a slide; it was rewritten to expect this.
- Viewer, lift 5,000 in Daventry: drops 36, 108, 180, 252, 324 over the 5 air updates, Fall at update 6, lands at update 12 after 4,100 units and dies (before: landed at update 22).

**Still broken / not tested:** Not checked by eye in the viewer. The Daventry hillside numbers in the provenance file (Fall at update 17 on a 52° hill) are from before this change and weren't redone. Fall start still sets the timer to 0.36 (visible in the log: timer 0.30 → 0.42, drop 468 instead of 396): that's step G2.
**Next:** G2.

## 2026-10-12 Study session: building collision traced exactly (Q31 finished)

**Changed:** `docs/behaviour/connor-physics.md` (Collision shape: the building test step by step, solid surfaces, when objects collide, hidden objects), `docs/behaviour/QUESTIONS.md` (Q31 now answered). Scratch addresses in `re/notes.md`.
**Why:** The user asked for exact rules instead of measurements for the three open Q31 points.
**Tested how:** Traced in `Mask.exe` with Ghidra: the collision manager's event handling, the dispatcher that picks a test by collision kind, the ball-stack-against-building test, the BSP trace and its helpers, the hit handler, collider registration and `KQObject::setHidden`. Some class tables were found by scanning the exe copy in `re/` for the class type records.
**Result:**

- Buildings (`KQInterior`) are their own collision kind. All four of Connor's balls (r 200) are traced through the `.dig` BSP (nodes, leaves, planes). The earliest hit counts, and the contact is pulled back 25 toward his old position. Moves under 0.1 aren't tested.
- Solid = a BSP leaf whose first byte has bit 0 set. Surfaces and materials don't matter.
- Hidden objects still collide (hiding only changes the drawing). Objects with type 0 and mask 0 never get a collider.

**Still broken / not tested:** Nothing of this has been checked in the running game. Connor against monsters (ball stack against ball stack) isn't traced. The game-wide "ready" check that can skip all pairs isn't traced (probably loading).
**Next:** B can implement building collision from the note. A possible check in the game: walk into a hidden object and see if it blocks.

## 2026-10-12 Study session: wall collision traced (Q31), Q32 confirmed

**Changed:** `docs/behaviour/connor-physics.md` (Collision shape, Walls, open question 7), `docs/behaviour/connor-movement.md` (a note on update counts), `docs/behaviour/QUESTIONS.md` (Q31, Q32). Scratch addresses in `re/notes.md`.
**Why:** B's questions for M2 step 2d (walls).
**Tested how:** Traced in `Mask.exe` with Ghidra. Some class tables were found by scanning the exe copy in `re/` for the class type records, because Ghidra hadn't linked them. Q31 (e) was checked against the wall log (`re/dbg/mem_move.log`).
**Result:**

- The precise collision uses a stack of balls built from the box: Connor has r = 200, four balls at heights 200, 600, 1000, 1400. They don't turn with him, so turning on the spot is never checked. The box only sizes the balls and a rough pre-check.
- Against objects: balls swept against the other object's box (which turns with that object). Against buildings: a ball swept through the `.dig` BSP, first hit, blocking from both sides.
- Q31 (e): `crash` plays in full, 441 back. My earlier "5 updates" counted log lines (drawn frames), not ticks. Corrected, and a caveat about update counts added to the measured sections.
- Q32: B's explanation of the timer oddity fits the traced rules; confirmed.

**Still broken / not tested:** Which radius and balls the building test uses (probably 200) was not followed to the end. Which building surfaces are solid (Q31 c) and whether hidden objects collide (Q31 d) were not traced. Stopped there rather than keep going in circles.
**Next:** Either trace those three in Ghidra, or measure them in the original with the reader (stop distance from a wall gives the radius).

## 2026-10-12 Study session: movement, turning, jumps and walls measured in the running original

**Changed:** `docs/behaviour/connor-movement.md` (new "Measured" section, open questions 1 and 2), `docs/behaviour/connor-physics.md` (Walls, open question 3), `docs/behaviour/QUESTIONS.md` (Q24, Q25). In `re/` (not committed): the reader now also logs which way Connor faces.
**Why:** The game was still open; the same reader could answer Q24, Q25 and check the wall rules before B's step 2d.
**Tested how:** The user, in the original on flat ground: walked 4 times and ran 4 times (start and stop each), turned on the spot both ways, did running jumps; then at a building wall, walked and ran into it head-on and at an angle. The reader logged every update (`re/dbg/mem_move.log`).
**Result:**

- Speeds match the data: walk about 1,100/s (2% under 1,127), run about 2,520, back about 1,080. Turning 98–102°/s (data 100).
- Q24: the distance per tick changes smoothly with the step cycle (walk 70–81), so steps are spread within each frame.
- Q25 (partly): stops wait 0–4 updates, then a listed stop animation plays; walking fits "the next listed point". Running not settled.
- Running jump: about 6,480 to the landing; `jumprun` if run is still held, `jumpstop` if not.
- Walls confirm the traced rule: head-on (under 20° from straight in) walking stops at once with no stop animation; running crashes for 5 updates and is pushed back about 440–600. Otherwise he slides along the wall at step × sin(angle).
- The user's angled tests were 21.6° and 35° from straight on, not 45°. Both slid, as the user saw.

**Still broken / not tested:** Run stop choice (Q25) not settled. The animation phase isn't logged (not found in memory). Only one jump of each ending. Only one wall.
**Next:** B can use these for step 2d (walls) and the start/stop animations. A: find where the animation phase is stored, then repeat the run stops.

## 2026-10-12 Study session: slopes measured in the running original (open questions 6 and 7)

**Changed:** `docs/behaviour/connor-physics.md` (open questions 6 and 7), `docs/behaviour/QUESTIONS.md` (Q30 b). No code.
**Why:** Two slope puzzles: in the original he never falls going down walkable hills, and he barely climbs steep ones. Our rules predicted otherwise.
**Tested how:** The user played the original normally while `re/dbg/mem_fall.py` read Connor's mover from memory (log `re/dbg/mem_slope.log`). Walked and ran down a hill of 19° to 37°, then ran into a 45–54° hillside for about 12 seconds.
**Result:**

- Walking down 34–37°: off the ground every other update (drop 36), lands the next. Air count never above 1.
- Running down: 1–3 updates off the ground at a time (once 4), never 5, so no Fall.
- Running into the 45–54° hillside: at most 80 units up, 2–4 updates in the air per try, back on the flat each time, no Fall.
- No extra pull onto the slope: every drop was the gravity rule. Likely explanation for both (not proven): the gravity is 4 times stronger than our code had, so he gets back down before the count reaches 5.
- Also seen, not understood: on flat ground at the foot, the fall timer kept growing for a few updates while he counted as on the ground.

**Still broken / not tested:** The explanation isn't tested. It needs B's code with the corrected gravity, run on a similar slope.
**Next:** Session B, after the gravity change: rerun the walking and running hill tests and the steep-slope run, and compare air counts and climb height with these numbers.

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

## 2026-10-11 Slopes follow-up: Q28 (a) and Q29 answers

**Changed:** `crates/kq8-game/src/mover.rs`. The gravity drop is projected onto the pull: d·sinθ along a slope, the full d straight down (Q28 a). The test lift no longer clears "on ground". Unit tests: the 45° test checks the note's numbers; the lift test expects Fall at update 6; the ground-check test sets the flag directly. Also `docs/DESIGN.md` (plan) and `docs/provenance/README.md`.
**Why:** Session A answered Q28 (a) and Q29. The note was right about the drop, and our code moved Connor by the full drop along the slope. Fall should start after 5 updates in a row that ended off the ground. For real updates our count already did that, as the walking log shows (26–30 off the ground, Fall at 31). But the test lift cleared "on ground", so the lift itself counted as an update in the air, and lifted falls started one update early (at 5).
**Tested how:** `cargo test -p kq8-game`. Viewer runs read from the logs: Daventry hill walking and running (`--at 286169,190211,0.291 --hold 1,0,0` / `1,0,1`), `--lift 2000`, `--lift 5000`.
**Result:**

- Unit tests: 24 of 24 pass. At 45° the gains are 79.5, 70.5, 61.5, 52.5, 43.5 (the note: 80, 71, 62, 53, 44).
- Walking at the hill: 1 fall in 101 updates (before: 12 in 217). That fall is the first approach: 58 units up, 5 updates off the ground, and Fall starts although he lands in that update, as Q29 says. After it he goes in and out of the slope with the walk playing: 20–30 units up, 4 updates off the ground, back on the flat. That matches the user's check of the original.
- Running: still falls every time, with 249–271 units of slide (before: 196–214).
- `--lift 2000`: Fall at update 6, safe after 1,775. `--lift 5000`: Fall at 6, dies after 4,775.

**Still broken / not tested:**

- Running into a steep slope climbs far higher than in the original and always falls. Not explained by the traced rules (`connor-physics.md` open question 7).
- The first walking contact still gives one short fall. Whether the original does this too is not checked.
- Q28 (b), (c) open. Nothing looked at by eye.

**Next:** The user compares walking and running into a steep slope with the original. Open question 7 is for session A.

**Added later (same day):** The user reports that falls are much slower than in the original, both off steep slopes and off ledges. Measured and asked as Q30, no code change:

- Off the top of a steep slope: he floats 4 updates and ends 729 above it. In Fall the drop goes along the slope (the pull stays downhill until he lands), so 62→157 down and 58→148 sideways per update.
- Off a ledge: Fall at 117 units per update, +18 per update.
- `MASK/console.cs` still has `KQGame::FPSLimit 15`; the Windows 11 fix only adds `PlayFromCD 0` (compared with `console.bak`).
- Daventry's ground has no cliffs (a scan found no drop steeper than about 60° over 500 units), so a ledge was simulated with the lift.

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

## 2026-10-11 M2 step 2c (slopes) and the falling part of 2e

**Changed:** `crates/kq8-game/src/mover.rs` (terrain tested at the end of the up and down passes, steep push-off, the downhill rule, gravity along the pull with the 30° rule, Fall asked for on every update from 5 in the air, the ground check no longer resets the fall timer, fall events in the report, `update_with`). `crates/kq8-game/src/connor.rs` (modes free/fall/land/dying/dead, one-shot animations). `crates/kq8-viewer`: loads `jumphold`, `jumpend` and `jumpdie`; a loop name missing in a model now plays the model's first loop (`objects.md` Q13 b) instead of failing; new `--at X,Y,H` start option; new `connor.log` columns. Also `docs/DESIGN.md` (plan), `docs/provenance/README.md`, `docs/behaviour/QUESTIONS.md` (Q28), `README.md`.
**Why:** Q26 and Q23 are answered. The user asked for slopes and falling together, following `connor-physics.md` with the recommended rule for walkable downhill.
**Tested how:** `cargo test -p kq8-game` (new unit tests on made-up ground: 45° hill, downhill planes at 5–36°, ledges of 300 and 5,000, the ground check and the fall timer, an up-pass hit, landing and death animations). The viewer, run by me, with the log read afterwards (not looked at):

- steep hill: `--at 286169,190211,0.291 --hold 1,0,1`;
- steepest walkable slope found: `--at 293994,218295,-1.222 --hold 1,0,1`;
- falls: `--lift 2000 --hold 1,0,0` and `--lift 5000`.

The two spots came from a throwaway scan of Daventry's terrain in the scratch folder, not kept.
**Result:**

- Unit tests: 20 pass.
- Hill (ground normal z 0.610, about 52°): the down pass hits steep ground every update. He climbs 60, 60, 45, 31, 17 units of height, with a push-off of 148–173 units each update. Fall starts at update 17 (5 updates in the air), he slides back and lands at update 19 after 214 units, `jumpend` plays for 10 updates, then he runs at the hill again.
- Walkable slope (normal z 0.808, 36°): 34 updates, dropping about 122 per update; 21 `snap` and 13 `walkable` down passes; the air counter never left 0; no Fall.
- `--lift 2000`: Fall at update 5, safe landing at 14 after 1,875, `jumpend`, then walking again.
- `--lift 5000`: lands at 22 after 4,856, `jumpdie` for 10 updates, then the Die state on its last frame.

**Failed attempt:** At first the walk step for an update was taken before the mover decided on Fall. On a steep hill, the update where Fall started still pushed him uphill, so "moved up" ended the fall at once. Fall then restarted every update, and he crept up the hill about 1.3 units per update forever (unit-test trace). Fixed by deciding on Fall first (`Mover::update_with`), so that update already uses `jumphold`'s step.
**Still broken / not tested:**

- Nothing looked at by eye: the animations, the camera during falls and slides, and the keyboard (the runs used `--hold`).
- 4 camera unit tests fail. This is not from this change: the manual values in commit `10cf08b` (`AIM_HEIGHT` 800, `HEIGHT_RATE` 0.5) no longer match the tests.
- The note's 45° example differs from its own gravity rules (Q28 a).
- `jumpdie`'s steps are not applied (Q28 b). Keys and turning are ignored while falling and landing (Q28 c).
- `C_jhld.kqs` has no loop named `C_jhld`; its first loop is used, as traced.
- No sound (`jumpdie.aud`).
- Buildings and objects are still not solid (2d).
- **Found after the user's check of the original:** walking slowly into an unwalkable slope never starts the falling animation in the original, but ours does. Walking head-on at the Daventry hill: 12 short falls in 217 updates; each time he is back over flat ground but 4.8 units above it when the 5th update in the air comes. Running: 5 falls in 102 updates, 196–214 units of slide each. Asked as Q29; no code change until it's answered. The user's further checks: it depends on height, not speed. He barely climbs (much less than ours) and is pushed in and out of the slope, and only falls when he gets high enough to be off the ground. Added to Q29 (d, e).

**Next:** The user checks by eye (test steps in the chat). Then 2d (walls), which needs the box-vs-building test from session A.

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
