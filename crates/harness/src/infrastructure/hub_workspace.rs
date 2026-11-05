use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::application::{HubWorkspacePort, MemberCheckout, PortError};

#[derive(Clone, Copy, Default)]
pub struct FileSystemHubWorkspace;

impl HubWorkspacePort for FileSystemHubWorkspace {
    fn read_hub_map(&self, hub: &Path) -> Result<String, PortError> {
        let path = hub.join("docs/README.md");
        fs::read_to_string(&path)
            .map_err(|error| PortError::new(format!("{}: {error}", path.display())))
    }

    fn resolve_member(&self, hub: &Path, location: &str) -> Result<MemberCheckout, PortError> {
        let hub = canonical(hub)?;
        let candidate = hub.join(location);
        if !candidate.is_dir() {
            return Err(PortError::new(format!(
                "{location} is not checked out next to the hub ({} does not exist)",
                candidate.display()
            )));
        }
        let root = canonical(&candidate)?;
        if root == hub {
            return Err(PortError::new(format!("{location} is the hub itself")));
        }
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(&root)
            .output()
            .map_err(|error| PortError::new(format!("git: {error}")))?;
        let toplevel = String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned();
        if !output.status.success() || canonical(Path::new(&toplevel))? != root {
            return Err(PortError::new(format!(
                "{location} is not the root of a Git repository"
            )));
        }
        Ok(MemberCheckout {
            hub_from_member: relative(&root, &hub),
            root,
        })
    }

    fn read_claude(&self, member: &Path) -> Result<Option<String>, PortError> {
        let path = claude_path(member)?;
        match fs::read_to_string(&path) {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(PortError::new(format!("{}: {error}", path.display()))),
        }
    }

    fn write_claude(&self, member: &Path, content: &str) -> Result<(), PortError> {
        let path = claude_path(member)?;
        let temp = member.join(".CLAUDE.md.harness-hub.tmp");
        fs::write(&temp, content)
            .and_then(|_| fs::rename(&temp, &path))
            .map_err(|error| {
                let _ = fs::remove_file(&temp);
                PortError::new(format!("{}: {error}", path.display()))
            })
    }
}

/// Refuse a symlinked CLAUDE.md so a write cannot land outside the member.
fn claude_path(member: &Path) -> Result<PathBuf, PortError> {
    let path = member.join("CLAUDE.md");
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(PortError::new(format!(
            "refusing symlink: {}",
            path.display()
        ))),
        Ok(metadata) if !metadata.is_file() => Err(PortError::new(format!(
            "{} is not a regular file",
            path.display()
        ))),
        _ => Ok(path),
    }
}

fn canonical(path: &Path) -> Result<PathBuf, PortError> {
    path.canonicalize()
        .map_err(|error| PortError::new(format!("{}: {error}", path.display())))
}

/// `to` as seen from `from`; both are canonical absolute paths.
fn relative(from: &Path, to: &Path) -> String {
    let from = from.components().collect::<Vec<Component>>();
    let to = to.components().collect::<Vec<Component>>();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let parts = std::iter::repeat_n("..".to_owned(), from.len() - common)
        .chain(
            to[common..]
                .iter()
                .map(|part| part.as_os_str().to_string_lossy().into_owned()),
        )
        .collect::<Vec<_>>();
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_between_siblings_and_nested_dirs() {
        assert_eq!(
            relative(Path::new("/w/orders"), Path::new("/w/hub")),
            "../hub"
        );
        assert_eq!(
            relative(Path::new("/w/team/orders"), Path::new("/w/hub")),
            "../../hub"
        );
    }

    #[test]
    fn rejects_non_git_directories_and_symlinked_claude() {
        let workspace = tempfile::tempdir().unwrap();
        let hub = workspace.path().join("hub");
        let plain = workspace.path().join("plain");
        fs::create_dir_all(&hub).unwrap();
        fs::create_dir_all(&plain).unwrap();
        let error = FileSystemHubWorkspace
            .resolve_member(&hub, "../plain")
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("not the root of a Git repository"));
        assert!(FileSystemHubWorkspace
            .resolve_member(&hub, "../absent")
            .is_err());

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/etc/hosts", plain.join("CLAUDE.md")).unwrap();
            assert!(FileSystemHubWorkspace.read_claude(&plain).is_err());
            assert!(FileSystemHubWorkspace.write_claude(&plain, "x").is_err());
        }
    }
}
