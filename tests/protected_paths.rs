use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn expression(patterns: &[&str]) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut child = Command::new(root.join("scripts/autodev-protected-paths.sh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("run the protected path script");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(patterns.join("\n").as_bytes())
        .expect("write patterns");
    let output = child.wait_with_output().expect("collect output");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("utf-8")
        .trim()
        .to_owned()
}

fn matches(patterns: &[&str], path: &str) -> bool {
    let expression = expression(patterns);
    Command::new("grep")
        .args(["-qE", &expression])
        .stdin(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(path.as_bytes())?;
            child.wait()
        })
        .expect("run grep")
        .success()
}

// The approved policy is the only source of these paths, so the conversion has
// to survive every shape the policy is allowed to use.
#[test]
fn a_globstar_covers_everything_under_a_directory() {
    let patterns = [".autodev/**", "adr/**"];
    assert!(matches(&patterns, ".autodev/config.yaml"));
    assert!(matches(&patterns, "adr/0001-thin-first-version.md"));
    assert!(!matches(&patterns, "docs/project-overview.md"));
}

#[test]
fn an_exact_file_matches_only_itself() {
    let patterns = ["docs/project-overview.md"];
    assert!(matches(&patterns, "docs/project-overview.md"));
    assert!(!matches(&patterns, "docs/project-overviewXmd"));
    assert!(!matches(&patterns, "elsewhere/docs/project-overview.md"));
    assert!(!matches(&patterns, "docs/project-overview.md.bak"));
}

// A single star stops at a path separator, so a nested file is not caught by
// a pattern that only names one level.
#[test]
fn a_single_star_does_not_cross_a_directory() {
    let patterns = ["docs/*.md"];
    assert!(matches(&patterns, "docs/readme.md"));
    assert!(!matches(&patterns, "docs/nested/readme.md"));
}

#[test]
fn an_empty_list_is_refused_instead_of_matching_nothing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut child = Command::new(root.join("scripts/autodev-protected-paths.sh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run the script");
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("collect output");
    assert!(!output.status.success());
}
