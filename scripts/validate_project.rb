# frozen_string_literal: true

require "digest"
require "pathname"
require "tsort"
require "yaml"

module Autodev
  module ProjectValidation
    CONFIG_PATH = ".autodev/config.yaml"
    APPROVAL_PATH = ".autodev/approval.yaml"

    class InvalidProject < StandardError; end

    module_function

    def validate(project_root)
      root = Pathname.new(project_root).expand_path
      invalid!("project root does not exist: #{root}") unless root.directory?
      root = root.realpath

      config = yaml!(root.join(CONFIG_PATH), "config")
      overview_relative = config["project_overview"]
      tasks_relative = config["task_graph"]
      overview_path = project_path!(root, overview_relative, "project_overview")
      tasks_path = project_path!(root, tasks_relative, "task_graph")

      validate_overview!(overview_path)
      validate_tasks!(root, tasks_path)
      validate_approval!(root.join(APPROVAL_PATH), {
        overview_relative => overview_path,
        tasks_relative => tasks_path
      })
      []
    rescue InvalidProject => error
      [error.message]
    end

    def yaml!(path, label)
      invalid!("#{label} file is missing: #{path}") unless path.file?
      parse_yaml!(path.read, label, path.to_s)
    rescue SystemCallError => error
      invalid!("#{label} cannot be read: #{error.message}")
    end

    def parse_yaml!(text, label, filename)
      value = YAML.safe_load(
        text,
        permitted_classes: [],
        permitted_symbols: [],
        aliases: false,
        filename: filename
      )
      invalid!("#{label} must contain a YAML mapping") unless value.is_a?(Hash)
      value
    rescue Psych::Exception => error
      invalid!("#{label} is invalid YAML: #{error.message.lines.first.strip}")
    end

    def project_path!(root, relative, label)
      unless relative.is_a?(String) && !relative.strip.empty?
        invalid!("config #{label} must be a non-empty project-relative path")
      end

      candidate = root.join(relative).cleanpath
      invalid!("config #{label} escapes the project root: #{relative}") unless inside?(root, candidate)
      if candidate.exist? && !inside?(root, candidate.realpath)
        invalid!("config #{label} resolves outside the project root: #{relative}")
      end
      candidate
    rescue SystemCallError => error
      invalid!("config #{label} cannot be resolved: #{error.message}")
    end

    def inside?(root, path)
      path == root || path.to_s.start_with?("#{root}#{File::SEPARATOR}")
    end

    def validate_overview!(path)
      invalid!("project overview is missing: #{path}") unless path.file?
      text = path.read
      match = text.match(/\A---[ \t]*\r?\n(?<yaml>.*?)\r?\n---[ \t]*\r?\n/m)
      invalid!("project overview is missing YAML frontmatter") unless match

      metadata = parse_yaml!(match[:yaml], "project overview frontmatter", path.to_s)
      invalid!("project overview status must be approved") unless metadata["status"] == "approved"

      section = text.match(/^##[ \t]+Open questions[ \t]*\r?$\n(?<body>.*?)(?=^##[ \t]+|\z)/im)
      invalid!("project overview is missing the Open questions section") unless section
      body = section[:body].gsub(/<!--.*?-->/m, "").strip
      invalid!("project overview has unresolved material questions") unless body.match?(/\ANone(?:\.|\s|\z)/i)
    rescue SystemCallError => error
      invalid!("project overview cannot be read: #{error.message}")
    end

    def validate_tasks!(root, path)
      document = yaml!(path, "task graph")
      invalid!("task graph status must be approved") unless document["status"] == "approved"
      tasks = document["tasks"]
      invalid!("task graph must contain at least one task") unless tasks.is_a?(Array) && !tasks.empty?

      ids = []
      dependencies = {}
      tasks.each_with_index do |task, index|
        invalid!("task at index #{index} must be a YAML mapping") unless task.is_a?(Hash)
        id = task["id"]
        invalid!("task at index #{index} has an empty id") unless present?(id)
        invalid!("duplicate task id: #{id}") if ids.include?(id)
        ids << id

        invalid!("task #{id} has an empty title") unless present?(task["title"])
        dependencies[id] = string_list!(task["depends_on"], "task #{id} depends_on")
        invalid!("task dependency cycle") if dependencies[id].include?(id)
        string_list!(task["verify"], "task #{id} verification", allow_empty: false)
        references = string_list!(task["references"], "task #{id} references", allow_empty: false)
        references.each do |reference|
          target = project_path!(root, reference.split("#", 2).first, "task #{id} reference")
          invalid!("task #{id} reference is missing: #{reference}") unless target.file?
        end
      end

      dependencies.each do |id, deps|
        unknown = deps - ids
        invalid!("task #{id} has unknown dependencies: #{unknown.join(', ')}") unless unknown.empty?
      end

      each_node = ->(&block) { ids.each(&block) }
      each_child = ->(id, &block) { dependencies.fetch(id).each(&block) }
      TSort.tsort(each_node, each_child)
    rescue TSort::Cyclic
      invalid!("task dependency cycle")
    end

    def string_list!(value, label, allow_empty: true)
      valid = value.is_a?(Array) && value.all? { |item| present?(item) }
      valid &&= !value.empty? unless allow_empty
      invalid!("#{label} must be a non-empty string list") unless valid
      value
    end

    def validate_approval!(path, planned_files)
      approval = yaml!(path, "approval record")
      invalid!("approval record status must be approved") unless approval["status"] == "approved"
      invalid!("approval record is missing approved_by") unless present?(approval["approved_by"])
      invalid!("approval record is missing approved_at") unless present?(approval["approved_at"])

      files = approval["files"]
      unless files.is_a?(Hash) && files.keys.sort == planned_files.keys.sort
        invalid!("approval record must cover exactly the configured planning files")
      end

      planned_files.each do |relative, file_path|
        expected = files[relative]
        unless expected.is_a?(String) && expected.match?(/\A[0-9a-f]{64}\z/)
          invalid!("approval record has an invalid SHA-256 digest for #{relative}")
        end
        invalid!("approved planning file is missing: #{relative}") unless file_path.file?
        actual = Digest::SHA256.file(file_path).hexdigest
        invalid!("approved planning file changed: #{relative}") unless actual == expected
      end
    end

    def present?(value)
      value.is_a?(String) && !value.strip.empty?
    end

    def invalid!(message)
      raise InvalidProject, message
    end
  end
end

if $PROGRAM_NAME == __FILE__
  errors = Autodev::ProjectValidation.validate(ARGV.fetch(0, "."))
  if errors.empty?
    puts "Project contract valid."
    exit 0
  end

  warn "ERROR: #{errors.first}"
  exit 1
end
