use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::hash::Hash;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const CONFIG_PATH: &str = ".autodev/config.yaml";
const APPROVAL_PATH: &str = ".autodev/approval.yaml";
pub const GITHUB_API_VERSION: &str = "2026-03-10";

pub type Result<T> = std::result::Result<T, ValidationError>;
type GitHubApi<'a> = dyn Fn(&str, bool) -> Result<JsonValue> + 'a;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationError(String);

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    project_overview: String,
    #[serde(default)]
    task_graph: Option<String>,
    #[serde(default)]
    task_source: Option<TaskSource>,
    #[serde(default)]
    knowledge_roots: Vec<String>,
    #[serde(default)]
    learning_candidate_inbox: Option<String>,
    #[serde(default)]
    authorization: Option<AuthorizationConfig>,
    #[serde(default)]
    delivery: Option<DeliveryConfig>,
    #[serde(default)]
    engine_policy: Option<EnginePolicyConfig>,
    #[serde(default)]
    selected_engine: Option<SelectedEngineConfig>,
    #[serde(default)]
    tools: BTreeMap<String, ToolConfig>,
    #[serde(default)]
    knowledge: Option<KnowledgeConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, tag = "type")]
enum TaskSource {
    #[serde(rename = "local_file")]
    LocalFile { path: String },
    #[serde(rename = "github_issues")]
    GitHubIssues {
        repository: String,
        #[serde(default)]
        root_issue: Option<u64>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskGraph {
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    required_planning_transition: Option<PlanningTransition>,
    tasks: Vec<LocalTask>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanningTransition {
    after_task: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct LocalTask {
    id: String,
    title: String,
    outcome: String,
    depends_on: Vec<String>,
    references: Vec<String>,
    verify: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Approval {
    status: Option<String>,
    approved_by: Option<String>,
    approved_at: Option<String>,
    #[serde(default)]
    files: Option<BTreeMap<String, String>>,
    #[serde(default)]
    planning_revision: Option<PlanningRevision>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanningRevision {
    project_overview: OverviewRevision,
    #[serde(default)]
    task_source: Option<GitHubTaskSourceRevision>,
    #[serde(default)]
    project: Option<ProjectRevisionRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct OverviewRevision {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GitHubTaskSourceRevision {
    #[serde(rename = "type")]
    kind: String,
    repository: String,
    root_issue: u64,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectRevisionRecord {
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorizationConfig {
    ready_label: String,
    #[serde(default)]
    refusal_labels: Vec<String>,
    authorizer_roles: Vec<String>,
    cancel_roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeliveryConfig {
    base_branch: String,
    protected_paths: Vec<String>,
    required_checks: Vec<String>,
    review: ReviewConfig,
    merge_mode: String,
    correction_budget: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewConfig {
    required_approvals: u32,
    require_code_owner_review: bool,
    require_thread_resolution: bool,
    require_up_to_date_branch: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnginePolicyConfig {
    allowed_providers: Vec<String>,
    data_use_boundary: String,
    cost_class: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectedEngineConfig {
    provider: String,
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolConfig {
    purpose: String,
    interface: String,
    permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KnowledgeConfig {
    #[serde(default)]
    sources: Vec<KnowledgeSourceConfig>,
    candidate_carrier: KnowledgeDestinationConfig,
    target: KnowledgeDestinationConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KnowledgeSourceConfig {
    alias: String,
    #[serde(default)]
    visibility: Visibility,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KnowledgeDestinationConfig {
    identity: String,
    visibility: Visibility,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum Visibility {
    #[default]
    Private,
    Internal,
    Public,
}

#[derive(Clone, Debug, Deserialize)]
struct GitHubIssue {
    id: u64,
    #[serde(default)]
    node_id: Option<String>,
    number: u64,
    title: String,
    body: Option<String>,
    repository_url: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    labels: Vec<GitHubLabel>,
    #[serde(default)]
    pull_request: Option<JsonValue>,
}

#[derive(Clone, Debug, Deserialize)]
struct GitHubLabel {
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GitHubRepository {
    id: u64,
    node_id: String,
    full_name: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPermission {
    permission: String,
    #[serde(default)]
    role_name: Option<String>,
    user: GitHubPermissionUser,
}

#[derive(Debug, Deserialize)]
struct GitHubPermissionUser {
    id: u64,
    login: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectedIssue {
    id: u64,
    number: u64,
    title: String,
    body: Option<String>,
    parent_id: Option<u64>,
    position: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DependencyEdge {
    blocking_id: u64,
    blocked_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GitHubIssueProjection {
    repository: String,
    root_issue: u64,
    issues: Vec<ProjectedIssue>,
    dependencies: Vec<DependencyEdge>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectionOutput {
    pub sha256: String,
    pub projection: GitHubIssueProjection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectRevisionOutput {
    pub sha256: String,
    pub projection: ProjectRevisionProjection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectRevisionProjection {
    schema_version: u8,
    project_overview: OverviewRevision,
    task_source: ProjectTaskSource,
    authorization: ProjectAuthorization,
    delivery: ProjectDelivery,
    engine_policy: ProjectEnginePolicy,
    tools: BTreeMap<String, ProjectTool>,
    knowledge: ProjectKnowledge,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProjectTaskSource {
    #[serde(rename = "type")]
    kind: &'static str,
    repository_id: u64,
    repository_node_id: String,
    repository: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProjectAuthorization {
    ready_label: String,
    refusal_labels: Vec<String>,
    authorizer_roles: Vec<String>,
    cancel_roles: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProjectDelivery {
    base_branch: String,
    protected_paths: Vec<String>,
    required_checks: Vec<String>,
    review: ReviewConfig,
    merge_mode: String,
    correction_budget: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProjectEnginePolicy {
    allowed_providers: Vec<String>,
    data_use_boundary: String,
    cost_class: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProjectTool {
    purpose: String,
    interface: String,
    permissions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProjectKnowledge {
    sources: Vec<ProjectKnowledgeSource>,
    candidate_carrier: ProjectKnowledgeDestination,
    target: ProjectKnowledgeDestination,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProjectKnowledgeSource {
    alias: String,
    visibility: Visibility,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ProjectKnowledgeDestination {
    identity: String,
    visibility: Visibility,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValidatedTaskSnapshot {
    pub project_revision: ProjectRevisionOutput,
    pub task_snapshot: TaskSnapshotOutput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TaskSnapshotOutput {
    pub sha256: String,
    pub projection: TaskSnapshotProjection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TaskSnapshotProjection {
    schema_version: u8,
    repository_node_id: String,
    issue: TaskIssueIdentity,
    title: String,
    body: Option<String>,
    blocked_by: Vec<TaskIssueIdentity>,
    project_revision_sha256: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TaskIssueIdentity {
    issue_id: u64,
    issue_node_id: String,
    issue_number: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReadyEvent {
    pub repository_id: u64,
    pub run_id: u64,
    pub issue_id: u64,
    pub issue_node_id: String,
    pub issue_number: u64,
    pub label: String,
    pub actor_id: u64,
    pub actor: String,
    pub actor_role: String,
    pub task_sha256: String,
    pub project_revision_sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeStatus {
    Active,
    Suspended,
    SupersessionPending,
    Superseded,
    Abandoned,
    Merged,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskAuthorization {
    pub episode: EpisodeIdentity,
    pub project_revision_sha256: String,
    pub agent_input_sha256: String,
    pub authorization_event: AuthorizationEventIdentity,
    pub authorizing_actor: String,
    pub authorizing_actor_id: u64,
    pub authorizing_role: String,
    pub status: EpisodeStatus,
    pub cleanup_complete: bool,
    pub replacement: Option<EpisodeReplacement>,
    #[serde(default)]
    pub processed_transition_events: Vec<TransitionEventIdentity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpisodeReplacement {
    pub task_sha256: String,
    pub project_revision_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpisodeIdentity {
    pub repository_id: u64,
    pub repository_node_id: String,
    pub repository: String,
    pub issue_number: u64,
    pub task_sha256: String,
    pub authorization_generation: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AuthorizationEventIdentity {
    pub repository_id: u64,
    pub run_id: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TransitionEventIdentity {
    pub repository_id: u64,
    pub event_id: u64,
    pub kind: EpisodeEventKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationAction {
    Start,
    Replay,
    AlreadyActive,
    SupersessionRequired,
    Merged,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AuthorizationDecision {
    pub action: AuthorizationAction,
    pub authorization: TaskAuthorization,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeEventKind {
    NeedsInputRecorded,
    OperationalResume,
    IssueEdited,
    ReadyRemoved,
    IssueClosed,
    CancelCommand,
    PullRequestClosedWithoutMerge,
    IssueReopened,
    CleanupCompleted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpisodeEvent {
    pub kind: EpisodeEventKind,
    pub repository_id: u64,
    pub event_id: u64,
    pub episode: EpisodeIdentity,
    pub project_revision_sha256: String,
    pub actor: Option<RepositoryActor>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RepositoryActor {
    pub id: u64,
    pub login: String,
    pub role: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionAction {
    Updated,
    GateFailed,
    NoOp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TransitionDecision {
    pub action: TransitionAction,
    pub authorization: TaskAuthorization,
    pub cleanup_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyStatus {
    pub issue_node_id: String,
    pub approved_task_sha256: Option<String>,
    pub project_revision_sha256: String,
    pub authorization_generation: u64,
    pub evidence_verified: bool,
    pub evidence_task_sha256: String,
    pub evidence_project_revision_sha256: String,
    pub evidence_authorization_generation: u64,
    pub merged_into: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedEvidence {
    pub task_sha256: String,
    pub project_revision_sha256: String,
    pub authorization_generation: u64,
    pub verified: bool,
}

pub fn validate(project_root: &Path) -> Result<Option<ProjectionOutput>> {
    validate_with_api(project_root, &request_github)
}

pub fn validate_with_api(
    project_root: &Path,
    github_api: &GitHubApi<'_>,
) -> Result<Option<ProjectionOutput>> {
    let root = canonical_project_root(project_root)?;
    let config_path = project_file(&root, CONFIG_PATH, "config")?;
    let approval_path = project_file(&root, APPROVAL_PATH, "approval record")?;
    let config_bytes = read_file(&config_path, "config")?;
    let approval_bytes = read_file(&approval_path, "approval record")?;
    let config: Config = parse_yaml(&config_bytes, "config")?;
    let approval: Approval = parse_yaml(&approval_bytes, "approval record")?;
    validate_approval_identity(&approval)?;
    let overview_path = project_file(&root, &config.project_overview, "project_overview")?;
    let overview_bytes = read_file(&overview_path, "project overview")?;
    validate_overview(&overview_bytes)?;

    match configured_task_source(&config)? {
        TaskSource::LocalFile { path } => {
            let tasks_path = project_file(&root, &path, "task graph")?;
            let tasks_bytes = read_file(&tasks_path, "task graph")?;
            validate_local_tasks(&root, &tasks_bytes)?;
            let planned_files = BTreeMap::from([
                (config.project_overview.as_str(), overview_bytes.as_slice()),
                (path.as_str(), tasks_bytes.as_slice()),
            ]);
            validate_local_approval(&approval, &planned_files)?;
            Ok(None)
        }
        source @ TaskSource::GitHubIssues {
            root_issue: Some(_),
            ..
        } => {
            let approved_projection = validate_github_approval_metadata(
                &approval,
                &config.project_overview,
                &overview_bytes,
                &source,
            )?;
            let projection =
                projection_output(github_issue_projection(&root, &source, github_api)?)?;
            if projection.sha256 != approved_projection {
                return Err(ValidationError::new(
                    "approval task_source does not match the current GitHub Issue Graph",
                ));
            }
            Ok(Some(projection))
        }
        TaskSource::GitHubIssues {
            root_issue: None, ..
        } => {
            let revision = build_project_revision(
                &config,
                &config.project_overview,
                &overview_bytes,
                github_api,
            )?;
            validate_project_approval(&approval, &revision)?;
            Ok(None)
        }
    }
}

pub fn task_source_projection(project_root: &Path) -> Result<ProjectionOutput> {
    task_source_projection_with_api(project_root, &request_github)
}

pub fn task_source_projection_with_api(
    project_root: &Path,
    github_api: &GitHubApi<'_>,
) -> Result<ProjectionOutput> {
    let root = canonical_project_root(project_root)?;
    let config_path = project_file(&root, CONFIG_PATH, "config")?;
    let config_bytes = read_file(&config_path, "config")?;
    let config: Config = parse_yaml(&config_bytes, "config")?;
    let source = configured_task_source(&config)?;
    if !matches!(
        source,
        TaskSource::GitHubIssues {
            root_issue: Some(_),
            ..
        }
    ) {
        return Err(ValidationError::new(
            "configured task source is not a rooted github_issues source",
        ));
    }
    projection_output(github_issue_projection(&root, &source, github_api)?)
}

pub fn project_revision(project_root: &Path) -> Result<ProjectRevisionOutput> {
    project_revision_with_api(project_root, &request_github)
}

pub fn project_revision_with_api(
    project_root: &Path,
    github_api: &GitHubApi<'_>,
) -> Result<ProjectRevisionOutput> {
    let root = canonical_project_root(project_root)?;
    let config_path = project_file(&root, CONFIG_PATH, "config")?;
    let config_bytes = read_file(&config_path, "config")?;
    let config: Config = parse_yaml(&config_bytes, "config")?;
    let overview_path = project_file(&root, &config.project_overview, "project_overview")?;
    let overview_bytes = read_file(&overview_path, "project overview")?;
    validate_overview(&overview_bytes)?;
    build_project_revision(
        &config,
        &config.project_overview,
        &overview_bytes,
        github_api,
    )
}

pub fn task_snapshot(project_root: &Path, issue_number: u64) -> Result<ValidatedTaskSnapshot> {
    task_snapshot_with_api(project_root, issue_number, &request_github)
}

pub fn task_snapshot_with_api(
    project_root: &Path,
    issue_number: u64,
    github_api: &GitHubApi<'_>,
) -> Result<ValidatedTaskSnapshot> {
    if issue_number == 0 {
        return Err(ValidationError::new(
            "task issue number must be a positive integer",
        ));
    }
    let root = canonical_project_root(project_root)?;
    let config_path = project_file(&root, CONFIG_PATH, "config")?;
    let approval_path = project_file(&root, APPROVAL_PATH, "approval record")?;
    let config_bytes = read_file(&config_path, "config")?;
    let approval_bytes = read_file(&approval_path, "approval record")?;
    let config: Config = parse_yaml(&config_bytes, "config")?;
    let approval: Approval = parse_yaml(&approval_bytes, "approval record")?;
    validate_approval_identity(&approval)?;
    let overview_path = project_file(&root, &config.project_overview, "project_overview")?;
    let overview_bytes = read_file(&overview_path, "project overview")?;
    validate_overview(&overview_bytes)?;
    let project_revision = build_project_revision(
        &config,
        &config.project_overview,
        &overview_bytes,
        github_api,
    )?;
    validate_project_approval(&approval, &project_revision)?;
    let task_snapshot = build_task_snapshot(&root, &project_revision, issue_number, github_api)?;
    Ok(ValidatedTaskSnapshot {
        project_revision,
        task_snapshot,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    Validate {
        root: PathBuf,
    },
    PrintProjectRevision {
        root: PathBuf,
    },
    PrintTaskProjection {
        root: PathBuf,
    },
    PrintValidatedTaskProjection {
        root: PathBuf,
    },
    PrintTaskSnapshot {
        root: PathBuf,
        issue: u64,
    },
    Authorize {
        root: PathBuf,
        issue: u64,
        event: PathBuf,
        agent_input: PathBuf,
        prior: Option<PathBuf>,
    },
    Transition {
        root: PathBuf,
        event: PathBuf,
        current: PathBuf,
    },
}

/// Parses the delivery adapter's command line. The three original print
/// commands keep their positional project root so recorded evidence commands
/// stay runnable.
pub fn parse_cli(arguments: &[String]) -> Result<CliCommand> {
    let Some((command, rest)) = arguments.split_first() else {
        return Ok(CliCommand::Validate {
            root: PathBuf::from("."),
        });
    };
    match command.as_str() {
        "--print-project-revision" => Ok(CliCommand::PrintProjectRevision {
            root: positional_root(rest)?,
        }),
        "--print-task-projection" => Ok(CliCommand::PrintTaskProjection {
            root: positional_root(rest)?,
        }),
        "--print-validated-task-projection" => Ok(CliCommand::PrintValidatedTaskProjection {
            root: positional_root(rest)?,
        }),
        "--print-task-snapshot" => {
            let mut options = CliOptions::parse(rest, command)?;
            let command = CliCommand::PrintTaskSnapshot {
                root: options.path("root")?,
                issue: options.issue()?,
            };
            options.finish()?;
            Ok(command)
        }
        "--authorize" => {
            let mut options = CliOptions::parse(rest, command)?;
            let command = CliCommand::Authorize {
                root: options.path("root")?,
                issue: options.issue()?,
                event: options.path("event")?,
                agent_input: options.path("agent-input")?,
                prior: options.optional_path("prior"),
            };
            options.finish()?;
            Ok(command)
        }
        "--transition" => {
            let mut options = CliOptions::parse(rest, command)?;
            let command = CliCommand::Transition {
                root: options.path("root")?,
                event: options.path("event")?,
                current: options.path("current")?,
            };
            options.finish()?;
            Ok(command)
        }
        unsupported if unsupported.starts_with("--") => Err(ValidationError::new(format!(
            "unsupported command: {unsupported}"
        ))),
        _ => Ok(CliCommand::Validate {
            root: positional_root(arguments)?,
        }),
    }
}

struct CliOptions {
    command: String,
    values: BTreeMap<String, String>,
}

impl CliOptions {
    fn parse(arguments: &[String], command: &str) -> Result<Self> {
        let mut values = BTreeMap::new();
        let mut remaining = arguments.iter();
        while let Some(argument) = remaining.next() {
            let name = argument.strip_prefix("--").ok_or_else(|| {
                ValidationError::new(format!("{command} expects --name value options"))
            })?;
            let value = remaining.next().ok_or_else(|| {
                ValidationError::new(format!("{command} option --{name} is missing its value"))
            })?;
            if values.insert(name.to_owned(), value.clone()).is_some() {
                return Err(ValidationError::new(format!(
                    "{command} option --{name} is repeated"
                )));
            }
        }
        Ok(Self {
            command: command.to_owned(),
            values,
        })
    }

    fn path(&mut self, name: &str) -> Result<PathBuf> {
        self.optional_path(name)
            .ok_or_else(|| ValidationError::new(format!("{} requires --{name}", self.command)))
    }

    fn optional_path(&mut self, name: &str) -> Option<PathBuf> {
        self.values
            .remove(name)
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }

    fn issue(&mut self) -> Result<u64> {
        let value = self
            .values
            .remove("issue")
            .ok_or_else(|| ValidationError::new(format!("{} requires --issue", self.command)))?;
        value
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|issue| *issue > 0)
            .ok_or_else(|| {
                ValidationError::new("task issue number must be a positive integer".to_owned())
            })
    }

    fn finish(self) -> Result<()> {
        match self.values.keys().next() {
            Some(unsupported) => Err(ValidationError::new(format!(
                "{} does not accept --{unsupported}",
                self.command
            ))),
            None => Ok(()),
        }
    }
}

fn positional_root(arguments: &[String]) -> Result<PathBuf> {
    match arguments {
        [] => Ok(PathBuf::from(".")),
        [root] => Ok(PathBuf::from(root)),
        _ => Err(ValidationError::new("expected one project root")),
    }
}

fn authorize_task(
    project_revision: &ProjectRevisionOutput,
    task_snapshot: &TaskSnapshotOutput,
    integrity_filtered_agent_input: &[u8],
    event: &ReadyEvent,
    prior: &[TaskAuthorization],
) -> Result<AuthorizationDecision> {
    let source = &project_revision.projection.task_source;
    let policy = &project_revision.projection.authorization;
    if event.repository_id != source.repository_id
        || event.issue_id != task_snapshot.projection.issue.issue_id
        || event.issue_node_id != task_snapshot.projection.issue.issue_node_id
        || event.issue_number != task_snapshot.projection.issue.issue_number
        || event.label != policy.ready_label
        || event.actor_id == 0
        || event.run_id == 0
        || event.actor.trim().is_empty()
        || !policy.authorizer_roles.contains(&event.actor_role)
        || event.task_sha256 != task_snapshot.sha256
        || event.project_revision_sha256 != project_revision.sha256
    {
        return Err(ValidationError::new(
            "ready event is not authorized by the approved project revision",
        ));
    }
    let event_identity = AuthorizationEventIdentity {
        repository_id: event.repository_id,
        run_id: event.run_id,
    };
    let agent_input_sha256 = sha256_hex(integrity_filtered_agent_input);
    let mut matching = prior
        .iter()
        .filter(|record| {
            record.episode.repository_id == source.repository_id
                && record.episode.repository_node_id == source.repository_node_id
                && record.episode.issue_number == event.issue_number
        })
        .collect::<Vec<_>>();
    matching.sort_by_key(|record| record.episode.authorization_generation);
    let mut generations = HashSet::new();
    let mut events = HashSet::new();
    for record in &matching {
        if !generations.insert(record.episode.authorization_generation)
            || !events.insert(&record.authorization_event)
        {
            return Err(ValidationError::new(
                "task authorization history contains conflicting records",
            ));
        }
    }
    if let Some(existing) = matching
        .iter()
        .find(|record| record.authorization_event == event_identity)
    {
        if existing.episode.task_sha256 != task_snapshot.sha256
            || existing.project_revision_sha256 != project_revision.sha256
            || existing.agent_input_sha256 != agent_input_sha256
            || existing.authorizing_actor_id != event.actor_id
        {
            return Err(ValidationError::new(
                "ready event replay does not match its authorization record",
            ));
        }
        return Ok(AuthorizationDecision {
            action: AuthorizationAction::Replay,
            authorization: (*existing).clone(),
        });
    }

    if let Some(existing) = matching.last() {
        let same_snapshot = existing.episode.task_sha256 == task_snapshot.sha256
            && existing.project_revision_sha256 == project_revision.sha256;
        match existing.status {
            EpisodeStatus::Merged => {
                return Ok(AuthorizationDecision {
                    action: AuthorizationAction::Merged,
                    authorization: (*existing).clone(),
                });
            }
            EpisodeStatus::Active | EpisodeStatus::Suspended if same_snapshot => {
                return Ok(AuthorizationDecision {
                    action: AuthorizationAction::AlreadyActive,
                    authorization: (*existing).clone(),
                });
            }
            EpisodeStatus::Active
            | EpisodeStatus::Suspended
            | EpisodeStatus::SupersessionPending => {
                let mut superseding = (*existing).clone();
                superseding.status = EpisodeStatus::SupersessionPending;
                superseding.replacement = Some(EpisodeReplacement {
                    task_sha256: task_snapshot.sha256.clone(),
                    project_revision_sha256: project_revision.sha256.clone(),
                });
                if superseding.cleanup_complete {
                    superseding.status = EpisodeStatus::Superseded;
                }
                return Ok(AuthorizationDecision {
                    action: AuthorizationAction::SupersessionRequired,
                    authorization: superseding,
                });
            }
            EpisodeStatus::Superseded | EpisodeStatus::Abandoned if !existing.cleanup_complete => {
                return Ok(AuthorizationDecision {
                    action: AuthorizationAction::SupersessionRequired,
                    authorization: (*existing).clone(),
                });
            }
            EpisodeStatus::Superseded => {
                let replacement = existing.replacement.as_ref().ok_or_else(|| {
                    ValidationError::new("superseded episode is missing replacement digests")
                })?;
                if replacement.task_sha256 != task_snapshot.sha256
                    || replacement.project_revision_sha256 != project_revision.sha256
                {
                    return Err(ValidationError::new(
                        "new authorization does not match the recorded supersession",
                    ));
                }
                return start_authorization(
                    source,
                    task_snapshot,
                    event,
                    event_identity,
                    agent_input_sha256,
                    next_generation(existing)?,
                );
            }
            EpisodeStatus::Abandoned => {
                return start_authorization(
                    source,
                    task_snapshot,
                    event,
                    event_identity,
                    agent_input_sha256,
                    next_generation(existing)?,
                );
            }
        }
    }

    start_authorization(
        source,
        task_snapshot,
        event,
        event_identity,
        agent_input_sha256,
        1,
    )
}

pub fn authorize_task_with_api(
    project_revision: &ProjectRevisionOutput,
    task_snapshot: &TaskSnapshotOutput,
    integrity_filtered_agent_input: &[u8],
    event: &ReadyEvent,
    prior: &[TaskAuthorization],
    github_api: &GitHubApi<'_>,
) -> Result<AuthorizationDecision> {
    validate_task_snapshot_integrity(project_revision, task_snapshot)?;
    let decision = authorize_task(
        project_revision,
        task_snapshot,
        integrity_filtered_agent_input,
        event,
        prior,
    )?;
    if decision.action == AuthorizationAction::Replay {
        return Ok(decision);
    }
    validate_repository_actor(
        project_revision,
        &RepositoryActor {
            id: event.actor_id,
            login: event.actor.clone(),
            role: event.actor_role.clone(),
        },
        github_api,
    )?;
    Ok(decision)
}

pub fn transition_episode_with_api(
    project_revision: &ProjectRevisionOutput,
    current: &TaskAuthorization,
    event: &EpisodeEvent,
    github_api: &GitHubApi<'_>,
) -> Result<TransitionDecision> {
    validate_project_revision_integrity(project_revision)?;
    if event.repository_id == 0
        || event.event_id == 0
        || event.repository_id != current.episode.repository_id
        || event.episode != current.episode
        || event.project_revision_sha256 != current.project_revision_sha256
    {
        return Err(ValidationError::new(
            "episode event does not match the current episode",
        ));
    }
    let event_identity = TransitionEventIdentity {
        repository_id: event.repository_id,
        event_id: event.event_id,
        kind: event.kind,
    };
    if let Some(processed) = current
        .processed_transition_events
        .iter()
        .find(|processed| {
            processed.repository_id == event_identity.repository_id
                && processed.event_id == event_identity.event_id
        })
    {
        if processed.kind != event_identity.kind {
            return Err(ValidationError::new(
                "episode event ID was already used for another transition",
            ));
        }
        return Ok(transition_decision(
            TransitionAction::NoOp,
            current.clone(),
            false,
        ));
    }
    let actor_required = matches!(
        event.kind,
        EpisodeEventKind::IssueEdited
            | EpisodeEventKind::OperationalResume
            | EpisodeEventKind::ReadyRemoved
            | EpisodeEventKind::IssueClosed
            | EpisodeEventKind::CancelCommand
            | EpisodeEventKind::PullRequestClosedWithoutMerge
    );
    let actor_role = if actor_required {
        let actor = event.actor.as_ref().ok_or_else(|| {
            ValidationError::new("episode transition is missing its repository actor")
        })?;
        validate_repository_actor(project_revision, actor, github_api)?;
        Some(actor.role.as_str())
    } else {
        None
    };
    let mut decision = transition_episode(project_revision, current, event, actor_role)?;
    decision
        .authorization
        .processed_transition_events
        .push(event_identity);
    Ok(decision)
}

fn transition_episode(
    project_revision: &ProjectRevisionOutput,
    current: &TaskAuthorization,
    event: &EpisodeEvent,
    actor_role: Option<&str>,
) -> Result<TransitionDecision> {
    let replacing_with_project = event.kind == EpisodeEventKind::CleanupCompleted
        && current.status == EpisodeStatus::SupersessionPending
        && current.replacement.as_ref().is_some_and(|replacement| {
            replacement.project_revision_sha256 == project_revision.sha256
        });
    if current.project_revision_sha256 != project_revision.sha256 && !replacing_with_project {
        return Err(ValidationError::new(
            "episode belongs to another project revision",
        ));
    }
    if current.status == EpisodeStatus::Merged {
        return Ok(transition_decision(
            TransitionAction::NoOp,
            current.clone(),
            false,
        ));
    }
    if current.status == EpisodeStatus::Superseded {
        return Ok(transition_decision(
            TransitionAction::NoOp,
            current.clone(),
            false,
        ));
    }
    if current.status == EpisodeStatus::Abandoned
        && !matches!(
            event.kind,
            EpisodeEventKind::CleanupCompleted | EpisodeEventKind::IssueReopened
        )
    {
        return Ok(transition_decision(
            TransitionAction::NoOp,
            current.clone(),
            false,
        ));
    }

    let mut next = current.clone();
    let policy = &project_revision.projection.authorization;
    let actor_is_authorizer = actor_role.is_some_and(|role| {
        policy
            .authorizer_roles
            .iter()
            .any(|allowed| allowed == role)
    });
    let actor_can_cancel =
        actor_role.is_some_and(|role| policy.cancel_roles.iter().any(|allowed| allowed == role));

    match event.kind {
        EpisodeEventKind::NeedsInputRecorded if next.status == EpisodeStatus::Active => {
            next.status = EpisodeStatus::Suspended;
            Ok(transition_decision(TransitionAction::Updated, next, false))
        }
        EpisodeEventKind::NeedsInputRecorded if next.status == EpisodeStatus::Suspended => {
            Ok(transition_decision(TransitionAction::NoOp, next, false))
        }
        EpisodeEventKind::NeedsInputRecorded => Ok(transition_decision(
            TransitionAction::GateFailed,
            next,
            false,
        )),
        EpisodeEventKind::OperationalResume
            if next.status == EpisodeStatus::Suspended && actor_is_authorizer =>
        {
            next.status = EpisodeStatus::Active;
            Ok(transition_decision(TransitionAction::Updated, next, false))
        }
        EpisodeEventKind::OperationalResume => Ok(transition_decision(
            TransitionAction::GateFailed,
            next,
            false,
        )),
        EpisodeEventKind::IssueEdited if actor_is_authorizer => {
            next.status = EpisodeStatus::SupersessionPending;
            next.cleanup_complete = false;
            next.replacement = None;
            Ok(transition_decision(TransitionAction::Updated, next, true))
        }
        EpisodeEventKind::IssueEdited => Ok(transition_decision(
            TransitionAction::GateFailed,
            next,
            false,
        )),
        EpisodeEventKind::ReadyRemoved
        | EpisodeEventKind::IssueClosed
        | EpisodeEventKind::CancelCommand
        | EpisodeEventKind::PullRequestClosedWithoutMerge
            if actor_can_cancel =>
        {
            next.status = EpisodeStatus::Abandoned;
            next.cleanup_complete = false;
            Ok(transition_decision(TransitionAction::Updated, next, true))
        }
        EpisodeEventKind::ReadyRemoved
        | EpisodeEventKind::IssueClosed
        | EpisodeEventKind::CancelCommand
        | EpisodeEventKind::PullRequestClosedWithoutMerge => Ok(transition_decision(
            TransitionAction::GateFailed,
            next,
            false,
        )),
        EpisodeEventKind::IssueReopened => {
            Ok(transition_decision(TransitionAction::NoOp, next, false))
        }
        EpisodeEventKind::CleanupCompleted => match next.status {
            EpisodeStatus::SupersessionPending if next.cleanup_complete => {
                Ok(transition_decision(TransitionAction::NoOp, next, false))
            }
            EpisodeStatus::SupersessionPending => {
                next.cleanup_complete = true;
                if next.replacement.is_some() {
                    next.status = EpisodeStatus::Superseded;
                }
                Ok(transition_decision(TransitionAction::Updated, next, false))
            }
            EpisodeStatus::Abandoned if !next.cleanup_complete => {
                next.cleanup_complete = true;
                Ok(transition_decision(TransitionAction::Updated, next, false))
            }
            EpisodeStatus::Abandoned => {
                Ok(transition_decision(TransitionAction::NoOp, next, false))
            }
            _ => Err(ValidationError::new(
                "episode has no authorized cleanup to complete",
            )),
        },
    }
}

fn validate_task_snapshot_integrity(
    project_revision: &ProjectRevisionOutput,
    task_snapshot: &TaskSnapshotOutput,
) -> Result<()> {
    validate_project_revision_integrity(project_revision)?;
    if task_snapshot.projection.project_revision_sha256 != project_revision.sha256 {
        return Err(ValidationError::new(
            "task snapshot is bound to another project revision",
        ));
    }
    if json_digest(&task_snapshot.projection, "task snapshot")? != task_snapshot.sha256 {
        return Err(ValidationError::new(
            "task snapshot output has an invalid digest",
        ));
    }
    Ok(())
}

fn validate_project_revision_integrity(project_revision: &ProjectRevisionOutput) -> Result<()> {
    if json_digest(&project_revision.projection, "project revision")? != project_revision.sha256 {
        return Err(ValidationError::new(
            "project revision output has an invalid digest",
        ));
    }
    Ok(())
}

pub fn complete_episode_merge(
    project_revision: &ProjectRevisionOutput,
    current: &TaskAuthorization,
    evidence: &VerifiedEvidence,
    merged_into: &str,
) -> Result<TransitionDecision> {
    validate_project_revision_integrity(project_revision)?;
    if current.project_revision_sha256 != project_revision.sha256
        || current.status != EpisodeStatus::Active
        || !evidence.verified
        || evidence.task_sha256 != current.episode.task_sha256
        || evidence.project_revision_sha256 != current.project_revision_sha256
        || evidence.authorization_generation != current.episode.authorization_generation
        || merged_into != project_revision.projection.delivery.base_branch
    {
        return Err(ValidationError::new(
            "merge completion requires active authorization, verified evidence, and the approved base branch",
        ));
    }
    let mut next = current.clone();
    next.status = EpisodeStatus::Merged;
    next.cleanup_complete = true;
    Ok(transition_decision(TransitionAction::Updated, next, false))
}

pub fn dependencies_ready(
    project_revision: &ProjectRevisionOutput,
    task_snapshot: &TaskSnapshotOutput,
    statuses: &[DependencyStatus],
) -> Result<bool> {
    validate_task_snapshot_integrity(project_revision, task_snapshot)?;
    let base_branch = &project_revision.projection.delivery.base_branch;
    if base_branch.trim().is_empty() {
        return Err(ValidationError::new("dependent base branch is empty"));
    }
    let mut by_issue = HashMap::new();
    for status in statuses {
        if by_issue
            .insert(status.issue_node_id.as_str(), status)
            .is_some()
        {
            return Err(ValidationError::new(
                "dependency status contains a duplicate issue",
            ));
        }
    }
    if by_issue.len() != task_snapshot.projection.blocked_by.len() {
        return Ok(false);
    }
    Ok(task_snapshot
        .projection
        .blocked_by
        .iter()
        .all(|dependency| {
            by_issue
                .get(dependency.issue_node_id.as_str())
                .is_some_and(|status| {
                    let approved_task = status.approved_task_sha256.as_deref();
                    approved_task.is_some_and(is_sha256)
                        && status.project_revision_sha256 == project_revision.sha256
                        && status.authorization_generation > 0
                        && status.evidence_verified
                        && approved_task == Some(status.evidence_task_sha256.as_str())
                        && status.evidence_project_revision_sha256 == status.project_revision_sha256
                        && status.evidence_authorization_generation
                            == status.authorization_generation
                        && status.merged_into.as_deref() == Some(base_branch.as_str())
                })
        }))
}

fn next_generation(current: &TaskAuthorization) -> Result<u64> {
    current
        .episode
        .authorization_generation
        .checked_add(1)
        .ok_or_else(|| ValidationError::new("authorization generation overflow"))
}

fn start_authorization(
    source: &ProjectTaskSource,
    task_snapshot: &TaskSnapshotOutput,
    event: &ReadyEvent,
    event_identity: AuthorizationEventIdentity,
    agent_input_sha256: String,
    generation: u64,
) -> Result<AuthorizationDecision> {
    let authorization = TaskAuthorization {
        episode: EpisodeIdentity {
            repository_id: source.repository_id,
            repository_node_id: source.repository_node_id.clone(),
            repository: source.repository.clone(),
            issue_number: event.issue_number,
            task_sha256: task_snapshot.sha256.clone(),
            authorization_generation: generation,
        },
        project_revision_sha256: task_snapshot.projection.project_revision_sha256.clone(),
        agent_input_sha256,
        authorization_event: event_identity,
        authorizing_actor: event.actor.clone(),
        authorizing_actor_id: event.actor_id,
        authorizing_role: event.actor_role.clone(),
        status: EpisodeStatus::Active,
        cleanup_complete: false,
        replacement: None,
        processed_transition_events: Vec::new(),
    };
    Ok(AuthorizationDecision {
        action: AuthorizationAction::Start,
        authorization,
    })
}

fn transition_decision(
    action: TransitionAction,
    authorization: TaskAuthorization,
    cleanup_required: bool,
) -> TransitionDecision {
    TransitionDecision {
        action,
        authorization,
        cleanup_required,
    }
}

fn build_project_revision(
    config: &Config,
    overview_relative: &str,
    overview_bytes: &[u8],
    github_api: &GitHubApi<'_>,
) -> Result<ProjectRevisionOutput> {
    let TaskSource::GitHubIssues {
        repository,
        root_issue: None,
    } = configured_task_source(config)?
    else {
        return Err(ValidationError::new(
            "project revision requires a rootless github_issues task source",
        ));
    };
    let authorization = config.authorization.as_ref().ok_or_else(|| {
        ValidationError::new("rootless github_issues config is missing authorization")
    })?;
    let delivery = config
        .delivery
        .as_ref()
        .ok_or_else(|| ValidationError::new("rootless github_issues config is missing delivery"))?;
    let engine_policy = config.engine_policy.as_ref().ok_or_else(|| {
        ValidationError::new("rootless github_issues config is missing engine_policy")
    })?;
    let selected_engine = config.selected_engine.as_ref().ok_or_else(|| {
        ValidationError::new("rootless github_issues config is missing selected_engine")
    })?;
    let knowledge = config.knowledge.as_ref().ok_or_else(|| {
        ValidationError::new("rootless github_issues config is missing knowledge")
    })?;

    let repository_value = github_get(github_api, &format!("repos/{repository}"), false)?;
    let github_repository: GitHubRepository = serde_json::from_value(repository_value)
        .map_err(|error| ValidationError::new(format!("GitHub repository is invalid: {error}")))?;
    if github_repository.id == 0
        || github_repository.node_id.trim().is_empty()
        || !github_repository
            .full_name
            .eq_ignore_ascii_case(&repository)
    {
        return Err(ValidationError::new(
            "GitHub repository does not match the configured Task Source",
        ));
    }

    let ready_label = required_text(&authorization.ready_label, "authorization ready_label")?;
    let refusal_labels = normalized_set(
        &authorization.refusal_labels,
        "authorization refusal_labels",
    )?;
    if refusal_labels.contains(&ready_label) {
        return Err(ValidationError::new(
            "authorization ready_label must not be a refusal label",
        ));
    }
    let authorizer_roles = normalized_roles(
        &authorization.authorizer_roles,
        "authorization authorizer_roles",
    )?;
    let cancel_roles = normalized_roles(&authorization.cancel_roles, "authorization cancel_roles")?;

    let base_branch = required_text(&delivery.base_branch, "delivery base_branch")?;
    let protected_paths = normalized_set(&delivery.protected_paths, "delivery protected_paths")?;
    let required_checks = normalized_set(&delivery.required_checks, "delivery required_checks")?;
    if protected_paths.is_empty() || required_checks.is_empty() {
        return Err(ValidationError::new(
            "delivery protected_paths and required_checks must be non-empty",
        ));
    }
    let merge_mode = required_text(&delivery.merge_mode, "delivery merge_mode")?;
    if !matches!(merge_mode.as_str(), "human" | "auto_after_gates") {
        return Err(ValidationError::new(
            "delivery merge_mode must be human or auto_after_gates",
        ));
    }

    let allowed_providers = normalized_set(
        &engine_policy.allowed_providers,
        "engine_policy allowed_providers",
    )?;
    if allowed_providers.is_empty() {
        return Err(ValidationError::new(
            "engine_policy allowed_providers must be non-empty",
        ));
    }
    let selected_provider = required_text(&selected_engine.provider, "selected_engine provider")?;
    required_text(&selected_engine.name, "selected_engine name")?;
    required_text(&selected_engine.version, "selected_engine version")?;
    if !allowed_providers.contains(&selected_provider) {
        return Err(ValidationError::new(
            "selected_engine provider is outside engine_policy",
        ));
    }

    let mut tools = BTreeMap::new();
    for (name, tool) in &config.tools {
        let name = required_text(name, "tool name")?;
        let value = ProjectTool {
            purpose: required_text(&tool.purpose, &format!("tool {name} purpose"))?,
            interface: required_text(&tool.interface, &format!("tool {name} interface"))?,
            permissions: normalized_set(&tool.permissions, &format!("tool {name} permissions"))?,
        };
        if value.permissions.is_empty() || tools.insert(name.clone(), value).is_some() {
            return Err(ValidationError::new(format!(
                "tool {name} must have unique name and non-empty permissions"
            )));
        }
    }

    let mut sources = Vec::new();
    let mut source_aliases = HashSet::new();
    for source in &knowledge.sources {
        let alias = required_text(&source.alias, "knowledge source alias")?;
        if !source_aliases.insert(alias.clone()) {
            return Err(ValidationError::new(format!(
                "duplicate knowledge source alias: {alias}"
            )));
        }
        if source
            .path
            .as_deref()
            .is_some_and(|path| path.trim().is_empty())
        {
            return Err(ValidationError::new(format!(
                "knowledge source {alias} has an empty path"
            )));
        }
        sources.push(ProjectKnowledgeSource {
            alias,
            visibility: source.visibility,
        });
    }
    sources.sort_by(|left, right| left.alias.cmp(&right.alias));
    let candidate_carrier =
        project_knowledge_destination(&knowledge.candidate_carrier, "knowledge candidate_carrier")?;
    if sources
        .iter()
        .any(|source| candidate_carrier.visibility > source.visibility)
    {
        return Err(ValidationError::new(
            "knowledge candidate_carrier is less restrictive than a source",
        ));
    }
    let target = project_knowledge_destination(&knowledge.target, "knowledge target")?;

    for path in &config.knowledge_roots {
        if path.trim().is_empty() {
            return Err(ValidationError::new(
                "knowledge_roots must not contain an empty path",
            ));
        }
    }
    if config
        .learning_candidate_inbox
        .as_deref()
        .is_some_and(|path| path.trim().is_empty())
    {
        return Err(ValidationError::new(
            "learning_candidate_inbox must not be empty",
        ));
    }

    let projection = ProjectRevisionProjection {
        schema_version: 1,
        project_overview: OverviewRevision {
            path: overview_relative.to_owned(),
            sha256: sha256_hex(overview_bytes),
        },
        task_source: ProjectTaskSource {
            kind: "github_issues",
            repository_id: github_repository.id,
            repository_node_id: github_repository.node_id,
            repository: github_repository.full_name,
        },
        authorization: ProjectAuthorization {
            ready_label,
            refusal_labels,
            authorizer_roles,
            cancel_roles,
        },
        delivery: ProjectDelivery {
            base_branch,
            protected_paths,
            required_checks,
            review: delivery.review.clone(),
            merge_mode,
            correction_budget: delivery.correction_budget,
        },
        engine_policy: ProjectEnginePolicy {
            allowed_providers,
            data_use_boundary: required_text(
                &engine_policy.data_use_boundary,
                "engine_policy data_use_boundary",
            )?,
            cost_class: required_text(&engine_policy.cost_class, "engine_policy cost_class")?,
        },
        tools,
        knowledge: ProjectKnowledge {
            sources,
            candidate_carrier,
            target,
        },
    };
    Ok(ProjectRevisionOutput {
        sha256: json_digest(&projection, "project revision")?,
        projection,
    })
}

fn build_task_snapshot(
    root: &Path,
    project_revision: &ProjectRevisionOutput,
    issue_number: u64,
    github_api: &GitHubApi<'_>,
) -> Result<TaskSnapshotOutput> {
    let source = &project_revision.projection.task_source;
    let issue_value = github_get(
        github_api,
        &format!("repos/{}/issues/{issue_number}", source.repository),
        false,
    )?;
    let issue = parse_issue(issue_value, &source.repository, "task issue")?;
    if issue.number != issue_number {
        return Err(ValidationError::new(
            "GitHub task issue does not match the requested issue number",
        ));
    }
    if issue.state.as_deref() != Some("open") {
        return Err(ValidationError::new("GitHub task issue is not open"));
    }
    let labels = issue
        .labels
        .iter()
        .map(|label| label.name.as_str())
        .collect::<HashSet<_>>();
    let policy = &project_revision.projection.authorization;
    if !labels.contains(policy.ready_label.as_str()) {
        return Err(ValidationError::new(
            "GitHub task issue is missing the approved ready label",
        ));
    }
    if policy
        .refusal_labels
        .iter()
        .any(|label| labels.contains(label.as_str()))
    {
        return Err(ValidationError::new(
            "GitHub task issue has an approved refusal label",
        ));
    }
    let projected = ProjectedIssue {
        id: issue.id,
        number: issue.number,
        title: issue.title.clone(),
        body: issue.body.clone(),
        parent_id: None,
        position: 0,
    };
    validate_github_task_body(root, &projected)?;
    let issue_identity = task_issue_identity(&issue)?;
    let endpoint = format!(
        "repos/{}/issues/{issue_number}/dependencies/blocked_by?per_page=100",
        source.repository
    );
    let blockers = json_list(
        github_get(github_api, &endpoint, true)?,
        "GitHub blocked_by",
    )?;
    let mut blocked_by = Vec::new();
    let mut blocker_nodes = HashSet::new();
    for blocker in blockers {
        let blocker = parse_issue(blocker, &source.repository, "dependency endpoint")?;
        let identity = task_issue_identity(&blocker)?;
        if identity.issue_node_id == issue_identity.issue_node_id {
            return Err(ValidationError::new(
                "GitHub task issue cannot block itself",
            ));
        }
        if !blocker_nodes.insert(identity.issue_node_id.clone()) {
            return Err(ValidationError::new(
                "GitHub task issue has a duplicate dependency",
            ));
        }
        blocked_by.push(identity);
    }
    blocked_by.sort();
    let projection = TaskSnapshotProjection {
        schema_version: 1,
        repository_node_id: source.repository_node_id.clone(),
        issue: issue_identity,
        title: issue.title,
        body: issue.body,
        blocked_by,
        project_revision_sha256: project_revision.sha256.clone(),
    };
    Ok(TaskSnapshotOutput {
        sha256: json_digest(&projection, "task snapshot")?,
        projection,
    })
}

fn task_issue_identity(issue: &GitHubIssue) -> Result<TaskIssueIdentity> {
    let issue_node_id = issue
        .node_id
        .as_deref()
        .filter(|node_id| !node_id.trim().is_empty())
        .ok_or_else(|| ValidationError::new("GitHub issue is missing node_id"))?;
    Ok(TaskIssueIdentity {
        issue_id: issue.id,
        issue_node_id: issue_node_id.to_owned(),
        issue_number: issue.number,
    })
}

fn project_knowledge_destination(
    destination: &KnowledgeDestinationConfig,
    label: &str,
) -> Result<ProjectKnowledgeDestination> {
    Ok(ProjectKnowledgeDestination {
        identity: required_text(&destination.identity, label)?,
        visibility: destination.visibility,
    })
}

fn required_text(value: &str, label: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError::new(format!("{label} must be non-empty")));
    }
    Ok(value.to_owned())
}

fn validate_actor_login(login: &str) -> Result<()> {
    if login.is_empty()
        || login.starts_with('-')
        || login.ends_with('-')
        || !login
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(ValidationError::new(
            "ready event actor is not a supported human GitHub login",
        ));
    }
    Ok(())
}

fn validate_repository_actor(
    project_revision: &ProjectRevisionOutput,
    actor: &RepositoryActor,
    github_api: &GitHubApi<'_>,
) -> Result<()> {
    validate_actor_login(&actor.login)?;
    let source = &project_revision.projection.task_source;
    let endpoint = format!(
        "repos/{}/collaborators/{}/permission",
        source.repository, actor.login
    );
    let permission: GitHubPermission =
        serde_json::from_value(github_get(github_api, &endpoint, false)?).map_err(|error| {
            ValidationError::new(format!(
                "GitHub collaborator permission is invalid: {error}"
            ))
        })?;
    let role_matches = permission
        .role_name
        .as_deref()
        .map_or(permission.permission == actor.role, |role| {
            role == actor.role
        });
    if permission.user.id != actor.id
        || !permission.user.login.eq_ignore_ascii_case(&actor.login)
        || !role_matches
    {
        return Err(ValidationError::new(
            "event actor does not match current repository permission",
        ));
    }
    Ok(())
}

fn normalized_set(values: &[String], label: &str) -> Result<Vec<String>> {
    let mut output = values
        .iter()
        .map(|value| required_text(value, label))
        .collect::<Result<Vec<_>>>()?;
    output.sort();
    if output.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ValidationError::new(format!(
            "{label} must not contain duplicates"
        )));
    }
    Ok(output)
}

fn normalized_roles(values: &[String], label: &str) -> Result<Vec<String>> {
    let output = normalized_set(values, label)?;
    if output.is_empty()
        || output
            .iter()
            .any(|role| !matches!(role.as_str(), "write" | "maintain" | "admin"))
    {
        return Err(ValidationError::new(format!(
            "{label} must contain only write, maintain, or admin"
        )));
    }
    Ok(output)
}

fn json_digest(value: &impl Serialize, label: &str) -> Result<String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| ValidationError::new(format!("{label} cannot be serialized: {error}")))?;
    Ok(sha256_hex(&bytes))
}

fn projection_output(projection: GitHubIssueProjection) -> Result<ProjectionOutput> {
    Ok(ProjectionOutput {
        sha256: projection_digest(&projection)?,
        projection,
    })
}

fn canonical_project_root(path: &Path) -> Result<PathBuf> {
    if !path.is_dir() {
        return Err(ValidationError::new(format!(
            "project root does not exist: {}",
            path.display()
        )));
    }
    path.canonicalize()
        .map_err(|error| ValidationError::new(format!("project root cannot be resolved: {error}")))
}

fn configured_task_source(config: &Config) -> Result<TaskSource> {
    match (&config.task_graph, &config.task_source) {
        (Some(_), Some(_)) | (None, None) => Err(ValidationError::new(
            "config must select exactly one task source",
        )),
        (Some(path), None) => Ok(TaskSource::LocalFile { path: path.clone() }),
        (None, Some(source)) => {
            match source {
                TaskSource::LocalFile { path } if path.trim().is_empty() => {
                    return Err(ValidationError::new(
                        "config task_source path must be non-empty",
                    ));
                }
                TaskSource::GitHubIssues {
                    repository,
                    root_issue,
                } => {
                    validate_repository(repository)?;
                    if root_issue.is_some_and(|issue| issue == 0) {
                        return Err(ValidationError::new(
                            "config task_source root_issue must be a positive integer",
                        ));
                    }
                }
                _ => {}
            }
            Ok(source.clone())
        }
    }
}

fn validate_repository(repository: &str) -> Result<()> {
    let mut parts = repository.split('/');
    let valid_part = |part: &str| {
        !part.is_empty()
            && part != "."
            && part != ".."
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))
    };
    if !parts.next().is_some_and(valid_part)
        || !parts.next().is_some_and(valid_part)
        || parts.next().is_some()
    {
        return Err(ValidationError::new(
            "config task_source repository must be OWNER/REPO",
        ));
    }
    Ok(())
}

fn read_file(path: &Path, label: &str) -> Result<Vec<u8>> {
    fs::read(path).map_err(|error| {
        let kind = if error.kind() == std::io::ErrorKind::NotFound {
            "file is missing"
        } else {
            "cannot be read"
        };
        ValidationError::new(format!("{label} {kind}: {}: {error}", path.display()))
    })
}

fn parse_yaml<T: for<'de> Deserialize<'de>>(bytes: &[u8], label: &str) -> Result<T> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| ValidationError::new(format!("{label} is not UTF-8: {error}")))?;
    yaml_serde::from_str(text)
        .map_err(|error| ValidationError::new(format!("{label} is invalid YAML: {error}")))
}

fn project_file(root: &Path, relative: &str, label: &str) -> Result<PathBuf> {
    if relative.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "{label} must be a non-empty project-relative path"
        )));
    }
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() || escapes_root(relative_path) {
        return Err(ValidationError::new(format!(
            "{label} escapes the project root: {relative}"
        )));
    }
    let candidate = root.join(relative_path);
    if !candidate.is_file() {
        return Err(ValidationError::new(format!(
            "{label} is missing: {}",
            candidate.display()
        )));
    }
    let resolved = candidate
        .canonicalize()
        .map_err(|error| ValidationError::new(format!("{label} cannot be resolved: {error}")))?;
    if !resolved.starts_with(root) {
        return Err(ValidationError::new(format!(
            "{label} resolves outside the project root: {relative}"
        )));
    }
    Ok(resolved)
}

fn escapes_root(path: &Path) -> bool {
    let mut depth = 0usize;
    for component in path.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir if depth > 0 => depth -= 1,
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return true,
        }
    }
    false
}

fn validate_overview(bytes: &[u8]) -> Result<()> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| ValidationError::new(format!("project overview is not UTF-8: {error}")))?;
    let (frontmatter, markdown) = split_frontmatter(text, "project overview")?;
    let metadata: yaml_serde::Value = yaml_serde::from_str(frontmatter).map_err(|error| {
        ValidationError::new(format!(
            "project overview frontmatter is invalid YAML: {error}"
        ))
    })?;
    if !matches!(metadata, yaml_serde::Value::Mapping(_)) {
        return Err(ValidationError::new(
            "project overview frontmatter must contain a YAML mapping",
        ));
    }

    let sections = markdown_sections(markdown)?;
    let body = sections.get("open questions").ok_or_else(|| {
        ValidationError::new("project overview is missing the Open questions section")
    })?;
    let answer = remove_html_comments(body);
    let answer = answer.trim();
    let suffix = answer.get(4..).unwrap_or_default();
    let valid_suffix = suffix
        .chars()
        .next()
        .is_none_or(|character| character == '.' || character.is_whitespace());
    if !answer
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("none"))
        || !valid_suffix
    {
        return Err(ValidationError::new(
            "project overview has unresolved material questions",
        ));
    }
    Ok(())
}

fn split_frontmatter<'a>(text: &'a str, label: &str) -> Result<(&'a str, &'a str)> {
    let mut lines = text.split_inclusive('\n');
    let first = lines.next().unwrap_or_default();
    if first.trim() != "---" {
        return Err(ValidationError::new(format!(
            "{label} is missing YAML frontmatter"
        )));
    }
    let start = first.len();
    let mut offset = start;
    for line in lines {
        if line.trim() == "---" {
            let body_start = offset + line.len();
            return Ok((&text[start..offset], &text[body_start..]));
        }
        offset += line.len();
    }
    Err(ValidationError::new(format!(
        "{label} is missing YAML frontmatter"
    )))
}

fn validate_local_tasks(root: &Path, bytes: &[u8]) -> Result<()> {
    let graph: TaskGraph = parse_yaml(bytes, "task graph")?;
    for (label, value) in [
        ("project", graph.project.as_deref()),
        ("status", graph.status.as_deref()),
        ("source", graph.source.as_deref()),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(ValidationError::new(format!(
                "task graph {label} must not be empty"
            )));
        }
    }
    if graph.tasks.is_empty() {
        return Err(ValidationError::new(
            "task graph must contain at least one task",
        ));
    }

    let mut ids = HashSet::new();
    let mut dependencies = HashMap::new();
    for (index, task) in graph.tasks.iter().enumerate() {
        if task.id.trim().is_empty() {
            return Err(ValidationError::new(format!(
                "task at index {index} has an empty id"
            )));
        }
        if !ids.insert(task.id.clone()) {
            return Err(ValidationError::new(format!(
                "duplicate task id: {}",
                task.id
            )));
        }
        if task.title.trim().is_empty() {
            return Err(ValidationError::new(format!(
                "task {} has an empty title",
                task.id
            )));
        }
        if task.outcome.trim().is_empty() {
            return Err(ValidationError::new(format!(
                "task {} has an empty outcome",
                task.id
            )));
        }
        if task.verify.is_empty() || task.verify.iter().any(|check| check.trim().is_empty()) {
            return Err(ValidationError::new(format!(
                "task {} verification must be a non-empty string list",
                task.id
            )));
        }
        if task.references.is_empty()
            || task
                .references
                .iter()
                .any(|reference| reference.trim().is_empty())
        {
            return Err(ValidationError::new(format!(
                "task {} references must be a non-empty string list",
                task.id
            )));
        }
        for reference in &task.references {
            validate_planning_reference(root, reference, &format!("task {}", task.id))?;
        }
        dependencies.insert(task.id.clone(), task.depends_on.clone());
    }

    for (id, task_dependencies) in &dependencies {
        let unknown: Vec<_> = task_dependencies
            .iter()
            .filter(|dependency| !ids.contains(*dependency))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            return Err(ValidationError::new(format!(
                "task {id} has unknown dependencies: {}",
                unknown.join(", ")
            )));
        }
    }
    if let Some(transition) = &graph.required_planning_transition {
        if transition.after_task.trim().is_empty() || !ids.contains(&transition.after_task) {
            return Err(ValidationError::new(
                "required_planning_transition must name a known task",
            ));
        }
        if transition.reason.trim().is_empty() {
            return Err(ValidationError::new(
                "required_planning_transition reason must be non-empty",
            ));
        }
    }
    validate_dependency_cycles(&ids.into_iter().collect::<Vec<_>>(), &dependencies)
}

fn github_issue_projection(
    root: &Path,
    source: &TaskSource,
    github_api: &GitHubApi<'_>,
) -> Result<GitHubIssueProjection> {
    let TaskSource::GitHubIssues {
        repository,
        root_issue: Some(root_issue),
    } = source
    else {
        return Err(ValidationError::new(
            "task source is not a rooted github_issues source",
        ));
    };

    let root_value = github_get(
        github_api,
        &format!("repos/{repository}/issues/{root_issue}"),
        false,
    )?;
    let root_issue_value = parse_issue(root_value, repository, "root issue")?;
    if root_issue_value.number != *root_issue {
        return Err(ValidationError::new(
            "GitHub root issue does not match config",
        ));
    }
    let mut issues = Vec::new();
    let mut issue_ids = HashSet::new();
    let mut issue_numbers = HashSet::new();
    visit_issue(
        root,
        repository,
        root_issue_value,
        None,
        0,
        true,
        github_api,
        &mut issues,
        &mut issue_ids,
        &mut issue_numbers,
    )?;

    let task_issues = &issues[1..];
    if task_issues.is_empty() {
        return Err(ValidationError::new(
            "GitHub Issue Graph must contain at least one task",
        ));
    }
    let task_members: HashMap<_, _> = task_issues
        .iter()
        .map(|issue| (issue.number, issue.id))
        .collect();
    let task_ids: HashSet<_> = task_members.values().copied().collect();
    let dependencies = github_dependencies(github_api, repository, task_issues, &task_members)?;
    let dependency_map: HashMap<_, Vec<_>> = task_ids
        .iter()
        .map(|id| {
            let blockers = dependencies
                .iter()
                .filter(|edge| edge.blocked_id == *id)
                .map(|edge| edge.blocking_id)
                .collect();
            (*id, blockers)
        })
        .collect();
    validate_dependency_cycles(
        &task_ids.iter().copied().collect::<Vec<_>>(),
        &dependency_map,
    )?;

    Ok(GitHubIssueProjection {
        repository: repository.clone(),
        root_issue: *root_issue,
        issues,
        dependencies,
    })
}

#[allow(clippy::too_many_arguments)]
fn visit_issue(
    root: &Path,
    repository: &str,
    issue: GitHubIssue,
    parent_id: Option<u64>,
    position: usize,
    is_root: bool,
    github_api: &GitHubApi<'_>,
    issues: &mut Vec<ProjectedIssue>,
    issue_ids: &mut HashSet<u64>,
    issue_numbers: &mut HashSet<u64>,
) -> Result<()> {
    if !issue_ids.insert(issue.id) {
        return Err(ValidationError::new(format!(
            "GitHub Issue Graph contains duplicate issue id: {}",
            issue.id
        )));
    }
    if !issue_numbers.insert(issue.number) {
        return Err(ValidationError::new(format!(
            "GitHub Issue Graph contains duplicate issue number: {}",
            issue.number
        )));
    }
    let projected = ProjectedIssue {
        id: issue.id,
        number: issue.number,
        title: issue.title,
        body: issue.body,
        parent_id,
        position,
    };
    if !is_root {
        validate_github_task_body(root, &projected)?;
    }
    let id = projected.id;
    let number = projected.number;
    issues.push(projected);

    let endpoint = format!("repos/{repository}/issues/{number}/sub_issues?per_page=100");
    let children = json_list(
        github_get(github_api, &endpoint, true)?,
        "GitHub sub-issues",
    )?;
    for (child_position, child) in children.into_iter().enumerate() {
        let child = parse_issue(child, repository, "task issue")?;
        visit_issue(
            root,
            repository,
            child,
            Some(id),
            child_position,
            false,
            github_api,
            issues,
            issue_ids,
            issue_numbers,
        )?;
    }
    Ok(())
}

fn github_dependencies(
    github_api: &GitHubApi<'_>,
    repository: &str,
    tasks: &[ProjectedIssue],
    task_members: &HashMap<u64, u64>,
) -> Result<Vec<DependencyEdge>> {
    let mut edges = BTreeMap::new();
    for task in tasks {
        let blocked_by_endpoint = format!(
            "repos/{repository}/issues/{}/dependencies/blocked_by?per_page=100",
            task.number
        );
        let blocking_endpoint = format!(
            "repos/{repository}/issues/{}/dependencies/blocking?per_page=100",
            task.number
        );
        let blocked_by = json_list(
            github_get(github_api, &blocked_by_endpoint, true)?,
            "GitHub blocked_by",
        )?;
        let blocking = json_list(
            github_get(github_api, &blocking_endpoint, true)?,
            "GitHub blocking",
        )?;

        for value in blocked_by {
            let other = parse_issue(value, repository, "dependency endpoint")?;
            validate_dependency_member(&other, task_members)?;
            let edge = DependencyEdge {
                blocking_id: other.id,
                blocked_id: task.id,
            };
            edges.insert((edge.blocking_id, edge.blocked_id), edge);
        }
        for value in blocking {
            let other = parse_issue(value, repository, "dependency endpoint")?;
            validate_dependency_member(&other, task_members)?;
            let edge = DependencyEdge {
                blocking_id: task.id,
                blocked_id: other.id,
            };
            edges.insert((edge.blocking_id, edge.blocked_id), edge);
        }
    }
    Ok(edges.into_values().collect())
}

fn github_get(github_api: &GitHubApi<'_>, endpoint: &str, paginated: bool) -> Result<JsonValue> {
    github_api(endpoint, paginated).map_err(|error| {
        ValidationError::new(format!("GitHub API request failed for {endpoint}: {error}"))
    })
}

pub fn github_api_args(endpoint: &str, paginated: bool) -> Vec<String> {
    let mut arguments = vec![
        "api".to_owned(),
        endpoint.to_owned(),
        "-H".to_owned(),
        "Accept: application/vnd.github+json".to_owned(),
        "-H".to_owned(),
        format!("X-GitHub-Api-Version: {GITHUB_API_VERSION}"),
    ];
    if paginated {
        arguments.extend(["--paginate".to_owned(), "--slurp".to_owned()]);
    }
    arguments
}

pub fn request_github(endpoint: &str, paginated: bool) -> Result<JsonValue> {
    let output = Command::new("gh")
        .args(github_api_args(endpoint, paginated))
        .env("GH_PROMPT_DISABLED", "1")
        .output()
        .map_err(|error| ValidationError::new(format!("GitHub CLI is unavailable: {error}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr
            .lines()
            .next()
            .filter(|line| !line.trim().is_empty())
            .map(str::trim)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("exit {}", output.status));
        return Err(ValidationError::new(detail));
    }
    parse_github_response(&output.stdout, paginated)
}

pub fn parse_github_response(bytes: &[u8], paginated: bool) -> Result<JsonValue> {
    let value: JsonValue = serde_json::from_slice(bytes).map_err(|error| {
        ValidationError::new(format!("GitHub API returned invalid JSON: {error}"))
    })?;
    if !paginated {
        return Ok(value);
    }
    let pages = value
        .as_array()
        .ok_or_else(|| ValidationError::new("GitHub paginated response is incomplete"))?;
    if pages.is_empty() {
        return Err(ValidationError::new(
            "GitHub paginated response is incomplete",
        ));
    }
    let mut items = Vec::new();
    for page in pages {
        let page = page
            .as_array()
            .ok_or_else(|| ValidationError::new("GitHub paginated response is incomplete"))?;
        items.extend(page.iter().cloned());
    }
    Ok(JsonValue::Array(items))
}

fn json_list(value: JsonValue, label: &str) -> Result<Vec<JsonValue>> {
    value
        .as_array()
        .cloned()
        .ok_or_else(|| ValidationError::new(format!("{label} response must be a list")))
}

fn parse_issue(value: JsonValue, repository: &str, label: &str) -> Result<GitHubIssue> {
    let issue: GitHubIssue = serde_json::from_value(value)
        .map_err(|error| ValidationError::new(format!("{label} is invalid: {error}")))?;
    if issue.id == 0 {
        return Err(ValidationError::new(format!("{label} has an invalid id")));
    }
    if issue.number == 0 {
        return Err(ValidationError::new(format!(
            "{label} has an invalid number"
        )));
    }
    if issue.title.trim().is_empty() {
        return Err(ValidationError::new(format!("{label} has an empty title")));
    }
    if issue.pull_request.is_some() {
        return Err(ValidationError::new(format!("{label} is a pull request")));
    }
    let expected = format!("https://api.github.com/repos/{repository}");
    if !issue.repository_url.eq_ignore_ascii_case(&expected) {
        return Err(ValidationError::new(format!(
            "{label} belongs to another repository"
        )));
    }
    Ok(issue)
}

fn validate_dependency_member(issue: &GitHubIssue, task_members: &HashMap<u64, u64>) -> Result<()> {
    if task_members.get(&issue.number) != Some(&issue.id) {
        return Err(ValidationError::new(
            "dependency endpoint is outside the root Issue Graph",
        ));
    }
    Ok(())
}

fn validate_github_task_body(root: &Path, task: &ProjectedIssue) -> Result<()> {
    let body = task
        .body
        .as_deref()
        .filter(|body| !body.trim().is_empty())
        .ok_or_else(|| {
            ValidationError::new(format!("task issue #{} has an empty body", task.number))
        })?;
    let sections = markdown_sections(body)?;
    for heading in ["outcome", "planning references", "verification"] {
        if !sections.contains_key(heading) {
            return Err(ValidationError::new(format!(
                "task issue #{} is missing ## {}",
                task.number,
                title_case(heading)
            )));
        }
    }

    if remove_html_comments(&sections["outcome"]).trim().is_empty() {
        return Err(ValidationError::new(format!(
            "task issue #{} has an empty Outcome",
            task.number
        )));
    }
    let references = markdown_bullets(
        &sections["planning references"],
        &format!("task issue #{} Planning references", task.number),
    )?;
    let checks = markdown_bullets(
        &sections["verification"],
        &format!("task issue #{} Verification", task.number),
    )?;
    if references.is_empty() {
        return Err(ValidationError::new(format!(
            "task issue #{} has no Planning references",
            task.number
        )));
    }
    if checks.is_empty() {
        return Err(ValidationError::new(format!(
            "task issue #{} has no Verification checks",
            task.number
        )));
    }
    for reference in references {
        validate_planning_reference(
            root,
            markdown_link_target(&reference),
            &format!("task issue #{}", task.number),
        )?;
    }
    Ok(())
}

fn markdown_sections(text: &str) -> Result<BTreeMap<String, String>> {
    let mut sections = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.split_inclusive('\n') {
        if let Some(heading) = h2_heading(line) {
            let key = heading.to_ascii_lowercase();
            if sections.insert(key.clone(), String::new()).is_some() {
                return Err(ValidationError::new(format!(
                    "document contains duplicate ## {heading}"
                )));
            }
            current = Some(key);
        } else if let Some(key) = &current {
            sections
                .get_mut(key)
                .expect("section exists")
                .push_str(line);
        }
    }
    Ok(sections)
}

fn h2_heading(line: &str) -> Option<String> {
    let line = line.trim_end();
    let rest = line.strip_prefix("##")?;
    if rest.starts_with('#') || !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    let heading = rest.trim().trim_end_matches('#').trim();
    (!heading.is_empty()).then(|| heading.to_owned())
}

fn markdown_bullets(body: &str, label: &str) -> Result<Vec<String>> {
    let clean = remove_html_comments(body);
    clean
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let item = line
                .strip_prefix("- ")
                .or_else(|| line.strip_prefix("-\t"))
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .ok_or_else(|| {
                    ValidationError::new(format!("{label} must use one plain bullet per item"))
                })?;
            let lower = item.to_ascii_lowercase();
            if lower.starts_with("[ ]") || lower.starts_with("[x]") {
                return Err(ValidationError::new(format!(
                    "{label} must not use mutable checkboxes"
                )));
            }
            Ok(item.to_owned())
        })
        .collect()
}

fn markdown_link_target(value: &str) -> &str {
    if value.starts_with('[') && value.ends_with(')') {
        if let Some(separator) = value.find("](") {
            let target = &value[separator + 2..value.len() - 1];
            return target.split_whitespace().next().unwrap_or(target);
        }
    }
    value
}

fn remove_html_comments(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(start) = remaining.find("<!--") {
        result.push_str(&remaining[..start]);
        let after_start = &remaining[start + 4..];
        let Some(end) = after_start.find("-->") else {
            return result;
        };
        remaining = &after_start[end + 3..];
    }
    result.push_str(remaining);
    result
}

fn validate_planning_reference(root: &Path, reference: &str, label: &str) -> Result<()> {
    let path = reference.split('#').next().unwrap_or_default();
    project_file(root, path, &format!("{label} reference"))?;
    Ok(())
}

fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

fn validate_dependency_cycles<T>(nodes: &[T], dependencies: &HashMap<T, Vec<T>>) -> Result<()>
where
    T: Clone + Eq + Hash,
{
    let mut remaining = HashMap::new();
    let mut dependents: HashMap<T, Vec<T>> = HashMap::new();
    let mut ready = VecDeque::new();

    for node in nodes {
        let task_dependencies = dependencies.get(node).map(Vec::as_slice).unwrap_or(&[]);
        remaining.insert(node.clone(), task_dependencies.len());
        if task_dependencies.is_empty() {
            ready.push_back(node.clone());
        }
        for dependency in task_dependencies {
            dependents
                .entry(dependency.clone())
                .or_default()
                .push(node.clone());
        }
    }

    let mut visited = 0usize;
    while let Some(node) = ready.pop_front() {
        visited += 1;
        for dependent in dependents.get(&node).map(Vec::as_slice).unwrap_or(&[]) {
            let count = remaining
                .get_mut(dependent)
                .expect("every dependent is a task");
            *count -= 1;
            if *count == 0 {
                ready.push_back(dependent.clone());
            }
        }
    }

    if visited != nodes.len() {
        return Err(ValidationError::new("task dependency cycle"));
    }
    Ok(())
}

fn validate_local_approval(
    approval: &Approval,
    planned_files: &BTreeMap<&str, &[u8]>,
) -> Result<()> {
    if approval.planning_revision.is_some() {
        return Err(ValidationError::new(
            "local approval must not contain a GitHub planning_revision",
        ));
    }
    let files = approval.files.as_ref().ok_or_else(|| {
        ValidationError::new("approval record must cover exactly the configured planning files")
    })?;
    let expected_keys: Vec<_> = planned_files.keys().copied().collect();
    let actual_keys: Vec<_> = files.keys().map(String::as_str).collect();
    if actual_keys != expected_keys {
        return Err(ValidationError::new(
            "approval record must cover exactly the configured planning files",
        ));
    }
    for (relative, bytes) in planned_files {
        validate_file_digest(files.get(*relative), bytes, relative)?;
    }
    Ok(())
}

fn validate_github_approval_metadata(
    approval: &Approval,
    overview_relative: &str,
    overview_bytes: &[u8],
    source: &TaskSource,
) -> Result<String> {
    if approval.files.is_some() {
        return Err(ValidationError::new(
            "GitHub approval must not contain a local files mapping",
        ));
    }
    let revision = approval.planning_revision.as_ref().ok_or_else(|| {
        ValidationError::new(
            "approval record must contain project_overview and task_source planning revisions",
        )
    })?;
    if revision.project.is_some() {
        return Err(ValidationError::new(
            "rooted GitHub approval must not contain a project revision",
        ));
    }
    if revision.project_overview.path != overview_relative {
        return Err(ValidationError::new(
            "approval project_overview does not match config",
        ));
    }
    validate_file_digest(
        Some(&revision.project_overview.sha256),
        overview_bytes,
        overview_relative,
    )?;

    let TaskSource::GitHubIssues {
        repository,
        root_issue: Some(root_issue),
    } = source
    else {
        unreachable!("rooted GitHub approval requires a rooted GitHub source");
    };
    let recorded = revision
        .task_source
        .as_ref()
        .ok_or_else(|| ValidationError::new("rooted GitHub approval is missing task_source"))?;
    if recorded.kind != "github_issues"
        || recorded.repository != *repository
        || recorded.root_issue != *root_issue
    {
        return Err(ValidationError::new(
            "approval task_source does not match config",
        ));
    }
    if !is_sha256(&recorded.sha256) {
        return Err(ValidationError::new(
            "approval task_source has an invalid SHA-256 digest",
        ));
    }
    Ok(recorded.sha256.clone())
}

fn validate_project_approval(
    approval: &Approval,
    project_revision: &ProjectRevisionOutput,
) -> Result<()> {
    if approval.files.is_some() {
        return Err(ValidationError::new(
            "rootless GitHub approval must not contain a local files mapping",
        ));
    }
    let revision = approval.planning_revision.as_ref().ok_or_else(|| {
        ValidationError::new("approval record must contain project_overview and project revisions")
    })?;
    if revision.task_source.is_some() {
        return Err(ValidationError::new(
            "rootless GitHub approval must not contain a rooted task_source revision",
        ));
    }
    if revision.project_overview != project_revision.projection.project_overview {
        return Err(ValidationError::new(
            "approval project_overview does not match the current project revision",
        ));
    }
    let recorded = revision.project.as_ref().ok_or_else(|| {
        ValidationError::new("rootless GitHub approval is missing project revision")
    })?;
    if !is_sha256(&recorded.sha256) || recorded.sha256 != project_revision.sha256 {
        return Err(ValidationError::new("approved project revision changed"));
    }
    Ok(())
}

fn validate_approval_identity(approval: &Approval) -> Result<()> {
    if approval.status.as_deref() != Some("approved") {
        return Err(ValidationError::new(
            "approval record status must be approved",
        ));
    }
    if approval
        .approved_by
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        return Err(ValidationError::new(
            "approval record is missing approved_by",
        ));
    }
    if approval
        .approved_at
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        return Err(ValidationError::new(
            "approval record is missing approved_at",
        ));
    }
    Ok(())
}

fn validate_file_digest(expected: Option<&String>, bytes: &[u8], relative: &str) -> Result<()> {
    let expected = expected.filter(|value| is_sha256(value)).ok_or_else(|| {
        ValidationError::new(format!(
            "approval record has an invalid SHA-256 digest for {relative}"
        ))
    })?;
    let actual = sha256_hex(bytes);
    if actual != *expected {
        return Err(ValidationError::new(format!(
            "approved planning file changed: {relative}"
        )));
    }
    Ok(())
}

fn projection_digest(projection: &GitHubIssueProjection) -> Result<String> {
    json_digest(projection, "GitHub projection")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
