//! `kq8`: command-line checks for the KQ8 rewrite.
//!
//! Usage:
//!   kq8 vol-verify [--game DIR]
//!
//! The game folder defaults to `$KQ8_GAME_DIR`, then `./MASK`.
//! Logs go to `kq8-artifacts/logs/`.

mod log;
mod vol_verify;

use std::path::PathBuf;
use std::process::ExitCode;

use kq8_data::GameDir;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let mut game: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game" => game = args.next().map(PathBuf::from),
            other => {
                eprintln!("unknown argument: {other}");
                return usage();
            }
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

    match command.as_deref() {
        Some("vol-verify") => vol_verify::run(&game),
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: kq8 vol-verify [--game DIR]");
    ExitCode::FAILURE
}
