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

// `**/` means zero or more directories, so a pattern that names a file under
// a tree must also protect that file at the top of the tree.
#[test]
fn a_globstar_between_slashes_matches_zero_directories() {
    let patterns = ["docs/**/README.md"];
    assert!(matches(&patterns, "docs/README.md"));
    assert!(matches(&patterns, "docs/guide/README.md"));
    assert!(matches(&patterns, "docs/guide/deep/README.md"));
    assert!(!matches(&patterns, "docs/README.md.bak"));
    assert!(!matches(&patterns, "other/README.md"));
}

// A leading globstar has to reach the repository root, or a policy naming
// **/README.md would protect every README except the one at the top.
#[test]
fn a_leading_globstar_reaches_the_repository_root() {
    let patterns = ["**/README.md"];
    assert!(matches(&patterns, "README.md"));
    assert!(matches(&patterns, "docs/README.md"));
    assert!(matches(&patterns, "docs/guide/README.md"));
    assert!(!matches(&patterns, "README.md.bak"));
}

// The conversion parks globstars on a sentinel, so a pattern carrying that
// sentinel would be rewritten into a pattern nobody approved.
#[test]
fn a_pattern_containing_the_sentinel_is_refused() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut child = Command::new(root.join("scripts/autodev-protected-paths.sh"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run the script");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"foo@@GLOBSTARDIR@@bar")
        .expect("write pattern");
    let output = child.wait_with_output().expect("collect output");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

// `**` is a whole segment. A shape this conversion cannot express is refused
// rather than compiled into a pattern that covers less than it reads like.
#[test]
fn an_unsupported_globstar_shape_is_refused() {
    for pattern in ["a**b", "**/**/README.md", "docs/a**/x.md"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut child = Command::new(root.join("scripts/autodev-protected-paths.sh"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run the script");
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(pattern.as_bytes())
            .expect("write pattern");
        let output = child.wait_with_output().expect("collect output");
        assert!(!output.status.success(), "{pattern}");
        assert!(output.stdout.is_empty(), "{pattern}");
    }
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
