# Fighting: attacks, hits, damage, dying

Study session A, 2026-10-20. Traced in `Mask.exe` unless marked "data"
(read from the game's files), "manual" (`MASK/Manual.pdf`), "inferred" or
"not sure". Starts where `character-movement.md` rule 25 stops ("the
attack starts"). Answers `character-movement.md` open question 2 (the
fight) and the part of `anim-scripts.md` rule 20 that was "not followed
further"; refines `character-movement.md` rules 3, 18 and 23 (said in
rules 11, 27 and 28 below). No question in `QUESTIONS.md` was open on this
topic.

## Purpose

How characters (`KQMonster`) and Connor hurt each other: when a
character's attack is due and what it plays, when a swing or a shot
counts as a hit, how much health it takes, how a character reacts to
being hit, how it dies and what it leaves behind, and how ranged
characters keep their distance. Enough for B to make Daventry's bone
goblins, goblins, spriggans, zombies and bow spriggans fight Connor as in
the original, and for Connor to hit back.

Builds on: `character-movement.md` (alarm, chase, the chase timer, the
leash, reach), `characters.md` (states, the movement list),
`anim-scripts.md` (requests, "can be cut short", the `interruptable`
cue, rules 19–20 on stats), `sound.md` rule 34 (cue names), `items.md`,
`inventory.md` (Connor's equipment adds to his stats).

Words used here:

- **Update**: one game tick, 15 per second.
- **Stats**: five whole numbers every fighter carries (rule 1).
- **Hand damage** / **ranged damage**: the second and third stats.
- **Reach**: the character's `attackDist` (`character-movement.md` rule
  25; the monster list's column, the type's for `@`).
- **Attack time**: the moment the character's next attack is due.
- **Kind** of a hit: `Normal`, `Projectile`, `Fire` or `Ice`. The last
  kind a character was hit with is kept; it picks the die movement and
  the corpse (rules 31, 33).
- **Can be cut short**: the running movement may be replaced at once
  (`anim-scripts.md` rule 4.3); a movement that has ended counts too.
- **Random**: a fresh uniform random number from 0 up to (not including) 1.
- **Round**: to the nearest whole number (halves up); **cut**: drop the
  fraction (toward zero).

## Rules

### 1. What a fighter carries

1. **Five stats** (traced and data), always written in this order (in
   `stats=` lines, in `KQStatsChangeRequestEvent`, in the item list's
   `Hlth SDam LDam Prot Exp` columns): **health, hand damage, ranged
   damage, protection, experience**. Protection is capped at 100 whenever
   it is set. (Same order as `anim-scripts.md` rule 19.)
2. **A character from the monster list** (`<world>/kq/monsters.kq`) gets
   (traced):
   - health = the type's `Health` + `xtraHealth`;
   - hand damage **and** ranged damage = the type's `Damage` +
     `xtraDamage` (`Monstats.mtl` has one `Damage` column; it fills both);
   - protection = the type's `Protect` + `xtraProtect`, capped at 100;
   - experience = the type's `Experience` + `xtraExp` (this is what Connor
     gets for killing it, rule 32);
   - `@` in an `xtra` column counts as 0;
   - then every item it **holds** adds to these: first the type's
     `invHold` items, then the list's `invHold` items (comma separated;
     `none` or `@` = no items). Each adds its `Hlth` to health and its
     `Prot` to protection (cap 100). A **`LongWeapon`** item adds its
     `LDam` to **ranged** damage and becomes the weapon it shoots with,
     which makes the character **ranged** (rule 35). Any other item adds
     its `SDam` to **hand** damage (armour adds 0). Items' `Exp` is not
     added.
3. **The combat level** (traced): a game-wide factor, **1.0 by default**;
   the console command `setCombatLevel 1|2|3` sets **0.66 / 1.0 / 1.5**
   (its usage text says "3 is hardest"); the game's own menus also set it
   (which screen: not traced). Every world's `newworld.cs` runs
   `applyCombatLevel 1` before loading its objects; from then on each
   character entering the world (not Connor) has its **health, hand
   damage and ranged damage** multiplied by the factor and **cut** to a
   whole number. Protection and experience are not scaled. Loading a world
   switches `applyCombatLevel` off again until its `newworld.cs` runs.
   The factor also changes the damage roll against characters (rule 18).
4. **Connor** (traced and data): `GAME/Kq/Connor.kq` `stats=10 10 0 0 0`
   (health 10, hand damage 10, ranged 0, protection 0, experience 0),
   `level=1`, `attackDist=2350`. Carried equipment adds its stats
   (`inventory.md` rule 14: a hand weapon its `SDam`, a ranged weapon its
   `LDam`, armour its `Prot`). Each level up gives hand damage +5, ranged
   damage +2, protection +1 and a full heal to 10 × level
   (`anim-scripts.md` rule 19; confirmed: the +5 goes to hand damage).
5. **Daventry's fighters** at combat level 1.0 (data, by rules 2 and 3;
   reach and attack delay from the list, else the type):

   | Character(s) | Type | Health | Hand | Ranged | Prot | Exp | Reach | Delay (s) |
   | --- | --- | --- | --- | --- | --- | --- | --- | --- |
   | `goblin22` (Sara's house) | goblin2 (bone goblin) | 15 | 2 | 2 | 0 | 3 | 1,500 | 1 |
   | `goblin18` / `goblin20` / `goblin21` | goblin2 | 30 / 40 / 50 | 5 | 5 | 4 / 4 / 14 | 3 | 1,500 | 0.5 / 0.5 / 1 |
   | `gobcon1` (Connor's house) | goblin | 15 | 2 | 2 | 0 | 4 | 1,500 | 0.5 |
   | `sprigan1` / `sprigan19` | sprigan | 25 / 50 | 7 / 2 | 7 / 2 | 5 | 7 | 1,200 | 0.5 |
   | `sprigwnd2` / `sprigwnd3` (windmill) | sprigan | 80 | 10 | 10 | 12 | 7 | 1,200 | 0.1 / 0.2 |
   | `Brdgsprig` / `Fallsprig` | sprigan | 75 / 80 | 10 | 10 | 15 | 7 | 1,200 | 0.2 |
   | `RuinSprig1`–`4` | sprigan | 130, 130, 120, 120 | 15, 16, 20, 20 | same | 20 | 7 | 1,200 | 0.4, 0.5, 0.3, 0.5 |
   | `bossprig` (holds Leather Armor) | sprigan | 230 | 23 | 23 | 32 | 7 | 1,200 | 0.2 |
   | `zograv1`, `zograv2` (behind the farm) | zombie | 10 | 3 | 3 | 2 | 2 | 1,100 | 0.5 |
   | graveyard `zograv3`–`17` | zombie | 110–130 | 13–15 | same | 10 | 2 | 1,100 | 0.1–0.5 |
   | `RoadZ1`–`RoadZ5` | zombie | 40, 45, 50, 50, 43 | 6 | 6 | 37 | 2 | 1,100 | 0.2–0.5 |
   | `sprgnbow` (graveyard tomb roof) | sprgnbowtype (holds Small Crossbow) | 130 | −3 | **2** | 15 | 15 | 12,000 | 1 |
   | `sprigan` (hero's tomb) / `sprigwnd1` (in the windmill) | sprgnbowtype2 (Small Crossbow) | 130 | 8 / 6 | **13 / 11** | 15 / 5 | 12 / 10 | 12,000 / 6,000 | 0.5 |

   Chickens, the pig and bats: health 5, damage 5, protection 3,
   experience 0, reach 0 (they never attack). The Knight, the Henchman
   and `sprdump` are scripted (not covered here).

### 2. When an attack is due

6. **The attack time** (traced) starts at **0** when the character is
   made, so **its first attack is due at once**: the first time it is in
   a position to attack (rule 7), it attacks, with no wait. (What delays
   a first swing in practice is the 0–1 s reaction after the alarm and
   the chase itself, `character-movement.md` rule 23; this fits the
   user's check: "the attack seems due soon after it gets within reach".)
   The attack time is **not** reset by a new alarm or a new chase: a
   character that attacked a minute ago may attack at once again.
7. **When the attack starts** (traced; the positions are
   `character-movement.md` rule 25):
   - **Standing** (Wait, with a target): every update, if it **can see**
     Connor (the once-a-second sight result) and he is closer than its
     reach (ranged characters: 1.25 × reach) and the attack time has
     come.
   - **Moving** (chasing): every update, if it can see Connor, he is
     closer than its reach (ranged characters too: 1 × reach here) and the
     attack time has come, **whatever Connor is doing**. Before the attack
     time it follows the stop rules of `character-movement.md` rule 25.
   - In both, the attack starts **only if the running movement can be
     cut short or has ended**; otherwise it is tried again next update.
     `walk`, `run` and `stop` can be cut short in every Daventry list; a
     `react` (rule 23), `birth` or a running attack can't (rule 12).
   - Distance is the straight 3D distance between their positions (feet).
   - The character does **not** have to face Connor to start: it is
     turning toward him all the time (standing: on the spot; moving: by
     steering, 3.5 rad/s), and whether it faces him only matters at the
     hit (rule 13).
8. **What it plays** (traced): a character always asks for its movement
   named **`Attack`** (any capitals; Daventry's lists call it `attack`),
   as an ordinary request from its own list (`anim-scripts.md` rules 4–5);
   it starts at once (rule 7 made sure it can). `Attack2` and `AttackB`
   are Connor's (rule 40). No random choice for characters.
9. **During the attack** (traced and data):
   - the attack movements' models have **no forward step** (every frame
     at 0, checked for `g2_attck`, `g-attack`, `spattack`, `z-attack`,
     `spbattck`), so the character **stands still** while it swings
     (fits the user's check "it can't attack and move forward at the same
     time"), also when it attacked from its moving state;
   - it **keeps turning toward Connor** (standing: on the spot; moving:
     its steering goes on);
   - a moving character's stuck check (`character-movement.md` rule 7)
     is **off** while the attack plays and starts again with the next
     `walk` / `run`;
   - the attack time is set to "never" while the attack plays, and the
     chase timer to 5 s from its start.
10. **When it ends** (traced): the attack time becomes **now +
    `attackDelay`** and the chase timer 5 s from now (this is the side
    effect of `character-movement.md` rule 18, which applies to any
    movement named `Attack` or `Attack2` that ends). Then a standing
    character asks for `Stop` at a random frame and speed
    (`characters.md` rule 18); a moving one asks for its `walk` / `run`
    again and goes on chasing (`character-movement.md` rule 4). So
    between two swings there is **the swing's length + `attackDelay`**.
11. **When it is cut short** (traced): if the attack never ends because
    something replaced it, the "never" is put right the next time one of
    these happens: the character **enters** its standing or moving state
    (a state change), or it **is hit** (rule 22): then too the attack time
    becomes now + `attackDelay` (chase timer 5 s).
11b. **The swallowed swing** (traced, added 2026-10-20 after the user's
    check in the original: "sometimes he makes the attack sound but no
    animation, then he attacks normally; when walking or running away
    they attack at once"). It falls out of rules 7, 11 and 12 with
    `character-movement.md` rule 25:
    1. A **chasing** (moving) character comes within reach while its
       attack is due: it starts `Attack` from its moving state. The
       attack's sound cue at phase 0 plays (goblin, bone goblin, zombie).
    2. In the **next update** its attack time is "never", so the moving
       state's "before the attack is due" rule applies: if **Connor has
       no forward / back key held**, it switches to standing.
    3. Entering the standing state asks for `Stop` at a random frame.
       The attack is still in its first, cut-short moment (rule 12:
       0.18–0.25 s; one update is 0.067 s), so **`Stop` replaces it**:
       the swing is never seen. The attack time becomes now +
       `attackDelay`.
    4. `attackDelay` later (0.5–1 s) it swings for real, from standing.
    So: Connor standing (or only turning) when it arrives → sound, no
    swing, then a real swing; Connor walking or running (key held) →
    the moving state never switches and **the first swing plays in full
    at once**. If its attack was not due yet when it arrived, it stops
    first and swings normally later (no swallowed swing): hence
    "sometimes". Spriggans' attack sound comes at 0.1875, after the cut,
    so for them it is a silent skip. Not checked: whether this is what
    the user saw every time (it is the only path traced that gives a
    sound without a swing). **Not to be reproduced** (the user,
    2026-10-20): see Do not reproduce.
12. **Daventry's attack movements** (data; seconds at speed 1.0):

    | List | Attack length | Can be cut short | Hit (`Do` cue) at | `react` length |
    | --- | --- | --- | --- | --- |
    | bone goblin (`goblin2.anm`) | 1.00 | first 0.19 (cue `interruptable 0` at 0.1875) | 0.50 | 0.53 |
    | goblin (`goblin.anm`) | 0.80 | first 0.18 (0.2308) | 0.55 (0.6923) | 0.60 |
    | spriggan (`sprigan.anm`) | 1.00 | first 0.125 | 0.50 | 1.33 |
    | zombie (`zombie.anm`) | 1.33 | first 0.25 (0.1905) | 0.63 (0.4762) | 0.80 |
    | bow spriggan (`sprgnbow.anm`, `sprgnbo2.anm`) | 1.20 | never (no cue) | 0.15 (0.125) | — |

    Each melee attack starts with `interruptable 1` at 0 and switches to
    `interruptable 0` a little later (`anim-scripts.md` rule 7): a hit in
    that first moment can make it flinch instead (rule 23); after it, the
    swing always finishes. The idle lists' `attack` (`gob2idle.anm`,
    `sprgidle.anm`) has **no `Do` cue**: a fidget swing never hits.

### 3. The swing lands (characters without a weapon to shoot)

13. **The hit test** happens when the attack movement passes its **`Do`
    cue** (traced; the cue name is matched ignoring capitals, `sound.md`
    rule 34). It needs a target (Connor) and passes when **both**:
    - **in reach**: the straight 3D distance from the attacker's position
      to the target's position (both at the feet) is **at most** the
      reach. If the target is **Connor and his back key is held** at that
      moment (walking backward; `character-movement.md` rule 25 uses the
      same key state), the reach is **1.25 ×** (a step back does not save
      him at once);
    - **in front**: the target lies in the attacker's front half: the
      direction from the attacker to the target makes an angle of
      **less than 90°** with the way the attacker faces (exactly 90° counts
      as behind). The test uses the 3D direction, so height differences
      tilt it slightly.
    A character facing away (still turning, rule 7) or one Connor has
    stepped out of reach of by the `Do` moment **misses** (the original
    logs "`<name>` missing `<target>`, too far away"; nothing else
    happens: no sound, no message).
14. **A hit** (traced) sends the target a hit with the attacker's **hand
    damage** (rule 1; at least what the stats say, even 0 or negative).
    Its kind is **`Fire` if the attacker carries a Flame Sword**
    (`INVITEM_FlameSword`, id 1110), else `Normal`. (For Connor: the Flame
    Sword's fire kills, rule 31.)
15. **Characters with no collision shape, on their way home** (traced;
    answers the bracket in `character-movement.md` rule 28; none in
    Daventry): instead of taking up the chase again they **"zap"
    Connor**: whenever he is in view and within reach and their attack
    time has come, he gets a `Normal` hit of their hand damage at once,
    with no movement and no reach or front test, and the next one is due
    `attackDelay` later.

### 4. Damage

16. **One hit, two random steps** (traced). Let *D* be the hit's damage.
    1. **The roll**: *R* = round(*D* × (*a* + random × *b*)), at least
       1. The range depends on who is hit:
       - **Connor**: *a* = 0.25, *b* = 0.75: **25 % to 100 %** of *D*;
       - **a character**, combat level below 1.0: 0.85, 0.40: **85 % to
         125 %**; combat level 1.0 or more: 0.75, 0.50: **75 % to
         125 %**.
    2. **Protection**: if the victim's protection *P* is **exactly 100**,
       or *D* is 0, no health is lost. Otherwise the protection is rolled
       too: *P'* = round(*P* × (0.75 + random × 0.5)), at most 100; the
       loss = cut(*R* × (100 − *P'*) ÷ 100), **at least 1**. Health drops
       by the loss.
17. The roll *R* (not the loss) is what decides the flinch (rule 23).
18. **Death**: when health is **below 0** after a hit (0 is still
    alive), the victim dies (rule 30; Connor: his dying state, rule 29).
19. **Worked examples** (inferred from rules 16–18, combat level 1.0):
    - Connor (hand 10, no weapon) hits `goblin22` (health 15, protection
      0): *R* = 8 to 12, all of it lost; the first hit leaves 3 to 7, the
      second always takes it below 0: **it dies at the second hit**.
    - `goblin22` (hand 2) hits Connor (protection 0): *R* = round(0.5 to
      2) = 1 or 2; he loses 1 or 2 of his 10.
    - A graveyard zombie (hand 15) hits Connor (protection 0): *R* = 4 to
      15, all lost: at level 1 (10 health) one hit of 11 or more kills
      him.

### 5. Being hit (characters)

20. **No knock-back** (traced): nothing in the hit moves the victim or
    the attacker, for characters or for Connor.
21. **A hit on a dying character** (rule 30) still takes health and still
    alarms others (rule 26), but changes nothing else (it can't leave the
    dying state).
22. **What a hit does to a character, in order** (traced):
    1. the hit's **kind** is remembered (rule 30);
    2. if it was not active (`character-movement.md` rule 20), it
       becomes active (if there is room);
    3. its **chase timer** is set to 5 s;
    4. its **hit count** goes up by 1; from the **second** counted hit on
       it moves at `run` whenever it starts moving (rule 27);
    5. if its `likelyToFlee` is **above 0.5**, it **must flee** (rule 37);
    6. the damage (rule 16); if it dies, the rest is skipped;
    7. if its attack time was "never" (an attack under way), it becomes
       now + `attackDelay` (rule 11);
    8. the flinch or the ouch sound (rules 23, 24);
    9. it learns **where Connor is now** (as if it had just seen him);
    10. the **alarm with "run"** spreads (rule 26).
23. **The flinch (`react`)** (traced): the character plays its movement
    named **`React`** (any capitals) when **all** of:
    - the roll *R* is **more than `normalReactPercent` × *D*** (default
      **0.9**; no Daventry file sets it): with the 75–125 % roll and
      *D* = 10 (Connor's fists) this is true **60 %** of the time (*R* 10,
      11 or 12); for small *D* the rounding makes it more likely (*D* = 2:
      always);
    - the running movement can be cut short (so not during another
      `react`, not after an attack's `interruptable 0`, not during
      `birth` or `hide`);
    - its last flinch **ended more than 2 s ago** (fixed for every
      ordinary character; no key changes it, traced 2026-10-22, Q136).
      `reactPause` and `stunPause` are read only by two special classes
      (none in Daventry): the Dead City tower skeleton
      (`KQTowerSkeleton`, `skeltowr.kq`: defaults 2 s and 3 s, file 1.2
      and 1.5) and Temple 4's Lucreto (`KQLucreto`, `lucend.kq`). The
      skeleton: when its `reactback` ends while **moving**, the gap
      before its next flinch is `reactPause`; while **standing**, the
      fixed 2 s; either way it is then stunned for `stunPause`, its chase
      timer is set to 5 s, and the ground 400 ahead of it is probed:
      nothing under that point and it **dies** (that is how it falls off
      the tower), otherwise it goes back to standing. Lucreto: after its
      `reactback` it stands for a random `reactPause` − 1 to `reactPause`
      seconds (1 to 2 s) before moving again.
    A **special** flinch replaces it when the character's
    `specialReactAnim` is set and *R* is more than `specialReactPercent` ×
    *D* (no Daventry file has them). `KQMonster::setReactPercent` and
    `KQMonster::setSpecialReact` set these from scripts. The flinch is an
    ordinary movement in its current state (standing or moving): when it
    ends, the chase timer is set to 5 s and the 2 s pause starts
    (`character-movement.md` rule 18), and the standing / moving state goes
    on (Stop at a random frame, or `walk` / `run` again). The `react`
    models have no forward step (data): it flinches on the spot.
    Flinching inside an attack's first moment **cancels that attack**; the
    next one is due `attackDelay` after the hit (rule 22.7).
24. **The ouch sound** (traced): when it does **not** flinch, its
    `ouchSound` (Daventry: `gblnouch.aud`, `sprgouch.aud`, `zombouch.aud`;
    default `sword.aud`) plays as a placed one-shot at its position (min
    5,000, max 25,000, as `sound.md` rule 35), **at most once per
    second** (a flinch also starts that second).
25. **A surprise line** (traced; no Daventry file has the keys): a
    character hit while it could **not** see Connor may say one of its
    `Surprise…` lines (`characters.md`), at most once every 20 s for the
    whole game.
26. **The alarm with "run"** (traced; refines `character-movement.md`
    rules 23–24): after every hit, **every active character** with an
    alarm radius above 0 that **can see Connor** (or whose team has a
    member that can) is alarmed **with "run"**, the hit one included:
    - one that had no target is alarmed as in `character-movement.md`
      rule 23 (0–1 s reaction, alarm line, team), and then:
    - in every case its **hit count becomes 2** (so from now on it always
      runs), it switches to **`run`** at once if it is moving, and its
      **chase timer becomes 10 s**.
    So **one hit is enough** to make a character that can see Connor run
    after him with a 10 s chase timer (a flinch ending later sets the
    timer back to 5 s, rule 23). Without sight it takes two hits (rule
    22.4).
27. **Correction of `character-movement.md` rule 3** ("after it has been
    hit twice it always runs"): true for its own hits, but rule 26 makes
    it run after **one** hit whenever it can see Connor.
28. **Hit while not alarmed** (traced): a character that has no target
    when it is hit (it could not see Connor, or its alarm radius is 0)
    and is standing:
    - with **no team**, a chase radius above 0 (and not tied to another
      object; not sure what sets that, never in Daventry): it starts
      moving at `run` **without a target**. While so, every update: if
      Connor is within its alarm radius it is alarmed (no reaction
      delay); after its chase timer, beyond 4 × its alarm radius from
      Connor it gives up, beyond its chase radius from home it runs home;
      otherwise it heads for a point **about 30° (0.52 rad) to one side of
      Connor** (see Do not reproduce: the original's point is wrong);
    - otherwise (a team, or a guard with chase radius 0, e.g. a chicken
      or the pig): it is alarmed at once (no reaction delay). A chicken
      or the pig then stands and turns to Connor (its reach is 0) and
      gives up after its 5 s chase timer (twice its alarm radius is 0)
      (inferred from the traced rules).

### 6. Connor being hit

29. Traced, Connor's side only as far as the fight needs:
    - hits are ignored while he is dying;
    - the roll is 25–100 % (rule 16), then his protection;
    - he **flinches** by the same rule 23 (his `react`, `normalReactPercent`
      0.9): with his 25–100 % roll that is only about **13 %** of hits
      (when *R* rounds to more than 0.9 *D*), and only when his running
      movement can be cut short and his last flinch ended over 2 s ago;
      else his `ouchSound` (`conrow3.aud`) at most once a second;
    - his health is kept at most 10 × level;
    - two one-off tips are sent to Connor (cue numbers, not traced
      further): one the first time he is hit by a character in one of his
      weapon modes (not sure which), one the first time his health falls
      below half of his maximum;
    - below 0 he goes into his dying state: `die`, or `dropweapon` then
      `firedie` / `icedie` for fire and ice (not followed further: game
      over is another study).

### 7. Dying

30. **Entering the dying state** (traced), at once when health falls
    below 0 (rule 18):
    - health is set to 0; a conversation it was in ends (inferred: the
      code ends its talk state; not checked);
    - its **die movement** by the last hit's kind: `Normal` → **`Die`**;
      `Projectile` → **`projdie`** if its list has one (Daventry: the
      spriggans' `sprigan.anm`), else `Die`; `Fire` → **`firedie`**;
      `Ice` → **`icedie`**. (All Daventry lists have `die`, `firedie`,
      `icedie`.) An unknown kind plays nothing;
    - its **drops** happen now (rule 31);
    - it leaves the active list (frees a place, `character-movement.md`
      rule 20); its gravity is switched off and it becomes **not solid**
      (collision type and mask 0: Connor walks through it, nothing tests
      against it);
    - it can **never** change state again (scripts' `setState` included);
    - a script's `KQMonster::setState <name> Die now` skips the movement:
      it goes straight to rule 32.
31. **Drops** (traced): its drop list is the monster list's `invDrop`
    (`@` or `none` = nothing) followed by the type's `invDrop` (unless
    `none`), comma separated, quotes and an `INVITEM_` prefix allowed,
    `SilverCoins(3)` = 3 coins. Each entry is first **compared with the
    named parts of the character's current pose** (traced 2026-10-24,
    Q169): a `.kqs` model is made of named parts, and the die models
    carry the character's weapon as a part named exactly like its drop
    entry (`spbdie.kqs` has a part `SmCrossbow`, `skbsdie.kqs` a part
    `BroadSword`, `skcmdie.kqs` `Mace`, `sk_die3.kqs` `SkelKingSword`;
    the walk, stop and attack models have no such part). The comparison
    ignores case and is made after the quotes and the `INVITEM_` prefix
    are taken off.
    - **An entry that names a part** is **not made now and not thrown**:
      it is kept for the end of the die movement (rule 31a). Only one
      entry is kept: if several entries name parts, the last one wins and
      the earlier ones are lost (made neither now nor later) [traced; no
      data entry has two].
    - **Every other entry** becomes a new item, started at
      the character's position (its feet, plus an offset that is 0 for
      Daventry's characters; which key sets it: not sure), and **thrown
      exactly like `KQObject::tossItems`** (`items.md` rule 12): its item
    mover picks, when it starts, a random direction all round, 75°–85°
    up, 5,000–6,500 units/s, and the whole flight (arc, what stops it,
    bounces, tumble, settling, land sound) is the same: see
    `thrown-items.md` (traced 2026-10-20, Q137). (Corrected
    2026-10-20: this rule first said "no throw"; the throw is made by the
    item mover's own start, which the first reading missed. The user saw
    it in the original: the items fling upward and land with a sound.)
    They can be clicked and picked up while still in the air (the user's
    check; items are clickable wherever they are, `items.md` rule 4).
    (Corrected 2026-10-24, Q169: this rule first said every entry is
    thrown, and called the kept entry "an object already in the world";
    it is the part-named entry above. The user saw it in the original:
    coins, potions, mushrooms and crystals fly; the weapons don't.)
    Which pose is "current" at the dying start: the die movement's first
    frame (inferred: the weapon parts exist only in the die models, and
    the user saw the weapons kept, not thrown). So a death whose movement
    has no such part throws the weapon like the rest: the bow spriggans'
    **fire** death `Spbfdie.kqs` has only a part `crossbow`, not
    `SmCrossbow`, so a burnt bow spriggan's crossbow is thrown [data].
    The items held (`invHold`) are not items in the world: they only add
    to the character's numbers (rule 2) and are never dropped; a held
    weapon is dropped only if it is also in `invDrop` [traced].
31a. **The kept weapon, when the die movement ends** (traced 2026-10-24,
    Q169), first of all the end steps (before rule 32's experience and
    corpse):
    1. the item is made and its **own model's part `body` is laid exactly
       on the part of the same name in the character's last frame**:
       place and full turn (it can lie on its side, tilted, as the
       weapon lies in the die movement), taken from the character's place
       and turn. If the pose has no such part by then: the character's
       place and turn, 750 units up (a fallback; no data case);
    2. **a line test**: from 1,000 units above the item's place to
       1,000 units above the character's feet. If the terrain, or any
       solid object other than the dying character, cuts that line, the
       item is **thrown instead**, like the rule 31 items, from the
       character's feet. (This keeps a weapon from being left inside a
       wall or under the floor.)
    3. otherwise it **stays exactly there**: no throw, no fall, no
       landing sound [traced: it is placed, not given the falling mover;
       that nothing later makes it fall is inferred]. It can be clicked
       and picked up at once.
    The item is a new world item like the thrown ones (the script
    variable `NewInvItem` is set to it, as for them). In the die movement
    the character's own model shows the weapon falling; when the movement
    ends the model is replaced by the corpse (rule 32), and the real item
    is already lying where the model's weapon was, so the swap is not
    visible. With a script's `Die now` (rule 30) these end steps come at
    once; whether a part was matched at that dying start (no die
    movement is played): not sure.
32. **When the die movement ends** (traced), after rule 31a:
    1. Connor gets its **experience** (rule 2) as a stats event (fifth
       number only; `anim-scripts.md` rule 19 then applies his levels),
       whoever killed it;
    2. a **corpse** is left: a new plain object, drawn with a shape chosen
       by the kind: `Normal` → `deadDts`; `Projectile` → `projdKqs` if set,
       else `deadDts`; `Ice` → `idedKqs` if set, else **nothing**;
       `Fire` → **nothing** (burnt away). It is placed exactly where the
       character stood, turned the same way, casts no shadow, and uses
       the same alternate texture if the character had one
       (`sprigan(test2.bmp,test3.bmp)`, `characters.md`). It is not solid
       (inferred: a plain object with no collision set, not checked);
    3. its script gets **`Die <kind>`** (`Normal`, `Projectile`, `Fire`,
       `Ice`; answers `anim-scripts.md` rule 20's open end);
    4. the character is removed from the world.
    Daventry (data): goblin `gdead.kqs` / ice `G_ided.kqs`; bone goblin
    `g2dead.kqs` / `G2_ided.kqs`; spriggan `spdeath.kqs` / `spided.kqs` /
    projectile `sp_dead8.KQS`; zombie `zdead.kqs` / `Z-ided.kqs`; bow
    spriggans `spbdeath.kqs` / `spbided.kqs`.
33. **A first-kill tip** (traced): the first time any character dies
    whose type is not `bat`, `chicken` or `checken2` (sic: so `chicken2`
    does count), Connor gets a one-off cue 1.5 s later (a help tip; not
    followed further).

### 8. Ranged characters (`doMode=1`)

34. **Who is ranged** (traced and data): a character that holds a
    **`LongWeapon`** (rule 2) is switched to `doMode` 1 when it is made.
    In Daventry: the three bow spriggans (`sprgnbow`, `sprigan` at the
    hero's tomb, `sprigwnd1` in the windmill), all from their type's
    `invHold INVITEM_SmCrossbow`. None of their `.kq` files has a `doMode`
    key.
35. **Distances** (traced; `character-movement.md` rule 25): standing, it
    starts moving after Connor only beyond **1.25 × reach** (or out of
    sight) and shoots when he is within 1.25 × reach; moving, it shoots
    within reach and stops within **0.75 × reach** before its attack is
    due.
36. **The shot** (traced, partly): at the attack movement's `Do` cue it
    fires the weapon's projectile (the item's last column, Small Crossbow
    → `Prscbolt.kq`: Daventry's says speed 8,000, `maxDistance` 18,000)
    from an offset for that weapon (`shootOffsets` keys, Connor-style; none
    for Daventry's spriggans, so their default), **aimed ahead of Connor**
    when he moves: it takes his current velocity and the bolt's speed and
    aims where they would meet; it aims at a point above his feet (about
    three quarters of the way up; not sure). No reach or front test: it
    always shoots. The bolt carries the shooter's **ranged damage**
    (inferred from the original's log line; the hand-over was not
    followed). It hits the first character or Connor in its way (never
    its shooter); the hit's kind is `Projectile` (so the target plays
    `projdie` if it dies of it). A target whose collision has the
    "immune to projectiles" flag takes 0 and its script gets
    `ProjImmune`. The bolt's flight itself (gravity, lifetime) was not
    traced. **Traced 2026-10-20 in `ranged-weapons.md` rules 21–31**:
    it starts at the `arrow` part of the shooter's model (found at its
    first shot), aims 0.75 × the target's box top above its feet
    (Connor: 1,200), flies straight and pitched at its file's speed with
    no gravity, and is removed at the first object, character, Connor or
    building it touches, below the floor grid, or past `maxDistance`.
37. **Fleeing / keeping distance** (traced): only for a ranged character
    with a **chase radius above 0** (and not tied to another object, as
    rule 28):
    - once a second while it stands with a target, if it **must flee**
      (rule 22.5) or, with probability **`likelyToFlee`** (default
      **0.5**), when Connor is closer than **`attackTooClose` × reach**
      (default **0.25**): it starts "keeping distance": `run`, moving;
    - while keeping distance it heads straight **away from Connor** (the
      point as far beyond itself as Connor is on the other side) and stops
      (standing again) once he is farther than **`attackFarEnough` ×
      reach** (default **0.75**); its leash still applies (beyond its chase
      radius from home after its chase timer: it runs home);
    - "must flee" is cleared when it starts keeping distance;
    - the monster list's reach column can give all three: `attackDist
      tooClose farEnough` (Gnome's slime: `5000 0.4 3.0`); a single number
      keeps the defaults. `likelyToFlee` and `mustFlee` are `.kq` keys.
38. **Daventry's bow spriggans** (data with the rules above): guards
    (`guard`, chase radius 0) with no wander points: they **never move
    and never flee**. Each stands, turns to Connor once alarmed (alarm
    radius 12,000; `sprigwnd1` 6,000), and shoots whenever he is in view
    within 1.25 × reach (15,000; `sprigwnd1` 7,500) and its attack is due:
    a shot every 1.2 s + delay (1 s or 0.5 s). Their arrows do 2 (`sprgnbow`),
    13 (`sprigan`) and 11 (`sprigwnd1`) before the rolls. Their special
    deaths are scripted (`sprgnbow.cs`: Connor's weapon picks a cutscene;
    not part of this note).

### 9. Connor hitting back

39. **Controls** (manual; the input side not traced): with a hand weapon
    drawn (its bottom-bar slot) the pointer is a sword; **each left click
    makes Connor swing toward the click**, turning round first if it was
    behind him; he always swings, "if he's not close enough he'll just
    hit dead air". **Shift + left click** is the "big" swing. **Delete**
    switches between hand and ranged weapon. With a ranged weapon each
    left click shoots toward the click. **Ctrl** held gives the plain
    pointer for a moment. The clicked character becomes his target
    (`clicking.md`). **Traced 2026-10-20**: readying, Delete, Alt, the
    pointers and Ctrl in `ranged-weapons.md` rules 1–12; the ranged
    click and shot in rules 13–25; how a swing click picks its target in
    rule 32.
40. **Which swing** (traced): a plain click plays **`Attack` or
    `AttackB`**, 50/50 (in some case always `Attack`: not traced which); a
    click while a swing is already playing does nothing; Shift plays
    **`Attack2`**. Like the characters', it starts only when his running
    movement can be cut short. Each has its `Do` cue (e.g. bare fists,
    `confists.anm`: `attack` 0.44, `attackb` 0.33, `attack2` 0.25 of the
    movement). There is no attack delay for hand weapons.
41. **His hit** (traced): the same test as rule 13, with **his** reach
    (`attackDist` 2,350) and his target, and the same messages. Damage =
    his **hand damage** (rule 4: 10 at the start, plus weapon and
    levels); the big swing multiplies it by a random factor from
    **`minDamageFactor` to `maxDamageFactor`** (`Connor.kq`: 0.75 to 4;
    the manual says "0.75 to 5"), cut to a whole number. Kind `Fire` with
    the Flame Sword (rule 14), else `Normal`. Then rules 16–33 for the
    victim (75–125 % roll, its protection, flinch, alarm with "run",
    dying, drops, experience to Connor).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| update | 15 per second | traced (earlier notes) |
| stats order | health, hand, ranged, protection (cap 100), experience | traced |
| character stats | type + `xtra`; `Damage` fills hand and ranged; held items add | traced |
| combat level factor | 0.66 / 1.0 (default) / 1.5; health, hand, ranged × factor, cut | traced |
| Connor start | 10 health, hand 10, ranged 0, prot 0; reach 2,350 | data |
| first attack | due at once (attack time starts at 0) | traced |
| next attack | swing end + `attackDelay` (cut short: from the state change or hit) | traced |
| attack start needs | in sight, within reach (ranged standing 1.25 ×), movement can be cut short | traced |
| chase timer | 5 s at swing start, swing end, flinch end, each hit; 10 s after alarm with "run" | traced |
| hit test | at `Do`; distance ≤ reach (× 1.25 if Connor holds back); target in front half (< 90°) | traced |
| roll on Connor | round(*D* × (0.25 + random × 0.75)), ≥ 1 | traced |
| roll on a character | round(*D* × (0.75 + random × 0.5)); combat level < 1: (0.85 + random × 0.4) | traced |
| protection | *P* = 100: no loss; else *P'* = round(*P* × (0.75 + random × 0.5)) ≤ 100; loss = cut(*R* × (100 − *P'*)/100) ≥ 1 | traced |
| death | health < 0 | traced |
| flinch | *R* > 0.9 × *D*, movement can be cut short, last flinch ended > 2 s ago | traced |
| ouch sound | when no flinch, at most 1 per s, placed 5,000 / 25,000 | traced |
| surprise line | hit unseen, at most 1 per 20 s game-wide | traced |
| alarm with "run" | hit count 2, `run`, chase timer 10 s | traced |
| hit while not alarmed | runs to ~30° beside Connor; gives up beyond 4 × alarm radius | traced |
| ranged | holds a `LongWeapon`; stop 0.75 ×, shoot 1.25 × (standing) / 1 × (moving) reach | traced |
| flee | once a second, chance `likelyToFlee` 0.5 within `attackTooClose` 0.25 × reach; stop beyond `attackFarEnough` 0.75 × reach; must flee if `likelyToFlee` > 0.5 and hit | traced |
| drops | thrown as `tossItems` at the dying start: random heading, 75°–85° up, 5,000–6,500 units/s | traced, user-checked |
| dropped weapon (entry named like a part of the die model) | not thrown; at the die movement's end its `body` part laid on that part of the last frame (place and turn), stays there; thrown instead if the line 1,000 above it to 1,000 above the feet is blocked | traced, user-checked (kept, not thrown) |
| bolt (Daventry `Prscbolt.kq`) | speed 8,000, `maxDistance` 18,000 | data |
| big swing | × 0.75 to 4 (`Connor.kq`) | traced, data |
| first-kill tip | 1.5 s after | traced |

## Data used

- `GAME/Kq/Monstats.mtl`: `Health`, `Damage`, `Protect`, `Experience`,
  `invHold`, `invDrop`, `attackDist` (with optional `tooClose farEnough`),
  `attackDelay`.
- `<world>/kq/monsters.kq`: `xtraHealth`, `xtraDamage`, `xtraProtect`,
  `xtraExp`, `invHold`, `invDrop`, `attackDist`, `attackDelay`.
- `GAME/Kq/Invinfo.itl`: `Hlth SDam LDam Prot`, category (`LongWeapon`),
  id 1110 (Flame Sword), the last column (projectile file).
- Character `.kq`: `deadDts`, `idedKqs`, `projdKqs`, `ouchSound`,
  `normalReactPercent`, `specialReactAnim`, `specialReactPercent`,
  `likelyToFlee`, `mustFlee`, `Surprise…`, `script` (gets `Die`).
- `.anm`: `attack` (with `interruptable` and `Do` cues), `react`, `die`,
  `projdie`, `firedie`, `icedie`; Connor's `attack`, `attackb`,
  `attack2`, `react`, `dropweapon`.
- The die movements' `.kqs` models: the names of their parts (a part
  named like a drop entry, rule 31); the dropped item's own model: its
  part `body` (rule 31a).
- `GAME/Kq/Connor.kq`: `stats`, `level`, `attackDist`, `minDamageFactor`,
  `maxDamageFactor`, `shootOffsets`.
- `<world>/kq/newworld.cs` (`applyCombatLevel 1`); projectile files
  (`Prscbolt.kq`).

## Edge cases

- A character whose `react` is playing can't start an attack (rule 7)
  and can't flinch again (rule 23): a fast attacker can hit it every swing
  without it ever attacking back only if each hit lands while it can be
  cut short (walk, run, stop, an attack's first moment).
- Hand damage 0 or less (e.g. `sprgnbow`'s −3): the roll still gives at
  least 1, but with *D* = 0 no health is lost (rule 16.2). Negative *D*:
  the roll is still at least 1 and, *D* not being 0, the loss is at least
  1 (inferred from the traced steps; no Daventry melee character has it).
- Health exactly 0 is alive; a 0-damage hit that triggers death can't
  happen.
- A kill by a `Fire` hit leaves no corpse; an `Ice` kill leaves one only
  if `idedKqs` is set (all Daventry fighters set it).
- Drops happen when the dying starts, before the die movement; experience
  only when it ends. The exception is the weapon named like a part of
  the die model: it appears when the movement ends (rule 31a).
- Chickens, the pig and bats can be hit and killed like the others (5
  health, 3 protection; 0 experience). Being hit doesn't make them flee
  (rule 28).
- The `attack` fidget from an idle list never hits, but its end still
  pushes the next attack `attackDelay` away (`character-movement.md`
  rule 18).

## Do not reproduce

- **The 30° approach of a character hit while not alarmed** (rule 28):
  the original takes the angle of the line to Connor with a plain
  arctangent of dy/dx (it loses the half-plane) and then turns that line
  by its own angle plus 0.52 rad, so the point it heads for is at about
  twice the intended angle and can be far off to one side or behind it.
  Do instead: head for the point at Connor's distance, 0.52 rad
  (anticlockwise) off the straight line to him.
- **`checken2`** in the first-kill tip's exclusion list is a typo for
  `chicken2`. Do instead: exclude `chicken2` too (or keep it: it only
  changes when a tip shows).
- **The manual's "0.75 to 5"** for the big swing disagrees with
  `Connor.kq` (4). Do: use the data (4).
- Negative hand damage (data in `sprgnbow`): keep it as data; it is never
  used by a ranged character.
- **The swallowed swing** (rule 11b): a slip in the original (the stop
  rule meant for "before the attack is due" also fires while an attack
  has just started). **Decided by the user (2026-10-20): do not
  reproduce.** Do instead: while a moving character's `Attack` is
  playing, don't apply the moving state's stop rule (no switch to
  standing); it stays in its moving state until the swing ends (rule 10:
  then `walk` / `run` again, and the stop rule applies as usual, with the
  attack time now + `attackDelay`). So a chasing character that arrives
  when its attack is due **always swings in full at once**, whether
  Connor stands or walks. Everything else in rules 7–11 stays (a state
  change for any other reason still cuts the attack, rule 11).
- Keep the rest, including the asymmetric rolls (25–100 % on Connor,
  75–125 % on characters), the first attack being due at once, and the
  1.25 × reach against a Connor walking backward.

## How to check

**Our logs (B):** per attack: who, distance, angle off its heading,
Connor's back key, attack time before / after, the movement started; per
`Do`: hit or miss (why), *D*, *R*, *P'*, loss, health after; per hit
taken: flinch or ouch (and why not), hit count, chase timer, who got the
alarm with "run"; per death: kind, die movement, drops (names, place),
experience sent, corpse shape, the `Die` line to the script.

**In the original (user, by eye), Daventry:**

1. **First swing**: let `goblin22` chase Connor and stand still: does it
   swing as soon as it is within about 1,500 of him, without pausing
   first? Between swings, about 2 s (1 s swing + 1 s delay)?
2. **Walking backward**: as it swings, walk backward: it should still hit
   him unless he is more than about 1,875 away at the hit moment (half a
   second into the swing).
3. **Behind it**: circle round a swinging goblin: a swing that starts
   while he is behind or to its side should miss.
4. **Flinch**: hit a goblin with the fists: it flinches on most hits
   (about 3 in 5), never twice within about 2 s; Connor himself flinches
   rarely (about 1 hit in 8).
5. **Run after one hit**: hit a bone goblin that was walking after Connor:
   from then on it should always run, and keep chasing for longer.
6. **Death**: kill a goblin: it plays its death, items appear at its feet
   at once (not when it is down), a body stays; the experience meter
   moves only when the death ends.
   **Checked by the user in the original (2026-10-20):** the items
   fling upward from it and fall to the ground with a sound (corrected
   in rule 31); they can be picked up on the ground or in the air; the
   experience bar moves when the death animation ends (as rule 32).
   Also check 1: sometimes the goblin attacks at once, sometimes it
   makes the attack sound with no swing and then attacks normally; while
   Connor walks or runs away it attacks at once on reaching him
   (explained in rule 11b).
6a. **Dropped weapon** (rule 31a): kill a bow spriggan with the fists
   or a sword (not fire): its crossbow should not fly; when the death
   ends it should lie where the dying model let it fall, tilted as in
   the model, and stay there (no bounce, no land sound). In Dead City,
   a skeleton with a `BroadSword` drop: the coins and mushrooms fly at
   once, the sword appears only at the end, where the death left it.
   If a skeleton dies against a wall or a pillar, the sword may be
   thrown instead (the line test). A bow spriggan burnt to death
   should throw its crossbow (its fire death has no `SmCrossbow` part).
   **Checked by the user in the original (2026-10-24):** the weapons
   are not thrown and lie where the death movement leaves them; the
   other items fly.
7. **Bow spriggan** at the windmill (`sprigwnd1`): it never leaves its
   spot, shoots about every 1.7 s while Connor is in view and within
   about 7,500, and leads its shot when he runs across.
8. **Combat level**: if the menus offer a difficulty, the same goblin
   should take visibly more hits on the hardest setting.

## Open questions

1. ~~The input side of Connor's attack (rule 39)~~: answered
   2026-10-20 in `ranged-weapons.md` (rules 1–20 and 32).
2. ~~The bolt's flight~~: answered 2026-10-20 in `ranged-weapons.md`
   rules 21–31 (the shooter's ranged damage at the shot; no gravity;
   removed on the first touch, below the floor grid or past
   `maxDistance`; aim 0.75 × the target's box top).
3. ~~Which menu sets the combat level~~: answered 2026-10-21 in
   `main-menu.md` rule 10 (New → "Combat difficulty": Easy 0.66,
   Normal 1.0, Hard 1.5). Still open: whether a character added by
   the `KQMonster::add` command gets the combat factor twice (once when
   added, once when it enters the world) (rule 3). Daventry's monsters all
   come from `monsters.kq` (once).
4. ~~Whether the dropped items are spread out~~: answered, they are
   thrown (rule 31, corrected after the user's check), except the
   weapon named like a part of the die model (rule 31a, Q169).
5. Connor's death and game over (rule 29), and the tips' texts (rules
   29, 33): other studies.
6. What "tied to another object" is (rules 28, 37); it is never set for a
   Daventry character as far as the data shows (not sure).
