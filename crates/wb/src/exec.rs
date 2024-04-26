mod task_path;

use std::{io, process::Command};

use futures::future::join_all;
use thiserror::Error;
use tokio::runtime::Runtime;

use crate::config::{Config, Run, Shell, Task};

pub use task_path::TaskPath;

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Task not found: {0}")]
    TaskNotFound(TaskPath),
    #[error("Invalid task path: {0:?}")]
    InvalidTaskPath(String),
    #[error("Multiple errors occurred")]
    Aggregate(Vec<ExecError>),
    #[error("Shell required in task {0} for command: {1:?}")]
    ShellRequired(TaskPath, String),
}

pub fn exec(config: &Config, task_path: TaskPath) -> Result<(), ExecError> {
    Runtime::new()?.block_on(async { exec_task(config, &Context::new(task_path)).await })
}

struct Context {
    parents: Vec<TaskPath>,
    task_path: TaskPath,
}

impl Context {
    fn new(task_path: TaskPath) -> Self {
        Self {
            parents: Vec::new(),
            task_path,
        }
    }

    fn next(&self, task_path: TaskPath) -> Self {
        let mut parents = self.parents.clone();
        parents.push(task_path.clone());

        Self { parents, task_path }
    }
}

async fn exec_task(config: &Config, task_context: &Context) -> Result<(), ExecError> {
    let task = task_path::get_task_at_path(config, &task_context.task_path)
        .ok_or(ExecError::TaskNotFound(task_context.task_path.clone()))?;

    exec_all_dependencies(config, task_context, task).await?;

    let shell = resolve_shell(task);

    let mut child = match task.run {
        Run::String(ref command) => {
            let shell = shell.ok_or_else(|| {
                ExecError::ShellRequired(task_context.task_path.clone(), command.clone())
            })?;

            Command::new(shell).arg("-c").arg(command).spawn()?
        }
        Run::Args(ref args) => Command::new(args[0].as_str()).args(&args[1..]).spawn()?,
    };

    child.wait()?;

    Ok(())
}

const DEFAULT_SHELL: &str = "sh";

fn resolve_shell(task: &Task) -> Option<String> {
    match task.shell {
        None => None,
        Some(Shell::Bool(true)) => Some(DEFAULT_SHELL.to_owned()),
        Some(Shell::Bool(false)) => None,
        Some(Shell::String(ref shell)) => Some(shell.clone()),
    }
}

async fn exec_all_dependencies(
    config: &Config,
    task_context: &Context,
    task: &Task,
) -> Result<(), ExecError> {
    if let Some(ref dependencies) = task.dependencies {
        let errors = join_all(
            dependencies
                .iter()
                .map(|dependency| exec_dependency(config, task_context, dependency.as_str()))
                .into_iter(),
        )
        .await
        .into_iter()
        .filter(|item| item.is_err())
        .map(|item| item.unwrap_err())
        .collect::<Vec<_>>();

        if !errors.is_empty() {
            return Err(ExecError::Aggregate(errors));
        }
    }

    Ok(())
}

async fn exec_dependency(
    config: &Config,
    task_context: &Context,
    dependency_name: &str,
) -> Result<(), ExecError> {
    let dependency_path = TaskPath::parse(dependency_name)
        .ok_or_else(|| ExecError::InvalidTaskPath(dependency_name.to_owned()))?;

    Ok(exec_task(config, &task_context.next(dependency_path)).await?)
}
