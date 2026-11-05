use serde::Serialize;

use crate::application::{FreshnessReport, HubMode, HubReport, MemberOutcome};
use crate::domain::Freshness;
use crate::interface::presenter::CommandExit;

#[derive(Serialize)]
struct HubOutput<'a> {
    operation: &'static str,
    mode: &'static str,
    ok: bool,
    members: Vec<MemberOutput<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace: Option<WorkspaceOutput<'a>>,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    unlisted: &'a [String],
}

#[derive(Serialize)]
struct WorkspaceOutput<'a> {
    path: &'a str,
    outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<&'a str>,
}

#[derive(Serialize)]
struct MemberOutput<'a> {
    repository: &'a str,
    location: &'a str,
    outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<&'a str>,
}

fn outcome(outcome: &MemberOutcome) -> (&'static str, Option<&str>) {
    match outcome {
        MemberOutcome::Current => ("current", None),
        MemberOutcome::Updated => ("updated", None),
        MemberOutcome::WouldUpdate => ("stale", None),
        MemberOutcome::Covered => ("covered", None),
        MemberOutcome::Failed(reason) => ("failed", Some(reason)),
    }
}

pub fn present_hub_sync(report: &HubReport, json: bool) -> CommandExit {
    let ok = report.ok();
    let code = if ok { 0 } else { 1 };
    let mode = match report.mode {
        HubMode::Apply => "apply",
        HubMode::DryRun => "dry-run",
        HubMode::Check => "check",
    };
    if json {
        let output = HubOutput {
            operation: "hub-sync",
            mode,
            ok,
            members: report
                .members
                .iter()
                .map(|member| {
                    let (outcome, detail) = outcome(&member.outcome);
                    MemberOutput {
                        repository: &member.name,
                        location: &member.location,
                        outcome,
                        detail,
                    }
                })
                .collect(),
            workspace: report.workspace.as_ref().map(|workspace| {
                let (outcome, detail) = outcome(&workspace.outcome);
                WorkspaceOutput {
                    path: &workspace.path,
                    outcome,
                    detail,
                }
            }),
            unlisted: &report.unlisted,
        };
        return CommandExit {
            code,
            stdout: format!("{}\n", serde_json::to_string_pretty(&output).unwrap()),
            stderr: String::new(),
        };
    }

    let mut out = String::new();
    if let Some(workspace) = &report.workspace {
        let (label, detail) = outcome(&workspace.outcome);
        out.push_str(&format!(
            "Workspace ({mode}):\n  {label} {}",
            workspace.path
        ));
        if let Some(detail) = detail {
            out.push_str(&format!(": {detail}"));
        }
        out.push('\n');
    }
    out.push_str(&format!("Hub members ({mode}):\n"));
    if report.members.is_empty() {
        out.push_str(
            "  none listed with a relative Location under Related Repositories in docs/README.md\n",
        );
    }
    for member in &report.members {
        let (label, detail) = outcome(&member.outcome);
        out.push_str(&format!("  {label} {} ({})", member.name, member.location));
        if let Some(detail) = detail {
            out.push_str(&format!(": {detail}"));
        }
        out.push('\n');
    }
    if !report.unlisted.is_empty() {
        out.push_str("Git repositories in the workspace not listed under Related Repositories:\n");
        for name in &report.unlisted {
            out.push_str(&format!("  unlisted {name}\n"));
        }
    }
    if !ok && report.mode == HubMode::Check {
        out.push_str(
            "Fix failed members, then run `harness hub sync` in the hub to refresh stale ones.\n",
        );
    }
    CommandExit {
        code,
        stdout: out,
        stderr: String::new(),
    }
}

#[derive(Serialize)]
struct FreshnessOutput<'a> {
    operation: &'static str,
    ok: bool,
    pages: usize,
    sources: Vec<SourceOutput<'a>>,
}

#[derive(Serialize)]
struct SourceOutput<'a> {
    page: &'a str,
    line: usize,
    source: &'a str,
    outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

fn freshness(outcome: &Result<Freshness, String>) -> (&'static str, Option<String>) {
    match outcome {
        Ok(Freshness::Current) => ("current", None),
        Ok(Freshness::Moved(line)) => ("moved", Some(format!("now starts at line {line}"))),
        Ok(Freshness::Stale(reason)) => ("stale", Some(reason.clone())),
        Err(reason) => ("error", Some(reason.clone())),
    }
}

pub fn present_hub_freshness(report: &FreshnessReport, json: bool) -> CommandExit {
    let ok = report.ok();
    let code = if ok { 0 } else { 1 };
    if json {
        let output = FreshnessOutput {
            operation: "hub-check-fresh",
            ok,
            pages: report.pages,
            sources: report
                .sources
                .iter()
                .map(|check| {
                    let (outcome, detail) = freshness(&check.outcome);
                    SourceOutput {
                        page: &check.page,
                        line: check.line,
                        source: &check.source,
                        outcome,
                        detail,
                    }
                })
                .collect(),
        };
        return CommandExit {
            code,
            stdout: format!("{}\n", serde_json::to_string_pretty(&output).unwrap()),
            stderr: String::new(),
        };
    }
    let mut out = format!(
        "Hub knowledge sources: {} in {} pages\n",
        report.sources.len(),
        report.pages
    );
    for check in &report.sources {
        let (label, detail) = freshness(&check.outcome);
        out.push_str(&format!(
            "  {label} {}:{} {}",
            check.page, check.line, check.source
        ));
        if let Some(detail) = detail {
            out.push_str(&format!(": {detail}"));
        }
        out.push('\n');
    }
    if !ok {
        out.push_str(
            "Re-check the stale claims against the member code and update their markers.\n",
        );
    }
    CommandExit {
        code,
        stdout: out,
        stderr: String::new(),
    }
}
