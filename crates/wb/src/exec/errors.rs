use thiserror::Error;

use super::{task_path::TaskPathParsingError, TaskPath};
use std::io;

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Task not found: {0}")]
    TaskNotFound(TaskPath),
    #[error("Error while parsing task path: {0:?}")]
    TaskPathParsing(#[from] TaskPathParsingError),
    #[error("Multiple errors occurred")]
    Aggregate(Vec<ExecError>),
    #[error("Shell required in task {0} for command: {1:?}")]
    ShellRequired(TaskPath, String),
}
