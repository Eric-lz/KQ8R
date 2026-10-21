# Azriel's Hammer: the thrown hammer that comes back

Study session A, 2026-10-20. Traced in `Mask.exe` unless marked "data"
(read from the game's files), "inferred" or "not sure". Builds on
`ranged-weapons.md` (readying, the ranged click, the projectile class);
this note covers what is different for `INVITEM_AzrielHammer`. Not
needed for Daventry (Connor gets the hammer in Dead City, from the altar
`altar_a1.cs`; Swamp's `constart.cs` gives it to him at the start of
that world). No question in `QUESTIONS.md` was open on this topic.

## Purpose

How Connor throws Azriel's Hammer: it is his ranged weapon, but instead
of shooting bolts he throws the hammer, which flies straight, hits one
thing, flies back the way it came and is caught. Enough for B to build it
when Dead City is reached.

Words used here as in `ranged-weapons.md` (update, weapon mode, ranged
slot item, bolt).

## Rules

### 1. Readying

1. The hammer is a `LongWeapon` with `BowMode` (data: `Invinfo.itl`, id
   1140, `LDam` 10, projectile file `Prhammer.kq`), so readying,
   switching and putting it away are exactly `ranged-weapons.md` rules
   1–12, with its own list `Conhamr.anm` (`Connor.kq` `LongAnimList`
   `AzrielHammer`): `away` (`ch_shth`, `Hamraway.aud`), `out`
   (`ch_draw`, `hamrout.aud`), and the throw's movements `attack`,
   `wait`, `catch` (data). The pointer is the bow pointer.

### 2. The throw click

2. A left click with the hammer ready goes through `ranged-weapons.md`
   rules 13–18 unchanged (an item on the pointer first, Ctrl, the talk
   check, the aim point, the steepness limit). Then, **instead of the
   shot** (traced):
   - in third person he **turns toward the aim point** (first person: no
     turn), but there is **no waiting for the turn** (the 45° rule of
     `ranged-weapons.md` rule 20 does not apply): the throw starts at
     once;
   - he goes into the **hammer-throw state**, and the item on the
     pointer, if any, is let go;
   - there is **no moving throw**: walking or running, a click also
     starts the throw state;
   - **no throw while jumping, falling, landing or stumbling**, and the
     click is **not kept for later** (the user's check in the original,
     2026-10-20; the click itself has no state test, so it is the state
     change that is refused there). Treat it like `ranged-weapons.md`
     rule 5: throw only from standing or moving.
3. **The hammer-throw state** (traced):
   - on entering, **`handsOff` is switched on** (no keys, no clicks, no
     pointer change) and **`attack`** plays (`ch_atack.kqs`, 19 frames;
     `hamrtoss.aud` at 0, the `do` cue at 0.579);
   - at the **`do` cue** the hammer leaves (rule 5);
   - when `attack` ends, **`wait`** plays (`ch_wait`, repeating);
   - when the hammer comes back (rule 9), `wait` is stopped and
     **`catch`** plays (`ch_catch`, `Hamrctch.aud` at 0);
   - when `catch` ends: `handsOff` off, back to standing.
   - The state can't be left before `catch` has ended (then only to
     standing, moving, a Special list or dying).
4. Only one hammer is in the air at a time (follows from rule 3: he
   can't click again until he has caught it).

### 3. The hammer in flight (`KQHammer`)

5. **The start** (traced): as `ranged-weapons.md` rules 21–25: damage
   is Connor's ranged damage at the throw (rule 8 adds to it); it starts
   at his position + his turn × the place of the model part named in its
   file (`meshName=hammer`) in the frame playing at the `do` cue, else
   his `shootOffsets` line `INVITEM_AzrielHammer -41 -605 1226`; it is
   aimed at the aim point (pitched), with no lead. The throw model's
   `hammer` part (data, `Ch_atack.kqs`) is present up to frame 10 of 19
   and gone after; at frame 10 it is at about **(−804, 234, 1,109)**
   (804 to his right, 234 behind him, 1,109 up). The `do` cue at 0.579
   falls at frame 10.4. **The user saw the hammer leave from in front of
   Connor** (2026-10-20), so the game reads the following frame, which
   has no `hammer` part: **use the fallback, (−41, −605, 1,226): 605 in
   front of his feet, 1,226 up.** The hammer flies at the aim point from
   there.
6. **Its file** (data, `<world>/kq/prhammer.kq`, all `classType=KQHammer`,
   `shapeName=hammer.kqs`, `meshName=hammer`, no shadow):

   | World(s) | Speed | `maxDistance` | `modifier` | `minDamage` | `maxDamage` |
   | --- | --- | --- | --- | --- | --- |
   | Daventry | 24,000 | 15,000 | 30 | 0 | 300 |
   | Dead City | 24,000 | 12,000 | 30 | 0 | 300 |
   | Swamp | 30,000 | 20,000 | 30 | 0 | 300 |
   | Barren, Ice World | 24,000 | 20,000 | — | — | — |
   | Castle of the Dead, Gnome, Snow Exit, Temples 1–4 | 24,000 | 12,000 | — | — | — |

   Defaults for missing keys (traced): `modifier`, `minDamage`,
   `maxDamage` **0**; speed 2,000 and `maxDistance` 8,000 as any bolt.
   Its collision type is 1 and its mask **0x221F** (a bolt's 0x1F plus
   0x200 and 0x2000).
7. **The way out** (traced): straight along its direction at its speed,
   no gravity, an ordinary collision move, its distance counted, as a
   bolt (`ranged-weapons.md` rule 28). It **turns back** (rule 9) at the
   first of:
   - **touching anything** its mask meets (objects, characters,
     buildings, …; Connor himself is ignored on the way out);
   - **its distance passing `maxDistance`**;
   - **reaching the floor grid** (at or below it after a move).
8. **What a touch does on the way out** (traced), before it turns back:
   - the Dead City objects **`lever1`** and **`tmblrcyl`** (the lever and
     its tumbler): the script `lever1.cs` runs with `lever1 ConnorAction`
     (as if Connor had used the lever: it drops the bridge; the script's
     own comment says so). No damage;
   - another **object or character** whose collision type has the
     "immune to projectiles" bit 0x4000: a hit of **0** (no `ProjImmune`
     line, unlike a bolt);
   - any other **object or character**: a hit of
     **`modifier` + Connor's ranged damage, then capped at `maxDamage`,
     then raised to at least `minDamage`**, kind `Projectile` (so
     `projdie` / `projdKqs` if it kills), then `fighting.md` rules 16–33
     for the victim;
   - a building or other surface: no hit.
   It hits **one thing only**: the first touch turns it back.
9. **Turning back and the way home** (traced): the hammer is put back at
   its last position before the touch, its direction is **reversed**
   (it flies back along the line it came, it does **not** home in on
   Connor), its counted distance restarts at 0 and its range for the way
   back becomes **the distance it had flown out minus one update's
   step** (speed ÷ 15). On the way back **only Connor** can stop it (its
   mask becomes Connor's type alone). The way back ends at the first of:
   - **touching Connor**;
   - its distance passing that range (so it stops about one step short
     of where it started, near his hand);
   - reaching the floor grid.
   At that moment the hammer is **removed** and Connor is told it has
   come back: he plays `catch` (rule 3). So he always gets it back, even
   if it never reaches him.
10. Daventry has `prhammer.kq` too (data), so the hammer works there if
    a script gives it (none does).

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| throw | no 45° wait; turns while `attack` plays; hammer at `do` 0.579 | traced, data |
| during the throw | `handsOff` on until `catch` ends | traced |
| movements | `attack` → `wait` (repeats) → `catch` | traced, data |
| damage | clamp(`modifier` + ranged damage) to `maxDamage`, then ≥ `minDamage`; kind `Projectile` | traced |
| defaults | `modifier`, `minDamage`, `maxDamage` 0 | traced |
| Dead City | speed 24,000, range 12,000, +30, 0–300 | data |
| collision | type 1, mask 0x221F out; Connor only back | traced |
| turns back at | first touch, `maxDistance`, floor grid | traced |
| way back | reversed line, range = way out − speed/15 | traced |
| caught | touching Connor, end of the way back, or floor grid | traced |

## Data used

- `GAME/Kq/Invinfo.itl` (`INVITEM_AzrielHammer`), `GAME/Kq/Connor.kq`
  (`LongAnimList`, `shootOffsets1`), `GAME/Kq/Conhamr.anm` (`attack`,
  `wait`, `catch`, `away`, `out`), `Ch_atack.kqs` (the `hammer` part),
  `<world>/kq/prhammer.kq`, `GAME/patch/BOUNDS.KQ` (`hammer.kqs`: about
  170 × 100 × 70), Dead City `lever1.kq`, `lever1.cs`, `tmblrcyl.kq`.

## Edge cases

- The hammer turns back from **any** touch, also a wall or a tree, and
  from the ground: a throw at the ground near him comes straight back.
- On the way back it passes through everything but Connor: it can't hit
  a second creature.
- Connor is in `handsOff` for the whole flight, so a long throw (up to
  about 0.5 s out and 0.5 s back in Dead City, plus the movements)
  leaves him unable to act; he can still be hit.
- A hit on `tmblrcyl` runs `lever1.cs` as if `lever1` had been hit
  (deliberate in the original: both are the lever).

## Do not reproduce

- **No damage in 9 worlds** (data with the traced defaults): Barren,
  Ice World, Castle of the Dead, Gnome, Snow Exit and the four Temples
  have no `modifier` / `minDamage` / `maxDamage` keys, so the cap is 0
  and every hit does **0** (no health lost, `fighting.md` rule 16).
  **Not reachable in normal play** (the user, 2026-10-20): in Swamp the
  player must trade the hammer for a bow to go on, and can't get it back,
  so the hammer is only ever used in Daventry, Dead City and Swamp. For
  now read the files as they are. **Later (M7, the user's idea):** give
  every weapon a default base damage, so late-game weapons brought early
  by cheat codes still work.
- The order "cap at `maxDamage`, then raise to `minDamage`" means a
  minimum above the maximum wins; no file does this. Keep it.

## How to check

**Our logs (B):** per throw: aim point, start offset (part found or
fallback), damage parts (modifier, ranged damage, result); per update:
position, distance, out / back; the touch (name, type, damage or
`lever1.cs`), the turn-back point, the way-back range; the catch (how:
Connor, range, floor grid) and the time from throw to catch.

**In the original (user, by eye), Dead City:**

**Checked by the user in the original (2026-10-20):** 1, 3, 4 and 5 as
described; 2 as described (the distance can't be judged by eye; the
round trip seemed to take about **1.5 s** after the hammer leaves his
hand, "with a grain of salt": by the files it is 1.0 s in Dead City
(12,000 out and back at 24,000) and 1.33 s in Swamp (20,000 at
30,000)). Also: no throwing while jumping, falling, landing or
stumbling, not queued (rule 2); the hammer leaves from in front of him
(rule 5). 6: see Do not reproduce (not reachable in normal play).

1. Throw at a creature: one creature hit, the hammer flies back along
   the same line and he catches it; he can't move until the catch.
2. Throw at nothing in particular: it flies about 12,000 and comes back.
3. Throw at the ground nearby: it comes back at once.
4. Throw at the lever by the bridge (`lever1`, or its tumbler): the
   bridge drops.
5. Two creatures in a line: only the first is hit.
6. In a world without the damage keys (e.g. Ice World), a hit does
   nothing to the creature (rule's Do not reproduce): worth checking
   before deciding.

## Open questions

1. ~~The start frame~~: settled by the user's check (rule 5: in front,
   the fallback offset).
2. ~~Refused states~~: settled by the user's check (rule 2).
3. ~~The worlds without the damage keys~~: not reachable in normal play
   (Do not reproduce).
4. The user's round-trip estimate (about 1.5 s) is longer than the
   files give (1.0 s Dead City, 1.33 s Swamp). Not enough to act on;
   B's log of the time from throw to catch can be compared by eye.
