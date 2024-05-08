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

use std::{
    fmt::{self, Display},
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{
    config::{Run, Task},
    exec::{Output, TaskPath},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

pub trait Log {
    fn log_message<Message: Display>(&self, level: Level, message: Message);

    fn log_exec_output(&self, task_path: &TaskPath, task: &Task, output: &Output);

    fn log_exec_skipped(&self, task_path: &TaskPath, reason: impl Display);
}

#[derive(Clone)]
pub struct Logger {
    min_level: Level,
}

impl Logger {
    pub const fn new(min_level: Level) -> Self {
        Self { min_level }
    }
}

impl Log for Logger {
    fn log_message<Message: Display>(&self, level: Level, message: Message) {
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

    fn log_exec_output(&self, task_path: &TaskPath, task: &Task, output: &Output) {
        if output.console_output.is_empty() {
            println!(
                "{} {}",
                "◆".green().dimmed(),
                format_task_completion_message(
                    task_path,
                    task,
                    output.exit_code,
                    output.duration,
                    false
                ),
            );
        } else {
            println!(
                "{} {}{}",
                "◆".green().dimmed(),
                format_task_completion_message(
                    task_path,
                    task,
                    output.exit_code,
                    output.duration,
                    true
                ),
                ":".dimmed().white(),
            );

            if let Err(e) = io::stdout().write_all(&output.console_output) {
                self.log_message(Level::Error, e.to_string());
            }
        }
    }

    fn log_exec_skipped(&self, task_path: &TaskPath, reason: impl Display) {
        println!(
            "{} {}",
            "◆".green().dimmed(),
            format!("'{}' skipped ({})", task_path, reason).yellow()
        );
    }
}

pub trait Progress {
    fn begin_task(&self, task_path: &TaskPath, task: &Task);

    fn complete_task(&self);

    fn clear(&self) -> Result<(), io::Error>;
}

#[derive(Clone)]
pub struct Context {
    logger: Logger,
    multi_progress: Option<Arc<Mutex<MultiProgress>>>,
    progress_bar_tasks: Option<Arc<Mutex<ProgressBar>>>,
}

impl Context {
    pub fn new(logger: Logger, progress_bar_ticks: Option<u64>) -> Self {
        if let Some(progress_bar_ticks) = progress_bar_ticks {
            let multi_progress = MultiProgress::new();

            let progress_bar_tasks = multi_progress.add(ProgressBar::new(progress_bar_ticks));

            progress_bar_tasks.set_style(
                ProgressStyle::with_template("【{pos}/{len}】{bar:30.green/dim.white} {msg}")
                    .expect("error with template"),
            );

            let multi_progress = Arc::new(Mutex::new(multi_progress));

            let progress_bar_tasks = Arc::new(Mutex::new(progress_bar_tasks));

            Self {
                logger,
                multi_progress: Some(multi_progress),
                progress_bar_tasks: Some(progress_bar_tasks),
            }
        } else {
            Self {
                logger,
                multi_progress: None,
                progress_bar_tasks: None,
            }
        }
    }
}

impl Log for Context {
    fn log_message<Message: Display>(&self, level: Level, message: Message) {
        if level >= self.logger.min_level {
            if let Some(multi_progress) = &self.multi_progress {
                multi_progress
                    .as_ref()
                    .lock()
                    .expect("progress bar mutex is poisoned")
                    .clear()
                    .unwrap();
            }

            self.logger.log_message(level, message);

            if let Some(progress_bar_tasks) = &self.progress_bar_tasks {
                progress_bar_tasks
                    .as_ref()
                    .lock()
                    .expect("progress bar mutex is poisoned")
                    .tick();
            }
        }
    }

    fn log_exec_output(&self, task_path: &TaskPath, task: &Task, output: &Output) {
        if let Some(multi_progress) = &self.multi_progress {
            multi_progress
                .as_ref()
                .lock()
                .expect("progress bar mutex is poisoned")
                .clear()
                .unwrap();
        }

        self.logger.log_exec_output(task_path, task, output);

        if let Some(progress_bar_tasks) = &self.progress_bar_tasks {
            progress_bar_tasks
                .as_ref()
                .lock()
                .expect("progress bar mutex is poisoned")
                .tick();
        }
    }

    fn log_exec_skipped(&self, task_path: &TaskPath, reason: impl Display) {
        if let Some(multi_progress) = &self.multi_progress {
            multi_progress
                .as_ref()
                .lock()
                .expect("progress bar mutex is poisoned")
                .clear()
                .unwrap();
        }

        self.logger.log_exec_skipped(task_path, reason);

        if let Some(progress_bar_tasks) = &self.progress_bar_tasks {
            progress_bar_tasks
                .as_ref()
                .lock()
                .expect("progress bar mutex is poisoned")
                .tick();
        }
    }
}

impl Progress for Context {
    fn begin_task(&self, task_path: &TaskPath, task: &Task) {
        if let Some(progress_bar_tasks) = &self.progress_bar_tasks {
            progress_bar_tasks
                .as_ref()
                .lock()
                .expect("progress bar mutex is poisoned")
                .set_message(format!(
                    "{} {}{} {}",
                    "◆".green().dimmed(),
                    format!("'{task_path}'").green(),
                    ":".dimmed(),
                    match &task.run {
                        Run::String(value) => value.clone(),
                        Run::Args(values) => values.join(" "),
                    }
                    .dimmed()
                ));
        }
    }

    fn complete_task(&self) {
        if let Some(progress_bar_tasks) = &self.progress_bar_tasks {
            progress_bar_tasks
                .as_ref()
                .lock()
                .expect("progress bar mutex is poisoned")
                .inc(1);
        }
    }

    fn clear(&self) -> Result<(), io::Error> {
        if let Some(multi_progress) = &self.multi_progress {
            multi_progress
                .as_ref()
                .lock()
                .expect("progress bar mutex is poisoned")
                .clear()?;
        }

        Ok(())
    }
}

fn format_task_completion_message(
    task_path: &TaskPath,
    task: &Task,
    exit_code: i32,
    duration: Duration,
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
            format!(" in {}:", format_duration(duration)).dimmed(),
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
            format!(" in {}:", format_duration(duration)).dimmed(),
            match &task.run {
                Run::String(value) => value.clone(),
                Run::Args(values) => values.join(" "),
            }
            .dimmed()
        )
    }
}

fn format_duration(duration: Duration) -> impl Display {
    let secs = duration.as_secs_f64();

    if secs < 60.0 {
        return format!("{:.2}s", secs);
    }

    let mins = secs / 60.0;
    let secs = secs % 60.0;

    if mins < 60.0 {
        return format!("{}m{:.2}s", mins.floor(), secs);
    }

    let hours = mins / 60.0;
    let mins = mins % 60.0;

    format!("{}h{}m{:.2}s", hours.floor(), mins.floor(), secs)
}
