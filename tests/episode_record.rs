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
