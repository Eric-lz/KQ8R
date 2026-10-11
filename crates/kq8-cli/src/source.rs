//! Loading a single game file by name, either loose or from an archive.
//!
//! - `GAME/8Gui/daventry.pbm`: a path inside the game folder.
//! - `vol:Daventry/8bit/NAME.BMP`: entry NAME of `data/Daventry/8bit.vol`.

use kq8_data::{GameDir, VolArchive};

pub fn load(game: &GameDir, spec: &str) -> Result<Vec<u8>, String> {
    if let Some(rest) = spec.strip_prefix("vol:") {
        let mut parts = rest.splitn(3, '/');
        let (Some(world), Some(volume), Some(name)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(format!("'{spec}': expected vol:<World>/<volume>/<NAME>"));
        };
        let path = game
            .volume_path(world, volume)
            .ok_or(format!("no archive data/{world}/{volume}.vol"))?;
        let vol = VolArchive::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let entry = vol
            .find(name)
            .ok_or(format!("'{name}' not found in {}", path.display()))?;
        vol.unpack(entry).map(|u| u.data).map_err(|e| e.to_string())
    } else {
        let path = game
            .find(spec)
            .ok_or(format!("'{spec}' not found in {}", game.root().display()))?;
        std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// File name part of a spec, for naming output files.
pub fn stem(spec: &str) -> String {
    let name = spec.rsplit(['/', '\\']).next().unwrap_or(spec);
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    stem.replace(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_', "_")
}
