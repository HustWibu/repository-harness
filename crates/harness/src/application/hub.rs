//! `harness hub sync`: write or check the pointer block in each member
//! repository listed by the hub (decision 0036).

use std::path::{Path, PathBuf};

use crate::application::PortError;
use crate::domain::{apply_hub_block, parse_hub_members, render_hub_block, HubMapError};

/// A member repository resolved on disk.
#[derive(Clone, Debug)]
pub struct MemberCheckout {
    pub root: PathBuf,
    /// The hub as seen from the member, for example `../system-knowledge`.
    pub hub_from_member: String,
}

pub trait HubWorkspacePort {
    /// Contents of the hub's `docs/README.md`.
    fn read_hub_map(&self, hub: &Path) -> Result<String, PortError>;
    /// Resolve `location` (relative to the hub) to a Git repository root.
    fn resolve_member(&self, hub: &Path, location: &str) -> Result<MemberCheckout, PortError>;
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
pub struct HubReport {
    pub mode: HubMode,
    pub members: Vec<MemberReport>,
}

impl HubReport {
    /// Check fails on anything not current; apply and dry-run fail only on errors.
    pub fn ok(&self) -> bool {
        self.members.iter().all(|member| match &member.outcome {
            MemberOutcome::Current | MemberOutcome::Updated => true,
            MemberOutcome::WouldUpdate => self.mode == HubMode::DryRun,
            MemberOutcome::Failed(_) => false,
        })
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
        Ok(HubReport { mode, members })
    }

    fn sync_member(
        &self,
        hub: &Path,
        name: &str,
        location: &str,
        mode: HubMode,
    ) -> Result<MemberOutcome, HubError> {
        let checkout = self.workspace.resolve_member(hub, location)?;
        let existing = self.workspace.read_claude(&checkout.root)?;
        let block = render_hub_block(&checkout.hub_from_member, name);
        let wanted = apply_hub_block(existing.as_deref(), &block)?;
        if existing.as_deref() == Some(wanted.as_str()) {
            return Ok(MemberOutcome::Current);
        }
        Ok(match mode {
            HubMode::Apply => {
                self.workspace.write_claude(&checkout.root, &wanted)?;
                MemberOutcome::Updated
            }
            HubMode::DryRun | HubMode::Check => MemberOutcome::WouldUpdate,
        })
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
        fn resolve_member(&self, _hub: &Path, location: &str) -> Result<MemberCheckout, PortError> {
            if location.contains("missing") {
                return Err(PortError::new(format!("{location} is not checked out")));
            }
            Ok(MemberCheckout {
                root: PathBuf::from(location),
                hub_from_member: "../hub".to_owned(),
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
| orders | service | ../orders |\n| gone | service | ../missing |\n"
                    .to_owned(),
            claude: RefCell::new(BTreeMap::from([(
                PathBuf::from("../orders"),
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
        assert!(!check.ok());

        let dry = app.sync(hub, HubMode::DryRun).unwrap();
        assert_eq!(dry.members[0].outcome, MemberOutcome::WouldUpdate);
        assert_eq!(
            app.workspace.claude.borrow()[Path::new("../orders")],
            "# Orders\n"
        );

        let apply = app.sync(hub, HubMode::Apply).unwrap();
        assert_eq!(apply.members[0].outcome, MemberOutcome::Updated);
        let written = app.workspace.claude.borrow()[Path::new("../orders")].clone();
        assert!(written.starts_with("# Orders\n\n<!-- HARNESS-HUB:BEGIN -->"));
        assert!(written.contains("`../hub/docs/services/orders/`"));

        let again = app.sync(hub, HubMode::Check).unwrap();
        assert_eq!(again.members[0].outcome, MemberOutcome::Current);
    }
}
