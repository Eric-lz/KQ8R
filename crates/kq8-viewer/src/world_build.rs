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
//!
//! Ours: the original builds a world from its scripts in the same program
//! too (`world-change.md` rules 7–8), but how our viewer is split into
//! resources and systems is our own.

use std::time::Instant;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use kq8_data::GameDir;

use crate::log::Log;
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
    sound: Option<sound_view::SoundSim>,
    /// How long the reading took.
    pub took_ms: u128,
}

/// The world waiting to be put in by [`build_pending`].
#[derive(Resource, Default)]
pub struct PendingWorld(pub Option<(WorldRead, Options)>);

/// Reads world `name` (as `main` did before 6e-1).
pub fn read_world(game: &GameDir, name: &str, opts: &Options, log: &mut Log) -> Result<WorldRead, String> {
    let started = Instant::now();
    let world = load::World::open(game, name)?;
    log.summary(format!("world: {}", world.name));
    // M1 step 5b: the lights for models; step 6: the haze.
    let lights = load::load_object_lights(game, &world, log);
    let mut ground = load::load_ground(game, &world, log)?;
    let haze = load::load_haze(game, &world, Some(&ground.terrain.settings), log);
    let mut objects = load::load_objects(game, &world, &ground.terrain, log);
    let animators = std::mem::take(&mut objects.animators);
    let animation = load::prepare_ground_animation(game, &world, &mut ground, animators, log);
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
    let sound = sound_view::SoundSim::new(game, name, opts.sound, audibles, music);
    Ok(WorldRead { name: name.to_string(), lights, haze, ground, objects, animation, caption, gui, map, sound, took_ms: started.elapsed().as_millis() })
}

/// The model lights and haze for `ObjectLight`, which the app needs
/// before its plugins are added (`object_light::add`).
pub fn object_light(read: &WorldRead, opts: &Options) -> object_light::ObjectLight {
    object_light::ObjectLight::new(read.lights.clone(), opts.light, read.haze, opts.haze)
}

/// Puts a read world's resources in. `ObjectLight` is put in by the
/// caller (see [`object_light`]).
pub fn insert_world(world: &mut World, read: WorldRead, opts: &Options) {
    let WorldRead { ground, mut objects, animation, caption, gui, map, sound, .. } = read;
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
    if let Some(sound) = sound {
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
}

/// Startup (and later on request): puts the pending world in and spawns
/// it, then logs what it holds.
pub fn build_pending(world: &mut World) {
    let Some((read, opts)) = world.resource_mut::<PendingWorld>().0.take() else { return };
    let started = Instant::now();
    let (name, read_ms) = (read.name.clone(), read.took_ms);
    insert_world(world, read, &opts);
    world.run_schedule(BuildWorld);
    let counts = Counts::of(world);
    let log = world.resource::<Shared>().log.clone();
    let mut log = log.lock().unwrap();
    log.summary(format!("world built: {name}: read {read_ms} ms, spawned in {} ms; {}", started.elapsed().as_millis(), counts.line()));
}

/// Adds the marking of world entities to the app.
pub fn add(app: &mut App) {
    app.init_resource::<PendingWorld>().init_schedule(BuildWorld).add_observer(mark_world_entity);
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
