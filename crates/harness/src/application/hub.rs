//! `harness hub sync`: write or check the pointer block in each member
//! repository listed by the hub (decision 0036).

use std::path::{Path, PathBuf};

use crate::application::PortError;
use crate::domain::{
    apply_hub_block, parse_hub_members, render_hub_block, render_workspace_block, HubMapError,
};

/// A member repository resolved on disk.
#[derive(Clone, Debug)]
pub struct MemberCheckout {
    pub root: PathBuf,
    /// The hub as seen from the member, for example `../system-knowledge`.
    pub hub_from_member: String,
}

/// The directory holding the hub, whose `CLAUDE.md` Claude Code loads in every
/// repository below it.
#[derive(Clone, Debug)]
pub struct WorkspaceCheckout {
    pub root: PathBuf,
    /// The hub's absolute path.
    pub hub_path: String,
}

pub trait HubWorkspacePort {
    /// Contents of the hub's `docs/README.md`.
    fn read_hub_map(&self, hub: &Path) -> Result<String, PortError>;
    /// Whether the hub has `docs/services/<member>/`, which the pointer names.
    fn has_member_docs(&self, hub: &Path, member: &str) -> Result<bool, PortError>;
    /// Resolve `location` (relative to the hub) to a Git repository root.
    fn resolve_member(&self, hub: &Path, location: &str) -> Result<MemberCheckout, PortError>;
    /// The hub's parent directory, which must not be inside a Git repository.
    fn resolve_workspace(&self, hub: &Path) -> Result<WorkspaceCheckout, PortError>;
    fn read_claude(&self, member: &Path) -> Result<Option<String>, PortError>;
    fn write_claude(&self, member: &Path, content: &str) -> Result<(), PortError>;
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
        })
    }

    /// Write one block into the workspace directory's `CLAUDE.md` and check
    /// that every member is a Git root below it; members are not written.
    pub fn sync_workspace(&self, hub: &Path, mode: HubMode) -> Result<HubReport, HubError> {
        let members = parse_hub_members(&self.workspace.read_hub_map(hub)?)?;
        let workspace = self.workspace.resolve_workspace(hub)?;
        let members = members
            .into_iter()
            .map(|member| {
                let outcome = self
                    .check_covered(hub, &workspace.root, &member.name, &member.location)
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
        Ok(HubReport {
            mode,
            members,
            workspace: Some(WorkspaceReport {
                path: workspace.root.join("CLAUDE.md").display().to_string(),
                outcome,
            }),
        })
    }

    fn check_covered(
        &self,
        hub: &Path,
        root: &Path,
        name: &str,
        location: &str,
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
        Ok(MemberOutcome::Covered)
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
        let existing = self.workspace.read_claude(root)?;
        let wanted = apply_hub_block(existing.as_deref(), block)?;
        if existing.as_deref() == Some(wanted.as_str()) {
            return Ok(MemberOutcome::Current);
        }
        Ok(match mode {
            HubMode::Apply => {
                self.workspace.write_claude(root, &wanted)?;
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use super::*;

    struct Fixture {
        map: String,
        claude: RefCell<BTreeMap<PathBuf, String>>,
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
        fn resolve_workspace(&self, _hub: &Path) -> Result<WorkspaceCheckout, PortError> {
            Ok(WorkspaceCheckout {
                root: PathBuf::from("/w"),
                hub_path: "/w/hub".to_owned(),
            })
        }
        fn read_claude(&self, member: &Path) -> Result<Option<String>, PortError> {
            Ok(self.claude.borrow().get(member).cloned())
        }
        fn write_claude(&self, member: &Path, content: &str) -> Result<(), PortError> {
            self.claude
                .borrow_mut()
                .insert(member.to_path_buf(), content.to_owned());
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
        assert_eq!(check.members[0].outcome, MemberOutcome::Covered);
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
    }
}
