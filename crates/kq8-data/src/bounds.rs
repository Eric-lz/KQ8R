//! The game-wide table of collision boxes, `GAME/patch/BOUNDS.KQ`
//! (section `[BoundingBoxes]`, class `KQBoundingBoxManager`).
//!
//! Rules from `docs/behaviour/connor-physics.md` "Where every object's box
//! comes from" (Q33): `nItems=<n>`, then lines
//! `itemN=<name> xmin ymin zmin xmax ymax zmax`, in the object's own space.
//! Lines starting with `;` are comments. The fields are separated by one or
//! more spaces (seen in the data). A line such as `#item968=...` is not an
//! `itemN` key, so it is not an entry.
//!
//! Every entry is kept, in item order, as written (`kq8 building-verify`
//! reports on them). `lookup` gives the box the game uses for a name
//! (Q34: case matters, duplicates as read from the running game,
//! inside-out boxes fixed).

use crate::kq_text;

/// Names that really appear twice (same case) in `GAME/patch/BOUNDS.KQ`,
/// and the item the game ends up using (connor-physics.md "How the table
/// is read", read from the running game, Q34). Any other duplicate: the
/// first in item order.
const DUPLICATES: [(&str, usize); 2] = [("rat", 8), ("razetta.kqs", 775)];

#[derive(Debug, Clone, PartialEq)]
pub struct BoxEntry {
    /// The N of `itemN`.
    pub item: usize,
    /// Name as written (a shape such as `signpst1.kqs`, or an object name).
    pub name: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone, Default)]
pub struct BoundingBoxes {
    /// Sorted by item number.
    pub entries: Vec<BoxEntry>,
    /// `nItems` as written, if present.
    pub declared: Option<usize>,
}

impl BoundingBoxes {
    /// Reads the table. Lines that can't be read are returned as problems
    /// (one text each) and left out.
    pub fn parse(bytes: &[u8]) -> (Self, Vec<String>) {
        let mut problems = Vec::new();
        let sections = kq_text::parse(bytes);
        let Some(section) = sections.iter().find(|s| s.name.eq_ignore_ascii_case("BoundingBoxes")) else {
            problems.push("no [BoundingBoxes] section".to_string());
            return (Self::default(), problems);
        };
        let declared = match section.get("nItems").map(|v| v.parse::<usize>()) {
            Some(Ok(n)) => Some(n),
            Some(Err(_)) => {
                problems.push(format!("nItems={} is not a number", section.get("nItems").unwrap_or("")));
                None
            }
            None => None,
        };
        let mut entries = Vec::new();
        for (key, value) in &section.entries {
            let Some(item) = key
                .get(..4)
                .filter(|p| p.eq_ignore_ascii_case("item"))
                .and_then(|_| key[4..].parse::<usize>().ok())
            else {
                continue;
            };
            let fields: Vec<&str> = value.split_whitespace().collect();
            let numbers: Option<Vec<f32>> = fields.get(1..).and_then(|f| f.iter().map(|v| v.parse().ok()).collect());
            match (fields.first(), numbers) {
                (Some(name), Some(n)) if n.len() == 6 => entries.push(BoxEntry {
                    item,
                    name: name.to_string(),
                    min: [n[0], n[1], n[2]],
                    max: [n[3], n[4], n[5]],
                }),
                _ => problems.push(format!("{key}={value}: expected a name and 6 numbers")),
            }
        }
        entries.sort_by_key(|e| e.item);
        (BoundingBoxes { entries, declared }, problems)
    }

    /// The box the game uses for `name` (connor-physics.md "How the table
    /// is read", traced, Q34): only keys `item0` … `item<nItems − 1>`;
    /// names compared byte by byte (case matters); the two names that
    /// really appear twice resolved as read from the running game; on
    /// each axis the smaller number is the min (inside-out boxes fixed).
    pub fn lookup(&self, name: &str) -> Option<BoxEntry> {
        let in_range = |e: &&BoxEntry| self.declared.is_none_or(|n| e.item < n);
        let found: Vec<&BoxEntry> = self.entries.iter().filter(in_range).filter(|e| e.name == name).collect();
        let chosen = match found.as_slice() {
            [] => return None,
            [one] => *one,
            several => DUPLICATES
                .iter()
                .find(|(n, _)| *n == name)
                .and_then(|(_, item)| several.iter().find(|e| e.item == *item))
                .unwrap_or(&several[0]),
        };
        let mut e = chosen.clone();
        for k in 0..3 {
            if e.min[k] > e.max[k] {
                std::mem::swap(&mut e.min[k], &mut e.max[k]);
            }
        }
        Some(e)
    }

    /// Every entry whose name equals `name`, ignoring case, in item order.
    pub fn find_all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a BoxEntry> + 'a {
        self.entries.iter().filter(move |e| e.name.eq_ignore_ascii_case(name))
    }

    /// Names (lower case) that appear more than once, ignoring case, with
    /// whether their boxes differ.
    pub fn duplicates(&self) -> Vec<(String, bool)> {
        let mut by_name: std::collections::BTreeMap<String, Vec<&BoxEntry>> = Default::default();
        for e in &self.entries {
            by_name.entry(e.name.to_ascii_lowercase()).or_default().push(e);
        }
        by_name
            .into_iter()
            .filter(|(_, v)| v.len() > 1)
            .map(|(name, v)| {
                let differ = v.iter().any(|e| e.min != v[0].min || e.max != v[0].max);
                (name, differ)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[u8] = b"[BoundingBoxes]\r\n\
        classType=KQBoundingBoxManager\r\n\
        nItems=4\r\n\
        \";xmin, ymin, zmin, xmax, ymax, zmax\"\r\n\
        ;item0=Connor -500 -370 0 250 250 1600\r\n\
        item0=Connor -200.000000 -200.000000 0.000000 200.000000 200.000000 1300.000000\r\n\
        item2=elchair      -320 -10 200 310 480 1000\r\n\
        item1=rat -70 -200 0 30 70 70\r\n\
        #item3=Rat -1 -1 0 1 1 1\r\n\
        item3=Rat -80 -60 0 80 60 30\r\n\
        item4=broken 1 2 3\r\n";

    #[test]
    fn reads_items_skips_comments_and_hash_lines() {
        let (b, problems) = BoundingBoxes::parse(SAMPLE);
        assert_eq!(b.declared, Some(4));
        assert_eq!(b.entries.iter().map(|e| e.item).collect::<Vec<_>>(), [0, 1, 2, 3]);
        let connor: Vec<_> = b.find_all("CONNOR").collect();
        assert_eq!(connor.len(), 1);
        assert_eq!((connor[0].min, connor[0].max), ([-200.0, -200.0, 0.0], [200.0, 200.0, 1300.0]));
        // Several spaces between fields.
        assert_eq!(b.find_all("elchair").next().unwrap().min, [-320.0, -10.0, 200.0]);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].starts_with("item4="));
    }

    #[test]
    fn duplicates_are_kept_and_reported() {
        let (b, _) = BoundingBoxes::parse(SAMPLE);
        let rats: Vec<_> = b.find_all("rat").map(|e| e.item).collect();
        assert_eq!(rats, [1, 3]);
        assert_eq!(b.duplicates(), [("rat".to_string(), true)]);
    }

    #[test]
    fn lookup_follows_the_game() {
        let (b, _) = BoundingBoxes::parse(
            b"[BoundingBoxes]
nItems=6
            item0=Connor -200 -200 0 200 200 1300
            item1=door.kqs 10 -240 20 -1520 10 3020
            item2=rat 1 1 1 2 2 2
            item3=rat 3 3 3 4 4 4
            item4=other 0 0 0 1 1 1
            item5=other 0 0 0 2 2 2
            item6=late 0 0 0 1 1 1
",
        );
        assert_eq!(b.lookup("Connor").map(|e| e.max), Some([200.0, 200.0, 1300.0]));
        assert_eq!(b.lookup("connor"), None, "case matters");
        // Inside-out on x: fixed.
        let door = b.lookup("door.kqs").unwrap();
        assert_eq!((door.min, door.max), ([-1520.0, -240.0, 20.0], [10.0, 10.0, 3020.0]));
        // Duplicates: rat as the note says (item 8 in the real table is the
        // first; here no item 8, so the first), any other: the first.
        assert_eq!(b.lookup("rat").map(|e| e.item), Some(2));
        assert_eq!(b.lookup("other").map(|e| e.item), Some(4));
        // Past nItems: not an entry.
        assert_eq!(b.lookup("late"), None);
    }

    #[test]
    fn lookup_razetta_uses_item_775() {
        let (b, _) = BoundingBoxes::parse(
            b"[BoundingBoxes]
nItems=800
            item686=razetta.kqs -160 -110 0 140 60 530
            item775=razetta.kqs -150 -150 0 150 150 750
",
        );
        assert_eq!(b.lookup("razetta.kqs").map(|e| (e.item, e.max)), Some((775, [150.0, 150.0, 750.0])));
    }
}
