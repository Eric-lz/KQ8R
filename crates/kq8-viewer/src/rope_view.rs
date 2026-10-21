//! The rope state in the viewer (M6 6a-3): runs `kq8_core::rope::Climb`
//! on Connor, the camera and the separate rope model, each update after
//! his own update (`ropeandhook.md` rules 10–21).
//!
//! What it does to the game, by the note's rules: his movements (Rope
//! mode), gravity and step heights (his mover), placing and lifting him,
//! his turn to the rope's place (the `walkTo` turn), the camera (stays,
//! spots, looks at the aim point), handsOff with the pointer kept, empty
//! hands, the hook icons hidden while he hangs. The rope model (`rope`,
//! `Conrope.anm`) is drawn 2,000 behind him by `connor_view::connor_draw`;
//! its cues' sounds play from here. Every step goes to `rope.log` (and
//! `connor.log`).

use bevy::prelude::*;

use kq8_core::active::WorldState;
use kq8_core::collide::{BoxCollider, BuildingCollider};
use kq8_core::rope::{Anim, Climb, RopeHost, Stage};
use kq8_core::weapons::WeaponMode;

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::game_camera::GameCam;
use crate::terrain_view::{Buildings, GroundLookup, ObjectColliders};
use crate::Shared;

/// The game as the rope state sees it.
struct Host<'a> {
    c: &'a mut ConnorSim,
    cam: Option<&'a mut GameCam>,
    w: &'a mut WorldState,
    terrain: Option<&'a kq8_data::Terrain>,
    buildings: &'a [BuildingCollider],
    objects: &'a [BoxCollider],
    /// He was put somewhere at once this update (for the traps).
    placed: bool,
}

impl RopeHost for Host<'_> {
    fn note(&mut self, line: String) {
        self.c.log.detail(format!("connor update {}: {line}", self.c.updates));
        self.w.rope_note(line);
    }

    fn play(&mut self, anim: Anim) -> bool {
        match self.c.rope_movement(anim) {
            Some(name) => {
                self.c.walker.rope_play(name);
                true
            }
            None => false,
        }
    }

    fn connor_at(&self) -> [f32; 3] {
        self.c.mover.position
    }

    fn heading(&self) -> f32 {
        self.c.walker.heading
    }

    fn set_gravity(&mut self, on: bool) {
        self.c.mover.use_gravity = on;
    }

    fn step_heights(&self) -> (f32, f32) {
        (self.c.mover.step_up, self.c.mover.step_down)
    }

    fn set_step_heights(&mut self, up: f32, down: f32) {
        self.c.mover.step_up = up;
        self.c.mover.step_down = down;
    }

    fn place(&mut self, position: [f32; 3], heading: f32) {
        self.placed |= position != self.c.mover.position;
        self.c.put(position, heading);
    }

    fn turn_to(&mut self, point: [f32; 2]) {
        use kq8_core::connor_auto::{AutoKind, AutoMove};
        let line = self.c.walker.start_auto(AutoMove::new(AutoKind::Point, point, None));
        self.c.log.detail(format!("connor update {}: rope state: {line}", self.c.updates));
    }

    fn camera_at(&self) -> [f32; 3] {
        self.cam.as_ref().map_or(self.c.mover.position, |c| c.position())
    }

    fn camera_stay(&mut self) {
        let at = self.c.mover.position;
        if let Some(c) = self.cam.as_mut() {
            let line = c.follow(false, at);
            self.w.rope_note(format!("camera: {line}"));
        }
    }

    fn camera_to(&mut self, spot: [f32; 3], aim: [f32; 3]) {
        if let Some(c) = self.cam.as_mut() {
            let line = c.look_from(spot, aim);
            self.w.rope_note(format!("camera: {line}"));
        }
    }

    fn solid_between(&self, from: [f32; 3], to: [f32; 3]) -> Option<(f32, String)> {
        let d = |p: [f32; 3]| (0..3).map(|k| (p[k] - from[k]).powi(2)).sum::<f32>().sqrt();
        let mut hits: Vec<([f32; 3], String)> = Vec::new();
        if let Some(t) = self.terrain {
            hits.extend(crate::game_camera::surface_line(t)(from, to).map(|p| (p, "the terrain".to_string())));
        }
        hits.extend(self.buildings.iter().filter_map(|b| Some((b.line_hit(from, to)?, format!("building {}", b.name)))));
        hits.extend(self.objects.iter().filter_map(|o| Some((o.line_hit(from, to)?, format!("object {}", o.name)))));
        hits.into_iter().min_by(|a, b| d(a.0).total_cmp(&d(b.0))).map(|(p, what)| (p[2], what))
    }

    fn start_rope_model(&mut self) -> bool {
        match self.c.rope_movement(Anim::Rope("rope")) {
            Some(name) => {
                self.c.rope_model = Some((name, 0.0, 0.0));
                true
            }
            None => false,
        }
    }

    fn hands_off(&mut self, on: bool) {
        self.c.hands_off = on;
    }

    fn hang(&mut self, phase: f32) {
        match self.c.rope_movement(Anim::Rope("climb")) {
            Some(name) => self.c.walker.rope_hang_at(name, phase),
            None => self.w.rope_note("rope state: climb (Conrope.anm) is not loaded: he can't hang"),
        }
    }

    fn empty_hands(&mut self) {
        self.w.set_weapon(WeaponMode::None, None);
        let line = self.c.ready_set(None, WeaponMode::None, None, true, false, false);
        self.c.log.detail(format!("connor update {}: rope state: empty hands: {line}", self.c.updates));
    }

    fn on_rope(&mut self, on: bool) {
        self.w.connor_on_rope = on;
    }

    fn climbed_to_top(&self) -> bool {
        self.c.walker.climb_dir > 0
    }

    fn camera_follow(&mut self) {
        let at = self.c.mover.position;
        if let Some(c) = self.cam.as_mut() {
            if !c.following {
                let line = c.follow(true, at);
                self.w.rope_note(format!("camera: {line}"));
            }
        }
    }

    fn restore_hand(&mut self, mode: WeaponMode, weapon: Option<String>) -> bool {
        if mode == WeaponMode::None {
            return false;
        }
        self.w.set_weapon(mode, weapon.clone());
        let list = crate::door_view::weapon_list(self.w);
        let line = self.c.ready_set(list, mode, weapon, true, false, false);
        self.c.log.detail(format!("connor update {}: rope state: the weapon back: {line}", self.c.updates));
        true
    }

    fn leave(&mut self) {
        self.c.walker.rope_leave();
        let lock = kq8_core::connor_inventory::rope_lock(self.w, false, "the rope state is over");
        if !lock.is_empty() {
            self.w.rope_note(format!("rope state{lock} (rule 26)"));
        }
    }
}

/// `ropeandhook.md` rule 22 (M6 6a-4): while he hangs, the camera's
/// height and the aim point follow the climb's direction this update; the
/// camera keeps its x and y and looks at the aim point. Logged every
/// update he climbs, and when he stops.
fn climb_step(cl: &mut Climb, host: &mut Host) {
    let dir = host.c.walker.climb_dir;
    let (forward, turn) = (host.c.intent.forward, host.c.intent.turn);
    let frame = host.c.loop_of(host.c.walker.movement).map_or(0, |l| l.frame_at(host.c.walker.phase));
    if dir == 0 {
        if host.c.rope_was_climbing {
            host.c.rope_was_climbing = false;
            host.note(format!("climb: frozen at frame {frame} (phase {:.3}): {} (rule 22)", host.c.walker.phase, if forward == 0 && turn == 0 { "no key" } else { "no forward or back key" }));
        }
        return;
    }
    host.c.rope_was_climbing = true;
    let dt = (1.0 / kq8_core::UPDATES_PER_SECOND) as f32;
    let (camera, aim) = kq8_core::rope::climb_camera(host.camera_at(), cl.aim, cl.range, dir, dt);
    cl.aim = aim;
    if let Some(c) = host.cam.as_mut() {
        c.look_from(camera, aim);
    }
    host.w.rope_note(format!(
        "climb: {} held: {}, frame {frame} (phase {:.3}); camera z {:.0} (range {:.0}..{:.0}), aim z {:.0} (rule 22)",
        if forward > 0 { "forward" } else { "back" },
        if dir > 0 { "up (forwards)" } else { "down (backwards)" },
        host.c.walker.phase,
        camera[2],
        cl.range.0,
        cl.range.1,
        aim[2]
    ));
}

/// One update of the rope state, after Connor's own update.
#[allow(clippy::too_many_arguments)]
pub fn rope_step(
    connor: Option<ResMut<ConnorSim>>,
    camera: Option<ResMut<GameCam>>,
    sim: Option<ResMut<DoorSim>>,
    lookup: Option<Res<GroundLookup>>,
    buildings: Option<Res<Buildings>>,
    objects: Option<Res<ObjectColliders>>,
    shared: Res<Shared>,
) {
    let (Some(mut connor), Some(mut sim)) = (connor, sim) else { return };
    let (c, d) = (&mut *connor, &mut *sim);
    let mut camera = camera;
    let asked = d.rope_asked.take();
    // The separate rope model's own update (rule 15): its phase, its
    // sounds, its end.
    let mut model_ended = false;
    if let Some((name, _, phase)) = c.rope_model {
        let (share, cues) = match c.loop_of(name) {
            Some(l) if l.play_time() > 0.0 => {
                let share = (1.0 / kq8_core::UPDATES_PER_SECOND) as f32 * l.speed.abs() / l.play_time();
                let to = (phase + share).min(1.0);
                (share, l.cues_between(phase, to).into_iter().map(str::to_string).collect::<Vec<_>>())
            }
            _ => (1.0, Vec::new()),
        };
        let to = (phase + share).min(1.0);
        for cue in cues {
            let mut words = cue.split_whitespace();
            if words.next().is_some_and(|a| a.eq_ignore_ascii_case("playsound")) {
                if let Some(file) = words.next() {
                    let at = c.mover.position;
                    c.sounds.push(kq8_core::sound::Start::placed_one_shot(file, at, "the rope model's cue".into()));
                }
            }
            d.active.world.rope_note(format!("the rope model's cue '{cue}' fires (phase {phase:.3} -> {to:.3})"));
        }
        if to >= 1.0 {
            c.rope_model = None;
            model_ended = true;
        } else {
            c.rope_model = Some((name, phase, to));
        }
    }
    let ended = std::mem::take(&mut c.rope_ended);
    let mut climb = c.rope.take();
    let drawn_before = (c.walker.movement, c.walker.phase, c.walker.mode);
    let placed = {
        let mut host = Host {
            c: &mut *c,
            cam: camera.as_deref_mut(),
            w: &mut d.active.world,
            terrain: lookup.as_ref().map(|l| &l.0),
            buildings: buildings.as_ref().map_or(&[][..], |b| &b.0[..]),
            objects: objects.as_ref().map_or(&[][..], |o| &o.0[..]),
            placed: false,
        };
        if let Some(how) = asked {
            if climb.is_some() {
                host.note(format!("rope state asked ({how}): already in it; nothing"));
            } else if !host.c.state_request("rope state", &how) {
                host.note(format!("rope state asked ({how}): refused, his jump is not finished (connor-physics.md); the rope stays out of the pointer"));
            } else {
                // Rules 12-13: the rope's first-person lock released and set
                // again at once (no flicker: "Do not reproduce").
                let lock = kq8_core::connor_inventory::rope_lock(host.w, true, "the rope state");
                if !lock.is_empty() {
                    host.note(format!("rope state{lock} (rule 13)"));
                }
                let (rope, mode, weapon) = (host.w.rope.clone(), host.w.weapon_mode, host.w.weapon_ready.clone());
                climb = Climb::start(&rope, mode, weapon, &mut host);
            }
        }
        if let Some(cl) = climb.as_mut() {
            if cl.stage == Stage::Hanging {
                climb_step(cl, &mut host);
            }
            for _ in ended {
                // His own `stop` while the rope model plays never ends
                // (a loop); anything else that ends is the stage's movement.
                if cl.stage == Stage::RopeModel {
                    continue;
                }
                cl.ended(&mut host);
            }
            if model_ended && cl.stage == Stage::RopeModel {
                cl.ended(&mut host);
            }
        }
        host.placed
    };
    // M6 6a fix (the user's check: snaps): a movement started here is
    // drawn from its own start at once.
    if (c.walker.movement, c.walker.phase, c.walker.mode) != drawn_before {
        let was = c.drawn_phase();
        c.redraw_from_walker();
        let line = format!("drawn: {} from phase {:.3} (was drawing phase {was:.3}: the old movement's end)", c.walker.movement, c.walker.phase);
        c.log.detail(format!("connor update {}: {line}", c.updates));
        d.active.world.rope_note(line);
    }
    if placed {
        let p = c.mover.position;
        let name = c.placement.name.clone();
        let calls = d.connor_placed_by_rope(&name, p, &c.mover.balls, &shared);
        let calls = if calls.is_empty() { "no trap call".to_string() } else { calls.join("; ") };
        d.active.world.rope_note(format!("placed at ({:.0}, {:.0}, {:.0}): the traps' stay test, then the enter test there (stand-in, Q147 e): {calls}", p[0], p[1], p[2]));
    }
    c.rope = climb.filter(|cl| !cl.done);
    d.active.world.connor_roping = c.rope.is_some();
    d.flush_rope(&shared);
}
