use colored::Colorize;
use std::{
    fmt::{self, Display},
    io::{self, Write},
};

use crate::{
    config::{Run, Task},
    exec::{ExecOutput, TaskPath},
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Level {
    Status,
    Warning,
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Status => "status".bold().to_string(),
                Self::Warning => "warning".bold().yellow().to_string(),
                Self::Error => "error".bold().red().to_string(),
            }
        )
    }
}

#[derive(Clone)]
pub struct Logger {
    pub min_level: Level,
}

impl Logger {
    pub fn emit(&self, level: Level, message: impl Display) {
        if level >= self.min_level {
            println!(
                "{} {}{}{} {}",
                "◆".green().dimmed(),
                "[".dimmed(),
                level,
                "]:".dimmed(),
                message
            );
        }
    }

    pub fn emit_exec_output(&self, task_path: &TaskPath, task: &Task, output: &ExecOutput) {
        if output.console_output.is_empty() {
            println!(
                "{} {}",
                "◆".green().dimmed(),
                self.format_task_completion_message(task_path, task, output.exit_code, false),
            );
        } else {
            println!(
                "{} {}{}",
                "◆".green().dimmed(),
                self.format_task_completion_message(task_path, task, output.exit_code, true),
                ":".dimmed().white(),
            );

            if let Err(e) = io::stdout().write_all(&output.console_output) {
                self.emit(Level::Error, e.to_string());
            }
        }
    }

    fn format_task_completion_message(
        &self,
        task_path: &TaskPath,
        task: &Task,
        exit_code: i32,
        has_output: bool,
    ) -> impl Display {
        if exit_code == 0 {
            format!(
                "{}{} {}",
                format!(
                    "'{}' {}",
                    task_path,
                    if has_output { "output" } else { "completed" }
                )
                .green(),
                ":".dimmed(),
                match &task.run {
                    Run::String(value) => value.clone(),
                    Run::Args(values) => values.join(" "),
                }
                .dimmed()
            )
        } else {
            format!(
                "{}{} {}",
                format!("'{task_path}' failed with exit code {exit_code}").red(),
                ":".dimmed(),
                match &task.run {
                    Run::String(value) => value.clone(),
                    Run::Args(values) => values.join(" "),
                }
                .dimmed()
            )
        }
    }
}
