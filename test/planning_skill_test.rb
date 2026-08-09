# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "pathname"
require "tmpdir"
require "yaml"

require_relative "../scripts/validate_project"

class PlanningSkillTest < Minitest::Test
  FIXTURE = Pathname.new(__dir__).join("fixtures/planning-skill").expand_path
  EXECUTION_FIXTURE = Pathname.new(__dir__).join("fixtures/execution-learning").expand_path

  def test_forward_test_reaches_an_approved_handoff
    project = FIXTURE.join("project")
    overview = project.join("docs/project-overview.md").read
    task_graph = YAML.safe_load(project.join("tasks.yaml").read, aliases: false)
    tasks = task_graph.fetch("tasks")

    assert_empty Autodev::ProjectValidation.validate(project)
    assert_equal "autodev-rebuild", task_graph.fetch("project")
    assert_includes overview, "](../../knowledge/previous-autodev-retrospective.md)"
    assert_includes overview, "evidence, not authority"
    assert tasks.all? { |task| task.fetch("references").all? { |reference| reference.start_with?("docs/project-overview.md#") } }
    refute project.join("evidence").exist?
  end

  def test_forward_test_executes_an_unchanged_revision_and_proposes_one_novel_learning
    project = EXECUTION_FIXTURE.join("project")
    approval = YAML.safe_load(project.join(".autodev/approval.yaml").read, aliases: false)
    evidence_metadata, evidence_body = read_markdown(project.join("evidence/build-check-in-sheet.md"))

    assert_empty Autodev::ProjectValidation.validate(project)
    assert_equal <<~MARKDOWN, project.join("output/check-in.md").read
      # Volunteer check-in

      - 001 | Kim, Mina | 09:00
      - 002 | Lee Jun | 09:15
    MARKDOWN
    assert_equal "build-check-in-sheet", evidence_metadata.fetch("task")
    assert_equal "verified", evidence_metadata.fetch("status")
    assert_equal approval.fetch("files"), evidence_metadata.fetch("planning_revision")
    assert_includes evidence_body, "Ruby `CSV` parsed both source rows"
    assert_includes evidence_body, "[Volunteer check-in sheet](../output/check-in.md)"

    candidates = Dir[EXECUTION_FIXTURE.join("candidate-inbox/*.md")].sort.map { |path| read_markdown(path) }
    assert_equal ["dismissed", "pending"], candidates.map { |metadata, _body| metadata.fetch("status") }.sort
    pending_metadata, pending_body = candidates.find { |metadata, _body| metadata["status"] == "pending" }
    assert_equal "build-check-in-sheet", pending_metadata.fetch("task")
    assert_includes pending_body, "Parse CSV with a standards-compliant CSV parser"
    %w[Learning Context Applies\ when Evidence].each do |heading|
      assert_match(/^## #{heading}\n\n\S/m, pending_body)
    end
    assert_equal 1, candidates.count { |_metadata, body| body.include?("Numeric-looking identifiers") }

    Dir.mktmpdir("autodev-altered") do |directory|
      blocked_fixture = Pathname.new(directory)
      FileUtils.cp_r("#{EXECUTION_FIXTURE}/.", blocked_fixture)
      altered = blocked_fixture.join("project")
      FileUtils.rm_rf(altered.join("output"))
      FileUtils.rm_rf(altered.join("evidence"))
      blocked_fixture.join("candidate-inbox/parse-quoted-csv-fields.md").delete
      overview = altered.join("docs/project-overview.md")
      overview.write("#{overview.read}\nChanged after approval.\n")
      assert_includes Autodev::ProjectValidation.validate(altered),
        "approved planning file changed: docs/project-overview.md"
      refute altered.join("output").exist?
      refute altered.join("evidence").exist?
      assert_equal ["keep-opaque-identifiers-as-text.md"],
        Dir[blocked_fixture.join("candidate-inbox/*.md")].map { |path| File.basename(path) }
    end

    Dir[Pathname.new(__dir__).join("../evidence/*.md")].each do |path|
      metadata, = read_markdown(path)
      assert_instance_of String, metadata.fetch("verified_at")
      assert metadata.fetch("planning_revision").all? { |relative, digest|
        relative.is_a?(String) && digest.match?(/\A[0-9a-f]{64}\z/)
      }
    end
  end

  private

  def read_markdown(path)
    text = Pathname.new(path).read
    match = text.match(/\A---[ \t]*\r?\n(?<yaml>.*?)\r?\n---[ \t]*\r?\n/m)
    [YAML.safe_load(match[:yaml], aliases: false), match.post_match]
  end
end
