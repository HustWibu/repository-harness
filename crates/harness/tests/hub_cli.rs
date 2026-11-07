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

    assert!(fs::read_to_string(orders.join("AGENTS.md"))
        .unwrap()
        .contains("../system-knowledge/docs/services/orders-service/"));
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

    // Drift in Codex alone must make the joint check stale.
    fs::write(orders.join("AGENTS.md"), "# Codex team rules\n").unwrap();
    assert_eq!(harness(&["--check"], &hub).status.code(), Some(1));
    assert!(harness(&[], &hub).status.success());
    assert!(fs::read_to_string(orders.join("AGENTS.md"))
        .unwrap()
        .starts_with("# Codex team rules\n"));

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
fn hub_sync_workspace_writes_both_parent_entries_and_keeps_their_text() {
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
    fs::write(root.join("AGENTS.md"), "# Codex notes\n").unwrap();

    let check = harness(&["--workspace", "--check"], &hub);
    assert_eq!(check.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&check.stdout);
    assert!(stdout.contains("stale"), "{stdout}");
    assert!(stdout.contains("stale orders-service"), "{stdout}");

    assert!(harness(&["--workspace"], &hub).status.success());
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(agents.starts_with("# Codex notes\n\n<!-- HARNESS-HUB:BEGIN -->"));
    assert!(agents.contains(&format!("`{}/docs/services/<repository>/`", hub.display())));
    assert!(fs::read_to_string(root.join("orders-service/AGENTS.md"))
        .unwrap()
        .contains("`../system-knowledge/docs/services/orders-service/`"));
    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(claude.starts_with("# Workspace notes\nkeep this\n\n<!-- HARNESS-HUB:BEGIN -->"));
    assert!(claude.contains(&format!("`{}/docs/services/<repository>/`", hub.display())));
    assert!(!root.join("orders-service/CLAUDE.md").exists());
    assert!(!root.join("billing-service/CLAUDE.md").exists());
    assert!(harness(&["--workspace", "--check"], &hub).status.success());
}

fn commit(repo: &Path, file: &str, content: &str) -> String {
    fs::create_dir_all(repo.join(file).parent().unwrap()).unwrap();
    fs::write(repo.join(file), content).unwrap();
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .current_dir(repo)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    git(&["add", "-A"]);
    git(&["commit", "-qm", "c"]);
    git(&["rev-parse", "HEAD"])
}

fn fresh(hub: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_harness"))
        .args(["hub", "check-fresh", "--directory"])
        .arg(hub)
        .output()
        .unwrap()
}

#[test]
fn hub_check_fresh_follows_cited_lines_across_member_commits() {
    let workspace = tempfile::tempdir().unwrap();
    let root = workspace.path();
    let hub = root.join("system-knowledge");
    let orders = root.join("orders-service");
    git_repo(&hub);
    git_repo(&orders);
    git_repo(&root.join("billing-service"));
    git_repo(&root.join("stray-tool"));
    git_repo(&root.join(".worktree-copy"));
    fs::create_dir_all(hub.join("docs/services/orders-service")).unwrap();
    fs::create_dir_all(hub.join("docs/services/billing-service")).unwrap();
    fs::write(hub.join("docs/README.md"), MAP).unwrap();

    let revision = commit(&orders, "src/App.java", "class App {\n  send();\n}\n");
    let digest = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(b"  send();\n"))
    };
    fs::write(
        hub.join("docs/services/orders-service/README.md"),
        format!(
            "# orders\n\nOrders sends on start.\n<!-- harness-source: orders-service@{revision}:src/App.java#L2-L2 sha256={digest} -->\n"
        ),
    )
    .unwrap();

    let current = fresh(&hub);
    assert!(current.status.success(), "{current:?}");
    assert!(String::from_utf8_lossy(&current.stdout)
        .contains("current docs/services/orders-service/README.md:4"));

    commit(
        &orders,
        "src/App.java",
        "// header\nclass App {\n  send();\n}\n",
    );
    let moved = fresh(&hub);
    assert!(moved.status.success());
    assert!(String::from_utf8_lossy(&moved.stdout).contains("now starts at line 3"));

    commit(
        &orders,
        "src/App.java",
        "// header\nclass App {\n  sendAsync();\n}\n",
    );
    let stale = fresh(&hub);
    assert_eq!(stale.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&stale.stdout).contains("stale"));

    let sync = harness(&["--workspace", "--check"], &hub);
    assert!(String::from_utf8_lossy(&sync.stdout).contains("unlisted stray-tool"));
    assert!(!String::from_utf8_lossy(&sync.stdout).contains("unlisted system-knowledge"));
    assert!(!String::from_utf8_lossy(&sync.stdout).contains("unlisted .worktree-copy"));
}

#[cfg(unix)]
#[test]
fn hub_sync_refuses_codex_symlink_before_changing_claude() {
    let workspace = tempfile::tempdir().unwrap();
    let hub = workspace.path().join("hub");
    let member = workspace.path().join("orders-service");
    git_repo(&hub);
    git_repo(&member);
    fs::create_dir_all(hub.join("docs/services/orders-service")).unwrap();
    fs::write(
        hub.join("docs/README.md"),
        "## Related Repositories\n| Repository | Role | Location |\n| --- | --- | --- |\n| orders-service | service | ../orders-service |\n",
    )
    .unwrap();
    fs::write(member.join("CLAUDE.md"), "team rules\n").unwrap();
    let outside = workspace.path().join("outside");
    fs::write(&outside, "keep\n").unwrap();
    std::os::unix::fs::symlink(&outside, member.join("AGENTS.md")).unwrap();
    assert_eq!(harness(&[], &hub).status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(member.join("CLAUDE.md")).unwrap(),
        "team rules\n"
    );
    assert_eq!(fs::read_to_string(outside).unwrap(), "keep\n");
}
