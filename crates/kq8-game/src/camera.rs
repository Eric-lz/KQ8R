//! The third-person camera that follows Connor.
//!
//! Rules from the behaviour note `docs/behaviour/camera.md` ("What the camera
//! looks at", "Distance", "Height above the ground", "Swinging and zooming
//! with the mouse", "Tilt"). Runs once per game update (15 per second), so
//! the note's per-update rates are used as traced. Not here yet: the
//! obstacle check (M2 step 2d), the automatic swing behind Connor (Q27 c),
//! first person, shake and cutscene cameras.
//!
//! Stand-ins until Q27 is answered: what the tilt turns (the caller draws
//! the view looking at the aim point, pitched by `tilt - DEFAULT_TILT`),
//! and in fixed mode Page Up = closer. `mouse_forward` positive = zoom in;
//! the viewer sends mouse-down as positive (the user's check of the
//! original).

use crate::mover::GroundLookup;

/// The camera aims this far above the followed object's base.
pub const AIM_HEIGHT: f32 = 800.0;  // modified manually to what feels more natural, but could be wrong
/// The camera stays at least this far above the terrain under it and above
/// the followed object's base.
pub const MIN_HEIGHT: f32 = 1400.0;
/// Within this of its wanted height the camera jumps to it...
pub const HEIGHT_SNAP: f32 = 100.0;
/// ...otherwise it closes this share of the gap per update.
pub const HEIGHT_RATE: f32 = 0.5;   // modified manually to what feels more natural, but could be wrong
/// Swing per mouse unit (radians).
pub const SWING_PER_UNIT: f32 = 3.0 * std::f32::consts::FRAC_PI_4;
/// Zoom per mouse unit (normal mode).
pub const ZOOM_PER_UNIT: f32 = 5000.0;
/// Tilt per mouse unit (fixed mode, or normal mode with Shift).
pub const TILT_PER_UNIT: f32 = 0.1 * 7.5;
/// Tilt per update while Page Up/Down is held (0.1 × 3 × 0.05).
pub const TILT_PER_UPDATE: f32 = 0.015;
/// Fixed-mode zoom per update while Page Up/Down is held (0.05 × 2,500).
pub const FIXED_ZOOM_PER_UPDATE: f32 = 125.0;
/// Third-person tilt limits.
pub const TILT_LIMIT: f32 = 1.57;
pub const DEFAULT_TILT: f32 = 0.2;
/// Tilt snaps back once the followed object moved more than this
/// (|Δx| + |Δy|) in one update, in normal mode.
pub const MOVE_RESETS_TILT: f32 = 10.0;
/// Range before any script sets one.
pub const DEFAULT_RANGE: (f32, f32) = (1500.0, 5000.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Stays put unless outside the min/max range.
    Normal,
    /// Keeps this distance to the aim point (F11).
    Fixed { distance: f32 },
}

/// This update's input. Mouse values are in the game's mouse units.
#[derive(Debug, Clone, Copy, Default)]
pub struct Input {
    /// Right mouse button held (`MoveCamera`).
    pub mouse_held: bool,
    /// Sideways mouse movement since the last update.
    pub mouse_x: f32,
    /// Forward (positive) / back mouse movement since the last update.
    pub mouse_forward: f32,
    /// Shift held (`ModifyDn`).
    pub shift: bool,
    /// Page Up (+1) / Page Down (−1) held (`RotateX`).
    pub tilt_keys: i8,
    /// Home pressed (`ResetX`).
    pub reset_tilt: bool,
    /// F11 pressed (`CameraChangeMode`).
    pub change_mode: bool,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: [f32; 3],
    pub tilt: f32,
    pub mode: Mode,
    /// (min, max) distance, set by `KQCamera::setRange`.
    pub range: (f32, f32),
    last_base: Option<[f32; 3]>,
    mouse_was_held: bool,
}

/// What one update did, for the log.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub aim: [f32; 3],
    pub distance: f32,
    pub terrain_below: Option<f32>,
    pub events: Vec<String>,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// `aim + direction(from aim towards camera) × distance`.
fn at_distance(aim: [f32; 3], camera: [f32; 3], distance: f32) -> [f32; 3] {
    let d = sub(camera, aim);
    let len = length(d);
    let dir = if len < 1e-3 { [0.0, 1.0, 0.0] } else { d.map(|v| v / len) };
    std::array::from_fn(|i| aim[i] + dir[i] * distance)
}

impl Camera {
    pub fn new(position: [f32; 3]) -> Self {
        Camera { position, tilt: DEFAULT_TILT, mode: Mode::Normal, range: DEFAULT_RANGE, last_base: None, mouse_was_held: false }
    }

    /// `KQCamera::setRange <min> <max>`: refused unless min < max.
    pub fn set_range(&mut self, min: f32, max: f32) -> bool {
        if min < max {
            self.range = (min, max);
            true
        } else {
            false
        }
    }

    /// One update following an object whose base is at `base`.
    pub fn update(&mut self, base: [f32; 3], input: &Input, ground: &dyn GroundLookup) -> Report {
        let mut report = Report::default();
        let aim = [base[0], base[1], base[2] + AIM_HEIGHT];
        let (min, max) = self.range;

        if input.change_mode {
            self.mode = match self.mode {
                Mode::Normal => Mode::Fixed { distance: length(sub(self.position, aim)).clamp(min + 1.0, max - 1.0) },
                Mode::Fixed { .. } => Mode::Normal,
            };
            report.events.push(format!("camera mode -> {:?}", self.mode));
        }
        if input.reset_tilt {
            self.tilt = DEFAULT_TILT;
        }
        // Page Up/Down: tilt in normal mode, zoom in fixed mode.
        if input.tilt_keys != 0 {
            match &mut self.mode {
                Mode::Normal => self.tilt += f32::from(input.tilt_keys) * TILT_PER_UPDATE,
                Mode::Fixed { distance } => {
                    *distance = (*distance - f32::from(input.tilt_keys) * FIXED_ZOOM_PER_UPDATE).clamp(min + 1.0, max - 1.0)
                }
            }
        }

        // Mouse, only while the button is held; the first update after
        // pressing uses no movement.
        let (mx, mf) = if input.mouse_held && self.mouse_was_held { (input.mouse_x, input.mouse_forward) } else { (0.0, 0.0) };
        self.mouse_was_held = input.mouse_held;
        if mx != 0.0 {
            // Swing round the aim point about the vertical axis.
            let a = mx * SWING_PER_UNIT;
            let (s, c) = a.sin_cos();
            let [dx, dy, _] = sub(self.position, aim);
            self.position[0] = aim[0] + dx * c - dy * s;
            self.position[1] = aim[1] + dx * s + dy * c;
        }
        if mf != 0.0 {
            let zooms = matches!(self.mode, Mode::Normal) != input.shift;
            if !zooms {
                // Opposite sign to the zoom: mouse down (positive) tilts
                // the way Page Down does (checked by eye by the user
                // against the original, 2026-10-11).
                self.tilt -= mf * TILT_PER_UNIT;
            } else {
                match &mut self.mode {
                    Mode::Normal => {
                        let before = sub(self.position, aim);
                        let new = length(before) - mf * ZOOM_PER_UNIT;
                        // Never past the aim point: then the minimum, same side.
                        let new = if new <= 0.0 { min + 1.0 } else { new };
                        self.position = at_distance(aim, self.position, new);
                    }
                    Mode::Fixed { distance } => {
                        *distance = (*distance - mf * ZOOM_PER_UNIT).clamp(min + 1.0, max - 1.0);
                    }
                }
            }
        }

        // Distance.
        let d = length(sub(self.position, aim));
        match self.mode {
            Mode::Normal if d > max => self.position = at_distance(aim, self.position, max - 1.0),
            Mode::Normal if d < min => self.position = at_distance(aim, self.position, min + 1.0),
            Mode::Normal => {}
            Mode::Fixed { distance } => self.position = at_distance(aim, self.position, distance),
        }

        // Height: the higher of terrain under the camera + 1,400 and the
        // base + 1,400; jump within 100, else 10% of the gap.
        report.terrain_below = ground.ground(self.position[0], self.position[1]).map(|g| g.0);
        let wanted = report.terrain_below.map_or(base[2], |t| t.max(base[2])) + MIN_HEIGHT;
        let gap = wanted - self.position[2];
        self.position[2] = if gap.abs() <= HEIGHT_SNAP { wanted } else { self.position[2] + gap * HEIGHT_RATE };

        // Tilt snaps back once the followed object moves (normal mode).
        if let (Mode::Normal, Some(last)) = (self.mode, self.last_base) {
            let moved = (base[0] - last[0]).abs() + (base[1] - last[1]).abs();
            if moved > MOVE_RESETS_TILT && self.tilt != DEFAULT_TILT && mf == 0.0 {
                self.tilt = DEFAULT_TILT;
                report.events.push("tilt back to default (Connor moved)".into());
            }
        }
        self.last_base = Some(base);
        self.tilt = self.tilt.clamp(-TILT_LIMIT, TILT_LIMIT);
        report.aim = aim;
        report.distance = length(sub(self.position, aim));
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Flat(f32);
    impl GroundLookup for Flat {
        fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((self.0, [0.0, 0.0, 1.0]))
        }
    }

    fn horizontal(c: &Camera, aim: [f32; 3]) -> f32 {
        ((c.position[0] - aim[0]).powi(2) + (c.position[1] - aim[1]).powi(2)).sqrt()
    }

    #[test]
    fn pulled_in_on_a_leash_and_pushed_out() {
        let mut c = Camera::new([0.0, 9000.0, 1400.0]);
        let r = c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert!((r.distance - 4999.0).abs() < 1.0, "{}", r.distance);
        let mut c = Camera::new([0.0, 300.0, 1400.0]);
        let r = c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert!((r.distance - 1501.0).abs() < 1.0, "{}", r.distance);
        // Inside the range it stays where it is.
        let mut c = Camera::new([0.0, 3000.0, 1400.0]);
        c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert_eq!(c.position, [0.0, 3000.0, 1400.0]);
    }

    #[test]
    fn height_jumps_when_close_and_eases_when_far() {
        let mut c = Camera::new([0.0, 3000.0, 1350.0]);
        c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert_eq!(c.position[2], 1400.0);
        let mut c = Camera::new([0.0, 3000.0, 400.0]);
        c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert!((c.position[2] - 500.0).abs() < 1e-3, "{}", c.position[2]);
        // Terrain under the camera higher than Connor's base wins.
        let mut c = Camera::new([0.0, 3000.0, 5450.0]);
        c.update([0.0; 3], &Input::default(), &Flat(4000.0));
        assert_eq!(c.position[2], 5400.0);
    }

    #[test]
    fn swing_keeps_the_distance() {
        let mut c = Camera::new([0.0, 3000.0, 1400.0]);
        let held = Input { mouse_held: true, mouse_x: 1.0, ..Input::default() };
        c.update([0.0; 3], &held, &Flat(0.0)); // first update: no movement
        assert_eq!(c.position, [0.0, 3000.0, 1400.0]);
        c.update([0.0; 3], &held, &Flat(0.0));
        assert!((horizontal(&c, [0.0; 3]) - 3000.0).abs() < 0.5);
        assert!(c.position[0].abs() > 1000.0, "{:?}", c.position);
    }

    #[test]
    fn zoom_stays_in_range_and_never_passes_the_aim() {
        let mut c = Camera::new([0.0, 3000.0, 1400.0]);
        let zoom_in = Input { mouse_held: true, mouse_forward: 10.0, ..Input::default() };
        c.update([0.0; 3], &zoom_in, &Flat(0.0));
        let r = c.update([0.0; 3], &zoom_in, &Flat(0.0));
        assert!((r.distance - 1501.0).abs() < 1.0, "{}", r.distance);
        assert!(c.position[1] > 0.0, "same side: {:?}", c.position);
    }

    #[test]
    fn fixed_mode_keeps_its_distance() {
        let mut c = Camera::new([0.0, 3000.0, 1400.0]);
        c.update([0.0; 3], &Input { change_mode: true, ..Input::default() }, &Flat(0.0));
        assert_eq!(c.mode, Mode::Fixed { distance: 3000.0 });
        let r = c.update([2000.0, 0.0, 0.0], &Input::default(), &Flat(0.0));
        assert!((r.distance - 3000.0).abs() < 1.0, "{}", r.distance);
        // Page Up in fixed mode: 125 closer.
        c.update([2000.0, 0.0, 0.0], &Input { tilt_keys: 1, ..Input::default() }, &Flat(0.0));
        assert_eq!(c.mode, Mode::Fixed { distance: 2875.0 });
    }

    #[test]
    fn tilt_keys_and_reset() {
        let mut c = Camera::new([0.0, 3000.0, 1400.0]);
        for _ in 0..15 {
            c.update([0.0; 3], &Input { tilt_keys: 1, ..Input::default() }, &Flat(0.0));
        }
        assert!((c.tilt - (DEFAULT_TILT + 0.225)).abs() < 1e-4, "{}", c.tilt);
        // Connor moves 75 (a walking update): back to default.
        c.update([75.0, 0.0, 0.0], &Input::default(), &Flat(0.0));
        assert_eq!(c.tilt, DEFAULT_TILT);
        c.tilt = 1.0;
        c.update([75.0, 0.0, 0.0], &Input { reset_tilt: true, ..Input::default() }, &Flat(0.0));
        assert_eq!(c.tilt, DEFAULT_TILT);
    }

    #[test]
    fn set_range_refuses_min_not_below_max() {
        let mut c = Camera::new([0.0; 3]);
        assert!(!c.set_range(2500.0, 1000.0));
        assert_eq!(c.range, DEFAULT_RANGE);
        assert!(c.set_range(1000.0, 2500.0));
    }
}
