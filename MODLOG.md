# MODLOG

What changed and what was tested. Newest entries at the top.
Format: date and title, then Changed / Why / Tested how / Result / Still broken / Next.

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
