//! Readers for King's Quest: Mask of Eternity game files.
//!
//! Everything here reads from the player's own install. Nothing in this
//! crate writes to the game folder.

pub mod action_map;
pub mod aud;
pub mod bitmap;
pub mod bounds;
pub mod building;
pub mod chunks;
pub mod dil;
pub mod effects;
pub mod envinfo;
pub mod font;
pub mod game_dir;
pub mod haze;
pub mod items;
pub mod kq_text;
pub mod lightmap;
pub mod lights;
pub mod material;
pub mod msg;
pub mod objects;
pub mod palette;
pub mod pers;
pub mod shape;
pub mod terrain;
pub mod vol;
pub mod wall_light;
pub mod world_script;

pub use bitmap::{Bitmap, BitmapFormat};
pub use game_dir::GameDir;
pub use material::{Lighting, Material, MaterialKind, MaterialList};
pub use palette::{Palette, PaletteFormat};
pub use shape::Shape;
pub use terrain::{Terrain, TerrainSettings};
pub use vol::{Unpacked, VolArchive, VolEntry, VolError};
