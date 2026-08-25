use autodev_planning_revision::{
    KaneoProjectedTask, KaneoTaskProjectionInput, KaneoTaskRelation, kaneo_task_projection,
    validate_kaneo_task_projection,
};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn kaneo_projection_binds_task_content_and_relations() {
    let project = Project::new();
    let input = projection_input();
    let projection = kaneo_task_projection(project.root(), input.clone()).expect("projection");
    project.approve(&projection.sha256);

    validate_kaneo_task_projection(project.root(), input.clone()).expect("approved projection");

    let mut changed = input.clone();
    changed.tasks[0].title.push_str(" changed");
    let error = validate_kaneo_task_projection(project.root(), changed)
        .expect_err("changed task must invalidate approval");
    assert!(
        error.to_string().contains("current Kaneo Task Graph"),
        "{error}"
    );

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
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn approve(&self, projection_sha256: &str) {
        let overview_sha256 =
            sha256(&fs::read(self.root.join("docs/project-overview.md")).unwrap());
        fs::write(
            self.root.join(".autodev/approval.yaml"),
            format!(
                concat!(
                    "status: approved\n",
                    "approved_by: tester\n",
                    "approved_at: '2026-08-25T00:00:00Z'\n",
                    "planning_revision:\n",
                    "  project_overview:\n",
                    "    path: docs/project-overview.md\n",
                    "    sha256: {}\n",
                    "  task_source:\n",
                    "    type: kaneo\n",
                    "    server: https://cloud.kaneo.app/api/mcp\n",
                    "    workspace_id: workspace-1\n",
                    "    project_id: project-1\n",
                    "    sha256: {}\n",
                ),
                overview_sha256, projection_sha256
            ),
        )
        .expect("approval");
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            write!(&mut output, "{byte:02x}").expect("write digest");
            output
        })
}
