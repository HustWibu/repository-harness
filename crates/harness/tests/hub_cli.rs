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
    fs::create_dir_all(hub.join("docs")).unwrap();
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
