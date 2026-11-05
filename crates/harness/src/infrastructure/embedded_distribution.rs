use sha2::{Digest, Sha256};

use crate::application::{CoreDistributionPort, PortError};
use crate::domain::{ContentHash, CoreDistribution, DistributionFile, RelativePath};

#[derive(Clone, Copy, Default)]
pub struct EmbeddedCoreDistribution;

impl CoreDistributionPort for EmbeddedCoreDistribution {
    fn current(&self) -> Result<CoreDistribution, PortError> {
        let mut files = Vec::new();
        let claude_block = include_bytes!("../../../../scripts/claude-harness-block.md");
        let mut claude = b"# Project Rules\n\n".to_vec();
        claude.extend_from_slice(claude_block);
        add(&mut files, "CLAUDE.md", &claude)?;
        add(
            &mut files,
            ".claude/skills/audit-onboarding-proposal/SKILL.md",
            include_bytes!("../../../../.claude/skills/audit-onboarding-proposal/SKILL.md"),
        )?;
        add(
            &mut files,
            ".claude/skills/audit-onboarding-proposal/scripts/validate_evidence_capsule.py",
            include_bytes!(
                "../../../../.claude/skills/audit-onboarding-proposal/scripts/validate_evidence_capsule.py"
            ),
        )?;
        add(
            &mut files,
            ".claude/skills/encode-invariant/SKILL.md",
            include_bytes!("../../../../.claude/skills/encode-invariant/SKILL.md"),
        )?;
        add(
            &mut files,
            ".claude/skills/improve-harness/SKILL.md",
            include_bytes!("../../../../.claude/skills/improve-harness/SKILL.md"),
        )?;
        add(
            &mut files,
            ".claude/skills/onboard-repository/SKILL.md",
            include_bytes!("../../../../.claude/skills/onboard-repository/SKILL.md"),
        )?;
        add(
            &mut files,
            ".claude/skills/onboard-repository/references/evidence-capsule-v1.md",
            include_bytes!(
                "../../../../.claude/skills/onboard-repository/references/evidence-capsule-v1.md"
            ),
        )?;
        add(
            &mut files,
            ".claude/skills/onboard-repository/references/evidence-capsule-v2.md",
            include_bytes!(
                "../../../../.claude/skills/onboard-repository/references/evidence-capsule-v2.md"
            ),
        )?;
        add(
            &mut files,
            ".claude/skills/onboard-repository/references/boundary-edges.md",
            include_bytes!(
                "../../../../.claude/skills/onboard-repository/references/boundary-edges.md"
            ),
        )?;
        add(
            &mut files,
            ".claude/skills/onboard-repository/scripts/emit_evidence_bundle.py",
            include_bytes!(
                "../../../../.claude/skills/onboard-repository/scripts/emit_evidence_bundle.py"
            ),
        )?;
        add(
            &mut files,
            ".claude/skills/onboard-repository/scripts/render_patch.py",
            include_bytes!("../../../../.claude/skills/onboard-repository/scripts/render_patch.py"),
        )?;
        add(
            &mut files,
            "docs/WORKFLOW.md",
            include_bytes!("../../../../docs/WORKFLOW.md"),
        )?;
        add(
            &mut files,
            "docs/README.md",
            include_bytes!("../../../../docs/README.md"),
        )?;
        add(
            &mut files,
            "docs/patterns/encoding-invariants.md",
            include_bytes!("../../../../docs/patterns/encoding-invariants.md"),
        )?;
        add(
            &mut files,
            "docs/product/README.md",
            include_bytes!("../../../../docs/product/README.md"),
        )?;
        add(
            &mut files,
            "docs/plans/README.md",
            include_bytes!("../../assets/docs/plans/README.md"),
        )?;
        add(
            &mut files,
            "docs/plans/active/README.md",
            include_bytes!("../../../../docs/plans/active/README.md"),
        )?;
        add(
            &mut files,
            "docs/plans/completed/README.md",
            include_bytes!("../../assets/docs/plans/completed/README.md"),
        )?;
        add(
            &mut files,
            "docs/decisions/README.md",
            include_bytes!("../../assets/docs/decisions/README.md"),
        )?;
        add(
            &mut files,
            "docs/templates/application-runbook.md",
            include_bytes!("../../../../docs/templates/application-runbook.md"),
        )?;
        add(
            &mut files,
            "docs/templates/decision.md",
            include_bytes!("../../../../docs/templates/decision.md"),
        )?;
        add(
            &mut files,
            "docs/templates/exec-plan.md",
            include_bytes!("../../../../docs/templates/exec-plan.md"),
        )?;
        add(
            &mut files,
            "docs/templates/harness-improvement.md",
            include_bytes!("../../../../docs/templates/harness-improvement.md"),
        )?;
        add(
            &mut files,
            "docs/templates/session-handoff.md",
            include_bytes!("../../../../docs/templates/session-handoff.md"),
        )?;
        add(
            &mut files,
            "docs/templates/evaluator-rubric.md",
            include_bytes!("../../../../docs/templates/evaluator-rubric.md"),
        )?;
        add(
            &mut files,
            "docs/templates/interface-contract.md",
            include_bytes!("../../../../docs/templates/interface-contract.md"),
        )?;
        Ok(CoreDistribution {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            files,
        })
    }
}

fn add(files: &mut Vec<DistributionFile>, path: &str, content: &[u8]) -> Result<(), PortError> {
    let path = RelativePath::parse(path).map_err(|error| PortError::new(error.to_string()))?;
    let hash = ContentHash::parse(format!("{:x}", Sha256::digest(content)))
        .map_err(|error| PortError::new(error.to_string()))?;
    files.push(DistributionFile {
        path,
        content: content.to_vec(),
        hash,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_payload_is_generic_and_complete() {
        let distribution = EmbeddedCoreDistribution.current().unwrap();
        distribution.validate().unwrap();
        let embedded_paths = distribution
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let manifest_paths = include_str!("../../../../scripts/harness-install-files.txt")
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(embedded_paths, manifest_paths);
        let claude = distribution
            .files
            .iter()
            .find(|file| file.path.as_str() == "CLAUDE.md")
            .unwrap();
        let claude = String::from_utf8(claude.content.clone()).unwrap();
        assert!(
            claude.contains("Only invoke a Harness skill when the user explicitly requests it.")
        );
        assert!(!claude.contains("AGENTS.md"));
        assert!(!claude.contains("Current Upstream Goal"));
        let plans = distribution
            .files
            .iter()
            .find(|file| file.path.as_str() == "docs/plans/README.md")
            .unwrap();
        assert!(!String::from_utf8_lossy(&plans.content).contains("rust-harness-core"));
        for skill in [
            ".claude/skills/onboard-repository/SKILL.md",
            ".claude/skills/audit-onboarding-proposal/SKILL.md",
            ".claude/skills/encode-invariant/SKILL.md",
            ".claude/skills/improve-harness/SKILL.md",
        ] {
            let skill = distribution
                .files
                .iter()
                .find(|file| file.path.as_str() == skill)
                .unwrap();
            let skill = String::from_utf8_lossy(&skill.content);
            assert!(skill.contains("authorized") || skill.contains("read-only"));
            assert!(skill.contains("\ndisable-model-invocation: true\n"));
        }
    }
}
