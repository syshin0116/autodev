use autodev_planning_revision::{
    KaneoProjectedTask, KaneoTaskProjectionInput, KaneoTaskRelation, kaneo_task_projection,
    validate_kaneo_task_projection,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn kaneo_projection_binds_task_content_and_relations() {
    let project = Project::new();
    let input = projection_input();
    let projection = kaneo_task_projection(project.root(), input.clone()).expect("projection");

    validate_kaneo_task_projection(project.root(), input.clone()).expect("current projection");

    let mut changed = input.clone();
    changed.tasks[0].title.push_str(" changed");
    let changed = validate_kaneo_task_projection(project.root(), changed)
        .expect("fresh external task state is authoritative");
    assert_ne!(projection.sha256, changed.sha256);

    let mut reordered = input;
    reordered.tasks.reverse();
    reordered.relations.reverse();
    validate_kaneo_task_projection(project.root(), reordered)
        .expect("response order is not planning content");
}

#[test]
fn kaneo_projection_rejects_invalid_membership_and_dependency_cycles() {
    let project = Project::new();
    let mut wrong_project = projection_input();
    wrong_project.tasks[0].project_id = "another-project".to_owned();
    let error = kaneo_task_projection(project.root(), wrong_project)
        .expect_err("cross-project task must fail");
    assert!(error.to_string().contains("another project"), "{error}");

    let mut cycle = projection_input();
    cycle.relations.push(KaneoTaskRelation {
        source_task_id: "task-2".to_owned(),
        target_task_id: "task-1".to_owned(),
        relation_type: "blocks".to_owned(),
    });
    let error =
        kaneo_task_projection(project.root(), cycle).expect_err("dependency cycle must fail");
    assert!(error.to_string().contains("dependency cycle"), "{error}");
}

fn projection_input() -> KaneoTaskProjectionInput {
    KaneoTaskProjectionInput {
        server: "https://cloud.kaneo.app/api/mcp".to_owned(),
        workspace_id: "workspace-1".to_owned(),
        project_id: "project-1".to_owned(),
        tasks: vec![
            task("task-1", 1, "Write the plan"),
            task("task-2", 2, "Build the result"),
        ],
        relations: vec![KaneoTaskRelation {
            source_task_id: "task-1".to_owned(),
            target_task_id: "task-2".to_owned(),
            relation_type: "blocks".to_owned(),
        }],
    }
}

fn task(id: &str, number: u64, title: &str) -> KaneoProjectedTask {
    KaneoProjectedTask {
        id: id.to_owned(),
        number,
        project_id: "project-1".to_owned(),
        title: title.to_owned(),
        description: concat!(
            "## Outcome\n\n",
            "The requested state exists.\n\n",
            "## Planning references\n\n",
            "- docs/project-overview.md#success-criteria\n\n",
            "## Verification\n\n",
            "- Inspect the resulting state.\n",
        )
        .to_owned(),
    }
}

struct Project {
    root: PathBuf,
}

impl Project {
    fn new() -> Self {
        let unique = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "autodev-kaneo-{}-{nanos}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(root.join(".autodev")).expect("state directory");
        fs::create_dir_all(root.join("docs")).expect("docs directory");
        fs::write(
            root.join("docs/project-overview.md"),
            concat!(
                "---\n",
                "id: kaneo-test\n",
                "---\n\n",
                "# Test project\n\n",
                "## Success criteria\n\n",
                "- The result works.\n\n",
                "## Open questions\n\n",
                "None.\n",
            ),
        )
        .expect("overview");
        fs::write(
            root.join(".autodev/config.yaml"),
            concat!(
                "project_overview: docs/project-overview.md\n",
                "task_source:\n",
                "  type: kaneo\n",
                "  server: https://cloud.kaneo.app/api/mcp\n",
                "  workspace_id: workspace-1\n",
                "  project_id: project-1\n",
            ),
        )
        .expect("config");
        run_git(&root, &["init", "--quiet"]);
        run_git(&root, &["config", "user.email", "autodev@example.com"]);
        run_git(&root, &["config", "user.name", "Autodev Test"]);
        run_git(
            &root,
            &["add", ".autodev/config.yaml", "docs/project-overview.md"],
        );
        run_git(&root, &["commit", "--quiet", "-m", "Add planning files"]);
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_git(root: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {:?}: {}",
        arguments,
        String::from_utf8_lossy(&output.stderr)
    );
}
