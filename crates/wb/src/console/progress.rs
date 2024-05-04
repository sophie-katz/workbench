use std::io;

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{
    config::{Config, Run, Task},
    exec::{count_dependencies_of_path, TaskPath, TaskPathParsingError},
};

pub struct ProgressContext {
    multi_progress: MultiProgress,
    progress_bar_tasks: ProgressBar,
}

impl ProgressContext {
    pub fn new(config: &Config, target_task_path: TaskPath) -> Result<Self, TaskPathParsingError> {
        let multi_progress = MultiProgress::new();

        let progress_bar_tasks = multi_progress.add(ProgressBar::new(
            1 + count_dependencies_of_path(config, &target_task_path)? as u64,
        ));

        progress_bar_tasks.set_style(
            ProgressStyle::with_template("【{pos}/{len}】{bar:30.green/dim.white} {msg}")
                .expect("error with template"),
        );

        Ok(Self {
            multi_progress,
            progress_bar_tasks,
        })
    }

    pub fn begin_task(&mut self, task_path: &TaskPath, task: &Task) {
        self.progress_bar_tasks.set_message(format!(
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

    pub fn complete_task(&mut self) {
        self.progress_bar_tasks.inc(1);
    }

    pub fn clear(&self) -> Result<(), io::Error> {
        self.multi_progress.clear()
    }
}
