//! Imports original `Mask.exe` `save<N>.vol` archives (binary `PERS` current world
//! + embedded ASCII kept-world `.vol` archives and `.kq` maps) into `kq8-core`'s
//! [`SavedWorldSnapshot`] (`docs/behaviour/savefile.md`, M4 step 4h-3).

use std::collections::{BTreeMap, BTreeSet};

use kq8_data::items::ItemTypes;
use kq8_data::kq_text;
use kq8_data::magic_map::{MapKeys, MAP_COUNT};
use kq8_data::objects::{angles_of, rotation};
use kq8_data::pvol::Pvol;
use kq8_data::save_pers::{
    parse_pers, KqAnimObject, KqConner, KqInventory, KqInventoryItem, KqObject, PersObject,
};

use crate::active::{KeptChestState, KeptDoorState, KeptState};
use crate::chest::ChestState;
use crate::collide::Placement;
use crate::door::DoorState;
use crate::item::{ItemObject, Place};
use crate::journey::Journey;
use crate::magic_map::{ConnorMapMarks, START_VIEW, TELEPORT_MARKS};
use crate::potions::Running;
use crate::save_game::{
    parse_game_state_kq, parse_world_var_cs, KeptWorldSnapshot, SavedCameraState,
    SavedWorldSnapshot,
};
use crate::stats::{ConnorStats, Stats};
use crate::weapons::WeaponMode;

fn ascii_text(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).trim().to_string()
}

fn strip_kq_ext(s: &str) -> String {
    let base = s.rsplit(['/', '\\']).next().unwrap_or(s);
    base.strip_suffix(".kq")
        .or_else(|| base.strip_suffix(".KQ"))
        .or_else(|| {
            let low = base.to_ascii_lowercase();
            low.ends_with(".kq").then(|| &base[..base.len() - 3])
        })
        .unwrap_or(base)
        .to_string()
}

fn pipe_list(b: &[u8]) -> Vec<String> {
    ascii_text(b)
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Parses `newWorld.cs` for `setCurrentTime <secs>` and `loadKQ <file>` object names (`savefile.md` rule 39).
fn parse_new_world_cs(bytes: &[u8]) -> (Option<f64>, BTreeSet<String>) {
    let mut current_time = None;
    let mut present = BTreeSet::new();
    let text = String::from_utf8_lossy(bytes);
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut words = line.split_whitespace();
        let Some(cmd) = words.next() else { continue };
        if cmd.eq_ignore_ascii_case("setCurrentTime") {
            if let Some(t) = words.next().and_then(|s| s.parse::<f64>().ok()) {
                current_time = Some(t);
            }
        } else if cmd.eq_ignore_ascii_case("loadKQ") {
            if let Some(file) = words.next() {
                let stem = strip_kq_ext(file).to_ascii_lowercase();
                if !stem.is_empty() {
                    present.insert(stem);
                }
            }
        }
    }
    (current_time, present)
}

fn placement_from_object(obj: &KqObject) -> Placement {
    if let Some(t) = obj.transform() {
        Placement {
            rotation: t.rotation,
            position: t.position,
        }
    } else {
        Placement::at([0.0; 3])
    }
}

fn anim_to_kept_state(ao: &KqAnimObject) -> KeptState {
    KeptState {
        name: ao.object.visible.base.name.clone().unwrap_or_default(),
        movement: ao.curr_anim_name.clone().unwrap_or_default(),
        phase: ao.anim_pos,
        waiting: ao.next_anim_name.clone().filter(|s| !s.is_empty()),
        placement: placement_from_object(&ao.object),
        collision_type: Some(ao.object.collider_type),
    }
}

fn resolve_item_kind(
    item_type_id: i32,
    str_name: Option<&str>,
    types: Option<&ItemTypes>,
) -> String {
    if let Some(s) = str_name.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(found) = types.and_then(|t| t.find(s)) {
            return found.name.clone();
        }
        return kq8_data::items::full_name(s);
    }
    let id_str = item_type_id.to_string();
    if let Some(found) = types.and_then(|t| t.find(&id_str)) {
        return found.name.clone();
    }
    id_str
}

fn item_from_pers(
    it: &KqInventoryItem,
    owner: Option<&str>,
    types: Option<&ItemTypes>,
) -> ItemObject {
    let name = it.object.visible.base.name.clone().unwrap_or_default();
    let kind = resolve_item_kind(it.item_type, None, types);
    let place = match owner {
        Some(who) => Place::Carried(who.to_string()),
        None => {
            let p = placement_from_object(&it.object);
            Place::World {
                position: p.position,
                angles: angles_of(p.rotation),
            }
        }
    };
    ItemObject {
        name,
        kind,
        quantity: i64::from(it.quantity),
        third: i64::from(it.user_id as i32),
        place,
        hidden: it.object.visible.hidden != 0,
        script: it.object.script.clone().filter(|s| !s.is_empty()),
        do_not_point_at: it.object.do_not_point_at != 0,
        added: 0,
    }
}

fn parse_stats_string(s: Option<&str>, level: i32) -> Option<ConnorStats> {
    let words: Vec<&str> = s?.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    let (stats, _) = Stats::parse(&words);
    Some(ConnorStats {
        stats,
        level: i64::from(level.max(1)),
    })
}

fn parse_map_chunk(data: &[u8]) -> Option<MapKeys> {
    if data.starts_with(b"PERS") || data.starts_with(b"DICT") {
        if let Ok(PersObject::KqMap(m)) = parse_pers(data) {
            return Some(m.to_map_keys());
        }
        None
    } else {
        kq_text::parse(data)
            .iter()
            .find_map(|s| MapKeys::from_section(s).and_then(Result::ok))
    }
}

fn record_map(keys: MapKeys, label: &str, journey: &mut Journey) {
    if let Some(idx) = keys.index() {
        if idx < journey.map_marks.visited.len() {
            journey.map_marks.visited[idx] = true;
        }
        if keys.earned_teleport && idx < TELEPORT_MARKS {
            journey.map_marks.teleport_earned[idx] = true;
        }
        journey
            .other_maps
            .insert(idx, (keys.name.clone(), keys, label.to_string()));
    }
}

fn apply_connor_pers(
    c: &KqConner,
    world_base: &str,
    types: Option<&ItemTypes>,
    world_vars: &[(String, String)],
) -> (
    Option<[f32; 3]>,
    Option<([f32; 3], f32)>,
    Option<(String, f32)>,
    Journey,
    Option<crate::rope::Rope>,
    Option<crate::rope::Climb>,
    Vec<crate::talk::SavedTalkQueue>,
) {
    let placement = placement_from_object(&c.monster.anim_object.object);
    let pos = placement.position;
    let heading = angles_of(placement.rotation)[2];
    let connor_at = Some([pos[0], pos[1], heading]);
    let connor_pos = Some((pos, heading));
    let connor_anim = Some((
        c.monster
            .anim_object
            .curr_anim_name
            .clone()
            .unwrap_or_else(|| "Stop".into()),
        c.monster.anim_object.anim_pos,
    ));

    let mut items = Vec::new();
    if let Some((_, inv_obj)) = &c.monster.inventory {
        let inv: Option<&KqInventory> = match inv_obj.as_ref() {
            PersObject::KqInventory { data: i, .. } => Some(i),
            _ => None,
        };
        if let Some(inv) = inv {
            for entry in inv.items.iter().flatten() {
                if let PersObject::KqInventoryItem(it) = entry.1.as_ref() {
                    items.push(item_from_pers(it, Some("Connor"), types));
                }
            }
        }
    }

    let weapon_mode =
        WeaponMode::from_number(i64::from(c.monster.do_mode)).unwrap_or(WeaponMode::None);
    let mut weapon_ready = None;
    let mut projectile = None;
    if let Some(types) = types {
        let find_equipped = |category: &str, placeholder: &str| -> Option<(String, Option<String>)> {
            let candidates: Vec<&ItemObject> = items
                .iter()
                .filter(|it| {
                    types
                        .find(&it.kind)
                        .is_some_and(|t| t.category.eq_ignore_ascii_case(category))
                })
                .collect();
            let chosen = candidates
                .iter()
                .copied()
                .find(|it| !it.kind.eq_ignore_ascii_case(placeholder))
                .or_else(|| candidates.first().copied())?;
            let extra = types.find(&chosen.kind).and_then(|t| t.extra.clone());
            Some((chosen.name.clone(), extra))
        };
        match weapon_mode {
            WeaponMode::Hand => {
                if let Some((name, _)) = find_equipped("ShortWeapon", "INVITEM_Fists") {
                    weapon_ready = Some(name);
                }
            }
            WeaponMode::Ranged => {
                if let Some((name, proj)) = find_equipped("LongWeapon", "INVITEM_NoRanged") {
                    weapon_ready = Some(name);
                    projectile = proj;
                }
            }
            WeaponMode::None => {}
        }
    }

    let mut potions = Vec::new();
    let spell_specs = [
        (c.clarity_spell, "INVITEM_Clarity"),
        (c.strength_spell, "INVITEM_Strength"),
        (c.invulnerability_spell, "INVITEM_Invulnerable"),
        (c.invisibility_spell, "INVITEM_Invisible"),
    ];
    for (slot, (secs, kind)) in spell_specs
        .into_iter()
        .filter(|(s, _)| *s > 0)
        .enumerate()
    {
        let item_name = items
            .iter()
            .find(|it| it.kind.eq_ignore_ascii_case(kind))
            .map(|it| it.name.clone())
            .unwrap_or_else(|| kind.to_string());
        let mut r = Running::new(&item_name, kind, i64::from(secs), 0);
        r.next_tick = 0;
        r.place = Some(slot.min(6));
        potions.push((r, 15));
    }

    let mut visited = [false; MAP_COUNT];
    let mut teleport_earned = [false; TELEPORT_MARKS];
    // Best-effort map bits while Q208 settles exact `worldFlags` layout:
    // if Connor carries the Magic Map, saved map chunks below mark visited/teleport too.
    let _ = (&mut visited, &mut teleport_earned);

    let vars = world_vars
        .iter()
        .filter(|(n, _)| !crate::journey::belongs_to_world(n, world_base))
        .cloned()
        .collect();

    let journey = Journey {
        from: world_base.to_string(),
        stats: parse_stats_string(c.monster.anim_object.object.stats.as_deref(), c.level),
        items,
        hand: c.item_in_hand.clone().filter(|s| !s.is_empty()),
        weapon_ready,
        weapon_mode,
        last_away_ranged: c.was_in_bow_mode != 0,
        projectile,
        shoot_offset: [0.0; 3],
        potions,
        pickup_experience: Vec::new(),
        flags: BTreeMap::new(),
        in_temple4: false,
        immunities: c.monster.anim_object.object.immunities as u16,
        raw_world_flags: Some(c.world_flags),
        map_marks: ConnorMapMarks {
            visited,
            teleport_earned,
            sage_gnome: false,
        },
        map_view: START_VIEW,
        other_maps: BTreeMap::new(),
        combat_level: 1.0,
        next_no_name: 1,
        helps_asked: BTreeSet::new(),
        connor_tips: [false; 2],
        kill_tip: false,
        first_kill_remark: false,
        not_starting_help: c.start_help.clamp(0, 2) as u8,
        tip_bits: 0,
        god_mode: false,
        ai_off: false,
        allow_shadows: c.allow_shadows_in_game != 0,
        vars,
    };

    let nonzero3 = |v: [f32; 3]| v.iter().any(|x| *x != 0.0).then_some(v);
    let rope_status = crate::rope::Status::from_pers(c.rope_status);
    let has_rope_info = rope_status != crate::rope::Status::None
        || c.rope_cam_top_limit != 0.0
        || c.rope_cam_bottom_limit != 0.0
        || nonzero3(c.rope_cam_loc1).is_some()
        || nonzero3(c.rope_cam_loc2).is_some()
        || nonzero3(c.rope_cam2_loc).is_some()
        || c.rope_cam_ang != 0.0
        || c.rope_cam_range != 0.0
        || nonzero3(c.rope_loc).is_some()
        || nonzero3(c.rope_dir).is_some();
    let rope = has_rope_info.then_some(crate::rope::Rope {
        status: rope_status,
        place: c.rope_loc,
        dir: c.rope_dir,
        cam: [nonzero3(c.rope_cam_loc1), nonzero3(c.rope_cam_loc2)],
        top_cam: nonzero3(c.rope_cam2_loc),
        angle: (c.rope_cam_ang != 0.0 || c.rope_cam_range != 0.0)
            .then_some((c.rope_cam_ang, c.rope_cam_range)),
        top_limit: c.rope_cam_top_limit,
        bottom_limit: c.rope_cam_bottom_limit,
        changed_at: None,
    });

    let rope_climb = match c.monster.state.as_ref().map(|(_, o)| o.as_ref()) {
        Some(PersObject::KqState(kq8_data::save_pers::KqState {
            kind:
                kq8_data::save_pers::KqStateKind::ConnerHookRope {
                    done_roping: 0,
                    do_mode_before,
                    con_pos,
                    on_rope,
                    first_to_third,
                    bottom_cam_limit,
                    top_cam_limit,
                    con_cam_bottom_limit,
                    con_cam_top_limit,
                    rope_cam_loc1,
                    rope_cam_loc2,
                    orig_step_up,
                    orig_step_down,
                    ..
                },
            ..
        })) => Some(crate::rope::Climb {
            from: if c.rope_status == 1 {
                crate::rope::End::Top
            } else {
                crate::rope::End::Bottom
            },
            stage: if *on_rope != 0 {
                crate::rope::Stage::Hanging
            } else if c.rope_status == 1 {
                crate::rope::Stage::Drawdown
            } else {
                crate::rope::Stage::Btoss
            },
            place: c.rope_loc,
            dir: c.rope_dir,
            cam: [nonzero3(*rope_cam_loc1), nonzero3(*rope_cam_loc2)],
            top_limit: *con_cam_top_limit,
            bottom_limit: *con_cam_bottom_limit,
            hand_before: WeaponMode::from_number(i64::from(*do_mode_before))
                .unwrap_or(WeaponMode::None),
            weapon_before: None,
            first_to_third: *first_to_third != 0,
            aim: nonzero3(*con_pos).unwrap_or(pos),
            range: (*bottom_cam_limit, *top_cam_limit),
            saved_steps: (*orig_step_up != 0.0 || *orig_step_down != 0.0)
                .then_some((*orig_step_up, *orig_step_down)),
            done: false,
        }),
        _ => None,
    };

    let mut active_talk = Vec::new();
    if let Some((_, talk_obj)) = &c.monster.talker {
        if let PersObject::KqTalk(t) = talk_obj.as_ref() {
            let issuer = t
                .talker_name
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(crate::talk::CONNOR)
                .to_string();
            let tuples: Vec<(kq8_data::msg::Tuple, bool)> = t
                .talk_list
                .iter()
                .flatten()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 5 {
                        return None;
                    }
                    let file: u16 = parts[0].parse().ok()?;
                    let noun: u8 = parts[1].parse().ok()?;
                    let verb: u8 = parts[2].parse().ok()?;
                    let case: u8 = parts[3].parse().ok()?;
                    let seq: u8 = parts[4].parse().ok()?;
                    let flag_last = parts.get(6).and_then(|s| s.parse::<i32>().ok()).is_some_and(|v| v != 0);
                    Some((kq8_data::msg::Tuple::new(file, noun, verb, case, seq), flag_last))
                })
                .collect();
            if let Some((first_tuple, _)) = tuples.first().copied() {
                let len = tuples.len();
                let nofade = t.no_fade != 0;
                let lines = tuples
                    .into_iter()
                    .enumerate()
                    .map(|(idx, (tuple, flag_last))| crate::talk::SavedTalkEntry {
                        tuple,
                        first: if tuple.file == first_tuple.file
                            && tuple.noun == first_tuple.noun
                            && tuple.verb == first_tuple.verb
                            && tuple.case == first_tuple.case
                        {
                            first_tuple
                        } else {
                            tuple
                        },
                        last: flag_last || idx + 1 == len,
                        nofade,
                    })
                    .collect();
                active_talk.push(crate::talk::SavedTalkQueue { issuer, lines });
            }
        }
    }

    (connor_at, connor_pos, connor_anim, journey, rope, rope_climb, active_talk)
}

fn parse_kept_vol(
    bytes: Vec<u8>,
    types: Option<&ItemTypes>,
    journey: &mut Journey,
    label: &str,
) -> Result<(String, KeptWorldSnapshot), String> {
    let pvol = Pvol::parse(bytes)?;
    let chunk = |name: &str| -> Option<Vec<u8>> {
        let e = pvol.find(name)?;
        pvol.data(e).ok().map(|b| b.to_vec())
    };
    let world_raw = chunk("worldName")
        .map(|b| ascii_text(&b))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| strip_kq_ext(label));
    let world_name = strip_kq_ext(&world_raw);
    let mut kw = KeptWorldSnapshot {
        vars: chunk("WorldVar.cs")
            .map(|b| parse_world_var_cs(&b))
            .unwrap_or_default(),
        ..Default::default()
    };
    if let Some(nw) = chunk("newWorld.cs") {
        let (t, present) = parse_new_world_cs(&nw);
        kw.current_time = t;
        kw.present_objects = (!present.is_empty()).then_some(present);
    }

    for entry in &pvol.entries {
        let lower = entry.name.to_ascii_lowercase();
        if !lower.ends_with(".kq") {
            continue;
        }
        let Ok(data) = pvol.data(entry) else { continue };
        if lower.ends_with("map.kq") {
            if let Some(mk) = parse_map_chunk(data) {
                record_map(mk, &format!("imported from {label}"), journey);
            }
            continue;
        }
        if data.starts_with(b"PERS") || data.starts_with(b"DICT") {
            if let Ok(obj) = parse_pers(data) {
                apply_pers_world_object(obj, types, &mut kw);
            }
            continue;
        }
        for s in kq_text::parse(data) {
            apply_ascii_section(&s, types, &mut kw);
        }
    }
    Ok((world_name.to_ascii_lowercase(), kw))
}

fn apply_pers_world_object(
    obj: PersObject,
    types: Option<&ItemTypes>,
    kw: &mut KeptWorldSnapshot,
) {
    match obj {
        PersObject::KqDoor(d) => {
            let ao = &d.anim_object;
            let name = ao.object.visible.base.name.clone().unwrap_or_default();
            if let Some(p) = &mut kw.present_objects {
                p.insert(name.to_ascii_lowercase());
            }
            kw.doors.push(KeptDoorState {
                name,
                state: DoorState::from_pers(d.state),
                movement: ao.curr_anim_name.clone().unwrap_or_default(),
                phase: ao.anim_pos,
                waiting: ao.next_anim_name.clone().filter(|s| !s.is_empty()),
                placement: placement_from_object(&ao.object),
                collision_type: ao.object.collider_type,
            });
        }
        PersObject::KqChest(ch) => {
            let ao = &ch.anim_object;
            let name = ao.object.visible.base.name.clone().unwrap_or_default();
            if let Some(p) = &mut kw.present_objects {
                p.insert(name.to_ascii_lowercase());
            }
            kw.chests.push(KeptChestState {
                name,
                state: ChestState::from_pers(ch.chest_state),
                remembered: ch.inv_item_name.clone().filter(|s| !s.is_empty()),
                movement: ao.curr_anim_name.clone().unwrap_or_default(),
                phase: ao.anim_pos,
                waiting: ao.next_anim_name.clone().filter(|s| !s.is_empty()),
                placement: placement_from_object(&ao.object),
                hidden: ao.object.visible.hidden != 0,
                collision_type: ao.object.collider_type,
            });
        }
        PersObject::KqAnimObject(ao) | PersObject::KqAnimOnce(kq8_data::save_pers::KqAnimOnce { anim_object: ao, .. }) => {
            let st = anim_to_kept_state(&ao);
            if !st.name.is_empty() {
                if let Some(p) = &mut kw.present_objects {
                    p.insert(st.name.to_ascii_lowercase());
                }
                kw.states.push(st);
            }
        }
        PersObject::KqPortal(p) => {
            let st = anim_to_kept_state(&p.anim_object);
            if !st.name.is_empty() {
                kw.states.push(st);
            }
        }
        PersObject::KqBoulder(b) => {
            let st = anim_to_kept_state(&b.anim_object);
            if !st.name.is_empty() {
                kw.states.push(st);
            }
        }
        PersObject::KqLavaTile(l) => {
            let st = anim_to_kept_state(&l.anim_object);
            if !st.name.is_empty() {
                kw.states.push(st);
            }
        }
        PersObject::KqMonster { data: m, .. } => {
            let st = anim_to_kept_state(&m.anim_object);
            if !st.name.is_empty() {
                if let Some(p) = &mut kw.present_objects {
                    p.insert(st.name.to_ascii_lowercase());
                }
                kw.states.push(st);
            }
        }
        PersObject::KqTowerSkeleton(t) => {
            let st = anim_to_kept_state(&t.monster.anim_object);
            if !st.name.is_empty() {
                if let Some(p) = &mut kw.present_objects {
                    p.insert(st.name.to_ascii_lowercase());
                }
                kw.states.push(st);
            }
        }
        PersObject::KqMandragorRoot(r) => {
            let st = anim_to_kept_state(&r.monster.anim_object);
            if !st.name.is_empty() {
                if let Some(p) = &mut kw.present_objects {
                    p.insert(st.name.to_ascii_lowercase());
                }
                kw.states.push(st);
            }
        }
        PersObject::KqMandragorTree(t) => {
            let st = anim_to_kept_state(&t.monster.anim_object);
            if !st.name.is_empty() {
                if let Some(p) = &mut kw.present_objects {
                    p.insert(st.name.to_ascii_lowercase());
                }
                kw.states.push(st);
            }
        }
        PersObject::KqInterior(int) => {
            if int.using_alt_lighting != 0 {
                if let Some(name) = int.object.visible.base.name {
                    kw.alt_lightmaps.insert(name.to_ascii_lowercase(), true);
                }
            }
        }
        PersObject::KqRiverControl(rc) => {
            if rc.rusted_one != 0 {
                if let Some(name) = rc.object.visible.base.name {
                    kw.rusted_rivers.push(name);
                }
            }
        }
        PersObject::KqWorldState(ws) => {
            kw.pending_events
                .extend(ws.events.into_iter().flatten());
        }
        PersObject::KqMusic(m) => {
            kw.music_stack = Some(m.music.into_iter().flatten().collect());
        }
        PersObject::KqInventory { data: wi, .. } => {
            let mut items = Vec::new();
            for entry in wi.items.into_iter().flatten() {
                if let PersObject::KqInventoryItem(it) = *entry.1 {
                    items.push(item_from_pers(&it, None, types));
                }
            }
            kw.world_items = Some(items);
        }
        PersObject::KqObject(o) => {
            if let Some(shape) = &o.shape_name {
                let low = shape.to_ascii_lowercase();
                if o.collider_type == 0
                    && low.ends_with(".kqs")
                    && (low.contains("dead") || low.contains("ided") || low.contains("death") || low.contains("cadth"))
                {
                    let name = o
                        .visible
                        .base
                        .name
                        .clone()
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "corpse".to_string());
                    let p = placement_from_object(&o);
                    kw.corpses.push(crate::fight::Corpse {
                        name,
                        shape: shape.clone(),
                        position: p.position,
                        rotation: p.rotation,
                        loaded: true,
                        solid: false,
                    });
                }
            }
        }
        _ => {}
    }
}

fn apply_ascii_section(
    s: &kq_text::Section,
    types: Option<&ItemTypes>,
    kw: &mut KeptWorldSnapshot,
) {
    let class = s.get("classType").map(str::trim).unwrap_or("");
    let num = |k: &str| s.number(k).and_then(Result::ok);
    let pos = [
        num("locX").unwrap_or(0.0) as f32,
        num("locY").unwrap_or(0.0) as f32,
        num("locZ").unwrap_or(0.0) as f32,
    ];
    let ang = [
        num("dirX").unwrap_or(0.0) as f32,
        num("dirY").unwrap_or(0.0) as f32,
        num("dirZ").unwrap_or(0.0) as f32,
    ];
    let placement = Placement {
        rotation: rotation(ang),
        position: pos,
    };
    let movement = s.get("currAnimName").map(str::trim).unwrap_or("").to_string();
    let phase = num("animPos").unwrap_or(0.0) as f32;
    let waiting = s
        .get("nextAnimName")
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let col = num("colliderType").map(|v| v as u32);

    if class.eq_ignore_ascii_case("KQDoor") {
        let state = DoorState::from_pers(num("state").unwrap_or(2.0) as i32);
        kw.doors.push(KeptDoorState {
            name: s.name.clone(),
            state,
            movement,
            phase,
            waiting,
            placement,
            collision_type: col.unwrap_or(crate::collide::DEFAULT_TYPE),
        });
    } else if class.eq_ignore_ascii_case("KQChest") {
        let state = ChestState::from_pers(num("chestState").unwrap_or(0.0) as i32);
        let remembered = s
            .get("invItemName")
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let hidden = num("hidden").is_some_and(|v| v != 0.0);
        kw.chests.push(KeptChestState {
            name: s.name.clone(),
            state,
            remembered,
            movement,
            phase,
            waiting,
            placement,
            hidden,
            collision_type: col.unwrap_or(crate::chest::CHEST_TYPE),
        });
    } else if class.eq_ignore_ascii_case("KQWorldInventory") {
        kw.world_items.get_or_insert_with(Vec::new);
    } else if class.eq_ignore_ascii_case("KQInventoryItem") {
        let raw_type = s.get("itemType").map(str::trim).unwrap_or("");
        let kind = resolve_item_kind(raw_type.parse::<i32>().unwrap_or(0), Some(raw_type), types);
        let item = ItemObject {
            name: s.name.clone(),
            kind,
            quantity: num("quantity").unwrap_or(1.0) as i64,
            third: num("userId").unwrap_or(-1.0) as i64,
            place: Place::World {
                position: pos,
                angles: ang,
            },
            hidden: num("hidden").is_some_and(|v| v != 0.0),
            script: s
                .get("script")
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            do_not_point_at: num("doNotPointAt").is_some_and(|v| v != 0.0),
            added: 0,
        };
        kw.world_items.get_or_insert_with(Vec::new).push(item);
    } else if class.eq_ignore_ascii_case("KQWorldState") {
        let n = num("nEvents").unwrap_or(0.0) as usize;
        for i in 0..n {
            if let Some(ev) = s.get(&format!("event{i}")).map(str::trim).filter(|v| !v.is_empty()) {
                kw.pending_events.push(ev.to_string());
            }
        }
    } else if class.eq_ignore_ascii_case("KQMusic") {
        let n = num("nMusic").unwrap_or(0.0) as usize;
        let mut stack = Vec::new();
        for i in 0..n {
            if let Some(m) = s.get(&format!("music{i}")).map(str::trim).filter(|v| !v.is_empty()) {
                stack.push(m.to_string());
            }
        }
        kw.music_stack = Some(stack);
    } else if class.eq_ignore_ascii_case("KQInterior") {
        if num("usingAltLighting").is_some_and(|v| v != 0.0) {
            kw.alt_lightmaps.insert(s.name.to_ascii_lowercase(), true);
        }
    } else if class.eq_ignore_ascii_case(crate::active::RIVER_CONTROL_CLASS) {
        if num("rustedOne").is_some_and(|v| v != 0.0) {
            kw.rusted_rivers.push(s.name.clone());
        }
    } else if class.eq_ignore_ascii_case("KQAnimObject")
        || class.eq_ignore_ascii_case("KQAnimOnce")
        || class.eq_ignore_ascii_case("KQBoulder")
        || class.eq_ignore_ascii_case("KQLavaTile")
        || class.eq_ignore_ascii_case("KQPortal")
        || kq8_data::characters::Class::of(class).is_some()
    {
        if !movement.is_empty() || phase != 0.0 || pos != [0.0; 3] {
            kw.states.push(KeptState {
                name: s.name.clone(),
                movement,
                phase,
                waiting,
                placement,
                collision_type: col,
            });
        }
    }
}

/// Imports an original `Mask.exe` `save<N>.vol` archive (binary `PERS` current world
/// + embedded ASCII kept worlds and maps) into a full [`SavedWorldSnapshot`].
pub fn import_original_save(
    bytes: &[u8],
    item_types: Option<&ItemTypes>,
) -> Result<SavedWorldSnapshot, String> {
    let pvol = Pvol::parse(bytes.to_vec())?;
    let chunk_bytes = |name: &str| -> Option<Vec<u8>> {
        let e = pvol.find(name)?;
        pvol.data(e).ok().map(|b| b.to_vec())
    };
    let world_raw = chunk_bytes("worldName")
        .map(|b| ascii_text(&b))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "save archive missing worldName chunk".to_string())?;
    let world = strip_kq_ext(&world_raw);

    let world_vars = chunk_bytes("WorldVar.cs")
        .map(|b| parse_world_var_cs(&b))
        .unwrap_or_default();

    let (next_no_name, difficulty_factor) = match chunk_bytes("GameState.kq") {
        Some(data) if data.starts_with(b"PERS") || data.starts_with(b"DICT") => {
            if let Ok(PersObject::KqGameState(gs)) = parse_pers(&data) {
                (gs.next_no_name.max(1) as usize, gs.difficulty_factor)
            } else {
                parse_game_state_kq(&data).unwrap_or((1, 1.0))
            }
        }
        Some(data) => parse_game_state_kq(&data).unwrap_or((1, 1.0)),
        None => (1, 1.0),
    };

    let mut cur_kept = KeptWorldSnapshot {
        vars: world_vars
            .iter()
            .filter(|(k, _)| crate::journey::belongs_to_world(k, &world))
            .cloned()
            .collect(),
        ..Default::default()
    };
    if let Some(nw) = chunk_bytes("newWorld.cs") {
        let (t, present) = parse_new_world_cs(&nw);
        cur_kept.current_time = t;
        cur_kept.present_objects = (!present.is_empty()).then_some(present);
    }

    let mut connor_at = None;
    let mut connor_pos = None;
    let mut connor_anim = None;
    let mut journey_opt: Option<Journey> = None;
    let mut rope: Option<crate::rope::Rope> = None;
    let mut rope_climb: Option<crate::rope::Climb> = None;
    let mut active_talk: Vec<crate::talk::SavedTalkQueue> = Vec::new();
    let mut camera: Option<SavedCameraState> = None;
    let mut extra_maps: Vec<(String, MapKeys)> = Vec::new();

    let kept_vol_names: BTreeSet<String> = chunk_bytes("worldVolumes")
        .map(|b| pipe_list(&b).into_iter().map(|s| s.to_ascii_lowercase()).collect())
        .unwrap_or_default();
    let extra_map_names: BTreeSet<String> = chunk_bytes("maps")
        .map(|b| pipe_list(&b).into_iter().map(|s| s.to_ascii_lowercase()).collect())
        .unwrap_or_default();

    for entry in &pvol.entries {
        let lower = entry.name.to_ascii_lowercase();
        let Ok(data) = pvol.data(entry) else { continue };
        if !lower.ends_with(".kq") {
            continue;
        }
        let is_pers = data.starts_with(b"PERS") || data.starts_with(b"DICT");
        if lower == "connor.kq" {
            if is_pers {
                if let Ok(PersObject::KqConner(c)) = parse_pers(data) {
                    let (at, pos, anim, j, r, rc, talk) =
                        apply_connor_pers(&c, &world, item_types, &world_vars);
                    connor_at = at;
                    connor_pos = pos;
                    connor_anim = anim;
                    journey_opt = Some(j);
                    rope = r;
                    rope_climb = rc;
                    active_talk = talk;
                }
            }
            continue;
        }
        if lower == "kqcamera.kq" {
            if is_pers {
                if let Ok(PersObject::KqCamMover(cam)) = parse_pers(data) {
                    let min_d = if cam.conner_min_dist > 0.0 {
                        cam.conner_min_dist
                    } else {
                        1500.0
                    };
                    let max_d = if cam.cam_move_radius > 0.0 {
                        cam.cam_move_radius
                    } else {
                        5000.0
                    };
                    camera = Some(SavedCameraState {
                        position: cam.loc,
                        heading: cam.dir[2],
                        view_tilt: cam.dir[0],
                        tilt: cam.camera_rotation[0],
                        roll: cam.dir[1],
                        following: cam.follow_object_name.is_some() && cam.pause_follow == 0,
                        follow_object: cam.follow_object_name.clone().filter(|s| !s.is_empty()),
                        range: (min_d, max_d),
                        mode_fixed: false,
                        first_person: cam.is_first_person != 0,
                        hold: cam.fix_height != 0,
                        offset: cam.third_camera_offset,
                    });
                }
            }
            continue;
        }
        if lower == "gamestate.kq" {
            continue;
        }
        if lower.ends_with("map.kq") || extra_map_names.contains(&lower) {
            if let Some(mk) = parse_map_chunk(data) {
                extra_maps.push((entry.name.clone(), mk));
            }
            continue;
        }
        if is_pers {
            let obj = parse_pers(data)
                .map_err(|e| format!("failed to parse binary PERS {}: {e}", entry.name))?;
            apply_pers_world_object(obj, item_types, &mut cur_kept);
        } else {
            for s in kq_text::parse(data) {
                apply_ascii_section(&s, item_types, &mut cur_kept);
            }
        }
    }

    let mut journey = journey_opt.unwrap_or_else(|| Journey {
        from: world.clone(),
        stats: None,
        items: Vec::new(),
        hand: None,
        weapon_ready: None,
        weapon_mode: WeaponMode::None,
        last_away_ranged: false,
        projectile: None,
        shoot_offset: [0.0; 3],
        potions: Vec::new(),
        pickup_experience: Vec::new(),
        flags: BTreeMap::new(),
        in_temple4: false,
        immunities: 0,
        raw_world_flags: None,
        map_marks: ConnorMapMarks::default(),
        map_view: START_VIEW,
        other_maps: BTreeMap::new(),
        combat_level: difficulty_factor,
        next_no_name,
        helps_asked: BTreeSet::new(),
        connor_tips: [false; 2],
        kill_tip: false,
        first_kill_remark: false,
        not_starting_help: 0,
        tip_bits: 0,
        god_mode: false,
        ai_off: false,
        allow_shadows: true,
        vars: world_vars
            .iter()
            .filter(|(k, _)| !crate::journey::belongs_to_world(k, &world))
            .cloned()
            .collect(),
    });
    journey.combat_level = difficulty_factor;
    journey.next_no_name = next_no_name;

    for (name, mk) in extra_maps {
        record_map(mk, &format!("imported from {name}"), &mut journey);
    }

    let mut kept_worlds = BTreeMap::new();
    for entry in &pvol.entries {
        let lower = entry.name.to_ascii_lowercase();
        if !lower.ends_with(".vol") && !kept_vol_names.contains(&lower) {
            continue;
        }
        if let Ok(data) = pvol.data(entry) {
            if let Ok((wkey, kw)) = parse_kept_vol(data.to_vec(), item_types, &mut journey, &entry.name) {
                kept_worlds.insert(wkey, kw);
            }
        }
    }
    kept_worlds.insert(world.to_ascii_lowercase(), cur_kept);

    Ok(SavedWorldSnapshot {
        world,
        connor_at,
        connor_pos,
        connor_anim,
        camera,
        rope,
        rope_climb,
        active_talk,
        journey,
        kept_worlds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn imports_synthetic_pers_save_archive() {
        use kq8_data::save_pers::*;
        let gs = PersObject::KqGameState(KqGameState {
            base: KqBase {
                name: Some("GameState".into()),
            },
            next_no_name: 77,
            difficulty_factor: 1.5,
        });
        let ws = PersObject::KqWorldState(KqWorldState {
            base: KqBase {
                name: Some("worldState".into()),
            },
            events: vec![Some("KQCueEvent Connor 12.5 2300 144 (null)".into())],
        });
        let door = PersObject::KqDoor(KqDoor {
            anim_object: KqAnimObject {
                object: KqObject {
                    visible: KqVisibleBase {
                        base: KqBase {
                            name: Some("Condoor".into()),
                        },
                        ..Default::default()
                    },
                    collider_type: 0x5021,
                    mover_type: 0,
                    mover: MoverData::Inline {
                        mover_name: Some("M".into()),
                        inline: KqMoverInline {
                            transform: TMat3F {
                                flags: 0,
                                rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                                position: [268623.0, 225311.0, 13981.0],
                            },
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                },
                anim_pos: 1.0,
                curr_anim_name: Some("opened".into()),
                ..Default::default()
            },
            state: 1,
        });
        let con = PersObject::KqConner(Box::new(KqConner {
            monster: KqMonster {
                anim_object: KqAnimObject {
                    object: KqObject {
                        visible: KqVisibleBase {
                            base: KqBase {
                                name: Some("Connor".into()),
                            },
                            ..Default::default()
                        },
                        stats: Some("40 50 10 5 200".into()),
                        mover_type: 0,
                        mover: MoverData::Inline {
                            mover_name: Some("C".into()),
                            inline: KqMoverInline {
                                transform: TMat3F {
                                    flags: 0,
                                    rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                                    position: [202336.0, 262300.0, 8209.0],
                                },
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    },
                    curr_anim_name: Some("Stop".into()),
                    anim_pos: 0.25,
                    ..Default::default()
                },
                do_mode: 4,
                inventory: Some((
                    "inv".into(),
                    Box::new(PersObject::KqInventory {
                        class: "KQConnerInventory".into(),
                        data: KqInventory {
                            base: KqBase {
                                name: Some("inv".into()),
                            },
                            owner_id: 0,
                            items: vec![Some((
                                "NoName5".into(),
                                Box::new(PersObject::KqInventoryItem(KqInventoryItem {
                                    object: KqObject {
                                        visible: KqVisibleBase {
                                            base: KqBase {
                                                name: Some("NoName5".into()),
                                            },
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    quantity: 3,
                                    user_id: u32::MAX,
                                    time_left: 0.0,
                                    item_type: 3020,
                                })),
                            ))],
                        },
                    }),
                )),
                ..Default::default()
            },
            level: 4,
            world_flags: 0x8,
            ..Default::default()
        }));

        let gs_b = write_pers(&gs);
        let ws_b = write_pers(&ws);
        let door_b = write_pers(&door);
        let con_b = write_pers(&con);
        let new_world = b"setCurrentTime 12.0\r\nloadKQ Condoor.kq\r\n";
        let world_var = b"set Daventry::girlhut::FirstTime 0\r\n";
        let vol = Pvol::build(&[
            ("savedGameName", b"synth"),
            ("worldName", b"Daventry.kq"),
            ("savedGameVersion", b"1.000000"),
            ("GameState.kq", &gs_b),
            ("worldState.kq", &ws_b),
            ("Condoor.kq", &door_b),
            ("Connor.kq", &con_b),
            ("newWorld.cs", new_world),
            ("WorldVar.cs", world_var),
            ("worldVolumes", b""),
            ("maps", b""),
        ]);

        let snap = import_original_save(&vol, None).unwrap();
        assert_eq!(snap.world, "Daventry");
        assert_eq!(snap.journey.next_no_name, 77);
        assert!((snap.journey.combat_level - 1.5).abs() < 0.001);
        assert_eq!(snap.journey.raw_world_flags, Some(0x8));
        assert_eq!(snap.journey.stats.unwrap().stats.0, [40, 50, 10, 5, 200]);
        assert_eq!(snap.connor_pos.unwrap().0, [202336.0, 262300.0, 8209.0]);
        let kw = snap.kept_worlds.get("daventry").unwrap();
        assert_eq!(kw.doors.len(), 1);
        assert_eq!(kw.doors[0].state, DoorState::Opened);
        assert_eq!(kw.pending_events.len(), 1);
    }

    #[test]
    fn imports_real_mask_saves_1_and_18_if_present() {
        let p1 = Path::new("../../MASK/save/save1.vol");
        let p18 = Path::new("../../MASK/save/save18.vol");
        if !p1.exists() || !p18.exists() {
            return;
        }
        let itl_bytes = std::fs::read("../../MASK/GAME/Kq/Invinfo.itl").ok();
        let types = itl_bytes.as_deref().map(|b| ItemTypes::parse(b).0);

        let b1 = std::fs::read(p1).unwrap();
        let s1 = import_original_save(&b1, types.as_ref()).unwrap();
        assert_eq!(s1.world, "Daventry");
        assert_eq!(s1.journey.next_no_name, 439);
        assert!((s1.journey.combat_level - 0.66).abs() < 0.01);
        assert_eq!(s1.journey.stats.unwrap().stats.0, [10, 10, 0, 0, 0]);
        let cam1 = s1.camera.as_ref().unwrap();
        assert!((cam1.heading - (-1.8)).abs() < 0.01);
        assert!((cam1.view_tilt - (-0.1278)).abs() < 0.01);
        assert!((cam1.tilt - (-0.2)).abs() < 0.01);
        let kw1 = s1.kept_worlds.get("daventry").unwrap();
        assert!(kw1.doors.iter().any(|d| d.name.eq_ignore_ascii_case("Condoor")));
        assert_eq!(kw1.pending_events.len(), 2);

        let b18 = std::fs::read(p18).unwrap();
        let s18 = import_original_save(&b18, types.as_ref()).unwrap();
        assert_eq!(s18.world, "Swamp");
        assert_eq!(s18.journey.raw_world_flags, Some(0x420038));
        assert!(s18.kept_worlds.contains_key("castled"));
        assert!(s18.kept_worlds.contains_key("daventry"));
        assert!(s18.kept_worlds.contains_key("deadcity"));
        assert!(s18.kept_worlds.contains_key("swamp"));
        assert!(s18.journey.other_maps.len() >= 3);
    }
}
