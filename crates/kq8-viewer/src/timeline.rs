//! Where launch and loading time goes (the user's question, 2026-10-23:
//! 6.3 s by stopwatch from `cargo play` to ready, 3.5 s in the log).
//!
//! One clock from the program's start: [`mark`] writes "timeline: +N ms
//! what" lines to `viewer.log`. Running totals ([`Counter`]) add up the
//! time spent in the loader's busy parts while a world is read; nested
//! parts are counted in each (an animation's archive unpacking is in both
//! "unpacking" and "animations").

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

use crate::log::Log;

static START: OnceLock<Instant> = OnceLock::new();

/// Starts the clock (first thing in `main`).
pub fn start() {
    START.get_or_init(Instant::now);
}

/// Milliseconds since the program started.
pub fn ms() -> u128 {
    START.get_or_init(Instant::now).elapsed().as_millis()
}

/// A "timeline: +N ms what" line.
pub fn mark(log: &mut Log, what: &str) {
    log.summary(format!("timeline: +{} ms {what}", ms()));
}

/// The loader's busy parts.
#[derive(Clone, Copy)]
pub enum Counter {
    /// Entries unpacked from the archives.
    Unpack,
    /// Other worlds' archives opened (a texture not in this world's).
    OtherWorlds,
    /// Model files parsed.
    Parse,
    /// Animations read (`load_animation_skinned`, all of it).
    Animation,
    /// Still models read (`load_model`, all of it).
    Model,
    /// Textures read (`load_texture`, all of it).
    Texture,
}

const NAMES: [&str; 6] = ["archive unpacking", "other worlds' archives opened", "model files parsed", "animations", "still models", "textures"];
static NS: [AtomicU64; 6] = [const { AtomicU64::new(0) }; 6];
static COUNT: [AtomicU64; 6] = [const { AtomicU64::new(0) }; 6];

/// Adds the time since `since` to `c`.
pub fn add(c: Counter, since: Instant) {
    NS[c as usize].fetch_add(since.elapsed().as_nanos() as u64, Ordering::Relaxed);
    COUNT[c as usize].fetch_add(1, Ordering::Relaxed);
}

/// Names seen per counter (how many different files were behind it).
static SEEN: std::sync::Mutex<[Option<std::collections::HashSet<String>>; 6]> = std::sync::Mutex::new([None, None, None, None, None, None]);

/// Notes that `c` was for `name` (for the "different" count).
pub fn seen(c: Counter, name: &str) {
    if let Ok(mut s) = SEEN.lock() {
        s[c as usize].get_or_insert_with(Default::default).insert(name.to_ascii_lowercase());
    }
}

/// Back to zero (before a world is read).
pub fn reset() {
    for k in 0..NAMES.len() {
        NS[k].store(0, Ordering::Relaxed);
        COUNT[k].store(0, Ordering::Relaxed);
    }
    if let Ok(mut s) = SEEN.lock() {
        *s = Default::default();
    }
}

/// "archive unpacking 812 ms (4,321), ..." since the last reset.
pub fn counters_line() -> String {
    (0..NAMES.len())
        .map(|k| {
            let different = SEEN.lock().ok().and_then(|s| s[k].as_ref().map(|h| format!(", {} different", h.len()))).unwrap_or_default();
            format!("{} {} ms ({}{different})", NAMES[k], NS[k].load(Ordering::Relaxed) / 1_000_000, COUNT[k].load(Ordering::Relaxed))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Adds the time it lives to its counter (for functions with many exits).
pub struct Timer(Counter, Instant);

impl Timer {
    pub fn new(c: Counter) -> Self {
        Timer(c, Instant::now())
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        add(self.0, self.1);
    }
}

/// How many render pipelines (shaders for a kind of material and mesh)
/// are still being prepared, written by the render world each frame.
#[derive(bevy::prelude::Resource, Clone, Default)]
struct Waiting(std::sync::Arc<std::sync::atomic::AtomicUsize>);

/// The launch marks after the world is read: the app's first systems, the
/// window, the first frame (in `frame_stats`), and each time no pipeline
/// is waiting any more after some were (the first: everything on screen
/// is drawn as it should be).
pub fn add_marks(app: &mut bevy::prelude::App) {
    use bevy::prelude::*;
    let waiting = Waiting::default();
    app.insert_resource(waiting.clone())
        .add_systems(Startup, |shared: Res<crate::Shared>| mark(&mut shared.log.lock().unwrap(), "the app's first systems run (Startup)"))
        .add_systems(PreUpdate, |mut created: MessageReader<bevy::window::WindowCreated>, shared: Res<crate::Shared>| {
            for _ in created.read() {
                mark(&mut shared.log.lock().unwrap(), "the window is made");
            }
        })
        .add_systems(Last, pipelines_mark);
    if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
        render.insert_resource(waiting).add_systems(bevy::render::Render, |cache: Res<bevy::render::render_resource::PipelineCache>, waiting: Res<Waiting>| {
            waiting.0.store(cache.waiting_pipelines().count(), Ordering::Relaxed);
        });
    }
}

/// Logs when pipelines start being prepared and when none is waiting any
/// more (the first few times).
fn pipelines_mark(waiting: bevy::prelude::Res<Waiting>, shared: bevy::prelude::Res<crate::Shared>, mut seen: bevy::prelude::Local<(usize, u32)>) {
    let now = waiting.0.load(Ordering::Relaxed);
    let (was, times) = *seen;
    if times < 6 && (was == 0) != (now == 0) {
        let what = if now > 0 { format!("{now} pipelines (shaders) being prepared") } else { "no pipeline waiting: everything shown is drawn as it should be".to_string() };
        mark(&mut shared.log.lock().unwrap(), &what);
        seen.1 += 1;
    }
    seen.0 = now;
}
