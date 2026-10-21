# Ranged weapons: readying, switching, shooting, the bolt's flight

Study session A, 2026-10-20. Traced in `Mask.exe` unless marked "data"
(read from the game's files), "inferred" or "not sure". Covers Connor's
side of ranged weapons (bows and crossbows) and the projectile both he
and the characters shoot. Answers `fighting.md` open questions 1 (the
input side, for the ranged weapon and the parts shared with hand
weapons) and 2 (the bolt's flight), and parts of `QUESTIONS.md` Q138
(readying, the pointer) and Q139 (how an attack click finds its target).
Rocks (`prrock.kq`, thrown with the same projectile class, `parabola=1`)
and the Azriel Hammer (its own state) are **not** covered.

## Purpose

What happens when Connor readies a bow or crossbow, switches between it
and his hand weapon, aims and shoots, and how the arrow or bolt flies and
hits. Enough for B to give Connor the Small Crossbow that `sprgnbow`
drops in Daventry, and to replace the stand-in flight of the bow
spriggans' bolts (STATUS "Stand-ins of 5c") with the real one.

Words used here:

- **Update**: one game tick, 15 per second.
- **Weapon mode**: what Connor has ready. Three values, the same as
  `KQConner::setDoMode` uses: **hand** (a hand weapon ready, 0),
  **ranged** (a bow or crossbow ready, 1), **none** (nothing ready, 4).
  Connor starts with none.
- **Movement list**: the `.anm` set his movements are taken from
  (`characters.md`, `anim-scripts.md`).
- **Ranged slot item**: the one item of category `LongWeapon` he carries
  (`inventory.md`: a new one replaces the old one). At the start this is
  `INVITEM_NoRanged` (no real weapon).
- **Can be cut short**: as `fighting.md` (the running movement may be
  replaced at once, or has ended).
- **Bolt**: any `KQProjectile` (arrows and crossbow bolts alike).

## Rules

### 1. Asking for a weapon mode

1. **Three things ask for a new weapon mode** (traced):
   - **a click on a weapon slot** of the bottom bar: the item's mode
     column (`Invinfo.itl`) `SwordMode` asks for hand, `BowMode` for
     ranged, `HandMode` for none; `NoModeChange` (e.g. `NoRanged`) asks
     nothing (`inventory.md` rule 9);
   - **the Delete key** (`Connor.sae`: Delete = action `Draw`), rule 2;
   - **the Alt key** (`Connor.sae`: Alt = action `Away`), rule 3.
   All three are ignored while `handsOff` is on. Scripts can also ask
   (`KQConner::setDoMode`, rule 9).
2. **Delete** (traced) asks, by what is ready now:
   - **hand weapon ready** → ranged, but **only if the ranged slot item is
     a real weapon** (anything but `NoRanged`); with `NoRanged` nothing
     happens;
   - **ranged ready** → hand;
   - **nothing ready** → ranged if the last weapon he put away was the
     ranged one (rule 3, and rule 6's "putting away" from ranged), else
     hand. At the start that is hand (his fists).
   So Delete never puts the weapon away; it swaps between the two, and
   with nothing ready it draws the one he had last.
3. **Alt** (traced) puts away whatever is ready (asks for none) and
   remembers whether it was the ranged one (for Delete, rule 2). With
   nothing ready it does nothing.
4. **The request** (traced): if the mode asked for is **the one already
   ready, it becomes none** (that is why a second click on the ready
   slot puts the weapon away). A request for ranged needs a `LongWeapon`
   item in the inventory (`NoRanged` counts; see Do not reproduce).
5. **Only standing or moving** (traced): the request is taken **only
   while Connor is in his standing (Wait) or moving (Move) state**. In any
   other state it is **dropped, not kept for later**: jumping, falling
   and landing, a crash / stumble, pushing, a Special list, talking, a
   weapon swap already under way, the rope, the tiles and the others.
   (Fits the user's check, Q138: no readying during a jump, its landing,
   a stumble or while a draw plays.)
   Whether a swing or a shot under way in Wait / Move is cut by the swap
   or delays it: not sure (the state change may wait until the standing
   movement is `Stop`; not traced to the end).

### 2. The swap (Connor's `SwapWeapons` state)

6. **What plays** (traced; names are movement names in his lists). An
   accepted request puts Connor into a weapon swap. The current movement
   is first made "can be cut short". Then, in this order of tests:
   1. from **none** to **hand** when his hand weapon is the **Fists**:
      the list is switched to the fists' list **first**, then **`draw`**
      plays (`confists.anm`);
   2. from **hand** (Fists) to **none**: **`undraw`** plays in the fists'
      list, and the list is switched back to his own when it ends;
   3. from **hand** with the big-swing key (Shift) held, and not in first
      person: **`away2`** plays (in every Daventry list it is the same
      model as `away`); then as 4;
   4. **every other case** (any change involving the ranged weapon, or a
      hand weapon other than the Fists): **`away`** plays **in the list
      that is current now**; when it ends, the weapon mode changes, the
      list is switched to the new mode's list, and **`out`** plays from
      the new list.
   When the last movement (`draw`, `undraw` or `out`) ends, the swap is
   over and he goes back to **standing** (Wait). So for the crossbow:
   - **readying it from none**: his own `away` (`Conner.anm`: `c_shth`,
     hand to the back), then the crossbow's `out` (`conscbow.anm`:
     `scbdraw.KQS`, `Whipout.aud` at 0);
   - **putting it away**: the crossbow's `away` (`scbshth.KQS`,
     `Whipout.aud` at 0.31), then his own `out` (`c_shth` played
     backwards);
   - **crossbow → fists** (Delete): the crossbow's `away`, then the
     fists' `out` (`cf_out`), not `draw`;
   - **fists → crossbow**: the fists' `away`, then the crossbow's `out`.
7. **The swap can't be left** until its last movement has ended (then
   only to standing, moving, a Special list or dying). Movement keys
   pressed meanwhile take effect after it (inferred: the standing state's
   own rules then start the walk).
8. **When the weapon mode changes** (traced), at the moment of rule 6:
   - the movement list becomes the new mode's (rule 10), and the
     movement playing now is asked again by name in the new list (so a
     `walk` goes on as the weapon's `walk`);
   - the pointer changes (rule 11);
   - nothing else changes: the ranged slot item's projectile file and
     shoot offset were already set when the item was picked up
     (rule 21).
   The item held on the pointer, if any, is likely let go (a call that
   resets the pointer's item; not checked in the original).
9. **Scripts** (`KQConner::setDoMode <0|1|4> [movement]`, traced): if the
   mode differs, a jump is ended, his standing movement becomes `Stop`,
   he is put into standing and the same request is made (so the full
   swap of rule 6 plays). The optional second word names a movement to
   play instead of `away` / `out` / `draw` (no Daventry script uses it).
   Daventry scripts that put the weapon away this way: `ax.cs`,
   `girl.cs`, `knight.cs` (data). `KQConner::setRequiredWeaponMode`
   makes a swap that ends in another mode tell a script `WrongWeapon`
   and swap again to the required one (no Daventry script uses it).

### 3. The ranged movement list and pointer

10. **The list** (traced): with the ranged mode ready, Connor's **whole
    movement list** is the ranged slot item's list from `Connor.kq`'s
    `LongAnimList` (first person: `LongFpAnimList`), looked up by the
    item's name: `INVITEM_SmCrossbow` → `SmCrossBow` → `conscbow.anm`.
    It has its own `stop`, `walk`, `run`, jumps, `react`, `die` and so
    on (55 movements, data), and **`attack`**, `away`, `away2`, `out`.
    There is no `attackb` or `attack2` in the ranged lists (data).
    For hand weapons the list switch is the same step with
    `ShortAnimList`; for none, his own `AnimList` (`Conner.anm`)
    (inferred from the same step; the hand list's lookup was not read).
    Answers Q138 (a): the whole list is replaced.
11. **The pointer** (traced, three pictures): none → the play pointer;
    hand → the sword pointer; ranged → the bow pointer. Their resource
    numbers step by two (sword, play, bow), which with `interface.md`
    rule 12's list gives **`IDBMA_CURS03`** (sword), **`IDBMA_CURS04`**
    (play), **`IDBMA_CURS05`** (bow: the arrow picture, as B found)
    (the number-to-name step is inferred; the three-way choice is traced).
12. **Ctrl** (`CursorChgDown` / `Up`, traced): while Ctrl is held **with a
    weapon ready** (hand or ranged; also while throwing a rock), the play
    pointer shows and a left click is a **plain click** (talk, take,
    use), not an attack. Releasing it gives the weapon's pointer back.
    With nothing ready, or with an item on the pointer, Ctrl does
    nothing.

### 4. A left click with the ranged weapon ready

13. **Order of a world click** (traced): ignored while `handsOff`, while
    dying or in the portal state; in a conversation it goes to the
    conversation. **An item on the pointer wins**: the click uses the
    item (a rock is thrown, the rope is used, others are offered),
    whatever weapon is ready. Then: hand ready → the swing click
    (rule 30), ranged ready → the shot click (this section), none → the
    plain click (`clicking.md`).
14. **Ctrl held** → plain click (rule 12).
15. **Talking instead of shooting** (traced): the click line (`clicking.md`
    rule 1) is first tested against things whose collision type has bit
    **0x40, 0x20 or 0x08**; if the first one hit is a **character** whose
    collision type has bit **0x8000**, or an object named **`icewater`**,
    he acts on it as with a plain click (talk to it, etc.) and does not
    shoot. Daventry: the Wizard and the Knight (type 0xC021) (data).
16. **The aim point** (traced): the click line is tested again against
    things whose type has bit **0x40 or 0x08**: plain objects and
    characters (0x41, 0x42, barrels 0x141) and **all buildings** (0x28
    and 0x88: the church and the Wizard's house too). The aim point is
    where the line first meets one of them; if it meets none, it is
    where the line meets the **drawn ground**; if it meets no ground
    either (the sky), it is the line's far end. If the thing hit is a
    **character**, the aim point's x and y become the character's
    position (the middle of its feet) and its height stays the height
    where the line met its box. Clicking anything (character, object,
    building) also makes it his "target", which is only used for the
    log line here.
    - **Right button held** (camera drag): the click point is replaced by
      a point the camera keeps (most likely the centre of the view; not
      traced further).
17. **The shoot offset** (traced and data): the ranged slot item's name is
    looked up in `Connor.kq`'s `shootOffsets` lines (any capitals):
    `INVITEM_SmCrossBow -315 -327 1027` (x, y, z in Connor's own space).
    With no real ranged item the name `INVITEM_All` is used (no such
    line: the offset stays as it was). This offset is only a fallback:
    at the shot the bolt's own part of the model replaces it (rule 24).
18. **The steepness limit** (traced): let *h* be the height difference
    between the aim point and Connor's feet + the offset's z, and *d* the
    level distance from Connor's position to the aim point. If the line
    is steeper than **1 radian from the vertical** (about 32.7° above or
    below level: *d* < tan(1) × *h* ≈ 1.557 × *h*), the aim point is
    moved straight out (same direction, same height) to the level
    distance **1.557 × *h***. So he never shoots more steeply than about
    33°: a click on the ground at his feet sends the bolt down at 33°,
    not straight down.
19. **The Azriel Hammer** as ranged item goes into its own state
    instead (not covered; not in Daventry).
20. **Turning and shooting** (traced; turning: 3.5 rad/s,
    `connor-movement.md`):
    - in third person he **turns toward the aim point** (on the spot);
      in first person he does not turn;
    - if the aim point is **more than 45°** off the way he faces, the
      shot **waits for the turn**: it is asked as soon as the turn has
      **3 updates or fewer to go** (less than about 0.7 rad, 40°, left),
      and the turn finishes meanwhile; a new turn order (another click,
      a turn key) loses the waiting shot;
    - within 45°, the shot is asked at once (and he keeps turning).
    **Standing** (Wait):
    - the shot starts only if his running movement **can be cut short**
      (`stop` can; the crossbow's `attack` has no `interruptable` cue, so
      a click during a shot does nothing);
    - he plays **`attack`** (`conscbow.anm`: `Scbatak.kqs`, 9 frames; its
      `do` cue and `playsound Crossbow.aud` both at 0.222). The bolt
      leaves **at the `do` cue** (rule 23). When `attack` ends he plays
      `stop` again. There is **no extra delay**: the next shot can start
      as soon as `attack` has ended. Ammunition: none; shots are
      unlimited (nothing is counted or used up).
    **Moving** (Move, walking or running with the keys):
    - he does **not** play `attack`: if at least **0.5 s** (his
      `attackDelay`, default 0.5; `Connor.kq` has none) has passed since
      his last shot, the bolt leaves **at once** from his current
      movement (rule 23), the ranged item's use sound (`crossbow.aud`,
      the item list's second sound column) plays as a flat one-shot, and
      the next moving shot is due 0.5 s later. Otherwise the click does
      nothing;
    - a shot while **running** makes him **walk** (the run is switched
      off); it is switched back on when, **1 s** (twice the delay) after
      a moving shot, no other shot has been made for about that long,
      or at the next click while he is not moving;
    - the end of a standing `attack` also sets the next moving shot to
      0.5 s later.

### 5. The shot

21. **What he shoots** (traced and data): when a `LongWeapon` item goes
    into his inventory, its **projectile file** (the last column of
    `Invinfo.itl`) becomes his: Small Crossbow → `Prscbolt.kq`, Short Bow
    and Long Bow → `Prarrow.kq`, Crossbow → `Prcbolt.kq`, Small Compound
    → `Prsccblt.kq`, Compound → `Prccbolt.kq`, Flame Bow → `Prfarrow.kq`,
    Ice Crossbow → `PRicycle.kq`. `NoRanged` has none (no bolt).
    `Connor.kq`'s `projectileFile=PRarrow.kq` and `shootOffsetX/Y/Z` are
    only starting values that the first ranged item replaces.
    **`shootDamage=15` is never read** (the game has no such key).
22. **Damage** (traced): the bolt carries Connor's **ranged damage** at
    the moment of the shot (`fighting.md` rule 4: 0 at the start + the
    weapon's `LDam` + 2 per level; Small Crossbow at level 1: **5**). The
    same for characters (their ranged damage, `fighting.md` rule 2).
23. **Where it starts** (traced): at **Connor's position + his turn ×
    the start offset**. The offset is in his own model space (his front
    is −y): it turns with him (his full turn, including any lean).
24. **The start offset** (traced and data): the bolt's file names a
    model part (`meshName=arrow`). If the model Connor shows **in the
    frame playing at that moment** has a part of that name, the offset
    is **that part's own place in that frame** (the part's translation
    in the `.kqs` frame, before any frame step); it is also kept as his
    new shoot offset. Otherwise the shoot offset of rule 17 is used.
    All the crossbow models have a part `arrow` (data):
    - standing shot, `Scbatak.kqs` at the `do` cue (frames 1–2 of 9):
      about **(−200 to −231, −832, 1,062 to 1,067)**: 832 in front of
      his feet, about 220 to his right (inferred: −x is his right when
      −y is his front), 1,065 up;
    - moving shot, `scbwalk.KQS`: about (−117 to −157, −495 to −601,
      833 to 876) depending on the frame; `scbstop.KQS` about (−170,
      −385, 857).
    The part is gone in the frames after the shot (`Scbatak` frames 3–6
    have none), so the timing matters little in practice.
    For **characters** the same, with two differences: the part is
    looked up **only at their first shot with a weapon** (then kept), and
    their fallback is their own `shootOffsets` (none in Daventry, so
    0, 0, 0 until the first shot finds the part; `spbattck.KQS` has an
    `arrow` part, about (−55, −1,367, 954) at its `do` frame) (traced
    and data).
25. **Where it aims** (traced):
    - **Connor's bolt**: at the aim point of rules 16 and 18. **No lead**
      for a moving target.
    - **A character's bolt**: at its target's position (for Connor: its
      record of where he is), raised by **0.75 × the top of the target's
      collision box** (Connor's box top 1,600 → **1,200 above his
      feet**), then **led** for Connor's movement (`fighting.md` rule
      36). So B's "3/4 of Connor's height" is right; the start (B: 3/4 of
      the shooter's height) is rule 24's part instead.
    - In a rock-throw (not covered) the bolt's kind becomes 4.

### 6. The bolt's flight

26. **The bolt's file** (data; Daventry's `kq/pr*.kq`):

    | File | Shape | Speed (units/s) | `maxDistance` | Other |
    | --- | --- | --- | --- | --- |
    | `prscbolt.kq` (Small Crossbow, the spriggans) | `scbolt.kqs` | 8,000 | 18,000 | — |
    | `prarrow.kq` | `arrow.kqs` | 8,000 | 18,000 | — |
    | `prcbolt.kq`, `prccbolt.kq`, `prsccblt.kq` | `cbolt`, `ccbolt`, `sccbolt` | 8,000 | 18,000 | — |
    | `prfarrow.kq` | `frarrow1.kqs` | 6,000 | 12,000 | `weaponType=2` (Fire), `emitter=firebits.kq` |
    | `pricycle.kq` | `icycle.kqs` | 6,000 | 12,000 | `weaponType=3` (Ice), `emitter=icebits.kq` |
    | `prmonarw.kq` | `arrow.kqs` | 7,000 | 18,000 | `colliderMask=13` |

    All have `classType=KQProjectile`, `meshName=arrow`, `shadowMask=0`
    (no shadow), `collisionDetail=1`. Defaults when a key is missing
    (traced): speed **2,000**, `maxDistance` **8,000**, kind
    **Projectile**, collision type **1**, collision mask **0x1F**.
27. **Its direction** (traced): when it enters the world it is placed at
    its start point, **turned to point at the aim point**: heading from
    the level direction, and **pitch** = asin(height difference ÷
    straight distance) (no roll). It keeps that direction for good.
    Draw the bolt's model with both (its model's front is +y, inferred
    from the box in `BOUNDS.KQ`: `scbolt.kqs` 0 to 470 along y).
28. **Its movement** (traced): every update it moves **speed × 1/15 s**
    straight along that direction (8,000 → 533 units per update), with
    **no gravity** and no slowing: a straight line through the aim point
    and on beyond it. Its distance travelled is counted; once it is
    **more than `maxDistance`** (18,000: after 2.25 s) the bolt is
    removed. The move is an ordinary collision move (`connor-physics.md`
    sweep), so it can't pass through what it tests (rule 29).
29. **What stops it** (traced):
    - **anything its collision mask meets** (the pair rule of
      `connor-collision.md`: tested when the bolt's mask has a bit of
      the other's type): with the default mask 0x1F: plain objects
      (0x41), characters (0x42), Connor (4), buildings (0x28 / 0x88, bit
      0x8), and type 0x10. Its **shooter** is never hit (ignored). Each
      shape is tested by its own kind (boxes, character cylinders,
      buildings' `.dig` solids). A bolt that **starts inside** a
      character's cylinder hits it (the start-inside rule of
      `connor-collision.md` counts for bolts). Its own shape is its
      model's `BOUNDS.KQ` box (`scbolt.kqs`: about 100 × 470 × 50);
      what `collisionDetail=1` changes was not traced;
    - **the ground**: after each move, if the bolt is **at or below the
      floor grid** height (`terrh2`, the same height the thrown items
      use, `thrown-items.md`), it is removed;
    - `maxDistance` (rule 28).
30. **What a hit does** (traced): at the first thing it touches:
    - if that thing is an **object or a character** (anything placed by
      the game, not a building): if its collision type has the
      **"immune to projectiles" bit 0x4000**, it gets a hit of **0**
      damage (and the bolt's own script, if any, gets `ProjImmune`;
      `fighting.md` rule 36 says the victim's script gets it, both
      happen); otherwise it gets a hit of the bolt's damage (rule 22),
      kind **`Projectile`** (or `Fire` / `Ice` by `weaponType`), then
      `fighting.md` rules 16–33 (roll, protection, flinch, alarm, death
      with `projdie`, drops, experience). On Connor, `fighting.md` rule
      29;
    - a building, the ground, or the end of its range: no damage;
    - in every case the bolt is moved to the touching point and
      **removed at once** (its script, if any, gets `Die`; Daventry's
      bolts have none). **It does not stick**, bounce or fall. No sound
      is played by the bolt itself (no `ouchSound` in these files; the
      victim's ouch sound is `fighting.md` rule 24).
31. **Characters' bolts are the same** (rules 23–30 except the start
    offset and aim of rules 24–25). So the spriggans' bolt (`Prscbolt.kq`)
    flies at 8,000 straight from the arrow part at the time of its `do`
    cue toward 1,200 above Connor's (led) feet, pitched, no gravity, and
    is stopped by objects, other characters, Connor and buildings, or the
    floor grid, or after 18,000. A spriggan's bolt can hit another
    character in the way (and hurt it).

### 7. Hand weapons, the parts this trace showed (Q139)

32. **The swing click** (traced; Ctrl held → plain click, rule 12):
    - the click line is tested against things whose type has bit
      **0x40 or 0x20**: characters (0x42), plain objects (0x41), barrels
      (0x141), clickable things (0x21, doors, chests, items) and the
      clickable buildings (0x28) (not the 0x88 ones);
    - **nothing hit, or a building**: he turns toward the click point and
      swings at once (no new target);
    - a **character with type bit 0x8000** (or `icewater`): plain click
      (rule 15);
    - **anything else**: if its position is **within his `attackDist`
      (2,350)** of his (straight 3D distance between the two positions),
      it **becomes his target**; in any case he turns toward its
      position; if it is more than 45° off his facing the swing waits
      until the turn has **8 updates or fewer to go** (about 1.9 rad),
      else it starts at once (first person: at once, no turn);
    - **the target stays**: a later click on the ground or on something
      out of reach does not clear it, so the next swing's hit test
      (`fighting.md` rule 41: in reach and in front at the `Do` cue)
      still uses it.
    - which swing plays (`Attack` / `AttackB`, Shift `Attack2`) is
      `fighting.md` rule 40.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| weapon modes | hand 0, ranged 1, none 4 (start: none) | traced |
| requests taken | standing or moving only; dropped otherwise | traced |
| same mode asked | becomes none (put away) | traced |
| swap movements | `away` (old list) → switch → `out` (new list); Fists: `draw` / `undraw`; Shift: `away2` | traced, data |
| pointers | play `IDBMA_CURS04`, hand `CURS03`, ranged `CURS05` | traced (names inferred) |
| aim pick | types with 0x40 or 0x08, else drawn ground, else line end | traced |
| talk pick | types 0x40 / 0x20 / 0x08; a character with 0x8000 → plain click | traced |
| steepness limit | 1 rad from vertical (≈ 33° from level) | traced |
| turn before shot | > 45° off: shot when ≤ 3 updates of turn left (3.5 rad/s) | traced |
| standing shot | `attack` (9 frames), bolt at `do` 0.222, no extra delay | traced, data |
| moving shot | no movement; every ≥ 0.5 s; walk instead of run, run back after ~1 s | traced |
| Connor `attackDelay` | 0.5 s (default) | traced |
| ammunition | none (unlimited) | traced |
| damage | shooter's ranged damage (Connor with Small Crossbow, level 1: 5) | traced, data |
| start | position + turn × the `arrow` part's place (fallback `shootOffsets`) | traced |
| `Scbatak` arrow part at `do` | ≈ (−231, −832, 1,067) | data |
| character's aim | target + 0.75 × its box top (Connor: +1,200), led | traced |
| bolt speed / range (`Prscbolt.kq`) | 8,000 units/s, 18,000 (2.25 s) | data |
| bolt defaults | 2,000 / 8,000, kind Projectile, type 1, mask 0x1F | traced |
| flight | straight line, pitched to the aim, no gravity | traced |
| stops at | first object, character, Connor or building; floor grid; range | traced |
| immune bit | collision type 0x4000 → 0 damage, `ProjImmune` | traced |
| swing target | picked thing within 2,350; kept until another pick | traced |
| turn before swing | > 45° off: swing when ≤ 8 updates of turn left | traced |

## Data used

- `MASK/Connor.sae`: Delete = `Draw`, Alt = `Away`, Ctrl =
  `CursorChgDown` / `CursorChgUp`, Shift = `ModifyDn` / `ModifyUp`.
- `GAME/Kq/Connor.kq`: `LongAnimList` / `LongFpAnimList`,
  `ShortAnimList`, `AnimList`, `shootOffsets` (`nShootOffsets`),
  `projectileFile`, `shootOffsetX/Y/Z` (start values only),
  `shootDamage` (unused), `attackDist`.
- `GAME/Kq/Invinfo.itl`: category `LongWeapon`, the mode column
  (`BowMode`, `NoModeChange`), the second sound column (`crossbow.aud`),
  the last column (projectile file).
- `GAME/Kq/conscbow.anm` (and the other ranged lists): `attack` (`do`
  cue), `away`, `away2`, `out`; `Conner.anm`: `away`, `out`;
  `confists.anm`: `draw`, `undraw`.
- `.kqs` models: the part named `arrow` (`Scbatak.kqs`, `scbwalk.KQS`,
  `scbstop.KQS`, `spbattck.KQS`).
- `<world>/kq/pr*.kq`: `velocity`, `maxDistance`, `shapeName`,
  `meshName`, `weaponType`, `emitter`, `colliderMask`, `shadowMask`,
  `script` (rocks).
- `GAME/patch/BOUNDS.KQ`: the bolts' boxes.
- Scripts: `KQConner::setDoMode`, `KQConner::setRequiredWeaponMode`.

## Edge cases

- **No real ranged weapon** (`NoRanged`): the slot click asks nothing,
  Delete from the hand weapon does nothing, Delete with nothing ready
  draws the hand weapon. If he put a bow away with Alt and then lost it,
  Delete with nothing ready would ready the "ranged" mode with
  `NoRanged` (his own list, no bolt): see Do not reproduce.
- **Picking up a new ranged weapon while the ranged mode is ready**: the
  new item's projectile and shoot offset are taken at once, and the
  movement list is refreshed to the new weapon's list straight away
  (the movement playing is asked again by name), with no swap played.
- A **click far behind him** while standing: he turns; the shot comes
  only near the end of the turn (rule 20).
- **Clicking the sky**: the aim point is the click line's far end; the
  steepness limit still applies.
- **A click on a creature behind a wall**: the wall is met first; the
  bolt aims at the wall.
- A bolt aimed down at the ground near him is removed as soon as it
  goes below the floor grid (a short flight).
- A bolt shot inside a building flies until it touches the building's
  surface (buildings are in its mask).
- Bolts already flying keep flying when the shooter dies or Connor
  changes weapon.

## Do not reproduce

- **`NoRanged` counts as a ranged weapon** for the request (rule 4), so
  the edge case above can ready an empty ranged mode. Do instead: a
  request for ranged needs a real ranged item (not `NoRanged`); else
  ask for the hand weapon.
- **The shot that waits for a turn is lost** when another turn order
  replaces the turn (rule 20). Harmless; keep it or not.
- **A moving shot makes him walk** and the run comes back about 1 s
  later (rule 20). It looks like a deliberate slow-down while shooting;
  keep it (it is what the user would see), but it is fine to restore
  the run more simply (1 s after the last moving shot).
- **`shootDamage`** in `Connor.kq` is never read: do not use it.
- **The kept "arrow part" offset for characters** (rule 24: found at
  the first shot only, from whatever frame plays then): keep it; a
  character's `attack` is always the movement playing at its `do` cue,
  so the result is the same.
- Keep the rest, including no ammunition and the steepness limit.

## How to check

**Our logs (B):** per request: the key or slot, the state (taken or
dropped), the mode asked, the movements played (`away` / `out` /
`draw` / `undraw`), the list after the switch, the pointer; per shot
click: the pick (talk / aim thing / ground / sky), the aim point before
and after the steepness limit, the angle off his facing, waiting for the
turn or not, standing or moving, the delay test; per shot: the start
offset (part found or fallback), start point, aim point, direction
(heading, pitch), damage; per bolt update: position, distance
travelled; at the end: what it touched (name, type), damage sent, or
floor grid / range.

**In the original (user, by eye), Daventry** (take the Small Crossbow
from `sprgnbow`, `fighting.md` rule 31):

**Checked by the user in the original (2026-10-20):** everything works
pretty much as described.

1. **Readying**: click the crossbow's slot while standing: his hand goes
   back, then the crossbow comes out with a whip sound; the pointer is
   an arrow. Click the slot again: the crossbow goes away and his empty
   hands come back. Press Alt with it ready: the same. Hold Ctrl: the
   normal pointer, and a click talks / takes instead of shooting.
2. **Delete**: with the crossbow ready, Delete gives the fists (crossbow
   away, then fists out, no `draw`); Delete again gives the crossbow.
   Put it away with Alt, then Delete: the crossbow comes back (not the
   fists). Delete during a jump or a stumble: nothing, also afterwards
   (not remembered).
3. **Standing shot**: click a spot ahead: he aims, the bolt leaves from
   the crossbow's tip early in the motion (a little over a fifth of
   it, with the crossbow sound) and flies straight through the spot and
   on. Fast clicks: one shot per
   motion, no queue.
4. **Click at his feet**: the bolt goes down at about 33°, not
   straight down, and disappears where it meets the ground.
5. **Click behind him**: he turns first and shoots near the end of the
   turn.
6. **Moving shot**: walk and click: no shooting motion, the bolt leaves
   at once with the crossbow sound, at most two shots a second. Run and
   click: he slows to a walk and starts running again about a second
   after the last shot.
7. **The bolt**: about 3 × his running speed (8,000 vs 2,518), gone
   after about 18,000 units (2.25 s) if it hits nothing, never falls,
   never sticks in a wall or the ground.
8. **Spriggan bolts**: `sprigwnd1` at the windmill: its bolt starts at
   its crossbow, flies at Connor's chest height (1,200 up), tilted when
   he stands higher or lower, and a goblin standing in the way would be
   hit instead.
9. **Talking with a weapon ready**: a click on the Wizard with the
   crossbow ready talks to him (no shot).

## Open questions

1. Whether a swap request during a standing swing or shot cuts it or is
   refused (rule 5: depends on when the standing state lets the swap
   in; not traced to the end).
2. Where the camera's kept point for clicks with the right button held
   lies (rule 16; most likely the view's centre).
3. What `collisionDetail=1` changes for the bolt's own shape (rule 29).
4. The number-to-name step for the pointers (rule 11) is inferred from
   their order; the three-way choice is traced.
5. Rocks (`prrock.kq`: the same class with `parabola=1`,
   `peakHeightOffset`, `upVelocity`, a script) and the Azriel Hammer are
   not covered (not needed for Daventry's crossbow).
