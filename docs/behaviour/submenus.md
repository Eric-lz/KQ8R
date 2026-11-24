# Main menu submenus (`Load Game`, `Save Game`, `Options`, `Graphics`, `Remap Move Keys`, `Credits`, `Replay Intro`)

## Purpose

Describe every screen opened from the main menu (`Mainmenu.gui`) and from the Options menu (`Options.gui`): how saved games are scanned, displayed, saved, loaded, and deleted (`Loadgame.gui` and `Savegame.gui`); how audio, subtitle, help, graphics, and key-binding settings are edited and persisted (`Options.gui`, `Graphics.gui`, `movekey.gui`); how the 26-page credits roll works (`credits.gui`); and how full-screen AVI movies (`W32opn_1.dll`, `W32mdl_1.dll`, `W32ndg_1.dll`) are played from `Replay Intro`, startup, and scripts.

For the main menu itself, see [main-menu.md](file:///c:/Games/KQ8R/docs/behaviour/main-menu.md). For how worlds and `cur/` archives are packed and restored across world transitions, see [world-change.md](file:///c:/Games/KQ8R/docs/behaviour/world-change.md).

## Rules

### 1. Screen switching

1. **Nine GUI screens.** The GUI manager holds nine screen objects indexed `0` through `8`:
   - `0`: Main Menu (`MainMenu.kq` -> `Mainmenu.gui`)
   - `1`: Gameplay (`GPlayScr.kq` -> `GPlayScr.gui`)
   - `2`: Options (`Options.kq` -> `Options.gui`)
   - `3`: Loading (`Loading.kq` -> `Loading.gui`)
   - `4`: Graphics (`Graphics.kq` -> `Graphics.gui`)
   - `5`: Load Game (`loadgame.kq` -> `Loadgame.gui`)
   - `6`: Save Game (`savegame.kq` -> `Savegame.gui`)
   - `7`: Remap Move Keys (`movekeys.kq` -> `movekey.gui`)
   - `8`: Credits (`credits.kq` -> `credits.gui`)
2. **Activation lifecycle.** Switching to a screen deactivates the previous screen and activates the target screen. While any menu screen (`0`, `2`, `4`, `5`, `6`, `7`, `8`) is active, the menu theme `theme.aud` keeps playing in the background unless explicitly faded out (when loading a save, leaving Credits, or playing the intro movie).

---

### 2. Load Game (`Loadgame.gui`) and Save Game (`Savegame.gui`)

3. **Save folder scan and ordering.** Both screens scan `<game>/save/*.vol` on activation:
   - Each `.vol` archive is opened and checked for a `savedGameName` chunk. Archives without a valid `savedGameName` are ignored.
   - The number `<N>` is parsed from the filename `save<N>.vol` (stripping the `save` prefix and `.vol` suffix).
   - The list is sorted in **descending order of `<N>`** (newest slot number `save<N>.vol` at the top of the list, oldest at the bottom).
4. **10-row list and highlight bar.**
   - Up to 10 save names are shown at once in `IDCTL_LOAD_LIST0..LIST9` (or `IDCTL_SAVE_LIST0..LIST9`), starting at `topIndex` (`0`-based index into the sorted save list).
   - `selectedRow` (`0..9`) tracks which visible row is highlighted. The highlight bitmap `sl_hilite.pba` (`IDCTL_LOAD_HILITE` / `IDCTL_SAVE_HILITE`) moves vertically to center over `LIST[selectedRow]`.
   - Clicking a non-empty row sets `selectedRow` to that row and plays `click.aud`.
   - Pressing **Up** or **Down** moves `selectedRow` by one row and plays `click.aud`:
     - On **Load Game**, when `totalSaves == 0`, Up/Down do nothing; otherwise `selectedRow` cannot move past `totalSaves - 1`.
     - On **Save Game**, when `totalSaves == 0`, `selectedRow` stays at `0`; when `totalSaves > 0`, `selectedRow` can move across `0..min(totalSaves - 1, 9)`.
     - Moving Up at `selectedRow == 0` when `topIndex > 0` scrolls the list up by one (`topIndex -= 1`).
     - Moving Down at `selectedRow == 9` when `totalSaves - topIndex > 10` scrolls the list down by one (`topIndex += 1`).
5. **Scrollbar (`UP`, `DOWN`, `TAB`).**
   - Clicking `UP` (`sl_scton.pba`) or `DOWN` (`sl_scbon.pba`) plays `button1.aud` and scrolls `topIndex` by `-1` or `+1` when `totalSaves > 10`. Scrolling up increments `selectedRow` (clamped to `9`) and scrolling down decrements `selectedRow` (clamped to `0`) so the highlight tries to stay on the same save.
   - The thumb `TAB` (`sl_scbox.pba`, 20×20) moves vertically across a **190-pixel span** (`y = 205..395` in 640×480, or `y = 265..455` in 800×600):
     `thumb_y = track_top + (topIndex * 190) / (totalSaves - 10)`.
   - Clicking the scrollbar track (`x` in `68..87`, `y` in `205..414`) above or below the thumb scrolls up or down by **10 rows** (one full page).
   - Dragging the thumb with the mouse maps the cursor's `y` across the 190-pixel span to `topIndex`.
6. **Right-hand action buttons (`Load`/`Save`, `Delete`, `Cancel`).**
   - Three stacked 148×37 buttons sit on the right:
     1. Top (`(434, 192)`): **Load** (`#118` on `Loadgame.gui`) or **Save** (`#121` on `Savegame.gui`)
     2. Middle (`(434, 234)`): **Delete** (`#120`)
     3. Bottom (`(434, 276)`): **Cancel** (`#111`)
   - Just like the two-button popups in [main-menu.md](file:///c:/Games/KQ8R/docs/behaviour/main-menu.md), the first button control (`IDCTL_LOAD_OK` / `IDCTL_SAVE_OK`) owns the golden highlight graphic (`sl_hibtn.pba`), while the other two controls own the dark graphic (`sl_buton.pba`).
   - `activeButton` (`1` = Load/Save, `2` = Delete, `3` = Cancel) starts at `1` when the screen opens.
   - Pressing **Left** or **Right** arrow (when no popup is open) moves the golden highlight across the three buttons (`Left`: `3 -> 2 -> 1`, no wrap; `Right`: `1 -> 2 -> 3`, no wrap) by swapping the button rectangles and labels, and plays `button1.aud`.
   - Pressing **Enter** activates `activeButton`. Clicking any of the three button positions directly activates the button at that vertical position (`1` = top, `2` = middle, `3` = bottom) and plays `button1.aud`.
   - Pressing **Escape** (or activating **Cancel**) resets `activeButton` to `1` and returns to the Main Menu (`screen 0`).
7. **Thumbnail preview (`160 × 120`, `(428, 316)`).**
   - Each save stores a 19,204-byte raw thumbnail (`current.bin`): a 4-byte little-endian palette ID followed by `160 × 120 = 19,200` bytes of 8-bit pixels.
   - If the 4-byte palette ID is `999` (captured in 24-bit Direct3D/3Dfx mode, where pixels were already quantized against `Menus.ppl`), the 19,200 indices are copied directly into the thumbnail bitmap. Otherwise (captured in 8-bit DirectDraw mode), the indices are remapped from that world's palette ID to `Menus.ppl`.
   - On **Load Game**:
     - On initial screen open, the selected row's thumbnail is loaded immediately from `current.bin` inside `save/<file>.vol`.
     - When the player changes the selected row (via mouse click, Up/Down, or scrollbar), the save name box (`IDCTL_LOAD_NAME`) updates at once, and a **1.0-second (1000 ms) debounce timer** starts. Only after the selection has settled for 1000 ms does the game open `save/<file>.vol` and decode `current.bin`.
     - If no save is selected or `current.bin` is missing from the archive, the fallback bitmap `sl_notmb.pbm` is shown.
   - On **Save Game**:
     - On screen open, the live screenshot `<game>/current.bin` (captured when pausing gameplay into the main menu) is decoded and shown in `IDCTL_SAVE_THUMBNAIL`. It does not change when clicking existing save rows.
8. **Save name text entry (`Savegame.gui`).**
   - Selecting an existing row in the list copies that save's name into the name box (`IDCTL_SAVE_NAME`). On screen open, the box starts with the top row's name (or empty if no saves exist).
   - A blinking pipe character (`"|"`) is appended to the displayed text in `IDCTL_SAVE_NAME` and toggled on/off every **300 ms**.
   - **Typing**:
     - Printable ASCII characters (`0x20..0x7E`, including shifted punctuation and numpad characters) are appended to the save name while both of these limits hold:
       - Current length before insertion is **at most 29 characters** (maximum save name length is **30 characters**).
       - A weighted character-width sum (in tenths of a pixel: space = `50`, `'I'`/`'i'`/`'j'`/`'l'` = `50..62`, `'M'`/`'m'` = `200`, `'W'` = `222`, default = `125`) is **at most 2599** (`259.9` pixels) so the text never overflows the 310-pixel name box.
     - **Backspace** deletes the last character. Holding Backspace auto-repeats after a **500 ms** initial delay at **200 ms** intervals.
     -Note: While typing on the Save Game screen, letter keys and Space type into the name box; **Up/Down** arrows navigate the save list, **Left/Right** arrows move the button highlight (`Save` / `Delete` / `Cancel`), **Enter** triggers the highlighted button, and **Escape** cancels back to the Main Menu.
9. **Confirmation and error popups on Load and Save (`sl_win2.pbm`).**
   - While a popup is open, normal list/button input is blocked; **Left/Right** swap the two popup buttons (`Yes` / `Cancel`) and play `button1.aud`, **Enter** activates the highlighted popup button, and **Escape** closes the popup (same as `Cancel`).
   - **Delete confirmation** (clicking `Delete` when a save is selected):
     - Prompt: `500.MSG` `#119` (`"Delete saved game?"`) followed by a space and the selected save's name. If the combined string exceeds 38 characters, it is split into two lines at the space closest to the midpoint.
     - Buttons: **Yes** (`#29`) and **Cancel** (`#35`), with **Cancel** highlighted by default.
     - Confirming **Yes** deletes `save/<file>.vol` from disk, rescans `save/*.vol`, clamps `selectedRow` to the remaining save count, refreshes the list (and thumbnail on Load Game), and closes the popup.
   - **Overwrite confirmation** (activating `Save` when the stripped save name matches an existing save case-insensitively):
     - Prompt: `500.MSG` `#125` (`"Replace saved game?"`) followed by a space and the save name (split at the middle space if `> 38` characters).
     - Buttons: **Yes** (`#29`) and **Cancel** (`#35`), with **Cancel** highlighted by default.
     - Confirming **Yes** closes the popup and overwrites that existing `save<N>.vol` file.
   - **Maximum saves / low disk space error on Save** (activating `Save` with a new save name when `totalSaves >= 50` or free disk space `< 4 MB`):
     - Prompt: `500.MSG` `#166` (split into 2 or 3 lines).
     - Single centered button: **OK** (`#148`, highlighted). Pressing Enter, Escape, or clicking OK closes the popup without saving.
   - **Low disk space error on Load** (`restoreWorld` fails its 110 MB free-space check):
     - Prompt: 3-line message from `500.MSG` `#156`.
     - Single centered button: **OK** (`#148`, highlighted).

---

### 3. How `save<N>.vol` is written and loaded

10. **Writing a saved game (`save/save<N>.vol`).**
    - Leading spaces are stripped from the typed save name. If the resulting name is empty, activating `Save` does nothing.
    - If overwriting an existing save name, its existing `save<N>.vol` filename is reused. Otherwise, the game picks the **lowest unused integer `N >= 1`** (`save1.vol`, `save2.vol`, ...).
    - The pointer changes to the hourglass wait cursor (`IDBMA_WAITCURS`, tag `1022`), and the red progress bar `IDCTL_SAVE_RED` (`savered.pbm` inside `savebar.pba` at `(455, 165)`) advances from `0%` to `100%` as world objects are serialized.
    - First, the current world's state is written to `cur/<World>.vol` (and `cur/<currentMap>.kq` is removed so the current world's map is not stored twice).
    - Next, `save/save<N>.vol` is created as a standard `PVOL` archive containing:
      1. `savedGameName`: null-terminated ASCII string of the stripped save name.
      2. Current world chunks:
         - `worldName`: `<World>.kq` (e.g. `Daventry.kq`)
         - `savedGameVersion`: `"1.000000"`
         - Every persistent object `<name>.kq` in the current world
         - `worldState.kq`: current world flags (`firstTime`, `frozen`, `teleported`, `justLoaded`, camera mode)
         - `WorldVar.cs`: current world console variables
         - `newWorld.cs`: generated script that re-Execution-loads every saved `.kq` object on restore
      3. `current.bin`: the 19,204-byte screenshot thumbnail.
      4. `Connor.kq`: Connor's full state, inventory, health, experience, weapons, and armor.
      5. `<map>.kq`: the current world's automap object (e.g. `davMap.kq`).
      6. `GameState.kq`: global game state containing the `nextNoName` counter and **`difficultyFactor`** (so the combat difficulty slider setting is persisted per save file).
      7. Every `cur/*.vol` archive for visited worlds (including `cur/<currentWorld>.vol`), plus `worldVolumes` (a `|`-separated list of those `.vol` filenames).
      8. Every `cur/*.kq` automap file for previously visited worlds, plus `maps` (a `|`-separated list of those `.kq` filenames).
    - **Post-save transition**: once `save/save<N>.vol` is closed, the normal cursor (`Cursgui.pba`) is restored and the game transitions to **Main Menu (`screen 0`)** — not directly back to gameplay.
11. **Loading a saved game.**
    - Activating `Load` when a valid save is selected fades out `theme.aud` over 1 second and runs `restoreWorld`:
      1. Deletes all existing files in `cur/`.
      2. Finds the `save/save<N>.vol` archive matching the selected `savedGameName`.
      3. Checks `savedGameVersion` against `"1.000000"` and reads `worldName`.
      4. Reads `worldVolumes` and extracts each listed `<World>.vol` chunk from `save<N>.vol` into `cur/<World>.vol`.
      5. Reads `maps` and extracts each listed `<map>.kq` chunk from `save<N>.vol` into `cur/<map>.kq`.
      6. Activates the target world's Loading screen (`screen 3`, `<world>load.pbm`) and runs `Mask.cs RestoreWorld`, which loads `GameState.kq`, runs `<world>.cs Init` and `<world>.cs Load` (reading `WorldVar.cs`, `newWorld.cs`, `Connor.kq`, `<map>.kq`, and `worldState.kq` from the save volume), sets `game_running = 1`, unpauses the game, and switches to Gameplay (`screen 1`).

---

### 4. Options Menu (`Options.gui`)

12. **Nine rows (`0..8`) and two spinning mask cursors.**
    - The Options screen has nine rows, flanked by **two** 30×30 spinning mask cursors (`IDCTL_OPT_CURSOR1` at `x = 10..40` and `IDCTL_OPT_CURSOR2` at `x = 600..630`, both using `Mskicng.pba`), which move vertically together to center on the selected row:
      - `0`: **Graphics Menu** (`IDCTL_OPT_MODE_LABEL`, `#6` `"Graphics"`) -> opens Graphics (`screen 4`)
      - `1`: **Remap Move Keys** (`IDCTL_OPT_REMAP_MOVEKEYS`, `#60` `"Remap Move Keys"`) -> opens Remap Move Keys (`screen 7`)
      - `2`: **Music Volume** (`IDCTL_OPT_MUSICVOLUME_LABEL` `#13`, slider `IDCTL_OPT_MUSIC_VOLUME`)
      - `3`: **Sound Effects Volume** (`IDCTL_OPT_SFXVOLUME_LABEL` `#19`, slider `IDCTL_OPT_SFX_VOLUME`)
      - `4`: **Speech Volume** (`IDCTL_OPT_SPEECHVOLUME_LABEL` `#22`, slider `IDCTL_OPT_SPEECH_VOLUME`)
      - `5`: **Subtitles** (`IDCTL_OPT_SUBTITLES_LABEL` `#23`, value `IDCTL_OPT_SUBTITLES_TEXT`: `#29` `"Yes"` / `#14` `"No"`)
      - `6`: **Starting Help** (`IDCTL_OPT_STARTHELP_LABEL` `#127`, value `IDCTL_OPT_STARTHELP_BOOL`: `#29` `"Yes"` / `#14` `"No"`)
      - `7`: **Credits** (`IDCTL_OPT_CREDITS`, `#149` `"Credits"`) -> opens Credits (`screen 8`)
      - `8`: **Back to Main Menu** (`IDCTL_OPT_BACKTOMAIN`, `#25` `"Back to Main Menu"`) -> saves `Options.cs` and returns to Main Menu (`screen 0`)
13. **Navigation and toggling on Options.**
    - **Up / Down** moves the selection across enabled rows (`0..8`, wrapping around) and plays `click.aud`.
    - **Left / Right**:
      - On a volume slider row (`2`, `3`, `4`), steps the slider by **`-5` / `+5`** (snapped to a multiple of `5` in `0..100`).
      - On **Subtitles** (`5`) or **Starting Help** (`6`), toggles the value between `Yes` and `No` and plays `intrface.aud`.
    - **Enter** or **left mouse click**:
      - On `0` (Graphics), `1` (Remap Move Keys), or `7` (Credits), plays `intrface.aud` and switches to that submenu.
      - On `5` (Subtitles) or `6` (Starting Help) (clicking either the label or the Yes/No value text), toggles `Yes`/`No` and plays `intrface.aud`.
      - On `8` (Back to Main Menu), or pressing **Escape** from any row, writes `Options.cs`, plays `intrface.aud` (when triggered via button/Enter), and returns to Main Menu (`screen 0`).
14. **Volume sliders (`0..100` -> `0.0..1.0`).**
    - On screen open, each slider position is initialized to `round(volume * 100.0)`.
    - Dragging or clicking the 200-pixel slider track (`x = 330..530`) or pressing Left/Right updates the integer slider value `val` in `0..100`.
    - **Low-volume floor**: when converting `val` back to a float volume, if `0 < val < 10`, the game clamps `val` to `10` before multiplying by `0.01` (so any non-zero slider position produces a volume in `0.10..1.00`, while `0` produces `0.0` mute).
    - Moving a volume slider updates the corresponding audio group live (`Music` group `0x30`, `SFX` group `0x07`, `Speech` group `0x08`).
15. **Subtitles and Starting Help.**
    - **Subtitles** toggles the console variable `Options::Captions` between `1` (`Yes`) and `0` (`No`).
    - **Starting Help** controls the global `prohibitHelpPopup` state (`0` = show help popup on new game -> `Yes`; `1` = suppress help popup -> `No`). If `prohibitHelpPopup == 2` (locked off), both the label and value are disabled (drawn dithered/greyed) and display `No`.
16. **Persisting `Options.cs`.**
    - Leaving Options back to the Main Menu overwrites `<game>/Options.cs` with 10 console commands:
      ```
      KQSound::setSpeechVol <float>
      KQSound::setSfxVol <float>
      KQSound::setMusicVol <float>
      setShadowMask <int>
      useDynamicLighting <true|false>
      set Options::Captions <TRUE|FALSE>
      notStartingHelp <int>
      assignGModeName <Software|Direct3D|Glide>
      setBrightness <int>
      set800mode <0|1>
      ```

---

### 5. Graphics Menu (`Graphics.gui`)

17. **Twelve rows (`0..11`) and active markers.**
    - Flanked by two spinning mask cursors (`IDCTL_GRA_CURSOR1` at `x = 10..40`, `IDCTL_GRA_CURSOR2` at `x = 600..630`):
      - `0`: **DirectDraw** (`IDCTL_GRA_DIRECTDRAW`, `#7` `"DirectDraw"`)
      - `1`: **Direct3D** (`IDCTL_GRA_DIRECT3D`, `#5` `"Direct3D"`)
      - `2`: **3Dfx** (`IDCTL_GRA_3DFX`, `#1` `"3Dfx"`)
      - `3`: **640x480** (`IDCTL_GRA_640X480`, `#116` `"640x480"`)
      - `4`: **800x600** (`IDCTL_GRA_800X600`, `#117` `"800x600"`)
      - `5`: **Dynamic Lights** (`IDCTL_GRA_DYNAMIC_LIGHTS` `#8`, value `IDCTL_GRA_DYNAMIC_LIGHT_BOOL`: `#17` `"On"` / `#16` `"Off"`)
      - `6`: **Connor Shadows** (`IDCTL_GRA_CONNOR_SHADOWS` `#24`, value `IDCTL_GRA_CONNOR_SHADOW_BOOL`, bit `1` = `0x01` of `shadowMask`)
      - `7`: **Monster Shadows** (`IDCTL_GRA_MONSTER_SHADOWS` `#26`, value `IDCTL_GRA_MONSTER_SHADOW_BOOL`, bit `1` = `0x02` of `shadowMask` — matches default `KQMonster` `shadowMask=2` including the Wizard in `characters.md` rule 36, confirmed live 2026-11-24)
      - `8`: **Character Shadows** (`IDCTL_GRA_CHARACTER_SHADOWS` `#27`, value `IDCTL_GRA_CHARACTER_SHADOW_BOOL`, bit `2` = `0x04` of `shadowMask` — matches explicit `shadowMask=4` NPCs such as the Girl, Ferryman, Archon)
      - `9`: **Object Shadows** (`IDCTL_GRA_OBJECT_SHADOWS` `#28`, value `IDCTL_GRA_OBJECT_SHADOW_BOOL`, bit `4` = `0x08` of `shadowMask`)
      - `10`: **Brightness** (`IDCTL_GRA_BRIGHTNESS` `#2`, slider `IDCTL_GRA_BRIGHTNESS_SLIDER`)
      - `11`: **Back to Options Menu** (`IDCTL_GRA_BACKTOMAIN`, `#31` `"Back to Options Menu"`) -> returns to Options (`screen 2`)
    - The currently active renderer (`0..2`) and active resolution (`3..4`) are displayed wrapped in angle brackets: `"> Direct3D <"`, `"> 640x480 <"`.
18. **Renderer availability and feature restrictions.**
    - Renderers not supported by the system (e.g. `3Dfx` when no Glide driver is present, or `DirectDraw` when running in 800×600) are disabled and drawn greyed out.
    - In **DirectDraw (Software)** mode:
      - `800x600` (`4`), `Character Shadows` (`8`, bit `0x02`), and `Object Shadows` (`9`, bit `0x08`) are disabled (greyed out), and bits `0x02` and `0x08` are cleared from `shadowMask`.
      - Switching from Direct3D to DirectDraw opens a confirmation popup (`IDCTL_GRA_POP_BKGND`, `#169`, buttons `Yes` `#29` / `Cancel` `#35`, default `Cancel`). Confirming `Yes` writes `Options.cs` with `assignGModeName Software` and `set800mode 0` and exits the executable (`PostQuitMessage(0)`), because the original engine could not switch from Direct3D back to DirectDraw in-process.
    - In **3Dfx (Glide)** mode:
      - `Dynamic Lights` (`5`) is forced on, its row is disabled (greyed out), and its value displays `#137` (`0x455`) instead of `On`/`Off`.
    - In **Direct3D** mode:
      - All four shadow toggles (`6..9`), `Dynamic Lights` (`5`), and both resolutions (`3..4`) are enabled.
19. **Brightness slider (`0..100`, default `50`).**
    - Left/Right arrows step the brightness slider by **`-5` / `+5`** (`0..100`).
    - The integer slider value `val` (`0..100`) maps linearly to the renderer brightness factor in **`0.3 .. 1.7`**:
      $$\text{brightness} = 0.3 + \frac{\text{val}}{100.0} \times 1.4$$
      At `val = 0`, brightness is `0.3`; at `val = 50` (default), brightness is `1.0`; at `val = 100`, brightness is `1.7`.

---

### 6. Remap Move Keys (`movekey.gui`, `movekeys.kq`)

20. **Fourteen rows (`0..13`) and key table.**
    - Flanked by two spinning mask cursors (`IDCTL_MOVEKEYS_CURS1` at `x = 120..150`, `IDCTL_MOVRKEYS_CURS2` at `x = 490..520`):
      - `0`: **Forward** (`#36`, default `KeyChar0 = 38`, `KeyName0 = "#77"` `"Up Arrow"`, `KeyWord0 = "up"`)
      - `1`: **Back** (`#37`, default `KeyChar1 = 40`, `KeyName1 = "#78"` `"Down Arrow"`, `KeyWord1 = "down"`)
      - `2`: **Right** (`#39`, default `KeyChar2 = 39`, `KeyName2 = "#80"` `"Right Arrow"`, `KeyWord2 = "right"`)
      - `3`: **Left** (`#38`, default `KeyChar3 = 37`, `KeyName3 = "#79"` `"Left Arrow"`, `KeyWord3 = "left"`)
      - `4`: **Run** (`#40`, default `KeyChar4 = 35`, `KeyName4 = "#81"` `"End"`, `KeyWord4 = "end"`)
      - `5`: **Jump** (`#44`, default `KeyChar5 = 45`, `KeyName5 = "#82"` `"Insert"`, `KeyWord5 = "insert"`)
      - `6`: **Switch Weapon** (`#53`, default `KeyChar6 = 46`, `KeyName6 = "#83"` `"Delete"`, `KeyWord6 = "delete"`)
      - `7`: **Center Camera** (`#58`, default `KeyChar7 = 36`, `KeyName7 = "#84"` `"Home"`, `KeyWord7 = "home"`)
      - `8`: **Open/Close Map** (`#59`, default `KeyChar8 = 9`, `KeyName8 = "#85"` `"Tab"`, `KeyWord8 = "tab"`)
      - `9`: **Modify Action** (`#63`, default `KeyChar9 = 16`, `KeyName9 = "#86"` `"Shift"`, `KeyWord9 = "shift"`)
      - `10`: **Toggle Cursor** (`#64`, default `KeyChar10 = 17`, `KeyName10 = "#87"` `"Ctrl"`, `KeyWord10 = "control"`)
      - `11`: **Put Away** (`#113` `IDCTL_MOVEKEYS_BTMBAR` at `y = 368`, default `KeyChar11 = 18`, `KeyName11 = "#99"` `"Alt"`, `KeyWord11 = "alt"`)
      - `12`: **Toggle Interface** (`#66` `IDCTL_MOVEKEYS_INVBAR` at `y = 396`, default `KeyChar12 = 32`, `KeyName12 = "#88"` `"Space"`, `KeyWord12 = "space"`)
      - `13`: **Back to Options Menu** (`#31` `IDCTL_MOVEKEYS_TO_MAIN` at `y = 432`) -> saves `movekeys.kq` and returns to Options (`screen 2`).
21. **Rebinding a key and conflict resolution.**
    - Pressing **Enter** on row `0..12` or left-clicking row `0..12` enters **key-capture mode** for that row:
      - The row's key label changes to `500.MSG` `#112` (`"???"`).
      - While key-capture mode is active, normal menu navigation (`Up`/`Down`, `Escape`, clicking `Back to Options Menu`) is locked out until a valid key is bound.
    - When the player presses a key in key-capture mode:
      - **Reserved keys rejected**: `F2` (`0x71`), `F11` (`0x7A`), `F12` (`0x7B`), `Escape` (`0x1B`), `Enter` (`0x0D`), `Page Up` (`0x21`), `Page Down` (`0x22`), top-row digits `'1'..'8'` (`0x31..0x38`), numpad `'1'..'9'` (`0x61..0x69`), and any key not in the game's bindable key table are rejected. Pressing a rejected key sets the row's right-hand label to `500.MSG` `#67` (`"Invalid Key"`) and stays in key-capture mode waiting for another key.
      - **Duplicate key displacement**: if the pressed key is valid, the game checks whether any of the other 12 actions (`0..12`) already uses that virtual key code:
        - If no other action uses it, the key is bound to the current row, its display name and `.sae` action binding are updated, and key-capture mode ends.
        - If another action `otherRow` already uses that key, the new key is bound to the current row, **`otherRow` is unbound** (`KeyChar = 99`, label set to `#112` `"???"`), the selection and spinning mask cursors immediately jump to `otherRow`, and **key-capture mode stays active on `otherRow`** so the player must choose a replacement key for the displaced action before they can leave the screen.
22. **Persisting `movekeys.kq`.**
    - Activating `Back to Options Menu` (`13`) or pressing **Escape** (when not in key-capture mode) writes `KeyChar0..12`, `KeyName0..12`, `KeyWord0..12`, `moveSound`, and `selectSound` to `<game>/movekeys.kq`, plays `intrface.aud`, and switches to Options (`screen 2`).

---

### 7. Credits Screen (`credits.gui`)

23. **26 timed pages from `500.MSG`.**
    - Opening Credits (`screen 8`) starts at page `0` and steps through **26 pages (`0..25`)**, reading `500.MSG` entry `(noun = 0, verb = 0, case = page + 1, seq = 1)` (cases `1..26`).
    - On the final page (`page 25`, `500.MSG` case `26`), `%d` in the message text is formatted with Connor's current experience/score (`0` if no game is loaded).
    - Each page's string is split on `'\n'` into lines:
      - Lines starting with `'*'` strip the leading `'*'` and render in the **small font** (`20sl.pft`, control slots `IDCTL_1135..1150`, up to 16 lines, **17 px** line height).
      - All other lines render in the **large font** (`27sl.pft`, control slots `IDCTL_1122..1134`, up to 13 lines, **30 px** line height).
    - The active lines on a page are vertically centered on the screen (starting at `top_y = (screen_height - total_text_height) / 2`, where `screen_height` is `480` in 640×480 or `600` in 800×600).
24. **Page timing and final team photo (`kqteam08.pbm`).**
    - Pages `0..24` stay on screen for:
      $$\text{duration} = 2000 + 500 \times \text{line\_count}\text{ ms}$$
    - The final page (`page 25`, case `26`) displays the 400×300 team photograph `kqteam08.pbm` (`IDCTL_1162`, at `(120, 10)-(520, 310)`), shifts its text block down by `+145` px (`0x91`) so it sits below the photo, and stays on screen for:
      $$\text{duration} = 14000 + 500 \times \text{line\_count}\text{ ms}$$
      (`4000` ms base + `500` ms per line + `10000` ms extra hold).
25. **Exiting Credits.**
    - Pressing **Escape** at any time during the credits, or letting page `25` finish its timer, fades out `theme.aud` over 1 second and returns to the **Main Menu (`screen 0`)**.

---

### 8. Replay Intro and AVI Movie Playback (`playMovie`)

26. **Three AVI movie files named `.dll`.**
    - The game ships three full-motion video files in the root `<game>/` folder, stored as standard RIFF AVI files (`640×480`, `15.0` fps / `66,667` µs per frame, Intel Indeo Video 5.0 `iv50` video stream + PCM audio stream) with `.dll` extensions:
      - `W32opn_1.dll`: opening intro movie (`1609` frames, ~107.3 s)
      - `W32mdl_1.dll`: mid-game movie (`255` frames, ~17.0 s)
      - `W32ndg_1.dll`: ending movie (`774` frames, ~51.6 s)
27. **Where movies are triggered.**
    - **Game startup**: plays `w32opn_1.dll` once before entering the initial game flow, with skipping enabled (`allowSkip = true`).
    - **Main Menu -> `Replay Intro`**:
      1. Fades out the main menu music `theme.aud` over 1 second.
      2. Calls `playMovie("w32opn_1.dll", allowSkip = true)`.
      3. Clears the menu music playing flag and sets the music restart timer to `now + 200 ms`, so `theme.aud` starts playing again from the beginning 200 ms after the movie ends or is skipped, remaining on the Main Menu (`screen 0`).
    - **Script command `playMovie <filename>`**:
      - Used by world cutscenes (e.g. `W32mdl_1.dll`, `W32ndg_1.dll`). Calls `playMovie(filename, allowSkip = false)` (Wait: in the original engine, the script command passes `allowSkip = 0`, whereas startup and Replay Intro pass `allowSkip = 1`).
28. **Playback and skipping behavior.**
    - If the movie file does not exist on disk (`fopen` fails), `playMovie` returns immediately without error.
    - Before opening the movie, the game suspends its sound engine, switches out of 3Dfx/800×600 into a 640×480 window if needed, creates an MCI video window (`MCIWndCreateA`), reduces the MCI volume slightly (`current_vol - 10`), drains up to 20 pending Windows messages, and starts playback.
    - While the movie is playing (polling `MCIWNDM_GETMODE` every `50 ms` until `MCI_MODE_STOP`):
      - When `allowSkip` is true, any `WM_KEYDOWN`, `WM_KEYUP`, `WM_LBUTTONDOWN`, or `WM_RBUTTONDOWN` event sends `MCI_STOP` (`0x808`) to stop playback immediately.
    - When playback stops, the MCI window is destroyed, the previous renderer / resolution / palette / brightness are restored, and the game's sound engine is resumed.

## Numbers and units

| Quantity | Value | Unit / meaning |
|---|---|---|
| GUI screen indices | `0..8` | `0` Main Menu, `1` Gameplay, `2` Options, `3` Loading, `4` Graphics, `5` Load, `6` Save, `7` Move Keys, `8` Credits |
| Save/Load visible list rows | `10` | `LIST0..LIST9` at `x = 95`, `y = 185..435` (25 px per row) |
| Save/Load scrollbar thumb span | `190` | pixels (`y = 205..395` in 640×480; `y = 265..455` in 800×600) |
| Save/Load track page scroll | `10` | rows per click above/below thumb |
| Maximum saved games | `50` | `save/*.vol` count limit before Save shows `#166` popup |
| Minimum free disk space to save | `4` | MB (`4,194,304` bytes) before Save shows `#166` popup |
| Minimum free disk space to load | `110` | MB (`115,343,360` bytes) before Load shows `#156` popup |
| Load thumbnail debounce delay | `1000` | ms after changing selected row before decoding `current.bin` |
| Save name caret blink interval | `300` | ms between toggling trailing `"\|"` on and off |
| Save name Backspace repeat | `500` / `200` | ms initial delay / ms repeat interval while held |
| Save name maximum length | `30` | characters (`<= 29` before insertion) |
| Save name maximum weighted width | `259.9` | pixels (`2599` tenths of a pixel) |
| Volume slider range and arrow step | `0..100`, step `5` | non-zero values `< 10` clamp to `10` (`0.10..1.00`), `0` is `0.0` |
| Brightness slider range and formula | `0..100` (default `50`) | `brightness = 0.3 + (val / 100.0) * 1.4` (range `0.3..1.7`) |
| Shadow mask bits (`setShadowMask`) | `0x01..0x0F` | bit `0x01` Connor, `0x02` Monster, `0x04` Character, `0x08` Object |
| Configurable move keys | `13` | actions `0..12` in `movekeys.kq` (plus row `13` Back to Options) |
| Credits page count | `26` | pages `0..25` (`500.MSG` cases `1..26`) |
| Credits line heights | `30` / `17` | pixels (`27sl.pft` large lines / `20sl.pft` `'*'`-prefixed small lines) |
| Credits page duration (`0..24`) | `2000 + 500 * lines` | ms |
| Credits final page duration (`25`) | `14000 + 500 * lines` | ms (with `kqteam08.pbm` 400×300 at `(120, 10)`) |
| AVI movie format (`W32*.dll`) | `640×480`, `15.0` fps | Intel Indeo Video 5.0 (`iv50`) AVI container |
| Replay Intro music restart delay | `200` | ms after movie ends before `theme.aud` restarts |

## Data used

- **Archive `GAME/Kq/KqGui.kq`** (and `MASK/movekeys.kq`, `MASK/Options.cs`, `MASK/save/save<N>.vol`):
  - **Load Game**: `Loadgame.kq`, `Loadgame.gui`, `saveload.pbm` (`save4x6.pbm` / `save6x8.pbm`), `sl_hilite.pba`, `sl_buton.pba`, `sl_hibtn.pba`, `sl_scton.pba`, `sl_scbon.pba`, `sl_scbox.pba`, `sl_notmb.pbm`, `sl_win2.pbm`, `s_ybtn.pba`, `s_yhibtn.pba`, fonts `20sl.pft`, `27sl.pft`, `consoles.pft`.
  - **Save Game**: `Savegame.kq`, `Savegame.gui`, plus `savebar.pba` and `savered.pbm` (progress bar), and `<game>/current.bin` (live 19,204-byte thumbnail).
  - **Options**: `Options.kq`, `Options.gui`, `options.pbm` (`opt4x6.pbm` / `opt6x8.pbm`), `Mskicng.pba` (two spinning mask cursors), `sldrbar.pba`, `sldrtab.pba`, `20o.pft`, `27o.pft`, `27oh.pft`, `<game>/Options.cs`.
  - **Graphics**: `Graphics.kq`, `Graphics.gui`, `options.pbm`, `Mskicng.pba`, `sldrbar.pba`, `sldrtab.pba`, `sl_win2.pbm`.
  - **Remap Move Keys**: `<game>/movekeys.kq`, `movekey.gui`, `options.pbm`, `Mskicng.pba`, `15m.pft`, `15mh.pft`, `20o.pft`, `27o.pft`, `27oh.pft`.
  - **Credits**: `credits.KQ`, `credits.gui`, `options.pbm`, `kqteam08.pbm` (400×300 team photo), `20sl.pft`, `27sl.pft`, `500.MSG` cases `1..26`.
  - **Movies**: `<game>/W32opn_1.dll`, `<game>/W32mdl_1.dll`, `<game>/W32ndg_1.dll`.

## Edge cases

- **Saving returns to the Main Menu, not Gameplay.** After a save completes, the game calls `selectScreen(0)` (Main Menu) with `theme.aud` still playing and the game still paused. The player then clicks `Resume Game` or presses `Escape` on the Main Menu to return to gameplay.
- **Left/Right arrows on Load and Save move vertically across `Load`/`Save`, `Delete`, and `Cancel`.** Because `Up`/`Down` arrows are dedicated to the 10-row save list, keyboard selection among the three right-hand action buttons uses `Left` and `Right` arrows (swapping the golden highlight `sl_hibtn.pba` across the three stacked buttons without wrapping).
- **Duplicate key binding forces immediate re-bind of the displaced action.** You cannot leave `Remap Move Keys` with two actions sharing a key or with an unbound `"???"` action: binding a key already used by another row immediately jumps the `"???"` prompt to the displaced row and keeps `Escape` and `Back to Options Menu` locked until that displaced row is given a unique key.
- **Credits exits to the Main Menu (`screen 0`), not Options (`screen 2`).** Even though `Credits` is opened from a button inside `Options.gui`, both pressing `Escape` during the credits and reaching the end of page `25` fade out `theme.aud` and switch to `screen 0` (Main Menu, which restarts `theme.aud` on activation).
- **Combat difficulty is saved in `GameState.kq` inside `save<N>.vol`.** `Options.cs` does not store `difficultyFactor`; instead, `GameState.kq` inside each `save<N>.vol` stores `difficultyFactor` alongside `nextNoName`, while changing the slider on the Main Menu updates the live global immediately.

## Do not reproduce

- **110 MB / 4 MB free disk space checks (`GetDiskFreeSpaceA`).** Just as noted in [world-change.md](file:///c:/Games/KQ8R/docs/behaviour/world-change.md), the original 32-bit disk space arithmetic is unnecessary and prone to overflow on modern multi-terabyte drives.
- **Quitting the game when switching renderer (`PostQuitMessage(0)`).** Our wgpu viewer uses a single modern hardware renderer, so the legacy DirectDraw / Direct3D / 3Dfx mode switch and quit-on-DirectDraw popup do not apply to the modern renderer (if the Graphics screen is shown for fidelity, renderer/resolution rows can either reflect the viewer's mode or be informational while keeping Dynamic Lights, the four Shadow masks, and Brightness functional).
- **Win32 `SetCursorPos` on `Enter`.** As noted in [main-menu.md](file:///c:/Games/KQ8R/docs/behaviour/main-menu.md) rule 16, `Mask.exe` calls Win32 `SetCursorPos` when pressing `Enter` on Main Menu, Load, Save, and Graphics, but because the game renders its own software cursor (`Cursgui.pba`) at internal coordinates and blocks mouse-move updates while the Enter latch is held, the visible software cursor never jumps. Do not jump the visible cursor when pressing `Enter`.

## How to check

1. **Save & Load round-trip**:
   - Start a new game or load Daventry, press `Escape` to open the Main Menu, and verify `current.bin` is captured and `Save Game` is enabled.
   - Open `Save Game`: verify existing `save/save<N>.vol` files are listed in descending `<N>` order, the live thumbnail is shown at `(428, 316)`, the `"|"` caret blinks every 300 ms in the name box, typing/Backspace edits the name (up to 30 chars / 259.9 px), `Left`/`Right` moves the golden highlight across `Save`, `Delete`, and `Cancel`, saving writes a valid `save<N>.vol` (or prompts `Replace saved game?` on an existing name) and returns to the Main Menu.
   - Open `Load Game`: verify moving across rows updates the name immediately and loads the save's `current.bin` thumbnail after 1.0 s, `Delete` opens the `Delete saved game? <name>` confirmation popup with `Cancel` highlighted by default, and `Load` fades out `theme.aud` and restores the saved world, Connor's state, and `difficultyFactor`.
2. **Options, Graphics, Remap Move Keys, and Credits**:
   - Open `Options`: verify the two spinning mask cursors move together across all 9 rows, `Music`/`SFX`/`Speech` sliders step by 5 with `Left`/`Right` and update audio volumes live (with the `10%` floor for non-zero values), `Subtitles` and `Starting Help` toggle between `Yes` and `No`, and `Back to Main Menu` (or `Escape`) persists `Options.cs`.
   - Open `Graphics`: verify the four shadow toggles (`Connor`, `Monster`, `Character`, `Object`), `Dynamic Lights`, and `Brightness` (`0.3..1.7`, default `50 -> 1.0`) update live.
   - Open `Remap Move Keys`: verify all 13 actions display their bound key names from `movekeys.kq`, pressing `Enter` or clicking an action prompts `???`, reserved keys (`F2`, `F11`, `F12`, `Escape`, `Enter`, `PageUp`, `PageDown`, `1..8`, `Numpad 1..9`) show `Invalid Key`, binding an already-used key displaces the old action and immediately prompts `???` on the displaced row, and `Back to Options Menu` writes `movekeys.kq`.
   - Open `Credits`: verify pages `0..25` (`500.MSG` cases `1..26`) advance with `2000 + 500 * lines` ms timing (`14000 + 500 * lines` ms and `kqteam08.pbm` on page 25) and `Escape` or completion fades out `theme.aud` and returns to the Main Menu.
3. **Replay Intro**:
   - Click `Replay Intro` on the Main Menu: verify `theme.aud` fades out, `W32opn_1.dll` plays (and can be skipped with any key or mouse click), and `theme.aud` restarts 200 ms after returning to the Main Menu.

## Open questions

None.
