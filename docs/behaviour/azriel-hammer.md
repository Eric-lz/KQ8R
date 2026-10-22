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
   - **The throw's `handsOff` keeps the panels and the pointer** (traced,
     2026-10-22, Q148 e): it is the "pointer kept" form, which skips
     everything a script's `handsOff 1` does to the screen: the bottom
     bar stays, no spinning mask, the pointer is not hidden (so
     characters still see him, `potions.md` rule 26). It only blocks
     keys and clicks. When it is switched off after `catch`, the keys
     still held are sent again (holding forward walks him off at once)
     and the traps get a "moved" check, as at the end of any `handsOff`.
   - **Requests refused during the throw** (traced, 2026-10-22, Q148 d):
     Connor keeps **one** waiting request; each refused request replaces
     the one before (the last one wins). When `catch` ends, the throw
     asks for standing; the waiting request is applied **first**, in
     place of standing, if the throw may go to it (standing, moving, a
     Special list, dying). So a hit that takes him below 0 during the
     throw makes him die right after the catch, and a script's Special
     list plays then. The general rule for the waiting request is in
     "Connor's waiting request" below.
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
   - **its distance passing `maxDistance`**: the distance is counted
     **before** the update's move; when the count passes `maxDistance`
     the hammer turns back **without making that move** (it stays where
     it was at the start of the update);
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
   its position **at the start of that update** (for all three causes:
   a touch, the range, the floor grid), its direction is **reversed**
   (it flies back along the line it came, it does **not** home in on
   Connor), its counted distance restarts at 0 and its range for the way
   back becomes **the distance it had counted minus one update's step**
   (speed ÷ 15). Because the count already includes the step of the
   turn-back update, this is exactly the distance from its start to the
   point where it was put back (corrected 2026-10-22, Q148 a). On the way back **only Connor** can stop it (its
   mask becomes Connor's type alone). The way back ends at the first of:
   - **touching Connor**;
   - its distance passing that range (counted the same way, before the
     move): it flies back **to its start point** and is caught in the
     update after it gets there (corrected 2026-10-22: not one step
     short). With rounding in the sums it may be caught one update
     earlier, one step short; either is fine;
   - reaching the floor grid.
   At that moment the hammer is **removed** and Connor is told it has
   come back: he plays `catch` (rule 3). So he always gets it back, even
   if it never reaches him.
   **Worked example** (Daventry: speed 24,000, `maxDistance` 15,000, one
   step 1,600 at 15 updates per second): the count passes 15,000 in the
   10th update (16,000), so it turns back after 9 moves at **14,400**;
   range back **14,400**; 9 moves back to the start, caught in the 10th:
   **20 updates, 1.33 s** from leaving the hand to the catch. Dead City
   (12,000): turns at 11,200, 16 updates, 1.07 s. Swamp (30,000 /
   20,000, step 2,000): turns at 20,000, **22 updates, 1.47 s**, which
   fits the user's stopwatch (about 1.5 s) better than the old 1.33 s.
9a. **How the model is turned** (traced, data; Q148 b): `hammer.kqs`
   has two identical still frames and no movement, and the flight adds
   no spin. The model is turned to point **along its flight** when it
   leaves (as a bolt), and turned to point along the reversed direction
   at the turn-back, so it simply flips round once.
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
| way back | reversed line, range = count − speed/15 = distance to the turn-back point | traced |
| caught | touching Connor, end of the way back (at its start point), or floor grid | traced |
| round trip | Daventry 20 updates (1.33 s), Dead City 16 (1.07 s), Swamp 22 (1.47 s) | traced, data |
| model | no spin; points along its flight, flipped at the turn-back | traced, data |

## Connor's waiting request (traced, 2026-10-22)

Applies to every state of Connor, not only the throw; written here
because Q148 needed it.

- When a state change is asked for and the current state refuses it,
  it is kept as Connor's **waiting request** (one slot, with its
  movement-list name if any). A later refused request **replaces** it,
  except as below.
- When a new request comes while one is waiting, the waiting one is
  tried **first**: if the current state may go to it, it is applied and
  the new one becomes the waiting one. This is skipped (the waiting one
  is dropped and the new one taken instead) when the new one is one of
  the "strong" states and the waiting one is not. Strong states:
  weapon swap, Special list, talk, rock throw, hammer throw.
- A waiting request is also tried again whenever a movement ends while
  Connor is standing (and in one other place not identified). Nothing
  else retries it, so in
  the hammer throw it is applied at the catch's end (rule 3).
- A Choice state set by a script overrides all of this (Q155,
  `characters.md`).

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
hand, "with a grain of salt": by the files it is 1.07 s in Dead City
and 1.47 s in Swamp, rule 9's worked example). Also: no throwing while jumping, falling, landing or
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
4. ~~The round-trip time~~: the user timed it **in Swamp** by hand with
   a stopwatch (about 1.5 s, a couple of hundred ms either way); the
   files give 1.47 s there (rule 9's worked example; first written as
   1.33 s, 20,000 out and back at 30,000), which is within that error.
   Settled: use the files.
5. ~~Q148 (a), (b), (d), (e)~~: answered 2026-10-22 (rules 3, 7, 9,
   9a; "Connor's waiting request"). (c) was settled by the user's check.
