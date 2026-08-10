use autodev_planning_revision::{task_source_projection, validate};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    let first = arguments.next();
    let mode = match first.as_deref().and_then(|value| value.to_str()) {
        Some("--print-task-projection") => 1,
        Some("--print-validated-task-projection") => 2,
        _ => 0,
    };
    let project_root = if mode == 0 {
        first
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        arguments
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    if arguments.next().is_some() {
        eprintln!("ERROR: expected one project root");
        return ExitCode::FAILURE;
    }

    if mode == 1 {
        match task_source_projection(&project_root).and_then(|value| {
            serde_json::to_string_pretty(&value)
                .map_err(|error| autodev_planning_revision::ValidationError::new(error.to_string()))
        }) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("ERROR: {error}");
                ExitCode::FAILURE
            }
        }
    } else if mode == 2 {
        match validate(&project_root)
            .and_then(|value| {
                value.ok_or_else(|| {
                    autodev_planning_revision::ValidationError::new(
                        "configured task source is not github_issues",
                    )
                })
            })
            .and_then(|value| {
                serde_json::to_string_pretty(&value).map_err(|error| {
                    autodev_planning_revision::ValidationError::new(error.to_string())
                })
            }) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("ERROR: {error}");
                ExitCode::FAILURE
            }
        }
    } else {
        match validate(&project_root) {
            Ok(_) => {
                println!("Planning revision valid.");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("ERROR: {error}");
                ExitCode::FAILURE
            }
        }
    }
}
