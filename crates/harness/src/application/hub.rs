//! `harness hub sync`: write or check the pointer block in each member
//! repository listed by the hub (decision 0036).

use std::path::{Path, PathBuf};

use crate::application::PortError;
use crate::domain::{
    apply_hub_block, assess, parse_hub_members, render_hub_block, render_workspace_block,
    source_markers, Freshness, HubMapError,
};

/// A member repository resolved on disk.
#[derive(Clone, Debug)]
pub struct MemberCheckout {
    pub root: PathBuf,
    /// The hub as seen from the member, for example `../system-knowledge`.
    pub hub_from_member: String,
}

/// The directory holding the hub and its workspace instruction files.
#[derive(Clone, Debug)]
pub struct WorkspaceCheckout {
    pub root: PathBuf,
    /// The hub's absolute path.
    pub hub_path: String,
}

/// A member file at the pinned revision and at HEAD; `None` where unreadable.
pub type MemberSource = (Option<Vec<u8>>, Option<Vec<u8>>);

pub trait HubWorkspacePort {
    /// Contents of the hub's `docs/README.md`.
    fn read_hub_map(&self, hub: &Path) -> Result<String, PortError>;
    /// Whether the hub has `docs/services/<member>/`, which the pointer names.
    fn has_member_docs(&self, hub: &Path, member: &str) -> Result<bool, PortError>;
    /// Resolve `location` (relative to the hub) to a Git repository root.
    fn resolve_member(&self, hub: &Path, location: &str) -> Result<MemberCheckout, PortError>;
    /// The hub's parent directory, which must not be inside a Git repository.
    fn resolve_workspace(&self, hub: &Path) -> Result<WorkspaceCheckout, PortError>;
    /// Git repository roots directly inside the workspace directory, by name.
    fn workspace_repositories(&self, root: &Path) -> Result<Vec<String>, PortError>;
    /// Markdown pages under the hub's `docs/`, as (hub-relative path, text).
    fn knowledge_pages(&self, hub: &Path) -> Result<Vec<(String, String)>, PortError>;
    /// A member file at `revision` and at HEAD; `None` where it cannot be read.
    fn member_source(
        &self,
        member: &Path,
        revision: &str,
        path: &str,
    ) -> Result<MemberSource, PortError>;
    fn read_entry(&self, member: &Path, entry: &str) -> Result<Option<String>, PortError>;
    fn write_entry(&self, member: &Path, entry: &str, content: &str) -> Result<(), PortError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HubMode {
    Apply,
    DryRun,
    Check,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberOutcome {
    Current,
    Updated,
    WouldUpdate,
    /// Reachable below the workspace, so the workspace block covers it.
    Covered,
    /// The member could not be synced or checked; the reason is attached.
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct MemberReport {
    pub name: String,
    pub location: String,
    pub outcome: MemberOutcome,
}

#[derive(Clone, Debug)]
pub struct WorkspaceReport {
    pub path: String,
    pub outcome: MemberOutcome,
}

#[derive(Clone, Debug)]
pub struct HubReport {
    pub mode: HubMode,
    pub members: Vec<MemberReport>,
    /// Set by `--workspace`, which writes the workspace file instead of members.
    pub workspace: Option<WorkspaceReport>,
    /// Git repositories in the workspace that Related Repositories does not
    /// list; reported so a missing member is never silent.
    pub unlisted: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SourceCheck {
    pub page: String,
    pub line: usize,
    /// `repository:path#Lstart-Lend`, or the raw marker when it is malformed.
    pub source: String,
    pub outcome: Result<Freshness, String>,
}

#[derive(Clone, Debug)]
pub struct FreshnessReport {
    pub pages: usize,
    pub sources: Vec<SourceCheck>,
}

impl FreshnessReport {
    /// Moved lines still say the same thing; only changed or unreadable fail.
    pub fn ok(&self) -> bool {
        self.sources.iter().all(|check| {
            matches!(
                check.outcome,
                Ok(Freshness::Current) | Ok(Freshness::Moved(_))
            )
        })
    }
}

impl HubReport {
    /// Check fails on anything not current; apply and dry-run fail only on errors.
    pub fn ok(&self) -> bool {
        let ok = |outcome: &MemberOutcome| match outcome {
            MemberOutcome::Current | MemberOutcome::Updated | MemberOutcome::Covered => true,
            MemberOutcome::WouldUpdate => self.mode == HubMode::DryRun,
            MemberOutcome::Failed(_) => false,
        };
        self.members.iter().all(|member| ok(&member.outcome))
            && self
                .workspace
                .as_ref()
                .is_none_or(|workspace| ok(&workspace.outcome))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HubError {
    #[error("{0}")]
    Map(#[from] HubMapError),
    #[error(transparent)]
    Port(#[from] PortError),
}

pub struct HubApplication<W> {
    workspace: W,
}

impl<W: HubWorkspacePort> HubApplication<W> {
    pub fn new(workspace: W) -> Self {
        Self { workspace }
    }

    pub fn sync(&self, hub: &Path, mode: HubMode) -> Result<HubReport, HubError> {
        let members = parse_hub_members(&self.workspace.read_hub_map(hub)?)?;
        let members = members
            .into_iter()
            .map(|member| {
                let outcome = self
                    .sync_member(hub, &member.name, &member.location, mode)
                    .unwrap_or_else(|error| MemberOutcome::Failed(error.to_string()));
                MemberReport {
                    name: member.name,
                    location: member.location,
                    outcome,
                }
            })
            .collect();
        Ok(HubReport {
            mode,
            members,
            workspace: None,
            unlisted: Vec::new(),
        })
    }

    /// Write workspace instruction blocks and member Codex pointers, since
    /// Codex discovery stops at each member Git root.
    pub fn sync_workspace(&self, hub: &Path, mode: HubMode) -> Result<HubReport, HubError> {
        let members = parse_hub_members(&self.workspace.read_hub_map(hub)?)?;
        let workspace = self.workspace.resolve_workspace(hub)?;
        let members: Vec<MemberReport> = members
            .into_iter()
            .map(|member| {
                let outcome = self
                    .check_covered(hub, &workspace.root, &member.name, &member.location, mode)
                    .unwrap_or_else(|error| MemberOutcome::Failed(error.to_string()));
                MemberReport {
                    name: member.name,
                    location: member.location,
                    outcome,
                }
            })
            .collect();
        let outcome = self
            .write_block(
                &workspace.root,
                &render_workspace_block(&workspace.hub_path),
                mode,
            )
            .unwrap_or_else(|error| MemberOutcome::Failed(error.to_string()));
        let hub_name = hub_dir_name(&workspace.hub_path);
        let listed = members
            .iter()
            .map(|member| last_segment(&member.location))
            .collect::<Vec<_>>();
        let unlisted = self
            .workspace
            .workspace_repositories(&workspace.root)?
            .into_iter()
            .filter(|name| *name != hub_name && !listed.contains(name))
            .collect();
        Ok(HubReport {
            mode,
            members,
            unlisted,
            workspace: Some(WorkspaceReport {
                path: workspace.root.display().to_string(),
                outcome,
            }),
        })
    }

    /// Check every source marker in the hub's pages against member HEADs.
    pub fn check_fresh(&self, hub: &Path) -> Result<FreshnessReport, HubError> {
        let members = parse_hub_members(&self.workspace.read_hub_map(hub)?)?;
        let pages = self.workspace.knowledge_pages(hub)?;
        let mut sources = Vec::new();
        for (page, text) in &pages {
            for (line, marker) in source_markers(text) {
                let check = |source: String, outcome| SourceCheck {
                    page: page.clone(),
                    line,
                    source,
                    outcome,
                };
                let marker = match marker {
                    Ok(marker) => marker,
                    Err(reason) => {
                        sources.push(check(
                            text.lines().nth(line - 1).unwrap_or("").trim().to_owned(),
                            Err(reason),
                        ));
                        continue;
                    }
                };
                let label = format!(
                    "{}:{}#L{}-L{}",
                    marker.repository, marker.path, marker.start_line, marker.end_line
                );
                let outcome = members
                    .iter()
                    .find(|member| member.name == marker.repository)
                    .ok_or_else(|| {
                        format!(
                            "{} is not listed under Related Repositories",
                            marker.repository
                        )
                    })
                    .and_then(|member| {
                        let checkout = self
                            .workspace
                            .resolve_member(hub, &member.location)
                            .map_err(|error| error.to_string())?;
                        let (pinned, head) = self
                            .workspace
                            .member_source(&checkout.root, &marker.revision, &marker.path)
                            .map_err(|error| error.to_string())?;
                        Ok(assess(&marker, pinned.as_deref(), head.as_deref()))
                    });
                sources.push(check(label, outcome));
            }
        }
        Ok(FreshnessReport {
            pages: pages.len(),
            sources,
        })
    }

    fn check_covered(
        &self,
        hub: &Path,
        root: &Path,
        name: &str,
        location: &str,
        mode: HubMode,
    ) -> Result<MemberOutcome, HubError> {
        if let Some(missing) = self.missing_member_docs(hub, name)? {
            return Ok(missing);
        }
        let checkout = self.workspace.resolve_member(hub, location)?;
        if !checkout.root.starts_with(root) {
            return Ok(MemberOutcome::Failed(format!(
                "{location} is outside {}, so the workspace block does not reach it",
                root.display()
            )));
        }
        // Codex starts instruction discovery at the member Git root, so a
        // parent workspace file alone cannot cover a session launched there.
        let outcome = self.write_entries(
            &checkout.root,
            &render_hub_block(&checkout.hub_from_member, name),
            mode,
            &["AGENTS.md"],
        )?;
        Ok(if outcome == MemberOutcome::Current {
            MemberOutcome::Covered
        } else {
            outcome
        })
    }

    fn missing_member_docs(
        &self,
        hub: &Path,
        name: &str,
    ) -> Result<Option<MemberOutcome>, HubError> {
        Ok((!self.workspace.has_member_docs(hub, name)?).then(|| {
            MemberOutcome::Failed(format!(
                "the hub has no docs/services/{name}/ for the pointer to name; create it in the hub first"
            ))
        }))
    }

    fn write_block(
        &self,
        root: &Path,
        block: &str,
        mode: HubMode,
    ) -> Result<MemberOutcome, HubError> {
        self.write_entries(root, block, mode, &["CLAUDE.md", "AGENTS.md"])
    }

    fn write_entries(
        &self,
        root: &Path,
        block: &str,
        mode: HubMode,
        entries: &[&str],
    ) -> Result<MemberOutcome, HubError> {
        // Validate every selected entrypoint before writing any of them.
        let mut changes = Vec::new();
        for &entry in entries {
            let existing = self.workspace.read_entry(root, entry)?;
            let wanted = apply_hub_block(existing.as_deref(), block)?;
            if existing.as_deref() != Some(wanted.as_str()) {
                changes.push((entry, wanted));
            }
        }
        if changes.is_empty() {
            return Ok(MemberOutcome::Current);
        }
        Ok(match mode {
            HubMode::Apply => {
                for (entry, wanted) in changes {
                    self.workspace.write_entry(root, entry, &wanted)?;
                }
                MemberOutcome::Updated
            }
            HubMode::DryRun | HubMode::Check => MemberOutcome::WouldUpdate,
        })
    }

    fn sync_member(
        &self,
        hub: &Path,
        name: &str,
        location: &str,
        mode: HubMode,
    ) -> Result<MemberOutcome, HubError> {
        if let Some(missing) = self.missing_member_docs(hub, name)? {
            return Ok(missing);
        }
        let checkout = self.workspace.resolve_member(hub, location)?;
        let block = render_hub_block(&checkout.hub_from_member, name);
        self.write_block(&checkout.root, &block, mode)
    }
}

fn last_segment(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
        .to_owned()
}

fn hub_dir_name(hub_path: &str) -> String {
    last_segment(hub_path)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use super::*;

    struct Fixture {
        map: String,
        claude: RefCell<BTreeMap<PathBuf, String>>,
        pages: Vec<(String, String)>,
    }

    impl HubWorkspacePort for Fixture {
        fn read_hub_map(&self, _hub: &Path) -> Result<String, PortError> {
            Ok(self.map.clone())
        }
        fn has_member_docs(&self, _hub: &Path, member: &str) -> Result<bool, PortError> {
            Ok(member != "undocumented")
        }
        fn resolve_member(&self, _hub: &Path, location: &str) -> Result<MemberCheckout, PortError> {
            if location.contains("missing") {
                return Err(PortError::new(format!("{location} is not checked out")));
            }
            Ok(MemberCheckout {
                root: PathBuf::from("/w").join(location.trim_start_matches("../")),
                hub_from_member: "../hub".to_owned(),
            })
        }
        fn workspace_repositories(&self, _root: &Path) -> Result<Vec<String>, PortError> {
            Ok(vec!["hub".into(), "orders".into(), "stray".into()])
        }
        fn knowledge_pages(&self, _hub: &Path) -> Result<Vec<(String, String)>, PortError> {
            Ok(self.pages.clone())
        }
        fn member_source(
            &self,
            _member: &Path,
            _revision: &str,
            path: &str,
        ) -> Result<MemberSource, PortError> {
            Ok(match path {
                "src/Same.java" => (Some(b"a\nb\n".to_vec()), Some(b"a\nb\n".to_vec())),
                _ => (Some(b"a\nb\n".to_vec()), Some(b"a\nc\n".to_vec())),
            })
        }
        fn resolve_workspace(&self, _hub: &Path) -> Result<WorkspaceCheckout, PortError> {
            Ok(WorkspaceCheckout {
                root: PathBuf::from("/w"),
                hub_path: "/w/hub".to_owned(),
            })
        }
        fn read_entry(&self, member: &Path, entry: &str) -> Result<Option<String>, PortError> {
            let key = if entry == "CLAUDE.md" {
                member.to_path_buf()
            } else {
                member.join(entry)
            };
            Ok(self.claude.borrow().get(&key).cloned())
        }
        fn write_entry(&self, member: &Path, entry: &str, content: &str) -> Result<(), PortError> {
            self.claude.borrow_mut().insert(
                if entry == "CLAUDE.md" {
                    member.to_path_buf()
                } else {
                    member.join(entry)
                },
                content.to_owned(),
            );
            Ok(())
        }
    }

    fn fixture() -> Fixture {
        Fixture {
            map:
                "## Related Repositories\n\n| Repository | Role | Location |\n| --- | --- | --- |\n\
| orders | service | ../orders |\n| gone | service | ../missing |\n\
| undocumented | service | ../undocumented |\n"
                    .to_owned(),
            pages: Vec::new(),
            claude: RefCell::new(BTreeMap::from([(
                PathBuf::from("/w/orders"),
                "# Orders\n".to_owned(),
            )])),
        }
    }

    #[test]
    fn check_reports_stale_then_apply_writes_then_check_passes() {
        let app = HubApplication::new(fixture());
        let hub = Path::new("hub");

        let check = app.sync(hub, HubMode::Check).unwrap();
        assert_eq!(check.members[0].outcome, MemberOutcome::WouldUpdate);
        assert!(matches!(check.members[1].outcome, MemberOutcome::Failed(_)));
        assert!(matches!(
            &check.members[2].outcome,
            MemberOutcome::Failed(reason) if reason.contains("docs/services/undocumented/")
        ));
        assert!(!check.ok());

        let dry = app.sync(hub, HubMode::DryRun).unwrap();
        assert_eq!(dry.members[0].outcome, MemberOutcome::WouldUpdate);
        assert_eq!(
            app.workspace.claude.borrow()[Path::new("/w/orders")],
            "# Orders\n"
        );

        let apply = app.sync(hub, HubMode::Apply).unwrap();
        assert_eq!(apply.members[0].outcome, MemberOutcome::Updated);
        let written = app.workspace.claude.borrow()[Path::new("/w/orders")].clone();
        assert!(written.starts_with("# Orders\n\n<!-- HARNESS-HUB:BEGIN -->"));
        assert!(written.contains("`../hub/docs/services/orders/`"));

        let again = app.sync(hub, HubMode::Check).unwrap();
        assert_eq!(again.members[0].outcome, MemberOutcome::Current);
    }

    #[test]
    fn workspace_mode_writes_only_the_workspace_file_and_checks_members() {
        let app = HubApplication::new(fixture());
        let hub = Path::new("hub");

        let check = app.sync_workspace(hub, HubMode::Check).unwrap();
        assert_eq!(check.members[0].outcome, MemberOutcome::WouldUpdate);
        assert!(matches!(check.members[1].outcome, MemberOutcome::Failed(_)));
        assert!(matches!(check.members[2].outcome, MemberOutcome::Failed(_)));
        let workspace = check.workspace.as_ref().unwrap();
        assert_eq!(workspace.outcome, MemberOutcome::WouldUpdate);
        assert!(!app.workspace.claude.borrow().contains_key(Path::new("/w")));

        let apply = app.sync_workspace(hub, HubMode::Apply).unwrap();
        assert_eq!(apply.workspace.unwrap().outcome, MemberOutcome::Updated);
        let claude = app.workspace.claude.borrow();
        assert!(claude[Path::new("/w")].contains("`/w/hub/docs/services/<repository>/`"));
        assert_eq!(claude[Path::new("/w/orders")], "# Orders\n");
        assert_eq!(apply_unlisted(&app), vec!["stray".to_owned()]);
    }

    fn apply_unlisted(app: &HubApplication<Fixture>) -> Vec<String> {
        app.sync_workspace(Path::new("hub"), HubMode::Check)
            .unwrap()
            .unlisted
    }

    #[test]
    fn check_fresh_reports_current_stale_unlisted_and_malformed_sources() {
        let marker = |repository: &str, path: &str| {
            format!(
                "<!-- harness-source: {repository}@{}:{path}#L2-L2 sha256={} -->",
                "a".repeat(40),
                crate::domain::sha256_hex(b"b\n")
            )
        };
        let mut fixture = fixture();
        fixture.pages = vec![(
            "docs/services/orders/README.md".into(),
            format!(
                "claim\n{}\n{}\n{}\n<!-- harness-source: broken -->\n",
                marker("orders", "src/Same.java"),
                marker("orders", "src/Changed.java"),
                marker("ghost", "src/Same.java"),
            ),
        )];
        let report = HubApplication::new(fixture)
            .check_fresh(Path::new("hub"))
            .unwrap();
        let outcomes = report
            .sources
            .iter()
            .map(|check| match &check.outcome {
                Ok(Freshness::Current) => "current",
                Ok(Freshness::Moved(_)) => "moved",
                Ok(Freshness::Stale(_)) => "stale",
                Err(_) => "error",
            })
            .collect::<Vec<_>>();
        assert_eq!(outcomes, ["current", "stale", "error", "error"]);
        assert_eq!(report.sources[0].line, 2);
        assert!(!report.ok());
    }
}
