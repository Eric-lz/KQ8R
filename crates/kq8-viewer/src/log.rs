//! Log file writer, same behaviour as the one in `kq8-cli`: every line
//! goes to `kq8-artifacts/logs/<name>.log`, summary lines also to the
//! terminal. Lines are flushed at once so the log is complete even if the
//! window is closed abruptly.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct Log {
    file: File,
    path: PathBuf,
    /// While set, summary lines go to the file only (used while loading
    /// hundreds of object models).
    pub quiet: bool,
}

impl Log {
    pub fn create(name: &str) -> io::Result<Self> {
        let dir = Path::new("kq8-artifacts").join("logs");
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{name}.log"));
        Ok(Log { file: File::create(&path)?, path, quiet: false })
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
