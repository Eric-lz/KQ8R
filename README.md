# King's Quest: Mask of Eternity reimplementation in Rust

Unofficial fan project, not affiliated with or endorsed by the game's
owners. Contains no game files: you need your own copy of the game.
The code is written with an AI coding agent (Claude Code); see
[CREDITS.md](CREDITS.md).

- Game version: 1.03 (GOG)
- Game files in ./MASK/ folder (read only, never committed)
- Using NO-CD patch ([KQ8FixWin11](https://github.com/SegMash/KQ8FixWin11))

## What works

- Reading the game's packed world archives (`data/<World>/*.vol`):
  all 48 archives, 14,444 files, decompress and match the installed copies
  byte-for-byte. Tested on Windows 11 with the GOG 1.03 install.

## What doesn't work yet

Everything else. There is nothing to play yet: no graphics, sound, or game
logic.

## Try it

```sh
cargo run --release -- vol-verify
```

Use `--game <folder>` or the `KQ8_GAME_DIR` environment variable if the game
is not in `./MASK/`. Logs are written to `kq8-artifacts/logs/`.

## More

- [docs/DESIGN.md](docs/DESIGN.md): how the project works and the plan
- [docs/RESEARCH.md](docs/RESEARCH.md): existing tools and documentation
- [CREDITS.md](CREDITS.md): sources and licenses
- [MODLOG.md](MODLOG.md): change log
