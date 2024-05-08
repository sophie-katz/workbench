// Copyright 2024 Sophie Katz
//
// This file is part of Workbench.
//
// Workbench is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Workbench is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Workbench. If not,
// see <https://www.gnu.org/licenses/>.

mod files;
mod handlers;
mod shell;
mod task_path;

use std::fs;

use futures::future::join_all;
use tokio::runtime::Runtime;

use crate::{
    config::{Config, Files, Task},
    console::{Log, Progress},
    error::WorkbenchError,
};

use async_recursion::async_recursion;

pub use task_path::{count_dependencies_of_path, get_task_at_path, TaskPath};

pub use handlers::Output;

pub use self::files::resolve_paths;

pub fn exec<ConsoleContext: 'static + Log + Progress + Clone + Send + Sync>(
    config: Config,
    console_context: &ConsoleContext,
    target_task_path: TaskPath,
) -> Result<bool, WorkbenchError> {
    let result = Runtime::new()?.block_on(exec_task(config, console_context, target_task_path));

    console_context.clear()?;

    result
}

#[async_recursion]
async fn exec_task<ConsoleContext>(
    config: Config,
    console_context: &ConsoleContext,
    task_path: TaskPath,
) -> Result<bool, WorkbenchError>
where
    ConsoleContext: 'static + Log + Progress + Clone + Send + Sync,
{
    let task = task_path::get_task_at_path(&config, &task_path)
        .ok_or_else(|| WorkbenchError::TaskNotFound(task_path.clone()))?;

    if !exec_all_dependencies(config.clone(), console_context, task.clone()).await? {
        return Ok(false);
    }

    console_context.begin_task(&task_path, task);

    let reason = should_run_task(task)?;

    if let Some(reason) = reason {
        console_context.log_exec_skipped(&task_path, reason);

        console_context.complete_task();

        Ok(true)
    } else {
        let output = handlers::handle_execution(&task_path, task).unwrap();

        console_context.log_exec_output(&task_path, task, &output);

        console_context.complete_task();

        Ok(output.exit_code == 0)
    }
}

async fn exec_all_dependencies<ConsoleContext: 'static + Log + Progress + Clone + Send + Sync>(
    config: Config,
    console_context: &ConsoleContext,
    task: Task,
) -> Result<bool, WorkbenchError> {
    if let Some(dependencies) = &task.dependencies {
        let mut futures = Vec::new();

        for dependency in dependencies {
            futures.push(tokio::spawn(exec_dependency(
                config.clone(),
                console_context.clone(),
                dependency.clone(),
            )));
        }

        let results = join_all(futures.into_iter())
            .await
            .into_iter()
            .map(|item| item?)
            .collect::<Vec<_>>();

        let any_failures = results
            .iter()
            .any(|result| !result.as_ref().unwrap_or(&true));

        let errors = results
            .into_iter()
            .filter(Result::is_err)
            .map(|item| item.unwrap_err())
            .collect::<Vec<WorkbenchError>>();

        if !errors.is_empty() {
            return Err(WorkbenchError::Aggregate(errors));
        }

        if any_failures {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn exec_dependency<ConsoleContext: 'static + Log + Progress + Clone + Send + Sync>(
    config: Config,
    console_context: ConsoleContext,
    dependency: String,
) -> Result<bool, WorkbenchError> {
    exec_task(
        config,
        &console_context,
        TaskPath::parse(dependency.as_str())?,
    )
    .await
}

fn should_run_task(task: &Task) -> Result<Option<String>, WorkbenchError> {
    if let Some(inputs) = &task.inputs {
        if let Some(outputs) = &task.outputs {
            if !should_run_tasks_files(inputs, outputs)? {
                return Ok(Some("cached".to_owned()));
            }
        }
    }

    Ok(None)
}

fn should_run_tasks_files(inputs: &Files, outputs: &Files) -> Result<bool, WorkbenchError> {
    // Resolve paths
    let input_paths = resolve_paths(inputs)?;
    let output_paths = resolve_paths(outputs)?;

    // Find the timestamp of the most recently modified input file
    let mut input_most_recently_modified = None;

    for path in input_paths {
        let modified = fs::metadata(path)?.modified()?;

        input_most_recently_modified = match input_most_recently_modified {
            Some(value) => Some(if value > modified { value } else { modified }),
            None => Some(modified),
        }
    }

    if let Some(input_most_recently_modified) = input_most_recently_modified {
        if output_paths.is_empty() {
            return Ok(true);
        }

        // Check all output files...
        for path in output_paths {
            let modified = fs::metadata(path)?.modified()?;

            // If any inputs were modified more recently than an output, then the task should run
            if modified < input_most_recently_modified {
                return Ok(true);
            }
        }

        Ok(false)
    } else {
        // If there are no input paths, just return true
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fmt::Display,
        io,
        sync::{Arc, RwLock},
    };

    use crate::{
        config::Run,
        console::{Context, Level, Logger},
    };

    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TaskOrderEntry {
        Begin(String),
        Complete(String),
    }

    #[derive(Clone, Default)]
    struct MockConsoleContext {
        task_order: Arc<RwLock<Vec<TaskOrderEntry>>>,
    }

    impl MockConsoleContext {
        pub fn take_task_order(self) -> Vec<TaskOrderEntry> {
            Arc::try_unwrap(self.task_order)
                .unwrap()
                .into_inner()
                .unwrap()
        }
    }

    impl Log for MockConsoleContext {
        fn log_message<Message: Display>(&self, _level: Level, _message: Message) {}

        fn log_exec_output(&self, task_path: &TaskPath, _task: &Task, _output: &Output) {
            self.task_order
                .write()
                .unwrap()
                .push(TaskOrderEntry::Complete(task_path.to_string()));
        }

        fn log_exec_skipped(&self, task_path: &TaskPath, _reason: impl Display) {
            self.task_order
                .write()
                .unwrap()
                .push(TaskOrderEntry::Complete(task_path.to_string()));
        }
    }

    impl Progress for MockConsoleContext {
        fn begin_task(&self, task_path: &TaskPath, _task: &Task) {
            self.task_order
                .write()
                .unwrap()
                .push(TaskOrderEntry::Begin(task_path.to_string()));
        }

        fn complete_task(&self) {}

        fn clear(&self) -> Result<(), io::Error> {
            Ok(())
        }
    }

    #[test]
    fn test_exec_target_task_path_not_found() {
        let config = Config {
            tasks: None,
            namespaces: None,
        };

        let console_context = Logger::new(Level::Status);

        let console_context = Context::new(console_context, Some(1));

        let result = exec(config, &console_context, TaskPath::parse("a").unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_exec_one_task() {
        let config = Config {
            tasks: Some(HashMap::from([(
                "a".to_owned(),
                Task {
                    run: Run::Args(vec!["true".to_owned()]),
                    shell: None,
                    dependencies: None,
                    inputs: None,
                    outputs: None,
                    description: None,
                    examples: None,
                    usage: None,
                },
            )])),
            namespaces: None,
        };

        let console_context = MockConsoleContext::default();

        let result = exec(config, &console_context, TaskPath::parse("a").unwrap());

        assert!(result.unwrap());

        assert!(
            console_context.take_task_order()
                == vec![
                    TaskOrderEntry::Begin("a".to_owned()),
                    TaskOrderEntry::Complete("a".to_owned())
                ]
        );
    }

    #[test]
    fn test_exec_task_with_one_dependency() {
        let config = Config {
            tasks: Some(HashMap::from([
                (
                    "a".to_owned(),
                    Task {
                        run: Run::Args(vec!["true".to_owned()]),
                        shell: None,
                        dependencies: None,
                        inputs: None,
                        outputs: None,
                        description: None,
                        examples: None,
                        usage: None,
                    },
                ),
                (
                    "b".to_owned(),
                    Task {
                        run: Run::Args(vec!["true".to_owned()]),
                        shell: None,
                        dependencies: Some(vec!["a".to_owned()]),
                        inputs: None,
                        outputs: None,
                        description: None,
                        examples: None,
                        usage: None,
                    },
                ),
            ])),
            namespaces: None,
        };

        let console_context = MockConsoleContext::default();

        let result = exec(config, &console_context, TaskPath::parse("b").unwrap());

        assert!(result.unwrap());

        assert_eq!(
            console_context.take_task_order(),
            vec![
                TaskOrderEntry::Begin("a".to_owned()),
                TaskOrderEntry::Complete("a".to_owned()),
                TaskOrderEntry::Begin("b".to_owned()),
                TaskOrderEntry::Complete("b".to_owned())
            ]
        );
    }

    #[test]
    fn test_exec_task_with_two_dependencies() {
        let config = Config {
            tasks: Some(HashMap::from([
                (
                    "a".to_owned(),
                    Task {
                        run: Run::Args(vec!["sleep".to_owned(), "0.01".to_owned()]),
                        shell: None,
                        dependencies: None,
                        inputs: None,
                        outputs: None,
                        description: None,
                        examples: None,
                        usage: None,
                    },
                ),
                (
                    "b".to_owned(),
                    Task {
                        run: Run::Args(vec!["sleep".to_owned(), "0.02".to_owned()]),
                        shell: None,
                        dependencies: None,
                        inputs: None,
                        outputs: None,
                        description: None,
                        examples: None,
                        usage: None,
                    },
                ),
                (
                    "c".to_owned(),
                    Task {
                        run: Run::Args(vec!["true".to_owned()]),
                        shell: None,
                        dependencies: Some(vec!["a".to_owned(), "b".to_owned()]),
                        inputs: None,
                        outputs: None,
                        description: None,
                        examples: None,
                        usage: None,
                    },
                ),
            ])),
            namespaces: None,
        };

        let console_context = MockConsoleContext::default();

        let result = exec(config, &console_context, TaskPath::parse("c").unwrap());

        assert!(result.unwrap());

        let task_order = console_context.take_task_order();

        dbg!(&task_order);

        assert!(
            task_order
                == vec![
                    TaskOrderEntry::Begin("a".to_owned()),
                    TaskOrderEntry::Begin("b".to_owned()),
                    TaskOrderEntry::Complete("a".to_owned()),
                    TaskOrderEntry::Complete("b".to_owned()),
                    TaskOrderEntry::Begin("c".to_owned()),
                    TaskOrderEntry::Complete("c".to_owned())
                ]
                || task_order
                    == vec![
                        TaskOrderEntry::Begin("b".to_owned()),
                        TaskOrderEntry::Begin("a".to_owned()),
                        TaskOrderEntry::Complete("a".to_owned()),
                        TaskOrderEntry::Complete("b".to_owned()),
                        TaskOrderEntry::Begin("c".to_owned()),
                        TaskOrderEntry::Complete("c".to_owned())
                    ]
        );
    }

    #[test]
    fn test_exec_task_chain() {
        let config = Config {
            tasks: Some(HashMap::from([
                (
                    "a".to_owned(),
                    Task {
                        run: Run::Args(vec!["true".to_owned()]),
                        shell: None,
                        dependencies: None,
                        inputs: None,
                        outputs: None,
                        description: None,
                        examples: None,
                        usage: None,
                    },
                ),
                (
                    "b".to_owned(),
                    Task {
                        run: Run::Args(vec!["true".to_owned()]),
                        shell: None,
                        dependencies: Some(vec!["a".to_owned()]),
                        inputs: None,
                        outputs: None,
                        description: None,
                        examples: None,
                        usage: None,
                    },
                ),
                (
                    "c".to_owned(),
                    Task {
                        run: Run::Args(vec!["true".to_owned()]),
                        shell: None,
                        dependencies: Some(vec!["b".to_owned()]),
                        inputs: None,
                        outputs: None,
                        description: None,
                        examples: None,
                        usage: None,
                    },
                ),
            ])),
            namespaces: None,
        };

        let console_context = MockConsoleContext::default();

        let result = exec(config, &console_context, TaskPath::parse("c").unwrap());

        assert!(result.unwrap());

        let task_order = console_context.take_task_order();

        assert_eq!(
            task_order,
            vec![
                TaskOrderEntry::Begin("a".to_owned()),
                TaskOrderEntry::Complete("a".to_owned()),
                TaskOrderEntry::Begin("b".to_owned()),
                TaskOrderEntry::Complete("b".to_owned()),
                TaskOrderEntry::Begin("c".to_owned()),
                TaskOrderEntry::Complete("c".to_owned())
            ]
        );
    }
}
