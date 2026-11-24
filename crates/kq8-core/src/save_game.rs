//! Saved game world snapshot serialization and original chunk parsers
//! (`docs/behaviour/submenus.md` rules 10–11, M4 step 4g-4).
//!
//! Packs full cross-world and per-world viewer state (`Journey`, `KeptWorld`,
//! Connor's position/heading) into `kq8r_state.txt` inside `save<N>.vol`,
//! plus reads/formats the original text/binary chunks `WorldVar.cs` and
//! `GameState.kq`.

use std::collections::{BTreeMap, BTreeSet};

use kq8_data::magic_map::{MapIcon, MapKeys};

use crate::active::KeptState;
use crate::collide::Placement;
use crate::item::{ItemObject, Place};
use crate::journey::Journey;
use crate::magic_map::{ConnorMapMarks, View, START_VIEW};
use crate::potions::{Effect, Running};
use crate::stats::{ConnorStats, Stats};
use crate::weapons::WeaponMode;

const MAGIC_HEADER: &str = "KQ8R_SAVE_V1";

/// Serializable snapshot of a visited world (`world_build::KeptWorld`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KeptWorldSnapshot {
    pub vars: Vec<(String, String)>,
    pub gone: BTreeSet<String>,
    pub came_in: Vec<String>,
    pub states: Vec<KeptState>,
    pub rusted_rivers: Vec<String>,
    pub alt_lightmaps: BTreeMap<String, bool>,
}

/// Complete saved-game snapshot stored in `kq8r_state.txt` inside `save<N>.vol`.
#[derive(Debug, Clone, PartialEq)]
pub struct SavedWorldSnapshot {
    pub world: String,
    pub connor_at: Option<[f32; 3]>,
    pub journey: Journey,
    pub kept_worlds: BTreeMap<String, KeptWorldSnapshot>,
}

/// Formats world script variables in `WorldVar.cs` syntax (`set <k> <v>\r\n`).
pub fn format_world_var_cs(vars: &[(String, String)]) -> String {
    let mut out = String::new();
    for (k, v) in vars {
        out.push_str(&format!("set {k} {v}\r\n"));
    }
    out
}

/// Parses `set <k> <v>` lines from `WorldVar.cs` (`submenus.md` rule 10).
pub fn parse_world_var_cs(bytes: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(bytes);
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        let rest = if let Some(s) = line.strip_prefix("set ") {
            s
        } else if let Some(s) = line.strip_prefix("SET ") {
            s
        } else {
            continue;
        };
        let rest = rest.trim_start();
        let mut parts = rest.splitn(2, char::is_whitespace);
        let Some(k) = parts.next().map(str::trim).filter(|s| !s.is_empty()) else {
            continue;
        };
        let v = parts.next().map_or("", str::trim).to_string();
        out.push((k.to_string(), v));
    }
    out
}

/// Parses `(nextNoName, difficultyFactor)` from either KQ8R's text `GameState.kq`
/// or an original binary `PERS` `KQGameState` blob (trailing 8 bytes).
pub fn parse_game_state_kq(bytes: &[u8]) -> Option<(usize, f32)> {
    if !bytes.starts_with(b"PERS") {
        let text = String::from_utf8_lossy(bytes);
        let mut next_no_name = None;
        let mut factor = None;
        for raw in text.lines() {
            let line = raw.trim();
            if let Some(v) = line.strip_prefix("nextNoName=") {
                next_no_name = v.trim().parse::<usize>().ok();
            } else if let Some(v) = line.strip_prefix("difficultyFactor=") {
                factor = v.trim().parse::<f32>().ok();
            }
        }
        if let (Some(n), Some(f)) = (next_no_name, factor) {
            return Some((n, f));
        }
    }
    if bytes.starts_with(b"PERS") && bytes.len() >= 8 {
        let n = u32::from_le_bytes(bytes[bytes.len() - 8..bytes.len() - 4].try_into().ok()?) as usize;
        let f = f32::from_le_bytes(bytes[bytes.len() - 4..].try_into().ok()?);
        if f.is_finite() && (0.5..=2.0).contains(&f) {
            return Some((n, f));
        }
    }
    None
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

fn unesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('t') => out.push('\t'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn fmt_opt_str(v: &Option<String>) -> String {
    match v {
        Some(s) => format!("S:{}", esc(s)),
        None => "-".into(),
    }
}

fn parse_opt_str(s: &str) -> Option<String> {
    s.strip_prefix("S:").map(unesc)
}

fn fmt_f32_slice(vals: &[f32]) -> String {
    vals.iter().map(|v| v.to_bits().to_string()).collect::<Vec<_>>().join(",")
}

fn parse_f32_hex_or_dec<const N: usize>(s: &str) -> Result<[f32; N], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != N {
        return Err(format!("expected {N} floats, got {}", parts.len()));
    }
    let mut out = [0.0f32; N];
    for (slot, p) in out.iter_mut().zip(parts) {
        if let Ok(bits) = p.parse::<u32>() {
            *slot = f32::from_bits(bits);
        } else {
            *slot = p.parse::<f32>().map_err(|_| format!("bad float {p}"))?;
        }
    }
    Ok(out)
}

fn fmt_place(place: &Place) -> String {
    match place {
        Place::Carried(who) => format!("C:{}", esc(who)),
        Place::World { position, angles } => {
            format!("W:{}:{}", fmt_f32_slice(position), fmt_f32_slice(angles))
        }
        Place::Gone => "G".into(),
    }
}

fn parse_place(s: &str) -> Result<Place, String> {
    if s == "G" {
        return Ok(Place::Gone);
    }
    if let Some(who) = s.strip_prefix("C:") {
        return Ok(Place::Carried(unesc(who)));
    }
    if let Some(rest) = s.strip_prefix("W:") {
        let (pos_s, ang_s) = rest.split_once(':').ok_or("bad World place")?;
        return Ok(Place::World {
            position: parse_f32_hex_or_dec::<3>(pos_s)?,
            angles: parse_f32_hex_or_dec::<3>(ang_s)?,
        });
    }
    Err(format!("bad place {s}"))
}

impl SavedWorldSnapshot {
    /// Serializes this snapshot into `KQ8R_SAVE_V1` text for `kq8r_state.txt`.
    pub fn format_kq8r_state(&self) -> String {
        let mut out = String::new();
        out.push_str(MAGIC_HEADER);
        out.push('\n');
        out.push_str(&format!("world\t{}\n", esc(&self.world)));
        if let Some([x, y, h]) = self.connor_at {
            out.push_str(&format!("connor_at\t{}\t{}\t{}\n", x.to_bits(), y.to_bits(), h.to_bits()));
        }

        let j = &self.journey;
        out.push_str(&format!(
            "journey\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            esc(&j.from),
            j.combat_level.to_bits(),
            j.next_no_name,
            u8::from(j.in_temple4),
            j.immunities,
            j.not_starting_help,
            j.tip_bits,
            u8::from(j.god_mode),
            u8::from(j.ai_off),
            u8::from(j.allow_shadows),
            u8::from(j.last_away_ranged),
            j.weapon_mode.number(),
            u8::from(j.kill_tip),
            u8::from(j.connor_tips[0]),
            u8::from(j.connor_tips[1]),
            j.shoot_offset[0].to_bits(),
            j.shoot_offset[1].to_bits(),
            j.shoot_offset[2].to_bits(),
        ));
        if let Some(h) = &j.hand {
            out.push_str(&format!("hand\t{}\n", esc(h)));
        }
        if let Some(w) = &j.weapon_ready {
            out.push_str(&format!("weapon_ready\t{}\n", esc(w)));
        }
        if let Some(p) = &j.projectile {
            out.push_str(&format!("projectile\t{}\n", esc(p)));
        }
        if let Some(s) = &j.stats {
            out.push_str(&format!(
                "stats\t{}\t{}\t{}\t{}\t{}\t{}\n",
                s.level, s.stats.0[0], s.stats.0[1], s.stats.0[2], s.stats.0[3], s.stats.0[4]
            ));
        }
        out.push_str(&format!(
            "map_view\t{}\t{}\t{}\t{}\n",
            j.map_view.left.to_bits(),
            j.map_view.top.to_bits(),
            j.map_view.right.to_bits(),
            j.map_view.bottom.to_bits(),
        ));
        let vis: Vec<&str> = j.map_marks.visited.iter().map(|&b| if b { "1" } else { "0" }).collect();
        let tel: Vec<&str> = j.map_marks.teleport_earned.iter().map(|&b| if b { "1" } else { "0" }).collect();
        out.push_str(&format!(
            "map_marks\t{}\t{}\t{}\n",
            vis.join(","),
            tel.join(","),
            u8::from(j.map_marks.sage_gnome)
        ));

        for item in &j.items {
            out.push_str(&format!(
                "item\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                esc(&item.name),
                esc(&item.kind),
                item.quantity,
                item.third,
                fmt_place(&item.place),
                u8::from(item.hidden),
                fmt_opt_str(&item.script),
                u8::from(item.do_not_point_at),
                item.added,
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
            out.push_str(&format!("pickup_exp\t{}\t{}\t{}\n", secs.to_bits(), amt, esc(kind)));
        }
        for (k, &v) in &j.flags {
            out.push_str(&format!("flag\t{}\t{}\n", esc(k), u8::from(v)));
        }
        for (k, v) in &j.vars {
            out.push_str(&format!("jvar\t{}\t{}\n", esc(k), esc(v)));
        }
        for (&idx, (file, mk, note)) in &j.other_maps {
            let fog_r = mk.fog_radius.map_or_else(|| "-".into(), |f| f.to_bits().to_string());
            let mname = mk.map_name.map_or_else(|| "-".into(), |n| n.to_string());
            let runs_s: Vec<String> = mk.runs.iter().map(i64::to_string).collect();
            out.push_str(&format!(
                "omap\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                idx,
                esc(file),
                esc(note),
                esc(&mk.name),
                mk.north.to_bits(),
                mk.south.to_bits(),
                mk.east.to_bits(),
                mk.west.to_bits(),
                u8::from(mk.use_fog),
                fog_r,
                u8::from(mk.hold_map_loc),
                u8::from(mk.is_active),
                esc(&mk.blank_map),
                esc(&mk.full_map),
                esc(&mk.arrow_pba),
                mname,
                u8::from(mk.earned_teleport),
                fmt_opt_str(&mk.teleport_script),
                fmt_opt_str(&mk.teleport_portal),
                runs_s.join(","),
            ));
            for ic in &mk.icons {
                out.push_str(&format!(
                    "omap_icon\t{}\t{}\t{}\t{}\t{}\n",
                    idx,
                    esc(&ic.picture),
                    esc(&ic.file),
                    ic.x,
                    ic.y,
                ));
            }
        }

        for (wname, kw) in &self.kept_worlds {
            out.push_str(&format!("kw\t{}\n", esc(wname)));
            for (k, v) in &kw.vars {
                out.push_str(&format!("kw_var\t{}\t{}\t{}\n", esc(wname), esc(k), esc(v)));
            }
            for g in &kw.gone {
                out.push_str(&format!("kw_gone\t{}\t{}\n", esc(wname), esc(g)));
            }
            for c in &kw.came_in {
                out.push_str(&format!("kw_came\t{}\t{}\n", esc(wname), esc(c)));
            }
            for r in &kw.rusted_rivers {
                out.push_str(&format!("kw_river\t{}\t{}\n", esc(wname), esc(r)));
            }
            for (a, &v) in &kw.alt_lightmaps {
                out.push_str(&format!("kw_alt\t{}\t{}\t{}\n", esc(wname), esc(a), u8::from(v)));
            }
            for st in &kw.states {
                let rot_flat: Vec<f32> = st.placement.rotation.iter().flatten().copied().collect();
                let col_s = st.collision_type.map_or_else(|| "-".into(), |c| c.to_string());
                out.push_str(&format!(
                    "kw_state\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    esc(wname),
                    esc(&st.name),
                    esc(&st.movement),
                    st.phase.to_bits(),
                    fmt_opt_str(&st.waiting),
                    fmt_f32_slice(&st.placement.position),
                    fmt_f32_slice(&rot_flat),
                    col_s,
                ));
            }
        }
        out
    }

    /// Parses `kq8r_state.txt` produced by `format_kq8r_state`.
    pub fn parse_kq8r_state(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "kq8r_state.txt is not UTF-8")?;
        let mut lines = text.lines();
        let Some(first) = lines.next().map(str::trim) else {
            return Err("empty kq8r_state.txt".into());
        };
        if first != MAGIC_HEADER {
            return Err(format!("unexpected header '{first}'"));
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

        let mut world = String::new();
        let mut connor_at: Option<[f32; 3]> = None;
        let mut journey = Journey {
            from: String::new(),
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
            map_marks: ConnorMapMarks::default(),
            map_view: START_VIEW,
            other_maps: BTreeMap::new(),
            combat_level: 1.0,
            next_no_name: 1,
            helps_asked: BTreeSet::new(),
            connor_tips: [false; 2],
            kill_tip: false,
            not_starting_help: 0,
            tip_bits: 0,
            god_mode: false,
            ai_off: false,
            allow_shadows: true,
            vars: Vec::new(),
        };
        let mut kept_worlds: BTreeMap<String, KeptWorldSnapshot> = BTreeMap::new();

        for raw in lines {
            if raw.trim().is_empty() {
                continue;
            }
            let f: Vec<&str> = raw.split('\t').collect();
            match f[0] {
                "world" if f.len() >= 2 => world = unesc(f[1]),
                "connor_at" if f.len() >= 4 => {
                    connor_at = Some([
                        parse_f32_bits(f[1])?,
                        parse_f32_bits(f[2])?,
                        parse_f32_bits(f[3])?,
                    ]);
                }
                "journey" if f.len() >= 19 => {
                    journey.from = unesc(f[1]);
                    journey.combat_level = parse_f32_bits(f[2])?;
                    journey.next_no_name = f[3].parse().map_err(|_| "bad next_no_name")?;
                    journey.in_temple4 = parse_bool(f[4]);
                    journey.immunities = f[5].parse().map_err(|_| "bad immunities")?;
                    journey.not_starting_help = f[6].parse().map_err(|_| "bad not_starting_help")?;
                    journey.tip_bits = f[7].parse().map_err(|_| "bad tip_bits")?;
                    journey.god_mode = parse_bool(f[8]);
                    journey.ai_off = parse_bool(f[9]);
                    journey.allow_shadows = parse_bool(f[10]);
                    journey.last_away_ranged = parse_bool(f[11]);
                    let mode_num: i64 = f[12].parse().unwrap_or(4);
                    journey.weapon_mode = WeaponMode::from_number(mode_num).unwrap_or(WeaponMode::None);
                    journey.kill_tip = parse_bool(f[13]);
                    journey.connor_tips = [parse_bool(f[14]), parse_bool(f[15])];
                    journey.shoot_offset = [
                        parse_f32_bits(f[16])?,
                        parse_f32_bits(f[17])?,
                        parse_f32_bits(f[18])?,
                    ];
                }
                "hand" if f.len() >= 2 => journey.hand = Some(unesc(f[1])),
                "weapon_ready" if f.len() >= 2 => journey.weapon_ready = Some(unesc(f[1])),
                "projectile" if f.len() >= 2 => journey.projectile = Some(unesc(f[1])),
                "stats" if f.len() >= 7 => {
                    let level: i64 = f[1].parse().map_err(|_| "bad stats level")?;
                    let mut nums = [0i64; 5];
                    for (slot, s) in nums.iter_mut().zip(&f[2..7]) {
                        *slot = s.parse().map_err(|_| "bad stats num")?;
                    }
                    journey.stats = Some(ConnorStats { stats: Stats(nums), level });
                }
                "map_view" if f.len() >= 5 => {
                    journey.map_view = View {
                        left: parse_f64_bits(f[1])?,
                        top: parse_f64_bits(f[2])?,
                        right: parse_f64_bits(f[3])?,
                        bottom: parse_f64_bits(f[4])?,
                    };
                }
                "map_marks" if f.len() >= 4 => {
                    for (slot, b) in journey.map_marks.visited.iter_mut().zip(f[1].split(',')) {
                        *slot = parse_bool(b);
                    }
                    for (slot, b) in journey.map_marks.teleport_earned.iter_mut().zip(f[2].split(',')) {
                        *slot = parse_bool(b);
                    }
                    journey.map_marks.sage_gnome = parse_bool(f[3]);
                }
                "item" if f.len() >= 10 => {
                    journey.items.push(ItemObject {
                        name: unesc(f[1]),
                        kind: unesc(f[2]),
                        quantity: f[3].parse().map_err(|_| "bad item qty")?,
                        third: f[4].parse().map_err(|_| "bad item third")?,
                        place: parse_place(f[5])?,
                        hidden: parse_bool(f[6]),
                        script: parse_opt_str(f[7]),
                        do_not_point_at: parse_bool(f[8]),
                        added: f[9].parse().map_err(|_| "bad item added")?,
                    });
                }
                "potion" if f.len() >= 9 => {
                    let kind = unesc(f[2]);
                    let place = if f[6] == "-" { None } else { f[6].parse().ok() };
                    let until: u64 = f[8].parse().map_err(|_| "bad potion until")?;
                    journey.potions.push((
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
                    journey.pickup_experience.push((
                        parse_f64_bits(f[1])?,
                        f[2].parse().map_err(|_| "bad pickup amt")?,
                        unesc(f[3]),
                    ));
                }
                "flag" if f.len() >= 3 => {
                    journey.flags.insert(unesc(f[1]), parse_bool(f[2]));
                }
                "jvar" if f.len() >= 3 => {
                    journey.vars.push((unesc(f[1]), unesc(f[2])));
                }
                "omap" if f.len() >= 21 => {
                    let idx: usize = f[1].parse().map_err(|_| "bad omap idx")?;
                    let fog_radius = if f[10] == "-" { None } else { Some(parse_f64_bits(f[10])?) };
                    let map_name = if f[16] == "-" { None } else { f[16].parse().ok() };
                    let runs = if f[20].is_empty() {
                        Vec::new()
                    } else {
                        f[20]
                            .split(',')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.parse::<i64>().map_err(|_| "bad map run".to_string()))
                            .collect::<Result<Vec<_>, _>>()?
                    };
                    let mk = MapKeys {
                        name: unesc(f[4]),
                        north: parse_f64_bits(f[5])?,
                        south: parse_f64_bits(f[6])?,
                        east: parse_f64_bits(f[7])?,
                        west: parse_f64_bits(f[8])?,
                        use_fog: parse_bool(f[9]),
                        fog_radius,
                        hold_map_loc: parse_bool(f[11]),
                        is_active: parse_bool(f[12]),
                        blank_map: unesc(f[13]),
                        full_map: unesc(f[14]),
                        arrow_pba: unesc(f[15]),
                        map_name,
                        earned_teleport: parse_bool(f[17]),
                        teleport_script: parse_opt_str(f[18]),
                        teleport_portal: parse_opt_str(f[19]),
                        icons: Vec::new(),
                        runs,
                    };
                    journey.other_maps.insert(idx, (unesc(f[2]), mk, unesc(f[3])));
                }
                "omap_icon" if f.len() >= 6 => {
                    let idx: usize = f[1].parse().map_err(|_| "bad omap_icon idx")?;
                    if let Some((_, mk, _)) = journey.other_maps.get_mut(&idx) {
                        mk.icons.push(MapIcon {
                            picture: unesc(f[2]),
                            file: unesc(f[3]),
                            x: f[4].parse().map_err(|_| "bad icon x")?,
                            y: f[5].parse().map_err(|_| "bad icon y")?,
                        });
                    }
                }
                "kw" if f.len() >= 2 => {
                    kept_worlds.entry(unesc(f[1])).or_default();
                }
                "kw_var" if f.len() >= 4 => {
                    kept_worlds
                        .entry(unesc(f[1]))
                        .or_default()
                        .vars
                        .push((unesc(f[2]), unesc(f[3])));
                }
                "kw_gone" if f.len() >= 3 => {
                    kept_worlds
                        .entry(unesc(f[1]))
                        .or_default()
                        .gone
                        .insert(unesc(f[2]));
                }
                "kw_came" if f.len() >= 3 => {
                    kept_worlds
                        .entry(unesc(f[1]))
                        .or_default()
                        .came_in
                        .push(unesc(f[2]));
                }
                "kw_river" if f.len() >= 3 => {
                    kept_worlds
                        .entry(unesc(f[1]))
                        .or_default()
                        .rusted_rivers
                        .push(unesc(f[2]));
                }
                "kw_alt" if f.len() >= 4 => {
                    kept_worlds
                        .entry(unesc(f[1]))
                        .or_default()
                        .alt_lightmaps
                        .insert(unesc(f[2]), parse_bool(f[3]));
                }
                "kw_state" if f.len() >= 9 => {
                    let pos = parse_f32_hex_or_dec::<3>(f[6])?;
                    let rot_flat = parse_f32_hex_or_dec::<9>(f[7])?;
                    let rotation = [
                        [rot_flat[0], rot_flat[1], rot_flat[2]],
                        [rot_flat[3], rot_flat[4], rot_flat[5]],
                        [rot_flat[6], rot_flat[7], rot_flat[8]],
                    ];
                    let collision_type = if f[8] == "-" { None } else { f[8].parse().ok() };
                    kept_worlds.entry(unesc(f[1])).or_default().states.push(KeptState {
                        name: unesc(f[2]),
                        movement: unesc(f[3]),
                        phase: parse_f32_bits(f[4])?,
                        waiting: parse_opt_str(f[5]),
                        placement: Placement { rotation, position: pos },
                        collision_type,
                    });
                }
                other => return Err(format!("unknown record '{other}' in kq8r_state.txt")),
            }
        }
        if world.is_empty() {
            return Err("missing world name in kq8r_state.txt".into());
        }
        Ok(SavedWorldSnapshot {
            world,
            connor_at,
            journey,
            kept_worlds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::magic_map::TELEPORT_MARKS;

    #[test]
    fn kq8r_snapshot_round_trips_connor_journey_and_kept_worlds() {
        let mut flags = BTreeMap::new();
        flags.insert("FoundCrystal".into(), true);
        let mut kept = KeptWorldSnapshot::default();
        kept.vars.push(("Daventry::Alchemist::BoardsUp".into(), "1".into()));
        kept.gone.insert("ax".into());
        kept.came_in.push("prhammer.kq".into());
        kept.rusted_rivers.push("river1".into());
        kept.alt_lightmaps.insert("mill".into(), true);
        kept.states.push(KeptState {
            name: "door01".into(),
            movement: "open".into(),
            phase: 0.75,
            waiting: Some("idle".into()),
            placement: Placement::at([100.0, 200.0, 300.0]),
            collision_type: Some(0x20),
        });
        let mut kept_worlds = BTreeMap::new();
        kept_worlds.insert("daventry".into(), kept);

        let snap = SavedWorldSnapshot {
            world: "Daventry".into(),
            connor_at: Some([193000.0, 193000.0, 3.1416]),
            journey: Journey {
                from: "Daventry".into(),
                stats: Some(ConnorStats {
                    stats: Stats([20, 15, 2, 10, 42]),
                    level: 2,
                }),
                items: vec![ItemObject {
                    name: "NoName5".into(),
                    kind: "INVITEM_Crystal".into(),
                    quantity: 3,
                    third: -1,
                    place: Place::Carried("Connor".into()),
                    hidden: false,
                    script: Some("crystal.cs".into()),
                    do_not_point_at: true,
                    added: 12,
                }],
                hand: Some("NoName5".into()),
                weapon_ready: Some("ax".into()),
                weapon_mode: WeaponMode::Hand,
                last_away_ranged: false,
                projectile: Some("PRarrow.kq".into()),
                shoot_offset: [0.0, 10.0, 20.0],
                potions: vec![(Running::new("p1", "INVITEM_Strength", 20, 100), 15)],
                pickup_experience: vec![(0.3, 25, "INVITEM_Ashes".into())],
                flags,
                in_temple4: false,
                immunities: 0x04,
                map_marks: ConnorMapMarks {
                    visited: [true, false, true, false, false, false, false, false, false, false, false, false, false, false, false, false],
                    teleport_earned: [true; TELEPORT_MARKS],
                    sage_gnome: true,
                },
                map_view: START_VIEW,
                other_maps: BTreeMap::new(),
                combat_level: 1.5,
                next_no_name: 42,
                helps_asked: BTreeSet::new(),
                connor_tips: [true, false],
                kill_tip: true,
                not_starting_help: 1,
                tip_bits: 0x15,
                god_mode: false,
                ai_off: true,
                allow_shadows: true,
                vars: vec![("Global::Flag".into(), "line1\nline2\tval".into())],
            },
            kept_worlds,
        };

        let text = snap.format_kq8r_state();
        let reparsed = SavedWorldSnapshot::parse_kq8r_state(text.as_bytes()).unwrap();
        assert_eq!(reparsed, snap);
    }

    #[test]
    fn parses_original_worldvar_cs_and_binary_gamestate_kq() {
        let wv = format_world_var_cs(&[
            ("Daventry::Arrow1::Solved".into(), "1".into()),
            ("Daventry::Note".into(), "hello world".into()),
        ]);
        let parsed_wv = parse_world_var_cs(wv.as_bytes());
        assert_eq!(
            parsed_wv,
            vec![
                ("Daventry::Arrow1::Solved".into(), "1".into()),
                ("Daventry::Note".into(), "hello world".into()),
            ]
        );

        let text_gs = b"[GameState]\r\nnextNoName=17\r\ndifficultyFactor=1.50\r\n";
        assert_eq!(parse_game_state_kq(text_gs), Some((17, 1.5)));

        // Binary PERS blob ending with (u32 439, f32 0.66) like MASK/save/save1.vol.
        let mut pers = b"PERS\0\0\0\0KQGameState\0".to_vec();
        pers.extend_from_slice(&439u32.to_le_bytes());
        pers.extend_from_slice(&0.66f32.to_le_bytes());
        assert_eq!(parse_game_state_kq(&pers), Some((439, 0.66)));
    }
}
