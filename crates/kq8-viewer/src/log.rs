//! Log file writer, same behaviour as the one in `kq8-cli`: every line
//! goes to `kq8-artifacts/logs/<name>.log`, summary lines also to the
//! terminal. Lines are flushed at once so the log is complete even if the
//! window is closed abruptly.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Log {
    file: File,
    path: PathBuf,
    /// While set, summary lines go to the file only (used while loading
    /// hundreds of object models).
    pub quiet: bool,
}

/// The logs opened in this run (M6 6e-1): the first open of a name
/// empties its file, later ones (a world built again) add to it.
static OPENED: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

impl Log {
    pub fn create(name: &str) -> io::Result<Self> {
        let dir = Path::new("kq8-artifacts").join("logs");
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{name}.log"));
        let first = OPENED.lock().map(|mut o| o.insert(name.to_string())).unwrap_or(true);
        let file = if first { File::create(&path)? } else { OpenOptions::new().append(true).create(true).open(&path)? };
        Ok(Log { file, path, quiet: false })
    }

    /// Adds `line` to every log opened in this run except `skip` (M6
    /// 6e-2: the world change line). Only for logs whose writer is closed
    /// (another open writer would write over it); `skip` the others.
    pub fn mark_all(line: &str, skip: &[&str]) {
        let names: Vec<String> = OPENED.lock().map(|o| o.iter().cloned().collect()).unwrap_or_default();
        for name in names.iter().filter(|n| !skip.contains(&n.as_str())) {
            let path = Path::new("kq8-artifacts").join("logs").join(format!("{name}.log"));
            if let Ok(mut f) = OpenOptions::new().append(true).open(&path) {
                let _ = writeln!(f, "{line}");
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn detail(&mut self, line: impl AsRef<str>) {
        let _ = writeln!(self.file, "{}", line.as_ref());
    }

    pub fn summary(&mut self, line: impl AsRef<str>) {
        if !self.quiet {
            println!("{}", line.as_ref());
        }
        self.detail(line);
    }
}
