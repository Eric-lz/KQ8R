//! `kq8`: command-line checks for the KQ8 rewrite.
//!
//! Usage:
//!   kq8 vol-verify [--game DIR]
//!   kq8 aud-verify [--wav <file>] [--game DIR]
//!   kq8 bitmap-verify [--game DIR]
//!   kq8 bitmap-png <file> [--palette <file>] [--game DIR]
//!   kq8 shape-verify [--game DIR]
//!   kq8 shape-parts <file> [--game DIR]   (a model's named parts per loop)
//!   kq8 terrain-verify [--game DIR]
//!   kq8 lightmap-verify [--game DIR]
//!   kq8 lighting-verify [--game DIR]
//!   kq8 building-verify [--game DIR]
//!   kq8 item-verify [--game DIR]
//!   kq8 msg-verify [--game DIR]
//!   kq8 font-verify [--game DIR]
//!   kq8 gui-verify [--game DIR]
//!   kq8 character-verify [--game DIR]
//!   kq8 conv-verify [--game DIR]
//!   kq8 map-verify [--game DIR]
//!   kq8 text <file> [--game DIR]   (prints a text file, e.g. a script)
//!
//! `<file>` is a path in the game folder (`GAME/8Gui/daventry.pbm`) or an
//! archive entry (`vol:Daventry/8bit/NAME.BMP`).
//! The game folder defaults to `$KQ8_GAME_DIR`, then `./MASK`.
//! Logs go to `kq8-artifacts/logs/`.

mod aud_verify;
mod bitmap_png;
mod bitmap_verify;
mod character_verify;
mod conv_verify;
mod font_verify;
mod gui_verify;
mod building_verify;
mod item_verify;
mod lighting_verify;
mod lightmap_verify;
mod log;
mod map_verify;
mod msg_verify;
mod shape_parts;
mod shape_verify;
mod source;
mod terrain_verify;
mod vol_verify;

use std::path::PathBuf;
use std::process::ExitCode;

use kq8_data::GameDir;

const USAGE: &str = "usage:
  kq8 vol-verify [--game DIR]
  kq8 aud-verify [--wav <file>] [--game DIR]
  kq8 bitmap-verify [--game DIR]
  kq8 bitmap-png <file> [--palette <file>] [--game DIR]
  kq8 shape-verify [--game DIR]
  kq8 shape-parts <file> [--game DIR]
  kq8 terrain-verify [--game DIR]
  kq8 lightmap-verify [--game DIR]
  kq8 lighting-verify [--game DIR]
  kq8 building-verify [--game DIR]
  kq8 item-verify [--game DIR]
  kq8 msg-verify [--game DIR]
  kq8 font-verify [--game DIR]
  kq8 gui-verify [--game DIR]
  kq8 character-verify [--game DIR]
  kq8 conv-verify [--game DIR]
  kq8 map-verify [--game DIR]
  kq8 text <file> [--game DIR]";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let mut game: Option<PathBuf> = None;
    let mut palette: Option<String> = None;
    let mut wav: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game" => game = args.next().map(PathBuf::from),
            "--palette" => palette = args.next(),
            "--wav" => wav = args.next(),
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
        (Some("aud-verify"), []) => aud_verify::run(&game, wav.as_deref()),
        (Some("bitmap-verify"), []) => bitmap_verify::run(&game),
        (Some("bitmap-png"), [image]) => bitmap_png::run(&game, image, palette.as_deref()),
        (Some("shape-verify"), []) => shape_verify::run(&game),
        (Some("shape-parts"), [file]) => shape_parts::run(&game, file),
        (Some("terrain-verify"), []) => terrain_verify::run(&game),
        (Some("lightmap-verify"), []) => lightmap_verify::run(&game),
        (Some("lighting-verify"), []) => lighting_verify::run(&game),
        (Some("building-verify"), []) => building_verify::run(&game),
        (Some("item-verify"), []) => item_verify::run(&game),
        (Some("msg-verify"), []) => msg_verify::run(&game),
        (Some("font-verify"), []) => font_verify::run(&game),
        (Some("gui-verify"), []) => gui_verify::run(&game),
        (Some("character-verify"), []) => character_verify::run(&game),
        (Some("conv-verify"), []) => conv_verify::run(&game),
        (Some("map-verify"), []) => map_verify::run(&game),
        (Some("text"), [file]) => match source::load(&game, file) {
            // Printed only (never written into the repo), with line numbers.
            Ok(bytes) => {
                let text: String = bytes.iter().map(|&b| b as char).collect();
                for (n, line) in text.lines().enumerate() {
                    println!("{:4}  {line}", n + 1);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::FAILURE
}
