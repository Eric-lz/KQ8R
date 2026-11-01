//! The water test (docs/behaviour/water.md "The water test", traced):
//! after every move, an object is `submerged` when the ground square it is
//! on is a liquid (water, poison, fastpoison/lava, swamp) and the surface
//! there is at least 600 above its feet. For Connor it only changes which
//! animation plays (`waterwalk`, `waterrun`); nothing slows him.

use kq8_data::envinfo::LIQUID;

/// Surface at least this far above the feet → `submerged`.
pub const SUBMERGED_ABOVE: f32 = 600.0;

/// `water` = the surface height and ground flags at the position (from
/// `MoverWorld::water`); `feet` = the object's height.
pub fn submerged(water: Option<(f32, u32)>, feet: f32) -> bool {
    water.is_some_and(|(surface, flags)| flags & LIQUID != 0 && surface >= feet + SUBMERGED_ABOVE)
}

/// Liquid damage (water.md "Liquid damage", Q191; U-7): "deep" is a
/// liquid with the surface at least this far above the feet.
pub const DEEP_ABOVE: f32 = 250.0;
/// A timer of 0, or one not set: this much damage (instant death).
pub const UNSET_DAMAGE: i64 = 1000;
/// x = `RATE` × Connor's level ÷ the timer's value.
pub const RATE: f64 = 0.66;
/// Below 1 a second: 1 every n-th update, n at most this.
pub const MAX_EVERY: i64 = 8;
/// Connor's `poison<n>.aud` at most this often (seconds).
pub const POISON_SOUND_EVERY: f64 = 2.5;
/// Rule 6: an `ontop` square hurts with the surface at most this far below
/// the feet; he sinks 15 while it is less than 250 above them.
pub const ONTOP_BELOW: f32 = 100.0;

/// Rule 1: the timer variable for the square's flags (poison, then
/// fastpoison / lava, then swamp); `None`: no damage.
pub fn timer_name(flags: u32) -> Option<&'static str> {
    use kq8_data::envinfo::{FAST_POISON, POISON, SWAMP};
    if flags & POISON != 0 {
        Some("KQWorld::PoisonTime")
    } else if flags & FAST_POISON != 0 {
        Some("KQWorld::FastPoisonTime")
    } else if flags & SWAMP != 0 {
        Some("KQWorld::SwampTime")
    } else {
        None
    }
}

/// Rules 2-3: the damage this update, from the timer's `value` (`None` =
/// not set), Connor's `level` and `k`, the update's place in the current
/// second (the second's fraction × 15, rounded).
pub fn damage(value: Option<f64>, level: i64, k: i64) -> i64 {
    let v = value.unwrap_or(0.0);
    if v == 0.0 {
        return UNSET_DAMAGE;
    }
    let x = RATE * level as f64 / v;
    if x >= 1.0 {
        return x.floor() as i64;
    }
    let n = ((1.0 / x).floor() as i64).clamp(1, MAX_EVERY);
    i64::from(k % n == 0)
}

/// Rule 3: k for game time `now`.
pub fn place_in_second(now: f64) -> i64 {
    (now.fract() * 15.0).round() as i64
}

/// Rule 7: the hit's kind number: Fire (2) with `fire`, else Ice (3) with
/// `ice`, else Normal (0).
pub fn kind_number(flags: u32) -> u8 {
    use kq8_data::envinfo::{FIRE, ICE};
    if flags & FIRE != 0 {
        2
    } else if flags & ICE != 0 {
        3
    } else {
        0
    }
}

/// Rule 6: a liquid with the surface at least 250 above the feet.
pub fn deep(flags: u32, surface: f32, feet: f32) -> bool {
    flags & LIQUID != 0 && surface >= feet + DEEP_ABOVE
}

/// What Connor's water test saw after his move this update (U-7), for the
/// damage: the square's flags, the surface there, his feet, and whether
/// he moved at all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidSample {
    pub flags: u32,
    pub surface: f32,
    pub feet: f32,
    pub moved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_data::envinfo::{POISON, WATER};

    #[test]
    fn the_liquid_damage_numbers() {
        // water.md "Liquid damage" rule 3: PoisonTime 5.
        let hits = |level: i64| (0..16).filter(|&k| damage(Some(5.0), level, k) > 0).collect::<Vec<_>>();
        assert_eq!(hits(1), vec![0, 7, 14], "level 1: n 7");
        assert_eq!(hits(2), vec![0, 3, 6, 9, 12, 15], "level 2: n 3");
        assert_eq!(hits(3).len(), 8, "level 3: n 2");
        assert_eq!(damage(Some(5.0), 4, 1), 1, "level 4: every update");
        assert_eq!(damage(Some(5.0), 8, 1), 1);
        assert_eq!(damage(Some(5.0), 16, 1), 2);
        // FastPoisonTime 0, or a timer not set: 1,000.
        assert_eq!(damage(Some(0.0), 1, 3), 1000);
        assert_eq!(damage(None, 1, 3), 1000);
        assert_eq!(timer_name(POISON | 0x80), Some("KQWorld::PoisonTime"));
        assert_eq!(timer_name(0x80), Some("KQWorld::FastPoisonTime"));
        assert_eq!(timer_name(WATER | 0x200), None, "fire alone does nothing");
        assert_eq!(kind_number(0x80 | 0x200), 2);
        assert!(deep(POISON, 250.0, 0.0) && !deep(POISON, 249.0, 0.0) && !deep(0x4, 900.0, 0.0));
        assert_eq!(place_in_second(10.5), 8);
    }

    #[test]
    fn submerged_needs_a_liquid_and_600_of_depth() {
        // Daventry's river: surface 1,020 above the floor he stands on.
        assert!(submerged(Some((1020.0, WATER)), 0.0));
        assert!(submerged(Some((600.0, POISON)), 0.0));
        assert!(!submerged(Some((599.0, WATER)), 0.0));
        // Deep but not a liquid (e.g. under the Swamp shack).
        assert!(!submerged(Some((6885.0, 0x1)), 0.0));
        // Off the grid.
        assert!(!submerged(None, 0.0));
    }
}
