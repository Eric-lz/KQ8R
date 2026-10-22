# World change and loading

Study session A, 2026-10-21. Sources: the program (traced), the game's
scripts and data files in `MASK/` (data), and the user's own saved games
in `MASK/save/` and the kept-world folder `MASK/cur/` (read only, data).
Each rule says which. Builds on `magic-map.md` rule 40 (the Portal
state), `interface.md` rules 1–2, 19, `sound.md` rules 54–57, and
`objects.md` (what `loadKQ` does). Answers the six unknowns of
`docs/DESIGN.md` "6e" and `magic-map.md` open questions 4 and 5.

## Purpose

Moving Connor from one world (Daventry, Castle Daventry, Dead City, …)
to another, through a walking exit or a teleporter, and coming back
later to find the old world as he left it. The game does this by
**saving the world it leaves to disk exactly as a saved game would, and
loading the next one as if it were loading a saved game.** Connor
himself goes across as his own saved file.

## Words used here

- **Portal**: a `KQPortal` object. Its keys: `world` (the destination
  world's file, e.g. `Castled.kq`), `enterAnim` / `enterAnmFile` (the
  movement Connor plays before leaving, and the movement list it comes
  from), `exitAnim` / `exitAnmFile` (the movement he plays on arrival),
  `exitScript`, `destConnorLoc` / `destConnorDir`, `destCamLoc` /
  `destCamDir`, `noLoc`, and `script` (its own script).
- **Kept world**: the snapshot of a world Connor has left.
- **The `cur` folder**: `MASK/cur/`, where the game keeps the kept
  worlds and the hand-over files while a game is running.
- **Journey**: what goes with Connor from one world to the next.
- **Hand-over**: the moment the old world is saved and taken down and
  the new one is loaded.

## Rules

### What starts a world change

1. **Always a script** [data, traced]. The program never starts a
   world change by itself when Connor touches a portal. A script says
   `KQMonster::setState Connor Portal <portal> [<trap>]`: the Portal
   state of Connor, given the portal's object name and, for walking
   exits, the name of the trap that fired (`$1` in a trap script). The
   shipped starters:

   | World | Portal → destination | Started by |
   | --- | --- | --- |
   | Daventry | `portal1` → Castle Daventry | trap `portaltp` On (Connor first turned to heading 3.14; first time +25 experience and `Daventry::Castle::foundPortal`) |
   | Daventry | `davgate4` → Dead City | trap `sdwtrap1`: only with the `RingOfLight` (first time) or after `Daventry::BeenThere::DoneThat`; otherwise a short "can't pass" scene |
   | Castle Daventry | `portal2` → Daventry | trap `portalt2` On |
   | Dead City | `portal` → Daventry; `portaa10` → Swamp | traps `portaltp`, `port10tp` On |
   | Swamp | `skport3` (file `swpgate4.kq`) → Dead City | trap `portaltp` On |
   | Swamp | `swpgate5` → Gnome (loaded by `snail.cs`) | the portal's own script on `ConnorCollision` |
   | Gnome | `poolport` → Swamp | the portal's own script on `ConnorCollision` |
   | Barren | `portal6` (elevator) → Ice world | `elevator.cs` |
   | Ice world | `portal2` → Snow exit; `teleport` (elevator) → Barren | trap `portaltp`; `elevator.cs` |
   | Snow exit | `portal3` → Ice world; `portal4` → Temple 1 | trap `portalt3`; `archon.cs` |
   | Temples 1–4 | `portal` / `portal2` between neighbouring temples | traps `portalt`, `portalt2` |
   | Six worlds | the teleporter portals (`GAME/Kq/*tele.kq`) | the world's `TeleportScript` (`magic-map.md` rule 39) |

   Barren's `portal4.kq` (to `Dwarf.kq`) and the Ice world's `portal4.kq`
   (to `icelord.kq`) are never loaded by any script [data]: unused.
2. **Entering the Portal state** [traced]. Connor stops where he is and
   plays `stop`; the portal and trap names are remembered; the Yes/No
   popup opens (rule 26) unless one is already open. Nothing else
   happens until it is answered. (`magic-map.md` rule 40 step 1.)
3. **No** [traced]: as `magic-map.md` rule 40 step 2: the portal's own
   script gets `Decline`; if the portal's `noLoc` is not (−1, −1, −1)
   Connor is moved there (e.g. Daventry's `portal1`: 158510, 307740,
   9925, just back from the trap); Connor plays `stop` and goes back to
   Wait. The answer is final for this entry: walking into the trap
   again starts a new Portal state (the trap fires `On` again only if
   Connor left it).
4. **Yes** [traced]: as rule 40 step 3 (portal script `Accept`,
   `handsOff 1`, first person off, target cleared, the longest time
   step two frame periods), and the state marks itself "entered". Then:
   - **with an `enterAnim`**: Connor's movement is stopped and he plays
     the `enterAnim`, from `enterAnmFile` when the portal names one
     (`Elevator.anm`, `levator2.anm`, `poolport.anm`, `Snail.anm`), else
     from his current list (`teleportdown`, `portal`). When it ends,
     rule 5 runs.
   - **without one** (Castle, walking exits of the Snow exit and the
     temples, the Swamp's teleporter): rule 5 runs at once.
5. **Getting ready to leave** [traced]:
   1. Connor's next movement is set to the portal's `exitAnim`;
   2. if the portal has an `exitAnmFile`, Connor's movement list is
      switched to that file for the arrival (remembering his normal
      list to switch back to later). When the file is `conner.anm` and
      Connor wears the full armour, `conner2.anm` is used instead. (This
      armour swap is only on the "no `enterAnim`" path; on the other
      path the file is used as named. In the data the only portals with
      `exitAnmFile=Conner.anm` have no `enterAnim`, so the difference
      never shows.) **Correction to `magic-map.md` rule 40 step 4:** the
      armour swap applies to the arrival's movement list, not to the
      `enterAnim`;
   3. Connor is taken off the trap's list of things inside it, without
      the trap getting `Off`, so the trap does not hold him in the
      saved world;
   4. the portal hands over (rule 6).

### The hand-over

6. **The portal's hand-over** [traced]:
   - Connor's arrival place and turn are written into the script
     variable `KQPortal::ConLoc` as six numbers "x y z tiltX tiltY
     heading" (from `destConnorLoc` and `destConnorDir`; the turn is
     (0, 0, 0) when the portal gives none); the camera's into
     `KQPortal::CamLoc` (from `destCamLoc`, `destCamDir`);
   - `KQWorld::SaveGameName` is set to `cur` and `NewWorld` to the
     portal's `world`;
   - a "change world" request is queued; it runs after the current
     event has finished (so the rest of the script or animation step
     that caused it completes first).
7. **Taking the old world down** [traced], in this order:
   1. the game is paused; the effects and speech loudness are
      remembered and set to 0 (`sound.md` rule 55);
   2. the screen palette goes black; the loading screen's files are
      prepared (rule 21);
   3. **the old world is saved** into the `cur` folder (rules 11–15);
   4. the old world is taken down: its objects are deleted, `mask.cs
      UnloadWorld` runs (it frees the world's resources and its
      archives), `KQWorld::Name` / `Parent` are emptied, and the sound
      system is restarted (every playing sound stops);
   5. the new world's name and parent are read from its world file
      (`GAME/Kq/<World>.kq`: `KQWorld::Name`, `KQWorld::Parent`; e.g.
      Castle Daventry's parent is Daventry, whose sounds and texts it
      uses).
8. **Building the new world** [traced, data]:
   1. if the destination is **Temple 1**, every kept world archive
      (`cur/*.vol`) is deleted: from the temples on there is no way back
      [traced]; the kept maps (`cur/*map.kq`) stay;
   2. `mask.cs RestoreWorld` runs: the world's archives are opened,
      plus the `cur` folder and the kept world's archive
      `cur/<World>.vol` if there is one, and `cur` is put first in the
      file search (`KQGame::moveVolToFront cur`) [data]. So a file name
      is found in `cur` first, then in the kept world, then in the
      world's own archives [the `cur`-first part traced; that the kept
      archive comes before the world's own `Kq` archive is inferred: the
      game could not restore a world otherwise];
   3. `loadKQ gameState.kq` (rule 18), `world.cs <World> Init` (sky,
      palette names, poison times), `world.cs <World> Load`, which runs
      `newWorld.cs`: the **kept** world's generated one when the world
      was visited before (rule 14), else the world's own (a first
      visit), then `blackOut`, `KQGame::selectScreen 1` (the game play
      screen), `loadPalette`, `resetHazeColor`, `allowShadows`;
   4. the save archives are closed, `loadPalettes 0`, the loading
      pictures are dropped from memory;
   5. `cur/Connor.kq` and `cur/GameState.kq` are deleted (they were used
      up by the loading) [traced].
9. **The first drawn frame** [traced]: the remembered effects and speech
   loudness come back, every sound group is resumed (`sound.md` rule
   57b), and `world.cs <World> Begin` runs. `Begin` runs on **every**
   arrival, first or not; the scripts guard their first-time scenes with
   their own variables (Daventry: `Daventry::girlhut::FirstTime`).
10. **A new game** goes through the same building path but with
    `mask.cs NewWorld` instead of `RestoreWorld`, after **emptying the
    `cur` folder** [traced]. Loading a saved game also empties `cur`,
    then copies the save's kept worlds (its `worldVolumes` list) and
    kept maps (its `maps` list) into `cur`, then restores the save's
    current world [traced].

### What a kept world holds (unknown 2)

11. **When it is made** [traced]: at every hand-over, for the world
    being left (rule 7.3). A world never visited has none. Leaving it a
    second time overwrites it.
12. **What goes in `cur/<World>.vol`** [traced, data: `cur/CastleD.vol`,
    `cur/Daventry.vol`, and every save]:
    - `worldName` (`CastleD.kq`) and `savedGameVersion` (`1.000000`);
    - **every object the world holds that was loaded while the
      "persist manager" was on**, each as `<name>.kq` in the ordinary
      `.kq` text form with its whole current state: position and turn,
      health and state of characters, doors' and chests' state,
      traps (with what is inside them), loose items on the ground,
      objects loaded or created by scripts while playing (e.g.
      `NoName` items dropped by creatures), the terrain's changes
      (`nDeactivatedLights`, `nChangedHeights`), the camera
      (`KQCamera.kq`), the world's item list (`WorldInventory.kq`), the
      music list (`Music.kq`, `sound.md` rule 57j);
    - objects **deleted** while there (dead creatures removed, items
      taken) are simply not in it, so they don't come back;
    - `worldState.kq` (`KQWorldState`, a count of events: 0 in every one
      of the user's saves; not studied further);
    - `WorldVar.cs` (rule 13) and a generated `newWorld.cs` (rule 14).
13. **Script variables** [traced, data]. `WorldVar.cs` holds one `set
    <name> <value>` line for every script variable whose name **starts
    with the world's name** (Castle: `CastleD…`, e.g.
    `CastleD::King::Moved 1`) or **with `KQWorld`** (`KQWorld::Name`,
    `MaxMonsters`, the palettes, …), except `KQWorld::SaveGameName`. So
    variables are **per world by their name**: each world's scripts use
    only names of their own world (checked over all shipped scripts:
    Daventry's 395 uses are all `Daventry::…`, Dead City's 788 all
    `DeadCity::…`, and so on) [data]. Things that must cross worlds are
    kept on Connor instead (rule 17: his flags). Whether the left
    world's variables are also removed from memory was not traced; they
    are set again from `WorldVar.cs` on return either way.
14. **The generated `newWorld.cs`** [traced, data] replaces the world's
    own start script on return. In order:
    1. `setCurrentTime <t>`: the world's clock is put back to the value
       it had when Connor left, so **no time passes in a kept world**;
    2. `WorldVar.cs`;
    3. `loadKQ Terrain.kq`, `loadKQ Connor.kq`, `loadKQ KQCamera.kq`,
       `loadKQ <map>.kq` (the world's map, rule 16);
    4. `permWrld.cs $KQGame::runOptimal`: the world's own permanent
       objects (buildings, trees, …) loaded **fresh from the world's
       files**, not from the snapshot (they are loaded with the persist
       manager off, so they are never saved);
    5. one `setLoadProgress <n>` + `loadKQ <name>.kq` pair per kept
       object, in the order the world held them (n rising from about 61
       to 100);
    6. `loadKQ WorldState.kq`, `setLoadProgress 100`.
15. **Each restored object starts again as a loaded object** [traced
    through `objects.md` / `loadKQ`]: it gets its script's `Init` when it
    enters the world, as on a first visit, but with its saved values
    (e.g. a door saved open is loaded open).
16. **The map** [traced, data]. The current world's map object is saved
    on its own as `cur/<map>.kq` (e.g. `cur/davMap.kq`, `cur/castMap.kq`)
    with its fog, icons and `earnedTeleport` (`magic-map.md` rule 16). On
    return, the world's start script's `loadKQ davmap.kq` finds the kept
    one in `cur` first (rule 8.2), so the fog comes back. **This answers
    `magic-map.md` open question 5**: another world's kept map, looked
    up by its file name for paging (rule 34), is this file in `cur`.
    Kept maps are not deleted at Temple 1 (rule 8.1).

### The journey: what goes with Connor (unknown 3)

17. **Connor travels as his whole saved object** [traced, data]. At the
    hand-over he is written to `cur/Connor.kq` in the saved-game form,
    and the new world's `loadKQ connor.kq` (a first visit's own start
    script, e.g. Castle's `loadKQ connor.kq same 26545 98311 -1 0 0 0`)
    or `loadKQ Connor.kq` (a kept world's) finds that file in `cur`
    before the shipped `GAME/Kq/connor.kq` (rule 8.2). Everything that
    is part of Connor goes with him unchanged:
    - his statistics, health, level, experience, damage and protection;
    - his inventory, each item with its own state (counts, the item in
      hand, the armour, the readied weapons and his weapon mode);
    - his flags: the cross-world flags of `setConnorFlag`
      (`BeenToCastle`, `MetGirlInDC`, `SageGnomeEarnedTeleport`,
      `SawSwampVision`, `KnowAboutFeather`, `InTemple4`,
      `UsedElevator`) and the magic map's per-map "visited" and
      "teleporter earned" marks (`magic-map.md` rules 33, 37);
    - his current state (the Portal state, rule 19) and his movement
      list.
    The things that are **not** part of Connor stay with the world: the
    camera (the new world's own, placed by rule 19), the map (the new
    world's own), the music.
18. **`GameState.kq`** [traced] also crosses: the counter used to name
    unnamed objects (`NoName<n>`, so new names never repeat a kept
    world's) and the combat difficulty (0.66 / 1.0 / 1.5).
19. **Checked by the user (2026-10-22)**: health, level, items and their
    counts, weapon and armour values, the weapon in hand (if any), and
    **running potion effects with their remaining time** all carry over.
    Not traced: which fields of Connor's saved form hold them, and the
    rope's state (the rope cannot be in use during the Portal state:
    Connor is standing on a trap or a pad).

### The arrival (unknown 1)

20. In the new world [traced unless marked]:
    1. the world is built (rule 8) with Connor loaded by its `loadKQ
       connor.kq` line: when `KQPortal::ConLoc` is set, its six numbers
       **replace** the line's own position and turn, and the variable is
       emptied at once (so only the first Connor load uses it). A z of
       −1 (Dead City and Ice world teleporters, the Daventry–Dead City
       gate, the Dead City–Swamp gate) gets the usual ground rule
       (`objects.md` "Ground heights −1 and −2"). The camera object
       (`cam3rd.kq` on a first visit, `KQCamera.kq` on return) is placed
       from `KQPortal::CamLoc` the same way;
    2. Connor comes back **in the Portal state**, already "entered", with
       the portal's names and his next movement set to the `exitAnim`
       (rule 5), and the arrival movement list when the portal named one
       [traced that the state saves and restores these fields: portal
       and trap names, enter / exit movement and list, exit script,
       "entered", "done", "answered"];
    3. he plays the `exitAnim` (`teleportup`, `out`, `portalx`,
       `walkend`, `c_comnup`, `c_land`, `cotmp1s1`, `whirlx`,
       `outgnmpool`). [Traced: in this state, the end of any movement
       other than the enter and exit movements starts the `exitAnim`
       again; that the restored "next movement" is what starts it on
       arrival is inferred from rule 5.1.]
    4. **When the `exitAnim` ends** [traced]: Connor plays `stop`; first
       person is allowed again; the portal's `exitScript`, if any, runs
       with `$1` = the portal's name and `$2` = `Exit` (`shadbat.cs`,
       `entrswmp.cs`, `entrgnom.cs`, `eleport.cs`, `portal6.cs`,
       `Enter.cs`); the longest time step goes back to 1 s; the state is
       marked "done"; `handsOff 0`; Connor goes to Wait. (If the arrival
       list was switched, his normal list is set back [inferred from the
       remembered name in rule 5.2; not followed].)
    5. The map stays closed (`magic-map.md` rule 40).
    **This answers `magic-map.md` open question 4.**
21. **On a first visit versus a return** [data, traced]: a first visit
    runs the world's own `newWorld.cs` (all objects new, the world's
    `WorldVar.cs` start values, and, only when `$Testing` is true,
    `conStart.cs`; `Testing` is set only by the developer start scripts
    `GAME/Kq/<world>.cs`, never in a normal game). A return runs the
    kept one (rule 14). `world.cs Init`, `Load` and `Begin` run in both.

### The loading screen (unknown 4)

22. **Its files** [traced, data]. At the start of each hand-over the
    game copies into `GAME/8Gui/`:
    - `<world>.pbm` → `loading.pbm` (e.g. `daventry.pbm`, `castleD.pbm`,
      `deadcity.pbm`; from the world's resources if `GAME/8Gui` has none);
    - `<first 5 letters of the world>2.pbm` → `loading2.pbm`
      (`daven2.pbm`, `deadc2.pbm`, `barre2.pbm`, `gnome2.pbm`,
      `icewo2.pbm`; Castle Daventry uses `daven2.pbm`);
    - `L<first 7 letters>.ppl` → `loading.ppl` (`interface.md` rule 2).
    Old copies are deleted first; a read-only copy is made writable.
    (Do not reproduce: rule "Do not reproduce" 1.)
23. **Its layout** [data: `GAME/Resource/Loading.gui`, `GAME/Kq/Loading.kq`]:
    640 × 480, palette `loading.ppl`, not the world's palette. Two
    full-screen pictures (`Loading.pbm`, `loading2.pbm`), a text line
    (0, 415)–(640, 445) in `consolel.pft`, centred, a black bar
    (0, 462)–(640, 480) (`4black.pbm`) and three meters of 630 × 8 at
    (5, 468) in red, green and blue (`4red.pbm`, `4green.pbm`,
    `4blue.pbm`). **Two phases** (the user's check, 2026-10-22; which
    file is which from the data):
    1. **copying the world's files from the CD**: `loading2.pbm`, an
       overview of the whole game's universe with the loading world
       highlighted in colour, and a **green** meter. [Data: the
       `<5 letters>2.pbm` pictures differ from each other in only 15–22 %
       of their pixels (one shared overview, a different highlight),
       while the `<world>.pbm` pictures differ by 45–68 % (each its own
       picture).] With the user's patched install this phase is very
       short and the meter is not seen;
    2. **loading the world**: `loading.pbm`, a picture of that world, and
       a **red** meter.
    [Traced that one of the pictures is moved on and off screen during
    the hand-over and that the meter has three colour phases; the blue
    one was not seen.]
    **The meters and the black bar** (traced 2026-10-22, Q152): every
    picture on the loading screen is handled as a meter (the black bar
    and the full-screen pictures too). A meter's filled part is its own
    picture **stretched** to (630 x percent / 100, cut to whole pixels,
    at least 1 if the percent isn't 0) x 8; the unfilled rest is
    `4black.pbm`, stretched. Only the active colour's meter is on the
    screen (the other two are moved off). `IDCTL_LOADING_BLKBAR` is a
    meter fixed at 100 % of `4black.pbm` (palette index 10 all over),
    repainted with the meter and the text at **every meter step**, in
    both phases. So by the code rows 462-480 are a black strip covering
    the frame painted into the picture, with the coloured meter over it.
    **The user's check in the original (2026-10-22) says otherwise:** the
    meter's frame painted into the picture stays visible; black is drawn
    only **inside** it, where the meter grows (the slot), leaving the
    frame's bright outline. So build: the picture with its frame, the
    coloured part over the slot, and **black over the rest of the slot**
    (the meter's unfilled part, as above); do not draw the full-width
    black bar over the frame. Why the bar control doesn't show (paint
    order, or it isn't drawn) was not traced. The three meter phases are red, green, blue in that
    order by the code; the user saw green during the CD copy and red
    while loading, so which phase number each loading phase uses is
    worth a later look.
24. **The meter** [traced]: one colour shows at a time; it fills from
    left to right as the scripts' `setLoadProgress <percent>` lines are
    reached (with no number it moves on by one step). It is redrawn in
    1 % steps, each step drawing a whole frame, so the screen shows
    progress while the loading runs on the same thread. Nothing else
    moves; no input is handled during the load [inferred: the game is
    paused and only these redraws run].
25. **How long**: as long as the loading takes; no fixed time [traced:
    no timer]. Pressing keys does nothing during either phase (the
    user's check). The game play screen comes back with `selectScreen 1` in
    `world.cs Load` (rule 8.3), and the world appears on the first drawn
    frame (rule 9).

### The Yes/No popup (unknown 6)

26. **The question** [traced, data]: the help popup panel
    (`IDCTL_POPUP`, (140, 120)–(510, 320) on the game play screen,
    `interface.md` rule 7) with text 114 "You are about to load a new
    region. Are you sure you want to do that?", button 29 "Yes" and
    button 14 "No"; the other two button places are empty and the "turn
    off help" box is hidden.
    - The text is cut into lines of up to **38 characters** at a space
      (and at line breaks), at most **7 lines**, the unused lines empty
      [traced].
    - **Yes is the left button, No the right** (the user's check,
      2026-10-22; corrects the first version of this note, which had
      read the button places the other way round). Each is 88 wide, at a
      third and two thirds of the panel's width [traced].
    - **The game is paused while it is open** [traced]: the game's pause
      is set in a mode that, unlike the Pause key and the menu, does
      **not** pause the sounds, so music and looping sounds go on.
    - A **click** on a button closes the popup, unpauses the game, and
      sends the answer to Connor's Portal state [traced].
    - **Keys** (the user's check, 2026-10-22): one button is
      highlighted; **Left / Right** move the highlight between Yes and
      No; **Enter** chooses the highlighted one; **Escape** does what No
      does **and** opens the main menu (which pauses the game as usual);
      **Space** still shows / hides the bottom bar as in play; other keys
      do nothing. Which button is highlighted when the popup opens was
      not noted.
    - The popup is not shown (the answer is Yes at once) in the
      developer demo mode `KQGame::Demo` [traced]; not part of a normal
      game.
    **Details** (traced 2026-10-22, Q151):
    - **Yes is highlighted when the popup opens.** No highlight moves:
      the first button control (the layout's BTN1, picture
      `buyhibtn.pba`) *is* the highlight and starts holding Yes. Left /
      Right **swap** the two buttons' places, labels and answers, so the
      gold outline seems to move; each swap plays a click sound; Left
      works only if there is a button on that side. Yes is on the left,
      No on the right.
    - **Enter** (or F1) pressed: the pointer position is remembered, the
      pointer jumps to the highlighted button's centre (16 pixels
      lower), the button shows pressed and a "pressed" flag is set.
      **Released**: answers, only if that flag is set; the pointer goes
      back and the flag clears. Opening or closing a popup clears the
      flag, so an Enter held from before can't answer it and a held
      Enter never answers the next popup: one answer per press.
      (Auto-repeat saves the pointer again on every repeat, so after a
      long hold the pointer is put back on the button.)
    - **Escape**: in `handsOff`, nothing at all. Otherwise it answers
      with **the button that is not highlighted**, then opens the main
      menu. From the start that is No (the user's check); after Right
      has moved the highlight to No, Escape answers **Yes** (traced from
      the swap, not checked in the game; see "Do not reproduce").
    - **Button places:** with W the popup's width (inferred: the 370-wide
      dialog), the two buttons' left edges are at W/3 - 44 and 2W/3 - 44,
      each 88 wide (centres at a third and two thirds): Yes 79-167, No
      202-290 from the dialog's left edge. Vertically at 640 x 480: the
      highlighted button (BTN1) rows 145-183 (38 high), the other
      146-182 (36 high); heights stay with the control when they swap.
      With the "turn off help" box shown, BTN1 is at 142, 25 high.
      800 x 600: 44 lower. Three buttons: W/4, 2W/4, 3W/4, minus 44.
    - **The text:** text 114 has its own line break; explicit breaks are
      used only if one comes before the 39th character, otherwise the
      38-character wrap applies (here: 2 lines). 7 line boxes, each 360
      wide, centred. Line k of n: top = base + k x pitch + T[n][k],
      height = pitch; 640 x 480: base 12, pitch 18; 800 x 600: base 36,
      pitch 22. T (rows n = 1-7, then line 0, 1, ...): 1: 52; 2: 36, 44;
      3: 20, 28, 36; 4: 4, 12, 20, 28; 5: 4, 8, 12, 16, 20; 6: 2, 4, 6,
      8, 10, 12; 7: all 0. The portal text at 640 x 480: line 1 rows
      48-66, line 2 rows 74-92 (centres 57 and 83: **26 apart**). The
      first line keeps its trailing CR (probably draws nothing, not
      traced).
    - **The looks:** every button picture is **stretched to the button's
      box** (so `buyhibtn.pba`, 88 x 25, is stretched to 88 x 38). Normal:
      frame 0; pressed (mouse button or Enter held): **frame 1** of
      whichever picture that button has, so with Enter the highlighted
      button shows `buyhibtn.pba` frame 1; a mouse press on the other
      shows `yesbtn.pba` frame 1. Hover would ask for frame 2, which
      neither picture has (what is drawn then: not traced). The label
      moves **1 right and 1 down** while pressed.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| `KQPortal::ConLoc` / `CamLoc` | six numbers: x, y, z (world units), tilt x, tilt y, heading (radians) | traced |
| Turn when the portal gives none | 0, 0, 0 | traced |
| `noLoc` meaning "don't move" | (−1, −1, −1) | traced |
| Longest time step during the Portal state | 2 frame periods; back to 1 s on arrival | traced (`magic-map.md` rule 40) |
| Popup text | ≤ 38 characters per line, ≤ 7 lines | traced |
| Popup buttons | Yes left, No right; 88 wide, at 1/3 and 2/3 of the panel width | user's check, traced |
| Loading meter | 630 × 8 at (5, 468), redrawn every 1 % | data, traced |
| Kept newWorld.cs progress | about 61 → 100 over the kept objects | traced, data |
| World clock on return | as when left (`setCurrentTime`) | traced, data |
| Free disk space asked for | 110 MB (Do not reproduce 2) | traced |

## Data used

- `MASK/mask.cs` (`NewWorld`, `RestoreWorld`, `UnloadWorld`), each
  world's `kq/world.cs` (`Init`, `Load`, `Begin`, `Die`), `newworld.cs`,
  `permwrld.cs`, `WorldVar.cs`.
- Portal files: `GAME/Kq/*tele.kq` (teleporters) and the worlds'
  portal files (rule 1's table): keys listed under "Words used here".
  Text files spell the place `destConnorLocX` and the turn
  `destConnorDirX/Y/Z`; the saved form writes `destConnorlocX` and a
  turn matrix `destConnorRMat` [data].
- Trap and gate scripts of rule 1.
- `GAME/Kq/<World>.kq` (`KQWorld` name and parent), `GAME/Kq/connor.kq`.
- `GAME/Kq/Loading.kq`, `GAME/Resource/Loading.gui`, `GAME/8Gui/*.pbm`,
  `GAME/8Gui/L*.ppl`.
- Text 114, 29, 14 (`500.MSG`, the popup).
- The kept-world archive (`PVOL`, the same layout as a saved game:
  `magic-map.md` "Data used"); the hand-over files in `cur`:
  `<World>.vol`, `Connor.kq`, `<map>.kq`, `GameState.kq`.

## Edge cases

- **`ConLoc` not set** (a new game, a loaded save): Connor goes where
  the `loadKQ connor.kq` line (or his saved file) puts him [traced].
- **Two Connor loads in one build**: only the first uses `ConLoc`
  (it is emptied) [traced].
- **Returning to a world**: everything in rule 12 comes back as it was
  (the user's check, 2026-10-22: broken barrels stay broken, slain
  enemies stay dead **on the ground**, items dropped by enemies, chests
  or barrels stay where they fell, doors, levers, pushed rocks and
  puzzles stay as left). A creature still alive is where it was, with
  its health. Buildings and other permanent objects are fresh copies,
  but they don't change in play.
- **Leaving a world twice**: the kept world is overwritten with the
  newer state [traced].
- **Temple 1**: all kept worlds are deleted on the way in (rule 8.1);
  the temples' own portals then keep and restore each other as usual.
- **The trap after a No**: Connor is still inside the trap, so it does
  not fire again until he walks out and back in (rule 3) [inferred from
  the trap rules in `traps.md`].
- **The trap after a Yes**: Connor is removed from it without `Off`
  (rule 5.3), so when he comes back the trap is empty and fires `On`
  again when he next enters it (e.g. he arrives beside it, at the
  portal's destination, not inside) [traced, data].
- **The popup already open** when the Portal state starts: it is not
  opened again [traced].
- **A kept world with no kept archive file** (first visit): only the
  world's own files are searched [traced through rule 8.2]; a "could
  not load volume" message in the original's console is harmless
  [inferred].

## Do not reproduce

- **Escape answering the popup's other button** (Q151): after Left /
  Right has moved the highlight to No, Escape answers Yes in the
  original. Make Escape always mean No.

1. **Copying the loading pictures over files in the install**
   (`GAME/8Gui/loading.pbm`, `loading2.pbm`, `loading.ppl`, and making
   them writable). Ours: pick the world's pictures and palette directly
   by the rule of rule 22; never write to the game folder (AGENTS rule 4).
2. **The free disk space check** before restoring a world (110 MB
   minus the sizes of the files it needs, else a warning about the CD /
   disk) and the "play from CD" copying: leave out. The copy phase's
   screen (rule 23.1) then has nothing to show; ours goes straight to
   the loading phase (showing the overview briefly first is optional).
3. **The `cur` folder on disk** for kept worlds and the hand-over:
   ours may keep them in memory (the planned "journey" record and kept
   worlds), as long as the content and the rules (rule 8.1's Temple 1
   wipe, rule 15's `Init` on return, rule 14's frozen clock) stay the
   same. Write them to disk only for saved games, and never inside
   `MASK/`.
4. **The developer demo mode** (no popup) and the developer start
   scripts (`GAME/Kq/<world>.cs`, `Testing`): leave out of the game
   (a dev-console option of ours may do the same thing).
5. **The unused portals** to `Dwarf.kq` and `icelord.kq`: not loaded by
   any script; nothing to build.

## How to check

In the original game (the user), in this order:

1. **Popup and pause**: walk into the Daventry → Castle exit (the cave
   mouth north of the waterfall). The question appears; Connor stands
   still; creatures and the water stop moving, but the music and the
   waterfall's sound go on. **Yes is on the left, No on the right.**
2. **No**: click No: Connor is put back a step (south of the exit),
   the popup closes, play goes on. Walk out and back in: the question
   comes again.
3. **Yes and the loading screen**: walk in again, Yes: the screen goes
   black, then a full-screen picture of the destination with a
   coloured bar at the bottom that grows; note whether the picture
   changes during the load and the bar's colour(s). Pressing keys or
   clicking during the load does nothing.
4. **Arrival**: in Castle Daventry Connor walks out (`out`), the camera
   already placed; control comes back when he stops; the map window,
   if it was open, is closed.
5. **Keys on the popup**: at the next question, try Enter, Escape, Y,
   N, Left/Right: write down what each does.
6. **Kept world**: in Castle Daventry move something the game lets you
   change (open the fire door, take an item, kill a creature if any);
   go back to Daventry and return: it is as you left it. In Daventry
   before leaving, open a door or chest and leave an item on the
   ground; on return it is still there, and creatures you killed are
   still gone.
7. **Journey**: before leaving, note health, level, experience, the
   items and counts, the readied weapon and any running potion; all the
   same after arriving.
8. **Clock**: in a world with a timed event (a potion running, or a
   creature's respawn), leave and come back: nothing has moved on while
   away.
9. **Teleport**: with two teleporters earned, use the pad and the Go
   button: the same question; Yes: `teleportdown`, the loading screen,
   then `teleportup` on the other pad.
10. **Map after the trip**: open the map in the new world and page back:
    the left world's map shows its fog as it was.

In the data (B, any time):

11. `MASK/cur/` and every `MASK/save/save*.vol`: each kept world archive
    holds `worldName`, `savedGameVersion`, the objects, `Music.kq`,
    `worldState.kq`, `WorldVar.cs` (only names starting with the world's
    name or `KQWorld`, never `KQWorld::SaveGameName`) and a `newWorld.cs`
    beginning `setCurrentTime`, `WorldVar.cs`, `loadKQ Terrain.kq`,
    `loadKQ Connor.kq`, `loadKQ KQCamera.kq`, `loadKQ <map>.kq`,
    `permWrld.cs $KQGame::runOptimal` and ending `loadKQ WorldState.kq`,
    `setLoadProgress 100`. A save also lists them in `worldVolumes`
    (`CastleD.vol|Daventry.vol`) and `maps` (`castMap.kq|davMap.kq`).
12. A round trip in our build: the left world's object count and each
    object's saved values the same after coming back (log both).

## The user's check in the original (2026-10-22)

"How to check" 1–10, in the user's patched GOG install:

- 1: right, except **Yes is on the left and No on the right** (rule 26
  corrected).
- 2, 4, 9, 10: as described.
- 3: two loading phases (rule 23): the universe overview with a green
  meter while files are copied from the CD (very short with the patch),
  then the world's own picture with a red meter. Keys do nothing.
- 5: the keys of rule 26.
- 6: everything is kept (Edge cases, "Returning to a world").
- 7: the journey of rule 19.
- 8: no timed event in a world was found to compare; potion timers on
  Connor carry over (rule 19). The frozen world clock (rule 14) stays
  traced only.

## Open questions

1. ~~Which loading picture shows when, the meter's colours~~: answered
   by the user's check and the data (rule 23).
2. ~~The popup's keys~~: answered by the user's check (rule 26). Still
   open, small: which button is highlighted when it opens.
3. Whether the left world's script variables are removed from memory
   (rule 13); no shipped script reads another world's names, so it
   doesn't change play.
4. Where in Connor's saved form the rope state lives (rule 19), and
   whether the normal movement list comes back
   after an arrival with its own list (rule 20.4).
5. What `worldState.kq`'s events are (always 0 in the user's saves).
6. `magic-map.md` open question 3 (does the mini-map take the big map's
   zoom after a world change) is still not checked.
