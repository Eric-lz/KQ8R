# Health items and potions

Study session A, 2026-10-20. Sources: `Mask.exe` (studied, described in
our own words), `GAME/Kq/Invinfo.itl`, `GAME/Kq/Connor.kq`,
`GAME/Kq/orbit1.kq`, the world scripts (`daventry/kq/keepwall.cs`,
`wizchest.cs`, `barren/kq/gong*.cs`, `temple4/kq/archon.cs`,
`deadcity/kq/skingtp.cs`) and the texture lists in the worlds' `8bit.vol`.
Builds on `inventory.md` rules 9–18 (clicking a slot, applying an item)
and `interface.md` rules 20–33 (meters, slot numbers). Marks as in
`interface.md`: **[traced]**, **[data]**, **[manual]**, **[inferred]**,
**[not traced]**.

## Purpose

The eight `HealthItem` types fill the eight slots of the bottom bar.
Four heal Connor once (mushroom, crystal, sacred water, elixir of life);
four are potions that change him for a number of seconds: **Clarity**
(reveal hidden things), **Invisible**, **Invulnerable** (shield) and
**Strength**. While a potion runs, a timer icon drains in the status
icons, and for the shield and strength potions the bar's number turns
red.

## Rules

### Using an item

1. A left click on one of the eight slots, or keys **1–8** (and the
   number pad's 1–8), uses the item in that slot (`inventory.md`
   rules 9, 11). There is no "held" stage: the item is used at once,
   and anything that was on the pointer goes back to its slot
   [traced].
2. **Refused** (nothing happens, no sound, no remark) only during
   `handsOff` and while Connor is dying or dead [traced]. Everything else
   is allowed: walking, running, **jumping and falling, talking**, on
   the rope, swimming, fighting, first person [traced: these are the
   only checks on the way].
3. Connor plays **no movement** for it [traced: nothing asks his model
   for one].
4. The item's **use sound** (`useSnd` in `Invinfo.itl`) plays, not placed
   in 3D (same volume wherever Connor is), **only when the item takes
   effect** [traced]: `healing.aud` (all four health items),
   `clarity.aud`, `invisibl.aud`, `invuln.aud`, `strength.aud` [data].
5. **No remark ever** [traced]: not at full health, not when a potion of
   that kind is already running.
6. **Full health**: a health item is used anyway; one is taken from the
   stack and the sound plays; health stays at the maximum [traced].
7. **The same potion again while it runs**: nothing at all happens: no
   sound, the count does not drop, the timer is **not** restarted or
   lengthened [traced: the stack is one item object, already "applied";
   applying it again is ignored (`inventory.md` rule 14, "only if it isn't
   applied already")]. **Different potions** run at the same time, each
   with its own timer [traced].
8. Taking one from the stack (`inventory.md` rule 14): the count drops by
   1 **when the dose starts**. When the **last** dose of a potion is
   used, the item stays in the inventory with count 0 until its time
   is up, and only then leaves [traced]. A slot whose item has count 0
   is drawn **empty** (no picture, no number) [traced], so the slot
   looks empty while the last dose runs. Picking up the same potion in
   that time adds to that item (`inventory.md` rule 8): the slot shows
   the new count, and the potion running is not touched [inferred from
   the merge rule].

### Health items

9. Each heals once by its `Hlth` [data]: **Mushroom 5, Crystal 50,
   Sacred Water 100, Elixir of Life 1,000** (`Dur` −1, so the heal is
   never taken back; `inventory.md` rule 14).
10. Health is **capped at 10 × level** (`interface.md` rule 29) [traced].
    The Elixir always fills him up.
11. The heal reaches Connor as the normal stats event, a moment after
    the click (queued, handled in the same frame, `interface.md`
    rule 24) [traced]. So the meters show it as any stats change does:
    the red bar's new width, and if the bottom bar is hidden, the meters
    **pop up for 1.5 s** (`interface.md` rule 25) [inferred from those
    rules: every stats event sends the health percent].

### Potions: common rules

12. A potion's stats in `Invinfo.itl` are all 0 [data]; its effect is
    built into Connor, chosen by the item's ID [traced]. The stats
    event still runs at the start and at the end (with zeros), so the
    panel numbers and meters are refreshed then, and the meters pop up
    as in rule 11 [inferred].
13. At the start, in this order [traced]: the effect switches on
    (rules 17–29), the count drops by 1 (rule 8), the stats event is
    queued, the timer starts with `Dur` seconds (rule 31), the timer
    icon appears (rule 35). At the end: the stats event is queued, the
    effect switches off, the item leaves if its count is 0.
14. Durations [data]: **Clarity 60 s, Invisible 15 s, Invulnerable
    20 s, Strength 20 s**.
15. Nothing ends a potion early: not attacking, not being hit, not
    talking, not a cutscene, not a world change (rule 34) [traced: only
    the timer switches the effects off].
16. **Connor's face** changes with Clarity and Strength (rule 30); his
    body with Strength (rule 25) and Invisible (rule 21).

### Invulnerable (shield)

17. **Protection** (the armour number, stat 4) is raised to **its value
    + half its value** (half rounded down), but **at least 50 and at
    most 100** [traced]. Examples: 0 → 50, 10 → 50, 34 → 51, 40 → 60,
    66 → 99, 80 → 100. The manual's "increases to 50% or by 50%" is this
    [manual].
18. Hits use protection as usual (`fighting.md` rule 16), so the shield
    simply lowers the damage of every hit; there is no separate block
    [traced]. At 100 Connor takes **no damage at all** (`fighting.md`
    rule 16) [traced].
19. **In the last temple**, after the script command `setConnorFlag
    InTemple4 1` (`temple4/kq/archon.cs`, when Connor has the fifth mask
    piece, the grail, the temple armour and the temple sword), each hit
    uses only half the shield's raise, at most 85: protection for that
    hit = old value + (raised value − old value) / 2 (rounded down), cut
    to 85 [traced]. The number shown does not change.
20. **The orbiting shield**: an object `orbit1` (file `GAME/Kq/orbit1.kq`,
    class `KQOrbitObj`, model `invulnfx.kqs`, see-through 165 of 256, no
    shadow) circles Connor while the potion runs [traced, data]:
    - its centre is Connor's model part `body`, lowered by 820 and then
      raised by `orbitZoffset` 500 (so 320 below the `body` part);
    - radius `orbitRadius` 500; it turns by `orbitStepAng` z 0.2 rad
      **every update** (15 a second: 3 rad, about 172°, a second) [traced
      the step per update; the update rate inferred from the other
      objects];
    - the game also tries `orbit2.kq` and `orbit3.kq`; no such files
      exist, so there is one [traced, data];
    - it removes itself when the shield ends.
    At the end, protection goes back to **the value it had just before
    the potion** (cut to 100) [traced], see "Do not reproduce" 1.

### Strength

21. The **hand-weapon number** (stat 2) is raised by **half of itself,
    rounded down**; that half is remembered and taken off again at the
    end [traced]. Example: 25 → 37, back to 25. The **ranged number is
    not changed** [traced]. The manual's "hand weapon stat to 150%"
    [manual].
22. The raise is fixed at the start: a weapon picked up or a level-up
    during the potion changes the number as usual, and the end takes off
    the same remembered half [traced].
23. A hit's damage comes from the hand-weapon number (`fighting.md`),
    so Strength raises every melee hit by that much [inferred: the
    fight uses the stat itself].
24. The bar's hand-weapon number turns red while it runs (rule 37).
25. **Connor looks "iron"**: his body skin becomes the skin list entry
    for `INVITEM_Strength` (bit 2048 of `ConnerSkinList`): **`coniron.bmp`**
    in third person, **`armconfx.bmp`** for the first-person arms
    [traced, data]. The armour he wears does not show while it runs
    [traced: the strength entry replaces the armour combination]. When
    he wears the temple armour (`con65.bmp`) in third person, that
    texture is replaced by `coniron.bmp` too [traced]. At the end his
    normal skin comes back [traced].

### Invisible

26. **Characters stop seeing him** [traced, with `character-movement.md`
    rules 20–28]: characters know where Connor is from the reports of his
    position each time he moves (and at a few other moments). While the
    potion runs, **every such report puts him 100,000 units below where
    he really is**. So:
    - the sight line (to 1,250 above that point) goes deep under the
      ground and is blocked: nobody can see him (rule 20); a character
      with `canAlwaysSeeConner=1` skips that test but still finds him too
      far away (next point);
    - the distance to him is over 100,000, beyond any alarm radius (rule
      23) and leash (rule 27): characters already chasing him lose sight
      after 3 s (rule 26) or give up at the leash, and go home;
    - a report is also sent the moment the potion starts and the moment
      it ends, so the change is immediate [traced].
    The same lowering already happens **during cutscenes** (`handsOff`
    with the pointer hidden), **while Connor talks**, and **while he is on
    the rope** [traced]; Invisible adds a fourth case.
27. Attacking does **not** end it [traced: rule 15]. A character he hits
    is alarmed by the hit (`fighting.md`), but then looks for him at the
    false place [inferred from rule 26]. A character in its "hit" state
    with a hurt line set says one of those lines when he hits it while
    invisible, at most one such line every 20 s [traced in one character
    state; which characters use it not traced].
28. Scripts can ask: `KQConner::checkInvisible` sets `Console::Return`
    to 1 while it runs, else 0 [traced]. Used once: Dead City's skeleton
    king says a different line (`skingtp.cs`) [data].
29. **Connor is drawn see-through** [traced]: each of his textures is
    swapped for a "ghost" version for as long as the potion runs:
    - if a texture with the same name but the first two letters replaced
      by `I_` exists, that one is used (the worlds' `8bit.vol` have them
      only for weapons and a few items: `I_apons`, `I_dsword`, `I_ossbow`,
      `I_dagger`, `I_ield`, ... [data]);
    - otherwise (his body, face) a copy of the texture is made in which
      **3 of every 4 pixels are set to colour 0**, a regular mesh, and
      colour 0 is drawn as a hole [traced the pixel pattern; that colour
      0 is see-through inferred from the material switch set with it];
    - the mesh is roughly one kept pixel every 4 along a row, shifted by
      2 pixels on each following row (a diagonal grid) [traced];
    - his textures are converted again whenever his skin or face changes
      while it runs (armour, Strength, Clarity, talking mouth pictures)
      [traced];
    - at the end the original textures come back.

### Clarity (reveal)

30. **Connor's face** changes: the face list `ConnerLipsList` gets the
    Clarity bit (4): face pictures `cclf0`–`cclf7` instead of `cf0`–`cf7`
    (with the helmet `cclhf`, with the temple armour `cclmhf`; with
    Strength too `cclif`) [traced, data]. Strength sets bit 8 the same
    way (`cironf`).
31. **Reveal**: at the start, every object whose `groupsAlways` has
    **bit 2048** ("reveal" in the data's comments) gets its script run
    with `$2` = `Reveal` and `$3` = `True`; at the end the same with
    `$3` = `False` [traced, data]. What becomes visible is entirely the
    script's work. In Daventry:
    - `keepwall` (`keepwall.cs`): `KQObject::dither keepwall 1`, the
      wall becomes see-through, and 0 at the end;
    - `wizchest1` (`wizchest.cs`, a hidden chest with `hidden=1`,
      `dithered=1`): `setHidden wizchest1 0` and `collisionType ... 32`,
      so it appears and can be clicked and opened; at the end it is
      hidden again with collision type −32 [data].
    Other worlds: the Barren gongs change colour and set variables, Dead
    City's fake door and tile, Ice World's snow mane, Temple 1's bridge
    and beast [data].
32. Objects entering the world while Clarity runs do not get a `Reveal`
    [inferred: the call is made only at the start and the end].

### The timer

33. Each running potion counts **whole seconds**: it starts at `Dur`, and
    once a second of game time it drops by 1; when it would go below 1
    the potion ends [traced]. So a potion lasts exactly `Dur` seconds of
    game time.
34. **Game time** (answers Q109 (d)) [traced]: the same clock as all
    timed events, which moves with the frames (each frame adds the real
    time since the last one, but at most 1 s).
    - It **stops while the main menu is open** (the game is paused) and
      does not catch up afterwards.
    - It **does not stop** for `handsOff`, cutscenes or conversations.
    - It **keeps running when the game window loses focus** (the user's
      check in the original, 2026-10-20). Only the main menu (Esc) stops
      it. (A frame check in the program looked like it could stop the
      game when the window is inactive; in practice it doesn't.)
    - Potions **survive a world change and a saved game**: Connor's
      potion switches and the remembered raises are saved
      (`claritySpell`, `strengthSpell`, `invulnerabilitySpell`,
      `invisibilitySpell`, `strengthSpellPoints`, `armorBeforeSpell`,
      `altShort`, `altLong`, `altArmor`), and when Connor enters a world
      each running potion gets its timer icon back and its next 1 s tick
      [traced]. That the seconds left are saved with the item is
      inferred.
35. **The timer icon** [traced]: when a potion starts, its picture
    (`extraData` in `Invinfo.itl`: `IDBMA_CLAR`, `IDBMA_INV`,
    `IDBMA_VUL`, `IDBMA_STR` [data]) goes into the **first free place**
    of the status icons `IDCTL_TIMER` (7 places, `IDCTL_TIMER_1` to
    `_7`, laid out in the GUI file [data]).
    - It shows the part left: percent = seconds left × 100 / `Dur`
      (whole-number division); the picture is drawn in two parts split
      at that height, the lower part being the time left [traced].
      **Seen by the user in the original:** the icon appears whole, then
      is cut away **from the top down**; the used part shows only a
      **black background and a gold outline**. So the used part is the
      icon place's own background and frame, and only the lower part
      shows the picture.
    - It is redrawn only when the percent changes [traced].
    - When the percent drops below 1 (at the end) the place is freed
      and the icon disappears [traced].
    - An icon place with a picture of several frames and no duration
      (the spinning mask) animates at 10 frames a second instead; potion
      icons do not animate [traced].
36. **At the end**: the use sound does **not** play again; there is no
    remark and no sound of its own [traced: nothing is played on the
    way]. The effect switches off (rules 20, 22, 25, 29, 30, 31), the
    numbers are refreshed (rule 12).

### The red numbers on the bottom bar

37. While **Strength** runs, the hand-weapon slot's number is shown in
    the red font (`14red.pft`, `IDCTL_INVBAR_SHORT_ALTTXT`); while
    **Invulnerable** runs, the armour number is (`..._ARMOR_ALTTXT`)
    [traced]. The number is Connor's current (raised) stat, the same
    number as the white one [traced: the same value is written to
    both]. **The user sees only the red number** in the original, so
    the red text replaces the white one while it is on (whether the
    white one is hidden or just covered makes no visible difference).
38. They turn red with the stats refresh right after the start and back
    to white with the one right after the end (rule 13) [traced]. The
    number's change also plays the slot's flash (`interface.md`
    rule 33) [inferred: the number changes then].
39. The ranged slot has a red text and a switch too, but no potion sets
    it [traced]. It is saved (`altLong`) and so far unused.

### The candle (answers `inventory.md` rule 16)

40. The candle's 25 experience comes **when it is picked up**, not from
    its `Exp` column: picking up `INVITEM_Candle` gives +25 experience
    0.3 s later, built into the game for that item [traced]. (The same
    is done for other quest items, rule 41.) Clicking the candle in the
    top bar holds it (`inventory.md` rule 9); it is never "applied" by
    the bars, so its `Exp` 25 and `Dur` 60 s are never used that way and
    **no experience is taken away** after 60 s [traced: applying a quest
    item only happens on the inventory's own "use", which the bar click
    does not reach]. Correction of `inventory.md` rule 16 (inferred
    there).

### Other things the same code does (for completeness)

41. When items are **added**, Connor also [traced]:
    - the first time any health item or potion is added: asks for a help
      pop-up 3 s later (help 0x83; the help pop-ups are not studied);
      the first quest item: help 0x84;
    - gives built-in experience for some quest items, 0.3 s later:
      +25 for the ring of the dead hero, the ashes and the candle, +20
      for the lady bell, +10 for the rope and hook (and help 0x80 4.5 s
      later); at once: +1,000 the feather, +5,000 the green mushroom
      piece, +6,000 the mold, +10,000 the hearing horn, +25,000 the
      deciphering amulet, +50,000 the ladle [traced; names from the IDs
      in `Invinfo.itl`]. Each time the item is added, not only the first
      time [traced: no once-only check for these].

### Scripts and potions

42. [traced, data]
    - `give <potion> <n>` and chests add potions like any stacking item
      (`inventory.md` rule 8); nothing starts.
    - No script command starts or stops a potion, or reads its seconds.
    - `KQConner::checkInvisible` (rule 28); `setConnorFlag InTemple4`
      (rule 19).
    - `KQConner::setRelDamage x` only computes script values from Connor's
      level (`CurRelDamage` = −trunc(x × level × 10), `CurRelHealth` = its
      opposite) that trap scripts pass on [traced]; it does not touch
      potions. Damage sent that way goes through the normal hit, so the
      shield lowers it (rule 18) [inferred].
    - A script's `KQStatsChangeRequestEvent` changes stats directly
      (`clicking.md`); during a potion it is added on top of the raise,
      and the shield's end overwrites a protection change (Do not
      reproduce 1).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| Health items | Mushroom 5, Crystal 50, Sacred Water 100, Elixir 1,000 | data |
| Health cap | 10 × level | traced |
| Durations | Clarity 60 s, Invisible 15 s, Invulnerable 20 s, Strength 20 s | data |
| Timer step | 1 s of game time, whole seconds | traced |
| Shield | protection + ⌊p/2⌋, at least 50, at most 100 | traced |
| Shield in Temple 4 (`InTemple4`) | old + ⌊(raised − old)/2⌋, at most 85, per hit | traced |
| Orbit | radius 500, 0.2 rad per update, 320 below `body`, see-through 165 | data, traced |
| Strength | hand number + ⌊hand/2⌋, ranged unchanged | traced |
| Invisible | reported position 100,000 below | traced |
| Invisible texture mesh | 1 pixel kept in 4, diagonal | traced |
| Invisible: hurt line | at most one every 20 s | traced |
| Timer icon places | 7 (`IDCTL_TIMER_1`–`_7`), first free | traced, data |
| Candle | +25 experience 0.3 s after pickup | traced |
| Help on first health item | 3 s after | traced |

## Data used

- `GAME/Kq/Invinfo.itl`: rows 41–48 (`ID #` 3000–3070, `Hlth`, `Dur`,
  `useSnd`, `extraData`), `INVITEM_Candle`.
- `GAME/Kq/Connor.kq`: `ConnerSkinList` / `ConnerFpSkinList` (Strength
  bit 2048: `coniron.bmp`, `armconfx.bmp`), `ConnerLipsList` (Clarity
  bit 4, Strength bit 8).
- `GAME/Kq/orbit1.kq` (`KQOrbitObj`: `orbitRadius`, `orbitStartAng`,
  `orbitStepAng`, `firststep`, `orbitZoffset`), model `invulnfx.kqs`.
- The worlds' `8bit.vol`: `I_*` textures, `Cclf*`, `Cironf*`,
  `coniron.bmp`, `armconfx.bmp`.
- `GAME/Resource/kqGuiTag.TTAG`, `Gplayscr.gui`: `IDCTL_TIMER_1`–`_7`,
  `IDBMA_CLAR`, `IDBMA_INV`, `IDBMA_VUL`, `IDBMA_STR`.
- Objects with `groupsAlways` bit 2048 and their `Reveal` scripts.

## Edge cases

- Shield and Strength together: independent, both red.
- Strength + Clarity: face `cclif`. **Face combinations missing from
  `ConnerLipsList`** (helmet + Strength = 9, and 3, 7, 11, 13, 15): the
  game finds no entry and uses an invalid one [traced the failed
  lookup; what it then draws or whether it crashes not seen]. Do instead:
  rule "Do not reproduce" 3.
- A potion used during a conversation or a cutscene: works; Invisible
  changes nothing there (Connor's position is already hidden then).
- The last dose's slot looks empty while it runs (rule 8).
- Invisible ends during a chase: the next position report is the true
  one; characters notice him by the usual rules (`character-movement.md`
  rules 20, 23).

## Do not reproduce

1. **The shield's end overwrites protection** with the value from before
   the potion: armour put on or taken off, a level-up's +1, or a script's
   change during those 20 s are lost (or kept wrongly, if armour was
   taken off). Do instead: take off exactly what the shield added (like
   Strength does).
2. **Strength's end clears Invisible's drawing switch** when both run:
   the end of Strength resets a switch that Invisible set, so Connor may
   be drawn wrongly for the rest of the invisibility [traced the reset;
   the visible result not seen]. Do instead: each potion switches only
   its own things.
3. **Missing face combinations** (Edge cases): do instead: when the
   exact combination is missing, use the entry with the most of the
   wanted bits (e.g. helmet + Strength → `cironf`).
4. The timer icon keeps a reference to the potion even after the last
   dose's item is deleted, and guards against reading it with a range
   check. Do instead: remove the icon when the potion ends.
5. `Invinfo.itl`'s `Exp`/`Dur` of the candle are never used (rule 40):
   do not apply them.

## How to check

**Checks 2–6 were done by the user in the original on 2026-10-20: all
correct.** Check 7 (the candle in Daventry's church, `candle.kq` /
`candle.cs`): the experience bar rises a little on pickup and does not
drop (the exact +25 can't be read from the bar). The timer keeps
running when the window loses focus; only Esc stops it.

1. **Our logs (B)**: per use: slot, item, count before/after, refused
   (why), sound; per potion start/end: game time, stats before/after
   (protection, hand), seconds left each tick, icon place and percent;
   Invisible: the position each character is told, sight results;
   Clarity: each object told `Reveal True/False`.
2. **Daventry, in the original (user)**: find the potions (`INVITEM_strength`,
   `INVITEM_invulnerable` in `daventry/kq`, the Clarity from the wizard's
   star ball, `starball.cs`). Note the armour and hand numbers, drink the
   shield: armour becomes max(50, a + a/2) up to 100 in red, a shape
   (`invulnfx`) circles him, the icon drains over 20 s, the number goes back to
   white with the old value. Strength: hand number × 1.5 in red, Connor
   looks iron-grey, ranged number unchanged.
3. **Pause test (original)**: drink Strength, open the main menu (Esc)
   for 30 s, come back: the icon has not moved on and the potion still
   ends 20 s of play after it started.
4. **Same potion twice**: with 2 Strength potions, press 6 twice: the
   second press does nothing (no sound), the count stays at 1.
5. **Clarity in Daventry**: drink it near the keep wall: the wall turns
   see-through; near the wizard's tower the hidden chest (`wizchest1`)
   appears and can be opened; after 60 s both go back.
6. **Invisible**: let a bone goblin chase Connor, drink Invisible: it
   stops chasing within about 3 s and walks home; Connor looks like a
   mesh.
7. **Candle**: pick it up, the experience bar rises by 25 at once;
   wait more than 60 s: it does not drop.

## Open questions

1. ~~How the used part of a timer icon is drawn~~: answered by the
   user (rule 35).
2. ~~Whether the white number shows under the red one~~: only red is
   seen (rule 37).
3. The help pop-ups 0x80, 0x83, 0x84 (rule 41): which text box each is;
   belongs to a future study of the help pop-ups, not to potions.
4. What the game draws for a missing face combination (Edge cases):
   needs a late-game save (helmet worn + Strength); low priority, our
   fallback ("Do not reproduce" 3) is fine either way.
5. Whether the drawing switch Invisible sets (and Strength's end clears)
   changes how Connor is drawn beyond the textures (Do not reproduce 2).
   A check: drink Strength, wait about 10 s, drink Invisible; 10 s later
   Strength ends while Invisible still has about 5 s left. Does Connor's
   see-through look change at that moment? Low priority.
6. ~~The window losing focus~~: the clock keeps running (rule 34).
