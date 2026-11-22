//! Swamp's Mandragore (`KQMandragorTree` and `KQMandragorRoot`), Batch 3 step B3-4.
//! Rules from `docs/behaviour/fighting.md` section 7b (Session A, traced).

/// `KQMandragorRoot` collider mask restored when `Birth` finishes (`fighting.md` 7b).
pub const ROOT_RESURFACE_MASK: u32 = 0x201f;

/// Connor's stagger movement when knocked back by a Mandragore root (`fighting.md` 7b).
pub const REPEL_MOVEMENT: &str = "reactback";

/// Per-instance state for `KQMandragorTree` (`fighting.md` 7b).
#[derive(Debug, Clone)]
pub struct MandragorTree {
    pub root_down_count: i32,
    pub max_roots: i32,
}

impl MandragorTree {
    pub fn new(root_down_count: i32, max_roots: i32) -> Self {
        Self { root_down_count, max_roots }
    }
}

/// Lifecycle phase of a `KQMandragorRoot` (`fighting.md` 7b).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RootPhase {
    Alive,
    /// Sinking on `r_sink.kqs` while its parent tree is still alive; transitions to `Underground`.
    Sinking,
    /// Hidden underground until `wake_at` (game seconds).
    Underground { wake_at: f64 },
    /// Rising on `r_appr.kqs` (`Birth`); transitions to `Alive` or permanent death.
    Rebirthing,
    /// Parent tree died: plays `r_sink.kqs` cleanly to permanent removal (0 XP).
    PermanentDie,
}

/// Per-instance state for `KQMandragorRoot` (`fighting.md` 7b).
#[derive(Debug, Clone)]
pub struct MandragorRoot {
    pub offset: [f32; 3],
    pub rebirth_delay: f32,
    pub too_close: f32,
    pub repel_dist: f32,
    pub orig_health: i64,
    pub orig_keep_on_ground: bool,
    pub parent_tree: Option<String>,
    pub phase: RootPhase,
}

impl MandragorRoot {
    pub fn new(offset: [f32; 3], rebirth_delay: f32, too_close: f32, repel_dist: f32) -> Self {
        Self {
            offset,
            rebirth_delay,
            too_close,
            repel_dist,
            orig_health: 0,
            orig_keep_on_ground: true,
            parent_tree: None,
            phase: RootPhase::Alive,
        }
    }
}

/// Computes the parent `KQMandragorTree` name by replacing a trailing `'l'` or `'r'` with `'t'`
/// (`mand3l` -> `mand3t`, `fighting.md` 7b).
pub fn parent_tree_name(root_name: &str) -> Option<String> {
    let last = root_name.chars().next_back()?;
    if !matches!(last, 'l' | 'L' | 'r' | 'R') {
        return None;
    }
    let stem = &root_name[..root_name.len() - last.len_utf8()];
    Some(format!("{stem}t"))
}

/// Snaps a root to `tree_pos + tree_rot * offset` using KQ8's row-vector transform convention.
pub fn snap_position(tree_pos: [f32; 3], tree_rot: &[[f32; 3]; 3], offset: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|j| (0..3).map(|k| offset[k] * tree_rot[k][j]).sum::<f32>() + tree_pos[j])
}

/// Checks whether Connor is horizontally closer than `too_close` to the root (`fighting.md` 7b).
/// Returns `(new_connor_pos, facing_tree_heading)` preserving Connor's Z.
pub fn repel_connor(
    root_pos: [f32; 3],
    tree_pos: [f32; 3],
    connor_pos: [f32; 3],
    too_close: f32,
    repel_dist: f32,
) -> Option<([f32; 3], f32)> {
    let flat_d = (connor_pos[0] - root_pos[0]).hypot(connor_pos[1] - root_pos[1]);
    if flat_d >= too_close {
        return None;
    }
    let (vx, vy) = (tree_pos[0] - connor_pos[0], tree_pos[1] - connor_pos[1]);
    let len = vx.hypot(vy);
    let (ux, uy) = if len > 1e-4 {
        (vx / len, vy / len)
    } else {
        let (rx, ry) = (tree_pos[0] - root_pos[0], tree_pos[1] - root_pos[1]);
        let rlen = rx.hypot(ry);
        if rlen > 1e-4 {
            (rx / rlen, ry / rlen)
        } else {
            (0.0, 1.0)
        }
    };
    let pos = [root_pos[0] - ux * repel_dist, root_pos[1] - uy * repel_dist, connor_pos[2]];
    let heading = ux.atan2(-uy);
    Some((pos, heading))
}

impl crate::active::WorldState {
    /// Initializes all `KQMandragorTree` and `KQMandragorRoot` instances after world load
    /// (`fighting.md` 7b): switches gravity off, records root `origHealth`/`orig_keep_on_ground`,
    /// links each `'l'`/`'r'` root to its `'t'` tree, and snaps roots to their shoulder offsets.
    pub fn init_mandragores(&mut self) {
        let mut links: Vec<(usize, usize)> = Vec::new();
        for i in 0..self.anims.len() {
            let Some(c) = self.anims[i].character.as_deref_mut() else { continue };
            if c.mandragore_tree.is_some() {
                c.gravity = false;
                if let Some(m) = c.mover.as_deref_mut() {
                    m.use_gravity = false;
                }
                let name = self.anims[i].player.name.clone();
                self.character_note(&name, "KQMandragorTree: gravity switched off (fighting.md 7b)".to_string());
                continue;
            }
            if c.mandragore_root.is_some() {
                c.gravity = false;
                let keep = c.mover.as_deref_mut().map_or(true, |m| {
                    m.use_gravity = false;
                    m.keep_on_ground
                });
                let orig_hp = c.fight.as_ref().map_or(0, |f| f.stats.0[0]);
                if let Some(r) = c.mandragore_root.as_mut() {
                    r.orig_health = orig_hp;
                    r.orig_keep_on_ground = keep;
                }
                let root_name = self.anims[i].player.name.clone();
                if let Some(target) = parent_tree_name(&root_name) {
                    if let Some(ti) = self.anims.iter().position(|a| {
                        !a.gone && a.player.name.eq_ignore_ascii_case(&target) && a.character.as_deref().is_some_and(|ch| ch.mandragore_tree.is_some())
                    }) {
                        let tree_name = self.anims[ti].player.name.clone();
                        if let Some(r) = self.anims[i].character.as_deref_mut().and_then(|ch| ch.mandragore_root.as_mut()) {
                            r.parent_tree = Some(tree_name);
                        }
                        links.push((i, ti));
                    }
                }
            }
        }
        for (ri, ti) in links {
            let (tree_pos, tree_rot, tree_name) = {
                let p = &self.anims[ti].player;
                (p.base.position, p.base.rotation, p.name.clone())
            };
            let snapped = {
                let a = &mut self.anims[ri];
                let Some(c) = a.character.as_deref_mut() else { continue };
                let Some(r) = c.mandragore_root.as_ref() else { continue };
                let snapped = snap_position(tree_pos, &tree_rot, r.offset);
                a.player.base.position = snapped;
                if let Some(m) = c.mover.as_deref_mut() {
                    m.position = snapped;
                }
                if let Some(ai) = c.ai.as_mut() {
                    ai.home = snapped;
                }
                snapped
            };
            let root_name = self.anims[ri].player.name.clone();
            if let Some(cl) = self.clickables.iter_mut().find(|c| c.name.eq_ignore_ascii_case(&root_name)) {
                cl.placement.position = snapped;
            }
            self.character_note(
                &root_name,
                format!("KQMandragorRoot linked to {tree_name}: gravity off, snapped to ({:.0}, {:.0}, {:.0}) (fighting.md 7b)", snapped[0], snapped[1], snapped[2]),
            );
        }
    }

    /// Snaps all surviving linked roots whenever their parent `KQMandragorTree` moves or turns,
    /// and runs the Connor repel check for active roots (`fighting.md` 7b).
    pub(crate) fn sync_mandragore_roots(&mut self, tree_idx: usize) {
        let Some(tree) = self.anims.get(tree_idx) else { return };
        if tree.gone {
            return;
        }
        let tree_name = tree.player.name.clone();
        let tree_pos = tree.player.base.position;
        let tree_rot = tree.player.base.rotation;
        let tree_dead = tree.character.as_deref().is_some_and(|c| c.state == crate::character::State::Die);
        let mut active_roots = Vec::new();
        for ri in 0..self.anims.len() {
            if self.anims[ri].gone {
                continue;
            }
            let linked = self.anims[ri]
                .character
                .as_deref()
                .and_then(|c| c.mandragore_root.as_ref())
                .filter(|r| r.parent_tree.as_deref().is_some_and(|p| p.eq_ignore_ascii_case(&tree_name)))
                .map(|r| (r.offset, r.phase));
            let Some((offset, phase)) = linked else { continue };
            let snapped = snap_position(tree_pos, &tree_rot, offset);
            let a = &mut self.anims[ri];
            a.player.base.position = snapped;
            if let Some(c) = a.character.as_deref_mut() {
                if let Some(m) = c.mover.as_deref_mut() {
                    m.position = snapped;
                }
                if let Some(ai) = c.ai.as_mut() {
                    ai.home = snapped;
                }
            }
            let rname = self.anims[ri].player.name.clone();
            if let Some(cl) = self.clickables.iter_mut().find(|c| c.name.eq_ignore_ascii_case(&rname)) {
                cl.placement.position = snapped;
            }
            if !tree_dead && phase == RootPhase::Alive {
                active_roots.push(ri);
            }
        }
        for ri in active_roots {
            self.check_mandragore_repel(ri, "parent tree moved/turned");
        }
    }

    /// Runs the `KQMandragorRoot` proximity check and queues `Request::ConnorRepel` if Connor
    /// is within `tooClose`, not dying, and not already in `Special` (`fighting.md` 7b).
    pub(crate) fn check_mandragore_repel(&mut self, root_idx: usize, why: &str) {
        if self.connor_dying || self.connor_state == Some(crate::character::state_number::SPECIAL) {
            return;
        }
        if self.requests.iter().any(|r| matches!(r, crate::active::Request::ConnorRepel { .. })) {
            return;
        }
        let Some(connor) = self.connor_at else { return };
        let Some(a) = self.anims.get(root_idx) else { return };
        if a.gone {
            return;
        }
        let root_name = a.player.name.clone();
        let root_pos = a.player.base.position;
        let Some(root) = a.character.as_deref().and_then(|c| c.mandragore_root.as_ref()) else { return };
        let Some(parent_name) = root.parent_tree.as_deref() else { return };
        let (too_close, repel_dist) = (root.too_close, root.repel_dist);
        let Some(ti) = self.character_index(parent_name) else { return };
        let tree = &self.anims[ti];
        if tree.character.as_deref().is_some_and(|c| c.state == crate::character::State::Die) {
            return;
        }
        let tree_pos = tree.player.base.position;
        let Some((pos, heading)) = repel_connor(root_pos, tree_pos, connor, too_close, repel_dist) else {
            return;
        };
        let at = format!("{root_name} repel ({why}, fighting.md 7b)");
        let line = format!(
            "repels Connor (2D dist < {too_close:.0}) to ({:.0}, {:.0}, {:.0}) facing tree heading {heading:.3} -> Special {REPEL_MOVEMENT}",
            pos[0], pos[1], pos[2]
        );
        self.fight_note(&root_name, line);
        self.requests.push(crate::active::Request::ConnorRepel { position: pos, heading, at });
    }

    /// 0-damage hit on `KQMandragorTree` or `KQMandragorRoot` (`fighting.md` 7b): plays `Special laugh`
    /// on the tree (or parent tree without counting as a hit on the tree). Returns `true` if handled.
    pub(crate) fn mandragore_zero_hit_laugh(&mut self, j: usize, from: &str, kind: crate::fight::HitKind, now: f64) -> bool {
        let name = self.anims[j].player.name.clone();
        let (is_tree, parent_tree) = {
            let Some(c) = self.anims[j].character.as_deref() else { return false };
            (c.mandragore_tree.is_some(), c.mandragore_root.as_ref().and_then(|r| r.parent_tree.clone()))
        };
        if !is_tree && parent_tree.is_none() && self.anims[j].character.as_deref().is_none_or(|c| c.mandragore_root.is_none()) {
            return false;
        }
        let target_idx = if is_tree {
            Some(j)
        } else {
            parent_tree.as_deref().and_then(|p| self.character_index(p))
        };
        if let Some(ti) = target_idx {
            let ta = &mut self.anims[ti];
            let tname = ta.player.name.clone();
            if let Some(tc) = ta.character.as_deref_mut() {
                let dying = tc.state == crate::character::State::Die || tc.fight.as_ref().is_some_and(|f| f.dying.is_some());
                if !dying {
                    let st = tc.set_state(&mut ta.player, "Special", &["laugh".to_string()], now);
                    if ti == j {
                        self.fight_note(&tname, format!("0-damage hit from {from} ({}): plays Special laugh (fighting.md 7b): {st}", kind.name()));
                    } else {
                        self.fight_note(&name, format!("0-damage hit from {from} ({}): sends parent tree {tname} to Special laugh (fighting.md 7b)", kind.name()));
                        self.fight_note(&tname, format!("Special laugh (from root {name} 0-damage hit; fighting.md 7b): {st}"));
                    }
                }
            }
        } else {
            self.fight_note(&name, format!("0-damage hit from {from} ({}): no live parent tree (fighting.md 7b)", kind.name()));
        }
        true
    }

    /// Root death while parent `KQMandragorTree` is alive (`fighting.md` 7b): sinks with `Die`
    /// (`r_sink.kqs`), increments `rootDownCount`, and turns parent tree's `noTurn` ON.
    pub(crate) fn mandragore_root_sink(&mut self, j: usize, kind: crate::fight::HitKind) -> bool {
        let Some(parent_name) = self.anims[j]
            .character
            .as_deref()
            .and_then(|c| c.mandragore_root.as_ref())
            .and_then(|r| r.parent_tree.clone())
        else {
            return false;
        };
        let Some(ti) = self.character_index(&parent_name) else { return false };
        if self.anims[ti].character.as_deref().is_some_and(|c| c.state == crate::character::State::Die) {
            return false;
        }
        let a = &mut self.anims[j];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return false };
        let Some(r) = c.mandragore_root.as_mut() else { return false };
        if r.phase != RootPhase::Alive {
            return true;
        }
        r.phase = RootPhase::Sinking;
        c.special_hold = false;
        c.state = crate::character::State::Die;
        c.gravity = false;
        c.collision_type = 0;
        c.collision_mask = 0;
        c.turn_to = None;
        if let Some(m) = c.mover.as_deref_mut() {
            m.keep_on_ground = false;
        }
        if let Some(f) = c.fight.as_mut() {
            f.stats.0[0] = 0;
            f.last_kind = Some(kind);
            f.flinching = None;
            f.dying = Some(Some("Die".to_string()));
        }
        if let Some(ai) = c.ai.as_mut() {
            ai.active = false;
            ai.alarmed = false;
            ai.hunting = false;
            ai.on_way_home = false;
        }
        let st = a.player.start("Die");
        self.active_list.retain(|n| !n.eq_ignore_ascii_case(&name));
        let (down, max_roots, tname) = {
            let ta = &mut self.anims[ti];
            let tname = ta.player.name.clone();
            let tc = ta.character.as_deref_mut().unwrap();
            tc.no_turn = true;
            tc.turn_to = None;
            let t = tc.mandragore_tree.as_mut().unwrap();
            t.root_down_count = (t.root_down_count + 1).min(t.max_roots);
            (t.root_down_count, t.max_roots, tname)
        };
        self.fight_note(
            &name,
            format!(
                "root dies with live tree {tname} (fighting.md 7b): sinks ('Die' starts{}); collision 0/0, keepOnGround off; {tname} rootDownCount={down}/{max_roots}, noTurn ON",
                if st.unknown { " (not in list)" } else { "" }
            ),
        );
        true
    }

    /// Notifies all linked roots when their parent `KQMandragorTree` enters `Die`: clears links
    /// and kills them permanently (`r_sink.kqs`, 0 XP, `fighting.md` 7b).
    pub(crate) fn kill_mandragore_roots(&mut self, tree_name: &str, now: f64) {
        let mut to_kill = Vec::new();
        for ri in 0..self.anims.len() {
            if self.anims[ri].gone {
                continue;
            }
            let Some(r) = self.anims[ri].character.as_deref_mut().and_then(|c| c.mandragore_root.as_mut()) else { continue };
            if r.parent_tree.as_deref().is_some_and(|p| p.eq_ignore_ascii_case(tree_name)) {
                to_kill.push((ri, r.phase));
            }
        }
        for (ri, prev_phase) in to_kill {
            let rname = self.anims[ri].player.name.clone();
            let c = self.anims[ri].character.as_deref_mut().unwrap();
            if let Some(r) = c.mandragore_root.as_mut() {
                r.parent_tree = None;
                r.phase = RootPhase::PermanentDie;
            }
            match prev_phase {
                RootPhase::Underground { .. } => {
                    self.fight_note(&rname, format!("parent tree {tree_name} died while underground: permanently dead (0 XP, fighting.md 7b)"));
                    self.death_ended(ri, now);
                }
                RootPhase::Sinking => {
                    self.fight_note(&rname, format!("parent tree {tree_name} died while sinking: finishes sink permanently (0 XP, fighting.md 7b)"));
                }
                RootPhase::Alive | RootPhase::Rebirthing => {
                    if let Some(c) = self.anims[ri].character.as_deref_mut() {
                        c.unborn = false;
                        c.state = crate::character::State::Wait;
                        if let Some(f) = c.fight.as_mut() {
                            f.dying = None;
                        }
                    }
                    self.fight_note(&rname, format!("parent tree {tree_name} died: enters permanent Die (0 XP, fighting.md 7b)"));
                    self.character_dies(ri, crate::fight::HitKind::Normal, false);
                }
                RootPhase::PermanentDie => {}
            }
        }
    }

    /// Handles a `KQMandragorRoot` finishing `Die` (`r_sink.kqs`): enters underground wait if still
    /// linked to a live tree, returning `true` to suppress permanent `death_ended` (`fighting.md` 7b).
    pub(crate) fn mandragore_root_die_ended(&mut self, i: usize, now: f64) -> bool {
        let Some(a) = self.anims.get_mut(i) else { return false };
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return false };
        let Some(r) = c.mandragore_root.as_mut() else { return false };
        if r.phase != RootPhase::Sinking {
            return false;
        }
        let wake_at = now + f64::from(r.rebirth_delay.max(0.0));
        r.phase = RootPhase::Underground { wake_at };
        c.unborn = true;
        c.state = crate::character::State::Birth;
        if let Some(f) = c.fight.as_mut() {
            f.dying = None;
        }
        let delay = r.rebirth_delay;
        self.fight_note(&name, format!("sink ended: underground for rebirthDelay {delay} s (wakes at {wake_at:.2}; fighting.md 7b)"));
        true
    }

    /// Wakes `KQMandragorRoot` instances whose `rebirthDelay` has elapsed (`fighting.md` 7b).
    pub(crate) fn mandragore_underground_step(&mut self, now: f64) {
        let mut ready = Vec::new();
        for i in 0..self.anims.len() {
            if self.anims[i].gone {
                continue;
            }
            if let Some(RootPhase::Underground { wake_at }) = self.anims[i].character.as_deref().and_then(|c| c.mandragore_root.as_ref()).map(|r| r.phase) {
                if now + 1e-6 >= wake_at {
                    ready.push(i);
                }
            }
        }
        for i in ready {
            let parent = self.anims[i].character.as_deref().and_then(|c| c.mandragore_root.as_ref()).and_then(|r| r.parent_tree.clone());
            let tree_alive = parent
                .as_deref()
                .and_then(|p| self.character_index(p))
                .is_some_and(|ti| self.anims[ti].character.as_deref().is_some_and(|c| c.state != crate::character::State::Die));
            if !tree_alive {
                let name = self.anims[i].player.name.clone();
                if let Some(r) = self.anims[i].character.as_deref_mut().and_then(|c| c.mandragore_root.as_mut()) {
                    r.parent_tree = None;
                    r.phase = RootPhase::PermanentDie;
                }
                self.fight_note(&name, "rebirth due but parent tree gone: permanent death (0 XP, fighting.md 7b)");
                self.death_ended(i, now);
                continue;
            }
            let (name, orig_health, st_unknown) = {
                let a = &mut self.anims[i];
                let name = a.player.name.clone();
                let c = a.character.as_deref_mut().unwrap();
                let orig = c.mandragore_root.as_ref().unwrap().orig_health;
                c.mandragore_root.as_mut().unwrap().phase = RootPhase::Rebirthing;
                c.unborn = false;
                c.state = crate::character::State::Birth;
                if let Some(f) = c.fight.as_mut() {
                    f.stats.0[0] = orig;
                }
                let st = a.player.start("Birth");
                (name, orig, st.unknown)
            };
            self.fight_note(
                &name,
                format!(
                    "rebirth starts (fighting.md 7b): health restored to {orig_health}, Birth plays{}",
                    if st_unknown { " (not in list)" } else { "" }
                ),
            );
            self.check_mandragore_repel(i, "root Birth start");
        }
    }

    /// Handles `Birth` (`r_appr.kqs`) ending on a rebirthing `KQMandragorRoot` (`fighting.md` 7b).
    /// Restores collision `0x4042` / `0x201f`, decrements parent tree's `rootDownCount`, clears
    /// tree `noTurn` if `0`, and returns to `Wait` (or enters permanent `Die` if tree died).
    pub(crate) fn mandragore_root_resurfaced(&mut self, i: usize, _now: f64) -> bool {
        let Some(a) = self.anims.get_mut(i) else { return false };
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return false };
        let Some(r) = c.mandragore_root.as_mut() else { return false };
        if r.phase != RootPhase::Rebirthing {
            return false;
        }
        c.collision_type = kq8_data::characters::MANDRAGORE_TYPE;
        c.collision_mask = ROOT_RESURFACE_MASK;
        let keep = r.orig_keep_on_ground;
        if let Some(m) = c.mover.as_deref_mut() {
            m.keep_on_ground = keep;
        }
        let parent_name = r.parent_tree.clone();
        let tree_idx = parent_name
            .as_deref()
            .and_then(|p| self.character_index(p))
            .filter(|&ti| self.anims[ti].character.as_deref().is_some_and(|tc| tc.state != crate::character::State::Die));
        if let Some(ti) = tree_idx {
            let mut notes = Vec::new();
            {
                let a = &mut self.anims[i];
                let c = a.character.as_deref_mut().unwrap();
                c.mandragore_root.as_mut().unwrap().phase = RootPhase::Alive;
                c.enter_wait(&mut a.player, &mut notes);
            }
            let (down, max_roots, tname) = {
                let ta = &mut self.anims[ti];
                let tname = ta.player.name.clone();
                let tc = ta.character.as_deref_mut().unwrap();
                let t = tc.mandragore_tree.as_mut().unwrap();
                t.root_down_count = (t.root_down_count - 1).max(0);
                if t.root_down_count == 0 {
                    tc.no_turn = false;
                }
                (t.root_down_count, t.max_roots, tname)
            };
            self.fight_note(
                &name,
                format!(
                    "resurfaced (fighting.md 7b): collision 0x4042 / 0x201f restored, {tname} rootDownCount={down}/{max_roots}, noTurn={}; {}",
                    if down == 0 { "OFF" } else { "ON" },
                    notes.join("; ")
                ),
            );
        } else {
            if let Some(c) = self.anims[i].character.as_deref_mut() {
                if let Some(r) = c.mandragore_root.as_mut() {
                    r.parent_tree = None;
                    r.phase = RootPhase::PermanentDie;
                }
                c.state = crate::character::State::Wait;
            }
            self.fight_note(&name, "resurfaced but parent tree already died: permanent Die (0 XP, fighting.md 7b)");
            self.character_dies(i, crate::fight::HitKind::Normal, false);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_tree_name_replaces_trailing_side() {
        assert_eq!(parent_tree_name("mand3l").as_deref(), Some("mand3t"));
        assert_eq!(parent_tree_name("MAND11R").as_deref(), Some("MAND11t"));
        assert_eq!(parent_tree_name("mand3t"), None);
        assert_eq!(parent_tree_name(""), None);
    }

    #[test]
    fn snap_position_follows_tree_position_and_rotation() {
        let pos = [157_904.0, 140_178.0, 6120.0];
        let rot0 = kq8_data::objects::rotation([0.0, 0.0, 0.0]);
        assert_eq!(snap_position(pos, &rot0, [-600.0, -1000.0, 0.0]), [157_304.0, 139_178.0, 6120.0]);
        // Heading +pi/2 (+x forward, +y left): (-600, -1000, 0) snaps +1000 in X and -600 in Y.
        let rot90 = kq8_data::objects::rotation([0.0, 0.0, std::f32::consts::FRAC_PI_2]);
        let snapped = snap_position(pos, &rot90, [-600.0, -1000.0, 0.0]);
        assert!((snapped[0] - (pos[0] + 1000.0)).abs() < 1e-2);
        assert!((snapped[1] - (pos[1] - 600.0)).abs() < 1e-2);
        assert!((snapped[2] - pos[2]).abs() < 1e-2);
    }

    #[test]
    fn repel_connor_pushes_outward_from_root_and_faces_tree() {
        let tree = [0.0, 1000.0, 5000.0];
        let root = [0.0, 0.0, 5000.0];
        // At exact boundary (700) or beyond -> no repel.
        assert_eq!(repel_connor(root, tree, [0.0, -700.0, 5123.0], 700.0, 800.0), None);
        // Inside 700 -> pushed 800 from root along tree->Connor (-y), keeping Z, facing +y (PI).
        let (pushed, heading) = repel_connor(root, tree, [0.0, -200.0, 5123.0], 700.0, 800.0).unwrap();
        assert!((pushed[0] - 0.0).abs() < 1e-3);
        assert!((pushed[1] - (-800.0)).abs() < 1e-3);
        assert_eq!(pushed[2], 5123.0);
        assert!((heading - std::f32::consts::PI).abs() < 1e-4);
    }
}
