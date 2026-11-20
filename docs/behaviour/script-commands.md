# Script language gaps and shared commands (Batch 1)

Study session A, 2026-11-20. Sources: `Mask.exe` (studied in Ghidra and
described here in our own words) and the `.cs` scripts under `MASK/`.
Covers Batch 1 ("Shared / Everywhere") items from `docs/SCRIPT-COVERAGE.md`
not placed in a subsystem note.

Related notes for other Batch 1 items:
- Camera shake (`KQCamera::shake`): `camera.md` ("Shake and cutscene
  cameras").
- Proximity haze (`KQHazer`), `newSky`, `resetHazeColor`: `object-lighting.md`
  (rules 23a–23e, 23, 31).
- `doPopup`, `stopHelp`, `notStartingHelp`, `KQGame::selectScreen`, `blackOut`,
  `loadPalette`: `interface.md` (rules 1a–1f, 3a, 53–59).
- `allowShadows`: `characters.md` (rule 36).
- World `Load`, `setLoadProgress`, `activatePersistMgr`: `world-change.md`
  (rules 8, 24).
- `KQEnvInfo`: `water.md`. `KQCamMover`: `camera.md`.

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

### Loading, cache, timing, and persistence hints

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

## Data used

- `MASK/Snowexit/kq/worldinv.cs` (`;` lines), `MASK/GAME/Kq/getloc.cs`,
  `MASK/DeadCity/kq/skelking.cs`, `MASK/Gnome/kq/emptyloc.cs`.
- `MASK/IceWorld/kq/{icelord,lucreto,sentry}.cs` (`Preload`).
- `MASK/Barren/kq/{dockup,gsetup}.cs` (`setcat`).
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

## Open questions

- None for Batch 1 items; `KQGame::letCatchUp` exact delta clamping details
  outside loading pauses are unneeded as long as our fixed-timestep loop
  already caps large frame gaps cleanly.
