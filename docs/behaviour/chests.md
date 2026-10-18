# Chests

Study session A, 2026-10-13. Sources: the 10 `KQChest` files in all
worlds (Daventry: `smlchst3`, `smlchst4`, `wizchest1`, and the `jar`
template), their scripts, `GAME/Kq/Invinfo.itl`, and `Mask.exe` (studied,
described here in our own words). See `items.md` for what an item in the
world is and how Connor picks one up.

## Purpose

A chest is a clickable object that holds one stack of one item type. The
first click opens it and puts the item out; the second click takes the
item; after that the chest is empty and Connor says so.

## Rules

### Data

1. A chest's own keys: `invItemTypeName` (item type, with or without the
   `INVITEM_` prefix, any capitals: `silverCoins`, `INVITEM_Crystal`,
   `ladybell`), `invQuantity` (how many; default 1), `invLocX/Y/Z` (where
   the item appears), `invDirX/Y/Z` (its turn), and two optional remarks:
   `Take…` and `Empty…`, each six numbers `Msg`, `Noun`, `Verb`, `Case`,
   `Seq`, `End` (`smlchst3`: `EmptyNoun=109 EmptyVerb=14 EmptyCase=33
   EmptySeq=1 EmptyEnd=1`). The numbers pick a line from the message
   files (M4). No chest in the data has a `Take…` remark.
2. Defaults set by the chest itself: collision type **0x8021** (clickable
   0x20, solid 0x1, plus 0x8000), quantity 1, state "closed". A
   `colliderType` in the file replaces it (`wizchest1` uses 1: not
   clickable until its script adds 32).
3. Movements: `closed` and `open` (`loop=… 0.0 1.0 0 0`: from phase 0,
   forward, plays once). There is no `opened` movement; after `open`
   finishes the chest just stays on its last frame.

### Clicking (the play cursor, as for doors)

4. Picking, reach and "too far" work exactly as for doors (`doors.md`
   rules 10, 11, 14): nearest clickable object under the cursor; in reach
   when within 2 × `attackDist` (4,700) of the chest's box; too far →
   Connor's stock remark, nothing happens.
5. **With an item in hand** the chest is asked whether it accepts it
   (`clicking.md` rules 13–18); chests have no `acceptsInventory`, so the
   answer is no: Connor's stock
   "that doesn't work" remark and nothing else (`doors.md` rule 13).
6. **The chest's script is not run on a click.** A chest replaces the
   normal "`ConnorAction`" script call with its own three steps below.
   Its script (if any) still gets the other calls: `Init`, `EndLoop`
   with the movement name when a movement finishes (Swamp's `chest.cs`
   makes Connor speak on `EndLoop open`), `Cue`, hits.
7. **First click (closed):** the chest plays `open`. If it holds an item
   type, a new item object is made (`items.md`), of that type and
   quantity, and put into the world at `invLoc` with the turn `invDir`
   (a z of −1 or −2 means floor or surface height, as for `loadKQ`;
   none of the data uses that). The item does **not** fly out: it simply
   appears there (the data puts it just above or inside the chest:
   `smlchst3` 301 above the chest's base). The chest remembers the item
   object's name and is now "open". If the chest holds nothing, it goes
   straight to "empty".
8. **Second click (open):** if the remembered item still exists, it is
   **moved into Connor's inventory** (taken out of wherever it is, then
   added to him). The name is forgotten. Then, if the chest has a `Take`
   remark (its `Seq` is not 0), Connor turns to face the chest (unless
   the chest has `doNotPointAt`) and says it. The chest is now "empty".
   **Answers to Q61 (traced 2026-10-15):** (a) the item type's **pickup
   sound plays**: it is played by Connor's inventory on every add
   (`items.md` rule 7), not by the click, so taking from a chest sounds
   the same as a direct pickup. (b) The item object's own script is
   **not** run. (c) It is **the same add** into Connor's inventory as a
   direct pickup (same stacking, M4). The item's name is `NoName<n>`
   (`items.md` rule 9), given on the first click.
9. **Every later click (empty):** if it has an `Empty` remark, Connor
   turns to face the chest (same exception) and says it. Nothing else.
10. On the first click Connor does **not** turn to face the chest (the
    chest's own handler only turns him before a remark).
    **The remark's turn and wait** (traced 2026-10-18, Q103 e): the turn
    before a `Take`/`Empty` remark is the usual automatic turn, started
    in **any** state of Connor's (the chest doesn't check his state,
    unlike a `speak` or a click turn, `clicking.md` rule 28), and the
    remark then waits 0.5 s; with `doNotPointAt` there is no turn and the
    remark is queued at once (`talking.md` rule 19).

### Scripted chests (not `KQChest`)

11. Some chests are plain animated objects with a script instead:
    Daventry's `smlchst1` (the small wooden chest in the pub, by the keg)
    and `smlchst2` (at the farm). Their script on `ConnorAction`: if the
    world variable `Daventry::smlchstN::full` is `true`, set it `false`,
    put the coins into the world **next to** the chest with
    `KQMonster::inventory world SilverCoins 35 …` (300 units to the
    side), give the new coins a script (`Tavcoin.cs` / `Farmcoin.cs`:
    Connor makes a remark when he picks them up, then the script is
    removed), and play `open`. Otherwise Connor says line 109/14/33, the
    same "empty" remark as `smlchst3`'s. These need nothing beyond the
    script runner, `items.md` and the commands above.

### Checked by the user in the original (2026-10-13)

- **`smlchst3`** is the big chest on the pub's **second floor**. Click:
  it opens and shows a crystal. Clicking the crystal takes it into the
  inventory. Clicking the chest again: Connor says it's empty.
- **`smlchst1`** (small wooden chest by the keg): click: it opens and
  the coins appear **next to** it; click the chest again: Connor says
  it's empty.
- **Rules 7–9 confirmed (user, 2026-10-13, second try):** clicking only
  the chest: 1st click opens it and a crystal appears inside; 2nd click
  Connor takes the crystal (into the inventory); 3rd click Connor says
  it's empty. Exactly as traced.
- **Direct pickup case: solved (user, 2026-10-13, third try).** Clicking
  "the crystal" and then the chest also ended with "empty". The user's
  explanation is right: the crystal can't be clicked while it lies in the
  chest. `chest2.kqs`'s box (`BOUNDS.KQ`) is x −480…470, y −310…290,
  z 0…2,060; the crystal (`crystl.kqs`, x −60…70, y −80…80, z 0…290)
  appears at the chest's x, y and 301 above its base, so it sits **wholly
  inside the chest's box**. A click line to the crystal enters the
  chest's box first, and the nearest clickable box wins (`doors.md`
  Q45 b), so that click was the chest's second click (rule 8). Rules 7–9
  hold with no exception; **drop the stand-in below.** (For other chests
  an item placed outside the chest's box, e.g. `smlchst1`'s coins, can be
  clicked directly.)
- *(Superseded, kept for the record)* by rule 8, the first chest click
  after the crystal was picked up directly should say nothing (no `Take`
  remark) and only the click after that should give the `Empty` remark.
  The user heard "empty" on the next click. Either one click was missed,
  or picking the item up directly also empties the chest (not found in
  the code). **Stand-in for B: follow the user.** If the remembered item
  is no longer lying in the world, the click goes straight to "empty"
  and says the `Empty` remark. To re-check: open `smlchst3`, click the
  crystal, then click the chest exactly once and listen.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| default collision type | 0x8021 | traced |
| default quantity | 1 | traced |
| reach | as doors, 4,700 | traced (shared click code) |
| item placement | exactly `invLoc` / `invDir`, no toss | traced |

## Data used

- `KQChest` `.kq`: above keys, `animLoops` (`closed`, `open`), `script`,
  `colliderType`, `hidden`.
- `GAME/Kq/Invinfo.itl`: the item type list (`items.md`).
- Message files for the remarks (`talking.md`).

## Edge cases

- **The item can be clicked directly only if it lies outside the
  chest's click box** (for `smlchst3` it doesn't, see the user's checks
  below). Then it is an ordinary world item and Connor picks it up
  (`items.md`).
  The chest still remembers its name; the chest's second click then
  finds the item in Connor's inventory, takes it out and puts it back
  (no change), says the `Take` remark if any and becomes empty. Not
  checked in the game; our stand-in: if the item is already in Connor's
  inventory, the second click only does the remark and the state change.
- `wizchest1` (`hidden=1`, `colliderType=1`) can't be clicked or seen
  until `wizchest.cs` gets `Reveal True` (a Reveal potion, M4/M5), which
  unhides it and adds the click bit. `Reveal False` hides it again.
- `jar` is a template at (0, 0, 0) placed by a world script.
- A chest's state is saved with the game (`chestState`), as is the
  remembered item name (M6).

## Do not reproduce

- Nothing found.

## How to check

- **In our logs:** for each chest click: state before/after, the item
  type, quantity and object name made, where it was put; on the second
  click, Connor's inventory count for that type before and after.
- **In the original:** open `smlchst3`, the small chest **inside the
  pub** (tavern), next to the keg (crystals, at (202336, 263155)): the
  lid opens and the crystals appear on/above the chest; click the chest
  again: they go into the inventory; click again: Connor says it's
  empty. (`smlchst4`, 200 silver coins, is inside the castle keep, behind
  the locked keep doors.)

## Open questions

1. ~~Which line each remark is~~ Answered 2026-10-18 in `talking.md`
   (rules 1–4, 19): the six numbers are file, noun, verb, case, first
   and last sequence; `smlchst3`'s `Empty` remark is `0.MSG` 0/109/14/33/1
   "'Tis empty.".
2. What 0x8000 in the default collision type does.
