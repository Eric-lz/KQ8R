//! `kq8`: command-line checks for the KQ8 rewrite.
//!
//! Usage:
//!   kq8 vol-verify [--game DIR]
//!   kq8 bitmap-verify [--game DIR]
//!   kq8 bitmap-png <file> [--palette <file>] [--game DIR]
//!   kq8 shape-verify [--game DIR]
//!
//! `<file>` is a path in the game folder (`GAME/8Gui/daventry.pbm`) or an
//! archive entry (`vol:Daventry/8bit/NAME.BMP`).
//! The game folder defaults to `$KQ8_GAME_DIR`, then `./MASK`.
//! Logs go to `kq8-artifacts/logs/`.

mod bitmap_png;
mod bitmap_verify;
mod log;
mod shape_verify;
mod source;
mod vol_verify;

use std::path::PathBuf;
use std::process::ExitCode;

use kq8_data::GameDir;

const USAGE: &str = "usage:
  kq8 vol-verify [--game DIR]
  kq8 bitmap-verify [--game DIR]
  kq8 bitmap-png <file> [--palette <file>] [--game DIR]
  kq8 shape-verify [--game DIR]";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let mut game: Option<PathBuf> = None;
    let mut palette: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game" => game = args.next().map(PathBuf::from),
            "--palette" => palette = args.next(),
            a if a.starts_with("--") => {
                eprintln!("unknown option: {a}");
                return usage();
            }
            _ => positional.push(arg),
        }
    }

    let root = game
        .or_else(|| std::env::var_os("KQ8_GAME_DIR").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("MASK"));
    let game = match GameDir::open(&root) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("pass --game <folder> or set KQ8_GAME_DIR");
            return ExitCode::FAILURE;
        }
    };

    match (command.as_deref(), positional.as_slice()) {
        (Some("vol-verify"), []) => vol_verify::run(&game),
        (Some("bitmap-verify"), []) => bitmap_verify::run(&game),
        (Some("bitmap-png"), [image]) => bitmap_png::run(&game, image, palette.as_deref()),
        (Some("shape-verify"), []) => shape_verify::run(&game),
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::FAILURE
}
