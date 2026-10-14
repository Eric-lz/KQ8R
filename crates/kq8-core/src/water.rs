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

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_data::envinfo::{POISON, WATER};

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
