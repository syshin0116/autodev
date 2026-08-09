# frozen_string_literal: true

require "minitest/autorun"
require "pathname"
require "yaml"

require_relative "../scripts/validate_project"

class PlanningSkillTest < Minitest::Test
  FIXTURE = Pathname.new(__dir__).join("fixtures/planning-skill").expand_path

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
end
