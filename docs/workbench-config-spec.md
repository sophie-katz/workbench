# `workbench.yml` specification

```yaml
# Optional - a map of tasks to be run.
tasks:
  <task name>:
    # Required - the command to run. This can be a string like this, which will imply that the
    # command should be run in the shell:
    run: <command>
    # Of it can be an array of strings, which will imply that the command should be run as a binary:
    # run: ['<binary>', '<arg 1>', '<arg 2>', ...]

    # Optional - specifies whether or not to run the command in a shell. Defaults to true.
    shell: true
    # You can also use a custom shell like so:
    # shell: /bin/zsh

    # Optional - a list of dependencies that must complete successfully before this task runs.
    dependencies:
      # These are task paths that are structured the same as when they are specified on the command
      # line.
      - <task path>
      - <task path>
      ...

    # Optional - a list of files to use as inputs to the task. If any of these files are modified,
    # the task will re-run. Otherwise, it will not.
    #
    # You can use * and ** to match multiple files.
    inputs:
      - input_file
      - input_dir/**/*
      # You can exclude files using the ! prefix
      - "!excluded_input_file"
    # Alternatively, you can use the object syntax to specify a glob pattern and a list of excluded
    # files:
    # inputs:
    #   include:
    #     - input_file
    #     - input_dir/**/*
    #   exclude:
    #     - excluded_input_file

    # Optional - a list of files to use as outputs from the task.
    #
    # These use the same schema as 'inputs'.
    outputs:
      - output_file
      - output_dir/**/*

    # Optional - a custom usage string for the task. This will be displayed when the task is run
    # with the `.help` property.
    usage: "[OPTIONS]"

    # Optional - a description of the task. This will be displayed when the task is run with the
    # `.help` property.
    description: "This is a description of the task."

    # Optional - a list of examples of how to run the task. These will be displayed when the task is
    # run with the `.help` property.
    examples:
      # Each example has a required 'run' property that is the arguments passed into the task. This
      # can be left empty to denote the case when no arguments are passed in.
      - run:

      # You can also specify a description which will show up as a comment before the example.
      - description: "This is an example of the task."
        run: --flag

# Optional - a map of namespaces that can be used to group tasks.
namespaces:
  <namespace name>:
    # Tasks in the namespace follow the top-level tasks schema as well
    tasks: ...
```
