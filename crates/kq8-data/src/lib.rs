//! Readers for King's Quest: Mask of Eternity game files.
//!
//! Everything here reads from the player's own install. Nothing in this
//! crate writes to the game folder.

pub mod game_dir;
pub mod vol;

pub use game_dir::GameDir;
pub use vol::{Unpacked, VolArchive, VolEntry, VolError};
