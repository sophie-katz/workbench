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

use thiserror::Error;

use super::TaskPath;
use std::io;

#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug)]
pub enum WorkbenchError {
    #[error("{0}")]
    FromStdIoError(#[from] io::Error),
    #[error("{0}")]
    FromSerdeYamlError(#[from] serde_yaml::Error),
    #[error("{0}")]
    FromTokioJoinError(#[from] tokio::task::JoinError),
    #[error("{0}")]
    FromGlobPatternError(#[from] glob::PatternError),
    #[error("{0}")]
    FromGlobError(#[from] glob::GlobError),
    #[error("invalid task path {0:?}")]
    InvalidTaskPath(String),
    #[error("task {0} not found")]
    TaskNotFound(TaskPath),
    #[error("multiple errors occurred")]
    Aggregate(Vec<Self>),
    #[error("shell required in task {0} for command {1:?}")]
    ShellRequired(TaskPath, String),
}
