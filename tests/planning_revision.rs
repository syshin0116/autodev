use autodev_planning_revision::{
    AuthorizationAction, AuthorizationDecision, DependencyStatus, EpisodeEvent, EpisodeEventKind,
    EpisodeStatus, GITHUB_API_VERSION, ProjectRevisionOutput, ReadyEvent, RepositoryActor,
    TaskAuthorization, TaskSnapshotOutput, TransitionAction, ValidatedTaskSnapshot,
    ValidationError, VerifiedEvidence, authorize_task_with_api, complete_episode_merge,
    dependencies_ready, github_api_args, parse_github_response, project_revision_with_api,
    task_snapshot_with_api, task_source_projection_with_api, transition_episode_with_api,
    validate_with_api,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

type ApiResponses = BTreeMap<(String, bool), Value>;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn cli_accepts_an_exactly_approved_local_project_without_writes() {
    let project = TempProject::from_template();
    let before = snapshot_tree(project.root());

    let output = Command::new(env!("CARGO_BIN_EXE_autodev-planning-revision"))
        .arg(project.root())
        .output()
        .expect("run planning revision validator");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 stdout"),
        "Planning revision valid.\n"
    );
    assert_eq!(before, snapshot_tree(project.root()));
}

#[test]
fn local_source_rejects_invalid_task_graphs() {
    assert_invalid_local_task("unresolved material questions", |root| {
        replace_in(
            &root.join("docs/project-overview.md"),
            "None.",
            "- Choose a delivery date.",
        );
    });
    assert_invalid_local_task("duplicate task id", |root| {
        let path = root.join("tasks.yaml");
        let text = read(&path);
        let task_start = text.find("  - id:").expect("template task");
        fs::write(path, format!("{text}{}", &text[task_start..])).expect("duplicate task");
    });
    assert_invalid_local_task("unknown dependencies", |root| {
        replace_in(
            &root.join("tasks.yaml"),
            "depends_on: []",
            "depends_on:\n      - missing-task",
        );
    });
    assert_invalid_local_task("task dependency cycle", |root| {
        replace_in(
            &root.join("tasks.yaml"),
            "depends_on: []",
            "depends_on:\n      - first-verifiable-outcome",
        );
    });
    assert_invalid_local_task("verification must be a non-empty string list", |root| {
        replace_in(
            &root.join("tasks.yaml"),
            "verify:\n      - Replace with a runnable or human-verifiable check.",
            "verify: []",
        );
    });
    assert_invalid_local_task("reference is missing", |root| {
        replace_in(
            &root.join("tasks.yaml"),
            "docs/project-overview.md#success-criteria",
            "docs/missing.md#success-criteria",
        );
    });
    assert_invalid_local_task("empty outcome", |root| {
        replace_in(
            &root.join("tasks.yaml"),
            "outcome: Replace with the state this task must create.",
            "outcome: \"\"",
        );
    });
}

#[test]
fn local_source_rejects_missing_pending_or_changed_approval() {
    let missing = TempProject::from_template();
    fs::remove_file(missing.root().join(".autodev/approval.yaml")).expect("remove approval");
    assert_local_error(&missing, "approval record is missing");

    let pending = TempProject::from_template();
    replace_in(
        &pending.root().join(".autodev/approval.yaml"),
        "status: approved",
        "status: pending",
    );
    assert_local_error(&pending, "status must be approved");

    let changed_overview = TempProject::from_template();
    fs::write(
        changed_overview.root().join("docs/project-overview.md"),
        format!(
            "{}\nChanged after approval.\n",
            read(&changed_overview.root().join("docs/project-overview.md"))
        ),
    )
    .expect("change overview");
    assert_local_error(
        &changed_overview,
        "approved planning file changed: docs/project-overview.md",
    );

    let changed_tasks = TempProject::from_template();
    fs::write(
        changed_tasks.root().join("tasks.yaml"),
        format!(
            "{}# changed after approval\n",
            read(&changed_tasks.root().join("tasks.yaml"))
        ),
    )
    .expect("change tasks");
    assert_local_error(&changed_tasks, "approved planning file changed: tasks.yaml");
}

#[test]
fn local_source_rejects_paths_outside_the_project() {
    let project = TempProject::from_template();
    fs::write(
        project.root().join(".autodev/config.yaml"),
        "project_overview: ../project-overview.md\ntask_graph: tasks.yaml\n",
    )
    .expect("write escaping config");
    assert_local_error(&project, "escapes the project root");
}

#[test]
fn config_requires_exactly_one_task_source() {
    let both = TempProject::from_template();
    fs::write(
        both.root().join(".autodev/config.yaml"),
        concat!(
            "project_overview: docs/project-overview.md\n",
            "task_graph: tasks.yaml\n",
            "task_source:\n",
            "  type: github_issues\n",
            "  repository: example/autodev\n",
            "  root_issue: 10\n",
        ),
    )
    .expect("write dual-source config");
    assert_local_error(&both, "exactly one task source");

    let neither = TempProject::from_template();
    fs::write(
        neither.root().join(".autodev/config.yaml"),
        "project_overview: docs/project-overview.md\n",
    )
    .expect("write source-free config");
    assert_local_error(&neither, "exactly one task source");

    let typo = rootless_project();
    replace_in(
        &typo.root().join(".autodev/config.yaml"),
        "  repository: example/autodev\n",
        "  repository: example/autodev\n  root_issues: 10\n",
    );
    let error = project_revision_with_api(typo.root(), &rootless_project_api)
        .expect_err("unknown task-source fields must fail closed");
    assert!(error.to_string().contains("unknown field"), "{error}");
}

#[test]
fn rootless_project_revision_binds_policy_but_not_operational_engine_selection() {
    let project = rootless_project();
    let baseline =
        project_revision_with_api(project.root(), &rootless_project_api).expect("project revision");
    let projection =
        serde_json::to_value(&baseline.projection).expect("serialize project revision");
    assert_eq!(
        projection,
        json!({
            "schema_version": 1,
            "project_overview": {
                "path": "docs/project-overview.md",
                "sha256": sha256_file(&project.root().join("docs/project-overview.md")),
            },
            "task_source": {
                "type": "github_issues",
                "repository_id": 1000,
                "repository_node_id": "R_example_autodev",
                "repository": "example/autodev",
            },
            "authorization": {
                "ready_label": "autodev:ready",
                "refusal_labels": ["autodev:human-needed", "autodev:needs-input"],
                "authorizer_roles": ["admin", "write"],
                "cancel_roles": ["admin", "write"],
            },
            "delivery": {
                "base_branch": "main",
                "protected_paths": [".autodev/**", ".github/workflows/**"],
                "required_checks": ["autodev/gate", "project-ci"],
                "review": {
                    "required_approvals": 1,
                    "require_code_owner_review": false,
                    "require_thread_resolution": true,
                    "require_up_to_date_branch": true,
                },
                "merge_mode": "auto_after_gates",
                "correction_budget": 2,
            },
            "engine_policy": {
                "allowed_providers": ["anthropic", "openai"],
                "data_use_boundary": "no_training",
                "cost_class": "standard",
            },
            "tools": {
                "coding-agent": {
                    "purpose": "Implement approved repository tasks.",
                    "interface": "cli",
                    "permissions": ["patch_output", "repository_read"],
                },
            },
            "knowledge": {
                "sources": [{"alias": "personal", "visibility": "private"}],
                "candidate_carrier": {
                    "identity": "github:example/private-candidates",
                    "visibility": "private",
                },
                "target": {
                    "identity": "github:example/dev-knowledge",
                    "visibility": "public",
                },
            },
        })
    );

    let equivalent = rootless_project();
    replace_in(
        &equivalent.root().join(".autodev/config.yaml"),
        concat!(
            "    - alias: personal\n",
            "      path: /machine-one/knowledge\n",
        ),
        concat!(
            "    - alias: personal\n",
            "      visibility: private\n",
            "      path: /machine-two/knowledge\n",
        ),
    );
    replace_in(
        &equivalent.root().join(".autodev/config.yaml"),
        concat!(
            "selected_engine:\n",
            "  provider: openai\n",
            "  name: codex\n",
            "  version: gpt-5.6\n",
        ),
        concat!(
            "selected_engine:\n",
            "  provider: anthropic\n",
            "  name: claude-code\n",
            "  version: opus-4.1\n",
        ),
    );
    let equivalent = project_revision_with_api(equivalent.root(), &rootless_project_api)
        .expect("equivalent revision");
    assert_eq!(baseline.sha256, equivalent.sha256);

    let changed = rootless_project();
    replace_in(
        &changed.root().join(".autodev/config.yaml"),
        "purpose: Implement approved repository tasks.",
        "purpose: Publish repository releases.",
    );
    let changed =
        project_revision_with_api(changed.root(), &rootless_project_api).expect("changed revision");
    assert_ne!(baseline.sha256, changed.sha256);

    approve_rootless_project(project.root(), &baseline.sha256);
    assert_eq!(
        validate_with_api(project.root(), &rootless_project_api)
            .expect("approved project revision"),
        None
    );
    replace_in(
        &project.root().join(".autodev/config.yaml"),
        "merge_mode: auto_after_gates",
        "merge_mode: human",
    );
    let error = validate_with_api(project.root(), &rootless_project_api)
        .expect_err("changed project policy");
    assert!(
        error
            .to_string()
            .contains("approved project revision changed"),
        "{error}"
    );
}

#[test]
fn task_snapshot_is_scoped_to_one_raw_issue_and_its_dependencies() {
    use std::cell::RefCell;

    let project = approved_rootless_project();
    let responses = rootless_task_fixture();
    let calls = RefCell::new(Vec::new());
    let api = |endpoint: &str, paginated: bool| {
        calls.borrow_mut().push((endpoint.to_owned(), paginated));
        fake_github_api(&responses)(endpoint, paginated)
    };
    let validated = task_snapshot_with_api(project.root(), 21, &api).expect("task-scoped snapshot");
    let snapshot =
        serde_json::to_value(&validated.task_snapshot.projection).expect("serialize task snapshot");

    assert_eq!(snapshot["repository_node_id"], "R_example_autodev");
    assert_eq!(snapshot["issue"]["issue_id"], 201);
    assert_eq!(snapshot["issue"]["issue_node_id"], "I_201");
    assert_eq!(snapshot["issue"]["issue_number"], 21);
    assert_eq!(snapshot["title"], "Implement scoped authorization");
    assert_eq!(
        snapshot["blocked_by"],
        json!([{
            "issue_id": 200,
            "issue_node_id": "I_200",
            "issue_number": 20,
        }])
    );
    assert_eq!(
        snapshot["project_revision_sha256"],
        validated.project_revision.sha256
    );
    assert_eq!(
        calls.into_inner(),
        vec![
            ("repos/example/autodev".into(), false),
            ("repos/example/autodev/issues/21".into(), false),
            (
                "repos/example/autodev/issues/21/dependencies/blocked_by?per_page=100".into(),
                true,
            ),
        ]
    );

    let mut unrelated_changed = responses.clone();
    unrelated_changed.insert(
        ("repos/example/autodev/issues/99".into(), false),
        github_issue(999, 99, "Unrelated change", &task_body("Unrelated.")),
    );
    let unrelated_api = fake_github_api(&unrelated_changed);
    assert_eq!(
        validated,
        task_snapshot_with_api(project.root(), 21, &unrelated_api)
            .expect("unrelated issue is outside the snapshot")
    );

    let mut task_changed = responses.clone();
    let task = task_changed
        .get_mut(&("repos/example/autodev/issues/21".into(), false))
        .and_then(Value::as_object_mut)
        .expect("task issue");
    replace_issue_body(
        task,
        "Create the authorization record.",
        "Create a revised record.",
    );
    let changed_api = fake_github_api(&task_changed);
    assert_ne!(
        validated.task_snapshot.sha256,
        task_snapshot_with_api(project.root(), 21, &changed_api)
            .expect("changed task snapshot")
            .task_snapshot
            .sha256
    );

    let mut cross_repository = responses.clone();
    response_array_mut(
        &mut cross_repository,
        "repos/example/autodev/issues/21/dependencies/blocked_by?per_page=100",
    )[0]["repository_url"] = json!("https://api.github.com/repos/example/other");
    let error = task_snapshot_with_api(project.root(), 21, &fake_github_api(&cross_repository))
        .expect_err("cross-repository dependency");
    assert!(error.to_string().contains("another repository"), "{error}");

    let mut refused = responses;
    refused
        .get_mut(&("repos/example/autodev/issues/21".into(), false))
        .and_then(Value::as_object_mut)
        .expect("task issue")
        .insert(
            "labels".into(),
            json!([
                {"name": "autodev:ready"},
                {"name": "autodev:human-needed"},
            ]),
        );
    let error = task_snapshot_with_api(project.root(), 21, &fake_github_api(&refused))
        .expect_err("refusal label blocks the task");
    assert!(error.to_string().contains("refusal label"), "{error}");
}

#[test]
fn authorization_keeps_raw_and_agent_digests_and_serializes_generations() {
    let project = approved_rootless_project();
    let responses = rootless_task_fixture();
    let snapshot = task_snapshot_with_api(project.root(), 21, &fake_github_api(&responses))
        .expect("task snapshot");
    let event = ready_event(&snapshot, 1, "write");
    let first = authorize_rootless_task(
        &snapshot.project_revision,
        &snapshot.task_snapshot,
        b"integrity-filtered agent input",
        &event,
        &[],
    )
    .expect("authorized task");

    assert_eq!(first.action, AuthorizationAction::Start);
    assert_eq!(first.authorization.episode.authorization_generation, 1);
    assert_eq!(first.authorization.status, EpisodeStatus::Active);
    assert_eq!(
        first.authorization.episode.task_sha256,
        snapshot.task_snapshot.sha256
    );
    assert_eq!(
        first.authorization.agent_input_sha256,
        sha256_bytes(b"integrity-filtered agent input")
    );
    assert_ne!(
        first.authorization.episode.task_sha256,
        first.authorization.agent_input_sha256
    );
    let mut forged = snapshot.task_snapshot.clone();
    forged.sha256 = "a".repeat(64);
    let error = authorize_rootless_task(
        &snapshot.project_revision,
        &forged,
        b"integrity-filtered agent input",
        &event,
        &[],
    )
    .expect_err("forged raw digest");
    assert!(error.to_string().contains("invalid digest"), "{error}");
    let permission_api = |endpoint: &str, paginated: bool| {
        assert_eq!(
            endpoint,
            "repos/example/autodev/collaborators/dante/permission"
        );
        assert!(!paginated);
        Ok(json!({
            "permission": "write",
            "role_name": "write",
            "user": {"id": 7, "login": "dante"},
        }))
    };
    assert_eq!(
        authorize_task_with_api(
            &snapshot.project_revision,
            &snapshot.task_snapshot,
            b"integrity-filtered agent input",
            &event,
            &[],
            &permission_api,
        )
        .expect("current repository permission"),
        first
    );
    let no_permission_lookup = |_endpoint: &str, _paginated: bool| {
        panic!("an exact persisted replay must not recheck current permission")
    };
    let replay = authorize_task_with_api(
        &snapshot.project_revision,
        &snapshot.task_snapshot,
        b"integrity-filtered agent input",
        &event,
        std::slice::from_ref(&first.authorization),
        &no_permission_lookup,
    )
    .expect("ready event replay");
    assert_eq!(replay.action, AuthorizationAction::Replay);
    assert_eq!(replay.authorization, first.authorization);

    let wrong_role = ready_event(&snapshot, 2, "read");
    let mut wrong_label = ready_event(&snapshot, 3, "write");
    wrong_label.label = "autodev:other".into();
    let mut wrong_issue = ready_event(&snapshot, 4, "write");
    wrong_issue.issue_node_id = "I_other".into();
    let mut wrong_digest = ready_event(&snapshot, 8, "write");
    wrong_digest.task_sha256 = "b".repeat(64);
    for refused in [wrong_role, wrong_label, wrong_issue, wrong_digest] {
        assert!(
            authorize_rootless_task(
                &snapshot.project_revision,
                &snapshot.task_snapshot,
                b"filtered",
                &refused,
                &[],
            )
            .is_err(),
            "event {} must not authorize",
            refused.run_id
        );
    }

    let abandoned = transition_rootless_episode(
        &snapshot.project_revision,
        &first.authorization,
        &episode_event(&first.authorization, EpisodeEventKind::IssueClosed, 100),
        Some("write"),
    )
    .expect("authorized abandonment");
    assert_eq!(abandoned.authorization.status, EpisodeStatus::Abandoned);
    assert!(abandoned.cleanup_required);
    let abandoned = transition_rootless_episode(
        &snapshot.project_revision,
        &abandoned.authorization,
        &episode_event(
            &abandoned.authorization,
            EpisodeEventKind::CleanupCompleted,
            101,
        ),
        None,
    )
    .expect("abandonment cleanup");
    let second = authorize_rootless_task(
        &snapshot.project_revision,
        &snapshot.task_snapshot,
        b"integrity-filtered agent input",
        &ready_event(&snapshot, 5, "write"),
        std::slice::from_ref(&abandoned.authorization),
    )
    .expect("reauthorize abandoned task");
    assert_eq!(second.action, AuthorizationAction::Start);
    assert_eq!(second.authorization.episode.authorization_generation, 2);

    let mut exhausted = abandoned.authorization.clone();
    exhausted.episode.authorization_generation = u64::MAX;
    let error = authorize_rootless_task(
        &snapshot.project_revision,
        &snapshot.task_snapshot,
        b"integrity-filtered agent input",
        &ready_event(&snapshot, 7, "write"),
        &[exhausted],
    )
    .expect_err("generation overflow");
    assert!(error.to_string().contains("overflow"), "{error}");

    let changed_project = rootless_project();
    replace_in(
        &changed_project.root().join(".autodev/config.yaml"),
        "merge_mode: auto_after_gates",
        "merge_mode: human",
    );
    let changed_revision = project_revision_with_api(changed_project.root(), &rootless_project_api)
        .expect("changed project revision");
    let error = authorize_rootless_task(
        &changed_revision,
        &snapshot.task_snapshot,
        b"integrity-filtered agent input",
        &ready_event(&snapshot, 6, "write"),
        &[],
    )
    .expect_err("old task snapshot must not cross project revisions");
    assert!(
        error.to_string().contains("another project revision"),
        "{error}"
    );
}

#[test]
fn episode_transitions_require_the_bound_roles_and_keep_merge_terminal() {
    let (project, authorization) = authorized_rootless_task();

    let mut misrouted = episode_event(&authorization, EpisodeEventKind::IssueClosed, 199);
    misrouted.episode.issue_number += 1;
    let error = transition_rootless_episode(&project, &authorization, &misrouted, Some("write"))
        .expect_err("misrouted episode event");
    assert!(error.to_string().contains("does not match"), "{error}");

    let untrusted = transition_rootless_episode(
        &project,
        &authorization,
        &episode_event(&authorization, EpisodeEventKind::IssueClosed, 200),
        Some("read"),
    )
    .expect("untrusted close only fails the gate");
    assert_eq!(untrusted.action, TransitionAction::GateFailed);
    assert_eq!(untrusted.authorization.status, EpisodeStatus::Active);
    assert!(!untrusted.cleanup_required);
    let error = transition_rootless_episode(
        &project,
        &untrusted.authorization,
        &episode_event(
            &untrusted.authorization,
            EpisodeEventKind::NeedsInputRecorded,
            200,
        ),
        None,
    )
    .expect_err("failed gate event ID cannot be reused");
    assert!(error.to_string().contains("already used"), "{error}");

    let suspended = transition_rootless_episode(
        &project,
        &authorization,
        &episode_event(&authorization, EpisodeEventKind::NeedsInputRecorded, 201),
        None,
    )
    .expect("workflow suspension");
    assert_eq!(suspended.authorization.status, EpisodeStatus::Suspended);
    let replay = transition_rootless_episode(
        &project,
        &suspended.authorization,
        &episode_event(&authorization, EpisodeEventKind::NeedsInputRecorded, 201),
        None,
    )
    .expect("transition replay");
    assert_eq!(replay.action, TransitionAction::NoOp);
    let revoked = transition_rootless_episode(
        &project,
        &suspended.authorization,
        &episode_event(
            &suspended.authorization,
            EpisodeEventKind::ReadyRemoved,
            202,
        ),
        Some("write"),
    )
    .expect("human ready removal");
    assert_eq!(revoked.authorization.status, EpisodeStatus::Abandoned);
    let old_replay = transition_rootless_episode(
        &project,
        &revoked.authorization,
        &episode_event(&authorization, EpisodeEventKind::NeedsInputRecorded, 201),
        None,
    )
    .expect("older transition replay");
    assert_eq!(old_replay.action, TransitionAction::NoOp);
    let error = transition_rootless_episode(
        &project,
        &suspended.authorization,
        &episode_event(&authorization, EpisodeEventKind::IssueClosed, 201),
        Some("write"),
    )
    .expect_err("conflicting transition event identity");
    assert!(error.to_string().contains("already used"), "{error}");
    let resumed = transition_rootless_episode(
        &project,
        &suspended.authorization,
        &episode_event(
            &suspended.authorization,
            EpisodeEventKind::OperationalResume,
            203,
        ),
        Some("write"),
    )
    .expect("authorized operational resume");
    assert_eq!(resumed.authorization.status, EpisodeStatus::Active);

    let superseded = transition_rootless_episode(
        &project,
        &authorization,
        &episode_event(&authorization, EpisodeEventKind::IssueEdited, 204),
        Some("write"),
    )
    .expect("authorized task supersession");
    assert_eq!(
        superseded.authorization.status,
        EpisodeStatus::SupersessionPending
    );
    assert!(superseded.cleanup_required);
    let replacement_project = approved_rootless_project();
    let mut replacement_responses = rootless_task_fixture();
    replacement_responses
        .get_mut(&("repos/example/autodev/issues/21".into(), false))
        .and_then(Value::as_object_mut)
        .expect("task issue")
        .insert("title".into(), json!("Changed task"));
    let replacement = task_snapshot_with_api(
        replacement_project.root(),
        21,
        &fake_github_api(&replacement_responses),
    )
    .expect("replacement snapshot");
    let replacement_decision = authorize_rootless_task(
        &project,
        &replacement.task_snapshot,
        b"filtered replacement",
        &ready_event(&replacement, 205, "write"),
        std::slice::from_ref(&superseded.authorization),
    )
    .expect("record replacement digests");
    assert_eq!(
        replacement_decision.action,
        AuthorizationAction::SupersessionRequired
    );
    assert!(replacement_decision.authorization.replacement.is_some());

    let edited_again = transition_rootless_episode(
        &project,
        &replacement_decision.authorization,
        &episode_event(
            &replacement_decision.authorization,
            EpisodeEventKind::IssueEdited,
            206,
        ),
        Some("write"),
    )
    .expect("replacement changed before cleanup");
    assert!(edited_again.authorization.replacement.is_none());
    let cleaned_without_replacement = transition_rootless_episode(
        &project,
        &edited_again.authorization,
        &episode_event(
            &edited_again.authorization,
            EpisodeEventKind::CleanupCompleted,
            207,
        ),
        None,
    )
    .expect("cleanup before a valid replacement");
    assert_eq!(
        cleaned_without_replacement.authorization.status,
        EpisodeStatus::SupersessionPending
    );
    assert!(cleaned_without_replacement.authorization.cleanup_complete);

    let superseded = transition_rootless_episode(
        &project,
        &replacement_decision.authorization,
        &episode_event(
            &replacement_decision.authorization,
            EpisodeEventKind::CleanupCompleted,
            208,
        ),
        None,
    )
    .expect("supersession cleanup");
    assert_eq!(superseded.authorization.status, EpisodeStatus::Superseded);
    let still_superseded = transition_rootless_episode(
        &project,
        &superseded.authorization,
        &episode_event(
            &superseded.authorization,
            EpisodeEventKind::NeedsInputRecorded,
            209,
        ),
        None,
    )
    .expect("superseded state is terminal");
    assert_eq!(still_superseded.action, TransitionAction::NoOp);
    assert_eq!(
        still_superseded.authorization.status,
        EpisodeStatus::Superseded
    );

    let evidence = verified_evidence(&authorization);
    let mut unverified = evidence.clone();
    unverified.verified = false;
    assert!(complete_episode_merge(&project, &authorization, &unverified, "main").is_err());
    assert!(complete_episode_merge(&project, &authorization, &evidence, "release").is_err());
    let mut stale_evidence = evidence.clone();
    stale_evidence.authorization_generation += 1;
    assert!(complete_episode_merge(&project, &authorization, &stale_evidence, "main").is_err());
    let mut forged_project = project.clone();
    forged_project.sha256 = "a".repeat(64);
    assert!(complete_episode_merge(&forged_project, &authorization, &evidence, "main").is_err());
    let merged = complete_episode_merge(&project, &authorization, &evidence, "main")
        .expect("merge transition");
    assert_eq!(merged.authorization.status, EpisodeStatus::Merged);
    let after_close = transition_rootless_episode(
        &project,
        &merged.authorization,
        &episode_event(&merged.authorization, EpisodeEventKind::IssueClosed, 210),
        Some("write"),
    )
    .expect("merged state is absorbing");
    assert_eq!(after_close.action, TransitionAction::NoOp);
    assert_eq!(after_close.authorization.status, EpisodeStatus::Merged);
    assert_eq!(
        after_close.authorization.processed_transition_events.len(),
        1
    );
}

#[test]
fn approved_project_change_requires_cleanup_before_the_next_generation() {
    let (old_project, authorization) = authorized_rootless_task();
    let changed = rootless_project();
    replace_in(
        &changed.root().join(".autodev/config.yaml"),
        "merge_mode: auto_after_gates",
        "merge_mode: human",
    );
    let new_project = project_revision_with_api(changed.root(), &rootless_project_api)
        .expect("changed project revision");
    approve_rootless_project(changed.root(), &new_project.sha256);
    let replacement = task_snapshot_with_api(
        changed.root(),
        21,
        &fake_github_api(&rootless_task_fixture()),
    )
    .expect("task on changed project revision");

    let ready = ready_event(&replacement, 300, "write");
    let pending = authorize_rootless_task(
        &new_project,
        &replacement.task_snapshot,
        b"filtered",
        &ready,
        std::slice::from_ref(&authorization),
    )
    .expect("project supersession");
    assert_eq!(pending.action, AuthorizationAction::SupersessionRequired);
    assert_eq!(
        pending.authorization.status,
        EpisodeStatus::SupersessionPending
    );

    let cleaned = transition_rootless_episode(
        &new_project,
        &pending.authorization,
        &episode_event(
            &pending.authorization,
            EpisodeEventKind::CleanupCompleted,
            301,
        ),
        None,
    )
    .expect("project supersession cleanup");
    assert_eq!(cleaned.authorization.status, EpisodeStatus::Superseded);

    let next = authorize_rootless_task(
        &new_project,
        &replacement.task_snapshot,
        b"filtered",
        &ready,
        std::slice::from_ref(&cleaned.authorization),
    )
    .expect("authorization after project supersession");
    assert_eq!(next.action, AuthorizationAction::Start);
    assert_eq!(next.authorization.episode.authorization_generation, 2);
    assert_ne!(old_project.sha256, new_project.sha256);
}

#[test]
fn dependencies_require_verified_evidence_merged_into_the_task_base() {
    let project = approved_rootless_project();
    let snapshot = task_snapshot_with_api(
        project.root(),
        21,
        &fake_github_api(&rootless_task_fixture()),
    )
    .expect("task snapshot");
    let status = DependencyStatus {
        issue_node_id: "I_200".into(),
        approved_task_sha256: Some("a".repeat(64)),
        project_revision_sha256: snapshot.project_revision.sha256.clone(),
        authorization_generation: 1,
        evidence_verified: true,
        evidence_task_sha256: "a".repeat(64),
        evidence_project_revision_sha256: snapshot.project_revision.sha256.clone(),
        evidence_authorization_generation: 1,
        merged_into: Some("main".into()),
    };

    assert!(
        dependencies_ready(
            &snapshot.project_revision,
            &snapshot.task_snapshot,
            std::slice::from_ref(&status)
        )
        .expect("complete dependency")
    );
    assert!(
        !dependencies_ready(&snapshot.project_revision, &snapshot.task_snapshot, &[])
            .expect("missing dependency status")
    );
    assert!(
        !dependencies_ready(
            &snapshot.project_revision,
            &snapshot.task_snapshot,
            &[DependencyStatus {
                evidence_verified: false,
                ..status.clone()
            }],
        )
        .expect("unverified dependency")
    );
    assert!(
        !dependencies_ready(
            &snapshot.project_revision,
            &snapshot.task_snapshot,
            &[DependencyStatus {
                merged_into: Some("release".into()),
                ..status.clone()
            }],
        )
        .expect("dependency not merged into base")
    );
    assert!(
        !dependencies_ready(
            &snapshot.project_revision,
            &snapshot.task_snapshot,
            &[DependencyStatus {
                evidence_project_revision_sha256: "b".repeat(64),
                ..status
            }],
        )
        .expect("stale dependency evidence")
    );

    let changed = rootless_project();
    replace_in(
        &changed.root().join(".autodev/config.yaml"),
        "merge_mode: auto_after_gates",
        "merge_mode: human",
    );
    let changed_revision = project_revision_with_api(changed.root(), &rootless_project_api)
        .expect("changed project revision");
    let error = dependencies_ready(&changed_revision, &snapshot.task_snapshot, &[])
        .expect_err("old task snapshot");
    assert!(
        error.to_string().contains("another project revision"),
        "{error}"
    );
}

#[test]
fn planning_transition_must_follow_a_known_local_task() {
    let valid = TempProject::from_template();
    replace_in(
        &valid.root().join("tasks.yaml"),
        "tasks:\n",
        concat!(
            "required_planning_transition:\n",
            "  after_task: first-verifiable-outcome\n",
            "  reason: task_source_cutover\n",
            "tasks:\n",
        ),
    );
    approve_local(valid.root());
    let no_github =
        |_endpoint: &str, _paginated: bool| -> autodev_planning_revision::Result<Value> {
            panic!("local validation must not call GitHub")
        };
    assert_eq!(
        validate_with_api(valid.root(), &no_github).expect("known transition task"),
        None
    );

    let unknown = TempProject::from_template();
    replace_in(
        &unknown.root().join("tasks.yaml"),
        "tasks:\n",
        concat!(
            "required_planning_transition:\n",
            "  after_task: missing-task\n",
            "  reason: task_source_cutover\n",
            "tasks:\n",
        ),
    );
    approve_local(unknown.root());
    assert_local_error(
        &unknown,
        "required_planning_transition must name a known task",
    );
}

#[test]
fn github_projection_preserves_recursive_order_dependencies_and_golden_digest() {
    let project = github_project();
    let responses = github_fixture();
    let api = fake_github_api(&responses);
    let before_projection = snapshot_tree(project.root());

    let output = task_source_projection_with_api(project.root(), &api).expect("valid projection");
    let projection = serde_json::to_value(&output.projection).expect("serialize projection");

    assert_eq!(projection, expected_projection());
    assert_eq!(
        output.sha256,
        "9b680dc41e947ce6b31f2e5c9f38471431066fcdc24cc5b6ad64af413e143880"
    );
    assert_eq!(before_projection, snapshot_tree(project.root()));
    assert!(!project.root().join("tasks.yaml").exists());

    approve_github(project.root(), &output.sha256);
    let before_validation = snapshot_tree(project.root());
    let validated = validate_with_api(project.root(), &api).expect("approved GitHub revision");
    assert_eq!(validated, Some(output.clone()));
    assert_eq!(before_validation, snapshot_tree(project.root()));

    let mut changed = responses.clone();
    changed
        .get_mut(&("repos/example/autodev/issues/10".into(), false))
        .and_then(Value::as_object_mut)
        .expect("root issue")
        .insert("title".into(), json!("Changed after approval"));
    let changed_api = fake_github_api(&changed);
    let error = validate_with_api(project.root(), &changed_api).expect_err("stale approval");
    assert!(
        error
            .to_string()
            .contains("does not match the current GitHub Issue Graph"),
        "{error}"
    );
}

#[test]
fn github_digest_tracks_planning_fields_but_not_execution_metadata() {
    let project = github_project();
    let responses = github_fixture();
    let baseline = project_github(project.root(), &responses);

    let mut changed = responses.clone();
    changed
        .get_mut(&("repos/example/autodev/issues/10".into(), false))
        .and_then(Value::as_object_mut)
        .expect("root issue")
        .insert("title".into(), json!("Changed plan"));
    assert_ne!(
        baseline.sha256,
        project_github(project.root(), &changed).sha256
    );

    let mut changed = responses.clone();
    let task = changed
        .get_mut(&(
            "repos/example/autodev/issues/10/sub_issues?per_page=100".into(),
            true,
        ))
        .and_then(Value::as_array_mut)
        .expect("root children")
        .first_mut()
        .and_then(Value::as_object_mut)
        .expect("task issue");
    let body = task.get("body").and_then(Value::as_str).expect("task body");
    task.insert(
        "body".into(),
        json!(body.replace("Prepare the contract.", "Prepare the revised contract.")),
    );
    assert_ne!(
        baseline.sha256,
        project_github(project.root(), &changed).sha256
    );

    let mut changed = responses.clone();
    changed
        .get_mut(&(
            "repos/example/autodev/issues/10/sub_issues?per_page=100".into(),
            true,
        ))
        .and_then(Value::as_array_mut)
        .expect("root children")
        .reverse();
    assert_ne!(
        baseline.sha256,
        project_github(project.root(), &changed).sha256
    );

    let mut changed = responses.clone();
    changed
        .get_mut(&(
            "repos/example/autodev/issues/11/sub_issues?per_page=100".into(),
            true,
        ))
        .and_then(Value::as_array_mut)
        .expect("nested children")
        .clear();
    assert_ne!(
        baseline.sha256,
        project_github(project.root(), &changed).sha256
    );

    let mut changed = responses.clone();
    changed
        .get_mut(&(
            "repos/example/autodev/issues/11/dependencies/blocking?per_page=100".into(),
            true,
        ))
        .and_then(Value::as_array_mut)
        .expect("blocking response")
        .clear();
    changed
        .get_mut(&(
            "repos/example/autodev/issues/12/dependencies/blocked_by?per_page=100".into(),
            true,
        ))
        .and_then(Value::as_array_mut)
        .expect("blocked-by response")
        .clear();
    assert_ne!(
        baseline.sha256,
        project_github(project.root(), &changed).sha256
    );

    let mut metadata_only = responses.clone();
    for response in metadata_only.values_mut() {
        mutate_issue_metadata(response);
    }
    assert_eq!(baseline, project_github(project.root(), &metadata_only));
}

#[test]
fn github_projection_fails_closed_on_api_and_membership_errors() {
    let project = github_project();
    let before = snapshot_tree(project.root());
    let failed_api =
        |_endpoint: &str, _paginated: bool| Err(ValidationError::new("connection reset"));
    let error = task_source_projection_with_api(project.root(), &failed_api)
        .expect_err("network failure must reject the graph");
    assert!(
        error.to_string().contains("GitHub API request failed"),
        "{error}"
    );
    assert_eq!(before, snapshot_tree(project.root()));

    let mut cross_repository = github_fixture();
    first_root_child(&mut cross_repository).insert(
        "repository_url".into(),
        json!("https://api.github.com/repos/example/other"),
    );
    assert_github_error(&project, &cross_repository, "belongs to another repository");

    let mut pull_request = github_fixture();
    first_root_child(&mut pull_request).insert(
        "pull_request".into(),
        json!({"url": "https://api.github.com/pulls/1"}),
    );
    assert_github_error(&project, &pull_request, "is a pull request");

    let mut external_dependency = github_fixture();
    let outside = github_issue(999, 99, "Outside", &task_body("Outside."));
    response_array_mut(
        &mut external_dependency,
        "repos/example/autodev/issues/11/dependencies/blocked_by?per_page=100",
    )
    .push(outside);
    assert_github_error(
        &project,
        &external_dependency,
        "outside the root Issue Graph",
    );

    let mut malformed_list = github_fixture();
    malformed_list.insert(
        (
            "repos/example/autodev/issues/10/sub_issues?per_page=100".into(),
            true,
        ),
        json!({}),
    );
    assert_github_error(&project, &malformed_list, "response must be a list");
}

#[test]
fn github_projection_rejects_invalid_task_bodies_and_graph_cycles() {
    let project = github_project();

    let mut checkbox = github_fixture();
    replace_issue_body(
        first_root_child(&mut checkbox),
        "- Validate the result.",
        "- [ ] Validate the result.",
    );
    assert_github_error(&project, &checkbox, "must not use mutable checkboxes");

    let mut missing_section = github_fixture();
    replace_issue_body(
        first_root_child(&mut missing_section),
        "## Outcome",
        "## Result",
    );
    assert_github_error(&project, &missing_section, "missing ## Outcome");

    let mut missing_reference = github_fixture();
    replace_issue_body(
        first_root_child(&mut missing_reference),
        "docs/project-overview.md#goal",
        "docs/missing.md#goal",
    );
    assert_github_error(&project, &missing_reference, "reference is missing");

    let mut duplicate_membership = github_fixture();
    let duplicate = response_array_mut(
        &mut duplicate_membership,
        "repos/example/autodev/issues/10/sub_issues?per_page=100",
    )[0]
    .clone();
    response_array_mut(
        &mut duplicate_membership,
        "repos/example/autodev/issues/10/sub_issues?per_page=100",
    )
    .push(duplicate);
    assert_github_error(&project, &duplicate_membership, "duplicate issue id");

    let mut dependency_cycle = github_fixture();
    let prepare = response_array_mut(
        &mut dependency_cycle,
        "repos/example/autodev/issues/10/sub_issues?per_page=100",
    )[0]
    .clone();
    response_array_mut(
        &mut dependency_cycle,
        "repos/example/autodev/issues/12/dependencies/blocking?per_page=100",
    )
    .push(prepare);
    assert_github_error(&project, &dependency_cycle, "task dependency cycle");
}

#[test]
fn github_cli_contract_pins_version_and_rejects_incomplete_pages() {
    assert_eq!(
        github_api_args(
            "repos/example/autodev/issues/10/sub_issues?per_page=100",
            true,
        ),
        vec![
            "api",
            "repos/example/autodev/issues/10/sub_issues?per_page=100",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            &format!("X-GitHub-Api-Version: {GITHUB_API_VERSION}"),
            "--paginate",
            "--slurp",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>()
    );
    assert_eq!(
        parse_github_response(br#"[[{"id":1}],[]]"#, true).expect("complete pages"),
        json!([{"id": 1}])
    );

    for invalid in [
        br#"{}"#.as_slice(),
        br#"[]"#.as_slice(),
        br#"[{}]"#.as_slice(),
    ] {
        let error = parse_github_response(invalid, true).expect_err("incomplete pages");
        assert!(
            error
                .to_string()
                .contains("paginated response is incomplete"),
            "{error}"
        );
    }
    let error = parse_github_response(b"not JSON", true).expect_err("invalid JSON");
    assert!(error.to_string().contains("invalid JSON"), "{error}");
}

#[cfg(unix)]
#[test]
fn github_cli_disables_prompts_and_requests_paginated_sub_issues() {
    use std::os::unix::fs::PermissionsExt;

    let project = github_project();
    let bin = project.root().join("fake-bin");
    let log = project.root().join("gh.log");
    fs::create_dir(&bin).expect("create fake bin");
    let gh = bin.join("gh");
    fs::write(
        &gh,
        r#"#!/bin/sh
printf 'prompt=<%s>\n' "$GH_PROMPT_DISABLED" >> "$AUTODEV_GH_TEST_LOG"
printf 'arg=<%s>\n' "$@" >> "$AUTODEV_GH_TEST_LOG"
case "$2" in
  repos/example/autodev/issues/10)
    printf '%s' '{"id":100,"number":10,"title":"Autodev plan","body":"Approved task container.","repository_url":"https://api.github.com/repos/example/autodev"}'
    ;;
  *)
    printf '%s' '[[]]'
    ;;
esac
"#,
    )
    .expect("write fake gh");
    fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).expect("make fake gh executable");

    let mut paths = vec![bin];
    if let Some(current) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&current));
    }
    let path = std::env::join_paths(paths).expect("build test PATH");
    let output = Command::new(env!("CARGO_BIN_EXE_autodev-planning-revision"))
        .args([
            "--print-task-projection".as_ref(),
            project.root().as_os_str(),
        ])
        .env("PATH", path)
        .env("AUTODEV_GH_TEST_LOG", &log)
        .output()
        .expect("run projection CLI with fake gh");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("GitHub Issue Graph must contain at least one task")
    );
    let calls = read(&log);
    assert!(calls.lines().all(|line| line != "prompt=<>"), "{calls}");
    assert!(calls.contains("prompt=<1>"), "{calls}");
    assert!(calls.contains("arg=<--paginate>"), "{calls}");
    assert!(calls.contains("arg=<--slurp>"), "{calls}");
    assert!(
        calls.contains("arg=<X-GitHub-Api-Version: 2026-03-10>"),
        "{calls}"
    );
}

#[test]
fn issue_template_uses_the_approval_bound_sections_without_checkboxes() {
    let template =
        read(&Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/ISSUE_TEMPLATE/autodev-task.md"));
    assert!(template.contains("name: Autodev task"));
    assert!(template.contains("about: Define one approval-bound task outcome"));
    for heading in ["Outcome", "Planning references", "Verification"] {
        assert_eq!(
            template.matches(&format!("## {heading}\n")).count(),
            1,
            "{heading}"
        );
    }
    assert!(!template.contains("- [ ]"));
    assert!(!template.contains("- [x]"));
    assert!(!template.contains("- [X]"));
}

#[test]
fn captured_skill_artifacts_keep_the_planning_and_learning_contract() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let planning = manifest.join("test/fixtures/planning-skill/project");
    let no_github =
        |_endpoint: &str, _paginated: bool| -> autodev_planning_revision::Result<Value> {
            panic!("local validation must not call GitHub")
        };
    assert_eq!(
        validate_with_api(&planning, &no_github).expect("approved planning fixture"),
        None
    );
    let overview = read(&planning.join("docs/project-overview.md"));
    assert!(overview.contains("](../../knowledge/previous-autodev-retrospective.md)"));
    assert!(overview.contains("evidence, not authority"));
    assert!(!planning.join("evidence").exists());

    let fixture = manifest.join("test/fixtures/execution-learning");
    let project = fixture.join("project");
    assert_eq!(
        validate_with_api(&project, &no_github).expect("approved execution fixture"),
        None
    );
    let rows = read(&project.join("source/volunteers.csv"))
        .lines()
        .map(parse_csv_row)
        .collect::<Vec<_>>();
    assert_eq!(
        rows.first().expect("CSV header"),
        &["volunteer_id", "name", "arrival_window"]
    );
    let expected = format!(
        "# Volunteer check-in\n\n{}\n",
        rows[1..]
            .iter()
            .map(|row| format!("- {}", row.join(" | ")))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(read(&project.join("output/check-in.md")), expected);

    let approval: LocalApprovalFixture = parse_yaml_file(&project.join(".autodev/approval.yaml"));
    let (evidence, evidence_body): (EvidenceFixture, _) =
        parse_markdown(&project.join("evidence/build-check-in-sheet.md"));
    assert_eq!(evidence.task, "build-check-in-sheet");
    assert_eq!(evidence.status, "verified");
    assert_eq!(evidence.planning_revision, approval.files);
    assert!(evidence.verified_at.contains('T'));
    assert!(evidence_body.contains("[Volunteer check-in sheet](../output/check-in.md)"));

    let mut candidates = fs::read_dir(fixture.join("candidate-inbox"))
        .expect("candidate inbox")
        .map(|entry| {
            let path = entry.expect("candidate entry").path();
            parse_markdown::<CandidateFixture>(&path)
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.0.status.cmp(&right.0.status));
    assert_eq!(
        candidates
            .iter()
            .map(|(metadata, _)| metadata.status.as_str())
            .collect::<Vec<_>>(),
        ["dismissed", "pending"]
    );
    let (pending, body) = candidates
        .iter()
        .find(|(metadata, _)| metadata.status == "pending")
        .expect("pending candidate");
    assert_eq!(pending.task.as_deref(), Some("build-check-in-sheet"));
    for heading in ["Learning", "Context", "Applies when", "Evidence"] {
        assert!(body.contains(&format!("## {heading}\n\n")), "{heading}");
    }
}

#[derive(Deserialize)]
struct LocalApprovalFixture {
    files: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct EvidenceFixture {
    task: String,
    status: String,
    verified_at: String,
    planning_revision: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct CandidateFixture {
    status: String,
    task: Option<String>,
}

fn assert_invalid_local_task(expected: &str, mutate: impl FnOnce(&Path)) {
    let project = TempProject::from_template();
    mutate(project.root());
    approve_local(project.root());
    assert_local_error(&project, expected);
}

fn assert_local_error(project: &TempProject, expected: &str) {
    let before = snapshot_tree(project.root());
    let no_github =
        |_endpoint: &str, _paginated: bool| -> autodev_planning_revision::Result<Value> {
            panic!("local validation must not call GitHub")
        };
    let error = validate_with_api(project.root(), &no_github).expect_err(expected);
    assert!(
        error.to_string().contains(expected),
        "expected {expected:?}, got {error}"
    );
    assert_eq!(before, snapshot_tree(project.root()));
}

fn rootless_project() -> TempProject {
    let project = TempProject::from_template();
    fs::write(
        project.root().join(".autodev/config.yaml"),
        concat!(
            "project_overview: docs/project-overview.md\n",
            "task_source:\n",
            "  type: github_issues\n",
            "  repository: example/autodev\n",
            "authorization:\n",
            "  ready_label: autodev:ready\n",
            "  refusal_labels: [autodev:needs-input, autodev:human-needed]\n",
            "  authorizer_roles: [write, admin]\n",
            "  cancel_roles: [write, admin]\n",
            "delivery:\n",
            "  base_branch: main\n",
            "  protected_paths: [.github/workflows/**, .autodev/**]\n",
            "  required_checks: [project-ci, autodev/gate]\n",
            "  review:\n",
            "    required_approvals: 1\n",
            "    require_code_owner_review: false\n",
            "    require_thread_resolution: true\n",
            "    require_up_to_date_branch: true\n",
            "  merge_mode: auto_after_gates\n",
            "  correction_budget: 2\n",
            "engine_policy:\n",
            "  allowed_providers: [openai, anthropic]\n",
            "  data_use_boundary: no_training\n",
            "  cost_class: standard\n",
            "selected_engine:\n",
            "  provider: openai\n",
            "  name: codex\n",
            "  version: gpt-5.6\n",
            "tools:\n",
            "  coding-agent:\n",
            "    purpose: Implement approved repository tasks.\n",
            "    interface: cli\n",
            "    permissions: [repository_read, patch_output]\n",
            "knowledge:\n",
            "  sources:\n",
            "    - alias: personal\n",
            "      path: /machine-one/knowledge\n",
            "  candidate_carrier:\n",
            "    identity: github:example/private-candidates\n",
            "    visibility: private\n",
            "  target:\n",
            "    identity: github:example/dev-knowledge\n",
            "    visibility: public\n",
        ),
    )
    .expect("write rootless config");
    fs::remove_file(project.root().join("tasks.yaml")).expect("remove local task source");
    project
}

fn approved_rootless_project() -> TempProject {
    let project = rootless_project();
    let revision =
        project_revision_with_api(project.root(), &rootless_project_api).expect("project revision");
    approve_rootless_project(project.root(), &revision.sha256);
    project
}

fn rootless_project_api(
    endpoint: &str,
    paginated: bool,
) -> autodev_planning_revision::Result<Value> {
    if endpoint == "repos/example/autodev" && !paginated {
        Ok(json!({
            "id": 1000,
            "node_id": "R_example_autodev",
            "full_name": "example/autodev",
        }))
    } else {
        Err(ValidationError::new(format!(
            "project revision unexpectedly fetched {endpoint}"
        )))
    }
}

fn approve_rootless_project(root: &Path, project_sha256: &str) {
    fs::write(
        root.join(".autodev/approval.yaml"),
        format!(
            concat!(
                "project: fixture\n",
                "status: approved\n",
                "approved_by: user\n",
                "approved_at: \"2026-08-16T02:42:17+09:00\"\n",
                "planning_revision:\n",
                "  project_overview:\n",
                "    path: docs/project-overview.md\n",
                "    sha256: {}\n",
                "  project:\n",
                "    sha256: {}\n",
            ),
            sha256_file(&root.join("docs/project-overview.md")),
            project_sha256,
        ),
    )
    .expect("write rootless approval");
}

fn rootless_task_fixture() -> ApiResponses {
    let mut task = github_issue(
        201,
        21,
        "Implement scoped authorization",
        &task_body("Create the authorization record."),
    );
    task["labels"] = json!([{"name": "autodev:ready"}]);
    let dependency = github_issue(
        200,
        20,
        "Approve project policy",
        &task_body("Approve the project policy."),
    );
    BTreeMap::from([
        (
            ("repos/example/autodev".into(), false),
            rootless_project_api("repos/example/autodev", false).expect("repository fixture"),
        ),
        (("repos/example/autodev/issues/21".into(), false), task),
        (
            (
                "repos/example/autodev/issues/21/dependencies/blocked_by?per_page=100".into(),
                true,
            ),
            json!([dependency]),
        ),
    ])
}

fn ready_event(snapshot: &ValidatedTaskSnapshot, run_id: u64, actor_role: &str) -> ReadyEvent {
    ReadyEvent {
        repository_id: 1000,
        run_id,
        issue_id: 201,
        issue_node_id: "I_201".into(),
        issue_number: 21,
        label: "autodev:ready".into(),
        actor_id: 7,
        actor: "dante".into(),
        actor_role: actor_role.into(),
        task_sha256: snapshot.task_snapshot.sha256.clone(),
        project_revision_sha256: snapshot.project_revision.sha256.clone(),
    }
}

fn authorize_rootless_task(
    project: &ProjectRevisionOutput,
    task: &TaskSnapshotOutput,
    agent_input: &[u8],
    event: &ReadyEvent,
    prior: &[TaskAuthorization],
) -> autodev_planning_revision::Result<AuthorizationDecision> {
    let role = event.actor_role.clone();
    let api = |endpoint: &str, paginated: bool| {
        assert_eq!(
            endpoint,
            "repos/example/autodev/collaborators/dante/permission"
        );
        assert!(!paginated);
        Ok(json!({
            "permission": role,
            "role_name": role,
            "user": {"id": 7, "login": "dante"},
        }))
    };
    authorize_task_with_api(project, task, agent_input, event, prior, &api)
}

fn transition_rootless_episode(
    project: &ProjectRevisionOutput,
    current: &TaskAuthorization,
    event: &EpisodeEvent,
    actor_role: Option<&str>,
) -> autodev_planning_revision::Result<autodev_planning_revision::TransitionDecision> {
    let mut event = event.clone();
    event.actor = actor_role.map(|role| RepositoryActor {
        id: 7,
        login: "dante".into(),
        role: role.into(),
    });
    let role = actor_role.map(str::to_owned);
    let api = |endpoint: &str, paginated: bool| {
        assert_eq!(
            endpoint,
            "repos/example/autodev/collaborators/dante/permission"
        );
        assert!(!paginated);
        Ok(json!({
            "permission": role,
            "role_name": role,
            "user": {"id": 7, "login": "dante"},
        }))
    };
    transition_episode_with_api(project, current, &event, &api)
}

fn episode_event(
    authorization: &TaskAuthorization,
    kind: EpisodeEventKind,
    event_id: u64,
) -> EpisodeEvent {
    EpisodeEvent {
        kind,
        repository_id: authorization.episode.repository_id,
        event_id,
        episode: authorization.episode.clone(),
        project_revision_sha256: authorization.project_revision_sha256.clone(),
        actor: None,
    }
}

fn verified_evidence(authorization: &TaskAuthorization) -> VerifiedEvidence {
    VerifiedEvidence {
        task_sha256: authorization.episode.task_sha256.clone(),
        project_revision_sha256: authorization.project_revision_sha256.clone(),
        authorization_generation: authorization.episode.authorization_generation,
        verified: true,
    }
}

fn authorized_rootless_task() -> (ProjectRevisionOutput, TaskAuthorization) {
    let project = approved_rootless_project();
    let snapshot = task_snapshot_with_api(
        project.root(),
        21,
        &fake_github_api(&rootless_task_fixture()),
    )
    .expect("task snapshot");
    let authorization = authorize_rootless_task(
        &snapshot.project_revision,
        &snapshot.task_snapshot,
        b"filtered",
        &ready_event(&snapshot, 1, "write"),
        &[],
    )
    .expect("task authorization");
    (snapshot.project_revision, authorization.authorization)
}

fn github_project() -> TempProject {
    let project = TempProject::from_template();
    fs::write(
        project.root().join(".autodev/config.yaml"),
        concat!(
            "project_overview: docs/project-overview.md\n",
            "task_source:\n",
            "  type: github_issues\n",
            "  repository: example/autodev\n",
            "  root_issue: 10\n",
        ),
    )
    .expect("write GitHub config");
    fs::remove_file(project.root().join("tasks.yaml")).expect("remove local task source");
    project
}

fn approve_github(root: &Path, projection_sha256: &str) {
    fs::write(
        root.join(".autodev/approval.yaml"),
        format!(
            concat!(
                "project: fixture\n",
                "status: approved\n",
                "approved_by: user\n",
                "approved_at: \"2026-08-10T20:00:00+09:00\"\n",
                "planning_revision:\n",
                "  project_overview:\n",
                "    path: docs/project-overview.md\n",
                "    sha256: {}\n",
                "  task_source:\n",
                "    type: github_issues\n",
                "    repository: example/autodev\n",
                "    root_issue: 10\n",
                "    sha256: {}\n",
            ),
            sha256_file(&root.join("docs/project-overview.md")),
            projection_sha256,
        ),
    )
    .expect("write GitHub approval");
}

fn project_github(
    root: &Path,
    responses: &ApiResponses,
) -> autodev_planning_revision::ProjectionOutput {
    let api = fake_github_api(responses);
    task_source_projection_with_api(root, &api).expect("project GitHub fixture")
}

fn fake_github_api(
    responses: &ApiResponses,
) -> impl Fn(&str, bool) -> autodev_planning_revision::Result<Value> + '_ {
    move |endpoint, paginated| {
        responses
            .get(&(endpoint.to_owned(), paginated))
            .cloned()
            .ok_or_else(|| ValidationError::new(format!("missing fixture response: {endpoint}")))
    }
}

fn assert_github_error(project: &TempProject, responses: &ApiResponses, expected: &str) {
    let before = snapshot_tree(project.root());
    let api = fake_github_api(responses);
    let error = task_source_projection_with_api(project.root(), &api).expect_err(expected);
    assert!(
        error.to_string().contains(expected),
        "expected {expected:?}, got {error}"
    );
    assert_eq!(before, snapshot_tree(project.root()));
}

fn github_fixture() -> ApiResponses {
    let root = github_issue(100, 10, "Autodev plan", "Approved task container.");
    let prepare = github_issue(
        101,
        11,
        "Prepare the contract",
        &task_body("Prepare the contract."),
    );
    let verify = github_issue(
        102,
        12,
        "Verify the cutover",
        &task_body("Verify the cutover."),
    );
    let nested = github_issue(
        103,
        13,
        "Check nested work",
        &task_body("Check nested work."),
    );

    BTreeMap::from([
        (("repos/example/autodev/issues/10".into(), false), root),
        (
            (
                "repos/example/autodev/issues/10/sub_issues?per_page=100".into(),
                true,
            ),
            json!([prepare, verify]),
        ),
        (
            (
                "repos/example/autodev/issues/11/sub_issues?per_page=100".into(),
                true,
            ),
            json!([nested]),
        ),
        (
            (
                "repos/example/autodev/issues/12/sub_issues?per_page=100".into(),
                true,
            ),
            json!([]),
        ),
        (
            (
                "repos/example/autodev/issues/13/sub_issues?per_page=100".into(),
                true,
            ),
            json!([]),
        ),
        (
            (
                "repos/example/autodev/issues/11/dependencies/blocked_by?per_page=100".into(),
                true,
            ),
            json!([]),
        ),
        (
            (
                "repos/example/autodev/issues/11/dependencies/blocking?per_page=100".into(),
                true,
            ),
            json!([verify]),
        ),
        (
            (
                "repos/example/autodev/issues/12/dependencies/blocked_by?per_page=100".into(),
                true,
            ),
            json!([prepare]),
        ),
        (
            (
                "repos/example/autodev/issues/12/dependencies/blocking?per_page=100".into(),
                true,
            ),
            json!([]),
        ),
        (
            (
                "repos/example/autodev/issues/13/dependencies/blocked_by?per_page=100".into(),
                true,
            ),
            json!([]),
        ),
        (
            (
                "repos/example/autodev/issues/13/dependencies/blocking?per_page=100".into(),
                true,
            ),
            json!([]),
        ),
    ])
}

fn github_issue(id: u64, number: u64, title: &str, body: &str) -> Value {
    json!({
        "id": id,
        "node_id": format!("I_{id}"),
        "number": number,
        "title": title,
        "body": body,
        "repository_url": "https://api.github.com/repos/example/autodev",
        "state": "open",
        "labels": [],
        "assignees": [],
        "comments": 0,
    })
}

fn task_body(outcome: &str) -> String {
    format!(
        concat!(
            "## Outcome\n\n",
            "{}\n\n",
            "## Planning references\n\n",
            "- [Project goal](docs/project-overview.md#goal)\n\n",
            "## Verification\n\n",
            "- Validate the result.\n",
        ),
        outcome,
    )
}

fn expected_projection() -> Value {
    json!({
        "repository": "example/autodev",
        "root_issue": 10,
        "issues": [
            {
                "id": 100,
                "number": 10,
                "title": "Autodev plan",
                "body": "Approved task container.",
                "parent_id": null,
                "position": 0,
            },
            {
                "id": 101,
                "number": 11,
                "title": "Prepare the contract",
                "body": task_body("Prepare the contract."),
                "parent_id": 100,
                "position": 0,
            },
            {
                "id": 103,
                "number": 13,
                "title": "Check nested work",
                "body": task_body("Check nested work."),
                "parent_id": 101,
                "position": 0,
            },
            {
                "id": 102,
                "number": 12,
                "title": "Verify the cutover",
                "body": task_body("Verify the cutover."),
                "parent_id": 100,
                "position": 1,
            },
        ],
        "dependencies": [
            {"blocking_id": 101, "blocked_id": 102},
        ],
    })
}

fn first_root_child(responses: &mut ApiResponses) -> &mut serde_json::Map<String, Value> {
    response_array_mut(
        responses,
        "repos/example/autodev/issues/10/sub_issues?per_page=100",
    )
    .first_mut()
    .and_then(Value::as_object_mut)
    .expect("first root child")
}

fn response_array_mut<'a>(responses: &'a mut ApiResponses, endpoint: &str) -> &'a mut Vec<Value> {
    responses
        .get_mut(&(endpoint.to_owned(), true))
        .and_then(Value::as_array_mut)
        .expect("array response")
}

fn replace_issue_body(issue: &mut serde_json::Map<String, Value>, from: &str, to: &str) {
    let body = issue
        .get("body")
        .and_then(Value::as_str)
        .expect("issue body");
    issue.insert("body".into(), json!(body.replace(from, to)));
}

fn mutate_issue_metadata(value: &mut Value) {
    match value {
        Value::Array(values) => values.iter_mut().for_each(mutate_issue_metadata),
        Value::Object(issue) if issue.contains_key("id") => {
            issue.insert("state".into(), json!("closed"));
            issue.insert("labels".into(), json!([{"name": "changed"}]));
            issue.insert("assignees".into(), json!([{"login": "octocat"}]));
            issue.insert("comments".into(), json!(99));
        }
        _ => {}
    }
}

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn from_template() -> Self {
        let root = create_temp_dir();
        copy_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/project"),
            &root,
        );
        replace_in(
            &root.join("docs/project-overview.md"),
            "- Replace this line with unresolved material questions. Use `None.` only when none remain.",
            "None.",
        );
        replace_in(
            &root.join("tasks.yaml"),
            "project: replace-with-project-id",
            "project: fixture",
        );
        approve_local(&root);
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn create_temp_dir() -> PathBuf {
    for _ in 0..16 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "autodev-planning-revision-test-{}-{nanos}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create temporary project: {error}"),
        }
    }
    panic!("could not allocate a unique temporary project")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination");
    for entry in fs::read_dir(source).expect("read template directory") {
        let entry = entry.expect("template entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy template file");
        }
    }
}

fn approve_local(root: &Path) {
    fs::write(
        root.join(".autodev/approval.yaml"),
        format!(
            concat!(
                "project: fixture\n",
                "status: approved\n",
                "approved_by: user\n",
                "approved_at: \"2026-08-10T20:00:00+09:00\"\n",
                "files:\n",
                "  docs/project-overview.md: {}\n",
                "  tasks.yaml: {}\n",
            ),
            sha256_file(&root.join("docs/project-overview.md")),
            sha256_file(&root.join("tasks.yaml")),
        ),
    )
    .expect("write local approval");
}

fn sha256_file(path: &Path) -> String {
    sha256_bytes(&fs::read(path).expect("read digest input"))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("write digest");
    }
    output
}

#[derive(Debug, Eq, PartialEq)]
enum TreeEntry {
    Directory,
    File(Vec<u8>),
}

fn snapshot_tree(root: &Path) -> BTreeMap<PathBuf, TreeEntry> {
    fn visit(root: &Path, directory: &Path, entries: &mut BTreeMap<PathBuf, TreeEntry>) {
        let mut children = fs::read_dir(directory)
            .expect("read project tree")
            .collect::<std::io::Result<Vec<_>>>()
            .expect("read project entries");
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            let path = child.path();
            let relative = path
                .strip_prefix(root)
                .expect("relative project path")
                .to_owned();
            if child.file_type().expect("project entry type").is_dir() {
                entries.insert(relative, TreeEntry::Directory);
                visit(root, &path, entries);
            } else {
                entries.insert(
                    relative,
                    TreeEntry::File(fs::read(path).expect("read project file")),
                );
            }
        }
    }

    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

fn replace_in(path: &Path, from: &str, to: &str) {
    let text = read(path);
    assert!(
        text.contains(from),
        "missing replacement target {from:?} in {}",
        path.display()
    );
    fs::write(path, text.replacen(from, to, 1)).expect("write replacement");
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn parse_yaml_file<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    yaml_serde::from_str(&read(path))
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn parse_markdown<T: for<'de> Deserialize<'de>>(path: &Path) -> (T, String) {
    let text = read(path);
    let rest = text
        .strip_prefix("---\n")
        .unwrap_or_else(|| panic!("missing frontmatter in {}", path.display()));
    let (frontmatter, body) = rest
        .split_once("\n---\n")
        .unwrap_or_else(|| panic!("unclosed frontmatter in {}", path.display()));
    let metadata = yaml_serde::from_str(frontmatter)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
    (metadata, body.to_owned())
}

fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut characters = line.chars().peekable();
    let mut quoted = false;
    while let Some(character) = characters.next() {
        match character {
            '"' if quoted && characters.peek() == Some(&'"') => {
                field.push('"');
                characters.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut field)),
            _ => field.push(character),
        }
    }
    assert!(!quoted, "unclosed CSV quote");
    fields.push(field);
    fields
}
