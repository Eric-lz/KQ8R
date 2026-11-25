# Script language gaps and shared commands (Batch 1 & Batch 3)

Study session A, 2026-11-20. Sources: `Mask.exe` (studied in Ghidra and
described here in our own words) and the `.cs` scripts under `MASK/`.
Covers Batch 1 ("Shared / Everywhere") and Batch 3 ("Swamp") command items
from `docs/SCRIPT-COVERAGE.md` not placed in a subsystem note.

Related notes for other Batch 1 & Batch 3 items:
- **Batch 1:** Camera shake (`KQCamera::shake`): `camera.md` ("Shake and
  cutscene cameras"); Proximity haze (`KQHazer`), `newSky`,
  `resetHazeColor`: `object-lighting.md` (rules 23a–23e, 23, 31);
  `doPopup`, `stopHelp`, `notStartingHelp`, `KQGame::selectScreen`,
  `blackOut`, `loadPalette`: `interface.md` (rules 1a–1f, 3a, 53–59);
  `allowShadows`: `characters.md` (rule 36); World `Load`,
  `setLoadProgress`, `activatePersistMgr`: `world-change.md` (rules 8,
  24); `KQEnvInfo`: `water.md`; `KQCamMover`: `camera.md`.
- **Batch 3 (Swamp):** Swamp `Tile2` state, `KQSwampTile`, `fienfire*.cs`,
  and `Tile2` rock throwing: `tile-puzzle.md` (rules 28–41) and
  `ranged-weapons.md` (section 8e); Mandragore (`KQMandragorTree`,
  `KQMandragorRoot`): `fighting.md` (section 7b); `setFirstPerson`:
  `camera.md` (rule 8); `KQConner::pay`: `inventory.md` (rules 27–30);
  `KQConner::addImmunity` and `.kq` `immunities`: `water.md` ("Liquid
  damage" rule 5).

## Purpose

Explain the small script parser rules (`;`, quoting, case sensitivity, `.cs`
auto-execution fallback), shared console/gameplay commands (`inc`, `setcat`,
`alias`, `getConnorFlag`, `loadAnim`, `mouseAction`), and confirm which
commands in `world.cs` / `permwrld.cs` / cutscenes are pure loading or memory
hints with no visible gameplay effect.

## Rules

### Script console syntax and lookup

1. **`#` vs `;` (comments vs statement separator):**
   - `#` starts a comment to the end of the line.
   - **`;` is a statement separator** (identical to a newline), **not a
     comment**. A line starting with `;` produces an empty first statement
     followed by a second statement that **executes normally**.
   - **Data consequence:** the 11 lines in `Snowexit/worldinv.cs` starting
     `;KQMonster::inventory ...` execute in the original and spawn those 11
     mushrooms (even though the author likely tried to comment them out).
2. **Quoting and line limits:**
   - Single quotes `'...'` toggle variable (`$...`) and alias expansion off
     and back on.
   - Double quotes `"..."` group spaces into a single word while still
     expanding `$var` inside (unless disabled by single quotes).
   - Lines are limited to 512 bytes and at most 40 words per statement.
3. **Case sensitivity:** command names, variable names, and alias names are
   all **case-sensitive** (exact byte comparison).
4. **Unknown command → `.cs` auto-exec fallback:** when a command name is not
   in the built-in command table and `Console::AutoExec` is non-zero
   (default on), the console appends `.cs` (if the name contains no `.`) and
   runs that script file. Because Windows file names ignore case, both
   `getLoc` and `getloc` find `GAME/Kq/getloc.cs` (which calls
   `KQObject::getLoc $1`). If no `.cs` file matches either, the console prints
   `Console: Unknown command: <name>`, sets `$Console::Return` to **`1`**,
   and does nothing.
5. **Known harmless typos in shipped scripts:**
   - `DeadCity/skelking.cs:4` calls `KQMonster::clearstateChangeNotify`
     (lowercase `s`): fails command lookup and `.cs` lookup, sets
     `$Console::Return = 1`, does nothing (harmless because the Skeleton King
     is dying and about to be removed).
   - `Gnome/emptyloc.cs:10` calls `cls`: no built-in command or `cls.cs`
     exists; sets `$Console::Return = 1` and does nothing.
   - `Swamp/boiloil.cs:4` calls `beep`: no built-in command or `beep.cs`
     exists (`Mask.exe` has no `"beep"` string and never calls a beep
     routine), so it sets `$Console::Return = 1` and does nothing. In
     addition, `boiloil.kq` is commented out in `Swamp/newworld.cs` and has
     no `script=` key anyway.
6. **`Preload` in `IceWorld/icelord.cs`:** `Mask.exe` never sends a `Preload`
   event. That branch is invoked directly by Ice World scripts (`icelord.cs`,
   `lucreto.cs`, `sentry.cs` running `icelord.cs Icelord Preload`) to run
   `preloadShape` hints.

### Console variables and aliases

7. **`inc <varName>`:** requires exactly 2 words (else prints usage). Reads
   the variable's string as a base-10 integer (unset or non-numeric counts
   as `0`), adds `1`, writes the new decimal string back, and leaves
   `$Console::Return` `0`.
8. **`setcat <varName> [words...]`:** joins all words from the 3rd onward
   with **no spaces** (up to 512 bytes) and assigns the result to
   `<varName>`. With 2 words sets it to `""`; with fewer than 2 words lists
   all console variables. Returns `0`. Data: 3 uses, all in Barren
   (`dockup.cs`, `gsetup.cs`) to build gong trap and setup names.
9. **`alias [name [words...]]`:** with fewer than 2 words lists all aliases;
   with 2 or more words joins words from the 3rd onward with single spaces
   and stores a case-sensitive alias. Aliases expand **only in command
   position** (first word of a statement), up to 20 expansions per line.
   Data: 6 built-in debug aliases (`teleport`, `god`, `ungod`, `give`,
   `conCollide`, `noConCollide`), 6 in `console.cs`, and 45 developer warp
   shortcuts across `newWorld.cs` files.

### Gameplay and runtime commands

10. **`getConnorFlag <flagName>`:** requires 2 words (else prints an error
    and sets `$Console::Return` to `0`). Matches `<flagName>` **ignoring
    case** against Connor's 7 cross-world flags (the same flags set by
    `setConnorFlag` and preserved across worlds, `world-change.md`):
    - `BeenToCastle`
    - `MetGirlInDC`
    - `SageGnomeEarnedTeleport`
    - `SawSwampVision`
    - `KnowAboutFeather`
    - `InTemple4`
    - `UsedElevator`
    Sets `$Console::Return` to **`1`** if the flag is set, **`0`** if clear
    (or on an unknown flag name, which prints an error). Unlike
    `test`/`exist` (where `0` means true), callers check
    `if test $Console::Return == 1`. Data: Daventry `drawings.cs`,
    `girl.cs`, `horsie.cs` (`MetGirlInDC`); Temple 2 `cauldtp.cs`
    (`SawSwampVision`).
11. **`loadAnim` — spawns a live `KQAnimOnce`, not a loading hint:** dynamically
    creates a **`KQAnimOnce`** object (`anim-scripts.md` rule 11) in the
    world with `deleteWhenDone` initially `0` (so scripts follow with
    `setDeleteWhenDone $ObjectName 1` when they want auto-deletion), sets
    `$ObjectName` to its assigned name (`@` picks the next `NoName<n>`),
    replaces `z` of `−1` or `−2` with the ground height, and adds it to the
    world. Missing position/angle values default to `0`. Two forms selected
    by file extension (case-insensitive):
    - **`.kqs` or `.dts` (7–13 words):**
      `loadAnim <name> <file> <animPos> <timeScale> <cyclic> <interruptable> [x y z [rx ry rz]]`
      builds a single-movement list with movement name `"Stop"` (sequence name
      = file stem) and those loop settings.
    - **`.anm` (4–10 words):**
      `loadAnim <name> <file.anm> <loopName> [x y z [rx ry rz]]`
      loads the `.anm` movement list and starts on `<loopName>`.
    - **Data (20 uses):** Swamp `fienfire.cs` / `fienfir2.cs` / `fienfir3.cs`
      (`swmpgas.kqs`, followed by `setDeleteWhenDone $ObjectName 1` and
      `move $ObjectName 3 0 0 1000`), `GAME/Kq/swmpfrbl.cs` (`endfrbl.kqs`),
      Temple 4 `vortrap.cs` (`c_suck1/2`, `ci_suck1/2`, `cg_suck1/2`,
      `lu_suck1/2`, `c_imdone`, `vrtexbye`), Barren `anttest.cs`
      (`antdeath.kqs`), and `GAME/Kq/aload.cs`.
12. **`mouseAction <time> <x> <y> <click>`:** demo-playback command (exact
    5 words). Queues a cursor-move cue on Connor (immediately if `time ≤ 0`,
    else at `now + time`) that moves the OS pointer to screen coordinates
    `(x, y)`. If `click ≠ 0`, also queues a synthetic left-click event at
    `(x, y)` to Connor. Data: 2 active uses in the whole game
    (`Daventry/ax.cs`: `mouseAction 0 0 480 0` twice, `click = 0`, sliding the
    top bar away during the axe cutscene; plus a commented-out line in
    `wizard.cs`). See "Do not reproduce".
12a. **`KQConner::setRelDamage <fraction>` (traced 2026-11-20, Batch 3):**
    requires exactly 2 words (otherwise prints usage and returns `0`). Reads
    Connor's current level (`maximum health = level × 10`), parses
    `<fraction>` (`0.0` to `1.0`) as a decimal, and computes
    `d = trunc(−level × 10 × fraction)` (truncated toward zero). Sets two
    script variables:
    - **`$CurRelDamage`** = `d` (negative whole number passed to
      `KQStatsChangeRequestEvent`, e.g. `−2` at level 1 for `0.2`, `−5` for
      `0.5`);
    - **`$CurRelHealth`** = `−d` (positive whole number passed as the cue
      parameter to arrow shooters in `Swamp/arwtrap1–3.cs`).
    Leaves `$Console::Return` at `0`. Data: 8 active uses — Swamp
    `arwtrap1–3.cs` (`0.5`), legacy `fienfire/2/3.cs` (`.2`), Gnome
    `rollside.cs` (`0.2`), Barren `witchdoc.cs` (`1.0`).
12b. **`KQMonster::preSpecial <monsterName>` (traced 2026-11-20, Batch 3):**
    requires exactly 2 words (otherwise prints usage; unknown character
    prints `can't find monster`). Finds `<monsterName>`, turns ON its
    currently playing animation's **can-be-cut-short (interruptable)**
    flag, and sets its cached wanted movement to `"Stop"`. Scripts run it
    just before `KQMonster::setState <who> Special ...` so a
    non-interruptable movement in progress does not block or delay the
    `Special` animation. Leaves `$Console::Return` at `0`. Data: 8 uses —
    Swamp `swhnchmn.cs` (`Connor`), Gnome `armorgnm.cs`, `brikwall.cs`,
    `gnometlk.cs`, `weapngnm.cs`, Barren `chest1.cs`, `symbol.cs`, Ice World
    `grifnymp.cs`.
12c. **`conClickItem <objectName>` (traced and data, Batch 4):** requires
     exactly 2 words (`conClickItem <objectName>`); otherwise prints usage
     and sets `$Console::Return = 0`. Looks up `<objectName>` as a `KQObject`
     (if missing, prints `conClickItem: can't find object name <name>`,
     returns `0`) and looks up Connor (if missing, prints
     `conClickItem: can't find Connor`, returns `0`). Creates an empty-hand
     (`do`, no held item) `ConnorAction` event from Connor and dispatches it
     **synchronously** directly to the target object's event handler,
     bypassing reach checks, turning Connor, `handsOff`, and the event
     queue. Returns **`1`** (`$Console::Return = 1`) whenever both the
     target object and Connor exist. Data: 38 uses across Gnome (28, e.g.
     `armorgnm.cs`, `brnzplat.cs`, `chnmlst.cs`, `myarmor.cs` auto-picking
     up armor pieces from the armory stand) and Barren (10).
12d. **`cls` (traced and data, Batch 4):** built-in debug console command
     that clears the debug console's scrollback line buffer and returns
     `0` (`$Console::Return = 0`). Has no gameplay or visual effect; safe
     no-op returning `0`. Data: 1 use (`Gnome/EMPTYLOC.CS` line 10,
     developer leftover).
12e. **`useAlternateLightmap <interiorName> <0|1>` (traced and data, Batch
     4):** requires exactly 3 words (otherwise prints usage, returns `0`).
     Looks up `<interiorName>` as a `KQInterior` (prints
     `useAlternateLightmap: Could not find object <name>` if missing;
     silently ignores non-interiors) and switches its alternate lightmap
     flag (`0` = normal `.dil`, nonzero = `althwLightmapName` on hardware)
     with immediate lightmap reload and texture re-upload; returns `0`.
     Full rules in `object-lighting.md` rule 12. Data: 5 uses — Gnome
     `fusebox.cs` (`gnome13`, `gnome14` on/off) and `temple4/altar.cs`
     (`temple` on).

### Loading, cache, timing, and persistence hints

12f. **`KQObject::setGroupsActive3D <object> <flags>` and
    `KQObject::setGroupsActive3DFar <object> <flags>` (traced 2026-11-25,
    Batch 6):** sets the object's near (`groupsActive3D`) or far
    (`groupsActive3DFar`) active-group bitmask (integer parsed with
    `atoi`), and returns `0`. If the object is currently active (and far,
    for `setGroupsActive3DFar`), its live active groups are updated
    immediately (`groupsAlways | (groupsForced != 0 ? groupsForced :
    near | [far])`). Wrong argument count prints the usage line and
    returns `0`. Used by `IceWorld/snowmane.cs` (3 uses each) to make
    the invisible Snowmane visible (`1097` / `1088`) during a Reveal
    potion or on death, and invisible again (`9` / `0`) when Reveal ends.
12g. **`playAvi <filename>` (traced 2026-11-25, Batch 6):** plays a
    full-screen movie file from `<game>/<filename>` through the same
    movie player as `Replay Intro` (`submenus.md` section 8), with
    skipping disabled (`allowSkip = false`). Wrong argument count prints
    `usage:  playAvi filename`. Used by `Snowexit/PORTAL4.CS`
    (`w32mdl_1.dll`) and `temple4` (`w32ndg_1.dll`).
12h. **IceWorld `Preload` script call (data, Batch 6):** `Preload` in
    `IceWorld/icelord.cs` is **not** an engine script call — `icelord.cs`
    calls itself directly on line 88 (`icelord.cs Icelord Preload`) as a
    script subroutine.
12i. **`KQObject::setFootStepSound <obj> <int>` (traced 2026-11-25, Batch
    7):** Sets the object's footstep-sound index (`footStepSound`) to the
    parsed integer and returns `0`. Used by `temple1/sloshtrp.cs` to set
    `temple_e` to `3` (water/slosh footsteps) and back to `0` (default).
12j. **`KQCamera::setHeight2 <height>` and `KQCamera::setHeight <0|1>`
    (traced 2026-11-25, Batch 7):**
    - `KQCamera::setHeight2 <float>` stores an alternate follow-camera
      height (`camHeight2`, e.g. `2000` in `temple4/archon.cs`).
    - `KQCamera::setHeight <int>` switches the camera's active follow
      height (or zoomed follow height when zoomed) to the default
      `camHeight` when `0`, or to `camHeight2` when non-zero, then
      recomputes the follow camera position (`temple3/traphz1.cs`,
      `temple4/archon.cs`).
12k. **`KQConner::reduceHealth <fraction> <minFraction>` (traced
    2026-11-25, Batch 7):** Subtracts `trunc(level * 10 * fraction)` from
    Connor's health (the same level-scaled formula as `KQSpiriTrap`). If
    `(health - loss) / (level * 10)` would fall below `minFraction`, sets
    Connor's health to `trunc(level * 10 * minFraction)` instead. Applied
    as a plain health-change stats event (no attack, flinch, or
    protection) and returns `0`. Used by `temple4/altar.cs` (`0.5 0.02`,
    5 uses).
12l. **`killGame`, `doCredits`, and `checkSave` (traced 2026-11-25, Batch
    7):**
    - `killGame` marks the current game as ended and clears the running
      game state / pauses the world so `Save` and `Back to Game` are
      disabled on the Main Menu (`temple4/vortrap.cs` line 152).
    - `doCredits` (with no arguments) switches directly to the Credits
      screen (`screen 8`, `submenus.md` section 7). Used at the end of
      `temple4/vortrap.cs` after `killGame` and `playAvi w32ndg_1.dll`.
    - `checkSave` returns `1` if the player has saved the game at least
      once during this session, else `0`. Used by `Daventry/savegmh.cs`
      to show starting-help popup `#143` if the player has not saved yet.
12m. **`KQMonster::setState Connor Die [Now]` (traced 2026-11-25, Batch
    7):** Entering Connor's `Die` state zeroes his health and stops his
    speech/sounds. When the optional extra argument matches `"now"`
    (case-insensitive, used by `temple4/vortrap.cs` lines 131–137 along
    with `KQConner::reincarnate 0`), Connor **skips his death animation**
    and immediately finishes death (returning to the Main Menu). Without
    `"Now"`, he plays his normal death movement (`fighting.md`).
12n. **Remaining script-coverage stragglers (traced 2026-11-25, Batch
    7):**
    - `KQMonster::clearstateChangeNotify` (`DeadCity/SKELKING.CS`): simply
      a lowercase-`s` spelling of `KQMonster::clearStateChangeNotify`
      (commands match case-insensitively, rule 4).
    - `beep` (`Swamp/BOILOIL.CS` line 4): **not** a registered command in
      `Mask.exe` — leftover unknown command that safely sets
      `$Console::Return = 1` (rule 5).

13. **Pure memory/timing hints (confirmed safe no-ops for rendering and
    logic, aside from save tracking / loading bar):**

    | Command | Effect in `Mask.exe` | Rebuild behaviour |
    | --- | --- | --- |
    | `preloadShape <kqs>` | Preloads shape into cache and sets `letCatchUp` | No-op / log |
    | `preloadResource <name>` | Preloads named resource and sets `letCatchUp` | No-op / log |
    | `KQObject::preloadResources <obj>` | Warms that object's shape/animation cache | No-op / log |
    | `purgeResource <name>` / `purgeResources [arg]` | Frees unused cached resources | No-op / log |
    | `lockShape <kqs>` / `lockInterior <name>` | Pins shape/building against cache purges | No-op / log |
    | `KQGame::letCatchUp` | One-frame flag skipping catch-up delta after slow loads (33 uses: 29 Ice World + Daventry `watertp.cs`, Dead City `altar_a1.cs`, Gnome `brikwall.cs`) | No-op / log |
    | `activatePersistMgr <0\|1>` | Pauses (`0`) or resumes (`1`) persist manager save-diff tracking around `permwrld.cs` permanent loads | Save tracking only; no visible effect |
    | `setLoadProgress [percent [mode]]` | Advances loading meter (`world-change.md` rule 24): 0 args `+100 / totalSteps`, 1 arg sets percent, 2 args sets `totalSteps` and meter | Update loading meter if shown |

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| Console line limit / max words | 512 bytes / 40 words | traced |
| `alias` max expansions per line | 20 | traced |
| `setcat` max joined length | 512 bytes | traced |
| `stopHelp` mask | 7 bits (`0x7f`) | traced |
| `loadAnim` shipped uses | 20 scripts | data |
| `KQConner::setRelDamage` shipped uses | 8 scripts (`arwtrap1–3.cs` 0.5, `fienfir*.cs` .2, `rollside.cs` 0.2, `witchdoc.cs` 1.0) | data |
| `KQMonster::preSpecial` shipped uses | 8 scripts | data |
| `conClickItem` | 38 uses (Gnome 28, Barren 10); returns `1` on success | traced, data |
| `useAlternateLightmap` | 5 uses (Gnome 4, `temple4` 1); returns `0` | traced, data |
| `cls` | 1 use (`Gnome/EMPTYLOC.CS`); no-op returning `0` | traced, data |

## Data used

- `MASK/Snowexit/kq/worldinv.cs` (`;` lines), `MASK/GAME/Kq/getloc.cs`,
  `MASK/DeadCity/kq/skelking.cs`, `MASK/Gnome/kq/emptyloc.cs`.
- `MASK/swamp/kq/{boiloil,arwtrap1,arwtrap2,arwtrap3,fienfire,fienfir2,fienfir3,swhnchmn}.cs`.
- `MASK/IceWorld/kq/{icelord,lucreto,sentry,grifnymp}.cs` (`Preload`, `preSpecial`).
- `MASK/Barren/kq/{dockup,gsetup,witchdoc,chest1,symbol}.cs` (`setcat`, `setRelDamage`, `preSpecial`).
- `MASK/Gnome/kq/{rollside,armorgnm,brikwall,gnometlk,weapngnm,fusebox,brnzplat,chnmlst,myarmor}.cs`.
- `MASK/temple4/kq/altar.cs`.
- `MASK/daventry/kq/{drawings,girl,horsie,ax}.cs`,
  `MASK/temple2/kq/cauldtp.cs` (`getConnorFlag`, `mouseAction`).
- `MASK/swamp/kq/fienfir*.cs`, `MASK/temple4/kq/vortrap.cs`,
  `MASK/barren/kq/anttest.cs`, `MASK/GAME/kq/{swmpfrbl,aload}.cs`
  (`loadAnim`).

## Edge cases

- An unknown script command that also has no `.cs` file leaves
  `$Console::Return = 1`, which persists only until the next command updates
  `$Console::Return`.
- Calling `loadAnim` with `z` of `−1` or `−2` replaces `z` with the terrain /
  floor surface height just like `loadKQ`, and sets `$ObjectName` before
  subsequent lines such as `setDeleteWhenDone $ObjectName 1` and
  `move $ObjectName ...`.

## Do not reproduce

- **`mouseAction` moving the OS cursor** (rule 12): never warp the user's
  actual desktop mouse cursor in a modern rebuild; hiding the top bar during
  `handsOff` (or updating an internal UI cursor coordinate) satisfies the
  only shipped calls (`Daventry/ax.cs`: `0 0 480 0`).
- Remember **not** to treat `;` as a comment character (rule 1): if scripts
  are parsed faithfully, `;KQMonster::inventory ...` in `Snowexit` executes as
  in the original.

## How to check

- **Unit tests / script log (B):**
  - Parse a line starting with `;echo hi` and verify `"hi"` executes; check
    `#comment` still skips to end of line.
  - Test `inc x`, `setcat a b c` → `"bc"`, `alias` expansion only in word 0,
    and `getloc` finding `getloc.cs` regardless of case.
  - Trigger `fienfire.cs` in Swamp or run `loadAnim @ swmpgas.kqs 0 1 0 0
    100 100 -1` + `setDeleteWhenDone $ObjectName 1`: verify a `KQAnimOnce`
    spawns, plays `swmpgas.kqs` once, fires `Die`, and deletes itself.
  - Set `MetGirlInDC` via `setConnorFlag` and confirm `getConnorFlag
    metgirldc` sets `$Console::Return = 1` (case-insensitive).
  - Run `KQConner::setRelDamage 0.5` at level 1 and check `$CurRelDamage ==
    -5` and `$CurRelHealth == 5`; at level 2 (`0.2` → `-4` / `4`).
  - Run `KQMonster::preSpecial Connor` while Connor plays a non-interruptable
    movement, then `KQMonster::setState Connor Special ...`: confirm the
    `Special` movement starts immediately and cached movement resets to
    `"Stop"`.
  - Run `conClickItem <item>` on a placed world item: confirm it picks the
    item directly into Connor's inventory without reach/turn/`handsOff`
    checks and sets `$Console::Return = 1`.
  - Run `cls`: confirm it is accepted as a no-op returning `$Console::Return
    = 0`.
  - Run `useAlternateLightmap gnome13 1` then `0` in Gnome: confirm
    `gnome13` swaps immediately between `gnome13.dil` and `gnome13l.dil`.

## Open questions

- None for Batch 1 or Batch 3 command items; `KQGame::letCatchUp` exact
  delta clamping details outside loading pauses are unneeded as long as our
  fixed-timestep loop already caps large frame gaps cleanly.
