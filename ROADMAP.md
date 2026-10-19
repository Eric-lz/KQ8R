# Roadmap

A rough picture of how far the rewrite of *King's Quest: Mask of Eternity*
has come. The detailed plans are in [docs/DESIGN.md](docs/DESIGN.md), and
the full list of what works is in [docs/FEATURES.md](docs/FEATURES.md).

So far everything is built and tested in the first world, **Daventry**.

| Milestone | What it means | Status |
| --- | --- | --- |
| M0 | Read the game's files | ✅ Done |
| M1 | See Daventry | ✅ Done |
| M2 | Walk Connor around Daventry | ✅ Done |
| M3 | Daventry comes alive | 🟡 Mostly done |
| M4 | Talk, listen, use the menus | 🟡 Under way |
| M5 | Fight | ⬜ Not started |
| M6 | The whole game | ⬜ Not started |
| M7 | Match the original, then improve | ⬜ Not started |

## Done

- **M0 – Reading the game's files.** The game's archives, pictures and
  colour palettes are read straight from your own copy of the game.
- **M1 – Seeing Daventry.** The ground, buildings, trees and objects are
  drawn where the game puts them, with the original's lighting, shadows
  and distance fog. All 12 worlds load.
- **M2 – Walking around.** Connor walks, runs, steps back, turns, jumps
  and wades through water. He falls, slides off steep slopes and bumps
  into walls the way he does in the original. The camera follows him and
  keeps out of walls.
- **M3 – Daventry comes alive (mostly).** Doors, chests, items to pick
  up, traps, animated objects (windmill, mill wheel, fountain), falling
  leaves, flowing water, sound effects and music. The game's own puzzle
  scripts run, scripted cutscenes play, and the first puzzle (pushing
  the bricks) works.
- **M4 – So far.** Connor's remarks and other spoken lines play with
  their captions and lip sync. The on-screen interface works: the bottom
  bar (health, experience, weapons, potions), the top bar (quest items,
  gold) and holding an item on the mouse pointer to use it.

## Next

- **Conversations** with the wizard and the other people in Daventry.
- **The main menu** (Escape or the mask button), with the game paused
  while it is open.
- **What's left of M3:** a few cutscene and camera details, damage from
  rolling rocks, and pushable boulders.

## Later

- **M5 – Fighting.** Monsters that move, attack and die; Connor's weapons,
  armour, health and experience in battle.
- **M6 – The whole game.** Saving and loading, travelling between worlds,
  all 12 worlds playable, the cutscene videos, the options screen.
- **M7 – Match the original, then improve.** Play the original side by
  side with ours and fix every difference. After that, the improvements
  this project was started for: better controls, better graphics and
  gameplay fixes. (A first one is already in: the bottom bar's corner
  panels can sit at the window's edges on wide screens.)
