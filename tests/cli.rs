use autodev_planning_revision::{CliCommand, parse_cli};
use std::path::PathBuf;

fn parse(arguments: &[&str]) -> CliCommand {
    let arguments: Vec<String> = arguments.iter().map(|value| (*value).to_owned()).collect();
    parse_cli(&arguments).expect("supported command")
}

fn error(arguments: &[&str]) -> String {
    let arguments: Vec<String> = arguments.iter().map(|value| (*value).to_owned()).collect();
    parse_cli(&arguments)
        .expect_err("rejected command")
        .to_string()
}

#[test]
fn no_argument_validates_the_current_directory() {
    assert_eq!(
        parse(&[]),
        CliCommand::Validate {
            root: PathBuf::from(".")
        }
    );
}

#[test]
fn a_positional_root_selects_the_project() {
    assert_eq!(
        parse(&["/tmp/project"]),
        CliCommand::Validate {
            root: PathBuf::from("/tmp/project")
        }
    );
}

#[test]
fn recorded_print_commands_keep_their_positional_root() {
    assert_eq!(
        parse(&["--print-project-revision", "/tmp/project"]),
        CliCommand::PrintProjectRevision {
            root: PathBuf::from("/tmp/project")
        }
    );
    assert_eq!(
        parse(&["--print-task-projection"]),
        CliCommand::PrintTaskProjection {
            root: PathBuf::from(".")
        }
    );
    assert_eq!(
        parse(&["--print-validated-task-projection", "."]),
        CliCommand::PrintValidatedTaskProjection {
            root: PathBuf::from(".")
        }
    );
    assert_eq!(
        error(&["--print-project-revision", ".", "."]),
        "expected one project root"
    );
    assert_eq!(error(&[".", "."]), "expected one project root");
}

#[test]
fn a_task_snapshot_requires_a_root_and_a_positive_issue() {
    assert_eq!(
        parse(&["--print-task-snapshot", "--root", ".", "--issue", "7"]),
        CliCommand::PrintTaskSnapshot {
            root: PathBuf::from("."),
            issue: 7
        }
    );
    assert_eq!(
        error(&["--print-task-snapshot", "--root", "."]),
        "--print-task-snapshot requires --issue"
    );
    assert_eq!(
        error(&["--print-task-snapshot", "--issue", "7"]),
        "--print-task-snapshot requires --root"
    );
    for issue in ["0", "-1", "seven", ""] {
        assert_eq!(
            error(&["--print-task-snapshot", "--root", ".", "--issue", issue]),
            "task issue number must be a positive integer",
            "issue {issue}"
        );
    }
}

#[test]
fn authorization_reads_the_event_agent_input_and_optional_prior_record() {
    assert_eq!(
        parse(&[
            "--authorize",
            "--root",
            ".",
            "--issue",
            "7",
            "--event",
            "event.json",
            "--agent-input",
            "input.md",
        ]),
        CliCommand::Authorize {
            root: PathBuf::from("."),
            issue: 7,
            event: PathBuf::from("event.json"),
            agent_input: PathBuf::from("input.md"),
            prior: None,
        }
    );
    assert_eq!(
        parse(&[
            "--authorize",
            "--root",
            ".",
            "--issue",
            "7",
            "--event",
            "event.json",
            "--agent-input",
            "input.md",
            "--prior",
            "prior.json",
        ]),
        CliCommand::Authorize {
            root: PathBuf::from("."),
            issue: 7,
            event: PathBuf::from("event.json"),
            agent_input: PathBuf::from("input.md"),
            prior: Some(PathBuf::from("prior.json")),
        }
    );
    assert_eq!(
        error(&[
            "--authorize",
            "--root",
            ".",
            "--issue",
            "7",
            "--event",
            "event.json",
        ]),
        "--authorize requires --agent-input"
    );
}

#[test]
fn a_transition_reads_the_event_and_the_current_record() {
    assert_eq!(
        parse(&[
            "--transition",
            "--root",
            ".",
            "--event",
            "event.json",
            "--current",
            "record.json",
        ]),
        CliCommand::Transition {
            root: PathBuf::from("."),
            event: PathBuf::from("event.json"),
            current: PathBuf::from("record.json"),
        }
    );
    assert_eq!(
        error(&["--transition", "--root", ".", "--event", "event.json"]),
        "--transition requires --current"
    );
}

#[test]
fn malformed_options_are_rejected_instead_of_ignored() {
    assert_eq!(
        error(&["--transition", "--root", ".", "--event"]),
        "--transition option --event is missing its value"
    );
    assert_eq!(
        error(&["--transition", "--root", ".", "--root", "."]),
        "--transition option --root is repeated"
    );
    assert_eq!(
        error(&["--transition", "root", "."]),
        "--transition expects --name value options"
    );
    assert_eq!(
        error(&[
            "--transition",
            "--root",
            ".",
            "--event",
            "event.json",
            "--current",
            "record.json",
            "--force",
            "yes",
        ]),
        "--transition does not accept --force"
    );
    assert_eq!(error(&["--merge"]), "unsupported command: --merge");
}
