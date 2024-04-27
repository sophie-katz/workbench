mod errors;
mod shell;
mod task_context;
mod task_path;

use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Ref, RefCell},
    ops::Deref,
    process::Command,
    rc::Rc,
};

use futures::future::join_all;
use tokio::runtime::Runtime;

use crate::{
    config::{Config, Run, Task},
    console::{Level, Logger, ProgressContext},
};

pub use task_path::TaskPath;

use self::{errors::ExecError, task_context::TaskContext};

pub fn exec(
    config: Rc<RefCell<Config>>,
    logger: Rc<RefCell<Logger>>,
    task_path: TaskPath,
) -> Result<(), ExecError> {
    let progress_context = Rc::new(RefCell::new(ProgressContext::new(
        config.clone(),
        task_path.clone(),
    )));

    Runtime::new()?.block_on(async {
        exec_task(&TaskContext::new(
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

async fn exec_task(task_context: &TaskContext) -> Result<(), ExecError> {
    let config_ref = task_context.config.as_ref().borrow();

    let task = task_path::get_task_at_path(config_ref.deref(), &task_context.task_path)
        .ok_or_else(|| ExecError::TaskNotFound(task_context.task_path.clone()))?;

    exec_all_dependencies(task_context, task).await?;

    let shell = shell::resolve_shell(task);

    let mut child = match task.run {
        Run::String(ref command) => {
            let shell = shell.ok_or_else(|| {
                ExecError::ShellRequired(task_context.task_path.clone(), command.clone())
            })?;

            task_context
                .logger
                .as_ref()
                .borrow()
                .emit(Level::Status, format!("running command: {:?}", command));

            Command::new(shell).arg("-c").arg(command).spawn()?
        }
        Run::Args(ref args) => {
            task_context.logger.as_ref().borrow().emit(
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

async fn exec_all_dependencies(task_context: &TaskContext, task: &Task) -> Result<(), ExecError> {
    if let Some(ref dependencies) = task.dependencies {
        let mut futures = Vec::new();

        for dependency in dependencies {
            futures.push(async { exec_dependency(task_context, dependency.as_str()).await });
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

async fn exec_dependency(
    task_context: &TaskContext,
    dependency_name: &str,
) -> Result<(), ExecError> {
    let dependency_path = TaskPath::parse(dependency_name)
        .ok_or_else(|| ExecError::InvalidTaskPath(dependency_name.to_owned()))?;

    Ok(exec_task(&task_context.next(dependency_path)).await?)
}
