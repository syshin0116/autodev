use std::path::PathBuf;
use std::process::Command;

fn statuses() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("test/fixtures/dependency-status");
    let output = Command::new(root.join("scripts/autodev-dependency-status.sh"))
        .arg("owner/repo")
        .arg(fixtures.join("snapshot.json"))
        .arg("main")
        .env("AUTODEV_COMMENTS_DIR", &fixtures)
        .output()
        .expect("run the dependency status script");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json output")
}

// One status per declared dependency, or the readiness check cannot tell an
// unknown dependency from a complete one.
#[test]
fn every_declared_dependency_gets_a_status() {
    let statuses = statuses();
    let statuses = statuses.as_array().expect("array");
    assert_eq!(statuses.len(), 3);
    assert_eq!(statuses[0]["issue_node_id"], "I_five");
    assert_eq!(statuses[1]["issue_node_id"], "I_six");
    assert_eq!(statuses[2]["issue_node_id"], "I_seven");
}

#[test]
fn only_a_merged_episode_carries_verified_evidence() {
    let statuses = statuses();
    assert_eq!(statuses[0]["evidence_verified"], true);
    assert_eq!(statuses[0]["merged_into"], "main");
    assert_eq!(
        statuses[0]["evidence_task_sha256"],
        statuses[0]["approved_task_sha256"]
    );
}

// An authorized but unfinished dependency must not look complete, and neither
// must one nobody authorized at all.
#[test]
fn an_unfinished_or_missing_dependency_carries_none() {
    let statuses = statuses();
    for index in [1, 2] {
        assert_eq!(statuses[index]["evidence_verified"], false, "index {index}");
        assert!(statuses[index]["merged_into"].is_null(), "index {index}");
        assert_eq!(statuses[index]["evidence_authorization_generation"], 0);
    }
    assert_eq!(statuses[1]["authorization_generation"], 2);
    assert!(statuses[2]["approved_task_sha256"].is_null());
    assert_eq!(statuses[2]["authorization_generation"], 0);
}
