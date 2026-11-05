use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn harness(args: &[&str], hub: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_harness"))
        .args(["hub", "sync", "--directory"])
        .arg(hub)
        .args(args)
        .output()
        .unwrap()
}

fn git_repo(path: &Path) {
    fs::create_dir_all(path).unwrap();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
}

const MAP: &str = "# Map\n\n## Related Repositories\n\n\
| Repository | Role | Location |\n| --- | --- | --- |\n\
| orders-service | service | ../orders-service |\n\
| billing-service | service | ../billing-service |\n\
| remote-lib | shared library | https://example.invalid/lib.git |\n";

#[test]
fn hub_sync_writes_only_the_pointer_block_and_check_detects_drift() {
    let workspace = tempfile::tempdir().unwrap();
    let hub = workspace.path().join("system-knowledge");
    let orders = workspace.path().join("orders-service");
    let billing = workspace.path().join("billing-service");
    git_repo(&hub);
    git_repo(&orders);
    git_repo(&billing);
    for member in ["orders-service", "billing-service", "plain"] {
        fs::create_dir_all(hub.join("docs/services").join(member)).unwrap();
    }
    fs::write(hub.join("docs/README.md"), MAP).unwrap();
    fs::write(orders.join("CLAUDE.md"), "# Orders team rules\nkeep this\n").unwrap();

    let check = harness(&["--check"], &hub);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("stale orders-service"));

    let dry = harness(&["--dry-run"], &hub);
    assert!(dry.status.success());
    assert!(!billing.join("CLAUDE.md").exists());

    let sync = harness(&["--json"], &hub);
    assert!(
        sync.status.success(),
        "{}",
        String::from_utf8_lossy(&sync.stdout)
    );
    let report: serde_json::Value = serde_json::from_slice(&sync.stdout).unwrap();
    assert_eq!(report["members"].as_array().unwrap().len(), 2);
    assert_eq!(report["members"][0]["outcome"], "updated");

    let orders_claude = fs::read_to_string(orders.join("CLAUDE.md")).unwrap();
    assert!(
        orders_claude.starts_with("# Orders team rules\nkeep this\n\n<!-- HARNESS-HUB:BEGIN -->")
    );
    assert!(orders_claude.contains("`../system-knowledge/docs/services/orders-service/`"));
    let billing_claude = fs::read_to_string(billing.join("CLAUDE.md")).unwrap();
    assert!(billing_claude.starts_with("<!-- HARNESS-HUB:BEGIN -->"));
    assert!(!workspace.path().join("remote-lib").exists());

    assert!(harness(&["--check"], &hub).status.success());
    assert!(harness(&[], &hub).status.success());
    assert_eq!(
        fs::read_to_string(orders.join("CLAUDE.md")).unwrap(),
        orders_claude
    );

    // A listed member that is not a Git root fails without touching others.
    fs::create_dir_all(workspace.path().join("plain")).unwrap();
    fs::write(
        hub.join("docs/README.md"),
        format!("{MAP}| plain | service | ../plain |\n"),
    )
    .unwrap();
    let broken = harness(&[], &hub);
    assert_eq!(broken.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&broken.stdout)
        .contains("failed plain (../plain): ../plain is not the root of a Git repository"));
    assert!(!workspace.path().join("plain/CLAUDE.md").exists());

    // A member without docs/services/<name>/ in the hub is not pointed at it.
    fs::remove_dir(hub.join("docs/services/billing-service")).unwrap();
    let undocumented = harness(&["--check"], &hub);
    assert_eq!(undocumented.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&undocumented.stdout)
        .contains("failed billing-service (../billing-service): the hub has no docs/services/billing-service/"));
}

#[test]
fn hub_sync_without_a_map_section_is_an_error() {
    let hub = tempfile::tempdir().unwrap();
    fs::create_dir_all(hub.path().join("docs")).unwrap();
    fs::write(hub.path().join("docs/README.md"), "# Nothing\n").unwrap();
    let output = harness(&[], hub.path());
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Related Repositories"));
}

#[test]
fn hub_sync_workspace_writes_only_the_parent_claude_and_keeps_its_text() {
    let workspace = tempfile::tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let hub = root.join("system-knowledge");
    git_repo(&hub);
    git_repo(&root.join("orders-service"));
    git_repo(&root.join("billing-service"));
    for member in ["orders-service", "billing-service"] {
        fs::create_dir_all(hub.join("docs/services").join(member)).unwrap();
    }
    fs::write(hub.join("docs/README.md"), MAP).unwrap();
    fs::write(root.join("CLAUDE.md"), "# Workspace notes\nkeep this\n").unwrap();

    let check = harness(&["--workspace", "--check"], &hub);
    assert_eq!(check.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&check.stdout);
    assert!(stdout.contains("stale"), "{stdout}");
    assert!(stdout.contains("covered orders-service"), "{stdout}");

    assert!(harness(&["--workspace"], &hub).status.success());
    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(claude.starts_with("# Workspace notes\nkeep this\n\n<!-- HARNESS-HUB:BEGIN -->"));
    assert!(claude.contains(&format!("`{}/docs/services/<repository>/`", hub.display())));
    assert!(!root.join("orders-service/CLAUDE.md").exists());
    assert!(!root.join("billing-service/CLAUDE.md").exists());
    assert!(harness(&["--workspace", "--check"], &hub).status.success());
}
