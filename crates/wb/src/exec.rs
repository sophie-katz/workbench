mod errors;
mod handlers;
mod shell;
mod task_path;

use std::sync::{Arc, RwLock};

use futures::future::join_all;
use tokio::runtime::Runtime;

use crate::{
    config::{Config, Task},
    console::{Logger, ProgressContext},
};

use async_recursion::async_recursion;

pub use task_path::{count_dependencies_of_path, TaskPath, TaskPathParsingError};

pub use errors::ExecError;

pub use handlers::ExecOutput;

pub fn exec(config: Config, logger: Logger, target_task_path: TaskPath) -> Result<bool, ExecError> {
    let progress_context = Arc::new(RwLock::new(ProgressContext::new(
        &config,
        target_task_path.clone(),
    )?));

    let result = Runtime::new()?.block_on(exec_task(
        config,
        logger,
        progress_context.clone(),
        target_task_path,
    ));

    progress_context
        .as_ref()
        .write()
        .expect("writer handle panicked")
        .clear()?;

    result
}

#[async_recursion]
async fn exec_task(
    config: Config,
    logger: Logger,
    progress_context: Arc<RwLock<ProgressContext>>,
    task_path: TaskPath,
) -> Result<bool, ExecError> {
    let task = task_path::get_task_at_path(&config, &task_path)
        .ok_or_else(|| ExecError::TaskNotFound(task_path.clone()))
        .unwrap();

    if !exec_all_dependencies(
        config.clone(),
        logger.clone(),
        progress_context.clone(),
        task.clone(),
    )
    .await?
    {
        return Ok(false);
    }

    progress_context
        .as_ref()
        .write()
        .expect("writer handle panicked")
        .begin_task(&task_path, &task);

    let output = handlers::handle_execution(&logger, &task_path, task).unwrap();

    progress_context
        .as_ref()
        .write()
        .expect("writer handle panicked")
        .clear()
        .unwrap();

    logger.emit_exec_output(&task_path, &task, &output);

    progress_context
        .as_ref()
        .write()
        .expect("writer handle panicked")
        .complete_task();

    Ok(output.exit_code == 0)
}

async fn exec_all_dependencies(
    config: Config,
    logger: Logger,
    progress_context: Arc<RwLock<ProgressContext>>,
    task: Task,
) -> Result<bool, ExecError> {
    if let Some(dependencies) = &task.dependencies {
        let mut futures = Vec::new();

        for dependency in dependencies {
            futures.push(tokio::spawn(exec_dependency(
                config.clone(),
                logger.clone(),
                progress_context.clone(),
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
            .filter(|item| item.is_err())
            .map(|item| item.unwrap_err())
            .collect::<Vec<ExecError>>();

        if !errors.is_empty() {
            return Err(ExecError::Aggregate(errors));
        }

        if any_failures {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn exec_dependency(
    config: Config,
    logger: Logger,
    progress_context: Arc<RwLock<ProgressContext>>,
    dependency: String,
) -> Result<bool, ExecError> {
    exec_task(
        config,
        logger,
        progress_context,
        TaskPath::parse(dependency.as_str())?,
    )
    .await
}
