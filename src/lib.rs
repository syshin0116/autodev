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
struct Config {
    project_overview: String,
    #[serde(default)]
    task_graph: Option<String>,
    #[serde(default)]
    task_source: Option<TaskSource>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
enum TaskSource {
    #[serde(rename = "local_file")]
    LocalFile { path: String },
    #[serde(rename = "github_issues")]
    GitHubIssues { repository: String, root_issue: u64 },
}

#[derive(Debug, Deserialize)]
struct TaskGraph {
    tasks: Vec<LocalTask>,
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
    task_source: GitHubTaskSourceRevision,
}

#[derive(Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
struct GitHubIssue {
    id: u64,
    number: u64,
    title: String,
    body: Option<String>,
    repository_url: String,
    #[serde(default)]
    pull_request: Option<JsonValue>,
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
        source @ TaskSource::GitHubIssues { .. } => {
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
    if !matches!(source, TaskSource::GitHubIssues { .. }) {
        return Err(ValidationError::new(
            "configured task source is not github_issues",
        ));
    }
    projection_output(github_issue_projection(&root, &source, github_api)?)
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
                    if *root_issue == 0 {
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
    validate_dependency_cycles(&ids.into_iter().collect::<Vec<_>>(), &dependencies)
}

fn github_issue_projection(
    root: &Path,
    source: &TaskSource,
    github_api: &GitHubApi<'_>,
) -> Result<GitHubIssueProjection> {
    let TaskSource::GitHubIssues {
        repository,
        root_issue,
    } = source
    else {
        return Err(ValidationError::new("task source is not github_issues"));
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

fn request_github(endpoint: &str, paginated: bool) -> Result<JsonValue> {
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
        root_issue,
    } = source
    else {
        unreachable!("GitHub approval requires a GitHub source");
    };
    let recorded = &revision.task_source;
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
    let bytes = serde_json::to_vec(projection).map_err(|error| {
        ValidationError::new(format!("GitHub projection cannot be serialized: {error}"))
    })?;
    Ok(sha256_hex(&bytes))
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
