# Connor's inventory (items he carries, picking one, using it)

Study session A, 2026-10-18. Sources: `Mask.exe` (studied, described in
our own words), `GAME/Kq/Invinfo.itl`, `GAME/Kq/Connor.kq`, world
scripts, and the manual `MASK/Manual.pdf` (pages 20–27). Items lying in
the world and picking them up: `items.md`. The panels that show the
inventory: `interface.md`. Clicking in the world: `clicking.md`.

Marks: **[traced]**, **[data]**, **[manual]**, **[inferred]**,
**[not traced]** as in `interface.md`.

## Purpose

Everything Connor carries is one list of item objects. Where an item
shows on screen, whether it stacks, whether it replaces another, and
what a click on it does all follow from its type's **category** in
`Invinfo.itl`. There is no separate inventory window: the inventory is
the top bar (quest items, gold, mask) and the bottom bar (armour,
weapons, health/potion slots, rope, rocks), see `interface.md`.

## Rules

### Item types (data)

1. `Invinfo.itl` columns used here [data]: `IDName`, `ID #`, `Hlth`,
   `SDam`, `LDam`, `Prot`, `Exp` (the five stat changes the item gives,
   in the stats order of `interface.md` rule 27), `Comb` (1 = several
   can share one stack), `category`, `GUIBitmap` (slot picture),
   `Message` (description line, `talking.md` rule 20), `Mode` (weapon
   mode when readied: `SwordMode`, `BowMode`, `NoModeChange`), `Dur`
   (how long the effect lasts, seconds; 0 = while carried; −1 = once),
   `useSnd`, `cursor` (pointer picture while held; `none` = can't be
   held).
2. **Categories** [traced names and grouping; counts from data]:

   | Category | Items (count) | Where shown | Stacks | Can be held as pointer |
   | --- | --- | --- | --- | --- |
   | `QuestItem` | 69 | top bar strip | no | if it has a cursor (67 do) |
   | `Money` | silver coins | top bar, gold | yes | yes (`goldcurs`) |
   | `MaskPiece` | 7 | top bar, mask button | no | no |
   | `HealthItem` | 8 (4 health, 4 potions) | 8 slots | yes | no (used at once) |
   | `ShortWeapon` | 13 | hand-weapon slot | no | no (readied instead) |
   | `LongWeapon` | 20 | ranged slot | no | no |
   | `Armor`, `Gloves`, `FootWear`, `Helmet`, `UnderArmor`, `FullArmor`, `GodArmor` | 15 | armour slot (number; picture for some, `interface.md` rule 35) | no | no |
   | `Rope` | 1 | bottom right | no | yes (`curs06`) |
   | `Rock` | 1 | bottom right, count | yes | yes (`rokcurs`) |
   | `MagicMap` | 1 | map icon | no | no |

   (`Pants` exists as a category but no item uses it.)
3. **Which places overlap** [traced]: a full suit (`FullArmor`) covers
   body armour, pants, gloves and boots, **not the helmet**; an
   under-armour suit (`UnderArmor`, the chain mail suit) covers pants,
   gloves and boots; `GodArmor` has its own place. Each of
   `ShortWeapon`, `LongWeapon`, `Armor`, `Gloves`, `FootWear`, `Helmet`
   is one place.
4. **Connor's start** (`Connor.kq` `[ConnerInventory]`) [data]:
   `INVITEM_Piece1A` (a quest item: the first mask piece of the story),
   `INVITEM_Fists`, `INVITEM_NoRanged`, `INVITEM_NoArmor` (placeholders
   that fill the three slots with their empty pictures and give no
   stats), `INVITEM_Mask1A` (mask piece shown on the mask button).
   Each section also has `userId=-1` (rule 15).

### Quest strip: order, scrolling

5. Quest items (and only they) are listed in the strip **in the order
   they were added** [inferred: a plain list appended on add; not seen
   sorted anywhere].
6. The strip shows 7 at a time from a **first-shown index**. The left
   arrow lowers it by 1, the right arrow raises it by 1; it is then kept
   between 0 and (count − 7), and at 0 when there are 7 or fewer
   [traced].
7. **The item held as the pointer is left out of the strip** while it
   is held; the items after it move up one slot [traced]. (The limit in
   rule 6 still counts it, so with 8 items, one held, scrolled to the
   end, the last slot is empty.)

### Adding an item (picking up, `give`, chests)

8. When an item goes into Connor's inventory [traced]:
   - **Stacking** items (`HealthItem`, `Money`, `Rock`): if he already
     has an item of the **same type** and the type has `Comb` 1, the new
     quantity is added to it and the new item object is deleted.
     Otherwise it is added as its own entry.
   - **Quest items, mask pieces, rope, map**: always added as a new
     entry, no merging.
   - **Equipment** (weapons and armour categories): every item already
     carried that **shares a place** with it (rule 3) comes off:
     placeholders (`INVITEM_Fists`, `INVITEM_NoRanged`,
     `INVITEM_NoArmor`) are deleted; any other old item is **put into
     the world at the new item's position** (so it lies where the new
     one was picked up). Then the new item is **put on**: its stats are
     added to Connor's (rule 13). Example: wearing the full suit and
     picking up gloves takes the whole suit off (it drops) and puts the
     gloves on; the manual says the same [manual].
   - Then, for every add: all panels are refreshed (`interface.md`); a
     ranged weapon sets Connor's ranged weapon mode; a hand or ranged
     weapon re-readies his weapon if one is drawn [inferred]; the type's
     **pickup sound** plays (`items.md` rule 7); `INVITEM_TempleArmor`
     switches Connor's animation set (`contpsd2.anm`, `conicbw2.anm`).
   - A new quest item or mask piece pops the top bar down for 1.5 s and
     its slot blinks (`interface.md` rules 13, 44).

### Clicking an item in the bars (left button)

9. A left click on a slot sends "this item was chosen" to Connor. It is
   **ignored during `handsOff`** and while Connor is dead [traced].
   What happens depends on the category [traced]:
   - **Quest item or money**:
     - if it is the item already held: it is **put back** (pointer back
       to the play pointer, the item reappears in the strip);
     - `INVITEM_Scroll` can't be taken: Connor says the line `0.MSG`
       (0, 92, 14, 0) instead, at most once every 3 s and not while he
       talks;
     - otherwise, if the type has a cursor, it becomes **the item in
       hand**: the pointer turns into the type's cursor picture
       (`cursor` column, e.g. `INVITEM_Candle` → `candlcur.pba`,
       coins → `goldcurs.pba`) and the item leaves the strip (rule 7);
       a type with cursor `none` (`INVITEM_AntiPoisonFlower`) does
       nothing.
   - **Rock**: held like a quest item, except in three special states
     (puzzle states; **not** the rope, corrected 2026-10-21,
     `ropeandhook.md` rule 38), where Connor says "I have no
     time to trifle with that." (`0.MSG` 0, 0, 0, 0, seq 3).
   - **Rope**: if Connor isn't at a rope place, he says `0.MSG` file 0,
     noun 0, verb 58, case 37 "I am not near anything to climb." and the
     rope is **not** held; at a rope place it is held like a quest item and
     the next click in the world starts the climb (**corrected
     2026-10-21**: not "thrown at once"; see `ropeandhook.md` rules 8–12).
   - **Everything else** (weapons, armour, health items, potions, mask
     piece, map): **used at once**, nothing is held: the item is
     "applied" (rules 13–16) and the type's `useSnd` plays if it wasn't
     already applied.
   - Clicking a **hand weapon** slot also readies the weapon (draws it):
     the slot's weapon mode is asked for (`SwordMode`; `INVITEM_Fists`
     also has `SwordMode`, so Connor readies his fists). The ranged slot
     does the same with `BowMode`, but only when a bow is carried
     (`INVITEM_NoRanged` has `NoModeChange`, so nothing happens). Fighting is M5 [traced the request;
     what it does is M5].
   - The mask button opens the main menu instead (`interface.md` rule 41).
10. **Right click** on a slot: Connor describes the item
    (`talking.md` rule 20, built as sound only). Right click on an empty
    slot does nothing [traced].
    **`invClick <itemType> <N> <V> <C> <S>`** (traced 2026-10-28, batch
    2) changes the message numbers (noun, verb, condition, sequence,
    each kept as 0–255) Connor uses for that right click. **The original
    ignores the type word and always changes `INVITEM_Feather`** (the
    word appears only in its error text). Do instead: look the item up
    by name; the data's only use, Dead City `reaper.cs`
    `invClick INVITEM_Feather 76 0 0 1`, gives the same result (and
    those are already the Feather's numbers in `Invinfo.itl`, so nothing
    visible changes).
11. Keys 1–8 act as a left click on the 8 health/potion slots
    (`interface.md` rule 40).

### The item in hand, used on the world

12. With an item in hand, a left click in the world is the click of
    `clicking.md` rules 12–18 (accept lists, `ConnorAction` with `$3` =
    item object name, `$4` = type) [traced]. Then:
    - **accepted**: the hand is **emptied** and the pointer goes back
      to the play pointer, **except** for money while the script
      command `goldCoinPersists 1` is on (Gnome's apothecary,
      `apothgnm.cs`, turns it on and off around buying), when the coins
      stay in hand. The item itself is **not** removed by this: the
      script decides (`deleteObject $3`, as the wizard does);
    - **not accepted** ("This will not work."): the item **stays in
      hand**;
    - **out of reach**: "too far" (or the `TooFar` call), the item stays
      in hand.
    - Clicking the held item's own slot (or the empty place it left)
      puts it back (rule 9).
    - `goldCoinPersists 0` also empties the hand at once and resets the
      pointer [traced].
    Not traced: whether moving, a cutscene or a world change empties the
    hand. **The user (2026-10-18):** if the hand is emptied, the item
    only goes back to its bar slot; **no item is ever lost** that way.
    Items leave the inventory only when used (a puzzle or quest takes
    them, or a bottom-bar item is consumed).

### Applying an item (equipment, potions, health items)

13. Each carried item remembers **whom it is applied to**: the data
    calls it `userId` (−1 = not applied). This is the "third number"
    of `items.md` open question 1 [traced].
14. **Applying** an item to Connor (when equipment is put on, or a
    health/potion slot is used) [traced]:
    - only if it isn't applied already;
    - stacking items lose 1 from their count;
    - its five stat numbers (`Hlth SDam LDam Prot Exp`) are **added** to
      Connor's stats through the normal stats event (`interface.md`
      rules 24, 27–30; health is capped at the maximum);
    - `Dur` 0 (all equipment): stays applied while carried;
    - `Dur` > 0 (potions 15–60 s, the candle 60 s): a timer runs; at its
      end the item is un-applied (rule 15);
    - `Dur` −1 (the four health items): applied and at once un-applied,
      but **without** taking the stats back, so the heal stays.
15. **Un-applying** [traced]: for `Dur` ≥ 0 the same five numbers are
    **subtracted** again; `userId` goes back to −1; a stacking item
    whose count is now 0 is removed from the inventory.
16. **Quest items**: applying one first un-applies any other applied
    quest item, so only one quest item is "on" at a time [traced]. A
    bar click never applies a quest item (it is held instead, rule 9),
    so applying one comes from elsewhere (scripts or other code; not
    traced). Only the candle has a duration and stats among quest items
    (`Exp` 25, 60 s). **Corrected (`potions.md` rule 40):** the candle
    is never applied by the bars, so these two columns are not used; its
    25 experience comes, built in, when it is picked up and is never
    taken back.
17. Taking an item out of the inventory removes the given count; if
    that is all of it, the item is un-applied first and leaves the list
    (to the world, for `dropItem`, `items.md` rule 11) [traced].

### Potions (for later)

18. The four potions put a timer icon in the status icons
    (`Invinfo.itl` `extraData`: `IDBMA_CLAR`, `IDBMA_INV`, `IDBMA_VUL`,
    `IDBMA_STR`) for their `Dur` [data, manual]. Their real effects
    (shield, strength, reveal, invisibility), the timer and the red
    numbers: **`potions.md`**.

## The wizard's quest items (Daventry)

19. Handing the wizard an item is a **plain click with the item in
    hand** [data, `daventry/kq/wizard.cs`]: `wizard.kq` says
    `acceptsInventory=INVITEM_All`, so every item is accepted and his
    script gets `ConnorAction` with `$4` = the type. For the ring of the
    dead hero, the candle and the ashes it checks
    `Daventry::Wizard::metConnor == 2`, then `clearSpeak`, `handsOff 1`,
    `deleteObject $3`, sets `hasRing` / `hasCandle` / `hasTorch`, and
    answers with **`KQConner::conversation`**; other items (and those
    three before the right moment) get a `KQConner::conversation` reply
    too. `metConnor` becomes 1 and 2 only at the end of conversation
    lines (`TalkComplete` of 1000/4/14/27 seq 13 and 1000/4/14/30
    seq 5).
    So: **the inventory alone is enough to deliver the items** (the
    item leaves, the variables are set), but **what the wizard says and
    the state that allows the hand-over need conversations**
    (`KQConner::conversation`, not studied; `talking.md` open
    question 1).

## Asking what Connor carries in each place (`getConInventory`)

Study session A, 2026-10-24 (Q166). Used by Daventry's bell
(`bell.cs`), the windmill (`miltrap1.cs`, `wtrmwalk.cs`), Barren's
weapon seller (`tw_weird.cs`, `myweapon.cs`) and Gnome's armour
scripts.

20. `getConInventory` takes no arguments. It sets **nine** script
    variables, one per place, in this order [traced]:

    | Variable | Category looked for |
    | --- | --- |
    | `ConInv::Long` | `LongWeapon` |
    | `ConInv::Short` | `ShortWeapon` |
    | `ConInv::Armor` | `Armor` |
    | `ConInv::FullArmor` | `FullArmor` |
    | `ConInv::UnderArmor` | `UnderArmor` |
    | `ConInv::Pants` | `Pants` |
    | `ConInv::Gloves` | `Gloves` |
    | `ConInv::Helmet` | `Helmet` |
    | `ConInv::Boots` | `FootWear` |

    There is no variable for `GodArmor` or any other category.
21. **The value** is the `ID #` column (`Invinfo.itl`) of the item
    Connor **carries** whose category is **exactly** that category,
    written as a whole number (for example `1020` for
    `INVITEM_SmallAx`). If he carries none, the value is `0`
    [traced]. The search goes through his one item list in order and
    takes the first match; since equipment replaces whatever shares its
    place (rule 8), there is at most one [traced].
22. **It does not look at what is drawn.** It reads what he carries,
    whatever the weapon mode, and whether a weapon is out or not
    [traced]. (The bell checks the mode itself first with
    `KQConner::getDoMode` [data].)
23. **Placeholders count as items** [traced + data]: at the start
    Connor carries `INVITEM_Fists` (1280, `ShortWeapon`),
    `INVITEM_NoRanged` (1310, `LongWeapon`) and `INVITEM_NoArmor`
    (2120, `Armor`), so with nothing picked up `Short` = 1280,
    `Long` = 1310, `Armor` = 2120, and the other six are 0. Once a real
    item of that place is picked up the placeholder is deleted
    (rule 8) and the slot gives the new item's number.
24. **Exact category, not overlap** [traced]: a full suit (2080, 2100
    or 2130, category `FullArmor`) is reported only in `FullArmor`;
    `Armor`, `Gloves`, `Boots` then give 0 (wearing it took those off,
    `NoArmor` included, rule 8). The chain mail suit (2050) is reported
    only in `UnderArmor`. No item has the `Pants` category, so `Pants`
    is always 0 [data].
25. **Console output** [traced]: after setting the variables it prints
    nine lines, one per variable in the order above, as
    `ConInv::Long:       1310` (the variable name, a colon, padding to
    one column, the value). Nothing else changes: no stats, no panels,
    no sound.
26. **`$Console::Return` is always 0** [traced], also on failure. If
    Connor is missing it prints `getConInventory:  can't find Connor`;
    if he has no inventory, `getConInventory:  can't find Connor's
    inventory`; in both cases the variables are left as they were.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| quest strip | 7 slots, arrows ±1 | traced |
| health/potion slots | 8, fixed order | traced |
| Scroll refusal remark | at most every 3 s | traced |
| health items | Mushroom 5, Crystal 50, Sacred Water 100, Elixir 1,000 health | data |
| potion durations | Clarity 60 s, Invisible 15 s, Invulnerable 20 s, Strength 20 s | data |
| Candle | `Exp` 25, `Dur` 60 s (when it is applied was not traced; held items are not applied by the bar click) | data |
| Connor start | `stats=10 10 0 0 0`, `level=1` | data |

## Data used

- `GAME/Kq/Invinfo.itl`, `GAME/Kq/Connor.kq`.
- Scripts: `give`, `KQMonster::inventory`, `dropItem`,
  `checkInventory`, `goldCoinPersists`, accept lists.
- `daventry/kq/wizard.cs`, `wizard.kq`.

## Edge cases

- A type with `cursor` none in a stacking or quest category can't be
  held: clicking it does nothing (rule 9).
- Two items of one place picked up at once (a script `give` of a suit
  while wearing gloves): the gloves drop at the suit's position, which
  for a `give` (no position) is wherever the new item object stands
  [inferred; not seen in data].
- `NoArmor` gets a special mark when a non-body armour piece is put on
  while it is worn [traced; meaning not traced, probably so the empty
  armour picture is kept]. No visible effect known.

## Do not reproduce

- Nothing found yet.

## How to check

**Check 4 (both parts) was done by the user in the original on
2026-10-18: correct.** Check 5: from the user's memory, holding an
item closes the gap in the strip (not checked with 8 or more items).

1. **Our logs (B):** per inventory change: type, count before/after,
   merged or new, items taken off (deleted or dropped, with position),
   stats before/after; per slot click: category, result (held, put
   back, used, refused, remark); per world click with an item: accepted
   or not, hand emptied or kept.
2. **Daventry, in our build:** take the farm coins (or `--item`): one
   gold entry, the picture `2gold1`/`2gold2`/`2gold3` by count. Click
   the gold: the pointer is the coins; click a door that doesn't accept
   coins: "This will not work." and the coins stay in hand; click the
   gold slot again: pointer back.
3. **Daventry:** open `smlchst*` with mushrooms (`give mushroom 5`
   baskets): slot 1 shows 5; key 1: health +5 (capped at 10 × level),
   count 4.
4. **In the original (user):** pick up a weapon or armour while
   carrying one: the old one lies on the ground where the new one was.
   With an item in hand, click on something that refuses it: the item
   stays on the pointer.
5. **In the original:** with 8 or more quest items, hold one: the strip
   closes the gap.
6. **`getConInventory` (rules 20–26):** in our build at the Daventry
   start, `getConInventory` from the console should log `Short` 1280,
   `Long` 1310, `Armor` 2120, the rest 0. After `give SmallAx`:
   `Short` 1020. Then the bell (fists put away, small axe out, three
   hits) rings. In the original, the bell rings only with the small axe
   [data, `bell.cs`]. **Checked by the user in the original
   (2026-10-24):** with the small axe drawn, the bell rings at each hit
   and the third starts the Lake Sword scene.

## Open questions

1. Whether a held item goes back to its slot on world change, cutscene
   or death (rule 12). Either way it stays in the inventory (the user).
2. ~~What applying a quest item does beyond the stats (the candle)~~:
   the bars never apply one; the candle's experience: `potions.md`
   rule 40.
3. Weapon readying and the attack pointers: M5.
4. The rope and rock throwing: M6.
