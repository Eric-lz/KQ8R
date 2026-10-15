# Items in the world and picking them up

Study session A, 2026-10-13. Sources: `GAME/Kq/Invinfo.itl`, Daventry's
scripts, and `Mask.exe` (studied, described here in our own words). What
Connor's inventory does with an item once he has it (stacking, limits,
using, the inventory screen) is **M4** and not covered here.

## Purpose

Items (`KQInventoryItem`) are the coins, crystals, mushrooms, weapons and
quest objects Connor collects. In the world an item is a small clickable
object; clicking it in reach puts it into Connor's inventory. No world
file lists items lying around at the start (every world's
`worldinv.kq` says `numItems=0`): items get into the world through
chests (`chests.md`), world scripts, and things that drop or throw them.

## Rules

### Item types (data)

1. `GAME/Kq/Invinfo.itl` lists all 135 item types, one line each, tab
   separated: class, `IDName` (`INVITEM_Crystal`), id number, health,
   short and long damage, protection, experience, combat flag, category
   (`ShortWeapon`, `Armor`, `HealthItem`, ...), inventory picture,
   message (4 numbers), mode, duration, display flag, **land sound**,
   **pickup sound**, use sound, **shape** (`crystl.KQS`), cursor,
   **radius**, **translucency**, extra data. For M3 only the name, shape,
   sounds, radius and translucency matter.
2. **Type names in scripts and data** may leave out the `INVITEM_`
   prefix (`give mushroom 5`, `invItemTypeName=silverCoins`): if a name
   doesn't start with exactly `INVITEM_`, that is put in front; the name
   is then matched against the list **ignoring capitals**.
3. An item object carries: its type, a **quantity** (one object can be a
   stack, e.g. 200 silver coins), and a third number (default −1; meaning
   not traced, likely uses/charges, M4).

### An item lying in the world

4. Defaults of every item object: collision type **0x220** (clickable
   0x20, plus 0x200), mask 0x809 (the ground, buildings, and type 1), a
   ball-stack collision shape like monsters', and `doNotPointAt` on
   (Connor does **not** turn to face an item he clicks).
5. It is drawn with its type's shape, at the place it was put. A placed
   item does not fall or settle by itself; only thrown items fly (rule
   12).

### Picking an item up

6. Picking, reach and "too far" are the same as for doors (`doors.md`
   rules 10, 11, 14): nearest clickable object under the play cursor, in
   reach within 2 × `attackDist` (4,700) of the item's box, otherwise
   Connor's stock "too far" remark and nothing happens. Holding another
   item: the item is asked whether it accepts it; items don't, so the
   stock "that doesn't work" remark (`doors.md` rule 13).
7. In reach, the item is **taken out of the world** and **added to
   Connor's inventory** (the whole stack). If that worked:
   - the item type's **pickup sound** (`pickupSnd`) plays, **not placed
     in the world** (a plain sound, same loudness wherever Connor is).
     This is done by Connor's inventory itself, so it plays for **every**
     item that goes into it: a direct pickup, a chest's second click, and
     `give` / `KQMonster::inventory Connor` from scripts (`none` = no
     sound). (**Corrected 2026-10-15:** the first version said the pickup
     sound was positional and played once per item; that was the land
     sound, next point.)
   - only for an item that was **thrown** (rule 12; it still carries its
     throw) and whose **land sound** (`landSnd`) has not played yet: the
     land sound plays at the item's place (full to 5,000, silent at
     25,000), once. (Traced; why the game does this is not clear, perhaps
     so a throw caught early still makes its landing sound.)
   - the item disappears from the world;
   - if the item object has a script, it then runs as a normal click
     (`$2` = `ConnorAction`, `$3` = `$4` = `do`).
   - Connor's inventory also does type-specific things on any add
     (weapons and armour get readied, one quest item changes Connor's
     animation set); that is M4 and was not studied.
8. Picking up takes no time and plays no Connor animation in this step
   (not traced whether Connor bends down; none was found in this code).

### Script commands that make or move items

9. **`KQMonster::inventory <who> <type> [quantity] [x y z [rx ry rz]]`**
   makes a new item object of that type (quantity default 1):
   - `<who>` = `world` (any capitals): it is put into the world at x y z
     with turn rx ry rz (a z of −1 / −2 = floor / surface height, as for
     `loadKQ`). Example: `KQMonster::inventory world SilverCoins 400
     137060 277085 11912 0 0 0`.
   - `<who>` = a character's name: it goes straight into that
     character's inventory. **`give`** is a built-in shortcut for
     `KQMonster::inventory Connor` (`give mushroom 5`).
   - Afterwards the variable **`NewInvItem`** holds the new item
     object's name (see "The name of a new item object" below). Unknown type: error message,
     `$Console::Return` 1; success: 0.
   - The type must be given **by name**; an id number is not accepted
     here (traced). Compare rule 10.
   - **The name of a new item object (Q60, traced).** Items made by the
     game (this command, `give`, a chest's first click, `tossItems`) get
     no name of their own. The first time anything needs the name, the
     object is named **`NoName<n>`**, with `<n>` taken from **one counter
     for the whole game**, which then goes up by 1. The same counter names
     every other nameless object too, not only items, so `<n>` can't be
     worked out from the number of items alone.
     - The counter starts at **0** when the game program starts and is
       **not** reset when Connor goes to another world.
     - It is **kept in saved games** (in the game state, as `nextNoName`)
       and put back when a saved world is restored (`mask.cs` loads
       `gameState.kq` for that; its comment says "reset the NoName
       counter").
     - **Clashes:** no data file names an object `NoName…` (checked all
       worlds), so only a script could cause one. What the game does with
       two objects of one name was not traced.
     - **The name stays with the object** when it is carried, moved
       between inventories, thrown by `tossItems` (the same object flies),
       or put down by `dropItem` (the same object is moved). Exception,
       traced but not tied to a type: for one item category `dropItem`
       makes a **new** item object instead (new name).
     - For us: `NoName<n>` with one game-wide counter starting at 0, saved
       with the game (M6). B's stand-in works until saving exists; switch
       when convenient.

10. **`KQMonster::checkInventory <who> <type> [variable]`**: is there an
    item of that type in `<who>`'s inventory (or in the world, for
    `world`)? `$Console::Return` = **1 if found, 0 if not** (and 0 on any
    error). With a fourth word, that variable is set to the found item
    object's name. Note: here 1 means yes, unlike `test`.
    **The type may also be an id number** (Q62 b, traced): if the word
    is not a type name, it is read as a number and matched against the
    id numbers in `Invinfo.itl` (Gnome `ARMORGNM.CS` uses `2050`). Other
    places that look a type up by id exist in the exe; only this command
    was checked. `KQMonster::inventory`, `give` and accept lists take
    names only.
11. **`KQMonster::dropItem <who> <type> x y z rx ry rz`**: takes an item
    of that type out of `<who>`'s inventory and puts it in the world
    there. Errors if `<who>` doesn't carry one (usage text; not traced
    further). Used by `GAME/Kq/rockHit.cs` (`dropItem Connor
    INVITEM_Rock $KQObject::loc`).
12. **`KQObject::tossItems <object> <offsetZ> <type1,type2,...>`**:
    makes one item (quantity 1) per listed type (comma separated,
    `INVITEM_` prefix optional) and **throws** them all out of the
    object, from its position raised by `offsetZ`. Daventry: a smashed
    barrel (`brlupcr.cs` on `Die`: delete the barrel, load the broken one
    in its place, then `tossItems $ObjectName 0
    Crystal,Crystal,Crystal,Crystal`). Each
    thrown item gets its own flight:
    - a random direction all round (0–360°),
    - a steep launch: 75°–85° above the horizontal,
    - a launch speed of 5,000–6,500 units per second,
    - it falls back under gravity (the item mover's value looks like
      8,085 units/s²; not fully traced) and **lands when it comes down
      to the floor-grid height**, where it stays; the type's **land
      sound** plays then.
    The exact flight formula (how time steps through it) was not fully
    traced; the numbers above are the launch values.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| item collision type / mask | 0x220 / 0x809 | traced |
| reach | 4,700 (as doors) | traced |
| pickup sound | plain (not placed in the world), on every add to Connor's inventory | traced |
| land sound when a thrown item is picked up | full to 5,000, silent at 25,000 | traced (values) |
| toss direction | random 0–360° | traced |
| toss elevation | 75°–85° | traced |
| toss speed | 5,000–6,500 units/s | traced |
| toss gravity | about 8,085 units/s² | traced value, use not fully traced |

## Data used

- `GAME/Kq/Invinfo.itl` (types, shapes, sounds, radius, translucency).
- Scripts that call `KQMonster::inventory`, `give`, `checkInventory`,
  `dropItem`, `KQObject::tossItems`.
- Chests' `invItemTypeName` / `invQuantity` / `invLoc` (`chests.md`).

## Edge cases

- `give` with a type that doesn't exist: error line, nothing given.
- `tossItems` on an object that doesn't exist: error line, nothing made.
- A type's `pickupSnd` of `none` plays nothing.
- **An item type whose shape has no box in `GAME/patch/BOUNDS.KQ`
  can't be clicked while it lies in the world** (Q62 c, traced): with no
  box it gets no collision shape at all (`clicking.md` rule 8; its
  `NoName…` name isn't in the table either). The type's `Radius` is not
  used for this. Three types are affected (B's check):
  `INVITEM_TruthKey` (`truthkey.kqs`), `INVITEM_MarbleTablet2`
  (`tablet2.kqs`), `INVITEM_Orcbow` (`orcxbow.kqs`). That matters only if
  one of them is ever put down in the world; a chest's item goes to
  Connor on the chest's own second click, without clicking the item.
- Many Daventry scripts (`basket*.cs`) give items directly (`give
  mushroom 5`) without anything appearing in the world.

## Do not reproduce

- Nothing found.

## How to check

- **In our logs:** for every item made: type, quantity, object name,
  where (world position or whose inventory), and `NewInvItem`; for every
  pickup: distance, Connor's count of that type before and after.
- **In the original:** smash a crystal barrel with a weapon: they
  should fly up steeply in random directions and land around it; click
  each to pick it up.

## Open questions

1. What the third number of an item (default −1) is (M4).
2. Whether Connor plays a pick-up animation (none found in this code).
3. The exact flight of a tossed item (rule 12).
4. What collision bit 0x200 in an item's type does.
