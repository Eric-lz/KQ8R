//! Game rules for the KQ8 rewrite, without any drawing code.
//!
//! The rules come from the behaviour notes in `docs/behaviour/`; each
//! module names the note it follows. The game logic runs at a fixed
//! [`UPDATES_PER_SECOND`] (docs/DESIGN.md, M2 "Physics rate").

pub mod connor;
pub mod mover;

/// The original's update rate (`KQGame::FPSLimit 15`); every per-update
/// value in the rules assumes it.
pub const UPDATES_PER_SECOND: f64 = 15.0;
