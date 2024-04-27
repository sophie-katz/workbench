use thiserror::Error;

use super::TaskPath;
use std::io;

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
    #[error("Task failed: {0} (exit code: {1})")]
    TaskFailed(TaskPath, i32),
}
