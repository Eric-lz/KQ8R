//! Game rules for the KQ8 rewrite, without any drawing code.
//!
//! The rules come from the behaviour notes in `docs/behaviour/`; each
//! module names the note it follows. The game logic runs at a fixed
//! [`UPDATES_PER_SECOND`] (docs/DESIGN.md, M2 "Physics rate").

pub mod active;
pub mod anim;
pub mod camera;
pub mod camera_flight;
pub mod caption;
pub mod character;
pub mod character_ai;
pub mod chest;
pub mod click;
pub mod collide;
pub mod connor;
pub mod connor_auto;
pub mod connor_inventory;
pub mod connor_mover;
pub mod connor_start;
pub mod connor_update;
pub mod conversation;
pub mod door;
pub mod emitter;
pub mod events;
pub mod fight;
pub mod ground_anim;
pub mod gui_draw;
pub mod inventory;
pub mod item;
pub mod journey;
pub mod magic_map;
pub mod meters;
pub mod music;
pub mod named_anim;
pub mod panels;
pub mod portal;
pub mod potions;
pub mod rope;
pub mod script;
pub mod shadow;
pub mod sound;
pub mod stats;
pub mod talk;
pub mod trap;
pub mod turn_keys;
pub mod water;
pub mod weapons;

/// The original's update rate (`KQGame::FPSLimit 15`); every per-update
/// value in the rules assumes it.
pub const UPDATES_PER_SECOND: f64 = 15.0;
