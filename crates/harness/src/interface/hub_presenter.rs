use serde::Serialize;

use crate::application::{HubMode, HubReport, MemberOutcome};
use crate::interface::presenter::CommandExit;

#[derive(Serialize)]
struct HubOutput<'a> {
    operation: &'static str,
    mode: &'static str,
    ok: bool,
    members: Vec<MemberOutput<'a>>,
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
        };
        return CommandExit {
            code,
            stdout: format!("{}\n", serde_json::to_string_pretty(&output).unwrap()),
            stderr: String::new(),
        };
    }

    let mut out = format!("Hub members ({mode}):\n");
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
    if !ok && report.mode == HubMode::Check {
        out.push_str("Run `harness hub sync` in the hub to refresh stale members.\n");
    }
    CommandExit {
        code,
        stdout: out,
        stderr: String::new(),
    }
}
