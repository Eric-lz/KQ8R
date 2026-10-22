# Main menu

Study session A, 2026-10-21. Sources: `Mask.exe` (traced), the layout
`Mainmenu.gui` and the tag file (data, read by `kq8 gui-verify`),
`GAME/Kq/Mainmenu.kq`, `GAME/ENGLISH/500.MSG` (texts), `starttxt.txt`,
`mask.cs`. Marks: **[traced]** = read in the program, **[data]** = in the
game files, **[inferred]** = concluded, not seen directly.

## Purpose

The screen the game starts on and the one Escape opens during play. It
starts a new game (after asking for the combat difficulty), opens the
Load, Save and Options screens, replays the intro film, quits, or goes
back to the game. While it is open the game is paused.

## Rules

### The screen

1. **Files** [data]: `Mainmenu.kq` (class `KQMainMenuScreen`): layout
   `Mainmenu.gui` from `GAME/8Gui` (the 8-bit menu folder), palette
   `Menus.ppl`, background colour 0, `useWorldPal=0`, pointer
   `IDBMA_CURSGUI` (`Cursgui.pba`), theme music `theme.aud`, move sound
   `click.aud`, select sound `intrface.aud`. The background picture is
   `main18.pbm` (640 × 480).
2. **The items**, top to bottom, in the 640 × 480 layout (all 560 × 40,
   left edge x 64, font `27.pft`) [data]; the number is the item's place
   in the menu's list [traced]:

   | Place | Item | Text (`500.MSG`) | Rows |
   | --- | --- | --- | --- |
   | 0 | New | "New" | 125–165 |
   | 1 | Load | "Load" | 165–205 |
   | 2 | Save | "Save" | 205–245 |
   | 3 | Options | "Options" | 245–285 |
   | 4 | Replay Intro | "Replay Intro" | 285–325 |
   | 5 | Quit | "Quit" | 345–385 |
   | 6 | Back to Game | "Back to Game" | 405–445 |

   Places 7–9 are the three popup text lines (rule 14); they can never be
   chosen. The gap at 325–345 and 385–405 is in the layout.
3. **Which items can be chosen** (worked out each time the menu is set
   up) [traced]. "A game is running" means a New game was started or a
   saved game was loaded since the program started, and Connor has not
   died since (rule 25):
   - New, Options, Quit: always.
   - Load: only if there is at least one saved game in the save folder
     (rule 23) and no "corrupt saved game" question is waiting.
   - Save, Back to Game: only while a game is running.
   - Replay Intro: always (except in the demo version).
   An item that can't be chosen is **drawn greyed**: its text is
   see-through / dithered, the same look as the Invisibility potion's
   dither on Connor (`potions.md`) [the user's check in the original,
   2026-10-21]. A plain grey or half-transparent text is an acceptable
   stand-in until dithering exists. Up/Down skip it, a click on it does
   nothing [traced; Up/Down also checked by the user].
4. **Title** [traced, data]: the two lines at the top (`IDCTL_MAINMENU_TITLE1`
   y 20–50, `TITLE2` y 66–96, font `45.pft`, with the line picture
   `45-line.pbm` between them) are the **first two lines of
   `starttxt.txt`** in the game folder: "KING'S QUEST" and "MASK of
   ETERNITY". Line breaks become spaces.
5. **The mask cursor** (`IDCTL_MAINMENU_CURSOR`, `Mskcur.pba`, 40 × 40 at
   x 16) marks the chosen item: it moves up or down so that its middle
   is level with the middle of that item [traced]. With the 40-high
   items it sits exactly in each row.

### Opening and closing

6. **At program start** the menu is the first screen (`mask.cs` `Init`:
   `KQGame::selectScreen 0`) [data]. No game is running, so New is
   chosen and Save / Back to Game are off.
7. **Escape during play** (and the mask button on the top bar,
   `interface.md` rules 12, 14) opens it, except during `handsOff`
   [traced]. In order:
   1. the game-play panels are hidden;
   2. **the game is paused**: the world clock stops (nothing moves,
      timers and potion durations wait), and every sound is paused
      except the menu sounds and the menu theme (`sound.md` rule 54;
      groups 1 and 4 keep playing) [traced];
   3. a **160 × 120 thumbnail** of the screen is saved for the Save
      screen (rule 22);
   4. the "Connor has died" mark is cleared (rule 25);
   5. the menu screen is shown (palette `Menus.ppl` replaces the
      world's).
8. **Each time the menu screen is shown** (not when it comes back from
   Load, Save or Options, rule 11) [traced]:
   - the game's sounds are paused and **`theme.aud` starts** in the
     menu-theme group (music slider) [traced], at full loudness with
     **no fade-in**, as soon as the menu shows [the user's check,
     2026-10-21]; that it loops is [inferred];
   - the chosen item is **Back to Game** if a game is running, else
     **New**, and the mask cursor goes there;
   - the save folder is scanned (rule 23) and Load is switched on or off;
   - **clicks and keys are ignored for 0.2 s** (real time) so the key or
     click that opened it does nothing on the menu.
9. **Back to Game** (the item, or **Escape**, rule 16) [traced]:
   - the theme **fades out over 1 s** (volume falls at 1.0 per second)
     and is thrown away;
   - the game-play screen is shown, the pause ends (paused sounds
     resume, the world clock runs again from where it stopped, no
     catch-up), and the world's palettes are put back (`loadPalettes 0`,
     `interface.md` rule 1d).
   Nothing else changes: Connor, the camera and every timer are as they
   were when Escape was pressed.

### Choosing an item

10. **New** [traced]:
    - If a game is running: first a Yes/No question, "You already have
      a game running. Are you sure you want to start a NEW one?"
      (`500.MSG` 0 0 158), **No** highlighted. No closes it; Yes goes on.
    - Then a **disk-space check** of the original's install (rule 30):
      if it fails, a message with an OK button; OK closes it and nothing
      starts.
    - Then the **Combat difficulty** question (rule 15): Easy, Normal or
      Hard, **Normal** highlighted.
    - Choosing one sets the **combat level** (Easy 0.66, Normal 1.0,
      Hard 1.5; `fighting.md` rule 3), the theme fades out (1 s), and a
      new game starts in the first world (Daventry; the world load is
      `mask.cs` `NewWorld`, another note). From then on a game is
      running.
    - In the demo version there is no question: the level is set to
      0.66 and the game starts.
11. **Load, Save, Options** [traced]: open the Load screen, the Save
    screen or the Options screen. The theme **keeps playing** and the
    game stays paused; when those screens come back to the menu, rule 8
    is not repeated (the theme does not restart). What those screens do
    is for other notes.
12. **Replay Intro** [traced]: plays the opening film (`W32opn_1`, in
    the game folder), then shows the menu again with input blocked for
    0.2 s.
13. **Quit** [traced]: a Yes/No question, "Are you sure you want to exit
    the game?" (`500.MSG` 0 0 159), **No** highlighted. Yes ends the
    program at once (no save, no further question). No closes it.

### Popups

14. **One popup at a time**, drawn over the menu in the middle of the
    screen [traced, data]: the background `gm_win2.pbm` (360 × 120) at
    (140, 180)–(500, 300); up to three text lines (`consoleg.pft`,
    centred, 300 wide, x 170–470); two or three buttons 88 wide
    (`g_ybtn.pba` normal, `g_yhibtn.pba` highlighted, text
    `consoleg.pft`). Places used (y, 640 × 480) [traced]:

    | Popup | Text lines (top y, 20 high) | Buttons (top y, height) |
    | --- | --- | --- |
    | Yes/No question (New, corrupt save) | 200, 220 | Yes left, No right: 250, 36 |
    | Quit question | 215 | Yes left, No right: 250, 36 |
    | Combat difficulty | 215 | Easy, Normal, Hard: 250, 36 |
    | Disk space | 197, 215, 233 | OK (middle): 260, 25 |

    The buttons' x come from the layout: two buttons at x 188 and 361;
    three at x 158, 274, 392 [data]. Texts longer than 38 characters are
    split into two lines at the space nearest the middle; the disk text
    into three [traced]. Hidden popup parts are moved off screen.
15. **The highlight** [traced]: one button is highlighted. **Left/Right
    arrows** move it one button (no wrap-around, a click sound plays
    even when it can't move). Opening a popup puts it on its default
    (No; Normal; OK).
16. **Choosing a button**:
    - **mouse click** on a button picks that button at once;
    - **Enter**: pressing it shows the highlighted button pressed down
      and moves the mouse pointer onto it (16 pixels below its middle);
      **letting go** of Enter puts the pointer back where it was and
      picks the button [traced];
    - **Escape**: in the difficulty popup it closes the popup (back to
      the menu, nothing starts); in the other popups it does the same
      as Back to Game when a game is running (the popup is dropped), and
      nothing when no game is running [traced].
17. While a popup is open, **Up/Down still move the mask cursor** on the
    list underneath; the popup keeps the keyboard's Enter [traced].

### Keyboard and mouse on the list

18. **Up / Down**: the chosen item moves one place up or down,
    **wrapping around** (from New, Up goes to Back to Game if it is on,
    else to Quit), skipping items that are off [traced]. The move sound
    (`click.aud`) plays.
19. **Enter** picks the chosen item (on the key going down). The select
    sound (`intrface.aud`) plays first [traced].
20. **One action per key press**: holding a key does not repeat it; the
    next action needs the key let go [traced].
21. **Mouse**: a left click on an item's text chooses it and picks it at
    once (no move sound). Back to Game only reacts while a game is
    running. Moving the mouse moves the pointer; nothing was found that
    moves the mask cursor when the pointer only hovers over an item
    (not sure; "Open questions" 1).

### Data the menu writes or reads

22. **The thumbnail** (rule 7.3) [traced]: the screen is sampled every
    (width / 160)-th column and (height / 120)-th row (nearest pixel, no
    smoothing) into 160 × 120 palette indices. A 24-bit screen is turned
    into palette indices by picking, for each pixel, the palette colour
    with the smallest |ΔR| + |ΔG| + |ΔB|. The original writes it to
    `current.bin` in the game folder: a 4-byte number (the palette's id
    for an 8-bit screen, 999 for a 24-bit one) then 19,200 bytes
    (`MASK/current.bin` is 19,204 bytes) [data]. The Save screen uses it.
23. **The save folder scan** [traced]: the game makes `<game>/save` if
    missing and lists `save/*.vol`. Any file found switches Load on. A
    file whose `savedGameName` entry can't be read brings up the Yes/No
    question "A corrupt saved game was detected. Delete it?" (`500.MSG`
    0 0 157), No highlighted; Yes deletes the file, No keeps it. Only
    one such question per scan (the first unreadable file).
24. **Combat level** (rule 10) is the game-wide factor of
    `fighting.md` rule 3 (also set by the `setCombatLevel 1|2|3`
    command: 0.66 / 1.0 / 1.5).
25. **Connor's death** [traced]: when Connor dies, the game-play panels
    are hidden, a "Connor has died" mark is set, the game pauses and the
    **main menu opens**. With the mark set, Save and Back to Game are
    off and New is chosen, so the player must Load, start New, or quit.
    (What happens before that, the death itself, is `fighting.md` rule
    29.)

## Numbers and units

| What | Value | Mark |
| --- | --- | --- |
| Input ignored after the menu is shown or the film ends | 0.2 s (real time) | traced |
| Theme fade-out when leaving for the game | volume 1 → 0 at 1.0 / s (1 s) | traced |
| Combat level Easy / Normal / Hard | 0.66 / 1.0 / 1.5 | traced |
| Thumbnail | 160 × 120, 1 byte per pixel, + 4-byte header | traced, data |
| Popup text split | longer than 38 characters → two lines | traced |
| Mask cursor | centred on the item's height (factor 0.5) | traced |
| Item rows | 40 high, from y 125 (layout) | data |
| Enter-press pointer place | button's middle, 16 px lower | traced |

## Data used

- `GAME/Kq/Mainmenu.kq`, `GAME/Kq/KqGui.kq` (screen list: main menu is
  screen 0) [data].
- `Mainmenu.gui` and `kqGuiTag.TTAG` in `GAME/Resource` (control ids
  9 New, 10 Load, 11 Save, 14 Options, 500 Replay Intro, 12 Quit, 502
  Back, 146 mask cursor, 986/988/989 popup lines, 1004 popup background,
  1006 / 1007 two-button pair, 1008 / 1010 / 1011 three-button set)
  [data].
- Texts from `GAME/ENGLISH/500.MSG`, noun 0 verb 0, case = the number:
  2 Back to Game, 10 Load, 13 New, 19 Options, 20 Quit, 22 Replay Intro,
  24 Save, 29 Yes, 14 No, 148 OK, 120 Combat difficulty, 121 Easy,
  122 Normal, 123 Hard, 156 disk space, 157 corrupt save, 158 new game
  question, 159 quit question [data; which text each popup uses:
  traced]. The `#n` texts in the layout (e.g. `#119`, `#124 Retry`,
  `#118 Please Insert...`) are placeholders the program replaces.
- `GAME/8Gui`: `main18.pbm`, `45-line.pbm`, `Mskcur.pba`, `gm_win2.pbm`,
  `g_ybtn.pba`, `g_yhibtn.pba`, `gm_btn.pba`, fonts `27.pft`, `45.pft`,
  `consoleg.pft`, `Menus.ppl`, `Cursgui.pba` [data].
- `GAME/SOUND` (or wherever the sound volumes put them): `theme.aud`,
  `click.aud`, `intrface.aud` [data].
- `starttxt.txt` lines 1–2 (title) [traced, data].

## Edge cases

- **No game running** (start-up, or after Connor's death): Escape and
  Back to Game do nothing; the menu cannot be left except by New, Load
  or Quit [traced].
- **New while a game is running, Yes, then Escape in the difficulty
  popup**: back to the menu, the old game still there [traced].
- **Escape in a Yes/No popup while a game runs**: the game resumes; the
  question is simply dropped [traced].
- The **demo** version (`KQGame::Demo`) hides Load, Save and Replay
  Intro and skips the difficulty question. Not relevant to the full
  game.

## Do not reproduce

1. **Clicking "Yes" on the corrupt-saved-game question does nothing**
   in the original (only Enter deletes the file): the click handler has
   no case for that popup [traced]. Make a click on Yes do what Enter
   does.
2. **The disk-space check** before New: the original needs about 110 MB
   (115,343,360 bytes, minus what it has already unpacked) free on the
   game's drive, because it unpacks worlds onto the disk. We read the
   archives in memory and need no such check. Leave it out (and its
   message, `500.MSG` 0 0 156).
3. **Files in the game folder**: the original writes `current.bin`,
   makes `save/`, and on Quit clears the read-only mark of
   `starttxt.txt`. We never write to the game folder (AGENTS.md): keep
   the thumbnail in memory or in our own folder.
4. **CPU timing on first show**: the first time the menu is shown, the
   original times a busy loop to pick default detail settings. Not a
   menu behaviour; leave it out.
5. **Up/Down moving the list under an open popup** (rule 17) is
   harmless but odd; we may block it.

## How to check

In the original (GOG 1.03):

1. Start the game: the menu shows the two title lines, the mask beside
   **New**. Press Down repeatedly: the mask visits New, Load (only if
   there are saves), Options, Replay Intro, Quit and wraps back to New;
   Save and Back to Game are skipped.
2. Are Save and Back to Game **hidden** or **greyed** at start-up?
   (Rule 3; not settled.)
3. New → the "Combat difficulty" popup with Normal highlighted; Left
   moves the highlight to Easy, Right twice to Hard; each press clicks.
   Escape closes it. Pick Normal: Daventry starts; the menu music fades
   out over about a second.
4. In play, press Escape: the menu opens with the mask on **Back to
   Game**; the world sounds stop, the theme starts; a potion timer or a
   creature does not move while the menu is open. Escape again returns
   exactly where you were.
5. In play, Escape → New: "You already have a game running..." with No
   highlighted.
6. Escape → Quit: "Are you sure you want to exit the game?", No
   highlighted; hold Enter on Yes: the button stays pressed until Enter
   is let go, then the game closes.
7. Die (or let a creature kill Connor): the menu opens with the mask on
   New; Save and Back to Game are not available.

In our build: log each menu event (item chosen, popup opened, button
picked, combat level set, pause on/off, theme start/fade) to a
`menu.log` and compare with the above.

## Open questions

1. Whether hovering the mouse over an item moves the mask cursor (rule
   21): only the click path was found in the menu's own code. The
   user's check would settle it.
2. Whether an item that is off is hidden or drawn greyed (rule 3).
3. Whether the theme fades **in** when the menu opens, and what it does
   while the intro film plays (`sound.md` open question 5: the sound
   system is shut down during a video).
4. Whether the combat level chosen is kept in saved games (not traced;
   the Load screen is another study).
5. The death path: what the player sees between Connor's death and the
   menu (`fighting.md` open question 5).
