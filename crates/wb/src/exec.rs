mod errors;
mod shell;
mod task_context;
mod task_path;

use std::{process::Command, rc::Rc};

use futures::future::join_all;
use tokio::runtime::Runtime;

use crate::{
    config::{Config, Run, Task},
    console::{Level, Logger, ProgressContext},
};

pub use task_path::TaskPath;

use self::{errors::ExecError, task_context::TaskContext};

pub fn exec(config: &Config, logger: &Logger, task_path: TaskPath) -> Result<(), ExecError> {
    let mut progress_context = Rc::new(ProgressContext::new(config, task_path.clone()));

    Runtime::new()?.block_on(async {
        exec_task(&mut TaskContext::new(
            config,
            logger,
            progress_context,
            task_path,
        ))
        .await
    })
}

pub fn count_dependencies(config: &Config, task_path: TaskPath) -> u32 {
    let task = task_path::get_task_at_path(config, &task_path).unwrap();

    let mut count = 0;

    if let Some(ref dependencies) = task.dependencies {
        count += dependencies.len() as u32;

        for dependency in dependencies {
            count += count_dependencies(config, TaskPath::parse(dependency.as_str()).unwrap());
        }
    }

    count
}

async fn exec_task<'config, 'logger>(
    task_context: &mut TaskContext<'config, 'logger>,
) -> Result<(), ExecError> {
    let task = task_path::get_task_at_path(task_context.config, &task_context.task_path)
        .ok_or(ExecError::TaskNotFound(task_context.task_path.clone()))?;

    exec_all_dependencies(task_context, task).await?;

    let shell = shell::resolve_shell(task);

    let mut child = match task.run {
        Run::String(ref command) => {
            let shell = shell.ok_or_else(|| {
                ExecError::ShellRequired(task_context.task_path.clone(), command.clone())
            })?;

            task_context
                .logger
                .emit(Level::Status, format!("running command: {:?}", command));

            Command::new(shell).arg("-c").arg(command).spawn()?
        }
        Run::Args(ref args) => {
            task_context.logger.emit(
                Level::Status,
                format!("running command: {:?}", args.join(" ")),
            );

            Command::new(args[0].as_str()).args(&args[1..]).spawn()?
        }
    };

    let status = child.wait()?;

    let code = status.code().unwrap();

    if code != 0 {
        return Err(ExecError::TaskFailed(task_context.task_path.clone(), code));
    }

    Ok(())
}

async fn exec_all_dependencies<'config, 'logger>(
    task_context: &mut TaskContext<'config, 'logger>,
    task: &Task,
) -> Result<(), ExecError> {
    if let Some(ref dependencies) = task.dependencies {
        let mut futures = Vec::new();

        for dependency in dependencies {
            futures.push(async {
                let mut task_context_clone = task_context.clone();
                exec_dependency(&mut task_context_clone, dependency.as_str()).await
            });
        }

        let errors = join_all(futures.into_iter())
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

async fn exec_dependency<'config, 'logger>(
    task_context: &mut TaskContext<'config, 'logger>,
    dependency_name: &str,
) -> Result<(), ExecError> {
    let dependency_path = TaskPath::parse(dependency_name)
        .ok_or_else(|| ExecError::InvalidTaskPath(dependency_name.to_owned()))?;

    Ok(exec_task(&mut task_context.next(dependency_path)).await?)
}
