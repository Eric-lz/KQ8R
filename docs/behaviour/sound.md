# Sound effects

Study session A, 2026-10-17. Sources: the `.aud` files in `MASK/`, the
community format note `kq8aud.txt` from `libkq8fpc/doc/` (listed in
`docs/RESEARCH.md`), Daventry's and `GAME/`'s data and scripts, and
`Mask.exe` (studied, described here in our own words). Music and speech
are covered only as far as they share the sound system (section "Music
and speech").

Words used in this note:

- **Plain sound**: plays at the same loudness wherever Connor is, centred
  (no left/right).
- **Placed sound**: has a position in the world; its loudness depends on
  how far Connor is from it, and its left/right balance on where it is
  seen from the camera.
- **Group**: every sound belongs to one of six volume groups (below). The
  option sliders, ducking under speech and pausing all work on groups.
- **Loudness** is an amplitude factor from 0 (silent) to 1 (as recorded):
  0.5 means half the wave height (about −6 dB).

## Purpose

All sound effects, music and speech go through one engine (`KQSound`) that
reads Sierra "SOL" audio files (`.aud`), mixes them through DirectSound,
and handles loudness groups, a limit on how many sounds mix at once,
placed sounds, fades and looping. Effects come from animation cues
(doors, chests, attacks, Connor's moves), from scripts, from items, from
footsteps, from hurt sounds, and from looping sound objects placed in the
world (Daventry's waterfall, mill, windmill and fountain).

## The `.aud` file (measured on all files, and traced)

### What the files are (measured)

1. `MASK/` holds 1,750 files named `.aud` (counting each world's loose
   `sound` folder and `GAME/SOUND`; `MASK/data/` has copies). **1,749 are
   SOL files, 1 is a WAV file** (`GAME/SOUND/hm_iced.aud`, PCM, 16-bit,
   mono, 22,050 Hz). Speech uses the same SOL format but other names:
   2,849 files in the `English` folders (`A0RS0000.021` and so on).
2. The header is 14 bytes, little-endian:

   | Bytes | Value in every file | Meaning |
   | --- | --- | --- |
   | 0 | `0x0D` | Sierra resource type (audio) |
   | 1 | `0x0C` | length of the rest of the header |
   | 2–5 | `SOL` then a 0 byte | tag |
   | 6–7 | 22,050 (1,745 `.aud`, 2,839 speech) or 44,100 (4 `.aud`, 10 speech) | samples per second |
   | 8 | `0x0D` (1,657 `.aud`, all speech) or `0x1D` (92 `.aud`) | format bits |
   | 9–12 | data length in bytes | |
   | 13 | 0 | unused |

   Header length + data length = file length in **every** file (no
   exceptions, no trailing bytes).
3. Format bits: 0x01 = compressed, 0x04 = 16-bit, 0x08 = signed, 0x10 =
   stereo. So every file is **compressed 16-bit**; `0x1D` files are
   stereo. There are **no uncompressed SOL files** in the game.
4. Which files are stereo (`0x1D`): music (`M*.AUD`, `THEME.AUD`,
   `mcastle.aud`, `mhall.aud`, ...) and the long background beds
   (`AMB*.AUD`, Daventry's `FOREST.AUD` 178.5 s, IceWorld's `DilS.aud`
   and `nature.aud`). All sound effects and all speech are mono.
5. 44,100 Hz files: `barren/sound/GONGTURN.AUD` (mono) and
   `temple1/sound/mhall.aud` (stereo), plus 5 IceWorld speech files.
   **Daventry uses none.**
6. Lengths: shortest `GAME/SOUND/NONE.AUD` (8 samples, effectively
   silence), longest Dead City's `AMBDEAD.AUD` (187.6 s). All `.aud`
   together: 3 h 6 min; speech: 3 h 32 min.

### How the game decodes it (traced)

7. **Compressed data** is one byte per sample (per channel; stereo files
   alternate left byte, right byte). Each byte changes the previous
   sample value: the low 7 bits pick a step size from a fixed table of
   128 steps; bit 0x80 set means subtract the step, clear means add it.
   Each channel starts from 0 before the first byte. The step table is
   exactly the one in the community note (0, 8, 16, 32, 48, ... 0x3000,
   0x4000; checked entry by entry against the game's own table).
8. The sum is kept as a 16-bit value and **wraps round** on overflow (no
   clipping). Measured: decoding all 2,498 compressed files (effects,
   music, speech) **never overflows** (largest peak exactly 32,768 in
   size), so wrap or clip makes no difference. Use either.
9. Number of samples = data length ÷ channels. (For uncompressed SOL data,
   which no file uses, the game reads signed 16-bit samples, length ÷ (2
   × channels).)
10. **The game ignores header byte 1** and always starts the data at byte
    14. The 16-bit and signed bits are only checked when the file is
    opened (rule 12); the decoder always produces signed 16-bit samples.
11. The decoder keeps its running value between reads and resets it to 0
    only when reading restarts at sample 0 (so a loop decodes correctly;
    jumping into the middle of a file is never done).

### When a file is accepted (traced)

12. A file is taken as SOL when bytes 0–1 are `0D 0C`, bytes 2–5 are
    `SOL\0`, the format byte has no bits outside 0x01/0x04/0x08/0x10, and
    at least one of 0x04 (16-bit) or 0x08 (signed) is set. Otherwise it
    is tried as WAV: `RIFF` … `WAVE`, a `fmt ` chunk of at least 14 bytes,
    format tag 1 (plain PCM), 8-bit (unsigned, silence 0x80) or 16-bit
    samples, 1 or 2 channels, and a `data` chunk. Anything else is
    refused with a log line ("format of %s is unrecognized") and the
    sound plays nothing.
13. **Name lookup:** the game first tries the name exactly as given; if
    that file isn't found it replaces the extension with `.AUD` and
    tries again, then with `.WAV`. The search uses the normal file search
    list (`models-and-textures.md` "Where texture names are looked up":
    for Daventry, `daventry/sound` and the other `daventry` folders first,
    then `GAME/SOUND` and the other `GAME` folders). A few engine sounds
    are named with a folder (`daventry\sound\c_drwndi.aud`) and are
    looked up from the game folder.
14. Data names that only work through rule 13 (Daventry and `GAME`):
    `raven.snd` (`ravensrh.cs`, finds `RAVEN.AUD`), `sprgouch.wav`
    (`sprgnbow.kq`), `conrouch.wav` (`required.anm`), `intrface.wav`
    (`Graphics.kq`, `credits.KQ`), and the footstep names, which have no
    extension at all (`dirt1` → `DIRT1.AUD`, rule 38).

### The community note: right, wrong, incomplete

15. **Right:** the header layout and field meanings, 22,050 Hz "mostly,
    some 44,100", the format bits and the rule that only 0x01/0x04/0x08/
    0x10 may be set, the step table, the add/subtract rule, the starting
    value 0 per channel, the 16-bit wrap without clipping, stereo meaning
    music, the `.AUD`/`.WAV` retry, and `hm_iced.aud` being the only WAV.
16. **Wrong or misleading:** "16-bit expected to be set, required if
    signed is not present" is only the acceptance test; after that the
    game never looks at 0x04/0x08 and treats every file as signed 16-bit.
    The note's uncompressed case "resDataSize / channelCount / 2" is
    right, but no such file exists. The retry order is: name as given,
    then `.AUD`, then `.WAV` (the note doesn't give an order).
17. **Incomplete:** the note doesn't say that byte 1 is ignored (data
    always at byte 14), that the length always matches the file, that no
    file overflows, or which files are stereo. The WAV warning ("`fmt `
    chunk larger than 0x18 overflows a buffer") is correct in spirit; the
    game copies the whole chunk into an 18-byte format record. No game
    file is affected (`hm_iced.aud` has a 16-byte chunk). Do not
    reproduce: read only the fields you need.

## How a sound is played (traced)

### The output

18. The game mixes into a **stereo, 16-bit, 22,050 Hz** output. Each sound
    plays at its own file rate; 44,100 Hz files are converted by the mixer
    (Windows did this, not the game). Any modern mixer rate is fine.
19. Sounds up to **1 second long** are loaded whole; longer ones are
    streamed (decoded a little at a time into a 1-second buffer). This
    makes no audible difference; B can load everything whole or stream,
    as convenient.
20. A housekeeping timer runs **every 200 ms of real time** (not game
    time): it moves fades on (rule 33), tops up streamed sounds, and
    removes one-shot sounds that have finished. So a finished sound may
    count as "playing" for up to 0.2 s after it ends (rule 26).

### Loudness and balance

21. A sound's final loudness = its **own loudness** × its **group's
    loudness**, capped at 1. Own loudness is 1 for plain sounds and the
    distance factor (rule 29) for placed sounds; nothing in the data or
    scripts sets a per-sound loudness. Group loudness = the slider value
    × the duck factor (rules 50–53).
22. The game hands loudness to DirectSound as 20 × log10(loudness) dB, so
    the amplitude factor is exactly the loudness (to within 0.01 dB). 0
    or less is silence (−100 dB).
23. **Balance (pan)** is a number from −1 (left) to +1 (right). For a
    value p > 0 the left channel is multiplied by (1 − p) and the right
    stays at full; for p < 0 the right channel is multiplied by (1 + p).
    Plain sounds have p = 0. (Again exact, through DirectSound's dB
    scale.)

### Starting, stopping, looping

24. **One-shot** sounds (everything except looping objects and music)
    play once and are thrown away when they end or are stopped.
25. **Looping** sounds start over at the end with no gap; the decoder
    restarts cleanly (rule 11). A loop stops only when its owner stops
    it: the sound object is deleted or switched off (rules 46–48), or
    the music system fades it out (rule 57).
26. **How many at once (the mix limit):** the limit is **8** (script
    command `KQSound::setMixLimit <n>` changes it; no script in the game
    uses it). When a sound starts and **more than 8** sounds are already
    in the "playing" list, the game stops one first: among the playing
    sounds that **do not loop**, the one in the **lowest group** (group 0
    before group 1 and so on); if several tie, the **one that started
    first**. The new sound then starts. So up to 9 can be heard, and the
    new sound always wins, even over a sound of a higher group. If every
    playing sound loops, nothing is stopped and the new sound is simply
    added (no upper limit then).
27. What counts as "playing" for rule 26: every started sound that has not
    been stopped or paused, **including** placed sounds that are silent
    because Connor is beyond their reach but that were never heard (rule
    30), and failed sounds ("Do not reproduce" 2). Paused sounds (rules
    30, 54) don't count.

### Placed sounds: distance and balance

28. **Distance is measured from Connor** (his position; the same point the
    near/far test uses, `object-lighting.md` rule 34), in world units, in
    3D (height counts). The sound's position is **copied when the sound
    starts and never updated**: a one-shot played by a moving object stays
    where it started; a placed sound object does not follow its object if
    the object moves (none of Daventry's do).
29. **The distance factor**, with the sound's two distances min and max
    (D = distance from Connor):
    - D < min: factor **1**;
    - min ≤ D ≤ max: factor **1 − D ÷ (max − min)** (not D − min: see "Do
      not reproduce"), and 0 when that comes out negative;
    - D > max: factor 0 (rule 30).
    So the sound **drops suddenly** at min (from 1 to 1 − min ÷ (max −
    min)) and is **already silent at D = max − min**, before max.
30. Beyond max: a one-shot is stopped and thrown away at once; a looping
    sound is **paused** (it keeps its place in the file) and **resumes**
    from there when Connor comes back within max. Exception: a looping
    sound that was beyond max **when it started** is not paused; it plays
    on silently (and counts in rule 27) until Connor first comes in range.
31. **Balance comes from the camera**, not from Connor: the sound's
    position is taken into the camera's own axes; with s = the sideways
    and f = the forward-back offset (height ignored), p = **0.82 × s ÷
    √(s² + f²)**. So a sound exactly beside the camera gives |p| = 0.82
    (the far ear at 0.18 of full), one straight ahead or behind gives 0,
    and distance doesn't change the balance. **A sound on the screen's
    left plays from the left speaker, on the right from the right**
    (checked by the user in the original, 2026-10-17; it follows the
    camera, not Connor's facing). The balance is worked out for
    every placed sound within max, including inside min.
32. Placed sounds are re-evaluated **every frame** (distance, loudness,
    balance), from Connor's and the camera's current positions.

### Fades

33. A fade moves a sound's own loudness towards a target at a rate in
    loudness per second, applied in 0.2 s steps by the timer (rule 20),
    and stops exactly at the target. A fade that reaches 0 stops the
    sound. Only the music system uses fades: rate **0.75 per second**, so
    0 → 1 takes 7 steps (1.4 s).

## Where sounds come from

### Animation cues (`playsound`)

34. A movement in an `.anm` file can carry cues: `cueN=<phase> <action>
    <text>` (Daventry's door: `cue0=0.0 playsound door4.aud`). The action
    name is matched ignoring capitals against `EndLoop`, `FootStep`, `Do`,
    `PlaySound`, `SendEvent`, `Interruptable`; **an unknown name counts as
    `EndLoop`** (none in the data). When and how often a cue fires is in
    `doors.md` / `emitters.md` (the cue-range rule).
35. A **`PlaySound`** cue plays the named file as a **placed one-shot** at
    the animated object's position at that moment: min **5,000**, max
    **25,000**, group 0. It works for every animated object (doors,
    chests, Connor, monsters, scenery). `daventry/kq` and `GAME/Kq` hold 1,182
    `playsound` cues.

### Footsteps (`footstep` cues)

36. A **`FootStep`** cue (2,061 in `daventry/kq` and `GAME/Kq`, nearly
    all in Connor's `.anm` files) makes a sound **only for characters** (Connor and monsters);
    on other objects it does nothing (Daventry's `urn.anm` has 6, all
    silent).
37. The step is a **plain** sound (not placed), group 0. Which file: the
    ground under the character gives material flags (the same flags as
    `water.md` "Ground flags from `envInfo.kq`" for the terrain; for a
    building, rule 39). The first flag that matches, in this order, picks
    a set:
    0x1 → `dirt`; 0x2 → `grass`; 0x4 → `rock`; 0x8, 0x100 or 0x40 →
    `water`; 0x20 → `weirdb`. A ground with none of these (for example
    only 0x10, or only 0x80 lava) makes **no step sound**.
38. Each set has three files, `<set>1`, `<set>2`, `<set>3` (`dirt1` …;
    found as `DIRT1.AUD` by rule 13), picked at random: **25 % file 1,
    50 % file 2, 25 % file 3**. `dirt`, `grass`, `rock`, `water` are in
    `GAME/SOUND`; `weirda`/`weirdb` only in `deadcity/sound`. The `weirda`
    set is never chosen (rule 37).
39. A building can set `footsoundType=<n>` in its `.kq`: standing on it
    gives the flag 2ⁿ (n from 0 to 8; anything else gives 0x1, dirt).
    Daventry: `bigbrig2.kq` (bridge) `footsoundType=2` → 0x4, rock steps;
    `cav2.kq` `footsoundType=4` → 0x10 → **silent steps**. A script can
    change it: `KQObject::setFootStepSound <object> <n>` (used only in
    temple1). What a building without the key gives was not traced.

### Scripts

40. `KQSound::play <file>` plays a **plain one-shot** in group 0 (group 1
    instead if group 0 is paused at that moment, so a script sound still
    plays while the game is paused). Daventry uses it 26 times (bell,
    crossbow traps, breaking boxes, eating, drinking, the poor box,
    `fishies.cs`, ...). The debug alias `god` also plays `god.aud`.

### Hurt sounds (`ouchSound`)

41. An object's `.kq` may name `ouchSound=<file>` (Daventry: goblins,
    henchman, spriggans, zombie, `milltree.kq` `woodchop.aud`, the rolling
    rock `prrock.kq`; Connor `GAME/Kq/Connor.kq` `conrow3.aud`). When the
    object is hit, the sound plays as a **placed one-shot** at the
    object (5,000 / 25,000, group 0), but **at most once per 1.0 unit of
    the game clock** (seconds as far as seen) and not for damage of kind 4
    (not traced what kind 4 is). A hit from the environment (poison
    ground and the like) on Connor plays no ouch sound; instead, when the
    game asks for it, a plain `poison<n>.aud`.

### Items

42. As in `items.md` rule 7: the item type's **pickup sound** is a plain
    one-shot (group 0) on every add to Connor's inventory; the **land
    sound** of a thrown item is a placed one-shot (5,000 / 25,000). All
    placed one-shots in the game use these same two distances.

### Sound objects placed in the world (`KQAudibleObject`)

43. A world object with `classType=KQAudibleObject` is an invisible, looping
    sound source. Its `.kq` keys:

    | Key | Meaning |
    | --- | --- |
    | `soundFileName` | the file |
    | `isSound3D` | 1 = placed sound, 0 = plain looping sound |
    | `minDistance`, `maxDistance` | rule 29's min and max |
    | `deltaX/Y/Z` | read but **never used** |
    | `locX/Y/Z` | the position (as any object) |

    The files say "; 250 units per foot".
44. Daventry's six, all `isSound3D=1`, loaded by `newworld.cs` (and
    `bricks.cs`):

    | File | Object | Sound | min | max | at min | silent from |
    | --- | --- | --- | --- | --- | --- | --- |
    | `millsnd.kq` | WaterwheelSound | `watrweel.aud` | 750 | 20,000 | 0.96 | 19,250 |
    | `millsnd2.kq` | WatermillSound | `millsnd.aud` | 750 | 6,000 | 0.86 | 5,250 |
    | `watrfsnd.kq` | WaterfallSound | `watrfall.aud` | 1,500 | 35,000 | 0.96 | ~27,100 (rule 47) |
    | `windsnd.kq` | WindmillSound | `windmill.aud` | 1,500 | 20,000 | 0.92 | 18,500 |
    | `fountsnd.kq` | FountainSound | `fount.aud` | 1,500 | 20,000 | 0.92 | 18,500 |
    | `watrfsn2.kq` | Waterfall2Sound | `watrfal2.aud` | 1,500 | 30,000 | 0.95 | ~27,100 (rule 47) |

    ("at min" = the loudness just past min, from rule 29.) The waterfall
    and fountain and windmill have `deltaZ=1000`, which does nothing.
45. The sound is a looping placed sound in group 0 at the object's
    position.
46. It plays only while its object is **active**, that is near or far in
    the near/far/off test (`object-lighting.md` rules 34–36; hidden
    objects are off). Going off **stops** it; becoming active again
    **restarts it from the beginning**. When the object first enters the
    world it starts at once if active (partly traced).
47. For that test the object has no shape, so it is a ball of radius 100 at
    its position (`object-lighting.md` rule 35). Daventry's
    `visibleDistance` is 27,000, so the sound is **switched off beyond
    27,100 from Connor**, whatever its `maxDistance`. For the two
    waterfalls this cuts the sound off while it would still be heard
    (loudness about 0.19 and 0.05 there).
48. `deleteObject <name>` deletes the object and its sound stops at once.
    Daventry's `bricks.cs` does this when the waterfall is blocked
    (`deleteObject WaterfallSound`, then `loadKQ watrfsn2.kq`).

### Doors, chests and other animated objects

49. Doors, chests, jars, the mill and other animated objects make their
    sounds through `playsound` cues in their movements (rule 35). The
    viewer already logs these; they only need to be played as rule 35
    says.

## Volume settings

50. Six groups. What each holds, which slider sets it, whether speech
    ducks it, whether the game's pause pauses it:

    | Group | Used for | Slider (command) | Ducked by speech | Paused |
    | --- | --- | --- | --- | --- |
    | 0 | all world sound effects (rules 35–48) | sound effects (`setSfxVol`) | yes | yes |
    | 1 | menu / screen sounds (clicks, `moveSound`, `selectSound`, `intrface.aud`, ...) | sound effects | no | no |
    | 2 | (nothing found that uses it) | sound effects | yes | yes |
    | 3 | speech | speech (`setSpeechVol`) | no | yes |
    | 4 | menu theme music (`themeMusic=theme.aud`) | music (`setMusicVol`) | no | no |
    | 5 | world music (`KQMusic`, rule 57) | music | yes | yes |

51. **Script commands:** `KQSound::setSpeechVol <v>`, `KQSound::setSfxVol
    <v>`, `KQSound::setMusicVol <v>`, v from 0 to 1, set the slider value
    of their groups and change all sounds of those groups at once,
    including ones already playing. They appear only in `MASK/Options.cs`,
    which the game runs at start and **rewrites** (with the current values)
    when the options are saved.
52. **The option sliders** give slider position × 0.01, and a position from
    1 to 9 is raised to 10. So a slider at 0 is mute, otherwise loudness
    0.10 to 1.00. **Shipped values** (`MASK/Options.cs`): speech **0.34**
    (−9.4 dB), sound effects **0.14** (−17.1 dB), music **0.14**
    (−17.1 dB), i.e. sliders at 34, 14, 14. Before `Options.cs` runs all
    groups are at 1.
53. **Ducking:** while a line of speech plays, groups 0, 2 and 5 are
    multiplied by **0.2** (−14 dB); when it ends, by 1 again. So with the
    shipped settings a sound effect under speech plays at 0.14 × 0.2 =
    0.028. (Some lines skip the ducking; what decides that was not
    traced.)
54. **Pause:** when the game pauses, groups 0, 2, 3 and 5 are paused (each
    sound keeps its place) and resume afterwards; groups 1 and 4 keep
    playing. (Which screens pause the game was not traced.)
55. **World change:** while a new world loads, the slider values of groups
    0, 1, 2 (effects) and 3 (speech) are set to 0 and restored on the new world's first frame;
    music keeps playing.
56. `KQSound::enable 0|1` switches the whole sound system off or on (not
    used by any script). `KQSound::stats` prints a debug list.

## Music and speech (enough to see what they share)

57. **Music** (`KQMusic`): each world's `kq/music.kq` (`[Music]`,
    `classType=KQMusic`, `nMusic`, `musicN=`; Daventry `music0=forest.aud`)
    names its background track; scripts use `KQMusic::push <file>` (a
    tune on top: the current one fades out at 0.75/s and stops, the new
    one loops and fades in from 0) and `KQMusic::pop <file>` (remove it;
    if it was on top it fades out and the one below fades in again, from
    its beginning). `KQMusic::play` replaces the top. Daventry pushes and
    pops `mbattle.aud`, `mchurch.aud`, `municorn.aud`, `mwizard.aud`.
    Music files are stereo SOL `.aud`, played by the same engine as
    looping plain sounds in group 5, at the music slider × duck factor.
    Not studied further.
58. **Speech** files are the SOL files in the `English` folders, played
    by the same engine in group 3. Connor's own lines are plain; other
    characters' lines are **placed** at the speaker's position when the
    line starts, min **2,000**, max **30,000**. Speech ducks groups 0, 2
    and 5 (rule 53). Which file goes with which line (`.msg`, `.lip`) was
    not studied.

## Numbers and units

| Thing | Value | Status |
| --- | --- | --- |
| File header | 14 bytes; data always at byte 14 | measured + traced |
| Sample rates | 22,050 Hz (all Daventry files), 44,100 Hz (6 files elsewhere) | measured |
| Output | stereo, 16-bit, 22,050 Hz | traced |
| Whole vs streamed | ≤ 1 s whole, longer streamed (no audible difference) | traced |
| Housekeeping step | 200 ms real time | traced |
| Mix limit | 8 (a 9th may start; rule 26) | traced |
| Placed one-shots (cues, hurt, item land) | min 5,000, max 25,000 (silent from 20,000) | traced |
| Other characters' speech | min 2,000, max 30,000 | traced |
| Distance factor | 1 inside min; 1 − D ÷ (max − min) after | traced |
| Balance | 0.82 × sideways ÷ horizontal distance, camera axes | traced (sign open) |
| Hurt sound repeat | at most once per 1.0 game-clock unit | traced |
| Footstep choice | 25 % / 50 % / 25 % | traced |
| Fade rate (music) | 0.75 per second, 0.2 s steps | traced |
| Duck factor | 0.2 on groups 0, 2, 5 | traced |
| Shipped sliders | speech 0.34, effects 0.14, music 0.14 | data |
| Units | world units; "250 units per foot" (the `.kq` comment) | data |

## Data used

- `MASK/<world>/sound/*.aud`, `MASK/GAME/SOUND/*.aud`, `MASK/<world>/English/*` (speech).
- `.anm` cues: `playsound`, `footstep` (`GAME/Kq/*.anm`, `daventry/kq/*.anm`).
- `.kq` keys: `classType=KQAudibleObject` with `soundFileName`, `isSound3D`,
  `minDistance`, `maxDistance`, `deltaX/Y/Z`; `ouchSound`; `footsoundType`
  (buildings); `themeMusic`, `moveSound`, `selectSound` (menu screens);
  `[Music]` `music.kq`.
- Scripts: `KQSound::play`, `KQMusic::push/pop/play`,
  `KQObject::setFootStepSound`, `deleteObject`.
- `GAME/Kq/Invinfo.itl` land / pickup / use sounds (`items.md`).
- `kq/envInfo.kq` ground flags (`water.md`), `kq/Terrain/TERRAIN.KQ`
  `visibleDistance`.
- `MASK/Options.cs` (volumes).

## Edge cases

- **Missing file:** a log line ("'%s' not found") and no sound. But see
  "Do not reproduce": in the original the failed sound still sits in the
  "playing" list. Missing names, all game data: only `poffup.aud`
  (`Concbow.anm`). Missing **in Daventry's search list** but present in
  Dead City (so silent if they ever played in Daventry): `board.aud`,
  `feather.aud`, `mold.aud`, `skelhead.aud` (item sounds),
  `hm_die.aud` (in Swamp; Connor's weapon `.anm` files).
- A name with the wrong extension still plays if a `.AUD` or `.WAV` of the
  same name exists (rule 14).
- A broken file (not SOL, not plain-PCM WAV) plays nothing and is logged.
- Footstep cues on objects that are not characters do nothing; grounds
  with no matching flag make no step.
- Two placed sounds at the same place are simply both heard; there is no
  merging.
- A placed sound object deleted by a script stops at once, mid-sound.
- `KQSound::setMixLimit` with a non-number gives 0 (then every new sound
  stops one older non-looping sound first).

## Do not reproduce

1. **The distance factor ignores min** (1 − D ÷ (max − min) instead of
   1 − (D − min) ÷ (max − min)). It makes every placed sound jump down at
   min (a one-shot from 1 to 0.75 at 5,000) and go silent before max.
   This is what players heard, so it defines how loud things are in
   practice. **Decided by the user (2026-10-17): keep the original
   formula.** A switch to the corrected curve may be added later as an
   option; it is not part of the first version.
2. **Failed sounds count as playing**: a sound whose file is missing or
   unreadable is still put in the playing list, makes no sound, and only
   leaves it when the mix limit throws it out (or, if placed, when Connor
   walks beyond its max). Several of them can make real sounds get cut
   early. Instead: skip a failed sound entirely (log it once).
3. **Silent looping sounds count towards the mix limit** when they started
   out of range (rule 30). Low impact; B may count only sounds that are
   actually audible, or copy the original. Recommendation: copy it only if
   it is easy; nobody will hear the difference except when many sounds
   play at once near a looping source.
4. **Balance when the sound is exactly above or below the camera**:
   sideways and forward offsets both 0 gives a division by zero (in the
   original probably full right). Use centre (p = 0).
5. **`weirda` footsteps are never chosen** and `footsoundType=4` floors
   are silent. Looks like a bug (flag 0x10 is missing from the order in
   rule 37), but in Daventry it only affects `cav2.kq`, and Daventry has
   no `weirda` files anyway. Keep the original (silent) unless the user
   asks.
6. **The WAV header copy** (rule 17): read only the fields needed.
7. `deltaX/Y/Z` in sound objects do nothing; don't invent a meaning.

## How to check

- **Decoder (data):** decode every SOL file: sample count = data length ÷
  channels (e.g. `daventry/sound/AXEDOOR.AUD`: 11,847 samples, 0.537 s;
  `FOREST.AUD`: stereo, 178.5 s), and no file should ever overflow 16
  bits (0 of 2,498). Every file's header + data length = file length.
- **File lookup:** `raven.snd` and `sprgouch.wav` must resolve to the
  `.AUD` files; `poffup.aud` must log "not found" once.
- **Loudness, by log:** for each placed sound log Connor's distance and
  the loudness. Expected for a `playsound` cue: 1.0 at 4,999, 0.75 at
  5,000, 0.5 at 10,000, 0 at 20,000 and beyond (stopped beyond 25,000).
  Fountain: 0.919 at 1,500, 0.5 at 9,250, 0 from 18,500.
- **Group loudness, by log:** with the shipped `Options.cs`, a door sound
  next to Connor = 0.14; the same during a speech line = 0.028; a speech
  line = 0.34.
- **Mix limit, by log:** start 12 one-shots at once: the log should show
  the first three stopped as the 10th, 11th and 12th start (9 playing at
  the end); with Daventry's looping sources in the list they count too.
- **In the original, by ear (the user):** stand near the waterfall and walk
  away: the roar should cut off suddenly about 27,000 units away (rule
  47). Open a door from about 6,000 away: the creak is clearly quieter
  than from right next to it (0.7 against 1.0). Turn the camera so a
  sound source is on the screen's left and listen which speaker is
  louder: the left one (checked by the user, 2026-10-17).

## Open questions

1. ~~Which way is right?~~ **Answered by the user (2026-10-17):** screen
   left → left speaker, relative to the camera (rule 31).
2. ~~Keep the original distance curve or fix it?~~ **Decided by the user
   (2026-10-17):** keep the original; a fix may come later as an option
   ("Do not reproduce" 1).
3. Not traced: what makes a speech line skip ducking (rule 53); which
   screens pause the game (rule 54); what a building without
   `footsoundType` gives (rule 39); what damage kind 4 is (rule 41); eight
   other engine places that play placed one-shots with the same 5,000 /
   25,000 (likely weapon swings, projectiles, landings; same rules apply).
4. Music and speech were not studied beyond what is written here.
