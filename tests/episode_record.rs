use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn read_record(fixture: &str) -> Output {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Command::new(root.join("scripts/autodev-episode-record.sh"))
        .arg("owner/repo")
        .arg("16")
        .env(
            "AUTODEV_COMMENTS_FILE",
            root.join("test/fixtures/episode-record").join(fixture),
        )
        .output()
        .expect("run the episode record script")
}

fn json(fixture: &str) -> serde_json::Value {
    let output = read_record(fixture);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json output")
}

// The record can sit on any page of comments, and the newest marked comment
// wins so an upsert that lost a race cannot resurrect an older generation.
#[test]
fn the_last_marked_comment_across_pages_is_the_record() {
    let record = json("two-pages.json");
    assert_eq!(record["comment_id"], 4);
    assert_eq!(
        record["authorizations"][0]["episode"]["authorization_generation"],
        2
    );
}

#[test]
fn an_issue_without_a_record_reports_an_empty_list() {
    let record = json("no-record.json");
    assert!(record["comment_id"].is_null());
    assert_eq!(record["authorizations"].as_array().expect("array").len(), 0);
}

// A damaged record must stop the caller instead of silently looking like a
// first authorization, which would start a second episode.
#[test]
fn a_damaged_record_fails_instead_of_looking_empty() {
    for fixture in ["no-json-block.json", "not-an-array.json"] {
        let output = read_record(fixture);
        assert!(!output.status.success(), "{fixture}");
        assert!(output.stdout.is_empty(), "{fixture}");
    }
}

// The writer and the reader are the two halves of one format. A round trip is
// the only cheap way to keep them from drifting apart.
#[test]
fn a_written_record_reads_back_unchanged() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let temp = std::env::temp_dir().join(format!("autodev-record-{}", std::process::id()));
    fs::create_dir_all(&temp).expect("temp dir");
    let authorizations = temp.join("authorizations.json");
    let written = serde_json::json!([
        {"episode": {"issue_number": 16, "authorization_generation": 2}, "status": "abandoned"}
    ]);
    fs::write(
        &authorizations,
        serde_json::to_vec_pretty(&written).expect("serialize"),
    )
    .expect("write authorizations");

    let body = Command::new(root.join("scripts/autodev-record-write.sh"))
        .args(["owner/repo", "16", ""])
        .arg(&authorizations)
        .env("AUTODEV_RECORD_DRY_RUN", "1")
        .output()
        .expect("render the record comment");
    assert!(
        body.status.success(),
        "{}",
        String::from_utf8_lossy(&body.stderr)
    );

    let comments = temp.join("comments.json");
    let page = serde_json::json!([[{
        "id": 42,
        "body": String::from_utf8(body.stdout).expect("utf-8 body")
    }]]);
    fs::write(&comments, serde_json::to_vec(&page).expect("serialize")).expect("write comments");

    let read = Command::new(root.join("scripts/autodev-episode-record.sh"))
        .args(["owner/repo", "16"])
        .env("AUTODEV_COMMENTS_FILE", &comments)
        .output()
        .expect("read the record");
    assert!(
        read.status.success(),
        "{}",
        String::from_utf8_lossy(&read.stderr)
    );
    let record: serde_json::Value = serde_json::from_slice(&read.stdout).expect("json");
    assert_eq!(record["comment_id"], 42);
    assert_eq!(record["authorizations"], written);

    fs::remove_dir_all(&temp).ok();
}

#[test]
fn an_empty_authorization_list_is_refused() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let temp = std::env::temp_dir().join(format!("autodev-record-empty-{}", std::process::id()));
    fs::create_dir_all(&temp).expect("temp dir");
    let empty = temp.join("empty.json");
    fs::write(&empty, b"[]").expect("write empty");
    let output = Command::new(root.join("scripts/autodev-record-write.sh"))
        .args(["owner/repo", "16", ""])
        .arg(&empty)
        .env("AUTODEV_RECORD_DRY_RUN", "1")
        .output()
        .expect("run the writer");
    assert!(!output.status.success());
    fs::remove_dir_all(&temp).ok();
}
