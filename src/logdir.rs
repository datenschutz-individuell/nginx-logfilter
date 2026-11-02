// SPDX-FileCopyrightText: 2026 Olav Seyfarth
// SPDX-License-Identifier: CC0-1.0

//! Appends lines to `<dir>/<tag>.err`, keeping the files open.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::PathBuf;

pub struct LogDir {
    dir: PathBuf,
    open: HashMap<String, (File, u64)>,
}

impl LogDir {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), open: HashMap::new() }
    }

    /// Appends `line` and a newline. `tag` must already be validated by the caller.
    ///
    /// A file whose inode changed (logrotate moved it away) is reopened, so rotation needs no signal.
    pub fn append(&mut self, tag: &str, line: &str) -> io::Result<()> {
        let path = self.dir.join(format!("{tag}.err"));
        let current = fs::metadata(&path).map(|m| m.ino()).ok();
        let file = match self.open.get_mut(tag) {
            Some((file, ino)) if Some(*ino) == current => file,
            _ => {
                let file = OpenOptions::new().create(true).append(true).mode(0o640).open(&path)?;
                let ino = file.metadata()?.ino();
                &mut self.open.entry(tag.to_owned()).insert_entry((file, ino)).into_mut().0
            }
        };
        let mut buf = String::with_capacity(line.len() + 1);
        buf.push_str(line);
        buf.push('\n');
        file.write_all(buf.as_bytes()) // one write per line, so concurrent readers never see half a line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_is_noticed() {
        let d = std::env::temp_dir().join(format!("nginx-logfilter-test-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        let mut logs = LogDir::new(&d);
        logs.append("t", "one").unwrap();
        fs::rename(d.join("t.err"), d.join("t.err.1")).unwrap();
        logs.append("t", "two").unwrap();
        assert_eq!(fs::read_to_string(d.join("t.err.1")).unwrap(), "one\n");
        assert_eq!(fs::read_to_string(d.join("t.err")).unwrap(), "two\n");
        fs::remove_dir_all(d).unwrap();
    }
}
