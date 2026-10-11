//! `kq8-viewer`: shows one KQ8 model in a window (M1 step 1).
//!
//! Usage:
//!   kq8-viewer [<model.kqs>] [--world <World>] [--palette <file>]
//!              [--game DIR] [--frames N]
//!
//! Defaults: model `smlchest.KQS`, world `Daventry`, the world's own
//! palette. `--frames N` closes the window by itself after N frames.
//! Mouse: left-drag to turn around the model, wheel to zoom.
//! Keys: arrows turn, R resets the view, Esc quits.
//!
//! What is deliberately simple here (see docs/DESIGN.md, M1 step 1b):
//! no lighting, both sides of every triangle drawn, no animation.

mod load;
mod log;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat};
use kq8_data::GameDir;

use load::{Look, Model, World};
use log::Log;

/// Game units per viewer unit. Models are a few hundred game units
/// across; dividing by 100 keeps numbers in a comfortable range for the
/// renderer. A display choice only, not a claim about the game's units.
const UNITS_PER_METRE: f32 = 100.0;

/// Game space is Z-up (measured, see docs/provenance); Bevy is Y-up.
/// (x, y, z) -> (x, z, -y) is a rotation, not a mirror. Whether the game
/// itself is left- or right-handed is open (docs/behaviour/QUESTIONS.md Q2).
fn to_bevy(p: [f32; 3], centre: [f32; 3]) -> [f32; 3] {
    let d = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
    [d[0] / UNITS_PER_METRE, d[2] / UNITS_PER_METRE, -d[1] / UNITS_PER_METRE]
}

struct Args {
    model: String,
    world: String,
    palette: Option<String>,
    game: PathBuf,
    frames: Option<u64>,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        model: "smlchest.KQS".into(),
        world: "Daventry".into(),
        palette: None,
        game: std::env::var_os("KQ8_GAME_DIR").map(PathBuf::from).unwrap_or_else(|| "MASK".into()),
        frames: None,
    };
    let mut it = std::env::args().skip(1);
    let mut positional = Vec::new();
    while let Some(arg) = it.next() {
        let mut value = || it.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--world" => a.world = value()?,
            "--palette" => a.palette = Some(value()?),
            "--game" => a.game = value()?.into(),
            "--frames" => a.frames = Some(value()?.parse().map_err(|_| "--frames needs a number".to_string())?),
            s if s.starts_with("--") => return Err(format!("unknown option {s}")),
            _ => positional.push(arg),
        }
    }
    match positional.as_slice() {
        [] => {}
        [m] => a.model = m.clone(),
        _ => return Err("give at most one model name".into()),
    }
    Ok(a)
}

/// Shared with `main` so the final numbers can be written after the
/// window closes.
#[derive(Resource, Clone)]
struct Shared {
    log: Arc<Mutex<Log>>,
    stats: Arc<Mutex<Stats>>,
}

#[derive(Default)]
struct Stats {
    frames: u64,
    first_frame_after_ms: Option<u128>,
    worst_frame_ms: f32,
    total_frame_ms: f64,
}

#[derive(Resource)]
struct Settings {
    frame_limit: Option<u64>,
    started: Instant,
    last_report: Instant,
    radius: f32,
}

#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    distance: f32,
    default_distance: f32,
}

#[derive(Resource)]
struct Loaded(Option<Model>);

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("usage: kq8-viewer [<model.kqs>] [--world <World>] [--palette <file>] [--game DIR] [--frames N]");
            return ExitCode::FAILURE;
        }
    };
    let mut log = match Log::create("viewer") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("log file: {}", log.path().display()));
    let loaded = GameDir::open(&args.game)
        .map_err(|e| format!("{e} (pass --game <folder> or set KQ8_GAME_DIR)"))
        .and_then(|game| {
            let world = World::open(&game, &args.world)?;
            log.summary(format!("world: {}", world.name));
            load::load_model(&game, &world, &args.model, args.palette.as_deref(), &mut log)
        });
    let model = match loaded {
        Ok(m) => m,
        Err(e) => {
            log.summary(format!("error: {e}"));
            log.summary("RESULT: FAIL");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("loading took {} ms", started.elapsed().as_millis()));

    let size: [f32; 3] = std::array::from_fn(|i| model.max[i] - model.min[i]);
    let radius = 0.5 * (size[0] * size[0] + size[1] * size[1] + size[2] * size[2]).sqrt() / UNITS_PER_METRE;
    let shared = Shared { log: Arc::new(Mutex::new(log)), stats: Arc::new(Mutex::new(Stats::default())) };
    let title = format!("KQ8 viewer - {}", model.label);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title, ..default() }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.18, 0.2, 0.24)))
        .insert_resource(shared.clone())
        .insert_resource(Loaded(Some(model)))
        .insert_resource(Settings { frame_limit: args.frames, started, last_report: Instant::now(), radius })
        .insert_resource(Orbit { yaw: 0.6, pitch: 0.35, distance: radius * 3.0, default_distance: radius * 3.0 })
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_input, place_camera, frame_stats).chain())
        .run();

    let mut log = shared.log.lock().unwrap();
    let s = shared.stats.lock().unwrap();
    let avg = if s.frames > 0 { s.total_frame_ms / s.frames as f64 } else { 0.0 };
    log.summary(format!(
        "window closed: frames={} first_frame_after_ms={:?} average_frame_ms={avg:.2} worst_frame_ms={:.2}",
        s.frames, s.first_frame_after_ms, s.worst_frame_ms
    ));
    let ok = s.frames > 0;
    log.summary(if ok { "RESULT: PASS (window ran; whether it looks right needs your eyes)" } else { "RESULT: FAIL (no frames drawn)" });
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn setup(
    mut commands: Commands,
    mut loaded: ResMut<Loaded>,
    shared: Res<Shared>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let model = loaded.0.take().expect("model is loaded once");
    let centre: [f32; 3] = std::array::from_fn(|i| 0.5 * (model.min[i] + model.max[i]));
    let mut log = shared.log.lock().unwrap();
    let mut vertices = 0;
    for group in model.groups {
        let mut positions = Vec::with_capacity(group.triangles.len() * 3);
        let mut uvs = Vec::with_capacity(group.triangles.len() * 3);
        for t in &group.triangles {
            for k in 0..3 {
                positions.push(to_bevy(t.positions[k], centre));
                uvs.push(t.uvs[k]);
            }
        }
        vertices += positions.len();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.compute_flat_normals();

        let base = StandardMaterial {
            unlit: true,
            cull_mode: None,
            double_sided: true,
            ..default()
        };
        let material = match group.look {
            Look::Texture { width, height, rgba, .. } => {
                let mut image = Image::new(
                    Extent3d { width, height, depth_or_array_layers: 1 },
                    TextureDimension::D2,
                    rgba,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..ImageSamplerDescriptor::linear()
                });
                StandardMaterial { base_color_texture: Some(images.add(image)), ..base }
            }
            Look::Color([r, g, b]) => StandardMaterial { base_color: Color::srgb_u8(r, g, b), ..base },
            Look::Missing(_) => StandardMaterial { base_color: Color::srgb(1.0, 0.0, 1.0), ..base },
        };
        commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(materials.add(material))));
        log.detail(format!("spawned mesh for material {} ({} triangles)", group.material, group.triangles.len()));
    }
    log.summary(format!(
        "spawned: vertices={vertices} centre_moved_to_origin=({:.1}, {:.1}, {:.1}) game units",
        centre[0], centre[1], centre[2]
    ));
    commands.spawn((Camera3d::default(), Transform::default()));
}

fn orbit_input(
    mut orbit: ResMut<Orbit>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if buttons.pressed(MouseButton::Left) {
        orbit.yaw -= motion.delta.x * 0.005;
        orbit.pitch += motion.delta.y * 0.005;
    }
    let turn = 1.5 * time.delta_secs();
    if keys.pressed(KeyCode::ArrowLeft) {
        orbit.yaw += turn;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        orbit.yaw -= turn;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        orbit.pitch += turn;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        orbit.pitch -= turn;
    }
    orbit.pitch = orbit.pitch.clamp(-1.5, 1.5);
    if scroll.delta.y != 0.0 {
        orbit.distance *= 0.9f32.powf(scroll.delta.y.signum());
    }
    if keys.just_pressed(KeyCode::KeyR) {
        orbit.yaw = 0.6;
        orbit.pitch = 0.35;
        orbit.distance = orbit.default_distance;
    }
    if keys.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}

fn place_camera(orbit: Res<Orbit>, settings: Res<Settings>, mut camera: Single<&mut Transform, With<Camera3d>>) {
    let d = orbit.distance.max(settings.radius * 0.2);
    let eye = Vec3::new(
        d * orbit.pitch.cos() * orbit.yaw.sin(),
        d * orbit.pitch.sin(),
        d * orbit.pitch.cos() * orbit.yaw.cos(),
    );
    **camera = Transform::from_translation(eye).looking_at(Vec3::ZERO, Vec3::Y);
}

fn frame_stats(
    mut settings: ResMut<Settings>,
    shared: Res<Shared>,
    orbit: Res<Orbit>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let mut s = shared.stats.lock().unwrap();
    s.frames += 1;
    let ms = time.delta_secs() * 1000.0;
    if s.frames == 1 {
        s.first_frame_after_ms = Some(settings.started.elapsed().as_millis());
    } else {
        // The first frame's time includes start-up, so it is left out.
        s.total_frame_ms += ms as f64;
        s.worst_frame_ms = s.worst_frame_ms.max(ms);
    }
    if settings.last_report.elapsed().as_secs_f32() >= 2.0 {
        settings.last_report = Instant::now();
        shared.log.lock().unwrap().detail(format!(
            "frames={} last_frame_ms={ms:.2} worst_frame_ms={:.2} camera yaw={:.2} pitch={:.2} distance={:.2}",
            s.frames, s.worst_frame_ms, orbit.yaw, orbit.pitch, orbit.distance
        ));
    }
    if settings.frame_limit.is_some_and(|n| s.frames >= n) {
        commands.write_message(AppExit::Success);
    }
}
