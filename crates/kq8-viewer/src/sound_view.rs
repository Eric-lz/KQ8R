//! Sound effects in the viewer (M3 step 3h-2): takes the sounds the
//! world's objects and scripts start (`kq8-core` `active.rs`), finds and
//! decodes their files (`kq8-data` `aud.rs`, `sound.md` rule 13), runs
//! the rules each frame (`kq8-core` `sound.rs`) with Connor's position and
//! the drawn camera's axes, and hands the orders to the mixer and sound
//! card (`kq8-audio`). Everything goes to `kq8-artifacts/logs/sound.log`.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;
use kq8_audio::{Command, Output};
use kq8_core::sound::{Audible, Ears, Found, Groups, Order, SoundSystem, View, GROUPS};
use kq8_data::aud::{self, Sound};
use kq8_data::GameDir;

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::log::Log;
use crate::object_light::ObjectLight;
use crate::UNITS_PER_METRE;

/// A file found and decoded, or why not, by lower-case sound name.
type Cached = Result<(Arc<Sound>, String), String>;

#[derive(Resource)]
pub struct SoundSim {
    system: SoundSystem,
    /// The world's sound objects (M3 step 3h-3).
    audibles: Vec<Audible>,
    output: Option<Output>,
    cache: HashMap<String, Cached>,
    game: GameDir,
    world: String,
    log: Log,
    frames: u64,
}

impl SoundSim {
    /// Reads the sliders from `MASK/Options.cs` and, if `on`, opens the
    /// sound card. Without a card the rules and the log still run.
    pub fn new(game: &GameDir, world: &str, on: bool, audibles: Vec<Audible>) -> Option<SoundSim> {
        let mut log = match Log::create("sound") {
            Ok(l) => l,
            Err(e) => {
                eprintln!("sound: cannot create sound.log: {e}");
                return None;
            }
        };
        log.detail(format!("world {world}; rules from docs/behaviour/sound.md (M3 step 3h-2)"));
        let mut groups = Groups::default();
        match game.find("Options.cs").map(std::fs::read) {
            Some(Ok(bytes)) => {
                for line in groups.apply_script(&bytes) {
                    log.detail(format!("Options.cs: {line}"));
                }
            }
            _ => log.detail("Options.cs not found: every group at 1 (rule 52)"),
        }
        let values: Vec<String> = (0..GROUPS).map(|g| format!("{g}: {:.2}", groups.loudness(g))).collect();
        log.summary(format!("sound: group loudness {}", values.join(", ")));
        let output = if on {
            match Output::open() {
                Ok(o) => {
                    log.summary(format!("sound: output {}", o.description));
                    Some(o)
                }
                Err(e) => {
                    log.summary(format!("sound: no output ({e}); the rules and this log still run"));
                    None
                }
            }
        } else {
            log.summary("sound: output off (--sound off); the rules and this log still run");
            None
        };
        log.summary(format!("sound: {} sound objects (KQAudibleObject)", audibles.len()));
        Some(SoundSim { system: SoundSystem::new(groups), audibles, output, cache: HashMap::new(), game: game.clone(), world: world.to_string(), log, frames: 0 })
    }
}

/// The decoded file for a sound name, looked up once.
fn load(cache: &mut HashMap<String, Cached>, game: &GameDir, world: &str, name: &str) -> Cached {
    let key = name.to_ascii_lowercase();
    if let Some(c) = cache.get(&key) {
        return c.clone();
    }
    let got = match aud::find(game, world, name) {
        None => Err(format!("'{name}' not found (tried {})", aud::name_variants(name).join(", "))),
        Some(path) => {
            let rel = path.strip_prefix(game.root()).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| aud::parse(&b)) {
                Ok(s) => Ok((Arc::new(s), rel)),
                Err(e) => Err(format!("{rel}: {e}")),
            }
        }
    };
    cache.insert(key, got.clone());
    got
}

fn found(c: &Cached) -> Result<Found, String> {
    c.as_ref().map(|(snd, path)| Found { path: path.clone(), seconds: snd.seconds() }).map_err(Clone::clone)
}

/// Game-space point from a Bevy one (the inverse of `to_bevy` with centre
/// 0: (x, y, z) -> (x, -z, y), metres to units).
fn to_game(v: Vec3) -> [f32; 3] {
    [v.x * UNITS_PER_METRE, -v.z * UNITS_PER_METRE, v.y * UNITS_PER_METRE]
}

fn direction_to_game(v: Vec3) -> [f32; 3] {
    [v.x, -v.z, v.y]
}

/// Each frame, after the camera is placed: start what the world asked for,
/// redo the placed sounds, send the orders.
pub fn sound_step(
    sim: Option<ResMut<SoundSim>>,
    doors: Option<ResMut<DoorSim>>,
    connor: Option<Res<ConnorSim>>,
    camera: Query<&Transform, With<Camera3d>>,
    light: Option<Res<ObjectLight>>,
    time: Res<Time<Real>>,
) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    sim.frames += 1;
    let Ok(cam) = camera.single() else { return };
    let view = View {
        position: to_game(cam.translation),
        right: direction_to_game(*cam.right()),
        forward: direction_to_game(*cam.forward()),
    };
    // Rule 28: distance from Connor; no Connor in this world: the camera.
    let listener = connor.map_or(view.position, |c| c.mover.position);
    let ears = Ears { listener, view };
    let mut doors = doors;
    let starts = doors.as_mut().map(|d| std::mem::take(&mut d.active.world.sounds)).unwrap_or_default();
    let mut decoded: HashMap<String, Arc<Sound>> = HashMap::new();
    let (game, world) = (&sim.game, sim.world.as_str());
    for s in &starts {
        let file = load(&mut sim.cache, game, world, &s.name);
        if let Ok((snd, _)) = &file {
            decoded.insert(s.name.clone(), snd.clone());
        }
        sim.system.start(s, found(&file), &ears);
    }
    // Sound objects (M3 3h-3): on while near or far (rules 46-47), stopped
    // by deleteObject (rule 48).
    let visible = light.map_or(kq8_data::haze::NO_TERRAIN_VISIBLE_DISTANCE, |l| l.haze.visible);
    for a in &mut sim.audibles {
        let deleted = doors.as_ref().is_some_and(|d| d.active.world.is_deleted(&a.name));
        let (cache, decoded) = (&mut sim.cache, &mut decoded);
        a.update(&mut sim.system, &ears, visible, deleted, &mut |name| {
            let file = load(cache, game, world, name);
            if let Ok((snd, _)) = &file {
                decoded.insert(name.to_string(), snd.clone());
            }
            found(&file)
        });
    }
    sim.system.update(time.delta_secs_f64(), &ears);
    let orders = std::mem::take(&mut sim.system.orders);
    let lines = std::mem::take(&mut sim.system.log);
    let frame = sim.frames;
    for line in lines {
        sim.log.detail(format!("frame {frame} {line}"));
    }
    if let Some(out) = &sim.output {
        let commands = orders.into_iter().filter_map(|o| match o {
            Order::Start { id, name, looping, gains } => decoded.get(&name).map(|sound| Command::Start { id, sound: sound.clone(), looping, gains }),
            Order::Gains { id, gains } => Some(Command::Gains { id, gains }),
            Order::Pause { id } => Some(Command::Pause { id }),
            Order::Resume { id } => Some(Command::Resume { id }),
            Order::Stop { id } => Some(Command::Stop { id }),
        });
        out.send(commands);
    }
}
