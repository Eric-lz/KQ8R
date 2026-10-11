//! Locating files inside the game install.
//!
//! The original game ran on Windows, where file names ignore case
//! (`DAVENTRY`, `Daventry` and `daventry` are the same folder). The install
//! mixes cases freely, so lookups here ignore case on every platform.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The 12 worlds, as named under `data/` in the GOG 1.03 install.
pub const WORLDS: [&str; 12] = [
    "Daventry", "Swamp", "DeadCity", "Gnome", "Barren", "IceWorld",
    "Snowexit", "temple1", "temple2", "temple3", "temple4", "CastleD",
];

/// The four archives every world folder has.
pub const VOLUMES: [&str; 4] = ["kq", "resource", "8bit", "8gui"];

/// Root of a game install (the folder that contains `Mask.exe`).
#[derive(Debug, Clone)]
pub struct GameDir {
    root: PathBuf,
}

impl GameDir {
    /// Checks that `root` looks like a KQ8 install and wraps it.
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = root.into();
        let dir = GameDir { root };
        if dir.find("Mask.exe").is_none() || dir.find("data").is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "{} does not look like a KQ8 install (no Mask.exe or data folder)",
                    dir.root.display()
                ),
            ));
        }
        Ok(dir)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a `/`-separated relative path, ignoring case in every part.
    /// Returns `None` if any part does not exist.
    pub fn find(&self, relative: &str) -> Option<PathBuf> {
        find_ci(&self.root, relative)
    }

    /// Path of `data/<world>/<volume>.vol`.
    pub fn volume_path(&self, world: &str, volume: &str) -> Option<PathBuf> {
        self.find(&format!("data/{world}/{volume}.vol"))
    }
}

/// Resolves `relative` under `base`, matching each path part without
/// regard to ASCII case. An exact match is preferred when one exists.
pub fn find_ci(base: &Path, relative: &str) -> Option<PathBuf> {
    let mut current = base.to_path_buf();
    for part in relative.split(['/', '\\']).filter(|p| !p.is_empty()) {
        let exact = current.join(part);
        if exact.exists() && name_matches_exactly(&current, part) {
            current = exact;
            continue;
        }
        let found = fs::read_dir(&current).ok()?.find_map(|e| {
            let e = e.ok()?;
            e.file_name()
                .to_str()
                .filter(|n| n.eq_ignore_ascii_case(part))
                .map(|_| e.path())
        })?;
        current = found;
    }
    Some(current)
}

// On Windows `exists()` already ignores case, so check the listing to know
// whether the spelling really matches. Cheap enough for our usage.
fn name_matches_exactly(dir: &Path, part: &str) -> bool {
    fs::read_dir(dir)
        .map(|mut it| it.any(|e| e.map(|e| e.file_name() == part).unwrap_or(false)))
        .unwrap_or(false)
}
