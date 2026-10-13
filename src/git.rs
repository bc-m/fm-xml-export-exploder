//! Versions the exploded output with git: one repository per output folder, one commit
//! per run when something changed.
//!
//! Uses the installed `git` CLI, so the user's git configuration (identity, signing,
//! default branch) applies. Output folders that already belong to another repository
//! (e.g. a target directory the user versions themselves) are left alone.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::config::OutputTree;

/// The folder that gets its own repository, and the databases exported into it.
pub fn repositories(
    root_out_dir: &Path,
    output_tree: &OutputTree,
    db_names: &[String],
) -> BTreeMap<PathBuf, Vec<String>> {
    let mut repositories: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    for db_name in db_names {
        match output_tree {
            // <target>/<db>/<domain>
            OutputTree::Db => {
                repositories
                    .entry(root_out_dir.join(db_name))
                    .or_default()
                    .push(db_name.clone());
            }
            // <target>/<domain>/<db>: every domain folder the database was written to
            OutputTree::Domain => {
                let Ok(entries) = fs::read_dir(root_out_dir) else {
                    continue;
                };
                for domain_dir in entries.flatten().map(|entry| entry.path()) {
                    if domain_dir.join(db_name).is_dir() {
                        repositories
                            .entry(domain_dir)
                            .or_default()
                            .push(db_name.clone());
                    }
                }
            }
        }
    }
    repositories
}

/// Commits each output folder to its own repository. Does nothing without a git CLI.
pub fn commit_output(repositories: &BTreeMap<PathBuf, Vec<String>>) {
    if !is_git_available() {
        println!("git not found, output is not committed.");
        return;
    }
    for (dir, db_names) in repositories {
        let mut db_names = db_names.clone();
        db_names.sort();
        let message = format!("Export {}", db_names.join(", "));
        match commit(dir, &message) {
            Ok(CommitResult::Committed) => println!("→ Committed {}", dir.display()),
            Ok(CommitResult::NoChanges) => println!("→ No changes in {}", dir.display()),
            Ok(CommitResult::InOtherRepository) => println!(
                "→ Not committing {}: it is already inside another git repository",
                dir.display()
            ),
            Err(err) => eprintln!("Failed to commit {}: {err}", dir.display()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CommitResult {
    Committed,
    NoChanges,
    InOtherRepository,
}

/// Initializes `dir` as a repository if needed and commits all changes in it.
pub fn commit(dir: &Path, message: &str) -> Result<CommitResult, String> {
    if !dir.join(".git").exists() {
        if is_inside_work_tree(dir) {
            return Ok(CommitResult::InOtherRepository);
        }
        run(dir, &["init", "--quiet"])?;
    }

    run(dir, &["add", "--all"])?;
    // Exit code 0 means nothing is staged
    if git(dir, &["diff", "--cached", "--quiet"])
        .map_err(|err| err.to_string())?
        .status
        .success()
    {
        return Ok(CommitResult::NoChanges);
    }
    run(dir, &["commit", "--quiet", "--message", message])?;
    Ok(CommitResult::Committed)
}

fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn is_inside_work_tree(dir: &Path) -> bool {
    git(dir, &["rev-parse", "--is-inside-work-tree"])
        .is_ok_and(|output| output.status.success() && output.stdout.starts_with(b"true"))
}

fn git(dir: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        // Never act on a repository chosen by the caller's environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
}

fn run(dir: &Path, args: &[&str]) -> Result<(), String> {
    let output = git(dir, args).map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fm-xml-export-exploder-git-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Repository-local identity, so tests don't depend on the machine's git config
    fn init_with_identity(dir: &Path) {
        run(dir, &["init", "--quiet"]).unwrap();
        run(dir, &["config", "user.name", "Test"]).unwrap();
        run(dir, &["config", "user.email", "test@example.com"]).unwrap();
        run(dir, &["config", "commit.gpgsign", "false"]).unwrap();
    }

    fn commit_count(dir: &Path) -> usize {
        let output = git(dir, &["rev-list", "--count", "HEAD"]).unwrap();
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(0)
    }

    #[test]
    fn commits_only_when_something_changed() {
        if !is_git_available() {
            return;
        }
        let dir = temp_dir("changes");
        init_with_identity(&dir);

        fs::write(dir.join("script.txt"), "Set Variable [ $a ; 1 ]").unwrap();
        assert_eq!(commit(&dir, "Export DB"), Ok(CommitResult::Committed));
        assert_eq!(commit(&dir, "Export DB"), Ok(CommitResult::NoChanges));

        fs::write(dir.join("script.txt"), "Set Variable [ $a ; 2 ]").unwrap();
        assert_eq!(commit(&dir, "Export DB"), Ok(CommitResult::Committed));
        assert_eq!(commit_count(&dir), 2);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn leaves_folders_inside_other_repositories_alone() {
        if !is_git_available() {
            return;
        }
        let parent = temp_dir("parent");
        init_with_identity(&parent);
        let output = parent.join("MyDatabase");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("script.txt"), "Beep").unwrap();

        assert_eq!(
            commit(&output, "Export MyDatabase"),
            Ok(CommitResult::InOtherRepository)
        );
        assert!(!output.join(".git").exists());

        fs::remove_dir_all(&parent).unwrap();
    }

    #[test]
    fn initializes_new_repositories() {
        if !is_git_available() {
            return;
        }
        let dir = temp_dir("init");
        fs::write(dir.join("script.txt"), "Beep").unwrap();

        // The commit itself may fail on machines without a git identity
        let _ = commit(&dir, "Export DB");
        assert!(dir.join(".git").is_dir());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn one_repository_per_db_or_domain_folder() {
        let root = temp_dir("repositories");
        for path in ["scripts/A", "scripts/B", "tables/A", "themes/C"] {
            fs::create_dir_all(root.join(path)).unwrap();
        }
        let db_names = vec!["A".to_string(), "B".to_string()];

        let by_db = repositories(&root, &OutputTree::Db, &db_names);
        assert_eq!(
            by_db.keys().cloned().collect::<Vec<_>>(),
            vec![root.join("A"), root.join("B")]
        );

        let by_domain = repositories(&root, &OutputTree::Domain, &db_names);
        assert_eq!(by_domain[&root.join("scripts")], ["A", "B"]);
        assert_eq!(by_domain[&root.join("tables")], ["A"]);
        assert!(!by_domain.contains_key(&root.join("themes")));

        fs::remove_dir_all(&root).unwrap();
    }
}
