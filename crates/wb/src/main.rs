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

#![warn(
    clippy::all,
    clippy::pedantic,
    // clippy::nursery,
    // clippy::cargo
)]

mod builtin;
mod cli;
mod config;
mod console;
mod error;
mod exec;

use clap::CommandFactory;
use cli::{TaskArgs, WorkbenchArgs};
use config::{resolve_path, Config};
use console::{Context, Level, Log, Logger};
use exec::TaskPath;
use std::{env, path::PathBuf, process::exit};

fn main() {
    let (workbench_args, task_args) = cli::parse_args();

    if workbench_args.disable_color {
        colored::control::set_override(false);
    }

    if workbench_args.disable_unicode {
        todo!();
    }

    let logger = create_logger(&workbench_args);

    log_args(&logger, &workbench_args, &task_args);

    let config = load_config(&logger, &workbench_args);

    let target_task_path = resolve_target_task_path(&logger, &task_args);

    #[allow(clippy::bool_to_int_with_if)]
    if let Some(succeeded) = builtin::try_exec_builtin_property(&logger, &config, &target_task_path)
    {
        exit(if succeeded { 0 } else { 1 });
    }

    #[allow(clippy::bool_to_int_with_if)]
    if let Some(succeeded) = builtin::try_exec_builtin_task(&config, &target_task_path) {
        exit(if succeeded { 0 } else { 1 });
    }

    if target_task_path.property.is_some() {
        logger.log_message(
            Level::Error,
            format!(
                "unknown property {:?} for task {target_task_path}",
                target_task_path.property.as_ref().unwrap()
            ),
        );
        exit(1);
    }

    let console_context =
        create_console_context(&workbench_args, logger, &config, &target_task_path);

    match exec::exec(config, &console_context, target_task_path) {
        Ok(succeeded) => {
            if !succeeded {
                exit(1);
            }
        }
        Err(e) => {
            console_context.log_message(Level::Error, e.to_string());
            exit(1);
        }
    }
}

const fn create_logger(workbench_args: &WorkbenchArgs) -> Logger {
    Logger::new(if workbench_args.verbose {
        Level::Status
    } else {
        Level::Warning
    })
}

fn log_args(logger: &impl Log, workbench_args: &WorkbenchArgs, task_args: &TaskArgs) {
    logger.log_message(
        Level::Status,
        format!("workbench arguments: {workbench_args:?}"),
    );

    logger.log_message(Level::Status, format!("task arguments: {task_args:?}"));
}

fn resolve_config_path(logger: &impl Log, workbench_args: &WorkbenchArgs) -> PathBuf {
    if let Some(config_path) = &workbench_args.config {
        return PathBuf::from(config_path);
    }

    let current_dir = match env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(err) => {
            logger.log_message(
                Level::Error,
                format!("unable to determine current directory: {err}"),
            );

            exit(1);
        }
    };

    if let Some(config_path) = resolve_path(&current_dir, None) {
        config_path
    } else {
        logger.log_message(
            Level::Error,
            "unable to find a configuration file - are you sure you're in a Workbench workspace?",
        );

        exit(1);
    }
}

fn load_config(logger: &impl Log, workbench_args: &WorkbenchArgs) -> Config {
    let config_path = resolve_config_path(logger, workbench_args);

    let config = match config::load(&config_path) {
        Ok(config) => config,
        Err(err) => {
            logger.log_message(Level::Error, err);
            exit(1);
        }
    };

    logger.log_message(Level::Status, format!("loaded configuration: {config:?}"));

    config
}

fn resolve_target_task_path(logger: &impl Log, task_args: &TaskArgs) -> TaskPath {
    match &task_args.target_task_path {
        Some(path_string) => match TaskPath::parse(path_string.as_str()) {
            Ok(task_path) => task_path,
            Err(err) => {
                logger.log_message(Level::Error, err);
                exit(1);
            }
        },
        None => {
            logger.log_message(Level::Error, "no target specified");
            println!();
            WorkbenchArgs::command().print_help().unwrap();
            exit(1);
        }
    }
}

fn create_console_context(
    workbench_args: &WorkbenchArgs,
    logger: Logger,
    config: &Config,
    target_task_path: &TaskPath,
) -> Context {
    if workbench_args.disable_progress {
        return Context::new(logger, None);
    }

    let progress_bar_ticks = 1 + match exec::count_dependencies_of_path(config, target_task_path) {
        Ok(count) => count.unwrap_or(0),
        Err(e) => {
            logger.log_message(Level::Error, e.to_string());
            exit(1);
        }
    };

    Context::new(logger, Some(progress_bar_ticks))
}
