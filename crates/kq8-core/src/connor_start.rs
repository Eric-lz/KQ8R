//! What the start-up scripts give Connor, done when play starts (DESIGN
//! "Connor's levels checked", step L-1).
//!
//! Every world's `conStart.cs` gives Connor items
//! (`KQMonster::inventory Connor ...`), sets his level
//! (`KQConner::setLevel N`) and sends him a stats event. The items are
//! made while the scripts load the world; here, in script order:
//!
//! - an item goes into his inventory by the rule of a `give`
//!   (`inventory.md` rule 8: equipment is put on and its numbers added);
//! - `setLevel` sets his level (stand-in, Q149 a);
//! - `sendEvent` is sent as a script sends it (`anim-scripts.md` rule 19
//!   for the stats event).

use crate::active::WorldState;
use crate::script::ScriptHost;

/// One start-up line for Connor, in script order.
#[derive(Debug, Clone, PartialEq)]
pub enum StartLine {
    /// An item the scripts made, carried by Connor (its object name).
    Item { name: String, at: String },
    /// `KQConner::setLevel` or `sendEvent`, words expanded.
    Command { words: Vec<String>, at: String },
}

/// Does the lines in order; returns one log line each.
pub fn run(w: &mut WorldState, lines: &[StartLine]) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        match line {
            StartLine::Item { name, at } => {
                crate::connor_inventory::receive(w, name, None, at);
                out.push(format!("{at}: {name} goes into Connor's inventory (inventory.md rule 8)"));
            }
            StartLine::Command { words, at } => {
                let result = ScriptHost::world_command(w, words, at);
                out.push(format!("{at}: {}: {result:?}", words.join(" ")));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active::Active;
    use crate::stats::{ConnorStats, Stats};

    #[test]
    fn set_level_sets_only_the_level() {
        let mut a = Active::new(kq8_data::world_script::Vars::default(), vec![], vec![], ["Connor".to_string()], []);
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        let words: Vec<String> = ["KQConner::setLevel", "8"].iter().map(|s| s.to_string()).collect();
        let log = run(&mut a.world, &[StartLine::Command { words, at: "conStart.cs line 14".into() }]);
        assert_eq!(log, vec!["conStart.cs line 14: KQConner::setLevel 8: Some(\"0\")".to_string()]);
        assert_eq!(a.world.connor_stats, Some(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 8 }));
    }
}
