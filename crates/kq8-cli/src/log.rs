//! Minimal log file writer: every line goes to the log file, and lines
//! marked as summary are also printed to the terminal.

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

pub const ARTIFACTS_DIR: &str = "kq8-artifacts";

pub struct Log {
    file: BufWriter<File>,
    path: PathBuf,
}

impl Log {
    /// Creates (or overwrites) `kq8-artifacts/logs/<name>.log`.
    pub fn create(name: &str) -> io::Result<Self> {
        let dir = Path::new(ARTIFACTS_DIR).join("logs");
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{name}.log"));
        let file = BufWriter::new(File::create(&path)?);
        Ok(Log { file, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Log file only.
    pub fn detail(&mut self, line: impl AsRef<str>) {
        let _ = writeln!(self.file, "{}", line.as_ref());
    }

    /// Log file and terminal.
    pub fn summary(&mut self, line: impl AsRef<str>) {
        println!("{}", line.as_ref());
        self.detail(line);
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        let _ = self.file.flush();
    }
}
