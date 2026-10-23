# MODLOG

What changed and what was tested. Newest entries at the top.
Format: date and title, then Changed / Why / Tested how / Result / Still broken / Next.

## 2026-10-22 Q143 (c) settled: the spinning mask is shrunk to its place

**Changed:** Docs only: `potions.md` rule 35 "Details", `QUESTIONS.md` Q143, `STATUS.md`.
**Why:** The user checked the original: the mask's edges are not cut off. The code clips each status picture to its 48 × 48 place, so the 57 × 57 mask must be shrunk to fit.
**Tested how:** The user, by eye in the original; the clipping was traced earlier today.
**Result:** Shrunk to 48 × 48, no overlap into the next place. The scaling itself was not traced.
**Still broken / not tested:** The user thinks ours overlaps the next icon a little; to be checked in session B.
**Next:** B: draw the mask shrunk into its own place.

## 2026-10-22 Q126, Q152, Q14 (b) settled with the user's checks in the original

**Changed:** Docs only: `conversation.md` rules 14, 15 (the camera's swing centre), `world-change.md` rule 23 (the loading meter's black), `objects.md` "See-through objects" rule 4 (the pad); `QUESTIONS.md` Q14, Q126, Q152.
**Why:** The user checked the three answers that disagreed with their memory: the conversation camera swings round a point a bit in front of the partner, between them and Connor (Wizard and Uglybeast); the loading meter's painted frame stays, black only inside it; a screenshot shows the teleport pad solid.
**Tested how:** Q126 re-traced in `Mask.exe`: the factor the study fork read as 0 is an 8-byte 0.5 (it had been read as a 4-byte number), and the other end is Connor's talking spot from the partner's info file.
**Result:** Q126: the centre is half-way between the partner's feet and Connor's talking spot, + 1,400; matches the user. Q152: the code's full-width black bar doesn't show in the original; build black over the meter slot's unfilled part only. Q14 (b): solid, as traced.
**Still broken / not tested:** Why the loading black bar doesn't show was not traced. Q126's other conclusions (range, height rule, obstacle check, end placement) are unchanged.
**Next:** B builds Q126 (talking spot instead of Connor's position) and Q152 (black in the slot).

## 2026-10-22 Study (A): the non-blocking questions Q14, Q42, Q112, Q125, Q126, Q136, Q138, Q140, Q143–Q145, Q150–Q152

**Changed:** Docs only. `connor-physics.md` "Jumping" rule 2 (the jump is picked from the held keys); `conversation.md` rules 13–15 (keys during the approach, Jump/push dropped, the camera's swing centre); `fighting.md` rule 23 (`reactPause` is for two special classes only); `ranged-weapons.md` rules 24, 25, 29 and open questions 1, 3 (the `do` frame, aim 975 not 1,200, the bolt's ball, `collisionDetail`, a swap during a swing); `anim-scripts.md` rule 5 (Connor's movements start at their `loop=` phase); `objects.md` (new "Dithered objects"; the teleport pad drawn solid); `potions.md` rules 20, 35 (three shields, how orbits move; status icon places and frames); `models-and-textures.md` (a texture's own colour table is ignored; flag 0x2000's alpha from the palette's `tpal`); `magic-map.md` rules 16, 21, 27, 34 and `interface.md` rule 44; `world-change.md` rules 23, 26 and a Do-not-reproduce item. `QUESTIONS.md`: those 15 marked answered (Q138's earlier history kept).
**Why:** The rest of session B's open questions from M4–M6.
**Tested how:** Traced in `Mask.exe` with Ghidra (five study forks); scratch in `re/scratch/` (untracked). Data checked where the notes say "data".
**Result:** Changes B's builds in these places: the portal popup opens with **Yes** highlighted (Escape answers the other button); the mini-map slot **never blinks**; map icons on another world's map are drawn after the colour match; `dither 1` is a screen-door pattern on the texture, not 50 % see-through; the mask is clipped to its own icon place; the character's bolt start uses the frame after the update's step; the jump at a wall comes from the held key; turn keys work during a conversation's approach; the conversation camera swings round the partner's feet + 1,400 (not half-way). Confirmed as built: the bolt's ball of 235, aim 975, the swap waits for a standing swing, the icon frames, three shields, entry 0 black.
**Still broken / not tested:** Nothing checked in the running original. Three answers disagree with the user's memory and need a look: the conversation camera's swing centre (Q126: re-check with Connor far from the partner), the loading screen's black strip (Q152) and the teleport pad (Q14 b: drawn solid by the code). Not traced: whether the 57 × 57 mask is shrunk or cut, the popup's hover frame, the loading phases' colours.
**Next:** B builds the changes above. The user's three checks.

## 2026-10-22 Study (A): the blocking questions Q153–Q159, plus Q146, Q148, Q149

**Changed:** Docs only. `docs/behaviour/objects.md` ("Loops" rules 2, 4; "Loops that move the whole object" rule 6: which frame a phase shows, the 0.0 cue), `anim-scripts.md` (new rules 33–37: `KQObject::setScript`, `loadObject`, `sendEvent @` in cues, `random`, the ravens), `characters.md` (new sections: Connor's Choice state, `StateChange` messages; a Do-not-reproduce item), `interface.md` (rules 22, 26, 26a, 30, 30a: the level-up flame and `LEVEL.AUD`, `STATS.AUD`, `setLevel`), `azriel-hammer.md` (rules 3, 7, 9, 9a, "Connor's waiting request"), `potions.md` rule 26, short pointers in `traps.md` rule 15 and `talking.md` rule 13. `QUESTIONS.md`: Q146, Q148, Q149, Q153–Q159 marked answered.
**Why:** Session B's questions were blocking the Castle's key puzzle (Q153), the roof jump scene (Q155–Q157), the tomb spriggan (Q154), the ravens' caw (Q158) and the experience sound (Q159).
**Tested how:** Traced in `Mask.exe` with Ghidra (three parallel study forks plus this session); addresses and scratch notes in `re/` (untracked). Checked against the data files (frame counts, scripts).
**Result:** Main answers: the frame for phase p is floor(p × frames + 0.001), so the King's portrait at 0.99 shows its last (pushed) frame, as the user saw (our round() rule was wrong for every repeating object). Choice: forward/back picks the point nearest to a probe 200 ahead/behind him, `DoChoice` gets the triple's number. `StateChange` `$4` = old state, `$5` = new; 32 = weapon swap ended. `setScript` calls the new script's `Init` at once. `STATS.AUD` plays when the experience percent is set, not 0 and not lower than the last shown. `setLevel` sets only the number. The hammer's range back is the full way out (Swamp round trip 1.47 s, fits the user's 1.5 s).
**Still broken / not tested:** Nothing checked in the running original this time. Marked "inferred" in the notes: corpses from `loadObject` not solid; `STATS.AUD` once at every world arrival / load; a death lost in a Choice. Not traced: the level-up flame's speed and length, whether it plays at a world start, `$Testing`.
**Next:** B: build Q153 (frame rule), Q155–Q157 (roof scene), Q154, Q158, Q159. A: the remaining non-blocking questions (Q112, Q125, Q126, Q136, Q140, Q143–Q145, Q150–Q152), under way.

## 2026-10-22 Q159 (a): the user confirms STATS.AUD is the experience sound

**Changed:** Docs only: `docs/behaviour/QUESTIONS.md` Q159 marked partly answered.
**Why:** The user listened to `kq8-artifacts/wav/STATS.wav` (from `kq8 aud-verify --wav GAME/SOUND/STATS.AUD`): it is the sound the original plays when Connor gains experience.
**Tested how:** The user, by ear.
**Result:** Which file: settled. When it plays, with a level-up, and how: still open.
**Still broken / not tested:** No experience sound yet.
**Next:** A: Q159 (b)–(d); then B builds it.

## 2026-10-22 The experience sound: Q159 (not fixed)

**Changed:** Docs only: `docs/behaviour/QUESTIONS.md` Q159.
**Why:** The user: the original plays a short sound whenever Connor gains experience; ours doesn't. No note describes it and no data file names a sound for it (searched every `.cs`, `.kq`, `.anm`, `.gui`, `.snd` and `GAME/`; `GAME/SOUND/STATS.AUD` is named nowhere, so the program itself must play it). Not built from a guess.
**Tested how:** Searches only.
**Result:** —
**Still broken / not tested:** No experience sound until Q159 is answered.
**Next:** A: Q159 (with Q155–Q158).

## 2026-10-22 QUESTIONS table repaired; the ravens' caw: Q158 (not fixed)

**Changed:** Docs only: `docs/behaviour/QUESTIONS.md`: every line up to Q131 was followed by an extra empty line (since commit `7d6c385`, 2026-10-20, which rewrote the file with doubled line breaks), so the table broke after its first row; the 140 extra lines removed. Q89 (`handsOff 1|0`) and Q144 (`dither <name> 0|1`) had a bare `|` inside code that split their row; escaped. New **Q158**.
**Why:** The user: the table no longer shows correctly; the ravens should caw now and then and never do.
**Tested how:** Every table row now has 5 cells (checked by script), no empty line inside the table; the diff is only the removed lines and the two rows. The caw: `doors.log` of the `--begin` run: "object Ravens: cue 'sendEvent @ KQCueEvent 10 0' fires (cue action not done)" about every 3 s; the script command `random` is not built.
**Result:** —
**Still broken / not tested:** The table not looked at in a Markdown viewer. The ravens don't caw until Q158 is answered.
**Next:** A: Q158 (with Q155–Q157).

## 2026-10-22 Fix: the raven by Sarah's house can be clicked

**Changed:** Viewer `load.rs` (`object_colliders`): a character prepared for a script's `loadKQ` gets its click box at the start, switched off. `kq8-core` `active.rs`: `loadKQ` switches it on; the start's `Init` skips switched-off boxes. Unit test `a_character_loaded_while_playing_gets_init_then_and_can_be_clicked`. Provenance row.
**Why:** The user: with `--begin` the raven appears next to Connor but can't be clicked; in the original clicking it plays a short scene (it flies off, Connor: "Begone, raven"). The raven (`Ravens`, `ravensrh.kq`) is loaded by `stngrl.cs` during the opening, and characters loaded that way had no click box.
**Tested how:** core tests (515 pass). `kq8-viewer --world Daventry --begin --frames 5000 --sound off --no-ai --click Ravens,400`; also `--console "goto Condoor",5 --click Condoor,30 --click Ravenc,200` (the raven at Connor's house); `doors.log`, `talk.log`.
**Result:** Update 256: `loadKQ ravensrh.kq`: "Ravens can be clicked now", its `Init` then (not at the start). Update 400: the click runs `ravensrh.cs ConnorAction do`: handsOff, Connor "Begone, raven! Leave her be!" (+5 experience), turns to it, `raven.snd`, the camera placed, the raven plays `fromsarah`; update 463: Connor "Hmmm...whither does it fly?", the camera follows him, handsOff 0, the raven deleted. `Ravenc` (Connor's house) also clickable once his door loads it (that run: too far away, the "too far" remark).
**Still broken / not tested:** Not looked at or listened to. The ravens' caw cue (`sendEvent @ KQCueEvent 10 0` in their movement) is still "cue action not done", so they never caw. `Ravenc`'s own scene not run to the end.
**Next:** The user's check.

## 2026-10-22 The roof jump on the bow spriggan: Q155–Q157 (not fixed)

**Changed:** Docs only: `docs/behaviour/QUESTIONS.md` Q155 (Connor's `Choice` state), Q156 (`StateChange` messages and their state numbers), Q157 (`loadObject`).
**Why:** The user: the graveyard cutscene where Connor jumps from the roof onto the bow spriggan doesn't work. No behaviour note covers what it needs, so it is not built from guesses.
**Tested how:** `kq8-viewer --world Daventry --frames 2000 --sound off --no-ai --command "move Connor 0 228300 183200 12500 0 0 -1.57",5`; `traps.log`.
**Result:** Update 7: `roofTrap` fires, `rooftrap.cs On` runs (Connor and the camera placed, `walkstopright`). Update 30: Cue 1 reaches `KQMonster::setState Connor Choice rooftrap.cs,0,220000,183200,1,230000,183200`, which is a **stub** (nothing done), then `handsOff 0`: Connor just stands on the roof with control back; no `StartChoice` / `DoChoice` ever comes. After the choice, `sprgnbow.cs` also needs `StateChange … 32` from Connor (our notify relays only `TalkComplete`) and `loadObject` (not built).
**Still broken / not tested:** The roof scene (and Barren's basilisk, Ice World's orcs, which use the same state).
**Next:** A: Q155, Q156, Q157; then B builds the Choice state.

## 2026-10-22 Fix: the ghost knight appears when the tomb's spriggan dies

**Changed:** Viewer `load.rs` (`Placer::set_character_script`): a start-up `KQObject::setScript <name> <file>` on a character (not only an item) replaces its script. QUESTIONS **Q154** (built from the data, no note). Provenance row.
**Why:** The user: killing the bow spriggan in front of the hero's tomb never started the knight's scene. `newWorld.cs` gives that spriggan `sprigan2.cs` (and `bossprig` `bossprig.cs`) right after `loadKQ monsters.kq`; we only did this for items, so it ran its type's `sprgnbow.cs` and never sent the knight his cue.
**Tested how:** `kq8-viewer --world Daventry --frames 3000 --sound off --command "move Connor 0 226146 306500 -1 0 0 3.1416",5 --console "kill sprigan",30`; `items.log`, `doors.log`, `talk.log`.
**Result:** Load: "character script now Some("sprigan2.cs")" (and `bossprig.cs`). Update 123: `sprigan2.cs sprigan Die Normal` (map icon, +75 experience, handsOff); update 153: `knight.cs Knight Cue 10` (camera placed, `checkDistance tombdor2 … 3460 <= 3500: 1`, Connor placed at the tomb, `birthx` with `Herobrth.aud`); update 182: the knight's line (A0RS0700.061, 13.7 s, "Hark, fellow Champion…"); update 397: `TalkComplete` → `Hero::Met 1`, the camera follows Connor, handsOff 0.
**Still broken / not tested:** Not looked at or listened to. Killed with the console, not in a real fight (the same death path). The knight's status lines say "Connor 100027 away" (not looked into). `setScript` on a character while playing is still "not tracked".
**Next:** The user's check; A: Q154.

## 2026-10-22 Fix: the axe can break the hero's tomb door

**Changed:** `kq8-core` `fight.rs` (`object_at`): doors with a script are also found when Connor's swing (or a bolt) hits its target. Unit test `connor_s_hit_on_a_door_runs_die_below_0` (active.rs). Provenance row.
**Why:** The user: swinging the axe at the bolted iron door never hit it. The click made the door his target, but at the hit only characters, clickables and animated objects were looked at, so it was "dead air".
**Tested how:** core tests (514 pass). `kq8-viewer --world Daventry --frames 1500 --sound off --command "move Connor 0 226146 307900 -1 0 0 3.1416",5 --console "kill sprigan",6 --command "give SmallAx",7 --swing tombdor2,60 --aim tombdor2,120`; `fight.log`, `doors.log`.
**Result:** Update 120: the click line meets `tombdor2` (type 0x5021), 2,166 away, his target; update 127: "hits object tombdor2 … health 1 -> -24"; `tombdor2.cs` gets `Die Normal`: map icon, `open` plays (sound at its start, turns to 90°), +20 experience, `HeroDoor::Locked 0`, not clickable any more. House doors keep 10,000 health, so a swing doesn't open them.
**Still broken / not tested:** Not looked at. The target needs Connor within 2,350 of the door's own position (its hinge corner, `ranged-weapons.md` rule 32), not of its middle. With a real mouse not tried (`--aim` uses the same pick).
**Next:** The user's check.

## 2026-10-23 Animation memory 4g: the old drawing removed, measured

**Changed:** Viewer: `load.rs`: `Animation::frame_count` replaces the frames' triangles (not built any more; the materials used come from the meshes' faces). anim_view, door_view, connor_view: no frame entities or meshes; movements keep their frame counts; `connor_view::animation_materials` (was `spawn_frames`; the first frame's normals for the light log from the model); Connor's and characters' face and potion-look lists hold only the drawn (animated) materials. `anim_gpu.rs`: `Slots` without old root and mode (`root`); `DrawMode`, F5 and `--anim-draw` removed (README). `shadow_view.rs`: `FrameSources` keeps only the parts' points (the step-1 mesh path gone); a mesh on the graphics card only gives no shadow points. `main.rs`: the 2-second frame line also gives that window's slowest frame. DESIGN, STATUS.
**Why:** DESIGN "Animation memory" 4g; the user: Connor talking still slowed a little (the old drawing's 467 face materials).
**Tested how:** `cargo test --workspace`, clippy clean. Again: Daventry shadows, the Wizard and Connor talking, the mirror scene, the Lake Sword set in Dead City, the rope, Invisible, a barrel's wreck (commands as in 4b-4f). The tour of all worlds (`--world Daventry --frames 30000 --sound off`, `--console "goworld <W>",200` for each; `bench-4g-viewer.log`, `bench-4g-world.log`). Connor reading the sign against the same run without the click: `viewer.log` "last_2s".
**Result:** No graphics errors anywhere. Connor's mouth: 36 swaps on **1** face material (was 467, 1,401 at worst); Invisible: 47 materials (was ~1,080); the Lake Sword set loads in 230 ms (was 520). The tour: **Daventry in play 787 MB working set** (1.82 GB private; step 1: 1.51 GB, before: 2.64 GB); every world in play 0.77-1.0 GB; Daventry 3,565 entities (was 133,200); read 3.7 s, spawned in 0.4 s; average frame 6.71 ms (7.33 ms before step 4). Talking: slowest frame per 2 s 19.5 / 13.6 / 10.4 ms against 11.5 / 14.3 / 11.1 ms without (one 19.5 ms frame where the talk starts; otherwise the same).
**Still broken / not tested:** **The user's look (2026-10-23): no difference from before, "perfect"; Task Manager shows 840 MB in Daventry (was 2.5 GB).** The pack stays in normal memory too (~70 MB in Daventry). Reading is a little slower than before 4a (3.7 s against 3.2-3.4 s): the packing's check runs on every read.
**Next:** The user's look; then back to the game (Q153, saved games).

## 2026-10-23 Fix (4f): Connor's new drawing shares one material per look (talking stuttered)

**Changed:** Viewer `anim_gpu.rs`: `Copies` (animated copies by the material they copy, made once) passed to `Slots::spawn` / `extend`. connor_view: `group_materials` (his materials grouped by texture file and settings, one standing for each group); both slot sets and the weapon sets share the groups and copies (`ConnorSim::groups`, `copies`); `share_copies` adds each copy once.
**Why:** The user: bad stutter while Connor talks, only him. Each mouth shape sets the texture on every face material and Bevy rebuilds each one's bind group; 4f had tripled his face materials (467 old, 467 for his new parts, 467 for the rope model: 1,401).
**Tested how:** clippy clean, `cargo test --workspace`. His mouth (`--at 245000,255000,0 --click alcmsign,30 --frames 1500`), invisibility (`--at 286784,192527,1.5708 --key f3,1 --command "give Invisible 1,10" --key 8,150 --frames 2000`); `viewer.log`, `connor.log`.
**Result:** "48 materials stand for 1547"; each mouth shape on 468 face materials (467 old + 1), the same 36 swaps; invisible: 477 + 650 materials (was 1,386 + 1,854).
**Still broken / not tested:** **Not looked at**: the stutter. The old drawing's 467 remain until 4g removes them (then 1).
**Next:** The user's look at Connor talking.

## 2026-10-23 Animation memory 4f: Connor drawn the new way (F5 compares)

**Changed:** Viewer `anim_gpu.rs`: `List::Connor`; `AnimGpu::add_connor` (a weapon set loaded while playing: packed, new topology meshes, the buffers sent again; `install` split into `add_topologies` and `upload`); `Slots::extend`, `hide`, `set_offset`. connor_view: `DrawnMovement::anim`; `spawn_frames` also returns its materials by index; `spawn_connor` gives him two slot sets (him, the rope model) and moves his old frames under the old root; `share_copies` adds the new materials to his face and potion looks; `connor_draw` shows the slots (the rope model at its offset, hidden when not on the rope); `late_set_step` packs a weapon set, grows the slots and shares the copies. anim_view: his shadow reads the slots while the new drawing is shown. terrain_view: the GPU data handed over after Connor is made. DESIGN.
**Why:** DESIGN "Animation memory" 4f.
**Tested how:** `cargo test --workspace`, clippy clean. Dead City with a weapon set loaded while playing (`--world DeadCity --sound off --frames 1500 --no-ai --command "give LakeSword 1,5" --key left:IDCTL_INVBAR_SHORT,30`); the rope (`--world Daventry --sound off --frames 4000 --command "give RopeAndHook 1,5" --at 152500,207968,-1.5708 --hold 1,0,0,60 --key left:IDCTL_INVBAR_ROPE,80 --aim-point 150000,207968,30000,90`); invisibility (`--at 286784,192527,1.5708 --key f3,1 --command "give Invisible 1,10" --key 8,150`); his mouth (`--at 245000,255000,0 --click alcmsign,30 --frames 1500`). Logs: `viewer.log`, `connor.log`, `rope.log`, `characters.log`, error output.
**Result:** No graphics errors in any. Connor: 28 part entities, the rope model 28. The Lake Sword set: 43 animations packed while playing, check position 0.0080, normal 0.000026, buffers sent again in 94 ms (the set's whole load 520 ms). Invisible: 1,386 + 1,854 materials changed (his old ones and both copies). His mouth: 36 swaps as before, each on 1,401 face materials (467 x 3). His shadow: 384 model triangles.
**Still broken / not tested:** **Not looked at**: Connor walking, running, jumping, fighting, the rope, invisible, talking, a weapon swap, with F5. A weapon set loaded while playing now stops the game about 0.1 s longer (the buffers sent again).
**Next:** The user's look (4e and 4f); then 4g (the old way removed, measured).

## 2026-10-23 Animation memory 4e: copy templates and script-made copies drawn the new way

**Changed:** Viewer anim_view: every animated object gets `Slots` (also the unused copy templates, shown with H); a `Recipe` keeps the slots' inputs (`SlotMovement`s and materials) and `spawn_copy` gives a script's copy its own slots. `copies_view.rs` passes the GPU data and materials. DESIGN: items, bodies, bolts, effects, potions' orbits and still copies use still models (not animation frames), left as they are.
**Why:** DESIGN "Animation memory" 4e.
**Tested how:** `cargo test --workspace`, clippy clean. `kq8-viewer --world Daventry --sound off --at 198000,262152,1.5708 --key left:IDCTL_INVBAR_SHORT,40 --aim barrel.07,60 --frames 900` (a barrel bursts, its wreck is a copy): `viewer.log`, error output.
**Result:** Daventry: 143 of 143 animated objects drawn the new way (1,296 part entities), 34 of 34 doors and chests. The wreck: "copy NoName22 of barrelsm: drawn, 1 movement(s), 24 part(s), 12 new-way part(s)". No graphics errors.
**Still broken / not tested:** **Not looked at**: the barrel's wreck with F5; objects shown with H.
**Next:** The user's look; then 4f (Connor).

## 2026-10-23 Fix (4d), second try: each part drawn at its own centre (see-through order)

**Changed:** Viewer `anim_gpu.rs`: `Slots::show` puts each part entity at its part's ball centre (and moves its box the other way); `object_light.wesl`: the animated vertex step takes the point relative to that centre (the part's ball centre at 0); `shadow_view.rs`: `AnimSlot::offset`, a slot's shadow points taken back by it.
**Why:** The first fix (material order) didn't stop the ghost knight's staff flickering (the user's look). Found in Bevy 0.20's source (`bevy_pbr` `material.rs`, `bevy_render` `mesh/mod.rs`): see-through parts are sorted by the distance to their mesh's box centre, and a mesh with no points of its own (our topology meshes) has the centre (0, 0, 0), so every part of the knight tied at his origin and the order was left to chance. The old frames' meshes had real centres.
**Tested how:** clippy clean. Daventry `--frames 1300 --sound off --anim-draw old|new`: error output, `characters.log` (the goblin's shadow), `shadows.log`.
**Result:** No graphics errors; the goblin's shadow the same both ways (224 model triangles, 142-152 pieces, 87 building triangles); plain objects' shadows the same (9, 559 pieces). The material order (first fix) stays: harmless.
**Still broken / not tested:** **The user's look (2026-10-23): the ghost knight is perfect now.** Within one material the old drawing sorted the whole material's mesh by one centre; now each part by its own (closer to what is really in front): see-through figures could differ slightly from before in where their own parts overlap.
**Next:** The user's look at the ghost knight; if it still flickers, stop and write STATUS (two attempts).

## 2026-10-23 Fix (4d): a see-through figure's parts keep their drawing order

**Changed:** `kq8-data` `anim_pack.rs`: each packed frame's parts are put in material order, then node order (the old drawing's order: one mesh per material).
**Why:** The user's look: the keep's see-through ghost knight's staff flickered where his body was behind it, new drawing only. Our reading: see-through parts are drawn back to front without the depth test; the staff and the body are at the same distance (the knight's centre), so the order of the slots decides, and slots were filled in node order, which changes from frame to frame.
**Tested how:** `cargo test -p kq8-data`; Daventry 300 frames: the pack's check unchanged (20,851,803 corners, position 0.0080, normal 0.000026), no graphics errors.
**Result:** —
**Still broken / not tested:** **Not looked at.** Not sure Bevy settles equal distances by entity order every time; if the flicker stays, the next idea is to give each part its own distance (its node's centre), which the old drawing didn't have either.
**Next:** The user's look at the ghost knight with F5.

## 2026-10-23 Animation memory 4d: characters drawn the new way (F5 compares)

**Changed:** Viewer anim_view: characters (and those prepared for a script's loadKQ) get `Slots` like still objects (only the unused non-characters, the copy templates, stay old: 4e); the animated copies of face materials join `face`, so mouth shapes reach both drawings; a see-through copy keeps a face material itself; character shadows read the slots while the new drawing is shown. `anim_gpu.rs`: `Slots::drawn_root`; hidden slots lose `AnimSlot` (no shadow from parts a frame lacks). DESIGN.
**Why:** DESIGN "Animation memory" 4d.
**Tested how:** `cargo test --workspace`, clippy clean. Daventry and Castle Daventry `--frames 1300 --sound off --anim-draw old|new`: `viewer.log`, `characters.log` shadow lines. The Wizard speaking (`--at 189300,198274,0 --command "KQMonster::speak Wizard Connor 1000 4 0 8 1 1",30`) both ways. The mirror scene (`--world CastleD --frames 9000 --command "move Connor 0 43178 103481 130 0 0 3.1416",5 --click mirror1,280`).
**Result:** Daventry: 134 of 143 animated objects drawn the new way (63 characters), 1,222 part entities; the Castle 9 of 9. No graphics errors. Character shadows from the slots: the goblin 224 model triangles and 43 building surfaces, Connor 384, as with the old frames. The Wizard's mouth: the same codes both ways, each on 34 face materials (old and new). Lucreto loaded by the mirror script, his mouth on 6 face materials, deleted at update 507, as before.
**Still broken / not tested:** No memory saved yet (until 4g). **The user's look (2026-10-23): all as before, except the keep's see-through ghost knight: his staff flickers where his body is behind it** (fixed below, not yet looked at).
**Next:** 4e (items, bodies, effects, potions' orbits, script copies).

## 2026-10-23 Animation memory 4c: doors, chests and scripted objects drawn the new way (F5 compares)

**Changed:** Viewer `anim_gpu.rs`: 4b's per-object drawing moved into a shared `Slots` (spawn, `set_mode`, `stale`, `show`, `forget_shown`; `List` Objects / Doors; `AnimGpu::frames`); anim_view uses it. door_view: each built door animation keeps its materials by index and its model (`BuiltDoorAnim`); `spawn_body` gives doors, chests and scripted objects slots (old frames under the old root); `show_frame` sets mode and slots (the same last-frame fallback as the old frames); `door_draw` passes the GPU data and mode; a count line in `viewer.log`. DESIGN.
**Why:** DESIGN "Animation memory" 4c.
**Tested how:** `cargo test --workspace`, clippy clean. Daventry and Castle Daventry, `--frames 1300 --sound off --anim-draw old` and `new`: error output, `viewer.log`, `shadows.log`.
**Result:** Daventry: 34 of 34 doors, chests and scripted objects drawn the new way (141 part entities), plus 4b's 71 objects; Castle: 5 of 5 (9) plus 8 (41). No graphics errors. Plain objects' shadows identical in both modes (Daventry 9 shadows, 559 pieces; Castle 8, 1,389 pieces).
**Still broken / not tested:** No memory saved yet (until 4g). **The user's look (2026-10-23): everything exactly as before.**
**Next:** 4d (characters).

## 2026-10-23 Animation memory 4b: the new drawing for still animated objects (F5 compares)

**Changed:** New viewer `anim_gpu.rs` (`AnimBuffers`: points, normals, node-frames as storage buffers every object material binds; `AnimGpu::install` per world: buffer contents and one topology mesh per topology; `bounds`; `DrawMode`, F5, `--anim-draw old|new`). `object_light.rs`: `ObjectMaterial` binds the buffers (4-6), `animated` flag and `ObjectKey`: the animated pipeline takes texture places and point numbers (`ATTRIBUTE_ANIM_CORNER`), shader def `ANIMATED`. `object_light.wesl`: `model_corner` (animated: the `MeshTag`'s node-frame places the file's point, 16-bit normal, ball), the lighting unchanged. `anim_pack.rs`: `Part::node`. `load.rs` / `world_build.rs`: the pack kept with the objects (`pack_world`). anim_view: still animated objects (not characters, not unused) get `NewDraw`: the old frames under one root, part entities ("slots") under another; `show_slots` puts each part's topology mesh, animated material and tag; see-through copies for the new materials. shadow_view: `AnimSlot`, `PartQuery`; a slot's shadow from its model; the plain-object shadow key includes what slots show. README (F5), DESIGN.
**Why:** DESIGN "Animation memory" 4b.
**Tested how:** `cargo test --workspace`, clippy clean. `kq8-viewer --world Daventry --frames 1300 --sound off --anim-draw old` and `new`: error output, `viewer.log`, `shadows.log`. Castle 120 frames (the old path's shader unchanged).
**Result:** No shader or graphics errors either way. Daventry: 71 still animated objects drawn the new way, 332 part entities; the buffers 71 MB (points 17.5, normals 35.1, node-frames 5.4, topologies 13.9). Plain objects' shadows the same in both modes at frames 1, 601, 1201 (9 shadows, 559 pieces, 31 building surfaces).
**Still broken / not tested:** No memory saved yet (both drawings exist until 4g). **The user's look (2026-10-23): the water mill and the windmill's sails, F5 changes nothing at all.**
**Next:** 4c (doors and chests).

## 2026-10-23 Animation memory 4a: the compact data, checked (not drawn yet)

**Changed:** New `kq8-data` `anim_pack.rs` (`Pack::add`: points as in the file, per node-frame normals at 16 bits and the place with the vertex frame folded in, topologies per mesh/texture frame/material with shared corners; an unpack-and-compare check against `Shape::geometry`; unit tests `normals_pack_to_within_a_hair`, `folding_the_scale_and_origin_gives_the_same_point`). `shape.rs`: `Triangle::face`. Viewer `world_build.rs`: `pack_line` packs every animation of the world after reading and logs sizes and the check in `viewer.log`. DESIGN.
**Why:** DESIGN "Animation memory" 4a.
**Tested how:** `cargo test --workspace`, clippy clean; all 11 worlds `--frames 20 --sound off`, `viewer.log` "animation pack".
**Result:** Every frame of every animation in all 11 worlds unpacks to today's triangles: positions within 0.0080-0.0117 game units, normals within 0.000026, texture places and part balls exact. Daventry: 71.3 MB against 954.5 MB (13x less); Castle 16.4 MB against 217.8 MB; Ice World 66.6 MB against 873.3 MB. Packing takes 0.3-1.4 s on the reading thread (Daventry 1.4 s).
**Still broken / not tested:** Reading a world is slower by that time until the old way goes (4g). No made-up-model test: the real data of all worlds is the check.
**Next:** 4b: the shader and still animated objects, with a key to switch old/new.

## 2026-10-23 Plan: animation memory step 4 now (step 3 skipped)

**Changed:** Docs only: DESIGN "Animation memory" (step 3 skipped; step 4 planned in full: the compact data, the shader, sub-steps 4a-4g, risks), STATUS.
**Why:** The user chose to go straight to step 4 (step 3's restructuring is part of it).
**Tested how:** — (checked that Bevy 0.20 has `MeshTag` for per-entity numbers in the shader; our shaders already use storage buffers.)
**Result:** —
**Still broken / not tested:** —
**Next:** The user's go on the plan; then 4a.

## 2026-10-22 Animation memory step 1: animation frames only on the graphics card

**Changed:** Viewer `load.rs`: `FrameShape` (the parsed model and the loop's first sequence) on each `Animation`. `shadow_view.rs`: `FrameSources` (per frame mesh: its model, sequence, material; the mesh made graphics-card-only; a frame's points built again from the model with `Shape::geometry` for shadows, ~4,000 recent frames kept; counts and time logged); `mesh_triangles` / `frame_triangles` use it. `anim_view.rs`, `door_view.rs`, `connor_view.rs` (also late sets) register their frames; `terrain_view.rs` passes it; `world_build.rs`: a fresh one per world (per-world list), a line in `world.log`. DESIGN.
**Why:** DESIGN "Animation memory" step 1, the user's go.
**Tested how:** `cargo test --workspace` (all pass), clippy clean. The version before and after on the same route (`--world Daventry --frames 6000 --sound off --console "goworld CastleD",200 --console "goworld Daventry",200 ...`), `world.log`. Daventry 900 frames against a baseline: `characters.log`, `shadows.log`. All 11 worlds with 200 updates each: no panic.
**Result:** Daventry in play: 2.64 GB → 1.51 GB working set (private 4.83 → 3.70 GB); Castle Daventry 1.37 → 1.11 GB. Worlds in play now 1.5-2.4 GB (Ice World the largest). Shadows: the same triangle counts and building surfaces as before; 100 frames' points built in 900 frames of Daventry, 0.054 ms each; character shadows still 1.55 ms a frame. Reading times unchanged. Right after building, the figure is briefly high (the meshes reach the graphics card on the next frame).
**Still broken / not tested:** Graphics card memory not measured (unchanged by design). **The user's look (2026-10-23): everything works just like before.**
**Next:** Step 3.

## 2026-10-22 Plan: shadows from the compact model (step 1); shadow map noted for M7

**Changed:** Docs only: DESIGN "Animation memory" step 1 (shadows build the shown frame's triangles from the parsed model, not a positions copy) and M7 (the shadow map, which also fixes the double darkening).
**Why:** The user asked whether shadows could avoid the normal-memory copy, and about a more modern way.
**Tested how:** —
**Result:** —
**Still broken / not tested:** —
**Next:** Step 1 when the user says go.

## 2026-10-22 Plan: animation memory (DESIGN), blending counted

**Changed:** DESIGN: new "Outside the milestones: animation memory" (measurements; steps 1 and 3 now, 2 skipped, 4 after M7) and an M7 note on smooth animation. `shape.rs` `PartStats`: parts that switch mesh, part steps from one frame to the next and those that jump; the "parts count" line shows them. STATUS.
**Why:** The user chose the plan: faithful first (steps 1 and 3, drawing unchanged), frames on the graphics card later with smooth animation.
**Tested how:** All 11 worlds, `--frames 20 --sound off`; `viewer.log` "parts count".
**Result:** Parts that switch mesh at some point: 135-620 per world (about a quarter of the parts). Part steps that would jump (mesh switch or part count change, loop wrap included): 5.2-10.7 % (Daventry 8.2 %, Ice World 10.7 %); the rest can be blended. Found for step 1: shadows read the shown frame's points back from the meshes (`shadow_view::frame_triangles`).
**Still broken / not tested:** The jump count matches parts by their order in the frame; where the order changes it overcounts.
**Next:** Step 1 when the user says go.

## 2026-10-22 Parts count: moving parts would save only 12-15 %

**Changed:** `kq8-data` `shape.rs`: `PartStats`, `part_stats` (per loop: frames, part-frames, triangles stored now and as distinct part shapes, parts changing form or texture, matrix kinds), `matrix_kind` (unit test `matrix_kinds`). Viewer `load.rs`: each `Animation` carries its `part_stats`; `world_build.rs`: a "parts count" line in `viewer.log` after each world is read (objects, doors, Connor). STATUS.
**Why:** The user asked whether animations can move parts instead of storing whole frames; measured first.
**Tested how:** `cargo test -p kq8-data matrix_kinds`; `kq8-viewer --world <W> --frames 20 --sound off` for all 11 worlds; `viewer.log`.
**Result:** Moving parts would keep **85-91 %** of today's triangles (Daventry 6.95 M -> 5.98 M; Castle 1.59 M -> 1.44 M). Over half the parts (51-64 %) change form every frame: the big parts carry their own vertex positions per frame (the file stores them as 4 bytes per vertex, shared by the faces). No slanted matrices; a third or more have uneven sizes. Our meshes store 48 bytes per triangle corner (position, texture place, normal, part ball), corners not shared, so one stored vertex frame becomes roughly 50-70x larger in memory.
**Still broken / not tested:** —
**Next:** The plan changes: moving parts is not worth it. The user chooses among: no normal-memory copy of animation meshes, one entity per drawn part with the mesh swapped per frame, shared corners, or frames on the graphics card in the file's compact form.

## 2026-10-22 Memory by kind in the log

**Changed:** Viewer `world_build.rs`: after each world is built, `world.log` gets "memory by kind": meshes (count, MB, entities) for the ground, buildings, still objects, animation frames, other (Connor, items, effects, shadows) and meshes no entity uses; meshes in all; textures (count, MB). Sizes are the copies in normal memory (the graphics card holds its own). STATUS.
**Why:** The user asked why Daventry takes ~2.6 GB, and asked to measure before choosing an optimisation.
**Tested how:** `kq8-viewer --world Daventry --frames 60 --sound off`, the same with `--world CastleD`; `world.log`, `viewer.log`. clippy clean.
**Result:** Daventry (working set 2.0 GB at this point): **animation frames 56,468 meshes, 955 MB, 95,441 entities**; ground 58 meshes 95 MB; buildings 176 meshes 8 MB; still objects 147 meshes 1 MB; other 5 meshes; no entity 1,005 meshes 4 MB; meshes in all 1,063 MB; textures 3,020, 233 MB. Castle Daventry (0.6 GB): animation frames 9,990 meshes 218 MB, 10,658 entities; meshes in all 226 MB; textures 84 MB. Identical copies were already shared (each model and animation file read once); the size is every frame of every movement stored as a whole mesh, and each placed animated object spawning an entity per frame.
**Still broken / not tested:** The ~0.7 GB not in meshes or textures (engine, entities, collision, sounds) is not broken down.
**Next:** The user chooses: animation frames built when first played, or animating by moving parts.

## 2026-10-22 Purge delay on/off measured 3 times each; MIMALLOC_PURGE_DELAY respected

**Changed:** Viewer `memory.rs`: `set_allocator` leaves the purge delay alone when `MIMALLOC_PURGE_DELAY` is set (mimalloc reads it itself), so the setting can be switched without rebuilding. STATUS.
**Why:** The user asked for more consistent data comparing purge delay 0 (ours) and 1000 ms (mimalloc's default), and how to turn the setting off.
**Tested how:** `MIMALLOC_PURGE_DELAY=1000` then unset, `--world CastleD --frames 60`: `viewer.log` "purge delay 1000 ms" / "0 ms". The tour (as in the allocator comparison) 6 times, alternating off/on (`bench-purge.txt`, `bench-off-N-*.log`, `bench-on-N-*.log`).
**Result:** Read time for all 15 worlds: off 38.4 / 37.1 / 37.1 s, on 35.1 / 37.8 / 35.7 s (no real difference). Daventry on the 3rd return: off 3.2 / 3.5 / 3.5 s, on 3.4 / 3.7 / 3.3 s. Working set, average after each world built: off 2.79 / 2.69 / 2.68 GB, on 2.36 / 2.38 / 2.38 GB; highest: off 3.72 / 3.41 / 3.76 GB, on 3.11 / 3.13 / 3.11 GB. Private, average: off 4.44 / 4.49 / 4.55 GB, on 4.13 / 4.33 / 4.34 GB; highest about 5.8-6.0 GB both. The earlier one-off 7.3 s read did not come back.
**Still broken / not tested:** —
**Next:** —

## 2026-10-22 The viewer uses the mimalloc allocator (purge delay 0)

**Changed:** Workspace and viewer `Cargo.toml`: `mimalloc` 0.1.52 and `libmimalloc-sys` 0.1.49 (mimalloc 3.3.2, feature `extended`). `main.rs`: mimalloc as the global allocator; `memory.rs`: `set_allocator` (purge delay 0, first thing in `main`) and `allocator_line` (logged after each world is built). DESIGN, STATUS.
**Why:** The user's choice after the comparison: with the Windows allocator reading a world slowed with every world change.
**Tested how:** The tour (`--world Daventry --frames 30000 --sound off`, `--console "goworld <World>",30` for all 10 worlds, then Daventry ↔ CastleD twice) once each with the Windows allocator, mimalloc, and mimalloc with `MIMALLOC_PURGE_DELAY=0`; `bench-allocators.txt`, `bench-*-viewer.log`, `bench-*-world.log`. Then the built-in setting: `--world CastleD --frames 60 --sound off`, `viewer.log`. clippy clean.
**Result:** Daventry read fresh → 3rd return: Windows 3.5 → 14.6 s, mimalloc 3.1 → 3.5 s, purge delay 0 3.2 → 3.7 s (one slow 7.3 s read on the 1st return). Memory at the end: 3.3 / 5.4 GB, 3.5 / 5.4 GB, 3.0 / 5.1 GB (working set / private). `viewer.log`: "allocator: mimalloc 3.3.2, purge delay 0 ms, purge decommits 1".
**Still broken / not tested:** One run per setup. The tour wasn't rerun with the setting built in (it is the same value, read back in the log). The earlier "1 GB more" with mimalloc didn't show this time.
**Next:** Sharing identical meshes, when the user wants optimisation.

## 2026-10-22 Measurement: memory and read times over all worlds (no code change)

**Changed:** Docs only: STATUS ("Allocator": the numbers and the planned comparison).
**Why:** The user asked how much memory we use, whether it grows as the game goes on, and about allocators (wants Linux later).
**Tested how:** `kq8-viewer --world Daventry --frames 30000 --sound off` with `--console "goworld <World>",30` for DeadCity, Swamp, Gnome, Barren, IceWorld, Temple1–4, Daventry, CastleD, Daventry, CastleD, Daventry (8.5 minutes); `world.log`, `viewer.log`.
**Result:** Working set 1.8 GB fresh in Daventry, 2.0–2.8 GB in the other worlds, 3.0–3.1 GB in Daventry after the tour (private 4.6–4.9 GB), then ~40 MB more per return. Daventry's read: 3.4 s fresh, 9.4–9.8 s after the tour. What crosses worlds (journey, kept variables, gone lists, maps) is kilobytes.
**Still broken / not tested:** Long play inside one world not measured. The allocator comparison not run (the user, later).
**Next:** The allocator comparison when the user asks.

## 2026-10-22 Fix: a turn key let go during the popup no longer sticks

**Changed:** Viewer `connor_view.rs`: a Left or Right key let go while the game was paused is also released in the turn keys' own record (direction back to 0). Provenance row.
**Why:** The user: walking into a portal trap holding Left, then No: he was put back but kept turning left until Left was pressed again. The earlier fix (Forward) dropped missed keys from the movement keys only; the turn keys keep their own record, which never got the release.
**Tested how:** Build only.
**Result:** —
**Still broken / not tested:** **Not tested**: needs real keys (the test keys and `--hold` don't go through the keyboard).
**Next:** The user's check.

## 2026-10-22 End of session (B): STATUS and FEATURES brought up to date

**Changed:** Docs only: STATUS ("Where we are": today's fixes; open questions Q153 blocking, Q151, Q152), FEATURES (the popup's buttons, parent worlds' voices, the portrait).
**Why:** The user ends the session ("the highlighted buttons are good enough; a study session can answer the rest").
**Tested how:** The user, by eye and ear: the popup's buttons acceptable; Castle voices and the mirror scene work.
**Result:** —
**Still broken / not tested:** Q153 (the King's portrait) blocks the Castle's key puzzle. The allocator choice (reading slows with each world change) is still the user's to make.
**Next:** A: Q153, Q151, Q152. B: after Q153, the portrait; then saved games (kept worlds in full).

## 2026-10-22 Fix: the popup's buttons (highlight, held down, answer on release)

**Changed:** `kq8-core` `gui_draw.rs`: a stretched picture piece (`Piece::Stretched`, `Canvas::picture_stretched`; unit test `a_stretched_picture_is_not_repeated`). Viewer `gui_view.rs`: the popup's buttons have three looks (plain `yesbtn.pba` frame 0; highlighted `buyhibtn.pba`, the gold outline, stretched to the button; held down `yesbtn.pba` frame 1 with the label moved 1 right, 1 down) and answer when the mouse button or Enter is let go (the mouse: on the button it went down on). QUESTIONS Q151 (e).
**Why:** The user's look: the gold outline was gone; the frame shown was the held-down one; in the original a button goes down while the mouse button or Enter is held and acts when let go, its label moving a little.
**Tested how:** core tests (gui_draw 6 pass); `kq8-viewer --world Daventry --frames 900 --sound off --command "KQMonster::setState Connor Portal portal1 portaltp",30`; `gui.log`.
**Result:** The popup draws with `buyhibtn.pba` (88 × 25, now stretched, not repeated) and `yesbtn.pba`. 
**Still broken / not tested:** **Press and release not tested** (real mouse and keyboard only; the test keys answer at once). The stretch and the label's 1-pixel move are stand-ins (Q151 e). Not looked at.
**Next:** The user's look.

## 2026-10-22 Fix: after No, Connor stands at once

**Changed:** `kq8-core` `connor.rs` (`Walker::stand_now`), `active.rs` (`Request::ConnorStandNow`), `portal.rs` (No asks for it); viewer `door_view.rs` applies it (and clears his intent).
**Why:** The user's look: after No he was put back, the key no longer stuck, but he still played the walk's stopping movement as if Forward had just been let go; he must go straight to standing so he doesn't walk into the trap again.
**Tested how:** core 512 tests. `kq8-viewer --world Daventry --frames 2000 --sound off --command "move Connor 0 158510 307740 9925 0 0 3.1416",2 --hold 1,0,0,5,22 --key popup:no,22`; `connor.log`, `traps.log`.
**Result:** Update 23: "Connor straight to standing (stop from its start)"; update 24: at `noLoc`, movement `stop` from phase 0 (before: `walk`, then the stop transition); the trap fired once.
**Still broken / not tested:** Not looked at. With a real key (not `--hold`) not tried.
**Next:** The popup's buttons (press and release).

## 2026-10-22 King's portrait snaps back: Q153 (not fixed)

**Changed:** Docs only: `docs/behaviour/QUESTIONS.md` Q153.
**Why:** The user: in Castle Daventry the King's portrait, once pushed, snaps back, so the key behind it can't be taken.
**Tested how:** `kq8-viewer --world CastleD --frames 5000 --sound off --command "move Connor 0 15390 90700 -1 0 0 0",5 --hold 1,0,0,10,60`; `doors.log`.
**Result:** The push works as scripted (update 29: `Push`, `keys.kq` loaded, `push`, Connor's `c_pshpic`; update 74: `stop2` at phase 0.99, speed 0). `stop2` is a repeating loop of 46 frames, and both the start-frame rule (`objects.md` rule 2: round(0.99 × 46) = 46) and our play rule (rule 6, round(phase × frames) mod frames) give frame 0: the unpushed pose. The user sees it stay pushed in the original, so the notes' rule is off here; the frame rule for repeating loops was never traced. Not changed: a guess would move every repeating object's frames by half a frame.
**Still broken / not tested:** The portrait snaps back; the Castle key puzzle is blocked until Q153 is answered.
**Next:** A: Q153.

## 2026-10-22 Fix: the popup's text and buttons

**Changed:** Viewer `gui_view.rs`: the question's lines are centred across and down in the area above the buttons, 24 layout pixels apart (no longer in the layout's top slots); both buttons use `yesbtn.pba` (frame 0 plain, frame 1 highlighted); the lines' places logged in `gui.log`. QUESTIONS Q151 (d).
**Why:** The user's look: the text should be centred both ways with a bit more space between the lines; the highlighted button's picture was shorter than the button and drew wrapped. `gui.log`: `buyhibtn.pba` (the layout's BTN1 picture) is 88 × 25, `yesbtn.pba` 88 × 36 with 2 frames.
**Tested how:** `kq8-viewer --world Daventry --frames 900 --sound off --command "KQMonster::setState Connor Portal portal1 portaltp",30`; `gui.log`.
**Result:** "text placed …: [(42, 54, 276), (45, 78, 269)]": both lines centred on x 180 of the 360-wide background, the block centred in rows 0–148.
**Still broken / not tested:** Not looked at. The line pitch, the area and the highlighted frame are stand-ins (Q151 d).
**Next:** The user's look at the popup.

## 2026-10-22 Fix: a key let go during the popup stayed held

**Changed:** Viewer `connor_view.rs` (`connor_keys`): a movement key that is held in Connor's list but not down any more is dropped (and its repeat ends), logged in `connor.log`.
**Why:** The user: after No at a walking exit Connor was put back but kept walking forward into the trap again, even with Forward let go. While the question is open the game's systems (also the keys') don't run, so the key's release was never seen.
**Tested how:** Built only. The test options (`--hold`) don't use the keyboard, so this needs a real key: **not tested**.
**Result:** —
**Still broken / not tested:** Not tested. The camera keys (`Camera.sae`) may have the same problem after a pause (not checked).
**Next:** The user tries a walking exit, No, with the key let go during the question.

## 2026-10-22 Fix: a character loaded by a script can speak (the Castle's mirror)

**Changed:** `kq8-core` `active.rs`: characters prepared for a script's `loadKQ` (`dormant_monsters`) become speakers when loaded. Viewer `load.rs`, `door_view.rs`: such characters are noted at the start.
**Why:** The user: Castle Daventry's magic mirror scene never went on; Lucreto never spoke. `mirror.cs` loads `lucreto.kq` and at once has him speak (1000 33 14 52); our speech rules knew only the characters placed at the start, so the line was refused ("'Lucreto' is not KQMonster") and the scene waited for its end forever. An older gap (`talking.md` rule 6 applied to start-up objects only).
**Tested how:** core 512 tests, clippy clean. `kq8-viewer --world CastleD --frames 9000 --sound off --command "move Connor 0 43178 103481 130 0 0 3.1416",5 --click mirror1,280`; `doors.log`, `talk.log`.
**Result:** "Lucreto is a character: it can speak now"; his three lines play (mouth shapes), `TalkComplete … 3` → "Die Luke Die!!!!!", Lucreto deleted, +25 experience (level 2), the mirror's `transback`.
**Still broken / not tested:** Not looked at or heard. Lucreto's goodbye / welcome-back lines (conversations) not needed here.
**Next:** The other points of the user's list.

## 2026-10-22 Fix: worlds with a parent use its voices and sounds

**Changed:** Viewer `world_build.rs`: `parent_world` (`parentWorld` in `GAME/Kq/<World>.kq`); message files, voices and sound effects are looked up in the parent world's folders (Castle Daventry → Daventry, Snow exit → Ice world, Temples 2–4 → Temple 1).
**Why:** The user: in Castle Daventry most of Connor's lines showed their caption and ducked the music, but no voice was heard. Castle's own `English` folder holds only `1000.MSG` (the same file as Daventry's) and its `Sound` folder is empty; `mask.cs` takes `VolumeText` and `VolumeSound` from `$KQWorld::Parent` (`world-change.md` rule 7.5). Our build looked only in the world's own folder and `GAME`. An older gap: launching these worlds directly had it too.
**Tested how:** `kq8-viewer --world CastleD --begin --frames 600 --sound off`; `viewer.log`, `talk.log`.
**Result:** "world CastleD: texts, voices and sounds from its parent world Daventry"; Castle's `Begin` line 1000 79 0 10 plays with "voice daventry/English/A0RS2700.0A1 (4.302 s)" (before: no voice).
**Still broken / not tested:** Not heard. Snow exit and Temples 2–4 not run. The folders are found by name through the game folder, which also matches the install script's unpacked copies (`MASK/daventry/…`); unchanged.
**Next:** The user's listen in Castle Daventry.

## 2026-10-22 6e-8: kept worlds, first stage (6e done)

**Changed:** `kq8-core` `active.rs`: `gone_names` (deleted objects, items Connor carries or used up, dead or gone characters). Viewer `world_build.rs`: `KeptWorld` (the world's variables, its gone names) kept when Connor leaves, its map's saved keys put in the journey's other maps; arriving there again hands them to the reading (`load::Arrival`); Temple 1 drops the kept worlds. `load.rs`: kept variables set over the start values; gone objects not placed. `map_view.rs`: a world's own kept map replaces its shipped file; kept maps are not overwritten by shipped ones. README, FEATURES (new section), STATUS, DESIGN (6e done), provenance.
**Why:** M6 6e step 8 (`world-change.md` rules 8.1, 12, 13, 14, 16), the last step of 6e.
**Tested how:** clippy clean; core 512, data 138 tests. `kq8-viewer --world Daventry --begin --frames 5000 --sound off --command "give MagicMap",5 --command "deleteObject wizbook",45 --console "goworld CastleD",50 --console "goworld Daventry",60`; the same without `--begin`; `--world Daventry --command "give MagicMap",5 --console "goworld temple1",30`. `viewer.log`, `doors.log`, `map.log`.
**Result:** Leaving Daventry: "91 variables, 7 objects gone, its map". Back: "91 variables set over the start values; 1 objects that left it not loaded again: [wizbook]"; "davmap.kq: the map kept when Connor left Daventry"; `Begin`: first visit `FirstTime == 1` (the opening scene), return `FirstTime == 0` (no scene). Daventry's fog 1,696 pixels out on leaving, 2,078 on return (1,696 + 382 new). Without `--begin` the return still had `FirstTime == 1`: the scene had never played, so that is right. To Temple 1: "1 kept worlds dropped, the maps kept".
**Still broken / not tested:** Not looked at. What a kept world does not keep yet (positions, chests opened without a variable, items dropped there, bodies, terrain changes, the clock) comes with saved games. Items made by start scripts (`NoName`) that Connor took come back under a new name.
**Next:** The user plays world changes by eye; decides on the allocator; then saved games.

## 2026-10-22 6e-7: walking exits

**Changed:** `kq8-core` `portal.rs`: Connor's `stop` is no longer asked for as a Special movement (start of the state and after No). Viewer `load.rs`: the movements of portals' own lists are loaded (this world's portals' `enterAnmFile`/`enterAnim`; the arriving portal's `exitAnmFile`/`exitAnim`, handed over in `Arrival`); `world_build.rs` passes it. DESIGN, provenance.
**Why:** M6 6e step 7 (`world-change.md` rule 1's table, rules 3, 4, 20.3).
**Tested how:** clippy clean, core 512 tests. Daventry: `--command "move Connor 0 158510 307740 9925 0 0 3.1416",2 --hold 1,0,0,5,30` (walk north into `portaltp`) with `--key popup:no,22 --hold 1,0,0,40,90`; then with `--key popup:yes,22 --console "move Connor 3 0 0 0 0 0 -1.5708",30 --hold 1,0,0,40,140 --key popup:yes,40` (to Castle and back). Barren: `--command "KQMonster::setState Connor Portal portal6",30 --key popup:yes,30`. `traps.log`, `viewer.log`.
**Result:** Walking in: `portaltp` On at update 22 → `portaltp.cs` → the question. No: `noLoc`, Off at 23; walking again: On at 31, the question again. First try he could not walk after No: his `stop` was played as a Special movement, which repeats and never ends (fixed). Yes: Castle Daventry; walking west into `portalt2` (On at Castle update 80) → Yes → Daventry. Barren elevator: `elevator.anm/c_goup`, Ice world `elevat2.anm/c_comnup`, `eleport.cs portal6 Exit`. A first `--at` placed Connor under the cave (the ground, not the path's floor): placed by `move` instead.
**Still broken / not tested:** Not looked at. Back in Daventry the opening scene replays (`Begin` with `FirstTime` 1) and moves Connor away from the portal's place: 6e-8. Dead City, Swamp, Gnome, Snow exit and temple exits not walked (same rules). The Gnome pool and the snail not run.
**Next:** 6e-8, kept worlds (first stage).

## 2026-10-22 6e-6c: teleporting works end to end

**Changed:** `kq8-core` `active.rs`: `loadKQ` of a `GAME/Kq` portal file while playing (`load_portal`, `set_portal_files`). Viewer `load.rs`: the `KQPortal` files of `GAME/Kq` read at the start; `door_view.rs` hands them to the rules; `gui_view.rs`: `gui_step` leaves `popup:` test keys to the popup step (it dropped them). DESIGN, provenance.
**Why:** M6 6e step 6, third part (`magic-map.md` rule 39): the teleport scripts load the shown world's portal file at their pad before the Portal state.
**Tested how:** clippy clean, core 512 tests. `kq8-viewer --world Daventry --at 217338,283698,3.1416 --kept-maps MASK/save/save18.vol --frames 4000 --sound off --command "give MagicMap",5 --command "earnTeleport barrMap.kq",8 --key tab,10 --key left:IDCTL_MAPWIN_RIGHT,20 --key left:IDCTL_MAPWIN_RIGHT,30 --key left:IDCTL_MAPWIN_TELEPORT,40 --key popup:yes,40 --console where,80`.
**Result:** "portal files in GAME/Kq: Davtele, Deadtele, Icetele, Swamtele, bartele, gnomtele". Go → `Davtele.cs` → "loadKQ bartele.kq: the portal NoName23 to Barren.kq" → the question → Yes → `teleportdown` → hand-over → Barren: Connor at (183,000, 95,500) heading −2.0, camera at (182,700, 90,500, 3,400) → `teleportup` → control back at update 23. Found on the way: `--key popup:yes` at update 41 never fired (the game paused at 40), and at 40 the panels' step had eaten it.
**Still broken / not tested:** Not looked at. Connor arrives at z 2,000 and drops to 1,277 (the portal's z is above the ground there; not compared with the original).
**Next:** 6e-7, walking exits.

## 2026-10-22 6e-6b: Yes leads to the world change

**Changed:** Viewer `portal_view.rs`: `leave_step` (after Yes: waits for the `enterAnim` to end, then asks for the world change with the portal's places), `world_of` (a portal's `world` file to our world name). `main.rs`: the system. DESIGN, provenance.
**Why:** M6 6e step 6, second part (`world-change.md` rules 4–6).
**Tested how:** clippy clean. `kq8-viewer --world Daventry --frames 3000 --sound off --command "KQMonster::setState Connor Portal davgate4 sdwtrap1",30 --key popup:yes,31`; the same with `portal1 portaltp` and `--key popup:left,31 --key popup:enter,31` (and once with only `popup:enter`). `viewer.log`, `doors.log`, `connor.log`.
**Result:** davgate4: Yes → `Accept` → `handsOff 1` → Connor plays `portal` → "the hand-over: to DeadCity" → loading screen → Dead City, `portalx`, `shadbat.cs davgate4 Exit`, `handsOff 0`. portal1: Enter with No highlighted answers No (`Decline`, `noLoc`); Left then Enter answers Yes → Castle Daventry at once (no `enterAnim`), Connor at (9,000, 111,805), `out`, control back at update 8.
**Still broken / not tested:** Not looked at. The full-armour list swap (rule 5.2) not built. Test keys after the answering one in the same update are dropped.
**Next:** 6e-6c, the teleport scripts' portal files (`loadKQ … @` of a `GAME/Kq` file while playing).

## 2026-10-22 6e-6a: the Portal state's question (popup, pause, No)

**Changed:** New `kq8-core` `portal.rs` (`PortalState`, the start of the state, `portal_answer`, `portal_entered`, `popup_lines`; 3 unit tests); `active.rs`: `KQMonster::setState Connor Portal` now starts it, the world's portals' keys (`portals`). Viewer: the loader collects every placed `KQPortal`'s keys; `gui_view.rs`: the popup panel drawn (eighth drawn picture), `popup_step` (opens it, keys, clicks, `--key popup:…` test keys); `world_build::running` (the game's updates and the world's drawing stop while it is open; the panels and sound go on). An old map test updated (its "not built" line). QUESTIONS Q151 (c). DESIGN (6e-6 in three commits), provenance.
**Why:** M6 6e step 6, first part (`world-change.md` rules 1–3, 26; `magic-map.md` rule 40).
**Tested how:** core 512 tests, clippy clean. `kq8-viewer --world Daventry --frames 900 --sound off --command "KQMonster::setState Connor Portal portal1 portaltp",30 --key popup:left,31 --key popup:no,31 --console where,40`; and the same without answering, 700 frames. `viewer.log`, `gui.log`, `connor.log`.
**Result:** "the Portal state's question opens: [\"You are about to load a new region.\", \"Are you sure you want to do that?\"], buttons \"Yes\" (left) and \"No\" (right)". Unanswered for 700 frames: the game stayed at update 31; the popup slid in in 0.2 s, drawn 463 × 250. No: `portal1` gets `Decline` (its script ran), Connor moved to `noLoc` (158,510, 307,740), play goes on.
**Still broken / not tested:** **Not looked at** (the popup's look, the buttons' places, the highlight). Yes leaves Connor waiting for the hand-over (6e-6b). A click on the bottom bar while the popup is open is queued for after it. Real keys and mouse clicks not tried (only test keys).
**Next:** 6e-6b, Yes: the enter movement and the world change.

## 2026-10-22 6e-5: `world.cs Begin` on every arrival

**Changed:** Viewer `world_build.rs` (`BeginNow`, `begin_step`: `world.cs <World> Begin` once on a new world's first update, before the portal arrival), `main.rs` (launch option `--begin`). FEATURES, DESIGN, provenance.
**Why:** M6 6e step 5 (`world-change.md` rule 9); our build never ran `Begin`.
**Tested how:** clippy clean. `kq8-viewer --world Swamp --frames 6000 --sound off --console "goworld Daventry",30 --console "goworld Swamp",60 --console "goworld Daventry",60`; `kq8-viewer --world Daventry --begin --frames 900 --sound off`. `viewer.log`, `doors.log`.
**Result:** "world.cs Daventry Begin … run" / "world.cs Swamp Begin … run" on each arrival. Daventry's `Begin` finds `Daventry::girlhut::FirstTime == 1` both times and plays the opening scene (`handsOff 1`, `move Connor 0 277744 199029 -1 0 0 1.57`, the camera move, the cue to `stngrl`): the expected gap until its variables are kept (6e-8). `--begin` starts the same scene at launch.
**Still broken / not tested:** Not looked at; whether the opening scene runs to its end in our build not checked. Launch without `--begin` stays as before (no `Begin`).
**Next:** 6e-6, the Yes/No popup and the Portal state.

## 2026-10-22 6e-4b: arriving through a portal

**Changed:** New viewer `portal_view.rs` (`PortalArrival` from a `KQPortal` file, `read_portal`, `arrival_step`: places, `handsOff 1`, the `exitAnim`, then the `exitScript` with `<portal> Exit` and `handsOff 0`). `world_build.rs`: a change request can carry a portal; z −1 goes through the ground placement. `kq8-core` `Active::run_script`. `load.rs`: six portal movements added to Connor's loaded movements. Console word `goworld portal <file>`. FEATURES, DESIGN, provenance.
**Why:** M6 6e step 4, second half (`world-change.md` rules 6, 20).
**Tested how:** clippy clean; core 509, viewer 12 tests. `kq8-viewer --world Swamp --frames 3000 --sound off --console "goworld portal Davtele.kq",30 --console where,120`; `kq8-viewer --world Daventry --frames 3500 --sound off --console "goworld portal davgate4.kq",30 --console where,150`. `viewer.log`, `doors.log`, `connor.log`.
**Result:** Davtele: Connor at (217,338, 283,698, 18,931) on the keep's pad, the camera at (217,713, 279,768, 20,330) tilted −0.2; `teleportup` plays (22 frames, `poofup.aud` cue); at update 23 control comes back. First run found `teleportup` "not loaded from his own list": our list of Connor's movements didn't have the portal ones; added. davgate4: Dead City, on the ground at (182,262, 45,000, 864) heading 3.14; `portalx` 32 updates; `shadbat.cs davgate4 Exit` runs (it starts the shadow-bat scene; `ShadBat` itself is not built yet).
**Still broken / not tested:** **Not looked at.** The elevator, pool and snail portals' movements (`Elevator.anm`, `elevat2.anm`, `Snail.anm`, …) are not loaded: those arrivals play nothing yet (6e-7). The full-armour list swap (rule 5.2) is not built. The arrival's camera then follows Connor as usual (not checked against the original).
**Next:** 6e-5, `world.cs Begin`.

## 2026-10-22 Loading screen kept at 4:3

**Changed:** Viewer `loading_screen.rs`: the 640 × 480 layout sits in the largest 4:3 box that fits the window, centred, black around it; the box's size is logged when the screen closes (`world_build.rs`).
**Why:** The user: the meter now works like the original, but the pictures were stretched to the 16:9 window.
**Tested how:** `kq8-viewer --world Daventry --frames 2500 --sound off --console "goworld Swamp",50`; `viewer.log`.
**Result:** "loading screen closed: layout box Some(Vec2(960.0, 720.0)) in window Some((1280, 720))": 4:3, 160 pixels black on each side.
**Still broken / not tested:** Not looked at. A window narrower than 4:3 (taller than wide) not tried.
**Next:** The user's look; then 6e-4b.

## 2026-10-22 Fix 6e-4a again: the meter's frame is in the picture; no black bar

**Changed:** Viewer `loading_screen.rs`: the black bar is no longer drawn; the meter is `4red.pbm` stretched over (5, 468)–(635, 476) (`NodeImageMode::Stretch`). `QUESTIONS.md` Q152.
**Why:** The user's second look: the red rectangle had no background; in the original the red texture grows inside a meter frame.
**Tested how:** Measured the loading pictures (`daventry.pbm` in `ldaventr.ppl`, row by row): rows 464–479 are a light border round a dark slot exactly at the meter's place (x 5–635, rows 468–475); `Swamp`, `castleD`, `deadcity`, `temple1`, `barren` the same. `Loading.gui` (from `kq8 gui-verify`) has no frame picture of its own. One run `--world Daventry --console "goworld Swamp",50`.
**Result:** The frame the user saw in the first build was the picture's own: my 640 × 18 black bar covers it (in the first build the bar showed as a small square for the same reason as the meter: Bevy kept the 6 × 6 pictures at their own size; the default `Auto` image mode). Now "meter 4red.pbm 6x6 stretched to 630x8; no black bar (Q152)".
**Mistake:** In the first fix I read "the bar background is correctly drawn" as my black bar and drew it bigger, which hid the real frame. I should have measured the picture first.
**Still broken / not tested:** Not looked at. When the original shows the black bar, and stretch or tile for the meter: Q152.
**Next:** The user's look; then 6e-4b.

## 2026-10-22 Fix 6e-4a: the loading meter; `--console` lines after a world change

**Changed:** Viewer `loading_screen.rs`: the black bar and the red meter are rectangles in the colour of `4black.pbm` / `4red.pbm` (6 × 6 swatches), not stretched pictures. `console.rs`: once a line asks for a world change, the lines after it (typed or `--console`) wait for the next world. Provenance unchanged (same rules).
**Why:** The user's look: (1) with `--console "goworld Swamp",100 --console "goworld Daventry",100` Swamp never loaded (both lines ran in the same update; the second replaced the first — my test command, though I had noted the pitfall); (2) the red meter showed as a small square moving from outside the bar to the middle of the screen, with a black box there that the original doesn't have; the bar itself was right.
**Tested how:** The same command again, 4,000 frames; `viewer.log`.
**Result:** Daventry → Swamp at update 100, then Swamp → Daventry at Swamp's update 100. "bar [0, 0, 0, 255], meter [208, 0, 0, 255]"; meter steps as before.
**Mistake:** I gave the user a test command with two world changes at the same update.
**Still broken / not tested:** The new meter is **not looked at**. Why the stretched picture drew as a moving square was not worked out (no longer used).
**Next:** The user's look at the meter; then 6e-4b.

## 2026-10-22 6e-4a: the loading screen, the world read on another thread

**Changed:** New viewer `loading_screen.rs` (the world's loading picture in its `L<7 letters>.ppl` palette, the black bar, the red meter; opened at the take-down, closed when the world is built). `world_build.rs`: the next world is read on a thread (`Step::Loading`), the meter fed from the scripts' `setLoadProgress`, frame times logged; the sound system is made on the main thread. `kq8-data` `world_script.rs`: `Host::load_progress`. `load.rs`: the progress counter. `main.rs`: `frame_stats` skips its log line while the reading thread holds the log. DESIGN (6e-4 split in 4a / 4b), provenance.
**Why:** M6 6e step 4, first half (`world-change.md` rules 7.2, 22–25).
**Tested how:** clippy clean; viewer 12 tests (new `file_names_follow_rule_22`), data 138. `kq8-viewer --world Daventry --frames 4000 --sound off --console "goworld Swamp",40 --console "goworld Daventry",60`; `viewer.log`.
**Result:** Swamp: "loading screen: GAME/8Gui/Swamp.pbm in GAME/8Gui/lSwamp.ppl …", 558 frames in 3,100 ms, longest frame 7.9 ms (the window no longer freezes while reading); meter 0 → 20 → 60 → 100 %. Back to Daventry: 1,167 frames in 6,484 ms, longest 8.1 ms; meter 13 % at once, then 18 … 100 % with the object scripts. Counts after the rebuild as before (132,897 entities).
**Still broken / not tested:** **Not looked at** (the picture's look and the meter's place are from the layout numbers only). Building the world after the read still stops the window for 0.5–0.9 s (spawning). The text line of `Loading.gui` is not drawn. Reading is slower on the thread while the window draws (Daventry 6.5 s here).
**Next:** 6e-4b, the arrival (portal places, `exitAnim`, `exitScript Exit`).

## 2026-10-22 6e-3: the journey (`goworld`)

**Changed:** New `kq8-core` `journey.rs` (`Journey::take` / `give` / `describe`, `connor_summary`; 2 unit tests). `active.rs`: every `setConnorFlag` flag kept (`connor_flags`). `item.rs`: the `NoName` counter can be read and moved on. `kq8-data` `Vars::iter`. Viewer: `load_objects` / `read_world` take an `Arrival` (counter carried, `conStart.cs` not run); `world_build.rs` takes the journey before the take-down and gives it after the build, `--item` only in the first world, the combat level before the characters enter; new `kq8-artifacts/logs/world.log`; console word `goworld <World> [x y heading]`. FEATURES, DESIGN, provenance.
**Why:** M6 6e step 3 (`world-change.md` rules 13, 17–19, 21).
**Tested how:** `cargo test --workspace` (core 509, data 138, viewer 11), clippy clean. `kq8-viewer --world Daventry --at 193000,193000,3.1416 --frames 1400 --sound off --command "give MagicMap",5 --command "give Strength 2",6 --command "give Dagger",7 --key 6,20 --hold 1,0,0,25,90 --console "goworld Swamp",100`; and `--world Daventry --frames 1600 --sound off --command "give MagicMap",5 --console "goworld Swamp 150000 80000 1.0",50 --console "goworld daventry",100`. `world.log`, `items.log`, `map.log`, `viewer.log`.
**Result:** `world.log` "Connor leaving Daventry" and "Connor arriving in Swamp" are the same line (level, stats, 7 items with their applied marks, the Strength potion). The potion left Daventry at 15 s and counts on in Swamp (11 s at Swamp update 54). Swamp's own start items for Connor (`Mask1a`, `Fist1`, …) are replaced; the `NoName` counter goes on from 25. Swamp's map switches on with the carried `MagicMap` (index 2 visited). Placed at the asked spot (150,000, 80,000, ground 5,235, heading 1.0). Back in Daventry: 132,897 entities as after a reload.
**Still broken / not tested:** Not looked at. A Clarity potion running on arrival does not reveal the new world's hidden objects (the note doesn't say whether it should). `goworld` lets a dead Connor travel (seen: health −3 carried from a Swamp spot). Meshes after a Daventry rebuild differ by a few dozen between runs (57,651 / 57,683; not growing within a run). The weapon in hand on arrival: built (takes its stance at once) but not run.
**Next:** 6e-4, the loading screen and the arrival.

## 2026-10-22 6e-2: taking a world down (`reload`)

**Changed:** Viewer `world_build.rs`: `WorldChange` (a change asked for, the current world, a count of worlds built), `world_change_step` at the end of each frame (take down: despawn every `WorldEntity`, remove the 50 per-world resources; wait 2 frames; read and build the next world), `playing` (the game's systems run only while a world is in), per-part read timings. New `memory.rs` (working set and private memory, Windows). `console.rs`: console word `reload`; test option `--console "WORDS",N`. `log.rs`: the world change line in every log. Systems that keep values between frames start again with a new world (`anim_view`, `draw_distance`, `gui_view` ×2, `potion_view`, `shadow_view`, `talk_view`, `terrain_view`). `main.rs`: the run condition on the game's systems. FEATURES, DESIGN, provenance.
**Why:** M6 6e step 2: one world must come down cleanly before another is built.
**Tested how:** clippy clean, viewer tests (11) pass. `kq8-viewer --world Daventry --frames 2500 --sound off --console reload,40 --console reload,80 --console reload,120`; `--world Swamp --frames 1500 --sound off --console reload,40 --console reload,80`; `--world Swamp --frames 1200 --sound off --hold 1,0,0,0,40 --console where,45 --console reload,50 --console where,60`. `viewer.log` "world change" / "world built" lines.
**Result:** Daventry, three reloads: after each take-down 448 entities (0 world), 15 meshes, 12 images, 0 world materials, 0 of 50 resources; after each rebuild 132,897 entities, 57,651 meshes, 3,009 images, the same materials. Swamp, two reloads: 35 meshes / 150,456 entities each time. Private memory after take-down 2,541 → 2,905 → 2,961 MB (levelling off). Connor comes back on the ground where he stood (Swamp 176,863, 84,355, heading −1.70). Every log has its `== world change: Swamp -> Swamp (console reload) ==` line.
**Found and fixed on the way:** (1) 9 meshes kept per reload: the shadow systems kept their old shadows (fixed by starting their kept values again). (2) `--console` lines were ignored once the terminal input was closed (as in piped test runs). (3) Three `--console` lines at the same update all fire in the first world.
**Dead end / not fixed:** Reading gets slower each reload (Daventry objects 3.4 → 4.8 → 5.4 → 5.9 s). Not our code: with the `mimalloc` allocator (tried, then removed) it stays at 3.1 s, but holds about 1 GB more memory (private 4.5–5.0 GB after take-down instead of 2.5–3.0). A choice for the user; nothing changed.
**Still broken / not tested:** Not looked at (the window should show the same world again after a 4–6 s freeze; the freeze goes away with the background loading of 6e-4). Typing `reload` by hand not tested (only `--console`). 15–35 meshes stay after a take-down (not growing; who holds them not found). F7/F8/F9 go back to the launch settings after a reload. `--hold` counts updates per world, so a held walk plays again in the new world.
**Next:** 6e-3, the journey (Connor's things carried; `goworld`).

## 2026-10-22 6e-1: one way to build a world (no behaviour change)

**Changed:** New viewer `world_build.rs` (`read_world`: every file a world needs, as `main` read it; `insert_world`: its resources; the `BuildWorld` schedule: the start-up systems that were `Startup`; `WorldEntity` put on every entity spawned with a `Transform` or `Node`; the 50 per-world resources listed; entity and asset counts). `main.rs` uses it; `log.rs`: a log is emptied on its first open in a run, added to after that. DESIGN, provenance.
**Why:** M6 6e step 1: building a world must be something the running program can do again (world changes, 6e-2 on).
**Tested how:** `cargo clippy --release -p kq8-viewer` clean, `cargo test --release -p kq8-viewer` (11 pass). The previous commit and this one, each `kq8-viewer --world Daventry --frames 300 --sound off` and `--world Swamp`; every log's common part compared, and the new build run twice to see what changes between runs anyway.
**Result:** All logs the same except lines tied to real time (sound times, the river animation's timing, shadow cost per frame), which differ just as much between two runs of the same build. `viewer.log` has the same lines; the panel, map and sound reading now come before "loading took" (it includes them), and a new line: "world built: Daventry: read 3492 ms, spawned in 637 ms; entities 132882 (132449 marked WorldEntity); meshes 57640, images 3005, materials: object 4281, standard 3, ground 206, building 376; per-world resources 50 of 50" (same counts in a second run). Load 3,444 → 3,493 ms (+1.4 %).
**Mistake:** To get clean logs for the comparison I deleted every `kq8-artifacts/logs/*.log`, which also removed about 100 older named logs from earlier steps (e.g. `camera-3m4-watermill.log`, `map-verify.log`). They were test output (not in git) and can be made again, but I should have looked first. The `run-*.txt` files are still there.
**Still broken / not tested:** Not looked at (no visible change intended). 433 entities are not marked (Bevy's own, e.g. the window and observers; not checked one by one: 6e-2 lists them). Nothing builds a second world yet.
**Next:** 6e-2, taking a world down (`reload`).

## 2026-10-22 Plan 6e updated from world-change.md

**Changed:** Docs only: `docs/DESIGN.md` "6e" (rewritten against `world-change.md`), `docs/behaviour/QUESTIONS.md` (new Q151).
**Why:** The 6e plan was written before the study; session B checks it against the note before building.
**Tested how:** Read `world-change.md`, `magic-map.md` rule 40, the viewer's start-up and `WorldState`; the portal files (`Davtele.kq`, Daventry `portal1.kq`, `portaltp.cs`), Daventry `world.cs` and the `Init` parts of its scripts (data, read only). Measured one world's memory: `target\release\kq8-viewer.exe --world Daventry --frames 400 --sound off`, polled from PowerShell.
**Result:** The plan's architecture stays (same program, marked world entities, one build function, the journey record, kept worlds in stages). Daventry takes **3.4 GB** peak working set (6.1 GB private), so keeping left worlds alive in memory (about 25 GB for 8) is out; full kept worlds stay with saved games. The first stage now keeps more than the maps: the world's **variables** (rule 13; Daventry's `Begin` would otherwise replay the opening scene, and `bricks.cs`, `milltree.cs`, `urn1.cs` set themselves up from them) and a **"gone" list** (taken or deleted objects not loaded again). New step `world.cs Begin` (our build never ran it; at launch only with `--begin`). No `conStart.cs` on a world change (rule 21). Steps are now 6e-1 to 6e-8. Q151: the popup's first highlighted button (stand-in No).
**Still broken / not tested:** Nothing built.
**Next:** 6e-1, one way to build a world.

## 2026-10-22 world-change.md: the user's check in the original

**Changed:** Docs only: `docs/behaviour/world-change.md` (rules 19, 23, 25, 26, Edge cases, Do not reproduce 2, a "user's check" section, open questions), STATUS.
**Why:** The user ran "How to check" 1–10 in the original (patched GOG install).
**Tested how:** The user, by eye. For which loading picture is which: the `GAME/8Gui` pictures compared byte by byte (the `<5 letters>2.pbm` set differ from each other in 15–22 % of pixels, the `<world>.pbm` set in 45–68 %).
**Result:** All as described except **Yes is left, No right** (the first version had the button places the wrong way round). The popup's keys: Left/Right move the highlight, Enter chooses, Escape = No + main menu, Space still toggles the bottom bar. Loading has two phases: CD copy (`loading2.pbm`, the universe overview with the world highlighted, green meter; very short with the patch), then the world (`loading.pbm`, red meter); keys do nothing. Everything in a world is kept (broken barrels, bodies on the ground, dropped items, doors, levers, rocks, puzzles). Health, level, items, weapon and armour, the weapon in hand and running potions with their timers carry over.
**Still broken / not tested:** The popup's first highlighted button; the frozen world clock (no timed event found to compare).
**Next:** B: 6e from `world-change.md`.

## 2026-10-21 Study 6e-0: the world change and loading (session A)

**Changed:** Docs only: new `docs/behaviour/world-change.md`; `magic-map.md` (rule 34's kept-map lookup now traced, rule 40 step 4 corrected and step 5 traced, open questions 4 and 5 answered); STATUS. Scratch notes (addresses) in `re/notes.md`, not committed.
**Why:** DESIGN "6e" step 0: the six unknowns (arrival, kept worlds, the journey, the loading screen, walking exits, the Yes/No popup) and `magic-map.md` open questions 4–5, before B builds moving between worlds.
**Tested how:** Ghidra on `Mask.exe` (the portal hand-over, the Portal state's answer and animation-end handling, the world save / unload / restore, the variables written per world, the generated start script, the loading screen meter, the popup) and the data: all 27 portal files, every script with `setState Connor Portal`, `MASK/mask.cs`, the worlds' `world.cs` / `newworld.cs` / `permwrld.cs`, and the user's `MASK/cur/` and `MASK/save/save10.vol` read with a scratch script (read only).
**Result:** A world change saves the left world like a saved game (`cur/<World>.vol`: objects, `WorldVar.cs` with only that world's and `KQWorld` variables, a `newWorld.cs` beginning `setCurrentTime`), saves Connor whole to `cur/Connor.kq` (stats, items, flags, map marks, his Portal state) plus `GameState.kq` (unnamed-object counter, difficulty) and the map to `cur/<map>.kq`, then loads the new world with `cur` first in the file search. `KQPortal::ConLoc` / `CamLoc` replace the place on the first load of Connor and the camera. Every walking exit is started by a script. Arriving in Temple 1 deletes all kept worlds. Popup: 38-character lines, No left / Yes right, game paused but sounds go on.
**Still broken / not tested:** Not checked in the original by the user yet ("How to check" 1–10). Not settled: which loading picture shows when and the meter's colours, the popup's keys, where potion timers and the rope state sit in Connor's saved form, whether left-world variables are cleared from memory.
**Next:** The user's check in the original; B: 6e-1 to 6e-7 from `world-change.md`.

## 2026-10-21 STATUS: Swamp's start-up equipment checked (stale line removed)

**Changed:** Docs only: STATUS ("Found in 6b, not fixed" removed).
**Why:** The line said Swamp's `conStart.cs` chain mail, boots and gloves were carried but not put on; L-1 (the start scripts' items given by the rule of a `give`) should have fixed it.
**Tested how:** `cargo play --world Swamp --frames 400 --sound off`; `items.log`, `gui.log`.
**Result:** Fixed since L-1: "apply NoName52 (INVITEM_ChainMailShirt, put on) … [0, 0, 0, 3, 0] added", boots +1, gloves +1 (armour 5 before the stats event); the screen opens with armour 21 (16 from the level-15 event + 5), the armour slot shows the chain shirt (`2armr2.pbm`).
**Still broken / not tested:** —
**Next:** —

## 2026-10-21 Plan 6e: moving between worlds

**Changed:** Docs only: `docs/DESIGN.md` "6e" (plan), STATUS.
**Why:** The user: the teleport needs a different world loaded while the game runs; plan it for a new session B.
**Tested how:** Not applicable (a plan). Read the viewer's start-up (`main.rs`: one world read before the window opens, ~60 resources, 4 groups of start-up systems, 18 logs emptied when opened), the notes that touch a world change (`magic-map.md` rule 40, `interface.md` rules 1, 1d, 19, `sound.md` 55, 57j, `potions.md`, `objects.md` 1, 5), and the data (21 walking-exit portals in 11 worlds, e.g. Daventry `portal1.kq` to Castle Daventry).
**Result:** Plan: in the same program (not a restart); world entities marked and taken down; one function builds a world (launch included), loading on a background thread behind the loading screen; Connor's "journey" as one record (also for saved games later); kept worlds first only their maps, in full later with saved games. Steps 6e-0 (A: study the world change, six unknowns) and 6e-1 to 6e-7 (B), each with a test.
**Still broken / not tested:** Nothing built.
**Next:** A: 6e-0. B: 6e-1 to 6e-3 (they don't need 6e-0).

## 2026-10-21 Magic map: the user's check in our build

**Changed:** Docs only: FEATURES, STATUS.
**Why:** The user played the magic map built in 6d-1 to 6d-7.
**Tested how:** The user, by eye, in our build.
**Result:** "The map works as expected"; no bugs noticed.
**Still broken / not tested:** The Portal state and the world change (not built).
**Next:** Plan moving between worlds (DESIGN "6e").

## 2026-10-21 6d-7: teleporting up to the world change

**Changed:** `kq8-core` `magic_map.rs` (`onTeleport` / `offTeleport`, `earnTeleport [file]`, `teleportersEarned`, the "Go" button's lit test on every change, `Active::map_go`), `active.rs` (dispatch; `setState Connor Portal` logged as not built), `script.rs` (four stubs gone); viewer `gui_view.rs` (the lit picture, the button's click), `map_view.rs` / `main.rs` (the map handed to the rules first in each update).
**Why:** M6 6d step 7 (`magic-map.md` rules 35–39).
**Tested how:** `cargo test --workspace` (core 507 pass), clippy clean. `cargo play --world Daventry --at 217338,283698,3.1416 --kept-maps MASK/save/save18.vol --command "give MagicMap",5 --command "earnTeleport barrMap.kq",8 --key tab,10 --key left:IDCTL_MAPWIN_TELEPORT,15 --key left:IDCTL_MAPWIN_RIGHT,20 --key left:IDCTL_MAPWIN_RIGHT,30 --key left:IDCTL_MAPWIN_TELEPORT,40 --frames 600 --sound off`; `map.log`, `doors.log`.
**Result:** Standing on the keep's pad, its trap's `onTeleport` sets "on the pad" (first run: "KQMap not loaded", because the map was handed over after the traps; fixed by handing it over first). Unlit click: help 151 (logged). Dead City shown: unlit (not earned in save18). Barren shown (the Sage's mark): "the Go button lights". Click: `Davtele.cs` runs with `$1 = bartele.kq`; its `loadKQ bartele.kq @ …` is not done (no template while playing) and `setState Connor Portal` is logged as not built.
**Still broken / not tested:** **The Portal state (the Yes/No question, `teleportdown`, the world change, arrival) is not built:** it needs moving between worlds, which doesn't exist. Help popups are logged only. Not looked at.
**Next:** The user decides how moving between worlds should be built (it changes how the viewer loads a world); then the Portal state.

## 2026-10-21 6d-6: other worlds' maps (the page arrows)

**Changed:** `kq8-core` `magic_map.rs` (`match_palette`, the palettes by index, `OtherMap`, `show_other` / `show_own`, the snap-back, `WorldState::map_page`; the picture reader shared, `Arc`), `active.rs` (`other_maps`); viewer `map_view.rs` (the 16 palettes, the shipped map files, `--kept-maps SAVE`), `gui_view.rs` (the big map shows the shown picture, the mini-map this world's; the plate follows; no arrow on another world's map; the page arrows' clicks), `main.rs` (option, help).
**Why:** M6 6d step 6 (`magic-map.md` rules 9, 32–34).
**Tested how:** `cargo test -p kq8-core` (506 pass), clippy clean. `cargo play --world Daventry --at 193000,193000,3.1416 --kept-maps MASK/save/save18.vol --command "give MagicMap",5 --key tab,10 --key left:IDCTL_MAPWIN_RIGHT,20 --key left:IDCTL_MAPWIN_RIGHT,30 --key left:IDCTL_MAPWIN_RIGHT,40 --key left:IDCTL_MAPWIN_LEFT,50 --hold 1,0,0,60,90 --frames 800 --sound off`; `map.log`, `gui.log`.
**Result:** From save18: Dead City (53,055 pixels revealed, 24 icons) and Castle Daventry kept, marked visited. Right: Daventry → "Dimension of Death" (237 of 256 colours moved) → "Castle Daventry" → "Kingdom of Daventry"; left from Daventry → Castle. No arrow on the other maps. Walking: snapped back to Daventry at update 72.
**Still broken / not tested:** **Not looked at** (whether the matched colours look right). Kept maps come only from `--kept-maps` until moving between worlds exists. The page arrows' second frame (pressed?) is not used.
**Next:** 6d-7, teleporting up to the world change.

## 2026-10-21 6d-5: the mini-map in the bottom-right slot

**Changed:** Viewer `gui_view.rs`: the `IDCTL_INVBAR_MINIMAP` slot draws the live map (its own view's size, Connor's pixel at the middle, the arrow) while the map is active; the item's blink shows the empty picture; `gui.log` lines for each redraw with the real time.
**Why:** M6 6d step 5 (`magic-map.md` rule 27, `interface.md` rules 44, 45).
**Tested how:** `cargo play --world Daventry --at 193000,193000,3.1416 --command "give MagicMap",5 --key mapzoom:64,40 --hold 1,0,0,10,60 --frames 500 --sound off`, and the map given at update 60 (`--frames 700`) for the blink; `gui.log`.
**Result:** The slot shows the map at 298.3 × 298.3 (the start view's size) with the arrow's top-left at (−5, −5) from the slot's centre, scrolling with Connor; a big-map zoom to 596.5 leaves it at 298.3. Blink: map, empty, map, empty, map at 0.25 s steps. Found on the way: with the map given at update 5 the whole blink falls in the start-up stall (frames 1.5 s apart), so it isn't seen; not a bug of the blink.
**Still broken / not tested:** **Not looked at.** Whether the original blinks the live map or the item's picture (`todo.pbm`) is not in the note; we blink the live map.
**Next:** 6d-6, other worlds' maps.

## 2026-10-21 6d-4: pan, zoom and follow on the magic map

**Changed:** `kq8-core` `magic_map.rs` (`View::pan`, `zoom`, `follow_once`, `MapState::follow_step`, the follow mark); viewer `gui_view.rs` (`MapDrag`: a left or right drag that starts on the map area; the follow at each draw of the shown window; test keys `--key mappan:DX:DY,N` and `--key mapzoom:DX,N` in layout pixels), `game_camera.rs` (no camera swing during a map drag), `main.rs`.
**Why:** M6 6d step 4 (`magic-map.md` rules 22–25).
**Tested how:** `cargo test -p kq8-core` (unit tests for pan limit, zoom ×2 per 64 and limits 115–3,680, follow steps), clippy clean. `cargo play --world Daventry --at 193000,193000,3.1416 --command "give MagicMap",5 --key tab,10 --key mapzoom:192,15 --key mappan:5000:0,20 --hold 1,0,0,25,80 --frames 700 --sound off`; `map.log`.
**Result:** Zoom 192 px: 298.3 → 2,386.2 wide (× 8). Pan 5,000 right: stops with the picture's left edge on the area's centre (left 0.00). Once Connor moves to a new pixel, the follow brings the arrow's top-left from far outside to (275.8, 275.9) of the 296 area in 8 draws of 6 steps, then stops until his next pixel.
**Still broken / not tested:** **The real mouse drags are not tested** (the test keys use the same rules but skip the mouse); the camera not swinging during a right drag on the map is not tested. The follow margin is the stand-in (20 inside on all sides).
**Next:** 6d-5, the mini-map.

## 2026-10-21 6d-3: the magic map's window

**Changed:** `kq8-core` `magic_map.rs` (`View`, the start view, the arrow's place; views set up from the one stored on Connor), `panels.rs` (`toggle_map`), `gui_draw.rs` (`Piece::Map`: the map at its view with the arrow, cut to the area); viewer `gui_view.rs` (the map window as a panel: borders, area, name plate, page arrows, "Go" off; the map key from `movekeys.kq` row 8; a left click on the mini-map slot; `--key tab,N`), `map_view.rs`.
**Why:** M6 6d step 3 (`magic-map.md` rules 5–8, 18, 22, 29–31).
**Tested how:** `cargo test -p kq8-core` (501 pass), clippy clean. `cargo play --world Daventry --at 193000,193000,3.1416 --key tab,3 --command "give MagicMap",5 --key tab,30 --command "handsOff 1",60 --key tab,70 --command "handsOff 0",90 --key tab,120 --hold 1,0,0,10,100 --frames 800 --sound off`, and one with `--key left:IDCTL_INVBAR_MINIMAP,60` and `,70`; `gui.log`.
**Result:** Tab before the map: "not active; nothing". After it: the map slides in (0.2 s, `map.aud`); `handsOff 1` slides it out; Tab during handsOff brings it in; `handsOff 0` leaves it as it is; Tab closes it; the slot click closes and opens it. Name plate "Kingdom of Daventry" (one line). Drawn at the start view (298.3 × 298.3 layout px); the arrow's top-left (−34.0, 81.5) from the area's centre for pixel (191, 368), moving up 0.65 per map row walking north.
**Still broken / not tested:** **Not looked at** (no screenshot; checked in the logs only). Mouse on the map comes in 6d-4; clicks on it are ignored for now.
**Next:** 6d-4, pan, zoom and follow.

## 2026-10-21 6d-2: the magic map's state (no drawing)

**Changed:** New `kq8-core` `magic_map.rs` (map pixel, fog radius, the reveal disc, arrow frame, icons, the map's own picture, saved keys; `KQMap::activate` / `deactivate` / `stopLoc` / `useFog` / `setFogRadius`, `addMapIcon` (no longer a stub), `removeMapIcon`; Connor's visited marks), fields on `WorldState`; viewer `load.rs` takes the world's `<x>map.kq` from `GAME/Kq`; new viewer `map_view.rs` (pictures, icons, the update after the trap step); new `kq8-artifacts/logs/map.log`.
**Why:** M6 6d step 2 (`magic-map.md` rules 2–4, 7, 9–17, 19–21, 33).
**Tested how:** `cargo test -p kq8-core` (499 pass), clippy clean. `cargo play --world Daventry --at 193000,193000,3.1416 --command "give MagicMap",5 --command "KQMap::stopLoc 1",60 --command "KQMap::stopLoc 0",80 --hold 1,0,1,10,100 --turn 120,-1 --frames 900 --sound off`; `map.log`.
**Result:** "update 6: the MagicMap item arrives: the map davmap.kq is active"; (193000, 193000) → map pixel (191, 368), 1,696 pixels revealed; running north one map pixel per ~410 units, 46 new pixels each (the 2 × 23 band); nothing between stopLoc 1 and 0, then 368 at once (eight rows); turning right: arrow frames 0, 1, 2 … 11 (clockwise). Map index 0 marked visited.
**Still broken / not tested:** Nothing drawn yet. Icons not run in the viewer (Daventry adds them later in the story; unit-tested). The map's state isn't saved (no saves yet).
**Next:** 6d-3, the map window.

## 2026-10-21 6d-1: the magic map's files read (map, portal, saved fog)

**Changed:** New `kq8-data` `magic_map.rs` (`KQMap` and `KQPortal` keys, the fog's run lengths both ways) and `pvol.rs` (saved-game archive, read only); new `kq8 map-verify` (`kq8-cli` `map_verify.rs`). Plan "6d" in DESIGN; provenance.
**Why:** M6 6d, step 1 of building the magic map from `magic-map.md`.
**Tested how:** `cargo test -p kq8-data` (138 pass), clippy clean; `cargo run -- map-verify` (log `kq8-artifacts/logs/map-verify.log`).
**Result:** All 16 map files and 6 portal files read. Units per map pixel and fog radii match the note (Daventry 409.6 / 24 px, Snow exit 131 from `hazeDistance` 18,000). All 22 saves read (the index is padded to an even offset, not 4: save2), 29 saved maps: save10's Daventry 108,474 pixels revealed (51.3 %), Dead City (save18) 53,055, Castle 53,546. Daventry and Dead City run lists written back identical. The Castle's saved map has column 460 (an unused place) marked revealed in rows 194–256: the note's "Do not reproduce" 2 (one pixel past the copy) at the right edge; ignored on reading as the note says, so its written-back list differs but gives the same fog.
**Still broken / not tested:** The current world's map in a save is a binary form: not read. Nothing in the game uses these readers yet.
**Next:** 6d-2, the map state (no drawing).

## 2026-10-21 Magic map: the user's check in the original

**Changed:** Docs only: `magic-map.md` rules 25 and 27, edge cases, "Do not reproduce" 4, "How to check", open questions 2 and 3; STATUS.
**Why:** The user ran `magic-map.md` "How to check" 1–11 in the original.
**Tested how:** The user, by eye, in the original (GOG 1.03).
**Result:** All 11 points as described. The big map first shows all of Daventry; the mini-map shows only the part around Connor and does not change size when the big map is zoomed (it keeps its own zoom). The view scrolls to keep the arrow in at all four edges once Connor moves (margin not measurable by eye; build rule: 20 pixels inside the visible area). The map opens during conversations and scripted scenes.
**Still broken / not tested:** Whether the mini-map picks up the big map's zoom after a world change; the arrival side of a world change (not traced).
**Next:** B: build the map from the note (the session B prompt's mini-map stand-in is now wrong: the mini-map keeps its own zoom).

## 2026-10-21 Study (A): the magic map

**Changed:** New behaviour note `docs/behaviour/magic-map.md`. Small updates: `interface.md` rule 11 (the map key checks only that the map is active), rule 45 and open question 4 (the map slot is a live mini-map), QUESTIONS Q108 (b, partly), STATUS. Scratch addresses in `re/notes.md` (not committed).
**Why:** The magic map and teleporting are M6 work; B needs a note to build them from.
**Tested how:** Traced in `Mask.exe` with Ghidra; data from `GAME/Kq/*map.kq`, `*tele.kq/.cs`, the world scripts, `Gplayscr.gui` (from `gui-verify` logs), `500.MSG`, `Invinfo.itl`; the 16 `maparrow.pba` frames dumped and looked at; all 28 map pictures measured (460 × 460); the user's saved games read (not changed) to see the saved map format. Nothing run in the original.
**Result:** Map pixel = (x − west) ÷ (east − west) × 460 across, (1 − (y − south) ÷ (north − south)) × 460 down (north up). Fog radius in pixels = 460 × `fogRadius` ÷ (east − west) (Daventry 24; missing → `hazeDistance`, Snow exit 131), revealed only when Connor reaches a new map pixel; saved as `rle0`… over 464-place rows, hidden run first, `-1` at the end (save 10's Daventry: `rle0=18768` … `rle705=-1`). Arrow frame = round((π − heading) × 16 ÷ 2π) mod 16, frame 0 up, clockwise. Start view fits the whole map (× 0.648); zoom ×2 per 64 px of right-drag, limits ×¼–×8; left-drag pans with the centre kept on the map. Page arrows step through visited worlds' maps (colours matched to the current palette). "Go" lights on a pad when the shown world's teleporter is earned; it runs `<world>tele.cs`, which starts the Portal state ("You are about to load a new region…", Yes → `teleportdown`, world change, `teleportup`; the map ends closed). Bugs not to copy: "Go" stays lit after a snap-back; one extra revealed pixel per row after reload; `useFog 0` lost on reload.
**Still broken / not tested:** **Not checked in the original.** Not settled: which coordinates the follow margins (20–280, 20–340) are in; whether the mini-map's zoom follows the big map's at once; the arrival side of a world change (not followed); how another world's kept map is found (inferred from the saves).
**Next:** The user: `magic-map.md` "How to check" 1–11 in the original (7 and 6 matter most). B: build the map (M6) from the note.

## 2026-10-21 Main menu: the user's check in the original

**Changed:** Docs only: `main-menu.md` rules 3, 8, 21, 25, "How to check" 2, open questions 1–3 answered; STATUS.
**Why:** The user ran `main-menu.md` "How to check" 1–7 in the original.
**Tested how:** The user, by eye and ear, in the original (GOG 1.03).
**Result:** All 7 points as described. Items that are off are drawn greyed (dithered, like the Invisibility potion; plain grey is fine as a stand-in). Hovering doesn't move the mask cursor. The theme starts at once with no fade. After Connor dies: a couple of seconds of `handsOff`, then the menu appears.
**Still broken / not tested:** Whether saved games keep the combat level; what happens during the death `handsOff`; the theme during the intro film (later).
**Next:** B can build the menu from `main-menu.md` when the user wants it.

## 2026-10-21 Study (A): the main menu

**Changed:** New behaviour note `docs/behaviour/main-menu.md`. Small updates: `fighting.md` open question 3 (combat level menu answered), `interface.md` rule 14 and `sound.md` rule 54 (the menu's pause now traced), STATUS. Scratch addresses in `re/notes.md` (not committed).
**Why:** The main menu was postponed (2026-10-20); this gives session B a note to build it from when its turn comes.
**Tested how:** Traced in `Mask.exe` with Ghidra; layouts from `kq8-artifacts/logs/gui-verify.log`; texts read from `GAME/ENGLISH/500.MSG` (noun 0, verb 0, case = the `#n` number); `MASK/current.bin` is 19,204 bytes, which fits the thumbnail format. Nothing run in the original.
**Result:** Items New / Load / Save / Options / Replay Intro / Quit / Back to Game, and when each is on (Save and Back only while a game runs; Load only with saves). Keys: Up/Down wrap and skip items that are off, Enter, Escape = Back to Game, Left/Right only in popups, one action per press, input ignored 0.2 s after the menu opens. Popups: new game while one runs, quit, corrupt save, Combat difficulty (Easy 0.66, Normal 1.0, Hard 1.5, Normal first). Opening pauses the game (all sounds but the menu's) and starts `theme.aud`; Back fades it out over 1 s. Connor's death opens the menu with Save and Back off. Found a bug in the original not to copy: a mouse click on Yes in the corrupt-save question does nothing.
**Still broken / not tested:** **Not checked in the original.** Not settled: whether hovering the mouse over an item moves the mask cursor, whether items that are off are hidden or greyed, whether the theme fades in, and whether the combat level is kept in saved games.
**Next:** The user: `main-menu.md` "How to check" 1–7 in the original. B: build the menu when the user wants it (with a launch option that skips it, as asked 2026-10-20).

## 2026-10-21 Fix: the Wizard's "Come closer, lad!" (`checkDistance` built, Q142)

**Changed:** `kq8-core` `active.rs`: the `checkDistance` command (`check_distance`), unit test `check_distance_is_straight_3d_and_inclusive`; `script.rs` comment. Provenance, STATUS, QUESTIONS (Q142 built).
**Why:** The user: the Wizard never calls Connor over from a distance. `wiztrap.cs` (a 16,000 × 16,000 trap around the Wizard) runs `checkDistance Wizard Connor 5000` on entry, at 6 s and every 8 s after, and speaks `1000 4 0 119` when the answer is 0 (farther). We didn't have the command: unknown commands leave 1, so he never spoke. Built from `anim-scripts.md` rule 31 (Q142's answer, waiting to be built).
**Tested how:** `cargo test -p kq8-core` (490 pass), clippy clean. `cargo play --world Daventry --at 193000,193000,3.1416 --frames 600 --sound off`; `--at 192500,196500,3.1416 --frames 200`; the axe: `--sound off --at 145910,249400,0 --no-ai --god --click ax,40 --frames 300`; `traps.log`, `talk.log`, `doors.log`.
**Result:** Far: "checkDistance Wizard (190807, 198274, 6630) to Connor (193000, 193000, 7844): 5839 > 5000: 0", the Wizard says "Come closer, lad!" (`a0rs0400.3b1`, 1.645 s) at update 1 and again at the cue at update 91 (6 s), next cue at 14 s. Close: 2452 <= 5000: 1, silent. The axe from the windmill floor: 2388 > 1700: 0, the Henchman's line `1000 9 14 18`, no axe (before: always given).
**Still broken / not tested:** **Not heard by the user.** The axe from the hay bale (should work within about 830 sideways) was not run. Other worlds' 27 `checkDistance` scripts were not run.
**Next:** The user: walk toward the Wizard from the south-east and listen.

## 2026-10-21 Levels: the user's check in the original

**Changed:** Docs only: Q149 marked partly answered; provenance, DESIGN, STATUS.
**Why:** The user compared our levels, the flame and the sound with the original.
**Tested how:** The user, by eye and ear, in the original and in our build.
**Result:** The experience needed per level (10, doubling to level 14, then 150,000) and the level-ups match the original. The level-up sound is `LEVEL.AUD` (Q149 c answered; no longer a stand-in). The flame plays on a level-up.
**Still broken / not tested:** The flame's speed, playing once and hiding were not compared (Q149 b). Q149 a, d, e still open.
**Next:** —

## 2026-10-21 L-3: the level-up sound

**Changed:** `kq8-core`: `stats::LEVEL_UP_SOUND` (`LEVEL.AUD`), `WorldState::level_up_sound`, called where Connor's level goes up (`active.rs` stats event, `connor_inventory.rs` an item applied, `fight.rs` a kill's experience); the unit test `stats_events_add_and_an_object_below_0_gets_die` checks it plays once. DESIGN, provenance.
**Why:** The user: the original plays a sound on a level-up. No note or data file names it; `GAME/SOUND/LEVEL.AUD` is the only sound with a level name (stand-in, Q149 c).
**Tested how:** `cargo test --workspace` (638 pass), clippy clean. Daventry with sound on, `--command "sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 11,20"` and `... 5,40`; `sound.log`. `kq8 aud-verify --wav GAME/SOUND/LEVEL.AUD` wrote `kq8-artifacts/wav/LEVEL.wav` (1.792 s).
**Result:** "start #5 LEVEL.AUD (GAME/SOUND/LEVEL.AUD, 1.792 s) group 0 plain ... [level up to 2]", finished 1.9 s later; nothing for the +5 (no level-up).
**Still broken / not tested:** **Not heard by the user.** Whether it is the right sound, and its loudness (the effects group's), are stand-ins (Q149 c). The item and kill paths were not run in the viewer (same function; unit test covers the stats event).
**Next:** The user: listen to `LEVEL.wav`, level up in game (`xp 11` in the console).

## 2026-10-21 L-2: the level-up flame over the level number

**Changed:** Viewer `gui_view.rs`: `level_flame` (start time), `LEVEL_FLAME`, `flame_file`; the meters' sending starts it when the level sent is higher than the last one (not the first sending when a world opens); it is drawn in its layout place (`IDCTL_METERS_LEVELUP`, (113, 0)-(143, 14) of the meters, over the level number) frame by frame and hidden when it has played out. DESIGN, provenance.
**Why:** The user: on a level-up the original plays a flame on the level number. `interface.md` rule 26 names the picture (`flame14.pba`) but not when or how it plays (open question 3, Q149 b).
**Tested how:** `cargo test --workspace` (638 pass), clippy clean. Daventry with `--command "sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 11,20"` (and +5 at 60, +30 at 70); Dead City started plainly; `gui.log`.
**Result:** "level 1 -> 2: the level-up flame starts (17 frames at 15 per second, once)" at update 21, "played out after 1142 ms"; no flame for the +5 (no level-up); again for level 2 → 3. Dead City's start at level 8: no flame.
**Still broken / not tested:** **Not looked at by eye**: whether it sits right over the number, its colours and see-through parts. Speed (15 per second), once, and hidden after: stand-ins (Q149 b, Q108 d).
**Next:** L-3, the level-up sound.

## 2026-10-21 Levels checked; L-1: a world's start-up script gives Connor his level, stats and equipment

**Changed:** `kq8-data/world_script.rs`: `KQConner::setLevel` and `sendEvent` go to the host (`connor_line`) instead of only being counted. New `kq8-core/connor_start.rs` (`StartLine`, `run`, unit test). `active.rs`: the `KQConner::setLevel` command (`set_level`). Viewer `load.rs`: the items made for Connor, `setLevel` and stats events to Connor are kept in script order (`connor_start`); `door_view.rs` does them when play starts. DESIGN plan "Connor's levels checked", Q149, provenance.
**Why:** The user asked to check experience, levels, health, armour and attack. The level rules were right (see Result), but a world started directly (all but Daventry) gave Connor only the items of its `conStart.cs`: Dead City started at level 1, 10 health, hand 10, armour 0, with the Small Axe and leather armour carried but not counted.
**Tested how:** `cargo test --workspace` (638 pass), clippy clean. Daventry with `--command "sendEvent 0 Connor KQStatsChangeRequestEvent ..."` (experience 10, 1, 20, 1,000, 0, 0, health −5, +500), `--command "give ..."` for the Small Axe, leather armour / boots / gloves and Small Crossbow; Dead City, Swamp, temple1, Daventry started plainly; `viewer.log`, `gui.log`, `items.log`.
**Result:** Levels: 10 is not enough at level 1, 11 is; one level per event; thresholds 10, 20, 40, 80, 160, 320, 640; each level +10 health (full heal), hand +5, ranged +2, armour +1; health capped at 10 × level; experience bar = experience × 100 / threshold. Equipment while playing: hand +15, armour +2 / +1 / +1, ranged +5. All match the notes. Start-up, before → after: Dead City level 1 [10, 10, 0, 0, 0] → level 8 [80, 60, 19, 11, 0]; Swamp → level 15 [150, 110, 38, 21, 0]; temple1 → level 43 [430, 340, 182, 67, 34970]; Daventry unchanged (level 1 [10, 10, 0, 0, 0]).
**Still broken / not tested:** **Not looked at by eye.** `setLevel` only sets the level number (stand-in, Q149 a). temple1's quest items give built-in experience before `setLevel` (two extra level-ups, Q149 e). Not a bug, by the notes: after a level-up the experience bar is not redrawn when the experience number itself didn't change (`interface.md` rule 22), so it can show the old level's percent until the next change.
**Next:** L-2, the level-up flame; L-3, the level-up sound.

## 2026-10-21 Quieter terminal: the movements' "frame steps" lines go to viewer.log only

**Changed:** Viewer `load.rs` (`connor_set`): the "connor: <movement>: can be cut short …; frame steps …" line is written with `detail` (file only) instead of `summary` (terminal and file).
**Why:** The user saw these long lines in the terminal at start-up and at a `give`; they are only for checking movements in the log.
**Tested how:** Release build; `kq8-viewer --world DeadCity --sound off --frames 600 --command "give LakeSword 1,5"`, its terminal output saved and counted.
**Result:** "frame steps" lines: 0 in the terminal, 287 in `viewer.log` (start-up and the Lake Sword's set). Nothing else changes.
**Still broken / not tested:** —
**Next:** —

## 2026-10-21 M6 6c-2: a weapon's movement set is loaded when it is given

**Changed:** Viewer `load.rs`: `weapon_sets` also returns the weapons it left out; `LateSets` (game folder, world, palette, face texture, those weapons) goes with Connor's assets; `read_set`, `ghost_texture`. `connor_view.rs`: the drawing of one animation is now `spawn_frames` (used by `spawn_connor` and the new step); new `late_set_step` (after `door_step`): a carried weapon whose set isn't loaded gets it, drawn under his body. `potion_view.rs`: `ConnorLooks::add` and a redo of the looks. `main.rs`: the step registered. DESIGN, provenance.
**Why:** The user's check: the Lake Sword given in Dead City had no stance. The user chose loading at the give over loading every set (+2.2 s per world).
**Tested how:** `cargo test --workspace` (637 pass), clippy clean. `kq8-viewer --world DeadCity --sound off --frames 1500 --no-ai --command "give LakeSword 1,5" --key left:IDCTL_INVBAR_SHORT,30`; `viewer.log`, `fight.log`, `connor.log`. Daventry's start-up lines and first-frame light line compared with the earlier build.
**Result:** Dead City: "weapon set conSword.anm (invitem_lakesword) loaded while playing: 39 movements, 38 animations, 115 materials (1 I_ pictures), 157 ms" at the give (update 5); start-up still 2.4 s. Readied at 30: `["away", "consword.anm/out"]`, then `consword.anm/stop` drawn (the sword stance). Looks redone: 823 → 900 materials. Daventry: start-up lines and light identical.
**Still broken / not tested:** **Not looked at by eye.** Strength or Invisible on a set loaded this way not run (the looks are redone, logged). The 157 ms are a pause in that one update.
**Next:** The user: the Lake Sword in Dead City by eye.

## 2026-10-21 M6 6c-1: one movement set's loading as its own function

**Changed:** Viewer `load.rs`: the loop in `place_connor` that reads one movement set (his own or a weapon's) is now `connor_set`, a function the viewer can also call while playing (6c-2). Plan "6c" in `docs/DESIGN.md`. Also tried and undone before this: loading every weapon's set in every world (+2.2 s per load: Dead City 2.4 → 4.6 s, Daventry 3.4 → 5.6 s); the user chose loading a set when its weapon is given.
**Why:** The user's check: the Lake Sword given by the console in Dead City has no stance; its set `conSword.anm` is not loaded there (no Dead City file names it).
**Tested how:** Release build; the "connor:" lines of `viewer.log` for Daventry (1,601) and Swamp (1,451) compared with the previous commit's build: identical.
**Result:** No behaviour change.
**Still broken / not tested:** The Lake Sword in Dead City still has no stance (6c-2).
**Next:** 6c-2, the set loaded at the give.

## 2026-10-21 Fix: every weapon's draw and put-away has both parts in every world

**Changed:** Viewer `load.rs`: Connor's own `away` and `out` (`Conner.anm`) are always loaded with the weapon lists. Provenance (5d fix 10's table).
**Why:** The user's check in Swamp: drawing the sword or the hammer started with his hand already at the holster, and putting them away ended abruptly (as the dagger before 5d fix 10). The two-part swap of 5d fix 10 was right, but its first part (drawing) and last part (putting away) are his own `away` / `out`, which were loaded only in worlds whose scripts name them (Daventry's `urn1.cs`, `wheeltp1.cs`); elsewhere the swap skipped them silently.
**Tested how:** `cargo test --workspace` (637 pass), clippy clean. `kq8-viewer --world Swamp --sound off --frames 3000 --no-ai --key left:IDCTL_INVBAR_SHORT,20 --key left:IDCTL_INVBAR_SHORT,90 --key left:IDCTL_INVBAR_LONG,160 --key left:IDCTL_INVBAR_LONG,230`; `fight.log`, `connor.log`.
**Result:** Before: `["conkng.anm/out"]`, `["conkng.anm/away"]`, `["conhamr.anm/out"]`, `["conhamr.anm/away"]`. After: `["away", "conkng.anm/out"]` (20–37), `["conkng.anm/away", "out"]` (90–105), `["away", "conhamr.anm/out"]` (160–179), `["conhamr.anm/away", "out"]` (230–247).
**Still broken / not tested:** **Not looked at by eye.** Only Swamp run; the other worlds load the same way.
**Next:** The user: the sword and hammer draw by eye in Swamp.

## 2026-10-21 M6 6b fix: the hammer's return cuts the throw short

**Changed:** Viewer `connor_view.rs`: `throw_caught` starts `catch` at once also while `attack` plays (it waited for `attack`'s end); `throw_step` doesn't start `catch` again when `attack` was already cut; each throw movement is drawn from its start at once (`redraw_from_walker`, as the rope's 6a fix). Q148 (c) marked answered; provenance, DESIGN, STATUS.
**Why:** The user's check in the original: right against a wall Connor plays the catch the moment the hammer leaves his hand; ours finished the throw first.
**Tested how:** `cargo test --workspace` (637 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 1500 --no-ai --command "give AzrielHammer 1,5" --command "move Connor 0 280866 195989 15000 0 0 0.75,10" --key left:IDCTL_INVBAR_LONG,20 --aim-point 281478,195332,14757,80`; `fight.log`.
**Result:** `do` at 91; the hammer touches the cottage in its first update (470 out) and is back at 93: "'conhamr.anm/catch' starts, 'conhamr.anm/attack' cut short at 0.722"; standing at 104 (1.60 s, was 1.93 s). A longer throw (back after `attack` ended) is unchanged.
**Still broken / not tested:** **Not looked at by eye.** Q148 a, b, d, e still open.
**Next:** The user: the wall throw by eye.

## 2026-10-21 M6 6b-3: Connor throws Azriel's Hammer and catches it

**Changed:** `kq8-core/connor.rs`: walker mode `Throw` (`throw_play`, `throw_leave`), shielded from falls, landings and pushes in `connor_update.rs` like the rope's; unit test. Viewer `connor_view.rs`: `Throw`, `throw_click`, `throw_caught`, `throw_step`, the `do` cue, state requests refused during the throw. `door_view.rs`: with a `KQHammer` bolt file the click throws instead of shooting; the hammer launched at the cue from his shoot offset; `hammers_back` → `catch`; characters still see him during the throw. `gui_view.rs`: the throw's handsOff leaves the panels alone. `load.rs`: the hammer list's `wait` and `catch` loaded; every ranged weapon's bolt file the world has is read (a console `give` in Daventry). Q148 (c)–(e); provenance; DESIGN.
**Why:** `azriel-hammer.md` rules 2–5, 10: the throw state (`attack`, `wait`, `catch`, handsOff until the catch).
**Tested how:** `cargo test --workspace` (637 pass), clippy clean. Viewer runs, read in `fight.log` / `connor.log`: Swamp `--key left:IDCTL_INVBAR_LONG,20 --aim-point 165975,80956,6646,60` (the user's own aim point); Daventry with `--command "give AzrielHammer 1,5"`: `--aim goblin22,60`, then Connor moved 5,000 south of the goblin (`move Connor 0 289284 187527 15000 0 0 3.14,10`) and `--aim-point` at the goblin, at the ground, and with forward held.
**Result:** Swamp: click 60 (turned 20°, no waiting), `do` 71, out 7 updates to the object aimed at, back 7, caught at 86 (`catch` at once), standing at 97 (2.47 s); start offset (−41, −605, 1,226). Daventry: off the cottage wall; at goblin22 (which had run up to him): hit for 40, it died, the hammer back into Connor's hands the next update; at the ground: back at once; held forward sent again after the catch.
**Still broken / not tested:** **Not looked at by eye.** Not run: a throw from a real walk (Connor had walked into a tree), a full-range throw at nothing, a hit taking Connor below 0 during a throw, Dead City's lever (no Dead City yet). Stand-ins asked in Q148: (c) back before `attack` ends → `catch` after it; (d) requests during the throw wait for the catch; (e) the panels stay. How the hammer model is turned in flight is not in the note (drawn like a bolt, Q148 b).
**Next:** The user: throw the hammer by eye in Swamp (or `give AzrielHammer` in Daventry).

## 2026-10-21 M6 6b-2: Azriel's Hammer flies out and back

**Changed:** `kq8-core/fight.rs`: `Bolt` gets an optional `HammerFlight` (way out / back, the damage keys); `launch_bolt` names it `hammer<n>`; `bolts_step` hands a hammer to the new `hammer_step` (turn back, the way back, the catch) and `hammer_hits` (rule 8); `hammer_damage`. `active.rs`: `hammer_calls` (the lever's `ConnorAction`, called like `ProjImmune`) and `hammers_back` (for the viewer, step 6b-3); 5 unit tests. Q148 in `QUESTIONS.md`; provenance; DESIGN.
**Why:** `azriel-hammer.md` rules 6–9: one thing hit, back along the same line, always caught.
**Tested how:** `cargo test --workspace` (636 pass, 5 new), clippy clean. Core only: nothing in the viewer uses it yet.
**Result:** Daventry's file, thrown level at nothing: turns back at 14,400, comes back 12,800, ends at update 19 (1.27 s) without reaching him (as rule 9: "about one step short"). At a character 4,400 away: hit for 40 (30 + 10), back after 5 updates; the character behind it untouched. At the ground: back at once. Connor on the line: passed through on the way out, stops it on the way back. `tmblrcyl` runs `lever1`'s script; an immune object takes 0, no `ProjImmune`.
**Still broken / not tested:** Not run in the viewer (the throw state is 6b-3): a click now would launch the hammer and it would fly back, but Connor would not wait or catch it. Where it turns back at its range or the ground is our reading (**Q148 a**); how the model is turned in flight is not in the note (Q148 b).
**Next:** 6b-3, the throw state.

## 2026-10-21 M6 6b-1: Azriel's Hammer: its file is read, and Connor takes it at the start

**Changed:** `kq8-data/characters.rs`: `ProjectileKeys` also reads `classType=KQHammer` (flag `hammer`, `modifier`, `minDamage`, `maxDamage`, default mask 0x221F). Viewer `door_view.rs`: Connor's fight (`set_connor_fight`) and the projectile files are set up after the items the start-up scripts made, not before. `load.rs`: the log line shows the hammer's keys. Plan "6b" in `docs/DESIGN.md`; provenance.
**Why:** The user threw the hammer in Swamp and got an arrow. Two causes: `prhammer.kq` is a `KQHammer`, which our reader turned away, and Connor's ranged weapon was looked for before `constart.cs`'s items reached core, so he kept `PRarrow.kq` and its offset. Source: `azriel-hammer.md` rules 5, 6; `ranged-weapons.md` rules 17, 21.
**Tested how:** `cargo test --workspace` (631 pass, 1 new), clippy clean; `kq8-viewer --world Swamp --sound off --frames 60`, read `fight.log` and `characters.log`.
**Result:** `fight.log`: "carried at the start: INVITEM_AzrielHammer: shoots Prhammer.kq; shoot offset (-41, -605, 1226)". `characters.log`: speed 30,000, range 20,000, mask 0x221F, modifier 30, 0–300 (as the note's table for Swamp).
**Still broken / not tested:** Nothing of the throw yet: a click would now launch the hammer as a plain bolt (one way, removed at the first touch); 6b-2 and 6b-3 build the return and the throw state. Found on the way, **not fixed** (separate step): the start-up scripts' items don't go through Connor's inventory rules at all (`receive`), so the chain mail, boots and gloves `constart.cs` gives are not put on (protection 0 in Swamp).
**Next:** 6b-2, the hammer's flight out and back (core).

## 2026-10-21 The user's check of the Rope and Hook

**Changed:** This entry; STATUS, FEATURES, README.
**Why:** The user tried the rope in our viewer after Q147's answers were built.
**Tested how:** The user's check by eye, at two rope places.
**Result:** "The rope is working perfectly."
**Still broken / not tested:** Ropes of the other worlds not tried (Daventry only); saved games and first person don't exist yet. The note's open question 7 (the top trap's `On` mid-climb) is still open, small.
**Next:** —

## 2026-10-21 M6 6a: Q147 (a), (b), (d) as built; stand-in labels removed

**Changed:** Log texts and comments in `rope.rs`, `door_view.rs`, `rope_view.rs`: no more "stand-in" / "pointer kept" where A's answers confirm what we built (the first free icon place; the rope's handsOff is a plain `handsOff 1`, which is what ours already did; `taway`'s step). STATUS, DESIGN, provenance.
**Why:** A answered Q147 (`ropeandhook.md` rules 7, 13, 24).
**Tested how:** `cargo test` (619 pass), clippy clean, release build.
**Result:** No behaviour change in this step.
**Still broken / not tested:** **None of the Q147 changes looked at by eye yet.** Left open: the note's open question 7 (the top trap's `On` mid-climb in the original).
**Next:** The user: the rope by eye in Daventry (the rope flying up now, the camera after the throw).

## 2026-10-21 M6 6a, A's answer to Q147 (c): the top camera's one-time flag

**Changed:** Viewer `game_camera.rs`: `skip_next_move` / `move_skipped`: after the rope is taken at the top, Connor's first move is not followed (once); cleared when the rope state moves the camera. `door_view.rs` sets it with the rope camera; `rope_view.rs` releases the rope's first-person lock at that move. `active.rs` comment.
**Why:** `ropeandhook.md` rule 9 as answered by A (Q147 c, traced): the camera keeps following, but a flag makes it skip his first move, and the first-person lock is released then; the rope stays on the pointer.
**Tested how:** Release build. `kq8-viewer --world Daventry --sound off --frames 2500 --at 152500,207968,-1.5708 --command "give RopeAndHook 1,5" --command "move Connor 0 147200 207586 13300 0 0 1.57,10" --key left:IDCTL_INVBAR_ROPE,60 --hold 0,1,0,90,93 --hold 0,1,0,110,113`; `rope.log`, `camera.log`, `gui.log`.
**Result:** 61: the camera at the `setRopeCam2` spot (151584, 210996, 8520). Tick 91 (his first turn): "not followed, the camera stays; the flag is cleared"; 92: the lock released, the rope still in hand (no hand change in `gui.log`). Tick 92 on: followed as usual (pulled onto the 5,000 range at (149960, 209733, 10623)).
**Still broken / not tested:** **Not looked at by eye.** First person doesn't exist, so "back to first person" is only the lock.
**Next:** the docs for Q147 (a), (b), (d) (already as built).

## 2026-10-21 M6 6a, A's answer to Q147 (e): the traps when the rope places Connor

**Changed:** Viewer `door_view.rs` `connor_placed_by_rope`: only the stay test (as a script's `move`, `traps.md` rule 5); the enter test at the new place is gone, so the other end's `On` comes at his next move. `rope_view.rs` log text.
**Why:** `ropeandhook.md` rule 18 as measured by A (Q147 e): the end he left gets `Off` at once, the other end's `On` later.
**Tested how:** Release build, the foot climb up (`--at 152500,207968,-1.5708 ... --hold 1,0,0,200,300`); `rope.log`.
**Result:** 162 (the lift): "ropetrap Off Connor", status bottom → none. 277 top end; 323 (during `taway`, his first move): `ropetrp2` On, status none → top, `HookDown` shown.
**Still broken / not tested:** In the original the top's `On` came during the climb (5.5 s after the lift, the cause not found: the note's open question 7); ours comes at his first move, in `taway`. Only the climb up run in the viewer.
**Next:** the top camera's flag (Q147 c).

## 2026-10-21 M6 6a, A's measurement: the grab's camera spot on a tie

**Changed:** `rope.rs` `camera_spot`: of the game's own two spots, the + one when equally near (within 0.5, ours); `setRopeCam` spots keep "2 when equal". Unit test with A's church numbers.
**Why:** `ropeandhook.md` rule 18 (measured by A, Q147 f): from the throw camera straight behind him both spots are equally near and the original takes the + one; ours took the other.
**Tested how:** `cargo test -p kq8-core rope` (14 pass: church foot → (208837, 188873, 13104) as measured), clippy clean. The foot climb run; `rope.log`.
**Result:** Update 162: both spots 5,110.3 from the camera; spot 1 (150148, 205088, 12600) taken.
**Still broken / not tested:** **Not looked at by eye.** The 0.5 tolerance is ours.
**Next:** the traps at a placing (Q147 e), the top camera's flag (Q147 c).

## 2026-10-21 A: Q142 and Q141 answered (`checkDistance`, the walk after the axe swap)

**Changed:** `docs/behaviour/anim-scripts.md` rules 31 (`checkDistance`) and 32 (a script's walk during a weapon swap), numbers, "How to check"; QUESTIONS Q141, Q142 answered; STATUS. No code.
**Why:** Q142 blocks Daventry's axe puzzle (ours always gives the axe); Q141 had two small points left.
**Tested how:** Read in `Mask.exe` (Ghidra). Checked the axe numbers against the data: `ax.kq`, `newworld.cs`'s hay bale, `BOUNDS.KQ`. Not measured in the running original: the code is plain, and the user's check already gives the outcome.
**Result:** `checkDistance` measures straight 3D distance between positions (Connor's feet, an object's placed point). It leaves 1 if the distance is at most the max, 0 if farther or on any error. Axe: from the floor his feet are 2,475 or more below it, so never; from the bale it works within about 830 sideways. Q141: any swap holds a script's walk as the pending request until the swap ends. The `move` between only places him.
**Still broken / not tested:** The 830 reach is worked out from the bale's box and Connor's feet height, not measured. Whether the automatic turn turns Connor during a swap was not traced.
**Next:** B: build `checkDistance` (the axe puzzle) and drop the always-1 stand-in; nothing to change for Q141.

## 2026-10-21 M6 6a, A's answer to Q147 (f): the rope model's place and the throw camera

**Changed:** `rope.rs`: `ROPE_MODEL_OFFSET` (2,000 along the rope's +y), `THROW_CAM` = 2,250 behind, 350 up, tilt 1.27, heading the rope's + 3.14 (`THROW_CAM_TURN`, the original's own number); the unit test is now A's measured church case. Viewer `connor_view.rs`: the rope model's frame drawn 2,000 behind Connor. The `--rope-cam` test option is removed (the values are known). FEATURES.
**Why:** `ropeandhook.md` rule 15 as answered by A (Q147 f, traced and measured in the original). The rope model was drawn at Connor's own place, inside the wall: why the user saw no rope flying up.
**Tested how:** `cargo test -p kq8-core rope` (14 pass; the measured case: camera (206337, 188623, 8654), looking (0, −0.296, 0.955)), clippy clean. The foot climb run; `rope.log`.
**Result:** At 142 the camera goes to (149896, 207588, 8150) = the rope's place + 2,250 behind, + 350 up, tilt 1.270, looking along (−0.296, 0, 0.955), in the same update as his placing.
**Still broken / not tested:** **Not looked at by eye** (the rope flying up, from the new place). The grab still takes the − spot on an exact tie (next step).
**Next:** the grab's tie, the traps at a placing, the top camera's flag.

## 2026-10-21 A: Q147 answered (rope: icon place, handsOff, top camera, taway, traps, the throw camera)

**Changed:** `docs/behaviour/ropeandhook.md` rules 7, 9, 13, 15, 18, 23, 24, numbers, "How to check", open questions (2 answered, 7 new); QUESTIONS Q147 answered; STATUS. No code.
**Why:** B's Q147 (a–f), mainly (f): the note's open question 2, where the camera goes at the end of `btoss`.
**Tested how:** Read in `Mask.exe` (Ghidra). Then measured in the running original with a new read-only memory reader (reads the game's memory from outside, changes nothing; kept in `re/`, not committed) while the user climbed down Daventry's church rope and threw it from below.
**Result:** (f) Camera = rope place + (0, 2,250, 350) in the rope's frame, angles (1.27, 0, rope heading + 3.14). Measured: (206337, 188623, 8654), (1.27, 0, 3.14) for the rope at (206337, 186373, 8304), heading 0. (0, −500, 2,000) is unused. The rope model is 2,000 behind him. Connor's jump (1,840 in this run) happens in the same update as the camera cut; he is very likely out of view (inferred). (a) First free status place. (b) The rope's `handsOff` is a plain `handsOff 1`; the note's "pointer kept" was wrong, corrected. (c) The top camera flag skips following his first move, which also gives the first-person lock back. (d) `taway` moves him 405.2 forward. (e) `Off` of the end he left first, then `On` of the other end at his next counted move.
**Still broken / not tested:** My first reading of the `btoss` camera came out at three times Connor's position (a dead end): I had missed that the code zeroes the position before turning the offsets. Only a heading-0 rope was measured; that the offset turns with the rope is from the code only. Why the top trap's `On` came 5.5 s into the climb is not found (open question 7). Whether the jump is visible was not looked at by eye.
**Next:** B: replace the `--rope-cam` stand-in with these numbers, make the rope's handsOff a plain `handsOff 1`, and skip the camera's first follow after the top rope is taken.

## 2026-10-21 M6 6a fix 2: the camera after the throw, farther back and adjustable

**Changed:** `rope.rs`: `THROW_CAM` [behind, up, tilt] = [2000, 1400, 1.27] (was 250 behind), kept in the climb's record; `throw_camera` takes it. Viewer: launch option `--rope-cam BACK,HEIGHT,TILT` (`main.rs`, `rope_view.rs` `RopeCam`), FEATURES.
**Why:** The user's look: the snaps are fixed, but the camera after the throw was too close to the wall and didn't show the rope flying up. The exact placement is not known (Q147 f), so the user can try values against the original.
**Tested how:** `cargo test -p kq8-core rope` (14 pass), clippy clean. The foot climb run (`--at 152500,207968,-1.5708 ... --aim-point 150000,207968,30000,90`); `rope.log`.
**Result:** At 142 the camera goes to (149646, 207588, 9200): 2,000 behind him, 1,400 up, tilt 1.270, looking toward the wall and up.
**Still broken / not tested:** **Not looked at by eye.** Whether the rope is seen flying up from there is for the user to check; the values are a stand-in (Q147 f).
**Next:** The user: compare with the original, try `--rope-cam` if it's still off.

## 2026-10-21 M6 6a fix: rope animation snaps and the camera after the throw

**Changed:** Viewer `connor_view.rs`: `redraw_from_walker`, `phase_step_now`, `drawn_phase`; `rope_view.rs`: when the rope state starts a movement after his own update, the drawing starts from that movement's phase at once (logged). `rope.rs`: `throw_camera` (the camera at the end of `btoss`), 1 unit test. QUESTIONS Q147 (f), DESIGN, STATUS, provenance.
**Why:** The user's look: Connor's animation snapped (the last frame first) when taking the rope out, throwing, grabbing, getting control back, getting off and putting the rope away; and after the throw the camera should be close to his eye level looking straight up at the rope flying up (open question 2, the user's check in the original).
**Tested how:** `cargo test` (619 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 6000 --at 152500,207968,-1.5708 --command "give RopeAndHook 1,5" --hold 1,0,0,60 --key left:IDCTL_INVBAR_ROPE,80 --aim-point 150000,207968,30000,90 --hold 1,0,0,200,300`; `rope.log`.
**Result:** The cause, from the log: every movement the rope state started was drawn for one update at the phase the old one ended on (1.000, i.e. the new one's last frame). Now `away`, `draw`, `btoss`, `stop`, `bon`, `climb`, `toff`, `taway`, `stop` each "drawn ... from phase 0.000". At 142 the camera goes to (147896, 207586, 9200) (250 behind him, 1,400 up), tilt 1.270, looking along (−0.296, 0, 0.955): toward the wall and up; at 162 the grab moves it to spot 2 as before.
**Still broken / not tested:** **Not looked at by eye.** The throw camera's exact placement is our stand-in (Q147 f). Connor still jumps to the rope's place at the end of `btoss` (rule 15, traced); whether the original hides that is asked (Q147 f).
**Next:** The user: look at the climb again.

## 2026-10-21 M6 6a-6: while Connor is on the rope

**Changed:** `fight.rs`: hits on Connor ignored in the rope state. `weapons.rs`: weapon requests ignored there (keys, slots, scripts; no remark). `active.rs`: characters are told Connor is 100,000 below for "the rope state" (`potions.md` rule 26's code). Viewer: `connor_view.rs` `roping`, every state request refused; `door_view.rs` drops it (not tried again). 1 new unit test. `potions.rs` comment. DESIGN (6a done), provenance, STATUS, FEATURES (a rope section), README.
**Why:** Step 6 of `ropeandhook.md` (rules 27–30), and the end of the 6a plan.
**Tested how:** `cargo test` (all 618 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 4000 --at 152500,207968,-1.5708 --command "give RopeAndHook 1,5" --hold 1,0,0,60 --key left:IDCTL_INVBAR_ROPE,80 --aim-point 150000,207968,30000,90 --command "KQMonster::setState Connor special stop,205" --key left:IDCTL_INVBAR_SHORT,210`; the same with `--command "KQMonster::inventory world INVITEM_Crystal 1 147300 207586 12900,195" --click NoName23,225`; `rope.log`, `characters.log`, `fight.log`, `items.log`.
**Result:** `characters.log` update 91: "characters are told Connor is 100,000 below where he is: the rope state" (the bats then count him 94,000 away). 206: "setState special: refused, Connor is in the rope state; dropped". 210: the hand-weapon slot "ignored, he is in the rope state; no remark". 225, while hanging: the crystal "IN REACH: picked up". Unit test: a hit does no damage and asks no flinch; off the rope it hurts again.
**Still broken / not tested:** **Not looked at by eye.** No creature near a rope place chased him in a run (the report is the same code as Invisible and cutscenes, which were tested with goblin22). A hit while on the rope was run only in the unit test. Rock throws and first person don't exist yet.
**Next:** The user: the rope by eye in Daventry (wizard's house, the cave). A: Q147, the note's open question 2.

## 2026-10-21 M6 6a-5: getting off the rope at either end

**Changed:** `rope.rs`: the climb's end at the top (`toff`, the camera follows, `taway`) and at the foot (the 1,000 / 4,900 drop, gravity on, `boff`, `baway`), then gravity, the step heights, the camera, the weapon (`out` only if he held one), standing; `done`; 2 new unit tests. Viewer `rope_view.rs`: `camera_follow`, `restore_hand` (the weapon's list back at once, then its `out`), `leave`; the record is dropped when done. `door_view.rs`: `weapon_list` visible to the crate. DESIGN, provenance.
**Why:** Step 5 of `ropeandhook.md` (rules 23–26).
**Tested how:** `cargo test -p kq8-core` (481 pass), clippy clean. Viewer, Daventry, `--sound off`, `--command "give RopeAndHook 1,5"`: (up) `--frames 6000 --at 152500,207968,-1.5708 --hold 1,0,0,60 --key left:IDCTL_INVBAR_ROPE,80 --aim-point 150000,207968,30000,90 --hold 1,0,0,200,300`; (down, fists ready) `--frames 7000 --command "move Connor 0 147200 207586 13300 0 0 1.57,10" --key left:IDCTL_INVBAR_SHORT,20 --key left:IDCTL_INVBAR_ROPE,60 --aim-point 150000,207968,30000,70 --hold -1,0,0,220,330`; `rope.log`, `connor.log`, `fight.log`.
**Result:** Up (empty hands): top end at 277 at (147646, 207586, 12800); `toff` 277–297, the camera follows; `taway` 297–332 (moved 405 forward, Q147 d); steps back to 325/325; nothing drawn; standing at 332, landing on the ledge at z 12,795. Down (fists): hanging at 211; bottom end at 297, moved to (148616, 207585, 7920) (1,000 toward his back), traps `ropetrp2 Off`, `ropetrap On` (status top → none → bottom, `HookUp` back); `boff` 297–329, `baway` 329–369, the camera follows; steps unchanged (never saved from the top); the fists' list back, `confists.anm/out` 369–382; standing at (148526, 207614, 7583).
**Still broken / not tested:** **Not looked at by eye.** The hook icon reappears in the place after the spinning mask's (handsOff is on while he gets off: Q147 a, b). A ranged weapon held at the start not run. `taway`'s 405 forward step is our stand-in (Q147 d).
**Next:** 6a-6, while on the rope.

## 2026-10-21 M6 6a-4: climbing the rope

**Changed:** `connor.rs`: the climb update (forward up, back down, anything else frozen; ends at the end it plays toward), 1 unit test. `rope.rs`: `climb_camera` (750 a second, the height in its range), 1 unit test; the climb's end is logged (getting off is step 5). Viewer `rope_view.rs`: each update while he hangs, the camera and aim point follow and every climbing update is logged (key, direction, frame, phase, camera height, range, aim). Test option `--hold F,T,R,FROM,N` (held from update FROM to N), repeatable (`main.rs`, `connor_view.rs`, FEATURES). DESIGN, provenance.
**Why:** Step 4 of `ropeandhook.md` (rules 22–23).
**Tested how:** `cargo test -p kq8-core` (479 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 5000 --at 152500,207968,-1.5708 --command "give RopeAndHook 1,5" --hold 1,0,0,60 --key left:IDCTL_INVBAR_ROPE,80 --aim-point 150000,207968,30000,90 --hold 1,0,0,200,230 --hold 0,1,0,230,245 --hold -1,0,0,260,275`; `rope.log`, `connor.log`.
**Result:** Hanging from 193. Forward 201–230: frame 1 → 30 (phase +0.013 an update, 78 frames in 5.13 s), camera z 12,650 → 14,100 (+50 an update), aim z 7,850 → 9,300. Turn key 231–245: "frozen at frame 30: no forward or back key", heading −1.570 and place unchanged. Back 261–275: frame 29 → 15, camera 14,050 → 13,350. Then "frozen: no key". His place stays (147646, 207586, 12800) throughout.
**Still broken / not tested:** **Not looked at by eye.** Reaching either end only logs (getting off is step 5). The climb's sound cues fire in both directions (the note doesn't say). The camera's top limit was not reached in this run (unit test only).
**Next:** 6a-5, getting off.

## 2026-10-21 M6 6a-3: the click with the rope and the sequence to hanging

**Changed:** `rope.rs`: the rope state (`Climb`, its stages, the camera spots, the height ranges, a `RopeHost` interface; 4 new unit tests with a pretend game). `connor.rs`: walker mode `Rope` (`rope_play`, `rope_hang_at`, `rope_leave`; the climb frozen for now). `connor_update.rs`: no fall, landing, death or push state change in the Rope mode. Viewer: new `rope_view.rs` (`rope_step` after Connor's update: the sequence on Connor, the camera, the rope model's phase and sounds, the traps after a placing); `connor_view.rs` (rope fields, `put`, `rope_movement`, the rope model drawn at his place); `door_view.rs` (a world click with the rope in hand, `connor_placed_by_rope`); `game_camera.rs` `look_from`; `load.rs` loads `Conrope.anm`'s 12 movements in worlds with rope scripts. QUESTIONS Q147 (d) narrowed, (e) added. DESIGN, provenance.
**Why:** Step 3 of `ropeandhook.md` (rules 10–21, 28 in part, 31).
**Tested how:** `cargo test -p kq8-core` (477 pass), clippy clean. Viewer, Daventry, `--sound off --frames 4000 --command "give RopeAndHook 1,5"` with (foot) `--at 152500,207968,-1.5708 --hold 1,0,0,60 --key left:IDCTL_INVBAR_ROPE,80 --aim-point 150000,207968,30000,90` and (top) `--command "move Connor 0 147200 207586 13300 0 0 1.57,10" --key left:IDCTL_INVBAR_ROPE,60 --aim-point 150000,207968,30000,70`; `rope.log`, `connor.log`, `viewer.log`.
**Result:** Foot: click at 90 → the camera stays at (152811, 207770, 9028), `away` 91–97, empty hands, `draw` 97–114, gravity off, `btoss` 114–142, steps 325/325 saved and 0, placed at (147646, 207586, 7800) heading −1.570, the rope model 142–162 (`cr_rope.aud`, `cr_rope2.aud`); grab at (147646, 207586, 7800); camera spot 2 (150144, 210088, 12600) (5,025 vs 5,200 from the camera) looking at the grab; lifted to 12,800; traps: `ropetrap Off`, then `ropetrp2 On` (status bottom → none → top; the first visit's +25 experience); `bon` 162–193; hanging at frame 0, range 12,600..17,350 (nothing above). Top: `away`, `drawdown` 77–94, placed at (147616, 207586, 12820), camera spot 1 (152620, 212582, 10820) at 2,980, `ttoss` 94–173, `ton` 173–210, the half turn to 4.712, hanging at the last frame, range 7,216..10,820 (the terrain at 6,966 below). He stays put while hanging (update 441 same place). Only `taway` has an own step (405).
**Still broken / not tested:** **Not looked at by eye** (how the rope model and the animations line up with the wall). Climbing and getting off are not built: he hangs frozen for good (steps 4–5). A Connor holding a weapon at the start not run (only empty hands). The camera during `btoss` stays (open question 2's stand-in). The order of the trap calls is our stand-in (Q147 e).
**Next:** 6a-4, climbing.

## 2026-10-21 M6 6a-2: the rope button

**Changed:** `connor_inventory.rs`: a click on the rope by the rope status (ignored in the rope state; put back if held; none: the remark only, not held; bottom: held; top: the lock, the camera request, held), `rope_lock`; 1 new unit test (the test item list gets the rope). `active.rs`: `connor_roping`, `rope_first_person_lock`, `Request::RopeCamera`. Viewer `door_view.rs`: the rope camera as a `move KQCamera`. DESIGN, provenance.
**Why:** Step 2 of `ropeandhook.md` (rules 8, 9; replaces the Q110 stand-in "held and the remark").
**Tested how:** `cargo test -p kq8-core` (473 pass), clippy clean. Viewer, Daventry, `--command "give RopeAndHook 1,5"` with: (top) `--command "move Connor 0 147200 207586 13300 0 0 1.57,10" --key left:IDCTL_INVBAR_ROPE,60 --key left:IDCTL_INVBAR_ROPE,90`; (foot) `--at 152500,207968,-1.5708 --hold 1,0,0,60 --key left:IDCTL_INVBAR_ROPE,80` then moved out at 120; (no place) the button pressed at the house's foot before the rope was given. `rope.log`, `camera.log`, `gui.log`.
**Result:** Top: `ropetrp2` sets top and `setRopeCam2 (151584, 210996, 8520)`; at 60 "first-person key locked", the camera moved from (149875, 207575, 10353) to the spot, looking at 1,000 above Connor (z 13795), no camera update after; the rope in hand (`IDBMA_CURS06`); at 90 put back, the lock released, the camera left there. Foot: held at 80; walking out (setRope None) at 121 puts it back (rule 6), pointer `IDBMA_CURS04`. No place: "I am not near anything to climb." (0 0 58 37 1), not held.
**Still broken / not tested:** **Not looked at by eye.** The camera's "flag that keeps it there" is not known (Q147 c). A world click with the rope does nothing special yet (step 3). No first person to force off (logged).
**Next:** 6a-3, the click and the sequence to hanging.

## 2026-10-21 M6 6a-1: the rope commands, the rope status and the hook icons

**Changed:** New `crates/kq8-core/src/rope.rs` (the status, the rope place's settings, the six `KQConner::setRope*` commands, the hook icon wanted, 4 unit tests). `active.rs`: the commands go there (no longer stubs), `connor_rope`, `rope_icon_step` each update, `rope_note` for the new `rope.log`. `potions.rs`: `StatusIcon::Hook`. Viewer: `gui_view.rs` draws the hook icon in its status place; `door_view.rs` writes `rope.log`. DESIGN (the 6a plan, 6 steps), QUESTIONS Q147, provenance.
**Why:** Step 1 of `ropeandhook.md` (rules 1–7).
**Tested how:** `cargo test -p kq8-core` (472 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 2500 --at 152500,207968,-1.5708 --command "give RopeAndHook 1,5" --hold 1,0,0,60 --command "move Connor 0 153000 207968 8000 0 0 0,150"`; the same without the `give`; `rope.log`, `gui.log`, `traps.log`.
**Result:** With the rope: update 31 `ropetrp1.cs`: "status none -> bottom ... place (147646, 207586, 7800), direction (0, 0, -1.570)", "HookUp shown in IDCTL_TIMER_1" (`hookup.pba`, 1 frame); `useropeh.cs` asks for pop-up 145 (still a stub). Update 151 (moved out): "bottom -> none", "HookUp hidden". Without the rope: "setRope bottom: refused: Connor carries no rope (rule 4); status stays none", no icon.
**Still broken / not tested:** **Not looked at by eye** (the icon's place). The icon's place is a stand-in (first free, Q147 a). Top places and `setRopeCam` not run in the viewer (unit tests only). The rope button is still the old stand-in (step 2).
**Next:** 6a-2, the rope button.

## 2026-10-21 The user's check of the Rope and Hook study

**Changed:** `ropeandhook.md` rules 9, 22, 25, 30, 32, "Do not reproduce", "How to check", open questions; STATUS.
**Why:** The user checked the study in the original and answered open questions 1, 3–6.
**Tested how:** The user's check in the original game.
**Result:** Everything matches except the weapon at the end: it is drawn only if Connor held one when the climb started (my reading of the code had suggested a weapon put away earlier could be drawn too). Only forward climbs up, only back climbs down, other movement keys do nothing (the code had read as "any movement but back climbs up"). Putting the rope back at the top leaves the camera where it is. Clicking an item while hanging picks it up. Rock throws on the rope are buggy in the original: we ignore them. Mid-climb saves load fine.
**Still broken / not tested:** Open question 2 (the camera during the throw from below) is still only partly traced.
**Next:** B can implement the rope (M6).

## 2026-10-21 Study: the Rope and Hook (A)

**Changed:** New `docs/behaviour/ropeandhook.md`. Small corrections in `inventory.md` rule 9 (the rope is held at a rope place, not thrown at once; the rock's three refusing states are not the rope), `talking.md` (weapon remark not on the rope), `connor-movement.md` "Turning" (the rope is not one of the four no-lean states), `interface.md` open question 3 (hook icons answered). `QUESTIONS.md` Q110 partly answered; STATUS. Scratch notes with addresses only in `re/` (not committed).
**Why:** The user asked for the study of the Rope and Hook.
**Tested how:** Study only (Ghidra on `Mask.exe`; data: `Conrope.anm`, `Invinfo.itl`, `0.MSG`, every world's rope scripts). No code written, nothing run.
**Result:** 27 rope places (54 trap scripts in 7 worlds), each top 4,840–5,032 above its foot. Traced: the six `KQConner::setRope*` commands, the hook icons, the inventory click, the click in the world, the whole rope state (12 animations in order, camera spots and height range, the 5,000 lift, the 50-per-update climb camera, getting off at either end, weapon back), no damage and no state change while roping. Found an ineffective camera-angle test (every click starts the climb) and a Daventry script that sets its top camera only on the first visit.
**Still broken / not tested:** Nothing checked in the original. Partly traced: the camera placement during the throw from below (open question 2), what "last put away" weapon is drawn at the end (4), mid-climb saves (5), clicks and rocks while hanging (3).
**Next:** B can implement it (M6); the user could check open question 4 in the original (put the sword away, climb, see if he draws it at the top).

## 2026-10-21 M5 5g-9: characters lose Connor during cutscenes

**Changed:** `active.rs`: `connor_hands_off` (set by the viewer each update, `door_view.rs`), `connor_hidden_why` (Invisible or a cutscene), the reported place uses it, each change logged once in `characters.log`. `potions.rs`: Invisible's own lines shortened (the change is logged by the new check). QUESTIONS Q146 (talk; which handsOff), Q145 (b) answered by the user, Q143 marked low priority. DESIGN, provenance. 1 unit test extended.
**Why:** `potions.md` rule 26: the same lowering "during cutscenes"; the user said go ahead.
**Tested how:** `cargo test` (core 468 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 3000 --at 286784,192527,1.5708 --key f3,1 --command "handsOff 1,150" --command "handsOff 0,300"`; `characters.log`.
**Result:** goblin22 chases and attacks; update 151 "told Connor is 100,000 below: a cutscene (handsOff)"; 165 can't see him; 209 gives up, home at 237; 301 "told where Connor really is again"; 315 sees him and is alarmed again.
**Still broken / not tested:** "While Connor talks" not built (Q146 a); the rope doesn't exist yet. Not run through a real Daventry cutscene with a chaser (only the `handsOff` command). Not looked at by eye.
**Next:** A: Q144, Q146.

## 2026-10-21 The user's check of the shield's orbits

**Changed:** This entry; STATUS, FEATURES, provenance.
**Why:** The user looked at the 5g-4 fix in our viewer.
**Tested how:** The user's check by eye.
**Result:** "Now it's correct": three semi-transparent shields circling Connor, facing as in the original.
**Still broken / not tested:** The shield's and Strength's numbers (armour, hand) not yet checked by the user.
**Next:** —

## 2026-10-21 M5 5g-4 fix: three shields, facing out

**Changed:** Viewer `load.rs` loads `orbit1.kq`, `orbit2.kq` and `orbit3.kq` (all in `GAME/Kq`); `potion_view.rs` draws one shield per file and turns each a quarter turn more than its orbit angle. QUESTIONS Q145 (the note says orbit2 and orbit3 don't exist). Provenance.
**Why:** The user's check: the original shows three semi-transparent shields spinning round Connor; ours showed one, turned 90°.
**Tested how:** `kq8-viewer --world Daventry --sound off --frames 600 --command "give Invulnerable 1,10" --key 5,30`; `items.log`.
**Result:** "the shield's orbits appear: 3"; at update 30 orbit1 at angle 0, orbit2 at 2.09, orbit3 at −2.09, each 500 from his `body` part. The model is wide along x and thin along y: before, its wide side lay along the radius (edge-on from Connor); now its flat side faces out.
**Still broken / not tested:** **Not looked at by eye** after the fix. Which side of the shield (its bulge) faces out is our choice (Q145 b). The user checked Invisible's and Clarity's effects; the shield's and Strength's numbers are checked in the logs only.
**Next:** The user: look at the shields again; check the armour and hand numbers.

## 2026-10-21 M5 5g-8: experience when some items are added

**Changed:** `potions.rs`: the list of items with built-in experience (rules 40–41). `connor_inventory.rs`: every add gives it (0.3 s later through a small queue in `timers`, or at once) and logs the help pop-ups 0x83 / 0x84 / 0x80 (not studied: logged only). `active.rs`: the queue and the pop-ups asked. DESIGN (5g done), provenance. 1 new unit test.
**Why:** Step 8 of `potions.md` (rules 40–41; corrects `inventory.md` rule 16: the candle's 25 comes at pickup).
**Tested how:** `cargo test` (core 468 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 600 --command "give Candle 1,10" --command "give Feather 1,20" --command "give Mushroom 1,20"`; `items.log`.
**Result:** Candle added at update 10, "+25 experience" at 15 (0.33 s): level 1 → 2, 15 left. Feather at 20: +1,000 at once, level 3. Help 0x84 at the candle, 0x83 at the mushroom (logged). Unit test: still 15 at update 2000 (never taken back).
**Still broken / not tested:** The help pop-ups themselves (not studied). Picking the candle up in the church by hand not run (the church is walled in for our tests).
**Next:** The user's checks by eye; A: Q143, Q144.

## 2026-10-21 M5 5g-7: the clarity potion

**Changed:** `potions.rs`: Clarity queues `Reveal True` / `False` for the objects with `groupsAlways` bit 2048. `active.rs`: the calls run at the start of the next update; `KQObject::dither` (kept for the viewer); `hidden_now` lets a chest's or scripted object's own hidden state win over its file's `hidden=` (a revealed `wizchest1` stayed undrawn). Viewer: `load.rs` collects the reveal objects at load (every class), `door_view.rs` hands them to the rules, `potion_view.rs` `dither_step` draws dithered objects half see-through (stand-in). QUESTIONS Q144. DESIGN, provenance. 1 new unit test. (The Clarity face came with 5g-5.)
**Why:** Step 7 of `potions.md` (rules 30–32).
**Tested how:** `cargo test` (core 467 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 6500 --command "give Clarity 1,10" --key 7,30`; and `--frames 900 --at 141087,203800,3.1416 --lift 6000 --command "give Clarity 1,10" --key 7,30 --click wizchest1,60`; `items.log`, `doors.log`, `viewer.log`.
**Result:** Reveal objects at load: `keepwall`, `wizchest1`. Update 30: `keepwall.cs` runs `KQObject::dither keepwall 1` (1 part half see-through); `wizchest.cs`: hidden false, collision 0x1 → 0x21. Update 930 (60 s): `Reveal False`, dither 0, the wall back. By the chest: `[wizchest1] Off -> Near (drawn)`, the click opens it and puts out the lady bell.
**Still broken / not tested:** **Not looked at by eye.** How a dithered object really looks is not known (Q144; stand-in half see-through); `dithered=1` ignored. `KQObject::setGroupsAlways` is still a stub, so a script can't add or remove the reveal bit. Objects loaded while playing are never on the reveal list (the note says new ones get no `True`; whether they get `False` is not said).
**Next:** 5g-8, experience at pickup.

## 2026-10-21 M5 5g-6: the invisible potion

**Changed:** `active.rs`: `connor_reported` (100,000 below while Invisible runs), `connor_invisible`, `KQConner::checkInvisible`. `character_ai.rs`: sight, alarm, chase, leash and leaving the active list use the reported place. `potions.rs`: start/end lines in `characters.log`. Viewer: `load.rs` loads the `I_` versions of his textures (entry 0 see-through); `object_light.rs` / `.wesl`: a `cutout` switch (holes where alpha < ½); `potion_view.rs`: the look is worked out from Strength, the face set and Invisible together, with mesh copies (1 pixel in 4) where there is no `I_` picture. DESIGN, provenance. 1 new unit test.
**Why:** Step 6 of `potions.md` (rules 26–29).
**Tested how:** `cargo test` (core 466 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 4000 --at 286784,192527,1.5708 --key f3,1 --command "give Invisible 1,10" --key 8,150`; `characters.log`, `items.log`.
**Result:** goblin22 sees Connor 2,500 away at update 15 and attacks. Invisible at 150: at 165 "can't see Connor (blocked by the ground)", "Connor 100005 away"; at 209 "gives up: has not seen Connor for 45 updates (rule 26); on its way home"; home at 236. Look: "445 materials with an I_ picture, 585 with a mesh copy"; ended at 375 (15 s).
**Still broken / not tested:** **Not looked at by eye** (how the mesh looks; linear texture filtering may soften the dots). The hurt line while invisible (rule 27) not built. The same lowering during cutscenes, talk and the rope (rule 26) not built: waits for the user (DESIGN 5g-6). Dead City's `checkInvisible` not run.
**Next:** 5g-7, clarity.

## 2026-10-21 M5 5g-5: the strength potion and Connor's potion faces

**Changed:** `potions.rs`: Strength's +half to the hand number and its end; `list_bits` (a skin list's bits from the items applied). kq8-data `characters.rs`: `SkinList` (`KQSkinList`: entries, items, exact and best match). Viewer `load.rs`: Connor's skin and face lists, `coniron.bmp` and the face sets loaded at start; `connor_view.rs`: every material of his model kept with its texture name; `potion_view.rs` `look_step`: the iron skin while Strength runs, the face set from the face list each update. DESIGN, provenance. 1 new unit test; test item list fixed (potions' numbers are all 0).
**Why:** Step 5 of `potions.md` (rules 21–25, 30, "Do not reproduce" 3).
**Tested how:** `cargo test` (core 465, data 131 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 3000 --command "give Strength 1,10" --command "give Clarity 1,10" --key 6,30 --key 7,60`; `items.log`, `viewer.log`.
**Result:** Hand 10 → 15 at update 30, back to 10 at 330 (red as in 5g-3). "his skin con0.bmp drawn as coniron.bmp on 448 materials", back at 331. Faces: `cf` at start, `cironf` (bits 8), `cclif` with Clarity (12), `cclf` after Strength (4). Face sets found in Daventry: `chf` and `cclhf` have no pictures in any world.
**Still broken / not tested:** **Not looked at by eye.** First-person arms (`armconfx.bmp`) not done (no first person); the temple armour's skin and armour skins in general not built. Clarity's face now shows, its reveal is step 7. The helmet's face set `chf` doesn't exist in the data: with the helmet our face stays `cf` (logged).
**Next:** 5g-6, invisible.

## 2026-10-21 M5 5g-4: the shield potion

**Changed:** `potions.rs`: the shield's raise (p + ⌊p/2⌋, 50–100), effect on/off, the Temple 4 hit rule, the orbit's place. `connor_inventory.rs`: a potion's effect switches on before the count drops and off after the end's stats event; the record remembers what it added. `active.rs`: `setConnorFlag InTemple4 0|1`. `fight.rs`: hits on Connor use the Temple 4 rule. kq8-data `characters.rs`: `OrbitKeys` (`orbit1.kq`). Viewer: new `potion_view.rs` (the orbit drawn and moved every update), `load.rs` loads `orbit1.kq` and `invulnfx.kqs`, `connor_view.rs` `part_now`. DESIGN, provenance. 3 new unit tests (+ the clippy note of 5g-3's test fixed).
**Why:** Step 4 of `potions.md` (rules 17–20, "Do not reproduce" 1).
**Tested how:** `cargo test` (core 465, data 130 pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 3000 --command "give LeatherArmor 1,10" --command "give Invulnerable 1,10" --key 5,30 --command "give LeatherGloves 1,100"`; `items.log`, `gui.log`.
**Result:** Protection 2 → 50 at update 30; the gloves at 100 → 51; at 330 "protection 51 -> 3 (the 48 it added taken off)". The orbit appears at 30 (1 model part), 301 places logged, 500 from his `body` part, 320 below it, +0.2 rad each update; removed at 331. Unit test: in the last temple 60 is used as 50.
**Still broken / not tested:** **Not looked at by eye** (the orbit's look, size, see-through, where angle 0 points). A hit on Connor with the shield was not run in the viewer (unit test only). The original restores the old value at the end; we take off what was added (on purpose).
**Next:** 5g-5, strength.

## 2026-10-21 M5 5g-3: potion timer icons and red numbers

**Changed:** `potions.rs`: the 7 status icon places (`take_place`, `free_place`), the red switches (`red_numbers`). `connor_inventory.rs`: a potion takes the first free place at its start and frees it at its end. `meters.rs`: the red switches are sent with the numbers. `gui_draw.rs`: `Piece::Lower` (only a picture's lowest rows). Viewer `gui_view.rs`: the status icons drawn as one 336 × 57 picture at the screen's right, standing on the bottom bar; a potion's part left from its `.pba` frame 1, the part used from frame 0 (black square, gold outline); the spinning mask takes a place too and plays at 10 frames a second (was 15); the red number replaces the white one. QUESTIONS Q143. DESIGN, provenance. 2 new unit tests.
**Why:** Step 3 of `potions.md` (rules 35, 37–39).
**Tested how:** `cargo test -p kq8-core` (all pass), clippy clean. `kq8-viewer --world Daventry --sound off --frames 3000 --command "give Strength 1,10" --command "give Invulnerable 1,10" --key 6,30 --key 5,40`; and with `--command "handsOff 1,40"` / `"handsOff 0,80"`; `gui.log`.
**Result:** Strength in `IDCTL_TIMER_1` at update 30, the shield in `_2` at 40; each drops 5 % a second (lowest 45, 43, 40 ... 2 rows of 48 from frame 1); `_1` empty at 331, `_2` at 341. Stats events: "red armour false, hand true", then both true, then each back to false at its end. The mask took `_2` beside a running Strength and left it at handsOff 0.
**Still broken / not tested:** **Not looked at by eye** (where the icons stand, how the red number looks). Which frame is which, and the mask's place, are our reading (Q143). Potions still have no effect (steps 4–7).
**Next:** 5g-4, the shield.

## 2026-10-21 M5 5g-2: the potions' whole-second timer

**Changed:** New `crates/kq8-core/src/potions.rs` (`Running`: item, effect, `Dur`, seconds left, next tick; `Effect` by item type). `active.rs`: `item_timers` replaced by `potions`. `connor_inventory.rs`: applying an item with a duration starts its record; `timers` takes one second off per 15 updates and un-applies it when it would drop below 1, logging each second. DESIGN, provenance. 1 new unit test, 1 updated.
**Why:** Step 2 of `potions.md` (rules 33–34).
**Tested how:** `cargo test -p kq8-core potion` (2 pass). `kq8-viewer --world Daventry --sound off --frames 3000 --command "give Strength 1,10" --key 6,30`; `items.log`.
**Result:** "update 45: 19 s left of 20, 95 %", ... "update 315: 1 s left of 20, 5 %", "update 330: its time is up at game time 22.000", count 0: the item leaves. 300 updates = 20 s.
**Still broken / not tested:** Losing window focus not tested (Bevy keeps updating an unfocused window, so it should keep counting). No saves or world changes yet to keep potions across. No main menu yet to pause it.
**Next:** 5g-3, the timer icons and red numbers.

## 2026-10-21 M5 5g-1: using a health item or potion

**Changed:** `connor_inventory.rs`: a slot click or key 1–8 is refused while Connor is dying or dead (until now only in handsOff); the same potion clicked while it runs does nothing and says so in the log; the use line names the slot. `gui_view.rs`: the viewer's own "dead" stand-in removed. DESIGN (the 5g plan, 8 steps), provenance. 1 new unit test.
**Why:** Step 1 of `potions.md` (rules 1–13).
**Tested how:** `cargo test -p kq8-core connor_inventory` (6 pass). `kq8-viewer --world Daventry --sound off --frames 600 --command "give Strength 2,10" --key 6,30 --key 6,40`, and with `give Mushroom 2` and a −6 health event, keys 1 at 20 and 25; `items.log`.
**Result:** Mushroom: health 4 → 9 → 10 (capped), `healing.aud` both times, count 2 → 1 → 0, then it leaves. Strength at update 30: count 2 → 1, `strength.aud`; at update 40: "already running: nothing (no sound, count stays x1, timer untouched)".
**Still broken / not tested:** Potions still do nothing (steps 4–7). The dying refusal is tested only in the unit test. Not looked at by eye.
**Next:** 5g-2, the whole-second timer.

## 2026-10-21 The user's check of the console

**Changed:** This entry.
**Why:** The user tried the console (typing line, `list`, `goto`, `heal`, `kill`, ...) while playing.
**Tested how:** The user's check, typing in the terminal.
**Result:** "Works very nicely." The typing line at the bottom of the terminal is now checked by hand.
**Still broken / not tested:** As in the entries below (`goto` can land in a wall; items of later worlds may have no model).
**Next:** —

## 2026-10-21 Console: list, goto, heal, kill and a fuller help

**Changed:** New `crates/kq8-viewer/src/console_words.rs`: `help` grouped by use, `list` (item types by category; creatures, doors, chests, objects of this world, nearest first), `find`, `inv`, `stats`, `heal`, `xp`, `god`, `ai`, `goto`, `spawn`, `kill <name>` / `kill near [distance]`, `where`. `fight.rs`: `console_kill` (the death of a hit, no alarm), 1 unit test. `console.rs` uses them. DESIGN (step 3), README, FEATURES, provenance.
**Why:** The user asked for a more complete help and commands like `list items`, `list weapons`, `list creatures`.
**Tested how:** `cargo test` (new test `the_console_kill_dies_as_from_a_hit_once`), clippy clean. Piped into `cargo play --world Daventry --sound off --frames 200` / `900`: the commands above; `console.log`, `fight.log`, `doors.log`, `connor.log`.
**Result:** `list weapons` 33 types with their damage; `list creatures` 57 with health and distance; `find wiz` 5 things. `xp 100`: level 1 → 2. `kill goblin22`: dies at once, drops a mushroom, at update 31 its body and 3 experience to Connor. `kill near 30000`: 4 creatures. `kill Wizard`: refused (never fights). `goto Wizard`: Connor moved 1,000 from him, turned to face him (heading −1.559). `spawn Crystal 3`: lying 600 in front of him. Help examples run: `sendEvent 0 Wizdoor KQDoorEvent 1` opens the door, `setCombatLevel 3`, `KQSound::play eat.aud` result 0.
**Still broken / not tested:** Not looked at by eye. `goto` can drop Connor into a wall or onto a roof (it is a jump, not a walk). Items of later worlds may have no model in Daventry. `list` also shows the game's placeholder types (`Fists`, `NoRanged`, `Lava`, ...).
**Next:** The user: try the commands while playing.

## 2026-10-21 Console: a typing line at the bottom of the terminal

**Changed:** `console.rs` (`screen`): in a real terminal the last line is the typing line, a divider above it, and the game's output scrolls only above (scrolling region); keys read one at a time with `crossterm` (new dependency, MIT; `Cargo.toml`, CREDITS), Enter / Backspace / Esc / Up / Down / Ctrl+C. `main.rs` gives the terminal back when the window closes; a panic does too. Piped input still reads plain lines. DESIGN (step 2), README, FEATURES, provenance.
**Why:** The user: the game's output kept pushing the half-typed command up the screen.
**Tested how:** clippy clean; `printf "give templesword
where
" | cargo play --world Daventry --sound off --frames 100` (the plain path), `console.log`.
**Result:** The plain path works as before (`give templesword` result 0, `where`).
**Still broken / not tested:** The typing line itself is **not tested**: it only turns on in a real terminal window, which this session can't type into. Unix terminals not tried (raw mode there changes how line ends print). Left/Right to move inside the text not done.
**Next:** The user: run `cargo play` in a terminal and type.

## 2026-10-21 A command console in the terminal

**Changed:** New `crates/kq8-viewer/src/console.rs`, wired in `main.rs`: lines typed in the terminal that started the game run as script commands at the next update (the same runner as `--command`), with `help` and `where` added. The terminal shows each line, the log lines it wrote and its result; also `kq8-artifacts/logs/console.log`. DESIGN (plan, done), README, FEATURES, provenance.
**Why:** The user asked for a console to type commands like `give templesword` or `move`.
**Tested how:** clippy clean. `printf "help
where
give SmCrossbow
give templesword
move Connor 3 0 0 2000
where
frobnicate 1 2
" | cargo play --world Daventry --sound off --frames 300`; `console.log`, `connor.log`.
**Result:** All lines ran at update 0. `give templesword`: INVITEM_TempleSword carried, result 0. `move Connor 3 0 0 2000`: `connor.log` "moved Connor from (277744, 199295, 13633) ... to (277744, 199295, 15633)", then he falls. `frobnicate`: "unknown command". The game's own terminal output kept printing.
**Still broken / not tested:** Not typed by hand (input was piped in); not looked at by eye. `where` right after a `move` in the same update still shows the old place (the move is applied later in the update). No in-game text box.
**Next:** The user: try it by typing in the terminal.

## 2026-10-20 A: the user checks potions in the original

**Changed:** `potions.md`: rules 34, 35, 37, "How to check" and open questions updated from the user's check.
**Why:** The user tried the note's "How to check" list in the original.
**Tested how:** The user's check in the original game (Daventry).
**Result:** Checks 2–6 correct. The church candle raises the experience bar a little and it never drops. The timer icon is cut away from the top down, the used part showing a black background and gold outline. Only the red number is seen. The timer keeps running when the window loses focus (the note had guessed it stops); only Esc stops it.
**Still broken / not tested:** Missing face combinations (needs a late-game save); the help pop-ups (not this topic). Strength ending during Invisible, checked after: the mesh stays and the iron look goes, nothing wrong visible (open question 5 closed).
**Next:** B: build `potions.md`.

## 2026-10-20 A: health items and potions

**Changed:** New `docs/behaviour/potions.md`. `inventory.md` rules 16, 18 and open question 2, `interface.md` rule 32 point there (the candle's rule corrected). QUESTIONS Q109 partly answered. STATUS. Scratch notes in `re/notes.md`.
**Why:** The user asked for the study of health items and potions (what B needs for the potion effects, timers, icons and red numbers).
**Tested how:** Traced in `Mask.exe` with Ghidra; checked against the data (`Invinfo.itl`, `Connor.kq` skin and face lists, `orbit1.kq`, the `Reveal` scripts, `I_*` / `cclf*` / `cironf*` / `coniron` textures in the worlds' `8bit.vol`). Not checked in the original game.
**Result:** Use refused only in `handsOff` or dying; no movement or remark; a running potion can't be stacked or restarted. Shield: protection + half (50–100), halved in Temple 4, one `invulnfx` orbit; Strength: hand + half, iron skin and face; red numbers on both. Invisible: Connor's reported position is 100,000 below him, so characters lose him; drawn as a mesh. Clarity: `Reveal True/False` to objects with group bit 2048. Timers: whole seconds of game time, paused by the main menu, kept over world changes and saves; drain icon in `IDCTL_TIMER`. Candle: +25 experience at pickup, never taken back. Found: the shield's end overwrites protection (Do not reproduce 1).
**Still broken / not tested:** How the used part of a timer icon looks; whether the white number hides under the red one; missing face combinations (helmet + Strength); the help pop-ups. None checked in the original (note's "How to check").
**Next:** The user: optional checks in the original (note's "How to check" 2–7). B: build `potions.md`.

## 2026-10-21 The user's check: the axe cutscene looks just like the original

**Changed:** This entry; Q141 marked checked.
**Why:** The user played the axe cutscene after the Q141 fix.
**Tested how:** The user's check by eye.
**Result:** "Now it looks just like the original": he stops, draws the axe, walks out, and the cutscene and the camera play as in the original.
**Still broken / not tested:** The axe is still taken from the windmill floor (Q142, `checkDistance`).
**Next:** A: Q142 (and Q140).

## 2026-10-21 Axe cutscene: Connor draws the axe, then walks out (Q141, the user's check)

**Changed:** `connor_view.rs` (`ready_set`, `weapon_step`, `held_walk`): a script's walk asked while a script's weapon swap starts (`ax.cs`: `setDoMode 0`, then `walkTo` in the same event) waits until the swap is over, then starts. Replaces the stand-in of earlier today (the set switched at once, no draw). QUESTIONS (Q141 answered by the user's check, built), provenance.
**Why:** The user's check in the original: in that cutscene Connor stops, draws the axe, then walks outside; ours didn't draw it.
**Tested how:** `cargo test --workspace`, clippy clean. Viewer: `cargo play --world Daventry --sound off --at 145000,249800,-1.5708 --no-ai --god --click Winddoor,5 --click ax,40 --hold 1,0,0,150 --frames 6000` (the door opened first, as in play); `connor.log` (copy `connor-q141-draw-then-walk.log`), `doors.log`.
**Result:** Update 42: the walk is held; his `away`, then `conaxe.anm/out` with `Weapon.aud`. Update 58: the swap is over, the walk starts. Update 84: the walk is done (1,456 of 1,414), cue 20; the cutscene goes on to `handsOff 0` at update 594. With the door closed (an earlier test) the walk stops head-on at `Winddoor`; in play the door is open.
**Still broken / not tested:** Not looked at by eye. No unit test (the hold lives in the viewer's Connor). Only a script's request with a script's walk is held; other combinations are not traced. The axe from the floor (Q142).
**Next:** The user: the cutscene by eye. A: Q142.

## 2026-10-21 The user's check of the axe cutscene: works

**Changed:** This entry; Q141 notes the user's check.
**Why:** The user played the axe cutscene and the fight after it, after the trap and camera fixes.
**Tested how:** The user's check by eye.
**Result:** "Working correctly now." The cutscene and the fight after it work almost perfectly; the camera's jump back up during `cdhm2` is gone (before the fix it jumped up, looked down, flew down a little, then changed back).
**Still broken / not tested:** The axe is still taken from the windmill floor (Q142, `checkDistance`). The axe appears with no draw animation and the walk out stops at `Winddoor` (stand-ins, Q141).
**Next:** A: Q142, Q141, Q140.

## 2026-10-21 The camera no longer jumps back up during the axe cutscene

**Changed:** `camera_flight.rs`: a flight's last update adds its step like every other update (exactly on the spot when nothing moved the camera), instead of snapping to the spot. So when `move KQCamera` places the camera during a flight, the remaining steps go on from there and the flight ends away from its spot. Before, the last update snapped back to the spot. 1 new test; the log wording in `game_camera.rs`; provenance.
**Why:** The user's check of the axe cutscene: after Connor walks out looking at the axe, the camera jolts upwards and looks straight ahead. In the user's `camera.log`: at the end of `cdhm1` (`ax.cs` lines 40–42) the camera is put high (19,550), tilted down, and flies 10 updates toward (139760, 249700, 17550); `cdhm2` (line 45) moves it to (137200, 250000, 16500) after 8 of them; our last update then snapped it to 17,550. `camera.md` rule 6 ("`move KQCamera` during a flight ... the flight's remaining steps are then added from the new place, so it ends away from the spot").
**Tested how:** `cargo test --workspace` (458 core tests), clippy clean. Viewer: the cutscene run of the entry below; `camera.log` (copy `camera-axe-cutscene.log`).
**Result:** The flight now ends at (137350, 249919, 16445): the two steps after the move, then the height rule's lift to ground + 1,400 (16,100 → 16,445). No jump up to 17,550. An undisturbed flight still ends exactly on its spot (unit test; the watermill flight test unchanged).
**Still broken / not tested:** Not looked at by eye. The user thought the camera should look up at the sky there. By the script it looks steeply down from high up at the end of `cdhm1` (tilt −1.15, looking 66° down), then the `cdhm2` shot (tilt −0.24). Whether our tilt sign for `move KQCamera` matches the original has not been checked for so steep an angle: needs the user's look.
**Next:** The user: the cutscene by eye again, especially the shot at the end of `cdhm1`.

## 2026-10-21 The axe cutscene starts: a trap loaded while playing has its box

**Changed:** A `KQTrap` prepared for a script's `loadKQ` (Daventry: `henchtp.kq`, loaded by `ax.cs` when Connor takes the axe) is built at load like any trap, switched off; the `loadKQ` switches it on (`load.rs` `dormant_traps`, `door_view.rs`, `active.rs` `load_kq`); no `Init` at the start for it (it gets its `Init` at the `loadKQ`). **Stand-in (Q141):** a script's weapon request while a scripted walk runs switches the weapon's set at once and plays nothing, so the walk goes on (`connor_view.rs` `ready_set`). QUESTIONS (Q141), provenance.
**Why:** The user: after taking the axe and walking out, the cutscene never started. The trap that starts it came in without a box ("prepared copies have no collider; not done"), so Connor's walk never touched it. Then, once it fired, the axe's swap (from `setDoMode 0`) cancelled the scripted walk that follows it in the same event, so the walk's cue never came and the cutscene hung with `handsOff` on.
**Tested how:** `cargo test --workspace` (457 core tests, 1 new: a trap loaded while playing), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 144500,249800,-1.5708 --no-ai --god --click ax,30 --hold 1,0,0,150 --frames 6000`; `traps.log`, `doors.log` (copy `doors-axe-cutscene.log`), `connor.log` (copy `connor-axe-cutscene.log`).
**Result:** Update 41: `Henchtp: On Connor`, cue 5 to `ax`; the cutscene runs: camera cuts, `setDoMode 0` (the axe's set at once), the walk, cue 20 at update 49, Connor's `henchman.anm` list, the battle music, the Henchman's lines and `cdhm5`, and at update 559 `handsOff 0`, the camera follows Connor again, `ax` deleted. `henchtp.cs` gets `Init` once (at the `loadKQ`).
**Still broken / not tested:** Not looked at by eye. The walk out runs head-on into `Winddoor` and ends early (the cue still comes; asked in Q141). The axe appears in his hand with no draw (stand-in, Q141). The axe is still taken from the floor (Q142, `checkDistance`). The test click skips the distance check, and the test walks in from the west, not out of the windmill's door.
**Next:** The user: the cutscene by eye. A: Q141, Q142.

## 2026-10-21 The user's check of ranged weapons (5f); the axe can be taken from the floor (Q142)

**Changed:** QUESTIONS: new Q142 (`checkDistance`). STATUS, FEATURES: 5f checked by the user. This entry.
**Why:** The user checked the crossbow against the original, and reported that Connor takes the windmill's axe from the floor, where the original says he can't reach it (the hay bale must be pushed under it and climbed).
**Tested how:** The user's check by eye. The axe: `cargo play --world Daventry --sound off --at 145910,249400,0 --no-ai --god --click ax,40 --frames 600`; `doors.log` (copy `doors-q142-axe.log`).
**Result:** Crossbow: "just like the original". Axe: `ax.cs` line 4 runs `checkDistance Connor ax 1700`, which we don't have ("ERROR unknown command"); an unknown command leaves 1 in `$Console::Return`, which the script reads as "close enough", so the axe is given and the cutscene starts. No behaviour note describes `checkDistance` (what distance, between what, which result), so it is asked as Q142 instead of guessed.
**Still broken / not tested:** The axe puzzle until Q142 is answered and built. The axe cutscene itself (Q141) not run by the user yet.
**Next:** A: Q142 (and Q140, Q141). B: build `checkDistance` from the answer, then test pushing the bale under the axe and taking it from the bale.

## 2026-10-20 M5 5f: end of the ranged-weapons session (B)

**Changed:** STATUS.md ("Where we are", the 5c and 5f stand-ins, the next step, the open questions), QUESTIONS (Q138, Q139 marked built), DESIGN (5f done).
**Why:** `docs/SESSIONS.md`, the end of an implementation session.
**Tested how:** — (documents only).
**Result:** 5f-1 to 5f-7 are in, one commit each. Open for A: Q140 (the bolt's shape, Connor's aim height 975 vs 1,200, the frame at the cue), Q141 (a script's walk during the swap), `ranged-weapons.md` open questions 1–5.
**Still broken / not tested:** Nothing in 5f looked at by eye or compared with the original.
**Next:** The user: `ranged-weapons.md` "How to check" 1–9 in ours. B: `potions.md`.

## 2026-10-20 M5 5f-7: the swing click picks its target as traced

**Changed:** With a hand weapon ready, the click line is tested against the characters, objects and buildings whose type has bit 0x40 or 0x20 (`weapons.rs`: `SWING_PICK`). The Wizard and the Knight (type bit 0x8000) get the plain click. Nothing or a building: he turns to the click point and swings at once. Anything else within his reach (2,350, 3D between the positions) becomes his target, and the target stays for later swings; more than 45° off, the swing waits until 8 updates of turn are left. `--swing NAME,N` clicks NAME the same way. FEATURES, provenance, DESIGN.
**Why:** `ranged-weapons.md` rule 32 (Q139 a–d) replaces 5c fixes 3–5's stand-ins.
**Tested how:** `cargo test --workspace`, clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --god --key left:IDCTL_INVBAR_SHORT,40 --swing goblin22,60 --aim-point 289284,190100,14100,90 --frames 1500`; barrel.12 again (`--at 188861,270713,0 --key left:IDCTL_INVBAR_SHORT,40 --swing barrel.12,60 --frames 1000`, `fight-5f7-barrel.log`); `fight.log`, `items.log`.
**Result:** goblin22, 1,154 away and 130° off: his target; the swing waits and starts with 1.80 rad of turn left; it hits. The ground click at update 90 picks nothing: his target stays goblin22, `attackb` starts at once and hits goblin22 (40° off at `Do`). barrel.12 (1,522 away) is his target, bursts, and its four crystals fly.
**Still broken / not tested:** Not looked at by eye. The click point when the line meets nothing is still where it meets the level of his feet (the note doesn't say). How objects take damage is still the stand-in (Q139 e).
**Next:** The user's check of 5f (the note's "How to check").

## 2026-10-20 M5 5f-6: the moving shot

**Changed:** `connor_view.rs`: a shot asked while Connor walks or runs plays no `attack`: if 0.5 s (his `attackDelay`) have passed since the last shot, the bolt leaves at once from the `arrow` part in the frame playing; otherwise nothing. A moving shot while running switches the run off; it comes back 1 s after the last moving shot. The end of a standing `attack` sets the next moving shot 0.5 s later. `door_view.rs`: the item's use sound (`crossbow.aud`) plays flat. FEATURES, provenance, DESIGN.
**Why:** `ranged-weapons.md` rule 20, "Moving".
**Tested how:** `cargo test --workspace`, clippy clean. Viewer: `cargo play --world Daventry --sound off --at 146500,248928,0 --no-ai --god --command "give SmCrossbow",20 --key left:IDCTL_INVBAR_LONG,30 --hold 1,0,1,140 --aim-point 146500,236000,15500,60 --aim-point 146500,236000,15500,63 --aim-point 146500,236000,15500,70 --aim-point 146500,236000,15500,72 --frames 1800`; `fight.log` (copy `fight-5f6-moving.log`), `sound.log`.
**Result:** Update 60: 79° off, waits for the turn. 63: a new click (39° off) replaces it; running: the bolt leaves at once from `stoprun`'s part (−187, −351, 744), `crossbow.aud` plays, he walks. 70: "less than 0.5 s ago: nothing". 72: walking, the bolt leaves from `walk` frame 4 (−135, −588, 868; the note's range). 87: the run is on again (1 s after 72). (The camera's line to the test point met the windmill, so the bolts aimed at its wall.)
**Still broken / not tested:** Not looked at by eye. The run restored the simpler way (1 s after the last moving shot), as "Do not reproduce" allows; the original also restores it at a click while standing (not built). No unit test for the moving shot (it lives in the viewer's Connor).
**Next:** 5f-7, the swing click's target.

## 2026-10-20 M5 5f-5: Connor shoots the crossbow (standing)

**Changed:** With the ranged weapon ready a left click is a shot click (`weapons.rs`: `mask_pick`, `steepness_limit`, `connor_shoot`; viewer: the click, `ShotClick`, `Shot`). An item on the pointer wins, also over the swing click; Ctrl gives the plain click. The talk pick sends a click on the Wizard or the Knight (type bit 0x8000) to the plain click. The aim pick finds the creature, object, building, drawn ground or the line's far end; the aim is never steeper than 1 rad from the vertical. Connor turns to it; more than 45° off, the shot is asked when 3 updates of turn are left. Standing, `attack` plays if his movement can be cut short; at its `do` cue the bolt leaves from the `arrow` part of the frame at the cue (kept as his shoot offset), aimed at the aim point, with his ranged damage. New test option `--aim-point X,Y,Z,N`. README, FEATURES, provenance, DESIGN.
**Why:** `ranged-weapons.md` rules 12–18, 20, 22–25.
**Tested how:** `cargo test --workspace` (456 core tests, 2 new: the steepness limit, Connor's bolt start and hit), clippy clean. Viewer (each after `--command "give SmCrossbow",20 --key left:IDCTL_INVBAR_LONG,30 --no-ai --god`): `--at 289284,190300,0 --aim goblin22,70 --aim goblin22,100`; `--at 289284,190300,0 --aim-point 289284,190100,14100,70 --aim-point 289284,190100,14100,72`; `--at 192300,198274,-1.5708 --aim Wizard,70`; `--at 146500,248928,0 --aim-point 146500,248700,15210,70 --aim-point 146500,240000,15400,110` (`fight-5f5-ground.log`); `fight.log`, `viewer.log`, `conversation.log`.
**Result:** goblin22 133° behind: he turns, the shot is asked with 0.69 rad of turn left, `attack` starts; at its `do` (phase 0.2222) the bolt leaves from (−231, −832, 1,067), frame 2 (the note's number), and hits goblin22 after 2 updates: 15 → 11; the second shot (21° off) at once: 11 → 7. `attack` ends 7 updates after it starts; no extra delay. A click on the ground at his feet is moved out by the steepness limit and the bolt is removed at the floor grid after one update. A second click during `attack`: "can't be cut short: no shot". The Wizard: the talk pick gives the plain click, the conversation starts. Shots at the windmill stop at its wall, no damage.
**Still broken / not tested:** Not looked at by eye. Moving shots (5f-6). The right button held (open question 2) not done. The click during a shot still turns him toward the new point (the note's order: the turn first, then the shot asked), not checked in the original.
**Next:** 5f-6, the moving shot.

## 2026-10-20 M5 5f-4: the Small Crossbow can be readied

**Changed:** The loader reads `Connor.kq`'s `LongAnimList`, `shootOffsets` and the start values (`projectileFile`, `shootOffsetX/Y/Z`), loads the ranged weapons' movement lists the world can need (`conscbow.anm`, found in `MONSTERS.KQ`'s drop), their `attack` / `away` / `away2` / `out`, and the bolt files and shapes of the ranged items. A `LongWeapon` going into Connor's inventory sets his projectile file and shoot offset; picked up with the ranged mode ready, it becomes the ready one at once. With the ranged mode ready his movements come from its list. FEATURES, provenance, DESIGN.
**Why:** `ranged-weapons.md` rules 10, 17, 21 (5f-3 built the modes and the swap).
**Tested how:** `cargo test --workspace`, clippy clean. Viewer: `cargo play --world Daventry --sound off --at 146500,248928,0 --no-ai --god --command "give SmCrossbow",20 --key left:IDCTL_INVBAR_LONG,30 --key delete,70 --key delete,110 --key alt,150 --key delete,190 --key left:IDCTL_INVBAR_LONG,230 --frames 1800`; `fight.log` (copy `fight-5f4-crossbow.log`), `gui.log` (copy `gui-5f4-crossbow.log`), `viewer.log`.
**Result:** The crossbow gives `Prscbolt.kq` and the offset (−315, −327, 1,027). Its slot: his `away` then `conscbow.anm/out`, pointer `IDBMA_CURS05`. Delete: the crossbow's `away`, the fists' `out` (not `draw`), pointer CURS03. Delete: the fists' `away`, the crossbow's `out`. Alt: the crossbow's `away`, his own `out`, play pointer. Delete with nothing ready: the crossbow (put away last). Its slot again with it ready: put away. As the note's "How to check" 1–2.
**Still broken / not tested:** Not looked at by eye. Clicking with the crossbow ready doesn't shoot yet (5f-5); it is a plain click. Picking it up from `sprgnbow`'s drop not run (the test gives it with `give`). Picking up a second ranged weapon with the ranged mode ready not run (none in Daventry).
**Next:** 5f-5, the standing shot.

## 2026-10-20 M5 5f-3: weapon modes, Delete and Alt, the swap as traced

**Changed:** New `kq8-core/src/weapons.rs`: Connor's weapon mode (hand, ranged, none) and the request rules. A weapon slot asks for its item's mode; **Delete** swaps hand and ranged (with nothing ready, the one put away last); **Alt** puts the weapon away; the ready one asked again is put away; ranged without a real ranged weapon asks for the hand one. `KQConner::setDoMode` works (it was a stub), `getDoMode` gives the mode. The viewer takes a request only while Connor stands or moves (otherwise it is dropped and logged) and plays the swap in rule 6's order: Fists from / to none `draw` / `undraw`; otherwise `away` in the old list, the switch, `out` from the new list (Shift: `away2`). The pointer follows the mode at the switch (sword, bow, play). `--key delete,N`, `--key alt,N`. README keys, provenance, DESIGN.
**Why:** `ranged-weapons.md` rules 1–9, 11, 12 replace the readying stand-ins (Q138).
**Tested how:** `cargo test --workspace` (454 core tests, 4 new: Delete, Alt and slots, scripts, the requests taken or dropped). Q141 asked, clippy clean. Viewer: `cargo play --world Daventry --sound off --at 146500,248928,0 --no-ai --god --key delete,30 --key delete,60 --key alt,90 --key delete,140 --frames 1500`; the same with `--key delete,30 --jump 80 --key alt,83 --command "KQConner::setDoMode 4",150 --command "KQConner::setDoMode 0",200` (`fight-5f3-requests.log`); the fists' swing test at goblin22 again; `fight.log`, `connor.log`.
**Result:** Delete at 30: fists, `draw` (case 1). Delete at 60: "no real ranged weapon: nothing". Alt at 90: `undraw`, his own set at update 97 (case 2). Delete at 140: the fists again. Alt during the jump: "dropped, he is not standing or moving (mode jump)". `setDoMode 4` puts the fists away, `setDoMode 0` draws them. goblin22 still dies from two punches after a slot click.
**Still broken / not tested:** Not looked at by eye. Cases 3–4 (`away2`, `away` → `out`) need a ranged or other hand weapon: 5f-4. Shift for `away2` is not reachable from a test option. A request during a swing waits for it to end (open question 1, ours). `setDoMode` during a jump doesn't end the jump. The axe cutscene's two lines run by hand (`--at 142879,250000,-1.57 --command "give SmallAx",30 --command "KQConner::setDoMode 0",60 --command "KQConner::walkTo 141600 249800 ax 20 0",60`): the swap starts (case 4: his `away`, then the axe's `out`), but the walk cuts it at the next update and the axe's list switches at once (`fight-5f3-axe.log`); what the original does is Q141. Before this step `setDoMode` did nothing there. The cutscene itself not run.
**Next:** 5f-4, the crossbow readied.

## 2026-10-20 M5 5f-2: a character's bolt starts at its `arrow` model part

**Changed:** The loader keeps, for each frame of each character movement, the named model parts and their places (`Shape::part_places`, `DoorMovement::frame_parts`). At a ranged character's shot, the bolt file's part (`meshName=arrow`) in the frame at the `do` cue gives the start offset; it is kept for later shots (fallback 0, 0, 0). The start is the character's position + the offset turned with it. `anim.log` lists the `arrow` part per frame of each movement that has one. Q140 (c) asked. Provenance, DESIGN.
**Why:** `ranged-weapons.md` rule 24 replaces the 3/4-of-its-height stand-in.
**Tested how:** `cargo test --workspace` (450 core tests, 1 new: the part at the cue's frame, then kept), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 146500,248928,1.5708 --god --frames 400`; `fight.log` (copy in `fight-5f2-windmill.log`), `anim.log`.
**Result:** `sprigwnd1`'s offset is (−55, −1,367, 954), from `spbattck` frame 2 of 19: the note's number. The frame one update later (the one shown when the cue is noticed) would give (176, −1,468, 1,098); which one the game uses is Q140 (c). Its first shot, made while it is still turning toward Connor, starts inside the fence `Rfence` and hits it at once (a start inside counts); the next one hits Connor after 1,245.
**Still broken / not tested:** Not looked at by eye. Characters' own `shootOffsets` are not read (none in Daventry). The frame choice (Q140 c).
**Next:** 5f-3, weapon modes and the swap.

## 2026-10-20 M5 5f-1: the bolt's flight as traced (the bow spriggans)

**Changed:** Plan 5f (ranged weapons, 7 steps) in `docs/DESIGN.md`. `fight.rs`: a bolt is turned once toward its aim (heading and pitch) and moves 533 units per update (8,000 / 15), no gravity. Its move is swept with its `BOUNDS.KQ` box's ball stack against the buildings, the objects and characters its mask (0x1F) covers and Connor; never its shooter. At the first touch it hits and is removed; otherwise it is removed at or below the floor grid, or past `maxDistance`. Immune bit 0x4000: a hit of 0 and `ProjImmune` to the script. Objects get 5c fix 5's barrel stand-in. A character aims at Connor + 0.75 × his box top (975), led as before. `ProjectileKeys` reads `colliderMask`, `weaponType`, `meshName` and the note's defaults; the loader adds the bolt's box. The viewer draws the bolt pitched, its model's front along the flight. Q140 asked. Provenance, FEATURES.
**Why:** `ranged-weapons.md` rules 25–31 replace 5c-9's stand-in flight.
**Tested how:** `cargo test --workspace` (449 core tests, 3 new: straight flight and the first character hit, floor grid / range / shooter spared, the immune bit), clippy clean. Viewer at the windmill: `cargo play --world Daventry --sound off --at 146500,248928,1.5708 --god --frames 900` (Connor stands) and `--at 146500,248928,0 --hold 1,0,1,60 --god --frames 1200` (he runs); `fight.log`.
**Result:** Standing: `sprigwnd1` shoots at update 31, then every 26 updates (1.73 s); each bolt is pitched −3.8°, moves 533 per update and touches Connor at update 5, 2,615 from its start (11 damage, ignored by god mode). Running: bolt1 touches the windmill's wall after 3,112, the others Connor after 3,300.
**Still broken / not tested:** Not looked at by eye. The bolt still leaves from 3/4 of the shooter's eye height (5f-2 next). The bolt's own shape (one ball of radius 235) and the aim height (975 vs the note's 1,200) are open: Q140. Fire / ice bolts' sparks (`emitter`) not drawn (none in Daventry). No character in Daventry with the immune bit is a fighter; `ProjImmune` reaches the Wizard's / Knight's script only if they are hit (not run).
**Next:** 5f-2, the bolt starts at the shooter's `arrow` part.

## 2026-10-20 A: the user checks Azriel's Hammer in the original

**Changed:** `azriel-hammer.md`: rules 2 and 5, "How to check", Do not reproduce and open questions updated from the user's check.
**Why:** The user tried the "How to check" list in the original.
**Tested how:** The user's check in the original game.
**Result:** Checks 1–5 as described. No throw while jumping, falling, landing or stumbling, not queued. The hammer leaves from in front of Connor, so the start is the fallback offset (−41, −605, 1,226), not the model part. The 0-damage worlds can't be reached with the hammer in normal play (it is traded away in Swamp); a default base damage for every weapon is an M7 idea (cheats).
**Still broken / not tested:** — (the round trip: the user timed ~1.5 s in Swamp by hand, ± a couple of hundred ms; the files give 1.33 s there, within that error; open question 4 closed).
**Next:** B: build it when Dead City is reached.

## 2026-10-20 A: Azriel's Hammer

**Changed:** New `docs/behaviour/azriel-hammer.md`; `ranged-weapons.md` rule 19 and open question 5 point there. Scratch notes in `re/notes.md`.
**Why:** The user asked for the study of Azriel's Hammer.
**Tested how:** Traced in `Mask.exe` with Ghidra; checked against the data (`prhammer.kq` in all 12 worlds, `Conhamr.anm`, `Ch_atack.kqs`, Dead City `lever1.cs`). Not checked in the original game.
**Result:** A click throws (no moving throw, no waiting for the turn): `handsOff`, `attack`, hammer at its `do` cue, `wait`, `catch`. The hammer flies straight, hits one thing (damage = `modifier` + ranged damage, capped to `minDamage`–`maxDamage`; `lever1` / `tmblrcyl` run `lever1.cs`), turns back at that touch, its range or the ground, flies back along the same line touching only Connor, and is always caught. Found: 9 worlds' `prhammer.kq` lack the damage keys, so by the defaults the hammer does 0 damage there.
**Still broken / not tested:** The start frame at the `do` cue; whether the throw is refused during a jump; the 0-damage worlds need the user's check / decision.
**Next:** The user: check in the original (note's "How to check"), decide on the 0-damage worlds. B: build it when Dead City is reached.

## 2026-10-20 A: the user checks ranged weapons in the original

**Changed:** `ranged-weapons.md` "How to check": the user's check recorded.
**Why:** The user compared the note with the original game.
**Tested how:** The user's check in the original (Daventry).
**Result:** Everything works pretty much as the note describes.
**Still broken / not tested:** The note's open questions 1–5 are unchanged.
**Next:** B: build `ranged-weapons.md`.

## 2026-10-20 The user's check of thrown items (5e): matches the original

**Changed:** QUESTIONS Q137 marked built, with the user's check; this entry.
**Why:** The user compared the barrel's crystals with the original.
**Tested how:** The user's check by eye, in our build and the original.
**Result:** "Pretty much exactly like the original", the tumble included (so `thrown-items.md` open question 2, the "slow" tumble, is settled by eye; for A to close in the note).
**Still broken / not tested:** Items bouncing off doors or chests not run (no throw reaches one in Daventry).
**Next:** —

## 2026-10-20 M5 5e-3: thrown items tumble about one axis, as traced

**Changed:** `item.rs`: a thrown item's tumble is one random number from −0.5 to +0.5 rad **per update**, drawn at each throw (and each throw again). It turns about its own x axis by that much every update until the flight ends, also after it touches something. At the end its angles are 0, 0, 0. The stand-in (three axes, 90–180° per second, back to its old turn) is gone, with `flight_rest`. `items.log` shows the turn each update. Provenance, DESIGN (5e done).
**Why:** `thrown-items.md` rules 2, 3, 9, 10 (Q137 e).
**Tested how:** `cargo test --workspace` (446 core tests; the old tumble test replaced), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 188861,270713,0 --key left:IDCTL_INVBAR_SHORT,40 --swing barrel.12,60 --frames 1000`; `items.log`.
**Result:** The four crystals turn 0.15, 0.19, 0.44 and −0.21 rad per update (about 130°, 163°, 378° and −180° per second), the same step every update, and end flat with no turn. The throws differ from 5e-1/5e-2 because one random number is drawn for the tumble instead of three (NoName26 now hits the pub's wall, at update 71).
**Still broken / not tested:** Not looked at by eye. The user saw a "slow" tumble in the original; the traced rate is up to 430° per second (`thrown-items.md` open question 2): needs a look side by side.
**Next:** The user's check of thrown items against the original (`thrown-items.md` "How to check").

## 2026-10-20 M5 5e-2: thrown items bounce up again off barrels and other scripted or animated objects

**Changed:** `Flight::update` (`item.rs`) handles each touch in order: a touch on something with a script, or on an animated object, throws the item again from the touch point with fresh random launch values and tumble (`throw_again`). Anything else still ends the arc. `World::flights_step` (`fight.rs`) builds the list of what throws again: doors, chests, the animated objects (not characters), and clickables and buildings with a script. Provenance, DESIGN (5e-2 done), FEATURES.
**Why:** `thrown-items.md` rule 12.3 (Q137 c3): Daventry's barrels all have scripts, so items bounce off them.
**Tested how:** `cargo test --workspace` (446 core tests, 1 new: a wide "barrel" throws it up until it comes down off it; a plain object of the same size stops it), clippy clean. Viewer: barrels stand 2,600+ apart, so a test spot 800 from barrel.13 throws 40 crystals: `cargo play --world Daventry --sound off --at 189500,268000,0 --command "loadKQ barelded.kq testspot 189087 266690 8185 0 0 0",30 --command "KQObject::tossItems testspot 0 Crystal,…(40 times)",40 --frames 1500`; `items.log`. The crystal-barrel run of 5e-1 again (no barrel in the way there: same result as 5e-1).
**Result:** 3 of the 40 crystals came down on barrel.13 and were thrown again 8 times in all (NoName23 4 times: it hits the pub's eave, drops onto barrel.13, which stands under it, flies up, and so on). All 40 flights end held on the ground (the last at update 93). The land sound once per crystal.
**Still broken / not tested:** Not looked at by eye. Doors and chests throwing items again not run (no throw reaches one in Daventry's own scripts). The tumble is still the stand-in (5e-3). The test spot is a copy of the wreck (it would stop items, but they start inside it, which is not a touch).
**Next:** 5e-3, the tumble.

## 2026-10-20 M5 5e-1: thrown items fly the traced arc and are stopped by walls, roofs and objects

**Changed:** The thrower in `item.rs` (`Flight::update`) replaces 5c-8's stand-in. Each update it works out the exact arc from the launch point and moves the item there with the characters' mover. The mover uses the item's `BOUNDS.KQ` box, type 0x220, mask 0x809 and steps 0, and meets the floor grid, the buildings and the objects with type bit 0x1 or 0x100. At the first touch the land sound plays and the arc ends; then the item falls under ordinary gravity until it is held for one update (or 10 s). Flights now run in the viewer's `character_gravity` (`World::flights_step` gets the ground, buildings and objects), not in core's update. Plan 5e in `docs/DESIGN.md`; provenance; FEATURES.
**Why:** `thrown-items.md` (Q137). Items flew through the pub's walls and landed under its floor (the user's check).
**Tested how:** `cargo test --workspace` (445 core tests, 4 new: the exact arc and landing, the drop at a wall's foot, staying on a building floor, an item with no box), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 188861,270713,0 --key left:IDCTL_INVBAR_SHORT,40 --swing barrel.12,60 --frames 1000`, and goblin22's drops (`--at 289284,190300,0 --key left:IDCTL_INVBAR_SHORT,40 --swing goblin22,60 --swing goblin22,90 --frames 1500`); `items.log`.
**Result:** barrel.12's four crystals: NoName25 hits the pub at z 9,817 (update 71), falls straight down beside it (x, y unchanged; drops 36, 108, 180, …) to the floor grid at 8,160 and ends at update 80. NoName24 glances off the pub's wall low down and ends at update 91. NoName23 and 26 land on the open ground (rule 14, then the terrain touch). Each plays `potnland.aud` once, at its first touch. goblin22's mushrooms: one lands 1,491 away; the other comes down on a steep slope and slides down it (rule 17) to rest 510 away, `mushland.aud` once each.
**Still broken / not tested:** Not looked at by eye. Barrels stop items instead of throwing them again (5e-2). The tumble is still the stand-in (5e-3). Up-pass hits don't slide (the mover's rule; the note's rule 12.1 says they do for non-upward surfaces; it changes nothing visible). Gravity after the arc assumed as Connor's (`thrown-items.md` open question 3).
**Next:** 5e-2 (thrown again off barrels and animated objects).

## 2026-10-20 A: Connor's ranged weapons and the bolt's flight

**Changed:** New `docs/behaviour/ranged-weapons.md` (readying, Delete / Alt / slot clicks, the weapon swap's movements, the three pointers and Ctrl, the shot click, standing and moving shots, the start point, aim, flight and hit of bolts, and the swing click's target). `fighting.md` rules 36 and 39 and open questions 1–2 point there; QUESTIONS Q138 mostly answered, Q139 answered (except how objects take damage). Scratch notes in `re/notes.md`.
**Why:** The user asked for the study of Connor's ranged weapons, so B can give him the Small Crossbow and replace the spriggans' stand-in bolt.
**Tested how:** Traced in `Mask.exe` with Ghidra; checked against the data (`Connor.kq`, `Connor.sae`, `Invinfo.itl`, `conscbow.anm` and the other lists, `pr*.kq`, `BOUNDS.KQ`, the `arrow` part in `Scbatak.kqs`, `scbwalk.KQS`, `spbattck.KQS`). Not checked in the original game.
**Result:** Requests only in standing / moving (dropped otherwise); a swap plays `away` → list switch → `out` (Fists: `draw` / `undraw`); standing shot plays `attack`, bolt at its `do` cue, no extra delay, no ammunition; moving shot fires at once every ≥ 0.5 s and slows a run to a walk; aim at the clicked thing / ground, at most ~33° up or down; bolt starts at the model's `arrow` part (crossbow ≈ 832 ahead, 1,065 up), flies straight and pitched at 8,000 with no gravity, removed at the first object / character / Connor / building, below the floor grid or after 18,000; characters aim 1,200 above Connor's feet.
**Still broken / not tested:** Whether a swap cuts a swing under way; where the right-button click point lies; `collisionDetail=1`; pointer names inferred from order. Rocks and the Azriel Hammer not covered. Nothing checked in the original yet ("How to check" in the note).
**Next:** B: build `ranged-weapons.md` (Connor's crossbow; the bolt's real flight for the spriggans too).

## 2026-10-20 Test switch: god mode (F3, `--god`)

**Changed:** A core switch `god_mode`: every hit on Connor (swings and bolts all go through `hit_connor`) does nothing, so no health lost, no flinch, no death; his mover's new `no_fall_death` makes a fall past the death limit a safe landing (the viewer copies the switch into it every frame). F3 flips it, `--god` starts with it on, `--key f3,N` presses it in a test; an on-screen line under the AI line shows it. README key list updated.
**Why:** The user asked for god mode for testing.
**Tested how:** `cargo test --workspace` (442 core tests, 1 new), clippy clean. Viewer: `--at 289284,190300,0 --god --frames 1500` (goblin22); `--at 150552,269500,0 --no-ai --lift 8000 --frames 900` without and with `--god`; `fight.log`, `connor.log`.
**Result:** goblin22's 4 hits logged as "nothing, god mode is on", health unchanged. The 7,100 fall: "Connor dies" without, "safe landing" with god mode.
**Still broken / not tested:** The user's check (2026-10-20): F3, the on-screen line, hits and falls work. Nothing else hurts him yet (no poison, drowning or traps that damage), so those aren't covered.
**Next:** —

## 2026-10-20 Test switch: on-screen line for the AI switch

**Changed:** A line under the haze line (top left) shows the AI switch: "AI: on (F4)" in white, or "AI: off, creatures ignore Connor (F4)" in yellow. It follows core's switch every frame, so F4, `--no-ai` and `--key f4` all update it.
**Why:** The user asked for an on-screen indicator.
**Tested how:** Clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --no-ai --key f4,100 --frames 1500`; `viewer.log`.
**Result:** The run passes; the log shows the switch off at update 0 and on again at update 100.
**Still broken / not tested:** — (the user's check, 2026-10-20: the line shows and switches as it should).
**Next:** —

## 2026-10-20 Test switch: creatures' AI off (F4, `--no-ai`)

**Changed:** A core switch `ai_off`: no character gets alarmed, so none chases, attacks or shoots at Connor; alarmed or hunting ones give up and go home (rule 28's give-up). F4 flips it while playing, `--no-ai` starts with it on, `--key f4,N` presses it in a test. README key list updated.
**Why:** The user asked for a way to play without creatures attacking (testing).
**Tested how:** `cargo test --workspace`, clippy clean. Viewer at `--at 289284,190300,0` (goblin22): without the switch, `--frames 1200`; with `--no-ai`; with `--key f4,100 --frames 3000`; `fight.log`, `characters.log`, `viewer.log`.
**Result:** Without: goblin22 alarmed at update 15, 9 hits on Connor. `--no-ai`: no alarm, 0 hits. `--key f4,100`: hits at 48 and 78, the switch at 100, goblin22 gives up at 101, no hits to update 759.
**Still broken / not tested:** The user's check (2026-10-20): F4 works in play. Hitting a creature with the AI off not tested (the test punch missed: too far). Bolts already flying when it is switched off still fly and can hit. Switching back on: creatures notice him again by the usual rules (not run).
**Next:** —

## 2026-10-20 A: the user confirms thrown items fall at a building wall

**Changed:** `thrown-items.md` open question 1 closed and "How to check" updated; Q137 (c1) marked confirmed in QUESTIONS.
**Why:** The user looked again in the original: items hit the pub's wall and then fall down, no bounce.
**Tested how:** The user's check in the original game.
**Result:** Matches the trace (`thrown-items.md` rule 12: a building without a script ends the arc; the item drops at the wall's foot).
**Still broken / not tested:** The tumble's speed vs "slow" (open question 2) still not settled.
**Next:** B: build `thrown-items.md`.

## 2026-10-20 A: thrown items' flight studied (Q137)

**Changed:** New note `docs/behaviour/thrown-items.md`; `items.md` rule 12 (and its table and open question 3) and `fighting.md` rule 31 point to it; `fighting.md` open question 2 notes that the bolt uses other code. QUESTIONS (Q137 answered), STATUS. Scratch notes with addresses in `re/notes.md` (not committed).
**Why:** B's stand-in lets thrown items pass through the pub's wall and land under its floor (the user's check); Q137 (c) was blocking.
**Tested how:** Study only (Ghidra on `Mask.exe`, plus Daventry's `pub.kq`, `brlupcr.kq`, `brlupxcr.kq`). No code written, nothing run.
**Result:** The flight is an exact arc from the launch point (8,085 units/s², 15 updates a second), moved with the ordinary collision passes. It is blocked by the floor grid, buildings and objects with type bit 0x1 / 0x100; it flies through characters, Connor and other items. A touch slides it along the surface for the rest of the update. Then a scripted or animated object (the barrels) throws it again, steeply up; anything else ends the arc, and it settles under ordinary gravity. Land sound at the first touch. Tumble: one axis, a random -0.5 to +0.5 rad per update. At the end its turn is set to 0, 0, 0.
**Still broken / not tested:** The user saw an item bounce off the pub's wall; the trace says a building without a script stops the arc (it drops at the wall's foot), so that bounce may have been a barrel: needs one more look in the original. The "slow" tumble seen by the user vs up to 430°/s traced: not sure. Gravity after the arc assumed as measured for Connor (not measured for items). The bolt's flight is still open.
**Next:** B: replace the 5c-8 / fix 7 stand-ins with `thrown-items.md`. The user: watch a crystal hit the pub's wall in the original (`thrown-items.md` "How to check").

## 2026-10-21 The user's check of shadows on buildings after 4d-7b-5c / 5d

**Changed:** Docs only: DESIGN, FEATURES, STATUS.
**Why:** The user looked at the barn wall walk after the darkness and wall fixes.
**Tested how:** The user, by eye, in our viewer.
**Result:** "It looks a lot better now." Shadows on buildings are done.
**Still broken / not tested:** The shadow drawing as a whole may be redone later, when the user rebuilds the rendering after the game is finished.
**Next:** —

## 2026-10-21 M4 step 4d-7b-5d: the head's shadow climbs the wall too (the user's check)

**Changed:** `kq8-core/src/shadow.rs`: the search area's far end is the shadow of the top of the model's ball (the original: of the band's top, 1,079 for Connor standing); test updated. Provenance, DESIGN.
**Why:** The user's check of 4d-7b-5c: better, but the shadow's head still went under the barn wall before showing on it. The head is above the band's top, so its shadow ran past the area.
**Tested how:** `cargo test --workspace`, `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --sound off --at 240000,217000,3.1416 --hold 1,0,0,80 --frames 500`, `characters.log`.
**Result:** The barn wall's 7 surfaces are picked from update 18, before the shadow reaches them, so the area no longer holds any of it back; then 95, 183, 265 pieces on the barn as Connor nears it. Box: walking 637 × 3,495, standing 505 × 3,089. Shadow time 3.2 ms a frame on average.
**Still broken / not tested:** **Not looked at by eye.** If the head still goes under first, the cause is not the choice of surfaces (they are all picked) and we keep it as is (the user: close enough; the rendering will be redone after the game is finished).
**Next:** The user's look.

## 2026-10-21 M4 step 4d-7b-5c: shadows darker, and on walls as soon as they reach them (the user's check)

**Changed:** `kq8-viewer/src/shadow_view.rs`: the shadow's 0.5 is now of the shown brightness (sRGB 0.5, linear 0.214). `kq8-core/src/shadow.rs`: the search area's sides are no longer cut to 1,800 (the band's height cut stays); test updated. Provenance, DESIGN.
**Why:** The user's check: shadows lighter than the original, and on a wall they appear late (the ground shadow first slides under the wall). The sun's angle was suspected; it is the game's own (`tsLightAim` (0, 1, −0.66) in Daventry's `SunlgtHW.kq` and `SUNLIGHT.KQ`), so it is kept. The real causes: Bevy multiplies in linear light (0.5 showed as about 73 %), and the traced area ended about 1,750 from Connor along the sun while his shadow reaches about 1,970. Not reproducing the cut follows the user's earlier decision (no cut shadows).
**Tested how:** `cargo test --workspace`, `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --sound off --at 240000,217000,3.1416 --hold 1,0,0,80 --frames 500`, `characters.log`.
**Result:** The barn wall receives from update 23 (58 pieces), was update 30 with the cut (update 22 with the first stand-ins); walking box 637 × 1,972, standing 505 × 2,139.
**Still broken / not tested:** **Not looked at by eye**: the darkness and the wall timing need the user's look. The darker value is our reading of "halves" (blending not traced).
**Next:** The user's look at the barn wall.

## 2026-10-21 M4 step 4d-7b-5b: shadows on buildings use the model's stored ball (viewer, Q135 built)

**Changed:** `kq8-viewer`: `load.rs` (models and movements keep their `.kqs` ball); `shadow_view.rs` (`ModelBall` component, `caster` from a frame's place, `receivers` takes the caster and reports a fallback; plain objects find the ball of what is shown); `anim_view.rs`, `connor_view.rs`, `door_view.rs` (every shown frame carries its movement's ball), `terrain_view.rs` (still objects' parts carry their model's). Logs: per caster s, where the ball came from, the box (own space). DESIGN (4d-7b-5 done), provenance, STATUS, FEATURES, QUESTIONS (Q135 built).
**Why:** A's answer to Q135 (`characters.md` rule 37 (2)); 5a built the rule, this passes it the real ball and heading.
**Tested how:** `cargo test --workspace`, `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --sound off --at 240000,217000,3.1416 --hold 1,0,0,80 --frames 500` (into the barn wall) and `--at 146500,248928,1.5708 --frames 400` (the windmill floor); `characters.log`, `shadows.log`.
**Result:** Barn: walking, s 318, box 637 × 1,800, z −919..881 (band cut to 1,800); standing at the wall, s 252, box 505 × 1,800 × 1,766, z −687..1,079 (A's numbers for Connor standing); 8 barn surfaces, 257 of 477 pieces on the barn (the stand-ins gave 21 surfaces, 143 pieces). Windmill: Connor facing +x gets a 1,800 × 505 box (turned with him), 4 floor surfaces, 270 pieces on them; spriggan `sprigwnd1` (s 485) 12 surfaces. No caster used the fallback box. Shadow time 2.4 ms (barn) / 4.2 ms (windmill, 5 casters) a frame on average.
**Still broken / not tested:** **Not looked at by eye** with the new numbers. Items lying in the world use our fallback box (not checked).
**Next:** The user's look at the barn wall and the windmill floor.

## 2026-10-21 M4 step 4d-7b-5a: the search box on buildings as traced (core, Q135)

**Changed:** `kq8-core/src/shadow.rs`: `Caster` (own space + the model's stored ball), `search_box`, `building_receivers` rewritten to A's answer (s = 0.287 × ball radius; reach radius + s + 500; band s above to 6 s below the ball's centre, at most 1,800; the rectangle around the 2 s square and its copy along the sun, sides at most 1,800, turning with the heading; no downward sun = no search); the stand-in constants removed; 8 tests (4 replaced). `kq8-viewer`: `shadow_view.rs` builds a fallback caster from the box around the model's points (until 5b), `terrain_view.rs` load line. DESIGN (4d-7b-5 planned), provenance.
**Why:** A answered Q135 (`characters.md` rule 37 (2)).
**Tested how:** `cargo test --workspace` (441 core tests), `cargo clippy --workspace --all-targets` clean.
**Result:** All pass, with A's numbers: Connor standing (ball 879 / 827): s 252, band 1,079 above to 687 below his feet; running (1,134 / 799): cut to 885 above / 915 below; in Daventry's sun facing along it the area is 505 × 2,140, cut to 1,800 along the sun; a caster turned 45 degrees leaves out a tile its unturned box would take.
**Still broken / not tested:** The viewer doesn't yet pass the model's stored ball or the heading: every caster uses the fallback box (world axes), logged in the load line only. Not run in the viewer.
**Next:** 4d-7b-5b, the model's ball in the viewer.

## 2026-10-20 M5 5d fix 11: a weapon picked up while one is ready: straight into its stance

**Changed:** Picking up a hand weapon while another is ready switches Connor's movement set at once; nothing plays (before: the old one's put-away and the new one's draw).
**Why:** The user's check in the original.
**Tested how:** `cargo test --workspace` (437 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 260784,228600,3.1416 --key left:IDCTL_INVBAR_SHORT,40 --click NoName1,70 --key left:IDCTL_INVBAR_SHORT,150 --frames 1500`; `fight.log`, `connor.log`.
**Result:** 40: the fists' `draw`; 71: the dagger picked up, its set at once, nothing plays; 150: the dagger's `away`, his own `out`.
**Still broken / not tested:** Not looked at.
**Next:** The user's next check.

## 2026-10-20 M5 5d fix 10: drawing and sheathing in two parts; a weapon picked up while ready; no clicks meanwhile

**Changed:** Weapons other than the fists are drawn with his own `away` (hand to the sheath) then the weapon's `out`, and put away with the weapon's `away` then his own `out` (as `conversation.md` rule 7); the set switches when the second part starts. A hand weapon picked up while one is ready becomes the ready one. While a draw or put-away plays, slot clicks are refused.
**Why:** The user's checks in the original: the dagger's draw lacked its start and its sheath its end; picking up the dagger with the fists out switches to the dagger; no slot click until the draw has finished.
**Tested how:** `cargo test --workspace` (437 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 260784,228600,3.1416 --key left:IDCTL_INVBAR_SHORT,40 --click NoName1,70 --key left:IDCTL_INVBAR_SHORT,150 --key left:IDCTL_INVBAR_SHORT,210 --key left:IDCTL_INVBAR_SHORT,215 --frames 2200`; `fight.log`, `connor.log`, `items.log`.
**Result:** 40: the fists' `draw`. 70: the dagger picked up becomes ready: the fists' `undraw`, then `condag.anm/out` (set switched at 78). 150: `condag.anm/away`, his own `out` (set off at 166, ends 174). 210: his own `away`, `condag.anm/out` (ends 232). 215: the click refused.
**Still broken / not tested:** Not looked at. What the original plays exactly (Q138).
**Next:** The user's look.

## 2026-10-20 M5 5d fix 9: the dagger has its own movement set

**Changed:** A hand weapon's movement set is also loaded when a world script names the weapon (Connor is loaded before `WorldInv.cs` makes the dagger, so only the fists' set was loaded). A list without `draw` / `undraw` (all but the fists) readies with `out` and puts away with `away` (stand-in, Q138).
**Why:** The user's check: the dagger had no draw, holster or stance; only its swings played.
**Tested how:** `cargo test --workspace` (437 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 260784,228600,3.1416 --click NoName1,30 --key left:IDCTL_INVBAR_SHORT,60 --key left:IDCTL_INVBAR_SHORT,150 --frames 1300` (in Connor's house: the dagger is `NoName1`); `fight.log`, `connor.log`.
**Result:** Sets loaded: fists, dagger, small axe, Lake sword. The dagger is picked up; readying at 60 plays `condag.anm/out` (16 frames), then `stop` is drawn from `condag.anm`; at 150 `condag.anm/away`, then his own `stop`.
**Still broken / not tested:** Not looked at. Whether the original plays `out` or `draw` for the fists (Q138).
**Next:** The user's look.

## 2026-10-20 M5 5d fix 8: a pushed barrel is hit where it is

**Changed:** `push_apply` also moves the pushed object's box in the world's list, which clicks, Connor's swings and `getLoc` use; before, only the collider and the model moved.
**Why:** The user's check: a pushed barrel couldn't be hit; a click where it had stood broke it, and the burst appeared there.
**Tested how:** `cargo test --workspace` (437 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 205364,267646,1.5708 --hold 1,0,0,60 --key left:IDCTL_INVBAR_SHORT,90 --swing barrel.04,120 --frames 1500`; `connor.log`, `fight.log`, `viewer.log`.
**Result:** barrel.04 pushed to (208225, 267646); the swing at 120 targets it there and hits at 124 (distance 988); `brlupms.cs` loads the burst and the wreck at (208225, 267646, 8345).
**Still broken / not tested:** Not looked at.
**Next:** The dagger's movement set.

## 2026-10-20 M5 5d-5: `KQObject::tossItems` from a script (the crystal and mushroom barrels)

**Changed:** `KQObject::tossItems <object> <offsetZ> <types>` is built (it was a stub): one item per type thrown from the object's place raised by `offsetZ`, with the drops' throw and tumble (shared code `toss`). `getLoc`'s object lookup is shared (`place_of`). `--swing NAME,N` can aim at an object. A clippy warning from 5d-4 fixed.
**Why:** Plan 5d step 5; the user's check: barrels should throw their items like the creatures.
**Tested how:** `cargo test --workspace` (437 core tests, 1 new), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 188861,270713,0 --key left:IDCTL_INVBAR_SHORT,40 --swing barrel.12,60 --frames 1000`; `fight.log`, `items.log`, `traps.log`, `viewer.log`.
**Result:** Update 64: the punch kills barrel.12; `brlupcr.cs` loads the burst `NoName22` and throws four crystals from it (`NoName23`–`26`); they land at updates 84–86, 1,271–2,304 away, with `potnland.aud`; update 88 the burst's end loads the wreck. A first try with `--aim` failed: the camera there sits behind the pub, so the click line hits the pub.
**Still broken / not tested:** Not looked at. The mushroom barrels and the alchemist's boards not run (same scripts' pattern). Picking the crystals up not run.
**Next:** The user's check of the barrels.

## 2026-10-20 M5 5d-4: animated copies loaded while playing (the barrel bursts and leaves its wreck)

**Changed:** `anim_view` keeps a recipe (frames' meshes and materials, steps, script) and the player of every unused animated object; core gets the players of the animated templates; a copy core makes is drawn from the recipe and played by core (`AnimSim::spawn_copy`, through `copies_view`).
**Why:** Plan 5d step 4 (the barrels' burst `barelupx`, `brlupxcr`, `brlupxms`; the alchemist's boards).
**Tested how:** `cargo test --workspace` (436 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 198000,262152,1.5708 --key left:IDCTL_INVBAR_SHORT,40 --aim barrel.07,60 --frames 900`; `viewer.log`, `traps.log`.
**Result:** Update 64 the punch; 65: barrel.07 deleted, `NoName22` (the burst, 24 parts) drawn at its place; 88: the burst's end gives `Die`, its script deletes it and loads `NoName23` (the wreck, with its collider) at the same place.
**Still broken / not tested:** Not looked at. The crystal and mushroom barrels' `tossItems` (5d-5). The boards not run.
**Next:** 5d-5, `KQObject::tossItems` from a script.

## 2026-10-20 M5 5d-3: still copies loaded while playing are drawn

**Changed:** The loader keeps the files a script may load (named on a `loadKQ` line, no place of their own) as templates, built at the start and hidden; their boxes from `BOUNDS.KQ`. Core gets the still ones; the viewer keeps their models (new `copies_view.rs`) and draws each copy core asks for, with a collider when its box and type collide with Connor.
**Why:** Plan 5d step 3 (the barrels' wreck `barelded`).
**Tested how:** `cargo test --workspace` (436 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 193000,193000,3.1416 --command "loadKQ barelded.kq @ 193000 194500 -1 0 0 0.5",30 --frames 300`; `viewer.log`, `traps.log`.
**Result:** 7 templates found (2 still, 5 animated). The command makes `NoName22` (still, with its box) at (193000, 194500, 7377) (z −1 → the floor height); the viewer draws it (1 part) with a collider (−130..160 × −190..150 × 0..40, type 0x41).
**Still broken / not tested:** Not looked at. Copies have no draw-distance test or camera box. Animated copies: 5d-4.
**Next:** 5d-4, animated copies (the burst).

## 2026-10-20 M5 5d-2: `loadKQ` while playing with a name and a place (core)

**Changed:** Core keeps templates of the files scripts may load (given by the viewer; none yet). A runtime `loadKQ` with a name or place, or for a file with no prepared copy, makes a new copy: the name rule (`same`, `@` → `NoName<n>`, a new name), the place and angles (or the file's), the −1 / −2 ground rule, a name in use refused; the object is in core's world at once, `$ObjectName` is set, its script gets `Init`, and `Request::Spawn` asks the viewer to draw it (only logged for now).
**Why:** Plan 5d step 2 (the barrels' `loadKQ barelupx.kq @ $KQObject::loc`).
**Tested how:** `cargo test --workspace` (436 core tests, 1 new), clippy clean.
**Result:** The new test: two `@` copies are `NoName0` and `NoName1` at the given places and angles; `same` keeps the file's name and place; a second `same` is refused; four spawn requests.
**Still broken / not tested:** The viewer gives no templates yet and draws nothing: 5d-3 and 5d-4.
**Next:** 5d-3, templates for still models (the wreck).

## 2026-10-20 M5 5d-1: `KQAnimOnce` objects report their end

**Changed:** The loader marks `classType=KQAnimOnce` objects as play-once (it never did: the flag was only set in tests). The other animated objects (`world.anims`) now handle a play-once end as `world.objects` did: `Die Normal` to the script, `getEndLoop`'s cue, `setDeleteWhenDone`.
**Why:** Plan 5d step 1: the barrel's burst (`barelupx`, a `KQAnimOnce`) must tell its script it ended, which then loads the wreck.
**Tested how:** `cargo test --workspace` (435 core tests, 1 new), clippy clean. No Daventry object is a `KQAnimOnce` at the start, so nothing to see in the viewer yet.
**Result:** The new test: the object's script gets `Die Normal` at the end of `open`, no `EndLoop`, it is deleted.
**Still broken / not tested:** Not run in the viewer (comes with 5d-4).
**Next:** 5d-2, the name and place rules of a runtime `loadKQ`.

## 2026-10-20 M5 5c fix 7: thrown items tumble

**Changed:** A thrown item turns about all three axes while it flies (each at a random 90–180° per second, either way: a stand-in) and gets its resting angles back when it lands. Q137 (e) notes the user's answer.
**Why:** The user's check in the original: dropped items tumble slowly and end in their normal resting pose.
**Tested how:** `cargo test --workspace` (434 core tests, 1 new), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --key left:IDCTL_INVBAR_SHORT,40 --aim goblin22,60 --aim goblin22,90 --aim goblin22,110 --aim goblin22,130 --frames 2500`; `fight.log`.
**Result:** goblin22 dies at 94; its two mushrooms fly 19 and 24 updates and land 1,034 and 1,556 away, as before.
**Still broken / not tested:** The tumble is not looked at (drawing). The rates are ours.
**Next:** Runtime `loadKQ` with a name or position and `KQObject::tossItems` (the barrels' burst, wreck and items): plan in DESIGN.md first.

## 2026-10-20 M5 5c fix 6: a thrown item is drawn where it landed

**Changed:** `item_view.rs`: a world item's model is moved whenever its place changes. Before, it was moved only while its flight was listed, and the landing update removes the flight, so the model stayed at its last place in the air.
**Why:** The user's check: dropped items sit a little above the ground, and only a click on the ground below them picks them up (the click box was on the ground, the model was not).
**Tested how:** `cargo build`, `cargo test --workspace` (433 core tests), clippy clean. Not run in the viewer: the fix is in the drawing, which only the eye can check.
**Result:** —
**Still broken / not tested:** Not looked at. The tumble while flying is not built (asked the user how the items rest after landing).
**Next:** The tumble; runtime `loadKQ` with a position (the barrel's burst and wreck).

## 2026-10-20 M5 5c fix 5: Connor's punches break barrels (as far as their script)

**Changed:** Objects with a script but no clickable bit (Daventry's barrels) are kept with their box: they now get `Init`, `getLoc` finds them, and the attack click can target them. Connor's swing on such an object takes rolled damage off its stats (all 0 without a `stats=` line); below 0 its script gets `Die <kind>`. Q139 updated (the user's answer: a click near a creature misses; the object hit is a stand-in).
**Why:** The user's check: the barrels near the pub can't be broken.
**Tested how:** `cargo test --workspace` (433 core tests, 1 new), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 198000,262152,1.5708 --key left:IDCTL_INVBAR_SHORT,40 --aim barrel.07,60 --frames 800`; `fight.log`, `viewer.log`.
**Result:** Update 64: the punch hits barrel.07 (distance 1,544; health 0 → −9); `barelup.cs` gets `Die Normal`: `getLoc` gives its place, it is deleted, `breakbox.aud` plays. Then `loadKQ barelupx.kq @ <its place>` is not done (a position on a runtime `loadKQ` is not built): no burst, no wreck yet.
**Still broken / not tested:** The burst and the wreck (next: runtime `loadKQ` with a position, more than one copy of a file). How the original rolls damage on objects and which objects can be hit (Q139 e).
**Next:** The dropped items' landing height and click spot, their tumble; then runtime `loadKQ` with a position.

## 2026-10-20 M5 5c fix 4: Connor's punches find their target

**Changed:** With a hand weapon ready, the click line also tests the boxes of the characters that fight (not dying), whatever their collision type; the nearest one not behind a building becomes Connor's target. `--aim NAME` can aim at a character. New question Q139.
**Why:** The user's check: punches did no damage. The log of the user's run: every mouse click "meets nothing clickable", target none, "hits dead air". By the notes no creature is clickable (type 0x42 has no bit 0x20), so a swing could never have a target.
**Tested how:** `cargo test --workspace` (432 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --key left:IDCTL_INVBAR_SHORT,40 --aim goblin22,60 --aim goblin22,75 --aim goblin22,90 --aim goblin22,105 --aim goblin22,120 --frames 2500`; `viewer.log`, `fight.log`.
**Result:** Update 60: the line meets goblin22's box at 4,677; he turns, the swing hits at 74 (loss 11, health 15 → 4); at 93 the second hit (loss 8) kills it; at 124 its death ends: 3 experience, a corpse. Later clicks: target none (it's gone), dead air. A first run was spoiled: the fists slot clicked at update 10 was refused because Connor was still landing from `--at` (fix 3 working).
**Still broken / not tested:** The real mouse click on a creature is not run by me (the `--aim` click takes the same path). How the original finds the target is a stand-in (Q139): a click near but not on a creature misses.
**Next:** The user's look.

## 2026-10-20 M5 5c fix 3: the landing and the stumble; the weapon away before a conversation

**Changed:** No readying or putting away while a jump's landing (`jumpend`, `jumprun`, `jumpstop`) or a stumble (`crash`) plays. Readying sets Connor's hand mode (sword 0 / empty 4, also answered by `getDoMode`). `conversation.md` rule 7 is built: with a weapon drawn, `away` plays, he goes to bare hands (the weapon is no longer ready), `out` plays, then the conversation goes on. With a weapon ready, a click on a character that doesn't fight (the Wizard) is the plain click.
**Why:** The user's checks in the original: readying was still possible during the running-jump landing and the stumble; clicking the Wizard with the fists out puts them away, then the conversation starts.
**Tested how:** `cargo test --workspace` (432 core tests, 1 new), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 193000,198000,-1.7 --key left:IDCTL_INVBAR_SHORT,10 --click Wizard,30 --frames 1500` (`conversation.log`, `connor.log`); `--at 193000,193000,3.1416 --hold 1,0,1,80 --jump 30 --key left:IDCTL_INVBAR_SHORT,54 --key left:IDCTL_INVBAR_SHORT,100 --frames 1200` (`fight.log`).
**Result:** Wizard: `confists.anm/away` at 32–38, bare hands, `out` 39–44, the Wizard starts talking at 44. Landing: the click at 54 (`jumprun`) refused, the one at 100 readies.
**Still broken / not tested:** The stumble refusal is not run (by eye). The mouse click on the Wizard (the plain-click rule) is not run: `--click` skips the swing path. With the fists ready Connor can no longer push (rule of `pushing.md`, newly in effect): not checked in the original.
**Next:** The user's look.

## 2026-10-20 M5 5c fix 2: no readying during a jump; the put-away jitter

**Changed:** A click on the weapon slot while Connor jumps, falls, lands or dies is refused (the weapon stays as it was). The sword pointer now follows what Connor took, one update after the click. The drawing's blend between updates for a one-shot played backwards (`undraw`) now goes the same way as its phase (it went the other way: each update the picture jumped and slid back). Q138 notes the user's answers.
**Why:** The user's check in the original: no (un)readying during a jump; our put-away looked jittery.
**Tested how:** `cargo test --workspace` (431 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 193000,193000,3.1416 --jump 28 --key left:IDCTL_INVBAR_SHORT,32 --key left:IDCTL_INVBAR_SHORT,90 --key left:IDCTL_INVBAR_SHORT,150 --frames 1500`; `fight.log`, `gui.log`.
**Result:** Update 32 (mid-jump): refused, no sword pointer; 90: `draw`, sword at 91; 150: `undraw`, play pointer at 151.
**Still broken / not tested:** The jitter fix is not looked at by eye. Refusing while falling, landing or dying is ours. Switching to the other slot's weapon is not built (no ranged weapon in Daventry yet). Whether cutscenes put the weapon away: unknown (the watermill one doesn't; ours never does).
**Next:** The user's look at the put-away.

## 2026-10-20 M5 5c fix 1: the ready stance and the sword pointer

**Changed:** Readying a hand weapon switches Connor's movements to the weapon's own set (`confists.anm` for the fists: the fists-up stance, its walk, run, flinch and deaths) and plays its `draw`; putting it away plays `undraw` and switches back. The loader reads each ready-able weapon's whole set with its transitions (`load.rs` `weapon_sets`); `connor_view.rs` draws and times the weapon's version of each movement the walker asks for. The pointer is the sword (`IDBMA_CURS03`) while a weapon is ready, the golden pointer while Ctrl is held. A one-shot played backwards (`undraw`) now shows its frames last to first (`kq8-core` `MovementLoop::frame_at`). New question Q138; provenance "5c fix 1".
**Why:** The user's check in the original: clicking the fists slot should raise his fists and turn the pointer into a sword; ours did neither.
**Tested how:** `cargo test --workspace` (431 core tests, 1 new), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 193000,193000,3.1416 --key left:IDCTL_INVBAR_SHORT,30 --hold 1,0,0,90 --key left:IDCTL_INVBAR_SHORT,150 --frames 1500`; `fight.log`, `connor.log`, `gui.log`.
**Result:** Update 30: `confists.anm/draw` plays (he was walking: he stops for 8 updates), pointer `IDBMA_CURS03`; then `stopwalk`, `walk`, `walkstopleft`, `stop` are drawn from `confists.anm`; update 150: `undraw`, then his own `stop`, pointer `IDBMA_CURS04`. A first run at the windmill was spoiled: the bow spriggan killed Connor before the put-away.
**Still broken / not tested:** Not looked at by eye (the stance, the sword picture, the Ctrl switch). What readying really does is a stand-in (Q138): whether the walk stops for `draw`, when the set switches, what puts the weapon away. The `undraw` start phase (0.89) is not used.
**Next:** The user's look; then the next point from the fight check.

## 2026-10-20 `kq8 bitmap-png` reads cursor files

**Changed:** `kq8 bitmap-png` also takes a bitmap array (`.pba`: cursors, buttons) and writes its first bitmap.
**Why:** To tell which attack cursor is the sword (fix 1 above).
**Tested how:** `kq8 bitmap-png daventry/8gui/curs03.pba` (and `curs04`, `curs05`, `curs07`): PNGs in `kq8-artifacts/png/` (greyscale, no palette given).
**Result:** `curs03` 32×32 is a sword, `curs04` 20×20 the play pointer, `curs05` an arrow, `curs07` a rock or glove.
**Still broken / not tested:** Only the first bitmap of an array is written.
**Next:** —

## 2026-10-20 M5 5c finished: STATUS, FEATURES, README

**Changed:** Docs only: STATUS (where we are, the 5c stand-ins, next steps, Q136–Q137), FEATURES (the fight, the two new launch options), README (M5 so far, how to fight), DESIGN (5c done).
**Why:** End of the implementation session (`docs/SESSIONS.md`).
**Tested how:** —
**Result:** —
**Still broken / not tested:** The whole fight is checked in the logs only: not looked at by eye, not compared with the original, the real mouse swing not tried.
**Next:** The user's checks (`fighting.md` "How to check"); A: Connor's controls, the bolt's flight, Q136, Q137.

## 2026-10-20 M5 step 5c-10: the combat level

**Changed:** `kq8-data/world_script.rs`: `applyCombatLevel` reaches the loader. `kq8-core/fight.rs`, `active.rs`: `setCombatLevel 1|2|3`, the factor applied to characters loaded after `applyCombatLevel` when they enter the world. Viewer: the loader marks those characters; new launch option `--combat-level 1|2|3` (stand-in for the menu). 1 new test. DESIGN, provenance.
**Why:** `fighting.md` rule 3.
**Tested how:** `cargo test --workspace` (430 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --combat-level 3 --frames 30` and the same without the option; `fight.log`, `viewer.log`.
**Result:** `newWorld.cs line 10: applyCombatLevel 1` logged at load. Level 3: goblin22 `[15, 2, 2, 0, 3]` → `[22, 3, 3, 0, 3]`, sprgnbow `[130, -3, 2, 15, 15]` → `[195, -4, 3, 15, 15]`. Default: × 1, unchanged.
**Still broken / not tested:** Which menu sets the level in the original is not known (open question 3). Not played at level 1 or 3 beyond the numbers.
**Next:** End of 5c: STATUS, FEATURES, README; the user's checks in "How to check".

## 2026-10-20 M5 step 5c-9: the bow spriggans (stand-in bolt)

**Changed:** `kq8-data`: `ProjectileKeys` (a `KQProjectile` file). `kq8-core/fight.rs`: a ranged character's `Do` shoots (`shoot`, `lead`), bolts fly and hit (`Bolt`, `bolts_step`); `active.rs`: projectiles, bolts, Connor's last position; doMode 1 for ranged characters; `character_ai.rs`: the bolts each update. Viewer: the projectile file read and its shape loaded (`load.rs`), bolts drawn (`item_view.rs`). 2 new tests. DESIGN, provenance.
**Why:** `fighting.md` rules 34–38. The bolt's flight is not traced (open question 2): a simple stand-in, as you asked.
**Tested how:** `cargo test --workspace` (429 core tests), clippy clean. Viewer at the windmill: `cargo play --world Daventry --sound off --at 146500,248928,1.5708 --frames 3000` (Connor stands) and `--at 146500,248928,0 --hold 1,0,1,60 --frames 1200` (he runs); `fight.log`.
**Result:** `sprigwnd1` shoots at once (update 31), then every 26 updates (1.73 s = 1.2 s + 0.5 s; How to check 7 says about 1.7 s); each bolt flies 0.4 s and hits a standing Connor for D 11 (loss 3, 4, 6): he dies at the third. Running, the shot is aimed ahead using his speed; two bolts hit the windmill's wall.
**Still broken / not tested:** **Not looked at by eye.** The flight, start height and aiming height are stand-ins; the bolt has no pitch when drawn. Rule 37 (keeping distance) not built: no Daventry character uses it. The other two bow spriggans not run.
**Next:** 5c-10, the combat level.

## 2026-10-20 M5 step 5c-8: drops thrown

**Changed:** `kq8-core`: `item.rs` (`Flight`: the throw's launch and a per-update flight), `fight.rs` (the drops made and thrown when the dying starts; the flights each update, landing with the land sound), `active.rs`. Viewer: the drops' item models loaded (`load.rs`), a flying item moved each frame (`item_view.rs`). 1 new test. Q137 (the flight's exact stepping, not blocking). DESIGN, provenance.
**Why:** `fighting.md` rule 31 (the user saw the items fling up and land with a sound in the original), `items.md` rule 12.
**Tested how:** `cargo test --workspace` (428 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --swing goblin22,60 --swing goblin22,90 --frames 1500`; `fight.log`, `items.log`.
**Result:** At its death (update 94) goblin22 throws two mushrooms: 81.0° up at 6,209 units/s and 84.4° up at 6,347 units/s; they land at updates 117 and 118, 1,556 and 1,033 from it, each with `mushland.aud`.
**Still broken / not tested:** **Not looked at by eye.** The flight's stepping is a stand-in (Q137); nothing stops it on the way (walls). Picking one up in the air not tried.
**Next:** 5c-9, the bow spriggans.

## 2026-10-20 M5 step 5c-7: dying

**Changed:** `kq8-core`: `fight.rs` (`character_dies`, `death_ended`, `Corpse`, the first-kill tip), `active.rs` (`setState <name> Die [now]`, the die movement's end, the dead's `Die <kind>` script call), `character.rs` / `character_ai.rs` (a dying character never changes state and has no movement rules); `kq8-data`: `FightSettings::type_name`. Viewer: corpse shapes loaded with each fighter (`load.rs`), drawn when a corpse appears (`item_view.rs`). 2 new tests. DESIGN, provenance.
**Why:** `fighting.md` rules 21, 30, 32, 33.
**Tested how:** `cargo test --workspace` (427 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --swing goblin22,60 --swing goblin22,90 --frames 2000`; `fight.log`, `viewer.log`.
**Result:** Second punch (update 94): health 4 → −8, it dies: `Die` starts, off the active list, not solid, gravity off; its drops are listed (two mushrooms). At update 125 the death ends: Connor's experience 0 → 3, a corpse `g2dead.kqs` at (289716, 191415, 13770) is drawn ("corpse of goblin22: g2dead.kqs drawn"), it is removed and hidden; first-kill tip logged; it has no script ("Die Normal not called"). Matches How to check 6 (experience only when the death ends).
**Still broken / not tested:** **Not looked at by eye.** The corpse has no draw-distance test and no skin swap (bossprig's body will show the plain texture). Projectile, fire and ice deaths not run in the viewer. The drops don't appear yet (5c-8).
**Next:** 5c-8, drops thrown.

## 2026-10-20 M5 step 5c-6: characters being hit

**Changed:** `kq8-core/fight.rs`: `hit_character` does all of rule 22 (kind, active, chase timer, hit count, must flee, damage, attack time, flinch or ouch, where Connor is, the alarm with "run"), `run_alarm` (rule 26), `hit_unalarmed` (rule 28); `character.rs`: the flinch's end (2 s pause, chase timer); `character_ai.rs`: the hit count (always `run` from the second hit, cleared at home), the hunt after an unseen hit (0.52 rad beside Connor: the note's Do instead). 2 new tests. DESIGN, provenance.
**Why:** `fighting.md` rules 20–28.
**Tested how:** `cargo test --workspace` (425 core tests), clippy clean. Viewer: Connor punches goblin21 (health 50, protection 14) every 10 updates: `cargo play --world Daventry --sound off --at 150552,269500,0 --swing goblin21,60 --swing goblin21,70 ... --swing goblin21,400 --frames 3500`, `fight.log`.
**Result:** 9 hits land (loss 7–10 after its rolled protection 11–17); a flinch at update 94 (R 11), ended at 102; refused while its own `Attack` could not be cut short (77, 134) or R ≤ 9; ouch sound `gblnouch.aud` no more than once a second ("the last was 0.60 s ago"); its attack time put right while a swing was under way (rule 22.7). Each hit's alarm with "run" reached goblin21 and also Brdgsprig and Fallsprig (active and seeing Connor: rule 26 has no distance limit). Unit tests: never two flinches within 2 s of one ending; an unseen hit sends a goblin running beside Connor until he is within its alarm radius.
**Still broken / not tested:** **Not looked at by eye.** It doesn't die yet below 0 (5c-7). Whether far characters really join the alarm with "run" in the original (rule 26 as written) is not checked. Fleeing (rule 37) is 5c-9. Fewer flinches than "3 in 5" in this run because its own swing often can't be cut short; not compared with the original.
**Next:** 5c-7, dying.

## 2026-10-20 M5 step 5c-5: Connor hits back (stand-in input)

**Changed:** `kq8-core`: `fight.rs` (`connor_do`, `hit_character`, `cut_short_at`), `connor_inventory.rs` (the hand-weapon slot readies the weapon; a second click puts it away), `active.rs` (`weapon_ready`, `combat_level`). Viewer: `connor_view.rs` (his swing: turn toward the click, `attack` / `attackb` / `attack2` from the weapon's list, his `Do` cue), `door_view.rs` (a click with a weapon ready swings, Ctrl gives the plain click, Shift the big swing; `--swing NAME,N[,big]`), `load.rs` (loads each hand weapon list's three swings), `main.rs`. 3 new tests. DESIGN, provenance.
**Why:** `fighting.md` rules 39–41. The input side is not traced (open question 1): built from the manual's description, logged as a stand-in.
**Tested how:** `cargo test --workspace` (423 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --swing goblin22,60 --swing goblin22,90 --swing goblin22,92 --swing goblin22,120,big --frames 1500`, `fight.log`.
**Result:** Update 60: fists readied; goblin22 is 130° off his heading, so he turns first and `confists.anm/attack` starts at 70; its `Do` at 74 hits (1,154 away, reach 2,350): D 10, R 11 → goblin22 15 → 4. Update 90: a swing at once; the click at 92 does nothing (a swing is playing); `Do` at 94: R 8 → −4 (rule 19's "dies at the second hit"; dying is 5c-7). The Shift swing at 120: × 3.804 → D 38, R 46.
**Still broken / not tested:** **Not looked at by eye**, and not tried with the real mouse (only `--swing`). The goblin doesn't react or die yet (5c-6, 5c-7). Only his swings come from the weapon's list; he walks with `Conner.anm` (stand-in). Ranged weapons and Delete not done.
**Next:** 5c-6, characters being hit.

## 2026-10-20 M5 step 5c-4: the swing lands on Connor

**Changed:** `kq8-core/fight.rs`: `HitKind`, `Damage` (the rolls), `HitTest`, `ConnorFight`, the `Do` cue's hit test and `hit_connor`; `active.rs`: `Request::ConnorHurt` / `ConnorDies`, Connor's fight state, the fight's random draws; `connor.rs`: `Walker::death`. Viewer: `connor_view.rs` (`hurt`: flinch or ouch sound; `die_from_hit`), `door_view.rs` (the two requests; `Connor.kq`'s fight keys to core), `load.rs` (reads `ouchSound`, the damage factors, `ShortAnimList`; loads his `react`, `die`, `dropweapon`, `firedie`, `icedie`). 4 new tests. DESIGN, provenance.
**Why:** `fighting.md` rules 13, 14, 16–19, 29.
**Tested how:** `cargo test --workspace` (420 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --frames 3500` (Connor stands) and `--at 289284,190300,3.1416 --hold -1,0,0 --frames 2500` (he walks backward away from goblin22); `fight.log`, `connor.log`.
**Result:** Standing: every swing hits (1,154 away, 0° off its facing): 1, 1, 1, 1, 2, 2, 2, 2 lost (D 2, roll 25–100 %): health 10 → −2, dead at update 258; flinches (`react`, 0.8 s) at 169 and 229, refused at 199 ("last flinch ended 1.20 s ago"), the ouch sound otherwise; the half-health tip line at 168; `die` plays, then the Die state holds it (304); later hits "ignored, he is dying". Walking backward: the goblin swings from its run and hits at 1,642 and 1,503 (reach 1.25 × = 1,875), as How to check 2 says. Unit tests: rule 19's examples, the 60 % flinch share for D 10, the front-half and back-key tests, a miss behind a `noTurn` goblin.
**Still broken / not tested:** **Not looked at by eye.** Game over is not built (he stays dead, keys ignored; a stand-in). The tips are logged only. Circling round a swinging goblin (How to check 3) not run in the viewer (unit test only). Fire / ice deaths not run.
**Next:** 5c-5, Connor hits back.

## 2026-10-20 M5 step 5c-3: characters attack

**Changed:** `kq8-core`: `fight.rs` (the attack time, `is_attack`, the `Do` cue logged); `character.rs` (`start_attack`, the end and the cut-short cases of rules 10, 11); `character_ai.rs` (the attack in Wait and Move, replacing the 5a / 5b stand-in; no stop rule while a swing from Move plays: the user's decision on rule 11b); `active.rs` (`connor_back_key`, the `Do` cue, the character's fight lines to `fight.log`). Viewer `door_view.rs`: his back key. 2 new tests, 1 updated. DESIGN, provenance.
**Why:** `fighting.md` rules 6–12.
**Tested how:** `cargo test --workspace` (416 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --frames 3000` (Connor stands by Sara's house), `fight.log`, `doors.log`.
**Result:** goblin22 runs up and swings at once from its run (update 40, 1,352 away, attack time 0 -> never), its `Do` at update 48, the swing ends at 55 (next due now + 1 s); from then on a standing swing every 30 updates (70, 100, 130, ... 310): 1 s swing + 1 s delay, as How to check 1 says. `interruptable 1` / `0` fire at updates 41 / 43 (0.19 s). Unit test: the same with no stop between the moving swing's start and end.
**Still broken / not tested:** **Not looked at by eye.** No hit yet (the `Do` cue only logs). Connor walking away during the swing, the cut-short case and the bow spriggans' standing swing (1.25 ×) not run.
**Next:** 5c-4, the swing lands on Connor.

## 2026-10-20 M5 step 5c-2: the `interruptable` cue

**Changed:** `kq8-core/src/named_anim.rs`: a cue `interruptable <n>` sets the movement's "can be cut short" to n when the phase passes it; `active.rs`: the log says so instead of "cue action not done". 1 new test. DESIGN, provenance.
**Why:** `anim-scripts.md` rule 7; `fighting.md` rules 7, 12, 23 depend on it (an attack can be cut short only in its first moment).
**Tested how:** `cargo test --workspace` (414 core tests), clippy clean.
**Result:** Unit test with goblin2's `attack` (1 s): can't be cut short at the start (its `loop=` line), can from its first moving update (cue at 0), can't after phase 0.1875; a request then waits for the end.
**Still broken / not tested:** Not seen in the viewer: no Daventry movement that plays so far has the cue (only the attacks, next step).
**Next:** 5c-3, characters attack.

## 2026-10-20 M5 step 5c-1: reading the fight's numbers

**Changed:** Plan for the fight (5c-1 to 5c-10) in `docs/DESIGN.md`. `kq8-data/characters.rs`: the monster and type lists' fight columns (`xtra*`, `invHold`, `invDrop`, `attackDelay`, `tooClose farEnough`), the fight keys of a character file, `FightSettings` (rule 2: the five stats); `items.rs`: the projectile column. `kq8-core`: new `fight.rs` (`Fight`), `Character::fight`, `fight_note` and a new `kq8-artifacts/logs/fight.log`. `kq8 character-verify` prints each line's fight numbers. Q136 (`reactPause`, not blocking). Provenance.
**Why:** `fighting.md` rules 1, 2, 5: every fighter's stats before anything can hit.
**Tested how:** `cargo test --workspace`, `cargo clippy --workspace --all-targets` clean. `cargo run --release -p kq8-cli -- character-verify` (`character-verify.log`); `cargo play --world Daventry --sound off --frames 30` (`fight.log`).
**Result:** All of the note's rule 5 table matches, e.g. goblin22 `[15, 2, 2, 0, 3]` delay 1 s, drops two mushrooms; sprgnbow `[130, -3, 2, 15, 15]` ranged with `INVITEM_SmCrossbow` (`Prscbolt.kq`); bossprig `[230, 23, 23, 32, 7]` (Leather Armor +2 protection); RoadZ1 `[40, 6, 6, 37, 2]`; bats, chickens, pig `[5, 5, 5, 3, 0]`. `fight.log`: 52 fighters at load.
**Still broken / not tested:** Nothing fights yet. The combat level (rule 3) is 5c-10 (Daventry's 1.0 changes nothing).
**Next:** 5c-2, the `interruptable` cue.

## 2026-10-20 Q135 answered: which building surfaces receive a shadow (study, A)

**Changed:** `docs/behaviour/characters.md` rule 37 (2) (reach, band, area, surfaces left out; Connor and plain objects), Numbers and units, Do not reproduce; `docs/behaviour/QUESTIONS.md` (Q135 answered); STATUS.
**Why:** B built rule 37 (2) with stand-ins; Q135 asked for the traced numbers.
**Tested how:** Traced in `Mask.exe` (Ghidra). Data: model ball radii from the `.kqs` headers (Connor `C_stop` 879, `C_run` 1,134, goblin 942); surface types over all 153 `.dig` (172,783 surfaces, all type 0; material 255 only Castle D's 96). Nothing run in the game.
**Result:** The size is s = 0.287 × the shown model's stored radius (Connor 252), not the `BOUNDS.KQ` box. Reach: centre distance ≤ building radius + s + 500 (B's building ball is right). Band: s above to 6 × s below the model ball's centre, at most 1,800. Area: a rectangle in the character's own space (turns with it) around a 2 × s square and its copy moved along the sun to the feet, each side at most 1,800; it picks surfaces, it does not cut. The original does not leave out kind-0 materials (we keep skipping them). Two slips in the original noted under Do not reproduce. Differences from B's stand-ins: smaller caster size, band measured from the ball centre (Connor: 1,079 above the feet to 687 below), area turned with the character and often narrower than 1,800.
**Still broken / not tested:** Not checked in the game. That the game reads the model's stored ball into the place the shadow code uses is inferred from matching values, not traced in the file reader. Kind-0 surfaces counted in Daventry only (by B: none).
**Next:** B may swap the stand-ins for these numbers (not blocking; the user found the stand-ins look right).

## 2026-10-20 `fighting.md`: the swallowed swing is not reproduced

**Changed:** `docs/behaviour/fighting.md` rule 11b and Do not reproduce.
**Why:** The user's decision: it is a slip in the original, and `docs/SESSIONS.md` says not to reproduce bugs.
**Tested how:** — (docs only)
**Result:** Do instead: while a moving character's `Attack` plays, the moving state's stop rule is not applied; a chasing character that arrives with its attack due always swings in full at once.
**Still broken / not tested:** Nothing built yet.
**Next:** B builds the fight with this rule.

## 2026-10-20 The user's check of the fight in the original; `fighting.md` corrected

**Changed:** `docs/behaviour/fighting.md`: new rule 11b (the swallowed swing), rule 31 corrected (drops are thrown), numbers, Do not reproduce, How to check 6 (results), open question 4 closed.
**Why:** The user checked rules 6 and 31–32 in the original: (1) a goblin sometimes makes its attack sound with no swing and then attacks normally, but attacks at once when Connor walks or runs away; (2) the dropped items fling upward and land with a sound, can be picked up in the air, and the XP bar moves when the death animation ends.
**Tested how:** (1) Explained from code already traced: a chasing goblin whose attack is due starts it, and next update (Connor has no forward / back key) switches to standing, whose `Stop` cuts the attack in its first cut-short moment; the real swing comes `attackDelay` later. (2) Traced the item mover's start: it picks the same random throw as `tossItems` (75°–85°, 5,000–6,500 units/s). Nothing run.
**Result:** Both observations fit the note now. The first reading of the drops ("no throw") was wrong: it missed the mover's own start.
**Still broken / not tested:** Not checked whether every "sound, no swing" the user saw is this case. Spriggans should skip silently (their sound comes later); not checked.
**Next:** B builds the fight; keeping the swallowed swing is recommended (it comes free with the rules).

## 2026-10-20 The user's check of shadows on buildings (original game)

**Changed:** Docs only: DESIGN (4d-7b-3 checked), STATUS, FEATURES, provenance, QUESTIONS (Q135 update, not blocking).
**Why:** The user compared 4d-7b-3 with the original.
**Tested how:** The user, by eye, in our viewer and in the original.
**Result:** "Working exactly as expected." Shadows show inside a building when Connor stands outside it, in both. The original cuts some shadows off in places; **the user's decision: don't reproduce that**, keep ours as it is (whole surfaces, no cut).
**Still broken / not tested:** Q135's numbers are still stand-ins (not blocking).
**Next:** B: the fight (`fighting.md`); A: Q135 when convenient.

## 2026-10-20 M4 step 4d-7b-3c: shadows drawn on buildings (viewer)

**Changed:** `kq8-viewer`: `terrain_view.rs` (each building copy's drawn surfaces placed in the world at load, with its draw-distance ball: `BuildingShadows`); `draw_distance.rs` (each object's near / far / off state shared: `ObjectActivity`); `shadow_view.rs` (`receivers`: terrain plus the active buildings' chosen surfaces; plain objects use it, and are worked out again when the active buildings change); `anim_view.rs` (Connor and characters use it; log lines). DESIGN (4d-7b-3 done), provenance, STATUS, FEATURES, README.
**Why:** `characters.md` rule 37 (2): near and far buildings receive shadows. Open points are Q135, built as logged stand-ins.
**Tested how:** `cargo test --workspace`, `cargo clippy --workspace --all-targets` clean. Viewer runs, `characters.log`, `shadows.log`, `viewer.log`: (1) the windmill floor `cargo play --world Daventry --sound off --at 146500,248928,1.5708 --frames 400`; (2) Connor walking into the barn's south wall, the sun behind him `--at 240000,217000,3.1416 --hold 1,0,0,80 --frames 500`; (3) the open walk north by the church `--at 193000,193000,3.1416 --hold 1,0,1,90 --frames 900`.
**Result:** Load: 26 building copies receive, 19,660 drawn surfaces (none left out in Daventry). (1) Connor and spriggan `sprigwnd1` on the windmill floor: "1 of 5 active buildings in reach [Windmill], 10 building surfaces". (2) Before the wall 0 pieces on buildings, then 36, 168, 289 as he nears it; standing against it (update 44 on): 21 barn surfaces, 143 of 364 pieces on the barn. (3) No building in reach: terrain only, as before. Cost: the shadows of Connor and the characters take 4.7 ms a frame on average at the windmill (5 casters, 2 on the floor); 3.6 ms with the buildings switched off (a one-off measuring build, reverted). Plain objects: one object's shadow reaches 8 building surfaces at the windmill.
**Still broken / not tested:** **Not looked at by eye**, not compared with the original. All of Q135 (reach sizes, band, area, whole surfaces) are stand-ins. No occlusion: as rule 37 describes, a shadow is drawn on every chosen surface, so part of it can also land on a surface behind a wall (e.g. inside a building), which the original probably does too (not traced). The shadow time (3.6 ms before this step) is high; not optimised.
**Next:** The user's look (below); A answers Q135.

## 2026-10-20 Study (A): the fight — new `fighting.md`

**Changed:** Docs only. New `docs/behaviour/fighting.md`; pointers to it in `character-movement.md` (rules 3, 18, 25, open question 2 answered) and `anim-scripts.md` rule 20; STATUS. Scratch notes and addresses in `re/` (not committed).
**Why:** M5's next part: how characters attack, hit, take damage, flinch, die and drop items; ranged characters; Connor's side as far as needed to hit back. Starts where `character-movement.md` rule 25 stops.
**Tested how:** Traced in `Mask.exe` (Ghidra, plus a small instruction search in the scratchpad): the attack start in the standing and moving states, the movement-ended step, the `Do` cue's hit test, the hit message and its handlers (character and Connor), the damage roll, the flinch, the alarm spread, the dying state and its end, drops, experience, the monster list and type readers, the combat level, Connor's level up. Daventry's numbers computed from `Monstats.mtl`, `monsters.kq`, `Invinfo.itl` and the `.anm` / `.kqs` files. Nothing run in the game.
**Result:** First attack due **at once** (the attack time starts at 0), then swing length + `attackDelay`; an attack starts only when the running movement can be cut short; characters stand still while swinging (attack models have no step). Hit at the `Do` cue: distance ≤ reach (1.25 × if Connor holds back) and target in the front half. Damage: roll 25–100 % on Connor, 75–125 % on characters, then protection (rolled 75–125 %, 100 = immune), loss at least 1; dead below 0. Flinch when the roll > 0.9 × damage, at most once per 2 s after the last; else ouch sound. Every hit alarms everyone who sees Connor "with run" (always run, 10 s chase). Dying: drops at once at its feet, die movement by kind (`die`/`projdie`/`firedie`/`icedie`), then experience to Connor, a corpse (`deadDts`; none for fire), `Die <kind>` to its script. Ranged = holds a `LongWeapon` (the three bow spriggans; guards, never move or flee). Combat level 0.66 / 1.0 / 1.5 scales health and damage. Connor: stats 10/10/0/0/0, reach 2,350, `Attack`/`AttackB`, Shift = `Attack2` × 0.75–4.
**Still broken / not tested:** Nothing checked in the original. Not traced: the input side of Connor's attack, the bolt's flight, the aiming height, which menu sets the combat level, whether drops spread. One bug found (the 30° approach after an unseen hit), with a recommended fix in Do not reproduce.
**Next:** B builds the fight (M5); the user's "How to check" in the original.

## 2026-10-20 M4 step 4d-7b-3b: which building surfaces receive a shadow (core)

**Changed:** `kq8-core/src/shadow.rs`: `ReceivingBuilding` / `ReceivingSurface` (a building's ball and its drawn surfaces in the world), `building_receivers` (reach, height band, 1,800 area), constants; 4 unit tests. Provenance.
**Why:** `characters.md` rule 37 (2); the open numbers are Q135 stand-ins (see 4d-7b-3a).
**Tested how:** `cargo test --workspace` (413 core tests), `cargo clippy --workspace --all-targets` clean.
**Result:** All pass. The tests check: a building just inside / outside radius + caster's radius + 500; the band −2,600 .. 1,400 for a 1,300-high caster standing at 0; the area's edges 900 around the shadow's middle (985 toward +y in Daventry's sun for that caster), a 4,000-wide tile reaching in taken whole; a chosen floor 500 up gets the shadow lifted 20.
**Still broken / not tested:** Not used by the viewer yet, nothing to see.
**Next:** 4d-7b-3c, drawing them in the viewer.

## 2026-10-20 M4 step 4d-7b-3a: shadows on buildings planned, read against the data (Q135)

**Changed:** `docs/DESIGN.md` (4d-7b-3 planned in three steps, with stand-ins); `docs/behaviour/QUESTIONS.md` (Q135); `kq8-cli/src/building_verify.rs` (new lines: floor surfaces and their width, building ball radii).
**Why:** `characters.md` rule 37 (2) gives the idea (active buildings within reach, surfaces near the character's height, an area at most 1,800 across) but not all the numbers.
**Tested how:** `cargo run --release -p kq8-cli -- building-verify`, `kq8-artifacts/logs/building-verify.log`.
**Result:** RESULT: PASS (153 buildings). 33,962 surfaces face up, 17,295 of them wider than 1,800 (largest 4,096, Dead City `LITEHSE1`); Daventry's floors are tiles of up to about 2,040. Ball radii 876 (`zedrock2`) to 34,199 (Gnome `GNOME06`); Daventry up to 18,990 (`nchurch`). So whether the 1,800 area cuts the shadow or only picks surfaces shows in play. Q135 asks: the reach's sizes, the band's two numbers, the area's shape / centre / cut, invisible surfaces.
**Still broken / not tested:** Nothing built yet.
**Next:** 4d-7b-3b, the choice of surfaces in core.

## 2026-10-20 The user's check of the leash and the follow (original game)

**Changed:** `docs/behaviour/character-movement.md` How to check 4: the result recorded.
**Why:** Check of A's Q133 answer in the original.
**Tested how:** The user, by eye, in the original: ran away from a chasing goblin; walked with a goblin following, then stood still.
**Result:** Far enough away, the goblin runs back home. While Connor walks it keeps up with him and eventually closes in and stops to attack, standing still (it can't attack while moving); when he stands still it catches up and attacks too. Both fit rules 25 and 28 (follow a moving Connor; within reach, once the attack is due, attack whatever he does). B's stop-and-go stand-in is not what the original does; until the fight is built, the closest stand-in is "follow while he moves, stop within reach when the attack would be due".
**Still broken / not tested:** The exact distances and timings were not measured.
**Next:** B builds the Q131–Q134 answers.

## 2026-10-20 Study (A): characters moving on their own finished — Q131–Q134 answered

**Changed:** Docs only. `docs/behaviour/character-movement.md` rules 3, 7, 18, 21, 25, 27, 28, 30, 33, numbers, edge cases, Do not reproduce, How to check 4, open questions; `connor-collision.md` rule 18 (the cylinder, exact); `connor-physics.md` (landing on a `KQMonster`); QUESTIONS (Q131–Q134 answered); STATUS. Scratch notes in `re/` (not committed).
**Why:** B built the note (M5 5a) with stand-ins and asked Q131–Q134; the user's check by eye said movement looks right.
**Tested how:** Traced in `Mask.exe` (Ghidra): the moving and standing updates, the alarm, giving up, leaving the active list, the Birth state (enter, end of `hide`, hidden), the idle fidget and the shared "movement ended" step, the ball-stack builder and the ball-against-cylinder test, Connor's hit handling; sizes checked against `GAME/patch/BOUNDS.KQ`. Nothing run in the game.
**Result:** Q133: **no extra condition** stops the leash loop; the way-home re-take is the original's (it keeps `run`, has the 0–1 s reaction delay), so a character can't be leashed while Connor stays within its reach (in the original, that is while it fights him). "Not walking forward" = Connor has no forward / back key: a melee character stops only then, otherwise follows him at reach. Q134: the chase waits for `birth` in effect (the walk waits, the stuck check starts with the walk); a sunk zombie does **not** re-take underground: at the end of `hide` it leaves the active list, is hidden and put on its home point, and rises again only by rule 30 from there. Q132: world diagonals NE, NW, SW, SE; round restarts on each new move; level line from its feet; Connor never blocks it; Connor's cylinder for a character radius 200, 0–1,300. Open questions: all blocked → face north; reset distance = 3D to Connor; idle list swapped in for the fidget, own list back when any movement ends (side effect: `attack` / `react` ending resets timers). Q131: radius = ball radius, bottom = position + box min z, height = max(box height, box width) (goblin 0–1,600), widened by the mover's ball; vertical moves test one ball; landing on a monster's top = a landing plus a 250 push done as a real across move.
**Still broken / not tested:** Nothing checked in the original. Not sure: the stuck line's ground test at exactly foot height (fragile in the original; recommended rule given). Not traced: when the first attack is due, characters without a collision shape on their way home (fight study).
**Next:** B builds the answers; the user's "How to check" 4 (re-worded) in the original; A: the fight study.

## 2026-10-20 The user's check of characters moving on their own (5a, 5b): done

**Changed:** Docs only: FEATURES (checked, done), STATUS, DESIGN, README, provenance.
**Why:** The user compared the last look (the chase and the zombies) with the original.
**Tested how:** By eye in our viewer and in the original (user): a goblin's chase (walking away, standing, running far) and the road / graveyard zombies (sinking, rising again).
**Result:** "Both match exactly." Earlier the same day: movement and routes of the chickens and bats, the follow, the leash, the zombies' cycle.
**Still broken / not tested:** The attack is a stand-in until the fight study. Not run: losing sight, leaving and re-entering the graveyard trap, `wanderAndStop`, random wander points, ranged characters.
**Next:** A: the fight study. B: shadows on buildings (4d-7b-3).

## 2026-10-20 M5 5b finished: STATUS, FEATURES, QUESTIONS

**Changed:** Docs only: STATUS (where we are, stand-ins, next steps; Q131–Q134 built), FEATURES (characters moving on their own), QUESTIONS (Q131–Q134 marked built), DESIGN (5b done).
**Why:** End of the implementation session (`docs/SESSIONS.md`).
**Tested how:** —
**Result:** —
**Still broken / not tested:** The 5b changes are checked in the logs only; the attack is a stand-in until the fight study.
**Next:** The user's last look at the chase and the zombies; then 5a / 5b are done. A: the fight study.

## 2026-10-20 M5 step 5b-5: the zombies as traced (Q134)

**Changed:** `kq8-core/src/character_ai.rs`: a zombie gives up by playing `hide` where it is (still drawn, solid, active); when `hide` ends (or a team mate tells it during `hide`) it leaves the active list and goes underground on its home point; it rises again only by the rising test from home. The 5a fix's re-take underground and the 5a-8 stand-in (stay in Wait during `birth`) are removed. `active.rs`: solidity follows "underground", so a sinking zombie stays solid. Test updated. DESIGN, provenance.
**Why:** A's answer to Q134 (`character-movement.md` rule 30).
**Tested how:** `cargo test --workspace` (409 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 192916,313500,0 --hold 1,0,1 --turn 150,1 --frames 7000` (the road; Connor runs off, then circles) and `--at 215000,172000,0 --frames 3000` (the graveyard), `characters.log`.
**Result:** Road: RoadZ4 gives up at 8,008 from home (update 297), `hide` plays, at update 328 it is underground on its home point and off the list; with its 18,000 alarm radius it sees Connor from home at once and rises again there. RoadZ2, told by RoadZ3 while its `hide` played, went underground at home at once. Graveyard: 4 zombies rise and stop at their attack distance (1,100) facing a standing Connor; no give-ups.
**Still broken / not tested:** **Not looked at by eye.** Leaving the graveyard trap not run. No attack (fight study).
**Next:** STATUS, FEATURES; the user's look.

## 2026-10-20 M5 step 5b-4: the chase as traced (Q133, the user's check)

**Changed:** `kq8-core/src/character_ai.rs`: within reach a character stops only while Connor has no forward / back key, else follows him at reach (no more stop-and-go); ranged stop / start at 0.75 × / 1.25 ×; the way-home re-take only while moving, without the coin, heading home during its reaction delay; the standing leash and give-up rules; 3D home distance; `setHome` −1 / −2 placed; the chase timer after an `attack` / `react` fidget. `active.rs`: `connor_move_key`, set by the viewer from Connor's forward / back input. 1 new test. DESIGN, provenance.
**Why:** A's answer to Q133 and the user's check in the original (a goblin keeps up with a walking Connor; far enough away it runs home).
**Tested how:** `cargo test --workspace` (409 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 289284,190300,0 --hold 1,0,0 --frames 5000`, `characters.log`.
**Result:** goblin22 is alarmed, runs after Connor and from update 45 follows him at reach while he walks (no stops). It passes its leash at updates 126 and 202 but takes up the chase again at once (he is within reach: the original's behaviour, A); at 291 it gives up for good, reaches home at 335 and wanders again from 351. Unit test: it follows while the key is held and stops when it is let go.
**Still broken / not tested:** **Not looked at by eye.** No attack (fight study): where it would attack it only stops (standing Connor) or keeps following (walking Connor).
**Next:** 5b-5, the zombies.

## 2026-10-20 M5 step 5b-3: the stuck escape as traced (Q132)

**Changed:** `kq8-core/src/character.rs`: the stuck check starts only when the walk or run really plays; the world's diagonals NE, NW, SW, SE (±1,250 in x and y), the round starting at NE on each start of moving; all blocked: face north. Tests updated, DESIGN, provenance.
**Why:** A's answer to Q132 (a), (b) (`character-movement.md` rule 7).
**Tested how:** `cargo test --workspace`, clippy clean. Viewer: `cargo play --world Daventry --sound off --frames 2500`, `characters.log`.
**Result:** chicken1: "'walk' plays: the stuck check is on, first at 2.00" (update 15), stuck at update 30 (moved 90), north-east (2.356) free, snaps to it, runs. Unit tests: NE first, NW the next time, north when all are blocked.
**Still broken / not tested:** **Not looked at by eye.**
**Next:** 5b-4, the chase.

## 2026-10-20 M5 step 5b-2: landing on a character's top pushes Connor off (Q131 c)

**Changed:** `kq8-core/src/connor_mover.rs`: a down-pass landing on a character's top (its hit now carries the character's axis) is followed by a 250 level push away from it, as an across move. 1 new test. DESIGN, provenance.
**Why:** A's answer to Q131 (c) (`connor-physics.md`).
**Tested how:** `cargo test --workspace`, clippy clean.
**Result:** Unit test: Connor dropping onto a goblin 300 off its middle lands on its top (1,600) and is pushed out along +x; the note says he slides off its top about 250 an update.
**Still broken / not tested:** **Not looked at by eye**; not run in the viewer (needs a jump onto a goblin).
**Next:** 5b-3, the stuck escape.

## 2026-10-20 M5 step 5b-1: the exact cylinder of a character (Q131 a, b)

**Changed:** `kq8-core/src/collide.rs`: a character's cylinder is its ball stack wrapped up (bottom at its box's lowest z, height 2 × the radius for one ball), and a purely vertical move tests one ball. Plan for 5b (A's answers Q131–Q134 and the user's checks) in DESIGN. Tests updated, provenance.
**Why:** A's answer to Q131 (a), (b) in `connor-collision.md` rule 18.
**Tested how:** `cargo test --workspace`, clippy clean.
**Result:** A goblin's cylinder is now 0–1,600 (was our 1,300): the same stopping distance for Connor (1,000 from its position, which the user checked), but he meets its top 300 higher. Connor's own cylinder 0–1,300; a bat's from 1,170 above its position.
**Still broken / not tested:** **Not looked at by eye.** Landing on top is still an ordinary landing (5b-2).
**Next:** 5b-2.

## 2026-10-20 Fix (the user's check of 5a): zombies rise again after sinking

**Changed:** `kq8-core/src/character_ai.rs`: a zombie that gave up and sank back (Birth, still on its way home) now takes up the chase again by rule 28 (Connor seen within its attack distance, or it is within its home radius and he within its alarm radius) and rises `birthDelay` s later (rule 30), not before its `hide` has ended. The unit test now checks a second rise. Q134 updated. Provenance.
**Why:** The user saw zombies sink and never come back up; in the original they rise again. Built as written, a sunk zombie stayed active and on its way home, but rule 30 only starts from "not active" and our chase rules skipped Birth.
**Tested how:** `cargo test --workspace` (407 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 192916,313500,0 --hold 1,0,1 --turn 150,1 --frames 7000` (Connor runs off, then circles), `characters.log`.
**Result:** 9 rises of the road zombies in the run (RoadZ4 rose again 4 times as Connor circled back past it); none starts while `hide` still plays.
**Still broken / not tested:** **Not looked at by eye.** With Connor circling within its attack distance it gives up and rises again every few seconds (the leash loop, Q133). Whether the original uses this path is asked (Q134).
**Next:** the user's look; A: Q132–Q134.

## 2026-10-20 Fix (the user's check of 5a): the graveyard zombies gave up at once — `getLoc` while playing

**Changed:** `kq8-core/src/active.rs`: `KQObject::getLoc` / `getLoc` now work while playing (they only worked at load): `$KQObject::loc` and `$KQObject::locOnly` from the object's current place and turn. 1 new test. Provenance.
**Why:** The user saw the graveyard zombies rise for a few seconds, hide and never come back. `traps.log` showed `zomtrap.cs`'s `getloc zograv3` failing (unknown command), so every `KQMonster::setHome` got no position and their home stayed 100,000 south; 5 s into a chase they were beyond their chase radius from home (rule 27) and gave up.
**Tested how:** `cargo test --workspace`, clippy clean. Viewer: `cargo play --world Daventry --sound off --at 215000,172000,0 --frames 3000`, `traps.log`, `characters.log`.
**Result:** `getLoc zograv3: 205003 173351 8160`, `setHome: home (205003, 173351, 8160)`; the graveyard zombies rise (zograv6, 8, 9, 10 at update 15) and no give-up in about 25 s.
**Still broken / not tested:** **Not looked at by eye.** Leaving the graveyard trap (alarm 0, chase 100, moved south 10 s later) not run.
**Next:** the zombies rising again (next entry).

## 2026-10-20 M5 5a finished: the team check; STATUS, FEATURES, README

**Changed:** Docs only: STATUS (where we are, stand-ins, next steps, open questions), FEATURES (new section "Characters moving on their own"), README (M5 under way), QUESTIONS (Q128 and Q129 marked built), DESIGN (5a done).
**Why:** End of the implementation session (`docs/SESSIONS.md`).
**Tested how:** Viewer: `cargo play --world Daventry --sound off --at 138000,250000,0 --frames 1500`, `characters.log`.
**Result:** The windmill team (rule 24): sprigwnd3 sees Connor 11,176 away (alarm 12,000) and is alarmed; sprigwnd2, whose view a fence blocks, is made active and told by its team mate, as the note's check 5 expects.
**Still broken / not tested:** Nothing of 5a has been looked at by eye or compared with the original.
**Next:** The user's checks in the original; A: Q132–Q134.

## 2026-10-20 M5 step 5a-9: the flying mover corrected (Q129 built)

**Changed:** `kq8-core/src/connor_mover.rs`: `fly_update` follows `character-movement.md` rule 32: the 150 drop is all or nothing, and the raise looks down from the top of its shape to its feet against the drawn ground (the water surface) and buildings, at most 300, as an up pass; new `MoverWorld::line_hit` (the point a line meets first). `character.rs`: its log lines. 1 test rewritten. DESIGN, provenance.
**Why:** Rule 32 (A's answer to Q129), the last step of 5a.
**Tested how:** `cargo test --workspace` (406 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 161976,199500,0 --frames 2500`, `characters.log`.
**Result:** bat2 (placed at 6,790 over the lake) sinks and settles at **6,630**, the water surface, at update 3 ("dropped 150; raised 150 onto the ground"), then flies its loop over the lake at 1,159 units/s, staying at 6,630. Unit tests: over land from 1,000 it drops 150 at a time and stops 100 up (the next drop would hit the ground: cancelled); over the pond it ends every update on the surface.
**Still broken / not tested:** **Not looked at by eye.** A flyer starting less than 150 above land stays that high (the rule as written).
**Next:** the user's checks in the original (`character-movement.md` "How to check"); A: Q132–Q134, then the fight study.

## 2026-10-20 M5 step 5a-8: the zombies rise, and sink back

**Changed:** `kq8-core/src/character_ai.rs`: from Birth, a zombie seen within its alarm radius becomes active and alarmed and rises `birthDelay` s later once Connor is not standing on it (`birth`, drawn, solid, gravity on, turned to him); giving up with `hide` sends it underground again when `hide` ends. `character.rs`: the `unborn` flag. Viewer: `anim_view.rs` shows / hides the body from it, `draw_distance.rs` gives a risen one its own groups (drawn, solid). Stand-in: the chase waits until `birth` has ended (Q134 asked). 1 new test. DESIGN, provenance.
**Why:** `character-movement.md` rules 28, 30 (step 8 of 5a).
**Tested how:** `cargo test --workspace` (406 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 192916,313500,0 --frames 2500` (Connor on the road, 3,200 south of RoadZ1), `characters.log`.
**Result:** At update 15 all five road zombies see Connor and join the list; RoadZ2–RoadZ4 (alarm 18,000, birthDelay 0) rise at once and are drawn; RoadZ1 (4,000, 3 s) rises at update 60; RoadZ5 (3.5 s) doesn't rise, as Connor stands within 550 of it. After `birth` they walk after him (zombies have no `run`) and stop at 1,100 (their attack distance). Before the stand-in, they went into Move during `birth` and their stuck check snapped them away from Connor (Q134). Unit test: rise after 1 s, chase, give up when Connor is far, `hide`, underground again.
**Still broken / not tested:** **Not looked at by eye.** The graveyard trap (`zomtrap.cs`: their homes, radii, the move south) not run yet. A zombie that sank back stays active and can't rise again until it goes off (rules as written; asked in Q134).
**Next:** 5a-9, the flying mover corrected.

## 2026-10-20 M5 step 5a-7: alarm, chase, giving up, going home

**Changed:** `kq8-core/src/character_ai.rs`: the alarm (seen within the alarm radius, a random 0–1 s reaction, chase timer 5 s, the walk / run coin, telling the others and the team), the chase (turn to him standing, steer at him moving, stop within the attack distance where the fight will start), losing sight after 45 updates, the leash after the chase timer, giving up (on its way home; `hide` lists back into Birth), the way home and taking up the chase again; `KQMonster::goHome`, `setAlarmRadius`, `setChaseRadius`, `setHomeRadius` (no longer stubs in `script.rs`). No new fidget once alarmed. 2 new tests. New question **Q133**. DESIGN, provenance.
**Why:** `character-movement.md` rules 22–29 (step 7 of 5a).
**Tested how:** `cargo test --workspace` (405 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 286500,191000,0 --frames 1500` (Connor standing 3,175 from goblin22) and `--at 289284,190300,0 --hold 1,0,0 --frames 5000` (Connor walking away south), `characters.log`.
**Result:** Standing Connor: goblin22 is alarmed at update 15, reacts after 0.91 s, runs (the coin) and stops 1,474 from him (within 1,500) at update 50, "the attack would start here". Walking Connor: it follows, stopping and starting at 1,500 (our stand-in for the attack), the coin giving run or walk at each start; at update 136, 6,007 from home, it gives up (chase radius 6,000) and runs home, but takes up the chase again at once because he is within its attack distance (rule 28), and so on every 5 s (Q133). Unit tests: the whole chase, stop at reach, give up when Connor is far, home reached; `goHome` keeps it unalarmed until `goHome 0`; the three radius commands.
**Still broken / not tested:** **Not looked at by eye.** The stop at the attack distance stands in for the fight. The leash loop (Q133). Telling the others and teams not run in the game yet (the windmill spriggans); losing sight not seen in a run.
**Next:** 5a-8, the zombies rising; Q133 for A.

## 2026-10-20 M5 step 5a-6: active characters fidget

**Changed:** `kq8-core/src/character_ai.rs`: the idle rules (first fidget as soon as active, a random one of the idle list, the next 2 + round(random × 10) s after the last ended, a walking wanderer stops for it). Viewer `load.rs`: a character's `idleAnimFile` movements loaded with their models as `idle:<name>` (the model loading moved into `movement_animation`). 1 new test. DESIGN, provenance.
**Why:** `character-movement.md` rules 15–19 (step 6 of 5a).
**Tested how:** `cargo test --workspace` (403 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 283000,186000,0 --frames 3500`, `characters.log`.
**Result:** goblin22 sees Connor at update 15, joins the list, plays `react` at once, walks on at update 24 (next fidget due in 4 s), stops at update 84 for `attack`, walks on at 101 (next in 6 s), again at 191; reaches its second point at update 262 (about 13.5 s of walking for the 11,700 leg, as the note's check 2 expects, plus the fidgets). Unit test: the same sequence with a made-up fidget.
**Still broken / not tested:** **Not looked at by eye.** No alarm or chase yet (5a-7). How the idle list is swapped in is our reading (the note's open question 3).
**Next:** 5a-7, alarm, chase, going home.

## 2026-10-20 M5 step 5a-5: active characters wander their points

**Changed:** `kq8-core/src/character_ai.rs`: the wander points (`loop`, `patrol` there and back, `random`), following them (reached within 250, re-aim only after drifting 12,500 off, the next point after a stuck escape, `wanderAndStop`), `KQMonster::setHome` (no longer a stub). `character.rs`: steering ends once the character faces its point (rule 5). Viewer: `has_birth` per character. 3 new tests. DESIGN, provenance.
**Why:** `character-movement.md` rules 9–14 (step 5 of 5a).
**Tested how:** `cargo test --workspace` (402 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --frames 3500`, `characters.log`.
**Result:** chicken1 joins the list at update 15, reaches its first point (its own position) at once and turns to point 2; walking about 119 a second it is stuck at its first check (update 30, moved 90), snaps to a diagonal, runs at **947** from then on and takes the next point; points 3 and 4 reached at updates 293 and 379, then round again, as the note predicts for the chickens. Unit tests: a goblin walks to its second point and back; patrol order; random points 2,000–3,000 out; `setHome` doesn't move it.
**Still broken / not tested:** **Not looked at by eye.** No fidgets yet, so goblins and spriggans walk without stopping (5a-6). No alarm or chase (5a-7).
**Next:** 5a-6, idling.

## 2026-10-20 M5 step 5a-4: characters see Connor; the active list

**Changed:** New `kq8-core/src/character_ai.rs`: each character's movement rules (`Ai`: settings, home, eye height, what it knows), the sight test every 15th update, joining the world's active list (`KQWorld::MaxMonsters`, team members first), leaving it (off, pushed off) with the reset to home. `character.rs`, `active.rs`: the new state. Viewer: `anim_view.rs` gives each character its rules at load, `load.rs` / `door_view.rs` read `MaxMonsters` from `world.cs` and run the sight pass before the movers. 2 new tests. DESIGN, provenance.
**Why:** `character-movement.md` rules 20–22 (step 4 of 5a).
**Tested how:** `cargo test --workspace` (399 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --frames 1500`, `characters.log`.
**Result:** At the start goblin22 can't see Connor (blocked by building Cottage); chicken1 (eye 400) sees him at update 15 and joins the list (1 of 6); MaxMonsters 6 from `world.cs`. Unit tests: no test before update 15; three in Wait with room for two: the team member takes the place of the first one without a team, which (alarm radius 0) is put back home; a cart blocks the line, a character's type doesn't; going off with Connor within twice the alarm radius leaves it where it is.
**Still broken / not tested:** **Not looked at by eye.** Being active does nothing visible yet (wandering is 5a-5). The reset's distance and the 15th update's count are our readings. Hiding a zombie that goes back to Birth is not drawn.
**Next:** 5a-5, wandering.

## 2026-10-20 M5 step 5a-3: a character's own move meets objects, characters and Connor

**Changed:** `kq8-core/src/active.rs`: `characters_gravity` now builds each character's scene itself: terrain, buildings (mask 0x8), and the plain objects, other characters (cylinders where they stand now) and Connor (a cylinder from his box) whose type its mask covers in its own touch table (`OWN_TOUCH_OBSTACLES` in `character.rs`); `colliderMask=0` sees none of them. Viewer `door_view.rs` passes the object colliders. 1 new test. DESIGN, provenance.
**Why:** `character-movement.md` rules 6, 33–35 (A's answer to Q128); step 3 of 5a.
**Tested how:** `cargo test --workspace` (397 core tests), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 291102,193359,0 --command "KQMonster::setState goblin22 Move",30 --frames 1500`, `characters.log`.
**Result:** goblin22 walks into Connor and stops 1,000 from his position (its 800 + his 200, stand-in shape), is stuck at update 53 (moved 122 in 0.5 s), snaps to the free diagonal 2.356 and runs round him sliding along. Unit test: a cart (0x41) stops it 800 + 25 short of the box, a trap (0x10) does not, Connor 1,000 + 25, another goblin 1,600 + 25, `colliderMask=0` walks through all.
**Still broken / not tested:** **Not looked at by eye.** Connor's shape for a character is our stand-in (Q132 c). A character moving into Connor does not move him (rule 34: correct); Connor's own handling of a character pushing into him is the old one (it is a wall for him).
**Next:** 5a-4, sight and the active list.

## 2026-10-20 M5 step 5a-2: characters walk

**Changed:** `kq8-core/src/character.rs`: the Move state (new `Walk`): entering it (`Stop` at a random frame, then `walk` / `run`), the step from the movement playing (its frame steps × the phase moved, turned by its heading) given to its mover, steering with the 3.5 rad/s turn, the stuck check and escape, a once-a-second line; `KQMonster::setState <name> Move` now walks it straight on. `connor_mover.rs`: a character's across pass slides like an animated object (never head-on, 3 retries); the flying mover takes the step too; new line test `line_blocked` (drawn ground, buildings, objects, characters by mask). `collide.rs`: a line against a character's cylinder. `named_anim.rs` / viewer `anim_view.rs`: a character's repeating movement moves its place, not its drawing. New question **Q132** (the stuck escape's diagonals and line height; the shape of Connor for a character). 6 new tests. DESIGN, provenance.
**Why:** `character-movement.md` rules 1–8 (step 2 of 5a).
**Tested how:** `cargo test --workspace` (402 core tests pass), clippy clean. Viewer: `cargo play --world Daventry --sound off --at 286284,192527,1.5708 --command "KQMonster::setState goblin22 Move",30 --frames 2500`, read `characters.log`.
**Result:** goblin22 walks straight on at **917 units/s** (each once-a-second line: 917), meets Sara's cottage at update 93 and slides along its wall, is stuck at update 109 (moved 12 in 0.5 s): the first diagonal blocked by the ground, the second free, snap turn, `run`; after two more stuck events it runs off at **3,068 units/s**, steering off for 3 s each time. Unit tests: 917 / 3,068 a second from the steps, the model drawn at its place, the turn 3.5 rad/s and the arc, `noTurn`, stuck at 1 s and again at 1.5 s one diagonal further, all blocked → turned round, a wall slide that never stops head-on.
**Still broken / not tested:** **Not looked at by eye.** Nobody walks by themselves yet (wandering, chasing: 5a-4 to 5a-7). The character's own move does not see objects, other characters or Connor yet (5a-3). The diagonals' frame and the test line's height are stand-ins (Q132).
**Next:** 5a-3, a character's own touches.

## 2026-10-20 M5 step 5a-1: what the characters' movement rules read

**Changed:** Plan for M5's first part (5a, `character-movement.md`) in `docs/DESIGN.md`. `kq8-data/src/characters.rs`: the monster list's team, chase mode and attack distance, the type list's chase mode and attack distance (new `ChaseMode`); the character keys `homeRadius`, `chaseRadius`, `attackDist`, `team`, `canAlwaysSeeConner`, `wanderAndStop`, `doMode`; new `AiSettings` (the list's / type's values replacing the keys). `kq8 character-verify` prints them per line and `KQWorld::MaxMonsters` per world; the viewer keeps them per character and logs them at load in `characters.log`. Provenance.
**Why:** `character-movement.md` rules 9, 20, 23, 25, 27 (first step of 5a).
**Tested how:** `cargo test --workspace` (2 new tests), clippy clean; `kq8 character-verify` before and after (problems 63 both times); viewer `cargo play --world Daventry --frames 60 --sound off`, `characters.log`.
**Result:** goblin22: wanders (loop of 2), alarm 4,000, home / chase 3,000 / 6,000 (type `goblin2`), attack 1,500; sprigwnd2: team `windguys`, alarm 8,000, chase 4,000 / 9,000; bats and chickens alarm 0; RoadZ1 alarm 4,000, chase 8,000 / 12,000; the Wizard (own `.kq`): nothing. MaxMonsters: Daventry 6, Ice World 10, others 6.
**Still broken / not tested:** Nothing uses the values yet. A `.kq` character without radius keys gets 0 (our reading).
**Next:** 5a-2, walking.

## 2026-10-20 The user's check of solid characters (4d-12)

**Changed:** Docs only: FEATURES, STATUS, DESIGN, provenance.
**Why:** The user compared 4d-12 with the original.
**Tested how:** By eye in our viewer and in the original (user): walking straight and at an angle into the goblin, turning away, walking into the Wizard, the Wizard's conversation.
**Result:** All as in the original: he stops about as far from the goblin (our stand-in cylinder: 1,000 from its position) and slides round it; he can walk away; the Wizard and the conversation behave the same.
**Still broken / not tested:** Q131 stays open for A (the exact size rule, vertical moves, landing on a character's top); the user's check supports the stand-in size for the goblin. Characters moving into Connor (Q128 side).
**Next:** A's answer to Q131.

## 2026-10-19 Study (A): characters moving on their own; Q127–Q130

**Changed:** Docs only. New `docs/behaviour/character-movement.md` (walking, wandering, idling, noticing Connor, chasing up to the first swing, giving up and going home, the zombies' rising, the flying mover, a character's own touches). `characters.md`: rules 8, 29–31, 36, numbers table and open question 3 corrected / refined, link to the new note. `anim-scripts.md` rule 3: doors, chests and the waiting `Stop` (Q127). `QUESTIONS.md`: Q127–Q130 answered. STATUS. Scratch addresses in `re/notes.md` (not committed).
**Why:** The user asked for M5's first part (without the fight), and for Q127, Q129, Q130; Q128 belongs to the same topic.
**Tested how:** Traced in `Mask.exe` with Ghidra; speeds measured from the `.kqs` frame steps with a scratch script (it gives Connor's known 1,127 / 2,518, so the method holds); Daventry's `monsters.kq`, `Monstats.mtl`, the character `.kq` / `.anm` files, `BOUNDS.KQ`, `world.cs`, `zomtrap.cs`, `home1.cs` / `home6.cs` read for the data. **Nothing checked in the original game yet** (11 "How to check" items in the note).
**Result:** Speed comes only from the walk / run models (bone goblin 917 / 3,068, spriggan 1,588 / 3,366, zombie 511, bat 1,159, chicken 119 / 947); steering by the 3.5 rad/s turn, no path finding, a "stuck" escape. Only active characters (seen Connor once; at most 6 in Daventry; dropped when out of range, then put back home if Connor is far) wander, fidget (every 2–12 s) and get alarmed. Corrections to `characters.md`: the eye height is the box top, not 1,000; other characters don't block sight; the list's values replace the `.kq` keys; the flying drop is all or nothing and the raise sees the water surface (Q129: bats end on the surface). Q128: characters have their own touch table (objects, characters, Connor in their own move: obstacles). Q127: doors' and chests' own `open` replaces the waiting `Stop`. Q130: Connor's `shadowMask` is 1; `allowShadows 0` and see-through hide his shadow.
**Still broken / not tested:** Not checked in the game; the "all diagonals blocked" heading, the reset distance and the idle list swap are not sure (note's open questions 1–3); water for walking characters not followed; first person and Connor's shadow not traced; Q131 not studied.
**Next:** The user's checks in the original (`character-movement.md` "How to check"); B: build Q127–Q130, then M5's first part; A: Q131, then the fight study.

## 2026-10-20 M4 step 4d-12-3: characters solid for Connor in the viewer

**Changed:** `kq8-viewer`: `draw_distance.rs` new `CharactersSolid` (characters whose collider bit is on in the frame: near, not far, off or hidden); `door_view.rs` passes it to the running game, and `character_gravity` copies the solid characters' cylinders into a new resource `CharacterColliders` after the characters' gravity; `connor_view.rs` puts them in Connor's scene and logs each hit on a character to `characters.log` (who, where, distance from its position, surface direction, angle, slid or stayed); `anim_view.rs` gives each character its type, mask and box at load. FEATURES, README, STATUS, DESIGN, provenance.
**Why:** M4 4d-12 (`connor-collision.md` rules 2–4, 18–19; `characters.md` rule 33).
**Tested how:** `cargo test --workspace` (532 pass), clippy clean. Viewer (`cargo play --world Daventry --sound off`), reading `characters.log`, `connor.log`, `conversation.log`: start (`--frames 300`); goblin22 from the west, nearly straight (`--at 286284,192527,1.5708 --hold 1,0,0 --frames 1000`) and 500 to the side (`--at 286284,193027,1.5708 ...`); the same with `--turn 40,-1`; the Wizard (`--at 193000,198274,-1.5708 --hold 1,0,0`) and the conversation (`--at 193000,198000,-1.7 --click Wizard,30 --frames 2500`); the Uglybeast (`--at 258500,296500,2.03 --hold 1,0,0`).
**Result:** At the start goblin22 and chicken1 are solid (cylinders 800 and 210). goblin22: 3° off its middle, he touches at 1,000 from its position, slides 4 units an update at first, round it in about 50 updates, and walks on past; 34° off, 16 hits, slides round. Never closer than 1,000. With a turn key pressed after the bump he turns and walks away. Wizard (0xC021, radius 400): touched at 600, slides round him, then stops head-on at a stump (`stump2`), as at any object. The conversation still works: case "close" puts him on his spot 1,376 from the Wizard. Uglybeast (0xC022): 46 hits, all at 940, slides round. No hit ever started inside a character.
**Still broken / not tested:** **Not checked by eye.** The cylinder's size is our stand-in (Q131 a). Landing on a character's top is an ordinary landing (the 250 push asked, Q131 c). Characters still pass through Connor and each other in their own moves (Q128; they don't walk yet).
**Next:** the user's look; A's answer to Q131.

## 2026-10-20 M4 step 4d-12-2: characters in Connor's scene (core)

**Changed:** `kq8-core/src/connor_mover.rs`: the scene's new `characters` list (cylinders), tested after the objects; `Hit` says whether it is a character (`monster`), and such a hit is never head-on. `character.rs`: each character carries its collision type, mask and box. `active.rs`: `characters_solid` (set by the caller: collider on now) and `character_colliders`, rebuilt after the characters' gravity: not deleted, with a box, not in Birth, collider on, Connor's mask covers its type (0x10 ignored); a line in `characters.log` for each change. 4 unit tests. DESIGN, provenance.
**Why:** `connor-collision.md` rules 2–4, 18–19; `connor-physics.md` "Exact hit handling"; `characters.md` rules 1, 9, 16, 33.
**Tested how:** `cargo test --workspace` (532 pass, 4 new), clippy clean.
**Result:** Unit tests: walking straight into a goblin, he never gets within 1,000 of its position, is never head-on, and stays at the contact; aimed 300 to the side he slides round it and gets past; standing inside a goblin (a creature) he walks out; inside a type-0xC021 character (the Wizard's) he can't move; the list keeps a goblin and a Wizard, drops a type-0x10 one and one without a box, and logs when one becomes not solid (far, deleted).
**Still broken / not tested:** The viewer doesn't fill the list yet (4d-12-3): characters are still not solid in the game.
**Next:** 4d-12-3, the viewer.

## 2026-10-20 M4 step 4d-12-1: the ball-stack-against-cylinder test

**Changed:** `kq8-core/src/collide.rs`: new `CylinderCollider` (an upright cylinder on a ball-stack object's position: radius and height from its `BOUNDS.KQ` box, type, mask), its rough check and its sweep (`CylinderSweep`: the contact, and the balls that started inside a creature). 7 unit tests. Plan for 4d-12 (characters solid for Connor) in DESIGN. New question **Q131** (the cylinder's size; vertical moves; landing on top of a monster). Provenance.
**Why:** `connor-collision.md` rules 18–19; first step of making characters solid for Connor.
**Tested how:** `cargo test --workspace`, clippy clean.
**Result:** Unit tests: walking into a goblin (ball radius 800) stops his centre 1,000 from its position (1,025 after the pull-back), direction straight out; at an angle the direction goes through the hit point; coming down on its top stops his feet at its box height, direction straight up; from below, straight down; the first ball to reach it counts; starting inside a creature or Connor (types 0x42, 4, 0xC022) is no hit, inside the Wizard's type 0xC021 a hit where he stands; a move of 0.0000001 is not tested.
**Still broken / not tested:** Nothing uses it yet (4d-12-2). The cylinder's size is our stand-in (Q131).
**Next:** 4d-12-2, characters in Connor's scene.

## 2026-10-20 The user's check of plain objects' shadows (4d-7b-4)

**Changed:** Docs only: DESIGN, provenance.
**Why:** The user compared the barrels by the pub with the original.
**Tested how:** By eye in our viewer and in the original (user).
**Result:** One barrel casts a shadow and the others don't, in both, and it is the same barrel (`barrel.07`, the only barrel on the casting list in `shadows.log`; the others' files say `shadowMask=0`).
**Still broken / not tested:** Shadows on buildings (4d-7b-3).
**Next:** 4d-7b-3, after a question on the note's height band.

## 2026-10-20 M4 step 4d-7b-4: plain objects' shadows on the terrain (Q123 d)

**Changed:** `kq8-data/src/objects.rs`: `shadowMask` read for every placement (default 8). `kq8-viewer`: new `shadow_view.rs` (the shadow material, mesh and slot helpers moved from `anim_view.rs`, and `object_shadow_draw`): every drawn object whose distance test says it casts (world mask AND `shadowMask`, solid; not characters, not buildings) throws its shown parts onto the terrain the same way as the characters; worked out again only when its shown parts or place change. `draw_distance.rs`: the test carries the mask (0 for buildings and see-through objects). `load.rs`, `terrain_view.rs`: the world's mask as a resource. New log `shadows.log` (who casts, counts, time spent). DESIGN, provenance.
**Why:** A's answer to Q123 (d) (`characters.md` rule 37 "Plain objects").
**Tested how:** `cargo test --workspace` (521 pass), clippy clean. Viewer: `cargo play --world Daventry --frames 1300 --sound off`, terminal output kept, `shadows.log`, `viewer.log`.
**Result:** 77 of Daventry's 617 objects cast (most files say `shadowMask=0`): tables, stools, baskets, signposts, barrels, the hay bale, items lying about, …; 9 near the start, 333 pieces, each worked out once; the system takes 0.03 ms a frame; no warnings. (One run measured 17 ms a frame on average, the next 9.2 ms with the system timed at 0.03 ms: the 17 was something else on the machine.)
**Still broken / not tested:** **Not checked by eye.** Buildings still don't receive shadows (4d-7b-3). `allowShadows` is still taken as on in every world (as for characters; Castle of the Dead, Gnome and the Temples switch it off). A few click-box objects are on the "cast" list; they only throw a shadow if they have shown parts.
**Next:** 4d-7b-3 (buildings receive), after a question on the note's height band.

## 2026-10-20 The user's check of the shadows (4d-7b-2, -2b); the original's flicker not reproduced

**Changed:** Docs only: DESIGN (4d-7b-2 checked; the flicker not reproduced), STATUS, provenance.
**Why:** The user compared our shadows with the original.
**Tested how:** By eye in our viewer and in the original (user).
**Result:** The goblin's and Connor's shadows look right; wading, only the part above the water shows; at the top of a jump the shadow disappears and comes back as he comes down, as in the original. **Difference:** the original's shadow flickers and has gaps, seemingly at random; ours is smooth. The cause is not known. **The user's decision: do not reproduce the flicker**; ours stays.
**Still broken / not tested:** Shadows on buildings and from plain objects (4d-7b-3, -4).
**Next:** 4d-7b-3, -4; questions Q127–Q130 for A.

## 2026-10-20 M4 step 4d-7b-2b: Connor's shadow; no empty shadow meshes

**Changed:** `kq8-viewer/src/anim_view.rs`: `shadow_draw` also throws Connor's shown model frame onto the terrain, the same way as the characters'; the parts shared in `frame_triangles` and `show_shadow`. A shadow with nothing to draw now hides its entity and keeps its mesh (it used to get an empty mesh). `connor_view.rs`: `shown` readable by the shadow code. New question **Q130** (Connor's own cast test). Provenance.
**Why:** The user's check of 4d-7b-2 (2026-10-20): the goblin's shadow looks right, Connor has none; and Bevy warned every frame "Mesh … contains no vertices" (our empty shadow meshes: a character in range with no shadow, e.g. the spriggans on the castle).
**Tested how:** `cargo test --workspace` (521 pass), clippy clean. Viewer: `cargo play --world Daventry --at 212000,286000,0 --frames 400 --sound off`, terminal output kept and searched, `characters.log`.
**Result:** Connor: 384 model triangles, about 216 facing the sun, about 275 pieces each frame. The warning: 0 times in the run (three characters there have no shadow).
**Still broken / not tested:** Connor's shadow not checked by eye (the jump's 1,500 cut and wading not run). His cast test is ours (Q130).
**Next:** the user's look; then 4d-7b-3 (buildings) and 4d-7b-4 (plain objects).

## 2026-10-19 M4 step 4d-7b-2: characters' shadows on the terrain (Q123)

**Changed:** `kq8-viewer/src/anim_view.rs`: new `shadow_draw` (PostUpdate, after the transforms): for each character that casts (rule 36) and is drawn (near or far, not hidden), every frame its shown model frame's triangles go through `kq8_core::shadow` onto the terrain into its own mesh, drawn unlit with multiply blending at 0.5, no haze; a line per shadow once a second in `characters.log`. `load.rs`: the rule-36 result is kept on the character. `main.rs`: the system. DESIGN, provenance.
**Why:** A's answer to Q123 (`characters.md` rule 37) and the user's check of the look.
**Tested how:** `cargo test --workspace` (521 pass), clippy clean. Viewer (`cargo play --world Daventry --sound off`): `--frames 400` (start), `--at 212000,286000,0 --frames 300` (castle ruins); read `characters.log` and `viewer.log`.
**Result:** goblin22: 224 model triangles, about 107 facing the sun, 24 terrain triangles, about 150 pieces each frame. At the ruins: RuinSprig1, RuinSprig2 and Sprigan cast onto the terrain; RuinSprig3, RuinSprig4 (3,609 above) and bossprig (6,041 above) cast nothing on the terrain (beyond 1,500), as rule 37 says. Average frame 7.7 ms (start), 9.4 ms (ruins).
**Still broken / not tested:** **Not checked by eye** (please look: a dark copy of the figure falling toward +y, about 1.5 sideways per unit of height, half as bright, darker where parts overlap). Buildings don't receive shadows yet (4d-7b-3); plain objects don't cast yet (4d-7b-4). The blending (multiply by 0.5) is our reading of "strength 0.5" (not traced).
**Next:** 4d-7b-3, shadows on buildings.

## 2026-10-19 M4 step 4d-7b-1: shadow geometry (Q123)

**Changed:** New `kq8-core/src/shadow.rs`: the sun direction (first sun's aim made length 1, else (1, 1, −1)); `cast` (each sun-facing model triangle, cut to the part above the receiving plane, projected along the sun onto the plane lifted 20, cut to the receiving triangle, each piece kept on its own); `terrain_receivers` (the surface grid's triangles under the box moved along the sun, none beyond 1,500 from the ground). 6 unit tests. Shadow plan in DESIGN (4d-7b-1 to -4). Provenance.
**Why:** A's answer to Q123 (`characters.md` rule 37).
**Tested how:** `cargo test --workspace` (521 pass, 6 new), clippy clean. Nothing draws it yet.
**Result:** Unit tests: straight-down sun keeps the area and lifts 20; Daventry's sun (0, 1, −0.66) moves a point 1,000 up by 1,000/0.66 toward +y (less the lift); a triangle below the receiving plane casts nothing, one crossing it only its upper part; two triangles over one spot give two pieces; a shadow is cut to its receiver.
**Still broken / not tested:** `terrain_receivers` has no unit test (needs a terrain); it is first used in 4d-7b-2.
**Next:** 4d-7b-2, characters' shadows drawn on the terrain.

## 2026-10-19 M4 step 4d-6b-3: the flying mover (Q122 built)

**Changed:** `kq8-core/src/connor_mover.rs`: `fly_update` (steps 0, gravity off, the ordinary move, then down by up to 150 stopped as the down pass, then up by up to 300 if the terrain is above its feet within its height). `character.rs`: `flyingObject=1` characters use it, logged (first update, moving, settling on what). 1 unit test. New question **Q129** (the order of the flying drop and the `keepOnGround=1` ground check; buildings in the raise). `QUESTIONS.md` Q122 marked built. DESIGN, provenance.
**Why:** A's answer to Q122 (c) (`characters.md` rule 8).
**Tested how:** `cargo test --workspace` (515 pass, 1 new), clippy clean. Viewer: `cargo play --world Daventry --at 152000,201400,1.57 --frames 500 --sound off`, read `characters.log`.
**Result:** bat3 (6,350 above the terrain) comes down 150 per update and settles on the terrain at 7,650 at update 44. bat2 (over water) settles at 6,480, 150 under the water surface: the literal order (ground check onto the surface, then the drop to the floor side), asked as Q129.
**Still broken / not tested:** Not looked at by eye. The raise ignores buildings (Q129 b). How flyers fly about is M5. The Knight is not in the world before his scene.
**Next:** 4d-7b, shadows (Q123): plan first.

## 2026-10-19 M4 step 4d-6b-2: characters fall onto the ground and buildings (Q122)

**Changed:** `kq8-data/src/characters.rs`: `colliderType` / `colliderMask` read (defaults 0x42 / 0x281F). `kq8-core`: `Mover::character` takes the type (Fall only with bit 4); `Character` carries its mover, `gravity_step` (in range, not in Birth, standing still; a script's `move` is its start; logs the first update, leaving the ground, landing on what); `World::characters_gravity`. Viewer: each character gets a mover at load (its `BOUNDS.KQ` box, steps, `keepOnGround`), `draw_distance.rs` `character_box`, new fixed-update step `character_gravity` right after `door_step` (buildings only for a mask with 0x8). The click box follows. 1 unit test. DESIGN, provenance.
**Why:** A's answer to Q122 (`characters.md` rule 8).
**Tested how:** `cargo test --workspace` (514 pass, 1 new), clippy clean. Viewer (`cargo play --world Daventry --sound off`): `--at 146500,248928,1.5708 --frames 300` (windmill), `--at 212000,286000,0 --frames 400` (castle ruins), `--at 258500,296500,0.6 --frames 300` (Uglybeast), `--at 193000,198000,-1.7 --frames 300` (Wizard); read `characters.log`.
**Result:** sprigwnd1 (305 above the terrain) settles 12 onto the Windmill's floor (z 15,338); RuinSprig3 and RuinSprig4 (3,635 above the terrain) settle 26 onto the Castle (z 22,224); bossprig (6,035 above) is already standing on it; the others stand on the floor grid. The Wizard (mask 0): no terrain or building tests, stays at his floor. The Uglybeast: on the water surface.
**Still broken / not tested:** Not looked at by eye. Objects, other characters and Connor do not stop a character's fall (Q128). Flying characters (the bats, the Knight) do not move yet (4d-6b-3). Characters are still not solid for Connor.
**Next:** 4d-6b-3, the flying mover.

## 2026-10-19 M4 step 4d-6b-1: the mover's switches for characters

**Changed:** `kq8-core/src/connor_mover.rs`: three switches on the mover, all on (or off) for Connor as before: `falls` (the Fall state and "steep ground switches gravity on", type bit 4), `terrain_in_passes` (the terrain in the up and down passes, mask bit 0x800), `keep_on_surface` (`keepOnGround=1`: the ground check onto the water surface). New `Mover::character` (box as ball stack, own steps, mask, `keepOnGround` −1/1/0). 3 unit tests. Plan for gravity in DESIGN (4d-6b-1 to -3). New question **Q128** (which of `connor-collision.md` rule 3's reactions a character's own move has: by that table plain objects and Connor would not stop its fall). Provenance.
**Why:** A's answer to Q122 (`characters.md` rule 8, `connor-physics.md` "Other characters on the same mover").
**Tested how:** `cargo test --workspace` (513 pass, 3 new; Connor's mover tests unchanged), clippy clean. No viewer run: nothing uses it yet.
**Result:** Unit tests: a character dropped 3,000 lands with no Fall state and no events (Connor would have started Fall after 5 updates); with mask 0 no terrain hit, the ground check catches it; with `keepOnGround=1` it stands on a water surface at 500, off the water on the floor.
**Still broken / not tested:** Not used by the viewer yet (4d-6b-2).
**Next:** 4d-6b-2, characters fall in the viewer.

## 2026-10-19 M4 step 4d-11: transitions on characters (Q124 built)

**Changed:** `kq8-core/src/named_anim.rs`: the movement player plays `transitionN` lines (`connor-movement.md` rules 2–6): a request first chooses the transition whose from point comes next (−1 at once; a point already passed on a play-once is never reached; a newer request chooses again), waits for it (that update cut at the point), plays the transition movement from 0 at speed 1, and at its ordinary end starts the target at its phase (then asks again for another wanted movement). Like Connor's walker, the transition and its target move from the next update. `door.rs`: movements carry their transitions; `anim_view.rs`, `door_view.rs` pass them on; `load.rs` logs them as played. `character.rs`, `active.rs`: log lines. 3 unit tests. `QUESTIONS.md` Q124 marked built. DESIGN, provenance.
**Why:** A's answer to Q124 (`characters.md` rule 20).
**Tested how:** `cargo test --workspace` (510 pass, 3 new), clippy clean. Viewer: `cargo play --world Daventry --at 600,100,0 --frames 900 --sound off --command "sendEvent 0 Girl KQAnimPlayEvent cower",20 --command "sendEvent 0 Girl KQAnimPlayEvent stop",25`, read `characters.log`, `viewer.log`.
**Result:** Unit tests: `cower` asked for `stop` at phase 0.33 plays on to 0.882353 (cut there), `cowerstop` plays 15 updates, ends as an ordinary end, `stop` starts at 0; in Wait the end of `cowerstop` brings `Stop` back at a random frame (rule 20); after 0.88 no transition, `stop` waits for the end of `cower`. Viewer: the request runs the Girl's start-up, its Wait asks for `Stop` and chooses `cowerstop` from 0.882353; `viewer.log`: "movement cower has transitions, a character's: played".
**Still broken / not tested:** The Girl is never in range in a new game (she stands at (100, 100, 0)), so the transition has **not been seen playing in the viewer**. The `endLoop` cue (rule 20 b) waits for M5 (only on the Knight's `attack`). Backwards transitions are built as the note's "the other way round" but no data has one.
**Next:** 4d-6b, gravity (Q122): plan first.

## 2026-10-19 M4 step 4d-10: the end of a `speak` (Q117 built)

**Changed:** `kq8-core/src/character.rs` (`after_speech`) and `active.rs` (`talk_step`): when the line marked last of a `speak` ends in a character's queue (for a conversation, the partner's), before its `TalkComplete`: already in Wait, nothing (no `Stop`); otherwise `Stop` is asked for and it goes into Wait, accepted from no state, Talk, and Special once its list's last movement has started (else refused, the list goes on). Was: when the queue ran dry, for the last line's speaker, from no state or Wait. 1 unit test. `QUESTIONS.md` Q117 marked built. DESIGN, provenance.
**Why:** A's answer to Q117 (`talking.md` rule 13a).
**Tested how:** `cargo test --workspace` (507 pass, 1 new), clippy clean. Viewer (`--sound off`): `--at 252000,296000,0.6 --click Uglybeast,30 --frames 1500` (the "too far" line); `--at 193000,198000,-1.7 --click Wizard,30 --frames 11000` (the whole conversation); read `characters.log`, `conversation.log`.
**Result:** The Uglybeast's line: "already in Wait: nothing", then `ugly.cs` sends `stop` on `TalkComplete 1000 59 0 32 2` (as A's answer to Q120 b says). The Wizard: at each conversation end TalkExit comes first (Wait), then rule 13a finds it in Wait and does nothing (updates 1694 and 1979).
**Still broken / not tested:** Not looked at by eye. Walking out of Talk (rule 13a's first case) waits for M5. A Special list whose movement is replaced by this `Stop` stalls (literal reading; no Daventry case known).
**Next:** 4d-11, transitions on characters (Q124).

## 2026-10-19 M4 step 4d-9: a character's start-up (Q116, Q121 built)

**Changed:** `kq8-core/src/character.rs`: the start-up (`start_up`) runs once, at the first update or before the first movement request (`KQAnimPlayEvent`, a lip-sync code, Wait's or a Special list's request, a conversation), whatever the range; only with no state does it queue Birth or Wait; the Wait comes `birthDelay` s later in game time and is checked after each pass over the event queue (`wait_due`, whatever the range), so a delay of 0 lands before the first picture; mouth shape 0. Gone: the "empty name, no state" step and the Q121 stand-in. `active.rs`: `start_up_character`, `characters_due`. Tests: 3 replaced by 3. `QUESTIONS.md` Q116 and Q121 marked built. DESIGN, provenance.
**Why:** A's answers to Q116 and Q121 (`characters.md` rules 12, 15).
**Tested how:** `cargo test --workspace` (506 pass), clippy clean. Viewer (`cargo play --world Daventry`): `--frames 60`; `--at 170500,191000,0 --frames 400` (goblin18); `--frames 200 --command "KQMonster::setState sprdump special dump",10` (from afar); `--at 193000,198000,-1.7 --click Wizard,30 --frames 2200 --sound off`; read `characters.log`, `conversation.log`.
**Result:** goblin18: start-up at update 0 on `stop`, frame 0, speed 1; Wait at update 15 (1 s), `Stop` restarted at random frame 4, speed 1.085, as rule 12 describes. Characters with `birthDelay` 0 (goblin22, chicken1) get Wait and the random `Stop` in update 0, before the first picture. sprdump, sent its Special list while far away: the start-up runs at that request ("already in Special: kept, nothing queued"), then `dump` starts at once. The Wizard conversation runs as before (Talk at update 30, the camera cuts).
**Still broken / not tested:** Not looked at by eye. The Lady's `Init` (`KQAnimPlayEvent up`) not run: she is loaded later in the story.
**Next:** 4d-10, the end of a `speak` (Q117).

## 2026-10-19 M4 step 4d-8: the movement names start as `Stop` (Q121's correction)

**Changed:** `kq8-core/src/named_anim.rs`: new `with_stop_names`: both names start as `Stop`, `currAnimName` / `nextAnimName` replace them, the current movement is the one the name finds (else movement 0 under that name). Used for animated objects and characters (`anim_view.rs`) and scripted objects (`door_view.rs`); doors and chests keep the old load names. `viewer.log` lists each object whose first movement changed. 1 unit test. Plan in DESIGN ("A's answers Q116, Q117, Q121–Q124"). New question **Q127** (doors, chests and what clears the waiting `Stop`). Provenance.
**Why:** A's correction of `anim-scripts.md` rule 3 and `characters.md` rules 12, 14 (found while answering Q121).
**Tested how:** `cargo test --workspace` (506 pass, 1 new), clippy clean; `cargo play --world Daventry --frames 60`, read `viewer.log` and `anim.log`.
**Result:** 55 animated objects and characters now start on another movement than before: every Daventry character without `currAnimName` starts on `stop` (was movement 0, `attack`; the Lady `stop`, was `down`), as rule 14 says. The fish start at random frames and speeds (`fishies.01` 0.92, `.02` 1.02, `.03` 1.05; were all speed 1).
**Still broken / not tested:** Not looked at in the viewer by eye. Doors and chests wait for Q127. The start-up itself (rule 12) is still the old stand-in until 4d-9.
**Next:** 4d-9, the start-up.

## 2026-10-19 Study (A): shadows, second user's check (jump, water)

**Changed:** `docs/behaviour/characters.md` rule 37 (the 1,500 cut, water), "How to check", open question 2. No code.
**Why:** The user checked fading and water in the original.
**Tested how:** By eye in the original (user); compared with Connor's jump height in `connor-physics.md`.
**Result:** No fading with distance. Connor's shadow vanishes at the top of a standing jump and comes back on the way down: the jump peaks about 1,697 above the ground, past the traced 1,500 cut. Shadows lie on the water surface; a wading Connor's shadow shows only the part above the water.
**Still broken / not tested:** How the game leaves out parts below the water: not traced. The exact blend: not traced.
**Next:** B: build shadows from rule 37.

## 2026-10-19 Study (A): shadows, the user's check

**Changed:** `docs/behaviour/characters.md`: the user's results under "How to check" (points 6–8, 10), open question 2 narrowed. No code.
**Why:** Checks 6–10 listed after Q123.
**Tested how:** By eye in the original (user).
**Result:** Shadows fall to one side, like the hills' shadows (sun direction confirmed); about half as bright, a quarter where parts overlap (fits strength 0.5); characters and barrels cast shadows on building floors and walls (buildings receive, plain objects cast). Bats: can't be told (only visible once in range).
**Still broken / not tested:** The exact blend, fading with distance, shadows over water, the flying mover's settling: not traced or checked.
**Next:** B: build shadows from `characters.md` rule 37.

## 2026-10-19 Study (A): Q116, Q117, Q119–Q124 (characters: timing, mouth, start-up, gravity, shadows)

**Changed:** `docs/behaviour/characters.md`: rules 8 (what the mover collides with, flying, no Fall), 12 (start-up, rewritten), 14, 15 (corrected), 18 (same update; zero-length movements), 20 (transitions, `endLoop` cue), 27 (mouth poses hold), 36 (`allowShadows` data corrected), new 37 (how shadows are drawn); numbers, checks, open questions. `talking.md`: new rule 13a (the speaker goes to Wait). `anim-scripts.md`: rule 3 corrected (names start as `Stop`), rule 8 (delay-0 events are handled before drawing), rule 9, new rule 10a (loop duration 0 never ends). `connor-physics.md`: new section on characters. `QUESTIONS.md`: the eight marked answered. `STATUS.md`. Scratch addresses in `re/notes.md` (not committed). No code.
**Why:** B's open questions from M4 4d; Q119 and Q120 show as a snap and a flicker in our build.
**Tested how:** Traced in `Mask.exe` with Ghidra (event queue and frame loop, end of movement, line-complete handler, animation player rate, start-up and state switch, object constructors and key reader, monster-list parser, mover and flying mover, shadow drawing for animated and plain objects, `shadowvisible`). Data: `ugly.anm`, the mouth models' loop durations (0.0), `monsters.kq` columns, `ugly.cs`, `lady.cs`, `knight.anm` (the `endLoop` cue is on `attack`), every script's `allowShadows`, a scan of all worlds' objects for the `Stop` default (scratch script).
**Result:** Q119: everything sent with delay 0 runs before the frame is drawn, so `stand` replaces `Stop` before anything shows: no snap. Q120: the mouth poses have loop duration 0 and never end, so no `Stop` between codes; the "too far" line ends with `ugly.cs` sending `stop` (matches the user's check). Found on the way: every animated object starts with the names `Stop`/`Stop`, so every Daventry character starts on `stop`, the Lady and the Shadow are in Wait (not "no state"), and `flags=8` torches and fish do get the random start (corrects `anim-scripts.md` rule 3). Q122: characters are type 0x42 / mask 0x281F, terrain and buildings count, no Fall or fall death. Q123: shadows go along the sun, onto terrain (within 1,500) and buildings, each triangle projected separately, lifted 20, strength 0.5; plain objects the same; `shadowvisible` is only for the relighting tool. Two failed tries recorded: my first reading of the Lady's start (with an empty current name) gave a broken scene, which led to finding the `Stop` default.
**Still broken / not tested:** Nothing checked in the running original yet (checks listed in `characters.md` "How to check" 6–10). Not traced: how the renderer blends the 0.5 shadow, distance/haze fading of shadows, which model side the shadow keeps, shadows over water; the flying mover is traced but not fully sure.
**Next:** B: drain delay-0 events before drawing (fixes the snap), let zero-duration movements hold (fixes the flicker), the `Stop` default names, then gravity and shadows. User: the checks in `characters.md` "How to check" 6–10.

## 2026-10-19 M4 4e-8 to 4e-10: the user's check

**Changed:** `STATUS.md`, `docs/FEATURES.md`: the user's result. No code.
**Why:** The user ran the three tests of 4e-8 to 4e-10 in our viewer.
**Tested how:** By eye in our viewer (user).
**Result:** All three fixed: the right-button camera turns between the Wizard and Connor as in the original; the Uglybeast no longer snaps to idle at the end of its turn; its far-away line no longer flickers.
**Still broken / not tested:** The exact centre point of the right-button camera is still our stand-in (Q126). Q125 open. A's answers Q116, Q117, Q121–Q124 not built.
**Next:** B: A's remaining 4d answers, or the main menu. A: Q125, Q126.

## 2026-10-19 M4 step 4e-10: events sent with delay 0 are handled before drawing (Q119 built)

**Changed:** `kq8-core/src/active.rs`: at the end of every update the event queue is handled again, until nothing is due now (`handle_due`), so the ends of movements (`EndLoop`, `Cue 300`), a line's notify-group `TalkComplete` and what scripts send with delay 0 during the update arrive in the same update, before the frame is drawn (was: the next update). Four older tests changed to the new timing, 1 new. `QUESTIONS.md` Q119 marked built. DESIGN, provenance.
**Why:** A's answer to Q119 (`anim-scripts.md` rule 8): nothing sent with delay 0 waits for the next update. The one-update wait showed as the Uglybeast's snap at the end of `pivot` (`Stop` drawn for a frame at the old heading before `ugly.cs`'s `move` and `stand`), which the user saw again (2026-10-19).
**Tested how:** `cargo test --workspace` (505 pass), clippy clean. Viewer: `--at 258500,296500,0.6 --click Uglybeast,30 --frames 1800 --sound off`, read in `characters.log`, `doors.log`.
**Result:** Unit test: an `EndLoop` script's `sendEvent 0 … KQAnimPlayEvent stop` starts `stop` in the same update `open` ends. A door's `EndLoop` and its listener's `Cue 300`, a chest's and a basket's `EndLoop`, and the notify group's `TalkComplete` now come one update (1/15 s) earlier than before. Viewer: in update 80 `pivot` ends, Wait asks for `Stop`, then (same update, before drawing) `EndLoop pivot`, `ugly.cs`'s `move Uglybeast 3` to (262035, 298992) and `stand`, which replaces `Stop`: `Stop` is never drawn (before: drawn in update 80, `stand` in 81).
**Still broken / not tested:** Not checked by eye. This shifts the timing of M3's checked scenes (doors, chests, cutscene cues) by one update earlier; worth a look. Scripts run after the update (traps, in the viewer's `trap_step`) still have their delay-0 events handled at the start of the next update.
**Next:** The user's check of the Uglybeast and the right-button camera; B: A's answers Q116, Q117, Q121–Q124.

## 2026-10-19 M4 step 4e-9: a movement whose loop lasts 0 never ends (Q120 built)

**Changed:** `kq8-core/src/door.rs` (`DoorMovement::step_with`): a model loop of 0 s (or less) neither moves nor ends, loop or play-once (was: a play-once one ended at once). 1 unit test. `QUESTIONS.md` Q120 marked built. DESIGN, provenance.
**Why:** A's answer to Q120 (`anim-scripts.md` rule 10a): the Uglybeast's mouth poses last 0 s, so they hold; our "ended at once" made Wait bring `Stop` back between mouth codes: the flicker the user saw again (2026-10-19).
**Tested how:** `cargo test --workspace` (504 pass, 1 new; the new test fails without the change), clippy clean. Viewer (`--sound off`): `--at 252000,296000,0.6 --click Uglybeast,30 --frames 1500` (the "too far" line) and `--at 258500,296500,0.6 --click Uglybeast,30 --frames 1800`, read in `characters.log`.
**Result:** Unit test: a geometry talker in Wait given code 1 holds `f` for 30 updates, no `Stop`. Viewer, "too far" line: 16 mouth poses, 0 returns to `Stop` between them (before: `Stop` after every pose); at the end of its queue our Q117 stand-in still asks for `Stop` (A's answer: nothing while in Wait; `ugly.cs` sends `stop` itself), not changed here.
**Still broken / not tested:** Not checked by eye. Q117's answer not built. Which other models last 0 s (the note asks to check) is not counted: our model check does not report loop lengths.
**Next:** Q119 (the snap at the end of the Uglybeast's turn).

## 2026-10-19 M4 step 4e-8: the right-button camera turns between the two (the user's check)

**Changed:** `kq8-core/src/conversation.rs`: with the right button held during a conversation the camera is placed at `cam0` aimed at, and swings around, the point half-way between the partner and Connor (was: the partner). Unit test changed. `QUESTIONS.md` Q126: the user's observation added. DESIGN, provenance.
**Why:** The user's check in the original (2026-10-19): the camera pivots in the space between the Wizard and Connor. The note says "aimed at the partner"; the exact point is not traced (Q126).
**Tested how:** `cargo test -p kq8-core conversation` (11 pass). Viewer (after the open viewer was closed): `--at 193000,198000,-1.7 --click Wizard,30 --right 200 --frames 2600 --sound off`, read in `conversation.log`.
**Result:** Unit test: Bob at (1000, 1000, 0), Connor at (1200, 1000, 100) → the swing's centre (1100, 1000, 50). Viewer: at update 200 the camera at `cam0` aimed at (191489, 198182, 8030), swinging around (191489, 198182, 6630), half-way between the Wizard (190807, 198274) and Connor's spot.
**Still broken / not tested:** Not checked by eye (only `--right`, no real drag). The end of a conversation still puts the camera at `cam0` looking at the partner.
**Next:** Q120 and Q119 (the Uglybeast's flicker and snap).

## 2026-10-19 End of M4 4e (docs only)

**Changed:** `STATUS.md` (where we are, the 4e stand-ins, next steps, open questions Q125–Q126; older entries shortened; A's new entries kept), `README.md` (conversations under M4 "So far"; out of "Next"), `docs/FEATURES.md` (a "Conversations" section, `--right N`, what doesn't work).
**Why:** `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** Nothing to run (documents only). Full suite before: `cargo test --workspace` 503 pass, clippy clean.
**Result:** —
**Still broken / not tested:** Conversations not looked at by eye yet. Session A answered Q116–Q124 during this session (`characters.md`, `talking.md` rule 13a): rule 13a agrees with the conversation's end as built (partner to Wait before the last `TalkComplete`); the rest is 4d work, not built.
**Next:** The user's check of a conversation in the viewer; B: A's answers to Q116–Q124, or the main menu; A: Q125, Q126.

## 2026-10-19 M4 step 4e-7: Connor's gestures

**Changed:** `kq8-core/src/connor.rs`: `Walker::gesture` (codes 8–15 `gesturea`…`gestureh`, 16 `stop`, only while he stands with the keys choosing). `active.rs`: Connor's lip-sync codes 8–16 are passed on (`connor_gestures`). Viewer: `load.rs` loads his eight gesture movements; `door_view.rs` plays them on his walker; `talk_view.rs`: a log text. 1 unit test. DESIGN, provenance.
**Why:** Step 4e-7 of `conversation.md` (rule 18; Q114 left Connor's gestures for this step).
**Tested how:** `cargo test --workspace` (503 pass, 1 new), clippy clean. Viewer: `--at 193000,198000,-1.7 --click Wizard,30 --frames 2200 --sound off`, read in `connor.log`.
**Result:** Line 1 of the first meeting (Connor's, `A0RS040E.0R1`): code 12 at update 35 plays `gesturee`, 11 `gestured` (54), 14 `gestureg` (76), 13 `gesturef` (96), 16 `stop` (114, 122); the gestures play to their end (`gesturee` 19 updates) unless the next code comes first. The first try found no gesture in his list: the viewer loaded only the movements M2 and M3 needed; the eight gestures (46–51 frames, can be cut short) are now loaded too.
**Still broken / not tested:** **Not looked at by eye.** No weapons and no first person yet, so those two conditions never block a gesture. Gestures during his `speak` remarks (not conversations) follow the same rule but were not run.
**Next:** End of 4e: STATUS, README, FEATURES; the user's check in the viewer; A: Q125, Q126.

## 2026-10-19 M4 step 4e-6: goodbye and welcome back, run in the viewer

**Changed:** No code (built in 4e-2 and wired in 4e-3). `docs/DESIGN.md`, provenance (the viewer check).
**Why:** Step 4e-6 of `conversation.md` (rules 20–22): the note's "walk away, then click him again" check.
**Tested how:** Viewer (`--sound off`): `--at 193000,198000,-1.7 --hold 1,0,0,55 --command "KQConner::conversation Wizard 1000 4 14 27 1 13 0",20 --command "move Connor 0 193000 198000 -1 0 0 -1.7",130 --click Wizard,150 --frames 9000`; read in `conversation.log`, `talk.log`.
**Result:** As the note: out of the stay area at update 47 with 13 lines left: the goodbye 1000/4/0/7/1 ("Well, then, off with you!…") cuts line 1 at 1.27 of 6.8 s; it ends at update 108: the queue pauses with 13 lines waiting, `TalkExit`, no `TalkComplete`. Clicked again from the note's spot at update 150 (`ConnorAction do do`, `wizard.cs` gives the same conversation): the range is a duplicate (not added twice); the welcome back 1000/4/0/8/1 ("Ah, I see you have seen fit to return…", its camera 4) plays, no `TalkComplete`; then line 1 from its start (update 267), line 2, … with their cameras.
**Still broken / not tested:** **Not looked at by eye or heard.** The first walk-away run of this step wandered into the special area behind the Wizard (`Special` on the second click): the held forward key and the turn toward the Wizard took Connor through him (characters are not solid yet, 4d).
**Next:** 4e-7: Connor's gestures.

## 2026-10-19 M4 step 4e-5: conversation cameras and the right button

**Changed:** `kq8-core/src/conversation.rs`: a conversation line with a camera number cuts the camera to `cam<n>` (checked against `nCam`) and turns Connor to the partner once he is at his spot; the right button held during the stay test places the camera at `cam0` aimed at the partner and swings it around the partner, and later camera numbers are ignored. `active.rs`: `Request::CameraAround`; the line starts of the update are looked at. Viewer: `game_camera.rs` swings around a point instead of Connor (`around`; `follow` ends it); `door_view.rs` passes the right button to the conversation; new test option `--right N` (`main.rs`). FEATURES (`--right`), DESIGN, provenance.
**Why:** Step 4e-5 of `conversation.md` (rules 6, 14, 15).
**Tested how:** `cargo test --workspace` (502 pass, 1 new), clippy clean. Viewer (`--sound off`): `--at 193000,198000,-1.7 --click Wizard,30 --frames 32000`, and the same with `--right 200 --frames 9000`; read in `conversation.log`, `camera.log`.
**Result:** The Wizard's first meeting (lines 1–13, then 14 after the map scene) cuts to cameras 2 3 3 5 2 1 1 1 1 3 5 2 3 6, the note's list; Connor turns to the Wizard at each. `cam2` sits at (189954, 199109, 8090) looking along (0.81, −0.55, −0.19), toward Connor's spot. With `--right 200`: at update 200 the camera goes to `cam0` (196075, 200403, 8086) aimed at (190807, 198274, 8030), swings around the Wizard, and cameras 3, 5, 2 and 1 of the next lines are ignored. At the end the camera goes to `cam0` looking at the Wizard, then follows Connor (built in 4e-3).
**Still broken / not tested:** **Not looked at by eye**, and no real mouse drag tried (only `--right`, which presses nothing in the camera itself, so no swing was made). Q126 stand-in for the aim point and the swing's rules. The release of the right button's lock on entering (rule 6) is not built (logged as nothing to do).
**Next:** 4e-6: goodbye and welcome back in the viewer.

## 2026-10-19 M4 step 4e-4: clicking a conversation partner, `InvConv`

**Changed:** `kq8-core/src/active.rs`: a click on a character with an info object goes by the areas (`click_partner`: `Special`, `do` / the held item, `TooFar`), not by the reach; during the talk state a world click does nothing except one on the partner with an item held (`InvConv`, Connor leaves with `TalkExit`, then the ordinary click). 2 unit tests. DESIGN, provenance.
**Why:** Step 4e-4 of `conversation.md` (rules 5, 19).
**Tested how:** `cargo test --workspace` (501 pass, 2 new), clippy clean. Viewer (`--sound off`): `--click Wizard,30` from `--at 193000,198000,-1.7`, `191000,197000,-1.7` and `198000,198000,-1.7`; and `--at 193000,198000,-1.7 --item INVITEM_SilverCoins --command "KQConner::conversation Wizard 1000 4 14 27 1 13 0",20 --click Wizard,90`; read in `doors.log`, `talk.log`, `conversation.log`.
**Result:** As the note's "How to check": (193000, 198000) start area → `ConnorAction do do` → the first meeting ("Hello?! You're still alive!…"); (191000, 197000) special area `box2` (wins over `box0`) → `Special` → "Come around, lad...to where I can see you."; (198000, 198000) no area → `TooFar` → "Hello, there, sir wizard!"; the click turn each time. With the coins held during the conversation: `InvConv INVITEM_SilverCoins` to `wizard.cs`, the conversation ends with `TalkExit`, then `ConnorAction TestItem INVITEM_SilverCoins` starts conversation 1000/4/8/86, queued behind the 13 lines still waiting (rule 19: not emptied).
**Still broken / not tested:** **Not looked at by eye.** The horn on the Uglybeast during a conversation (`ugly.cs` answers `InvConv`) not run. A real mouse click during the talk state not tried (test clicks only).
**Next:** 4e-5: camera cuts and the right button.

## 2026-10-19 M4 step 4e-3: the conversation command and the two talk states

**Changed:** New `kq8-core/src/conversation.rs` (`KQConner::conversation`; Connor's talk state: the case, the approach, the stay test, the end; `TalkExit`). `connor.rs`: a `Talk` mode (the approach movement, else the keys; no Jump, no push). `character.rs`: the partner's Talk state (turns toward Connor every 4th update unless `noTurn`; Wait at the end). `active.rs`: the command, `speak` ignored while talking, `clearSpeak` on a partner (rule 23), the conversation's last line ends it before its `TalkComplete`, a goodbye's end sends `TalkExit`, a GUI order for the bottom bar. `talk.rs`: the "queue empty" event says which queue. Viewer: `load.rs` reads the info objects and the GB/WB keys; `door_view.rs` runs Connor's side after his move (new `conversation.log`) and doesn't cut lines short while he talks; `gui_view.rs` hides and brings back the bottom bar; `connor_view.rs`: the talk state allows the click turn. DESIGN, provenance.
**Why:** Step 4e-3 of `conversation.md` (rules 1, 2, 6–13, 15–17, 23).
**Tested how:** `cargo test --workspace` (499 pass, 6 new), clippy clean. Viewer (`--sound off`): `--at 193000,198000,-1.7 --command "KQConner::conversation Wizard 1000 4 14 27 1 13 0",30 --frames 32000`; `--at 195000,200000,1.0 --command "… 1 2 0",30`; `--at 193000,198000,-1.7 --hold 1,0,0 --command "… 1 13 0",20`; `--at 258500,296500,0.6 --click Uglybeast,30 --frames 34000`; read in `conversation.log`, `talk.log`, `connor.log`, `doors.log`.
**Result:** Wizard from the note's spot: case "close", 834 from the spot, Connor put on (192171, 198091, 6700); 13 lines; the last ends at update 1694: `TalkExit`, then `TalkComplete 1000 4 14 27 13`, which runs the map scene; line 14 then starts a new conversation with Connor 1,085 from the spot ("close"; the note says 1,083). Case "walk" from (195000, 200000): `stopwalk` 7 updates while turning, put 440 short at (192536, 198337) facing the spot, `walkstopright` ends 29 from the spot, then he turns to the Wizard. Walking away (forward held): out of the stay area at update 47 with 13 lines left: the goodbye "Well, then, off with you!…" cuts line 1, Connor goes to Move, `TalkExit` at the goodbye's end (update 108). Uglybeast: `ugly.cs` puts Connor 1 unit from the spot: case "there"; at the end `TalkExit` (cue 2 sent 0.5 s later), then `TalkComplete 1000 59 0 6 11`, as the note says; `gui.log`: the bottom bar slides away at update 82 and back at 1095.
**Still broken / not tested:** **Not looked at by eye.** Clicking a partner still uses the ordinary reach test (4e-4); no camera cuts (4e-5); the welcome back on return not run (4e-6); Connor's gestures (4e-7). Stand-ins: Q125 (keys during the approach, Jump and push), Q126 (camera aim). `clearSpeak` on a partner not run.
**Next:** 4e-4: clicks on a partner (areas, `InvConv`).

## 2026-10-19 M4 step 4e-2: the partner's conversation queue

**Changed:** `kq8-core/src/talk.rs`: a second queue per partner (`<name>Conv`) with its goodbye and welcome-back lines (`conv_queue`, `conv_start`, `conv_goodbye`, `clear_conv`); lines report their camera number when they start (conversations only); a finished line says whether it was the conversation's last; the goodbye pauses the queue and is reported for `TalkExit`; the logs name the queue (`WizardTalk`, `WizardConv`). `active.rs`: one test's expected log text. DESIGN, provenance.
**Why:** Step 4e-2 of `conversation.md` (rules 12, 14, 21–23).
**Tested how:** `cargo test --workspace` (493 pass, 3 new), clippy clean.
**Result:** Unit tests: a conversation of three lines plays with cameras 2, none, 5 and its third line is reported as the last; walking away during line 2 cuts it, the goodbye plays, the queue pauses with lines 2 and 3 waiting, no `TalkComplete`; told to start again: welcome back, line 2 from its start, line 3, the range not added twice; on the last line no goodbye; `clearSpeak` empties it.
**Still broken / not tested:** Nothing calls it yet (4e-3). Not run in the viewer.
**Next:** 4e-3: the command and the two talk states.

## 2026-10-19 M4 step 4e-1: reading the conversation info objects (plan for 4e)

**Changed:** `docs/DESIGN.md`: plan for 4e (conversations, 4e-1 to 4e-7). New `kq8-data/src/conv_info.rs` (`KQConvInfo`: talking spot, areas, cameras; the inside test). `kq8-data/src/characters.rs`: the goodbye and welcome-back keys (`GB…`, `WB…`). New `kq8 conv-verify` (`kq8-cli/src/conv_verify.rs`). `docs/behaviour/QUESTIONS.md`: Q125 (Connor's keys before the stay test, Jump and push while talking), Q126 (where the conversation camera aims). Provenance.
**Why:** First step of conversations (`conversation.md` rules 3, 20).
**Tested how:** `cargo test -p kq8-data` (125 pass, 4 new), clippy clean; `cargo run --release -p kq8-cli -- conv-verify`, read in `kq8-artifacts/logs/conv-verify.log`.
**Result:** 169 `KQConner::conversation` calls, all with `<enemy>` 0; 19 characters with goodbye/welcome-back keys; every partner's largest camera number is below its camera count (Daventry: Wizard and Uglybeast 6 of 7). The note's Daventry points: (193000, 198000) in areas 0 and 1, 834 from the spot; (191000, 197000) in 0 and 2; (198000, 198000) in none: all as the note. **17** info objects, not the note's 16: Barren has three (`WitchdrInfo`, `AlthillmanInfo` in `Altarman.kq`, `FrghillmanInfo` in `shaftinf.kq`); the note lists only the Witch Doctor. Not a question (no Daventry effect); worth a word to A.
**Still broken / not tested:** Nothing uses the info objects yet (4e-3).
**Next:** 4e-2: the conversation queue.

## 2026-10-19 Bars and captions in every world (Q118 built)

**Changed:** `kq8-viewer/src/load.rs`: new `screen_palette_paths` (the world's `KQWorld::objectPalette` from `world.cs`, then `loadingPalette`, `interiorPalette`, `terrainPalette`, `connorPalette`; each in the world's `resource` folder, then `GAME/Resource`). `gui_view.rs` (bars) and `talk_view.rs` (caption strip) use it instead of `<folder>.ppl`. `docs/provenance/README.md`, `STATUS.md`.
**Why:** `--world deadcity` and seven other worlds started with no bars and the stand-in caption panel: no `<folder>.ppl` there (Q118, `interface.md` rules 1c–1g).
**Tested how:** `cargo clippy -p kq8-viewer --all-targets` clean; `cargo play --world <W> --frames 30` (60 for DeadCity and Daventry) for all 12 worlds, reading `viewer.log` and `gui.log`.
**Result:** Every world logs its palette, e.g. "gui: … palette DeadCity/resource/newdead.ppl (world.cs: KQWorld::objectPalette newdead.ppl …)", CastleD `Castle.ppl`, IceWorld `ice.ppl`, SnowExit `exit.ppl`, Temple1–4 `temple01`–`04.ppl`; the caption uses the same file. DeadCity `gui.log` update 1: "the bars show: quest strip ["NoName28", "NoName29", "Mask1a"] … slots [3, 1, 2, 0, …], rope Some("NoName27")". Daventry unchanged (`daventry.ppl`).
**Still broken / not tested:** The fallbacks after `objectPalette` are not tested (no shipped world needs them). **User (2026-10-19):** the bars show in all worlds.
**Next:** Back to M4 4d (characters).

## 2026-10-19 Study (A): Q118, the user's check (world palette)

**Changed:** `docs/behaviour/interface.md`: new rule 1i (each world's `8gui` pictures are converted to its own palette), the user's result under "How to check" point 8. No code.
**Why:** The user compared the bars in Daventry, the Dead City and the Swamp in the original.
**Tested how:** By eye in the original (user). Then decoded each world's own `4red`, `4green`, `stontran`, `4left` and `2gold0` pictures with its own palette (script in the scratch folder).
**Result:** User: only the health and experience meters differ (a slightly different red and green). Data agrees: the world copies of the pictures differ byte for byte; the greys come out the same, the red is 152,8,0 / 224,24,0 / 184,0,0 and the green 0,104,0 / 16,72,0 / 8,72,0. A first try that put Daventry's pictures in the other palettes gave purple and orange, so a world's pictures must go with that same world's palette.
**Still broken / not tested:** Nothing open. Escape to the main menu and back (user, later the same day): nothing changes, as rule 1d says.
**Next:** B: use the world's own `8gui` pictures with its `objectPalette`.

## 2026-10-19 Study (A): Q118, which file is the world palette

**Changed:** `docs/behaviour/interface.md`: rule 1 and rule 2 extended, new section "The world palette" (rules 1a–1h), edge case updated, "How to check" point 8. `docs/behaviour/QUESTIONS.md`: Q118 answered. `STATUS.md`. Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** B's guess `<folder>/resource/<folder>.ppl` exists in only four worlds, so eight worlds drew no bars (user's report).
**Tested how:** Traced in `Mask.exe` with Ghidra (screen activation, `loadPalette`, `blackOut`, `loadPalettes`, the main window's palette, the camera's per-frame drawing, world loading). Checked the data: every `world.cs` (and the two `patch/WORLD.CS`) names one file for all five settings; searched every shipped `.cs`/`.kq` for `loadPalette`, `blackOut`, `loadPalettes` (only `world.cs` Load and `mask.cs`); `newdead.ppl` world and `GAME/Resource` copies have the same MD5.
**Result:** The screen with `useWorldPal=1` loads no palette; the window has one palette for everything, and every frame the 3D view ends on the world's `objectPalette`, which is what the panels show. `defaultPaletteName` in `GAME/Kq/<World>.kq` is never read (CastleD's names a missing file). Nothing changes the palette during play; after the menus the next frame puts the world palette back. A missing file leaves the last palette that loaded.
**Still broken / not tested:** Not checked in the running original; that the panels are drawn after the 3D view is inferred; which resource folder wins when both have the file is not traced (the copies are identical).
**Next:** B: read `objectPalette` from `world.cs` for the bars and captions (fall back to `loadingPalette`), then check in all twelve worlds.

## 2026-10-19 Q118: bars missing in eight worlds (question for A)

**Changed:** `docs/behaviour/QUESTIONS.md`: Q118 (which file is the game-play screen's world palette). No code.
**Why:** User's report: `--world deadcity` (and others) starts with no bars. `gui_view.rs` and `talk_view.rs` look only for `<folder>.ppl`, which exists only in Barren, Daventry, Gnome and Swamp; the others name theirs differently (`newdead.ppl`, `castle.ppl`, ...), so no palette is found and no bars are drawn.
**Tested how:** Read `viewer.log` (`gui: no palette among ["deadcity/resource/deadcity.ppl", ...]; no panels drawn`); listed each world's `resource/*.ppl` and `world.cs` palette lines; compared world and `GAME/Resource` copies (identical).
**Result:** Cause found; not fixed.
**Still broken / not tested:** Bars and caption strip missing in DeadCity, CastleD, IceWorld, SnowExit, Temple1–4.
**Next:** A answers Q118; B then reads the palette name from `world.cs` for the bars and captions.

## 2026-10-19 Q124: transitions and the `endLoop` cue on characters (question for A)

**Changed:** `docs/behaviour/QUESTIONS.md`: Q124. Viewer `load.rs`: the log line about the Girl's transition no longer says "from step 4d-3" (it was not built there) but "waits for Q124".
**Why:** `characters.md` rule 20 says transitions and the `endLoop` cue work for characters as for Connor, but not how a transition movement's end meets `EndLoop` and Wait's return to `Stop` (rule 18), nor whether the movement stops at the `endLoop` cue; `anim-scripts.md` rule 9 calls the cue M5. Daventry's only uses (the Girl's `cower` → `stop` via `cowerstop` at 0.882353; the Knight's `birthx`) are not reachable in a test run.
**Tested how:** Data checked (`girl.anm` `transition0=0.882353 stop 0.000000 cowerstop`); build only.
**Result:** —
**Still broken / not tested:** Character transitions and the `endLoop` cue not built.
**Next:** A: Q116, Q117, Q119–Q124.

## 2026-10-19 Q114 built: the duplicate check on the last line; Connor's mouth ignores codes above 7

**Changed:** `kq8-core/src/talk.rs`: a `speak` is a duplicate when its range's **last** line (not its first) is waiting or playing; the log calls codes 8–16 gestures. Viewer `talk_view.rs`: Connor's lip-sync codes above 7 leave his mouth as it is (was: `lip0`). `QUESTIONS.md` Q114 marked built; provenance.
**Why:** A's corrections to `talking.md` rules 9, 17, 18 (Q114), asked of B.
**Tested how:** `cargo test --workspace` (486 pass; 1 new test, 1 changed), clippy clean, release build.
**Result:** During line 1 of 1…2, a new 1…2, 1…1 or 2…2 adds nothing; after the remark ends it is queued again.
**Still broken / not tested:** Connor's mouth change not run in the viewer (no reachable Connor line with codes above 7 found). Connor's gestures (codes 8–16) not built: they come with conversations (`conversation.md` rule 18).
**Next:** conversations (`conversation.md`), when the user asks; 4d-6 and the 4d-7 drawing wait for Q122, Q123.

## 2026-10-19 M4 step 4d-7a: who casts a shadow (drawing on hold, Q123)

**Changed:** `kq8-data/src/characters.rs`: `shadow_mask_setting` (`setShadowMask` in `Options.cs`), `casts_shadow` (rule 36). Viewer `load.rs`: a "shadow" line per character in `characters.log`. `QUESTIONS.md`: Q123. DESIGN, provenance.
**Why:** Step 4d-7: the casting rule is described; the drawing is not (direction, what it lies on, darkness).
**Tested how:** `cargo test -p kq8-data` (121 pass, 1 new). Viewer `--frames 60`, read in `characters.log`.
**Result:** World mask 15 from `Options.cs`. 51 of 57 characters cast; no shadow for `chicken1`–`3`, `piggy1` (`shadowMask=0`), the Knight (165) and the Shadow (180) (see-through): as the note lists.
**Still broken / not tested:** No shadows drawn (Q123). `allowShadows` not read from the world's scripts (taken as on, its default).
**Next:** Q114's corrections for Connor's lip sync.

## 2026-10-19 M4 step 4d-6a: where characters start above the ground (gravity on hold, Q122)

**Changed:** Viewer `load.rs`: a "gravity check" line per character in `characters.log` (its z, the floor and surface grid heights under it, gravity, `keepOnGround`, `flyingObject`). `QUESTIONS.md`: Q122. DESIGN.
**Why:** 4d-6 (gravity) needs to know which characters would move; what a character's mover collides with is not in any note.
**Tested how:** Viewer `--frames 60`, read in `characters.log`.
**Result:** 46 of 57 characters start on the floor grid (gap 0); `bat1` on the water surface (638 above the floor). The other 10: `Ravenw` 615 and `sprgnbow` 210 (gravity off), `sprigwnd1` 305 (windmill), `RuinSprig3`/`4` 3,635 and `bossprig` 6,035 (ruins), `sprdump` 35, the Uglybeast 5 (gravity on); bats `bat2` 160 above the surface, `bat3` 6,350 (`flyingObject=1`).
**Still broken / not tested:** No gravity for characters: Q122 (their collision type/mask, buildings and terrain in their passes, flying characters, falls and fall death).
**Next:** 4d-7a: who casts a shadow (the drawing waits for a question).

## 2026-10-19 M4 step 4d-5c: a character's click box follows it

**Changed:** `kq8-core/src/active.rs`: every update a clickable character's click box takes the character's place and turn. Provenance.
**Why:** Until now the box stayed where the character was loaded, so after the Wizard's scripted moves (the ring scene) or a turn, clicks would miss him or hit empty space.
**Tested how:** `cargo test -p kq8-core` (new unit test `a_character_s_click_box_follows_it`: `move wiz 0 5000 0 0 …` moves the box to (5000, 0, 0)).
**Result:** As expected in the unit test.
**Still broken / not tested:** Not tried in the viewer (the Wizard's moves happen in his ring scene, which needs his conversation).
**Next:** 4d-6 plan.

## 2026-10-19 M4 step 4d-5b: `makeTranslucent`

**Changed:** `kq8-core/src/active.rs`: `makeTranslucent <name> <n>` for animated objects and characters (it was an unknown command); `AnimObject::translucency`. Viewer `anim_view.rs`: the object's parts get their own material copies and opacity n / 255 (256 solid). DESIGN, provenance.
**Why:** `characters.md` rule 22: `wizard.cs` makes the Wizard see-through (150) during the ring spell and solid (256) after.
**Tested how:** `cargo test --workspace` (483 pass, 1 new), clippy clean. Viewer: `--at 189300,198274,0 --command "makeTranslucent Wizard 150",20 --command "makeTranslucent Wizard 256",60`, read in `characters.log`.
**Result:** Update 21: opacity 1.000 → 0.588 on 3,347 parts (64 own materials); update 61: back to 1.000.
**Still broken / not tested:** **Not looked at by eye.** Connor's `makeTranslucent` (Dead City, M6) not built. The ring scene itself not run.
**Next:** a character's click box follows it; 4d-6 plan.

## 2026-10-19 M4 4d-3 to 4d-5: the user's check

**Changed:** `docs/behaviour/QUESTIONS.md`: the user's observations added to Q119 and Q120. No code.
**Why:** The user ran the four test procedures of 4d-5.
**Tested how:** By eye in our viewer (user).
**Result:** (1) The Wizard breathes, speaks with mouth and gestures, doesn't turn, stands again: as described. (2) The Uglybeast flickers a lot between two animations while speaking (Q120: our literal reading of rule 18). (3) A very brief snap back at the end of the Uglybeast's turn (Q119; the original shows none). (4) Chickens and pig stand doing their idle animation; the farm spriggan's one `attack` at the start was not seen (probably out of view).
**Still broken / not tested:** Q119 and Q120 wait for A; the spriggan's first `attack` not confirmed by eye.
**Next:** A: Q116, Q117, Q119–Q121. B: plan 4d-6 (gravity).

## 2026-10-19 M4 step 4d-5: the characters' mouths and gestures

**Changed:** `kq8-core/src/character.rs`: `mouth_code` (codes 0–7 mouth shape, a geometry talker's mouth movements, gestures 8–16), `after_speech` (Wait after the last line, Q117 stand-in). `active.rs`: the talk events of each update reach the characters; `Active::test_command`. Viewer: `load.rs` loads each character's `lip0`–`lip7` textures; `anim_view.rs` gives a character its own copies of its face materials and swaps `lip<n>` on them; new test option `--command "WORDS",N` (`main.rs`, `door_view.rs`). `QUESTIONS.md`: Q120. FEATURES (characters, `--command`), DESIGN, provenance.
**Why:** Step 4d-5 of `characters.md` (rules 25–28, Q115).
**Tested how:** `cargo test --workspace` (482 pass, 2 new), clippy clean. Viewer: `--at 189300,198274,0 --command "KQMonster::speak Wizard Connor 1000 4 0 8 1 1",30` and `--at 258500,296500,0.6 --command "KQMonster::speak Uglybeast Connor 1000 59 0 32 1 0",30`, read in `characters.log`.
**Result:** Wizard: his turn is blocked (`noTurn`); the line plays gestures `gesturee` (code 12), `gesturea` (8), `gesturee`, `gesturec` (10), then `stop` (16); his mouth texture changes about 40 times (`lip1`…`lip7` on his 15 face materials), back to `lip0` at the end; then Wait (`Stop`). Uglybeast: 16 mouth codes, each starting its mouth movement at once.
**Still broken / not tested:** **Not looked at by eye or heard.** Q120: in Wait the Uglybeast's one-frame mouth movements end and `Stop` comes back between codes (rule 18 read literally). Q117 stand-in for "goes to Wait". Connor's own codes 8–16 still change his mouth to `lip0` (Q114, for the conversations step). The Girl's and the Lady's mouths not run (not reachable or not loaded in a test).
**Next:** 4d-6 (gravity and steps): plan it in detail first.

## 2026-10-19 M4 step 4d-4: characters turn

**Changed:** `kq8-core/src/character.rs`: a turn request turns the character at 3.5 rad/s toward a point on the spot (`ask_turn`, `turn_step`, `heading_of`); `noTurn` / `setCanTurn 0` ignore it. `active.rs`: `speak` with a listener asks the issuing character to turn (it was only logged). DESIGN, provenance.
**Why:** Step 4d-4 of `characters.md` (rule 23).
**Tested how:** `cargo test --workspace` (480 pass, 3 new), clippy clean. Viewer: `--at 252000,296000,0.6 --click Uglybeast,30` (too far: `ugly.cs` makes the Uglybeast speak), read in `characters.log`, `doors.log`.
**Result:** Unit tests: a quarter turn takes 7 updates (6 × 0.233 rad and the rest), lands exactly on the heading, `stop` keeps playing; `noTurn` ignores the request. Viewer: "Uglybeast: speak to Connor: blocked: noTurn (rule 23)".
**Still broken / not tested:** **A real turn was not seen in the viewer**: Daventry's turning speakers without `noTurn` (the ravens, the Lady, the Henchman) only speak in scenes our test options can't reach yet. Which way a character's model faces is our reading (as Connor: −Y). The Talk state's turn (every 4th update) waits for conversations. A clickable character's click box doesn't turn with it.
**Next:** 4d-5: the mouth and gestures.

## 2026-10-19 M4 step 4d-3: the characters' states

**Changed:** New `kq8-core/src/character.rs` (first update, Birth, Wait and `Stop`, the return to `Stop`, no state, `setState`, Special lists). `active.rs`: a character is an `AnimObject` with a `character` part, updated only while near or far; `setState`, `setUseGravity`, `setMonUseGravity`, `setCanTurn`, `changeAnmFile` for characters. `named_anim.rs`: `randomize`. Viewer: `draw_distance.rs` tells core which characters are near or far (`CharactersInRange`); `anim_view.rs` builds the states, writes core's lines to `characters.log` with a count by state every 10 s; `door_view.rs` gives a script-loaded character its distance test. `QUESTIONS.md`: Q119, Q121 (first numbered Q118 by mistake: another session used Q118 at the same time; renumbered). DESIGN, provenance.
**Why:** Step 4d-3 of `characters.md`.
**Tested how:** `cargo test --workspace` (477 pass, 10 new), clippy clean. Viewer runs at the Wizard (`--at 189300,198274,0`), the Uglybeast (`--at 258500,296500,0.6 --click Uglybeast,30`), the farm (`--at 262000,212000,1.0`), the Shadow (`--at 239000,183200,0`), read in `characters.log`, `doors.log`.
**Result:** Wizard: `attack` (still) at update 0, Wait and `Stop` at update 1 (plain: `flags=32`). Ravenw: `Stop` waits for `attack`, starts at update 44 at random frame 40, speed 1.059. Chickens and pig start `Stop` at random frames 28, 15, 7. The farm spriggan plays `attack` once (about 1 s) before `Stop` (rule 14). Zombies in Birth. The Shadow: rule 12.1, no state. Characters far away are not updated (52 of 57 at the Wizard's house).
**Still broken / not tested:** **Not looked at by eye.** Q119: at the end of the Uglybeast's `pivot`, `Stop` shows for one update at the old heading before the script's `move` and `stand` (our `EndLoop` goes through the queue). Q121 stand-in for `sprdump` (not run). Script-loaded characters (Lady, ravens, Unicorn) not run. No turning, mouth, gestures, gravity.
**Next:** 4d-4: turning.

## 2026-10-19 M4 step 4d-2: characters drawn where they stand

**Changed:** Viewer `load.rs`: `loadKQ monsters.kq` places every line through its type's file (renamed, the line's position and heading, z −1 on the ground); a character's `animFile` list is attached so it is drawn like an animated object (one model per movement); skin swap (`load_animation_skinned`); characters clickable with their rule-9 box, not solid. `anim_view.rs`: characters' ball from the rule-9 box, characters with `Birth` (the zombies) hidden, new `kq8-artifacts/logs/characters.log`; a drawn animated object now follows its place in core (a script's `move`). `draw_distance.rs`: `test_character`. `docs/DESIGN.md`, provenance.
**Why:** Step 4d-2 of `characters.md`: the orange cubes become the characters.
**Tested how:** `cargo test --workspace` (467 pass), clippy clean. Viewer runs: `--frames 150` (start), and `--at 258500,296500,0.6 --click Uglybeast,30 --frames 2500`, read in `characters.log`, `viewer.log`, `doors.log`.
**Result:** 52 of 52 monster-list lines placed; 57 characters drawn (52 + Girl, Wizard, Ravenw, Shadow, Uglybeast), 21 zombies hidden, 0 characters left as markers. Wizard at (190807, 198274, 6630) heading −1.57; Knight and Henchman at (0, 0); the Girl at (100, 100). Boss spriggan's `test2.bmp` drawn as `test3.bmp`. Clicking the Uglybeast: `ugly.cs` plays `pivot` (update 30–80, its four grass sounds), `EndLoop pivot`, `move Uglybeast 3` → (262035, 298992), `stand`; then `KQConner::conversation` is unknown (conversations not built).
**Still broken / not tested:** **Not looked at by eye.** No states yet: characters play their load movement as plain objects (the Wizard shows his still `attack` pose, not `Stop`; one-shots like the ravens' `attack` hold their last frame). Not solid; no gravity, no turning, no mouth. The Lady, the three script-loaded ravens and the Unicorn are prepared but no run reached their scripts. A clickable character's click box stays where it was loaded after a `move` (the Wizard's ring scene).
**Next:** 4d-3: the states (first update, Wait and `Stop`, the return to `Stop`, Special lists).

## 2026-10-19 M4 step 4d-1: reading characters (plan for 4d)

**Changed:** `docs/DESIGN.md`: plan for 4d (characters, 4d-1 to 4d-7). New `kq8-data/src/characters.rs` (monster list lines, the type list `Monstats.mtl`, a character's keys with their defaults, the `KQLips` section, `.anm` movement lists). New `kq8 character-verify` (`kq8-cli/src/character_verify.rs`). `docs/provenance/README.md`. `docs/behaviour/QUESTIONS.md`: Q116 (`birthDelay` is not 0 for two Daventry goblins), Q117 ("the speaker goes to Wait" is not in `talking.md` rule 13).
**Why:** First step of drawing the characters (`characters.md`, session B).
**Tested how:** `cargo test -p kq8-data` (120 pass, 6 new), clippy clean; `kq8 character-verify` on the install.
**Result:** Daventry: 22 `KQMonster` + 2 `KQGeometryTalker` files and 52 monster-list lines, **0 problems**, as the note says; every line finds its type, file and box (the Knight's by name). `Stop` frames: `wizstop` 31, `chk_stop` 30, `UG_STOP` 40, `Ravnstp` 46 (the note's numbers). The Wizard's, the Girl's and the Lady's face texture are in their `Stop` models. Only the zombies have `Birth`. `goblin18` and `goblin20` have `birthDelay` 1 (the note says 0 for all: Q116). All worlds: 154 + 7 files, 595 lines, 63 problems, none in Daventry (Swamp's list skips 15 numbers, its mandragora types point at files that are not characters; later worlds, M6).
**Still broken / not tested:** Nothing drawn yet (4d-2).
**Next:** 4d-2: draw the characters where they stand.

## 2026-10-19 Study (A): characters, the user's check

**Changed:** `docs/behaviour/characters.md` rule 36 (shadows) and "How to check" (the user's results), open question 2 narrowed. `docs/DESIGN.md` M7 improvements: shadows drawn as one even shape.
**Why:** The user checked the five points of the note in the original.
**Tested how:** By eye in the original (user).
**Result:** (1) the Wizard doesn't turn to Connor: matches. (2) the Uglybeast's turn is a model movement, its heading unchanged: matches (`pivot` is a movement). (3) chickens: can't be told by eye. (4) zombies hidden until they rise near Connor: matches. (5) shadows are a dark copy of the figure; overlapping parts darken the ground twice.
**Still broken / not tested:** Shadow direction and `shadowvisible` not traced; whether chickens wander only after seeing Connor not confirmed.
**Next:** B: draw the characters from `characters.md`, check Q115.

## 2026-10-19 Study (A): characters (drawing and animating, not fighting)

**Changed:** New `docs/behaviour/characters.md` (36 rules): which classes are characters, the monster list and types, the drawing keys, the ground and the box, what plays at load, Wait's return to `Stop`, the Birth state, script commands on characters, turning, the mouth, gestures, when a character notices Connor, visibility and shadows. `docs/behaviour/conversation.md` rule 18 corrected (gesture condition) and open question 2 answered. `docs/behaviour/QUESTIONS.md`: Q115 (the correction, for B). `STATUS.md`. Scratch notes with addresses in `re/notes.md` (not committed).
**Why:** Other characters are still drawn as orange cubes; the next build step is to draw them so that conversations can be built on real, animated characters (Daventry first).
**Tested how:** Traced in `Mask.exe` with Ghidra; checked against the data with small scripts in the scratch folder (Daventry's 24 character files and 52 monster-list lines, animation lists, models' textures, `BOUNDS.KQ`, lip-sync files by speaker).
**Result:** Characters wait for Connor to be in range before their first movement; Wait asks for `Stop` at a random frame and speed 0.9–1.1 and comes back to it after any one-shot; `noTurn` blocks every turn (the Wizard, the Uglybeast); zombies are not drawn until born; the Uglybeast's mouth is 8 whole-model movements; gestures need only a movement that can be cut short (so `speak` lines gesture: the Knight's 6 files); 91 of 93 Daventry gesture lip files end with code 16 (`stop`); with `Options.cs`'s `setShadowMask 15`, characters cast shadows by default (mask 2) unless `shadowMask=0` or see-through.
**Still broken / not tested:** Nothing checked in the running original (the note lists five things to look at). Not traced: how shadows are drawn, `shadowvisible`, `flags` bit 0x20, wandering / idling / alarm (M5), `changeBmp`, `changeAnmFile`.
**Next:** B: draw the characters from `characters.md`, check Q115; then conversations.

## 2026-10-19 Study (A): conversations, the user's check

**Changed:** `docs/behaviour/conversation.md` rule 14 and open question 1: holding the right mouse button during a conversation places the camera once at `cam0` aimed at the partner and stops the camera cuts for the rest of that conversation. Scratch note in `re/notes.md`.
**Why:** The user checked the original: points 1–4 of "How to check" all as described; moving the camera cancels the cuts and lets it turn freely around the two talking.
**Tested how:** Traced the Connor flag the talk state tests: it is set while the camera-move (right) button is pressed.
**Result:** Matches the user's observation.
**Still broken / not tested:** Open questions 2 and 3 of the note.
**Next:** As before.

## 2026-10-19 Study (A): conversations

**Changed:** New `docs/behaviour/conversation.md` (23 rules). `docs/behaviour/talking.md`: rule 9 corrected (the duplicate check uses the range's last sequence, not the first), rules 15, 17, 18 (lip-sync codes 8–16 are gestures, not mouth shapes), open questions 1 and 2 answered. `docs/behaviour/QUESTIONS.md`: Q114 (the correction, for B). `STATUS.md`. Scratch notes with addresses in `re/notes.md` (not committed).
**Why:** Next study topic after M3 (STATUS "Next step"): the Wizard's and the Uglybeast's conversations in Daventry need it.
**Tested how:** Traced in `Mask.exe` with Ghidra; checked against the data with small scripts in the scratch folder (message files, `*info.kq`, lip-sync files, script calls).
**Result:** 169 `KQConner::conversation` calls, all with enemy flag 0; 16 `KQConvInfo` objects (Daventry: Wizard 3 boxes 7 cameras, Uglybeast 2 and 7); the largest camera number in every world's lines is within its partner's camera count; 19 characters have goodbye/welcome-back keys; 1,110 lip-sync files use gesture codes (Daventry 93). The Wizard's scripted spots fall in his stay area (1,083 and 344 units from the talking spot, so Connor is placed on it).
**Still broken / not tested:** Nothing checked in the running original yet (the note lists four things to look at). Not settled: the Connor setting that makes the talk state put the camera at `cam0` and stop the camera cuts; what else enables gestures; states that delay the talk state. Whether a line cut off by the goodbye sends its own "finished" message.
**Next:** B: build conversations from `conversation.md` and check Q114. A: Q108–Q113 when convenient.

## 2026-10-19 M3 is done (docs only)

**Changed:** `docs/DESIGN.md`: M3 marked done, its step plans (3a to 3o) moved unchanged to the new `docs/milestones/M3.md`, a short list left in their place. `README.md`: M3 "✅ Done", the M3 line without "(mostly)" (plus: the camera follows Connor as in the original), "What's left of M3" taken out of "Next". `docs/FEATURES.md`: the camera (view heading, right button, chest height), what's missing. `STATUS.md`: "Where we are", goal, what works, stand-ins, open questions, test counts (461). `QUESTIONS.md`: Q27, Q53, Q100, Q102 marked built; Q113 d and e settled by the user's checks, a–c left for A (not blocking).
**Why:** The user's check of the camera against the original after 3o-7: (a) Connor drifts to one side before the view catches up and locks, (b) the right button held keeps the camera's place relative to him, (c) the view points at about his chest; and the end of the watermill scene (`wtrmwalk.cs`): Connor a little left of centre, no snap, as ours. Then `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** Nothing to run (documents only).
**Result:** —
**Still broken / not tested:** Q113 a–c (the view heading's state around cutscene cameras) wait for A; low priority.
**Next:** A: conversations (`KQConner::conversation`), Q108–Q113 when convenient. B: the main menu with the game paused.

## 2026-10-19 M3 step 3o-7: the right button (the user's first check)

**Changed:** `kq8-core/src/camera.rs`: heading rule 2's "dragging" is now the right button held, moving or not (was: the mouse moved this update); the angle on screen it keeps is measured from where the camera was before that update's swing (was: after). Viewer `game_camera.rs`: pressing and letting go of the button each run a camera update. Unit test rewritten (a fast swing). `docs/DESIGN.md` (3o-7), provenance, `QUESTIONS.md` (Q113 e, for A to confirm).
**Why:** The user's check in the original: (1) a fast drag moved our camera far off to the side, leaving Connor off screen (a slow one only a little); the original only rotates round Connor wherever he is on screen. Cause: our lock took Connor's angle after the first swing of the drag, so a fast first swing became the kept offset. (2) In the original a click alone (no mouse movement) lets him drift afterwards, and (3) holding the button still keeps him at the same spot on screen; ours needed the mouse to move.
**Tested how:** `cargo test --workspace` (461 pass), clippy clean. Viewer: the walk-zone run of 3o-2 (`--at 282250,196409,3.1416 --hold 1,0,0,90`): `camera.log` identical to `camera-3o3-walkzone.log` (no mouse).
**Result:** Unit test: Connor 0.1 rad off; pressing the button locks at 0.1; swings of half a window per update turn the view by the whole swing, he stays 0.1 off; held still while he walks, still 0.1 off; let go: idle, offset 0.
**Still broken / not tested:** **Not tested in play** (needs the real mouse): the user's check again.
**Next:** The user's check.

## 2026-10-19 M3 step 3o-6: a script's `move` and the trap he was in

**Changed:** `kq8-core/src/trap.rs`: `Trap::stay_check`, only rule 7's stay test (nothing comes in). `active.rs`: `Active::connor_placed` runs it for every trap and calls the scripts (logged "stay test right after a script's move, rule 5"). Viewer `door_view.rs`: when a script's `move Connor` is applied, the stay test runs at the new spot at once and its calls are logged with the move. 2 unit tests. Provenance, `docs/DESIGN.md`.
**Why:** A's answer to Q102 b (`traps.md` rule 5 "Placed by a script"): the trap a script's `move` takes Connor out of gets `Off` right after the script, not at his next move.
**Tested how:** `cargo test --workspace` (461 pass, 2 new), clippy clean. Viewer: Connor walking south into the mill wheel's second trap, `cargo play --world Daventry --at 186300,234800,0 --hold 1,0,0 --frames 2500 --sound off` (`traps-3o6-wheel2.log`, `camera-3o6-wheel2.log`).
**Result:** `wheeltp2` On at update 22; its own line 8 moves him to (186300, 232800), still inside: "no trap left". Cue 2 (line 20) moves him under the wheel to (185800, 211300, 5610): "trap wheeltp2: Off Connor (stay test right after a script's move, rule 5)" and "script wheeltp2.cs wheeltp2 Off Connor" in the same tick (before: 60 updates later, after `handsOff 0`). `wheeltp1` is not entered (the jump's path is not swept, as before). The log labels the move's application one update number higher than the script line; it is the same tick (the counter goes up at the end of the scripts' update; already so before this step).
**Still broken / not tested:** Not seen by eye; nothing visible changes in Daventry (`wheeltp2.cs` does nothing on `Off`). What an `Off` script asks for is applied in the next update (our order).
**Next:** The user's check of the camera against the original; then close M3.

## 2026-10-19 M3 step 3o-5: the deep lift on the drawn surface; one `follow` finding both

**Changed:** `kq8-core/src/camera.rs`: the height rule's deep lift tests the line from the update's start to the new spot against the drawn surface (the exact test of 3o-4, through the new `ViewBlockers::surface_hit`) and puts the camera on the hit point; if the line meets nothing the camera stays at its deep spot (the old stepped floor-grid test and its "ground + 1,400" stand-in are gone); `follow_switch`: the "turn back" switch after a `follow`, the 0.4 rad test first, then the distance test. `camera_flight.rs`: flights and cuts pass the surface test to the same rule. Viewer `game_camera.rs` (`surface_line`, `follow` uses `follow_switch`), `door_view.rs` (the cut). 1 unit test new, 1 extended. Provenance, `docs/DESIGN.md`.
**Why:** A's answers to Q100 b and c (`camera.md` "Height above the ground" rule 2.1, "When the camera updates at all" item 3).
**Tested how:** `cargo test --workspace` (459 pass), clippy clean. Viewer (`--sound off`): the watermill (`camera-3o5-watermill.log`), the mill wheel (`camera-3o5-wheel.log`), the four windmill hill runs of 3o-4 (`camera-3o5-hill-H.log`).
**Result:** No change in any run: the watermill flight still ends at (188999.0, 229341.5, 10863.4); the three Daventry `follow Connor` calls find the view 0.157, 0.121 and 0.000 rad off, so the switch stays off. No run reached the deep lift (none logged `lifted_along_line` or `deep_line_missed`).
**Still broken / not tested:** **Not tested in play**: the deep lift needs the camera more than the range's maximum under the ground (a big cliff), and no Daventry `follow` finds both conditions; both are unit-tested only.
**Next:** 3o-6, a script's `move` and the trap it takes Connor out of.

## 2026-10-19 M3 step 3o-4: the view line against the ground, exactly

**Changed:** `kq8-core/src/camera.rs`: `ground_line_hit` is now exact, per triangle: the line is cut at every square edge and diagonal and at its end, the first cut on the other side of the drawn ground than the start gives the triangle, the hit is the exact crossing with its plane; a line coming **up** out of the ground counts too. The old stepped test is kept as `floor_line_hit_stepped` for the deep lift only (3o-5 changes that rule); 1 unit test replaces the old one. Viewer `game_camera.rs`: passes the square size. Provenance, `docs/DESIGN.md`.
**Why:** A's answer to Q53 a (`camera.md` "Obstacles" item 1).
**Tested how:** `cargo test --workspace` (458 pass; 1 test replaced), clippy clean. Viewer: Connor running down from the windmill's hill in four directions, `cargo play --world Daventry --at 144500,246500,H --hold 1,0,1,150 --frames 1200 --sound off` with H = 0, 1.5708, 3.1416, 4.7124 (`camera-3o4-hill-H.log`).
**Result:** Unit test: on a 3 × 3 grid with one point raised to 2,000, a level line at 1,000 along a square edge goes in at x = 512.0 (exactly half-way up the slope); a slanting line across edges and diagonals has no ground above it before its hit; a line starting inside the hill is blocked where it comes out; a line with no edge on the way is caught at its end point. Viewer: one hit in the four runs (H = 1.5708, update 78: "the_ground (154683, 246889, 15026)", then the avoidance swing of 0.102 per update as before).
**Still broken / not tested:** Not seen by eye. No holes (none known). The old and the new test were not run side by side on the same spot.
**Next:** 3o-5, the deep lift on the drawn surface and one `follow` finding both conditions.

## 2026-10-19 M3 step 3o-3: the mouse

**Changed:** Viewer `game_camera.rs`: the mouse's movement is gathered as a share of the window (one window width / height = one mouse unit), replacing the hand-set 800 pixels per unit; a mouse movement during a drag is a cause for a camera update, holding the button still is not. `kq8-core/src/camera.rs`: holding the button without a change puts the camera back at the previous update's offset from the aim point (then the leash, height and obstacle rules); `Camera::placed` forgets that offset after `move KQCamera`, a flight or a cut; 1 unit test. Provenance, `docs/DESIGN.md`.
**Why:** `camera.md` "Swinging and zooming" rules 1 and 6 (Q27 a): one window width = 3π/4, one window height = 5,000; each mouse move updates the camera; holding the button keeps the camera's place relative to Connor (the manual says so too).
**Tested how:** `cargo test --workspace` (458 pass, 1 new), clippy clean. Viewer: the walk-zone run of 3o-2 again (`camera-3o3-walkzone.log`): identical to `camera-3o2-walkzone.log`, as it must be without the mouse.
**Result:** As above. On a 1,280-pixel-wide window one pixel sideways swings 3π/4 ÷ 1,280 = 0.0018 rad (before: 0.0029); on a 720-high window one pixel forward/back zooms 6.9 units (before: 6.25).
**Still broken / not tested:** **Not tested in play**: the test options can't drive the mouse, so the swing, the zoom and the held button are unit-tested only. Not built: the recentring at 0.9 / 0.5 (rule 6, not settled).
**Next:** 3o-4, the view line against the ground.

## 2026-10-19 M3 step 3o-2: the tilt and the aim height

**Changed:** `kq8-core/src/camera.rs`: `AIM_HEIGHT` 800 → 1,400 (the note's; the 800 was set by hand because our view looked straight at the aim point); `DEFAULT_TILT` is now the note's −0.2 (Page Up still adds 0.015); the view tilt = the player's tilt + the angle up to the aim point, every update; `centre_line_height` (where the view's centre line passes Connor) and a unit test. `camera_flight.rs`: its aim height and default tilt are the same constants. Viewer `game_camera.rs`: the flights' starting view uses the new default; `camera.log` gets `centre_line_at_connor`. Provenance, `docs/DESIGN.md`.
**Why:** A's answer to Q27 b: at the default the camera looks 0.2 rad below the aim point; and the user's report that the camera points at Connor's hips (the hand-tuned 800).
**Tested how:** `cargo test --workspace` (457 pass, 1 new), clippy clean. Viewer (`--sound off`): inside the Sarawalk walk zone, `cargo play --world Daventry --at 282250,196409,3.1416 --hold 1,0,0,90 --frames 1500` (`camera-3o2-walkzone.log`, range 1,000–2,500 from `walk_la.cs` at update 1); outside walk zones, `--at 277800,198800,3.1416 --hold 1,0,0,90 --frames 1500` (`camera-3o2-walk.log`, range 1,500–5,000); the mill wheel and the watermill as in 3o-1 (`camera-3o2-wheel.log`, `camera-3o2-watermill.log`).
**Result:** Where the view's centre line passes Connor, above his feet (his box is 1,300 tall): standing at 2,499 away **898–900**, walking at 2,499 **893** (1,400 − 2,500 × tan 0.2); 1,310 when an obstacle pulled the camera in to 451. Outside walk zones the camera trails 5,000 away and the line passes lower: 782 at 3,046, **390** at 4,999. The camera is level with the aim point on flat ground, so the view tilt is exactly −0.200. Mill wheel first shot: tilt −0.832 (the note works out about −0.83). Watermill flight unchanged.
**Still broken / not tested:** Not seen by eye (the user's check (c)). At walking distance the line passes 893 above his feet, a little under the 900–1,000 expected; at the 5,000 of the default range (outside walk zones) it passes at his knees. If that looks wrong in play, it is a question for A, not a value to tune.
**Next:** 3o-3, the mouse.

## 2026-10-19 M3 step 3o-1: the view heading (four states)

**Changed:** `kq8-core/src/camera.rs`: the camera keeps its own view heading and tilt and a heading state (idle, catching up, locked, turning back), the "turn back" switch, the mouse offset; every camera update ends by aiming the view by `camera.md` "Turning the view toward Connor" (`HeadingStep` in the report); `Followed` carries Connor's facing (for "switched between forward and backward"); `follow_turn_start` removed; 5 unit tests. Viewer `game_camera.rs`: the view is drawn from the camera's heading and tilt; `follow` only sets the switch (over 0.4 rad off); flights, cuts and `move KQCamera` write the heading; the old stand-ins (`held_heading`, `view_turn`) are gone; `camera.log` gets the state, error, turn and what is left every update; the start-up update first looks straight at Connor. `docs/DESIGN.md`: the plan for 3o (6 steps). `QUESTIONS.md`: Q113. Provenance.
**Why:** A's answer to Q27 c and Q100 a (2026-10-19): the camera never swings behind Connor; only the view's heading turns, by four states. First step of closing M3.
**Tested how:** `cargo test --workspace` (456 pass, 5 new), `cargo clippy --workspace --all-targets` clean. Viewer (`--sound off`): Connor walking straight across the view, `cargo play --world Daventry --at 278500,197541,3.1416 --hold 1,0,0,90 --frames 1500` (`camera-3o1-side.log`); the mill wheel `--at 186300,226000,3.1416 --hold 1,0,0 --frames 1500` (`camera-3o1-wheel.log`); the watermill `--at 192408,227039,0 --frames 3000` (`camera-3o1-watermill.log`).
**Result:** Side walk: idle while the error grows to 0.236 (update 18), "idle->catching_up error +0.258" at update 19 (14.8°), then each update a turn of 1.25 × the drift (0.026 down to 0.008), what is left falling from 0.233 to 0.012; locked at update 96 and exactly on him while he moves. Mill wheel: the first shot's update (18) found the view still catching up from the walk in, 0.259 off: turned 0.318, ending 0.060 past him (asked as Q113 c). Watermill: the flight as before (ends at 188999.0, 229341.5, 10863.4, heading 2.153, tilt +0.016); after cue 6's `follow Connor` the view stays 0.121 rad off, idle (before: snapped onto him).
**Still broken / not tested:** Not seen by eye. Stand-ins (Q113): rule 1 only from idle, the exact aim before unlocking, the state through flights and cuts. The tilt is still our old stand-in (next step). Locking needs the error to fall under 0.02, which only happens while he keeps drifting sideways: walking straight away from a camera that trails behind him, the view stays at whatever angle it has (the rule as written).
**Next:** 3o-2, the tilt and the aim height.

## 2026-10-19 A: the last open questions blocking M3 (docs only)

**Changed:** `docs/behaviour/camera.md`: "Swinging back behind Connor" replaced by "Turning the view toward Connor" (the old text was wrong: the camera's position never swings behind him; only the view's heading turns, in four states with traced numbers); the mouse value (a fraction of the window), each mouse move updating the camera at once, holding the button keeping the camera fixed relative to Connor; the tilt (the view looks 0.2 rad below the aim point by default); how the camera starts in Daventry and when `walk.cs`/`run.cs` run (from the data); the exact ground test of the view line (any change of side, holes passed); the BSP test; the 3D avoidance direction; the deep lift; one `follow` finding both conditions; open questions 1 and 7. `traps.md` rule 5: a script's `move` / `loadKQ` and traps. `anim-scripts.md` rule 24 and open question 6 (closed). `emitters.md` catcher rule 5 (closed). `QUESTIONS.md`: Q27, Q53, Q100, Q101, Q102 answered, Q48 closed. Scratch addresses in `re/` (not committed).
**Why:** The user asked for every open question that blocks M3 (Daventry only) to be answered so M3 can be closed. Picked from `QUESTIONS.md`, `STATUS.md` and the README's "What's left of M3": Q27, Q48, Q53, Q100, Q101, Q102 and `anim-scripts.md` open question 6. Not taken: M1/M2 leftovers (Q14b teleport pad, not in Daventry; Q24, Q29e, Q30b, Q41, Q42, Q112) and M4 (Q106–Q110).
**Tested how:** Traced in `Mask.exe` (Ghidra) and checked against the data files (`cam3rd.kq`, `world.cs`, `stngrl.cs`, `walk*.cs`/`run*.cs` and the trap `.kq` files that name them, `wheeltp1/2.cs`, `permwrld.cs` load order). Nothing run in the game; the new camera rules are **not tested** in play.
**Result:** Answers that change what B built: (1) the view's heading rules (B has no heading rule yet besides the flights' and `follow`'s); (2) the default tilt looks 0.2 rad below the aim point (B's stand-in looks at it); (3) holding the right mouse button keeps the camera's offset to Connor; (4) the ground test of the view line counts a line coming **up** out of the ground (B's stand-in only counts going down); (5) one `follow` finding both conditions leaves the "turn back" switch **off** (B: on; never happens in Daventry); (6) the trap a script `move` takes Connor out of gets `Off` at the move, not at his next move (no visible difference in Daventry: the mill wheel traps do nothing on `Off`). Confirmed as built: Q53 b, c; Q101 (same update); Q102 a (no sweep, entered at the next move).
**Still broken / not tested:** Q27 a only partly: the step from the window fraction to the camera (signs, and whether the value is centred) was not found; the user's checked directions stand. Q48: where objects join the tick list was not found (one tick of a falling leaf; keep B's order). Not checked in the original by the user: the heading rules ("what this looks like in play" in the note gives a check).
**Next:** B: build (1)–(6) above, then the user's check of the camera against the original (walk off to one side: the view holds until he is about 14° off, then catches up and locks on him). Then M3 has no open study questions left.

## 2026-10-19 M3 is Daventry only (docs only)

**Changed:** `docs/DESIGN.md`: M3 now says its scope is Daventry only; things of later worlds (Gnome's rolling rocks hurting Connor, Gnome's pushable boulder) are listed under M6 instead, with damaging water and the other pushable classes. `STATUS.md`: those items taken out of "Later for A" and "What doesn't work yet" and pointed to M6. `docs/FEATURES.md`: the rolling rock line says it is Gnome, not M3. (The user had already taken them out of the README's "Next".)
**Why:** The user: the rolling rocks and the boulder were noted while testing a world past the middle of the game; they are irrelevant to Daventry, and sessions kept treating them as part of M3, so M3 could never be finished.
**Tested how:** Nothing to run (documents only). Searched all docs for "rolling rock", "boulder", `KQBoulder`, Q55: the MODLOG history and the behaviour notes describe how those objects work and stay as they are.
**Result:** —
**Still broken / not tested:** —
**Next:** —

## 2026-10-18 `cargo play --world` with no name lists the worlds

**Changed:** Viewer `main.rs`: `--world` with nothing after it, or followed by another option, prints the 12 worlds (from `kq8_data::game_dir::WORLDS`, Daventry marked as the default) and exits without opening the window; the `--help` text says so. `README.md` ("Try it") and `docs/FEATURES.md` mention it.
**Why:** The user asked for it.
**Tested how:** `cargo play --world` and `cargo play --world --frames 10` (the list, exit code 0); `cargo play --world swamp --frames 60 --sound off` still opens the Swamp (`viewer.log`: "world: Swamp"); clippy clean.
**Result:** As above.
**Still broken / not tested:** A misspelt world name still fails later with the loader's own error (not changed).
**Next:** —

## 2026-10-18 `cargo play --help`

**Changed:** Viewer `main.rs`: `--help` / `-h` prints every launch option, grouped as in `docs/FEATURES.md` ("Showing things", "Test options"), and exits without opening the window; a wrong option now prints its error and points to `--help` (instead of the two long usage lines, which had fallen behind: no `--aim`, `--special`). `README.md` ("Try it"): mentions `--help`; the link to the options in `docs/FEATURES.md` fixed (its anchor was misspelled). `docs/FEATURES.md`: mentions `--help`.
**Why:** The user asked for a command listing the launch options.
**Tested how:** `cargo play --help` (the list printed, exit code 0); `cargo play --bogus` (the error and the pointer to `--help`, exit code 1); clippy clean.
**Result:** As above.
**Still broken / not tested:** —
**Next:** Keep the list in `HELP` up to date when an option is added.

## 2026-10-18 README "Current state" is now a short roadmap (docs only)

**Changed:** `README.md`: the "Current state" section (the "what works" / "what doesn't work yet" lists, mostly repeated in `docs/FEATURES.md`) replaced by a short roadmap: the milestones M0–M7 with their status, what is done, next and later. The text is the user's edited version of the `ROADMAP.md` I wrote first; that commit was reverted (the user changed their mind: no separate file).
**Why:** The user asked for a short overview of how far along the rewrite is, in the README.
**Tested how:** Nothing to run (documents only).
**Result:** —
**Still broken / not tested:** —
**Next:** Update the table and lists when a milestone changes status.

## 2026-10-18 End of B's session (docs only)

**Changed:** `STATUS.md` ("Where we are": 4c-11, the back flip at a wall; the older 4a-7 to 4a-9 details folded into one line; next steps; Q111, Q112 in "Open questions"; test counts and the `--corners` option), `docs/FEATURES.md` (the back flip at a wall, checked by the user).
**Why:** `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** Nothing to run (documents only).
**Result:** —
**Still broken / not tested:** Stand-ins of Q108–Q112 stay until A traces them.
**Next:** A: conversations (then Q108–Q112). B: the main menu with the game paused.

## 2026-10-18 Fix: the back flip when backed against a wall

**Changed:** `kq8-core/src/connor.rs`: `jump_kind` gives `backJump` when he stands (`stop`) with the back wall block on, that is after a wall stopped him stepping back and back is still held (the block goes when back is let go); unit test extended. `QUESTIONS.md`: Q112. Provenance.
**Why:** The user's check in the original: backed against a wall with back held, Jump does the back flip; ours did the standing jump, because `connor-physics.md` "Jumping" rule 2 picks the jump from his movement only, and the wall stop makes it `stop`.
**Tested how:** `cargo test --workspace` (451 pass, 1 extended), clippy clean. Viewer: `cargo play --world Daventry --at 205900,267595,1.5708 --hold -1,0,0 --jump 25 --frames 900 --sound off` (backs west into the Pub; `connor-backflip-wall.log`).
**Result:** "head-on into a wall: movement back -> stop; blocked: back" at update 7; "jump: backJump from stop" at 26; the flip rises to z 10,316 gliding along the wall ("backJump hit Pub … glancing: gravity stays off, backJump plays on"), lands with `backJumpEnd` at 46, then stands with back still held.
**Dead ends recorded:** Two test spots were wrong: from (186300, 226000) stepping back he reached the mill wheel's trap first; at (237907, 187000) there is no wall (I misread the note's measured numbers). The Pub spot came from `connor-3m2-barrel-pub.log`.
**Still broken / not tested:** Not seen by eye. The exact rule is a stand-in (Q112): we test the back block, the original may test the key or the asked movement; it only differs in cases not seen yet.
**Next:** The user's check.

## 2026-10-18 M4 step 4c-11: corner panels (an option, not in the original)

**Changed:** Viewer `gui_view.rs`: with the corner panels on, the bottom bar's left panel stands at the window's left edge and its right panel and the spinning mask at the window's right edge; the slots, meters, top bar and caption stay on the centred 4:3 screen; clicks use the moved places. F6 switches it (logged in `gui.log`); `main.rs`: `--corners on|off` (default off). `docs/DESIGN.md` (4c-11), provenance, `docs/FEATURES.md`, `README.md` (F6).
**Why:** The user asked to see their third idea, as an option to switch on and off.
**Tested how:** `cargo test --workspace` (451 pass), clippy clean. Viewer: `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --corners on --frames 400 --sound off` (`gui-4c11-corners.log`).
**Result:** Window 1280 × 720: "IDCTL_INVBARL at left 0.00 … right 288.00", "IDCTL_INVBARR at left 992.00 … right 1280.00", slots and meters at 448–832 as before; the mask "at [876, 465, 933, 522]" in screen units, that is 1211–1280 in the window, on the bar at the right edge.
**Still broken / not tested:** Not seen by eye. F6 and mouse clicks on the moved panels are not tested by a run (they need the real keyboard and mouse).
**Next:** The user's look at it.

## 2026-10-18 M4 step 4c: checked by the user (docs only)

**Changed:** `docs/FEATURES.md` (the interface entries: checked by the user), `STATUS.md` (4c-6 to 4c-10, checked, test count, next steps).
**Why:** The user's check after 4c-10: "everything seems correct".
**Tested how:** Nothing to run (documents only).
**Result:** —
**Still broken / not tested:** The stand-ins of Q108–Q111 stay until A traces them.
**Next:** The corner panels pushed to the window's corners, as an option (the user's third idea).

## 2026-10-18 M4 step 4c-10: the meters' gap for good; the mask not on the meters alone

**Changed:** Viewer `gui_view.rs`, `talk_view.rs`: the panels' and the caption's pictures fill their box exactly (`NodeImageMode::Stretch`). `main.rs`, `terrain_view.rs`: the engine doesn't smooth the panels' edges (`UiAntiAlias::Off`). `gui_view.rs`: the spinning mask stands on the bottom bar only, not on the meters when they pop up alone. Provenance, `docs/DESIGN.md`.
**Why:** The user's fourth check: the gap was smaller but a few pixels of the world still showed beside the meters, flickering while the bar slides; the mask rose with the meters popping up alone, which the original's doesn't (the caption does).
**Tested how:** Logs of the user's own run (window 1280 × 720) showed the engine's boxes exactly edge to edge and no see-through pixels in our pictures, so I took a screenshot in a run (temporary code, removed) and read the pixels at the seams as numbers, with and without the edge smoothing, then with the fix. `cargo test --workspace` (451 pass), clippy clean. Bricks with the bar away `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --key space,3 --frames 2500 --sound off` (`gui-4c10-bricks-bar-away.log`).
**Result:** Before: at row 700 the meters' stone started at x 452 (136, 136, 128, 120, as their picture's first columns) with world colours (255, 198, 54…) at 448–451, and the same 4 pixels at 828–831: Bevy's default image mode keeps the picture's proportions, so 320 × 17 in 384 × 20 was drawn 376 wide and centred (and 395 wide when the sliding box was 21 high: the flicker). The edge smoothing only changed row 699. After: grey stone from 448 to 831 at every row, right next to the side panels. The mask: "at [743, 543, 800, 600]" while the meters pop up alone at 1.287 s.
**Dead end recorded:** 4c-9's rounding fix was right but only part of it; the edge smoothing (`UiAntiAlias`) was a wrong first guess (switched off anyway).
**Still broken / not tested:** Not seen by eye.
**Next:** The user's check of 4c-10.

## 2026-10-18 M4 step 4c-9: the gap beside the meters; the spinning mask on the bottom bar

**Changed:** `kq8-core/src/gui_draw.rs`: a panel is drawn where it stands (`draw` takes its layout place; `screen_size`): each edge is rounded at its screen place, so the picture has exactly the pixels of the place the engine gives it (unit test). Viewer `gui_view.rs`: passes each panel's place; the spinning mask stands on the top of the highest bottom panel (worked out before the panels are placed, as the caption's), rising and falling with the bar; new `gui_layout_log` writes where the engine put each bottom panel and its picture's size. Q111 closed by the user's checks. Provenance, `docs/FEATURES.md`, `docs/DESIGN.md`.
**Why:** The user's third check: a small gap the height of the meters between them and the side panels, flickering every frame while the bar slides; and the spinning mask should follow the bottom bar like the caption.
**Tested how:** `cargo test --workspace` (451 pass, 1 extended), clippy clean. Viewer: `cargo play --at 245000,255000,0 --key space,10 --key space,30 --frames 400 --sound off` (before the fix) and the bricks `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 2500 --sound off` (`gui-4c9-bricks.log`).
**Result:** Before: "engine: IDCTL_METERS at left 448.00 top 700.00 right 832.00 bottom 720.00 (… unrounded 384.00x20.40); picture 320x18", with the place going 20, 21, 20… pixels while sliding: 18 rows of picture for 17 rows of screen (14 × 1.25 = 17.5 rounded up, but its edges are 582.5 → 583 and 600). The left and right edges were exact. After: "picture 320x17". The mask: "[743, 465, 800, 522]" on the bar (its top at 520), sliding down with it to "[743, 543, 800, 600]".
**Still broken / not tested:** Not seen by eye. At window sizes that aren't a whole multiple of 800 × 600 the pixels are still unevenly stretched (some rows and columns doubled), which can shimmer slightly while a panel slides; the gap itself should be gone. The rope hook icon beside the mask: later, with the rope (M6).
**Next:** The user's check of 4c-9.

## 2026-10-18 M4 step 4c-8: the user's second check (Space's sounds, the whole mask, black behind the bars)

**Changed:** `kq8-core/src/panels.rs`: Space hiding the bottom bar plays `intrup.aud`, showing it `intrdown.aud` (swapped). `gui_draw.rs`: a one-colour fill piece (unit test). Viewer `gui_view.rs`: black behind the health and experience bars; the spinning mask drawn whole at its own 57 × 57 (it was cut to its 48 × 48 layout box) in the screen's bottom right corner; `gui.log` now lists each picture's size when loaded and any see-through pixels of a drawn panel. `QUESTIONS.md` Q111 (a, b, d answered by the user; c open). Provenance, `docs/FEATURES.md`, `docs/DESIGN.md`.
**Why:** The user's second check of 4c: the bottom bar's sounds were inverted; the mask was cut off on its right even when centred; the experience bar should have a black background (the world showed through its empty part).
**Tested how:** `cargo test --workspace` (451 pass, 1 extended), clippy clean. Viewer: the bricks `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --key space,3 --key space,9 --frames 2500 --sound off` (`gui-4c8-bricks.log`).
**Result:** "Space: intrup.aud" at 0.000 s (hiding), "Space: intrdown.aud" at 1.101 s (showing); "picture mskicn.pba: 16 frame(s), 57x57"; "spinning mask at [743, 543, 800, 600] of 800x600"; before the fix the log showed "IDCTL_METERS drawn 320x18: 1720 see-through pixels, columns 18-302, rows 5-12" (the empty bars), after it no see-through pixels in the meters.
**Dead end recorded:** Looking for the gap next to the meters: the layout has no gap (the meters run from 192 to 448, the side panels end and start there), the side panels' pictures are full 64 × 64 opaque stone, and the log shows no see-through pixels in any bottom panel. Not found yet (Q111 c).
**Still broken / not tested:** Not seen by eye. The gap between the meters' end pieces and the side panels (Q111 c).
**Next:** The user's description of the gap; A: Q108–Q111.

## 2026-10-18 M4 step 4c-7: Space's sounds, the spinning mask in the corner

**Changed:** `kq8-core/src/panels.rs`: Space plays `intrup.aud` when the bottom bar starts to show and `intrdown.aud` when it starts to hide (handsOff and the scripts still play nothing); tests updated. Viewer `gui_view.rs`: the status icons are drawn at their own size (not × 1.25) with the spinning mask's bottom right corner at the screen's bottom right corner; the place is logged. `QUESTIONS.md`: Q111. Provenance, `docs/FEATURES.md`, `docs/DESIGN.md`.
**Why:** The user's check of 4c in the original: Space makes the top bar's sound (cutscenes don't); the spinning mask is in the bottom right corner and smaller.
**Tested how:** `cargo test --workspace` (451 pass, 2 tests changed), clippy clean. Viewer: the bricks `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --key space,3 --key space,9 --frames 2500 --sound off` (`gui-4c7-bricks.log`, `sound-4c7-bricks.log`).
**Result:** "Space: intrdown.aud" at 0.000 s, "Space: intrup.aud" at 1.106 s, both started in `sound.log`; handsOff at 1.370 s slides the bar away with no sound; "status icons at [752, 552, 1088, 600] of 800x600" (the mask is the first 48 × 48 of that, in the corner).
**Not changed (asked as Q111):** the meter bars' width (ours follow the layout exactly: 86 of 256 pixels each, with 14-pixel end pieces) and the empty green bar at 0 experience (0 % wide by rule 22; nothing is drawn behind an empty bar, so the world shows through it).
**Still broken / not tested:** Not seen by eye. Which sound goes with which direction, and the mask's exact place and size, are stand-ins (Q111).
**Next:** The user's check of 4c-6 and 4c-7; A: Q108–Q111.

## 2026-10-18 M4 step 4c-6: the panels on a 4:3 screen in the middle of the window

**Changed:** `kq8-core/src/panels.rs`: `screen_frame`, the largest 4:3 rectangle centred in a window (unit test). Viewer: a `ScreenFrame` resource fitted each frame (`fit_screen`, logged in `viewer.log` when it changes); `gui_view.rs` places the panels on it in window pixels, maps the pointer to the game's screen through it, computes the click areas from it and sizes the pointer picture from its height; `talk_view.rs` places the caption on it. `docs/DESIGN.md` (4c-6, 4c-7 planned), provenance, `docs/FEATURES.md`.
**Why:** The user's check of 4c: the original runs at a fixed 4:3 and its panels are laid out for it; in a 16:9 window ours were stretched sideways. The user chose the "virtual 4:3 screen in the middle" (their second idea; the third, the corner panels in the window's corners, maybe later).
**Tested how:** `cargo test --workspace` (451 pass, 1 new), clippy clean. Viewer: `cargo play --at 245000,255000,0 --click alcmsign,10 --frames 200 --sound off`.
**Result:** "screen: window 1280x720: the game's 4:3 screen at left 160.0 top 0.0, 960.0x720.0"; the caption still at "left 168.75 top 486.00" of the 800 × 600 screen (that is 362.5 pixels from the window's left).
**Still broken / not tested:** Not seen by eye. Mouse clicks on the bars and the top bar following the mouse with the new mapping are not tested by a run (they need the real mouse).
**Next:** 4c-7, the user's corrections (Space's sounds, the spinning mask's place).

## 2026-10-18 M4 step 4c: end-of-session documents (docs only)

**Changed:** `STATUS.md` ("Where we are": 4c-1 to 4c-5; older 4a entries shortened; code layout, logs, test count; the 4c stand-ins and what is not built; Q105 moved to answered and built; Q108–Q110 listed; next steps).
**Why:** `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** Nothing to run (documents only).
**Result:** —
**Still broken / not tested:** 4c is not seen by eye yet.
**Next:** The user's checks of 4c by eye; A: Q108–Q110, then conversations.

## 2026-10-18 M4 step 4c-5: A's answers to Q105 (the turn after a click)

**Changed:** Viewer `door_view.rs`: Connor's state is recorded at each click; when the click's turn is applied, that state decides it, unless a state request of the script (a `setState` or a scripted walk that was not refused) was applied before it, which then decides; the log says which. `kq8-core` `active.rs`: buildings carry `doNotPointAt` (`load.rs` passes the value it already read), so a building with a script and `doNotPointAt=1` doesn't turn him; unit test extended. Provenance, `QUESTIONS.md` (Q105 built), `docs/DESIGN.md`.
**Why:** `clicking.md` rules 34–36 (A's answers to Q105): (a) our stand-in was right (no change); (b) the original looks at his state right after the script, before any update of his; (c) buildings read `doNotPointAt`.
**Tested how:** `cargo test --workspace` (450 pass, 1 extended), clippy clean. Viewer: the fish `cargo play --world Daventry --at 208502,209600,1.5708 --click fishies.01,10 --frames 600 --sound off` (`connor-4c5-fish.log`) and the tombstone `--at 204100,176500,0 --click tomb2.02,20 --frames 900`.
**Result:** Fish: "Connor's state after the script Wait (his state at the click, rule 35): turn to fishies.01" although its script deletes it (rule 34). Tombstone: one turn, as before.
**Still broken / not tested:** The "state the script asked for" path is not tested by a run: no Daventry click script sets Connor's state (the coffin lid does it on `Push`, the bell's line is commented out). No Daventry building sets `doNotPointAt` (Gnome's do).
**Next:** The user's checks by eye of 4c-1 to 4c-4; A: Q108–Q110.

## 2026-10-18 M4 step 4c-4: the item in hand

**Changed:** `kq8-core/src/connor_inventory.rs`: holding (a quest item, the coins, the rope or the rocks becomes the hand; the same again puts it back; the scroll's refusal remark at most every 3 s; the rope's remark), the hand after a world click (emptied when accepted, kept when refused or too far, coins kept while `goldCoinPersists 1`), the hand emptied when its item is gone. `active.rs`: the hand, `goldCoinPersists`. Viewer: `door_view.rs` world clicks use the hand (the old fake `--item` holder is gone: `--item TYPE` now gives a real `TestItem` and holds it); `gui_view.rs` leaves the held item out of the quest strip (the arrows still count it; a click on the empty place puts it back) and makes the mouse pointer the game's picture (`curs04.pba`, or the held type's cursor). `QUESTIONS.md`: Q110. Provenance, `docs/FEATURES.md`, `README.md`, `docs/DESIGN.md`.
**Why:** Step 4 of the interface plan (`inventory.md` rules 7, 9, 12; `clicking.md` rules 12–18).
**Tested how:** `cargo test --workspace` (450 pass, 1 new, 1 extended), clippy clean. Viewer runs (`--sound off`): coins `--at 228081,201000,1.5708 --click NoName9,10 --key left:IDCTL_INVBAR_GOLD,25 --click NoName10,35 --key left:IDCTL_INVBAR_GOLD,60 --frames 1500`; star globe `--at 144046,208300,1.5708 --lift 6300 --item INVITEM_SpellPage --click starball.01,40 --frames 2500`; scroll `--at 247573,258000,1.5708 --click NoName5,10 --key left:IDCTL_INVBAR_QUEST2,60 --key left:IDCTL_INVBAR_QUEST2,70`; rope `--item INVITEM_RopeAndHook --key left:IDCTL_INVBAR_ROPE,20 --key left:IDCTL_INVBAR_ROPE,40`.
**Result:** Coins (`inventory.md` check 2): held with the pointer `goldcurs.pba`; a click on the other coins: "This will not work", "it stays in hand"; the gold slot again: "put back, the play pointer again". Star globe: the page is accepted, its script deletes it, "the hand is emptied, the play pointer back". Scroll: Connor reads "PERMANENT SPELL OF MIGHT – NEED: …"; a second click within 3 s: nothing. Rope: held (`curs06.pba`), "I am not near anything to climb."
**Dead end recorded:** I first read the note's `0.MSG (0, 92, 14, 0)` as noun, verb, case, sequence (as the descriptions are written): `talk.log` said "line 0 0 92 14 0 is not in its message file". Read as file, noun, verb, case (sequence 1) both remarks name the right lines (Q110 a asks A to confirm).
**Still broken / not tested:** Not seen by eye (the pointer pictures, their click point). The quest strip closing the gap is not tested by a run (it needs 8 or more quest items). Stand-ins (Q110): the pointer's click point at its top-left, the rope held and the remark both, the gold button unchanged while the coins are held. Escape still quits: the original opens the main menu, which pauses the game (a later menu step must stop the updates and the panels' timers). Rope places and throwing: M6.
**Next:** 4c-5, A's answers to Q105.

## 2026-10-18 M4 step 4c-3: the inventory bars

**Changed:** New `kq8-core/src/connor_inventory.rs`: an item going into Connor's inventory (stacking into one entry; equipment pushing out what shares its place, placeholders deleted, others dropped where the new one lay, then put on), applying and un-applying (`userId`, `Dur` timers, stats through one stats event), a click on a bar slot (held / menu / applied with its use sound / ignored in handsOff), the description on a right click (`0.MSG` voice, plain). `active.rs`: pickups, chests and `give` all go through it; the potions' timers each update. `item.rs`: the order items went in. `panels.rs`: `moving`. Viewer `gui_view.rs`: the top bar (mask button, quest strip with arrows, gold and count) following the pointer and popping down for a new quest item; the eight slots' pictures and counts; rope and rocks; blinking; clicks on the bars and keys 1–8; `--key left:NAME` / `right:NAME`. `QUESTIONS.md`: Q109. Provenance, `docs/FEATURES.md`, `README.md` (keys), `docs/DESIGN.md`.
**Why:** Step 3 of the interface plan (`interface.md` rules 12–13, 38–45; `inventory.md` rules 1–11).
**Tested how:** `cargo test --workspace` (449 pass, 4 new), clippy clean. Viewer runs (`--frames 900`–`1500 --sound off`): mushroom basket `--at 233848,203000,1.5708 --click basket1,10 --key 1,40 --key 1,60 --key space,80 --key 1,100`; coins `--at 228081,201000,1.5708 --click NoName9,10 --click NoName10,30` (`gui-4c3-coins.log`); dagger `--at 260784,228000,1.5708 --click NoName1,10` (`gui-4c3-dagger.log`); scroll `--at 247573,258000,1.5708 --click NoName5,10` (`gui-4c3-scroll.log`); bar clicks `--key left:IDCTL_INVBAR_GOLD,30 --key left:IDCTL_INVBAR_MASK,40 --key right:IDCTL_HEALTHBAR_3,50 --key left:IDCTL_INVBAR_SHORT,60 --key left:IDCTL_INVBAR_QUESTR,70` and `right:` on the gold and mask.
**Result:** Basket: slot 1 shows 5 and blinks; key 1 → 4, 3, 2 with `healing.aud` (health already full, capped at 10); with the bar away the meters pop up. Coins: "NoName10 x5 joins NoName9: INVITEM_SilverCoins 7 -> 12". Dagger: "placeholder Fist1 (INVITEM_Fists) deleted", hand 10 → 15, its number flashes, slot picture `2sord1.pbm`. Scroll: strip ["Mask1a", "NoName5"], the top bar down at 1.178 s with `intrup.aud`, up at 2.684 s with `intrdown.aud`. Right click on the gold: "0.MSG 0 47 0 0 1: GAME/ENGLISH/A0001b00.001" played plain; mask button left: "opens the main menu; no menu yet"; empty slot: nothing.
**Still broken / not tested:** Not seen by eye. The top bar following the mouse is not tested by a run (it needs the real pointer). Quest items, gold, rope and rocks are not held yet (4c-4). Stand-ins (Q109): counts of 0 not written, the rocks picture, the blink's order, potions on game time, a `give`'s pushed-out piece at Connor's feet. Potions' real effects and their timer icons: M5. Weapon readying: M5.
**Next:** 4c-4, the item in hand.

## 2026-10-18 M4 step 4c-2: the always-visible bottom bar

**Changed:** New core modules `panels.rs` (which panels are shown, sliding in 0.2 s of real time, Space, handsOff, conversations, the meters' 1.5 s pop-up, `getGuiStatus`), `meters.rs` (health and experience percents, bar widths, the values sent after each stats event, which numbers changed), `gui_draw.rs` (a panel drawn from its layout block: pictures, numbers with their justification, bars cut to a width), `inventory.rs` (which picture the armour, hand and ranged slots show). `active.rs`: a counter of Connor's stats events; the script commands `setLowerGui`, `setUpperGui`, `setMapGui`, `getGuiStatus`. `item.rs`: items with their own names. `kq8-data`: `items.rs` reads the stat, `Comb`, picture, message, mode, `Dur`, use-sound and cursor columns; `font.rs` reads whole `.pba` files. Viewer: new `gui_view.rs` (draws bottom left, the eight slots, the meters, bottom right and the status icons; `gui.log`); `load.rs` / `door_view.rs` give Connor his start inventory from `Connor.kq`; the caption sits on the highest bottom panel as it slides; a click on a shown panel is not a world click; `--key NAME,N` test option. Docs: provenance, `docs/FEATURES.md`, `README.md` (Space), `docs/DESIGN.md`.
**Why:** Step 2 of the interface plan (`interface.md` rules 7–36; `inventory.md` rule 4).
**Tested how:** `cargo test --workspace` (445 pass, 12 new), clippy clean. Viewer: `cargo play --world Daventry --at 245000,255000,0 --key space,30 --key space,60 --click alcmsign,90 --frames 1500 --sound off`; the bricks `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 7000 --sound off`, once with `--key space,10` (`gui-4c2-bricks-bar-away.log`) and once without (`gui-4c2-bricks.log`).
**Result:** Start: "Connor [10, 10, 0, 0, 0] level 1: sent armour 0, hand 10, ranged 0, health 100 % (bar 107 of 107 px), experience 0 %, level 1"; slot pictures `2armr0`, `2sord0`, `2rng0` from the placeholders. Space: slides out at 2.501 s, fully hidden at 2.707 s; back the same way. The sign's caption sits on the bar: "top 486.00 bottom 520.00". Bricks: "level 2: sent armour 1, hand 15, ranged 2, health 100 %, experience 200 % (bar 108 of 108 px)", as `interface.md` check 2 predicts; the three numbers flash. With the bar away the meters pop up alone at 1.302 s and leave at 2.807 s. With the bar up, handsOff slides it away and brings it back at handsOff 0 (8.633 s); with it put away first, it stays away.
**Still broken / not tested:** Not seen by eye (the pictures, their colours, the numbers' places). Stand-ins (Q108): the 800 × 600 scaling of pictures, tiling, the 15 frames per second of the flashes and the spinning mask. Not built: the eight slots' contents, the top bar, the map, the help popup, the red boosted numbers (M5), the level-up flame (not traced); conversations don't exist yet, so rule 16 is only unit-tested; no game pause yet, so the panel timers never pause.
**Next:** 4c-3, the inventory bars.

## 2026-10-18 M4 step 4c-1: the caption strip as wide as the caption box, slightly left of centre

**Changed:** `kq8-core/src/caption.rs`: rows are cut letter by letter at the caption box's width and break at the last space (`interface.md` rule 47); the stone covers the whole box (rule 50); the box is rows × 15 + 4 high, grows upwards, and is placed on the highest bottom panel, centred between the map's right edge and the screen's right edge (rules 48–51). The stand-ins `WRAP_WIDTH` and `STRIP_MARGIN` are gone. Viewer `talk_view.rs`: reads `set800mode` from `Options.cs` (screen 800 × 600, layout × 1.25, box 400 wide), draws the caption as a box-sized picture at its place. `docs/DESIGN.md`: the plan for 4c (the interface and the inventory, five steps). `QUESTIONS.md`: Q108. Provenance, `docs/FEATURES.md`.
**Why:** The user's check in the original: a short line's strip is as wide as a long one's, and with the map closed the strip sits slightly left of the middle (Q107, `interface.md` rules 46–52).
**Tested how:** `cargo test --workspace` (433 pass; the 3 caption tests rewritten), clippy clean. Viewer: `cargo play --world Daventry --at 245000,255000,0 --click alcmsign,30 --frames 1500 --sound off` (`viewer-4c1-sign.log`).
**Result:** "caption picture shown: 2 row(s) [" The alchemist's shop. There could be something", "useful inside."], widths [371.0, 104.0] px; box 400x34 (the stone covers it all), left 168.75 top 566.00 bottom 600.00 of 800x600; centre 368.75". The break is where the user saw it (the leading space is kept, `talking.md` rule 3).
**Still broken / not tested:** Not seen by eye. The 800 × 600 screen and how pictures scale with it are stand-ins (Q108). There is no bottom bar yet, so the caption sits at the very bottom; no map, so it is always "map closed".
**Next:** 4c-2, the always-visible bottom bar.

## 2026-10-18 Study (A): the bottom bar after a cutscene, checked (docs only)

**Changed:** `interface.md` check 7.
**Why:** The user's check in the original.
**Tested how:** By the user in the original, by eye.
**Result:** A bottom bar that was showing comes back when the cutscene ends; one put away before stays away (as `interface.md` rules 15–16 say).
**Still broken / not tested:** Quest strip with 8+ items (from memory only).
**Next:** B: caption strip back to the box width; the always-visible panel.

## 2026-10-18 Study (A): more checks by the user (docs only)

**Changed:** `interface.md` rule 14, check 7, open question 2 (closed); `inventory.md` check 5.
**Why:** The user's further checks in the original.
**Tested how:** By the user in the original, by eye; check 5 from memory.
**Result:** The game stops running while the main menu (Escape) is open. A cutscene puts the bottom bar away; the spinning mask shows only in scenes that take control away (`handsOff`), not in the wizard's walk-away conversations, as rules 15–16 say. Holding an item closes the gap in the quest strip (from memory).
**Still broken / not tested:** The bar coming back after a cutscene, and staying away when put away first, not checked. Quest strip with 8+ items not checked.
**Next:** B: caption strip back to the box width; the always-visible panel.

## 2026-10-18 Study (A): the user's checks of the interface and inventory notes (docs only)

**Changed:** `interface.md` rules 14, 50, 51, "How to check", open question 2; `inventory.md` rule 12, "How to check", open question 1; `QUESTIONS.md` Q107; `STATUS.md`.
**Why:** The user checked the notes' predictions in the original.
**Tested how:** By the user in the original, by eye.
**Result:** All five checks as the notes say: start values and Space (interface check 3), the meters popping up alone (4), the top bar and new-item pop (5), the short caption's strip as wide as a long one and slightly left of centre with the map closed (6), the old weapon/armour dropping and a refused held item staying on the pointer (inventory check 4). Also: the bars don't interrupt play, only Escape (main menu) leaves the game screen; no item is ever lost from the inventory except by being used.
**Still broken / not tested:** The handsOff check (interface 7) and the quest-strip gap (inventory 5) are not done. B's 4b-3 strip, fitted to the text, is now known to be wrong (the strip is the box's width). Whether the world stops behind the main menu is not traced.
**Next:** B: caption strip back to the box width; then the always-visible panel.

## 2026-10-18 Study (A): the on-screen interface and Connor's inventory (docs only)

**Changed:** New `docs/behaviour/interface.md` (which files draw the panels; the panel groups and how they slide; Space / Tab / 1–8 / Escape / F1; the top bar following the mouse; when panels hide for `handsOff`, conversations, captions; first person hides nothing; the meters' formulas; Connor's five stats named; the slots, gold, blinking; the caption strip's wrap, height and place). New `docs/behaviour/inventory.md` (item categories and places; stacking; equipment swaps that drop the old piece; picking an item from the bars; what is held as the pointer and what is used at once; the held item after a world click; applying and un-applying items, `userId`; the wizard's items). `clicking.md` rules 34–36 (Q105). Pointers in `talking.md` (open question 4 answered), `items.md` (open question 1 answered), `anim-scripts.md` rule 19 (stat names). `QUESTIONS.md`: Q105 answered, Q106 partly, Q107 mostly. Scratch notes in `re/` (not committed).
**Why:** The user's request: B is to build the always-visible panel, then the inventory bars and using an item in the world.
**Tested how:** Traced in `Mask.exe` with Ghidra; data read from `Gplayscr.kq`, `Gplayscr.gui` (via B's `gui-verify` log), `movekeys.kq`, `Invinfo.itl`, `Connor.kq`, `wizard.cs`; the printed manual `MASK/Manual.pdf` pages 15–27 read for the player's view. Nothing was run in the game.
**Result:** Examples: health bar width = 86 × (health × 100 / (10 × level)) / 100 pixels; panels slide in 0.2 s; the meters pop up for 1.5 s on a change while the bar is away; the top bar comes down at pointer y 0 and goes up below y 60; a new weapon or armour piece pushes out the old one, which drops where the new one lay; accepted held items leave the hand, refused ones stay. The wizard's items: a plain held-item click delivers them, but his replies and the "ready" state need `KQConner::conversation` (not studied).
**Still broken / not tested:** Not checked in the original. Inferred, not traced: the 400-pixel caption width at 800 × 600, the caption's slightly-left-of-centre place with the map closed, the stone strip covering the whole caption box (the user saw it close to the text on long lines), justification bit meanings. Not traced: whether the menu pauses the world, rocks/minimap pictures, the level-up flame, picture attribute bits, index 0 in fonts.
**Next:** User checks listed in `interface.md` "How to check" 3–7 and `inventory.md` 4–5. B: the always-visible panel (`interface.md` rules 7–9, 20–36), then the bars and the held item (`inventory.md`). A: conversations.

## 2026-10-18 M4 step 4b-3: the user's check of the captions; the strip ends near the text

**Changed:** `kq8-core/src/caption.rs`: the stone strip no longer spans the whole width: it covers the widest line plus `STRIP_MARGIN` (16 pixels) on each side, centred (`strip_span`); unit test updated. Viewer log line gives the strip's span. `QUESTIONS.md` (Q107: the user's checks), provenance, `docs/FEATURES.md`.
**Why:** The user's check in the original: the strip reaches at most about two letters beyond the text on each side.
**Tested how:** By the user by eye (before this change): colours right; line breaks mostly right on several tombstones, not always (left for A, Q107 a). Then `cargo test --workspace` (432 pass), clippy clean, `cargo play --world Daventry --at 245000,255000,0 --click alcmsign,30 --frames 1500 --sound off` (`viewer-4b4-sign.log`).
**Result:** Sign: "strip x 121..518 (margin 16)" for the 365-pixel first line (365 + 2 x 16 = 397).
**Still broken / not tested:** The narrower strip is not seen by eye yet. The 16-pixel margin and the 400-pixel wrap are stand-ins until A answers Q107 (the user's estimate, not a measurement).
**Next:** The health panel and inventory bars from `Gplayscr.gui` once A's interface note is in.

## 2026-10-18 M4 step 4b-3: captions in the original font, on the stone strip

**Changed:** New `kq8-core/src/caption.rs` (word wrapping and the caption picture). Viewer `talk_view.rs`: `load_caption` reads the caption box from `Gplayscr.gui`, the font and strip it names, and the world's palette; `caption_picture_draw` draws the caption as a 640-wide picture at the bottom of the window, made again only when the text changes; the old 4a-4 text panel stays as a fallback. `main.rs`. `docs/DESIGN.md` (4b-3 plan with the user's observations), `QUESTIONS.md` (the user's check added to Q107), provenance.
**Why:** The user asked for the captions to look like the original. Place, centring, growing strip and the line break: the user's check in the original. Font, strip, line height and text offset: the layout data.
**Tested how:** `cargo test --workspace` (432 pass, 2 new), clippy clean. Viewer: `cargo play --world Daventry --at 245000,255000,0 --click alcmsign,30 --frames 1500 --sound off` (`viewer-4b3-sign.log`); `--at 204100,176500,0 --click tomb2.01,20` (`viewer-4b3-tomb.log`).
**Result:** Sign: "caption picture shown: 2 line(s) [\"The alchemist's shop. There could be something\", \"useful inside.\"], widths [365, 104] px", 640x32; hidden when the line ends. Tombstone: 4 lines (372, 331, 387, 332 pixels), 640x64.
**Still broken / not tested:** Not seen by eye (the user's check is next). Stand-ins: the wrap width (400), greedy word wrap, the strip across the full width, the world palette for the colours (Q106, Q107). There is no status bar yet, so the caption sits at the very bottom.
**Next:** The user's check by eye; then the health panel and inventory bars from the same layout.

## 2026-10-18 M4 step 4b-2: reading the screen layouts and the tag file; kq8 gui-verify

**Changed:** New `kq8-data/src/gui.rs` (the `.gui` layout tree and the text tag file). New `kq8 gui-verify` (`gui-verify.log`; copy `gui-verify-4b2.log`). `QUESTIONS.md`: Q107. Provenance.
**Why:** The layouts say where the caption box and the in-game panels are and which fonts and pictures they use. Layout from the community notes `kq8gui.txt` / `kq8tag.txt`.
**Tested how:** `cargo test --workspace` (430 pass, 2 new), clippy clean. `cargo run --release -p kq8-cli -- gui-verify`.
**Result:** `kqGuiTag.TTAG`: 739 tags, no line refused. All 9 layouts in `GAME/Resource` read to their exact end (241 blocks, every id has a name). `Gplayscr.gui` is the in-game screen: the caption box `IDDLG_CAPTION` (320×16 at (320, 400), font `console.pft`, picture `stontran.pbm`), the meters (health bar `4red.pbm`, experience `4green.pbm`, level number, level-up flame), 8 health slots with digit counters, the top bar (mask, 7 quest item slots, gold), bottom-left (armour, short and long weapon), bottom-right (rope, rocks, minimap), the map window and a yes/no/cancel popup.
**Still broken / not tested:** Nothing is drawn from the layouts yet. The caption box is one line of 320 pixels; how the original fits a long line in it is not in the data (Q107 a). Justification bits, the odd rectangles and the picture flags: Q107.
**Next:** 4b-3, captions in the original font, at the box's place.

## 2026-10-18 M4 step 4b-1: reading the game's fonts; kq8 font-verify

**Changed:** New `kq8-data/src/font.rs` (the `.pft` font: settings, character map, glyph table, glyph pictures from the bitmap array). New `kq8 font-verify` (`font-verify.log`, and a sheet of each font's glyphs in `kq8-artifacts/png/fonts/`). `docs/DESIGN.md`: the plan for 4b (fonts, screen layouts, captions in the original font). `QUESTIONS.md`: Q106. Provenance.
**Why:** The captions, the health panel and the inventory all draw text with the game's own fonts. Layout from the community notes `kq8pfon.txt` / `kq8pbma.txt`.
**Tested how:** `cargo test --workspace` (428 pass, 2 new), clippy clean. `cargo run --release -p kq8-cli -- font-verify`.
**Result:** 75 font files (60 different contents) all read to their exact last byte; every glyph fits its picture; bitmap counts match. Sizes: `20` (height 21), `27`/`36` (38), `45` (46), `20black` (29), the console fonts (15; `Resource` ones 12 and 15). The world fonts `14*.pft` hold only the space and digits 0-9. Glyphs are shaded: many palette values per font, and no font has its own palette (Q106: which palette).
**Still broken / not tested:** Nothing is drawn with these fonts yet. Nobody has looked at the sheets (they are grey: the real colours need the palette). What `sl`, `black`, `shdw`, `red` and flag bit 6 mean is not known.
**Next:** 4b-2, the screen layouts (`.gui`) and the tag file; the caption box is `IDDLG_CAPTION` in `Gplayscr.gui`.

## 2026-10-18 M4 steps 4a-7 to 4a-9: the user's checks; end-of-session documents (docs only)

**Changed:** `STATUS.md` ("Where we are": 4a-7 to 4a-9, older 3m/3n/4a entries shortened; the fixed differences, the answered Q103/Q104 notes and the done 4a checks removed; stand-ins of 4a and 4a-8 (Q105); next steps; 426 tests), `docs/FEATURES.md` (the click turn, turn keys during an automatic turn, the chest remark in any state; `--turn` / `--hold` wording; "doesn't turn to face" removed), `README.md` (remarks checked, Connor turns to what he clicks; M4 started), `docs/DESIGN.md` (4a-7 to 4a-9 done and checked), `QUESTIONS.md` (Q103, Q104 built and checked).
**Why:** `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** By the user in the viewer: the held turn key at the alchemist's sign, `tmbstn1` / `tmbstn2`, `wizbook` / `urn1`, the chest remark.
**Result:** All as described in the 4a-7 to 4a-9 entries, like the original.
**Still broken / not tested:** Q105 open (stand-ins). Two `eval` details of `anim-scripts.md` rule 18 not built (no data uses them).
**Next:** A: conversations and Q105. B, while waiting: a read-only survey of `loadKQ` with a name or place, and of scenery animations' sound cues, in Daventry.

## 2026-10-18 M4 step 4a-9: the chest remark turns Connor in any state; Q103 recorded

**Changed:** `kq8-core/src/active.rs`: a chest's `Take`/`Empty` remark no longer skips the turn when Connor is not in Wait or Move; only `doNotPointAt` gives no turn and no wait. The log says which state he was in. Unit test extended. Provenance: the 4a rows for Q103 (a)–(e) now cite the answers; no other code changed.
**Why:** Q103 (e), `talking.md` rule 19, `chests.md` rule 10. Q103 (a)–(c) confirmed our stand-ins.
**Tested how:** `cargo test --workspace` (426 pass), clippy clean. Viewer: `cargo play --world Daventry --at 202336,262000,1.5708 --click smlchst3,10 --click smlchst3,25 --jump 38 --click smlchst3,40 --frames 1500 --sound off` (`connor-4a9-chest-jump.log`, `doors-`, `talk-`).
**Result:** The third click comes at update 40, during the jump: "Connor (not Wait or Move) turns to face the chest … (any state, Q103 e)"; the turn runs in updates 42–48 while he is still in the jump; "'Tis empty." starts at 3.200 s (0.5 s after the click, at the next update).
**Still broken / not tested:** Not seen by eye. Two details of `eval` in `anim-scripts.md` rule 18 are not built (no data uses them; you asked for no code change there): `eval` with nothing after it gives 1 (the note says 0), and the words are not read again as a new line (no second `$` replacement). Recorded in the provenance.
**Next:** Your checks in the viewer; A answers Q105.

## 2026-10-18 M4 step 4a-8: Connor turns to face what he clicked

**Changed:** `kq8-core/src/active.rs`: new request `ClickTurn`; after every `ConnorAction` call (doors, scripted objects, scripted clickables, buildings with a script, items with a script) the click asks for a turn to the object's position unless it has `doNotPointAt`; `call_door` / `call_object` say whether a script ran; every click logs its call kind (`do`, item, `TooFar`) and "turn to <object> asked for" or why not; stock remarks and chests log "no turn". Another character's `speak` turn now says "always, no state check" (Q103 d; still only logged). `connor.rs`: one automatic turn (a cue-less request during a running turn only replaces the target; a `pointAndCue` keeps its cue). Viewer: `ConnorSim::click_turn_state` (Wait / Move), `door_view.rs` applies `ClickTurn` after the script's requests and logs "Connor's state after the script …". Unit tests updated (three counted the requests) and added. `QUESTIONS.md`: Q105. Provenance.
**Why:** Q104 (b), `clicking.md` rules 25–33, `doors.md` rule 16.
**Tested how:** `cargo test --workspace` (426 pass), clippy clean. Viewer runs (`--frames 1500 --sound off`): `--at 204100,176500,0 --click tomb2.01,20` and `--click tomb2.02,20` (the tombstones with scripts `tmbstn1.cs` / `tmbstn2.cs`), `--at 141703,203000,0 --lift 6000 --click wizbook,45`, `--at 216100,183500,0 --click urn1,20`; logs `connor-4a8-*.log`, `doors-4a8-*.log`, `talk-4a8-*.log`.
**Result:** `tmbstn1`: the line starts at 1.333 s, the moment of the click, and he turns while reading (updates 22–32, right, 0.233 per update). `tmbstn2`: the speak's turn and the click's are one turn ("the running turn gets this target"), the line starts at 1.867 s (0.5 s after the speak). `wizbook`: turns him (updates 46–60, ends facing it, heading 3.142). `urn1`: "doNotPointAt: no turn (rule 29)". A first `wizbook` run left Connor on the ground below the room (out of reach): "too far" remark, "no call: no turn".
**Still broken / not tested:** Not seen by eye. Q105 asked (an object deleted by its own click; the state is looked at one Connor update late in our build; buildings' `doNotPointAt` not read). Talk, weapon, throwing and tile-puzzle states don't exist yet. `Special` calls are not made yet (no conversations).
**Next:** 4a-9, the chest remark.

## 2026-10-18 M4 step 4a-7: turn keys are events; only a press cancels the automatic turn

**Changed:** New `kq8-core/src/turn_keys.rs` (the turn direction, changed only by a key going down or up). `connor_auto.rs`: the automatic turn is cancelled only by a turn key press, and during a turn without a walk forward/back and the run switch stay the player's. `connor.rs` (`Walker::turn_press`), `connor_update.rs` (the turn's end sets the direction to 0; logs: heading, turn direction and forward per turn update; "auto-turn cancelled"; "cue lost"). Viewer `connor_view.rs`: turn keys go through `TurnKeys` and every event is logged; `--turn` = pressed and held; `--hold`'s turn pressed at the start and let go at its end; `handsOff` off sends the held turn key again. `docs/DESIGN.md` (plan for 4a-7 to 4a-9), provenance.
**Why:** Q104 (a), `connor-movement.md` "Turning" rule 2, as the user saw in the original.
**Tested how:** `cargo test --workspace` (423 pass), clippy clean. Viewer: `cargo play --world Daventry --at 245000,255000,0 --turn 10,1 --click alcmsign,16 --frames 1500 --sound off` (`connor-4a7-sign-held-left.log`), and the same with `--click alcmsign,16 --turn 20,-1` (`connor-4a7-sign-press-during.log`).
**Result:** Held left: he spins at 0.117 per update until the click; the turn to the sign (updates 18–25) goes 0.233 per update (3.5 rad/s, nothing added) and is not cancelled; from update 25 his heading stays 2.4958 although left is still held (turn column 0). Right pressed during the turn (update 20): "auto-turn cancelled" at update 21, then he turns right at 0.117 per update. In both runs the line still starts at 1.600 s (0.5 s after the speak).
**Still broken / not tested:** Not tried by hand with the keyboard (the runs use `--turn`). The "other key sent again" and "handsOff off sends the held key again" paths are unit-tested only. When standing, the click turn still plays `Stop` for its first update, even while he was already turning on the spot (rules 24–25 as written).
**Next:** 4a-8, the turn after a click.

## 2026-10-18 The user's checks of Q104 in the original (docs only)

**Changed:** `connor-movement.md`, `clicking.md` ("How to check" and the rules: checked), `STATUS.md`. No code.
**Why:** The user tested the three predictions of the Q104 answer.
**Tested how:** By the user, by eye, in the original game.
**Result:** All three as traced: (1) holding left and clicking the sign, Connor turns to it and then stops turning although left is held; (2) `tmbstn1` starts reading at once and turns while reading, `tmbstn2` turns first and reads about half a second later; (3) `wizbook` turns him, `urn` doesn't.
**Still broken / not tested:** Our build still has the old behaviour (B's next step).
**Next:** Session B builds Q104 and the Q103 (e) fix.

## 2026-10-18 Study (A): Q104 (the automatic turn and held keys; the turn after a click) and Q103 (docs only)

**Changed:** `docs/behaviour/connor-movement.md` ("Turning" rule 2 corrected; How to check), `clicking.md` (new rules 25–33: Connor turns to face what he clicked; How to check), `talking.md` (rules 7, 10, 15, 19), `anim-scripts.md` (`eval` under rule 18; `pointAndCue` and turn-key edge cases), `doors.md` (rule 16, open question 3), `chests.md` (rule 10), `QUESTIONS.md` (Q103, Q104 answered). Scratch notes in `re/` (not committed). No code.
**Why:** The user's check of the original (Q104) and B's small points from 4a (Q103).
**Tested how:** Read in `Mask.exe` with Ghidra; data counts with grep over `MASK/`. Nothing run in the game.
**Result:** (1) Only a turn key **press** cancels an automatic turn; a held key is ignored during it (the 3.5 rad/s is unchanged) and does **not** take over when the turn ends (he stops turning until the key is pressed again; not checked in the game). Forward is not touched, so he walks/runs (and leans) through the turn. `pointAndCue` has no lock: a cancelled turn loses its cue. (2) The click turn belongs to every `ConnorAction` call (doors, scripted objects such as the tombstones, characters, buildings and items with a script), after the script; states Wait, Move, talk, weapon draw/put-away, throwing, tile puzzle; same turn; `doNotPointAt` stops it (items default to 1); `wizbook`/`wizquill` write `doNotPointAtMe`, which the game ignores, so they turn him. Stock remarks ("too far", "will not work") never turn him. (3) `eval` runs its words as a new line and keeps the inner result (B's stand-in right); `clearSpeak` leaves a line waiting out the 0.5 s alone (right); a placed voice stays where the line started (right); another character's `speak` turn: always, same 0.5 s; a chest remark turns him in any state of his, so the "not in Wait or Move → at once" stand-in is wrong.
**Mistake caught:** I first wrote that no Daventry object sets `tooFarAwayAction`; the data shows five (`girl`, `landslid`, `ropenhok`, `ugly`, `wizard`). Fixed in `clicking.md` rule 26.
**Still broken / not tested:** Nothing checked in the original beyond the user's earlier test. The "held key doesn't resume after the turn" rule and `wizbook` turning Connor need the user's check (`connor-movement.md` and `clicking.md` "How to check"). One switch on Connor that blocks all automatic turns is not identified. Whether any character file changes its turn rate was not traced.
**Next:** B builds the press-only cancel and the click turn (Q104), and drops the chest-state condition (Q103 e). The user checks the two points above.

## 2026-10-18 M4 4a: the user's checks of the chest remark and "This will not work" (docs only)

**Changed:** `docs/FEATURES.md`, `STATUS.md`: the checks recorded. No code.
**Why:** The user tested the two remarks not checked before.
**Tested how:** By the user, by eye and ear: clicking an empty chest; launching with `--item <type>` and clicking an object that doesn't take it.
**Result:** As expected: "'Tis empty." and "This will not work.". (The last code path that had never run in our own tests, `WONT_WORK`, is now checked.)
**Still broken / not tested:** Q104 (held turn key; the turn after a click) and Q103 still open. Other characters' voices not heard (no one else speaks outside conversations).
**Next:** A answers Q104 and Q103.

## 2026-10-18 M4 4a: the user's checks of remarks and speech (docs only)

**Changed:** `docs/behaviour/QUESTIONS.md`: Q104. `STATUS.md`, `docs/FEATURES.md`: the checks. No code.
**Why:** The user tested 4a in the viewer against the original.
**Tested how:** By the user, by eye and ear.
**Result:** Like the original: clicking the alchemist's sign, Connor turns, says the line, the caption shows, the background sound drops, his mouth moves; pressing forward doesn't stop that line; a movement key stops a tombstone's reading; clicking from too far gives "'Tis beyond my reach."; objects beyond the draw distance can't be clicked (nothing said; our build drops clicks on switched-off objects, `object-lighting.md` rule 41). Two differences: (1) in the original a held turn key does **not** stop the automatic turn to the clicked object, only a new press does; ours cancels it on any turn input, as `connor-movement.md` "Turning" rule 2 says. (2) In the original every tombstone turns Connor; ours turns him only for `tmbstn2`-`9` (`speak Connor $1`), not `tmbstn1`, `tmbstn10`-`19`, `tomb7` (`speak Connor Connor`, no turn by `talking.md` rule 7). The likely cause is the turn after a click (`doors.md` rule 16), which is not built (its states were never traced). Both asked as Q104.
**Still broken / not tested:** The two differences, until A answers Q104. Not checked: "This will not work.", the chest remark, other characters' voices.
**Next:** A answers Q104 (and Q103); then B builds the click turn and the press-only cancel.

## 2026-10-18 M4 step 4a: end-of-session documents (docs only)

**Changed:** `STATUS.md` ("Where we are": 4a; what to check by ear and eye; the 4a stand-ins; speech not done; next steps; Q103; 418 tests), `docs/FEATURES.md` (new section "Remarks and speech"; the not-working list), `README.md` (remarks in "What works"; `msg-verify`; talking removed from "What doesn't work", conversations stay), `docs/DESIGN.md` (4a done).
**Why:** `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** Read through; no code changed.
**Result:** —
**Still broken / not tested:** Every 4a step is checked in the logs and unit tests only, not by ear or eye. `talking.md` open questions 1-5 and Q103 stay open, so nothing is marked done in `QUESTIONS.md`.
**Next:** The user's checks (STATUS "To check by ear and eye (4a)"); A answers Q103 and studies conversations.

## 2026-10-18 M4 step 4a-6: Connor's mouth moves while he talks

**Changed:** Viewer `load.rs`: Connor's `baseLip` and the eight textures of his `lips=` list (`[ConnerLips]` `CF0`-`CF7.bmp`) are loaded. `connor_view.rs`: the materials showing the face texture are remembered, the eight images made. `talk_view.rs`: mouth changes and line ends go to `Speech`; `mouth_draw` puts `lip<n>` on the face materials (`lip0` for shapes of 8 or more and at rest), each change in `connor.log`. `main.rs`. Provenance.
**Why:** `talking.md` rules 11, 17-18.
**Tested how:** `cargo test --workspace` (418 pass), clippy clean. Viewer: `cargo play --world Daventry --at 245000,255000,0 --click alcmsign,30 --frames 1500 --sound off` (`connor.log`, `run-4a6-sign.txt`).
**Result:** `viewer.log`: "connor: mouth: baseLip cf0.bmp, lips [CF0.bmp … CF7.bmp]". `connor.log`: from update 40 "mouth shape 7: lip7 on 122 face material(s)" (one face material per animation), 36 swaps for the line's 39 changes (a change to the shape already shown is skipped), "mouth shape 0: lip0" at update 92 when the line ends. (A first try matched no material: a look's name is where it was found, `vol:Daventry/8bit/CF0.BMP`; fixed by comparing the file part.)
**Still broken / not tested:** Not seen by eye. Mouth changes come at most once per update (15 per second; the lip-sync file has 1/60 s steps). Other characters' mouths: only in `talk.log` (they are not drawn yet). Helmets and armour (`lipsList=`) not done (M5). Shapes above 7: `lip0` (rule 18; 1,992 such changes in the data, what the original does otherwise is open question 2).
**Next:** End-of-session documents; the user's checks.

## 2026-10-18 M4 step 4a-5: Connor's own remarks; cutting his line short

**Changed:** `kq8-core/src/talk.rs`: `TOO_FAR` (0 0 0 1 1), `WONT_WORK` (0 0 0 4 1), `cut_short` (rule 16; 1 unit test). `active.rs`: the "too far" and "This will not work" clicks (doors, scripted objects, buildings, items, chests) now say their line instead of only logging it; a chest's `Take…`/`Empty…` remark is said, with the turn to the chest unless `doNotPointAt` (then the 0.5 s wait); `connor_remark`, `chest_remark`. Viewer: a press of forward, back, a turn key or Jump (also `--jump`, `--turn`) and any click in the world (also `--click`, `--aim`) cut Connor's line short unless it has `#`; not during `handsOff` (`connor_view.rs`, `door_view.rs`). `docs/behaviour/QUESTIONS.md`: Q103 (e). Provenance.
**Why:** `talking.md` rules 16, 19; `chests.md` rules 8-10.
**Tested how:** `cargo test --workspace` (418 pass), clippy clean. Viewer (all `--sound off`): the chest, `cargo play --world Daventry --at 202336,262000,1.5708 --click smlchst3,10 --click smlchst3,40 --click smlchst3,80 --turn 105 --frames 2000` (`talk-4a5-chest.log`); a tombstone cut short, `cargo play --world Daventry --at 201000,178380,0 --click tomb2.01,20 --turn 45 --frames 1500`; too far, `cargo play --world Daventry --at 195000,178380,0 --click tomb2.01,20 --frames 900`.
**Result:** Chest: the 3rd click (update 80) turns Connor to `smlchst3` (`connor.log` from update 82), 0.5 s later "'Tis empty." (`GAME/ENGLISH/A000310E.0X1`, 0.994 s, `#`). Tombstone (`tmbstn1.cs`, 1000 44 14 0 1, no `#`, 11.971 s): the `--turn` press at update 45 cut it at 1.667 s; `sound.log`: the voice stopped, ducking off; no `TalkComplete`. Too far (distance 7,435, reach 4,700): "'Tis beyond my reach." (`GAME/ENGLISH/A0000000.011`, 1.736 s), no turn, `TalkComplete` to nobody.
**Still broken / not tested:** Not heard or seen by eye. "This will not work" not reached in a run (needs an item in hand that the target refuses; unit-test code path shared with "too far"). A press during a `#` line is logged as "not cut short" (unit test; not reached in a run). A chest with `doNotPointAt`, or Connor not in Wait or Move: queued at once (our reading, Q103 e). The Daventry monster-death remark and the inventory remarks are not built (no monsters die, no inventory screen).
**Next:** 4a-6, Connor's mouth.

## 2026-10-18 M4 step 4a-4: captions on screen

**Changed:** Viewer `talk_view.rs`: `Speech` keeps the caption (the issuer that showed it, the text); `setup_caption` / `caption_draw`: a panel at the bottom of the window, shown while the line's queue talks, hidden when it runs empty; each change in `viewer.log`. `main.rs`: the two systems. Provenance.
**Why:** `talking.md` rules 10-11 (captions follow `Options::Captions`, true in `Options.cs`).
**Tested how:** `cargo build`; viewer: `cargo play --world Daventry --at 245000,255000,0 --click alcmsign,30 --frames 1500 --sound off` (`viewer.log`, `run-4a4-sign.txt`).
**Result:** `viewer.log`: `caption shown: " The alchemist's shop. There could be something useful inside."` right after the line starts (update 38); `caption hidden` after it ends. RESULT: PASS.
**Still broken / not tested:** Not seen by eye. STAND-IN: the panel's look (Bevy's own font, white text on a dark band, 80% of the window wide, 40 px from the bottom) is ours; the original's caption panel is not studied (`talking.md` open question 4).
**Next:** 4a-5, Connor's own remarks and cutting a line short.

## 2026-10-18 M4 step 4a-3: hearing speech, and other sounds turned down

**Changed:** `kq8-core/src/sound.rs`: group 3 for speech, `Start::speech` (Connor's plain, others placed, 2,000 / 30,000), `set_ducking` (groups 0, 2, 5 x 0.2; 1 unit test). Viewer `talk_view.rs`: `Speech` resource and `talk_step` (the talk events become voice orders and the ducking switch, after the traps in each update); `sound_view.rs` plays and stops the voices and sets the ducking. Provenance.
**Why:** `talking.md` rules 10-11, 15; `sound.md` rules 53, 58.
**Tested how:** `cargo test --workspace` (417 pass), clippy clean. Viewer: `cargo play --world Daventry --at 245000,255000,0 --click alcmsign,30 --frames 1500 --sound off` (`sound.log`, `talk.log`, `run-4a3-sign.txt`). With `--sound off` the rules and the log run, nothing is heard.
**Result:** `sound.log`: "start #2 daventry/English/A0RS1J00.001 (3.542 s) group 3 plain, loudness 0.340"; "ducking on (rule 53): groups 0, 2, 5 x 0.2; loudness now 0: 0.028, 2: 0.028, 5: 0.028" in the same frame; "ducking off … 0: 0.140, 2: 0.140, 5: 0.140" at real 6.530 s, the voice finished at 6.602 s (the line ends in game time at 3.6 s, the next update after 3.542 s).
**Still broken / not tested:** Not heard by anyone yet (needs a run with sound). A cut-short voice (`clearSpeak`) is stopped by code not reached in a run (no Daventry script reached in our runs uses it). Another character's placed voice not seen in a run (no monster speaks without a conversation). The ducking may lift up to one update before the voice ends (game time vs real time).
**Next:** 4a-4, captions on screen.

## 2026-10-18 M4 step 4a-2: the speech queues and the script commands

**Changed:** New `kq8-core/src/talk.rs` (each character's queue, the duplicate check, the 0.5 s wait slot, line timing, mouth schedule, events for the viewer; 4 unit tests). `active.rs`: `KQMonster::speak`, `clearSpeak`, `testMessage`, `stateChangeNotify`, `clearStateChangeNotify` are real commands now (no longer stubs); `TalkComplete` to the issuer's script and, through the event queue, to the notify group; Connor turns to the listener (scripted turn) when in Wait or Move; monsters (name, place, script); 3 unit tests. `script.rs`: `eval` (stand-in, Q103). Viewer: new `talk_view.rs` (message files of the world's `English` folder then `GAME/ENGLISH`, voice and lip-sync lookup, `Options.cs`'s `set` lines), `talk.log`, the loader collects `KQMonster`s. `docs/behaviour/QUESTIONS.md`: Q103. Provenance.
**Why:** `docs/behaviour/talking.md` rules 6-15, "Do not reproduce".
**Tested how:** `cargo test --workspace` (416 pass: 110 + 290 + 5 + 11), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 245000,255000,0 --click alcmsign,30 --click alcmsign,45 --click alcmsign,50 --click alcmsign,60 --click alcmsign,120 --frames 3000 --sound off` (`talk.log`, `connor.log`, `run-4a2-sign.txt`).
**Result:** Update 30 (2.000 s): `alcmsign.cs` says `KQMonster::speak Connor alcmsign 1000 55 0 0 1 0`: "Connor turns to face alcmsign at (245734, 255974)", waits 0.5 s; `connor.log` updates 32-42: `stop`, then `left`, heading to 2.496. Update 38 (2.533 s): line #0 starts, voice `daventry/English/A0RS1J00.001` 3.542 s, plain, other sounds x 0.2, `#` (can't be cut short), 39 mouth changes, caption " The alchemist's shop. There could be something useful inside.". The clicks at updates 45, 50, 60: "duplicate: not queued" (the one at 50 replaced the waiting slot). Update 92 (6.133 s): finished, queue empty, "TalkComplete 1000 55 0 0 1: Connor has no script; notify group: nobody". The click at update 120 plays it again (line #1, 8.533-12.133 s). Daventry has 4 placed `KQMonster`s (Girl, Wizard, Ravenw, Shadow); captions true from `Options.cs`.
**Still broken / not tested:** Nothing is heard or shown yet (4a-3, 4a-4); the events are only logged. The notify group (`wizbook.cs`) is tested by a unit test only: reaching its line 1000 63 in the viewer needs its animation chain. Stand-ins (Q103): `eval`; waits kept through `clearSpeak`; a monster's turn not done. Monsters' scripts get `TalkComplete` but never `Init` (M5).
**Next:** 4a-3, the voice and the ducking.

## 2026-10-18 M4 step 4a-1: reading message, talker and lip-sync files

**Changed:** New `kq8-data/src/msg.rs` (message files, stage directions and the "no stop" mark, voice and lip-sync file names in base 36, `TALKERS.SH`, lip-sync files; 5 unit tests). New check `kq8 msg-verify` (`kq8-cli/src/msg_verify.rs`). Plan for 4a in `docs/DESIGN.md` (M4). `docs/provenance/README.md`.
**Why:** First step of talking (`docs/behaviour/talking.md` rules 1-5, 17): the game's lines and their files.
**Tested how:** `cargo test -p kq8-data msg` (5 pass); `cargo run -p kq8-cli -- msg-verify` (`msg-verify.log`).
**Result:** RESULT: PASS. 12 files, 2,080 lines, 476 with `#`; Daventry 254 lines (100 `#`, Connor 172, Wizard 40, Uglybeast 17, Girl 8, Knight 6), `0.MSG` 140 (21 `#`, 6 without a voice); lines without a voice file as the note says in every folder; `castled` and `temple2` are copies with no voices of their own; 1000 55 0 0 1 = `A0RS1J00.001` " The alchemist's shop. There could be something useful inside."; 0 0 0 1 1 "'Tis beyond my reach.", 0 0 0 4 1 "This will not work.", 0 109 14 33 1 "'Tis empty.". 1,538 lip-sync files read, 1,992 of 57,430 mouth changes have a shape above 7 (not traced, open question 2).
**Still broken / not tested:** Nothing plays yet; the lines are only read.
**Next:** 4a-2, the queues and the script commands.

## 2026-10-18 M3 3n: the user's check of the mill wheel fix (docs only)

**Changed:** `docs/FEATURES.md`, `STATUS.md`: the check recorded. No code.
**Why:** The user tested the fix for the double mill wheel cutscene.
**Tested how:** By the user, by eye, against the original.
**Result:** Everything works as expected: both sides of the mill wheel, walking, the watermill cutscene.
**Still broken / not tested:** Q100–Q102 still open (stand-ins in place).
**Next:** A answers Q100–Q102; B's next steps as in STATUS.

## 2026-10-18 M3 3n: the user's checks; a script's `move Connor` no longer sweeps the traps

**Changed:** `door_view.rs`: after a script's `move Connor`, the traps' swept test starts again from his new spot (the line from the old spot to the new one is no longer tested). `docs/behaviour/QUESTIONS.md`: Q102; a note on Q101. `docs/FEATURES.md`, `STATUS.md`: the user's checks.
**Why:** The user's checks of 3n: the mill wheel from the south (`wheeltp1.cs`) matches the original; walking feels like the original; the watermill walked into from outside matches. From the north, `wheeltp2.cs` ran and then `wheeltp1.cs` too, both cutscenes at once. Cause (`run-wheel2.txt` before the fix): `wheeltp2`'s cue 2 moves Connor from (186300, 232800) to (185800, 211300); our trap test swept that jump as a walk along the line, through `wheeltp1`'s box (y 227,000–228,000), so `wheeltp1` sent `On` with Connor at (185800, 211300), nowhere near it. `traps.md` rule 5 says whether a `move` counts was not traced (Q102).
**Tested how:** `cargo test --workspace` (404 pass), clippy clean. `cargo play --world Daventry --at 186300,234500,0 --hold 1,0,0 --frames 2500 --sound off` (from the north) and `… --at 186300,226000,3.1416 …` (from the south); the watermill walked into: `cargo play --world Daventry --at 192408,225500,3.1416 --hold 1,0,0,30 --frames 3500 --sound off` (`connor-watermill-walkin.log`).
**Result:** From the north: only `wheeltp2` runs; `Off` at update 133 (his first move after `handsOff 0`); the camera follows `wheeltp2`'s script only (`camera-wheel2-fixed.log`). From the south: as before. Watermill walked in: the scripted walk ends with `walk` at phase 0.4737 (he was still in `stopwalkstop` when the script moved him), the turn's `Left` drops the waiting `stopwalkstop` and starts at once, so no stop transition carries him; started inside the trap (`--at 192408,227039,0`) it ends at 0.9474 and `walkstopright` (~440 units) plays while he turns: the extra step the user saw. Both follow `anim-scripts.md` rules 24–25.
**Still broken / not tested:** The fix is not checked by eye. STAND-IN: whether a `move` enters a trap at the new spot at once, or ends the old one at once, is not traced (Q102; ours: only at his next move).
**Next:** The user checks the mill wheel from the north; A answers Q100–Q102.

## 2026-10-18 M3 step 3n: end-of-session documents (docs only)

**Changed:** `STATUS.md` ("Where we are": 3n; to check by eye; the 3n stand-ins; next step; Q97–Q99 built, Q100–Q101 asked), `docs/DESIGN.md` (3n done). README unchanged (no new keys or files; nothing new checked by eye).
**Why:** `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** Read through; no code changed.
**Result:** —
**Still broken / not tested:** Every 3n step is checked in the logs and unit tests only, not by eye.
**Next:** The user's checks by eye (STATUS "To check by eye (3n)"); A answers Q100, Q101.

## 2026-10-18 M3 step 3n-5: logs for the end of a scripted walk (Q98)

**Changed:** Viewer `connor_view.rs`: in the update a scripted walk ends (distance gone or head-on), `connor.log` gives the walk's phase, its stop transitions with their from points, and the next one from that phase; when that stop transition has ended, the updates it played in, its own steps in the data, the distance along his path and straight, and the heading change. No rule changed. `docs/behaviour/QUESTIONS.md`: Q101.
**Why:** A's answer to Q98 says the build already matches `anim-scripts.md` rules 24–25; these logs give the numbers open question 6 asks for, to compare with the user's check in the original.
**Tested how:** `cargo test --workspace` (404 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 192408,227039,0 --frames 3000 --sound off` (`connor-3n5-watermill.log`).
**Result:** Update 56: "the scripted walk ends at (190783.2, 230139.7), heading 3.546: walk at phase 0.9474 …; walk's stop transitions: [stopwalkstop from 0.90, walkstopright from 0.95, walkstopleft from 0.45]; the next from this phase: walkstopright from 0.95 (0.0026 of a loop on)" (2,779 gone of 2,736: 43 past, the last step was 75). Update 63: "walkstopright (its own steps in the data: 461.2 (left 20.1, forward 460.8) over 0.400 s) played in updates 57-62 (6 updates …): 441.7 along his path …; since the walk ended: 465.1 along his path, 437.2 straight, now at (190385.1, 230320.5), heading 3.546 -> 4.847". All as rules 24–25 say (never short, past by less than a step, the ordinary stop transition, the turn curving it, `Left` waiting for it). One detail the rules don't state decides this case: whether Wait asks for `Stop` in the same update the end is found (ours) or the next (then `walkstopleft`, about half a loop later): Q101. Neither explains the user's "a bit more than the original".
**Still broken / not tested:** The user's difference in the watermill cutscene is still not explained. The head-on end was not seen in a run.
**Next:** End-of-session documents; the user's checks by eye.

## 2026-10-18 M3 step 3n-4: the height on a flight's last update and after a cut (Q97)

**Changed:** `kq8-core/src/camera_flight.rs`: `settle_height` (the 3n-2 height rule with the aim point as not moved: lifted at once, snapped within 100, otherwise kept); the flight's last update and `cut` use it (`cut` now takes the ground and returns a `Cut`); the stand-in field `wanted_height` is gone; 1 new unit test, the watermill test checks "kept". Viewer `game_camera.rs`: the stand-in log wording ("would want … not applied (stand-in, not settled)") replaced by what the rule did, with the floor-grid ground under the camera.
**Why:** A's answer to Q97 (`camera.md` "Camera flights and cuts" rules 3, 4, 7).
**Tested how:** `cargo test --workspace` (404 pass), `cargo clippy --workspace --all-targets` clean. Viewer: the watermill (`cargo play --world Daventry --at 192408,227039,0 --frames 3000 --sound off`, `camera-3n4-watermill.log`), the mill wheel (`cargo play --world Daventry --at 186300,226000,3.1416 --hold 1,0,0 --frames 1500 --sound off`, `camera-3n4-wheel.log`).
**Result:** Watermill flight 31/31: at (188999.0, 229341.5, 10863.4), "kept (z before it 10863.4; wanted 11400.0; the floor-grid ground here 7650.0, + 1400 = 9050.0, 536.6 under the wanted height: kept)": the ground is **not** above 9,463.4, so no lift; same spot as before. Mill wheel flight 46/46: at the spot (185000, 213300, 8800), "kept", 1,790 above the wanted 7,010 (ground 5,590.8).
**Still broken / not tested:** Not checked by eye (nothing visible should change). No cut in Daventry's data (unit tests only).
**Next:** 3n-5, logs for the end of a scripted walk (Q98).

## 2026-10-18 M3 step 3n-3: `move KQCamera` while following keeps its heading

**Changed:** Viewer `game_camera.rs`: a `move KQCamera` while the camera follows Connor (and no flight) remembers the script's heading; the camera updates after it set the tilt at once (the view looks at the aim point, as any follow update) but keep that heading, and log how far off Connor it is; `follow`, a flight or a cut forget it. `docs/behaviour/QUESTIONS.md`: Q100 (this stand-in, and two small points from 3n-1/3n-2).
**Why:** A's answer to Q99 d (`camera.md` "`move KQCamera` while following"): the heading is not snapped; it goes through the swing-behind rules, which are not decoded (open question 7).
**Tested how:** `cargo test --workspace` (403 pass), `cargo clippy --workspace --all-targets` clean. Viewer: the mill wheel walked into (`cargo play --world Daventry --at 186300,226000,3.1416 --hold 1,0,0 --frames 1500 --sound off`, `camera-3n3-wheel.log`); the watermill (`cargo play --world Daventry --at 192408,227039,0 --frames 3000 --sound off`, `camera-3n3-watermill.log`).
**Result:** Mill wheel tick 18: "heading kept at -0.300 (STAND-IN, open question 7), +0.259 rad off Connor (-0.559); tilt set at once: -0.846" (the note: about 0.26 off, tilt about −0.83 with the aim 1,400 above his feet; ours aims 800 above). Position (185500.0, 226520.0, 9627.3) as in 3n-2. Watermill unchanged: its script uses `follow none`; the flight ends at (188999.0, 229341.5, 10863.4), heading 2.153, tilt +0.016.
**Still broken / not tested:** Not checked by eye. STAND-IN: the heading stays kept in later updates too, until the next `follow`, flight or cut (Q100 a; no Daventry case).
**Next:** 3n-4, the flight's last update (Q97 wording and the ground check).

## 2026-10-18 M3 step 3n-2: the refined camera height rule

**Changed:** `kq8-core/src/camera.rs`: `height_rule` (lift at once to ground + 1,400, or along the line when deeper; snap within 100; otherwise ease 10% from the height at the start of the update only if the aim point moved more than 20 by |Δx| + |Δy| + |Δz|, else kept; snapped with the mouse swing if lifted earlier), `height_share` (1 − 0.9^(15 × seconds)), the "lifted" memory; an update's order is now leash, height, then the push out to the minimum. `HEIGHT_RATE` (your hand-set 0.5) is replaced by `HEIGHT_EASE` 0.1. 3 new unit tests replace the old height test. Viewer `camera.log`: a last column with the height step (lifted / snapped / eased / kept), the aim's movement sum, the wanted height, the height at the start and before the rule.
**Why:** A's answers to Q97/Q99 c (`camera.md` "Height above the ground" rule 2).
**Tested how:** `cargo test --workspace` (403 pass), `cargo clippy --workspace --all-targets` clean. Viewer: the mill wheel walked into (`cargo play --world Daventry --at 186300,226000,3.1416 --hold 1,0,0 --frames 1500 --sound off`, `camera-3n2-wheel.log`), a 60-update walk (`cargo play --world Daventry --at 277800,198800,3.1416 --hold 1,0,0,60 --frames 2500 --sound off`, `camera-3n2-walk.log`).
**Result:** Mill wheel, the first shot (tick 18): "eased 1109.6 8523.1 9750.0": **(185500.0, 226520.0, 9627.3)**, as the note works out (~9,627); then no update until the flight; the second shot at exactly (185000, 237850, 10850). Walk: an update every tick while he walks; the camera trails at 5,000 (range 1,500–5,000) from tick 46; on flat ground the height only snaps; last update at tick 72 (his last move), "still" at 75, nothing after.
**Still broken / not tested:** Not checked by eye; the 10% easing may feel slower than your 0.5 on hills (one constant, `HEIGHT_EASE`). The easing on a slope was not seen in a run (the walks were on flat ground). STAND-IN: a deep lift whose line meets no ground goes to ground + 1,400.
**Next:** 3n-3, `move KQCamera` while following (the heading kept).

## 2026-10-18 M3 step 3n-1: the camera updates only when it should

**Changed:** `kq8-core/src/camera.rs`: `connor_moved` (position > 0.1 on an axis, or a facing change), `Camera::tick` (once per game update: 3 updates without a "moved" message make Connor still and reset the previous aim point), `Camera::not_moved` (for `follow`), `update_with` takes whether Connor moved; the heading no longer goes into the camera; 2 new unit tests. Viewer `game_camera.rs`: `camera_step` runs every tick but does a camera update only for a cause (Connor moved, `follow`, the "turn back" switch, the player's camera controls; flights as before); the "turn back" switch (on/off logged); every update's causes in `camera.log`'s last column, ticks without an update have no line. `door_view.rs`: a script's `move Connor` is labelled for the camera's log. Plan in `docs/DESIGN.md` (3n).
**Why:** A's answer to Q99 (`camera.md` "When the camera updates at all"): our camera recalculated every tick (Q53 d), which undid the mill wheel's script camera spots while Connor stood still.
**Tested how:** `cargo test --workspace` (401 pass), `cargo clippy --workspace --all-targets` clean. Viewer: the mill wheel walked into from the south, `cargo play --world Daventry --at 186300,226000,3.1416 --hold 1,0,0 --frames 1500 --sound off` (`camera-3n1-wheel.log`, `run-3n1-wheel.txt`); a walk into a tree, `cargo play --world Daventry --hold 1,0,0,30 --frames 400 --sound off` (`camera-3n1-walk.log`).
**Result:** Mill wheel: the trap fires at tick 18; its script moves the camera to (185500, 226520, 9750), then one camera update labelled `moved(script)` runs from there; none at tick 19 (Connor's landing moved him less than 0.1); "still" at tick 21; then **no update** until the flight: cue 4's shot stays exactly at (185000, 237850, 10850), tilt −0.25, heading −2.8, and the flight starts from there. The first shot's update ends at (185479.1, 226533.3, 9136.5): the height is still the old rule (3n-2 changes it); x, y moved 21 units because an avoidance swing started before the trap (the watermill building, ticks 14–17) was still running (the rules keep a running swing; the note's worked example didn't have one). Walk: an update every tick while he walks (ticks 2–15), he walks head-on into `tree.06` at update 15, no update from tick 16 on.
**Still broken / not tested:** Not checked by eye. The height easing is still the old rule (0.5 of the gap), now only in updates. STAND-IN: one update on the first tick (Daventry's start-up script says `KQCamera::follow none`; how the original starts following is Q27 d). The player's camera controls count as a cause while used (not re-studied by A). The "turn back" switch is not set by any Daventry script (not seen in a run).
**Next:** 3n-2, the refined height rule.

## 2026-10-18 Study (A): Q97, Q98, Q99 (camera updates, flight's last update, end of a scripted walk)

**Changed:** `docs/behaviour/camera.md` ("Height above the ground" rule 2 rewritten; "When the camera updates at all" rewritten; "`move KQCamera` while following" and the mill wheel worked through; flight/cut last update settled; edge cases, do-not-reproduce, how to check, open questions 4 and 6). `docs/behaviour/anim-scripts.md` (rules 24–25: how a scripted walk's end is checked and stops, the turn during a stop transition; how to check; open question 6). `docs/behaviour/QUESTIONS.md`: Q97–Q99 answered. `STATUS.md`. Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** B's questions from 3m-4 and the user's checks of the watermill walk and the mill wheel cutscene.
**Tested how:** Traced in `Mask.exe` with Ghidra (camera update, Connor's move commit, the event queue, flight/cut, Wait/Move); numbers worked out from the data and B's logs. Nothing run in the game.
**Result:** Q97: the aim point counts as not moved on a flight's last update (and a cut's), so the height never eases there; B's stand-in (watermill stays at 10,863.4) is right. Q99: the camera updates only on Connor's "moved" messages (position > 0.1 or facing changed), after `follow`, during flights, and every update while a "turn back" switch is on (Daventry's scripts keep it off); the height eases only when the aim point moved > 20, from the old height; the trap's camera update comes after the script's `move KQCamera`, so the mill wheel's second shot stays exactly where the script puts it. Q98: the end is checked before each update's move (never short) and plays the ordinary stop transition, which the next turn curves; this is what B built.
**Still broken / not tested:** Not tested in the game. The user's watermill difference is not explained (list of things to compare in `anim-scripts.md` open question 6). The heading after a `move KQCamera` while following depends on the swing-behind rules (open question 7, not decoded). How mouse and tilt controls trigger camera updates was not re-studied.
**Next:** B builds the camera update rules (fixes the mill wheel shots); the user compares Connor's end spot in the watermill cutscene.

## 2026-10-18 M3 step 3m: the user's check of the mill wheel cutscene (docs only)

**Changed:** `docs/behaviour/QUESTIONS.md`: Q99. `STATUS.md`: the check and Q99. No code.
**Why:** The user's check by eye: the camera in the mill wheel cutscene (`wheeltp1.cs`, both times: entered from the north and from the south side of the wheel) is placed differently from the original, "a bit zoomed in, maybe the height". (A crash the user first reported was the `--frames 4000` limit ending the run.)
**Tested how:** B read `camera.log` / `connor.log` / `traps.log` of the user's run.
**Result:** The flight itself ends on the script's spot. The two shots before it are wrong: the script has no `follow none`, so our camera keeps following and recalculates every update: the first shot is lowered from 9,750 to 8,523 by the height rule; cue 4's is pulled in about 5,300 by the leash and dropped to 8,540; both re-aim at Connor. Connor stands still meanwhile, and `camera.md` says the follow camera recalculates only while he moves or the player uses the camera (not built, Q53 d). The details that decide the first shot (order in the trap's update, what counts as moving, whether the height rule lowers) are not in the note: Q99.
**Still broken / not tested:** Not fixed; waits for A's answer to Q99.
**Next:** A answers Q99, Q98, Q97; then B builds "the camera updates only while Connor moves or is used".

## 2026-10-18 M3 step 3m: the user's check of the scripted walk (docs only)

**Changed:** `docs/behaviour/QUESTIONS.md`: Q98 (the end of a scripted walk). `STATUS.md`, `docs/FEATURES.md`: the check. The previous entry corrected: the flight the user watched is the watermill room's; the windmill tumble (`miltrap1.cs`) has no flight; the mill wheel's (`wheeltp1.cs`, Connor knocked under the wheel) has one and is not checked by eye. No code.
**Why:** The user's check by eye: in the watermill cutscene Connor turns left and walks forward a bit more than in the original.
**Tested how:** B read `connor-3m3-watermill.log` (updates 17–72).
**Result:** The walk itself ends where the note says (update 56, (190783, 230140), within one step of (190800, 230100)). Then the stop transition `walkstopright` plays for 6 updates while cue 2's `pointAndCue` turns him, and its steps (about 460) carry him on in a curve to the left, to (190385, 230321). This is the likely cause of what the user saw; the note does not say whether the end of a scripted walk plays the stop transition or how it combines with the next turn (Q98).
**Still broken / not tested:** Not fixed; waits for A's answer to Q98. The extra early walk during the turn (3m-3) may add to the difference; not separated.
**Next:** A answers Q98 (and Q97).

## 2026-10-18 M3 step 3m: the user's checks by eye (docs only)

**Changed:** `docs/FEATURES.md`, `README.md`, `STATUS.md`, `docs/behaviour/QUESTIONS.md` (Q97 note): the checks recorded. No code.
**Why:** The user tested 3m-1, 3m-2 and 3m-4 in the viewer against the original.
**Tested how:** By the user, by eye.
**Result:** Match the original: the windmill tumble, both forward and backward (the back flip, `JumpDir` 1, was a unit test only until now); barrels pushed into a wall, with the wall stopping Connor; the camera flight. With the Q97 stand-in (no height rule on the last update) the flight looks like the original.
**Still broken / not tested:** Not checked by eye: the scripted walk's `Stop`/`Left`/`Walk` during the turn (3m-3); the user did not say whether the mill wheel's flight was checked as well as the watermill's. Q97 is still open (looking right doesn't settle the height).
**Next:** A answers Q97 when convenient; B's next steps as in STATUS.

## 2026-10-18 M3 step 3m: end-of-session documents (docs only)

**Changed:** `STATUS.md` ("Where we are" rewritten for 3m, 3l shortened; to check by eye; the 3m stand-ins; next steps; Q97 under open questions; Q94–Q96 removed from them; 399 tests), `docs/FEATURES.md` (pushing into a wall, scripted turns, camera flights, the windmill tumble; the old problems removed), `README.md` (cutscenes line; the three fixed problems removed from "What doesn't work"), `docs/DESIGN.md` (3m done).
**Why:** `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** Read through; no code changed.
**Result:** —
**Still broken / not tested:** Every 3m step is checked in the logs and unit tests only, not by eye.
**Next:** The user's checks by eye (STATUS "To check by eye"); A answers Q97.

## 2026-10-18 M3 step 3m-4: camera flights and cuts (`addCamLoc`, `flyToCamLoc`, `cutToCamLoc`)

**Changed:** New `kq8-core/src/camera_flight.rs`: the spot list (`newlist` empties it), the spot nearest the camera, the flight's length (N = whole(seconds × 15) + 1 updates; no seconds: distance ÷ 5,000 per second), the flight (the same step every update, the roll in equal steps to `endRoll` or 0, exactly on the spot at update N, then pulled onto the range around the point 1,400 above the aim point), the aim (tilt −0.2 + the angle up to that point, set each update; heading locked within 0.02, else a third of the rest per update, one snap angle 0.063 on both sides), the cut; 6 unit tests. `active.rs`: the three commands become requests (`CamSpot`, `CamFly`, `CamCut`; `endRoll` only with exactly five numbers); 1 unit test. `script.rs`: two stubs fewer. Viewer `game_camera.rs`: the spot list, the flight under way (its first step in the update after the command), the "places"/"aims" switches (`follow Connor` both on and the obstacle avoidance on, `follow none` both off), a flight or cut switches following and avoidance off; after the flight the camera stays; an empty list logs an error and does nothing; `move KQCamera` keeps the roll (still not drawn). `door_view.rs` applies the requests.
**Why:** `camera.md` "Camera flights and cuts" rules 1–7 (A, 2026-10-18): both Daventry cutscenes with a flight (the watermill room, the mill wheel) had none; the user confirmed the jump at the end of a flight in the original.
**Tested how:** `cargo test --workspace` (399 pass), `cargo clippy --workspace --all-targets` clean. Viewer: the watermill `cargo play --world Daventry --at 192408,227039,0 --frames 3000 --sound off` and the mill wheel `cargo play --world Daventry --at 186300,227500,0 --frames 4000 --sound off`, `camera.log`, `traps.log` (copies `camera-3m4-watermill.log`, `camera-3m4-wheel.log`).
**Result:** Watermill: cue 4 at update 66 (after 3m-3's earlier arrival): "flight: N = 31 updates from (191885.8, 232227.7, 9260.0) to the spot (190000, 230000, 10600), step (−60.83, −71.86, 43.23)"; the same step every update; the heading turns a third per update, snaps at update 5 and is locked from 6; update 30 of the flight comes with cue 5 (one update before the end, as the note says); update 31: at (188999.0, 229341.5, 10863.4), "3725.8 from the point 1400 above the aim: pulled onto the range (1000, 2500)", heading 2.153, tilt +0.016; then no change until cue 6's `move KQCamera`. Mill wheel: N = 46, at the spot heading −2.761, tilt −0.893; the range is still 1,500–5,000 there (no `setRange`), so 2,800.7 is within it and the camera stays on the spot.
**Still broken / not tested:** Not checked by eye. STAND-IN: on the last update (and after a cut) the height rule is not applied; the log gives the height it would want (watermill: 11,400, 536.6 higher) (Q97 asked). `follow Connor` during a flight ends it (the original keeps adding the steps until N, overruled by the leash; no Daventry script does this). No cut in Daventry's data (unit test only). Roll still not drawn (0 in both scenes). Connor is not told when a flight ends (only some of his special states use it, not traced).
**Next:** End-of-session documents.

## 2026-10-18 Item descriptions don't duck the music (docs only)

**Changed:** `talking.md` rule 20: the user's check recorded.
**Why:** Open point of the previous entry.
**Tested how:** By the user, by ear, in the original (right-clicking an inventory item with music playing).
**Result:** The music stays at full volume during an item description, as traced (no ducking).
**Still broken / not tested:** —
**Next:** B builds remarks after M3 3m.

## 2026-10-18 Item descriptions have no caption (docs only)

**Changed:** `talking.md`: new rule 20 (item descriptions in the inventory), rule 16 corrected, captions check recorded.
**Why:** The user saw captions for the sign's line but none for item descriptions (right-clicking an inventory item).
**Tested how:** Traced in Ghidra after the user's report; the item lines checked against `Invinfo.itl` and `0.MSG` (dagger 25 0 0 1 "I carry a fine dagger.").
**Result:** Item descriptions don't use `speak` or the queue: the voice file is played directly as a plain sound (one at a time), so no caption, no ducking, no mouth, no `TalkComplete`. Rule 16 had wrongly said moving cuts them short.
**Still broken / not tested:** Whether other sounds stay at full volume during a description was not checked in the original. The inventory screen itself is M4.
**Next:** B builds remarks after M3 3m.

## 2026-10-18 The user's checks of the remarks study (docs only)

**Changed:** `talking.md` "How to check": the user's checks recorded.
**Why:** The user checked three points of the study in the original.
**Tested how:** By the user, by eye and ear, at the alchemist's sign in Daventry.
**Result:** All as traced: Connor turns to the sign and starts about 0.5 s later; other sounds drop while he speaks; pressing forward he walks and keeps talking; his mouth texture changes.
**Still broken / not tested:** Captions and cutting a line without `#` short were not checked.
**Next:** B builds remarks after M3 3m.

## 2026-10-18 Study session A: remarks and speech (docs only)

**Changed:** New `docs/behaviour/talking.md` (message files, sound and lip-sync file names, talkers, `KQMonster::speak`, the queue, captions, ducking, the mouth, `TalkComplete`, `stateChangeNotify`, `testMessage`, `clearSpeak`, what cuts a line short, Connor's built-in remarks). Pointers in `sound.md` (rule 53, 58, open question 3: `nofade`), `chests.md` (open question 1 answered), `doors.md` (stub table). Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** The user chose remarks as A's next study while B builds M3 3m: Connor says nothing yet (every remark is only logged), and the watermill cutscene's cue 5 speaks.
**Tested how:** Read in Ghidra (`Mask.exe`), and against the data with a throwaway reader in the scratchpad: all 12 message files parsed (format as in the community note `kq8msg.txt`), sound file names checked against the folders, Daventry's sign and chest lines looked up. Nothing was run in the original game.
**Result:** A line is a tuple (file, noun, verb, case, sequence) in `<file>.MSG`; its voice is `A…`, its mouth timing `S…` (base-36 name). Lines queue per character, duplicates are not queued, each line lasts as long as its sound; captions follow `Options::Captions`; other sounds go down to 0.2 unless `nofade`; Connor turns to what he talks to and starts 0.5 s later. `TalkComplete` runs after every line in the issuer's script (never for Connor) and in the scripts registered with `stateChangeNotify`. Lines without `#` are cut short by a movement/turn/Jump press or a world click. Stock remarks: "'Tis beyond my reach." (too far), "This will not work." (item refused). The community "TalkComplete freeze" matches the trace (a reused memory address); the note says how to avoid it.
**Still broken / not tested:** Conversations (`KQConner::conversation`, the talk state, `[n]` cameras) not studied. Not traced: which folders are searched, mouth shapes above 7, what ends a line without a sound file, the notify group's own forwarding (inferred from `wizbook.cs`). Nothing checked in the original.
**Next:** B: remarks after M3 3m (Connor's lines, captions, ducking, `TalkComplete`). A: conversations, or the scenery animation sounds.

## 2026-10-18 M3 step 3m-3: scripted walks: what plays during the turn, wall blocks (Q94)

**Changed:** `kq8-core/src/connor_auto.rs`: `AutoMove::turn_updates`, `turn_intent` (update 1 of the automatic turn nothing, so `Stop`; update 2 the turn direction as a turn key, `Left`/`Right`; from update 3 a walk or run while the turn finishes; `pointAndCue` `Left`/`Right` to the end), `walking`; `Before::Turned` / `TurnDone` carry the update number and the walker's intent; at the turn's end the walk distance is measured again; 1 new unit test, 2 changed. `connor.rs`: `Walker::turn_by_script` (the automatic turn's direction picks the movement, but his own key turning stays off); a scripted walk after its turn ignores the wall blocks. `connor_update.rs`: the intents and log lines per turn update; the head-on case also during the walking part of the turn; 1 new unit test (Stop/Left/Walk, and with a wall block Stop/Stop/Walk with the block kept). `active.rs`: `walkTo` / `runTo` with an object but not both cue numbers: console line, the walk, no cue (test extended).
**Why:** A's answer to Q94 (`anim-scripts.md` rules 24–25; rule 24 corrected): B's stand-ins for (b) a wall block and (c) what plays during the turn were wrong; (a) and (d) were right.
**Tested how:** `cargo test --workspace` (392 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 192408,227039,0 --frames 3000 --sound off` (the watermill cutscene), `connor.log`, `traps.log` (copies `connor-3m3-watermill.log`, `traps-3m3-watermill.log`).
**Result:** Cue 1's turn: update 18 "turn update 1 … asks for stop", 19 "turn update 2 … asks for left" (`stop` → `left`), 20 "scripted turn done in update 3: heading 3.546; Move starts afresh … walk distance measured again from (191877, 227585): 2736" (`left` → `walk`). Arrival at update 56 at (190783, 230140), 2779 of 2736 gone (one step past (190800, 230100)). That is 7 updates earlier than in 3l-2 (63): from `left` the walk starts without the slow `stop` → `walk` start; so all later cues are 7 updates earlier (2 at 56, 3 at 63, 4 at 66, 5 at 96, 6 at 126; control back at 127). Cue 2's `pointAndCue`: `stop` in update 1, `left` in updates 2–6, done in update 6, `stop` at 64.
**Still broken / not tested:** Not checked by eye. A wall block at the start, a head-on hit during the walk, `runTo` and `walkTo` with 4–5 words: unit tests only (none happen in Daventry). "Move starts afresh" only re-measures the distance; the walk animation is not restarted (the note doesn't say it is).
**Next:** 3m-4, camera flights.

## 2026-10-18 M3 step 3m-2: a push and a wall in the same sweep (Q95)

**Changed:** `kq8-core/src/connor_mover.rs`: new `Told` (the answer of `sweep_told`'s callback: contact, scripted push, or a physical push that moves his sweep to P). In `Scene::sweep_told`, after a physical push the objects later in the list are not tested (logged with their names). In the across pass the push no longer ends the pass: P = the sweep's start + 0.95 × the object's movement; a contact kept from earlier in the list (Daventry's buildings come first) is then handled with P as the wanted end and no movement: never head-on, P projected onto the surface's plane through the contact, he is put at the contact and slides there (hit result "after push"); a push inside a slide moves the object by the slide's step and P is measured from the slide's start. 2 new unit tests.
**Why:** Q95 (the user's check, 2026-10-18): Connor followed a barrel through the pub's wall; in the original the wall stops him. A's answer: `pushing.md` rule 4.3, `connor-physics.md` "Against a building" step 7.
**Tested how:** `cargo test --workspace` (390 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 207264,267646,-1.5708 --hold 1,0,0,300 --frames 2000 --sound off` (`barrel.04` pushed west into the pub's east wall), `connor.log` (copy `connor-3m2-barrel-pub.log`). The bricks run again (`--at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200`).
**Result:** Pushing from update 10 at 38.7 per update. Update 53: Connor reaches the wall at x 205452.9. Update 54: "push, then the kept hit on Pub: wanted end P (205416.2, …), movement 0 (not head-on); onto the plane through the contact: (205452.9, …)": he stays at 205452.9 (the same place a walking head-on stop puts him, update 64) while the barrel goes on into the wall. Updates 55–58: no pushing touch (his push step no longer reaches it). Update 59: Push → Move; the first walk step (75.1) reaches the barrel once more (moved to x 204827.6), Push again. Updates 60–63: no touch; update 64: Push → Move, walking head-on into the Pub: `walk` → `stop`, forward block on. His x never goes below 205452.9. Bricks: `Bricks` still gets `Push` at update 13.
**Still broken / not tested:** Not checked by eye. Objects later in the list after a push are not tested (stand-in: A left it open; no case in Daventry, where every building comes before the barrels and the barrels stand apart). The hay bale (a ball-stack pushable) pushed into a wall at an angle: not re-run.
**Next:** 3m-3, scripted walks (Q94).

## 2026-10-18 M3 step 3m-1: the windmill tumble: `KQConner::endJump`, `JumpDir` (Q96)

**Changed:** `kq8-core/src/active.rs`: `KQConner::JumpDir` and `KQConner::endJump` are world commands (no longer stubs): `JumpDir` answers 1 in the Jump state with `backJump`, else 0 (outside a jump with the console line); `endJump` outside the Jump state gives the console line and 0, in it `Request::EndJump` and 1; `WorldState::connor_jump` (set by the viewer before scripts run); 1 unit test. `connor.rs`: `Walker::jump_finished`. `connor_update.rs`: `finish_jump` (what a crash does to the jump, no state change) and `state_request` (during an unfinished jump: with `handsOff` on the jump is finished first, else refused); 2 unit tests. `script.rs`: two stubs fewer. Viewer: `door_view.rs` `trap_step`: applies `EndJump`; `setState … special` and a scripted walk go through `state_request`, a refused one is tried again at each later update; the requests of a trap's script count as inside Connor's update (a `move Connor` among them gets the rest of his update, as a `Push` call's did). `connor_view.rs`: `jump_state`, `end_jump`, `state_request`, `refused`. `--jump N` is now repeatable (`main.rs`, FEATURES).
**Why:** Q96 (b) blocked play: after the windmill tumble Connor hung 400 above the floor with gravity off and Jump never worked again. A's answer: `anim-scripts.md` rules 28–30, `connor-physics.md` "Jumping" rule 4.
**Tested how:** `cargo test --workspace` (388 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 191200,229700,-1.5708 --jump 20 --jump 90 --frames 2000 --sound off` (the user's jump into the sails, and a second jump after it), `traps.log`, `connor.log` (copies `traps-3m1-windmill.log`, `connor-3m1-windmill.log`). The watermill run (`--at 192408,227039,0 --frames 3000`) again, for the "inside his update" change.
**Result:** Update 38 (trap `On`): "JumpDir: 0 (Connor is in the Jump state with jump)", "endJump: jump finished at z=9279 (phase 0.900): gravity on, steps 325", `move Connor` inside his update (on the ground at once), `setState … normal` carried out in the same update. `connor.log` from update 39: mover state `normal`, gravity true, mode `special` (was: state `jump`, gravity false, from the trap on, in the user's log `connor-user-windmill.log`). Cue 300 at update 71 (33 updates after the trap): moved to z 8050, one gravity step to 8014, then on the Watermill's floor at 7,860 at update 74 (where he stood before the jump; the note's 7,650 is the terrain under that floor). The second Jump at update 91 works (`jump from stop at (191300, 229530, 7860)`). Watermill: cues at the same updates as in 3l-2 (16, 63, 70, 73, 103, 133); at update 1 he is now put on the floor in the same update (z 7860) instead of dropping from 8014.
**Still broken / not tested:** Not checked by eye. The back flip (`JumpDir` 1) into the sails: unit test only. A state request refused during a jump (no `handsOff`): unit test only, not reached in Daventry; when the original tries it again was not traced (stand-in: every later update). The "cut short" mark does nothing visible in our code.
**Next:** 3m-2, a push and a wall in the same sweep (Q95).

## 2026-10-18 The user's check of the camera flight's end jump (docs only)

**Changed:** `camera.md` ("How to check", "Do not reproduce"): the jump at the end of a flight is recorded as confirmed.
**Why:** The user watched the watermill room's first-visit cutscene in the original.
**Tested how:** By the user, by eye, in the original.
**Result:** The camera jumps toward the end of the flight, as traced (rule 4, last update). The size of the jump (about 1,227 units by the trace) was not measured.
**Still broken / not tested:** The rest of the flight rules (straight line, even speed, aim) are not checked separately; the height on the last update is still not settled.
**Next:** B builds `endJump` / `JumpDir`, Q95, Q94, the camera flights.

## 2026-10-18 Study session A: Q94–Q96 answered, camera flights (docs only)

**Changed:** `docs/behaviour/`: `anim-scripts.md` (rule 24 corrected and extended: what plays during `walkTo`'s turn, head-on, a wall block, `walkTo` without an object; rule 25; new rules 28–30: `KQConner::endJump`, `KQConner::JumpDir`, the windmill tumble), `pushing.md` rule 4.3 (a push and a wall in the same sweep) and a check, `connor-physics.md` (Jumping rule 4 and "Against a building" step 7: pointers), `camera.md` (new "Camera flights and cuts": `addCamLoc`, `flyToCamLoc`, `cutToCamLoc`, rules 1–7 with Daventry's numbers; open question 4 partly answered; edge cases, do not reproduce, checks), `QUESTIONS.md` (Q94–Q96 answered); STATUS. Scratch notes with addresses in `re/notes.md` (not committed). No code.
**Why:** The user asked A to answer the questions B left after M3 3l (Q94–Q96; Q96 b blocks play at the windmill) and to study the camera flights both Daventry cutscenes use.
**Tested how:** Read in Ghidra (`Mask.exe`) and checked against the data (`miltrap1.cs`, `wtrmwalk.cs`, `wheeltp1.cs`, `newworld.cs`, `permwrld.cs`, every `addCamLoc` / `walkTo` line). Nothing was run in the original game.
**Result:**

- Q96 (b): `endJump` finishes the jump as a crash does (gravity on, steps 325, jump allowed to end) without changing the state; the script's `setState … special` then replaces Jump at once, so no Fall follows. With `handsOff` on, any state request during a jump does the same. B's mover staying in its jump state is wrong. (a): `JumpDir` is 1 only during a `backJump`.
- Q95: the push moves Connor's position and his sweep to the 0.95 point mid-sweep; a wall found earlier in the load-order list (all Daventry buildings come before the barrels) is still handled afterwards, with the 0.95 point as the wanted end and zero movement: never head-on, a slide onto the wall plane. B's "end the pass at the 0.95 point" is wrong.
- Q94: head-on in a scripted walk: B's stand-in right. A wall block doesn't stop the walk (stand-in wrong). During the turn: `Stop`, then `Left`/`Right` for 1 update, then `Walk` before the turn ends; the walk distance is measured again at the turn's end (corrects rule 24). `walkTo x y` alone: no cue (stand-in right).
- Camera flights: one list of spots (`newlist` 1 empties it); fly/cut use the spot nearest the camera; a flight is a straight line at constant speed over whole(seconds × 15) + 1 updates (no seconds: distance ÷ 5,000 units/s); roll changes evenly to `endRoll` (0 if not given); the camera stops following, and if it was aiming it looks 1,400 above the aim point with the default −0.2 tilt; on the last update the camera is pulled onto the range around that point (watermill: N = 31, a jump of about 1,227 units at the end). `miltrap1.cs` has no flight (only `move KQCamera`); the mill wheel's `wheeltp1.cs` has one.

**Still broken / not tested:** Nothing checked in the original. Not settled: the height on a flight's last update; when a `setState` refused during an unfinished jump is tried again; objects later in the collision list after a push (zero-length move). The end-of-flight jump is traced only; the user should look for it in the watermill cutscene.
**Next:** B builds: `endJump`/`JumpDir` (unblocks the windmill), the push/wall order (Q95), the `walkTo` details (Q94), the camera flights.

## 2026-10-18 `KQConner::getDoMode` (the windmill tumble without a bow)

**Changed:** `kq8-core/src/active.rs`: `KQConner::getDoMode` is a world command: it answers Connor's hand mode, always 4 (empty hands) because he has no weapons yet and `setDoMode` is still a stub; 1 unit test.
**Why:** The user saw Connor holding a bow in the windmill tumble. `miltrap1.cs` picks the tumble by `getDoMode`; unknown to us, it left `$Console::Return` at 1 = bow. The values (sword 0, bow 1, empty hands 4) are in `pushing.md` ("Words used here").
**Tested how:** `cargo test --workspace` (385 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 191200,229700,-1.5708 --jump 20 --frames 2000 --sound off` (the user's standing jump into the sails), `traps.log`, `connor.log`.
**Result:** Update 38: trap `miltrap1` On, "KQConner::getDoMode: 4 (empty hands; no weapons yet)", `setState Connor special AnimList=conmill.anm,AnimListOn,normal` (the "no weapon out" branch); `normal` plays to update 72.
**Still broken / not tested:** Checked by eye by the user afterwards (2026-10-18): he tumbles with no weapon in hand, as in the original. Showing the weapon he holds needs weapons (M5). After the tumble the jump still never ends (Q96 b); `JumpDir` is a stub, so always the forward tumble (Q96 a).
**Next:** Study session A: Q95, Q96, Q94, the camera flights.

## 2026-10-18 M3 step 3l: the user's checks by eye (docs only)

**Changed:** `docs/behaviour/QUESTIONS.md`: Q95 (Connor and a wall while pushing a barrel into it), Q96 (`KQConner::JumpDir`, `endJump`). `FEATURES.md`, `README.md`, `STATUS.md`: the checks and the two problems. `DESIGN.md` M7: barrels through walls noted as an improvement for after M7. No code.
**Why:** The user tested 3l in the viewer against the original.
**Tested how:** By the user, by eye. B looked at the cause of each problem in the code and logs.
**Result:** Match the original: the bricks puzzle (no pop-up), the hay bale, turning during the stumble; the watermill cutscene runs, without its camera flight (not built). Problems: (1) a barrel pushed into the pub's wall goes through (as in the original), but Connor follows it through; in the original the wall stops him and he stops pushing once the barrel is too far in. Cause: after an accepted physical push our across pass puts him at start + 0.95 × the barrel's movement and ends, so a wall contact in that sweep is never handled; how the original combines the two is not in the note (Q95). (2) The windmill tumble (`miltrap1.cs`) shows a bow: `traps.log` has "ERROR unknown command 'KQConner::getDoMode'", so `$Console::Return` stays 1 and the script takes its "bow weapon" branch (`cbow`). `KQConner::JumpDir` is a stub (0: always the forward tumble). (3) After the tumble Jump doesn't work (the user's check): `connor.log` shows the mover still in its jump state from the jump into the sails (gravity off) from update 417 on, hanging at z 8050 over ground at 7650; the script's `KQConner::endJump`, which presumably ends it, is a stub (Q96 b).
**Still broken / not tested:** Both problems are not fixed yet (the user asked to wait).
**Next:** B: `KQConner::getDoMode` (returns the hand mode, 4; from `pushing.md`). A: Q95, Q96.

## 2026-10-18 M3 step 3l: end-of-session documents (docs only)

**Changed:** `STATUS.md` ("Where we are" rewritten for 3l and shortened, next steps, Q94 under open questions, 384 tests), `docs/FEATURES.md` (pushing, cutscenes, crash turning, torches in step; what still doesn't work), `README.md` (barrels and cutscenes in "What works", marked logs only; the pop-up and the watermill removed from "What doesn't work"), `docs/DESIGN.md` (3l done), `docs/behaviour/QUESTIONS.md` (Q84–Q93 marked built).
**Why:** `docs/SESSIONS.md` "At the end of the session" (B).
**Tested how:** Read through; no code changed.
**Result:** —
**Still broken / not tested:** Every 3l step is checked in the logs and unit tests only, not by eye (see each entry below).
**Next:** The user's checks by eye; A answers Q94 and studies the camera flights.

## 2026-10-18 M3 step 3l-7: `loadKQ` while playing calls `Init`, refuses a name in use (Q90)

**Changed:** `kq8-core/src/script.rs`: `ScriptHost::take_calls` (scripts a world command asks to run at once, before the next line) and `ScriptHost::event_done` (called when an event's script run has finished). `active.rs`: `loadKQ` runs the new object's script with `$1=<name> $2=Init` at once; a name still used by a live object, or deleted during the same event, is logged as an error and the load is refused (the copy stays prepared); deleted names are remembered per event (`deleted_this_event`); `set_dormant` takes the object's script; 1 new unit test. `kq8-viewer` `load.rs`: the prepared copies carry their `script=`.
**Why:** A's answer to Q90 (`objects.md` "Where a world's objects come from" rule 6, `clicking.md` rule 22): B's stand-in gave no call; the original stops with a fatal error on a name in use (we refuse instead, as the note asks).
**Tested how:** `cargo test --workspace` (384 pass), `cargo clippy --workspace --all-targets` clean. Viewer: the bricks run (`cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 2500 --sound off`), `doors.log`, `traps.log`.
**Result:** Update 63: "loadKQ waterfa2.kq: waterfall2 comes into the world (as at the start); its script gets Init now", "runs waterfa2.cs waterfall2 Init at once" (before line 35), then `watrfsn2.kq` ("no script"). Update 64: both come in as before.
**Still broken / not tested:** The name check is not reached in Daventry's play (unit test only). `loadKQ` with a name or position while playing (barrels, boards) is still not done. A copy that comes in still has no collider and isn't clickable.
**Next:** End-of-session documents (STATUS, FEATURES, README, DESIGN, QUESTIONS).

## 2026-10-18 M3 step 3l-6: animated objects' first update, Connor's levels, `Die`'s `$3` (Q85, Q86)

**Changed:** `kq8-core/src/named_anim.rs`: an object with no `currAnimName` and a waiting movement starts that movement plainly on its first update (no `flags` bit 8 random part) and its phase moves on in that same update; the bit 8 random part (start frame = round(random × (frames − 1)), phase = frame ÷ frames; a random speed 0.9–1.1 that **replaces** the `loop=` speed) belongs to the current movement only (`random_speed` replaces `speed_scale`; the Q85 stand-in's `load_random` / `kept_random` are gone); 2 tests rewritten, 1 changed. `stats.rs`: on level up the second stat +5 and the third +2 (were swapped); test with the bricks' numbers and the leftover acted on at the next event. `active.rs`: the event's fourth number is cut to 100 when read; Connor's "below 0" check before the cap and the level-up heal; every `Die` call carries `$3` = `Normal` (`DIE_KIND`; nothing is hit by a weapon yet). `anim_view.rs`: the load line says when the first update will drop the random part.
**Why:** A's answers to Q85 and Q86 (`anim-scripts.md` rules 3, 19–20, `objects.md` "Animated objects at load" rule 4).
**Tested how:** `cargo test --workspace` (383 pass), `cargo clippy --workspace --all-targets` clean. Viewer: the bricks run (`cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 2500 --sound off`) and a plain start (`--frames 200`), `doors.log`, `anim.log`.
**Result:** `doors.log` update 0: `Davtorch.01`–`.05`, `deadbird.01`/`.02`, the fish: "waiting movement starts (rule 3: no current movement name): 'stop' at phase 0.0000 speed 1" (all in step, as the original). Update 13 (bricks, +50): "Connor [10, 10, 0, 0, 0] level 1 -> [20, 15, 2, 1, 40] level 2".
**Still broken / not tested:** Not checked by eye (the torches now flicker in step, as A traced). `Die`'s `$3` is always `Normal` until weapons exist (M5). Connor dying from a stats event is still only logged.
**Next:** 3l-7, `loadKQ` while playing (`Init`, a name in use).

## 2026-10-18 M3 step 3l-5: turning during `crash`; a glancing jump hit keeps the jump (Q84)

**Changed:** `kq8-core/src/connor.rs`: the turn keys turn Connor at 1.75 rad/s while `crash` plays (run crash and jump crashes); forward, back and run still wait for its end; still no turning from a jump's start to its landing's end. `connor_update.rs`: a steep hit during a jump that is not a crash (every `backJump` hit, and the not-head-on cases our code can't reach yet) no longer switches gravity on: the jump goes on with gravity off and steps 0 until its animation ends, then Fall (noted once per jump). `connor_mover.rs`: `back_jump_hit` removed, `jump_glanced`. Tests: the run-crash test now holds a turn key and checks the turn; the back-flip test checks gravity stays off.
**Why:** A's answer to Q84 (`connor-physics.md` "Jumping" rule 5, "Walls"), confirmed by the user in the original for the run crash (2026-10-18); turning in jumps, falls and landings stays off (Q41).
**Tested how:** `cargo test --workspace` (382 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 237968.9,187493.2,-0.2055 --hold 1,0,1,40 --turn 5,1 --frames 1500 --sound off` (the note's measured run-crash spot), `connor.log`.
**Result:** Crash at update 7, contact y 187449.5 (the original: 187449.2). Updates 8–14 (`crash`): heading −0.2055 → 0.6112, +0.1167 per update (1.75 rad/s); the push-back follows his turning facing; `stoprun` again at 15.
**Still broken / not tested:** Not checked by eye. Whether he leans while turning in a run crash is not settled (the note); ours leans if forward is held (the lean switch is left on from `run`). Turning in a down-pass crash straight out of a jump: the note's stand-in (yes), not checked. The back-flip change (gravity off after a hit) is in a unit test only.
**Next:** 3l-6, animated objects' first update and Connor's levels.

## 2026-10-18 M3 step 3l-4: pushing as traced (barrels move, the hay bale slides, every touch told; Q87, Q88, Q91)

**Changed:** `kq8-core/src/connor_mover.rs`: new `MoverWorld::sweep_told`: every object Connor's balls touch is told at once, in the order the objects entered the world (`Scene::order`), and the push check runs there; an accepted push is not recorded; a contact replaces the kept one when it is at most as far (a tie keeps the later-tested; the "pushable first" stand-in and `TIE` are gone). `push_object`: box-shaped pushables (no `collisionShape`) move by his step and are never stopped; ball-stack ones slide along what they hit (the hit and 3 retries, a normal z between 0 and 0.7 counts as a vertical wall, no head-on stop); the ground only raises (with `keepOnGround`); any height change puts it back. The facing test of a physical push uses his position at the start of the update (Q87 e). 3 old tests rewritten, 2 new. `kq8-viewer`: `terrain_view.rs` `ColliderOrder` (load order of the colliders), used by `connor_step` and by the scene of a scripted `move Connor`.
**Why:** A's answers to Q87/Q88 (`pushing.md` rules 2.5, 4.1–4.3, `connor-physics.md` "Against a building" step 7) and the user's checks in the original (Q91, 2026-10-18): barrels move, go through the pub's wall, stop uphill; the hay bale slides along a wall at an angle.
**Tested how:** `cargo test --workspace` (382 pass), `cargo clippy --workspace --all-targets` clean. Viewer: barrel `cargo play --world Daventry --at 205364,267646,1.5708 --hold 1,0,0,300 --frames 3000 --sound off`; hay bale at an angle `--at 146500,248928,1.2` and `--at 146500,248928,1.9` with the same `--hold`; bricks `--at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 7000`; `connor.log` (copies `connor-3l4-barrel.log`, `connor-3l4-bale.log`, `connor-3l4-bale2.log`).
**Result:** Barrel `barrel.04`: pushed from update 17 ("box-shaped: never stopped"), 75.1 then 38.7 per update, x 206564 → 208960 in 61 updates; from update 81 the ground ahead is 13.9 higher: "height changed by 13.873: put back" every update (it stays, as uphill in the original). Hay bale at heading 1.9: 15 free moves, then "slid along Windmill" for 157 updates (12.5 per update along the wall, y 248928 → 251090) until the next wall ("stopped at Windmill"). At heading 1.2: one slide, then stuck in a corner. Bricks: unchanged (update 13 `Bricks` gets `Push`, `Table` is the head-on hit). Daventry's 295 object colliders were already in load order.
**Still broken / not tested:** Not checked by eye. A barrel pushed downhill floats (rule 4.2): unit test only, no slope near Daventry's barrels. A barrel through the pub's wall: unit test only (the barrels tried here met no wall). With two physical pushables touched in the same sweep, the last one's movement is the one Connor follows (not described).
**Next:** 3l-5, turning during `crash`.

## 2026-10-18 M3 step 3l-3: cutscene control (`handsOff`, the camera's view, `follow Connor`, `setThirdPerson`; Q89)

**Changed:** `kq8-core/src/camera.rs`: `view_direction` / `view_angles` (rx tilt, rz heading from +Y, anticlockwise), `follow_turn_start` / `follow_turn_step` (more than 0.4 rad off: 1/3 per update, snap under 0.063 on both sides); 2 unit tests. `active.rs`: `handsOff 1 N` logs N; `setThirdPerson` is a world command (the paired toggle: flips the first-person key's lock in third person; no longer a stub); 1 unit test. `kq8-viewer`: `game_camera.rs`: `move KQCamera` sets the view from its angles (flag bit 2 adds them; roll logged, not drawn); `follow Connor` starts from where the camera is (range at once by the normal rules, tilt default, the heading turn back drawn by `camera_draw`); `hands_off` ends fixed mode; the mouse look is ignored while hands are off. `connor_view.rs`: while hands are off, key presses (and the key repeat) are swallowed, releases still count, the run key is ignored; `keys_sent_again` at `handsOff 0`. `door_view.rs` wires them.
**Why:** A's answer to Q89 (`anim-scripts.md` rules 21–23, `camera.md` "Shake and cutscene cameras", "What the camera looks at" rule 1, "First person" rule 7). The user saw the bricks cutscene's camera differ from the original: our camera kept looking at Connor's last aim point.
**Tested how:** `cargo test --workspace` (380 pass), `cargo clippy --workspace --all-targets` clean. Viewer: the bricks (`cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 7000 --sound off`) and the watermill (`cargo play --world Daventry --at 192408,227039,0 --frames 3000 --sound off`), `camera.log`, `traps.log`, `connor.log`.
**Result:** Bricks: update 12 the camera looks along heading 0.900, tilt −0.070; 63 heading 0.170, tilt −0.350; 123 heading 1.000; at `follow Connor` (124) the view is 0.190 off Connor (the note says about 0.19), so it looks at him at once. At `handsOff 0` (124) "held keys sent again: Forward", and he walks on at once. Watermill: heading 2.632 (the note: it points at the `walkTo` target), at cue 3 0.155 off and at cue 6 0.121 off: at once both times.
**Still broken / not tested:** Not checked by eye. The gradual turn back (more than 0.4 rad off) is not reached in these two scenes: unit test only. Roll (ry) is not drawn (0 in both scenes). `addCamLoc` / `flyToCamLoc` / `cutToCamLoc` are still stubs, so the watermill's camera flight (cue 4) does not happen. The GUI panels, mask icon and pointer of `handsOff` don't exist yet. First person doesn't exist, so `setThirdPerson` only logs the key's lock.
**Next:** 3l-4, pushing (barrels move, hay bale slides, every touched object told).

## 2026-10-18 M3 step 3l-2: the watermill cutscene (`walkTo`, `pointAndCue`; Q93)

**Changed:** New `kq8-core/src/connor_auto.rs`: Connor's automatic turn (3.5 rad/s, the shorter way, exactly onto the heading; a player turn cancels it) and scripted walk (straight ahead after the turn, until the flat distance measured at its start is gone); 4 unit tests. `connor.rs`: `Walker::auto`, `start_auto` (a walk puts him in Wait), `stop_at_once`. `connor_update.rs` `tick`: the turn or the walk's intent before the walker's update; after the move, the walk's end (or a head-on hit, Q94 stand-in) and `Tick::auto_end`. `active.rs`: `KQConner::walkTo`, `KQConner::runTo`, `pointAndCue` and `walkExit` are world commands (requests; `walkTo` and `pointAndCue` were stubs), `connor_auto_cue` sends `KQCueEvent c d`; 1 unit test. `script.rs`: two stubs fewer. Viewer: `connor_view.rs` collects the ends, `door_view.rs` applies the requests (a walk switches `handsOff` on) and sends the cues. Q94 asked.
**Why:** The user's check (Q93): the watermill room's cutscene never gave control back. A's answer (`anim-scripts.md` rules 24–27): cue 1's `walkTo` and cue 2's `pointAndCue` send the next cues only when Connor arrives / has turned; we had neither.
**Tested how:** `cargo test --workspace` (377 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 192408,227039,0 --frames 3000 --sound off` (Connor starts inside the trap `Wtrmwalk.tp`), `traps.log`, `connor.log`.
**Result:** `traps.log`: update 1 `On` (handsOff 1, camera and Connor moved), 16 cue 1 (`walkTo 190800 230100`), 63 cue 2 (`pointAndCue`), 70 cue 3 (`follow Connor`), 73 cue 4, 103 cue 5, 133 cue 6 with `handsOff 0`: 63 updates (4.2 s) after the turn ended, as rule 27 says. `connor.log`: turn 2.970 → 3.546 in updates 18–20 (0.233 per update), "walks straight ahead for 2736", "scripted walk done: 2783 gone of 2736" at 63 (checked after each step of about 75), then turned to 4.847 by update 70. Controls back at 134.
**Still broken / not tested:** Not checked by eye. The camera parts of the cutscene (`addCamLoc`, `flyToCamLoc`, the view angles) are still stubs or stand-ins (3l-3). A request from a cue is applied after that update's Connor move, so the turn starts one update after the cue. Q94 open (head-on during a scripted walk, a wall block at the start, what plays during the turn, `walkTo` without an object).
**Next:** 3l-3, cutscene control (`handsOff` in full, the camera's view from `move KQCamera`, `follow Connor` from where it is).

## 2026-10-18 M3 step 3l-1: no pop-up at the bricks (Q92)

**Changed:** `kq8-core/src/connor_mover.rs`: the down pass and ground check moved into `down_and_check` (same code); new `Mover::after_move` (a script's `move`: one gravity step, the terrain raise) and `Mover::rest_of_update` (the down pass and ground check that still follow a script run inside his update); 2 new unit tests. `kq8-viewer`: `connor_view.rs` `move_to` runs them; `door_view.rs` `trap_step` marks the requests made by this update's `Push` calls as "inside his update" and gives `move_to` the same scene his update used (the pushed objects still passed through).
**Why:** The user saw Connor, the bricks and the table rise a few units and fall back at the start of the puzzle (Q92). A's answer (`pushing.md` rule 11, `objects.md` "(d)"): the `Push` script runs inside Connor's update, so after its `move Connor … 15000` the rest of that update puts him on the floor at once; the visible table and bricks are part of his model then.
**Tested how:** `cargo test --workspace` (372 pass), `cargo clippy --workspace --all-targets` clean. Viewer: `cargo play --world Daventry --at 156625,303748,1.57 --lift 6000 --hold 1,0,0,200 --frames 7000 --sound off`, `connor.log`.
**Result:** Update 13: "moved Connor … to (156652, 303600, 15000); the move step: gravity step 36.0 -> z 14964.0; inside his update (a Push call): the rest of it: drop 108.0, step-down 325, down pass on Cave (landing) -> z 14837.0". Connor's z in every update from 13 to 64 (the whole `push`): 14837.0, never 15,000. Cue 10 (update 124, a timed cue, outside his update): put at 14964 (one gravity step below 15,000), 14856 at 125, 14837 from 126.
**Still broken / not tested:** Not checked by eye. The cue-10 drop (about 130 units for one update) is what the note infers for the original (not checked there). The script still runs after his move, not in the middle of it; only its `move Connor` is followed by the rest of his update.
**Next:** 3l-2, the watermill cutscene (`walkTo`, `pointAndCue`).

## 2026-10-18 The user's checks of the Q84/Q87 answers (docs only)

**Changed:** `connor-physics.md` (Jumping rule 5, "Walls"): turning during `crash` confirmed; the sentence "turning runs in every state" removed and replaced by the user's rule (no turning from the start of a jump to the end of its landing). `pushing.md`: the checks recorded. `QUESTIONS.md` Q84, Q87, Q91: the user's checks added.
**Why:** The user checked the 2026-10-17 answers in the original game.
**Tested how:** By the user, in the original: barrels uphill and against the pub's wall, the hay bale at an angle into a wall, a run into a wall holding a turn key.
**Result:** All match the trace: barrel stops uphill, goes through the pub's wall; the hay bale slides; Connor turns during the stumble. The user also repeated that he does not turn during jumps, falls or landings (standing, running, back flip). The 2026-10-17 note had generalised "turning works in every state" from the code; that contradicts this, so the note now limits it to `crash`. A short look for what switches turning off in jumps found only the two hands-off switches (scripted and animation lock), not the cause; not pursued further (same gap as Q59).
**Still broken / not tested:** Downhill barrels can't be tried in Daventry. Turning in a down-pass crash straight out of a jump is not settled (stand-in: yes).
**Next:** B builds the 3i–3k fixes (see the 2026-10-17 entry).

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
