//! Exports a [`SavedWorldSnapshot`] into a `Mask.exe`-compatible `save<N>.vol`
//! archive: binary `PERS` chunks for the current world, ASCII `.vol` kept-world
//! archives, ASCII `.kq` maps, `WorldVar.cs` / `newWorld.cs`, `current.bin`,
//! and a lightweight `kq8r_meta.txt` chunk for KQ8R-specific extras
//! (`docs/behaviour/savefile.md`, M4 step 4h-4).

use std::collections::BTreeSet;

use kq8_data::items::ItemTypes;
use kq8_data::objects::{angles_of, rotation};
use kq8_data::pvol::Pvol;
use kq8_data::save_pers::{
    write_pers, KqAnimObject, KqBase, KqCamMover, KqChest, KqConner, KqDoor, KqGameState,
    KqInterior, KqInventory, KqInventoryItem, KqMap, KqMonster, KqMoverInline, KqMusic, KqObject,
    KqRiverControl, KqState, KqStateKind, KqVisibleBase, KqWorldState, MoverData, PersObject,
    TMat3F,
};

use crate::collide::Placement;
use crate::item::{ItemObject, Place};
use crate::magic_map::{View, TELEPORT_MARKS};
use crate::potions::{Effect, Running};
use crate::rope::{Climb, End, Rope, Stage, Status as RopeStatus};
use crate::save_game::{
    esc, fmt_f32_slice, fmt_opt_str, format_world_var_cs, parse_f32_hex_or_dec, parse_opt_str,
    unesc, SavedWorldSnapshot,
};
use crate::weapons::WeaponMode;

const META_HEADER: &str = "KQ8R_META_V1";

fn sanitize_stem(raw: &str, fallback: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or(raw).trim();
    let trimmed = base
        .strip_suffix(".kq")
        .or_else(|| base.strip_suffix(".KQ"))
        .unwrap_or(base)
        .trim();
    let clean: String = trimmed
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '|' | '\0' => '_',
            _ => c,
        })
        .collect();
    if clean.is_empty() {
        fallback.to_string()
    } else {
        clean
    }
}

fn unique_chunk_name(used: &mut BTreeSet<String>, stem: &str, ext: &str) -> String {
    let base = sanitize_stem(stem, "obj");
    let candidate = format!("{base}{ext}");
    let lower = candidate.to_ascii_lowercase();
    if used.insert(lower) {
        return candidate;
    }
    let mut n = 2usize;
    loop {
        let next = format!("{base}_{n}{ext}");
        if used.insert(next.to_ascii_lowercase()) {
            return next;
        }
        n += 1;
    }
}

fn kq_base(name: &str) -> KqBase {
    KqBase {
        name: (!name.is_empty()).then(|| name.to_string()),
    }
}

fn kq_visible(name: &str, hidden: bool) -> KqVisibleBase {
    KqVisibleBase {
        base: kq_base(name),
        active_3d_flag: 0,
        is_far: 0,
        too_far: 0.0,
        translucency: 256,
        haze_multiplier: 1.0,
        b_haze_solid: 0,
        hidden: i32::from(hidden),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_kq_object(
    name: &str,
    hidden: bool,
    placement: &Placement,
    collider_type: u32,
    script: Option<String>,
    do_not_point_at: bool,
    immunities: u32,
    stats: Option<String>,
) -> KqObject {
    KqObject {
        visible: kq_visible(name, hidden),
        collider_type,
        script,
        do_not_point_at: i32::from(do_not_point_at),
        immunities,
        stats,
        mover_type: 0,
        mover: MoverData::Inline {
            mover_name: None,
            inline: KqMoverInline {
                transform: TMat3F {
                    flags: 0,
                    rotation: placement.rotation,
                    position: placement.position,
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

fn build_anim_object(
    object: KqObject,
    movement: &str,
    phase: f32,
    waiting: Option<&str>,
) -> KqAnimObject {
    KqAnimObject {
        object,
        anim_pos: phase,
        curr_anim_name: (!movement.is_empty()).then(|| movement.to_string()),
        next_anim_name: waiting.filter(|s| !s.is_empty()).map(str::to_string),
        ..Default::default()
    }
}

fn item_type_id(kind: &str, item_types: Option<&ItemTypes>) -> i32 {
    if let Some(id) = item_types.and_then(|t| t.find(kind)).and_then(|it| it.id) {
        return id as i32;
    }
    kind.trim().parse::<i32>().unwrap_or(0)
}

fn build_pers_item(item: &ItemObject, item_types: Option<&ItemTypes>) -> (String, Box<PersObject>) {
    let placement = match &item.place {
        Place::World { position, angles } => Placement {
            rotation: rotation(*angles),
            position: *position,
        },
        Place::Carried(_) | Place::Gone => Placement::at([0.0; 3]),
    };
    let obj = build_kq_object(
        &item.name,
        item.hidden,
        &placement,
        crate::item::ITEM_TYPE,
        item.script.clone(),
        item.do_not_point_at,
        0,
        None,
    );
    let pers = PersObject::KqInventoryItem(KqInventoryItem {
        object: obj,
        quantity: item.quantity as i32,
        user_id: item.third as i32 as u32,
        time_left: 0.0,
        item_type: item_type_id(&item.kind, item_types),
    });
    (item.name.clone(), Box::new(pers))
}

fn build_connor_pers(snap: &SavedWorldSnapshot, item_types: Option<&ItemTypes>) -> PersObject {
    let placement = if let Some((pos, heading)) = snap.connor_pos {
        Placement {
            rotation: rotation([0.0, 0.0, heading]),
            position: pos,
        }
    } else if let Some([x, y, heading]) = snap.connor_at {
        Placement {
            rotation: rotation([0.0, 0.0, heading]),
            position: [x, y, 0.0],
        }
    } else {
        Placement::at([0.0; 3])
    };

    let j = &snap.journey;
    let stats_str = j.stats.as_ref().map(|s| {
        format!(
            "{} {} {} {} {}",
            s.stats.0[0], s.stats.0[1], s.stats.0[2], s.stats.0[3], s.stats.0[4]
        )
    });
    let (movement, phase) = snap
        .connor_anim
        .as_ref()
        .map_or(("Stop", 0.0), |(m, p)| (m.as_str(), *p));

    let obj = build_kq_object(
        "Connor",
        false,
        &placement,
        crate::collide::CONNOR_TYPE,
        None,
        false,
        u32::from(j.immunities),
        stats_str,
    );
    let anim_object = build_anim_object(obj, movement, phase, None);

    let mut inv_items = Vec::new();
    for item in &j.items {
        if let Place::Carried(who) = &item.place {
            if who.eq_ignore_ascii_case("Connor") || who.is_empty() {
                inv_items.push(Some(build_pers_item(item, item_types)));
            }
        }
    }
    let inventory = Some((
        "KQConnerInventory".to_string(),
        Box::new(PersObject::KqInventory {
            class: "KQConnerInventory".to_string(),
            data: KqInventory {
                base: kq_base("KQConnerInventory"),
                owner_id: 0,
                items: inv_items,
            },
        }),
    ));

    let spell_secs = |kind: &str| -> i32 {
        j.potions
            .iter()
            .find(|(r, _)| r.kind.eq_ignore_ascii_case(kind))
            .map_or(0, |(r, _)| r.left.max(0) as i32)
    };

    let state = snap.rope_climb.as_ref().map(|cl| {
        (
            "KQStateConnerHookRope".to_string(),
            Box::new(PersObject::KqState(KqState {
                base: kq_base("KQStateConnerHookRope"),
                from_state: 0,
                kind: KqStateKind::ConnerHookRope {
                    done_roping: i32::from(cl.done),
                    reached_top: 0,
                    do_mode_before: cl.hand_before.number() as i32,
                    old_cam_pos: snap.camera.as_ref().map_or([0.0; 3], |c| c.position),
                    con_pos: cl.aim,
                    on_rope: i32::from(cl.stage == Stage::Hanging),
                    hide_icon: i32::from(cl.stage == Stage::Hanging),
                    first_to_third: i32::from(cl.first_to_third),
                    bottom_cam_limit: cl.range.0,
                    top_cam_limit: cl.range.1,
                    con_cam_bottom_limit: cl.bottom_limit,
                    con_cam_top_limit: cl.top_limit,
                    rope_cam_loc1: cl.cam[0].unwrap_or([0.0; 3]),
                    rope_cam_loc2: cl.cam[1].unwrap_or([0.0; 3]),
                    orig_step_up: cl.saved_steps.map_or(0.0, |s| s.0),
                    orig_step_down: cl.saved_steps.map_or(0.0, |s| s.1),
                },
            })),
        )
    });

    let monster = KqMonster {
        anim_object,
        state,
        do_mode: j.weapon_mode.number() as i32,
        active: 1,
        inventory,
        ..Default::default()
    };

    let r = snap.rope.as_ref();
    let (rope_cam_ang, rope_cam_range) = r.and_then(|r| r.angle).unwrap_or((0.0, 0.0));

    PersObject::KqConner(Box::new(KqConner {
        monster,
        allow_shadows_in_game: u32::from(j.allow_shadows),
        first_person: i32::from(snap.camera.as_ref().is_some_and(|c| c.first_person)),
        rope_status: r.map_or(0, |r| r.status.to_pers()),
        rope_loc: r.map_or([0.0; 3], |r| r.place),
        rope_dir: r.map_or([0.0; 3], |r| r.dir),
        rope_cam_loc1: r.and_then(|r| r.cam[0]).unwrap_or([0.0; 3]),
        rope_cam_loc2: r.and_then(|r| r.cam[1]).unwrap_or([0.0; 3]),
        rope_cam2_loc: r.and_then(|r| r.top_cam).unwrap_or([0.0; 3]),
        rope_cam_ang,
        rope_cam_range,
        rope_cam_top_limit: r.map_or(0.0, |r| r.top_limit),
        rope_cam_bottom_limit: r.map_or(0.0, |r| r.bottom_limit),
        start_help: i32::from(j.not_starting_help),
        world_flags: j.raw_world_flags.unwrap_or(0),
        was_in_bow_mode: i32::from(j.last_away_ranged),
        level: j.stats.as_ref().map_or(1, |s| s.level.max(1) as i32),
        item_in_hand: j.hand.clone(),
        clarity_spell: spell_secs("INVITEM_Clarity"),
        strength_spell: spell_secs("INVITEM_Strength"),
        invulnerability_spell: spell_secs("INVITEM_Invulnerable"),
        invisibility_spell: spell_secs("INVITEM_Invisible"),
        ..Default::default()
    }))
}

fn format_ascii_anim_fields(
    out: &mut String,
    placement: &Placement,
    movement: &str,
    phase: f32,
    waiting: Option<&str>,
    collider_type: Option<u32>,
) {
    let pos = placement.position;
    let ang = angles_of(placement.rotation);
    out.push_str(&format!(
        "locX={}\r\nlocY={}\r\nlocZ={}\r\n",
        pos[0], pos[1], pos[2]
    ));
    out.push_str(&format!(
        "dirX={}\r\ndirY={}\r\ndirZ={}\r\n",
        ang[0], ang[1], ang[2]
    ));
    if !movement.is_empty() {
        out.push_str(&format!("currAnimName={movement}\r\n"));
    }
    out.push_str(&format!("animPos={phase}\r\n"));
    if let Some(w) = waiting.filter(|s| !s.is_empty()) {
        out.push_str(&format!("nextAnimName={w}\r\n"));
    }
    if let Some(col) = collider_type {
        out.push_str(&format!("colliderType={col}\r\n"));
    }
}

fn build_new_world_cs(current_time: Option<f64>) -> Vec<u8> {
    if let Some(t) = current_time {
        format!("setCurrentTime {t}\r\n").into_bytes()
    } else {
        Vec::new()
    }
}

fn build_ascii_kept_vol(
    world_key: &str,
    kw: &crate::save_game::KeptWorldSnapshot,
    world_vars_override: Option<&[(String, String)]>,
) -> Vec<u8> {
    let mut owned: Vec<(String, Vec<u8>)> = Vec::new();
    let world_kq = format!("{world_key}.kq");
    owned.push(("worldName".into(), world_kq.into_bytes()));
    let vars = world_vars_override.unwrap_or(&kw.vars);
    owned.push(("WorldVar.cs".into(), format_world_var_cs(vars).into_bytes()));
    owned.push(("newWorld.cs".into(), build_new_world_cs(kw.current_time)));

    let mut used = BTreeSet::new();
    for reserved in ["worldname", "worldvar.cs", "newworld.cs"] {
        used.insert(reserved.to_string());
    }

    for d in &kw.doors {
        let mut s = format!("[{}]\r\nclassType=KQDoor\r\nstate={}\r\n", d.name, d.state.to_pers());
        format_ascii_anim_fields(
            &mut s,
            &d.placement,
            &d.movement,
            d.phase,
            d.waiting.as_deref(),
            Some(d.collision_type),
        );
        let chunk = unique_chunk_name(&mut used, &d.name, ".kq");
        owned.push((chunk, s.into_bytes()));
    }

    for c in &kw.chests {
        let mut s = format!(
            "[{}]\r\nclassType=KQChest\r\nchestState={}\r\nhidden={}\r\n",
            c.name,
            c.state.to_pers(),
            u8::from(c.hidden)
        );
        if let Some(rem) = c.remembered.as_deref().filter(|v| !v.is_empty()) {
            s.push_str(&format!("invItemName={rem}\r\n"));
        }
        format_ascii_anim_fields(
            &mut s,
            &c.placement,
            &c.movement,
            c.phase,
            c.waiting.as_deref(),
            Some(c.collision_type),
        );
        let chunk = unique_chunk_name(&mut used, &c.name, ".kq");
        owned.push((chunk, s.into_bytes()));
    }

    for st in &kw.states {
        if st.name.is_empty() {
            continue;
        }
        let mut s = format!("[{}]\r\nclassType=KQAnimObject\r\n", st.name);
        format_ascii_anim_fields(
            &mut s,
            &st.placement,
            &st.movement,
            st.phase,
            st.waiting.as_deref(),
            st.collision_type,
        );
        let chunk = unique_chunk_name(&mut used, &st.name, ".kq");
        owned.push((chunk, s.into_bytes()));
    }

    if let Some(items) = &kw.world_items {
        let mut s = String::from("[WorldInventory]\r\nclassType=KQWorldInventory\r\nownerId=-1\r\n");
        for item in items {
            let (pos, ang) = match &item.place {
                Place::World { position, angles } => (*position, *angles),
                Place::Carried(_) | Place::Gone => ([0.0; 3], [0.0; 3]),
            };
            s.push_str(&format!(
                "\r\n[{}]\r\nclassType=KQInventoryItem\r\nitemType={}\r\nquantity={}\r\nuserId={}\r\nhidden={}\r\ndoNotPointAt={}\r\nlocX={}\r\nlocY={}\r\nlocZ={}\r\ndirX={}\r\ndirY={}\r\ndirZ={}\r\n",
                item.name,
                item.kind,
                item.quantity,
                item.third,
                u8::from(item.hidden),
                u8::from(item.do_not_point_at),
                pos[0],
                pos[1],
                pos[2],
                ang[0],
                ang[1],
                ang[2],
            ));
            if let Some(sc) = item.script.as_deref().filter(|v| !v.is_empty()) {
                s.push_str(&format!("script={sc}\r\n"));
            }
        }
        let chunk = unique_chunk_name(&mut used, "WorldInventory", ".kq");
        owned.push((chunk, s.into_bytes()));
    }

    if !kw.pending_events.is_empty() {
        let mut s = format!(
            "[worldState]\r\nclassType=KQWorldState\r\nnEvents={}\r\n",
            kw.pending_events.len()
        );
        for (i, ev) in kw.pending_events.iter().enumerate() {
            s.push_str(&format!("event{i}={ev}\r\n"));
        }
        let chunk = unique_chunk_name(&mut used, "worldState", ".kq");
        owned.push((chunk, s.into_bytes()));
    }

    if let Some(stack) = &kw.music_stack {
        let mut s = format!("[Music]\r\nclassType=KQMusic\r\nnMusic={}\r\n", stack.len());
        for (i, m) in stack.iter().enumerate() {
            s.push_str(&format!("music{i}={m}\r\n"));
        }
        let chunk = unique_chunk_name(&mut used, "Music", ".kq");
        owned.push((chunk, s.into_bytes()));
    }

    for (name, &alt) in &kw.alt_lightmaps {
        if alt {
            let s = format!("[{name}]\r\nclassType=KQInterior\r\nusingAltLighting=1\r\n");
            let chunk = unique_chunk_name(&mut used, name, ".kq");
            owned.push((chunk, s.into_bytes()));
        }
    }

    for name in &kw.rusted_rivers {
        let s = format!(
            "[{name}]\r\nclassType={}\r\nrustedOne=1\r\n",
            crate::active::RIVER_CONTROL_CLASS
        );
        let chunk = unique_chunk_name(&mut used, name, ".kq");
        owned.push((chunk, s.into_bytes()));
    }

    let refs: Vec<(&str, &[u8])> = owned.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
    Pvol::build(&refs)
}

fn format_kq8r_meta(snap: &SavedWorldSnapshot) -> Vec<u8> {
    let mut out = String::new();
    out.push_str(META_HEADER);
    out.push('\n');

    let j = &snap.journey;
    out.push_str(&format!(
        "meta_journey\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        esc(&j.from),
        j.combat_level.to_bits(),
        j.next_no_name,
        u8::from(j.in_temple4),
        j.tip_bits,
        u8::from(j.god_mode),
        u8::from(j.ai_off),
        u8::from(j.allow_shadows),
        u8::from(j.kill_tip),
        u8::from(j.connor_tips[0]),
        u8::from(j.connor_tips[1]),
        j.shoot_offset[0].to_bits(),
        j.shoot_offset[1].to_bits(),
        j.shoot_offset[2].to_bits(),
        fmt_opt_str(&j.weapon_ready),
    ));
    if let Some(p) = &j.projectile {
        out.push_str(&format!("projectile\t{}\n", esc(p)));
    }
    for (i, item) in j.items.iter().enumerate() {
        out.push_str(&format!(
            "item_meta\t{}\t{}\t{}\t{}\t{}\n",
            i,
            esc(&item.name),
            esc(&item.kind),
            item.added,
            match &item.place {
                Place::Carried(who) => esc(who),
                Place::World { .. } | Place::Gone => "-".into(),
            }
        ));
    }
    for (r, until) in &j.potions {
        let place_s = r.place.map_or_else(|| "-".into(), |p| p.to_string());
        out.push_str(&format!(
            "potion\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            esc(&r.item),
            esc(&r.kind),
            r.duration,
            r.left,
            r.next_tick,
            place_s,
            r.added,
            until,
        ));
    }
    for (secs, amt, kind) in &j.pickup_experience {
        out.push_str(&format!(
            "pickup_exp\t{}\t{}\t{}\n",
            secs.to_bits(),
            amt,
            esc(kind)
        ));
    }
    for (k, &v) in &j.flags {
        out.push_str(&format!("flag\t{}\t{}\n", esc(k), u8::from(v)));
    }
    out.push_str(&format!(
        "map_view\t{}\t{}\t{}\t{}\n",
        j.map_view.left.to_bits(),
        j.map_view.top.to_bits(),
        j.map_view.right.to_bits(),
        j.map_view.bottom.to_bits(),
    ));
    let vis: Vec<&str> = j
        .map_marks
        .visited
        .iter()
        .map(|&b| if b { "1" } else { "0" })
        .collect();
    let tel: Vec<&str> = j
        .map_marks
        .teleport_earned
        .iter()
        .map(|&b| if b { "1" } else { "0" })
        .collect();
    out.push_str(&format!(
        "map_marks\t{}\t{}\t{}\n",
        vis.join(","),
        tel.join(","),
        u8::from(j.map_marks.sage_gnome)
    ));
    if let Some(cam) = &snap.camera {
        out.push_str(&format!(
            "cam_meta\t{}\t{}\t{}\t{}\t{}\t{}\n",
            u8::from(cam.mode_fixed),
            cam.range.0.to_bits(),
            cam.range.1.to_bits(),
            cam.heading.to_bits(),
            cam.view_tilt.to_bits(),
            cam.tilt.to_bits(),
        ));
    }
    if let Some((anim, phase)) = &snap.connor_anim {
        out.push_str(&format!("connor_anim\t{}\t{}\n", esc(anim), phase.to_bits()));
    }
    if let Some(([x, y, z], h)) = snap.connor_pos {
        out.push_str(&format!(
            "connor_pos\t{}\t{}\t{}\t{}\n",
            x.to_bits(),
            y.to_bits(),
            z.to_bits(),
            h.to_bits()
        ));
    }
    if let Some(r) = &snap.rope {
        let fmt_opt3 = |v: Option<[f32; 3]>| {
            v.map_or_else(|| "-".into(), |a| fmt_f32_slice(&a))
        };
        let ang_s = r.angle.map_or_else(
            || "-".into(),
            |(a, b)| format!("{},{}", a.to_bits(), b.to_bits()),
        );
        let at_s = r.changed_at.map_or_else(|| "-".into(), |t| t.to_bits().to_string());
        out.push_str(&format!(
            "rope\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.status.to_pers(),
            fmt_f32_slice(&r.place),
            fmt_f32_slice(&r.dir),
            fmt_opt3(r.cam[0]),
            fmt_opt3(r.cam[1]),
            fmt_opt3(r.top_cam),
            ang_s,
            r.top_limit.to_bits(),
            r.bottom_limit.to_bits(),
            at_s,
        ));
    }
    if let Some(cl) = &snap.rope_climb {
        let fmt_opt3 = |v: Option<[f32; 3]>| {
            v.map_or_else(|| "-".into(), |a| fmt_f32_slice(&a))
        };
        let steps_s = cl.saved_steps.map_or_else(
            || "-".into(),
            |(a, b)| format!("{},{}", a.to_bits(), b.to_bits()),
        );
        out.push_str(&format!(
            "rope_climb\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            cl.from.as_str(),
            cl.stage.as_str(),
            fmt_f32_slice(&cl.place),
            fmt_f32_slice(&cl.dir),
            fmt_opt3(cl.cam[0]),
            fmt_opt3(cl.cam[1]),
            cl.top_limit.to_bits(),
            cl.bottom_limit.to_bits(),
            cl.hand_before.number(),
            fmt_opt_str(&cl.weapon_before),
            u8::from(cl.first_to_third),
            fmt_f32_slice(&cl.aim),
            format!("{},{}", cl.range.0.to_bits(), cl.range.1.to_bits()),
            steps_s,
            u8::from(cl.done),
        ));
    }

    for (wname, kw) in &snap.kept_worlds {
        for g in &kw.gone {
            out.push_str(&format!("kw_gone\t{}\t{}\n", esc(wname), esc(g)));
        }
        for c in &kw.came_in {
            out.push_str(&format!("kw_came\t{}\t{}\n", esc(wname), esc(c)));
        }
        for (a, &v) in &kw.alt_lightmaps {
            if !v {
                out.push_str(&format!(
                    "kw_alt\t{}\t{}\t{}\n",
                    esc(wname),
                    esc(a),
                    u8::from(v)
                ));
            }
        }
        if let Some(present) = &kw.present_objects {
            let joined: Vec<String> = present.iter().map(|s| esc(s)).collect();
            out.push_str(&format!("kw_present\t{}\t{}\n", esc(wname), joined.join(",")));
        }
        if let Some(items) = &kw.world_items {
            for (i, item) in items.iter().enumerate() {
                out.push_str(&format!(
                    "kw_item_meta\t{}\t{}\t{}\t{}\n",
                    esc(wname),
                    i,
                    esc(&item.kind),
                    item.added
                ));
            }
        }
    }

    let _ = (fmt_f32_slice, parse_f32_hex_or_dec::<3>);
    out.into_bytes()
}

/// Applies KQ8R-specific metadata (`kq8r_meta.txt`) on top of a [`SavedWorldSnapshot`]
/// parsed by [`crate::save_import::import_original_save`].
pub fn apply_kq8r_meta(snap: &mut SavedWorldSnapshot, bytes: &[u8]) -> Result<(), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "kq8r_meta.txt is not UTF-8")?;
    let mut lines = text.lines();
    let Some(first) = lines.next().map(str::trim) else {
        return Err("empty kq8r_meta.txt".into());
    };
    if first != META_HEADER {
        return Err(format!("unexpected kq8r_meta.txt header '{first}'"));
    }

    let parse_f32_bits = |s: &str| -> Result<f32, String> {
        s.parse::<u32>()
            .map(f32::from_bits)
            .map_err(|_| format!("bad f32 bits '{s}'"))
    };
    let parse_f64_bits = |s: &str| -> Result<f64, String> {
        s.parse::<u64>()
            .map(f64::from_bits)
            .map_err(|_| format!("bad f64 bits '{s}'"))
    };
    let parse_bool = |s: &str| -> bool { s == "1" || s.eq_ignore_ascii_case("true") };

    let mut replaced_potions = false;
    let mut cleared_flags = false;
    let mut cleared_pickup = false;

    for raw in lines {
        if raw.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = raw.split('\t').collect();
        match f[0] {
            "meta_journey" if f.len() >= 16 => {
                snap.journey.from = unesc(f[1]);
                snap.journey.combat_level = parse_f32_bits(f[2])?;
                snap.journey.next_no_name = f[3].parse().map_err(|_| "bad next_no_name")?;
                snap.journey.in_temple4 = parse_bool(f[4]);
                snap.journey.tip_bits = f[5].parse().map_err(|_| "bad tip_bits")?;
                snap.journey.god_mode = parse_bool(f[6]);
                snap.journey.ai_off = parse_bool(f[7]);
                snap.journey.allow_shadows = parse_bool(f[8]);
                snap.journey.kill_tip = parse_bool(f[9]);
                snap.journey.connor_tips = [parse_bool(f[10]), parse_bool(f[11])];
                snap.journey.shoot_offset = [
                    parse_f32_bits(f[12])?,
                    parse_f32_bits(f[13])?,
                    parse_f32_bits(f[14])?,
                ];
                snap.journey.weapon_ready = parse_opt_str(f[15]);
            }
            "projectile" if f.len() >= 2 => {
                snap.journey.projectile = Some(unesc(f[1]));
            }
            "item_meta" if f.len() >= 6 => {
                if let Ok(idx) = f[1].parse::<usize>() {
                    if let Some(it) = snap.journey.items.get_mut(idx) {
                        it.kind = unesc(f[3]);
                        if let Ok(added) = f[4].parse::<u64>() {
                            it.added = added;
                        }
                        if f[5] != "-" {
                            it.place = Place::Carried(unesc(f[5]));
                        }
                    }
                }
            }
            "potion" if f.len() >= 9 => {
                if !replaced_potions {
                    snap.journey.potions.clear();
                    replaced_potions = true;
                }
                let kind = unesc(f[2]);
                let place = if f[6] == "-" { None } else { f[6].parse().ok() };
                let until: u64 = f[8].parse().map_err(|_| "bad potion until")?;
                snap.journey.potions.push((
                    Running {
                        item: unesc(f[1]),
                        effect: Effect::of(&kind),
                        kind,
                        duration: f[3].parse().map_err(|_| "bad potion dur")?,
                        left: f[4].parse().map_err(|_| "bad potion left")?,
                        next_tick: f[5].parse().map_err(|_| "bad potion next_tick")?,
                        place,
                        added: f[7].parse().map_err(|_| "bad potion added")?,
                    },
                    until,
                ));
            }
            "pickup_exp" if f.len() >= 4 => {
                if !cleared_pickup {
                    snap.journey.pickup_experience.clear();
                    cleared_pickup = true;
                }
                snap.journey.pickup_experience.push((
                    parse_f64_bits(f[1])?,
                    f[2].parse().map_err(|_| "bad pickup amt")?,
                    unesc(f[3]),
                ));
            }
            "flag" if f.len() >= 3 => {
                if !cleared_flags {
                    snap.journey.flags.clear();
                    cleared_flags = true;
                }
                snap.journey.flags.insert(unesc(f[1]), parse_bool(f[2]));
            }
            "map_view" if f.len() >= 5 => {
                snap.journey.map_view = View {
                    left: parse_f64_bits(f[1])?,
                    top: parse_f64_bits(f[2])?,
                    right: parse_f64_bits(f[3])?,
                    bottom: parse_f64_bits(f[4])?,
                };
            }
            "map_marks" if f.len() >= 4 => {
                for (slot, b) in snap
                    .journey
                    .map_marks
                    .visited
                    .iter_mut()
                    .zip(f[1].split(','))
                {
                    *slot = parse_bool(b);
                }
                for (slot, b) in snap
                    .journey
                    .map_marks
                    .teleport_earned
                    .iter_mut()
                    .zip(f[2].split(','))
                {
                    *slot = parse_bool(b);
                }
                if snap.journey.map_marks.teleport_earned.len() > TELEPORT_MARKS {
                    snap.journey.map_marks.teleport_earned[TELEPORT_MARKS..].fill(false);
                }
                snap.journey.map_marks.sage_gnome = parse_bool(f[3]);
            }
            "cam_meta" if f.len() >= 7 => {
                if let Some(cam) = &mut snap.camera {
                    cam.mode_fixed = parse_bool(f[1]);
                    cam.range = (parse_f32_bits(f[2])?, parse_f32_bits(f[3])?);
                    cam.heading = parse_f32_bits(f[4])?;
                    cam.view_tilt = parse_f32_bits(f[5])?;
                    cam.tilt = parse_f32_bits(f[6])?;
                }
            }
            "connor_anim" if f.len() >= 3 => {
                snap.connor_anim = Some((unesc(f[1]), parse_f32_bits(f[2])?));
            }
            "connor_pos" if f.len() >= 5 => {
                let p = [
                    parse_f32_bits(f[1])?,
                    parse_f32_bits(f[2])?,
                    parse_f32_bits(f[3])?,
                ];
                let h = parse_f32_bits(f[4])?;
                snap.connor_pos = Some((p, h));
                snap.connor_at = Some([p[0], p[1], h]);
            }
            "rope" if f.len() >= 11 => {
                let status = RopeStatus::from_pers(f[1].parse().unwrap_or(0));
                let opt3 = |s: &str| -> Result<Option<[f32; 3]>, String> {
                    if s == "-" {
                        Ok(None)
                    } else {
                        Ok(Some(parse_f32_hex_or_dec::<3>(s)?))
                    }
                };
                let angle = if f[7] == "-" {
                    None
                } else {
                    let (a, b) = f[7].split_once(',').ok_or("bad rope angle")?;
                    Some((parse_f32_bits(a)?, parse_f32_bits(b)?))
                };
                let changed_at = if f[10] == "-" {
                    None
                } else {
                    Some(parse_f64_bits(f[10])?)
                };
                snap.rope = Some(Rope {
                    status,
                    place: parse_f32_hex_or_dec::<3>(f[2])?,
                    dir: parse_f32_hex_or_dec::<3>(f[3])?,
                    cam: [opt3(f[4])?, opt3(f[5])?],
                    top_cam: opt3(f[6])?,
                    angle,
                    top_limit: parse_f32_bits(f[8])?,
                    bottom_limit: parse_f32_bits(f[9])?,
                    changed_at,
                });
            }
            "rope_climb" if f.len() >= 16 => {
                let from = End::from_str(f[1]).ok_or_else(|| format!("bad rope end '{}'", f[1]))?;
                let stage =
                    Stage::from_str(f[2]).ok_or_else(|| format!("bad rope stage '{}'", f[2]))?;
                let opt3 = |s: &str| -> Result<Option<[f32; 3]>, String> {
                    if s == "-" {
                        Ok(None)
                    } else {
                        Ok(Some(parse_f32_hex_or_dec::<3>(s)?))
                    }
                };
                let (r0, r1) = f[13].split_once(',').ok_or("bad rope range")?;
                let saved_steps = if f[14] == "-" {
                    None
                } else {
                    let (a, b) = f[14].split_once(',').ok_or("bad rope steps")?;
                    Some((parse_f32_bits(a)?, parse_f32_bits(b)?))
                };
                snap.rope_climb = Some(Climb {
                    from,
                    stage,
                    place: parse_f32_hex_or_dec::<3>(f[3])?,
                    dir: parse_f32_hex_or_dec::<3>(f[4])?,
                    cam: [opt3(f[5])?, opt3(f[6])?],
                    top_limit: parse_f32_bits(f[7])?,
                    bottom_limit: parse_f32_bits(f[8])?,
                    hand_before: WeaponMode::from_number(f[9].parse().unwrap_or(4))
                        .unwrap_or(WeaponMode::None),
                    weapon_before: parse_opt_str(f[10]),
                    first_to_third: parse_bool(f[11]),
                    aim: parse_f32_hex_or_dec::<3>(f[12])?,
                    range: (parse_f32_bits(r0)?, parse_f32_bits(r1)?),
                    saved_steps,
                    done: parse_bool(f[15]),
                });
            }
            "kw_gone" if f.len() >= 3 => {
                snap.kept_worlds
                    .entry(unesc(f[1]))
                    .or_default()
                    .gone
                    .insert(unesc(f[2]));
            }
            "kw_came" if f.len() >= 3 => {
                snap.kept_worlds
                    .entry(unesc(f[1]))
                    .or_default()
                    .came_in
                    .push(unesc(f[2]));
            }
            "kw_alt" if f.len() >= 4 => {
                snap.kept_worlds
                    .entry(unesc(f[1]))
                    .or_default()
                    .alt_lightmaps
                    .insert(unesc(f[2]), parse_bool(f[3]));
            }
            "kw_present" if f.len() >= 3 => {
                let set = f[2]
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(unesc)
                    .collect::<BTreeSet<String>>();
                snap.kept_worlds
                    .entry(unesc(f[1]))
                    .or_default()
                    .present_objects = Some(set);
            }
            "kw_item_meta" if f.len() >= 5 => {
                if let Ok(idx) = f[2].parse::<usize>() {
                    if let Some(items) = snap
                        .kept_worlds
                        .get_mut(&unesc(f[1]))
                        .and_then(|kw| kw.world_items.as_mut())
                    {
                        if let Some(it) = items.get_mut(idx) {
                            it.kind = unesc(f[3]);
                            if let Ok(added) = f[4].parse::<u64>() {
                                it.added = added;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Exports a [`SavedWorldSnapshot`] into a `save<N>.vol` `PVOL` byte buffer:
/// binary `PERS` current world + ASCII kept `.vol` worlds + ASCII `.kq` maps
/// + `WorldVar.cs` / `newWorld.cs` / `current.bin` / `kq8r_meta.txt`.
pub fn export_save_vol(
    snap: &SavedWorldSnapshot,
    save_name: &str,
    thumb_bin: &[u8],
    item_types: Option<&ItemTypes>,
) -> Vec<u8> {
    let world_base = snap
        .world
        .trim()
        .strip_suffix(".kq")
        .or_else(|| snap.world.trim().strip_suffix(".KQ"))
        .unwrap_or_else(|| snap.world.trim());
    let world_base = if world_base.is_empty() {
        "Daventry"
    } else {
        world_base
    };
    let cur_key = world_base.to_ascii_lowercase();
    let cur_kw = snap.kept_worlds.get(&cur_key);

    let mut owned: Vec<(String, Vec<u8>)> = Vec::new();
    let mut used = BTreeSet::new();
    for reserved in [
        "savedgamename",
        "savedgameversion",
        "worldname",
        "worldvar.cs",
        "newworld.cs",
        "current.bin",
        "gamestate.kq",
        "worldvolumes",
        "maps",
        "kq8r_meta.txt",
        "kq8r_state.txt",
        "kqcamera.kq",
        "connor.kq",
        "worldinventory.kq",
        "worldstate.kq",
        "music.kq",
    ] {
        used.insert(reserved.to_string());
    }

    // Metadata strings have no trailing NUL (savefile.md rule 2).
    owned.push(("savedGameName".into(), save_name.as_bytes().to_vec()));
    owned.push(("savedGameVersion".into(), b"1.000000".to_vec()));
    owned.push(("worldName".into(), format!("{world_base}.kq").into_bytes()));

    // Combine current-world vars + global journey vars for WorldVar.cs.
    let mut combined_vars = Vec::new();
    let mut seen_vars = BTreeSet::new();
    if let Some(kw) = cur_kw {
        for (k, v) in &kw.vars {
            seen_vars.insert(k.to_ascii_lowercase());
            combined_vars.push((k.clone(), v.clone()));
        }
    }
    for (k, v) in &snap.journey.vars {
        if seen_vars.insert(k.to_ascii_lowercase()) {
            combined_vars.push((k.clone(), v.clone()));
        }
    }
    owned.push((
        "WorldVar.cs".into(),
        format_world_var_cs(&combined_vars).into_bytes(),
    ));
    owned.push((
        "newWorld.cs".into(),
        build_new_world_cs(cur_kw.and_then(|kw| kw.current_time)),
    ));
    owned.push(("current.bin".into(), thumb_bin.to_vec()));

    // Binary PERS chunks for current world.
    let game_state = PersObject::KqGameState(KqGameState {
        base: kq_base("GameState"),
        next_no_name: snap.journey.next_no_name.max(1) as i32,
        difficulty_factor: snap.journey.combat_level,
    });
    owned.push(("GameState.kq".into(), write_pers(&game_state)));

    if let Some(cam) = &snap.camera {
        let cam_pers = PersObject::KqCamMover(KqCamMover {
            visible: kq_visible("KQCamera", false),
            follow_object_name: cam.follow_object.clone(),
            pause_follow: i32::from(!cam.following),
            near_plane: 100.0,
            far_plane: 150_000.0,
            fov: 45.0,
            fix_height: i32::from(cam.hold),
            cam_move_radius: cam.range.1,
            conner_min_dist: cam.range.0,
            loc: cam.position,
            third_camera_offset: cam.offset,
            camera_rotation: [cam.tilt, 0.0, 0.0],
            dir: [cam.view_tilt, cam.roll, cam.heading],
            is_first_person: i32::from(cam.first_person),
            ..Default::default()
        });
        owned.push(("KQCamera.kq".into(), write_pers(&cam_pers)));
    }

    let connor_pers = build_connor_pers(snap, item_types);
    owned.push(("Connor.kq".into(), write_pers(&connor_pers)));

    if let Some(kw) = cur_kw {
        if let Some(world_items) = &kw.world_items {
            let items = world_items
                .iter()
                .map(|it| Some(build_pers_item(it, item_types)))
                .collect();
            let wi_pers = PersObject::KqInventory {
                class: "KQWorldInventory".to_string(),
                data: KqInventory {
                    base: kq_base("WorldInventory"),
                    owner_id: u32::MAX,
                    items,
                },
            };
            owned.push(("WorldInventory.kq".into(), write_pers(&wi_pers)));
        }
        if !kw.pending_events.is_empty() {
            let ws_pers = PersObject::KqWorldState(KqWorldState {
                base: kq_base("worldState"),
                events: kw
                    .pending_events
                    .iter()
                    .map(|e| Some(e.clone()))
                    .collect(),
            });
            owned.push(("worldState.kq".into(), write_pers(&ws_pers)));
        }
        if let Some(stack) = &kw.music_stack {
            let m_pers = PersObject::KqMusic(KqMusic {
                visible: kq_visible("Music", false),
                music: stack.iter().map(|s| Some(s.clone())).collect(),
            });
            owned.push(("Music.kq".into(), write_pers(&m_pers)));
        }
        for d in &kw.doors {
            let obj = build_kq_object(&d.name, false, &d.placement, d.collision_type, None, false, 0, None);
            let door = PersObject::KqDoor(KqDoor {
                anim_object: build_anim_object(obj, &d.movement, d.phase, d.waiting.as_deref()),
                state: d.state.to_pers(),
            });
            let chunk = unique_chunk_name(&mut used, &d.name, ".kq");
            owned.push((chunk, write_pers(&door)));
        }
        for c in &kw.chests {
            let obj = build_kq_object(
                &c.name,
                c.hidden,
                &c.placement,
                c.collision_type,
                None,
                false,
                0,
                None,
            );
            let chest = PersObject::KqChest(KqChest {
                anim_object: build_anim_object(obj, &c.movement, c.phase, c.waiting.as_deref()),
                chest_state: c.state.to_pers(),
                inv_item_name: c.remembered.clone(),
                ..Default::default()
            });
            let chunk = unique_chunk_name(&mut used, &c.name, ".kq");
            owned.push((chunk, write_pers(&chest)));
        }
        for st in &kw.states {
            if st.name.is_empty() {
                continue;
            }
            let obj = build_kq_object(
                &st.name,
                false,
                &st.placement,
                st.collision_type.unwrap_or(crate::collide::DEFAULT_TYPE),
                None,
                false,
                0,
                None,
            );
            let anim = PersObject::KqAnimObject(build_anim_object(
                obj,
                &st.movement,
                st.phase,
                st.waiting.as_deref(),
            ));
            let chunk = unique_chunk_name(&mut used, &st.name, ".kq");
            owned.push((chunk, write_pers(&anim)));
        }
        for (name, &alt) in &kw.alt_lightmaps {
            if alt {
                let int = PersObject::KqInterior(KqInterior {
                    object: build_kq_object(
                        name,
                        false,
                        &Placement::at([0.0; 3]),
                        0,
                        None,
                        false,
                        0,
                        None,
                    ),
                    using_alt_lighting: 1,
                    ..Default::default()
                });
                let chunk = unique_chunk_name(&mut used, name, ".kq");
                owned.push((chunk, write_pers(&int)));
            }
        }
        for name in &kw.rusted_rivers {
            let rc = PersObject::KqRiverControl(KqRiverControl {
                object: build_kq_object(
                    name,
                    false,
                    &Placement::at([0.0; 3]),
                    0,
                    None,
                    false,
                    0,
                    None,
                ),
                rusted_one: 1,
                ..Default::default()
            });
            let chunk = unique_chunk_name(&mut used, name, ".kq");
            owned.push((chunk, write_pers(&rc)));
        }
    }

    // Current world map as binary PERS; other worlds' maps as ASCII listed in `maps`.
    const MAP_WORLDS: [&str; 16] = [
        "daventry", "deadcity", "swamp", "gnome", "barren", "dwarf", "iceworld", "temple1",
        "temple2", "temple3", "temple4", "icelord", "icelord", "icelord", "castled", "snowexit",
    ];
    let mut extra_map_names = Vec::new();
    let mut cur_map_emitted = false;
    for (&idx, (file, mk, note)) in &snap.journey.other_maps {
        let is_cur_world = MAP_WORLDS
            .get(idx)
            .is_some_and(|w| w.eq_ignore_ascii_case(world_base));
        let is_cur = is_cur_world
            && (!cur_map_emitted || note.to_ascii_lowercase().contains(&cur_key));
        let stem_fallback = format!("{}.map", mk.name);
        let stem = sanitize_stem(file, &stem_fallback);
        let chunk = unique_chunk_name(&mut used, &stem, ".kq");
        if is_cur {
            cur_map_emitted = true;
            let kq_map = PersObject::KqMap(KqMap::from_map_keys(mk));
            owned.push((chunk, write_pers(&kq_map)));
        } else {
            extra_map_names.push(chunk.clone());
            owned.push((chunk, mk.to_text().into_bytes()));
        }
    }
    owned.push(("maps".into(), extra_map_names.join("|").into_bytes()));

    // Embedded ASCII kept-world .vol archives for non-current worlds.
    let mut vol_names = Vec::new();
    for (wkey, kw) in &snap.kept_worlds {
        if wkey.eq_ignore_ascii_case(&cur_key) {
            continue;
        }
        let chunk = unique_chunk_name(&mut used, wkey, ".vol");
        vol_names.push(chunk.clone());
        let vol_bytes = build_ascii_kept_vol(wkey, kw, None);
        owned.push((chunk, vol_bytes));
    }
    owned.push(("worldVolumes".into(), vol_names.join("|").into_bytes()));
    owned.push(("kq8r_meta.txt".into(), format_kq8r_meta(snap)));

    let refs: Vec<(&str, &[u8])> = owned.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
    Pvol::build(&refs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::active::{KeptChestState, KeptDoorState, KeptState};
    use crate::chest::ChestState;
    use crate::door::DoorState;
    use crate::journey::Journey;
    use crate::magic_map::{ConnorMapMarks, START_VIEW};
    use crate::save_game::{KeptWorldSnapshot, SavedCameraState};
    use crate::save_import::import_original_save;
    use crate::stats::{ConnorStats, Stats};
    use crate::weapons::WeaponMode;
    use kq8_data::kq_text;
    use kq8_data::magic_map::{MapIcon, MapKeys};
    use kq8_data::save_pers::parse_pers;

    fn sample_item_types() -> ItemTypes {
        let itl = b"[INVITEM_Sword]\nid=3\ncategory=ShortWeapon\n\
            [INVITEM_Crystal]\nid=9\ncategory=Inventory\n\
            [INVITEM_Clarity]\nid=15\ncategory=Inventory\n";
        ItemTypes::parse(itl).0
    }

    #[test]
    fn exported_save_has_valid_pvol_and_pers_and_round_trips() {
        let item_types = sample_item_types();
        let mut flags = BTreeMap::new();
        flags.insert("FoundCrystal".into(), true);

        let mut dv_kept = KeptWorldSnapshot {
            vars: vec![("Daventry::girlhut::FirstTime".into(), "0".into())],
            gone: ["removedrock".into()].into_iter().collect(),
            came_in: vec!["came_in_npc.kq".into()],
            states: vec![KeptState {
                name: "HayBale".into(),
                movement: "idle".into(),
                phase: 0.25,
                waiting: Some("next".into()),
                placement: Placement::at([1.0, 2.0, 3.0]),
                collision_type: Some(0x28),
            }],
            doors: vec![KeptDoorState {
                name: "Condoor".into(),
                state: DoorState::Opened,
                movement: "opened".into(),
                phase: 1.0,
                waiting: None,
                placement: Placement::at([2000.0, 3000.0, 100.0]),
                collision_type: 0x8021,
            }],
            chests: vec![KeptChestState {
                name: "smlchst3".into(),
                state: ChestState::Open,
                remembered: Some("NoName22".into()),
                movement: "open".into(),
                phase: 1.0,
                waiting: None,
                placement: Placement::at([2100.0, 3100.0, 100.0]),
                hidden: false,
                collision_type: 0x8021,
            }],
            world_items: Some(vec![ItemObject {
                name: "NoName22".into(),
                kind: "INVITEM_Crystal".into(),
                quantity: 3,
                third: -1,
                place: Place::World {
                    position: [2105.0, 3105.0, 105.0],
                    angles: [0.0, 0.0, 0.5],
                },
                hidden: false,
                script: Some("crystal.cs".into()),
                do_not_point_at: false,
                added: 7,
            }]),
            music_stack: Some(vec!["DAV1.AUD".into()]),
            current_time: Some(42.5),
            pending_events: vec!["99.000\tCondoor\tKQDoorEvent\t2".into()],
            present_objects: None,
            rusted_rivers: vec!["SwampRiver".into()],
            alt_lightmaps: [("altroom".into(), true), ("normroom".into(), false)]
                .into_iter()
                .collect(),
        };
        let mut sw_kept = KeptWorldSnapshot {
            vars: vec![("Swamp::Gate::Open".into(), "1".into())],
            doors: vec![KeptDoorState {
                name: "SwampGate".into(),
                state: DoorState::Closed,
                movement: "closed".into(),
                phase: 0.0,
                waiting: None,
                placement: Placement::at([50.0, 60.0, 70.0]),
                collision_type: 0x8021,
            }],
            current_time: Some(12.25),
            music_stack: Some(vec!["SWAMP.AUD".into()]),
            ..Default::default()
        };
        let _ = (&mut dv_kept, &mut sw_kept);

        let mut other_maps = BTreeMap::new();
        other_maps.insert(
            0usize,
            (
                "DavMap.kq".into(),
                MapKeys {
                    name: "DavMap".into(),
                    north: 350000.0,
                    south: 100000.0,
                    east: 300000.0,
                    west: 120000.0,
                    use_fog: true,
                    fog_radius: Some(4500.0),
                    hold_map_loc: false,
                    is_active: true,
                    blank_map: "dav_blank.bmp".into(),
                    full_map: "dav_full.bmp".into(),
                    arrow_pba: "maparrow.pba".into(),
                    map_name: Some(38),
                    earned_teleport: true,
                    teleport_script: Some("tele.cs".into()),
                    teleport_portal: Some("portal".into()),
                    icons: vec![MapIcon {
                        picture: "icon.pbm".into(),
                        file: "town.cs".into(),
                        x: 10,
                        y: 20,
                    }],
                    runs: vec![4, 1, 8],
                },
                "saved in Daventry".into(),
            ),
        );
        other_maps.insert(
            2usize,
            (
                "SwampMap.kq".into(),
                MapKeys {
                    name: "SwampMap".into(),
                    north: 200000.0,
                    south: 50000.0,
                    east: 220000.0,
                    west: 60000.0,
                    use_fog: true,
                    fog_radius: Some(3000.0),
                    hold_map_loc: false,
                    is_active: true,
                    blank_map: "sw_blank.bmp".into(),
                    full_map: "sw_full.bmp".into(),
                    arrow_pba: "maparrow.pba".into(),
                    map_name: Some(40),
                    earned_teleport: false,
                    teleport_script: None,
                    teleport_portal: None,
                    icons: Vec::new(),
                    runs: vec![12],
                },
                "saved in Swamp".into(),
            ),
        );

        let mut marks = ConnorMapMarks::default();
        marks.visited[0] = true;
        marks.visited[2] = true;
        marks.teleport_earned[0] = true;
        marks.sage_gnome = true;

        let mut kept_worlds = BTreeMap::new();
        kept_worlds.insert("daventry".into(), dv_kept.clone());
        kept_worlds.insert("swamp".into(), sw_kept.clone());

        let snap = SavedWorldSnapshot {
            world: "Daventry".into(),
            connor_at: Some([202336.0, 262300.0, 3.1416]),
            connor_pos: Some(([202336.0, 262300.0, 8209.5], 3.1416)),
            connor_anim: Some(("Stop".into(), 0.5)),
            camera: Some(SavedCameraState {
                position: [206063.0, 258979.6, 9882.3],
                heading: 3.14,
                view_tilt: -0.2,
                tilt: -0.2,
                roll: 0.0,
                following: true,
                follow_object: Some("Connor".into()),
                range: (1500.0, 5000.0),
                mode_fixed: false,
                first_person: false,
                hold: false,
                offset: [0.0, -300.0, 200.0],
            }),
            rope: Some(Rope {
                status: RopeStatus::Bottom,
                place: [200000.0, 260000.0, 8200.0],
                dir: [0.0, 0.0, 1.57],
                cam: [Some([198000.0, 260000.0, 12000.0]), None],
                top_cam: Some([198500.0, 260000.0, 15000.0]),
                angle: Some((1.5, 0.5)),
                top_limit: 16000.0,
                bottom_limit: 7000.0,
                changed_at: Some(10.0),
            }),
            rope_climb: Some(Climb {
                from: End::Bottom,
                stage: Stage::Hanging,
                place: [200000.0, 260000.0, 8200.0],
                dir: [0.0, 0.0, 1.57],
                cam: [Some([198000.0, 260000.0, 12000.0]), None],
                top_limit: 16000.0,
                bottom_limit: 7000.0,
                hand_before: WeaponMode::Hand,
                weapon_before: Some("Sword".into()),
                first_to_third: false,
                aim: [200000.0, 260000.0, 11000.0],
                range: (11000.0, 15500.0),
                saved_steps: Some((325.0, 325.0)),
                done: false,
            }),
            journey: Journey {
                from: "Daventry".into(),
                stats: Some(ConnorStats {
                    stats: Stats([20, 10, 5, 15, 250]),
                    level: 2,
                }),
                items: vec![
                    ItemObject {
                        name: "Sword".into(),
                        kind: "INVITEM_Sword".into(),
                        quantity: 1,
                        third: 0,
                        place: Place::Carried("Connor".into()),
                        hidden: false,
                        script: None,
                        do_not_point_at: false,
                        added: 4,
                    },
                    ItemObject {
                        name: "Clarity1".into(),
                        kind: "INVITEM_Clarity".into(),
                        quantity: 2,
                        third: 0,
                        place: Place::Carried("Connor".into()),
                        hidden: false,
                        script: None,
                        do_not_point_at: false,
                        added: 5,
                    },
                ],
                hand: Some("Sword".into()),
                weapon_ready: Some("Sword".into()),
                weapon_mode: WeaponMode::Hand,
                last_away_ranged: false,
                projectile: None,
                shoot_offset: [0.0, 10.0, 20.0],
                potions: vec![(
                    Running::new("Clarity1", "INVITEM_Clarity", 30, 5),
                    120,
                )],
                pickup_experience: vec![(15.0, 25, "INVITEM_Crystal".into())],
                flags,
                in_temple4: false,
                immunities: 0x04,
                raw_world_flags: None,
                map_marks: marks,
                map_view: START_VIEW,
                other_maps,
                combat_level: 1.5,
                next_no_name: 23,
                helps_asked: BTreeSet::new(),
                connor_tips: [true, false],
                kill_tip: true,
                not_starting_help: 1,
                tip_bits: 0x05,
                god_mode: false,
                ai_off: false,
                allow_shadows: true,
                vars: vec![("Global::Quest".into(), "2".into())],
            },
            kept_worlds,
        };

        let bytes = export_save_vol(&snap, "test4h4", b"THUMB", Some(&item_types));
        let pvol = Pvol::parse(bytes.clone()).expect("PVOL parses");

        // No trailing NUL on metadata strings (savefile.md rule 2).
        for (k, expected) in [
            ("savedGameName", "test4h4"),
            ("savedGameVersion", "1.000000"),
            ("worldName", "Daventry.kq"),
        ] {
            let e = pvol.find(k).expect(k);
            let data = pvol.data(e).unwrap();
            assert_eq!(data, expected.as_bytes(), "unexpected bytes for {k}");
        }

        // Verify all .kq and embedded .vol entries parse cleanly like save-verify.
        for e in &pvol.entries {
            let data = pvol.data(e).unwrap();
            let low = e.name.to_ascii_lowercase();
            if low.ends_with(".kq") {
                if data.starts_with(b"PERS") || data.starts_with(b"DICT") {
                    parse_pers(data).unwrap_or_else(|err| panic!("{}: {err}", e.name));
                } else {
                    assert!(
                        !kq_text::parse(data).is_empty(),
                        "{} should parse non-empty ASCII .kq",
                        e.name
                    );
                }
            } else if low.ends_with(".vol") {
                let inner = Pvol::parse(data.to_vec()).unwrap_or_else(|err| panic!("{}: {err}", e.name));
                for ie in &inner.entries {
                    if ie.name.to_ascii_lowercase().ends_with(".kq") {
                        let idata = inner.data(ie).unwrap();
                        assert!(
                            !kq_text::parse(idata).is_empty(),
                            "{} -> {} should parse non-empty ASCII .kq",
                            e.name,
                            ie.name
                        );
                    }
                }
            }
        }

        let dav_map_entry = pvol.find("DavMap.kq").expect("DavMap.kq");
        assert!(pvol.data(dav_map_entry).unwrap().starts_with(b"PERS"));
        let maps_entry = pvol.find("maps").expect("maps");
        assert_eq!(pvol.data(maps_entry).unwrap(), b"SwampMap.kq");

        // Round-trip through import_original_save + apply_kq8r_meta.
        let mut imported =
            import_original_save(&bytes, Some(&item_types)).expect("import_original_save");
        assert!(imported.rope.is_some());
        assert!(imported.rope_climb.is_some());
        assert_eq!(imported.rope_climb.as_ref().unwrap().stage, Stage::Hanging);
        let meta = pvol.find("kq8r_meta.txt").expect("kq8r_meta.txt");
        apply_kq8r_meta(&mut imported, pvol.data(meta).unwrap()).expect("apply_kq8r_meta");

        assert_eq!(imported.world, "Daventry");
        assert_eq!(imported.connor_pos, snap.connor_pos);
        assert_eq!(imported.connor_anim, snap.connor_anim);
        assert_eq!(imported.camera, snap.camera);
        assert_eq!(imported.rope, snap.rope);
        assert_eq!(imported.rope_climb, snap.rope_climb);
        assert_eq!(imported.journey.next_no_name, 23);
        assert!((imported.journey.combat_level - 1.5).abs() < 1e-6);
        assert_eq!(imported.journey.stats, snap.journey.stats);
        assert_eq!(imported.journey.items, snap.journey.items);
        assert_eq!(imported.journey.hand, snap.journey.hand);
        assert_eq!(imported.journey.weapon_ready, snap.journey.weapon_ready);
        assert_eq!(imported.journey.potions, snap.journey.potions);
        assert_eq!(imported.journey.pickup_experience, snap.journey.pickup_experience);
        assert_eq!(imported.journey.flags, snap.journey.flags);
        assert_eq!(imported.journey.map_marks, snap.journey.map_marks);
        assert_eq!(imported.journey.other_maps.len(), 2);

        let dv = imported.kept_worlds.get("daventry").expect("daventry kept");
        assert_eq!(dv.doors, dv_kept.doors);
        assert_eq!(dv.chests, dv_kept.chests);
        assert_eq!(dv.world_items, dv_kept.world_items);
        assert_eq!(dv.music_stack, dv_kept.music_stack);
        assert_eq!(dv.current_time, dv_kept.current_time);
        assert_eq!(dv.pending_events, dv_kept.pending_events);
        assert_eq!(dv.gone, dv_kept.gone);
        assert_eq!(dv.came_in, dv_kept.came_in);
        assert_eq!(dv.rusted_rivers, dv_kept.rusted_rivers);
        assert_eq!(dv.alt_lightmaps, dv_kept.alt_lightmaps);

        let sw = imported.kept_worlds.get("swamp").expect("swamp kept");
        assert_eq!(sw.doors, sw_kept.doors);
        assert_eq!(sw.vars, sw_kept.vars);
        assert_eq!(sw.music_stack, sw_kept.music_stack);
        assert_eq!(sw.current_time, sw_kept.current_time);
    }
}
