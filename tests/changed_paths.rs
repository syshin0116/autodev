use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(tree: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(tree)
        .args(arguments)
        .status()
        .expect("run git");
    assert!(status.success(), "git {arguments:?}");
}

fn repository(name: &str) -> PathBuf {
    let tree = std::env::temp_dir().join(format!("autodev-changed-{name}-{}", std::process::id()));
    fs::remove_dir_all(&tree).ok();
    fs::create_dir_all(tree.join("adr")).expect("create dirs");
    git(&tree, &["init", "-q", "."]);
    git(&tree, &["config", "user.email", "test@example.com"]);
    git(&tree, &["config", "user.name", "test"]);
    fs::write(tree.join("adr/0001.md"), "one\n").expect("write");
    git(&tree, &["add", "-A"]);
    git(&tree, &["commit", "-qm", "base"]);
    tree
}

fn changed(tree: &Path) -> Vec<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(root.join("scripts/autodev-changed-paths.sh"))
        .arg(tree)
        .output()
        .expect("run the changed paths script");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("utf-8")
        .lines()
        .map(str::to_owned)
        .collect()
}

// Moving a file out of a protected directory changes that directory, so the
// source has to appear or the protected-path gate never sees it.
#[test]
fn a_rename_lists_both_sides() {
    let tree = repository("rename");
    fs::create_dir_all(tree.join("docs")).expect("create docs");
    git(&tree, &["mv", "adr/0001.md", "docs/0001.md"]);
    let changed = changed(&tree);
    assert!(changed.contains(&"adr/0001.md".to_owned()), "{changed:?}");
    assert!(changed.contains(&"docs/0001.md".to_owned()), "{changed:?}");
    fs::remove_dir_all(&tree).ok();
}

// A space used to split one path into two, and the tail was checked against
// the protected paths instead of the real name.
#[test]
fn a_path_with_a_space_stays_one_path() {
    let tree = repository("space");
    fs::write(tree.join("adr/new record.md"), "two\n").expect("write");
    assert_eq!(changed(&tree), vec!["adr/new record.md".to_owned()]);
    fs::remove_dir_all(&tree).ok();
}

#[test]
fn a_clean_worktree_lists_nothing() {
    let tree = repository("clean");
    assert!(changed(&tree).is_empty());
    fs::remove_dir_all(&tree).ok();
}

// A newline in a name cannot be listed one per line, so the run stops instead
// of checking a mangled path.
#[test]
fn a_path_with_a_newline_is_refused() {
    let tree = repository("newline");
    fs::write(tree.join("adr/two\nlines.md"), "three\n").expect("write");
    assert_refused(&tree);
    fs::remove_dir_all(&tree).ok();
}

// A newline at either end used to survive, because dropping the empty line it
// produced left a truncated name that still counted as one record.
#[test]
fn a_path_with_a_boundary_newline_is_refused() {
    let tree = repository("boundary");
    fs::write(tree.join("adr/trailing\n"), "four\n").expect("write");
    assert_refused(&tree);
    fs::remove_dir_all(&tree).ok();
}

fn assert_refused(tree: &Path) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(root.join("scripts/autodev-changed-paths.sh"))
        .arg(tree)
        .output()
        .expect("run the changed paths script");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("newline"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
