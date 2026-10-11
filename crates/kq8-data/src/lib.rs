//! Readers for King's Quest: Mask of Eternity game files.
//!
//! Everything here reads from the player's own install. Nothing in this
//! crate writes to the game folder.

pub mod bitmap;
pub mod chunks;
pub mod game_dir;
pub mod palette;
pub mod vol;

pub use bitmap::{Bitmap, BitmapFormat};
pub use game_dir::GameDir;
pub use palette::{Palette, PaletteFormat};
pub use vol::{Unpacked, VolArchive, VolEntry, VolError};
