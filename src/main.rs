use autodev_planning_revision::{
    CliCommand, DependencyStatus, EpisodeEvent, ReadyEvent, Result, TaskAuthorization,
    ValidationError, VerifiedEvidence, authorize_task_with_api, complete_episode_merge,
    dependencies_ready, parse_cli, project_revision, request_github, task_snapshot,
    task_source_projection, transition_episode_with_api, validate,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let result = parse_cli(&arguments).and_then(run);

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

fn run(command: CliCommand) -> Result<String> {
    match command {
        CliCommand::Validate { root } => {
            validate(&root).map(|_| "Planning revision valid.".to_owned())
        }
        CliCommand::PrintProjectRevision { root } => project_revision(&root).and_then(json),
        CliCommand::PrintTaskProjection { root } => task_source_projection(&root).and_then(json),
        CliCommand::PrintValidatedTaskProjection { root } => validate(&root)
            .and_then(|value| {
                value.ok_or_else(|| {
                    ValidationError::new(
                        "configured task source is not a rooted github_issues source",
                    )
                })
            })
            .and_then(json),
        CliCommand::PrintTaskSnapshot { root, issue } => task_snapshot(&root, issue).and_then(json),
        CliCommand::Authorize {
            root,
            issue,
            event,
            agent_input,
            prior,
        } => {
            let validated = task_snapshot(&root, issue)?;
            let event: ReadyEvent = read_json(&event, "ready event")?;
            let agent_input = read_bytes(&agent_input, "agent input")?;
            let prior: Vec<TaskAuthorization> = match prior {
                Some(path) => read_json(&path, "prior authorization record")?,
                None => Vec::new(),
            };
            authorize_task_with_api(
                &validated.project_revision,
                &validated.task_snapshot,
                &agent_input,
                &event,
                &prior,
                &request_github,
            )
            .and_then(json)
        }
        CliCommand::Transition {
            root,
            event,
            current,
        } => {
            let project_revision = project_revision(&root)?;
            let event: EpisodeEvent = read_json(&event, "episode event")?;
            let current: TaskAuthorization = read_json(&current, "current authorization record")?;
            transition_episode_with_api(&project_revision, &current, &event, &request_github)
                .and_then(json)
        }
        CliCommand::CompleteMerge {
            root,
            current,
            evidence,
            merged_into,
        } => {
            let project_revision = project_revision(&root)?;
            let current: TaskAuthorization = read_json(&current, "current authorization record")?;
            let evidence: VerifiedEvidence = read_json(&evidence, "verified evidence")?;
            complete_episode_merge(&project_revision, &current, &evidence, &merged_into)
                .and_then(json)
        }
        CliCommand::DependenciesReady {
            root,
            issue,
            statuses,
        } => {
            let validated = task_snapshot(&root, issue)?;
            let statuses: Vec<DependencyStatus> = read_json(&statuses, "dependency statuses")?;
            let ready = dependencies_ready(
                &validated.project_revision,
                &validated.task_snapshot,
                &statuses,
            )?;
            json(serde_json::json!({ "ready": ready }))
        }
    }
}

fn read_bytes(path: &Path, label: &str) -> Result<Vec<u8>> {
    fs::read(path).map_err(|error| {
        ValidationError::new(format!(
            "{label} is unreadable at {}: {error}",
            path.display()
        ))
    })
}

fn read_json<T: DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    let bytes = read_bytes(path, label)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| ValidationError::new(format!("{label} is invalid: {error}")))
}

fn json(value: impl Serialize) -> Result<String> {
    serde_json::to_string_pretty(&value).map_err(|error| ValidationError::new(error.to_string()))
}
