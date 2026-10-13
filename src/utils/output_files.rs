//! Keeps track of the files a run writes, so the output can be updated in place:
//! unchanged files are left alone and only files that are no longer part of the
//! export are removed afterwards.

use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The files written while exploding one database. A database is processed on a
/// single thread, so no locking is needed.
#[derive(Debug, Default)]
pub struct OutputFiles {
    written: RefCell<HashSet<PathBuf>>,
}

impl OutputFiles {
    /// Writes `content` unless the file already has exactly this content.
    pub fn write(&self, path: &Path, content: &str) {
        let unchanged = fs::read(path).is_ok_and(|existing| existing == content.as_bytes());
        if !unchanged && let Err(err) = fs::write(path, content) {
            eprintln!("Error writing file {}: {}", path.display(), err);
        }
        self.written.borrow_mut().insert(path.to_path_buf());
    }

    /// Renames a written file; `to` is replaced if it exists.
    pub fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)?;
        let mut written = self.written.borrow_mut();
        written.remove(from);
        written.insert(to.to_path_buf());
        Ok(())
    }

    /// Whether `path` was written during this run.
    pub fn contains(&self, path: &Path) -> bool {
        self.written.borrow().contains(path)
    }

    /// Removes everything below `root` that wasn't written during this run, except
    /// `.git` directories. Directories that become empty are removed as well.
    pub fn remove_stale(&self, root: &Path) -> io::Result<()> {
        if self.remove_stale_below(root)? {
            fs::remove_dir(root)?;
        }
        Ok(())
    }

    /// Returns whether `dir` is empty afterwards because stale entries were removed.
    fn remove_stale_below(&self, dir: &Path) -> io::Result<bool> {
        let Ok(entries) = fs::read_dir(dir) else {
            return Ok(false);
        };
        let mut removed_any = false;
        let mut kept_any = false;
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.file_name().is_some_and(|name| name == ".git") {
                kept_any = true;
            } else if path.is_dir() {
                if self.remove_stale_below(&path)? {
                    fs::remove_dir(&path)?;
                    removed_any = true;
                } else {
                    kept_any = true;
                }
            } else if self.contains(&path) {
                kept_any = true;
            } else {
                fs::remove_file(&path)?;
                removed_any = true;
            }
        }
        // Empty directories created by this run are kept, as before
        Ok(removed_any && !kept_any)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    use super::OutputFiles;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fm-xml-export-exploder-output-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn unchanged_files_are_not_rewritten() {
        let dir = temp_dir("unchanged");
        let file = dir.join("script.txt");
        fs::write(&file, "Beep\n").unwrap();
        let old = SystemTime::now() - Duration::from_secs(3600);
        fs::File::options()
            .write(true)
            .open(&file)
            .unwrap()
            .set_modified(old)
            .unwrap();

        let output = OutputFiles::default();
        output.write(&file, "Beep\n");
        assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), old);

        output.write(&file, "Halt Script\n");
        assert_eq!(fs::read_to_string(&file).unwrap(), "Halt Script\n");
        assert!(output.contains(&file));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn removes_only_stale_entries() {
        let root = temp_dir("stale");
        for dir in ["scripts/Folder", "scripts/Old Folder", "empty", ".git"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        fs::write(root.join(".git/HEAD"), "ref").unwrap();
        fs::write(root.join("scripts/Old Folder/old.xml"), "").unwrap();
        fs::write(root.join("scripts/stale.xml"), "").unwrap();

        let output = OutputFiles::default();
        output.write(&root.join("scripts/Folder/kept.xml"), "kept");
        output.write(&root.join("scripts/moved.tmp"), "moved");
        output
            .rename(
                &root.join("scripts/moved.tmp"),
                &root.join("scripts/moved.xml"),
            )
            .unwrap();

        output.remove_stale(&root).unwrap();

        assert!(root.join("scripts/Folder/kept.xml").exists());
        assert!(root.join("scripts/moved.xml").exists());
        assert!(root.join(".git/HEAD").exists());
        assert!(!root.join("scripts/stale.xml").exists());
        assert!(!root.join("scripts/Old Folder").exists());
        // An empty directory without removed content is left as it is
        assert!(root.join("empty").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn removes_a_root_that_only_held_stale_files() {
        let root = temp_dir("stale-root");
        fs::write(root.join("old.xml"), "").unwrap();

        OutputFiles::default().remove_stale(&root).unwrap();
        assert!(!root.exists());
    }
}
