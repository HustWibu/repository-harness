//! Local-only installs list every Harness path in `.git/info/exclude` so the
//! core stays out of `git status`. The bootstrap creates the marked block; this
//! module only keeps an existing block current and never creates one.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::application::PortError;
use crate::domain::InstallationState;

pub const BEGIN: &str = "# BEGIN harness local-only (managed by harness; do not edit)";
pub const END: &str = "# END harness local-only";
/// Paths the bootstrap and maintenance CLI write besides the payload.
pub const RUNTIME_PATHS: [&str; 3] = [
    "/.harness-core/",
    "/.harness-backup/",
    "/scripts/bin/harness",
];

pub fn expected_lines(state: &InstallationState) -> Vec<String> {
    // shortcut: payload paths are plain names, so gitignore wildcards are not escaped; escape if a payload path ever contains * ? [ or \.
    RUNTIME_PATHS
        .iter()
        .map(|path| (*path).to_owned())
        .chain(
            state
                .files
                .iter()
                .map(|file| format!("/{}", file.path.as_str())),
        )
        .collect()
}

/// The lines inside the marked block, or `None` when this is not a
/// local-only install (no Git directory or no block).
pub fn read_block(root: &Path) -> Result<Option<Vec<String>>, PortError> {
    let Some(path) = exclude_path(root) else {
        return Ok(None);
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(None);
    };
    let mut lines = content.lines();
    if !lines.any(|line| line == BEGIN) {
        return Ok(None);
    }
    let block = lines
        .take_while(|line| *line != END)
        .map(str::to_owned)
        .collect();
    Ok(Some(block))
}

/// Rewrite an existing block to list exactly `state`'s paths; lines outside
/// the block are preserved byte for byte.
pub fn sync(root: &Path, state: &InstallationState) -> Result<(), PortError> {
    let Some(path) = exclude_path(root) else {
        return Ok(());
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(());
    };
    if !content.lines().any(|line| line == BEGIN) {
        return Ok(());
    }
    let mut output = String::new();
    let mut inside = false;
    for line in content.lines() {
        if line == BEGIN {
            inside = true;
            output.push_str(BEGIN);
            output.push('\n');
            for entry in expected_lines(state) {
                output.push_str(&entry);
                output.push('\n');
            }
            output.push_str(END);
            output.push('\n');
        } else if inside {
            inside = line != END;
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    if inside {
        return Err(PortError::new(format!(
            "{} has a Harness local-only block without its end marker",
            path.display()
        )));
    }
    if output != content {
        fs::write(&path, output).map_err(|error| PortError::new(error.to_string()))?;
    }
    Ok(())
}

fn exclude_path(root: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-path", "info/exclude"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let relative = String::from_utf8(output.stdout).ok()?;
    Some(root.join(relative.trim_end()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BaselineFile, ContentHash, RelativePath};

    fn state() -> InstallationState {
        InstallationState {
            schema_version: 1,
            core_version: "1.0.0".to_owned(),
            files: vec![BaselineFile {
                path: RelativePath::parse("docs/WORKFLOW.md").unwrap(),
                content: Vec::new(),
                hash: ContentHash::parse("0".repeat(64)).unwrap(),
            }],
        }
    }

    fn git_repo() -> tempfile::TempDir {
        let temp = tempfile::tempdir().unwrap();
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(temp.path())
            .status()
            .unwrap();
        assert!(status.success());
        temp
    }

    #[test]
    fn sync_fills_only_an_existing_block_and_keeps_user_lines() {
        let repo = git_repo();
        let exclude = repo.path().join(".git/info/exclude");
        fs::create_dir_all(exclude.parent().unwrap()).unwrap();
        fs::write(&exclude, "user-before\n").unwrap();
        sync(repo.path(), &state()).unwrap();
        assert_eq!(fs::read_to_string(&exclude).unwrap(), "user-before\n");
        assert_eq!(read_block(repo.path()).unwrap(), None);

        fs::write(
            &exclude,
            format!("user-before\n{BEGIN}\n/stale\n{END}\nuser-after\n"),
        )
        .unwrap();
        sync(repo.path(), &state()).unwrap();
        let content = fs::read_to_string(&exclude).unwrap();
        assert!(content.starts_with("user-before\n"));
        assert!(content.ends_with("user-after\n"));
        assert!(!content.contains("/stale"));
        assert_eq!(
            read_block(repo.path()).unwrap(),
            Some(expected_lines(&state()))
        );
    }

    #[test]
    fn sync_rejects_an_unterminated_block() {
        let repo = git_repo();
        let exclude = repo.path().join(".git/info/exclude");
        fs::create_dir_all(exclude.parent().unwrap()).unwrap();
        fs::write(&exclude, format!("{BEGIN}\n/stale\n")).unwrap();
        assert!(sync(repo.path(), &state()).is_err());
    }

    #[test]
    fn a_directory_outside_git_is_not_local_only() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(read_block(temp.path()).unwrap(), None);
        sync(temp.path(), &state()).unwrap();
    }
}
