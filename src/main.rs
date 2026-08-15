use autodev_planning_revision::{
    ValidationError, project_revision, task_source_projection, validate,
};
use serde::Serialize;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    let first = arguments.next();
    let command = first.as_deref().and_then(|value| value.to_str());
    let result = match command {
        Some("--print-task-projection") => {
            one_root(&mut arguments).and_then(|root| task_source_projection(&root).and_then(json))
        }
        Some("--print-validated-task-projection") => one_root(&mut arguments).and_then(|root| {
            validate(&root)
                .and_then(|value| {
                    value.ok_or_else(|| {
                        ValidationError::new(
                            "configured task source is not a rooted github_issues source",
                        )
                    })
                })
                .and_then(json)
        }),
        Some("--print-project-revision") => {
            one_root(&mut arguments).and_then(|root| project_revision(&root).and_then(json))
        }
        _ => {
            let root = first
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            if arguments.next().is_some() {
                Err(ValidationError::new("expected one project root"))
            } else {
                validate(&root).map(|_| "Planning revision valid.".to_owned())
            }
        }
    };

    match result {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("ERROR: {error}");
            ExitCode::FAILURE
        }
    }
}

fn one_root(
    arguments: &mut impl Iterator<Item = std::ffi::OsString>,
) -> Result<PathBuf, ValidationError> {
    let root = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    if arguments.next().is_some() {
        return Err(ValidationError::new("expected one project root"));
    }
    Ok(root)
}

fn json(value: impl Serialize) -> Result<String, ValidationError> {
    serde_json::to_string_pretty(&value).map_err(|error| ValidationError::new(error.to_string()))
}
