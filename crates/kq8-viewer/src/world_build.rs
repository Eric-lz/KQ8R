//! One way to build a world (M6 6e-1, `docs/DESIGN.md` "6e"): reading a
//! world's files, putting its resources in, and running the start-up
//! systems that spawn it, as one path that the launch uses and a world
//! change (6e-2 on) will use again.
//!
//! - [`read_world`] reads everything a world needs from the game's files
//!   (no Bevy; it can run on another thread later, 6e-4).
//! - [`insert_world`] puts the world's resources into the app.
//! - [`BuildWorld`] is the schedule of start-up systems that spawn the
//!   world's entities; [`build_pending`] runs it.
//! - Every entity spawned with a `Transform` or a `Node` gets
//!   [`WorldEntity`], unless it is [`Persistent`], so taking a world down
//!   (6e-2) removes exactly the world's entities.
//! - [`world_change_step`] (6e-2) takes the world down when a change is
//!   asked for ([`WorldChange::asked`]), waits two frames so Bevy frees
//!   the assets only the old world used, then reads and builds the next
//!   one. While it runs, the game's systems don't ([`playing`]).
//!
//! Ours: the original builds a world from its scripts in the same program
//! too (`world-change.md` rules 7–8), but how our viewer is split into
//! resources and systems is our own.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use kq8_data::GameDir;

use crate::log::Log;
use kq8_core::journey::{connor_summary, Journey};
use crate::{connor_view, gui_view, light_view, load, map_view, object_light, sound_view, talk_view, terrain_view, Shared};

/// The start-up systems that spawn a world (were `Startup` until 6e-1).
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuildWorld;

/// An entity that belongs to the world shown now.
#[derive(Component)]
pub struct WorldEntity;

/// An entity that stays when the world changes (none yet; the loading
/// screen's, 6e-4).
#[derive(Component)]
pub struct Persistent;

/// What the launch options say about building any world.
#[derive(Clone)]
pub struct Options {
    pub game: PathBuf,
    pub light: light_view::LightMode,
    pub haze: bool,
    pub corners: bool,
    pub sound: bool,
    pub kept_maps: Option<String>,
}

/// Everything read from a world's files.
pub struct WorldRead {
    pub name: String,
    lights: kq8_data::lights::Lights,
    haze: kq8_data::haze::Haze,
    ground: load::Ground,
    objects: load::Objects,
    animation: load::GroundAnimAssets,
    caption: Option<talk_view::CaptionStyle>,
    gui: Option<gui_view::Gui>,
    map: Option<map_view::MapSim>,
    /// The sound objects and the music, for the sound system made on the
    /// main thread (the sound card's stream stays on one thread).
    sound: (Vec<kq8_core::sound::Audible>, Option<kq8_core::music::Music>),
    /// How long the reading took.
    pub took_ms: u128,
}

/// The world waiting to be put in by [`build_pending`].
#[derive(Resource, Default)]
pub struct PendingWorld(pub Option<(WorldRead, Options)>);

/// A world change asked for: `reload` (6e-2), later `goworld` and the
/// portals.
#[derive(Debug, Clone)]
pub struct Request {
    pub world: String,
    /// Connor's place on arrival: x, y on the ground, heading (as
    /// `--at`); `None`: where the world's own start puts him.
    pub at: Option<[f32; 3]>,
    /// For the logs.
    pub why: String,
}

/// Where a world change is (6e-2).
#[derive(Default)]
enum Step {
    #[default]
    Playing,
    /// Taken down; `frames` more frames to wait before building. The
    /// journey taken from the old world (6e-3), if it had one.
    Down { request: Request, frames: u32, from: String, journey: Option<Box<Journey>> },
    /// The next world is being read on another thread (6e-4a) while the
    /// loading screen is drawn.
    Loading { request: Request, from: String, journey: Option<Box<Journey>>, read: JoinHandle<Result<WorldRead, String>>, meter: Meter },
}

/// The loading meter while a world is read (6e-4a, `world-change.md`
/// rule 24): the percent the scripts' `setLoadProgress` reached, and what
/// was drawn, for the log.
struct Meter {
    percent: Arc<AtomicU32>,
    started: Instant,
    last_frame: Instant,
    frames: u64,
    longest_frame_ms: f32,
    /// (ms since the start, percent) at each change.
    steps: Vec<(u128, u32)>,
}

/// The world shown now and the change under way, if any.
#[derive(Resource, Default)]
pub struct WorldChange {
    /// Set by whoever asks for a change; taken at the end of the frame.
    pub asked: Option<Request>,
    step: Step,
    /// The world built last (its name as asked for).
    pub current: String,
    /// How many worlds were built in this run: systems that keep values
    /// between frames reset them when it changes ([`new_world`]).
    pub generation: u64,
    /// The launch options, for building the next world.
    options: Option<Options>,
}

impl WorldChange {
    /// The game folder, once the launch options are kept.
    pub fn game(&self) -> Option<&PathBuf> {
        self.options.as_ref().map(|o| &o.game)
    }
}

/// The run condition of the game's systems: a world is in (6e-2).
pub fn playing(change: Res<WorldChange>) -> bool {
    matches!(change.step, Step::Playing)
}

/// For systems that keep values from frame to frame (`Local`): true the
/// first time it is asked after each new world, so they start again
/// (6e-2). `seen` is the system's own `Local<u64>`.
pub fn new_world(change: &WorldChange, seen: &mut u64) -> bool {
    let fresh = *seen != change.generation;
    *seen = change.generation;
    fresh
}

/// Frames to wait between taking a world down and building the next, so
/// the assets only the old world used are freed first (Bevy frees an
/// asset in a later frame, once nothing holds it). A world takes about
/// 3.4 GB (6e-1), so the two must not be in memory together.
const FREE_FRAMES: u32 = 2;

/// Reads world `name` (as `main` did before 6e-1).
/// `arrival`: a world change (6e-3), see `load::load_objects`.
/// `progress`: the loading meter's percent (6e-4a).
pub fn read_world(game: &GameDir, name: &str, opts: &Options, arrival: Option<load::Arrival>, progress: Option<Arc<AtomicU32>>, log: &mut Log) -> Result<WorldRead, String> {
    let started = Instant::now();
    let world = load::World::open(game, name)?;
    log.summary(format!("world: {}", world.name));
    // M1 step 5b: the lights for models; step 6: the haze.
    let mut parts = Vec::new();
    let mut part = |what: &str, since: &mut Instant| {
        parts.push(format!("{what} {} ms", since.elapsed().as_millis()));
        *since = Instant::now();
    };
    let mut since = Instant::now();
    let lights = load::load_object_lights(game, &world, log);
    let mut ground = load::load_ground(game, &world, log)?;
    let haze = load::load_haze(game, &world, Some(&ground.terrain.settings), log);
    part("ground", &mut since);
    let mut objects = load::load_objects(game, &world, &ground.terrain, log, arrival, progress);
    part("objects", &mut since);
    let animators = std::mem::take(&mut objects.animators);
    let animation = load::prepare_ground_animation(game, &world, &mut ground, animators, log);
    part("ground animation", &mut since);
    // Remarks and speech (M4 4a-2).
    objects.talk = Some(talk_view::load_talk(game, name, log));
    // The caption as the original's (M4 4b-3).
    let caption = talk_view::load_caption(game, name, log);
    // The panels (M4 4c-2).
    let gui = gui_view::load_gui(game, name, log).map(|mut gui| {
        gui.corners = opts.corners;
        gui
    });
    // The magic map (M6 6d-2).
    let map = Some(map_view::load_map(game, name, objects.map.take(), opts.kept_maps.as_deref(), log));
    let audibles = std::mem::take(&mut objects.audibles);
    let music = objects.music.take();
    let sound = (audibles, music);
    part("speech, panels, map", &mut since);
    log.summary(format!("world read: {name}: {}", parts.join(", ")));
    Ok(WorldRead { name: name.to_string(), lights, haze, ground, objects, animation, caption, gui, map, sound, took_ms: started.elapsed().as_millis() })
}

/// The model lights and haze for `ObjectLight`, which the app needs
/// before its plugins are added (`object_light::add`).
pub fn object_light(read: &WorldRead, opts: &Options) -> object_light::ObjectLight {
    object_light::ObjectLight::new(read.lights.clone(), opts.light, read.haze, opts.haze)
}

/// Remembers the launch options for later worlds (main, at launch).
pub fn remember_options(world: &mut World, opts: &Options) {
    world.resource_mut::<WorldChange>().options = Some(opts.clone());
}

/// Puts a read world's resources in. `ObjectLight` is put in by the
/// caller (see [`object_light`]).
pub fn insert_world(world: &mut World, read: WorldRead, opts: &Options) {
    let WorldRead { name, ground, mut objects, animation, caption, gui, map, sound: (audibles, music), .. } = read;
    if let Some(style) = caption {
        world.insert_resource(style);
    }
    if let Some(gui) = gui {
        world.insert_resource(gui);
    }
    if let Some(map) = map {
        world.insert_resource(map);
    }
    world.insert_resource(connor_view::FootSounds(std::mem::take(&mut objects.footsounds)));
    // The sound system starts again with each world (world-change.md
    // rule 7.4: every playing sound stops).
    if let Some(sound) = GameDir::open(&opts.game).ok().and_then(|g| sound_view::SoundSim::new(&g, &name, opts.sound, audibles, music)) {
        world.insert_resource(sound);
    }
    world.insert_resource(light_view::GroundLight::new(ground.light.clone(), ground.other_light.clone(), opts.light, ground.dynamic));
    world.insert_resource(terrain_view::LoadedGround(Some(ground)));
    world.insert_resource(crate::ground_anim_view::LoadedGroundAnim(Some(animation)));
    world.insert_resource(terrain_view::FloorView(0));
    world.insert_resource(terrain_view::LoadedObjects(Some(objects)));
    world.insert_resource(terrain_view::Fly::new());
    world.insert_resource(crate::door_view::LoadQueue::default());
    world.init_resource::<crate::door_view::Deleted>();
    world.init_resource::<crate::door_view::CharacterColliders>();
    world.init_resource::<talk_view::Speech>();
    world.init_resource::<gui_view::GuiCover>();
    world.init_resource::<gui_view::MapDrag>();
    world.init_resource::<gui_view::ScreenFrame>();
    world.init_resource::<gui_view::BarActions>();
    world.init_resource::<gui_view::CaptionPlace>();
    world.init_resource::<object_light::HazeCheck>();
    world.insert_resource(crate::game_camera::CameraChoice { game: false });
    // Empty at the start; `draw_distance::add` and `object_light::add` put
    // them in at launch, a world change takes them out (6e-2).
    world.init_resource::<crate::draw_distance::ActivePoint>();
    world.init_resource::<crate::draw_distance::ActivityLog>();
    world.init_resource::<crate::draw_distance::ObjectActivity>();
    world.init_resource::<crate::draw_distance::CharactersInRange>();
    world.init_resource::<crate::draw_distance::CharactersSolid>();
    world.init_resource::<crate::draw_distance::ActiveRules>();
    world.init_resource::<crate::draw_distance::ColliderSwitches>();
    world.init_resource::<crate::draw_distance::SwitchedOut>();
    world.init_resource::<crate::scene_lights::LightsGroup>();
    world.init_resource::<crate::scene_lights::GroundLights>();
}

/// Startup (and later on request): puts the pending world in and spawns
/// it, then logs what it holds.
pub fn build_pending(world: &mut World) {
    let Some((read, opts)) = world.resource_mut::<PendingWorld>().0.take() else { return };
    let started = Instant::now();
    let (name, read_ms) = (read.name.clone(), read.took_ms);
    insert_world(world, read, &opts);
    world.run_schedule(BuildWorld);
    {
        let mut change = world.resource_mut::<WorldChange>();
        change.current = name.clone();
        change.generation += 1;
    }
    let counts = Counts::of(world);
    let log = world.resource::<Shared>().log.clone();
    let mut log = log.lock().unwrap();
    log.summary(format!("world built: {name}: read {read_ms} ms, spawned in {} ms; {}; {}", started.elapsed().as_millis(), counts.line(), crate::memory::line()));
}

/// The end of every frame (6e-2): a change asked for takes the world
/// down; [`FREE_FRAMES`] frames later the next world is read and built.
pub fn world_change_step(world: &mut World) {
    let step = std::mem::take(&mut world.resource_mut::<WorldChange>().step);
    let log = world.resource::<Shared>().log.clone();
    match step {
        Step::Playing => {
            let Some(request) = world.resource_mut::<WorldChange>().asked.take() else { return };
            let from = world.resource::<WorldChange>().current.clone();
            let before = Counts::of(world);
            // The journey (6e-3, world-change.md rules 17-19), before the
            // world's resources go.
            let journey = world.get_resource::<crate::door_view::DoorSim>().map(|sim| {
                let j = Journey::take(&sim.active.world, &sim.active.scripts.vars, &from);
                (Box::new(j), connor_summary(&sim.active.world))
            });
            let (_, removed) = take_down(world);
            // Every log gets the line, after the old world's logs were
            // closed, so it comes before the new world's lines.
            let marker = format!("== world change: {from} -> {} ({}) ==", request.world, request.why);
            Log::mark_all(&marker, &["viewer", "console"]);
            let mut log = log.lock().unwrap();
            log.summary(&marker);
            log.summary(format!("world change: before taking {from} down: {}; {}", before.line(), crate::memory::line()));
            log.summary(format!("world change: {from} taken down: {} world entities despawned, {removed} per-world resources removed", before.world_entities));
            drop(log);
            let mut wlog = Log::create("world").ok();
            if let Some(wl) = wlog.as_mut() {
                wl.detail(&marker);
                wl.detail(format!("before taking {from} down: {}; {}", before.line(), crate::memory::line()));
                match &journey {
                    Some((j, connor)) => {
                        for line in j.describe() {
                            wl.detail(line);
                        }
                        wl.detail(format!("Connor leaving {from}: {connor}"));
                    }
                    None => wl.detail("no rules running in the old world: no journey"),
                }
            }
            let journey = journey.map(|(j, _)| j);
            let shared_log = world.resource::<Shared>().log.clone();
            crate::loading_screen::open(world, &request.world, &shared_log);
            world.resource_mut::<WorldChange>().step = Step::Down { request, frames: FREE_FRAMES, from, journey };
        }
        Step::Down { request, frames, from, journey } if frames > 0 => {
            world.resource_mut::<WorldChange>().step = Step::Down { request, frames: frames - 1, from, journey };
        }
        Step::Down { request, from, journey, .. } => {
            let after = Counts::of(world);
            log.lock().unwrap().summary(format!("world change: {FREE_FRAMES} frames after taking {from} down: {}; {}", after.line(), crate::memory::line()));
            list_unmarked(world, &mut log.lock().unwrap());
            let Some(opts) = world.resource::<WorldChange>().options.clone() else {
                log.lock().unwrap().summary("world change: no launch options kept; nothing built");
                return;
            };
            // Read on another thread (6e-4a); the window keeps drawing the
            // loading screen. The thread holds `viewer.log` meanwhile.
            let arrival = journey.as_ref().map(|j| load::Arrival { no_name_from: j.next_no_name });
            let percent = Arc::new(AtomicU32::new(0));
            let (name, shared_log, thread_percent) = (request.world.clone(), log.clone(), percent.clone());
            let read = std::thread::Builder::new().name("world read".into()).spawn(move || {
                let mut log = shared_log.lock().unwrap();
                GameDir::open(&opts.game).map_err(|e| e.to_string()).and_then(|game| read_world(&game, &name, &opts, arrival, Some(thread_percent), &mut log))
            });
            let read = match read {
                Ok(handle) => handle,
                Err(e) => {
                    log.lock().unwrap().summary(format!("world change: the reading thread could not start ({e}); closing"));
                    world.write_message(AppExit::error());
                    return;
                }
            };
            let now = Instant::now();
            let meter = Meter { percent, started: now, last_frame: now, frames: 0, longest_frame_ms: 0.0, steps: vec![(0, 0)] };
            world.resource_mut::<WorldChange>().step = Step::Loading { request, from, journey, read, meter };
        }
        Step::Loading { request, from, journey, read, mut meter } => {
            // One frame of the loading screen.
            let now = Instant::now();
            meter.frames += 1;
            meter.longest_frame_ms = meter.longest_frame_ms.max((now - meter.last_frame).as_secs_f32() * 1000.0);
            meter.last_frame = now;
            let p = meter.percent.load(Ordering::Relaxed);
            if meter.steps.last().is_some_and(|(_, q)| *q != p) {
                meter.steps.push((meter.started.elapsed().as_millis(), p));
            }
            crate::loading_screen::set_meter(world, p);
            if !read.is_finished() {
                world.resource_mut::<WorldChange>().step = Step::Loading { request, from, journey, read, meter };
                return;
            }
            let read = read.join().unwrap_or_else(|_| Err("the reading thread stopped with a panic".into()));
            {
                let steps: Vec<String> = meter.steps.iter().map(|(ms, p)| format!("{p} % at {ms} ms")).collect();
                let mut log = log.lock().unwrap();
                log.summary(format!(
                    "world change: loading screen: {} frames in {} ms, longest frame {:.1} ms; meter: {}",
                    meter.frames,
                    meter.started.elapsed().as_millis(),
                    meter.longest_frame_ms,
                    steps.join(", ")
                ));
            }
            let sizes = crate::loading_screen::close(world);
            log.lock().unwrap().summary(format!("world change: loading screen closed: {sizes}"));
            let Some(opts) = world.resource::<WorldChange>().options.clone() else { return };
            let read = match read {
                Ok(r) => r,
                Err(e) if request.world != from => {
                    // Back to the world we came from (still no world in,
                    // so the game's systems keep waiting).
                    log.lock().unwrap().summary(format!("world change: {} could not be read ({e}); back to {from}", request.world));
                    let back = Request { world: from.clone(), at: request.at, why: "the asked world could not be read".into() };
                    world.resource_mut::<WorldChange>().step = Step::Down { request: back, frames: 0, from, journey };
                    crate::loading_screen::open(world, &request.world, &log);
                    return;
                }
                Err(e) => {
                    log.lock().unwrap().summary(format!("world change: {} could not be read again ({e}); closing", request.world));
                    world.write_message(AppExit::error());
                    world.resource_mut::<WorldChange>().step = Step::Down { request, frames: u32::MAX, from, journey };
                    return;
                }
            };
            let mut light = object_light(&read, &opts);
            light.scene_lights = world.resource::<crate::scene_lights::SceneLights>().buffer.clone();
            world.insert_resource(light);
            if let Some(at) = request.at {
                world.resource_mut::<connor_view::StartAt>().0 = Some(at);
            }
            {
                let mut held = world.resource_mut::<crate::door_view::HeldItem>();
                // `--item` gives its item in the first world only.
                held.0 = None;
                // The combat level before the characters enter (rule 18;
                // fighting.md rule 3).
                if let Some(j) = &journey {
                    held.1 = combat_level_number(j.combat_level);
                }
            }
            world.resource_mut::<PendingWorld>().0 = Some((read, opts));
            build_pending(world);
            give_journey(world, journey.as_deref(), &request.world, &log);
        }
    }
}

/// `setCombatLevel`'s number for a factor (`fighting.md` rule 3).
fn combat_level_number(factor: f32) -> Option<i64> {
    [(1, 0.66), (2, 1.0), (3, 1.5)].iter().find(|(_, f)| (factor - f).abs() < 0.01).map(|(n, _)| *n)
}

/// Gives the journey to the world just built (6e-3) and logs Connor's
/// things on arrival (`world.log`): the same as on leaving.
fn give_journey(world: &mut World, journey: Option<&Journey>, to: &str, log: &std::sync::Arc<std::sync::Mutex<Log>>) {
    let Some(j) = journey else { return };
    let Some(mut sim) = world.get_resource_mut::<crate::door_view::DoorSim>() else {
        log.lock().unwrap().summary("world change: no rules running in the new world: the journey is lost");
        return;
    };
    let active = &mut sim.active;
    let lines = j.give(&mut active.world, &mut active.scripts.vars);
    let arrived = connor_summary(&active.world);
    let after = Counts::of(world);
    let mut log = log.lock().unwrap();
    for line in &lines {
        log.summary(format!("world change: {line}"));
    }
    if let Ok(mut wl) = Log::create("world") {
        for line in &lines {
            wl.detail(line);
        }
        wl.detail(format!("Connor arriving in {to}: {arrived}"));
        wl.detail(format!("after building {to}: {}; {}", after.line(), crate::memory::line()));
    }
}

/// Takes the world down: despawns every [`WorldEntity`] and removes the
/// per-world resources (their logs close with them). Returns how many of
/// each.
fn take_down(world: &mut World) -> (usize, usize) {
    let marked: Vec<Entity> = world.query_filtered::<Entity, With<WorldEntity>>().iter(world).collect();
    let mut despawned = 0;
    for e in marked {
        // Children go with their parent; they may be gone already.
        despawned += usize::from(world.try_despawn(e).is_ok());
    }
    (despawned, remove_world_resources(world))
}

/// The entities left that are not the world's, by their components, for
/// `viewer.log` (Bevy's own: the window, observers, ...).
fn list_unmarked(world: &mut World, log: &mut Log) {
    let left: Vec<Entity> = world.query_filtered::<Entity, Without<WorldEntity>>().iter(world).collect();
    let mut kinds: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for e in left {
        let names: Vec<String> = world.inspect_entity(e).map(|infos| infos.map(|(_, i)| short_name(&i.name().to_string())).collect()).unwrap_or_default();
        *kinds.entry(if names.is_empty() { "(no components)".into() } else { names.join(" + ") }).or_default() += 1;
    }
    log.detail(format!("world change: {} kinds of entities stay:", kinds.len()));
    for (k, n) in kinds {
        log.detail(format!("  {n} x {k}"));
    }
}

/// `bevy_ecs::observer::Observer` → `Observer`.
fn short_name(full: &str) -> String {
    full.split('<').next().unwrap_or(full).rsplit("::").next().unwrap_or(full).to_string()
}

/// Adds the marking of world entities to the app.
pub fn add(app: &mut App) {
    app.init_resource::<PendingWorld>()
        .init_resource::<WorldChange>()
        .init_schedule(BuildWorld)
        .add_observer(mark_world_entity)
        .add_systems(Last, world_change_step);
}

/// Every entity spawned with a `Transform` (drawn things, lights, the
/// camera) or a `Node` (panels, labels) belongs to the world, unless it
/// is marked [`Persistent`].
fn mark_world_entity(on: On<Add<(Transform, Node)>>, persistent: Query<(), With<Persistent>>, mut commands: Commands) {
    if !persistent.contains(on.entity) {
        if let Ok(mut e) = commands.get_entity(on.entity) {
            e.try_insert(WorldEntity);
        }
    }
}

/// The resources that belong to one world: put in by [`insert_world`] or
/// by the start-up systems. Taking a world down (6e-2) removes them.
macro_rules! per_world {
    ($($t:ty),* $(,)?) => {
        /// How many of the per-world resources are in the app now.
        fn world_resources(world: &World) -> (usize, usize) {
            let (mut present, mut total) = (0, 0);
            $(
                total += 1;
                present += usize::from(world.contains_resource::<$t>());
            )*
            (present, total)
        }

        /// Removes the per-world resources; returns how many there were.
        fn remove_world_resources(world: &mut World) -> usize {
            let mut removed = 0;
            $(
                removed += usize::from(world.remove_resource::<$t>().is_some());
            )*
            removed
        }
    };
}

per_world!(
    object_light::ObjectLight,
    talk_view::CaptionStyle,
    gui_view::Gui,
    map_view::MapSim,
    connor_view::FootSounds,
    sound_view::SoundSim,
    light_view::GroundLight,
    terrain_view::LoadedGround,
    crate::ground_anim_view::LoadedGroundAnim,
    terrain_view::FloorView,
    terrain_view::LoadedObjects,
    terrain_view::Fly,
    crate::door_view::LoadQueue,
    crate::door_view::Deleted,
    crate::door_view::CharacterColliders,
    talk_view::Speech,
    gui_view::GuiCover,
    gui_view::MapDrag,
    gui_view::ScreenFrame,
    gui_view::BarActions,
    gui_view::CaptionPlace,
    object_light::HazeCheck,
    crate::game_camera::CameraChoice,
    // Put in by the start-up systems.
    crate::anim_view::AnimSim,
    connor_view::ConnorSim,
    crate::game_camera::GameCam,
    crate::copies_view::TemplateDraw,
    crate::door_view::DoorSim,
    crate::effects_view::EffectsSim,
    crate::ground_anim_view::GroundAnimSim,
    crate::item_view::ItemDraw,
    crate::item_view::CorpseDraw,
    crate::potion_view::OrbitDraw,
    terrain_view::GroundLookup,
    terrain_view::ViewBlockerSet,
    crate::shadow_view::ShadowWorld,
    terrain_view::ColliderOrder,
    terrain_view::ObjectColliders,
    terrain_view::Buildings,
    crate::shadow_view::BuildingShadows,
    // `draw_distance::add` and `object_light::add` (empty at the start).
    crate::draw_distance::ActivePoint,
    crate::draw_distance::ActivityLog,
    crate::draw_distance::ObjectActivity,
    crate::draw_distance::CharactersInRange,
    crate::draw_distance::CharactersSolid,
    crate::draw_distance::ActiveRules,
    crate::draw_distance::ColliderSwitches,
    crate::draw_distance::SwitchedOut,
    crate::scene_lights::LightsGroup,
    crate::scene_lights::GroundLights,
);

/// Entity and asset counts, for the logs (a round trip must give the
/// same numbers, 6e-2).
pub struct Counts {
    pub entities: usize,
    pub world_entities: usize,
    pub meshes: usize,
    pub images: usize,
    pub object_materials: usize,
    pub standard_materials: usize,
    pub ground_materials: usize,
    pub building_materials: usize,
    pub resources: (usize, usize),
}

impl Counts {
    pub fn of(world: &mut World) -> Counts {
        let entities = world.query::<Entity>().iter(world).count();
        let world_entities = world.query_filtered::<Entity, With<WorldEntity>>().iter(world).count();
        Counts {
            entities,
            world_entities,
            meshes: world.resource::<Assets<Mesh>>().len(),
            images: world.resource::<Assets<Image>>().len(),
            object_materials: world.resource::<Assets<object_light::ObjectMaterial>>().len(),
            standard_materials: world.resource::<Assets<StandardMaterial>>().len(),
            ground_materials: world.resource::<Assets<crate::ground_haze::GroundMaterial>>().len(),
            building_materials: world.resource::<Assets<crate::building_light::BuildingMaterial>>().len(),
            resources: world_resources(world),
        }
    }

    pub fn line(&self) -> String {
        format!(
            "entities {} ({} marked WorldEntity); meshes {}, images {}, materials: object {}, standard {}, ground {}, building {}; per-world resources {} of {}",
            self.entities,
            self.world_entities,
            self.meshes,
            self.images,
            self.object_materials,
            self.standard_materials,
            self.ground_materials,
            self.building_materials,
            self.resources.0,
            self.resources.1
        )
    }
}
