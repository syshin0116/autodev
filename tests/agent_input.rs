use std::path::PathBuf;
use std::process::Command;

fn agent_input(association: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(root.join("scripts/autodev-agent-input.sh"))
        .arg(root.join("test/fixtures/agent-input/snapshot.json"))
        .arg(association)
        .output()
        .expect("run the agent input script");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf-8 output")
}

// The controller and the local runner compare this output by digest, so a
// change here invalidates every in-flight episode.
#[test]
fn a_trusted_author_contributes_the_body_and_dependencies() {
    assert_eq!(
        agent_input("OWNER"),
        "# Turn one authorized issue into a verified draft pull request\n\
         \n\
         ## Outcome\n\
         \n\
         One example outcome.\n\
         \n\
         \n\
         Blocked by: 5, 6\n"
    );
}

#[test]
fn an_untrusted_body_is_withheld_with_its_reason() {
    let input = agent_input("FIRST_TIME_CONTRIBUTOR");
    assert!(
        input.contains(
            "The issue body was withheld because its author association is FIRST_TIME_CONTRIBUTOR."
        ),
        "{input}"
    );
    assert!(!input.contains("One example outcome"), "{input}");
    assert!(input.ends_with("Blocked by: 5, 6\n"), "{input}");
}

#[test]
fn every_association_outside_the_trusted_set_withholds_the_body() {
    for association in ["CONTRIBUTOR", "NONE", "MANNEQUIN", "FIRST_TIMER"] {
        let input = agent_input(association);
        assert!(!input.contains("One example outcome"), "{association}");
    }
}

// An association the caller could not read is not a trust decision, so the
// script refuses instead of guessing.
#[test]
fn an_unreadable_association_aborts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(root.join("scripts/autodev-agent-input.sh"))
        .arg(root.join("test/fixtures/agent-input/snapshot.json"))
        .arg("")
        .output()
        .expect("run the agent input script");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
