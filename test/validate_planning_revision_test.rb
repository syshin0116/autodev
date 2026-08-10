# frozen_string_literal: true

require "date"
require "digest"
require "fileutils"
require "minitest/autorun"
require "pathname"
require "tmpdir"
require "yaml"

require_relative "../scripts/validate_planning_revision"

class ValidatePlanningRevisionTest < Minitest::Test
  def test_valid_approved_project
    with_project do |root|
      assert_empty Autodev::PlanningRevisionValidation.validate(root)
    end
  end

  def test_rejects_invalid_planning_state
    cases = [
      ["unresolved material questions", "project overview has unresolved material questions", lambda do |root|
        overview = root.join("docs/project-overview.md")
        overview.write(overview.read.sub("None.", "- Choose the delivery date."))
        refresh_hash(root, "docs/project-overview.md")
      end],
      ["duplicate task id", "duplicate task id", lambda do |root|
        mutate_tasks(root) { |tasks| tasks << Marshal.load(Marshal.dump(tasks.first)) }
      end],
      ["unknown dependency", "unknown dependencies", lambda do |root|
        mutate_tasks(root) { |tasks| tasks.first["depends_on"] = ["missing-task"] }
      end],
      ["self dependency", "task dependency cycle", lambda do |root|
        mutate_tasks(root) { |tasks| tasks.first["depends_on"] = [tasks.first["id"]] }
      end],
      ["dependency cycle", "task dependency cycle", lambda do |root|
        mutate_tasks(root) do |tasks|
          second = Marshal.load(Marshal.dump(tasks.first))
          second["id"] = "second-task"
          second["depends_on"] = [tasks.first["id"]]
          tasks.first["depends_on"] = [second["id"]]
          tasks << second
        end
      end],
      ["missing verification", "verification must be a non-empty string list", lambda do |root|
        mutate_tasks(root) { |tasks| tasks.first["verify"] = [] }
      end]
    ]

    cases.each do |name, expected, mutation|
      with_project do |root|
        mutation.call(root)
        errors = Autodev::PlanningRevisionValidation.validate(root)
        assert errors.any? { |error| error.include?(expected) }, "#{name}: #{errors.inspect}"
      end
    end
  end

  def test_rejects_invalid_approval_state
    cases = [
      ["missing approval", "approval record file is missing", lambda do |root|
        root.join(".autodev/approval.yaml").delete
      end],
      ["pending approval", "approval record status must be approved", lambda do |root|
        approval = read_yaml(root.join(".autodev/approval.yaml"))
        approval["status"] = "pending"
        write_yaml(root.join(".autodev/approval.yaml"), approval)
      end],
      ["changed overview", "approved planning file changed: docs/project-overview.md", lambda do |root|
        overview = root.join("docs/project-overview.md")
        overview.write("#{overview.read}\nChanged after approval.\n")
      end],
      ["changed tasks", "approved planning file changed: tasks.yaml", lambda do |root|
        root.join("tasks.yaml").open("a") { |file| file.puts "# changed after approval" }
      end]
    ]

    cases.each do |name, expected, mutation|
      with_project do |root|
        mutation.call(root)
        errors = Autodev::PlanningRevisionValidation.validate(root)
        assert errors.any? { |error| error.include?(expected) }, "#{name}: #{errors.inspect}"
      end
    end
  end

  def test_rejects_paths_outside_the_project
    with_project do |root|
      config_path = root.join(".autodev/config.yaml")
      config = read_yaml(config_path)
      config["project_overview"] = "../project-overview.md"
      write_yaml(config_path, config)

      errors = Autodev::PlanningRevisionValidation.validate(root)
      assert errors.any? { |error| error.include?("escapes the project root") }, errors.inspect
    end
  end

  private

  def with_project
    Dir.mktmpdir("autodev-project") do |directory|
      root = Pathname.new(directory)
      template = Pathname.new(__dir__).join("../templates/project").expand_path
      FileUtils.cp_r("#{template}/.", root)

      overview_path = root.join("docs/project-overview.md")
      overview = overview_path.read
        .sub(/- Replace this line.*$/, "None.")
      overview_path.write(overview)

      tasks = read_yaml(root.join("tasks.yaml"))
      tasks["project"] = "fixture"
      write_yaml(root.join("tasks.yaml"), tasks)

      write_yaml(root.join(".autodev/approval.yaml"), {
        "project" => "fixture",
        "status" => "approved",
        "approved_by" => "user",
        "approved_at" => "2026-08-09",
        "files" => {
          "docs/project-overview.md" => Digest::SHA256.file(root.join("docs/project-overview.md")).hexdigest,
          "tasks.yaml" => Digest::SHA256.file(root.join("tasks.yaml")).hexdigest
        }
      })

      yield root
    end
  end

  def mutate_tasks(root)
    path = root.join("tasks.yaml")
    document = read_yaml(path)
    yield document.fetch("tasks")
    write_yaml(path, document)
    refresh_hash(root, "tasks.yaml")
  end

  def refresh_hash(root, relative)
    approval_path = root.join(".autodev/approval.yaml")
    approval = read_yaml(approval_path)
    approval.fetch("files")[relative] = Digest::SHA256.file(root.join(relative)).hexdigest
    write_yaml(approval_path, approval)
  end

  def read_yaml(path)
    YAML.safe_load(path.read, permitted_classes: [Date], aliases: false)
  end

  def write_yaml(path, value)
    path.write(YAML.dump(value))
  end
end
