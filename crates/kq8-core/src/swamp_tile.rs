//! `KQSwampTile` (`tile-puzzle.md` rules 36–41): Swamp's 42 gas-trap and
//! stump floor tiles (`tile00`–`tile56`).

/// The `.kq` `classType` of a Swamp puzzle tile.
pub const CLASS: &str = "KQSwampTile";

/// Default `damFactor` when a tile's `.kq` file omits it (`tile-puzzle.md` rule 36).
pub const DEFAULT_DAM_FACTOR: f64 = 0.48;

/// Cooldown between gas + damage blasts while standing on an unsafe tile (seconds).
pub const TOUCH_COOLDOWN: f64 = 2.0;

/// Template file spawned (`KQAnimOnce`, auto-deleted) when an unsafe armed tile puffs gas.
pub const GAS_FILE: &str = "swmpgas.kqs";

/// Reveal swap textures preloaded in worlds with `KQSwampTile`s.
pub const REVEAL_TEXTURES: [&str; 4] = ["I_gastrp.bmp", "I_wtowr7.bmp", "I_trprng.bmp", "I_trunk.bmp"];

/// One placed `KQSwampTile` (`gastrap1`–`3` and `stump1`–`2`).
#[derive(Debug, Clone)]
pub struct SwampTile {
    pub name: String,
    pub height_num: i32,
    pub dam_factor: f64,
    pub revealed: bool,
    pub last_touch: Option<f64>,
}

impl SwampTile {
    pub fn new(name: impl Into<String>, height_num: i32, dam_factor: f64) -> Self {
        Self { name: name.into(), height_num, dam_factor, revealed: false, last_touch: None }
    }
}

/// Truthy check for `Swamp::Tile::PowerOn` and `Swamp::Tile::SwitchOn`.
///
/// `Swamp/kq/worldvar.cs` sets both to `"true"` / `"false"` strings rather than
/// `"1"` / `"0"`, whereas `script::number_value("true")` evaluates to `0.0`.
pub fn is_truthy(value: Option<&str>) -> bool {
    let Some(s) = value.map(str::trim).filter(|s| !s.is_empty()) else {
        return false;
    };
    if s.eq_ignore_ascii_case("true") {
        return true;
    }
    if s.eq_ignore_ascii_case("false") {
        return false;
    }
    crate::script::number_value(s) != 0.0
}

/// Rule 38–40: tiles with a script (`fiensafe.cs`) are safe; tiles without a script are unsafe.
pub fn is_safe(script: Option<&str>) -> bool {
    script.is_some()
}

/// Rule 39: vertical offset above the tile's position for spawning `swmpgas.kqs`.
pub fn gas_z_offset(height_num: i32) -> f32 {
    match height_num {
        1 => 750.0,
        2 | 12 => 1000.0,
        3 | 13 => 1250.0,
        _ => 1000.0,
    }
}

/// Rule 38: `(old_bmp, new_bmp)` pairs swapped on safe tiles when Clarity Reveal turns on.
pub fn reveal_pairs(height_num: i32) -> &'static [(&'static str, &'static str)] {
    match height_num {
        1..=3 => &[("gastrap.bmp", "I_gastrp.bmp"), ("wtower5.bmp", "I_wtowr7.bmp")],
        12..=13 => &[("trapring.bmp", "I_trprng.bmp"), ("trunk.bmp", "I_trunk.bmp")],
        _ => &[],
    }
}

/// Rule 40 step 2: direct health delta `trunc(-level * 10 * damFactor)` added to Connor's health.
pub fn touch_health_delta(level: i64, dam_factor: f64) -> i64 {
    (-(level as f64) * 10.0 * dam_factor).trunc() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truthy_handles_worldvar_strings_and_numbers() {
        assert!(!is_truthy(None));
        assert!(!is_truthy(Some("")));
        assert!(!is_truthy(Some("false")));
        assert!(!is_truthy(Some("FALSE")));
        assert!(!is_truthy(Some("0")));
        assert!(is_truthy(Some("true")));
        assert!(is_truthy(Some(" True ")));
        assert!(is_truthy(Some("1")));
    }

    #[test]
    fn offsets_pairs_and_touch_damage_match_rules_38_to_40() {
        assert_eq!(gas_z_offset(1), 750.0);
        assert_eq!(gas_z_offset(2), 1000.0);
        assert_eq!(gas_z_offset(12), 1000.0);
        assert_eq!(gas_z_offset(3), 1250.0);
        assert_eq!(gas_z_offset(13), 1250.0);
        assert_eq!(reveal_pairs(1).len(), 2);
        assert_eq!(reveal_pairs(3)[1], ("wtower5.bmp", "I_wtowr7.bmp"));
        assert_eq!(reveal_pairs(12)[0], ("trapring.bmp", "I_trprng.bmp"));
        assert_eq!(reveal_pairs(13)[1], ("trunk.bmp", "I_trunk.bmp"));
        assert!(reveal_pairs(0).is_empty());
        assert_eq!(touch_health_delta(1, DEFAULT_DAM_FACTOR), -4);
        assert_eq!(touch_health_delta(2, DEFAULT_DAM_FACTOR), -9);
        assert_eq!(touch_health_delta(5, DEFAULT_DAM_FACTOR), -24);
    }
}
