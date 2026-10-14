//! Settings of the game's small effect objects, read from their `.kq`
//! sections: emitters (`KQEmitter`, falling leaves, drips), particle
//! catchers (`KQParticleCatcher`) and ground animators
//! (`KQTerrainAnimator`, flowing water).
//!
//! Key names and defaults from the behaviour note
//! `docs/behaviour/emitters.md` ("Emitter keys"); the rules that use them
//! are in `kq8-core` (`emitter.rs`). Vectors are three keys `<name>X`,
//! `<name>Y`, `<name>Z`; a missing key is 0.

use crate::kq_text::Section;

/// An emitter's settings (`classType=KQEmitter`). The position (`loc`) and
/// `translucency` are read like any object's (`objects::Placement`).
#[derive(Debug, Clone, PartialEq)]
pub struct EmitterSettings {
    /// `shape`: the particle model (`.kqs`).
    pub shape: Option<String>,
    /// `radius`: particles start anywhere in a box of ± this around `loc`.
    pub radius: [f32; 3],
    /// `forceMin` / `forceMax`: each particle's velocity, units per tick.
    pub force_min: [f32; 3],
    pub force_max: [f32; 3],
    /// `gravity`: units per tick², added as age × gravity every tick.
    pub gravity: [f32; 3],
    /// `rotMin` / `rotMax`: each particle's spin, radians per tick.
    pub rot_min: [f32; 3],
    pub rot_max: [f32; 3],
    /// `maxSinPeriod`: swaying, swings per life, per axis.
    pub max_sin_period: [f32; 3],
    /// `maxLife`: particle life in ticks (default 100).
    pub max_life: u32,
    /// `maxParticles`: number of particles (default 200).
    pub max_particles: u32,
    /// `delayStart`: non-zero = particles appear one by one.
    pub delay_start: bool,
    /// `follow`: name of the object (or `KQCamera`) the emitter moves with.
    pub follow: Option<String>,
    /// `followMesh` set (any value but 0).
    pub follow_mesh: bool,
    pub follow_offset: [f32; 3],
    /// Values that could not be read (the default was used instead).
    pub problems: Vec<String>,
}

/// Default `maxLife` in ticks (emitters.md "Numbers and units").
pub const DEFAULT_MAX_LIFE: u32 = 100;
/// Default `maxParticles`.
pub const DEFAULT_MAX_PARTICLES: u32 = 200;

impl EmitterSettings {
    pub fn from_section(s: &Section) -> Self {
        let mut problems = Vec::new();
        let mut num = |k: &str| match s.number(k) {
            None => None,
            Some(Ok(v)) => Some(v),
            Some(Err(e)) => {
                problems.push(format!("{e}; 0 or the default used"));
                None
            }
        };
        let mut vector = |name: &str| ["X", "Y", "Z"].map(|a| num(&format!("{name}{a}")).unwrap_or(0.0) as f32);
        let radius = vector("radius");
        let force_min = vector("forceMin");
        let force_max = vector("forceMax");
        let gravity = vector("gravity");
        let rot_min = vector("rotMin");
        let rot_max = vector("rotMax");
        let max_sin_period = vector("maxSinPeriod");
        let follow_offset = vector("followOffset");
        // Whole numbers; a negative value counts as 0.
        let count = |v: Option<f64>, default: u32| v.map_or(default, |v| v.max(0.0) as u32);
        let max_life = count(num("maxLife"), DEFAULT_MAX_LIFE);
        let max_particles = count(num("maxParticles"), DEFAULT_MAX_PARTICLES);
        let delay_start = num("delayStart").is_some_and(|v| v != 0.0);
        let follow_mesh = s.get("followMesh").is_some_and(|v| v.trim() != "0");
        let text = |k: &str| s.get(k).map(str::trim).filter(|v| !v.is_empty()).map(str::to_string);
        EmitterSettings {
            shape: text("shape"),
            radius,
            force_min,
            force_max,
            gravity,
            rot_min,
            rot_max,
            max_sin_period,
            max_life,
            max_particles,
            delay_start,
            follow: text("follow"),
            follow_mesh,
            follow_offset,
            problems,
        }
    }
}

/// A particle catcher's settings (`classType=KQParticleCatcher`,
/// emitters.md "Particle catchers" rule 1).
#[derive(Debug, Clone, PartialEq)]
pub struct CatcherSettings {
    /// `emitter`: the emitter whose particles it hides.
    pub emitter: Option<String>,
    /// `interior`: the building whose box hides them.
    pub interior: Option<String>,
}

impl CatcherSettings {
    pub fn from_section(s: &Section) -> Self {
        let text = |k: &str| s.get(k).map(str::trim).filter(|v| !v.is_empty()).map(str::to_string);
        CatcherSettings { emitter: text("emitter"), interior: text("interior") }
    }
}

/// A ground animator's settings (`classType=KQTerrainAnimator`,
/// emitters.md "Ground animator keys"). Square coordinates are (column,
/// row from the bottom), as in `terrain.md`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimatorSettings {
    /// `origTexture0…`: animate every square with one of these materials
    /// (not read when `grid_blocks` is used).
    pub orig_textures: Vec<i64>,
    /// `excludeX0`/`excludeY0`…: squares to leave out.
    pub exclude: Vec<(i64, i64)>,
    /// `gridBlockX0`/`gridBlockY0`…: an explicit list of squares; `Some`
    /// when `numGridBlocks` is not 0.
    pub grid_blocks: Option<Vec<(i64, i64)>>,
    /// `materialIndex0…`: the cycle of materials.
    pub cycle: Vec<i64>,
    /// `materialIndexII0…`: the second cycle (only for the unused
    /// `terAnimSwitch` command).
    pub cycle2: Vec<i64>,
    /// `delay`: seconds between changes.
    pub delay: f64,
    /// `repeatflag=1`: loop the cycle.
    pub repeat: bool,
    /// `startNow=1`: start when loaded.
    pub start_now: bool,
    /// `frameCount`: saved-game state (0 = start).
    pub frame_count: i64,
    /// Entries or values that could not be read.
    pub problems: Vec<String>,
}

impl AnimatorSettings {
    pub fn from_section(s: &Section) -> Self {
        let mut problems = Vec::new();
        let num = |k: &str, problems: &mut Vec<String>| match s.number(k) {
            None => None,
            Some(Ok(v)) => Some(v),
            Some(Err(e)) => {
                problems.push(e);
                None
            }
        };
        // The first `count` entries `<key>0…` (`count` from its own key);
        // each entry is one or two numbers.
        let list = |count: &str, keys: &[&str], problems: &mut Vec<String>| -> Vec<Vec<i64>> {
            let n = num(count, problems).map_or(0, |v| v.max(0.0) as usize);
            let mut out = Vec::new();
            for k in 0..n {
                let names: Vec<String> = keys.iter().map(|key| format!("{key}{k}")).collect();
                let values: Vec<Option<f64>> = names.iter().map(|name| num(name, problems)).collect();
                if values.iter().all(Option::is_some) {
                    out.push(values.into_iter().map(|v| v.unwrap() as i64).collect());
                } else {
                    problems.push(format!("{count}={n} but {} is missing; entry skipped", names.join("/")));
                }
            }
            out
        };
        let pairs = |v: Vec<Vec<i64>>| v.into_iter().map(|p| (p[0], p[1])).collect::<Vec<_>>();
        let singles = |v: Vec<Vec<i64>>| v.into_iter().map(|p| p[0]).collect::<Vec<_>>();
        // With numGridBlocks not 0, origTexture and exclude are not read at all.
        let (grid_blocks, orig_textures, exclude) = if num("numGridBlocks", &mut problems).unwrap_or(0.0) != 0.0 {
            (Some(pairs(list("numGridBlocks", &["gridBlockX", "gridBlockY"], &mut problems))), Vec::new(), Vec::new())
        } else {
            (
                None,
                singles(list("numOrigTextures", &["origTexture"], &mut problems)),
                pairs(list("numExclude", &["excludeX", "excludeY"], &mut problems)),
            )
        };
        let cycle = singles(list("numMaterialIndex", &["materialIndex"], &mut problems));
        let cycle2 = singles(list("numMaterialIndexII", &["materialIndexII"], &mut problems));
        let delay = num("delay", &mut problems).unwrap_or(0.0);
        let repeat = num("repeatflag", &mut problems).is_some_and(|v| v != 0.0);
        let start_now = num("startNow", &mut problems).is_some_and(|v| v != 0.0);
        let frame_count = num("frameCount", &mut problems).map_or(0, |v| v as i64);
        AnimatorSettings { orig_textures, exclude, grid_blocks, cycle, cycle2, delay, repeat, start_now, frame_count, problems }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kq_text::parse;

    #[test]
    fn emitter_keys_and_defaults() {
        // Made-up section in the files' layout (not a copy of a game file).
        let s = &parse(b"[e]\nclassType=KQEmitter\nshape=bit.kqs\nradiusX=5\nforceMinZ=-2\nforceMaxZ=4\ngravityZ=-1\nrotMaxY=0.2\nmaxSinPeriodX=2\n;;maxSinPeriodY=9\ndelayStart=1\n")[0];
        let e = EmitterSettings::from_section(s);
        assert_eq!(e.shape.as_deref(), Some("bit.kqs"));
        assert_eq!(e.radius, [5.0, 0.0, 0.0]);
        assert_eq!((e.force_min[2], e.force_max[2]), (-2.0, 4.0));
        assert_eq!(e.gravity, [0.0, 0.0, -1.0]);
        assert_eq!(e.rot_max, [0.0, 0.2, 0.0]);
        assert_eq!(e.max_sin_period, [2.0, 0.0, 0.0]);
        assert_eq!((e.max_life, e.max_particles), (DEFAULT_MAX_LIFE, DEFAULT_MAX_PARTICLES));
        assert!(e.delay_start);
        assert!(e.follow.is_none() && !e.follow_mesh);
        assert!(e.problems.is_empty());
    }

    #[test]
    fn catcher_keys() {
        let s = &parse(b"[c]
classType=KQParticleCatcher
emitter=e1
interior=House
")[0];
        let c = CatcherSettings::from_section(s);
        assert_eq!((c.emitter.as_deref(), c.interior.as_deref()), (Some("e1"), Some("House")));
    }

    #[test]
    fn animator_keys() {
        let s = &parse(b"[a]\nclassType=KQTerrainAnimator\nrepeatflag=1\nstartNow=1\ndelay=0.3\nnumMaterialIndex=3\nmaterialIndex0=5\nmaterialIndex1=6\nmaterialIndex2=7\nmaterialIndex3=8\nnumOrigTextures=1\norigTexture0=5\nnumExclude=2\nexcludeX0=1\nexcludeY0=2\nexcludeX1=3\n")[0];
        let a = AnimatorSettings::from_section(s);
        // Only the first numMaterialIndex entries.
        assert_eq!(a.cycle, vec![5, 6, 7]);
        assert_eq!(a.orig_textures, vec![5]);
        assert_eq!(a.exclude, vec![(1, 2)]);
        assert_eq!(a.problems.len(), 1, "{:?}", a.problems);
        assert!(a.grid_blocks.is_none() && a.repeat && a.start_now && a.delay == 0.3 && a.frame_count == 0);
    }

    #[test]
    fn grid_blocks_replace_textures_and_excludes() {
        let s = &parse(b"[a]\nnumOrigTextures=1\norigTexture0=5\nnumExclude=1\nexcludeX0=1\nexcludeY0=1\nnumGridBlocks=1\ngridBlockX0=4\ngridBlockY0=9\ngridBlockX1=4\ngridBlockY1=10\n#numMaterialIndex=9\n")[0];
        let a = AnimatorSettings::from_section(s);
        assert_eq!(a.grid_blocks, Some(vec![(4, 9)]));
        assert!(a.orig_textures.is_empty() && a.exclude.is_empty() && a.cycle.is_empty());
    }

    #[test]
    fn unreadable_numbers_are_reported() {
        let s = &parse(b"[e]\nmaxLife=long\nmaxParticles=3\n")[0];
        let e = EmitterSettings::from_section(s);
        assert_eq!((e.max_life, e.max_particles), (DEFAULT_MAX_LIFE, 3));
        assert_eq!(e.problems.len(), 1);
    }
}
