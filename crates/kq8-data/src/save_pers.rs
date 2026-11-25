//! Saved-game binary `PERS` / `DICT` serializer and parser (`docs/behaviour/savefile.md`).
//!
//! Unlike 3D shape/material `PERS` blocks (`crate::pers`), saved-game `PERS`
//! blocks write **no version field** after the even-padded class name, pad
//! every block payload to a 2-byte boundary, and encode strings as `u32(0)`
//! (null) or `u32(even_padded_len)` + null-terminated ASCII.

use crate::magic_map::{MapIcon, MapKeys, MAP_COUNT};

/// 4x4-style rigid transform (`TMat3F`, 52 bytes): `u32 flags`, 3x3 row-major
/// rotation matrix, and translation `[x, y, z]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TMat3F {
    pub flags: u32,
    pub rotation: [[f32; 3]; 3],
    pub position: [f32; 3],
}

impl Default for TMat3F {
    fn default() -> Self {
        Self {
            flags: 0,
            rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            position: [0.0; 3],
        }
    }
}

/// Named child block reference (`Option<(name, Box<PersObject>)>`).
pub type NestedNamed = Option<(String, Box<PersObject>)>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqBase {
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqVisibleBase {
    pub base: KqBase,
    pub active_3d_flag: i32,
    pub is_far: i32,
    pub too_far: f32,
    pub translucency: i32,
    pub haze_multiplier: f32,
    pub b_haze_solid: i32,
    pub hidden: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KqMoverInline {
    pub transform: TMat3F,
    pub keep_on_ground: i32,
    pub use_gravity: i32,
    pub submerged: i32,
    pub max_slope_dot: f32,
}

impl Default for KqMoverInline {
    fn default() -> Self {
        Self {
            transform: TMat3F::default(),
            keep_on_ground: -1,
            use_gravity: 0,
            submerged: 0,
            max_slope_dot: 0.7071,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqMover {
    pub visible: KqVisibleBase,
    pub inline: KqMoverInline,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqAnimMover {
    pub mover: KqMover,
    pub rotate_speed: f32,
    pub fast: i32,
    pub look_at: [f32; 3],
    pub use_look_at: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoverData {
    Inline {
        mover_name: Option<String>,
        inline: KqMoverInline,
    },
    Nested(NestedNamed),
}

impl Default for MoverData {
    fn default() -> Self {
        Self::Inline {
            mover_name: None,
            inline: KqMoverInline::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KqGameState {
    pub base: KqBase,
    pub next_no_name: i32,
    pub difficulty_factor: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqWorldState {
    pub base: KqBase,
    pub events: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqMusic {
    pub visible: KqVisibleBase,
    pub music: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqCamMover {
    pub visible: KqVisibleBase,
    pub follow_object_name: Option<String>,
    pub pause_follow: i32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub fov: f32,
    pub tilt_flag: f32,
    pub zoom_flag: f32,
    pub pan_flag: f32,
    pub fix_height: i32,
    pub old_pan_flag: f32,
    pub old_zoom_flag: f32,
    pub old_ang_diff_z: f32,
    pub cam_move_radius: f32,
    pub short_cam_move_radius: f32,
    pub long_cam_move_radius: f32,
    pub cam_rotate_amount: f32,
    pub conner_min_dist: f32,
    pub cam_fly_dist: f32,
    pub ang_total: f32,
    pub ang_step: f32,
    pub roll_step: f32,
    pub cam_con_len: f32,
    pub shake_duration: f32,
    pub x_shake_max: f32,
    pub z_shake_max: f32,
    pub x_shake_amp: f32,
    pub z_shake_amp: f32,
    pub old_cmrx: f32,
    pub con_head_height: f32,
    pub cam_height1: f32,
    pub cam_height2: f32,
    pub x_shake_period: f32,
    pub z_shake_period: f32,
    pub loc: [f32; 3],
    pub old_cam_axis: [f32; 3],
    pub camera_offset: [f32; 3],
    pub third_camera_offset: [f32; 3],
    pub move_step: [f32; 3],
    pub camera_rotation: [f32; 3],
    pub camera_rotation_default: [f32; 3],
    pub dir: [f32; 3],
    pub third_camera_rotation: [f32; 3],
    pub connor_mat: TMat3F,
    pub old_connor_mat: TMat3F,
    pub cam_rot_mode_x: i32,
    pub cam_rot_mode_z: i32,
    pub con_rel_move: i32,
    pub old_con_rel_move: i32,
    pub check_move: i32,
    pub collision_radius: i32,
    pub old_cam_rot_mode_z: i32,
    pub move_cam: i32,
    pub aim_cam: i32,
    pub in_pan: i32,
    pub first_pan: i32,
    pub first_zoom: i32,
    pub touched_terrain: i32,
    pub is_first_person: i32,
    pub constop: i32,
    pub n_move_step: i32,
    pub zip_cam: i32,
    pub ang_rot: i32,
    pub action_cam: i32,
    pub shake: i32,
    pub shake_frame: i32,
    pub user_height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KqMapIconRecord {
    pub map_icon: Option<String>,
    pub src_icon: Option<String>,
    pub x_icon: i32,
    pub y_icon: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqMap {
    pub visible: KqVisibleBase,
    pub north: i32,
    pub west: i32,
    pub south: i32,
    pub east: i32,
    pub use_fog: i32,
    pub fog_radius: i32,
    pub hold_map_loc: i32,
    pub is_active: i32,
    pub blank_map: Option<String>,
    pub full_map: Option<String>,
    pub arrow_pba: Option<String>,
    pub map_name: Option<String>,
    pub earned_teleport: i32,
    pub teleport_script: Option<String>,
    pub teleport_portal_kq: Option<String>,
    pub icons: Vec<KqMapIconRecord>,
    pub rle_runs: Vec<i32>,
}

impl KqMap {
    /// Converts a binary `PERS` `KQMap` into `kq8_data::magic_map::MapKeys`.
    pub fn to_map_keys(&self) -> MapKeys {
        let name = self.visible.base.name.clone().unwrap_or_default();
        let map_name = self
            .map_name
            .as_deref()
            .and_then(|s| s.trim().strip_prefix('#'))
            .and_then(|s| s.parse::<u32>().ok());
        MapKeys {
            name,
            north: self.north as f64,
            south: self.south as f64,
            east: self.east as f64,
            west: self.west as f64,
            use_fog: self.use_fog != 0,
            fog_radius: (self.fog_radius > 0).then_some(self.fog_radius as f64),
            hold_map_loc: self.hold_map_loc != 0,
            is_active: self.is_active != 0,
            blank_map: self.blank_map.clone().unwrap_or_default(),
            full_map: self.full_map.clone().unwrap_or_default(),
            arrow_pba: self
                .arrow_pba
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "maparrow.pba".into()),
            map_name,
            earned_teleport: self.earned_teleport != 0,
            teleport_script: self.teleport_script.clone().filter(|s| !s.is_empty()),
            teleport_portal: self.teleport_portal_kq.clone().filter(|s| !s.is_empty()),
            icons: self
                .icons
                .iter()
                .map(|i| MapIcon {
                    picture: i.map_icon.clone().unwrap_or_default(),
                    file: i.src_icon.clone().unwrap_or_default(),
                    x: i.x_icon,
                    y: i.y_icon,
                })
                .collect(),
            runs: self.rle_runs.iter().map(|&r| r as i64).collect(),
        }
    }

    /// Builds a binary `PERS` `KQMap` from `MapKeys`.
    pub fn from_map_keys(m: &MapKeys) -> Self {
        Self {
            visible: KqVisibleBase {
                base: KqBase {
                    name: Some(m.name.clone()),
                },
                active_3d_flag: 0,
                is_far: 0,
                too_far: 0.0,
                translucency: 256,
                haze_multiplier: 1.0,
                b_haze_solid: 0,
                hidden: 0,
            },
            north: m.north.round() as i32,
            west: m.west.round() as i32,
            south: m.south.round() as i32,
            east: m.east.round() as i32,
            use_fog: i32::from(m.use_fog),
            fog_radius: m.fog_radius.map_or(0, |r| r.round() as i32),
            hold_map_loc: i32::from(m.hold_map_loc),
            is_active: i32::from(m.is_active),
            blank_map: Some(m.blank_map.clone()),
            full_map: Some(m.full_map.clone()),
            arrow_pba: Some(m.arrow_pba.clone()),
            map_name: m.map_name.map(|n| format!("#{n}")),
            earned_teleport: i32::from(m.earned_teleport),
            teleport_script: m.teleport_script.clone(),
            teleport_portal_kq: m.teleport_portal.clone(),
            icons: m
                .icons
                .iter()
                .map(|ic| KqMapIconRecord {
                    map_icon: Some(ic.picture.clone()),
                    src_icon: Some(ic.file.clone()),
                    x_icon: ic.x,
                    y_icon: ic.y,
                })
                .collect(),
            rle_runs: m.runs.iter().map(|&r| r as i32).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeactivatedLight {
    pub f_type: i32,
    pub f_red: f32,
    pub f_green: f32,
    pub f_blue: f32,
    pub f_w_position: [f32; 3],
    pub f_range: f32,
    pub f_falloff: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangedHeight {
    pub h_startx: i32,
    pub h_starty: i32,
    pub h_endx: i32,
    pub h_endy: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqTerrain {
    pub base: KqBase,
    pub material_list_file: Option<String>,
    pub material_bmp: Option<String>,
    pub height_bmp: Option<String>,
    pub height_bmp2: Option<String>,
    pub height_bmp3: Option<String>,
    pub height_bmp4: Option<String>,
    pub height_bmp_scale: i32,
    pub material_direction: Option<String>,
    pub floor_bmp_name: Option<String>,
    pub visible_distance: f32,
    pub perspective_distance: f32,
    pub haze_distance: f32,
    pub ground_scale: i32,
    pub smoothing: i32,
    pub detail_table: [f32; 12],
    pub block_size: i32,
    pub light_scale: i32,
    pub envinfo_name: Option<String>,
    pub tedfile_name: Option<String>,
    pub lightmap_name: Option<String>,
    pub shadowmap_name: Option<String>,
    pub sw_lightmap_name: Option<String>,
    pub sw_shadowmap_name: Option<String>,
    pub deactivated_lights: Vec<DeactivatedLight>,
    pub changed_heights: Vec<ChangedHeight>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqInventory {
    pub base: KqBase,
    pub owner_id: u32,
    pub items: Vec<NestedNamed>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqObject {
    pub visible: KqVisibleBase,
    pub shape_name: Option<String>,
    pub accepts_inventory: Option<String>,
    pub collider_type: u32,
    pub collider_mask: u32,
    pub collision_shape: u32,
    pub collision_detail: u32,
    pub collision_priority: u32,
    pub step_up_amount: f32,
    pub step_down_amount: f32,
    pub groups_always: u32,
    pub groups_active_3d: u32,
    pub groups_active_3d_far: u32,
    pub groups_invisible: u32,
    pub immunities: u32,
    pub accept_all_items: i32,
    pub alternate_bmp: Option<String>,
    pub original_bmp: Option<String>,
    pub dithered: i32,
    pub ouch_sound: Option<String>,
    pub script: Option<String>,
    pub shadow_visible: i32,
    pub con_collision_action: i32,
    pub world_collision_action: i32,
    pub too_far_away_action: i32,
    pub shadow_mask: u32,
    pub do_not_point_at: i32,
    pub locked_resources: i32,
    pub footsound_type: i32,
    pub ouch_time: f32,
    pub projectile_file: Option<String>,
    pub shoot_offset: [f32; 3],
    pub rel_aim_point: [f32; 3],
    pub obj_proj_damage: i32,
    pub last_weapon_type: i32,
    pub env_type: u32,
    pub stats: Option<String>,
    pub mover_type: i32,
    pub mover: MoverData,
}

impl KqObject {
    /// Returns the object's transform (`TMat3F`) regardless of inline vs nested mover.
    pub fn transform(&self) -> Option<&TMat3F> {
        match &self.mover {
            MoverData::Inline { inline, .. } => Some(&inline.transform),
            MoverData::Nested(Some((_, child))) => match child.as_ref() {
                PersObject::KqMover(m) => Some(&m.inline.transform),
                PersObject::KqAnimMover { data, .. } => Some(&data.mover.inline.transform),
                _ => None,
            },
            MoverData::Nested(None) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqInventoryItem {
    pub object: KqObject,
    pub quantity: i32,
    pub user_id: u32,
    pub time_left: f32,
    pub item_type: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqInterior {
    pub object: KqObject,
    pub material: Option<String>,
    pub sw_lightmap_name: Option<String>,
    pub alt_sw_lightmap_name: Option<String>,
    pub alt_hw_lightmap_name: Option<String>,
    pub using_alt_lighting: i32,
    pub trap_collider: NestedNamed,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqAudibleObject {
    pub object: KqObject,
    pub sound_file_name: Option<String>,
    pub is_sound_3d: i32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub delta: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqSwampTile {
    pub object: KqObject,
    pub height_num: i32,
    pub dam_factor: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqRiverControl {
    pub object: KqObject,
    pub bb_min: [f32; 3],
    pub bb_max: [f32; 3],
    pub stretch_dist: f32,
    pub dir: i32,
    pub rusted_one: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqAnimLoopEntry {
    pub movement_name: Option<String>,
    pub loop_spec: Option<String>,
    pub cues: Vec<Option<String>>,
    pub transitions: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqAnimLoopList {
    pub base: KqBase,
    pub movements: Vec<KqAnimLoopEntry>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqAnimObject {
    pub object: KqObject,
    pub anim_pos: f32,
    pub curr_anim_name: Option<String>,
    pub next_anim_name: Option<String>,
    pub pre_trans_anim_name: Option<String>,
    pub post_trans_anim_name: Option<String>,
    pub next_anim_start_time: f32,
    pub post_trans_anim_start_time: f32,
    pub flags: u32,
    pub flying_object: i32,
    pub turn_off_dyn_lighting: i32,
    pub anim_loops: NestedNamed,
    pub owns_anim_list: i32,
    pub notify_object: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqAnimOnce {
    pub anim_object: KqAnimObject,
    pub delete_when_done: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqDoor {
    pub anim_object: KqAnimObject,
    pub state: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqChest {
    pub anim_object: KqAnimObject,
    pub inv_item_type_name: Option<String>,
    pub inv_quantity: i32,
    pub inv_loc: [f32; 3],
    pub inv_dir: [f32; 3],
    pub chest_state: i32,
    pub inv_item_name: Option<String>,
    pub take_msg: [i32; 6],
    pub empty_msg: [i32; 6],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqPortal {
    pub anim_object: KqAnimObject,
    pub world: Option<String>,
    pub enter_anim: Option<String>,
    pub exit_anim: Option<String>,
    pub enter_anm_file: Option<String>,
    pub exit_anm_file: Option<String>,
    pub exit_script: Option<String>,
    pub dest_connor: TMat3F,
    pub dest_cam: TMat3F,
    pub no_loc: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqBoulder {
    pub anim_object: KqAnimObject,
    pub prev_pos: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqLavaTile {
    pub anim_object: KqAnimObject,
    pub hold_flag: i32,
    pub top_bmp: Option<String>,
    pub reversed: i32,
    pub metal: i32,
    pub unknown: i32,
    pub activated: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqTrap {
    pub visible: KqVisibleBase,
    pub loc: [f32; 3],
    pub trap_type: u32,
    pub inner_percent: f32,
    pub script: Option<String>,
    pub collider_mask: u32,
    pub bb_min: [f32; 3],
    pub bb_max: [f32; 3],
    pub lookup_call: i32,
    pub shrunk: i32,
    pub trapped_object: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqSpiriTrap {
    pub trap: KqTrap,
    pub spirit_state: i32,
    pub trap_status: i32,
    pub damage: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqHazer {
    pub visible: KqVisibleBase,
    pub loc: [f32; 3],
    pub inner_radius: f32,
    pub collider_mask: u32,
    pub radius: f32,
    pub collided: i32,
    pub haze_center_factor: f32,
    pub orig_haze_edge: f32,
    pub haze_edge: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqPointLight {
    pub visible: KqVisibleBase,
    pub ts_light_type: i32,
    pub position: [f32; 3],
    pub intensity: [f32; 3],
    pub attenuation: [f32; 2],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqTerrainAnimator {
    pub visible: KqVisibleBase,
    pub start_now: i32,
    pub repeat_flag: i32,
    pub frame_count: i32,
    pub delay: f32,
    pub material_index: Vec<i32>,
    pub material_index_ii: Vec<i32>,
    pub grid_blocks: Vec<[i32; 2]>,
    pub orig_textures: Vec<i32>,
    pub exclude: Vec<[i32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqLips {
    pub base: KqBase,
    pub lips: [Option<String>; 8],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqConversation {
    pub visible: KqVisibleBase,
    pub conv_loc: [f32; 3],
    pub list1: Vec<Option<String>>,
    pub list2: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqTalk {
    pub visible: KqVisibleBase,
    pub flags: i32,
    pub nostop: i32,
    pub no_fade: i32,
    pub talker_name: Option<String>,
    pub talk_list: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqMonster {
    pub anim_object: KqAnimObject,
    pub state: NestedNamed,
    pub next_state: i32,
    pub next_state_data: Option<String>,
    pub state_change_notify_group_name: Option<String>,
    pub state_change_members: Vec<Option<String>>,
    pub dependents: Vec<Option<String>>,
    pub guarding_dependent: i32,
    pub mon_tick_count: i32,
    pub ticks_since_seen_connor: i32,
    pub monster_type: Option<String>,
    pub alarm_radius: f32,
    pub returning_to_home_radius: i32,
    pub b_must_go_home: i32,
    pub keeping_distance: i32,
    pub home_radius: f32,
    pub chase_radius: f32,
    pub attack_dist: f32,
    pub attack_too_close: f32,
    pub attack_far_enough: f32,
    pub likely_to_flee: f32,
    pub must_flee: i32,
    pub attack_delay: f32,
    pub birth_delay: f32,
    pub alarm_time: f32,
    pub attack_time: f32,
    pub chase_time: f32,
    pub react_time: f32,
    pub hits_since_alarmed: i32,
    pub do_mode: i32,
    pub do_loc: [f32; 3],
    pub looked_up_projectile: i32,
    pub conner_dist: f32,
    pub conner_time: f32,
    pub last_conner_pos: [f32; 3],
    pub can_see_conner: i32,
    pub can_always_see_conner: i32,
    pub line_of_sight_offset: f32,
    pub con_vec: [f32; 3],
    pub in_good_bye: i32,
    pub no_turn: i32,
    pub start_loc: [f32; 3],
    pub min_wander_dist: f32,
    pub dead_dts: Option<String>,
    pub ided_kqs: Option<String>,
    pub projd_kqs: Option<String>,
    pub dead_bmp: Option<String>,
    pub normal_react_percent: f32,
    pub special_react_anim: Option<String>,
    pub special_react_percent: f32,
    pub special_connor_react_anim: Option<String>,
    pub idle_time: f32,
    pub idling: i32,
    pub active: i32,
    pub move_anim: Option<String>,
    pub last_anim_played: Option<String>,
    pub anim_file: Option<String>,
    pub idle_anim_file: Option<String>,
    pub saved_anim_list: Option<String>,
    pub inv_drop: Option<String>,
    pub dropped_weapon: Option<String>,
    pub self_member: i32,
    pub mon_use_gravity: i32,
    pub gb: [i32; 5],
    pub wb: [i32; 5],
    pub alarm: [i32; 6],
    pub surprise: [i32; 6],
    pub said_alarm_message: i32,
    pub chase_object: Option<String>,
    pub team: Option<String>,
    pub wander_pts: Vec<[f32; 3]>,
    pub wanders: i32,
    pub wander_and_stop: i32,
    pub cur_wander_pt: i32,
    pub shoot_offsets: Vec<Option<String>>,
    pub inventory: NestedNamed,
    pub base_lip: Option<String>,
    pub lips: NestedNamed,
    pub conversation: NestedNamed,
    pub talker: NestedNamed,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqTowerSkeleton {
    pub monster: KqMonster,
    pub real_alarm: f32,
    pub move_time: f32,
    pub time_to_run: f32,
    pub stun_pause: f32,
    pub react_pause: f32,
    pub b_greeted_connor: i32,
    pub old_cam_pos: [f32; 3],
    pub b_reacting_back: i32,
    pub b_screamed: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqMandragorRoot {
    pub monster: KqMonster,
    pub offset: [f32; 3],
    pub orig_health: i32,
    pub orig_birth_delay: f32,
    pub rebirth_delay: f32,
    pub repel_dist: f32,
    pub too_close: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqMandragorTree {
    pub monster: KqMonster,
    pub root_down_count: i32,
    pub max_roots: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqConAnimList {
    pub base: KqBase,
    pub entries: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqSkinList {
    pub base: KqBase,
    pub skins: Vec<Option<String>>,
    pub inv_items: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KqConner {
    pub monster: KqMonster,
    pub zoom_uly: f32,
    pub zoom_ulx: f32,
    pub zoom_lry: f32,
    pub zoom_lrx: f32,
    pub allow_shadows_in_game: u32,
    pub next_do_mode: i32,
    pub required_weapon_mode: i32,
    pub required_weapon_script: Option<String>,
    pub hands_off: i32,
    pub gui_bottom: i32,
    pub talk_gui_bottom: i32,
    pub hands_off_cursor: i32,
    pub anim_set_hands_off: i32,
    pub solved_tile3: i32,
    pub connor_exited: i32,
    pub first_person: i32,
    pub tile_puzzle_jump: i32,
    pub in_weapon_do_mode: i32,
    pub attack_fire_frame: i32,
    pub bow_attacking: i32,
    pub b_walk_to_loc: i32,
    pub b_can_change_fp_mode: i32,
    pub walk_to_location: [f32; 3],
    pub point_cue_flag: i32,
    pub point_cue_i1: i32,
    pub point_cue_i2: i32,
    pub walk_cue_flag: i32,
    pub run_cue_flag: i32,
    pub walk_cue_i1: i32,
    pub walk_cue_i2: i32,
    pub max_damage_factor: f32,
    pub min_damage_factor: f32,
    pub run_to_walk: i32,
    pub terrain_sound: i32,
    pub rope_status: i32,
    pub rope_loc: [f32; 3],
    pub rope_dir: [f32; 3],
    pub rope_cam_loc1: [f32; 3],
    pub rope_cam_loc2: [f32; 3],
    pub rope_cam2_loc: [f32; 3],
    pub rope_cam_ang: f32,
    pub rope_cam_range: f32,
    pub rope_cam_top_limit: f32,
    pub rope_cam_bottom_limit: f32,
    pub box_up_time: f32,
    pub wait_exit_time: f32,
    pub last_played_scroll: f32,
    pub locked_state_id: i32,
    pub locked_state_data: Option<String>,
    pub was_in_first: i32,
    pub was_in_third: i32,
    pub start_help: i32,
    pub world_flags: i32,
    pub was_in_bow_mode: i32,
    pub point_cue_obj: Option<String>,
    pub walk_cue_obj: Option<String>,
    pub allow_leaning: i32,
    pub level: i32,
    pub item_in_hand: Option<String>,
    pub clarity_spell: i32,
    pub strength_spell: i32,
    pub invulnerability_spell: i32,
    pub invisibility_spell: i32,
    pub strength_spell_points: i32,
    pub armor_before_spell: i32,
    pub alt_short: i32,
    pub alt_long: i32,
    pub alt_armor: i32,
    pub talk_mon_name: Option<String>,
    pub talk_enemy: i32,
    pub talk_done: i32,
    pub talk_mon_loc: [f32; 3],
    pub anim_list: NestedNamed,
    pub short_anim_list: NestedNamed,
    pub long_anim_list: NestedNamed,
    pub fp_anim_list: NestedNamed,
    pub short_fp_anim_list: NestedNamed,
    pub long_fp_anim_list: NestedNamed,
    pub skin_list: NestedNamed,
    pub skin_list_firstperson: NestedNamed,
    pub lips_list: NestedNamed,
    pub cur_skin: i32,
    pub old_skin: i32,
    pub cur_lips: i32,
    pub old_lips: i32,
    pub hold_help: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KqStateKind {
    Wait {
        class: String,
        con_radius: f32,
        mon_radius: f32,
    },
    Move {
        con_radius: f32,
        mon_radius: f32,
        started_moving: i32,
    },
    Unborn {
        has_pre_birth: i32,
        ready_to_birth: i32,
    },
    ConnerPortal {
        entered_portal: i32,
        done_portal: i32,
        processed_gui: i32,
        portal_name: Option<String>,
        trap_name: Option<String>,
        portal_enter_anim: Option<String>,
        portal_exit_anim: Option<String>,
        portal_enter_anm_file: Option<String>,
        portal_exit_anm_file: Option<String>,
        portal_exit_script: Option<String>,
    },
    ConnerHookRope {
        done_roping: i32,
        reached_top: i32,
        do_mode_before: i32,
        old_cam_pos: [f32; 3],
        con_pos: [f32; 3],
        on_rope: i32,
        hide_icon: i32,
        first_to_third: i32,
        bottom_cam_limit: f32,
        top_cam_limit: f32,
        con_cam_bottom_limit: f32,
        con_cam_top_limit: f32,
        rope_cam_loc1: [f32; 3],
        rope_cam_loc2: [f32; 3],
        orig_step_up: f32,
        orig_step_down: f32,
    },
    ConnerSpecial {
        anim_command_string: Option<String>,
        cur_anim_list: Option<String>,
        anim_list: Option<String>,
    },
    ConnerPush {
        pushed: i32,
    },
    ConnerFall {
        done_falling: i32,
    },
    ConnerJump {
        done_jumping: i32,
    },
    /// Base `KQState` and `KQStateDie` (no extra fields beyond `KQBase` + `from_state`).
    Empty {
        class: String,
    },
    /// Fallback for undocumented late-world `KQState*` subclasses.
    Unknown {
        class: String,
        remaining: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct KqState {
    pub base: KqBase,
    pub from_state: i32,
    pub kind: KqStateKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PersObject {
    KqBase(KqBase),
    KqVisibleBase(KqVisibleBase),
    KqMover(KqMover),
    KqAnimMover { class: String, data: KqAnimMover },
    KqGameState(KqGameState),
    KqWorldState(KqWorldState),
    KqMusic(KqMusic),
    KqCamMover(KqCamMover),
    KqMap(KqMap),
    KqTerrain(KqTerrain),
    KqInventory { class: String, data: KqInventory },
    KqObject(KqObject),
    KqInventoryItem(KqInventoryItem),
    KqInterior(KqInterior),
    KqAudibleObject(KqAudibleObject),
    KqSwampTile(KqSwampTile),
    KqRiverControl(KqRiverControl),
    KqAnimLoopList(KqAnimLoopList),
    KqAnimObject(KqAnimObject),
    KqAnimOnce(KqAnimOnce),
    KqDoor(KqDoor),
    KqChest(KqChest),
    KqPortal(KqPortal),
    KqBoulder(KqBoulder),
    KqLavaTile(KqLavaTile),
    KqTrap(KqTrap),
    KqSpiriTrap(KqSpiriTrap),
    KqHazer(KqHazer),
    KqPointLight(KqPointLight),
    KqTerrainAnimator(KqTerrainAnimator),
    KqLips(KqLips),
    KqConversation(KqConversation),
    KqTalk(KqTalk),
    KqMonster { class: String, data: KqMonster },
    KqTowerSkeleton(KqTowerSkeleton),
    KqMandragorRoot(KqMandragorRoot),
    KqMandragorTree(KqMandragorTree),
    KqConAnimList(KqConAnimList),
    KqSkinList(KqSkinList),
    KqConner(Box<KqConner>),
    KqState(KqState),
}

impl PersObject {
    pub fn class_name(&self) -> &str {
        match self {
            PersObject::KqBase(_) => "KQBase",
            PersObject::KqVisibleBase(_) => "KQVisibleBase",
            PersObject::KqMover(_) => "KQMover",
            PersObject::KqAnimMover { class, .. } => class,
            PersObject::KqGameState(_) => "KQGameState",
            PersObject::KqWorldState(_) => "KQWorldState",
            PersObject::KqMusic(_) => "KQMusic",
            PersObject::KqCamMover(_) => "KQCamMover",
            PersObject::KqMap(_) => "KQMap",
            PersObject::KqTerrain(_) => "KQTerrain",
            PersObject::KqInventory { class, .. } => class,
            PersObject::KqObject(_) => "KQObject",
            PersObject::KqInventoryItem(_) => "KQInventoryItem",
            PersObject::KqInterior(_) => "KQInterior",
            PersObject::KqAudibleObject(_) => "KQAudibleObject",
            PersObject::KqSwampTile(_) => "KQSwampTile",
            PersObject::KqRiverControl(_) => "RiverControl",
            PersObject::KqAnimLoopList(_) => "KQAnimLoopList",
            PersObject::KqAnimObject(_) => "KQAnimObject",
            PersObject::KqAnimOnce(_) => "KQAnimOnce",
            PersObject::KqDoor(_) => "KQDoor",
            PersObject::KqChest(_) => "KQChest",
            PersObject::KqPortal(_) => "KQPortal",
            PersObject::KqBoulder(_) => "KQBoulder",
            PersObject::KqLavaTile(_) => "KQLavaTile",
            PersObject::KqTrap(_) => "KQTrap",
            PersObject::KqSpiriTrap(_) => "KQSpiriTrap",
            PersObject::KqHazer(_) => "KQHazer",
            PersObject::KqPointLight(_) => "KQPointLight",
            PersObject::KqTerrainAnimator(_) => "KQTerrainAnimator",
            PersObject::KqLips(_) => "KQLips",
            PersObject::KqConversation(_) => "KQConversation",
            PersObject::KqTalk(_) => "KQTalk",
            PersObject::KqMonster { class, .. } => class,
            PersObject::KqTowerSkeleton(_) => "KQTowerSkeleton",
            PersObject::KqMandragorRoot(_) => "KQMandragorRoot",
            PersObject::KqMandragorTree(_) => "KQMandragorTree",
            PersObject::KqConAnimList(_) => "KQConAnimList",
            PersObject::KqSkinList(_) => "KQSkinList",
            PersObject::KqConner(_) => "KQConner",
            PersObject::KqState(s) => match &s.kind {
                KqStateKind::Wait { class, .. } => class,
                KqStateKind::Move { .. } => "KQStateMove",
                KqStateKind::Unborn { .. } => "KQStateUnborn",
                KqStateKind::ConnerPortal { .. } => "KQStateConnerPortal",
                KqStateKind::ConnerHookRope { .. } => "KQStateConnerHookRope",
                KqStateKind::ConnerSpecial { .. } => "KQStateConnerSpecial",
                KqStateKind::ConnerPush { .. } => "KQStateConnerPush",
                KqStateKind::ConnerFall { .. } => "KQStateConnerFall",
                KqStateKind::ConnerJump { .. } => "KQStateConnerJump",
                KqStateKind::Empty { class } => class,
                KqStateKind::Unknown { class, .. } => class,
            },
        }
    }

    pub fn base(&self) -> &KqBase {
        match self {
            PersObject::KqBase(b) => b,
            PersObject::KqVisibleBase(v) => &v.base,
            PersObject::KqMover(m) => &m.visible.base,
            PersObject::KqAnimMover { data, .. } => &data.mover.visible.base,
            PersObject::KqGameState(g) => &g.base,
            PersObject::KqWorldState(w) => &w.base,
            PersObject::KqMusic(m) => &m.visible.base,
            PersObject::KqCamMover(c) => &c.visible.base,
            PersObject::KqMap(m) => &m.visible.base,
            PersObject::KqTerrain(t) => &t.base,
            PersObject::KqInventory { data, .. } => &data.base,
            PersObject::KqObject(o) => &o.visible.base,
            PersObject::KqInventoryItem(i) => &i.object.visible.base,
            PersObject::KqInterior(i) => &i.object.visible.base,
            PersObject::KqAudibleObject(a) => &a.object.visible.base,
            PersObject::KqSwampTile(s) => &s.object.visible.base,
            PersObject::KqRiverControl(r) => &r.object.visible.base,
            PersObject::KqAnimLoopList(l) => &l.base,
            PersObject::KqAnimObject(a) => &a.object.visible.base,
            PersObject::KqAnimOnce(a) => &a.anim_object.object.visible.base,
            PersObject::KqDoor(d) => &d.anim_object.object.visible.base,
            PersObject::KqChest(c) => &c.anim_object.object.visible.base,
            PersObject::KqPortal(p) => &p.anim_object.object.visible.base,
            PersObject::KqBoulder(b) => &b.anim_object.object.visible.base,
            PersObject::KqLavaTile(l) => &l.anim_object.object.visible.base,
            PersObject::KqTrap(t) => &t.visible.base,
            PersObject::KqSpiriTrap(t) => &t.trap.visible.base,
            PersObject::KqHazer(h) => &h.visible.base,
            PersObject::KqPointLight(p) => &p.visible.base,
            PersObject::KqTerrainAnimator(t) => &t.visible.base,
            PersObject::KqLips(l) => &l.base,
            PersObject::KqConversation(c) => &c.visible.base,
            PersObject::KqTalk(t) => &t.visible.base,
            PersObject::KqMonster { data, .. } => &data.anim_object.object.visible.base,
            PersObject::KqTowerSkeleton(t) => &t.monster.anim_object.object.visible.base,
            PersObject::KqMandragorRoot(r) => &r.monster.anim_object.object.visible.base,
            PersObject::KqMandragorTree(t) => &t.monster.anim_object.object.visible.base,
            PersObject::KqConAnimList(l) => &l.base,
            PersObject::KqSkinList(l) => &l.base,
            PersObject::KqConner(c) => &c.monster.anim_object.object.visible.base,
            PersObject::KqState(s) => &s.base,
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
    dict: Vec<Option<PersObject>>,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            pos: 0,
            dict: Vec::new(),
        }
    }

    fn err(&self, msg: impl Into<String>) -> String {
        format!("{} at offset 0x{:X}", msg.into(), self.pos)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.pos.checked_add(n).ok_or_else(|| self.err("offset overflow"))?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| self.err(format!("need {n} bytes, only {} left", self.bytes.len().saturating_sub(self.pos))))?;
        self.pos = end;
        Ok(slice)
    }

    fn arr<const N: usize>(&mut self) -> Result<[u8; N], String> {
        Ok(self.take(N)?.try_into().unwrap())
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.arr()?))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.arr()?))
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.arr()?))
    }

    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.arr()?))
    }

    fn point3f(&mut self) -> Result<[f32; 3], String> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    fn tmat3f(&mut self) -> Result<TMat3F, String> {
        let flags = self.u32()?;
        let r0 = [self.f32()?, self.f32()?, self.f32()?];
        let r1 = [self.f32()?, self.f32()?, self.f32()?];
        let r2 = [self.f32()?, self.f32()?, self.f32()?];
        let position = self.point3f()?;
        Ok(TMat3F {
            flags,
            rotation: [r0, r1, r2],
            position,
        })
    }

    fn opt_string(&mut self) -> Result<Option<String>, String> {
        let len = self.u32()? as usize;
        if len == 0 {
            return Ok(None);
        }
        if len % 2 != 0 {
            return Err(self.err(format!("odd string padded length {len}")));
        }
        let raw = self.take(len)?;
        let nul = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        Ok(Some(String::from_utf8_lossy(&raw[..nul]).into_owned()))
    }

    fn count(&mut self, what: &str) -> Result<usize, String> {
        let n = self.i32()?;
        if !(0..=1_000_000).contains(&n) {
            return Err(self.err(format!("{what} count {n} out of range")));
        }
        Ok(n as usize)
    }

    fn string_vec(&mut self, what: &str) -> Result<Vec<Option<String>>, String> {
        let n = self.count(what)?;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(self.opt_string()?);
        }
        Ok(out)
    }

    fn point_vec(&mut self, what: &str) -> Result<Vec<[f32; 3]>, String> {
        let n = self.count(what)?;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(self.point3f()?);
        }
        Ok(out)
    }

    fn nested_named(&mut self) -> Result<NestedNamed, String> {
        let Some(name) = self.opt_string()? else {
            return Ok(None);
        };
        let obj = self.read_block()?;
        Ok(Some((name, Box::new(obj))))
    }

    fn read_base(&mut self) -> Result<KqBase, String> {
        Ok(KqBase {
            name: self.opt_string()?,
        })
    }

    fn read_visible(&mut self) -> Result<KqVisibleBase, String> {
        Ok(KqVisibleBase {
            base: self.read_base()?,
            active_3d_flag: self.i32()?,
            is_far: self.i32()?,
            too_far: self.f32()?,
            translucency: self.i32()?,
            haze_multiplier: self.f32()?,
            b_haze_solid: self.i32()?,
            hidden: self.i32()?,
        })
    }

    fn read_mover_inline(&mut self) -> Result<KqMoverInline, String> {
        Ok(KqMoverInline {
            transform: self.tmat3f()?,
            keep_on_ground: self.i32()?,
            use_gravity: self.i32()?,
            submerged: self.i32()?,
            max_slope_dot: self.f32()?,
        })
    }

    fn read_mover(&mut self) -> Result<KqMover, String> {
        Ok(KqMover {
            visible: self.read_visible()?,
            inline: self.read_mover_inline()?,
        })
    }

    fn read_anim_mover(&mut self) -> Result<KqAnimMover, String> {
        Ok(KqAnimMover {
            mover: self.read_mover()?,
            rotate_speed: self.f32()?,
            fast: self.i32()?,
            look_at: self.point3f()?,
            use_look_at: self.i32()?,
        })
    }

    fn read_object(&mut self) -> Result<KqObject, String> {
        let visible = self.read_visible()?;
        let shape_name = self.opt_string()?;
        let accepts_inventory = self.opt_string()?;
        let collider_type = self.u32()?;
        let collider_mask = self.u32()?;
        let collision_shape = self.u32()?;
        let collision_detail = self.u32()?;
        let collision_priority = self.u32()?;
        let step_up_amount = self.f32()?;
        let step_down_amount = self.f32()?;
        let groups_always = self.u32()?;
        let groups_active_3d = self.u32()?;
        let groups_active_3d_far = self.u32()?;
        let groups_invisible = self.u32()?;
        let immunities = self.u32()?;
        let accept_all_items = self.i32()?;
        let alternate_bmp = self.opt_string()?;
        let original_bmp = self.opt_string()?;
        let dithered = self.i32()?;
        let ouch_sound = self.opt_string()?;
        let script = self.opt_string()?;
        let shadow_visible = self.i32()?;
        let con_collision_action = self.i32()?;
        let world_collision_action = self.i32()?;
        let too_far_away_action = self.i32()?;
        let shadow_mask = self.u32()?;
        let do_not_point_at = self.i32()?;
        let locked_resources = self.i32()?;
        let footsound_type = self.i32()?;
        let ouch_time = self.f32()?;
        let projectile_file = self.opt_string()?;
        let shoot_offset = self.point3f()?;
        let rel_aim_point = self.point3f()?;
        let obj_proj_damage = self.i32()?;
        let last_weapon_type = self.i32()?;
        let env_type = self.u32()?;
        let stats = self.opt_string()?;
        let mover_type = self.i32()?;
        let mover = if mover_type == 0 {
            MoverData::Inline {
                mover_name: self.opt_string()?,
                inline: self.read_mover_inline()?,
            }
        } else {
            MoverData::Nested(self.nested_named()?)
        };
        Ok(KqObject {
            visible,
            shape_name,
            accepts_inventory,
            collider_type,
            collider_mask,
            collision_shape,
            collision_detail,
            collision_priority,
            step_up_amount,
            step_down_amount,
            groups_always,
            groups_active_3d,
            groups_active_3d_far,
            groups_invisible,
            immunities,
            accept_all_items,
            alternate_bmp,
            original_bmp,
            dithered,
            ouch_sound,
            script,
            shadow_visible,
            con_collision_action,
            world_collision_action,
            too_far_away_action,
            shadow_mask,
            do_not_point_at,
            locked_resources,
            footsound_type,
            ouch_time,
            projectile_file,
            shoot_offset,
            rel_aim_point,
            obj_proj_damage,
            last_weapon_type,
            env_type,
            stats,
            mover_type,
            mover,
        })
    }

    fn read_anim_object(&mut self) -> Result<KqAnimObject, String> {
        Ok(KqAnimObject {
            object: self.read_object()?,
            anim_pos: self.f32()?,
            curr_anim_name: self.opt_string()?,
            next_anim_name: self.opt_string()?,
            pre_trans_anim_name: self.opt_string()?,
            post_trans_anim_name: self.opt_string()?,
            next_anim_start_time: self.f32()?,
            post_trans_anim_start_time: self.f32()?,
            flags: self.u32()?,
            flying_object: self.i32()?,
            turn_off_dyn_lighting: self.i32()?,
            anim_loops: self.nested_named()?,
            owns_anim_list: self.i32()?,
            notify_object: self.opt_string()?,
        })
    }

    fn read_trap(&mut self) -> Result<KqTrap, String> {
        Ok(KqTrap {
            visible: self.read_visible()?,
            loc: self.point3f()?,
            trap_type: self.u32()?,
            inner_percent: self.f32()?,
            script: self.opt_string()?,
            collider_mask: self.u32()?,
            bb_min: self.point3f()?,
            bb_max: self.point3f()?,
            lookup_call: self.i32()?,
            shrunk: self.i32()?,
            trapped_object: self.string_vec("trappedObject")?,
        })
    }

    fn read_monster(&mut self) -> Result<KqMonster, String> {
        let anim_object = self.read_anim_object()?;
        let state = self.nested_named()?;
        let next_state = self.i32()?;
        let next_state_data = self.opt_string()?;
        let state_change_notify_group_name = self.opt_string()?;
        let state_change_members = self.string_vec("stateChangeMembers")?;
        let dependents = self.string_vec("dependents")?;
        let guarding_dependent = self.i32()?;
        let mon_tick_count = self.i32()?;
        let ticks_since_seen_connor = self.i32()?;
        let monster_type = self.opt_string()?;
        let alarm_radius = self.f32()?;
        let returning_to_home_radius = self.i32()?;
        let b_must_go_home = self.i32()?;
        let keeping_distance = self.i32()?;
        let home_radius = self.f32()?;
        let chase_radius = self.f32()?;
        let attack_dist = self.f32()?;
        let attack_too_close = self.f32()?;
        let attack_far_enough = self.f32()?;
        let likely_to_flee = self.f32()?;
        let must_flee = self.i32()?;
        let attack_delay = self.f32()?;
        let birth_delay = self.f32()?;
        let alarm_time = self.f32()?;
        let attack_time = self.f32()?;
        let chase_time = self.f32()?;
        let react_time = self.f32()?;
        let hits_since_alarmed = self.i32()?;
        let do_mode = self.i32()?;
        let do_loc = self.point3f()?;
        let looked_up_projectile = self.i32()?;
        let conner_dist = self.f32()?;
        let conner_time = self.f32()?;
        let last_conner_pos = self.point3f()?;
        let can_see_conner = self.i32()?;
        let can_always_see_conner = self.i32()?;
        let line_of_sight_offset = self.f32()?;
        let con_vec = self.point3f()?;
        let in_good_bye = self.i32()?;
        let no_turn = self.i32()?;
        let start_loc = self.point3f()?;
        let min_wander_dist = self.f32()?;
        let dead_dts = self.opt_string()?;
        let ided_kqs = self.opt_string()?;
        let projd_kqs = self.opt_string()?;
        let dead_bmp = self.opt_string()?;
        let normal_react_percent = self.f32()?;
        let special_react_anim = self.opt_string()?;
        let special_react_percent = self.f32()?;
        let special_connor_react_anim = self.opt_string()?;
        let idle_time = self.f32()?;
        let idling = self.i32()?;
        let active = self.i32()?;
        let move_anim = self.opt_string()?;
        let last_anim_played = self.opt_string()?;
        let anim_file = self.opt_string()?;
        let idle_anim_file = self.opt_string()?;
        let saved_anim_list = self.opt_string()?;
        let inv_drop = self.opt_string()?;
        let dropped_weapon = self.opt_string()?;
        let self_member = self.i32()?;
        let mon_use_gravity = self.i32()?;
        let gb = [self.i32()?, self.i32()?, self.i32()?, self.i32()?, self.i32()?];
        let wb = [self.i32()?, self.i32()?, self.i32()?, self.i32()?, self.i32()?];
        let alarm = [self.i32()?, self.i32()?, self.i32()?, self.i32()?, self.i32()?, self.i32()?];
        let surprise = [self.i32()?, self.i32()?, self.i32()?, self.i32()?, self.i32()?, self.i32()?];
        let said_alarm_message = self.i32()?;
        let chase_object = self.opt_string()?;
        let team = self.opt_string()?;
        let wander_pts = self.point_vec("wanderPts")?;
        let wanders = self.i32()?;
        let wander_and_stop = self.i32()?;
        let cur_wander_pt = self.i32()?;
        let shoot_offsets = self.string_vec("shootOffsets")?;
        let inventory = self.nested_named()?;
        let base_lip = self.opt_string()?;
        let lips = self.nested_named()?;
        let conversation = self.nested_named()?;
        let talker = self.nested_named()?;
        Ok(KqMonster {
            anim_object,
            state,
            next_state,
            next_state_data,
            state_change_notify_group_name,
            state_change_members,
            dependents,
            guarding_dependent,
            mon_tick_count,
            ticks_since_seen_connor,
            monster_type,
            alarm_radius,
            returning_to_home_radius,
            b_must_go_home,
            keeping_distance,
            home_radius,
            chase_radius,
            attack_dist,
            attack_too_close,
            attack_far_enough,
            likely_to_flee,
            must_flee,
            attack_delay,
            birth_delay,
            alarm_time,
            attack_time,
            chase_time,
            react_time,
            hits_since_alarmed,
            do_mode,
            do_loc,
            looked_up_projectile,
            conner_dist,
            conner_time,
            last_conner_pos,
            can_see_conner,
            can_always_see_conner,
            line_of_sight_offset,
            con_vec,
            in_good_bye,
            no_turn,
            start_loc,
            min_wander_dist,
            dead_dts,
            ided_kqs,
            projd_kqs,
            dead_bmp,
            normal_react_percent,
            special_react_anim,
            special_react_percent,
            special_connor_react_anim,
            idle_time,
            idling,
            active,
            move_anim,
            last_anim_played,
            anim_file,
            idle_anim_file,
            saved_anim_list,
            inv_drop,
            dropped_weapon,
            self_member,
            mon_use_gravity,
            gb,
            wb,
            alarm,
            surprise,
            said_alarm_message,
            chase_object,
            team,
            wander_pts,
            wanders,
            wander_and_stop,
            cur_wander_pt,
            shoot_offsets,
            inventory,
            base_lip,
            lips,
            conversation,
            talker,
        })
    }

    fn read_block(&mut self) -> Result<PersObject, String> {
        let tag = self.take(4)?;
        if tag == b"DICT" {
            let size = self.u32()?;
            if size != 4 {
                return Err(self.err(format!("unexpected DICT size {size}")));
            }
            let index = self.u32()? as usize;
            return self
                .dict
                .get(index)
                .and_then(|slot| slot.as_ref())
                .cloned()
                .ok_or_else(|| self.err(format!("invalid DICT index {index}")));
        }
        if tag != b"PERS" {
            return Err(self.err(format!("expected PERS or DICT, found {:02X?}", tag)));
        }
        let payload_size = self.u32()? as usize;
        if payload_size & 0x8000_0000 != 0 {
            return Err(self.err("unexpected 4-byte aligned PERS flag in save chunk"));
        }
        let payload_start = self.pos;
        let payload_end = payload_start
            .checked_add(payload_size)
            .ok_or_else(|| self.err("PERS payload overflow"))?;
        if payload_end > self.bytes.len() {
            return Err(self.err(format!(
                "PERS payload size {payload_size} exceeds input ({} left)",
                self.bytes.len() - payload_start
            )));
        }

        let name_len = self.u16()? as usize;
        if !(1..=128).contains(&name_len) {
            return Err(self.err(format!("PERS class name length {name_len} out of range")));
        }
        let padded_name = (name_len + 1) & !1;
        let name_bytes = self.take(padded_name)?;
        let class = String::from_utf8_lossy(&name_bytes[..name_len]).into_owned();

        let dict_slot = self.dict.len();
        self.dict.push(None);

        let obj = match class.as_str() {
            "KQBase" => PersObject::KqBase(self.read_base()?),
            "KQVisibleBase" => PersObject::KqVisibleBase(self.read_visible()?),
            "KQMover" => PersObject::KqMover(self.read_mover()?),
            "KQAnimMover" | "KQAnimFlyingMover" => PersObject::KqAnimMover {
                class: class.clone(),
                data: self.read_anim_mover()?,
            },
            "KQGameState" => PersObject::KqGameState(KqGameState {
                base: self.read_base()?,
                next_no_name: self.i32()?,
                difficulty_factor: self.f32()?,
            }),
            "KQWorldState" => PersObject::KqWorldState(KqWorldState {
                base: self.read_base()?,
                events: self.string_vec("events")?,
            }),
            "KQMusic" => PersObject::KqMusic(KqMusic {
                visible: self.read_visible()?,
                music: self.string_vec("music")?,
            }),
            "KQCamMover" => PersObject::KqCamMover(KqCamMover {
                visible: self.read_visible()?,
                follow_object_name: self.opt_string()?,
                pause_follow: self.i32()?,
                near_plane: self.f32()?,
                far_plane: self.f32()?,
                fov: self.f32()?,
                tilt_flag: self.f32()?,
                zoom_flag: self.f32()?,
                pan_flag: self.f32()?,
                fix_height: self.i32()?,
                old_pan_flag: self.f32()?,
                old_zoom_flag: self.f32()?,
                old_ang_diff_z: self.f32()?,
                cam_move_radius: self.f32()?,
                short_cam_move_radius: self.f32()?,
                long_cam_move_radius: self.f32()?,
                cam_rotate_amount: self.f32()?,
                conner_min_dist: self.f32()?,
                cam_fly_dist: self.f32()?,
                ang_total: self.f32()?,
                ang_step: self.f32()?,
                roll_step: self.f32()?,
                cam_con_len: self.f32()?,
                shake_duration: self.f32()?,
                x_shake_max: self.f32()?,
                z_shake_max: self.f32()?,
                x_shake_amp: self.f32()?,
                z_shake_amp: self.f32()?,
                old_cmrx: self.f32()?,
                con_head_height: self.f32()?,
                cam_height1: self.f32()?,
                cam_height2: self.f32()?,
                x_shake_period: self.f32()?,
                z_shake_period: self.f32()?,
                loc: self.point3f()?,
                old_cam_axis: self.point3f()?,
                camera_offset: self.point3f()?,
                third_camera_offset: self.point3f()?,
                move_step: self.point3f()?,
                camera_rotation: self.point3f()?,
                camera_rotation_default: self.point3f()?,
                dir: self.point3f()?,
                third_camera_rotation: self.point3f()?,
                connor_mat: self.tmat3f()?,
                old_connor_mat: self.tmat3f()?,
                cam_rot_mode_x: self.i32()?,
                cam_rot_mode_z: self.i32()?,
                con_rel_move: self.i32()?,
                old_con_rel_move: self.i32()?,
                check_move: self.i32()?,
                collision_radius: self.i32()?,
                old_cam_rot_mode_z: self.i32()?,
                move_cam: self.i32()?,
                aim_cam: self.i32()?,
                in_pan: self.i32()?,
                first_pan: self.i32()?,
                first_zoom: self.i32()?,
                touched_terrain: self.i32()?,
                is_first_person: self.i32()?,
                constop: self.i32()?,
                n_move_step: self.i32()?,
                zip_cam: self.i32()?,
                ang_rot: self.i32()?,
                action_cam: self.i32()?,
                shake: self.i32()?,
                shake_frame: self.i32()?,
                user_height: self.i32()?,
            }),
            "KQMap" => {
                let visible = self.read_visible()?;
                let north = self.i32()?;
                let west = self.i32()?;
                let south = self.i32()?;
                let east = self.i32()?;
                let use_fog = self.i32()?;
                let fog_radius = self.i32()?;
                let hold_map_loc = self.i32()?;
                let is_active = self.i32()?;
                let blank_map = self.opt_string()?;
                let full_map = self.opt_string()?;
                let arrow_pba = self.opt_string()?;
                let map_name = self.opt_string()?;
                let earned_teleport = self.i32()?;
                let teleport_script = self.opt_string()?;
                let teleport_portal_kq = self.opt_string()?;
                let c_map_icons = self.count("cMapIcons")?;
                let mut icons = Vec::with_capacity(c_map_icons);
                for _ in 0..c_map_icons {
                    icons.push(KqMapIconRecord {
                        map_icon: self.opt_string()?,
                        src_icon: self.opt_string()?,
                        x_icon: self.i32()?,
                        y_icon: self.i32()?,
                    });
                }
                let mut rle_runs = Vec::new();
                loop {
                    let run = self.i32()?;
                    if run < 0 {
                        break;
                    }
                    rle_runs.push(run);
                }
                PersObject::KqMap(KqMap {
                    visible,
                    north,
                    west,
                    south,
                    east,
                    use_fog,
                    fog_radius,
                    hold_map_loc,
                    is_active,
                    blank_map,
                    full_map,
                    arrow_pba,
                    map_name,
                    earned_teleport,
                    teleport_script,
                    teleport_portal_kq,
                    icons,
                    rle_runs,
                })
            }
            "KQTerrain" => {
                let base = self.read_base()?;
                let material_list_file = self.opt_string()?;
                let material_bmp = self.opt_string()?;
                let height_bmp = self.opt_string()?;
                let height_bmp2 = self.opt_string()?;
                let height_bmp3 = self.opt_string()?;
                let height_bmp4 = self.opt_string()?;
                let height_bmp_scale = self.i32()?;
                let material_direction = self.opt_string()?;
                let floor_bmp_name = self.opt_string()?;
                let visible_distance = self.f32()?;
                let perspective_distance = self.f32()?;
                let haze_distance = self.f32()?;
                let ground_scale = self.i32()?;
                let smoothing = self.i32()?;
                let mut detail_table = [0.0f32; 12];
                for slot in &mut detail_table {
                    *slot = self.f32()?;
                }
                let block_size = self.i32()?;
                let light_scale = self.i32()?;
                let envinfo_name = self.opt_string()?;
                let tedfile_name = self.opt_string()?;
                let lightmap_name = self.opt_string()?;
                let shadowmap_name = self.opt_string()?;
                let sw_lightmap_name = self.opt_string()?;
                let sw_shadowmap_name = self.opt_string()?;
                let n_deactivated = self.count("nDeactivatedLights")?;
                let mut deactivated_lights = Vec::with_capacity(n_deactivated);
                for _ in 0..n_deactivated {
                    deactivated_lights.push(DeactivatedLight {
                        f_type: self.i32()?,
                        f_red: self.f32()?,
                        f_green: self.f32()?,
                        f_blue: self.f32()?,
                        f_w_position: self.point3f()?,
                        f_range: self.f32()?,
                        f_falloff: self.f32()?,
                    });
                }
                let n_changed = self.count("nChangedHeights")?;
                let mut changed_heights = Vec::with_capacity(n_changed);
                for _ in 0..n_changed {
                    changed_heights.push(ChangedHeight {
                        h_startx: self.i32()?,
                        h_starty: self.i32()?,
                        h_endx: self.i32()?,
                        h_endy: self.i32()?,
                    });
                }
                PersObject::KqTerrain(KqTerrain {
                    base,
                    material_list_file,
                    material_bmp,
                    height_bmp,
                    height_bmp2,
                    height_bmp3,
                    height_bmp4,
                    height_bmp_scale,
                    material_direction,
                    floor_bmp_name,
                    visible_distance,
                    perspective_distance,
                    haze_distance,
                    ground_scale,
                    smoothing,
                    detail_table,
                    block_size,
                    light_scale,
                    envinfo_name,
                    tedfile_name,
                    lightmap_name,
                    shadowmap_name,
                    sw_lightmap_name,
                    sw_shadowmap_name,
                    deactivated_lights,
                    changed_heights,
                })
            }
            "KQInventory" | "KQWorldInventory" | "KQConnerInventory" => {
                let base = self.read_base()?;
                let owner_id = self.u32()?;
                let num_items = self.count("numItems")?;
                let mut items = Vec::with_capacity(num_items);
                for _ in 0..num_items {
                    items.push(self.nested_named()?);
                }
                PersObject::KqInventory {
                    class: class.clone(),
                    data: KqInventory {
                        base,
                        owner_id,
                        items,
                    },
                }
            }
            "KQObject" => PersObject::KqObject(self.read_object()?),
            "KQInventoryItem" => PersObject::KqInventoryItem(KqInventoryItem {
                object: self.read_object()?,
                quantity: self.i32()?,
                user_id: self.u32()?,
                time_left: self.f32()?,
                item_type: self.i32()?,
            }),
            "KQInterior" => PersObject::KqInterior(KqInterior {
                object: self.read_object()?,
                material: self.opt_string()?,
                sw_lightmap_name: self.opt_string()?,
                alt_sw_lightmap_name: self.opt_string()?,
                alt_hw_lightmap_name: self.opt_string()?,
                using_alt_lighting: self.i32()?,
                trap_collider: self.nested_named()?,
            }),
            "KQAudibleObject" => PersObject::KqAudibleObject(KqAudibleObject {
                object: self.read_object()?,
                sound_file_name: self.opt_string()?,
                is_sound_3d: self.i32()?,
                min_distance: self.f32()?,
                max_distance: self.f32()?,
                delta: self.point3f()?,
            }),
            "KQSwampTile" => PersObject::KqSwampTile(KqSwampTile {
                object: self.read_object()?,
                height_num: self.i32()?,
                dam_factor: self.f32()?,
            }),
            "RiverControl" => PersObject::KqRiverControl(KqRiverControl {
                object: self.read_object()?,
                bb_min: self.point3f()?,
                bb_max: self.point3f()?,
                stretch_dist: self.f32()?,
                dir: self.i32()?,
                rusted_one: self.i32()?,
            }),
            "KQAnimLoopList" => {
                let base = self.read_base()?;
                let n_movement = self.count("nMovement")?;
                let mut movements = Vec::with_capacity(n_movement);
                for _ in 0..n_movement {
                    let movement_name = self.opt_string()?;
                    let loop_spec = self.opt_string()?;
                    let cues = self.string_vec("cues")?;
                    let transitions = self.string_vec("transitions")?;
                    movements.push(KqAnimLoopEntry {
                        movement_name,
                        loop_spec,
                        cues,
                        transitions,
                    });
                }
                PersObject::KqAnimLoopList(KqAnimLoopList { base, movements })
            }
            "KQAnimObject" => PersObject::KqAnimObject(self.read_anim_object()?),
            "KQAnimOnce" => PersObject::KqAnimOnce(KqAnimOnce {
                anim_object: self.read_anim_object()?,
                delete_when_done: self.i32()?,
            }),
            "KQDoor" => PersObject::KqDoor(KqDoor {
                anim_object: self.read_anim_object()?,
                state: self.i32()?,
            }),
            "KQChest" => PersObject::KqChest(KqChest {
                anim_object: self.read_anim_object()?,
                inv_item_type_name: self.opt_string()?,
                inv_quantity: self.i32()?,
                inv_loc: self.point3f()?,
                inv_dir: self.point3f()?,
                chest_state: self.i32()?,
                inv_item_name: self.opt_string()?,
                take_msg: [
                    self.i32()?,
                    self.i32()?,
                    self.i32()?,
                    self.i32()?,
                    self.i32()?,
                    self.i32()?,
                ],
                empty_msg: [
                    self.i32()?,
                    self.i32()?,
                    self.i32()?,
                    self.i32()?,
                    self.i32()?,
                    self.i32()?,
                ],
            }),
            "KQPortal" => PersObject::KqPortal(KqPortal {
                anim_object: self.read_anim_object()?,
                world: self.opt_string()?,
                enter_anim: self.opt_string()?,
                exit_anim: self.opt_string()?,
                enter_anm_file: self.opt_string()?,
                exit_anm_file: self.opt_string()?,
                exit_script: self.opt_string()?,
                dest_connor: self.tmat3f()?,
                dest_cam: self.tmat3f()?,
                no_loc: self.point3f()?,
            }),
            "KQBoulder" => PersObject::KqBoulder(KqBoulder {
                anim_object: self.read_anim_object()?,
                prev_pos: self.point3f()?,
            }),
            "KQLavaTile" => PersObject::KqLavaTile(KqLavaTile {
                anim_object: self.read_anim_object()?,
                hold_flag: self.i32()?,
                top_bmp: self.opt_string()?,
                reversed: self.i32()?,
                metal: self.i32()?,
                unknown: self.i32()?,
                activated: self.i32()?,
            }),
            "KQTrap" => PersObject::KqTrap(self.read_trap()?),
            "KQSpiriTrap" => PersObject::KqSpiriTrap(KqSpiriTrap {
                trap: self.read_trap()?,
                spirit_state: self.i32()?,
                trap_status: self.i32()?,
                damage: self.f32()?,
            }),
            "KQHazer" => PersObject::KqHazer(KqHazer {
                visible: self.read_visible()?,
                loc: self.point3f()?,
                inner_radius: self.f32()?,
                collider_mask: self.u32()?,
                radius: self.f32()?,
                collided: self.i32()?,
                haze_center_factor: self.f32()?,
                orig_haze_edge: self.f32()?,
                haze_edge: self.f32()?,
            }),
            "KQPointLight" => PersObject::KqPointLight(KqPointLight {
                visible: self.read_visible()?,
                ts_light_type: self.i32()?,
                position: self.point3f()?,
                intensity: self.point3f()?,
                attenuation: [self.f32()?, self.f32()?],
            }),
            "KQTerrainAnimator" => {
                let visible = self.read_visible()?;
                let start_now = self.i32()?;
                let repeat_flag = self.i32()?;
                let frame_count = self.i32()?;
                let delay = self.f32()?;
                let n_mi = self.count("numMaterialIndex")?;
                let mut material_index = Vec::with_capacity(n_mi);
                for _ in 0..n_mi {
                    material_index.push(self.i32()?);
                }
                let n_mi2 = self.count("numMaterialIndexII")?;
                let mut material_index_ii = Vec::with_capacity(n_mi2);
                for _ in 0..n_mi2 {
                    material_index_ii.push(self.i32()?);
                }
                let n_gb = self.count("numGridBlocks")?;
                let mut grid_blocks = Vec::with_capacity(n_gb);
                for _ in 0..n_gb {
                    grid_blocks.push([self.i32()?, self.i32()?]);
                }
                let (orig_textures, exclude) = if n_gb == 0 {
                    let n_orig = self.count("numOrigTextures")?;
                    let mut orig = Vec::with_capacity(n_orig);
                    for _ in 0..n_orig {
                        orig.push(self.i32()?);
                    }
                    let n_exc = self.count("numExclude")?;
                    let mut exc = Vec::with_capacity(n_exc);
                    for _ in 0..n_exc {
                        exc.push([self.i32()?, self.i32()?]);
                    }
                    (orig, exc)
                } else {
                    (Vec::new(), Vec::new())
                };
                PersObject::KqTerrainAnimator(KqTerrainAnimator {
                    visible,
                    start_now,
                    repeat_flag,
                    frame_count,
                    delay,
                    material_index,
                    material_index_ii,
                    grid_blocks,
                    orig_textures,
                    exclude,
                })
            }
            "KQLips" => PersObject::KqLips(KqLips {
                base: self.read_base()?,
                lips: [
                    self.opt_string()?,
                    self.opt_string()?,
                    self.opt_string()?,
                    self.opt_string()?,
                    self.opt_string()?,
                    self.opt_string()?,
                    self.opt_string()?,
                    self.opt_string()?,
                ],
            }),
            "KQConversation" => PersObject::KqConversation(KqConversation {
                visible: self.read_visible()?,
                conv_loc: self.point3f()?,
                list1: self.string_vec("convList1")?,
                list2: self.string_vec("convList2")?,
            }),
            "KQTalk" => PersObject::KqTalk(KqTalk {
                visible: self.read_visible()?,
                flags: self.i32()?,
                nostop: self.i32()?,
                no_fade: self.i32()?,
                talker_name: self.opt_string()?,
                talk_list: self.string_vec("talkList")?,
            }),
            "KQMonster" | "KQGeometryTalker" | "KQDeath" => PersObject::KqMonster {
                class: class.clone(),
                data: self.read_monster()?,
            },
            "KQTowerSkeleton" => PersObject::KqTowerSkeleton(KqTowerSkeleton {
                monster: self.read_monster()?,
                real_alarm: self.f32()?,
                move_time: self.f32()?,
                time_to_run: self.f32()?,
                stun_pause: self.f32()?,
                react_pause: self.f32()?,
                b_greeted_connor: self.i32()?,
                old_cam_pos: self.point3f()?,
                b_reacting_back: self.i32()?,
                b_screamed: self.i32()?,
            }),
            "KQMandragorRoot" => PersObject::KqMandragorRoot(KqMandragorRoot {
                monster: self.read_monster()?,
                offset: self.point3f()?,
                orig_health: self.i32()?,
                orig_birth_delay: self.f32()?,
                rebirth_delay: self.f32()?,
                repel_dist: self.f32()?,
                too_close: self.f32()?,
            }),
            "KQMandragorTree" => PersObject::KqMandragorTree(KqMandragorTree {
                monster: self.read_monster()?,
                root_down_count: self.i32()?,
                max_roots: self.i32()?,
            }),
            "KQConAnimList" => PersObject::KqConAnimList(KqConAnimList {
                base: self.read_base()?,
                entries: self.string_vec("animList")?,
            }),
            "KQSkinList" => PersObject::KqSkinList(KqSkinList {
                base: self.read_base()?,
                skins: self.string_vec("skinList")?,
                inv_items: self.string_vec("invItems")?,
            }),
            "KQConner" => {
                let monster = self.read_monster()?;
                let zoom_uly = self.f32()?;
                let zoom_ulx = self.f32()?;
                let zoom_lry = self.f32()?;
                let zoom_lrx = self.f32()?;
                let allow_shadows_in_game = self.u32()?;
                let next_do_mode = self.i32()?;
                let required_weapon_mode = self.i32()?;
                let required_weapon_script = self.opt_string()?;
                let hands_off = self.i32()?;
                let gui_bottom = self.i32()?;
                let talk_gui_bottom = self.i32()?;
                let hands_off_cursor = self.i32()?;
                let anim_set_hands_off = self.i32()?;
                let solved_tile3 = self.i32()?;
                let connor_exited = self.i32()?;
                let first_person = self.i32()?;
                let tile_puzzle_jump = self.i32()?;
                let in_weapon_do_mode = self.i32()?;
                let attack_fire_frame = self.i32()?;
                let bow_attacking = self.i32()?;
                let b_walk_to_loc = self.i32()?;
                let b_can_change_fp_mode = self.i32()?;
                let walk_to_location = self.point3f()?;
                let point_cue_flag = self.i32()?;
                let point_cue_i1 = self.i32()?;
                let point_cue_i2 = self.i32()?;
                let walk_cue_flag = self.i32()?;
                let run_cue_flag = self.i32()?;
                let walk_cue_i1 = self.i32()?;
                let walk_cue_i2 = self.i32()?;
                let max_damage_factor = self.f32()?;
                let min_damage_factor = self.f32()?;
                let run_to_walk = self.i32()?;
                let terrain_sound = self.i32()?;
                let rope_status = self.i32()?;
                let rope_loc = self.point3f()?;
                let rope_dir = self.point3f()?;
                let rope_cam_loc1 = self.point3f()?;
                let rope_cam_loc2 = self.point3f()?;
                let rope_cam2_loc = self.point3f()?;
                let rope_cam_ang = self.f32()?;
                let rope_cam_range = self.f32()?;
                let rope_cam_top_limit = self.f32()?;
                let rope_cam_bottom_limit = self.f32()?;
                let box_up_time = self.f32()?;
                let wait_exit_time = self.f32()?;
                let last_played_scroll = self.f32()?;
                let locked_state_id = self.i32()?;
                let locked_state_data = self.opt_string()?;
                let was_in_first = self.i32()?;
                let was_in_third = self.i32()?;
                let start_help = self.i32()?;
                let world_flags = self.i32()?;
                let was_in_bow_mode = self.i32()?;
                let point_cue_obj = self.opt_string()?;
                let walk_cue_obj = self.opt_string()?;
                let allow_leaning = self.i32()?;
                let level = self.i32()?;
                let item_in_hand = self.opt_string()?;
                let clarity_spell = self.i32()?;
                let strength_spell = self.i32()?;
                let invulnerability_spell = self.i32()?;
                let invisibility_spell = self.i32()?;
                let strength_spell_points = self.i32()?;
                let armor_before_spell = self.i32()?;
                let alt_short = self.i32()?;
                let alt_long = self.i32()?;
                let alt_armor = self.i32()?;
                let talk_mon_name = self.opt_string()?;
                let talk_enemy = self.i32()?;
                let talk_done = self.i32()?;
                let talk_mon_loc = self.point3f()?;
                let anim_list = self.nested_named()?;
                let short_anim_list = self.nested_named()?;
                let long_anim_list = self.nested_named()?;
                let fp_anim_list = self.nested_named()?;
                let short_fp_anim_list = self.nested_named()?;
                let long_fp_anim_list = self.nested_named()?;
                let skin_list = self.nested_named()?;
                let skin_list_firstperson = self.nested_named()?;
                let lips_list = self.nested_named()?;
                let cur_skin = self.i32()?;
                let old_skin = self.i32()?;
                let cur_lips = self.i32()?;
                let old_lips = self.i32()?;
                let hold_help = self.string_vec("holdHelp")?;
                PersObject::KqConner(Box::new(KqConner {
                    monster,
                    zoom_uly,
                    zoom_ulx,
                    zoom_lry,
                    zoom_lrx,
                    allow_shadows_in_game,
                    next_do_mode,
                    required_weapon_mode,
                    required_weapon_script,
                    hands_off,
                    gui_bottom,
                    talk_gui_bottom,
                    hands_off_cursor,
                    anim_set_hands_off,
                    solved_tile3,
                    connor_exited,
                    first_person,
                    tile_puzzle_jump,
                    in_weapon_do_mode,
                    attack_fire_frame,
                    bow_attacking,
                    b_walk_to_loc,
                    b_can_change_fp_mode,
                    walk_to_location,
                    point_cue_flag,
                    point_cue_i1,
                    point_cue_i2,
                    walk_cue_flag,
                    run_cue_flag,
                    walk_cue_i1,
                    walk_cue_i2,
                    max_damage_factor,
                    min_damage_factor,
                    run_to_walk,
                    terrain_sound,
                    rope_status,
                    rope_loc,
                    rope_dir,
                    rope_cam_loc1,
                    rope_cam_loc2,
                    rope_cam2_loc,
                    rope_cam_ang,
                    rope_cam_range,
                    rope_cam_top_limit,
                    rope_cam_bottom_limit,
                    box_up_time,
                    wait_exit_time,
                    last_played_scroll,
                    locked_state_id,
                    locked_state_data,
                    was_in_first,
                    was_in_third,
                    start_help,
                    world_flags,
                    was_in_bow_mode,
                    point_cue_obj,
                    walk_cue_obj,
                    allow_leaning,
                    level,
                    item_in_hand,
                    clarity_spell,
                    strength_spell,
                    invulnerability_spell,
                    invisibility_spell,
                    strength_spell_points,
                    armor_before_spell,
                    alt_short,
                    alt_long,
                    alt_armor,
                    talk_mon_name,
                    talk_enemy,
                    talk_done,
                    talk_mon_loc,
                    anim_list,
                    short_anim_list,
                    long_anim_list,
                    fp_anim_list,
                    short_fp_anim_list,
                    long_fp_anim_list,
                    skin_list,
                    skin_list_firstperson,
                    lips_list,
                    cur_skin,
                    old_skin,
                    cur_lips,
                    old_lips,
                    hold_help,
                }))
            }
            "KQStateWait" | "KQStateConnerWait" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::Wait {
                    class: class.clone(),
                    con_radius: self.f32()?,
                    mon_radius: self.f32()?,
                },
            }),
            "KQStateMove" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::Move {
                    con_radius: self.f32()?,
                    mon_radius: self.f32()?,
                    started_moving: self.i32()?,
                },
            }),
            "KQStateUnborn" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::Unborn {
                    has_pre_birth: self.i32()?,
                    ready_to_birth: self.i32()?,
                },
            }),
            "KQStateConnerPortal" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::ConnerPortal {
                    entered_portal: self.i32()?,
                    done_portal: self.i32()?,
                    processed_gui: self.i32()?,
                    portal_name: self.opt_string()?,
                    trap_name: self.opt_string()?,
                    portal_enter_anim: self.opt_string()?,
                    portal_exit_anim: self.opt_string()?,
                    portal_enter_anm_file: self.opt_string()?,
                    portal_exit_anm_file: self.opt_string()?,
                    portal_exit_script: self.opt_string()?,
                },
            }),
            "KQStateConnerHookRope" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::ConnerHookRope {
                    done_roping: self.i32()?,
                    reached_top: self.i32()?,
                    do_mode_before: self.i32()?,
                    old_cam_pos: self.point3f()?,
                    con_pos: self.point3f()?,
                    on_rope: self.i32()?,
                    hide_icon: self.i32()?,
                    first_to_third: self.i32()?,
                    bottom_cam_limit: self.f32()?,
                    top_cam_limit: self.f32()?,
                    con_cam_bottom_limit: self.f32()?,
                    con_cam_top_limit: self.f32()?,
                    rope_cam_loc1: self.point3f()?,
                    rope_cam_loc2: self.point3f()?,
                    orig_step_up: self.f32()?,
                    orig_step_down: self.f32()?,
                },
            }),
            "KQStateConnerSpecial" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::ConnerSpecial {
                    anim_command_string: self.opt_string()?,
                    cur_anim_list: self.opt_string()?,
                    anim_list: self.opt_string()?,
                },
            }),
            "KQStateConnerPush" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::ConnerPush {
                    pushed: self.i32()?,
                },
            }),
            "KQStateConnerFall" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::ConnerFall {
                    done_falling: self.i32()?,
                },
            }),
            "KQStateConnerJump" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::ConnerJump {
                    done_jumping: self.i32()?,
                },
            }),
            "KQState" | "KQStateDie" => PersObject::KqState(KqState {
                base: self.read_base()?,
                from_state: self.i32()?,
                kind: KqStateKind::Empty {
                    class: class.clone(),
                },
            }),
            other if other.starts_with("KQState") => {
                let base = self.read_base()?;
                let from_state = self.i32()?;
                let left = payload_end.saturating_sub(self.pos);
                let remaining = self.take(left)?.to_vec();
                PersObject::KqState(KqState {
                    base,
                    from_state,
                    kind: KqStateKind::Unknown {
                        class: class.clone(),
                        remaining,
                    },
                })
            }
            _ => return Err(self.err(format!("unsupported PERS class '{class}'"))),
        };

        if self.pos != payload_end {
            return Err(self.err(format!(
                "{class}: parsed {} of {payload_size} payload bytes ({} leftover)",
                self.pos - payload_start,
                payload_end as isize - self.pos as isize
            )));
        }
        if payload_size % 2 == 1 && self.pos < self.bytes.len() {
            self.pos += 1;
        }
        self.dict[dict_slot] = Some(obj.clone());
        Ok(obj)
    }
}

/// Parses a top-level binary `PERS` (or `DICT`) chunk, enforcing zero unread trailing bytes.
pub fn parse_pers(bytes: &[u8]) -> Result<PersObject, String> {
    let mut r = Reader::new(bytes);
    let obj = r.read_block()?;
    if r.pos != bytes.len() {
        return Err(format!(
            "{}: {} trailing bytes after top-level PERS block (at 0x{:X} of 0x{:X})",
            obj.class_name(),
            bytes.len() - r.pos,
            r.pos,
            bytes.len()
        ));
    }
    Ok(obj)
}

/// Decodes a `KQMap` fog-of-war RLE run list (`rle_runs`, without trailing `-1`)
/// into `460 * 460` booleans (`true` = revealed).
pub fn decode_map_fog(runs: &[i32]) -> Vec<bool> {
    let as_i64: Vec<i64> = runs.iter().map(|&r| r as i64).collect();
    crate::magic_map::fog_from_runs(&as_i64).0
}

/// Encodes `460 * 460` fog booleans into `KQMap` RLE run lengths (without trailing `-1`).
pub fn encode_map_fog(fog: &[bool]) -> Vec<i32> {
    crate::magic_map::runs_from_fog(fog)
        .into_iter()
        .map(|r| r as i32)
        .collect()
}

/// Extracts `worldFlags` map visited / teleport-earned bits (`0..MAP_COUNT`) if
/// packed as bitfields, or leaves callers to inspect `KqConner::world_flags`.
pub const fn map_flag_count() -> usize {
    MAP_COUNT
}

struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { buf: Vec::new() }
    }

    fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn point3f(&mut self, p: [f32; 3]) {
        self.f32(p[0]);
        self.f32(p[1]);
        self.f32(p[2]);
    }

    fn tmat3f(&mut self, m: &TMat3F) {
        self.u32(m.flags);
        for row in &m.rotation {
            for &v in row {
                self.f32(v);
            }
        }
        self.point3f(m.position);
    }

    fn opt_string(&mut self, s: &Option<String>) {
        match s {
            None => self.u32(0),
            Some(text) => {
                let raw = text.as_bytes();
                let with_nul = raw.len() + 1;
                let padded = (with_nul + 1) & !1;
                self.u32(padded as u32);
                self.buf.extend_from_slice(raw);
                self.buf.push(0);
                if padded > with_nul {
                    self.buf.push(0);
                }
            }
        }
    }

    fn string_vec(&mut self, v: &[Option<String>]) {
        self.i32(v.len() as i32);
        for s in v {
            self.opt_string(s);
        }
    }

    fn point_vec(&mut self, v: &[[f32; 3]]) {
        self.i32(v.len() as i32);
        for &p in v {
            self.point3f(p);
        }
    }

    fn nested_named(&mut self, n: &NestedNamed) {
        match n {
            None => self.u32(0),
            Some((name, child)) => {
                self.opt_string(&Some(name.clone()));
                self.write_block(child);
            }
        }
    }

    fn write_base(&mut self, b: &KqBase) {
        self.opt_string(&b.name);
    }

    fn write_visible(&mut self, v: &KqVisibleBase) {
        self.write_base(&v.base);
        self.i32(v.active_3d_flag);
        self.i32(v.is_far);
        self.f32(v.too_far);
        self.i32(v.translucency);
        self.f32(v.haze_multiplier);
        self.i32(v.b_haze_solid);
        self.i32(v.hidden);
    }

    fn write_mover_inline(&mut self, m: &KqMoverInline) {
        self.tmat3f(&m.transform);
        self.i32(m.keep_on_ground);
        self.i32(m.use_gravity);
        self.i32(m.submerged);
        self.f32(m.max_slope_dot);
    }

    fn write_mover(&mut self, m: &KqMover) {
        self.write_visible(&m.visible);
        self.write_mover_inline(&m.inline);
    }

    fn write_anim_mover(&mut self, m: &KqAnimMover) {
        self.write_mover(&m.mover);
        self.f32(m.rotate_speed);
        self.i32(m.fast);
        self.point3f(m.look_at);
        self.i32(m.use_look_at);
    }

    fn write_object(&mut self, o: &KqObject) {
        self.write_visible(&o.visible);
        self.opt_string(&o.shape_name);
        self.opt_string(&o.accepts_inventory);
        self.u32(o.collider_type);
        self.u32(o.collider_mask);
        self.u32(o.collision_shape);
        self.u32(o.collision_detail);
        self.u32(o.collision_priority);
        self.f32(o.step_up_amount);
        self.f32(o.step_down_amount);
        self.u32(o.groups_always);
        self.u32(o.groups_active_3d);
        self.u32(o.groups_active_3d_far);
        self.u32(o.groups_invisible);
        self.u32(o.immunities);
        self.i32(o.accept_all_items);
        self.opt_string(&o.alternate_bmp);
        self.opt_string(&o.original_bmp);
        self.i32(o.dithered);
        self.opt_string(&o.ouch_sound);
        self.opt_string(&o.script);
        self.i32(o.shadow_visible);
        self.i32(o.con_collision_action);
        self.i32(o.world_collision_action);
        self.i32(o.too_far_away_action);
        self.u32(o.shadow_mask);
        self.i32(o.do_not_point_at);
        self.i32(o.locked_resources);
        self.i32(o.footsound_type);
        self.f32(o.ouch_time);
        self.opt_string(&o.projectile_file);
        self.point3f(o.shoot_offset);
        self.point3f(o.rel_aim_point);
        self.i32(o.obj_proj_damage);
        self.i32(o.last_weapon_type);
        self.u32(o.env_type);
        self.opt_string(&o.stats);
        match &o.mover {
            MoverData::Inline { mover_name, inline } => {
                self.i32(0);
                self.opt_string(mover_name);
                self.write_mover_inline(inline);
            }
            MoverData::Nested(child) => {
                let flag = if o.mover_type == 0 { 1 } else { o.mover_type };
                self.i32(flag);
                self.nested_named(child);
            }
        }
    }

    fn write_anim_object(&mut self, a: &KqAnimObject) {
        self.write_object(&a.object);
        self.f32(a.anim_pos);
        self.opt_string(&a.curr_anim_name);
        self.opt_string(&a.next_anim_name);
        self.opt_string(&a.pre_trans_anim_name);
        self.opt_string(&a.post_trans_anim_name);
        self.f32(a.next_anim_start_time);
        self.f32(a.post_trans_anim_start_time);
        self.u32(a.flags);
        self.i32(a.flying_object);
        self.i32(a.turn_off_dyn_lighting);
        self.nested_named(&a.anim_loops);
        self.i32(a.owns_anim_list);
        self.opt_string(&a.notify_object);
    }

    fn write_trap(&mut self, t: &KqTrap) {
        self.write_visible(&t.visible);
        self.point3f(t.loc);
        self.u32(t.trap_type);
        self.f32(t.inner_percent);
        self.opt_string(&t.script);
        self.u32(t.collider_mask);
        self.point3f(t.bb_min);
        self.point3f(t.bb_max);
        self.i32(t.lookup_call);
        self.i32(t.shrunk);
        self.string_vec(&t.trapped_object);
    }

    fn write_monster(&mut self, m: &KqMonster) {
        self.write_anim_object(&m.anim_object);
        self.nested_named(&m.state);
        self.i32(m.next_state);
        self.opt_string(&m.next_state_data);
        self.opt_string(&m.state_change_notify_group_name);
        self.string_vec(&m.state_change_members);
        self.string_vec(&m.dependents);
        self.i32(m.guarding_dependent);
        self.i32(m.mon_tick_count);
        self.i32(m.ticks_since_seen_connor);
        self.opt_string(&m.monster_type);
        self.f32(m.alarm_radius);
        self.i32(m.returning_to_home_radius);
        self.i32(m.b_must_go_home);
        self.i32(m.keeping_distance);
        self.f32(m.home_radius);
        self.f32(m.chase_radius);
        self.f32(m.attack_dist);
        self.f32(m.attack_too_close);
        self.f32(m.attack_far_enough);
        self.f32(m.likely_to_flee);
        self.i32(m.must_flee);
        self.f32(m.attack_delay);
        self.f32(m.birth_delay);
        self.f32(m.alarm_time);
        self.f32(m.attack_time);
        self.f32(m.chase_time);
        self.f32(m.react_time);
        self.i32(m.hits_since_alarmed);
        self.i32(m.do_mode);
        self.point3f(m.do_loc);
        self.i32(m.looked_up_projectile);
        self.f32(m.conner_dist);
        self.f32(m.conner_time);
        self.point3f(m.last_conner_pos);
        self.i32(m.can_see_conner);
        self.i32(m.can_always_see_conner);
        self.f32(m.line_of_sight_offset);
        self.point3f(m.con_vec);
        self.i32(m.in_good_bye);
        self.i32(m.no_turn);
        self.point3f(m.start_loc);
        self.f32(m.min_wander_dist);
        self.opt_string(&m.dead_dts);
        self.opt_string(&m.ided_kqs);
        self.opt_string(&m.projd_kqs);
        self.opt_string(&m.dead_bmp);
        self.f32(m.normal_react_percent);
        self.opt_string(&m.special_react_anim);
        self.f32(m.special_react_percent);
        self.opt_string(&m.special_connor_react_anim);
        self.f32(m.idle_time);
        self.i32(m.idling);
        self.i32(m.active);
        self.opt_string(&m.move_anim);
        self.opt_string(&m.last_anim_played);
        self.opt_string(&m.anim_file);
        self.opt_string(&m.idle_anim_file);
        self.opt_string(&m.saved_anim_list);
        self.opt_string(&m.inv_drop);
        self.opt_string(&m.dropped_weapon);
        self.i32(m.self_member);
        self.i32(m.mon_use_gravity);
        for &v in &m.gb {
            self.i32(v);
        }
        for &v in &m.wb {
            self.i32(v);
        }
        for &v in &m.alarm {
            self.i32(v);
        }
        for &v in &m.surprise {
            self.i32(v);
        }
        self.i32(m.said_alarm_message);
        self.opt_string(&m.chase_object);
        self.opt_string(&m.team);
        self.point_vec(&m.wander_pts);
        self.i32(m.wanders);
        self.i32(m.wander_and_stop);
        self.i32(m.cur_wander_pt);
        self.string_vec(&m.shoot_offsets);
        self.nested_named(&m.inventory);
        self.opt_string(&m.base_lip);
        self.nested_named(&m.lips);
        self.nested_named(&m.conversation);
        self.nested_named(&m.talker);
    }

    fn write_block(&mut self, obj: &PersObject) {
        let class = obj.class_name();
        let class_bytes = class.as_bytes();
        self.buf.extend_from_slice(b"PERS");
        let size_pos = self.buf.len();
        self.u32(0);
        let payload_start = self.buf.len();

        let name_len = class_bytes.len() as u16;
        self.buf.extend_from_slice(&name_len.to_le_bytes());
        self.buf.extend_from_slice(class_bytes);
        if class_bytes.len() % 2 == 1 {
            self.buf.push(0);
        }

        match obj {
            PersObject::KqBase(b) => self.write_base(b),
            PersObject::KqVisibleBase(v) => self.write_visible(v),
            PersObject::KqMover(m) => self.write_mover(m),
            PersObject::KqAnimMover { data, .. } => self.write_anim_mover(data),
            PersObject::KqGameState(g) => {
                self.write_base(&g.base);
                self.i32(g.next_no_name);
                self.f32(g.difficulty_factor);
            }
            PersObject::KqWorldState(w) => {
                self.write_base(&w.base);
                self.string_vec(&w.events);
            }
            PersObject::KqMusic(m) => {
                self.write_visible(&m.visible);
                self.string_vec(&m.music);
            }
            PersObject::KqCamMover(c) => {
                self.write_visible(&c.visible);
                self.opt_string(&c.follow_object_name);
                self.i32(c.pause_follow);
                self.f32(c.near_plane);
                self.f32(c.far_plane);
                self.f32(c.fov);
                self.f32(c.tilt_flag);
                self.f32(c.zoom_flag);
                self.f32(c.pan_flag);
                self.i32(c.fix_height);
                self.f32(c.old_pan_flag);
                self.f32(c.old_zoom_flag);
                self.f32(c.old_ang_diff_z);
                self.f32(c.cam_move_radius);
                self.f32(c.short_cam_move_radius);
                self.f32(c.long_cam_move_radius);
                self.f32(c.cam_rotate_amount);
                self.f32(c.conner_min_dist);
                self.f32(c.cam_fly_dist);
                self.f32(c.ang_total);
                self.f32(c.ang_step);
                self.f32(c.roll_step);
                self.f32(c.cam_con_len);
                self.f32(c.shake_duration);
                self.f32(c.x_shake_max);
                self.f32(c.z_shake_max);
                self.f32(c.x_shake_amp);
                self.f32(c.z_shake_amp);
                self.f32(c.old_cmrx);
                self.f32(c.con_head_height);
                self.f32(c.cam_height1);
                self.f32(c.cam_height2);
                self.f32(c.x_shake_period);
                self.f32(c.z_shake_period);
                self.point3f(c.loc);
                self.point3f(c.old_cam_axis);
                self.point3f(c.camera_offset);
                self.point3f(c.third_camera_offset);
                self.point3f(c.move_step);
                self.point3f(c.camera_rotation);
                self.point3f(c.camera_rotation_default);
                self.point3f(c.dir);
                self.point3f(c.third_camera_rotation);
                self.tmat3f(&c.connor_mat);
                self.tmat3f(&c.old_connor_mat);
                self.i32(c.cam_rot_mode_x);
                self.i32(c.cam_rot_mode_z);
                self.i32(c.con_rel_move);
                self.i32(c.old_con_rel_move);
                self.i32(c.check_move);
                self.i32(c.collision_radius);
                self.i32(c.old_cam_rot_mode_z);
                self.i32(c.move_cam);
                self.i32(c.aim_cam);
                self.i32(c.in_pan);
                self.i32(c.first_pan);
                self.i32(c.first_zoom);
                self.i32(c.touched_terrain);
                self.i32(c.is_first_person);
                self.i32(c.constop);
                self.i32(c.n_move_step);
                self.i32(c.zip_cam);
                self.i32(c.ang_rot);
                self.i32(c.action_cam);
                self.i32(c.shake);
                self.i32(c.shake_frame);
                self.i32(c.user_height);
            }
            PersObject::KqMap(m) => {
                self.write_visible(&m.visible);
                self.i32(m.north);
                self.i32(m.west);
                self.i32(m.south);
                self.i32(m.east);
                self.i32(m.use_fog);
                self.i32(m.fog_radius);
                self.i32(m.hold_map_loc);
                self.i32(m.is_active);
                self.opt_string(&m.blank_map);
                self.opt_string(&m.full_map);
                self.opt_string(&m.arrow_pba);
                self.opt_string(&m.map_name);
                self.i32(m.earned_teleport);
                self.opt_string(&m.teleport_script);
                self.opt_string(&m.teleport_portal_kq);
                self.i32(m.icons.len() as i32);
                for ic in &m.icons {
                    self.opt_string(&ic.map_icon);
                    self.opt_string(&ic.src_icon);
                    self.i32(ic.x_icon);
                    self.i32(ic.y_icon);
                }
                for &r in &m.rle_runs {
                    self.i32(r);
                }
                self.i32(-1);
            }
            PersObject::KqTerrain(t) => {
                self.write_base(&t.base);
                self.opt_string(&t.material_list_file);
                self.opt_string(&t.material_bmp);
                self.opt_string(&t.height_bmp);
                self.opt_string(&t.height_bmp2);
                self.opt_string(&t.height_bmp3);
                self.opt_string(&t.height_bmp4);
                self.i32(t.height_bmp_scale);
                self.opt_string(&t.material_direction);
                self.opt_string(&t.floor_bmp_name);
                self.f32(t.visible_distance);
                self.f32(t.perspective_distance);
                self.f32(t.haze_distance);
                self.i32(t.ground_scale);
                self.i32(t.smoothing);
                for &v in &t.detail_table {
                    self.f32(v);
                }
                self.i32(t.block_size);
                self.i32(t.light_scale);
                self.opt_string(&t.envinfo_name);
                self.opt_string(&t.tedfile_name);
                self.opt_string(&t.lightmap_name);
                self.opt_string(&t.shadowmap_name);
                self.opt_string(&t.sw_lightmap_name);
                self.opt_string(&t.sw_shadowmap_name);
                self.i32(t.deactivated_lights.len() as i32);
                for l in &t.deactivated_lights {
                    self.i32(l.f_type);
                    self.f32(l.f_red);
                    self.f32(l.f_green);
                    self.f32(l.f_blue);
                    self.point3f(l.f_w_position);
                    self.f32(l.f_range);
                    self.f32(l.f_falloff);
                }
                self.i32(t.changed_heights.len() as i32);
                for ch in &t.changed_heights {
                    self.i32(ch.h_startx);
                    self.i32(ch.h_starty);
                    self.i32(ch.h_endx);
                    self.i32(ch.h_endy);
                }
            }
            PersObject::KqInventory { data, .. } => {
                self.write_base(&data.base);
                self.u32(data.owner_id);
                self.i32(data.items.len() as i32);
                for item in &data.items {
                    self.nested_named(item);
                }
            }
            PersObject::KqObject(o) => self.write_object(o),
            PersObject::KqInventoryItem(i) => {
                self.write_object(&i.object);
                self.i32(i.quantity);
                self.u32(i.user_id);
                self.f32(i.time_left);
                self.i32(i.item_type);
            }
            PersObject::KqInterior(i) => {
                self.write_object(&i.object);
                self.opt_string(&i.material);
                self.opt_string(&i.sw_lightmap_name);
                self.opt_string(&i.alt_sw_lightmap_name);
                self.opt_string(&i.alt_hw_lightmap_name);
                self.i32(i.using_alt_lighting);
                self.nested_named(&i.trap_collider);
            }
            PersObject::KqAudibleObject(a) => {
                self.write_object(&a.object);
                self.opt_string(&a.sound_file_name);
                self.i32(a.is_sound_3d);
                self.f32(a.min_distance);
                self.f32(a.max_distance);
                self.point3f(a.delta);
            }
            PersObject::KqSwampTile(s) => {
                self.write_object(&s.object);
                self.i32(s.height_num);
                self.f32(s.dam_factor);
            }
            PersObject::KqRiverControl(r) => {
                self.write_object(&r.object);
                self.point3f(r.bb_min);
                self.point3f(r.bb_max);
                self.f32(r.stretch_dist);
                self.i32(r.dir);
                self.i32(r.rusted_one);
            }
            PersObject::KqAnimLoopList(l) => {
                self.write_base(&l.base);
                self.i32(l.movements.len() as i32);
                for mv in &l.movements {
                    self.opt_string(&mv.movement_name);
                    self.opt_string(&mv.loop_spec);
                    self.string_vec(&mv.cues);
                    self.string_vec(&mv.transitions);
                }
            }
            PersObject::KqAnimObject(a) => self.write_anim_object(a),
            PersObject::KqAnimOnce(a) => {
                self.write_anim_object(&a.anim_object);
                self.i32(a.delete_when_done);
            }
            PersObject::KqDoor(d) => {
                self.write_anim_object(&d.anim_object);
                self.i32(d.state);
            }
            PersObject::KqChest(c) => {
                self.write_anim_object(&c.anim_object);
                self.opt_string(&c.inv_item_type_name);
                self.i32(c.inv_quantity);
                self.point3f(c.inv_loc);
                self.point3f(c.inv_dir);
                self.i32(c.chest_state);
                self.opt_string(&c.inv_item_name);
                for &v in &c.take_msg {
                    self.i32(v);
                }
                for &v in &c.empty_msg {
                    self.i32(v);
                }
            }
            PersObject::KqPortal(p) => {
                self.write_anim_object(&p.anim_object);
                self.opt_string(&p.world);
                self.opt_string(&p.enter_anim);
                self.opt_string(&p.exit_anim);
                self.opt_string(&p.enter_anm_file);
                self.opt_string(&p.exit_anm_file);
                self.opt_string(&p.exit_script);
                self.tmat3f(&p.dest_connor);
                self.tmat3f(&p.dest_cam);
                self.point3f(p.no_loc);
            }
            PersObject::KqBoulder(b) => {
                self.write_anim_object(&b.anim_object);
                self.point3f(b.prev_pos);
            }
            PersObject::KqLavaTile(l) => {
                self.write_anim_object(&l.anim_object);
                self.i32(l.hold_flag);
                self.opt_string(&l.top_bmp);
                self.i32(l.reversed);
                self.i32(l.metal);
                self.i32(l.unknown);
                self.i32(l.activated);
            }
            PersObject::KqTrap(t) => self.write_trap(t),
            PersObject::KqSpiriTrap(t) => {
                self.write_trap(&t.trap);
                self.i32(t.spirit_state);
                self.i32(t.trap_status);
                self.f32(t.damage);
            }
            PersObject::KqHazer(h) => {
                self.write_visible(&h.visible);
                self.point3f(h.loc);
                self.f32(h.inner_radius);
                self.u32(h.collider_mask);
                self.f32(h.radius);
                self.i32(h.collided);
                self.f32(h.haze_center_factor);
                self.f32(h.orig_haze_edge);
                self.f32(h.haze_edge);
            }
            PersObject::KqPointLight(p) => {
                self.write_visible(&p.visible);
                self.i32(p.ts_light_type);
                self.point3f(p.position);
                self.point3f(p.intensity);
                self.f32(p.attenuation[0]);
                self.f32(p.attenuation[1]);
            }
            PersObject::KqTerrainAnimator(t) => {
                self.write_visible(&t.visible);
                self.i32(t.start_now);
                self.i32(t.repeat_flag);
                self.i32(t.frame_count);
                self.f32(t.delay);
                self.i32(t.material_index.len() as i32);
                for &v in &t.material_index {
                    self.i32(v);
                }
                self.i32(t.material_index_ii.len() as i32);
                for &v in &t.material_index_ii {
                    self.i32(v);
                }
                self.i32(t.grid_blocks.len() as i32);
                for &[x, y] in &t.grid_blocks {
                    self.i32(x);
                    self.i32(y);
                }
                if t.grid_blocks.is_empty() {
                    self.i32(t.orig_textures.len() as i32);
                    for &v in &t.orig_textures {
                        self.i32(v);
                    }
                    self.i32(t.exclude.len() as i32);
                    for &[x, y] in &t.exclude {
                        self.i32(x);
                        self.i32(y);
                    }
                }
            }
            PersObject::KqLips(l) => {
                self.write_base(&l.base);
                for s in &l.lips {
                    self.opt_string(s);
                }
            }
            PersObject::KqConversation(c) => {
                self.write_visible(&c.visible);
                self.point3f(c.conv_loc);
                self.string_vec(&c.list1);
                self.string_vec(&c.list2);
            }
            PersObject::KqTalk(t) => {
                self.write_visible(&t.visible);
                self.i32(t.flags);
                self.i32(t.nostop);
                self.i32(t.no_fade);
                self.opt_string(&t.talker_name);
                self.string_vec(&t.talk_list);
            }
            PersObject::KqMonster { data, .. } => self.write_monster(data),
            PersObject::KqTowerSkeleton(t) => {
                self.write_monster(&t.monster);
                self.f32(t.real_alarm);
                self.f32(t.move_time);
                self.f32(t.time_to_run);
                self.f32(t.stun_pause);
                self.f32(t.react_pause);
                self.i32(t.b_greeted_connor);
                self.point3f(t.old_cam_pos);
                self.i32(t.b_reacting_back);
                self.i32(t.b_screamed);
            }
            PersObject::KqMandragorRoot(r) => {
                self.write_monster(&r.monster);
                self.point3f(r.offset);
                self.i32(r.orig_health);
                self.f32(r.orig_birth_delay);
                self.f32(r.rebirth_delay);
                self.f32(r.repel_dist);
                self.f32(r.too_close);
            }
            PersObject::KqMandragorTree(t) => {
                self.write_monster(&t.monster);
                self.i32(t.root_down_count);
                self.i32(t.max_roots);
            }
            PersObject::KqConAnimList(l) => {
                self.write_base(&l.base);
                self.string_vec(&l.entries);
            }
            PersObject::KqSkinList(l) => {
                self.write_base(&l.base);
                self.string_vec(&l.skins);
                self.string_vec(&l.inv_items);
            }
            PersObject::KqConner(c) => {
                self.write_monster(&c.monster);
                self.f32(c.zoom_uly);
                self.f32(c.zoom_ulx);
                self.f32(c.zoom_lry);
                self.f32(c.zoom_lrx);
                self.u32(c.allow_shadows_in_game);
                self.i32(c.next_do_mode);
                self.i32(c.required_weapon_mode);
                self.opt_string(&c.required_weapon_script);
                self.i32(c.hands_off);
                self.i32(c.gui_bottom);
                self.i32(c.talk_gui_bottom);
                self.i32(c.hands_off_cursor);
                self.i32(c.anim_set_hands_off);
                self.i32(c.solved_tile3);
                self.i32(c.connor_exited);
                self.i32(c.first_person);
                self.i32(c.tile_puzzle_jump);
                self.i32(c.in_weapon_do_mode);
                self.i32(c.attack_fire_frame);
                self.i32(c.bow_attacking);
                self.i32(c.b_walk_to_loc);
                self.i32(c.b_can_change_fp_mode);
                self.point3f(c.walk_to_location);
                self.i32(c.point_cue_flag);
                self.i32(c.point_cue_i1);
                self.i32(c.point_cue_i2);
                self.i32(c.walk_cue_flag);
                self.i32(c.run_cue_flag);
                self.i32(c.walk_cue_i1);
                self.i32(c.walk_cue_i2);
                self.f32(c.max_damage_factor);
                self.f32(c.min_damage_factor);
                self.i32(c.run_to_walk);
                self.i32(c.terrain_sound);
                self.i32(c.rope_status);
                self.point3f(c.rope_loc);
                self.point3f(c.rope_dir);
                self.point3f(c.rope_cam_loc1);
                self.point3f(c.rope_cam_loc2);
                self.point3f(c.rope_cam2_loc);
                self.f32(c.rope_cam_ang);
                self.f32(c.rope_cam_range);
                self.f32(c.rope_cam_top_limit);
                self.f32(c.rope_cam_bottom_limit);
                self.f32(c.box_up_time);
                self.f32(c.wait_exit_time);
                self.f32(c.last_played_scroll);
                self.i32(c.locked_state_id);
                self.opt_string(&c.locked_state_data);
                self.i32(c.was_in_first);
                self.i32(c.was_in_third);
                self.i32(c.start_help);
                self.i32(c.world_flags);
                self.i32(c.was_in_bow_mode);
                self.opt_string(&c.point_cue_obj);
                self.opt_string(&c.walk_cue_obj);
                self.i32(c.allow_leaning);
                self.i32(c.level);
                self.opt_string(&c.item_in_hand);
                self.i32(c.clarity_spell);
                self.i32(c.strength_spell);
                self.i32(c.invulnerability_spell);
                self.i32(c.invisibility_spell);
                self.i32(c.strength_spell_points);
                self.i32(c.armor_before_spell);
                self.i32(c.alt_short);
                self.i32(c.alt_long);
                self.i32(c.alt_armor);
                self.opt_string(&c.talk_mon_name);
                self.i32(c.talk_enemy);
                self.i32(c.talk_done);
                self.point3f(c.talk_mon_loc);
                self.nested_named(&c.anim_list);
                self.nested_named(&c.short_anim_list);
                self.nested_named(&c.long_anim_list);
                self.nested_named(&c.fp_anim_list);
                self.nested_named(&c.short_fp_anim_list);
                self.nested_named(&c.long_fp_anim_list);
                self.nested_named(&c.skin_list);
                self.nested_named(&c.skin_list_firstperson);
                self.nested_named(&c.lips_list);
                self.i32(c.cur_skin);
                self.i32(c.old_skin);
                self.i32(c.cur_lips);
                self.i32(c.old_lips);
                self.string_vec(&c.hold_help);
            }
            PersObject::KqState(s) => {
                self.write_base(&s.base);
                self.i32(s.from_state);
                match &s.kind {
                    KqStateKind::Wait {
                        con_radius,
                        mon_radius,
                        ..
                    } => {
                        self.f32(*con_radius);
                        self.f32(*mon_radius);
                    }
                    KqStateKind::Move {
                        con_radius,
                        mon_radius,
                        started_moving,
                    } => {
                        self.f32(*con_radius);
                        self.f32(*mon_radius);
                        self.i32(*started_moving);
                    }
                    KqStateKind::Unborn {
                        has_pre_birth,
                        ready_to_birth,
                    } => {
                        self.i32(*has_pre_birth);
                        self.i32(*ready_to_birth);
                    }
                    KqStateKind::ConnerPortal {
                        entered_portal,
                        done_portal,
                        processed_gui,
                        portal_name,
                        trap_name,
                        portal_enter_anim,
                        portal_exit_anim,
                        portal_enter_anm_file,
                        portal_exit_anm_file,
                        portal_exit_script,
                    } => {
                        self.i32(*entered_portal);
                        self.i32(*done_portal);
                        self.i32(*processed_gui);
                        self.opt_string(portal_name);
                        self.opt_string(trap_name);
                        self.opt_string(portal_enter_anim);
                        self.opt_string(portal_exit_anim);
                        self.opt_string(portal_enter_anm_file);
                        self.opt_string(portal_exit_anm_file);
                        self.opt_string(portal_exit_script);
                    }
                    KqStateKind::ConnerHookRope {
                        done_roping,
                        reached_top,
                        do_mode_before,
                        old_cam_pos,
                        con_pos,
                        on_rope,
                        hide_icon,
                        first_to_third,
                        bottom_cam_limit,
                        top_cam_limit,
                        con_cam_bottom_limit,
                        con_cam_top_limit,
                        rope_cam_loc1,
                        rope_cam_loc2,
                        orig_step_up,
                        orig_step_down,
                    } => {
                        self.i32(*done_roping);
                        self.i32(*reached_top);
                        self.i32(*do_mode_before);
                        self.point3f(*old_cam_pos);
                        self.point3f(*con_pos);
                        self.i32(*on_rope);
                        self.i32(*hide_icon);
                        self.i32(*first_to_third);
                        self.f32(*bottom_cam_limit);
                        self.f32(*top_cam_limit);
                        self.f32(*con_cam_bottom_limit);
                        self.f32(*con_cam_top_limit);
                        self.point3f(*rope_cam_loc1);
                        self.point3f(*rope_cam_loc2);
                        self.f32(*orig_step_up);
                        self.f32(*orig_step_down);
                    }
                    KqStateKind::ConnerSpecial {
                        anim_command_string,
                        cur_anim_list,
                        anim_list,
                    } => {
                        self.opt_string(anim_command_string);
                        self.opt_string(cur_anim_list);
                        self.opt_string(anim_list);
                    }
                    KqStateKind::ConnerPush { pushed } => self.i32(*pushed),
                    KqStateKind::ConnerFall { done_falling } => self.i32(*done_falling),
                    KqStateKind::ConnerJump { done_jumping } => self.i32(*done_jumping),
                    KqStateKind::Empty { .. } => {}
                    KqStateKind::Unknown { remaining, .. } => self.buf.extend_from_slice(remaining),
                }
            }
        }

        let payload_size = (self.buf.len() - payload_start) as u32;
        self.buf[size_pos..size_pos + 4].copy_from_slice(&payload_size.to_le_bytes());
        if payload_size % 2 == 1 {
            self.buf.push(0);
        }
    }
}

/// Serializes a `PersObject` into a 2-byte aligned binary `PERS` block.
pub fn write_pers(obj: &PersObject) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_block(obj);
    w.buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_state_matches_exact_44_byte_layout_and_odd_class_padding() {
        let gs = PersObject::KqGameState(KqGameState {
            base: KqBase {
                name: Some("GameState".into()),
            },
            next_no_name: 439,
            difficulty_factor: 0.66,
        });
        let bytes = write_pers(&gs);
        assert_eq!(bytes.len(), 44, "KQGameState PERS chunk must be 44 bytes");
        assert_eq!(&bytes[0..4], b"PERS");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 36);
        assert_eq!(u16::from_le_bytes(bytes[8..10].try_into().unwrap()), 11);
        assert_eq!(&bytes[10..22], b"KQGameState\0");
        assert_eq!(parse_pers(&bytes).unwrap(), gs);
    }

    #[test]
    fn strings_distinguish_null_empty_odd_and_even() {
        let ws = PersObject::KqWorldState(KqWorldState {
            base: KqBase {
                name: Some("WorldState".into()),
            },
            events: vec![
                None,
                Some(String::new()),
                Some("abc".into()),
                Some("abcd".into()),
                Some("KQCueEvent Connor 308.749695 2300 144 (null)".into()),
            ],
        });
        let bytes = write_pers(&ws);
        assert_eq!(parse_pers(&bytes).unwrap(), ws);
    }

    #[test]
    fn round_trips_camera_map_inventory_door_chest_and_connor() {
        let mut fog = vec![false; 460 * 460];
        fog[10] = true;
        fog[460 * 5 + 123] = true;
        let map = PersObject::KqMap(KqMap::from_map_keys(&MapKeys {
            name: "davMap".into(),
            north: 344068.0,
            south: 155670.0,
            east: 303114.0,
            west: 114699.0,
            use_fog: true,
            fog_radius: Some(10000.0),
            hold_map_loc: false,
            is_active: true,
            blank_map: "davmap0.pbm".into(),
            full_map: "davmap.pbm".into(),
            arrow_pba: "maparrow.pba".into(),
            map_name: Some(38),
            earned_teleport: true,
            teleport_script: Some("Davtele.cs".into()),
            teleport_portal: Some("Davtele.kq".into()),
            icons: vec![MapIcon {
                picture: "davmap.pbm".into(),
                file: "dv_tree2.bmp".into(),
                x: 200,
                y: 150,
            }],
            runs: crate::magic_map::runs_from_fog(&fog),
        }));
        assert_eq!(parse_pers(&write_pers(&map)).unwrap(), map);

        let cam = PersObject::KqCamMover(KqCamMover {
            visible: KqVisibleBase {
                base: KqBase {
                    name: Some("KQCamera".into()),
                },
                ..Default::default()
            },
            follow_object_name: Some("Connor".into()),
            loc: [100.0, 200.0, 300.0],
            camera_rotation: [-0.2, 0.0, 1.5],
            connor_mat: TMat3F {
                flags: 3,
                rotation: [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                position: [150.0, 250.0, 50.0],
            },
            ..Default::default()
        });
        let cam_bytes = write_pers(&cam);
        assert_eq!(cam_bytes.len(), 502, "KQCamMover with followObjectName=Connor is 502 bytes");
        assert_eq!(parse_pers(&cam_bytes).unwrap(), cam);

        let item = PersObject::KqInventoryItem(KqInventoryItem {
            object: KqObject {
                visible: KqVisibleBase {
                    base: KqBase {
                        name: Some("NoName5".into()),
                    },
                    ..Default::default()
                },
                shape_name: Some("crystl.kqs".into()),
                ..Default::default()
            },
            quantity: 3,
            user_id: 0xFFFF_FFFF,
            time_left: -1.0,
            item_type: 3020,
        });
        let world_inv = PersObject::KqInventory {
            class: "KQWorldInventory".into(),
            data: KqInventory {
                base: KqBase {
                    name: Some("WorldInventory".into()),
                },
                owner_id: 0xFFFF_FFFF,
                items: vec![Some(("NoName5".into(), Box::new(item.clone())))],
            },
        };
        assert_eq!(parse_pers(&write_pers(&world_inv)).unwrap(), world_inv);

        let door = PersObject::KqDoor(KqDoor {
            anim_object: KqAnimObject {
                object: KqObject {
                    visible: KqVisibleBase {
                        base: KqBase {
                            name: Some("Condoor".into()),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                curr_anim_name: Some("opened".into()),
                anim_pos: 1.0,
                ..Default::default()
            },
            state: 3,
        });
        assert_eq!(parse_pers(&write_pers(&door)).unwrap(), door);

        let chest = PersObject::KqChest(KqChest {
            anim_object: KqAnimObject {
                object: KqObject {
                    visible: KqVisibleBase {
                        base: KqBase {
                            name: Some("smlchst3".into()),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                curr_anim_name: Some("open".into()),
                anim_pos: 1.0,
                ..Default::default()
            },
            inv_item_type_name: Some("INVITEM_Crystal".into()),
            inv_quantity: 1,
            chest_state: 1,
            inv_item_name: Some("NoName12".into()),
            ..Default::default()
        });
        assert_eq!(parse_pers(&write_pers(&chest)).unwrap(), chest);

        let connor = PersObject::KqConner(Box::new(KqConner {
            monster: KqMonster {
                anim_object: KqAnimObject {
                    object: KqObject {
                        visible: KqVisibleBase {
                            base: KqBase {
                                name: Some("Connor".into()),
                            },
                            ..Default::default()
                        },
                        stats: Some("25 14 0 3 500".into()),
                        ..Default::default()
                    },
                    curr_anim_name: Some("Stop".into()),
                    ..Default::default()
                },
                state: Some((
                    "KQStateConnerWait".into(),
                    Box::new(PersObject::KqState(KqState {
                        base: KqBase {
                            name: Some("KQStateConnerWait".into()),
                        },
                        from_state: 0,
                        kind: KqStateKind::Wait {
                            class: "KQStateConnerWait".into(),
                            con_radius: 100.0,
                            mon_radius: 100.0,
                        },
                    })),
                )),
                inventory: Some((
                    "ConnerInventory".into(),
                    Box::new(PersObject::KqInventory {
                        class: "KQConnerInventory".into(),
                        data: KqInventory {
                            base: KqBase {
                                name: Some("ConnerInventory".into()),
                            },
                            owner_id: 1,
                            items: vec![Some(("NoName5".into(), Box::new(item)))],
                        },
                    }),
                )),
                ..Default::default()
            },
            level: 2,
            item_in_hand: Some("NoName5".into()),
            anim_list: Some((
                "ConnerAnimList".into(),
                Box::new(PersObject::KqConAnimList(KqConAnimList {
                    base: KqBase {
                        name: Some("ConnerAnimList".into()),
                    },
                    entries: vec![Some("None conner.anm".into())],
                })),
            )),
            skin_list: Some((
                "ConnerSkinList".into(),
                Box::new(PersObject::KqSkinList(KqSkinList {
                    base: KqBase {
                        name: Some("ConnerSkinList".into()),
                    },
                    skins: vec![Some("conner.bmp 0".into())],
                    inv_items: vec![Some("INVITEM_LeatherArmor".into())],
                })),
            )),
            hold_help: vec![Some("101".into())],
            ..Default::default()
        }));
        assert_eq!(parse_pers(&write_pers(&connor)).unwrap(), connor);
    }
}
