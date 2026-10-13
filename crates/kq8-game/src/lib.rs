//! Game rules for the KQ8 rewrite, without any drawing code.
//!
//! The rules come from the behaviour notes in `docs/behaviour/`; each
//! module names the note it follows. The game logic runs at a fixed
//! [`UPDATES_PER_SECOND`] (docs/DESIGN.md, M2 "Physics rate").

pub mod active;
pub mod camera;
pub mod click;
pub mod collide;
pub mod connor;
pub mod door;
pub mod events;
pub mod mover;
pub mod player;
pub mod script;
pub mod water;

/// The original's update rate (`KQGame::FPSLimit 15`); every per-update
/// value in the rules assumes it.
pub const UPDATES_PER_SECOND: f64 = 15.0;
