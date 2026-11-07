use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::application::{
    HubWorkspacePort, MemberCheckout, MemberSource, PortError, WorkspaceCheckout,
};

#[derive(Clone, Copy, Default)]
pub struct FileSystemHubWorkspace;

impl HubWorkspacePort for FileSystemHubWorkspace {
    fn read_hub_map(&self, hub: &Path) -> Result<String, PortError> {
        let path = hub.join("docs/README.md");
        fs::read_to_string(&path)
            .map_err(|error| PortError::new(format!("{}: {error}", path.display())))
    }

    fn has_member_docs(&self, hub: &Path, member: &str) -> Result<bool, PortError> {
        Ok(hub.join("docs/services").join(member).is_dir())
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

    fn resolve_workspace(&self, hub: &Path) -> Result<WorkspaceCheckout, PortError> {
        let hub = canonical(hub)?;
        let root = hub
            .parent()
            .ok_or_else(|| PortError::new("the hub has no parent directory"))?
            .to_path_buf();
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(&root)
            .output()
            .map_err(|error| PortError::new(format!("git: {error}")))?;
        if output.status.success() {
            return Err(PortError::new(format!(
                "{} is inside the Git repository {}; run `harness hub sync` without --workspace",
                root.display(),
                String::from_utf8_lossy(&output.stdout).trim_end()
            )));
        }
        Ok(WorkspaceCheckout {
            hub_path: hub.display().to_string(),
            root,
        })
    }

    fn workspace_repositories(&self, root: &Path) -> Result<Vec<String>, PortError> {
        let mut names = Vec::new();
        for entry in fs::read_dir(root)
            .map_err(|error| PortError::new(format!("{}: {error}", root.display())))?
        {
            let path = entry
                .map_err(|error| PortError::new(error.to_string()))?
                .path();
            // Hidden directories hold worktrees and tool state, not members.
            let hidden = path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with('.'));
            if hidden || !path.join(".git").exists() || !path.is_dir() {
                continue;
            }
            let toplevel = git(&path, &["rev-parse", "--show-toplevel"]);
            if toplevel.and_then(|top| canonical(Path::new(&top)).ok()) == canonical(&path).ok() {
                if let Some(name) = path.file_name() {
                    names.push(name.to_string_lossy().into_owned());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    fn knowledge_pages(&self, hub: &Path) -> Result<Vec<(String, String)>, PortError> {
        let mut pages = Vec::new();
        let mut pending = vec![hub.join("docs")];
        while let Some(dir) = pending.pop() {
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(PortError::new(format!("{}: {error}", dir.display()))),
            };
            for entry in entries {
                let entry = entry.map_err(|error| PortError::new(error.to_string()))?;
                let kind = entry
                    .file_type()
                    .map_err(|error| PortError::new(error.to_string()))?;
                let path = entry.path();
                if kind.is_dir() {
                    pending.push(path);
                } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                    let text = fs::read_to_string(&path)
                        .map_err(|error| PortError::new(format!("{}: {error}", path.display())))?;
                    let relative = path.strip_prefix(hub).unwrap_or(&path);
                    pages.push((relative.to_string_lossy().replace('\\', "/"), text));
                }
            }
        }
        pages.sort();
        Ok(pages)
    }

    fn member_source(
        &self,
        member: &Path,
        revision: &str,
        path: &str,
    ) -> Result<MemberSource, PortError> {
        let show = |spec: String| {
            Command::new("git")
                .args(["show", &spec])
                .current_dir(member)
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| output.stdout)
        };
        Ok((
            show(format!("{revision}:{path}")),
            show(format!("HEAD:{path}")),
        ))
    }

    fn read_entry(&self, member: &Path, entry: &str) -> Result<Option<String>, PortError> {
        let path = entry_path(member, entry)?;
        match fs::read_to_string(&path) {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(PortError::new(format!("{}: {error}", path.display()))),
        }
    }

    fn write_entry(&self, member: &Path, entry: &str, content: &str) -> Result<(), PortError> {
        let path = entry_path(member, entry)?;
        let temp = member.join(format!(".{entry}.harness-hub.tmp"));
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| PortError::new(format!("{}: {error}", temp.display())))?;
        file.write_all(content.as_bytes())
            .and_then(|_| fs::rename(&temp, &path))
            .map_err(|error| {
                let _ = fs::remove_file(&temp);
                PortError::new(format!("{}: {error}", path.display()))
            })
    }
}

/// Refuse symlinked instruction files so a write cannot land outside the member.
fn entry_path(member: &Path, entry: &str) -> Result<PathBuf, PortError> {
    if !matches!(entry, "CLAUDE.md" | "AGENTS.md") {
        return Err(PortError::new("unsupported agent entrypoint"));
    }
    let path = member.join(entry);
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

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned()
    })
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

        let error = FileSystemHubWorkspace
            .resolve_workspace(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
            .unwrap_err();
        assert!(error.to_string().contains("without --workspace"));

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/etc/hosts", plain.join("CLAUDE.md")).unwrap();
            assert!(FileSystemHubWorkspace
                .read_entry(&plain, "CLAUDE.md")
                .is_err());
            assert!(FileSystemHubWorkspace
                .write_entry(&plain, "CLAUDE.md", "x")
                .is_err());
        }
    }
}
