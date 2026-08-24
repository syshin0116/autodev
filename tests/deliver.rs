use std::path::PathBuf;
use std::process::{Command, Output};

fn deliver(args: &[&str]) -> Output {
    Command::new("/bin/bash")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/autodev-deliver.sh"))
        .args(args)
        .env("PATH", "")
        .output()
        .expect("run the delivery script")
}

#[test]
fn help_is_local_without_weakening_required_arguments() {
    let help = deliver(&["--help"]);
    assert!(help.status.success());
    let usage = String::from_utf8(help.stdout).expect("utf-8 help");
    assert!(usage.contains("Usage:"));
    assert!(usage.contains("--issue"));
    assert!(usage.contains("--apply"));
    assert!(usage.contains("Required to push a branch or open a pull request."));

    let unsupported = deliver(&["--unknown"]);
    assert_eq!(unsupported.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unsupported.stderr).contains("--unknown"));

    assert!(!deliver(&[]).status.success());
    assert!(!deliver(&["--issue"]).status.success());
}
