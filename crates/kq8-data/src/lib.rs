//! Readers for King's Quest: Mask of Eternity game files.
//!
//! Everything here reads from the player's own install. Nothing in this
//! crate writes to the game folder.

pub mod action_map;
pub mod bitmap;
pub mod bounds;
pub mod building;
pub mod chunks;
pub mod game_dir;
pub mod kq_text;
pub mod material;
pub mod objects;
pub mod palette;
pub mod pers;
pub mod shape;
pub mod terrain;
pub mod vol;
pub mod world_script;

pub use bitmap::{Bitmap, BitmapFormat};
pub use game_dir::GameDir;
pub use material::{Material, MaterialKind, MaterialList};
pub use palette::{Palette, PaletteFormat};
pub use shape::Shape;
pub use terrain::{Terrain, TerrainSettings};
pub use vol::{Unpacked, VolArchive, VolEntry, VolError};
