//! `kq8-viewer`: shows a world's ground (M1 step 2) or one KQ8 model
//! (M1 step 1) in a window.
//!
//! Usage (`--help` prints the full list of options; `--world` with no
//! name lists the worlds):
//!   kq8-viewer [--world <World>] [--game DIR] [--frames N] [--lift N] [--hold F,T,R[,N]] [--jump N] [--turn N[,1|-1]]
//!              [--at X,Y,H] [--light x1|x2|off] [--renderer auto|vulkan|dx12]
//!   kq8-viewer --model <model.kqs> [--world <World>] [--palette <file>]
//!              [--game DIR] [--frames N]
//!
//! With no `--model`, the world's ground is shown (the default). `--terrain`
//! is still accepted and does nothing, so older commands keep working.
//! Defaults: world `Daventry`, the world's own palette. `--frames N` closes
//! the window by itself after N frames.
//! Model: left-drag to turn around the model, wheel to zoom; arrows turn,
//! R resets the view. Ground: W/A/S/D fly, Q/E down/up, Shift faster,
//! hold the right mouse button to look around, H shows/hides invisible
//! objects (green: no model, orange: a character whose movement list could
//! not be read; since M4 4d-2 characters are drawn). Connor (M2): keys
//! from Connor.sae (arrows walk/turn, End run on/off); the game camera
//! follows him (right mouse swing/zoom, Shift, Page Up/Down tilt, Home,
//! F11 fixed distance; keys from Camera.sae); F switches to the free
//! camera and back; G / B lift him 5,000 / 2,000 to test falling.
//! `--at X,Y,H` starts him on the ground at X, Y facing heading H (radians).
//! Doors (M3): left click on a door uses it (play cursor); `--click NAME,N`
//! clicks object NAME before update N + 1, `--item TYPE` puts an item of
//! that type in Connor's hand (tests). Emitters (M3 step 3b): falling
//! leaves and drips, logged to effects.log; flowing water squares, logged
//! to ground_anim.log. Ground light (M1 step 4): F7 cycles the light
//! map x1 / x2 / off (`--light` sets the start; default x1). Models are
//! lit by the world's ambient light and sun (M1 step 5b, `object_light`),
//! buildings by their `.dil` light maps (M1 step 5c, `building_light`);
//! F7 switches them too. Haze (M1 step 6): objects fade into the haze
//! colour with distance from the camera; F8 (`--haze on|off`) switches it.
//! `--renderer auto|vulkan|dx12` picks the graphics API (default auto).
//! `--dynamic on|off` stands in for `Options.cs`'s `useDynamicLighting`
//! (M1 7d): the ground's map choice and point lights on the ground and on
//! building surfaces; F9 switches it while the viewer runs (M1 7j).
//! Esc quits.
//!
//! What is deliberately simple here (see docs/DESIGN.md, M1 step 1b):
//! no animation in the model view. Back faces are hidden as in the original
//! (docs/behaviour/models-and-textures.md, Q2).

mod anim_view;
mod building_light;
mod connor_view;
mod copies_view;
mod door_view;
mod draw_distance;
mod effects_view;
mod game_camera;
mod ground_haze;
mod ground_anim_view;
mod gui_view;
mod item_view;
mod light_view;
mod load;
mod log;
mod object_light;
mod renderer;
mod scene_lights;
mod shadow_view;
mod sound_view;
mod talk_view;
mod terrain_view;
mod wall_lights;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use kq8_data::shape::Triangle;
use kq8_data::{GameDir, Lighting};

use load::{Model, World};
use log::Log;

/// Game units per viewer unit. Models are a few hundred game units
/// across; dividing by 100 keeps numbers in a comfortable range for the
/// renderer. A display choice only, not a claim about the game's units.
const UNITS_PER_METRE: f32 = 100.0;

/// Game space is right-handed and Z-up (docs/behaviour/models-and-textures.md,
/// Q2); Bevy is right-handed and Y-up. (x, y, z) -> (x, z, -y) is a
/// rotation, not a mirror.
fn to_bevy(p: [f32; 3], centre: [f32; 3]) -> [f32; 3] {
    let d = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
    [d[0] / UNITS_PER_METRE, d[2] / UNITS_PER_METRE, -d[1] / UNITS_PER_METRE]
}

struct Args {
    /// `--model NAME`: show this model alone instead of the ground.
    model: Option<String>,
    world: String,
    palette: Option<String>,
    game: PathBuf,
    frames: Option<u64>,
    /// Test: lift Connor this much at the start (like the G/B keys).
    lift: Option<f32>,
    /// Test: hold this intent (forward,turn,run e.g. `1,0,0`) for Connor,
    /// optionally only for the first N updates (`1,0,0,30`).
    hold: Option<([i8; 3], Option<u64>)>,
    /// Test: press Jump once, before update N + 1.
    jump: Vec<u64>,
    /// Test: press a turn key before update N + 1 and hold it (`N` or
    /// `N,1` left, `N,-1` right).
    turn: Option<(u64, i8)>,
    /// Test: start Connor on the ground at X, Y facing heading H (radians).
    at: Option<[f32; 3]>,
    /// Test: click object NAME before update N + 1 (may be given more
    /// than once).
    click: Vec<(String, u64)>,
    /// `--aim NAME,N`: a ray from the camera to NAME at update N (3e-4b).
    aim: Vec<(String, u64)>,
    /// `--swing NAME,N[,big]`: at update N Connor readies his hand weapon
    /// (if none is ready) and swings at NAME, as a click on it with the
    /// weapon ready (Shift with `big`; M5 5c-5 test, repeatable).
    swing: Vec<(String, u64, bool)>,
    /// `--combat-level 1|2|3`: the combat level before the world's
    /// characters enter it (M5 5c-10; stand-in for the menu).
    combat_level: Option<i64>,
    /// `--special ITEMS,N`: before update N + 1, Connor gets
    /// `KQMonster::setState Connor special ITEMS` as if from a script
    /// (M3 3i-7 test; ITEMS uses `;` between items, e.g.
    /// `AnimList=bricks.anm;AnimListOn;push`).
    special: Option<(String, u64)>,
    /// `--command "WORDS",N`: before update N + 1, one script command
    /// runs as if a script did (M4 4d-5 test; repeatable).
    command: Vec<(String, u64)>,
    /// `--right N`: the camera-move (right) button counts as held from
    /// update N on, for a conversation's stay test (M4 4e-5 test).
    right: Option<u64>,
    /// Test: Connor holds an item of this type.
    item: Option<String>,
    /// `--key NAME,N`: a panel key pressed before update N + 1 (M4 4c-2;
    /// `space`).
    key: Vec<(String, u64)>,
    /// `--light x1|x2|off`: how the ground's light map is applied at the
    /// start (M1 step 4c; default x1, the user's check); F7 changes it.
    light: light_view::LightMode,
    /// `--haze on|off`: haze at the start (M1 step 6); F8 switches it.
    haze: bool,
    /// `--corners on|off`: the bottom bar's corner panels at the window's
    /// edges (M4 4c-11, default off); F6 switches it.
    corners: bool,
    /// `--renderer auto|vulkan|dx12`: the graphics API (default auto).
    renderer: renderer::Renderer,
    /// `--dynamic on|off`: overrides `Options.cs`'s `useDynamicLighting`
    /// (M1 7d); `None` = as `Options.cs` says.
    dynamic: Option<bool>,
    /// `--sound on|off`: open the sound card (default on); off still runs
    /// the sound rules and `sound.log` (M3 3h-2).
    sound: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        model: None,
        world: "Daventry".into(),
        palette: None,
        game: std::env::var_os("KQ8_GAME_DIR").map(PathBuf::from).unwrap_or_else(|| "MASK".into()),
        frames: None,
        lift: None,
        hold: None,
        jump: Vec::new(),
        turn: None,
        at: None,
        click: Vec::new(),
        aim: Vec::new(),
        swing: Vec::new(),
        combat_level: None,
        special: None,
        command: Vec::new(),
        right: None,
        item: None,
        key: Vec::new(),
        light: light_view::LightMode::X1,
        haze: true,
        corners: false,
        renderer: renderer::Renderer::Auto,
        dynamic: None,
        sound: true,
    };
    let mut it = std::env::args().skip(1);
    let mut positional = Vec::new();
    while let Some(arg) = it.next() {
        let mut value = || it.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--world" => a.world = value()?,
            "--palette" => a.palette = Some(value()?),
            "--game" => a.game = value()?.into(),
            // The ground is the default now; kept so older commands still run.
            "--terrain" => {}
            "--model" => a.model = Some(value()?),
            "--hold" => {
                let v = value()?;
                let n: Vec<i64> = v.split(',').map(|x| x.trim().parse::<i64>()).collect::<Result<_, _>>().map_err(|_| "--hold needs forward,turn,run[,updates] e.g. 1,0,0".to_string())?;
                // Optional 4th number: let go after that many updates.
                let (keys, until) = match n[..] {
                    [f, t, r] => ([f, t, r], None),
                    [f, t, r, u] if u >= 0 => ([f, t, r], Some(u as u64)),
                    _ => return Err("--hold needs three or four numbers, e.g. 1,0,0 or 1,0,0,30".into()),
                };
                a.hold = Some((keys.map(|k| k.clamp(-1, 1) as i8), until));
            }
            "--at" => {
                let v = value()?;
                let n: Vec<f32> = v.split(',').map(|x| x.trim().parse::<f32>()).collect::<Result<_, _>>().map_err(|_| "--at needs x,y,heading e.g. 286169,190211,0.291".to_string())?;
                let [x, y, h] = n[..] else { return Err("--at needs three numbers, e.g. 286169,190211,0.291".into()) };
                a.at = Some([x, y, h]);
            }
            "--click" => {
                let v = value()?;
                let (name, n) = v.rsplit_once(',').ok_or("--click needs NAME,UPDATE e.g. Condoor,30".to_string())?;
                let n = n.trim().parse::<u64>().map_err(|_| "--click needs NAME,UPDATE e.g. Condoor,30".to_string())?;
                a.click.push((name.trim().to_string(), n));
            }
            "--combat-level" => {
                let v = value()?;
                let n = v.trim().parse::<i64>().ok().filter(|n| (1..=3).contains(n)).ok_or("--combat-level needs 1, 2 or 3".to_string())?;
                a.combat_level = Some(n);
            }
            "--swing" => {
                let v = value()?;
                let usage = "--swing needs NAME,UPDATE[,big] e.g. goblin22,60".to_string();
                let w: Vec<&str> = v.split(',').map(str::trim).collect();
                let (name, n, big) = match w[..] {
                    [name, n] => (name, n, false),
                    [name, n, b] if b.eq_ignore_ascii_case("big") => (name, n, true),
                    _ => return Err(usage),
                };
                a.swing.push((name.to_string(), n.parse::<u64>().map_err(|_| usage.clone())?, big));
            }
            "--aim" => {
                let v = value()?;
                let (name, n) = v.rsplit_once(',').ok_or("--aim needs NAME,UPDATE e.g. smlchst3,30".to_string())?;
                let n = n.trim().parse::<u64>().map_err(|_| "--aim needs NAME,UPDATE e.g. smlchst3,30".to_string())?;
                a.aim.push((name.trim().to_string(), n));
            }
            "--special" => {
                let v = value()?;
                let (items, n) = v.rsplit_once(',').ok_or("--special needs ITEMS,UPDATE e.g. kneel;kneelstop,30".to_string())?;
                let n = n.trim().parse::<u64>().map_err(|_| "--special needs ITEMS,UPDATE e.g. kneel;kneelstop,30".to_string())?;
                a.special = Some((items.replace(';', ","), n));
            }
            "--command" => {
                let v = value()?;
                let usage = "--command needs \"WORDS\",UPDATE e.g. \"KQMonster::speak Wizard Connor 1000 4 0 8 1 1\",30".to_string();
                let (words, n) = v.rsplit_once(',').ok_or(usage.clone())?;
                let n = n.trim().parse::<u64>().map_err(|_| usage)?;
                a.command.push((words.trim().to_string(), n));
            }
            "--right" => a.right = Some(value()?.trim().parse::<u64>().map_err(|_| "--right needs an update number, e.g. 60".to_string())?),
            "--item" => a.item = Some(value()?),
            "--key" => {
                let v = value()?;
                let (name, n) = v.rsplit_once(',').ok_or("--key needs NAME,UPDATE e.g. space,30".to_string())?;
                let n = n.trim().parse::<u64>().map_err(|_| "--key needs NAME,UPDATE e.g. space,30".to_string())?;
                a.key.push((name.trim().to_string(), n));
            }
            "--light" => a.light = light_view::LightMode::parse(&value()?)?,
            "--haze" => {
                a.haze = match value()?.to_ascii_lowercase().as_str() {
                    "on" | "1" => true,
                    "off" | "0" => false,
                    v => return Err(format!("--haze needs on or off, not '{v}'")),
                }
            }
            "--corners" => {
                a.corners = match value()?.to_ascii_lowercase().as_str() {
                    "on" | "1" => true,
                    "off" | "0" => false,
                    v => return Err(format!("--corners needs on or off, not '{v}'")),
                }
            }
            "--renderer" => a.renderer = renderer::Renderer::parse(&value()?)?,
            "--sound" => {
                a.sound = match value()?.as_str() {
                    "on" => true,
                    "off" => false,
                    v => return Err(format!("--sound needs on or off, not '{v}'")),
                }
            }
            "--dynamic" => {
                a.dynamic = Some(match value()?.to_ascii_lowercase().as_str() {
                    "on" | "1" => true,
                    "off" | "0" => false,
                    v => return Err(format!("--dynamic needs on or off, not '{v}'")),
                })
            }
            "--lift" => a.lift = Some(value()?.parse().map_err(|_| "--lift needs a number".to_string())?),
            "--jump" => a.jump.push(value()?.parse().map_err(|_| "--jump needs an update number".to_string())?),
            "--turn" => {
                let v = value()?;
                let n: Vec<i64> = v.split(',').map(|x| x.trim().parse::<i64>()).collect::<Result<_, _>>().map_err(|_| "--turn needs updates[,1 left / -1 right] e.g. 40,-1".to_string())?;
                a.turn = match n[..] {
                    [u] if u >= 0 => Some((u as u64, 1)),
                    [u, t] if u >= 0 && (t == 1 || t == -1) => Some((u as u64, t as i8)),
                    _ => return Err("--turn needs updates[,1 left / -1 right] e.g. 40 or 40,-1".into()),
                };
            }
            "--frames" => a.frames = Some(value()?.parse().map_err(|_| "--frames needs a number".to_string())?),
            s if s.starts_with("--") => return Err(format!("unknown option {s}")),
            _ => positional.push(arg),
        }
    }
    if !positional.is_empty() {
        return Err(format!("a model is given with --model, e.g. --model {}", positional[0]));
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

/// `--help` / `-h`: the launch options, grouped as in `docs/FEATURES.md`
/// ("Launch options").
const HELP: &str = "\
kq8-viewer: launches the game window.

Usage:
  cargo play [OPTIONS]                  (= cargo run --release -p kq8-viewer -- [OPTIONS])
  cargo play --world <world> [OPTIONS]  loads a specific world. 

Showing things:
  --world <name>             world to load (default Daventry; no name: list the worlds)
  --at X,Y,H                 start Connor at X, Y facing heading H (radians)
  --renderer auto|vulkan|dx12  graphics API (default auto)
  --game <folder>            where the game is (default MASK; or KQ8_GAME_DIR)
  --sound on|off             play sound through the sound card (default on)
  --light x1|x2|off          light of the ground and objects at the start (default x1; F7)
  --dynamic on|off           dynamic lights, instead of what Options.cs says
  --haze on|off              distance fog at the start (default on; F8)
  --corners on|off           bottom bar's corner panels at the window's edges (default off; F6)

Used mostly for debugging:
  --model <file>             show one model, e.g. --model smlchest.KQS
  --palette <file>           another palette for --model
  --frames <number>          close the window by itself after that many frames

Test options (drive Connor from the command line; ground view only).
N is an update number (15 updates per second):
  --at X,Y,H                 start Connor at X, Y facing heading H (radians)
  --lift <number>            lift him that high once at the start, then let him fall
  --hold F,T,R[,N]           hold forward (1 / -1 back), turn (1 left / -1 right),
                             run (1 / 0); let go of forward and turn after N updates
  --turn N[,1|-1]            press a turn key before update N + 1 and keep it pressed
  --jump N                   press Jump before update N + 1 (repeatable)
  --click NAME,N             click object NAME before update N + 1 (repeatable;
                             skips the line-of-sight test)
  --aim NAME,N               like --click, but only if the camera can see it
  --swing NAME,N[,big]       at update N ready the hand weapon and swing at NAME
                             (big: the Shift swing)
  --combat-level 1|2|3       the combat level (0.66 / 1.0 / 1.5) before the
                             characters enter the world
  --item TYPE                give Connor an item of that type and hold it
  --key NAME,N               press space or 1-8, or click a bar control
                             (left:NAME / right:NAME), before update N + 1 (repeatable)
  --special ITEMS,N          play Connor's special movements (separated by ;)
  --command \"WORDS\",N      run one script command at update N (repeatable)
  --right N                the right (camera-move) button counts as held from update N on (conversations)

  -h, --help                 show this list

Keys while playing are listed in README.md; details of every option in
docs/FEATURES.md (\"Launch options\").
";

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.iter().any(|a| a == "--help" || a == "-h") {
        print!("{HELP}");
        return ExitCode::SUCCESS;
    }
    // `--world` with no name after it (the end, or another option next):
    // the list of worlds.
    if raw.iter().enumerate().any(|(i, a)| a == "--world" && raw.get(i + 1).is_none_or(|next| next.starts_with('-'))) {
        println!("Worlds (not case-sensitive), e.g. cargo play --world swamp:");
        for w in kq8_data::game_dir::WORLDS {
            println!("  {w}{}", if w == "Daventry" { "  (default)" } else { "" });
        }
        return ExitCode::SUCCESS;
    }
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("Run `cargo play --help` for the list of options.");
            return ExitCode::FAILURE;
        }
    };
    if let Some(on) = args.dynamic {
        load::override_dynamic_lighting(on);
    }
    let mut log = match Log::create("viewer") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("log file: {}", log.path().display()));
    // Made once at start-up, so the size difference between the two
    // kinds does not matter.
    #[allow(clippy::large_enum_variant)]
    enum Scene {
        Model(Model),
        Ground(load::Ground, load::Objects, load::GroundAnimAssets),
    }
    let loaded = GameDir::open(&args.game)
        .map_err(|e| format!("{e} (pass --game <folder> or set KQ8_GAME_DIR)"))
        .and_then(|game| {
            let world = World::open(&game, &args.world)?;
            log.summary(format!("world: {}", world.name));
            // M1 step 5b: the lights for models, in both views; step 6: the
            // haze (no terrain in the model view: the note's defaults).
            let lights = load::load_object_lights(&game, &world, &mut log);
            let mut haze = None;
            let scene = match &args.model {
                None => {
                    let mut ground = load::load_ground(&game, &world, &mut log)?;
                    haze = Some(load::load_haze(&game, &world, Some(&ground.terrain.settings), &mut log));
                    let mut objects = load::load_objects(&game, &world, &ground.terrain, &mut log);
                    let animators = std::mem::take(&mut objects.animators);
                    let animation = load::prepare_ground_animation(&game, &world, &mut ground, animators, &mut log);
                    Scene::Ground(ground, objects, animation)
                }
                Some(name) => Scene::Model(load::load_model(&game, &world, name, args.palette.as_deref(), None, &mut log)?),
            };
            let haze = haze.unwrap_or_else(|| load::load_haze(&game, &world, None, &mut log));
            Ok((scene, lights, haze))
        });
    let (scene, lights, haze) = match loaded {
        Ok(s) => s,
        Err(e) => {
            log.summary(format!("error: {e}"));
            log.summary("RESULT: FAIL");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("loading took {} ms", started.elapsed().as_millis()));
    // Written before the window opens: if the API is missing, Bevy stops
    // with "Unable to find a GPU" and this is the last line.
    log.summary(format!("renderer: asking for {}", args.renderer.name()));

    let shared = Shared { log: Arc::new(Mutex::new(log)), stats: Arc::new(Mutex::new(Stats::default())) };
    let title = match &scene {
        Scene::Model(m) => format!("KQ8 viewer - {}", m.label),
        Scene::Ground(..) => format!("KQ8 viewer - {} ground", args.world),
    };
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window { title, ..default() }),
                ..default()
            })
            .set(args.renderer.plugin()),
    )
    .insert_resource(renderer::RendererChoice(args.renderer))
    .add_systems(Update, renderer::report_renderer)
    .insert_resource(ClearColor(Color::srgb(0.18, 0.2, 0.24)))
    .insert_resource(shared.clone())
    .insert_resource(object_light::ObjectLight::new(lights, args.light, haze, args.haze));
    object_light::add(&mut app);
    building_light::add(&mut app);
    ground_haze::add(&mut app);
    draw_distance::add(&mut app);
    match scene {
        Scene::Model(model) => {
            let size: [f32; 3] = std::array::from_fn(|i| model.max[i] - model.min[i]);
            let radius = 0.5 * (size[0] * size[0] + size[1] * size[1] + size[2] * size[2]).sqrt() / UNITS_PER_METRE;
            app.insert_resource(Loaded(Some(model)))
                .insert_resource(Settings { frame_limit: args.frames, started, last_report: Instant::now(), radius })
                .insert_resource(Orbit { yaw: 0.6, pitch: 0.35, distance: radius * 3.0, default_distance: radius * 3.0 })
                .add_systems(Startup, (setup, object_light::report_lights).chain())
                .add_systems(Update, (orbit_input, place_camera, frame_stats).chain());
        }
        Scene::Ground(ground, mut objects, animation) => {
            let audibles = std::mem::take(&mut objects.audibles);
            // Remarks and speech (M4 4a-2).
            if let Ok(g) = GameDir::open(&args.game) {
                objects.talk = Some(talk_view::load_talk(&g, &args.world, &mut shared.log.lock().unwrap()));
                // The caption as the original's (M4 4b-3).
                if let Some(style) = talk_view::load_caption(&g, &args.world, &mut shared.log.lock().unwrap()) {
                    app.insert_resource(style);
                }
                // The panels (M4 4c-2).
                if let Some(mut gui) = gui_view::load_gui(&g, &args.world, &mut shared.log.lock().unwrap()) {
                    gui.corners = args.corners;
                    app.insert_resource(gui);
                }
            }
            let music = objects.music.take();
            app.insert_resource(connor_view::FootSounds(std::mem::take(&mut objects.footsounds)));
            if let Some(sound) = GameDir::open(&args.game).ok().and_then(|g| sound_view::SoundSim::new(&g, &args.world, args.sound, audibles, music)) {
                app.insert_resource(sound);
            }
            app.insert_resource(light_view::GroundLight::new(ground.light.clone(), ground.other_light.clone(), args.light, ground.dynamic))
                .insert_resource(terrain_view::LoadedGround(Some(ground)))
                .insert_resource(ground_anim_view::LoadedGroundAnim(Some(animation)))
                .insert_resource(terrain_view::FloorView(0))
                .insert_resource(terrain_view::LoadedObjects(Some(objects)))
                .insert_resource(Settings { frame_limit: args.frames, started, last_report: Instant::now(), radius: 0.0 })
                .insert_resource(terrain_view::Fly::new())
                .insert_resource(connor_view::StartLift(args.lift))
                .insert_resource(connor_view::HoldIntent(args.hold))
                .insert_resource(connor_view::JumpAt(args.jump.clone()))
                .insert_resource(connor_view::TurnAt(args.turn))
                .insert_resource(connor_view::StartAt(args.at))
                .insert_resource(door_view::ClickAt(args.click))
                .insert_resource(door_view::SpecialAt(args.special))
                .insert_resource(door_view::CommandAt(args.command))
                .insert_resource(door_view::RightAt(args.right))
                .insert_resource(door_view::LoadQueue::default())
                .insert_resource(door_view::AimAt(args.aim))
                .insert_resource(door_view::SwingAt(args.swing))
                .init_resource::<door_view::Deleted>()
                .init_resource::<door_view::CharacterColliders>()
                .init_resource::<talk_view::Speech>()
                .insert_resource(door_view::HeldItem(args.item, args.combat_level))
                .insert_resource(gui_view::KeyAt(args.key))
                .init_resource::<gui_view::GuiCover>()
                .init_resource::<gui_view::ScreenFrame>()
                .init_resource::<gui_view::BarActions>()
                .init_resource::<gui_view::CaptionPlace>()
                // Game logic at the original's 15 updates per second
                // (docs/DESIGN.md, M2 "Physics rate").
                .insert_resource(Time::<Fixed>::from_hz(kq8_core::UPDATES_PER_SECOND))
                .insert_resource(game_camera::CameraChoice { game: false })
                // Doors first, so a door's box is where it is drawn when
                // Connor moves (M3 step 3a).
                .add_systems(FixedUpdate, (draw_distance::apply_collider_switches, door_view::door_step, door_view::character_gravity, connor_view::connor_step, connor_view::push_apply, door_view::trap_step, talk_view::talk_step, door_view::apply_loads, copies_view::spawn_copies, game_camera::camera_step, effects_view::effects_step, anim_view::anim_step).chain())
                .add_systems(Startup, (terrain_view::setup_ground, terrain_view::setup_objects, terrain_view::setup_h_label, ground_anim_view::setup_ground_anim, light_view::setup_light_label, object_light::report_lights, ground_haze::report_ground_haze).chain())
                .add_systems(Update, (game_camera::camera_input, terrain_view::fly_input, connor_view::connor_keys, door_view::door_click_input, connor_view::connor_draw, door_view::door_draw, door_view::hide_deleted, (item_view::item_draw, item_view::corpse_draw, item_view::bolt_draw), effects_view::effects_draw, ground_anim_view::ground_anim_step, ground_anim_view::ground_anim_draw, anim_view::anim_draw, game_camera::camera_draw, terrain_view::toggle_markers, terrain_view::toggle_floor, light_view::toggle_light, light_view::report_light, terrain_view::update_camera_label, terrain_view::report_camera, frame_stats).chain())
                // F7 also switches the models' light (M1 5b).
                .add_systems(
                    Update,
                    (object_light::follow_ground_mode, building_light::follow_mode).chain().after(light_view::toggle_light),
                )
                // Haze (M1 step 6): F8, its on-screen line, the log.
                .init_resource::<object_light::HazeCheck>()
                .add_systems(Startup, object_light::setup_haze_label)
                // Captions (M4 4a-4).
                .add_systems(Startup, (talk_view::setup_caption, talk_view::setup_caption_picture, gui_view::setup_gui))
                // The panels (M4 4c-2), before the caption is placed on them.
                .add_systems(FixedUpdate, gui_view::gui_step.after(talk_view::talk_step))
                // Characters' shadows (M4 4d-7b-2), with this frame's poses.
                .add_systems(PostUpdate, (anim_view::shadow_draw, shadow_view::object_shadow_draw).after(bevy::transform::TransformSystems::Propagate))
                .add_systems(Update, (gui_view::fit_screen, gui_view::gui_layout_log, gui_view::gui_draw, gui_view::pointer_draw, talk_view::caption_draw, talk_view::caption_picture_draw, talk_view::mouth_draw).chain())
                // Sound effects (M3 3h-2), once the camera is placed.
                .add_systems(Update, sound_view::sound_step.after(game_camera::camera_draw))
                .add_systems(Update, (object_light::toggle_haze, object_light::sync_background, ground_haze::follow_haze, object_light::report_haze, draw_distance::update_activity, scene_lights::gather_lights, light_view::toggle_dynamic, light_view::paint_ground, wall_lights::light_walls).chain().after(game_camera::camera_draw));
        }
    }
    app.run();

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
    mut materials: ResMut<Assets<object_light::ObjectMaterial>>,
    mut images: ResMut<Assets<Image>>,
    light: Res<object_light::ObjectLight>,
) {
    let model = loaded.0.take().expect("model is loaded once");
    let centre: [f32; 3] = std::array::from_fn(|i| 0.5 * (model.min[i] + model.max[i]));
    let mut log = shared.log.lock().unwrap();
    let mut vertices = 0;
    let (mut outward, mut triangles) = (0, 0);
    // Corner light in the model's own orientation (M1 5b), for the log.
    let (mut lit_min, mut lit_max, mut lit_sum, mut lit_n) = (f32::MAX, f32::MIN, 0.0f32, 0usize);
    let (mut normals, mut normals_out) = (0, 0);
    for group in model.groups {
        for t in &group.triangles {
            // Check from the numbers: does the front face point away from
            // the model's centre? (Expected for most triangles of a
            // box-like model such as the chest.)
            let [a, b, c] = [0, 2, 1].map(|k| Vec3::from(to_bevy(t.positions[k], centre)));
            triangles += 1;
            outward += ((b - a).cross(c - a).dot((a + b + c) / 3.0) > 0.0) as usize;
            // The same check for the corner normals (M1 5b).
            for (n, p) in t.normals.iter().zip(t.positions) {
                normals += 1;
                normals_out += usize::from(Vec3::from(direction_to_bevy(*n)).dot(Vec3::from(to_bevy(p, centre))) > 0.0);
            }
            if group.shading.lighting == Lighting::Smooth {
                for n in t.normals {
                    let l = light.corner_light(Vec3::from(direction_to_bevy(n)))[0];
                    (lit_min, lit_max, lit_sum, lit_n) = (lit_min.min(l), lit_max.max(l), lit_sum + l, lit_n + 1);
                }
            }
        }
        vertices += group.triangles.len() * 3;
        let material = object_light::object_material(group.look, group.shading, &light, &mut images);
        commands.spawn((Mesh3d(meshes.add(model_mesh(&group.triangles, group.shading.lighting, centre))), MeshMaterial3d(materials.add(material))));
        log.detail(format!("spawned mesh for material {} ({} triangles)", group.material, group.triangles.len()));
    }
    log.summary(format!("corner normals pointing away from the model centre: {normals_out} of {normals}"));
    if lit_n > 0 {
        log.summary(format!(
            "corner light (red) over {lit_n} smooth corners, model unturned: min {lit_min:.3} mean {:.3} max {lit_max:.3}",
            lit_sum / lit_n as f32
        ));
    }
    log.summary(format!(
        "front faces pointing away from the model centre: {outward} of {triangles} triangles"
    ));
    log.summary(format!(
        "spawned: vertices={vertices} centre_moved_to_origin=({:.1}, {:.1}, {:.1}) game units",
        centre[0], centre[1], centre[2]
    ));
    // Colours as computed, no tone-mapping curve (M1 step 4b).
    // No tone mapping, so no dithering either (it would do nothing).
    commands.spawn((
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        bevy::core_pipeline::tonemapping::DebandDither::Disabled,
        // 4c-10: no smoothing of the panels' edges.
        bevy::ui_render::UiAntiAlias::Off,
        Transform::default(),
    ));
}

/// A direction (a normal) from game space to Bevy space, length kept.
fn direction_to_bevy(n: [f32; 3]) -> [f32; 3] {
    [n[0], n[2], -n[1]]
}

/// A mesh of model triangles for `object_light::ObjectMaterial` (M1
/// step 5b). The game's front faces list their corners clockwise; Bevy's
/// are counter-clockwise, so the corners go in reverse order
/// (models-and-textures.md, Q2); the axis change in `to_bevy` is a
/// rotation, so it does not change the order. Each corner carries its
/// normal; for flat lighting all three carry the first corner's
/// (object-lighting.md rule 7).
fn model_mesh(triangles: &[Triangle], lighting: Lighting, centre: [f32; 3]) -> Mesh {
    let mut positions = Vec::with_capacity(triangles.len() * 3);
    let mut uvs = Vec::with_capacity(triangles.len() * 3);
    let mut normals = Vec::with_capacity(triangles.len() * 3);
    let mut balls = Vec::with_capacity(triangles.len() * 3);
    for t in triangles {
        // The part's ball for point lights (object-lighting.md rule 24).
        let [x, y, z, radius] = t.part_ball;
        let c = to_bevy([x, y, z], centre);
        for k in [0, 2, 1] {
            positions.push(to_bevy(t.positions[k], centre));
            uvs.push(t.uvs[k]);
            let n = if lighting == Lighting::Flat { t.normals[0] } else { t.normals[k] };
            normals.push(direction_to_bevy(n));
            balls.push([c[0], c[1], c[2], radius]);
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(object_light::ATTRIBUTE_PART_BALL, balls)
}

/// A building's mesh for `building_light::BuildingMaterial` (M1 step 5c):
/// `model_mesh` plus each corner's place in the building's light picture,
/// in the same corner order.
fn building_mesh(group: &load::Group, corners: &building_light::WallCorners) -> Mesh {
    let light_uvs: Vec<[f32; 2]> = group.light_uvs.iter().flat_map(|uv| [0, 2, 1].map(|k| uv[k])).collect();
    model_mesh(&group.triangles, Lighting::None, [0.0; 3])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, light_uvs)
        .with_inserted_attribute(building_light::ATTRIBUTE_WALL_SURFACE, corners.attribute(&group.triangles))
}

/// A triangle list (corners already in Bevy's counter-clockwise order).
fn mesh_from(positions: Vec<[f32; 3]>, uvs: Vec<[f32; 2]>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.compute_flat_normals();
    mesh
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
    orbit: Option<Res<Orbit>>,
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
            "frames={} last_frame_ms={ms:.2} worst_frame_ms={:.2}{}",
            s.frames,
            s.worst_frame_ms,
            orbit.map_or(String::new(), |o| format!(
                " camera yaw={:.2} pitch={:.2} distance={:.2}",
                o.yaw, o.pitch, o.distance
            ))
        ));
    }
    if settings.frame_limit.is_some_and(|n| s.frames >= n) {
        commands.write_message(AppExit::Success);
    }
}
