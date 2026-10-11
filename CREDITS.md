# Credits and lineage

This is an unofficial fan project. It contains no game assets and no
decompiled code; it reads data from the player's own copy of
*King's Quest: Mask of Eternity*. *King's Quest* and *Mask of Eternity* are
trademarks of their respective owners; this project is not affiliated with
or endorsed by them.

## Built on

| Project | Author(s) | Licence | Version we used | What we use |
| --- | --- | --- | --- | --- |
| [kqmoevol](https://svn.nicode.net/kqmoevol/) | Nico Bendlin | MIT | SVN trunk, read 2026-10-11 | Knowledge of the VOL archive layout. No code copied |
| [libkq8fpc format notes](https://svn.nicode.net/libkq8fpc/doc/) | Nico Bendlin | GPL-3.0 (whole repo) | SVN r34, read 2026-10-11 | Facts about file layouts only. No code or text copied |
| [IW4L](https://github.com/vladtrc/iw4L) | vladtrc | Apache-2.0 | `main`, read 2026-10-11 | Repository layout ideas, provenance practice. No code copied |
| [ai-game-modding-guides](https://github.com/trevaintdead/ai-game-modding-guides) | Trev and contributors | MIT | commit `0c14be4` | `MODLOG-template.md` and `STATUS-handoff.md` copied (licence below); working method in `docs/DESIGN.md` |
| [`explode`](https://github.com/agrif/explode) crate | Aaron Griffith | MIT | 0.1.2 | PKWARE DCL decompression |
| `arraydeque` crate (used by `explode`) | | MIT / Apache-2.0 | 0.4.5 | |
| [`png`](https://github.com/image-rs/image-png) crate | image-rs contributors | MIT OR Apache-2.0 | 0.18.1 | Writing PNG files for checking |
| [Bevy](https://bevy.org) | Bevy contributors | MIT OR Apache-2.0 | 0.20.0 | Window, 3D drawing, input (`kq8-viewer` only). Its many dependencies are listed in `Cargo.lock`; check their licences before any binary release |
| Used by `png`: `adler2`, `bitflags`, `cfg-if`, `crc32fast`, `fdeflate`, `flate2`, `miniz_oxide`, `simd-adler32`, `zlib-rs` | | MIT, Apache-2.0, Zlib or 0BSD (each permissive) | see `Cargo.lock` | |

Per-fact sources are listed in [docs/provenance/](docs/provenance/README.md).

## What's new in this project

Everything under `crates/` and `docs/` is written for this project.

## Tools and AI

- Agent and model: Claude Code (Claude Opus), which wrote the code and docs.
- Human work: Eric Peracchi Pisoni sets direction, rules and priorities,
  and tests.

## Game content

No game files are included. Players use their own copy of
*King's Quest: Mask of Eternity* (tested with GOG.com version 1.03).

## Licence texts

Crate licence texts ship with each crate's source. If this project is ever
distributed as a binary, include them alongside it.

### ai-game-modding-guides (MIT)

```text
MIT License

Copyright (c) 2026 Trev and contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
